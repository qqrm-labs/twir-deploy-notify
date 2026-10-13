//! Telegram pin guard: last-line-of-defence against duplicate deliveries.
//!
//! The delivery pipeline normally relies on a `last_sent` marker inherited
//! from the previous pipeline run. That chain is fragile: GitHub's run
//! listing (both `gh run list` and the REST runs endpoint with a status
//! filter) intermittently serves stale data, so a job may inherit an
//! outdated marker and treat an already delivered issue as new. When that
//! happens the channel itself is the only reliable witness: every
//! production delivery pins the first post of the issue, so the message
//! currently pinned in the channel identifies the last delivered issue.

use super::generator_shared::normalize_chat_id;
use log::{error, info, warn};
use reqwest::blocking::Client;
use std::error::Error;

/// Outcome of comparing the issue about to be delivered with the issue
/// pinned in the target channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinGuardVerdict {
    /// Delivery may proceed.
    Proceed,
    /// Delivery must not happen: the channel already carries issue `pinned`
    /// whose number is greater than or equal to `current`.
    BlockDuplicate { pinned: u64, current: u64 },
    /// Delivery must not happen: the channel pins issue `pinned` but the
    /// input carries no readable issue number, so the delivery cannot be
    /// proven to be anything other than a duplicate.
    BlockUnproven { pinned: u64 },
}

/// Decide whether delivering issue `current` would duplicate a delivery the
/// channel has already seen.
///
/// The production chat must never receive a duplicate, so the decision is
/// one-sided: a provable duplicate blocks, and an unverifiable input blocks
/// whenever the channel state is known. Only a known-new issue (or a channel
/// whose pin carries no issue number at all) proceeds.
pub fn evaluate_pin_guard(current: Option<u64>, pinned: Option<u64>) -> PinGuardVerdict {
    match (current, pinned) {
        (Some(current), Some(pinned)) if current <= pinned => {
            PinGuardVerdict::BlockDuplicate { pinned, current }
        }
        (None, Some(pinned)) => PinGuardVerdict::BlockUnproven { pinned },
        _ => PinGuardVerdict::Proceed,
    }
}

/// Extract a TWIR issue number from the text of a pinned message.
///
/// The first post of every delivery starts with the escaped header
/// `\#<number> — <date>`, and the link section appended to the posts points
/// at `.../this-week-in-rust-<number>/`. Either marker is sufficient.
pub fn parse_pinned_issue_number(text: &str) -> Option<u64> {
    let unescaped = text.replace('\\', "");
    if let Some(index) = unescaped.find("this-week-in-rust-") {
        let rest = &unescaped[index + "this-week-in-rust-".len()..];
        if let Some(number) = leading_number(rest) {
            return Some(number);
        }
    }
    unescaped
        .trim_start()
        .strip_prefix('#')
        .and_then(leading_number)
}

fn leading_number(text: &str) -> Option<u64> {
    let digits: String = text.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse().ok()
}

/// Fetch the text of the message currently pinned in `chat_id`.
///
/// Returns `Ok(None)` when the chat has no pinned message. Returns an error
/// when Telegram cannot be reached or answers with an error payload, so the
/// caller can fail closed instead of delivering unverified.
pub fn fetch_pinned_message_text(
    client: &Client,
    base_url: &str,
    token: &str,
    chat_id: &str,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    let url = format!("{}/bot{}/getChat", base_url.trim_end_matches('/'), token);
    let normalized_chat_id = normalize_chat_id(chat_id);
    let form = [("chat_id", normalized_chat_id.as_ref())];
    let response = client
        .post(&url)
        .form(&form)
        .send()
        .map_err(|error| error.without_url())?;
    let body = response.text()?;
    let raw: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse Telegram getChat response: {e}: {body}"))?;
    if !raw
        .get("ok")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        return Err(format!("Telegram getChat rejected the request: {body}").into());
    }
    Ok(raw
        .pointer("/result/pinned_message/text")
        .and_then(|value| value.as_str())
        .map(str::to_string))
}

/// Verify that delivering issue `current_number` to `chat_id` will not
/// duplicate an issue the channel already received.
///
/// Fails (blocking the delivery) when the channel's pinned message carries
/// an issue number greater than or equal to `current_number`, and when the
/// channel pins a known issue but the input's number cannot be read (an
/// unprovable delivery is treated as a duplicate). This is the fallback
/// signal: reaching this failure means the `last_sent` marker inherited
/// from the previous pipeline run did not stop the duplicate.
pub fn enforce_pin_guard(
    base_url: &str,
    token: &str,
    chat_id: &str,
    current_number: Option<u64>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = Client::new();
    let pinned_text = fetch_pinned_message_text(&client, base_url, token, chat_id)?;
    let pinned_number = pinned_text.as_deref().and_then(parse_pinned_issue_number);

    match (&pinned_text, pinned_number) {
        (Some(_), None) => {
            warn!("Pin guard: pinned message in {chat_id} carries no TWIR issue number; proceeding")
        }
        _ => info!(
            "Pin guard: {chat_id} last pinned issue number: {pinned_number:?}; current issue number: {current_number:?}"
        ),
    }

    match evaluate_pin_guard(current_number, pinned_number) {
        PinGuardVerdict::Proceed => Ok(()),
        PinGuardVerdict::BlockDuplicate { pinned, current } => {
            error!(
                "PIN GUARD ENGAGED: {chat_id} already has issue #{pinned} pinned; refusing to deliver issue #{current}"
            );
            Err(format!(
                "PIN GUARD ENGAGED: refusing to deliver issue #{current} to {chat_id} because issue #{pinned} is already pinned there. The last_sent marker inherited from the previous pipeline run failed to prevent this duplicate (unreliable previous-run lookup); Telegram is the source of truth. If issue #{current} is genuinely new, unpin or repin in the channel and re-run.",
            )
            .into())
        }
        PinGuardVerdict::BlockUnproven { pinned } => {
            error!(
                "PIN GUARD ENGAGED: {chat_id} pins issue #{pinned} but the input carries no readable issue number; refusing to deliver unverified content"
            );
            Err(format!(
                "PIN GUARD ENGAGED: refusing to deliver to {chat_id}: the input has no readable TWIR issue number while issue #{pinned} is already pinned there. A delivery that cannot be proven new is treated as a duplicate; fix the input (Number: header) or unpin in the channel and re-run.",
            )
            .into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_numbers_block_delivery() {
        assert_eq!(
            evaluate_pin_guard(Some(598), Some(598)),
            PinGuardVerdict::BlockDuplicate {
                pinned: 598,
                current: 598
            }
        );
    }

    #[test]
    fn older_issue_than_pinned_blocks_delivery() {
        assert_eq!(
            evaluate_pin_guard(Some(597), Some(598)),
            PinGuardVerdict::BlockDuplicate {
                pinned: 598,
                current: 597
            }
        );
    }

    #[test]
    fn newer_issue_than_pinned_proceeds() {
        assert_eq!(
            evaluate_pin_guard(Some(599), Some(598)),
            PinGuardVerdict::Proceed
        );
    }

    #[test]
    fn unreadable_input_number_blocks_when_channel_state_is_known() {
        assert_eq!(
            evaluate_pin_guard(None, Some(598)),
            PinGuardVerdict::BlockUnproven { pinned: 598 }
        );
    }

    #[test]
    fn unknown_channel_state_never_blocks() {
        assert_eq!(
            evaluate_pin_guard(Some(598), None),
            PinGuardVerdict::Proceed
        );
        assert_eq!(evaluate_pin_guard(None, None), PinGuardVerdict::Proceed);
    }

    #[test]
    fn parses_number_from_delivered_header() {
        let text = "\\#598 — 2026\\-09\\-23\n\n📰 **NEWS** 📰\n• [Item](https://example.com)";
        assert_eq!(parse_pinned_issue_number(text), Some(598));
    }

    #[test]
    fn parses_number_from_issue_link() {
        let text = "some post without header\n🌐 [View web version](https://this-week-in-rust.org/blog/2026/09/23/this-week-in-rust-598/) 🌐";
        assert_eq!(parse_pinned_issue_number(text), Some(598));
    }

    #[test]
    fn parses_number_from_plain_header() {
        assert_eq!(parse_pinned_issue_number("#598 — 2026-09-23"), Some(598));
    }

    #[test]
    fn non_twir_pin_has_no_number() {
        assert_eq!(
            parse_pinned_issue_number("Правила чата: будьте добры"),
            None
        );
        assert_eq!(parse_pinned_issue_number(""), None);
    }

    #[test]
    fn hash_inside_body_is_ignored() {
        let text = "announcement\nsee issue #100 below\nlink";
        assert_eq!(parse_pinned_issue_number(text), None);
    }

    #[test]
    fn link_without_digits_falls_back_to_header() {
        let text = "\\#597 — 2026\\-09\\-16\n\n[web](https://this-week-in-rust.org/blog/this-week-in-rust/)";
        assert_eq!(parse_pinned_issue_number(text), Some(597));
    }
}
