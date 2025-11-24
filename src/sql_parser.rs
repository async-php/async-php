use nom::{
    branch::alt,
    bytes::complete::{tag, take_till, take_until, take_while, take_while1},
    character::complete::{alphanumeric1, char, multispace0, one_of},
    combinator::{opt, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;

#[derive(Debug, Clone, PartialEq)]
pub enum PlaceholderKind {
    Positional(usize), // ? placeholders, numbered 1, 2, 3...
    Named(String),     // :name placeholders
}

#[derive(Debug, Clone)]
pub struct Placeholder {
    pub kind: PlaceholderKind,
    pub position: usize, // Position in the original SQL
}

#[derive(Debug)]
pub struct ParsedSql {
    pub rewritten: String,
    pub placeholders: Vec<Placeholder>,
}

/// Parse a single-quoted string literal ('...')
/// Handles escaped quotes ('')
fn single_quoted_string(input: &str) -> IResult<&str, &str> {
    let (input, _) = char('\'')(input)?;
    let mut result = String::new();
    let mut chars = input.chars().enumerate();
    let mut end_pos = 0;

    while let Some((pos, ch)) = chars.next() {
        if ch == '\'' {
            // Check if next is also a quote (escaped)
            if let Some((_, next_ch)) = chars.clone().next() {
                if next_ch == '\'' {
                    chars.next(); // consume the second quote
                    result.push('\'');
                    result.push('\'');
                    end_pos = pos + 2;
                    continue;
                }
            }
            // End of string
            return Ok((&input[end_pos + 1..], &input[..end_pos + 1]));
        }
        result.push(ch);
        end_pos = pos + 1;
    }

    // Unterminated string
    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
    )))
}

/// Parse a double-quoted string literal ("...")
/// Handles escaped quotes ("")
fn double_quoted_string(input: &str) -> IResult<&str, &str> {
    let (input, _) = char('"')(input)?;
    let mut result = String::new();
    let mut chars = input.chars().enumerate();
    let mut end_pos = 0;

    while let Some((pos, ch)) = chars.next() {
        if ch == '"' {
            // Check if next is also a quote (escaped)
            if let Some((_, next_ch)) = chars.clone().next() {
                if next_ch == '"' {
                    chars.next(); // consume the second quote
                    result.push('"');
                    result.push('"');
                    end_pos = pos + 2;
                    continue;
                }
            }
            // End of string
            return Ok((&input[end_pos + 1..], &input[..end_pos + 1]));
        }
        result.push(ch);
        end_pos = pos + 1;
    }

    // Unterminated string
    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
    )))
}

/// Parse PostgreSQL E-string (E'...' or e'...')
/// Handles standard backslash escapes
/// Returns everything after E/e (i.e., the quoted string '...')
fn e_string(input: &str) -> IResult<&str, &str> {
    // Check for E' or e' prefix
    if !(input.starts_with("E'") || input.starts_with("e'")) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Skip the E or e character, keep the opening quote
    let content_with_quotes = &input[1..];
    let content_after_quote = &content_with_quotes[1..];

    let mut chars = content_after_quote.chars().enumerate();
    let mut end_pos = 0;

    while let Some((pos, ch)) = chars.next() {
        if ch == '\\' {
            // Skip next character (escape sequence)
            chars.next();
            end_pos = pos + 2;
            continue;
        }
        if ch == '\'' {
            // Check for escaped quote ('')
            if let Some((_, next_ch)) = chars.clone().next() {
                if next_ch == '\'' {
                    chars.next();
                    end_pos = pos + 2;
                    continue;
                }
            }
            // End of string - return '...' including both quotes
            let total_len = end_pos + 2; // +1 for opening quote, +1 for closing quote
            return Ok((&content_after_quote[end_pos + 1..], &content_with_quotes[..total_len]));
        }
        end_pos = pos + 1;
    }

    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
    )))
}

/// Parse PostgreSQL dollar-quoted string ($tag$...$tag$)
fn dollar_quoted_string(input: &str) -> IResult<&str, &str> {
    let start = input;
    let (input, _) = char('$')(input)?;

    // Parse the tag (can be empty or contain alphanumeric/_)
    let (input, tag_str) = take_while(|c: char| c.is_alphanumeric() || c == '_')(input)?;
    let (input, _) = char('$')(input)?;

    let opening = format!("${}$", tag_str);
    let closing = opening.clone();
    let opening_len = opening.len();

    // Find the closing tag
    let mut pos = 0;
    let bytes = input.as_bytes();
    while pos < bytes.len() {
        if input[pos..].starts_with(&closing) {
            // Return the entire dollar-quoted string including opening and closing tags
            let total_len = opening_len + pos + closing.len();
            let content = &start[..total_len];
            let remaining = &start[total_len..];
            return Ok((remaining, content));
        }
        pos += 1;
    }

    // No closing tag found
    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
    )))
}

/// Parse line comment (-- ...)
fn line_comment(input: &str) -> IResult<&str, &str> {
    let start_input = input;
    let (input, _) = tag("--")(input)?;
    let (input, content) = take_till(|c| c == '\n')(input)?;
    let (input, newline) = opt(char('\n'))(input)?;

    let consumed = 2 + content.len() + if newline.is_some() { 1 } else { 0 };
    Ok((input, &start_input[..consumed]))
}

/// Parse block comment (/* ... */)
fn block_comment(input: &str) -> IResult<&str, &str> {
    let start = input;
    let (input, _) = tag("/*")(input)?;

    // Find the closing */
    let mut depth = 1;
    let mut pos = 0;
    let bytes = input.as_bytes();

    while pos < bytes.len() - 1 && depth > 0 {
        if &input[pos..pos + 2] == "/*" {
            depth += 1;
            pos += 2;
        } else if &input[pos..pos + 2] == "*/" {
            depth -= 1;
            pos += 2;
        } else {
            pos += 1;
        }
    }

    if depth == 0 {
        Ok((&input[pos..], &start[..2 + pos]))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )))
    }
}

/// Parse a positional placeholder (?)
fn positional_placeholder(input: &str) -> IResult<&str, ()> {
    let (input, _) = char('?')(input)?;
    // Check for ?? (escaped question mark in some contexts)
    if input.starts_with('?') {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )))
    } else {
        Ok((input, ()))
    }
}

/// Parse a named placeholder (:name)
/// Must not confuse with PostgreSQL :: cast operator
fn named_placeholder(input: &str) -> IResult<&str, &str> {
    let start = input;
    let (input, _) = char(':')(input)?;

    // Check that it's not ::
    if input.starts_with(':') {
        return Err(nom::Err::Error(nom::error::Error::new(
            start,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Must start with letter or underscore
    let first_char = input.chars().next();
    if !matches!(first_char, Some(c) if c.is_alphabetic() || c == '_') {
        return Err(nom::Err::Error(nom::error::Error::new(
            start,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Continue with alphanumeric or underscore
    let (input, name) = take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;
    Ok((input, name))
}

/// Main parser that rewrites SQL and extracts placeholders
pub fn parse_and_rewrite_sql(sql: &str, driver: &str) -> Result<ParsedSql, String> {
    let mut output = String::with_capacity(sql.len());
    let mut placeholders = Vec::new();
    let mut position = 0;
    let mut param_count = 0;
    let mut input = sql;

    while !input.is_empty() {
        // Try to parse various SQL constructs

        // 1. Try dollar-quoted string (PostgreSQL)
        if let Ok((remaining, matched)) = dollar_quoted_string(input) {
            output.push_str(matched);
            position += matched.len();
            input = remaining;
            continue;
        }

        // 2. Try E-string
        if input.starts_with('E') || input.starts_with('e') {
            if let Ok((remaining, matched)) = e_string(input) {
                // matched already includes everything after E/e'
                output.push(input.chars().next().unwrap()); // E or e
                output.push_str(matched);
                position += matched.len() + 1;
                input = remaining;
                continue;
            }
        }

        // 3. Try single-quoted string
        if let Ok((remaining, matched)) = single_quoted_string(input) {
            output.push('\'');
            output.push_str(matched);
            position += matched.len() + 1;
            input = remaining;
            continue;
        }

        // 4. Try double-quoted string
        if let Ok((remaining, matched)) = double_quoted_string(input) {
            output.push('"');
            output.push_str(matched);
            position += matched.len() + 1;
            input = remaining;
            continue;
        }

        // 5. Try line comment
        if let Ok((remaining, matched)) = line_comment(input) {
            output.push_str(matched);
            position += matched.len();
            input = remaining;
            continue;
        }

        // 6. Try block comment
        if let Ok((remaining, matched)) = block_comment(input) {
            output.push_str(matched);
            position += matched.len();
            input = remaining;
            continue;
        }

        // 7. Try positional placeholder
        if let Ok((remaining, _)) = positional_placeholder(input) {
            param_count += 1;
            placeholders.push(Placeholder {
                kind: PlaceholderKind::Positional(param_count),
                position,
            });

            // Rewrite to driver-specific format
            if driver == "pgsql" {
                let rewritten = format!("${}", param_count);
                position += rewritten.len();
                output.push_str(&rewritten);
            } else {
                output.push('?');
                position += 1;
            }

            input = remaining;
            continue;
        }

        // 8. Try named placeholder
        // But first check if it's :: (PostgreSQL cast operator)
        if input.starts_with("::") {
            // It's a cast operator, just copy both characters
            output.push_str("::");
            position += 2;
            input = &input[2..];
            continue;
        }

        if let Ok((remaining, name)) = named_placeholder(input) {
            param_count += 1;
            placeholders.push(Placeholder {
                kind: PlaceholderKind::Named(name.to_string()),
                position,
            });

            let original_len = name.len() + 1; // +1 for the :

            // Rewrite to driver-specific format
            if driver == "pgsql" {
                let rewritten = format!("${}", param_count);
                position += rewritten.len();
                output.push_str(&rewritten);
            } else {
                output.push('?');
                position += 1;
            }

            input = remaining;
            continue;
        }

        // 9. Regular character - just copy
        if let Some(ch) = input.chars().next() {
            output.push(ch);
            position += ch.len_utf8();
            input = &input[ch.len_utf8()..];
        }
    }

    Ok(ParsedSql {
        rewritten: output,
        placeholders,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_positional() {
        let result = parse_and_rewrite_sql("SELECT * FROM users WHERE id = ?", "pgsql").unwrap();
        assert_eq!(result.rewritten, "SELECT * FROM users WHERE id = $1");
        assert_eq!(result.placeholders.len(), 1);
        assert!(matches!(
            result.placeholders[0].kind,
            PlaceholderKind::Positional(1)
        ));
    }

    #[test]
    fn test_named_placeholder() {
        let result = parse_and_rewrite_sql(
            "SELECT * FROM users WHERE name = :username AND id = :user_id",
            "pgsql",
        )
        .unwrap();
        assert_eq!(
            result.rewritten,
            "SELECT * FROM users WHERE name = $1 AND id = $2"
        );
        assert_eq!(result.placeholders.len(), 2);
    }

    #[test]
    fn test_postgres_cast_operator() {
        let result = parse_and_rewrite_sql("SELECT id::integer FROM users WHERE age = ?", "pgsql")
            .unwrap();
        assert_eq!(result.rewritten, "SELECT id::integer FROM users WHERE age = $1");
        assert_eq!(result.placeholders.len(), 1);
    }

    #[test]
    fn test_string_literals() {
        let result =
            parse_and_rewrite_sql("SELECT 'test''s' WHERE name = ? AND x = ?", "pgsql").unwrap();
        assert_eq!(
            result.rewritten,
            "SELECT 'test''s' WHERE name = $1 AND x = $2"
        );
        assert_eq!(result.placeholders.len(), 2);
    }

    #[test]
    fn test_dollar_quoted() {
        let result = parse_and_rewrite_sql(
            "SELECT $$dollar ? quoted$$ WHERE id = ?",
            "pgsql",
        )
        .unwrap();
        assert_eq!(
            result.rewritten,
            "SELECT $$dollar ? quoted$$ WHERE id = $1"
        );
        assert_eq!(result.placeholders.len(), 1);
    }

    #[test]
    fn test_comments() {
        let result = parse_and_rewrite_sql(
            "SELECT * FROM users -- with ? placeholder\nWHERE id = ?",
            "pgsql",
        )
        .unwrap();
        assert_eq!(
            result.rewritten,
            "SELECT * FROM users -- with ? placeholder\nWHERE id = $1"
        );
        assert_eq!(result.placeholders.len(), 1);
    }
}
