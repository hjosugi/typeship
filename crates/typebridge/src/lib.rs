//! # typebridge
//!
//! `typebridge` turns Rust types into a TypeScript API surface — type
//! declarations, typed command wrappers, and a CI drift-check — without making
//! Rust look like TypeScript or TypeScript look like Rust.
//!
//! It is deliberately a **thin facade**, not another reflection engine. The Rust
//! ecosystem already has good per-type renderers ([`ts-rs`], [`specta`],
//! [`typeshare`]) and JSON-Schema generators ([`schemars`]). What none of them own
//! is the *assembly* layer: a deterministic, formatted, header-stamped file that
//! bundles declarations + command clients, plus a `--check` verb that fails CI when
//! the committed bindings drift. That is what this crate owns.
//!
//! ```text
//!   Rust types (serde) ──backend(ts-rs/…)──▶ declaration strings ┐
//!                                                                 ├─▶ Bridge ─▶ .ts
//!   command metadata ───────────────────────────────────────────┘     │
//!                                                              write() / check()
//! ```
//!
//! ## Design lineage
//!
//! The model is informed by the functional-programming codegen tradition
//! (`purescript-bridge`, `elm-bridge`, `haskell-to-elm`, `servant-foreign`,
//! aeson's `Generic` deriving). See `docs/design-fp-principles.md`. The principles
//! that shaped the API:
//!
//! - **Types are the single source of truth.** Never hand-maintain a parallel IDL;
//!   derive the TypeScript from the serde-annotated Rust type and CI-check the
//!   committed output (anti–OpenAPI-drift).
//! - **ADTs map to TypeScript unions.** A struct is a record; an enum is a closed
//!   discriminated/literal union ([`ir::TsType::StringLiteralUnion`]).
//! - **Totality / exhaustiveness.** Generated unions stay *closed* so a TypeScript
//!   `switch` can be proven exhaustive; opt into an emitted `assertNever` helper.
//! - **Derivation over hand-writing.** One generic emitter over an introspectable
//!   schema value ([`ir`]) rather than `format!` soup at every call site.
//! - **Naming is an isomorphism.** `snake_case` ⇄ `lowerCamelCase` is a
//!   bidirectional policy ([`naming`]), applied symmetrically.
//! - **Encode/decode symmetry has known break points.** serde `untagged`,
//!   `flatten`, `skip_serializing_if`, and `transparent` quietly desynchronise the
//!   wire shape; [`diagnostics`] names the hazards.
//!
//! ## Quick start
//!
//! ```
//! use typebridge::{Bridge, Command};
//! use typebridge::ir::{Decl, Field, TsType};
//!
//! let status = Decl::alias(
//!     "ConnectionStatus",
//!     TsType::string_literals(["connected", "idle"]),
//! );
//!
//! let ts = Bridge::tauri()
//!     .decl(&status)
//!     .command(Command::new("workspace_snapshot", "WorkspaceSnapshot"))
//!     .render();
//!
//! assert!(ts.contents.contains("export type ConnectionStatus = \"connected\" | \"idle\";"));
//! assert!(ts.contents.contains("export function workspaceSnapshot(): Promise<WorkspaceSnapshot>"));
//! ```
//!
//! [`ts-rs`]: https://github.com/Aleph-Alpha/ts-rs
//! [`specta`]: https://github.com/specta-rs/specta
//! [`typeshare`]: https://github.com/1Password/typeshare
//! [`schemars`]: https://github.com/GREsau/schemars

pub mod bridge;
pub mod check;
pub mod cli;
pub mod command;
pub mod diagnostics;
pub mod ir;
pub mod naming;

pub use bridge::{Bridge, Rendered};
pub use check::CheckOutcome;
pub use command::{Arg, Command, Transport};
