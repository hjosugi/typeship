//! ts-rs backend adapter for [`typebridge`].
//!
//! [`ts-rs`] owns the hard part — reading a `#[derive(TS)]` Rust type and
//! rendering its TypeScript declaration, honouring `#[ts(rename_all = …)]`,
//! `Option`, `Vec`, maps, and serde-style attributes. typebridge owns the
//! assembly around it: headers, typed command wrappers, an optional `assertNever`
//! helper, and the CI drift check.
//!
//! This crate is the seam between the two. [`decl`] turns any `T: TS` into a
//! [`typebridge::ir::Decl`] holding ts-rs's finished `export …` line verbatim
//! ([`DeclBody::Raw`]); the [`Bridge`] then assembles those declarations exactly
//! as it does hand-built ones.
//!
//! ```
//! use ts_rs::TS;
//! use typebridge::{Bridge, Command};
//!
//! #[derive(TS)]
//! #[ts(rename_all = "camelCase")]
//! struct WorkspaceSnapshot {
//!     active_connection_id: String,
//! }
//!
//! let ts = Bridge::tauri()
//!     .decl(&typebridge_ts_rs::decl::<WorkspaceSnapshot>())
//!     .command(Command::new("workspace_snapshot", "WorkspaceSnapshot"))
//!     .render();
//!
//! assert!(ts.contents.contains("export type WorkspaceSnapshot"));
//! assert!(ts.contents.contains("activeConnectionId: string"));
//! assert!(ts.contents.contains("export function workspaceSnapshot()"));
//! ```
//!
//! [`ts-rs`]: https://github.com/Aleph-Alpha/ts-rs
//! [`Bridge`]: typebridge::Bridge
//! [`DeclBody::Raw`]: typebridge::ir::DeclBody::Raw

use ts_rs::{Config, TS};
use typebridge::ir::Decl;

/// Lower a `#[derive(TS)]` type into a [`Decl`] using ts-rs's default config.
///
/// The type must *have* a declaration: structs and enums do; primitives, tuples,
/// and transparent wrappers do not, and ts-rs will panic if asked to declare one.
/// That mirrors ts-rs's own contract.
pub fn decl<T: TS + ?Sized>() -> Decl {
    decl_with::<T>(&Config::default())
}

/// Lower a `#[derive(TS)]` type into a [`Decl`] using a caller-supplied ts-rs
/// [`Config`] (for example, to set the large-integer type used for `u64`/`i64`).
pub fn decl_with<T: TS + ?Sized>(cfg: &Config) -> Decl {
    // `ident` is the bare name without generic arguments — a stable key for the
    // declaration. `decl` is the finished body; ts-rs prefixes `export ` itself
    // when exporting, so we mirror that to produce a complete statement.
    let name = <T as TS>::ident(cfg);
    let rendered = format!("export {}", <T as TS>::decl(cfg));
    Decl::raw(name, rendered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use typebridge::{Bridge, Command};

    #[derive(TS)]
    #[ts(rename_all = "camelCase")]
    #[allow(dead_code)]
    struct Connection {
        id: String,
        latency_ms: u32,
        rows: Option<String>,
        objects: Vec<String>,
    }

    #[derive(TS)]
    #[ts(rename_all = "lowercase")]
    #[allow(dead_code)]
    enum ConnectionStatus {
        Connected,
        Idle,
    }

    #[test]
    fn struct_lowers_to_export_with_camelcase_fields() {
        let decl = decl::<Connection>();
        let ts = decl.render();
        assert!(ts.starts_with("export type Connection ="), "{ts}");
        // ts-rs applied the camelCase rename; typebridge did not re-mangle it.
        assert!(ts.contains("latencyMs: number"), "{ts}");
        // By default ts-rs renders `Option<T>` as a present, nullable key
        // (`T | null`), not an optional key. To get `rows?: string`, the Rust
        // field needs `#[ts(optional)]` — see the note in the handoff doc.
        assert!(ts.contains("rows: string | null"), "{ts}");
        assert!(ts.contains("objects: Array<string>"), "{ts}");
        // Exactly one trailing newline.
        assert!(ts.ends_with(";\n"), "{ts:?}");
    }

    #[test]
    fn enum_lowers_to_closed_string_union() {
        let ts = decl::<ConnectionStatus>().render();
        assert!(
            ts.contains("export type ConnectionStatus = \"connected\" | \"idle\";"),
            "{ts}"
        );
    }

    #[derive(TS)]
    #[allow(dead_code)]
    enum Event {
        Connected,
        Query { sql: String, rows: u32 },
        Failed(String),
    }

    #[test]
    fn data_carrying_enum_lowers_to_discriminated_union() {
        // ts-rs renders a data-carrying enum as a union of per-variant objects.
        // typebridge wraps that verbatim; we only assert it stays a closed union.
        let ts = decl::<Event>().render();
        assert!(ts.starts_with("export type Event ="), "{ts}");
        assert!(ts.contains(" | "), "expected a union, got: {ts}");
        assert!(ts.ends_with(";\n"), "{ts:?}");
    }

    #[derive(TS)]
    #[allow(dead_code)]
    struct Wrapper<T> {
        value: T,
        ok: bool,
    }

    #[test]
    fn generic_type_keeps_bare_ident_and_renders_placeholder() {
        let decl = decl::<Wrapper<i32>>();
        // `ident` is the bare name, without the generic argument.
        assert_eq!(decl.name, "Wrapper");
        let ts = decl.render();
        assert!(ts.starts_with("export type Wrapper"), "{ts}");
        assert!(ts.contains("value:"), "{ts}");
    }

    #[derive(TS)]
    #[ts(rename_all = "camelCase")]
    #[allow(dead_code)]
    struct WithMap {
        counts: std::collections::HashMap<String, u32>,
    }

    #[test]
    fn map_field_lowers_to_a_keyed_value_type() {
        // We do not pin ts-rs's exact map syntax (Record vs index signature); we
        // assert the value type survives, which is what matters for consumers.
        let ts = decl::<WithMap>().render();
        assert!(ts.contains("counts:"), "{ts}");
        assert!(ts.contains("number"), "expected the map value type: {ts}");
    }

    #[test]
    fn bridge_assembles_ts_rs_decls_with_commands() {
        let ts = Bridge::tauri()
            .decl(&decl::<ConnectionStatus>())
            .decl(&decl::<Connection>())
            .command(Command::new("workspace_snapshot", "WorkspaceSnapshot"))
            .render();

        // Header + transport import from typebridge, type bodies from ts-rs.
        assert!(ts
            .contents
            .starts_with("// @generated by typebridge. Do not edit."));
        assert!(ts
            .contents
            .contains("import { invoke } from \"@tauri-apps/api/core\";"));
        assert!(ts.contents.contains("export type Connection ="));
        assert!(ts
            .contents
            .contains("export function workspaceSnapshot(): Promise<WorkspaceSnapshot>"));
    }
}
