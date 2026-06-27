use std::fs;
use std::io;
use std::path::Path;

use crate::check::CheckOutcome;

/// The rendered TypeScript module, plus the verbs to persist or verify it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rendered {
    /// The full file contents, terminated by a single newline.
    pub contents: String,
}

impl Rendered {
    /// Write the contents to `path`, creating parent directories as needed.
    pub fn write(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, &self.contents)
    }

    /// Compare the contents against the file at `path` without writing.
    ///
    /// Returns [`CheckOutcome::Missing`] if the file is absent,
    /// [`CheckOutcome::UpToDate`] if it matches, or [`CheckOutcome::Drift`] with
    /// the first differing line otherwise.
    pub fn check(&self, path: impl AsRef<Path>) -> io::Result<CheckOutcome> {
        let path = path.as_ref();
        let path_str = path.display().to_string();
        match fs::read_to_string(path) {
            Ok(actual) => Ok(CheckOutcome::compare(
                &path_str,
                &self.contents,
                Some(&actual),
            )),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                Ok(CheckOutcome::compare(&path_str, &self.contents, None))
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Bridge, Command};

    #[test]
    fn write_then_check_round_trips() {
        let dir = std::env::temp_dir().join("typeship-bridge-test");
        let path = dir.join("api.ts");
        let rendered = Bridge::tauri()
            .command(Command::new("ping", "boolean"))
            .render();

        rendered.write(&path).expect("write");
        let outcome = rendered.check(&path).expect("check");
        assert!(outcome.is_up_to_date(), "{}", outcome.summary());

        // Mutate the file on disk; the check must now report drift.
        std::fs::write(&path, "// tampered\n").expect("tamper");
        let drift = rendered.check(&path).expect("check");
        assert!(!drift.is_up_to_date());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
