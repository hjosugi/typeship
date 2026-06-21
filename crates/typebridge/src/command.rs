//! Typed command wrappers — the part `ts-rs` and friends do *not* generate.
//!
//! Per-type renderers stop at data shapes. But a Tauri app (or any RPC client)
//! also needs the *verbs*: a typed `invoke` wrapper per command so the frontend
//! never hand-writes `invoke("workspace_snapshot")` with a stringly-typed name and
//! an untyped result. This is the same move `servant-foreign` makes in Haskell —
//! derive the client functions from the same source the types come from.
//!
//! A [`Command`] pairs a Rust command name (`snake_case`, used verbatim as the
//! `invoke` key) with a return type and ordered arguments. The generated function
//! name is the naming-iso image of the command name (`workspace_snapshot` →
//! `workspaceSnapshot`).

use crate::ir::TsType;
use crate::naming::to_camel_case;

/// How a [`Command`] reaches the backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transport {
    /// Tauri `invoke("cmd", args)` from `@tauri-apps/api/core`.
    Tauri,
    /// A generic async `request("cmd", args)` helper the consumer supplies — a
    /// seam for HTTP / WebSocket transports without committing typebridge to one.
    Fetch,
}

impl Transport {
    /// The import line a [`crate::bridge::Bridge`] should emit for this transport,
    /// if any.
    pub(crate) fn import_line(self) -> Option<&'static str> {
        match self {
            Transport::Tauri => Some("import { invoke } from \"@tauri-apps/api/core\";"),
            // The consumer wires `request` to their own client; no fixed import.
            Transport::Fetch => None,
        }
    }
}

/// A single argument to a command. The `name` is the wire (camelCase) key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arg {
    /// The argument name as it appears in the `invoke` payload (camelCase).
    pub name: String,
    /// The argument's type.
    pub ty: TsType,
    /// Emit `name?: T` in the generated TypeScript function signature.
    pub optional: bool,
}

impl Arg {
    /// An argument whose name is already a wire identifier.
    pub fn new(name: impl Into<String>, ty: TsType) -> Self {
        Arg {
            name: name.into(),
            ty,
            optional: false,
        }
    }

    /// An argument declared with its Rust `snake_case` name; the wire name is
    /// derived through the naming isomorphism.
    pub fn rust(rust_name: &str, ty: TsType) -> Self {
        Arg::new(to_camel_case(rust_name), ty)
    }

    /// Mark this argument optional in the generated function signature.
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

/// A typed command: a backend verb plus its argument and return types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    /// The Rust command name, `snake_case`. Used verbatim as the transport key.
    pub rust_name: String,
    /// The resolved return type (the `T` in `Promise<T>`).
    pub ret: TsType,
    /// Arguments, in declaration order.
    pub args: Vec<Arg>,
    /// An optional doc comment.
    pub docs: Option<String>,
}

impl Command {
    /// A command named `rust_name` returning the named type `ret_type`.
    ///
    /// ```
    /// use typebridge::Command;
    /// let cmd = Command::new("workspace_snapshot", "WorkspaceSnapshot");
    /// assert_eq!(cmd.ts_name(), "workspaceSnapshot");
    /// ```
    pub fn new(rust_name: impl Into<String>, ret_type: impl Into<String>) -> Self {
        Command {
            rust_name: rust_name.into(),
            ret: TsType::named(ret_type),
            args: Vec::new(),
            docs: None,
        }
    }

    /// A command returning an arbitrary [`TsType`] (e.g. `void`, a union, an array).
    pub fn returning(rust_name: impl Into<String>, ret: TsType) -> Self {
        Command {
            rust_name: rust_name.into(),
            ret,
            args: Vec::new(),
            docs: None,
        }
    }

    /// Append an argument (builder style).
    pub fn arg(mut self, arg: Arg) -> Self {
        self.args.push(arg);
        self
    }

    /// Attach a doc comment.
    pub fn with_docs(mut self, docs: impl Into<String>) -> Self {
        self.docs = Some(docs.into());
        self
    }

    /// The generated TypeScript function name (the camelCase image of the command).
    pub fn ts_name(&self) -> String {
        to_camel_case(&self.rust_name)
    }

    /// Render the typed wrapper for the given transport, terminated by a newline.
    pub fn render(&self, transport: Transport) -> String {
        let params = self
            .args
            .iter()
            .map(|a| {
                let opt = if a.optional { "?" } else { "" };
                format!("{}{opt}: {}", a.name, a.ty.render())
            })
            .collect::<Vec<_>>()
            .join(", ");

        // The payload object passed to the transport, if there are args.
        let payload = if self.args.is_empty() {
            String::new()
        } else {
            let keys = self
                .args
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!(", {{ {keys} }}")
        };

        let ret = self.ret.render();
        let name = self.ts_name();
        let key = &self.rust_name;

        let body = match transport {
            Transport::Tauri => format!("  return invoke<{ret}>(\"{key}\"{payload});"),
            Transport::Fetch => format!("  return request(\"{key}\"{payload});"),
        };

        let mut out = String::new();
        if let Some(docs) = &self.docs {
            out.push_str(&format!("/** {docs} */\n"));
        }
        out.push_str(&format!(
            "export function {name}({params}): Promise<{ret}> {{\n{body}\n}}\n"
        ));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nullary_command_renders() {
        let ts = Command::new("workspace_snapshot", "WorkspaceSnapshot").render(Transport::Tauri);
        assert!(
            ts.contains("export function workspaceSnapshot(): Promise<WorkspaceSnapshot> {"),
            "{ts}"
        );
        assert!(
            ts.contains("return invoke<WorkspaceSnapshot>(\"workspace_snapshot\");"),
            "{ts}"
        );
    }

    #[test]
    fn arguments_become_params_and_payload() {
        let ts = Command::new("run_query", "QueryResult")
            .arg(Arg::rust("connection_id", TsType::string()))
            .arg(Arg::new("sql", TsType::string()))
            .render(Transport::Tauri);
        assert!(
            ts.contains("export function runQuery(connectionId: string, sql: string): Promise<QueryResult> {"),
            "{ts}"
        );
        // snake key preserved for the transport, camel keys in the payload.
        assert!(
            ts.contains("return invoke<QueryResult>(\"run_query\", { connectionId, sql });"),
            "{ts}"
        );
    }

    #[test]
    fn optional_arguments_become_optional_params() {
        let ts = Command::new("db_run_query", "QueryResult")
            .arg(Arg::rust("connection_id", TsType::string()))
            .arg(Arg::new("sql", TsType::string()))
            .arg(Arg::rust("max_rows", TsType::number()).optional())
            .render(Transport::Tauri);
        assert!(
            ts.contains(
                "export function dbRunQuery(connectionId: string, sql: string, maxRows?: number): Promise<QueryResult> {"
            ),
            "{ts}"
        );
        assert!(
            ts.contains(
                "return invoke<QueryResult>(\"db_run_query\", { connectionId, sql, maxRows });"
            ),
            "{ts}"
        );
    }

    #[test]
    fn fetch_transport_uses_request_helper() {
        let ts = Command::new("ping", "boolean").render(Transport::Fetch);
        assert!(ts.contains("return request(\"ping\");"), "{ts}");
    }
}
