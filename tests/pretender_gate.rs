//! Tests for the pretender structural-quality hard gate (specodelic-50r).
//!
//! The gate is `pretender check` in gate mode, configured by pretender.toml
//! (thresholds are a shrink-only ratchet: entries only move DOWN). The gate
//! runs via lefthook.yml pre-commit (the .beads/hooks shim chain — pretender
//! never claims core.hooksPath or writes .git/hooks) and `just ci` /
//! `just pretender-check`.
//!
//! pretender must be on PATH (the gate recipes already require it, so this
//! adds no new dependency to CI).

use std::fs;
use std::path::Path;
use std::process::Command;

fn manifest_dir() -> &'static str {
    env!("CARGO_MANIFEST_DIR")
}

fn run_pretender(dir: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new("pretender")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("pretender should be on PATH (installed by the gate recipes)");
    (
        out.status.code().unwrap_or(-1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

fn pretender_available() -> bool {
    Command::new("pretender").arg("--version").output().is_ok()
}

/// Acceptance (specodelic-50r): HEAD passes the gate.
#[test]
fn head_passes_gate() {
    if !pretender_available() {
        eprintln!("skipping: pretender not on PATH");
        return;
    }
    let repo = Path::new(manifest_dir());
    let (code, out) = run_pretender(repo, &["check"]);
    assert_eq!(code, 0, "HEAD must pass the pretender gate; output:\n{out}");
}

/// Acceptance (specodelic-50r): a seeded violation in a scratch fixture makes
/// the gate exit nonzero (gate mode promotes threshold breaches to failure —
/// verified empirically at authoring time; this test locks the semantics in).
#[test]
fn seeded_violation_fails_gate() {
    if !pretender_available() {
        eprintln!("skipping: pretender not on PATH");
        return;
    }
    let dir = std::env::temp_dir().join(format!("pretender-gate-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).unwrap();

    // A function far beyond any plausible threshold: deep nesting.
    let mut src = String::from("fn seeded_violation(x: i32) -> i32 {\n    let mut y = 0;\n");
    for i in 0..25 {
        src.push_str(&format!("    if x > {i} {{ y += {i}; }}\n"));
    }
    src.push_str("    y\n}\n");
    fs::write(dir.join("src/violation.rs"), &src).unwrap();

    // Gate config with a threshold the seeded function breaches.
    fs::write(
        dir.join("pretender.toml"),
        "[pretender]\nmode = \"gate\"\nlanguages = [\"rust\"]\n\n[thresholds]\ncognitive_max = 3\n",
    )
    .unwrap();

    let (code, out) = run_pretender(&dir, &["check"]);
    assert_ne!(
        code, 0,
        "seeded violation must fail the gate; output:\n{out}"
    );
    assert!(
        out.contains("VIOLATION"),
        "failure must carry a VIOLATION finding; output:\n{out}"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Constraint (AGENTS.md sibling-tool blockers): the gate config must never
/// direct pretender to install git hooks — the toml carries no hook wiring,
/// and `pretender hooks install` is forbidden in this repo (beads owns
/// core.hooksPath). Guarded here so the toml can't silently grow hook config.
#[test]
fn gate_config_never_installs_hooks() {
    let toml = fs::read_to_string(Path::new(manifest_dir()).join("pretender.toml")).unwrap();
    let configured = toml
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !configured.to_lowercase().contains("hook"),
        "pretender.toml must not configure hook installation (beads owns core.hooksPath); offending line(s): {configured}"
    );
}
