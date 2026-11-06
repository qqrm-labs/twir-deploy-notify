use std::collections::VecDeque;

/// Errors returned by [`validate_telegram_markdown`].
#[derive(Debug)]
pub enum MarkdownError {
    /// A Markdown entity was not closed properly or tags are mismatched.
    UnmatchedTag(String),
    /// The text contains an invalid escape or reserved character.
    InvalidEscape(String),
}

impl std::fmt::Display for MarkdownError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarkdownError::UnmatchedTag(s) | MarkdownError::InvalidEscape(s) => f.write_str(s),
        }
    }
}

impl std::error::Error for MarkdownError {}

/// Validate that the provided text conforms to the Telegram Markdown V2 rules
/// used in this project.
///
/// # Parameters
/// - `text`: Telegram-formatted Markdown to validate.
///
/// # Returns
/// - `Ok(())` if the text is valid.
/// - `Err(MarkdownError)` describing the first encountered problem
///   otherwise.
pub fn validate_telegram_markdown(text: &str) -> Result<(), MarkdownError> {
    let chars: Vec<char> = text.chars().collect();
    let mut stack: VecDeque<&str> = VecDeque::new();
    let mut in_code_block = false;
    if let Some(&first) = chars.first()
        && matches!(first, '-' | '>' | '#' | '+' | '=' | '{' | '}' | '.' | '!')
    {
        return Err(MarkdownError::InvalidEscape(
            "Post starts with reserved character".to_string(),
        ));
    }
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if in_code_block && ch != '`' {
            i += 1;
            continue;
        }
        match ch {
            '*' => {
                let token = if i + 1 < chars.len() && chars[i + 1] == '*' {
                    i += 1;
                    "**"
                } else {
                    "*"
                };
                toggle_token(token, &mut stack)?;
            }
            '_' => {
                let token = if i + 1 < chars.len() && chars[i + 1] == '_' {
                    i += 1;
                    "__"
                } else {
                    "_"
                };
                toggle_token(token, &mut stack)?;
            }
            '~' => toggle_token("~", &mut stack)?,
            '|' => {
                if i + 1 < chars.len() && chars[i + 1] == '|' {
                    i += 1;
                    toggle_token("||", &mut stack)?;
                } else if !in_code_block {
                    let prev = if i == 0 { None } else { Some(chars[i - 1]) };
                    if prev != Some('\\') {
                        return Err(MarkdownError::InvalidEscape(format!(
                            "Unescaped {ch} at {i}"
                        )));
                    }
                }
            }
            '`' => {
                let token = if i + 2 < chars.len() && chars[i + 1] == '`' && chars[i + 2] == '`' {
                    i += 2;
                    in_code_block = !in_code_block;
                    "```"
                } else {
                    "`"
                };
                toggle_token(token, &mut stack)?;
            }
            '[' => {
                let mut j = i + 1;
                while j < chars.len() {
                    if chars[j] == '\\' {
                        j += 1;
                    } else if chars[j] == ']' {
                        break;
                    }
                    j += 1;
                }
                if j >= chars.len() || j + 1 >= chars.len() || chars[j + 1] != '(' {
                    return Err(MarkdownError::UnmatchedTag(format!("Unmatched [ at {i}")));
                }
                j += 2;
                while j < chars.len() {
                    if chars[j] == '\\' {
                        j += 1;
                    } else if chars[j] == ')' {
                        break;
                    }
                    j += 1;
                }
                if j >= chars.len() {
                    return Err(MarkdownError::UnmatchedTag(format!("Unmatched [ at {i}")));
                }
                i = j;
            }
            ']' => {
                if !in_code_block {
                    let prev = if i == 0 { None } else { Some(chars[i - 1]) };
                    if prev != Some('\\') {
                        return Err(MarkdownError::InvalidEscape(format!(
                            "Unescaped {ch} at {i}"
                        )));
                    }
                }
            }
            '\\' => {
                i += 1; // skip escaped char
            }
            '-' | '>' | '#' | '+' | '=' | '{' | '}' | '.' | '!' => {
                if in_code_block {
                    i += 1;
                    continue;
                }
                let prev = if i == 0 { None } else { Some(chars[i - 1]) };
                let next = chars.get(i + 1).copied();
                if prev != Some('\\')
                    && !(prev.map(|c| c.is_ascii_alphanumeric()).unwrap_or(false)
                        && next.map(|c| c.is_ascii_alphanumeric()).unwrap_or(false))
                {
                    return Err(MarkdownError::InvalidEscape(format!(
                        "Unescaped {ch} at {i}"
                    )));
                }
            }
            '(' | ')' if !in_code_block => {
                let prev = if i == 0 { None } else { Some(chars[i - 1]) };
                if prev != Some('\\') {
                    return Err(MarkdownError::InvalidEscape(format!(
                        "Unescaped {ch} at {i}"
                    )));
                }
            }
            _ => {}
        }
        i += 1;
    }
    if let Some(tok) = stack.pop_back() {
        return Err(MarkdownError::UnmatchedTag(format!(
            "Unclosed {tok} entity"
        )));
    }
    Ok(())
}

fn toggle_token<'a>(token: &'a str, stack: &mut VecDeque<&'a str>) -> Result<(), MarkdownError> {
    if let Some(last) = stack.back() {
        if *last == token {
            stack.pop_back();
            return Ok(());
        }
        if stack.contains(&token) {
            return Err(MarkdownError::UnmatchedTag(format!(
                "Mismatched {token} entity"
            )));
        }
    }
    stack.push_back(token);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_validation() {
        validate_telegram_markdown("*bold*").unwrap();
        assert!(validate_telegram_markdown("*bold").is_err());
    }

    #[test]
    fn rejects_unescaped_dash() {
        assert!(validate_telegram_markdown("some - text").is_err());
    }

    #[test]
    fn accepts_escaped_dash() {
        assert!(validate_telegram_markdown("some \\- text").is_ok());
    }

    #[test]
    fn dash_inside_word_is_ok() {
        let msg =
            "Always wanted to contribute to open-source projects but did not know where to start?";
        assert!(validate_telegram_markdown(msg).is_ok());
    }

    #[test]
    fn rejects_dash_at_start() {
        assert!(validate_telegram_markdown("- bad").is_err());
    }

    #[test]
    fn rejects_unescaped_parentheses() {
        assert!(validate_telegram_markdown("text (test)").is_err());
    }

    #[test]
    fn accepts_escaped_parentheses() {
        assert!(validate_telegram_markdown("text \\(test\\)").is_ok());
    }

    #[test]
    fn rejects_unescaped_bracket() {
        assert!(validate_telegram_markdown("text ] ok").is_err());
    }

    #[test]
    fn rejects_single_pipe() {
        assert!(validate_telegram_markdown("a | b").is_err());
    }

    #[test]
    fn accepts_table_like_content_inside_code_block() {
        let text = "```\n| range |\n| [0.4%, 0.5%] |\n| literal * text _ text |\n```";
        assert!(validate_telegram_markdown(text).is_ok());
    }
}
