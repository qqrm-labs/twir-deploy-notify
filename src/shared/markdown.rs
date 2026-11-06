/// Escape plain text for Telegram MarkdownV2.
///
/// This follows the text-context rules in the [Telegram Bot API]: reserved
/// characters and literal backslashes are escaped. Link destinations and code
/// blocks retain their existing handling.
///
/// [Telegram Bot API]: https://core.telegram.org/bots/api#markdownv2-style
pub(crate) fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        if r"\_*[]()~`>#+-=|{}.!".contains(ch) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::escape;

    #[test]
    fn escapes_all_reserved_text_characters() {
        assert_eq!(
            escape(r"_ * [ ] ( ) ~ ` > # + - = | { } . !"),
            r"\_ \* \[ \] \( \) \~ \` \> \# \+ \- \= \| \{ \} \. \!"
        );
    }

    #[test]
    fn keeps_backslashes_literal_even_before_reserved_characters() {
        assert_eq!(escape(r"\path\file"), r"\\path\\file");
        assert_eq!(escape(r"\!"), r"\\\!");
        assert_eq!(escape("\\\\"), "\\\\\\\\");
    }

    #[test]
    fn preserves_plain_unicode_whitespace_and_empty_text() {
        let text = "Rust 🦀 Привет e\u{301}\n\t<&/:;,'\"";
        assert_eq!(escape(text), text);
        assert_eq!(escape(""), "");
    }
}
