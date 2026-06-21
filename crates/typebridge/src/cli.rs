//! A tiny, zero-dependency CLI driver for the `write` / `check` verbs.
//!
//! The "drift-check in CI" promise is only worth anything if wiring it up is
//! trivial. This module lets a consumer turn an assembled [`crate::Bridge`] into a
//! real generator binary (or `xtask`) in three lines:
//!
//! ```no_run
//! use std::process::ExitCode;
//! use typebridge::{Bridge, Command};
//!
//! fn main() -> ExitCode {
//!     let bridge = Bridge::tauri().command(Command::new("workspace_snapshot", "WorkspaceSnapshot"));
//!     typebridge::cli::run(&bridge, "src/generated/api.ts")
//! }
//! ```
//!
//! Then `cargo run -- write` regenerates the file and `cargo run -- check` fails
//! (non-zero exit) in CI when the committed bindings have drifted. No argument
//! parser, no third-party dependency.

use std::process::ExitCode;

use crate::bridge::Bridge;
use crate::check::CheckOutcome;

/// The result of a CLI invocation: a process exit code plus a human message.
///
/// Separating this from [`run`] keeps the decision logic pure and testable — `run`
/// only adds argv reading and printing on top.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliResult {
    /// `0` success, `1` drift/missing on `check`, `2` usage error.
    pub code: u8,
    /// Message to print (stdout when `code == 0`, stderr otherwise).
    pub message: String,
    /// Whether this invocation only printed help/usage.
    pub is_usage: bool,
}

impl CliResult {
    fn ok(message: impl Into<String>) -> Self {
        CliResult {
            code: 0,
            message: message.into(),
            is_usage: false,
        }
    }

    fn fail(code: u8, message: impl Into<String>) -> Self {
        CliResult {
            code,
            message: message.into(),
            is_usage: false,
        }
    }

    fn usage(code: u8) -> Self {
        CliResult {
            code,
            message: USAGE.to_string(),
            is_usage: true,
        }
    }
}

const USAGE: &str = "\
typebridge — assemble a TypeScript API surface from Rust types

USAGE:
    <bin> write [PATH]    render and write the bindings (PATH overrides the default)
    <bin> check [PATH]    render and compare; non-zero exit on drift or missing file
    <bin> help            show this message";

/// Run the CLI against process arguments, performing the requested IO, and return
/// a [`ExitCode`]. Prints the outcome (stdout on success, stderr on failure).
pub fn run(bridge: &Bridge, default_out: &str) -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = execute(&args, bridge, default_out);
    if result.code == 0 {
        println!("{}", result.message);
    } else {
        eprintln!("{}", result.message);
    }
    ExitCode::from(result.code)
}

/// The pure-ish core of the CLI: dispatch `args` (already stripped of `argv[0]`)
/// against `bridge`, performing file IO for `write`/`check`. Returns a
/// [`CliResult`] instead of exiting, so it is unit-testable.
pub fn execute(args: &[String], bridge: &Bridge, default_out: &str) -> CliResult {
    let (verb, rest) = match args.split_first() {
        Some((v, rest)) => (v.as_str(), rest),
        None => return CliResult::usage(2),
    };

    match verb {
        "help" | "--help" | "-h" => CliResult::usage(0),
        "write" => {
            let path = rest.first().map(String::as_str).unwrap_or(default_out);
            let rendered = bridge.render();
            match rendered.write(path) {
                Ok(()) => {
                    CliResult::ok(format!("wrote {path} ({} bytes)", rendered.contents.len()))
                }
                Err(e) => CliResult::fail(1, format!("failed to write {path}: {e}")),
            }
        }
        "check" => {
            let path = rest.first().map(String::as_str).unwrap_or(default_out);
            let rendered = bridge.render();
            match rendered.check(path) {
                Ok(outcome) if outcome.is_up_to_date() => CliResult::ok(outcome.summary()),
                Ok(outcome @ (CheckOutcome::Drift { .. } | CheckOutcome::Missing { .. })) => {
                    CliResult::fail(1, outcome.summary())
                }
                Ok(outcome) => CliResult::ok(outcome.summary()),
                Err(e) => CliResult::fail(1, format!("failed to read {path}: {e}")),
            }
        }
        other => CliResult::fail(2, format!("unknown command: {other}\n\n{USAGE}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;

    fn sample() -> Bridge {
        Bridge::tauri().command(Command::new("ping", "boolean"))
    }

    fn arg(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn no_args_is_usage_error() {
        let r = execute(&[], &sample(), "out.ts");
        assert_eq!(r.code, 2);
        assert!(r.is_usage);
    }

    #[test]
    fn help_is_zero_exit() {
        let r = execute(&arg("help"), &sample(), "out.ts");
        assert_eq!(r.code, 0);
        assert!(r.message.contains("USAGE"));
    }

    #[test]
    fn unknown_command_is_usage_error() {
        let r = execute(&arg("frobnicate"), &sample(), "out.ts");
        assert_eq!(r.code, 2);
        assert!(r.message.contains("unknown command: frobnicate"));
    }

    #[test]
    fn write_then_check_cycle() {
        let dir = std::env::temp_dir().join("typebridge-cli-test");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("api.ts");
        let path_str = path.to_str().unwrap().to_string();

        // check before write -> missing -> exit 1
        let pre = execute(&["check".into(), path_str.clone()], &sample(), "unused.ts");
        assert_eq!(pre.code, 1, "{}", pre.message);
        assert!(pre.message.contains("missing"));

        // write -> exit 0
        let w = execute(&["write".into(), path_str.clone()], &sample(), "unused.ts");
        assert_eq!(w.code, 0, "{}", w.message);
        assert!(w.message.starts_with("wrote "));

        // check after write -> up to date -> exit 0
        let ok = execute(&["check".into(), path_str.clone()], &sample(), "unused.ts");
        assert_eq!(ok.code, 0, "{}", ok.message);

        // tamper -> drift -> exit 1
        std::fs::write(&path, "// tampered\n").unwrap();
        let drift = execute(&["check".into(), path_str], &sample(), "unused.ts");
        assert_eq!(drift.code, 1, "{}", drift.message);
        assert!(drift.message.contains("stale"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_uses_default_path_when_omitted() {
        let dir = std::env::temp_dir().join("typebridge-cli-default");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("default.ts");
        let path_str = path.to_str().unwrap().to_string();

        let w = execute(&arg("write"), &sample(), &path_str);
        assert_eq!(w.code, 0, "{}", w.message);
        assert!(path.exists());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
