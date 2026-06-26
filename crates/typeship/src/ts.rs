//! Small TypeScript rendering helpers shared by the facade.

use std::fmt::Write as _;

/// Render a value as a double-quoted TypeScript string literal.
///
/// Generated values can come from Rust enum variants or command names. Escaping
/// here keeps those values as data, even when they contain quotes, backslashes, or
/// line terminators.
pub(crate) fn string_literal(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            ch if ch.is_control() => {
                let _ = write!(out, "\\u{:04X}", ch as u32);
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_quotes_backslashes_and_line_breaks() {
        assert_eq!(string_literal("a\"b"), "\"a\\\"b\"");
        assert_eq!(string_literal("a\\b"), "\"a\\\\b\"");
        assert_eq!(string_literal("a\nb\rc"), "\"a\\nb\\rc\"");
    }

    #[test]
    fn escapes_js_line_separators() {
        assert_eq!(string_literal("a\u{2028}b"), "\"a\\u2028b\"");
        assert_eq!(string_literal("a\u{2029}b"), "\"a\\u2029b\"");
    }

    #[test]
    fn preserves_printable_unicode() {
        assert_eq!(string_literal("\u{00FC}ber"), "\"\u{00FC}ber\"");
    }
}
