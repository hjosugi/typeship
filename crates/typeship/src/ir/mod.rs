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

mod decl;
mod ty;

pub use decl::{Decl, DeclBody, Field};
pub use ty::TsType;
