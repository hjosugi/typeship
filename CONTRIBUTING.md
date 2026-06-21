# Contributing to typebridge

Thanks for helping. typebridge is small on purpose: a thin, dependency-light
facade that assembles a TypeScript API surface from Rust-owned types and command
metadata. Keep changes small and well-tested, and keep the core crate free of
third-party dependencies.

## Development setup

Required:

- Rust stable (1.74+)

Run the same checks CI runs:

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Design rules

These are load-bearing; a change that breaks one needs a strong reason.

- **Core stays zero-dependency.** Anything that needs `serde`, `ts-rs`, `specta`,
  or `schemars` belongs in a separate backend adapter crate, not in
  `crates/typebridge`.
- **Output is a pure function of the input.** `Bridge::render` must be
  deterministic — no clocks, no environment, no map iteration order. This is what
  makes the CI drift check (`Rendered::check`) meaningful.
- **The IR is the contract.** Add a `TsType`/`Decl` variant before adding a
  special case to the renderer. The renderer should never branch on string
  contents.
- **Closed unions stay closed.** Rust enums render as string-literal or
  discriminated unions, never as a bare `string`. Totality is the point.
- **Naming is one bidirectional policy.** `snake_case ⇄ lowerCamelCase` lives in
  `naming`; do not hand-mangle identifiers elsewhere.

## Tests

- Unit tests live next to the code they cover.
- `crates/typebridge/tests/irodori_surface.rs` is the coordination contract with
  `irodori-table`. If you change rendering, update it deliberately and explain why
  in the PR — it documents the shape a real consumer depends on.

## Commit and PR hygiene

- One logical change per PR.
- Run `cargo fmt` before committing.
- Generated/committed artifacts must be regenerated in the same PR that changes
  their source, so the drift check stays green.
