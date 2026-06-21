# Security Policy

typebridge is a developer tool that turns Rust type/command metadata into
TypeScript source text. It does not run at application runtime and does not
process end-user data. Please report security issues privately when possible.

## Reporting

Do not open a public issue for a vulnerability. Use GitHub private vulnerability
reporting if it is enabled for the repository. If that is not available, contact
a maintainer privately and include:

- affected version or commit
- steps to reproduce
- impact
- whether the issue involves generated output, file writing, or CI integration

## Scope

In scope:

- generated TypeScript that does not match the serde wire shape it claims to
  describe (a correctness/safety bug, since consumers trust the types)
- path handling in `Rendered::write` / `Rendered::check` that could write or read
  outside an intended location
- a drift check that reports "up to date" when the committed file actually differs
- crashes from pathological IR input

Out of scope:

- vulnerabilities in downstream applications that consume the generated types
- the security of backends (`ts-rs`, `specta`, `schemars`) used to produce IR
- transport security of whatever RPC layer the generated `invoke`/`request`
  wrappers call into
