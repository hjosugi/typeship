//! Drift checking — the anti-OpenAPI guarantee.
//!
//! OpenAPI's failure mode is a hand-written spec document that silently drifts
//! from the code it claims to describe. typebridge's answer is the same one the
//! FP world reaches for: the generated artifact is a *pure function* of the
//! source types, so CI can re-render and assert byte-equality against the
//! committed file. If they differ, someone changed a Rust type without
//! regenerating — and the build fails instead of shipping a lie.
//!
//! [`crate::Rendered::check`] produces a [`CheckOutcome`]; a CI step turns a
//! non-up-to-date outcome into a non-zero exit.

/// The result of comparing freshly-rendered output against a committed file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckOutcome {
    /// The committed file matches the freshly-rendered output exactly.
    UpToDate,
    /// No file exists at the target path; bindings were never written.
    Missing {
        /// The path that was expected to hold the bindings.
        path: String,
    },
    /// The committed file differs from the freshly-rendered output.
    Drift {
        /// The path of the stale file.
        path: String,
        /// 1-based line number of the first differing line.
        first_diff_line: usize,
        /// The committed (stale) line, if any.
        committed: Option<String>,
        /// The expected (freshly-rendered) line, if any.
        expected: Option<String>,
    },
}

impl CheckOutcome {
    /// Compare `expected` (freshly rendered) against `actual` (on disk) for `path`.
    /// `actual` is `None` when the file is absent.
    pub(crate) fn compare(path: &str, expected: &str, actual: Option<&str>) -> Self {
        let Some(actual) = actual else {
            return CheckOutcome::Missing {
                path: path.to_string(),
            };
        };
        if expected == actual {
            return CheckOutcome::UpToDate;
        }
        // Find the first differing line for an actionable message.
        let mut exp_lines = expected.lines();
        let mut act_lines = actual.lines();
        let mut line = 0usize;
        loop {
            line += 1;
            match (exp_lines.next(), act_lines.next()) {
                (Some(e), Some(a)) if e == a => continue,
                (e, a) => {
                    return CheckOutcome::Drift {
                        path: path.to_string(),
                        first_diff_line: line,
                        committed: a.map(str::to_string),
                        expected: e.map(str::to_string),
                    };
                }
            }
        }
    }

    /// Whether the committed file is current.
    pub fn is_up_to_date(&self) -> bool {
        matches!(self, CheckOutcome::UpToDate)
    }

    /// A one-line, CI-log-friendly summary.
    pub fn summary(&self) -> String {
        match self {
            CheckOutcome::UpToDate => "bindings up to date".to_string(),
            CheckOutcome::Missing { path } => {
                format!("missing bindings: {path} (run the generator)")
            }
            CheckOutcome::Drift {
                path,
                first_diff_line,
                ..
            } => format!(
                "stale bindings: {path} differs at line {first_diff_line} (run the generator)"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_is_up_to_date() {
        let outcome = CheckOutcome::compare("a.ts", "x\ny\n", Some("x\ny\n"));
        assert!(outcome.is_up_to_date());
        assert_eq!(outcome.summary(), "bindings up to date");
    }

    #[test]
    fn absent_file_is_missing() {
        let outcome = CheckOutcome::compare("a.ts", "x\n", None);
        assert!(matches!(outcome, CheckOutcome::Missing { .. }));
        assert!(!outcome.is_up_to_date());
    }

    #[test]
    fn difference_reports_first_line() {
        let outcome = CheckOutcome::compare("a.ts", "x\nNEW\nz\n", Some("x\nOLD\nz\n"));
        match outcome {
            CheckOutcome::Drift {
                first_diff_line,
                committed,
                expected,
                ..
            } => {
                assert_eq!(first_diff_line, 2);
                assert_eq!(committed.as_deref(), Some("OLD"));
                assert_eq!(expected.as_deref(), Some("NEW"));
            }
            other => panic!("expected drift, got {other:?}"),
        }
    }

    #[test]
    fn committed_has_extra_trailing_lines() {
        // Expected ends sooner than the committed file.
        let outcome = CheckOutcome::compare("a.ts", "x\n", Some("x\nextra\n"));
        match outcome {
            CheckOutcome::Drift {
                first_diff_line,
                committed,
                expected,
                ..
            } => {
                assert_eq!(first_diff_line, 2);
                assert_eq!(committed.as_deref(), Some("extra"));
                assert_eq!(expected, None);
            }
            other => panic!("expected drift, got {other:?}"),
        }
    }

    #[test]
    fn committed_is_truncated() {
        // Committed file is missing trailing lines the fresh render has.
        let outcome = CheckOutcome::compare("a.ts", "x\ny\nz\n", Some("x\n"));
        match outcome {
            CheckOutcome::Drift {
                first_diff_line,
                committed,
                expected,
                ..
            } => {
                assert_eq!(first_diff_line, 2);
                assert_eq!(committed, None);
                assert_eq!(expected.as_deref(), Some("y"));
            }
            other => panic!("expected drift, got {other:?}"),
        }
    }

    #[test]
    fn trailing_newline_only_difference_is_drift() {
        // Byte-unequal even though every line matches: still drift, never up-to-date.
        let outcome = CheckOutcome::compare("a.ts", "x\ny\n", Some("x\ny"));
        assert!(!outcome.is_up_to_date());
        assert!(matches!(outcome, CheckOutcome::Drift { .. }));
    }

    #[test]
    fn summary_strings_are_actionable() {
        assert!(CheckOutcome::compare("a.ts", "x\n", None)
            .summary()
            .contains("run the generator"));
        assert!(CheckOutcome::compare("a.ts", "x\n", Some("y\n"))
            .summary()
            .contains("line 1"));
    }
}
