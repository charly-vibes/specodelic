//! `spk hooks install` / `spk hooks uninstall` — wire the specodelic
//! dual-format gate (`spk lint openspec`) into a repository's hook chain.
//!
//! Purpose: make dual-format drift fail at commit time, not just in CI.
//! Responsibilities: inject/remove a marker-guarded managed block into
//! the `pre-commit:` stage of the repo's lefthook config, purely
//! additively, idempotently, and with honest refusals (missing config,
//! unsupported framework, unanchorable structure). Rationale: beads owns
//! `core.hooksPath` in charly-family repos (AGENTS.md sibling-tool hard
//! blockers) and its shims chain to lefthook, so the lefthook config is
//! the sanctioned extension point — this module never claims
//! `core.hooksPath` and never writes `.git/hooks/*` or `.beads/hooks/*`.
//!
//! Wiring builds on `genesis::git_hooks` (0.8): `framework()` gates
//! the config type; marker conventions follow
//! `managed_block::BlockDef` (comment-prefixed variants). Stage-level
//! injection is local (see `wrapper_at` for why genesis's
//! `ensure_wired` is not used). The
//! one piece genesis deliberately excludes ("no bespoke string
//! surgery") is the *inside-mapping* insertion — empirically required,
//! because lefthook 1.13.6 rejects a stage section that ends up with
//! two `commands:` keys (`yaml: unmarshal errors: mapping key
//! "commands" already defined`), which would kill the entire hook chain
//! including beads' own gates. Indentation is never assumed: it is
//! inferred from the stage's first child key line (YAML allows any
//! consistent indent; mixed indent within one mapping fails the same
//! way). Upstream consolidation target: `lefthook::ensure_command_wired`
//! (beads specodelic-x56) — fold this local logic into genesis when it
//! ships.

use std::path::{Path, PathBuf};

use genesis::git_hooks::{Framework, framework};
use thiserror::Error;

/// The wired gate command — self-contained at hook time: `spk lint`
/// carries the dual-format rule and the full specodelic lint, and needs
/// no `openspec` CLI, `just`, or python (those stay CI-side).
pub const GATE_COMMAND: &str = "spk lint openspec";

/// The lefthook command entry name for the gate.
pub const COMMAND_NAME: &str = "specodelic-gates";

/// Hook stage wired by this module.
pub const STAGE: &str = "pre-commit";

/// Start marker — `# ` prefix makes the line a YAML comment in any
/// position (bare `<!-- … -->` is not valid YAML).
pub const MARKER_START: &str = "# <!-- SPK:START -->";

/// End marker (same comment-prefix rule).
pub const MARKER_END: &str = "# <!-- SPK:END -->";

/// Resolve the lefthook config path: `lefthook.yml` wins over
/// `lefthook.yaml` (mirrors genesis `lefthook::config_path`, which is
/// private).
fn config_path(root: &Path) -> Option<PathBuf> {
    for name in ["lefthook.yml", "lefthook.yaml"] {
        let path = root.join(name);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

/// True when the line is a column-0 YAML key (starts with a
/// non-whitespace character other than `#`) — a section boundary.
/// (Mirrors genesis `lefthook::is_column_zero_key`, private.)
fn is_column_zero_key(line: &str) -> bool {
    let first = line.chars().next();
    matches!(first, Some(c) if c != ' ' && c != '\t' && c != '#' && c != '\n' && c != '\r')
}

/// Find the column-0 stage-key anchor and return its byte offset.
/// `Err(())` when the stage key appears only in a non-anchorable form
/// (quoted, indented) — refuse to guess (genesis D6: refuse to anchor
/// unrecognizable structure). (Mirrors genesis `lefthook::find_anchor`,
/// private.)
fn find_anchor(content: &str, stage: &str) -> Result<Option<usize>, ()> {
    let anchor = format!("{stage}:");
    let mut offset = 0usize;
    for line in content.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        if line.trim_end() == anchor && is_column_zero_key(line) {
            return Ok(Some(start));
        }
        if line.contains(stage) {
            // Present but not as a column-0 `key:` anchor — unrecognized
            // structure, refuse.
            return Err(());
        }
    }
    Ok(None)
}

/// Extract the stage's section: from the anchor line to the next
/// column-0 key or EOF. (Mirrors genesis `lefthook::section`, private.)
fn stage_section(content: &str, anchor_offset: usize) -> &str {
    let rest = &content[anchor_offset..];
    let mut end = rest.len();
    let mut offset = 0usize;
    for (i, line) in rest.split_inclusive('\n').enumerate() {
        if i == 0 {
            offset = line.len();
            continue;
        }
        if is_column_zero_key(line) {
            end = offset;
            break;
        }
        offset += line.len();
    }
    &rest[..end]
}

/// Indentation (in spaces) of the first child line of a stage section —
/// the first non-blank, non-comment line. `None` when the section has
/// no child lines at all (the caller defaults to 2).
fn children_indent(section: &str) -> Option<usize> {
    for line in section.split_inclusive('\n').skip(1) {
        let trimmed = line.trim_end();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        return Some(line.len() - line.trim_start().len());
    }
    None
}

/// Byte offset and indent of a `commands:` key line anywhere within a
/// stage section (any indent — the caller compares against the
/// children indent to classify: at-children → insert, elsewhere →
/// refuse, absent → wrapper path).
fn find_commands_key(section: &str) -> Option<(usize, usize)> {
    let mut offset = 0usize;
    for line in section.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        if line.trim() == "commands:" {
            return Some((start, line.len() - line.trim_start().len()));
        }
    }
    None
}

/// The marker-guarded gate entry, formatted at `indent` (the stage's
/// children indent + 2 — inside the `commands:` mapping).
fn wired_entry_at(indent: usize) -> String {
    let pad = " ".repeat(indent);
    format!(
        "{pad}{MARKER_START}\n{pad}{COMMAND_NAME}:\n{pad}  run: {GATE_COMMAND}\n{pad}{MARKER_END}\n"
    )
}

/// The full `commands:` wrapper (wrapper wiring case). Deliberately
/// NOT delegated to genesis `lefthook::ensure_wired`, for two verified
/// reasons: (1) its contract wraps caller content with the markers, so
/// marker-bearing content gets double-wrapped; (2) it glues the END
/// marker onto the next existing line (its own tests pin
/// `END  parallel: true`), which with comment-prefixed markers turns
/// that line into a YAML comment and silently deletes the following
/// key — found by this module's red tests. Local injection keeps every
/// marker on its own line. Still the anchored-surgery class genesis
/// excludes — upstream consolidation target:
/// `lefthook::ensure_command_wired` (specodelic-x56). Entries at
/// commands indent + 2 (self-consistent new mapping).
fn wrapper_at(indent: usize) -> String {
    let pad = " ".repeat(indent);
    format!("{pad}commands:\n{}", wired_entry_at(indent + 2))
}

/// Errors from wiring. Every variant renders as a labeled message with
/// a remediation hint (output discipline).
#[derive(Debug, Error)]
pub enum HooksError {
    /// No `lefthook.yml` / `lefthook.yaml` in the repo root — never
    /// created from scratch (genesis D4).
    #[error(
        "no supported hook framework config found in {root}: expected lefthook.yml or lefthook.yaml — create one first (see https://lefthook.dev), then re-run: spk hooks install"
    )]
    MissingConfig {
        /// Repo root that was searched.
        root: PathBuf,
    },
    /// The repo uses a hook framework this module cannot wire.
    #[error(
        "unsupported hook framework `{framework}` in {root}: {detail} — wire the gate manually: add `specodelic-gates:\n  run: spk lint openspec` under pre-commit.commands"
    )]
    UnsupportedFramework {
        /// Framework detected (`husky`, `prek`).
        framework: &'static str,
        /// Repo root.
        root: PathBuf,
        /// What was detected.
        detail: String,
    },
    /// The stage or its `commands:` key cannot be anchored — the config
    /// is left unmodified (honest_anchor).
    #[error(
        "cannot anchor the `{stage}` stage in {path}: {reason} — file left unmodified; wire the gate manually or restructure the config"
    )]
    Unanchorable {
        /// Config file that was refused.
        path: PathBuf,
        /// Stage key that could not be anchored.
        stage: &'static str,
        /// What went wrong while anchoring.
        reason: String,
    },
    /// Markers present but unbalanced — refusing to strip would leave
    /// the config in an unknown state.
    #[error(
        "unbalanced managed-block markers in {path}: found one of the SPK:START/SPK:END pair but not both — fix or remove the markers manually"
    )]
    UnbalancedMarkers {
        /// Config file with the unbalanced markers.
        path: PathBuf,
    },
    /// I/O error while reading or writing the config.
    #[error("io error at {path}: {message}")]
    Io {
        /// Path the operation touched.
        path: PathBuf,
        /// Human-readable context.
        message: String,
        /// Underlying error.
        #[source]
        source: std::io::Error,
    },
}

/// Outcome of [`install`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireOutcome {
    /// The managed block was injected (either wiring case).
    Injected,
    /// The config already contained the block — file unchanged.
    AlreadyWired,
}

/// Outcome of [`uninstall`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnwireOutcome {
    /// The managed block was removed.
    Removed,
    /// No block present — reported as a no-op success, never an error.
    NotWired,
}

/// Wire the gate into `pre-commit` in the repo at `root`, idempotently
/// and additively (spec: `spk hooks install wires the gate additively
/// into the lefthook config`).
///
/// Two-case anchor (design Decision 1, empirically grounded):
/// - stage section has `commands:` at the children indent → the gate
///   entry is inserted *inside* the existing mapping (local surgery);
/// - otherwise the full `commands:` wrapper is injected via genesis
///   `lefthook::ensure_wired` (stage missing, or empty section);
/// - unanchorable structure refuses without modification.
///
/// Returns [`WireOutcome::AlreadyWired`] without writing when the
/// markers are already present.
pub fn install(root: &Path) -> Result<WireOutcome, HooksError> {
    // Framework gate (design Decision 3): lefthook → proceed; prek or
    // husky → labeled refusal with a manual-wiring hint; none → missing
    // config (never create one — genesis D4).
    match framework(root) {
        Framework::Lefthook => {}
        Framework::Prek => {
            return Err(HooksError::UnsupportedFramework {
                framework: "prek",
                root: root.to_path_buf(),
                detail: "prek.toml detected".to_string(),
            });
        }
        Framework::Husky => {
            return Err(HooksError::UnsupportedFramework {
                framework: "husky",
                root: root.to_path_buf(),
                detail: "pre-commit hook carries the husky sigil".to_string(),
            });
        }
        Framework::None => {
            return Err(HooksError::MissingConfig {
                root: root.to_path_buf(),
            });
        }
    }
    let path = config_path(root).ok_or_else(|| HooksError::MissingConfig {
        root: root.to_path_buf(),
    })?;
    let text = std::fs::read_to_string(&path).map_err(|source| HooksError::Io {
        path: path.clone(),
        message: "failed to read lefthook config".to_string(),
        source,
    })?;

    // Idempotence: markers already present → no write. Exactly one
    // marker → unknown state, refuse rather than guess.
    let has_start = text.contains(MARKER_START);
    let has_end = text.contains(MARKER_END);
    if has_start && has_end {
        return Ok(WireOutcome::AlreadyWired);
    }
    if has_start || has_end {
        return Err(HooksError::UnbalancedMarkers { path });
    }

    let stage_anchor = find_anchor(&text, STAGE).map_err(|()| HooksError::Unanchorable {
        path: path.clone(),
        stage: "pre-commit",
        reason: "stage key is quoted or otherwise not anchorable at column 0".to_string(),
    })?;

    if let Some(anchor) = stage_anchor {
        let section = stage_section(&text, anchor);
        match (children_indent(section), find_commands_key(section)) {
            // No child lines at all (empty stage) → wrapper path; the
            // wrapper goes directly after the anchor.
            (None, _) => {
                return wire_wrapper(&path, &text, Some(anchor), 2).map(|_| WireOutcome::Injected);
            }
            // `commands:` at the children indent → the in-mapping insert.
            (Some(indent), Some((offset, cmd_indent))) if cmd_indent == indent => {
                return insert_inside_commands(&path, &text, anchor + offset, cmd_indent);
            }
            // `commands:` present but at another indent — unrecognized
            // structure, refuse (honest_anchor).
            (Some(_), Some((_, cmd_indent))) => {
                return Err(HooksError::Unanchorable {
                    path,
                    stage: "pre-commit",
                    reason: format!(
                        "commands key is at indent {cmd_indent}, which differs from the stage's children indentation — unrecognized structure"
                    ),
                });
            }
            // Stage has children but no `commands:` key → wrapper path at
            // the stage's own children indent.
            (Some(indent), None) => {
                return wire_wrapper(&path, &text, Some(anchor), indent)
                    .map(|_| WireOutcome::Injected);
            }
        }
    }

    // Stage section missing entirely → wrapper path appended at EOF
    // (children indent defaults to 2).
    wire_wrapper(&path, &text, None, 2).map(|_| WireOutcome::Injected)
}

/// Wrapper wiring case (stage missing, empty stage, or stage without a
/// `commands:` key): insert `commands:` + the marker-guarded entry at
/// `indent` — after the stage anchor line when present, else appended
/// as a new stage section. Every marker on its own line; purely
/// additive; byte-format edges (no trailing newline) handled.
fn wire_wrapper(
    path: &Path,
    text: &str,
    anchor: Option<usize>,
    indent: usize,
) -> Result<WireOutcome, HooksError> {
    let insertion = wrapper_at(indent);
    let mut updated = String::with_capacity(text.len() + insertion.len() + 16);
    match anchor {
        Some(offset) => {
            // After the anchor line.
            let line_end = text[offset..]
                .find('\n')
                .map_or(text.len(), |nl| offset + nl + 1);
            updated.push_str(&text[..line_end]);
            if line_end == text.len() && !text.ends_with('\n') {
                updated.push('\n');
            }
            updated.push_str(&insertion);
            updated.push_str(&text[line_end..]);
        }
        None => {
            // Append the missing stage section at EOF.
            updated.push_str(text);
            if !updated.ends_with('\n') {
                updated.push('\n');
            }
            updated.push_str(STAGE);
            updated.push_str(":\n");
            updated.push_str(&insertion);
        }
    }
    std::fs::write(path, updated).map_err(|source| HooksError::Io {
        path: path.to_path_buf(),
        message: "failed to write lefthook config".to_string(),
        source,
    })?;
    Ok(WireOutcome::Injected)
}

/// In-mapping wiring case: insert the marker-guarded entry lines
/// directly after the `commands:` line (byte offset `commands_end` in
/// `text`). The entry indent is inferred from the mapping's existing
/// children — the first line after `commands:` indented deeper than
/// `commands_indent` — because YAML per-level indent is config-dependent
/// (a 4-space config nests entries at commands+4, not +2). Default:
/// commands_indent + 2 when the mapping is empty. Purely additive:
/// everything before and after is byte-identical.
fn insert_inside_commands(
    path: &Path,
    text: &str,
    commands_end: usize,
    commands_indent: usize,
) -> Result<WireOutcome, HooksError> {
    // Find the end of the commands line; handle a config whose last line
    // has no trailing newline.
    let line_end = text[commands_end..]
        .find('\n')
        .map_or(text.len(), |nl| commands_end + nl + 1);

    // Entry indent = first existing entry's indent (deeper than the
    // commands key, before the mapping closes); +2 default when empty.
    let mut entry_indent = commands_indent + 2;
    let mut offset = line_end;
    for line in text[line_end..].split_inclusive('\n') {
        let trimmed = line.trim_end();
        let indent = line.len() - line.trim_start().len();
        let is_child = indent > commands_indent
            && !trimmed.is_empty()
            && !trimmed.trim_start().starts_with('#');
        if is_child {
            entry_indent = indent;
            break;
        }
        if !trimmed.is_empty() && indent <= commands_indent {
            break; // mapping closed before any entry
        }
        offset += line.len();
    }
    let _ = offset;

    let mut updated = String::with_capacity(text.len() + 128);
    updated.push_str(&text[..line_end]);
    if line_end == text.len() && !text.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(&wired_entry_at(entry_indent));
    updated.push_str(&text[line_end..]);
    std::fs::write(path, updated).map_err(|source| HooksError::Io {
        path: path.to_path_buf(),
        message: "failed to write lefthook config".to_string(),
        source,
    })?;
    Ok(WireOutcome::Injected)
}

/// Remove the managed block from the lefthook config at `root`
/// (spec: `spk hooks uninstall strips only the managed block`).
///
/// Strips exactly the lines between the markers (inclusive); the rest
/// of the file is byte-identical. A stage section that install itself
/// appended is left behind as an empty stage section (documented
/// residue). No block present → [`UnwireOutcome::NotWired`], never an
/// error.
pub fn uninstall(root: &Path) -> Result<UnwireOutcome, HooksError> {
    // No config → no block can exist → reported no-op (never an error;
    // the framework gate is an install-time concern).
    let Some(path) = config_path(root) else {
        return Ok(UnwireOutcome::NotWired);
    };
    let text = std::fs::read_to_string(&path).map_err(|source| HooksError::Io {
        path: path.clone(),
        message: "failed to read lefthook config".to_string(),
        source,
    })?;

    // Line-based marker scan: markers may carry leading indent (the
    // in-mapping case) and CR (CRLF configs) — compare trimmed lines.
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let start_idx = lines.iter().position(|l| l.trim() == MARKER_START);
    let end_idx = lines.iter().position(|l| l.trim() == MARKER_END);
    let (Some(start_idx), Some(end_idx)) = (start_idx, end_idx) else {
        if start_idx.is_some() || end_idx.is_some() {
            return Err(HooksError::UnbalancedMarkers { path });
        }
        return Ok(UnwireOutcome::NotWired);
    };
    if start_idx > end_idx {
        return Err(HooksError::UnbalancedMarkers { path });
    }
    let mut updated = String::with_capacity(text.len());
    for (i, line) in lines.iter().enumerate() {
        if !(start_idx..=end_idx).contains(&i) {
            updated.push_str(line);
        }
    }
    std::fs::write(&path, updated).map_err(|source| HooksError::Io {
        path,
        message: "failed to write lefthook config".to_string(),
        source,
    })?;
    Ok(UnwireOutcome::Removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use genesis::fixture::Fixture;

    const TWO_SPACE: &str = "pre-commit:\n  commands:\n    sibling-blockers:\n      run: scripts/guards/sibling-blockers.sh .\npre-push:\n  commands:\n    sibling-blockers:\n      run: scripts/guards/sibling-blockers.sh .\n";

    fn fixture_with_config(contents: &str) -> Fixture {
        let fixture = Fixture::new().build().unwrap();
        std::fs::write(fixture.root().join("lefthook.yml"), contents).unwrap();
        fixture
    }

    fn wired_entry_at(indent: usize) -> String {
        let pad = " ".repeat(indent);
        format!(
            "{pad}# <!-- SPK:START -->\n{pad}specodelic-gates:\n{pad}  run: spk lint openspec\n{pad}# <!-- SPK:END -->\n"
        )
    }

    // -- install: entry inside an existing commands mapping (2-space) ----

    #[test]
    fn install_inserts_inside_existing_commands_mapping() {
        let fixture = fixture_with_config(TWO_SPACE);
        let outcome = install(fixture.root()).unwrap();
        assert_eq!(outcome, WireOutcome::Injected);
        let after = std::fs::read_to_string(fixture.root().join("lefthook.yml")).unwrap();
        // Entry is inside the pre-commit commands mapping, at entry indent.
        assert!(
            after.contains(&format!(
                "  commands:\n{}    sibling-blockers:",
                wired_entry_at(4)
            )),
            "entry must sit between `commands:` and the existing entries:\n{after}"
        );
        // No duplicate `commands:` key — the failure that kills the chain.
        assert_eq!(after.matches("commands:").count(), 2);
        // Pre-existing entries and their order are unchanged.
        assert!(after.contains(
            "    sibling-blockers:\n      run: scripts/guards/sibling-blockers.sh .\npre-push:"
        ));
    }

    // -- install: idempotence ---------------------------------------------

    #[test]
    fn install_is_idempotent() {
        let fixture = fixture_with_config(TWO_SPACE);
        install(fixture.root()).unwrap();
        let once = std::fs::read_to_string(fixture.root().join("lefthook.yml")).unwrap();
        let outcome = install(fixture.root()).unwrap();
        assert_eq!(outcome, WireOutcome::AlreadyWired);
        let twice = std::fs::read_to_string(fixture.root().join("lefthook.yml")).unwrap();
        assert_eq!(once, twice, "second install must not change the file");
        assert_eq!(once.matches("SPK:START").count(), 1);
    }

    // -- install: 4-space config (children-indent inference) --------------

    #[test]
    fn install_matches_children_indent_of_the_stage() {
        // 4-space children: injecting a 2-space wrapper would produce
        // mixed-indent keys in one mapping (verified: lefthook fails to
        // parse). The entry must be formatted at the stage's own indent.
        let config = "pre-commit:\n    commands:\n        lint:\n            run: lint\n";
        let fixture = fixture_with_config(config);
        install(fixture.root()).unwrap();
        let after = std::fs::read_to_string(fixture.root().join("lefthook.yml")).unwrap();
        assert!(
            after.contains(&format!(
                "    commands:\n{}        lint:",
                wired_entry_at(8)
            )),
            "entry must use the existing entries' indent (8 — YAML per-level indent is config-dependent):\n{after}"
        );
        assert_eq!(after.matches("commands:").count(), 1);
    }

    // -- install: no commands key in the stage (wrapper path) --------------

    #[test]
    fn install_uses_wrapper_when_stage_has_no_commands() {
        // Stage exists with another child key but no commands mapping.
        let config =
            "pre-commit:\n  parallel: true\npre-push:\n  commands:\n    test:\n      run: test\n";
        let fixture = fixture_with_config(config);
        let outcome = install(fixture.root()).unwrap();
        assert_eq!(outcome, WireOutcome::Injected);
        let after = std::fs::read_to_string(fixture.root().join("lefthook.yml")).unwrap();
        // Wrapper case: `commands:` + the marker-guarded entry injected
        // directly after the stage anchor, at the stage's children indent
        // (2), entry at +2; markers on their own lines (never glued onto
        // existing content — the genesis ensure_wired defect).
        assert!(
            after.contains(
                "pre-commit:\n  commands:\n    # <!-- SPK:START -->\n    specodelic-gates:\n      run: spk lint openspec\n    # <!-- SPK:END -->\n"
            ),
            "wrapper case must inject a 2-space commands wrapper after the anchor:\n{after}"
        );
        assert!(
            after.contains("  parallel: true"),
            "existing child must survive"
        );
    }

    #[test]
    fn install_appends_stage_section_when_missing() {
        // No pre-commit section at all — genesis ensure_wired appends one.
        let config = "pre-push:\n  commands:\n    test:\n      run: test\n";
        let fixture = fixture_with_config(config);
        let outcome = install(fixture.root()).unwrap();
        assert_eq!(outcome, WireOutcome::Injected);
        let after = std::fs::read_to_string(fixture.root().join("lefthook.yml")).unwrap();
        assert!(
            after.contains("pre-commit:"),
            "stage section appended:\n{after}"
        );
        assert!(after.contains("specodelic-gates:"), "gate wired:\n{after}");
        assert!(
            after.contains("    test:\n      run: test\n"),
            "pre-push intact"
        );
    }

    // -- install: refusal cases (honest_anchor) ----------------------------

    #[test]
    fn install_refuses_without_modification_when_stage_key_unanchorable() {
        let config = "\"pre-commit\":\n  commands:\n    a:\n      run: a\n";
        let fixture = fixture_with_config(config);
        let err = install(fixture.root()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("anchor"), "labeled anchor error: {msg}");
        let after = std::fs::read_to_string(fixture.root().join("lefthook.yml")).unwrap();
        assert_eq!(after, config, "file must be untouched");
    }

    #[test]
    fn install_refuses_mis_indented_commands_key() {
        // commands: present but at an indent that differs from the stage's
        // children — unrecognized structure, refuse.
        let config = "pre-commit:\n  other: x\n    commands:\n      a:\n        run: a\n";
        let fixture = fixture_with_config(config);
        let err = install(fixture.root()).unwrap_err();
        assert!(err.to_string().contains("anchor"), "labeled error: {err}");
        let after = std::fs::read_to_string(fixture.root().join("lefthook.yml")).unwrap();
        assert_eq!(after, config, "file must be untouched");
    }

    #[test]
    fn install_errors_on_missing_config_and_never_creates_one() {
        let fixture = Fixture::new().build().unwrap();
        let err = install(fixture.root()).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("no supported hook framework config"),
            "genesis-consistent message: {msg}"
        );
        assert!(msg.contains("create one first"), "remediation hint: {msg}");
        assert!(
            !fixture.root().join("lefthook.yml").exists(),
            "never created"
        );
    }

    // -- install: byte-format edge fixtures (design EDGE-003 matrix) -------

    #[test]
    fn install_handles_config_without_trailing_newline() {
        let config = "pre-commit:\n  commands:\n    a:\n      run: a"; // no \n
        let fixture = fixture_with_config(config);
        install(fixture.root()).unwrap();
        let after = std::fs::read_to_string(fixture.root().join("lefthook.yml")).unwrap();
        assert!(
            after.contains(&format!("  commands:\n{}    a:", wired_entry_at(4))),
            "entry inserted inside mapping:\n{after}"
        );
        assert_eq!(after.matches("commands:").count(), 1);
    }

    #[test]
    fn install_handles_crlf_config() {
        let config = "pre-commit:\r\n  commands:\r\n    a:\r\n      run: a\r\n";
        let fixture = fixture_with_config(config);
        install(fixture.root()).unwrap();
        let after = std::fs::read_to_string(fixture.root().join("lefthook.yml")).unwrap();
        // Existing bytes unchanged, entry inserted, no duplicate key.
        assert!(
            after.starts_with("pre-commit:\r\n  commands:\r\n"),
            "prefix intact:\n{after:?}"
        );
        assert!(
            after.contains("specodelic-gates:"),
            "entry wired:\n{after:?}"
        );
        assert_eq!(after.matches("commands:").count(), 1);
    }

    // -- install: framework gates ------------------------------------------

    #[test]
    fn install_refuses_prek_framework() {
        let fixture = Fixture::new().build().unwrap();
        std::fs::write(fixture.root().join("prek.toml"), "").unwrap();
        let err = install(fixture.root()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("prek"), "framework named: {msg}");
        assert!(msg.contains("manually"), "manual-wiring hint: {msg}");
    }

    #[test]
    fn install_refuses_husky_framework() {
        let fixture = Fixture::new().build().unwrap();
        let git_hooks_dir = fixture.root().join(".git").join("hooks");
        std::fs::create_dir_all(&git_hooks_dir).unwrap();
        std::fs::write(
            git_hooks_dir.join("pre-commit"),
            "#!/bin/sh\n. \"$(dirname -- \"$0\")\"/_/husky.sh\n",
        )
        .unwrap();
        let err = install(fixture.root()).unwrap_err();
        assert!(err.to_string().contains("husky"), "framework named: {err}");
    }

    // -- uninstall ----------------------------------------------------------

    #[test]
    fn uninstall_strips_only_the_marked_block() {
        let fixture = fixture_with_config(TWO_SPACE);
        install(fixture.root()).unwrap();
        let wired = std::fs::read_to_string(fixture.root().join("lefthook.yml")).unwrap();
        let outcome = uninstall(fixture.root()).unwrap();
        assert_eq!(outcome, UnwireOutcome::Removed);
        let after = std::fs::read_to_string(fixture.root().join("lefthook.yml")).unwrap();
        // The rest is byte-identical to the pre-install config: the block
        // lines removed, nothing else moved.
        let restored = wired
            .lines()
            .filter(|l| {
                let t = l.trim();
                !(t.contains("SPK:START")
                    || t.contains("SPK:END")
                    || t.contains("specodelic-gates:")
                    || t.contains("run: spk lint openspec"))
            })
            .collect::<Vec<_>>()
            .join("\n");
        let filtered = after.lines().collect::<Vec<_>>().join("\n");
        assert_eq!(filtered, restored, "only block lines removed:\n{after}");
        assert_eq!(after.matches("commands:").count(), 2);
    }

    #[test]
    fn uninstall_leaves_appended_stage_as_documented_residue() {
        // Install appended a missing pre-commit section; uninstall leaves
        // the (now empty) section behind — documented, not hidden.
        let config = "pre-push:\n  commands:\n    test:\n      run: test\n";
        let fixture = fixture_with_config(config);
        install(fixture.root()).unwrap();
        uninstall(fixture.root()).unwrap();
        let after = std::fs::read_to_string(fixture.root().join("lefthook.yml")).unwrap();
        assert!(
            after.contains("pre-commit:"),
            "empty stage section remains:\n{after}"
        );
        assert!(!after.contains("specodelic-gates:"), "gate gone:\n{after}");
    }

    #[test]
    fn uninstall_on_unwired_repo_is_a_no_op() {
        let fixture = fixture_with_config(TWO_SPACE);
        let outcome = uninstall(fixture.root()).unwrap();
        assert_eq!(outcome, UnwireOutcome::NotWired);
        let after = std::fs::read_to_string(fixture.root().join("lefthook.yml")).unwrap();
        assert_eq!(after, TWO_SPACE, "config unchanged");
    }

    #[test]
    fn uninstall_on_missing_config_is_a_no_op() {
        let fixture = Fixture::new().build().unwrap();
        let outcome = uninstall(fixture.root()).unwrap();
        assert_eq!(outcome, UnwireOutcome::NotWired);
    }

    #[test]
    fn uninstall_refuses_unbalanced_markers() {
        let config = format!(
            "pre-commit:\n  commands:\n{}\n    a:\n      run: a\n",
            wired_entry_at(4).lines().next().unwrap()
        );
        let fixture = fixture_with_config(&config);
        let err = uninstall(fixture.root()).unwrap_err();
        assert!(
            err.to_string().contains("unbalanced"),
            "labeled error: {err}"
        );
    }

    // -- no foreign writes (spec: never claims foreign ownership) -----------

    #[test]
    fn install_never_touches_hook_files_or_hooks_path() {
        // A bd-style repo: core.hooksPath → .beads/hooks, shims chained.
        let fixture = Fixture::new().with_git_init().build().unwrap();
        let beads_dir = fixture.root().join(".beads").join("hooks");
        std::fs::create_dir_all(&beads_dir).unwrap();
        std::fs::write(
            beads_dir.join("pre-commit"),
            "#!/bin/sh\nexec lefthook \"$@\"\n",
        )
        .unwrap();
        std::fs::write(fixture.root().join("lefthook.yml"), TWO_SPACE).unwrap();
        let shim_before = std::fs::read(beads_dir.join("pre-commit")).unwrap();
        std::process::Command::new("git")
            .args([
                "-C",
                fixture.root().to_str().unwrap(),
                "config",
                "--local",
                "core.hooksPath",
                ".beads/hooks",
            ])
            .output()
            .unwrap();

        install(fixture.root()).unwrap();

        let shim_after = std::fs::read(beads_dir.join("pre-commit")).unwrap();
        assert_eq!(shim_before, shim_after, "beads shim byte-identical");
        assert!(
            !fixture
                .root()
                .join(".git")
                .join("hooks")
                .join("pre-commit")
                .exists(),
            "no .git/hooks write"
        );
        let hooks_path = std::process::Command::new("git")
            .args([
                "-C",
                fixture.root().to_str().unwrap(),
                "config",
                "--local",
                "core.hooksPath",
            ])
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&hooks_path.stdout).trim(),
            ".beads/hooks",
            "core.hooksPath untouched"
        );
    }
}
