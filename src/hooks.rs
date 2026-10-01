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
//! Wiring rides `genesis::git_hooks` entirely: `framework()` gates the
//! config type; marker conventions are a `managed_block::BlockDef`
//! with custom comment-prefixed markers (bare `<!-- … -->` is not
//! valid YAML); the two-case anchor (in-mapping insert / wrapper) is
//! `lefthook::ensure_command_wired` — upstreamed from this module's
//! donor implementation (genesis-au8; beads specodelic-x56), so no
//! bespoke string surgery remains here. Indentation is never assumed:
//! genesis infers it from the stage's first child key line (YAML
//! allows any consistent indent; mixed indent within one mapping fails
//! the same way).

use std::path::{Path, PathBuf};

use genesis::git_hooks::lefthook::{self, Stage};
use genesis::git_hooks::{Framework, framework};
use genesis::managed_block::BlockDef;
use thiserror::Error;

/// The wired gate command — self-contained at hook time: `spk lint`
/// carries the dual-format rule and the full specodelic lint, and needs
/// no `openspec` CLI, `just`, or python (those stay CI-side).
pub const GATE_COMMAND: &str = "spk lint openspec";

/// The lefthook command entry name for the gate.
pub const COMMAND_NAME: &str = "specodelic-gates";

/// Hook stage wired by this module (reported in envelopes; the actual
/// wiring passes [`lefthook::Stage::PreCommit`] to genesis).
pub const STAGE: &str = "pre-commit";

/// Managed-block name (marker identity, envelope payloads).
pub const BLOCK_NAME: &str = "SPK";

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
/// The two-case anchor is genesis `lefthook::ensure_command_wired`
/// (upstreamed from this module's donor implementation — genesis-au8,
/// beads specodelic-x56):
/// - stage section has `commands:` at the children indent → the gate
///   entry is inserted *inside* the existing mapping at the mapping's
///   own entry indent;
/// - otherwise the full `commands:` wrapper is injected (stage missing,
///   empty, or without a `commands:` key);
/// - unanchorable structure refuses without modification.
///
/// The framework gate stays local (design Decision 3) so prek/husky
/// repos get the labeled refusal before genesis is even consulted.
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
    let block = BlockDef::with_markers(BLOCK_NAME, MARKER_START, MARKER_END);
    lefthook::ensure_command_wired(root, Stage::PreCommit, COMMAND_NAME, GATE_COMMAND, &block)
        .map(|outcome| match outcome {
            lefthook::WiredOutcome::Injected => WireOutcome::Injected,
            lefthook::WiredOutcome::AlreadyWired => WireOutcome::AlreadyWired,
        })
        .map_err(map_genesis_error)
}

/// Map a genesis [`lefthook::GitHooksError`] onto this module's
/// [`HooksError`], preserving the labeled message + remediation-hint
/// discipline (genesis messages lack the module's manual-wiring hints).
fn map_genesis_error(err: genesis::git_hooks::GitHooksError) -> HooksError {
    use genesis::git_hooks::GitHooksError as G;
    match err {
        G::MissingLefthookConfig { root } => HooksError::MissingConfig { root },
        G::UnbalancedLefthookMarkers { path, .. } => HooksError::UnbalancedMarkers { path },
        G::UnanchorableLefthookConfig {
            path,
            stage: _,
            message,
        } => HooksError::Unanchorable {
            path,
            stage: "pre-commit",
            reason: message,
        },
        G::Io {
            path,
            message,
            source,
        } => HooksError::Io {
            path,
            message,
            source,
        },
        other => HooksError::Unanchorable {
            path: PathBuf::from("lefthook.yml"),
            stage: "pre-commit",
            reason: format!("unexpected genesis error: {other}"),
        },
    }
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

    // -- golden round-trip: byte-identical wiring output (specodelic-x56
    //    MUST) — pins the wired bytes so the genesis migration cannot
    //    change behavior.

    #[test]
    fn golden_install_uninstall_round_trip_is_byte_identical_2_space() {
        let fixture = fixture_with_config(TWO_SPACE);
        let wired = "pre-commit:\n  commands:\n    # <!-- SPK:START -->\n    specodelic-gates:\n      run: spk lint openspec\n    # <!-- SPK:END -->\n    sibling-blockers:\n      run: scripts/guards/sibling-blockers.sh .\npre-push:\n  commands:\n    sibling-blockers:\n      run: scripts/guards/sibling-blockers.sh .\n";
        assert_eq!(install(fixture.root()).unwrap(), WireOutcome::Injected);
        let after = std::fs::read_to_string(fixture.root().join("lefthook.yml")).unwrap();
        assert_eq!(after, wired, "2-space golden bytes after install");
        assert_eq!(uninstall(fixture.root()).unwrap(), UnwireOutcome::Removed);
        let after = std::fs::read_to_string(fixture.root().join("lefthook.yml")).unwrap();
        assert_eq!(after, TWO_SPACE, "2-space round-trip restores the original");
    }

    #[test]
    fn golden_install_uninstall_round_trip_is_byte_identical_4_space() {
        let config = "pre-commit:\n    commands:\n        lint:\n            run: lint\n";
        let fixture = fixture_with_config(config);
        let wired = "pre-commit:\n    commands:\n        # <!-- SPK:START -->\n        specodelic-gates:\n          run: spk lint openspec\n        # <!-- SPK:END -->\n        lint:\n            run: lint\n";
        assert_eq!(install(fixture.root()).unwrap(), WireOutcome::Injected);
        let after = std_fs_read(fixture.root());
        assert_eq!(after, wired, "4-space golden bytes after install");
        assert_eq!(uninstall(fixture.root()).unwrap(), UnwireOutcome::Removed);
        let after = std_fs_read(fixture.root());
        assert_eq!(after, config, "4-space round-trip restores the original");
    }

    /// Read the fixture's lefthook.yml (test helper).
    fn std_fs_read(root: &Path) -> String {
        std::fs::read_to_string(root.join("lefthook.yml")).unwrap()
    }

    #[test]
    fn install_delegates_to_genesis_ensure_command_wired() {
        // The local two-case surgery is gone: install must route through
        // genesis::git_hooks::lefthook::ensure_command_wired.
        let config = "pre-commit:\n  commands:\n    a:\n      run: a\n";
        let fixture = fixture_with_config(config);
        install(fixture.root()).unwrap();
        let after = std_fs_read(fixture.root());
        // Byte-identical to the donor output the local code produced.
        let genesis_wired = "pre-commit:\n  commands:\n    # <!-- SPK:START -->\n    specodelic-gates:\n      run: spk lint openspec\n    # <!-- SPK:END -->\n    a:\n      run: a\n";
        assert_eq!(
            after, genesis_wired,
            "genesis wiring bytes match the donor output"
        );
    }

    #[test]
    fn install_refuses_husky_framework() {
        // Hermetic (genesis >= 0.8.2 resolves `core.hooksPath` across all
        // config scopes): a real git repo whose LOCAL config pins an empty
        // hooksPath, so detection falls back to the fixture's .git/hooks
        // instead of any developer-global hooksPath on the host (e.g. a
        // lefthook shim). Empty hooksPath = git disables hooks, which
        // resolve_hooks_dir reports as Disabled and falls back to
        // the default .git/hooks.
        let fixture = Fixture::new().with_git_init().build().unwrap();
        let git_dir = fixture.root().join(".git");
        let git_hooks_dir = git_dir.join("hooks");
        std::fs::create_dir_all(&git_hooks_dir).unwrap();
        std::fs::write(
            git_hooks_dir.join("pre-commit"),
            "#!/bin/sh\n. \"$(dirname -- \"$0\")\"/_/husky.sh\n",
        )
        .unwrap();
        use std::io::Write as _;
        let mut config = std::fs::OpenOptions::new()
            .append(true)
            .open(git_dir.join("config"))
            .unwrap();
        writeln!(config, "[core]\n\thooksPath =").unwrap();
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
