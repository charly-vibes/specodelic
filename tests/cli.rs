//! Integration tests — run the `specodelic` binary against the repo's own corpus.

use assert_cmd::Command;
use predicates::str::contains;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap()
}

#[test]
fn lint_corpus_reports_coverage_gaps_but_parses_clean() {
    // The corpus is the first dogfood target: every spec file must parse
    // and every reference must resolve (0 total_refs / parse failures);
    // known coverage gaps are tracked in beads (specodelic-*).
    let out = spk().args(["lint", "specs", "--json"]).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let data = &json["data"];
    assert_eq!(data["files_linted"], 18);
    let issues = data["issues"].as_array().unwrap();
    let non_coverage: Vec<_> = issues.iter().filter(|i| i["rule"] != "coverage").collect();
    assert!(
        non_coverage.is_empty(),
        "unexpected non-coverage lint findings: {non_coverage:?}"
    );
    // coverage gaps are known and tracked — lint exits 1 until they close
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn graph_corpus_is_fully_resolved() {
    let out = spk().args(["graph", "specs", "--json"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let data = &json["data"];
    assert!(data["dangling"].as_array().unwrap().is_empty());
    assert!(data["edges"].as_array().unwrap().len() > 300);
}

#[test]
fn lint_synthetically_bad_file_fails_with_rule_name() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad_spec.md");
    std::fs::write(
        &bad,
        "---\nid: bad_spec\nkind: intent\nstatement: \"the system should maybe work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[bad_spec]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s2 | |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[bad_spec.a]] | `g()` | `x` |\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    // ears_statement (no SHALL) + guard_required (empty guard) both fire
    assert!(stdout.contains("ears_statement"));
    assert!(stdout.contains("guard_required"));
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn new_scaffolds_a_spec_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = spk()
        .args(["new", "demo.thing", "--file", "demo-thing.md"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let content = std::fs::read_to_string(dir.path().join("demo-thing.md")).unwrap();
    assert!(content.contains("id: demo.thing"));
    // the scaffold is lintable shape-wise: model sections present with a
    // placeholder transition, Ubiquitous EARS statement
    let lint = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(lint.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["data"]["issues"].as_array().unwrap().len(), 0);
    assert_eq!(lint.status.code(), Some(0));
}

#[test]
fn doctor_checks_workspace() {
    spk()
        .args(["doctor", "--json"])
        .assert()
        .success()
        .stdout(contains("beads"));
}

#[test]
fn unimplemented_pipeline_commands_exit_nonzero() {
    spk()
        .args(["compile", "specs/specodelic.md"])
        .assert()
        .failure();
    spk().args(["verify"]).assert().failure();
    spk().args(["orchestrate"]).assert().failure();
}
