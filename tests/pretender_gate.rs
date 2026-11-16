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

/// Acceptance (specodelic-5ae1): a fetched-but-untracked vendored asset must
/// not red the gate. `just docs-mermaid` writes vendor/mermaid.min.js (one
/// minified, deeply nested line) and the gate has to absorb it.
///
/// Why this fails today: `[thresholds.vendor]` in pretender.toml carries only
/// `file_lines_max`; the vendor role therefore still resolves
/// cyclomatic/cognitive/abc/nesting/params from the global app block. The
/// bundle's minified function blows past the project ratchet (measured: 88
/// function-metric findings against the real mermaid.min.js). The asset is a
/// single line, so `file_lines` (1) is never the finding — the function body
/// is.
#[test]
fn vendor_minified_asset_passes_gate() {
    if !pretender_available() {
        eprintln!("skipping: pretender not on PATH");
        return;
    }
    let dir = std::env::temp_dir().join(format!("pretender-vendor-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    write_or_panic(fs::create_dir_all(dir.join("vendor")), "create vendor dir");

    // The real gate config, verbatim — thresholds are a shrink-only ratchet,
    // so the fixture must measure against today's numbers, not a retyped copy.
    let Ok(config) = fs::read_to_string(Path::new(manifest_dir()).join("pretender.toml")) else {
        panic!("repo pretender.toml must be readable");
    };
    write_or_panic(
        fs::write(dir.join("pretender.toml"), config),
        "write config",
    );

    // Synthetic stand-in for the fetched bundle: one line, one deeply nested
    // function. The long literal puts >4000 chars on that line while keeping
    // the line count at 1, so the file level never trips; only a function
    // metric can (31 branches vs the global cyclomatic_max of 20).
    let mut asset = String::from("function t(e){");
    for i in 0..30 {
        asset.push_str(&format!("if(e[{i}]){{"));
    }
    asset.push('"');
    asset.push_str(&"a".repeat(4200));
    asset.push_str("\";}");
    asset.push_str(&"}".repeat(30));
    asset.push_str("}\n");
    write_or_panic(
        fs::write(dir.join("vendor/mermaid.min.js"), &asset),
        "write vendor asset",
    );

    // Default walk from the fixture root: the config's relative vendor
    // excludes already cover this shape (specodelic-m1rk), so it must pass
    // and must keep passing if the exclude list is ever reworked.
    let (code, out) = run_pretender(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "default walk must not lint vendored assets; output:\n{out}"
    );

    // Then the absolute checkout-path shape — how the bundle actually leaked:
    // the config's vendor excludes are relative-anchored ("vendor/**",
    // "./vendor/**"), so absolute walk entries escape them.
    let fixture = dir.display().to_string();
    let (code, out) = run_pretender(&dir, &["check", fixture.as_str()]);
    assert_eq!(
        code, 0,
        "a present-but-untracked vendored asset must pass the gate; vendor role output:\n{out}"
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

/// Assert a scratch-fixture filesystem step without adding unwraps to the
/// test-role metric budget (pretender's test thresholds cap unwraps at 5).
fn write_or_panic(res: std::io::Result<()>, what: &str) {
    if let Err(e) = res {
        panic!("{what} failed: {e}");
    }
}
