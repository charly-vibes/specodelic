//! `spk parse` — structured Spec IR export (beads specodelic-9rv).
//!
//! Purpose: expose the typed `Spec` IR through the CLI so external fleet
//! tools (espectacular's `ah sync`) consume the structured layer without
//! reimplementing the markdown-table parser — the versioned binary is the
//! single integration boundary. Responsibilities: a thin, single-file
//! wrapper over [`spec::Spec::from_file`] with the hostile-input gates
//! (regular files only, bounded size — the specodelic-suz discipline) and
//! a labeled error type carrying a remediation hint for every failure
//! shape. Parse is syntax-only: it never embeds lint status (design D3 —
//! single-purpose envelopes; consumers chain `spk lint` themselves).
//!
//! Rationale: the lib work was already done (`spec::Spec` derives
//! `Serialize`); only the CLI surface and error shaping were missing.

use std::path::{Path, PathBuf};

use crate::spec::{self, ParseError, Spec};

/// The single-file input cap, matching the corpus-wide discipline in
/// main.rs (specodelic-suz: a FIFO or device read would block or OOM).
pub const MAX_INPUT_BYTES: u64 = 2 * 1024 * 1024;

/// Why `spk parse` failed — every variant is labeled (names the path)
/// and carries a remediation hint (error-contract discipline, design D4).
#[derive(Debug, thiserror::Error)]
pub enum ParseCommandError {
    /// The path does not exist (or metadata could not be read).
    #[error("{path}: file not found — parse takes exactly one existing spec file path")]
    NotFound { path: PathBuf },
    /// A FIFO, device, or other special file — never read (suz).
    #[error(
        "{path}: not a regular file — only regular files are ingested, never a FIFO, device, or other special file"
    )]
    NotRegularFile { path: PathBuf },
    /// Oversized regular file — rejected before read (suz).
    #[error(
        "{path}: exceeds the 2 MiB input cap — corpus files are ~10-50 KB; split or move the file"
    )]
    Oversized { path: PathBuf },
    /// The file exists but could not be read as UTF-8 text.
    #[error("{path}: unreadable ({source})")]
    Unreadable {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The file could not be parsed against the four-layer grammar.
    #[error("{error}")]
    Parse { error: ParseError },
}

impl ParseCommandError {
    /// The remediation hint for this failure shape.
    pub fn hint(&self) -> String {
        match self {
            ParseCommandError::NotFound { .. } => {
                "check the path — parse takes one existing spec .md file: spk parse <file>".into()
            }
            ParseCommandError::NotRegularFile { .. } => {
                "pass a regular *.md spec file as the argument".into()
            }
            ParseCommandError::Oversized { .. } => {
                "split or move the file, then re-run: spk parse <file>".into()
            }
            ParseCommandError::Unreadable { .. } => {
                "check file permissions, then re-run: spk parse <file>".into()
            }
            ParseCommandError::Parse { .. } => {
                "fix the reported frontmatter/table error, then re-run: spk parse <file>".into()
            }
        }
    }
}

/// Parse one spec file into the full structured IR. Thin wrapper over
/// [`Spec::from_file`] with the hostile-input gates (design D1: no new
/// IR, no transformation — the parsed `Spec` is emitted directly).
pub fn parse_file(path: &Path) -> Result<Spec, ParseCommandError> {
    let meta = std::fs::metadata(path).map_err(|_| ParseCommandError::NotFound {
        path: path.to_path_buf(),
    })?;
    if !meta.is_file() {
        return Err(ParseCommandError::NotRegularFile {
            path: path.to_path_buf(),
        });
    }
    if meta.len() > MAX_INPUT_BYTES {
        return Err(ParseCommandError::Oversized {
            path: path.to_path_buf(),
        });
    }
    // Frontmatter and no-frontmatter parse errors carry a placeholder
    // label (`<frontmatter>` / `<input>`) — remap to the real path so the
    // error envelope names the file (parse_str handles Cell labels
    // path-less too).
    let text = std::fs::read_to_string(path).map_err(|source| ParseCommandError::Unreadable {
        path: path.to_path_buf(),
        source,
    })?;
    spec::parse_str(&text)
        .map_err(|error| ParseCommandError::Parse {
            error: match error {
                ParseError::File(stem, msg) if stem.starts_with('<') => {
                    ParseError::File(path.display().to_string(), msg)
                }
                other => other,
            },
        })
        .map(|mut spec| {
            spec.path = Some(path.to_path_buf());
            spec
        })
}

/// Sanity shape for the unit-level contract: a missing file labels itself.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_names_the_path_and_hint() {
        let err = parse_file(Path::new("/nonexistent/ghost.md")).unwrap_err();
        assert!(err.to_string().contains("ghost.md"));
        assert!(err.hint().contains("spk parse"));
    }
}
