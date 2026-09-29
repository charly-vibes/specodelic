//! Integration tests — run the `specodelic` binary against the repo's own corpus.

use assert_cmd::Command;
use predicates::str::contains;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap()
}

fn write_bad_ears_spec(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!("---\nid: {id}\nkind: intent\nstatement: \"the system should maybe work\"\n---\n"),
    )
    .unwrap();
}

/// A lint-clean spec with a two-state model — the model-check fixture.
fn write_model_check_spec(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL be a model-check fixture\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | c1 | invariant | `holds` | [[{id}]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- s1\n\
             - s2\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | t | s1 | s2 | [[{id}.c1]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p | unit | [[{id}.c1]] | `g()` | `x` |\n"
        ),
    )
    .unwrap();
}

// ---- specodelic-zo9: dual_format_valid lint rule ----

#[test]
fn lint_flags_half_format_dual_file_via_cli() {
    // A file with frontmatter + ## ADDED Requirements but no sibling
    // ## Requirements is half a dual-format file — the rule fires so the
    // spec-integration protocol is tool-enforced, not convention.
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("spec.md"),
        "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## ADDED Requirements\n\n### Requirement: Something\nThe system SHALL do the thing.\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("dual_format_valid"),
        "half-format file flagged: {stdout}"
    );
    assert_eq!(out.status.code(), Some(1));
}

// ---- specodelic-6pi: recursive spec collection, no silent empty success ----

#[test]
fn lint_finds_specs_in_nested_directories() {
    // collect_specs must recurse: a spec in a subdirectory is linted,
    // not silently ignored (beads specodelic-6pi).
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("domain").join("deep");
    std::fs::create_dir_all(&nested).unwrap();
    write_bad_ears_spec(&nested.join("nested_spec.md"), "nested_spec");
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("nested_spec.md"),
        "nested spec must be linted: {stdout}"
    );
    assert!(stdout.contains("ears_syntax"));
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn lint_skips_hidden_and_build_directories() {
    // Recursive collection must not sweep .git/, target/, node_modules/:
    // exactly the top-level spec is linted, junk specs are never read.
    let dir = tempfile::tempdir().unwrap();
    write_bad_ears_spec(&dir.path().join("top_spec.md"), "top_spec");
    for junk in [".git", "target", "node_modules"] {
        let j = dir.path().join(junk);
        std::fs::create_dir_all(&j).unwrap();
        let id = format!("junk_{}", junk.trim_start_matches('.'));
        write_bad_ears_spec(&j.join(format!("{id}.md")), &id);
    }
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(json["data"]["files_linted"], 1, "only the top-level spec");
    let issues = json["data"]["issues"].as_array().unwrap();
    assert!(!issues.is_empty(), "top_spec's ears_syntax must fire");
    assert!(
        issues.iter().all(|i| i["file"]
            .as_str()
            .map(|f| f.contains("top_spec.md"))
            .unwrap_or(false)),
        "no finding may come from hidden/build dirs: {issues:?}"
    );
}

#[test]
fn lint_fails_with_hint_when_no_specs_found() {
    // A directory with no spec files must not yield a silent ok:true —
    // that's a false green (beads specodelic-6pi). Failure message rides
    // stderr even in JSON mode (convention of record).
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("README.md"), "# not a spec\n").unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("no spec files found"), "stderr: {stderr}");
    // JSON mode keeps the remediation hint inside the envelope (stdout);
    // the footer prints to stderr only in human mode (convention of record)
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("frontmatter"), "envelope hint: {stdout}");
    assert_eq!(out.status.code(), Some(1));
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
fn graph_same_id_files_are_file_scoped() {
    // Dual-format self-containment (Rule-of-5 CORR-001): two id:spec
    // files share one file id, but a ref in one must NOT resolve against
    // the other's rows — corpus-wide union false-resolves typos.
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    std::fs::create_dir_all(&a).unwrap();
    std::fs::create_dir_all(&b).unwrap();
    std::fs::write(
        a.join("spec.md"),
        "---\nid: spec\nkind: intent\nstatement: \"THE a SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| local_a | invariant | `x` | [[spec.row_in_b]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[spec.local_a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| pa | unit | [[spec.local_a]] | `g()` | `x` |\n",
    )
    .unwrap();
    std::fs::write(
        b.join("spec.md"),
        "---\nid: spec\nkind: intent\nstatement: \"THE b SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| row_in_b | invariant | `z` | |\n\n## Model\n\n### States\n\n- `s2`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s2 | s2 | [[spec.row_in_b]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| pb | unit | [[spec.row_in_b]] | `g()` | `z` |\n",
    )
    .unwrap();
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("[[spec.row_in_b]]"),
        "cross-file ref must dangle in the graph: {stdout}"
    );
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
    // the bare `rule` field was dropped in the review pass — rule_id is
    // the single naming (rule ids are stable, linter.<name>)
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
    assert!(!body.contains("{{"));
    assert!(!body.contains("shipped with the self-describing lint findings change"));
    // the rendered catalog enumerates every rule id the linter can emit
    // (review EDGE-001b) — loop over the lib's RULE_TABLE directly
    for (name, _) in specodelic::lint::RULE_TABLE {
        let id = format!("linter.{name}");
        assert!(body.contains(&id), "catalog missing rule id {id}");
    }
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
    // `compile` and `model-check` are implemented (specodelic-lnq,
    // specodelic-nx7); verify and orchestrate keep their stubs until
    // their tickets land.
    spk().args(["verify"]).assert().failure();
    spk().args(["orchestrate"]).assert().failure();
}

// ---- specodelic-nx7: the model_check step (add-model-check) ----

#[test]
fn model_check_reports_no_counterexample_after_compile() {
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("mc_demo.md");
    write_model_check_spec(&spec, "mc_demo");
    let out_dir = dir.path().join("specodelic");
    spk()
        .args([
            "compile",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .assert()
        .success();
    let out = spk()
        .args([
            "model-check",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let data = &json["data"];
    // Meter contract: .data.outcome for the single-file case.
    assert_eq!(data["outcome"], "no_counterexample");
    assert_eq!(data["files_checked"], 1);
    let checked = &data["checked"][0];
    assert_eq!(checked["backend"]["engine"], "stateright");
    assert!(
        checked["backend"]["version"]
            .as_str()
            .unwrap()
            .starts_with("0.")
    );
    assert_eq!(checked["bound"]["max_depth"], 100);
    assert_eq!(checked["invariants_checked"].as_array().unwrap().len(), 0);
    // states: s1 (init) -> s2
    assert_eq!(checked["states_explored"], 2);
    // The run report persists with artifact provenance.
    let report_path = out_dir.join("mc_demo.check.json");
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report_path).unwrap()).unwrap();
    assert_eq!(report["outcome"], "no_counterexample");
    assert_eq!(report["artifact_sha256"].as_str().unwrap().len(), 64);
}

#[test]
fn model_check_without_compiled_artifact_is_a_labeled_error() {
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("mc_demo.md");
    write_model_check_spec(&spec, "mc_demo");
    let out = spk()
        .args([
            "model-check",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            dir.path().join("nowhere").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let failed = &json["data"]["failed"][0];
    assert_eq!(failed["stage"], "missing_artifact");
    // remediation hint, never a silent no_counterexample
    assert!(
        failed["message"]
            .as_str()
            .unwrap()
            .contains("specodelic compile")
    );
}

#[test]
fn model_check_rejects_a_spec_edited_after_compile() {
    // CORR-001 (ro5): the run interprets the live spec's IR, so a module
    // compiled from an older Model section must be a labeled error —
    // never a clean run against a mixed (IR, artifact) pair.
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("mc_demo.md");
    write_model_check_spec(&spec, "mc_demo");
    let out_dir = dir.path().join("specodelic");
    spk()
        .args([
            "compile",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .assert()
        .success();
    // Edit the Model section WITHOUT recompiling: add a third state.
    let text = std::fs::read_to_string(&spec)
        .unwrap()
        .replace("- s2", "- s2\n- s3");
    std::fs::write(&spec, text).unwrap();
    let out = spk()
        .args([
            "model-check",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let failed = &json["data"]["failed"][0];
    assert_eq!(failed["stage"], "stale_artifact");
    assert!(
        failed["message"]
            .as_str()
            .unwrap()
            .contains("specodelic compile")
    );
}

#[test]
fn model_check_timed_out_when_bound_cannot_be_exhausted() {
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("mc_demo.md");
    write_model_check_spec(&spec, "mc_demo");
    let out_dir = dir.path().join("specodelic");
    spk()
        .args([
            "compile",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .assert()
        .success();
    // The model's diameter (1) reaches the stated depth cap, so
    // exhaustiveness within the bound cannot be proven.
    let out = spk()
        .args([
            "model-check",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--max-depth",
            "1",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["data"]["outcome"], "timed_out");
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
        assert!(!f["model_ir"]["states"].as_array().unwrap().is_empty());
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
fn explain_bare_lists_exactly_the_seven_topics() {
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
            "lint-rules",
            "dual-format"
        ]
    );
}

#[test]
fn explain_dual_format_topic_serves_the_migration_recipe() {
    // The dual-format topic must carry the protocol's naming law and the
    // migration path — the guidance a consumer needs when a
    // dual_format_valid finding confuses them (Rule-of-5 DRAFT-001).
    let out = spk()
        .args(["explain", "dual-format", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    let body = json["data"]["body"].as_str().unwrap();
    assert!(body.contains("id: spec"), "naming law: {body}");
    assert!(body.contains("## ADDED Requirements"), "delta half: {body}");
    assert!(body.contains("## Requirements"), "capability half: {body}");
    assert!(
        body.to_lowercase().contains("migration"),
        "migration path: {body}"
    );
    assert!(
        body.contains("dual_format_valid"),
        "enforcing rule named: {body}"
    );
    assert!(
        !body.contains("{{"),
        "no unfilled placeholder slots: {body}"
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

#[test]
fn doctor_self_hosting_reports_mode() {
    // this repo carries specs/specodelic.md → mode self_hosting
    let out = spk().args(["doctor", "--json"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(json["data"]["mode"], "self_hosting");
}

#[test]
fn doctor_consumer_with_corpus_reports_format_revision() {
    // consumer workspace: a corpus without the core spec still diagnoses
    // cleanly and reports the embedded guide's format_revision as ok
    let dir = tempfile::tempdir().unwrap();
    let specs = dir.path().join("specs");
    std::fs::create_dir(&specs).unwrap();
    std::fs::write(
        specs.join("my_tool.md"),
        "---\nid: my.tool\nkind: intent\nstatement: \"THE tool SHALL work\"\n---\n",
    )
    .unwrap();
    let out = spk()
        .args(["doctor", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(json["data"]["mode"], "consumer");
    assert_eq!(json["data"]["format_revision"], "specodelic.md Revision 8");
}

#[test]
fn doctor_empty_consumer_suggests_new() {
    let dir = tempfile::tempdir().unwrap();
    // human mode: the next-step footer (with the suggestion) prints to
    // stderr; JSON mode keeps everything in the envelope on stdout
    let out = spk()
        .args(["doctor", "--human"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("spk new"), "footer: {stderr}");
}

#[test]
fn doctor_warns_when_binary_lags_corpus() {
    // synthetic corpus declares Revision 99 while the binary embeds
    // Revision 8 → warning naming both revisions, exit 0 (never fails)
    let dir = tempfile::tempdir().unwrap();
    let specs = dir.path().join("specs");
    std::fs::create_dir(&specs).unwrap();
    std::fs::write(
        specs.join("specodelic.md"),
        "---\nid: specodelic\nkind: intent\nstatement: \"THE format SHALL be described\"\n---\n\n## Revision 1\n\n## Revision 99\n",
    )
    .unwrap();
    let out = spk()
        .args(["doctor", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "warn, never fail");
    // the warning rides the success envelope's warnings channel — repo
    // convention: only failure messages go to stderr, even in JSON mode
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    let warnings: Vec<String> = json["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["message"].as_str().unwrap().to_string())
        .collect();
    assert!(
        warnings.iter().any(|w| w.contains("99") && w.contains("8")),
        "warning names both revisions: {warnings:?}"
    );
}

#[test]
fn doctor_current_consumer_emits_no_currency_warning() {
    // corpus at the embedded revision (8) → no warning; also covers the
    // no-revision-heading skip via a second fixture
    let dir = tempfile::tempdir().unwrap();
    let specs = dir.path().join("specs");
    std::fs::create_dir(&specs).unwrap();
    std::fs::write(
        specs.join("specodelic.md"),
        "---\nid: specodelic\nkind: intent\nstatement: \"THE format SHALL be described\"\n---\n\n## Revision 8\n",
    )
    .unwrap();
    let out = spk()
        .args(["doctor", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(
        !String::from_utf8(out.stderr)
            .unwrap()
            .contains("newer than")
    );
}

#[test]
fn doctor_skips_currency_check_with_note_on_unreadable_corpus() {
    // review EDGE-001: an unreadable corpus must be skipped with an
    // informational warning, never silently treated as current
    let dir = tempfile::tempdir().unwrap();
    let specs = dir.path().join("specs");
    std::fs::create_dir(&specs).unwrap();
    std::fs::write(specs.join("specodelic.md"), b"\xff\xfe\x00binary").unwrap();
    let out = spk()
        .args(["doctor", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "warn, never fail");
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    let warnings: Vec<String> = json["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["message"].as_str().unwrap().to_string())
        .collect();
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("could not read") && w.contains("currency check skipped")),
        "expected an informational read-error note: {warnings:?}"
    );
    // and it must not claim currency: no lag warning, no fake-ok
    assert!(!warnings.iter().any(|w| w.contains("newer than")));
}

// ---- specodelic-ze4: spk feedback + spk init (managed block) ----

#[test]
fn feedback_dry_run_previews_issue_without_filing() {
    // dry-run must print the redacted body + the would-file command,
    // never invoke gh, and exit 0; content comes from piped stdin
    // (genesis handle_feedback contract) — --from-last-error is the
    // other source
    let dir = tempfile::tempdir().unwrap();
    let out = spk()
        .args(["feedback", "bug", "--dry-run"])
        .current_dir(dir.path())
        .env("NO_COLOR", "1")
        .write_stdin("spk lint crashes on empty dirs\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "dry-run never fails");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Would file"), "preview: {stderr}");
    assert!(
        stderr.contains("charly-vibes/specodelic"),
        "target repo: {stderr}"
    );
}

#[test]
fn feedback_title_flag_overrides_derived_title() {
    // genesis 0.8 FeedbackArgs.title: a user-supplied --title must override
    // the stdin-derived title and land verbatim in the would-file command
    let dir = tempfile::tempdir().unwrap();
    let out = spk()
        .args([
            "feedback",
            "bug",
            "--dry-run",
            "--title",
            "lint crashes on empty dirs",
        ])
        .current_dir(dir.path())
        .env("NO_COLOR", "1")
        .write_stdin("body text\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "dry-run never fails");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("--title \"[bug] lint crashes on empty dirs\""),
        "supplied title in would-file command: {stderr}"
    );
}

#[test]
fn feedback_rejects_unknown_kind_with_hint() {
    let out = spk().args(["feedback", "bugg"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("bug"), "valid kinds suggested: {stderr}");
}

#[test]
fn init_injects_specodelic_block_into_agents_md() {
    // spk init writes/refreshes a <!-- SPECODELIC:START/END --> block in
    // AGENTS.md carrying the lint rule catalog + format_revision
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("AGENTS.md"), "# My repo\n\nAgent notes.\n").unwrap();
    let out = spk()
        .args(["init", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let agents = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(
        agents.contains("<!-- SPECODELIC:START -->"),
        "block injected: {agents}"
    );
    assert!(agents.contains("<!-- SPECODELIC:END -->"));
    // existing content is preserved (block prepends, never replaces)
    assert!(agents.contains("# My repo"));
    assert!(agents.contains("Agent notes."));
    // block content is self-describing: rule catalog + revision + commands
    assert!(agents.contains("linter.ears_syntax"));
    assert!(agents.contains("linter.frontmatter_valid"));
    assert!(agents.contains("specodelic.md Revision 8"));
    assert!(agents.contains("spk lint"));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["data"]["block"], "injected");
    assert_eq!(json["data"]["file"], "AGENTS.md");
}

#[test]
fn init_updates_stale_block_in_place() {
    // second run updates the block in place: content between the markers
    // is refreshed, surrounding text preserved, exactly one block
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("AGENTS.md"), "# My repo\n\n<!-- SPECODELIC:START -->\nstale specodelic content\n<!-- SPECODELIC:END -->\n").unwrap();
    let out = spk()
        .args(["init", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let agents = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(
        !agents.contains("stale specodelic content"),
        "refreshed: {agents}"
    );
    assert!(agents.contains("# My repo"));
    assert_eq!(agents.matches("SPECODELIC:START").count(), 1);
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["data"]["block"], "updated");
}

#[test]
fn init_creates_agents_md_when_missing() {
    let dir = tempfile::tempdir().unwrap();
    let out = spk()
        .args(["init", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let agents = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(agents.contains("<!-- SPECODELIC:START -->"));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["data"]["block"], "created");
}

#[test]
fn doctor_reports_missing_or_stale_block() {
    // doctor: fresh block → silent ok; missing block → hint to run spk init
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("AGENTS.md"), "# My repo\n").unwrap();
    let out = spk()
        .args(["doctor", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "advisory, never fails");
    let stdout_str = String::from_utf8(out.stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout_str).unwrap();
    let hints: Vec<String> = json["hints"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["command"].as_str().unwrap_or_default().to_string())
        .chain(json["data"]["next_step"].as_str().map(|s| s.to_string()))
        .collect();
    // the doctor must point at `spk init` when the block is absent/stale
    // (advisory: warnings channel in JSON, stderr footer in human mode)
    let out2 = spk()
        .args(["doctor", "--human"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8(out2.stderr).unwrap();
    let stdout = String::from_utf8(out2.stdout).unwrap();
    assert!(
        stderr.contains("spk init") || stdout.contains("spk init"),
        "doctor should suggest spk init when block missing: {stderr} | {stdout}"
    );
    let json2: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let warnings: Vec<String> = json2["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["message"].as_str().unwrap_or_default().to_string())
        .collect();
    assert!(
        warnings.iter().any(|w| w.contains("spk init")),
        "warning names the remediation: {warnings:?}"
    );
    let _ = hints;
}

// ---- specodelic-cxr: spk hooks install / uninstall ----

/// Wire a fake `spk` on PATH so the install-time gate dry-run is
/// deterministic: "pass" exits 0, "fail" exits 1 with a lint summary.
fn fake_spk(dir: &std::path::Path, behavior: &str) -> std::path::PathBuf {
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let script = match behavior {
        "pass" => "#!/bin/sh\necho 'linted 12 files, 0 issues'\n",
        _ => "#!/bin/sh\necho 'linter.dual_format_valid: openspec/changes/x/spec.md' >&2\nexit 1\n",
    };
    std::fs::write(bin.join("spk"), script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(bin.join("spk"), std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    bin
}

/// A minimal repo fixture: `.git` marker, optional lefthook config,
/// optional openspec/ tree (the install pre-check target).
fn hooks_fixture(with_config: bool, with_openspec: bool) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    if with_config {
        std::fs::write(
            dir.path().join("lefthook.yml"),
            "pre-commit:\n  commands:\n    sibling-blockers:\n      run: scripts/guards/sibling-blockers.sh .\n",
        )
        .unwrap();
    }
    if with_openspec {
        std::fs::create_dir_all(dir.path().join("openspec")).unwrap();
    }
    dir
}

fn with_path(bin: &std::path::Path) -> String {
    format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

#[test]
fn hooks_install_wires_gate_and_reports_envelope() {
    let dir = hooks_fixture(true, true);
    let pathdir = tempfile::tempdir().unwrap();
    let bin = fake_spk(pathdir.path(), "pass");
    let out = spk()
        .args(["hooks", "install", "--json"])
        .current_dir(dir.path())
        .env("PATH", with_path(&bin))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"outcome\""), "envelope data: {stdout}");
    assert!(stdout.contains("wired"), "envelope data: {stdout}");
    let config = std::fs::read_to_string(dir.path().join("lefthook.yml")).unwrap();
    assert!(
        config.contains(
            "  commands:\n    # <!-- SPK:START -->\n    specodelic-gates:\n      run: spk lint openspec\n    # <!-- SPK:END -->\n    sibling-blockers:"
        ),
        "entry inside the existing commands mapping:\n{config}"
    );
    assert_eq!(config.matches("commands:").count(), 1, "no duplicate key");
    assert!(
        stdout.contains("gate_dry_run"),
        "dry-run reported: {stdout}"
    );
}

#[test]
fn hooks_install_warns_but_succeeds_when_gate_fails() {
    let dir = hooks_fixture(true, true);
    let pathdir = tempfile::tempdir().unwrap();
    let bin = fake_spk(pathdir.path(), "fail");
    let out = spk()
        .args(["hooks", "install", "--json"])
        .current_dir(dir.path())
        .env("PATH", with_path(&bin))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "install must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("spk hooks uninstall"),
        "escape hint in warnings: {stdout}"
    );
    assert!(
        std::fs::read_to_string(dir.path().join("lefthook.yml"))
            .unwrap()
            .contains("specodelic-gates:"),
        "still wired"
    );
}

#[test]
fn hooks_install_errors_on_missing_config_and_creates_none() {
    let dir = hooks_fixture(false, true);
    let pathdir = tempfile::tempdir().unwrap();
    let bin = fake_spk(pathdir.path(), "pass");
    let out = spk()
        .args(["hooks", "install", "--json"])
        .current_dir(dir.path())
        .env("PATH", with_path(&bin))
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no supported hook framework config"),
        "labeled error: {stderr}"
    );
    assert!(!dir.path().join("lefthook.yml").exists(), "never created");
}

#[test]
fn hooks_install_requires_openspec_dir() {
    let dir = hooks_fixture(true, false);
    let out = spk()
        .args(["hooks", "install", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("openspec"),
        "error names the missing openspec tree: {stderr}"
    );
    assert!(
        !std::fs::read_to_string(dir.path().join("lefthook.yml"))
            .unwrap()
            .contains("specodelic-gates:"),
        "config untouched when the pre-check fails"
    );
}

#[test]
fn hooks_uninstall_on_unwired_repo_is_a_noop() {
    let dir = hooks_fixture(true, true);
    let before = std::fs::read_to_string(dir.path().join("lefthook.yml")).unwrap();
    let out = spk()
        .args(["hooks", "uninstall", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "no-op is success; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("not_wired"), "envelope: {stdout}");
    let after = std::fs::read_to_string(dir.path().join("lefthook.yml")).unwrap();
    assert_eq!(before, after, "config unchanged");
}

#[test]
fn hooks_uninstall_strips_only_the_block() {
    let dir = hooks_fixture(true, true);
    let pathdir = tempfile::tempdir().unwrap();
    let bin = fake_spk(pathdir.path(), "pass");
    spk()
        .args(["hooks", "install", "--json"])
        .current_dir(dir.path())
        .env("PATH", with_path(&bin))
        .output()
        .unwrap()
        .status
        .success()
        .then_some(())
        .expect("install must succeed");
    let wired = std::fs::read_to_string(dir.path().join("lefthook.yml")).unwrap();
    let out = spk()
        .args(["hooks", "uninstall", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("unwired"), "envelope: {stdout}");
    let after = std::fs::read_to_string(dir.path().join("lefthook.yml")).unwrap();
    let restored: String = wired
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
    assert_eq!(
        after.lines().collect::<Vec<_>>().join("\n"),
        restored,
        "only block lines removed:\n{after}"
    );
}
