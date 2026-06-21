# typebridge

`typebridge` is a small Rust library for assembling generated TypeScript API
surfaces from Rust-owned types and command metadata.

The crate is deliberately a facade, not a reflection engine. Per-type generators
such as `ts-rs`, `specta`, `typeshare`, or `schemars` can own the hard problem of
reading Rust types. `typebridge` owns the assembly layer:

- deterministic generated-file headers;
- exported TypeScript declarations;
- typed command wrappers, currently for Tauri `invoke` and a generic `request`;
- a drift check for CI.

## Example

```rust
use typebridge::ir::{Decl, Field, TsType};
use typebridge::{Arg, Bridge, Command};

let profile = Decl::interface(
    "ConnectionProfile",
    [
        Field::rust("id", TsType::string()),
        Field::rust("host", TsType::string()).optional(),
    ],
);

let ts = Bridge::tauri()
    .decl(&profile)
    .command(
        Command::new("db_connect", "ConnectionInfo")
            .arg(Arg::new("profile", TsType::named("ConnectionProfile"))),
    )
    .render();

assert!(ts.contents.contains(
    "export function dbConnect(profile: ConnectionProfile): Promise<ConnectionInfo>"
));
```

## Workspace layout

- `crates/typebridge` — the core facade. **Zero third-party dependencies.** IR,
  renderer, command wrappers, drift check, and a tiny `cli` driver.
- `crates/typebridge-ts-rs` — the [`ts-rs`](https://github.com/Aleph-Alpha/ts-rs)
  backend adapter. `decl::<T>()` lowers a `#[derive(TS)]` type into a typebridge
  declaration. A `specta` / `schemars` adapter could sit alongside it.

## Generating in CI

`typebridge::cli::run` turns an assembled `Bridge` into a generator with `write`
and `check` verbs — `check` exits non-zero when the committed file has drifted:

```rust
fn main() -> std::process::ExitCode {
    let bridge = build_bridge();
    typebridge::cli::run(&bridge, "src/generated/api.ts")
}
```

See the end-to-end example (ts-rs types → assembly → CLI):

```sh
cargo run -p typebridge-ts-rs --example generate -- write /tmp/api.ts
cargo run -p typebridge-ts-rs --example generate -- check /tmp/api.ts
```

## Current Scope

The MVP covers the `irodori-table` desktop boundary:

- closed string-literal unions for Rust enums;
- interfaces for command payload structs;
- `snake_case` Rust names rendered as `camelCase` TypeScript names;
- optional object fields for serde shapes that may be absent;
- typed Tauri command wrappers (`invoke<T>`);
- byte-for-byte drift checking against committed generated files;
- pre-rendered declarations from a backend (ts-rs today) assembled verbatim.

## Development

```bash
cargo test
cargo package --list
```

`cargo package --list` is a useful packaging smoke test because the crate
manifest points at this README.

## License

Licensed under either MIT or 0BSD.
