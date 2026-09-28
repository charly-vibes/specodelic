//! Integration tests — run the `specodelic` binary against the repo's own corpus.

use assert_cmd::Command;
use predicates::str::contains;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap()
}

#[test]
fn lint_corpus_is_fully_clean() {
    // The corpus is the first dogfood target: every spec file must parse,
    // every reference must resolve, and every constraint must be covered
    // by a deriving property (beads specodelic-qc8 closed the last gaps).
    let out = spk().args(["lint", "specs", "--json"]).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let data = &json["data"];
    assert_eq!(data["files_linted"], 18);
    let issues = data["issues"].as_array().unwrap();
    assert!(issues.is_empty(), "corpus lint findings: {issues:?}");
    assert_eq!(out.status.code(), Some(0));
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
    // ears_syntax (no SHALL) + guard_required (empty guard) both fire
    assert!(stdout.contains("ears_syntax"));
    assert!(stdout.contains("guard_required"));
    // self-describing findings: rule id + semantics in the JSON payload
    assert!(stdout.contains("linter.ears_syntax"));
    assert!(stdout.contains("rule_semantics"));
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn bad_ears_finding_carries_rule_id_and_semantics() {
    // spec scenario: agent reads a finding without repo access — the
    // JSON finding names the rule id and states what the rule requires
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
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    let ears = json["data"]["issues"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["rule_id"] == "linter.ears_syntax")
        .expect("bad-EARS fixture must yield a linter.ears_syntax finding");
    assert!(!ears["rule_semantics"].as_str().unwrap().is_empty());
    assert_eq!(ears["rule"], "ears_syntax");
}

#[test]
fn human_output_shows_rule_id() {
    // spec scenario: human output shows the rule id
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad_spec.md");
    std::fs::write(
        &bad,
        "---\nid: bad_spec\nkind: intent\nstatement: \"the system should maybe work\"\n---\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--human"])
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(0));
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("linter.ears_syntax")
    );
}

#[test]
fn explain_lint_rules_renders_the_rule_catalog() {
    // `explain lint-rules` renders from the same table the linter emits
    // from — the rendered catalog is exactly the emittable rule id set
    let out = spk()
        .args(["explain", "lint-rules", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    let body = json["data"]["body"].as_str().unwrap();
    assert!(body.contains("linter.ears_syntax"));
    assert!(body.contains("linter.guard_required"));
    assert!(body.contains("linter.coverage"));
    assert!(!body.contains("{{"));
    assert!(!body.contains("shipped with the self-describing lint findings change"));
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
    // the scaffold carries per-layer guidance comments that teach the
    // format in place (task 4.1); closed sets render from the guide
    // constants
    assert!(content.contains("<!-- Intent layer:"));
    assert!(content.contains("<!-- kind: one of invariant | advisory | effect | extension_point"));
    assert!(content.contains("<!-- kind: one of unit | law"));
    assert!(content.contains("<!-- guard: must cite an invariant Constraint"));
    // the scaffold is lintable shape-wise: model sections present with a
    // placeholder transition, Ubiquitous EARS statement — i.e. linting a
    // valid spec containing the guidance comments yields no findings
    // (task 4.2; the linter never parses prose/comments)
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
    // `compile` is implemented (specodelic-lnq); model_check and verify
    // keep their stubs until their tickets land.
    spk().args(["model-check"]).assert().failure();
    spk().args(["verify"]).assert().failure();
    spk().args(["orchestrate"]).assert().failure();
}

#[test]
fn compile_corpus_succeeds_and_reports_all_artifacts() {
    let out = tempfile::tempdir().unwrap();
    let cmd = spk()
        .args([
            "compile",
            "specs",
            "--json",
            "--out-dir",
            out.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&cmd.stdout).unwrap();
    let data = &json["data"];
    assert_eq!(data["files_failed"], 0);
    // 18 corpus spec files (the exemption list's non-spec files are skipped)
    assert_eq!(data["files_compiled"], 18);
    for f in data["compiled"].as_array().unwrap() {
        assert!(
            f["artifacts"]["toml"]
                .as_str()
                .unwrap()
                .contains("[constraints]")
        );
        assert!(f["artifacts"]["props"].as_str().is_some());
        assert!(f["artifacts"]["tla"].as_str().unwrap().contains("MODULE "));
        assert!(f["model_ir"]["states"].as_array().unwrap().len() >= 1);
        assert_eq!(f["written"].as_array().unwrap().len(), 3);
    }
    assert_eq!(cmd.status.code(), Some(0));
}

#[test]
fn compile_refuses_lint_dirty_file_naming_precondition() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad_spec.md");
    std::fs::write(
        &bad,
        "---\nid: bad_spec\nkind: intent\nstatement: \"the system should maybe work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[bad_spec]] |\n\n## Model\n\n### States\n\n- s1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s2 | |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[bad_spec.a]] | `g()` | `x` |\n",
    )
    .unwrap();
    let out = spk()
        .args([
            "compile",
            dir.path().to_str().unwrap(),
            "--json",
            "--out-dir",
            dir.path().join("artifacts").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let failed = json["data"]["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0]["stage"], "precondition_satisfied");
    assert_eq!(out.status.code(), Some(1));
    // compile_is_total: no artifacts written for the refused file
    assert!(!dir.path().join("artifacts").exists());
}

#[test]
fn compile_round_trip_is_byte_stable() {
    let out = tempfile::tempdir().unwrap();
    let od = out.path().to_str().unwrap();
    // Reference resolution is corpus-wide (total_refs), so compile the
    // whole corpus; artifacts land in a tempdir out-dir.
    let args = ["compile", "specs", "--json", "--out-dir", od];
    spk().args(args).assert().success();
    let first_toml = std::fs::read_to_string(out.path().join("compile.toml")).unwrap();
    let first_props = std::fs::read_to_string(out.path().join("compile_props.rs")).unwrap();
    let first_tla = std::fs::read_to_string(out.path().join("compile.tla")).unwrap();
    spk().args(args).assert().success();
    let second_toml = std::fs::read_to_string(out.path().join("compile.toml")).unwrap();
    let second_props = std::fs::read_to_string(out.path().join("compile_props.rs")).unwrap();
    let second_tla = std::fs::read_to_string(out.path().join("compile.tla")).unwrap();
    assert_eq!(first_toml, second_toml);
    assert_eq!(first_props, second_props);
    assert_eq!(first_tla, second_tla);
    // the model artifact always ships: transitions → Next disjuncts
    assert!(first_tla.contains("MODULE compile"));
    assert!(first_tla.contains("\\/ vpc = \"not_started\""));
    // ids preserved: the source file's intent id anchors the TOML document
    assert!(first_toml.contains("source = \"compile\""));
    assert!(first_toml.contains("id = \"compile_is_total\""));
}

#[test]
fn explain_bare_lists_exactly_the_six_topics() {
    let out = spk().args(["explain", "--json"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    let ids: Vec<&str> = json["data"]["topics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "format",
            "ears",
            "kinds",
            "references",
            "lifecycle",
            "lint-rules"
        ]
    );
}

#[test]
fn explain_known_topic_works_offline_in_consumer_dir() {
    // consumer repo: no specs/ at all — the guide is embedded in the
    // binary, so the envelope still carries the full body
    let dir = tempfile::tempdir().unwrap();
    let out = spk()
        .args(["explain", "format", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["data"]["topic"], "format");
    assert_eq!(json["data"]["format_revision"], "specodelic.md Revision 8");
    let body = json["data"]["body"].as_str().unwrap();
    assert!(body.contains("## Constraints"));
    assert!(body.contains("## Properties"));
    assert!(body.contains("lifecycle"));
    assert!(
        !body.contains("{{"),
        "placeholders must be filled at render time"
    );
}

#[test]
fn explain_kinds_renders_enforced_closed_sets() {
    // the rendered kind sets equal the constants the linter enforces,
    // because both come from the same pub const source (spec scenario)
    let out = spk().args(["explain", "kinds", "--json"]).output().unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    let body = json["data"]["body"].as_str().unwrap();
    assert!(body.contains("`invariant`, `advisory`, `effect`, `extension_point`"));
    assert!(body.contains("`unit`, `law`"));
}

#[test]
fn explain_unknown_topic_fails_with_topic_hint() {
    let out = spk().args(["explain", "nope", "--json"]).output().unwrap();
    assert_ne!(out.status.code(), Some(0));
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(json["envelope_kind"], "error");
    // the failure message names every valid topic (repo convention: failure
    // messages go to stderr even in JSON mode)
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("unknown topic `nope`"));
    assert!(stderr.contains("format | ears | kinds | references | lifecycle | lint-rules"));
    assert!(json["hints"].as_array().unwrap().iter().any(|h| {
        h["command"]
            .as_str()
            .unwrap()
            .contains("specodelic explain")
    }));
}

#[test]
fn version_json_reports_format_revision() {
    let out = spk().args(["--version", "--json"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(json["envelope_kind"], "version");
    assert_eq!(json["data"]["name"], "specodelic");
    assert_eq!(json["data"]["format_revision"], "specodelic.md Revision 8");
}
