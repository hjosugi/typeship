use super::Field;
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
            TsType::StringLiteralUnion(values) => render_union(
                values
                    .iter()
                    .map(|value| string_literal(value))
                    .collect::<Vec<_>>(),
            ),
            TsType::Union(members) => {
                render_union(members.iter().map(TsType::render).collect::<Vec<_>>())
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

fn render_union(members: Vec<String>) -> String {
    if members.is_empty() {
        "never".into()
    } else {
        members.join(" | ")
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
}
