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

/// Render a single-line JSDoc comment.
pub(crate) fn doc_comment(docs: &str) -> String {
    let mut out = String::with_capacity(docs.len() + 7);
    out.push_str("/** ");

    let mut chars = docs.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\n' | '\r' | '\u{2028}' | '\u{2029}' => out.push(' '),
            '*' if chars.peek() == Some(&'/') => {
                out.push_str("* /");
                let _ = chars.next();
            }
            ch => out.push(ch),
        }
    }

    out.push_str(" */");
    out
}

/// A deterministic TypeScript module assembled from already-rendered blocks.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TsModule {
    blocks: Vec<String>,
}

impl TsModule {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn push(&mut self, block: impl Into<String>) {
        self.blocks.push(block.into());
    }

    pub(crate) fn push_rendered(&mut self, block: impl AsRef<str>) {
        self.push(block.as_ref().trim_end());
    }

    pub(crate) fn finish(self) -> String {
        format!("{}\n", self.blocks.join("\n\n"))
    }
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

    #[test]
    fn module_blocks_are_separated_and_terminated_once() {
        let mut module = TsModule::new();
        module.push("// header");
        module.push_rendered("export type A = string;\n");
        assert_eq!(module.finish(), "// header\n\nexport type A = string;\n");
    }

    #[test]
    fn doc_comment_is_single_line_jsdoc() {
        assert_eq!(doc_comment("hello"), "/** hello */");
    }

    #[test]
    fn doc_comment_neutralizes_comment_end_and_line_breaks() {
        assert_eq!(
            doc_comment("line one\nline two */ still docs"),
            "/** line one line two * / still docs */"
        );
    }
}
