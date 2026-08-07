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

/// ECMAScript reserved words. A generated module is always strict-mode ESM, so
/// the strict-only reservations (`let`, `static`, `yield`, …) bind just as hard
/// as the core ones. Kept sorted for readability; the lookup is a linear scan
/// over a list this short.
const RESERVED_WORDS: &[&str] = &[
    "arguments",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "eval",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

/// Whether `name` has the shape of a bare TypeScript identifier.
///
/// Approximates the ECMAScript `IdentifierName` grammar with [`char::is_alphabetic`]
/// / [`char::is_alphanumeric`], which keeps the core crate dependency-free while
/// still accepting non-ASCII identifiers (`über`) and rejecting the shapes that
/// actually break the output (`kebab-case`, `with space`, `""`, `1st`).
pub(crate) fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '$')
}

/// Whether `name` can *bind* — as a type name, a function name, or a parameter.
///
/// Stricter than [`is_identifier`]: a reserved word has identifier shape but
/// cannot be bound, so `export type void = …` and `function f(new: string)` are
/// both syntax errors. Member keys are the exception — `{ new: string }` is
/// perfectly legal — so they do not go through this check.
pub(crate) fn is_bindable_identifier(name: &str) -> bool {
    is_identifier(name) && !RESERVED_WORDS.contains(&name)
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
    fn identifier_shape_is_recognised() {
        for ok in ["name", "_private", "$dollar", "rowCount", "a1", "über"] {
            assert!(is_identifier(ok), "{ok} should be an identifier");
        }
        for bad in ["", "kebab-case", "with space", "1st", "a.b", "a:b"] {
            assert!(!is_identifier(bad), "{bad} should not be an identifier");
        }
    }

    #[test]
    fn reserved_words_have_identifier_shape_but_cannot_bind() {
        for word in ["function", "new", "void", "class", "let", "yield", "eval"] {
            assert!(is_identifier(word), "{word} has identifier shape");
            assert!(!is_bindable_identifier(word), "{word} must not bind");
        }
        assert!(is_bindable_identifier("rowCount"));
        assert!(!is_bindable_identifier(""));
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
