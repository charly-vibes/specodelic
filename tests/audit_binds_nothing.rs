//! Slice-3 CLI-level pin for `audit` (openspec/changes/add-evidence-audit,
//! task 3.1 red): the `binds_nothing` vacuity advisory rides the real
//! binary end-to-end — a spec whose evidence units cite nothing yields a
//! `bind:<row-id>` row with class `binds_nothing`, surfaced in the class
//! counts and on the envelope's warnings channel, exit 0, counts in BOTH
//! output views (JSON envelope and `--human`). The classification and
//! honesty pins live in `src/audit.rs` (`mod binds_nothing_tests`); GREEN
//! lands there (tasks 3.2). Read-only over the compiled artifacts: the
//! reused gate runs unchanged.

#[path = "common/mod.rs"]
mod common;

use assert_cmd::Command;
use common::UnwrapPanic;

/// A lint-clean, compilable CLI fixture whose only evidence unit (the
/// `locked` fragment `status != "open"`) cites no declared model element
/// (declared: `held`, `confirmed`, `confirm`, `reopen`, `locked`, `p`) —
/// the bowling-rev1 shape: green yet vacuous. The property predicate `x`
/// carries no marker, so it is not an evidence unit.
const CLI_SPEC: &str = r#"---
id: demo.bind_cli
kind: intent
statement: "THE system SHALL keep the ledger consistent"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.bind_cli]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.bind_cli.locked]] |
| reopen | confirmed | held | [[demo.bind_cli.locked]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[demo.bind_cli.locked]] | `g()` | `x` |
"#;

/// One permitted scenario — the oracle row is `current`; the vacuity
/// row rides beside it without failing the run.
const CONFIRM_HAPPY: &str = r#"{"id": "confirm-happy", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"status": "confirmed"}}]}"#;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap_p()
}

/// Compile the fixture spec into a temp out-dir (the reused gate
/// requires current artifacts — audit never re-compiles, like conform).
fn compiled_out_dir(dir: &std::path::Path, spec: &std::path::Path) -> String {
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

fn audit(
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
            &out_dir,
        ])
        .args(extra)
        .output()
        .unwrap_p();
    (out.status.code(), out.stdout, out.stderr)
}

/// The full honesty surface (delta scenario "Unbound evidence flagged,
/// sparsity respected" + "Unchecked never fails the run", CLI level):
/// the vacuous fragment flags `binds_nothing` beside the `current`
/// oracle row, the run exits 0, the counts surface in BOTH views, and
/// the envelope carries the warnings-channel advisory.
#[test]
fn binds_nothing_flags_on_the_warnings_channel_exit_zero() {
    let dir = tempfile::tempdir().unwrap_p();
    let spec = dir.path().join("demo.bind_cli.md");
    std::fs::write(&spec, CLI_SPEC).unwrap_p();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, CONFIRM_HAPPY).unwrap_p();

    // JSON view: the bind row, the class count, the warning, exit 0.
    let (code, stdout, _stderr) = audit(&spec, &corpus, &["--json"]);
    assert_eq!(code, Some(0), "binds_nothing never fails the run");
    let json: serde_json::Value = serde_json::from_slice(&stdout).unwrap_p();
    assert_eq!(json["ok"], serde_json::json!(true), "{json}");
    let data = &json["data"];
    let bind = data["records"]
        .as_array()
        .unwrap_p()
        .iter()
        .find(|r| r["scenario_id"] == "bind:locked")
        .expect("the vacuous fragment's bind row is present");
    assert_eq!(bind["class"], "binds_nothing");
    assert_eq!(bind["conform_verdict"], "agree");
    assert_eq!(data["class_counts"]["binds_nothing"], 1);
    assert_eq!(data["class_counts"]["current"], 1);
    let warnings = json["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| {
            w["message"]
                .as_str()
                .is_some_and(|s| s.contains("binds_nothing"))
        }),
        "the warnings channel names the vacuity advisory: {json}"
    );

    // Human view: the same counts, rendered (both views cannot drift);
    // at `-v` the warnings channel prints the advisory to stderr (the
    // default-verbosity human channel is the count line above it).
    let (code, stdout, stderr) = audit(&spec, &corpus, &["--human", "-v"]);
    assert_eq!(code, Some(0), "binds_nothing never fails the run");
    let human = String::from_utf8(stdout).unwrap_p();
    assert!(
        human.contains("binds_nothing 1"),
        "the human view surfaces the binds_nothing count: {human}"
    );
    assert!(
        human.contains("bind:locked"),
        "the human view names the bind row: {human}"
    );
    let err_text = String::from_utf8(stderr).unwrap_p();
    assert!(
        err_text.contains("binds_nothing"),
        "the verbose human mode's stderr carries the advisory: {err_text}"
    );
}
