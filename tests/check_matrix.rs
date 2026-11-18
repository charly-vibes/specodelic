//! CLI matrix for `check` (openspec/changes/add-check-judge-verb tasks
//! 1.1–1.5, 3.1): exit codes, the report shape shared with `audit`, the
//! ladder rung the verify stage unlocks, prior-report interop both ways,
//! determinism, and the refusal paths that must emit zero records.

#[path = "common/mod.rs"]
mod common;

use common::UnwrapPanic;
use std::path::Path;

use assert_cmd::Command;

const SCOPE_EMPTY: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap_p()
}

/// A lint-clean spec with one executable invariant and one property whose
/// predicate is supplied (`**rust:** true` passes, `**rust:** false`
/// fails, a sleeping block exercises the bound).
fn write_spec(path: &Path, id: &str, predicate: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL keep the ledger consistent\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | locked | invariant | `**rust:** state != \"open\"` | [[{id}]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- held\n\
             - confirmed\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | confirm | held | confirmed | [[{id}.locked]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p | unit | [[{id}.locked]] | `g()` | `{predicate}` |\n"
        ),
    )
    .unwrap_p();
}

fn compile(spec: &Path, out_dir: &Path) {
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
}

fn model_check(spec: &Path, out_dir: &Path) {
    spk()
        .args([
            "model-check",
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out_dir.to_str().unwrap_p(),
        ])
        .assert()
        .success();
}

/// One oracle corpus: a conforming trace and a contradicting one (the
/// declared target of `confirm` is `confirmed`, the trace observes `held`).
fn write_corpus(path: &Path) {
    std::fs::write(
        path,
        "{\"id\": \"confirm-happy\", \"setup\": {\"state\": \"held\"}, \"trace\": [{\"action\": \"confirm\", \"observations\": {\"state\": \"confirmed\"}}]}\n\
         {\"id\": \"wrong-obs\", \"setup\": {\"state\": \"held\"}, \"trace\": [{\"action\": \"confirm\", \"observations\": {\"state\": \"held\"}}]}\n",
    )
    .unwrap_p();
}

fn check_json(
    spec: &Path,
    out_dir: &Path,
    extra: &[&str],
) -> (Option<i32>, serde_json::Value, Vec<u8>) {
    let out = spk()
        .args([
            "check",
            spec.to_str().unwrap_p(),
            "--out-dir",
            out_dir.to_str().unwrap_p(),
            "--json",
        ])
        .args(extra)
        .output()
        .unwrap_p();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "check emits a JSON envelope ({e}): {}",
            String::from_utf8_lossy(&out.stdout)
        )
    });
    (out.status.code(), json, out.stdout)
}

fn row<'a>(json: &'a serde_json::Value, key: &str) -> &'a serde_json::Value {
    json["data"]["records"]
        .as_array()
        .unwrap_p()
        .iter()
        .find(|r| r["scenario_id"] == key)
        .unwrap_or_else(|| panic!("row {key} present: {json}"))
}

fn verified_fixture(td: &tempfile::TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let spec = td.path().join("demo.check.md");
    write_spec(&spec, "demo.check", "**rust:** true");
    let out = td.path().join("out");
    compile(&spec, &out);
    model_check(&spec, &out);
    (spec, out)
}

#[test]
fn verified_run_reports_rung_four_and_spec_internal_scope() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = verified_fixture(&td);
    let (code, json, _) = check_json(&spec, &out, &[]);
    assert_eq!(
        code,
        Some(0),
        "both gates green ⇒ a clean judgement: {json}"
    );
    assert_eq!(json["ok"], serde_json::json!(true));
    assert_eq!(json["data"]["ladder_rung"]["rung"], serde_json::json!(4));
    assert_eq!(json["data"]["scope_sha256"], serde_json::json!(SCOPE_EMPTY));
    assert!(
        json["data"]["evidence_scope"]
            .as_str()
            .unwrap_p()
            .contains("no oracle corpus supplied"),
        "the no-oracle scope is stated: {json}"
    );
    let model = row(&json, "verify:model");
    assert_eq!(model["conform_verdict"], "permitted");
    assert_eq!(model["class"], "current");
    let prop = row(&json, "verify:prop:p");
    assert_eq!(prop["conform_verdict"], "permitted");
    assert_eq!(prop["class"], "current");
}

#[test]
fn oracle_contradiction_exits_one_with_both_layers() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = verified_fixture(&td);
    let corpus = td.path().join("scenarios.jsonl");
    write_corpus(&corpus);
    let (code, json, _) = check_json(&spec, &out, &["--oracle", corpus.to_str().unwrap_p()]);
    assert_eq!(code, Some(1), "a contradicted record fails the run: {json}");
    assert_eq!(json["ok"], serde_json::json!(false));
    let bad = row(&json, "wrong-obs");
    assert_eq!(bad["class"], "contradicted");
    assert_eq!(bad["conform_verdict"], "forbidden");
    let good = row(&json, "confirm-happy");
    assert_eq!(good["class"], "current");
    assert_eq!(good["conform_verdict"], "permitted");
    // Verify-stage rows ride the same report: one vocabulary, both layers.
    assert_eq!(row(&json, "verify:model")["class"], "current");
    for record in json["data"]["records"].as_array().unwrap_p() {
        assert!(record.get("class").is_some() && record.get("conform_verdict").is_some());
    }
}

#[test]
fn missing_model_report_refuses_exit_two_with_hint() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("demo.check.md");
    write_spec(&spec, "demo.check", "**rust:** true");
    let out = td.path().join("out");
    compile(&spec, &out); // no model-check run
    let (code, json, stdout) = check_json(&spec, &out, &[]);
    assert_eq!(code, Some(2), "no judgement possible ⇒ exit 2: {json}");
    assert_eq!(json["ok"], serde_json::json!(false));
    assert!(json["data"].is_null(), "refusal emits zero records: {json}");
    let rendered = String::from_utf8_lossy(&stdout);
    assert!(
        rendered.contains("model-check"),
        "hint names the rerun: {rendered}"
    );
    assert!(
        !rendered.contains("inconclusive"),
        "reserved value never spelled"
    );
}

#[test]
fn failed_property_block_exits_one_and_does_not_lift_rung() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("demo.check.md");
    write_spec(&spec, "demo.check", "**rust:** false");
    let out = td.path().join("out");
    compile(&spec, &out);
    model_check(&spec, &out);
    let (code, json, _) = check_json(&spec, &out, &[]);
    assert_eq!(code, Some(1), "a failed block is a contradiction: {json}");
    let prop = row(&json, "verify:prop:p");
    assert_eq!(prop["conform_verdict"], "forbidden");
    assert_eq!(prop["class"], "contradicted");
    assert_ne!(
        json["data"]["ladder_rung"]["rung"],
        serde_json::json!(4),
        "a failed verify never reaches rung 4: {json}"
    );
}

#[test]
fn killed_runner_refuses_exit_two() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("demo.check.md");
    write_spec(
        &spec,
        "demo.check",
        "**rust:** { std::thread::sleep(std::time::Duration::from_secs(30)); true }",
    );
    let out = td.path().join("out");
    compile(&spec, &out);
    model_check(&spec, &out);
    let (code, json, stdout) = check_json(&spec, &out, &["--timeout-secs", "2"]);
    assert_eq!(code, Some(2), "a terminated run is not a verdict: {json}");
    assert!(json["data"].is_null(), "zero records: {json}");
    let rendered = String::from_utf8_lossy(&stdout);
    assert!(rendered.contains("2s wall-clock bound"), "{rendered}");
    assert!(!rendered.contains("inconclusive"), "{rendered}");
}

#[test]
fn check_report_is_a_valid_audit_prior_and_sustained_contradiction_fails() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = verified_fixture(&td);
    let corpus = td.path().join("scenarios.jsonl");
    write_corpus(&corpus);
    let corpus_arg = corpus.to_str().unwrap_p();
    // The audit baseline (its own exit is 1 for the contradiction).
    let audit = spk()
        .args([
            "audit",
            spec.to_str().unwrap_p(),
            "--oracle",
            corpus_arg,
            "--out-dir",
            out.to_str().unwrap_p(),
            "--json",
        ])
        .output()
        .unwrap_p();
    let baseline = td.path().join("audit.json");
    std::fs::write(&baseline, &audit.stdout).unwrap_p();
    // check consumes an audit report as its prior; the contradiction
    // persists, so the run still fails (the diff's overlay relabels rows,
    // the carried judgement does not).
    let (code, json, _) = check_json(
        &spec,
        &out,
        &[
            "--oracle",
            corpus_arg,
            "--before",
            baseline.to_str().unwrap_p(),
        ],
    );
    assert_eq!(code, Some(1), "sustained contradiction still fails: {json}");
    assert!(
        json["warnings"][0]["message"]
            .as_str()
            .unwrap_p()
            .contains("contradicted record(s)"),
        "the contradiction is named even though the overlay relabels it: {json}"
    );
    // Conversely an audit run consumes a check report as its prior.
    let (_, check_json_out, _) = check_json(&spec, &out, &["--oracle", corpus_arg]);
    let prior = td.path().join("check.json");
    std::fs::write(&prior, serde_json::to_vec(&check_json_out).unwrap_p()).unwrap_p();
    let audit_after = spk()
        .args([
            "audit",
            spec.to_str().unwrap_p(),
            "--oracle",
            corpus_arg,
            "--out-dir",
            out.to_str().unwrap_p(),
            "--before",
            prior.to_str().unwrap_p(),
            "--json",
        ])
        .output()
        .unwrap_p();
    assert_ne!(
        audit_after.status.code(),
        Some(2),
        "audit accepts a check report as a prior: {}",
        String::from_utf8_lossy(&audit_after.stdout)
    );
    let parsed: serde_json::Value = serde_json::from_slice(&audit_after.stdout).unwrap_p();
    assert!(
        !parsed["data"].is_null(),
        "the diff emitted a report: {parsed}"
    );
}

#[test]
fn reruns_are_byte_identical() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = verified_fixture(&td);
    let (_, _, first) = check_json(&spec, &out, &[]);
    let (_, _, second) = check_json(&spec, &out, &[]);
    assert_eq!(first, second, "identical inputs serialize identically");
}

#[test]
fn no_adapter_surface_exists() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("demo.check.md");
    write_spec(&spec, "demo.check", "**rust:** true");
    let rejected = spk()
        .args(["check", spec.to_str().unwrap_p(), "--adapter", "some-suite"])
        .output()
        .unwrap_p();
    assert_ne!(rejected.status.code(), Some(0), "no adapter flag exists");
    let stderr = String::from_utf8_lossy(&rejected.stderr);
    assert!(
        stderr.contains("unexpected argument"),
        "clap rejects the unknown flag: {stderr}"
    );
    let help = spk().args(["check", "--help"]).output().unwrap_p();
    let help = String::from_utf8_lossy(&help.stdout);
    assert!(!help.contains("--adapter"), "no adapter surface: {help}");
    assert!(!help.contains("--registry"), "no registry surface: {help}");
}

#[test]
fn lint_dirty_gate_refuses_exit_two() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("demo.bad.md");
    std::fs::write(
        &spec,
        "---\nid: demo.bad\nkind: intent\nstatement: \"the system should maybe work\"\n---\n",
    )
    .unwrap_p();
    let out = td.path().join("out");
    let (code, json, stdout) = check_json(&spec, &out, &[]);
    assert_eq!(code, Some(2), "a lint-dirty file is refused: {json}");
    assert!(json["data"].is_null(), "zero records: {json}");
    assert!(
        String::from_utf8_lossy(&stdout).contains("spk lint"),
        "the gate hint names lint: {}",
        String::from_utf8_lossy(&stdout)
    );
}

/// A prose-guarded trace: `unknown` by default, `agree` under
/// `--structural-only` — the flag must reach conform's walk.
fn write_prose_spec(path: &Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL classify prose-guarded traces honestly\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | gate | invariant | `a lock is held` | [[{id}]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- held\n\
             - confirmed\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | confirm | held | confirmed | [[{id}.gate]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p | unit | [[{id}.gate]] | `g()` | `**rust:** true` |\n"
        ),
    )
    .unwrap_p();
}

#[test]
fn structural_only_reaches_conforms_walk() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("demo.prose.md");
    write_prose_spec(&spec, "demo.prose");
    let out = td.path().join("out");
    compile(&spec, &out);
    model_check(&spec, &out);
    let corpus = td.path().join("trace.jsonl");
    std::fs::write(
        &corpus,
        "{\"id\": \"t1\", \"setup\": {\"state\": \"held\"}, \"trace\": [{\"action\": \"confirm\", \"observations\": {\"state\": \"confirmed\"}}]}\n",
    )
    .unwrap_p();
    let corpus_arg = corpus.to_str().unwrap_p();
    let (code_default, default_json, _) = check_json(&spec, &out, &["--oracle", corpus_arg]);
    assert_eq!(code_default, Some(0), "{default_json}");
    assert_eq!(row(&default_json, "t1")["conform_verdict"], "unknown");
    let (code_struct, struct_json, _) =
        check_json(&spec, &out, &["--oracle", corpus_arg, "--structural-only"]);
    assert_eq!(code_struct, Some(0), "{struct_json}");
    assert_eq!(row(&struct_json, "t1")["conform_verdict"], "agree");
    assert_eq!(row(&struct_json, "t1")["class"], "unchecked");
}

/// Every file under `dir`, sorted by relative path, as (path, bytes).
fn snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in std::fs::read_dir(&path).unwrap_p().flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p.strip_prefix(dir).unwrap_p().to_string_lossy().to_string();
                entries.push((rel, std::fs::read(&p).unwrap_p()));
            }
        }
    }
    entries.sort();
    entries
}

#[test]
fn refusals_leave_spec_and_artifacts_byte_identical() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("demo.check.md");
    write_spec(&spec, "demo.check", "**rust:** true");
    let out = td.path().join("out");
    compile(&spec, &out); // no model-check — the run refuses
    let spec_before = std::fs::read(&spec).unwrap_p();
    let out_before = snapshot(&out);
    let (code, json, _) = check_json(&spec, &out, &[]);
    assert_eq!(code, Some(2), "{json}");
    assert_eq!(std::fs::read(&spec).unwrap_p(), spec_before);
    assert_eq!(
        snapshot(&out),
        out_before,
        "no artifact written or rewritten"
    );
}

#[test]
fn empty_oracle_corpus_refuses_exit_two() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = verified_fixture(&td);
    let corpus = td.path().join("empty.jsonl");
    std::fs::write(&corpus, "\n\n").unwrap_p();
    let (code, json, _) = check_json(&spec, &out, &["--oracle", corpus.to_str().unwrap_p()]);
    assert_eq!(
        code,
        Some(2),
        "a requested-but-empty evidence set is a refusal: {json}"
    );
    assert!(json["data"].is_null(), "zero records: {json}");
}
