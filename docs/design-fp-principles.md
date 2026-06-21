# Functional Codegen Principles

`typebridge` follows the same broad shape as tools such as `purescript-bridge`,
`haskell-to-elm`, `servant-foreign`, and serde/aeson-style generic deriving:
describe the boundary once, treat that description as data, and render a target
language artifact deterministically.

## Source Types Own the Boundary

Rust command payloads are the source of truth. TypeScript declarations should be
derived from the Rust/serde wire shape rather than maintained as a parallel IDL.
The generated file is committed, but CI should be able to re-render it and fail
when it drifts.

## Products and Sums Stay Visible

Rust structs are product types and render as TypeScript object records or
interfaces. Rust enums are sum types and should remain closed TypeScript unions
where the wire format permits it. For C-like enums, string-literal unions keep
the frontend exhaustive:

```ts
export type ConnectionStatus = "connected" | "idle";
```

When a union is closed, a generated `assertNever` helper lets consumers turn a
new unhandled variant into a TypeScript compile error.

## Naming Is a Contract

Rust fields remain `snake_case`; JSON and TypeScript properties are commonly
`camelCase`. Treat that mapping as a single bidirectional policy rather than two
unrelated string transformations. Types such as `WorkspaceSnapshot` pass through
unchanged.

## Encode and Decode Must Agree

Serde attributes can change the JSON shape without changing the nominal Rust
type. A bridge must either model those attributes or warn clearly. Important
hazards include:

- `untagged` enums, because there is no discriminant to switch on;
- `flatten`, because nested fields are spliced into the parent object;
- `skip_serializing_if` and `default`, because keys may be absent;
- `transparent`, because the wrapper disappears on the wire;
- rename collisions, because two Rust fields can map to the same JSON key.

The `diagnostics` module keeps these hazards in one shared catalog so future
backend adapters and docs can use the same warning text.

## Assembly Is Separate From Reflection

The core crate does not need to know how `ts-rs`, `specta`, or `schemars`
discovers Rust types. Those tools can lower Rust types into declaration strings
or a small intermediate representation. `typebridge` then assembles a complete
TypeScript module with headers, imports, command wrappers, optional helpers, and
drift checking.
