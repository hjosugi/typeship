use super::TsType;
use crate::naming::to_camel_case;
use crate::ts::doc_comment;

/// A named field of a record (struct field, or a member of an inline object).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    /// The name *as it appears on the wire* — already `lowerCamelCase`.
    pub name: String,
    /// The field's type.
    pub ty: TsType,
    /// Emit `name?: T` instead of `name: T`. This is the right encoding for serde
    /// `Option` / `skip_serializing_if` fields that may be absent from the JSON.
    pub optional: bool,
    /// An optional doc comment, rendered as a leading `/** … */`.
    pub docs: Option<String>,
}

impl Field {
    /// A field whose name is already a wire (camelCase) identifier.
    pub fn new(name: impl Into<String>, ty: TsType) -> Self {
        Field {
            name: name.into(),
            ty,
            optional: false,
            docs: None,
        }
    }

    /// A field declared with its *Rust* `snake_case` name; the wire name is
    /// derived through the naming isomorphism ([`crate::naming::to_camel_case`]).
    /// This is the common case when mirroring a `#[serde(rename_all = "camelCase")]`
    /// struct.
    pub fn rust(rust_name: &str, ty: TsType) -> Self {
        Field::new(to_camel_case(rust_name), ty)
    }

    /// Mark the field optional (`name?: T`).
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Attach a doc comment.
    pub fn with_docs(mut self, docs: impl Into<String>) -> Self {
        self.docs = Some(docs.into());
        self
    }

    /// Render as a single inline member: `name: T` (no trailing punctuation).
    pub(super) fn render_inline(&self) -> String {
        let opt = if self.optional { "?" } else { "" };
        format!("{}{}: {};", self.name, opt, self.ty.render())
    }

    /// Render as an indented interface member, with an optional doc comment.
    fn render_member(&self, indent: &str) -> String {
        let opt = if self.optional { "?" } else { "" };
        let line = format!("{indent}{}{}: {};", self.name, opt, self.ty.render());
        match &self.docs {
            Some(docs) => format!("{indent}{}\n{line}", doc_comment(docs)),
            None => line,
        }
    }
}

/// The body of a top-level declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclBody {
    /// `export type Name = <ty>;`
    Alias(TsType),
    /// `export interface Name { … }`
    Interface(Vec<Field>),
    /// A pre-rendered declaration string, emitted verbatim.
    ///
    /// This is the backend seam: a per-type renderer (`ts-rs`, `specta`, …) lowers
    /// a Rust type into a finished `export …` line, and typeship assembles around
    /// it without reinterpreting its formatting. The string is expected to be a
    /// complete, `export`-prefixed declaration (no surrounding blank lines).
    Raw(String),
}

/// A top-level TypeScript declaration: a named type a consumer can import.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decl {
    /// The exported type name. Type names are `PascalCase` and pass through
    /// unchanged — they are *not* run through the naming iso.
    pub name: String,
    /// What the declaration is.
    pub body: DeclBody,
    /// An optional doc comment.
    pub docs: Option<String>,
}

impl Decl {
    /// `export type Name = <ty>;` — used for enums (string-literal unions),
    /// newtypes, and aliases.
    pub fn alias(name: impl Into<String>, ty: TsType) -> Self {
        Decl {
            name: name.into(),
            body: DeclBody::Alias(ty),
            docs: None,
        }
    }

    /// `export interface Name { … }` — used for structs (products).
    pub fn interface<I>(name: impl Into<String>, fields: I) -> Self
    where
        I: IntoIterator<Item = Field>,
    {
        Decl {
            name: name.into(),
            body: DeclBody::Interface(fields.into_iter().collect()),
            docs: None,
        }
    }

    /// A declaration rendered by a backend, wrapped verbatim.
    ///
    /// `name` is the bare type name (used for ordering and diagnostics); `rendered`
    /// is the finished `export …` string the backend produced — see
    /// [`DeclBody::Raw`]. A trailing newline is normalised away so the
    /// [`crate::Bridge`] controls spacing.
    pub fn raw(name: impl Into<String>, rendered: impl Into<String>) -> Self {
        let rendered = rendered.into();
        Decl {
            name: name.into(),
            body: DeclBody::Raw(rendered.trim_end().to_string()),
            docs: None,
        }
    }

    /// Attach a doc comment, rendered above the declaration.
    pub fn with_docs(mut self, docs: impl Into<String>) -> Self {
        self.docs = Some(docs.into());
        self
    }

    /// Render the full `export …` declaration, terminated by a newline.
    pub fn render(&self) -> String {
        let mut out = String::new();
        if let Some(docs) = &self.docs {
            out.push_str(&doc_comment(docs));
            out.push('\n');
        }
        match &self.body {
            DeclBody::Alias(ty) => {
                out.push_str(&format!("export type {} = {};\n", self.name, ty.render()));
            }
            DeclBody::Interface(fields) => {
                out.push_str(&format!("export interface {} {{\n", self.name));
                for field in fields {
                    out.push_str(&field.render_member("  "));
                    out.push('\n');
                }
                out.push_str("}\n");
            }
            DeclBody::Raw(rendered) => {
                out.push_str(rendered);
                out.push('\n');
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_decl_is_emitted_verbatim_with_one_newline() {
        // A backend hands us a finished line (here in ts-rs's single-line style).
        let decl = Decl::raw(
            "DbObject",
            "export type DbObject = { name: string, kind: DbObjectKind, rows?: string, };\n",
        );
        assert_eq!(
            decl.render(),
            "export type DbObject = { name: string, kind: DbObjectKind, rows?: string, };\n"
        );
    }

    #[test]
    fn alias_renders_string_literal_union() {
        let decl = Decl::alias(
            "ConnectionStatus",
            TsType::string_literals(["connected", "idle", "error"]),
        );
        assert_eq!(
            decl.render(),
            "export type ConnectionStatus = \"connected\" | \"idle\" | \"error\";\n"
        );
    }

    #[test]
    fn interface_uses_camelcased_rust_names() {
        let decl = Decl::interface(
            "DbObject",
            [
                Field::rust("name", TsType::string()),
                Field::rust("row_count", TsType::nullable(TsType::number())).optional(),
            ],
        );
        let ts = decl.render();
        assert!(ts.contains("export interface DbObject {"), "{ts}");
        assert!(ts.contains("  name: string;"), "{ts}");
        // snake_case -> camelCase, and optional key.
        assert!(ts.contains("  rowCount?: number | null;"), "{ts}");
    }
}
