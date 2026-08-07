//! Reasons the assembled module would not compile.
//!
//! [`crate::check`] guarantees the committed file matches what the generator
//! produces. That is worth nothing if the generator happily produces TypeScript
//! that does not compile, and there are two easy ways to get there:
//!
//! - **duplicate identifiers.** The naming isomorphism is lossy by design —
//!   `col_1` and `col1` both lower to `col1` — so two distinct Rust fields can
//!   land on one wire key. `tsc` answers `TS2300: Duplicate identifier`.
//! - **names that cannot bind.** A type name, function name, or parameter name
//!   has to be a bare, non-reserved identifier. `kebab-case` and `function` are
//!   neither, and the output is a syntax error.
//!
//! Member *keys* are the deliberate exception: a wire key is whatever serde puts
//! in the JSON and TypeScript can quote it, so they are never reported here.
//!
//! The scan is a pure function of the [`Bridge`] value, so it costs no IO and
//! runs before a single byte is written.
//!
//! [`Bridge`]: crate::bridge::Bridge

use crate::command::Command;
use crate::diagnostics::Hazard;
use crate::ir::{Decl, DeclBody, Field, TsType};
use crate::ts::is_bindable_identifier;

/// Why the module would not compile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefectKind {
    /// Two declarations export the same type name.
    DuplicateDeclaration,
    /// Two commands lower to the same TypeScript function name.
    DuplicateCommandFunction,
    /// Two arguments of one command lower to the same wire key.
    DuplicateCommandArgument,
    /// Two fields of one record lower to the same wire key.
    DuplicateRecordField,
    /// A declaration name cannot appear after `export type` / `export interface`.
    UnusableDeclarationName,
    /// A command name does not lower to a usable function name.
    UnusableCommandName,
    /// An argument name cannot appear as a parameter binding.
    UnusableArgumentName,
}

impl DefectKind {
    /// A short, stable explanation of why this breaks the output.
    pub fn note(self) -> &'static str {
        match self {
            DefectKind::DuplicateDeclaration => {
                "two declarations export the same type name; TypeScript rejects the \
                 module with a duplicate-identifier error"
            }
            DefectKind::DuplicateCommandFunction => {
                "two commands lower to the same function name; the second `export \
                 function` redeclares the first"
            }
            // Arguments and fields are the naming iso losing information, which is
            // exactly the hazard the diagnostics catalogue already names.
            DefectKind::DuplicateCommandArgument | DefectKind::DuplicateRecordField => {
                Hazard::RenameCollision.note()
            }
            DefectKind::UnusableDeclarationName => {
                "declaration name is empty, not an identifier, or a reserved word; a \
                 `export type` / `export interface` name cannot be quoted"
            }
            DefectKind::UnusableCommandName => {
                "command name does not lower to a usable function name; pick a Rust \
                 name whose camelCase image is a non-reserved identifier"
            }
            DefectKind::UnusableArgumentName => {
                "argument name is empty, not an identifier, or a reserved word; it \
                 appears both as a parameter binding and as a payload key, so it \
                 cannot be quoted"
            }
        }
    }
}

/// One reason the rendered module would fail to compile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Defect {
    /// What went wrong.
    pub kind: DefectKind,
    /// The offending identifier as it would appear in the output.
    pub name: String,
    /// Where it occurs — `"module"`, a command's Rust name, or a field path such
    /// as `"ProjectReport.generatedBy"`.
    pub location: String,
    /// For the duplicate kinds, how many times the identifier would be emitted
    /// (always `>= 2`). `None` for the name-validity kinds.
    pub occurrences: Option<usize>,
}

impl Defect {
    /// A one-line, CI-log-friendly rendering.
    pub fn render(&self) -> String {
        let count = match self.occurrences {
            Some(n) => format!(" is emitted {n} times —"),
            None => String::from(" —"),
        };
        format!(
            "{:?} at {}: `{}`{} {}",
            self.kind,
            self.location,
            self.name,
            count,
            self.kind.note()
        )
    }

    fn duplicate(kind: DefectKind, name: String, location: &str, occurrences: usize) -> Self {
        Defect {
            kind,
            name,
            location: location.to_string(),
            occurrences: Some(occurrences),
        }
    }

    fn unusable(kind: DefectKind, name: &str, location: &str) -> Self {
        Defect {
            kind,
            name: name.to_string(),
            location: location.to_string(),
            occurrences: None,
        }
    }
}

/// Names that appear more than once, in first-appearance order.
///
/// Deliberately a linear scan rather than a hash map: the counts stay in source
/// order, which keeps the reported defects deterministic — the same property the
/// renderer itself promises.
fn duplicates(names: &[String]) -> Vec<(String, usize)> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for name in names {
        match counts.iter_mut().find(|(seen, _)| seen == name) {
            Some((_, count)) => *count += 1,
            None => counts.push((name.clone(), 1)),
        }
    }
    counts.retain(|(_, count)| *count > 1);
    counts
}

fn push_field_duplicates(fields: &[Field], location: &str, out: &mut Vec<Defect>) {
    let names = fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>();
    for (name, occurrences) in duplicates(&names) {
        out.push(Defect::duplicate(
            DefectKind::DuplicateRecordField,
            name,
            location,
            occurrences,
        ));
    }
}

/// Walk a type expression, reporting duplicate keys in every inline object it
/// contains. [`TsType::Named`] is a reference, not a definition, so it is a leaf.
fn scan_type(ty: &TsType, location: &str, out: &mut Vec<Defect>) {
    match ty {
        TsType::Object(fields) => {
            push_field_duplicates(fields, location, out);
            for field in fields {
                scan_type(&field.ty, &format!("{location}.{}", field.name), out);
            }
        }
        TsType::Array(inner) => scan_type(inner, &format!("{location}[]"), out),
        TsType::Nullable(inner) => scan_type(inner, location, out),
        TsType::Record(_, value) => scan_type(value, &format!("{location}[key]"), out),
        TsType::Union(members) => {
            for member in members {
                scan_type(member, location, out);
            }
        }
        TsType::Named(_) | TsType::StringLiteralUnion(_) | TsType::Unknown => {}
    }
}

fn scan_decl(decl: &Decl, out: &mut Vec<Defect>) {
    match &decl.body {
        DeclBody::Interface(fields) => {
            push_field_duplicates(fields, &decl.name, out);
            for field in fields {
                scan_type(&field.ty, &format!("{}.{}", decl.name, field.name), out);
            }
        }
        DeclBody::Alias(ty) => scan_type(ty, &decl.name, out),
        // A backend already rendered this one; typeship does not reinterpret its
        // text, so its interior is out of scope for the scan.
        DeclBody::Raw(_) => {}
    }
}

/// Scan declarations and commands for anything that would not compile.
///
/// Type names and value names live in separate TypeScript namespaces, so a type
/// and a function may share a name without conflict; only same-namespace repeats
/// are reported.
pub(super) fn scan(decls: &[Decl], commands: &[Command]) -> Vec<Defect> {
    let mut out = Vec::new();

    let decl_names = decls.iter().map(|d| d.name.clone()).collect::<Vec<_>>();
    for (name, occurrences) in duplicates(&decl_names) {
        out.push(Defect::duplicate(
            DefectKind::DuplicateDeclaration,
            name,
            "module",
            occurrences,
        ));
    }

    let fn_names = commands.iter().map(Command::ts_name).collect::<Vec<_>>();
    for (name, occurrences) in duplicates(&fn_names) {
        out.push(Defect::duplicate(
            DefectKind::DuplicateCommandFunction,
            name,
            "module",
            occurrences,
        ));
    }

    for decl in decls {
        // A `Raw` body carries the backend's own `export …` text; `name` is only a
        // key for ordering and diagnostics, so it never reaches the output.
        if !matches!(decl.body, DeclBody::Raw(_)) && !is_bindable_identifier(&decl.name) {
            out.push(Defect::unusable(
                DefectKind::UnusableDeclarationName,
                &decl.name,
                "module",
            ));
        }
    }

    for command in commands {
        let ts_name = command.ts_name();
        if !is_bindable_identifier(&ts_name) {
            out.push(Defect::unusable(
                DefectKind::UnusableCommandName,
                &ts_name,
                &command.rust_name,
            ));
        }

        let arg_names = command
            .args
            .iter()
            .map(|a| a.name.clone())
            .collect::<Vec<_>>();
        for (name, occurrences) in duplicates(&arg_names) {
            out.push(Defect::duplicate(
                DefectKind::DuplicateCommandArgument,
                name,
                &command.rust_name,
                occurrences,
            ));
        }
        for arg in &command.args {
            if !is_bindable_identifier(&arg.name) {
                out.push(Defect::unusable(
                    DefectKind::UnusableArgumentName,
                    &arg.name,
                    &command.rust_name,
                ));
            }
        }
    }

    for decl in decls {
        scan_decl(decl, &mut out);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Arg;

    fn names(input: &[&str]) -> Vec<String> {
        input.iter().map(|s| s.to_string()).collect()
    }

    fn kinds(defects: &[Defect]) -> Vec<DefectKind> {
        defects.iter().map(|d| d.kind).collect()
    }

    #[test]
    fn duplicates_are_counted_in_first_appearance_order() {
        let found = duplicates(&names(&["b", "a", "b", "c", "a", "b"]));
        assert_eq!(
            found,
            vec![("b".to_string(), 3), ("a".to_string(), 2)],
            "order must follow first appearance, not sort order"
        );
    }

    #[test]
    fn a_well_formed_bridge_has_no_defects() {
        assert!(duplicates(&names(&["a", "b", "c"])).is_empty());
        assert!(scan(&[], &[]).is_empty());
        assert!(scan(
            &[Decl::interface(
                "Row",
                [Field::rust("row_count", TsType::number())]
            )],
            &[Command::new("run_query", "Row").arg(Arg::rust("connection_id", TsType::string()))],
        )
        .is_empty());
    }

    #[test]
    fn duplicate_declaration_names_are_reported() {
        let found = scan(
            &[
                Decl::alias("Dup", TsType::string()),
                Decl::alias("Other", TsType::string()),
                Decl::alias("Dup", TsType::number()),
            ],
            &[],
        );
        assert_eq!(kinds(&found), vec![DefectKind::DuplicateDeclaration]);
        assert_eq!(found[0].name, "Dup");
        assert_eq!(found[0].occurrences, Some(2));
    }

    #[test]
    fn commands_colliding_through_the_naming_iso_are_reported() {
        // `run_query` and `runQuery` both lower to `runQuery`.
        let found = scan(
            &[],
            &[
                Command::new("run_query", "A"),
                Command::new("runQuery", "B"),
            ],
        );
        assert_eq!(kinds(&found), vec![DefectKind::DuplicateCommandFunction]);
        assert_eq!(found[0].name, "runQuery");
    }

    #[test]
    fn arguments_colliding_through_the_naming_iso_are_reported() {
        // The digit case the naming module pins as a known non-roundtrip:
        // `col_1` and `col1` are distinct in Rust, identical on the wire.
        let found = scan(
            &[],
            &[Command::new("collide", "boolean")
                .arg(Arg::rust("col_1", TsType::string()))
                .arg(Arg::rust("col1", TsType::number()))],
        );
        assert_eq!(kinds(&found), vec![DefectKind::DuplicateCommandArgument]);
        assert_eq!(found[0].name, "col1");
        assert_eq!(found[0].location, "collide");
    }

    #[test]
    fn duplicate_interface_fields_are_reported_with_a_path() {
        let found = scan(
            &[Decl::interface(
                "Rec",
                [
                    Field::rust("row_count", TsType::number()),
                    Field::new("rowCount", TsType::string()),
                ],
            )],
            &[],
        );
        assert_eq!(kinds(&found), vec![DefectKind::DuplicateRecordField]);
        assert_eq!(found[0].location, "Rec");
    }

    #[test]
    fn nested_inline_objects_are_scanned() {
        let found = scan(
            &[Decl::interface(
                "Report",
                [Field::new(
                    "generatedBy",
                    TsType::array(TsType::object([
                        Field::new("id", TsType::string()),
                        Field::new("id", TsType::number()),
                    ])),
                )],
            )],
            &[],
        );
        assert_eq!(kinds(&found), vec![DefectKind::DuplicateRecordField]);
        assert_eq!(found[0].name, "id");
        assert_eq!(found[0].location, "Report.generatedBy[]");
    }

    #[test]
    fn alias_bodies_and_record_values_are_scanned() {
        let found = scan(
            &[Decl::alias(
                "Wrapped",
                TsType::record(
                    TsType::string(),
                    TsType::nullable(TsType::object([
                        Field::new("x", TsType::string()),
                        Field::new("x", TsType::string()),
                    ])),
                ),
            )],
            &[],
        );
        assert_eq!(kinds(&found), vec![DefectKind::DuplicateRecordField]);
        assert_eq!(found[0].location, "Wrapped[key]");
    }

    #[test]
    fn raw_declarations_are_opaque_to_the_scan() {
        // A backend owns the text; typeship does not re-parse it, and `name` never
        // reaches the output — so neither its interior nor its shape is checked.
        let found = scan(
            &[
                Decl::raw("B", "export type B = { x: string, x: number };"),
                Decl::raw("kebab-case", "export type Whatever = string;"),
            ],
            &[],
        );
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn a_type_and_a_function_may_share_a_name() {
        // Separate TypeScript namespaces: this is legal, not a defect.
        let found = scan(
            &[Decl::alias("ping", TsType::string())],
            &[Command::new("ping", "boolean")],
        );
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn unusable_declaration_names_are_reported() {
        let found = scan(
            &[
                Decl::alias("kebab-case", TsType::string()),
                Decl::interface("void", [Field::new("x", TsType::string())]),
            ],
            &[],
        );
        assert_eq!(
            kinds(&found),
            vec![
                DefectKind::UnusableDeclarationName,
                DefectKind::UnusableDeclarationName
            ]
        );
        assert_eq!(found[0].name, "kebab-case");
        assert_eq!(found[1].name, "void", "a reserved word cannot name a type");
    }

    #[test]
    fn unusable_command_and_argument_names_are_reported() {
        // `to_camel_case("_")` is empty, and `function` is reserved.
        let found = scan(
            &[],
            &[
                Command::new("_", "boolean"),
                Command::new("reserved", "boolean")
                    .arg(Arg::rust("function", TsType::string()))
                    .arg(Arg::rust("_", TsType::number())),
            ],
        );
        assert_eq!(
            kinds(&found),
            vec![
                DefectKind::UnusableCommandName,
                DefectKind::UnusableArgumentName,
                DefectKind::UnusableArgumentName,
            ]
        );
        assert_eq!(found[0].name, "");
        assert_eq!(found[1].name, "function");
        assert_eq!(found[2].name, "");
    }

    #[test]
    fn field_names_are_never_reported_because_they_can_be_quoted() {
        // `{ "kebab-case": string }` is exactly what a serde rename produces.
        let found = scan(
            &[Decl::interface(
                "Weird",
                [
                    Field::new("kebab-case", TsType::string()),
                    Field::new("", TsType::boolean()),
                    Field::new("function", TsType::number()),
                ],
            )],
            &[],
        );
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn render_mentions_the_identifier_and_the_location() {
        let duplicate = Defect::duplicate(
            DefectKind::DuplicateRecordField,
            "col1".to_string(),
            "Rec",
            2,
        );
        let text = duplicate.render();
        assert!(text.contains("col1"), "{text}");
        assert!(text.contains("Rec"), "{text}");
        assert!(text.contains("emitted 2 times"), "{text}");
        assert!(text.contains("rename collision"), "{text}");

        // The name-validity kinds carry no count, so the message must not claim one.
        let unusable = Defect::unusable(DefectKind::UnusableArgumentName, "function", "cmd");
        let text = unusable.render();
        assert!(text.contains("function"), "{text}");
        assert!(!text.contains("is emitted"), "{text}");
        assert!(!text.contains("times"), "{text}");
    }
}
