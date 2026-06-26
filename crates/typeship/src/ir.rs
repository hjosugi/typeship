//! The schema as a first-class value.
//!
//! The functional-programming codegen tradition (`purescript-bridge`,
//! `haskell-to-elm`, aeson's `Generic`) treats the cross-language boundary as a
//! *value you can introspect and transform*, not as `format!` text emitted at the
//! call site. [`TsType`] and [`Decl`] are that value: a small, closed intermediate
//! representation that backends (ts-rs, specta, schemars) lower into, and that a
//! single renderer lowers out to TypeScript.
//!
//! Two algebraic shapes carry most of the weight:
//!
//! - **product types** (Rust structs) become [`TsType::Object`] / an
//!   `export interface` — a record of named fields;
//! - **sum types** (Rust enums) become a *closed* union
//!   ([`TsType::StringLiteralUnion`] for C-like enums,
//!   [`TsType::Union`] for data-carrying ones) so a consumer `switch` can be
//!   proven exhaustive against [`crate::bridge::Bridge::with_assert_never`].
//!
//! Keeping the union closed is the whole point: an *open* union (`string`) throws
//! away the totality guarantee that makes sum types worth sharing.

use crate::naming::to_camel_case;
use crate::ts::string_literal;

/// A TypeScript type expression.
///
/// This is intentionally small. It is not a model of TypeScript's type system; it
/// is the subset that a serde-shaped wire format can actually inhabit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TsType {
    /// A primitive or a reference to a named declaration: `string`, `number`,
    /// `WorkspaceSnapshot`. Use [`TsType::string`], [`TsType::number`], etc. for
    /// the primitives and [`TsType::named`] for references.
    Named(String),
    /// `T[]` — a homogeneous array (Rust `Vec<T>` / `[T; N]`).
    Array(Box<TsType>),
    /// `T | null` — a serde `Option<T>` rendered at value position.
    ///
    /// Note the deliberate split from an *absent* field: `Option` at a struct
    /// field is usually better modelled as an optional key (`name?: T`) via
    /// [`Field::optional`]. `Nullable` is for `Option` that is genuinely present
    /// as `null` on the wire (e.g. inside a `Vec<Option<T>>`).
    Nullable(Box<TsType>),
    /// `Record<K, V>` — a serde map (`HashMap`, `BTreeMap`).
    Record(Box<TsType>, Box<TsType>),
    /// `"a" | "b" | "c"` — a C-like enum, the canonical closed union.
    StringLiteralUnion(Vec<String>),
    /// A general union `A | B | C`, e.g. a serde adjacently/internally tagged enum
    /// already lowered to its per-variant object types.
    Union(Vec<TsType>),
    /// An inline object literal `{ a: A; b: B }` (an anonymous product).
    Object(Vec<Field>),
    /// `unknown` — an opaque JSON value (`serde_json::Value`). Deliberately
    /// `unknown`, never `any`: it forces the consumer to narrow before use.
    Unknown,
}

impl TsType {
    /// A reference to a named type (a primitive name, or another declaration).
    pub fn named(name: impl Into<String>) -> Self {
        TsType::Named(name.into())
    }

    /// The `string` primitive.
    pub fn string() -> Self {
        TsType::Named("string".into())
    }

    /// The `number` primitive.
    pub fn number() -> Self {
        TsType::Named("number".into())
    }

    /// The `boolean` primitive.
    pub fn boolean() -> Self {
        TsType::Named("boolean".into())
    }

    /// The `bigint` primitive — the right target for Rust `u64`/`i64`/`u128`,
    /// whose range exceeds JavaScript's `number` (safe-integer) precision.
    pub fn bigint() -> Self {
        TsType::Named("bigint".into())
    }

    /// The `void` type — a command that resolves with no value (Rust `()`).
    pub fn void() -> Self {
        TsType::Named("void".into())
    }

    /// `T[]`.
    pub fn array(inner: TsType) -> Self {
        TsType::Array(Box::new(inner))
    }

    /// `T | null`.
    pub fn nullable(inner: TsType) -> Self {
        TsType::Nullable(Box::new(inner))
    }

    /// `Record<K, V>`.
    pub fn record(key: TsType, value: TsType) -> Self {
        TsType::Record(Box::new(key), Box::new(value))
    }

    /// A closed string-literal union, e.g. `Decl::alias("Status",
    /// TsType::string_literals(["connected", "idle"]))`.
    pub fn string_literals<I, S>(values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        TsType::StringLiteralUnion(values.into_iter().map(Into::into).collect())
    }

    /// A general union of the given member types.
    pub fn union<I>(members: I) -> Self
    where
        I: IntoIterator<Item = TsType>,
    {
        TsType::Union(members.into_iter().collect())
    }

    /// An inline object literal.
    pub fn object<I>(fields: I) -> Self
    where
        I: IntoIterator<Item = Field>,
    {
        TsType::Object(fields.into_iter().collect())
    }

    /// `unknown`.
    pub fn unknown() -> Self {
        TsType::Unknown
    }

    /// Render this type as a TypeScript type expression.
    pub fn render(&self) -> String {
        match self {
            TsType::Named(name) => name.clone(),
            TsType::Array(inner) => format!("{}[]", inner.render_atom()),
            TsType::Nullable(inner) => format!("{} | null", inner.render_atom()),
            TsType::Record(k, v) => format!("Record<{}, {}>", k.render(), v.render()),
            TsType::StringLiteralUnion(values) => {
                if values.is_empty() {
                    "never".into()
                } else {
                    values
                        .iter()
                        .map(|v| string_literal(v))
                        .collect::<Vec<_>>()
                        .join(" | ")
                }
            }
            TsType::Union(members) => {
                if members.is_empty() {
                    "never".into()
                } else {
                    members
                        .iter()
                        .map(TsType::render)
                        .collect::<Vec<_>>()
                        .join(" | ")
                }
            }
            TsType::Object(fields) => {
                if fields.is_empty() {
                    "{}".into()
                } else {
                    let body = fields
                        .iter()
                        .map(Field::render_inline)
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("{{ {body} }}")
                }
            }
            TsType::Unknown => "unknown".into(),
        }
    }

    /// Render in a position that requires an atomic type — parenthesising unions
    /// so `Vec<Option<T>>` becomes `(T | null)[]`, not the wrong `T | null[]`.
    fn render_atom(&self) -> String {
        match self {
            TsType::Nullable(_) | TsType::Union(_) | TsType::StringLiteralUnion(_) => {
                format!("({})", self.render())
            }
            other => other.render(),
        }
    }
}

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
    fn render_inline(&self) -> String {
        let opt = if self.optional { "?" } else { "" };
        format!("{}{}: {};", self.name, opt, self.ty.render())
    }

    /// Render as an indented interface member, with an optional doc comment.
    fn render_member(&self, indent: &str) -> String {
        let opt = if self.optional { "?" } else { "" };
        let line = format!("{indent}{}{}: {};", self.name, opt, self.ty.render());
        match &self.docs {
            Some(docs) => format!("{indent}/** {docs} */\n{line}"),
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
            out.push_str(&format!("/** {docs} */\n"));
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
    fn primitives_and_containers_render() {
        assert_eq!(TsType::string().render(), "string");
        assert_eq!(TsType::array(TsType::number()).render(), "number[]");
        assert_eq!(TsType::nullable(TsType::string()).render(), "string | null");
        assert_eq!(
            TsType::record(TsType::string(), TsType::number()).render(),
            "Record<string, number>"
        );
        assert_eq!(TsType::unknown().render(), "unknown");
    }

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
    fn array_of_union_is_parenthesised() {
        // The classic precedence bug: `T | null[]` would be wrong.
        let ty = TsType::array(TsType::nullable(TsType::string()));
        assert_eq!(ty.render(), "(string | null)[]");
    }

    #[test]
    fn empty_unions_are_never() {
        assert_eq!(
            TsType::string_literals(Vec::<String>::new()).render(),
            "never"
        );
        assert_eq!(TsType::union(Vec::<TsType>::new()).render(), "never");
    }

    #[test]
    fn string_literal_unions_escape_values() {
        assert_eq!(
            TsType::string_literals(["plain", "quote\"slash\\", "line\nbreak"]).render(),
            "\"plain\" | \"quote\\\"slash\\\\\" | \"line\\nbreak\""
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
