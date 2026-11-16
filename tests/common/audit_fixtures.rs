//! Shared CLI fixtures for the `audit` integration tests
//! (specodelic-l6md): both `tests/audit_oracle_rows.rs` and
//! `tests/audit_binds_nothing.rs` drive the real binary over a
//! compiled fixture spec, so the binary runner, the compile step, and
//! the permitted-scenario row live here once — the two targets cannot
//! drift apart.
//!
//! The SPEC fixtures stay in their owning test files: the oracle-row
//! and vacuity specs differ in intent and are not interchangeable.

use assert_cmd::Command;

use super::UnwrapPanic;

/// One permitted scenario — the oracle row is `current`; advisory rows
/// (vacuity) ride beside it without failing the run.
pub const CONFIRM_HAPPY: &str = r#"{"id": "confirm-happy", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"status": "confirmed"}}]}"#;

/// The CLI under test.
pub fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap_p()
}

/// Compile the fixture spec into a temp out-dir (the reused gate
/// requires current artifacts — audit never re-compiles, like conform).
pub fn compiled_out_dir(dir: &std::path::Path, spec: &std::path::Path) -> String {
    let out_dir = dir.join("out");
    spk()
        .args([
            "compile",
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out_dir.to_str().unwrap_p(),
        ])
        .assert()
        .success();
    out_dir.to_str().unwrap_p().to_string()
}

/// Run `audit` over `corpus` with `extra` flags appended, returning the
/// exit code and the raw streams — view selection (`--json`/`--human`)
/// stays with the caller.
pub fn audit_raw(
    spec: &std::path::Path,
    corpus: &std::path::Path,
    extra: &[&str],
) -> (Option<i32>, Vec<u8>, Vec<u8>) {
    let dir = spec.parent().unwrap_p().to_path_buf();
    let out_dir = compiled_out_dir(dir.as_path(), spec);
    let out = spk()
        .args([
            "audit",
            spec.to_str().unwrap_p(),
            "--oracle",
            corpus.to_str().unwrap_p(),
            "--out-dir",
            out_dir.as_str(),
        ])
        .args(extra)
        .output()
        .unwrap_p();
    (out.status.code(), out.stdout, out.stderr)
}

/// Run `audit --json` over `corpus` with `extra` flags appended and
/// parse the envelope (refusal paths still emit one, so a parse failure
/// is a fixture/wiring bug, never an expected mode).
pub fn audit_json(
    spec: &std::path::Path,
    corpus: &std::path::Path,
    extra: &[&str],
) -> (Option<i32>, serde_json::Value) {
    let dir = spec.parent().unwrap_p().to_path_buf();
    let out_dir = compiled_out_dir(dir.as_path(), spec);
    let out = spk()
        .args([
            "audit",
            spec.to_str().unwrap_p(),
            "--oracle",
            corpus.to_str().unwrap_p(),
            "--out-dir",
            out_dir.as_str(),
            "--json",
        ])
        .args(extra)
        .output()
        .unwrap_p();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "audit emits a JSON envelope ({e}): {}",
            String::from_utf8_lossy(&out.stdout)
        )
    });
    (out.status.code(), json)
}
