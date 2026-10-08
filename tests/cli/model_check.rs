// (split from tests/cli.rs — specodelic-g17 file_lines ratchet)
use super::*;

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
    // Meter contract: .data.outcome for the single-file case. The native
    // backend executes no invariant predicates, so an exhaustive run is
    // `exploration_only` — never the false-green `no_counterexample`
    // (specodelic-len).
    assert_eq!(data["outcome"], "exploration_only");
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
    assert_eq!(report["outcome"], "exploration_only");
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
    // 22 corpus spec files (the exemption list's non-spec files are skipped)
    assert_eq!(data["files_compiled"], 22);
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
            "dual-format",
            "packs"
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
    assert_eq!(json["data"]["format_revision"], "specodelic.md Revision 17");
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
fn explain_references_documents_file_qualified_refs() {
    // gh#3: the references topic must teach the file-qualification law —
    // [[<file-id>.<row-id>]], the `spec.` self-file prefix for dual-format
    // deltas, and that bare ids / bare text do not resolve.
    let out = spk()
        .args(["explain", "references", "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    let body = json["data"]["body"].as_str().unwrap();
    assert!(body.contains("file-qualified"));
    assert!(body.contains("[[<file-id>.<row-id>]]"));
    assert!(body.contains("[[spec."));
    assert!(body.to_lowercase().contains("bare"));
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
    assert_eq!(json["data"]["format_revision"], "specodelic.md Revision 17");
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
    assert_eq!(json["data"]["format_revision"], "specodelic.md Revision 17");
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
    // Revision 11 → warning naming both revisions, exit 0 (never fails)
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
        warnings.iter().any(|w| w.contains("99") && w.contains("9")),
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

// ---- specodelic-68m.1: claim gates govern the aggregate verdict ----
// (define-verification-claim-gates tasks 1.1; design D1/D2). The
// required-claim statuses — executable fragments, resolved citations,
// kernel claims — must govern the aggregate: a refuted required claim
// is counterexample_found, an unknown or missing one is
// exploration_only (never clean), a nonempty all-verified set over a
// completed bounded exploration is no_counterexample, prose-only rows
// stay explicitly unchecked, and an exhausted bound stays timed_out.
// The verdict gate follows: verify rejects every non-clean aggregate.

/// A lint-clean claim fixture: verbatim constraint rows plus one
/// coverage property per row, the transition guard citing `guard_row`
/// (the kernel_corpus.rs fixture shape).
fn write_claim_spec(path: &std::path::Path, id: &str, rows: &str, props: &str, guard_row: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL carry claim-gate fixtures\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             {rows}\
             \n## Model\n\
             \n### States\n\
             \n- s1\n\
             - s2\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | t | s1 | s2 | [[{id}.{guard_row}]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             {props}"
        ),
    )
    .unwrap();
}

/// The coverage property rows for the named constraint rows.
fn claim_props(id: &str, rows: &[&str]) -> String {
    rows.iter()
        .map(|r| {
            format!("| p_{r} | unit | [[{id}.{r}]] | `word()` | `**rust:** v0.len() >= 1` |\n")
        })
        .collect()
}

fn claim_compile(spec: &str, out: &std::path::Path) {
    spk()
        .args(["compile", spec, "--out-dir", out.to_str().unwrap()])
        .assert()
        .success();
}

/// Run model-check over one spec and return (exit code, parsed envelope).
fn claim_mc_json(spec: &str, out: &std::path::Path) -> (Option<i32>, serde_json::Value) {
    let result = spk()
        .args([
            "model-check",
            spec,
            "--json",
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    (result.status.code(), json)
}

/// Run verify over one spec and return (exit code, parsed envelope).
fn claim_verify_json(spec: &str, out: &std::path::Path) -> (Option<i32>, serde_json::Value) {
    let result = spk()
        .args(["verify", spec, "--json", "--out-dir", out.to_str().unwrap()])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    (result.status.code(), json)
}

/// The persisted run report for one stem.
fn claim_report(out: &std::path::Path, stem: &str) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(out.join(format!("{stem}.check.json"))).unwrap())
        .unwrap()
}

#[test]
fn false_kernel_claim_forces_counterexample_found() {
    // ka refutes (both rows trace to the same target — unique is
    // refuted), kb verifies: the refuted required claim must govern.
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("ckf.md");
    let rows = "| ka | invariant | `**kernel:** unique(traces_to)` | [[ckf]] |\n\
                | kb | invariant | `**kernel:** resolves(traces_to)` | [[ckf]] |\n";
    write_claim_spec(&spec, "ckf", rows, &claim_props("ckf", &["ka", "kb"]), "ka");
    let out = td.path().join("out");
    claim_compile(spec.to_str().unwrap(), &out);
    let (code, json) = claim_mc_json(spec.to_str().unwrap(), &out);
    // Completed runs keep model-check's CLI exit convention; the
    // aggregate verdict lives in the report, not the exit code.
    assert_eq!(code, Some(0));
    assert_eq!(json["data"]["outcome"], "counterexample_found");
    let checked = &json["data"]["checked"][0];
    assert_eq!(checked["outcome"], "counterexample_found");
    assert_eq!(checked["violated_invariant_id"], "ka");
    let statuses: Vec<(String, String)> = checked["invariant_statuses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            (
                s["id"].as_str().unwrap().to_string(),
                s["status"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert!(
        statuses.contains(&("ka".into(), "counterexample".into())),
        "{statuses:?}"
    );
    assert!(
        statuses.contains(&("kb".into(), "verified".into())),
        "{statuses:?}"
    );
    // The persisted report carries the same aggregate.
    let report = claim_report(&out, "ckf");
    assert_eq!(report["outcome"], "counterexample_found");
    assert_eq!(report["violated_invariant_id"], "ka");
    // The verify gate rejects the non-clean aggregate.
    let (vcode, vjson) = claim_verify_json(spec.to_str().unwrap(), &out);
    assert_eq!(vcode, Some(1));
    assert_eq!(vjson["data"]["status"], "model_not_clean");
}

#[test]
fn false_negated_citation_forces_counterexample_found() {
    // c1 verifies (s2 reachable is fine — blackhole is not); the
    // negated citation ¬[[c1]] therefore refutes.
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("cnf.md");
    let rows = "| c1 | invariant | `**rust:** state != \"blackhole\"` | [[cnf]] |\n\
                | c2 | invariant | `¬[[cnf.c1]]` | [[cnf]] |\n";
    write_claim_spec(&spec, "cnf", rows, &claim_props("cnf", &["c1", "c2"]), "c1");
    let out = td.path().join("out");
    claim_compile(spec.to_str().unwrap(), &out);
    let (code, json) = claim_mc_json(spec.to_str().unwrap(), &out);
    assert_eq!(code, Some(0));
    assert_eq!(json["data"]["outcome"], "counterexample_found");
    assert_eq!(json["data"]["checked"][0]["violated_invariant_id"], "c2");
    let report = claim_report(&out, "cnf");
    assert_eq!(report["outcome"], "counterexample_found");
    // No invented state trace: the citation counterexample carries
    // claim evidence, not a fabricated BFS path.
    assert!(report.get("trace").is_none() || report["trace"].is_null());
    let (vcode, vjson) = claim_verify_json(spec.to_str().unwrap(), &out);
    assert_eq!(vcode, Some(1));
    assert_eq!(vjson["data"]["status"], "model_not_clean");
}

#[test]
fn unknown_kernel_claim_prevents_clean() {
    // The ghost seed reaches an id outside the corpus — honest unknown.
    // Unknown required claims prevent clean: exploration_only, and
    // verify rejects it.
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("cuk.md");
    let rows = "| g1 | invariant | `**kernel:** reachable(demo.ghost.nowhere, cuk.g1, supersedes)` | [[cuk]] |\n";
    write_claim_spec(&spec, "cuk", rows, &claim_props("cuk", &["g1"]), "g1");
    let out = td.path().join("out");
    claim_compile(spec.to_str().unwrap(), &out);
    let (code, json) = claim_mc_json(spec.to_str().unwrap(), &out);
    assert_eq!(code, Some(0));
    assert_eq!(json["data"]["outcome"], "exploration_only");
    let statuses = &json["data"]["checked"][0]["invariant_statuses"];
    assert_eq!(statuses[0]["id"], "g1");
    assert_eq!(statuses[0]["status"], "unknown");
    assert_eq!(claim_report(&out, "cuk")["outcome"], "exploration_only");
    let (vcode, vjson) = claim_verify_json(spec.to_str().unwrap(), &out);
    assert_eq!(vcode, Some(1));
    assert_eq!(vjson["data"]["status"], "model_not_clean");
}

#[test]
fn missing_evidence_citation_prevents_clean_and_names_the_reason() {
    // A property row has no same-run invariant evidence — the required
    // claim is unknown with the labeled reason, so the aggregate is
    // exploration_only even though the exec leg reported clean facts.
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("cme.md");
    let rows = "| c1 | invariant | `**rust:** state != \"blackhole\"` | [[cme]] |\n\
                | c2 | invariant | `[[p_c1]]` | [[cme]] |\n";
    write_claim_spec(&spec, "cme", rows, &claim_props("cme", &["c1", "c2"]), "c1");
    let out = td.path().join("out");
    claim_compile(spec.to_str().unwrap(), &out);
    let (code, json) = claim_mc_json(spec.to_str().unwrap(), &out);
    assert_eq!(code, Some(0));
    assert_eq!(json["data"]["outcome"], "exploration_only");
    let statuses = &json["data"]["checked"][0]["invariant_statuses"];
    let c2 = statuses
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "c2")
        .unwrap();
    assert_eq!(c2["status"], "unknown");
    assert!(
        c2["reason"]
            .as_str()
            .unwrap()
            .contains("same-run invariant evidence"),
        "labeled reason: {c2}"
    );
    assert_eq!(claim_report(&out, "cme")["outcome"], "exploration_only");
    let (vcode, _) = claim_verify_json(spec.to_str().unwrap(), &out);
    assert_eq!(vcode, Some(1));
}

#[test]
fn all_true_claims_over_completed_exploration_verify() {
    // THE acceptance-policy flip (BREAKING): an empty-supersedes corpus
    // verifies acyclic and resolves over the v0 snapshot — every
    // required claim discharged, exploration completed, so the
    // aggregate is no_counterexample and BOTH gates verify end to end.
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("cat.md");
    let rows = "| ka | invariant | `**kernel:** acyclic(supersedes)` | [[cat]] |\n\
                | kb | invariant | `**kernel:** resolves(traces_to)` | [[cat]] |\n";
    write_claim_spec(&spec, "cat", rows, &claim_props("cat", &["ka", "kb"]), "ka");
    let out = td.path().join("out");
    claim_compile(spec.to_str().unwrap(), &out);
    let (code, json) = claim_mc_json(spec.to_str().unwrap(), &out);
    assert_eq!(code, Some(0));
    assert_eq!(json["data"]["outcome"], "no_counterexample");
    let statuses = &json["data"]["checked"][0]["invariant_statuses"];
    for s in statuses.as_array().unwrap() {
        assert_eq!(s["status"], "verified", "claim {}: {}", s["id"], s);
    }
    assert_eq!(claim_report(&out, "cat")["outcome"], "no_counterexample");
    let (vcode, vjson) = claim_verify_json(spec.to_str().unwrap(), &out);
    assert_eq!(vcode, Some(0));
    assert_eq!(vjson["data"]["status"], "verified");
}

#[test]
fn exhausted_bound_stays_timed_out_despite_verified_claims() {
    // D2 ordering: an exhausted budget outranks the all-verified leg —
    // a truncated exploration proves nothing about the claims.
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("cex.md");
    let rows = "| ka | invariant | `**kernel:** acyclic(supersedes)` | [[cex]] |\n\
                | kb | invariant | `**kernel:** resolves(traces_to)` | [[cex]] |\n";
    write_claim_spec(&spec, "cex", rows, &claim_props("cex", &["ka", "kb"]), "ka");
    let out = td.path().join("out");
    claim_compile(spec.to_str().unwrap(), &out);
    let result = spk()
        .args([
            "model-check",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            out.to_str().unwrap(),
            "--max-depth",
            "1",
        ])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(json["data"]["outcome"], "timed_out");
    assert_eq!(claim_report(&out, "cex")["outcome"], "timed_out");
    let (vcode, _) = claim_verify_json(spec.to_str().unwrap(), &out);
    assert_eq!(vcode, Some(1));
}

#[test]
fn no_required_claims_stay_explicitly_unchecked() {
    // D1: prose invariant rows are never required claims — no evaluator
    // opted in, no status fabricated. The empty required set reports
    // exploration_only and verify rejects it (D2 rule 5).
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("cpr.md");
    write_model_check_spec(&spec, "cpr");
    let out = td.path().join("out");
    claim_compile(spec.to_str().unwrap(), &out);
    let (code, json) = claim_mc_json(spec.to_str().unwrap(), &out);
    assert_eq!(code, Some(0));
    assert_eq!(json["data"]["outcome"], "exploration_only");
    assert!(
        json["data"]["checked"][0]["invariant_statuses"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(claim_report(&out, "cpr")["outcome"], "exploration_only");
    let (vcode, vjson) = claim_verify_json(spec.to_str().unwrap(), &out);
    assert_eq!(vcode, Some(1));
    // Both gates are always evaluated; this fixture's parameterless
    // property blocker is named first, so the model-gate leg is asserted
    // directly: the empty-required-set report is not clean.
    assert_eq!(vjson["data"]["blocked"][0]["model"]["state"], "not_clean");
}

// ---- specodelic-68m.5: users see the same claim blockers in every view ----
// (define-verification-claim-gates tasks 3.1; design D5 visible_scope).
// The CLI JSON envelope, the persisted `.check.json` report, the human
// output, and verify's view must state the SAME evaluated, unchecked, and
// blocking claim sets on mixed fixtures. The docs-consistency fixture
// fails when a version literal or capability status in the release docs
// conflicts with the Cargo metadata (D5: versions derive from Cargo
// metadata, never independently maintained literals).

/// Mixed fixture: one verified Rust claim, one unknown kernel claim, and
/// one prose-only invariant (explicitly unchecked). The shared fixture
/// shape of the parity tests below.
fn write_view_parity_spec(path: &std::path::Path, id: &str, ka_expr: &str) {
    let rows = format!(
        "| ka | invariant | `{ka_expr}` | [[{id}]] |\n\
          | kb | invariant | `**rust:** state != \"blackhole\"` | [[{id}]] |\n\
          | kp | invariant | prose guard holds across states | [[{id}]] |\n"
    );
    write_claim_spec(path, id, &rows, &claim_props(id, &["ka", "kb", "kp"]), "ka");
}

#[test]
fn mixed_fixture_views_agree_on_blockers_and_unchecked() {
    // Verified Rust + unknown kernel + prose-only unchecked: every view
    // names ka as the blocker, kp as unchecked, and the same counts.
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("cmx.md");
    write_view_parity_spec(
        &spec,
        "cmx",
        "**kernel:** reachable(demo.ghost.nowhere, cmx.ka, supersedes)",
    );
    let out = td.path().join("out");
    claim_compile(spec.to_str().unwrap(), &out);

    // JSON envelope vs persisted report: identical claim fields.
    let (code, json) = claim_mc_json(spec.to_str().unwrap(), &out);
    assert_eq!(code, Some(0));
    assert_eq!(json["data"]["outcome"], "exploration_only");
    let checked = &json["data"]["checked"][0];
    let expected = checked["expected_claim_ids"]
        .as_array()
        .expect("JSON envelope carries the required set")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(expected, vec!["ka".to_string(), "kb".to_string()]);
    let unchecked = checked["unchecked_claim_ids"]
        .as_array()
        .expect("JSON envelope carries the unchecked set")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(unchecked, vec!["kp".to_string()]);
    let claims = checked["claims"]
        .as_array()
        .expect("JSON carries claim records");
    assert_eq!(claims.len(), 2, "evaluated claims only: {claims:?}");
    let ka = claims.iter().find(|c| c["id"] == "ka").unwrap();
    assert_eq!(ka["evaluator"], "kernel");
    assert_eq!(ka["status"], "unknown");
    // Native kernel unknowns are honest unknowns without a synthetic
    // reason label (labeled reasons exist for unsupported backends,
    // parse failures and missing citation evidence — D2/D3 semantics
    // are not this ticket's); a reason, when present, is a string.
    if let Some(reason) = ka.get("reason") {
        assert!(reason.is_string(), "reason is a string when present: {ka}");
    }
    let kb = claims.iter().find(|c| c["id"] == "kb").unwrap();
    assert_eq!(kb["evaluator"], "rust");
    assert_eq!(kb["status"], "verified");
    let report = claim_report(&out, "cmx");
    assert_eq!(report["expected_claim_ids"], checked["expected_claim_ids"]);
    assert_eq!(
        report["unchecked_claim_ids"],
        checked["unchecked_claim_ids"]
    );
    assert_eq!(report["claims"], checked["claims"]);

    // Human output: the same blocker and unchecked claim by id.
    let human = spk()
        .args([
            "model-check",
            spec.to_str().unwrap(),
            "--human",
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(human.stdout).unwrap();
    assert!(
        stdout.contains("ka"),
        "human names the blocking claim: {stdout}"
    );
    assert!(
        stdout.contains("unknown"),
        "human names the blocker status: {stdout}"
    );
    assert!(
        stdout.contains("kp"),
        "human names the unchecked claim: {stdout}"
    );
    assert!(
        stdout.contains("2"),
        "human states the evaluated count: {stdout}"
    );

    // Verify's view: the blocked entry names the same blocker and the
    // same unchecked set.
    let (vcode, vjson) = claim_verify_json(spec.to_str().unwrap(), &out);
    assert_eq!(vcode, Some(1));
    let model = &vjson["data"]["blocked"][0]["model"];
    assert_eq!(model["state"], "not_clean");
    let blocking: Vec<String> = model["blocking_claims"]
        .as_array()
        .expect("verify names the blocking claims")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(blocking, vec!["ka".to_string()]);
    let v_unchecked: Vec<String> = model["unchecked_claim_ids"]
        .as_array()
        .expect("verify names the unchecked claims")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(v_unchecked, vec!["kp".to_string()]);
    let v_human = spk()
        .args([
            "verify",
            spec.to_str().unwrap(),
            "--human",
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let v_stdout = String::from_utf8(v_human.stdout).unwrap();
    assert!(
        v_stdout.contains("ka"),
        "verify human names the blocker: {v_stdout}"
    );
    assert!(
        v_stdout.contains("kp"),
        "verify human names the unchecked claim: {v_stdout}"
    );
}

#[test]
fn refuted_kernel_blocker_names_the_same_claim_in_every_view() {
    // Verified resolves + refuted unique: ka is the blocker in JSON,
    // the persisted report, human output, and verify's blocked entry.
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("crf.md");
    write_view_parity_spec(&spec, "crf", "**kernel:** unique(traces_to)");
    let out = td.path().join("out");
    claim_compile(spec.to_str().unwrap(), &out);
    let (code, json) = claim_mc_json(spec.to_str().unwrap(), &out);
    assert_eq!(code, Some(0));
    assert_eq!(json["data"]["outcome"], "counterexample_found");
    let checked = &json["data"]["checked"][0];
    let claims = checked["claims"]
        .as_array()
        .expect("JSON carries claim records");
    let ka = claims.iter().find(|c| c["id"] == "ka").unwrap();
    assert_eq!(ka["status"], "counterexample");
    let kb = claims.iter().find(|c| c["id"] == "kb").unwrap();
    assert_eq!(kb["status"], "verified");
    let report = claim_report(&out, "crf");
    assert_eq!(report["claims"], checked["claims"]);
    let human = spk()
        .args([
            "model-check",
            spec.to_str().unwrap(),
            "--human",
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(human.stdout).unwrap();
    assert!(
        stdout.contains("ka"),
        "human names the refuted claim: {stdout}"
    );
    assert!(
        stdout.contains("counterexample"),
        "human names the refuted status: {stdout}"
    );
    let (vcode, vjson) = claim_verify_json(spec.to_str().unwrap(), &out);
    assert_eq!(vcode, Some(1));
    let model = &vjson["data"]["blocked"][0]["model"];
    let blocking: Vec<String> = model["blocking_claims"]
        .as_array()
        .expect("verify names the blocking claims")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(blocking, vec!["ka".to_string()]);
}

#[test]
fn docs_carry_release_version_and_implemented_capability_status() {
    // D5: versions derive from Cargo metadata, never independently
    // maintained literals; the release docs state the implemented
    // claim-gate capability and the application-test distinction.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let version = env!("CARGO_PKG_VERSION");
    let docs = [
        "README.md",
        "docs/src/status.md",
        "openspec/project.md",
        "specs/STATUS.md",
    ];
    // Release-version statements only: `v<semver>` or `Version <semver>`
    // — a bare x.y.z elsewhere (e.g. "Invariant 3.2.5") is not a
    // maintained version literal.
    let literal = regex::Regex::new(r"(?:\bv|Version )(\d+\.\d+\.\d+)").unwrap();
    for doc in docs {
        let path = root.join(doc);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{doc} must be readable: {e}"));
        for cap in literal.captures_iter(&text) {
            let lit = cap.get(1).unwrap().as_str();
            assert_eq!(
                lit, version,
                "{doc} carries version literal {lit} but the release is {version} — versions derive from Cargo metadata"
            );
        }
    }
    // Capability status: the implemented claim-gate behavior is stated
    // where verify is described, and verification is distinguished from
    // application-test execution. Phrase assertions are whitespace-
    // normalized — markdown wrapping is not a capability change.
    let norm = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    let readme = norm(&std::fs::read_to_string(root.join("README.md")).unwrap());
    assert!(
        readme.contains("every opted-in invariant claim"),
        "README's verify status must state the implemented claim gate"
    );
    assert!(
        readme.contains("not a substitute for"),
        "README must distinguish verification from application-test execution"
    );
    for doc in [
        "docs/src/status.md",
        "openspec/project.md",
        "specs/STATUS.md",
    ] {
        let text = norm(&std::fs::read_to_string(root.join(doc)).unwrap());
        assert!(
            text.contains("invariant claim"),
            "{doc} must state the implemented claim-gate capability"
        );
    }
}

// ---- specodelic-68m.3: evidence belongs to the current scope ----
// (define-verification-claim-gates tasks 2.1; design D3). A stored
// report binds claim_schema_version 1, its qualified claim records and
// a scope digest of the structured inputs plus consumed artifacts;
// verify recomputes both from the live invocation and rejects stale,
// foreign or old evidence with a rerun hint — never rewriting a stored
// report to manufacture evidence. Reordering CLI paths and prose-only
// edits preserve the digest; separate dual-format runs bind their own
// structured contents, so swapped reports are rejected.

/// Kernel claims that all verify over the v0 snapshot — the freshness
/// fixtures' clean-evidence shape, evaluated in-process (no cargo on the
/// model-check leg, so the fixture is deterministic under parallel load).
fn scope_verified_rows(id: &str) -> String {
    format!(
        "| ka | invariant | `**kernel:** acyclic(supersedes)` | [[{id}]] |\n\
         | kb | invariant | `**kernel:** resolves(traces_to)` | [[{id}]] |\n"
    )
}

fn scope_props(id: &str) -> String {
    "| p_ka | unit | [[".to_string()
        + id
        + ".ka]] | `word()` | `**rust:** v0.len() >= 1` |\n"
        + "| p_kb | unit | [["
        + id
        + ".kb]] | `word()` | `**rust:** v0.len() >= 1` |\n"
}

fn write_scope_spec(path: &std::path::Path, id: &str, rows: &str, props: &str) {
    write_claim_spec(path, id, rows, props, "ka");
}

/// A two-file ordinary corpus, each file verified clean — the starting
/// point for every scope-freshness fixture.
fn write_scope_pair(dir: &std::path::Path) -> (String, String) {
    let a = dir.join("scp_a.md");
    let b = dir.join("scp_b.md");
    write_scope_spec(
        &a,
        "scp_a",
        &scope_verified_rows("scp_a"),
        &scope_props("scp_a"),
    );
    write_scope_spec(
        &b,
        "scp_b",
        &scope_verified_rows("scp_b"),
        &scope_props("scp_b"),
    );
    (
        a.to_str().unwrap().to_string(),
        b.to_str().unwrap().to_string(),
    )
}

/// Compile a batch into `out` (model-check consumes compile's output —
/// it never re-compiles).
fn scope_compile(specs: &[&str], out: &std::path::Path) {
    let mut args = vec!["compile".to_string()];
    args.extend(specs.iter().map(|s| s.to_string()));
    args.push("--out-dir".to_string());
    args.push(out.to_str().unwrap().to_string());
    spk().args(&args).assert().success();
}

/// Model-check a batch, returning (exit code, parsed envelope).
/// Remove the compiled props artifacts so verify's properties gate is
/// deterministically `missing_artifact` (never a real cargo run) — the
/// freshness fixtures pin the model gate, not the properties gate.
fn drop_props(out: &std::path::Path, stems: &[&str]) {
    for stem in stems {
        let _ = std::fs::remove_file(out.join(format!("{stem}_props.rs")));
    }
}

fn mc_batch(specs: &[&str], out: &std::path::Path) -> (Option<i32>, serde_json::Value) {
    let mut args = vec!["model-check".to_string()];
    args.extend(specs.iter().map(|s| s.to_string()));
    args.extend([
        "--json".to_string(),
        "--out-dir".to_string(),
        out.to_str().unwrap().to_string(),
    ]);
    let result = spk().args(&args).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    let code = result.status.code();
    assert_eq!(
        code,
        Some(0),
        "model-check must produce the report the fixture manipulates: {json}"
    );
    (code, json)
}

/// Verify a batch, returning (exit code, parsed envelope).
fn verify_batch(specs: &[&str], out: &std::path::Path) -> (Option<i32>, serde_json::Value) {
    let mut args = vec!["verify".to_string()];
    args.extend(specs.iter().map(|s| s.to_string()));
    args.extend([
        "--json".to_string(),
        "--out-dir".to_string(),
        out.to_str().unwrap().to_string(),
    ]);
    let result = spk().args(&args).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    (result.status.code(), json)
}

/// The persisted run report for the given stem.
fn read_check_report(out: &std::path::Path, stem: &str) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(out.join(format!("{stem}.check.json"))).unwrap())
        .unwrap()
}

fn write_check_report(out: &std::path::Path, stem: &str, report: &serde_json::Value) {
    std::fs::write(
        out.join(format!("{stem}.check.json")),
        serde_json::to_string_pretty(report).unwrap(),
    )
    .unwrap();
}

/// The scope digest a stored report carries — fails with the intended
/// reason when the report has none (the RED signal for 68m.3 GREEN).
fn scope_digest_of(report: &serde_json::Value) -> String {
    report["scope_sha256"]
        .as_str()
        .unwrap_or_else(|| panic!("stored report carries no scope_sha256 fingerprint: {report}"))
        .to_string()
}

/// A dual-format id:spec delta whose c1 exec invariant verifies or
/// refutes — identical local claim ids (c1, c2) either way, opposite
/// outcomes.
fn write_dual68(dir: &std::path::Path, passing: bool) -> String {
    std::fs::create_dir_all(dir).unwrap();
    let fragment = if passing {
        "state != \"blackhole\""
    } else {
        "state != \"s2\""
    };
    let path = dir.join("spec.md");
    std::fs::write(
        &path,
        format!(
            "---\nid: spec\nkind: intent\nstatement: \"THE delta SHALL be a scope-isolation fixture\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | c1 | invariant | `**rust:** {fragment}` | [[spec]] |\n\
             | c2 | invariant | `[[spec.c1]]` | [[spec]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- s1\n\
             - s2\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | t | s1 | s2 | [[spec.c1]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p_c1 | unit | [[spec.c1]] | `word()` | `**rust:** v0.len() >= 1` |\n\
             | p_c2 | unit | [[spec.c2]] | `word()` | `**rust:** v0.len() >= 1` |\n"
        ),
    )
    .unwrap();
    path.to_str().unwrap().to_string()
}

#[test]
fn report_binds_schema_claims_and_scope_digest() {
    let td = tempfile::tempdir().unwrap();
    let (a, _b) = write_scope_pair(td.path());
    let out = td.path().join("out");
    scope_compile(&[a.as_str()], &out);
    let (code, _json) = mc_batch(&[a.as_str()], &out);
    assert_eq!(code, Some(0));
    let report = read_check_report(&out, "scp_a");
    assert_eq!(report["claim_schema_version"], 1);
    let digest = scope_digest_of(&report);
    assert_eq!(digest.len(), 64);
    // Canonical qualified claim records: the required rust claim with
    // its evaluator kind; expected ids match the live required set.
    let claims = report["claims"].as_array().unwrap();
    assert_eq!(claims.len(), 2);
    assert_eq!(claims[0]["id"], "ka");
    assert_eq!(claims[0]["evaluator"], "kernel");
    assert_eq!(claims[0]["status"], "verified");
    assert_eq!(claims[1]["id"], "kb");
    assert_eq!(claims[1]["evaluator"], "kernel");
    assert_eq!(
        report["expected_claim_ids"],
        serde_json::json!(["ka", "kb"])
    );
    assert_eq!(report["unchecked_claim_ids"], serde_json::json!([]));
}

#[test]
fn old_report_requires_model_check_rerun() {
    let td = tempfile::tempdir().unwrap();
    let (a, _b) = write_scope_pair(td.path());
    let out = td.path().join("out");
    scope_compile(&[a.as_str()], &out);
    mc_batch(&[a.as_str()], &out);
    // Simulate a pre-claim-schema report: no version, no scope
    // fingerprint, no claim records.
    let mut old = read_check_report(&out, "scp_a");
    for key in [
        "claim_schema_version",
        "scope_sha256",
        "claims",
        "expected_claim_ids",
        "unchecked_claim_ids",
    ] {
        old.as_object_mut().unwrap().remove(key);
    }
    write_check_report(&out, "scp_a", &old);
    let before = std::fs::read_to_string(out.join("scp_a.check.json")).unwrap();
    drop_props(&out, &["scp_a"]);
    let (vcode, vjson) = verify_batch(&[a.as_str()], &out);
    assert_eq!(vcode, Some(1), "old reports never verify: {vjson}");
    // The MODEL gate is what rejected the evidence — assert it directly
    // (the top-level verdict names whichever gate blocked first).
    let entry = &vjson["data"]["blocked"][0];
    assert_eq!(entry["model"]["state"], "stale");
    assert!(
        entry["model"]["detail"]
            .as_str()
            .unwrap()
            .contains("claim schema"),
        "names the schema gap: {entry}"
    );
    // No report rewriting to manufacture evidence: the stored file is
    // byte-identical after the rejected verify.
    let after = std::fs::read_to_string(out.join("scp_a.check.json")).unwrap();
    assert_eq!(before, after);
}

#[test]
fn unrecognized_claim_schema_version_requires_rerun() {
    let td = tempfile::tempdir().unwrap();
    let (a, _b) = write_scope_pair(td.path());
    let out = td.path().join("out");
    scope_compile(&[a.as_str()], &out);
    mc_batch(&[a.as_str()], &out);
    let mut future = read_check_report(&out, "scp_a");
    future["claim_schema_version"] = serde_json::json!(99);
    write_check_report(&out, "scp_a", &future);
    drop_props(&out, &["scp_a"]);
    let (vcode, vjson) = verify_batch(&[a.as_str()], &out);
    assert_eq!(vcode, Some(1), "unrecognized schema versions never verify");
    let entry = &vjson["data"]["blocked"][0];
    assert_eq!(entry["model"]["state"], "stale");
    assert!(
        entry["model"]["detail"]
            .as_str()
            .unwrap()
            .contains("claim schema"),
        "{entry}"
    );
}

#[test]
fn missing_claim_record_requires_rerun() {
    let td = tempfile::tempdir().unwrap();
    let (a, _b) = write_scope_pair(td.path());
    let out = td.path().join("out");
    scope_compile(&[a.as_str()], &out);
    mc_batch(&[a.as_str()], &out);
    // Drop one required claim's record from the stored evidence.
    let mut report = read_check_report(&out, "scp_a");
    for key in ["claims", "invariant_statuses", "expected_claim_ids"] {
        if let Some(arr) = report[key].as_array_mut() {
            arr.retain(|s| s["id"].as_str() != Some("ka"));
        }
    }
    write_check_report(&out, "scp_a", &report);
    drop_props(&out, &["scp_a"]);
    let (vcode, vjson) = verify_batch(&[a.as_str()], &out);
    assert_eq!(vcode, Some(1), "missing required claims never verify");
    let entry = &vjson["data"]["blocked"][0];
    assert_eq!(entry["model"]["state"], "stale");
    assert!(
        entry["model"]["detail"].as_str().unwrap().contains("ka"),
        "names the missing claim: {entry}"
    );
}

#[test]
fn duplicate_claim_record_rejected() {
    let td = tempfile::tempdir().unwrap();
    let (a, _b) = write_scope_pair(td.path());
    let out = td.path().join("out");
    scope_compile(&[a.as_str()], &out);
    mc_batch(&[a.as_str()], &out);
    let mut report = read_check_report(&out, "scp_a");
    for key in ["claims", "invariant_statuses"] {
        let dup = report[key][0].clone();
        report[key].as_array_mut().unwrap().push(dup);
    }
    write_check_report(&out, "scp_a", &report);
    drop_props(&out, &["scp_a"]);
    let (vcode, vjson) = verify_batch(&[a.as_str()], &out);
    assert_eq!(
        vcode,
        Some(1),
        "duplicate claim records are invalid evidence"
    );
    let entry = &vjson["data"]["blocked"][0];
    assert_eq!(entry["model"]["state"], "stale");
    assert!(
        entry["model"]["detail"]
            .as_str()
            .unwrap()
            .contains("duplicate"),
        "{entry}"
    );
}

#[test]
fn reversed_order_and_prose_edits_preserve_scope_digest() {
    let td = tempfile::tempdir().unwrap();
    let (a, b) = write_scope_pair(td.path());
    let fwd = td.path().join("fwd");
    let rev = td.path().join("rev");
    scope_compile(&[a.as_str(), b.as_str()], &fwd);
    mc_batch(&[a.as_str(), b.as_str()], &fwd);
    scope_compile(&[b.as_str(), a.as_str()], &rev);
    mc_batch(&[b.as_str(), a.as_str()], &rev);
    // CLI file order is excluded: the digest binds structured content.
    let digest_fwd = scope_digest_of(&read_check_report(&fwd, "scp_a"));
    let digest_rev = scope_digest_of(&read_check_report(&rev, "scp_a"));
    assert_eq!(digest_fwd, digest_rev);
    // Prose-only edits never parse, so they preserve the digest too.
    let prose = std::fs::read_to_string(&a).unwrap()
        + "\n## Notes\n\nJust prose — never parsed, never part of the scope.\n";
    std::fs::write(&a, prose).unwrap();
    let prose_out = td.path().join("prose");
    scope_compile(&[a.as_str(), b.as_str()], &prose_out);
    mc_batch(&[a.as_str(), b.as_str()], &prose_out);
    let digest_prose = scope_digest_of(&read_check_report(&prose_out, "scp_a"));
    assert_eq!(digest_fwd, digest_prose);
}

#[test]
fn cross_file_edit_invalidates_clean_report() {
    let td = tempfile::tempdir().unwrap();
    let (a, b) = write_scope_pair(td.path());
    let out = td.path().join("out");
    scope_compile(&[a.as_str(), b.as_str()], &out);
    mc_batch(&[a.as_str(), b.as_str()], &out);
    // b changes structurally after the reports were written.
    let edited = std::fs::read_to_string(&b)
        .unwrap()
        .replace("resolves(traces_to)", "unique(traces_to)");
    std::fs::write(&b, edited).unwrap();
    spk()
        .args(["compile", b.as_str(), "--out-dir", out.to_str().unwrap()])
        .assert()
        .success();
    // a's stored evidence covered the old b: the whole invocation scope
    // is stale, so a's clean report must not verify either.
    drop_props(&out, &["scp_a", "scp_b"]);
    let (vcode, vjson) = verify_batch(&[a.as_str(), b.as_str()], &out);
    assert_eq!(vcode, Some(1));
    let blocked = vjson["data"]["blocked"].as_array().unwrap();
    let a_entry = blocked
        .iter()
        .find(|e| e["id"] == "scp_a")
        .unwrap_or_else(|| panic!("scp_a must be blocked: {vjson}"));
    assert_eq!(a_entry["model"]["state"], "stale");
    assert!(
        a_entry["model"]["detail"]
            .as_str()
            .unwrap()
            .contains("scope"),
        "names the scope mismatch: {a_entry}"
    );
}

#[test]
fn dropped_input_invalidates_report() {
    let td = tempfile::tempdir().unwrap();
    let (a, b) = write_scope_pair(td.path());
    let out = td.path().join("out");
    scope_compile(&[a.as_str(), b.as_str()], &out);
    mc_batch(&[a.as_str(), b.as_str()], &out);
    // Verify with b dropped from the invocation: a's evidence covered a
    // two-file scope, so a alone must not verify.
    drop_props(&out, &["scp_a"]);
    let (vcode, vjson) = verify_batch(&[a.as_str()], &out);
    assert_eq!(vcode, Some(1), "dropping an input invalidates the report");
    let entry = &vjson["data"]["blocked"][0];
    assert_eq!(entry["model"]["state"], "stale");
    assert!(
        entry["model"]["detail"].as_str().unwrap().contains("scope"),
        "{entry}"
    );
}

#[test]
fn artifact_edit_after_report_is_rejected() {
    // Pin: the pre-existing artifact staleness key stays enforced
    // alongside the scope digest — editing the compiled module after the
    // run is an artifact mismatch, failed closed.
    let td = tempfile::tempdir().unwrap();
    let (a, _b) = write_scope_pair(td.path());
    let out = td.path().join("out");
    scope_compile(&[a.as_str()], &out);
    mc_batch(&[a.as_str()], &out);
    let tla = out.join("scp_a.tla");
    std::fs::write(&tla, "MODULE scp_a edited").unwrap();
    drop_props(&out, &["scp_a"]);
    let (vcode, vjson) = verify_batch(&[a.as_str()], &out);
    assert_eq!(vcode, Some(1));
    assert_eq!(vjson["data"]["blocked"][0]["model"]["state"], "stale");
}

#[test]
fn dual_format_opposite_outcomes_isolated_scope_and_swapped_reports() {
    let td = tempfile::tempdir().unwrap();
    let d1 = write_dual68(&td.path().join("d1"), true);
    let d2 = write_dual68(&td.path().join("d2"), false); // opposite outcome
    let out = td.path().join("out");
    // Combined invocation: isolated_scope_required before any writes.
    let combined = spk()
        .args([
            "model-check",
            d1.as_str(),
            d2.as_str(),
            "--json",
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_ne!(combined.status.code(), Some(0));
    let stderr = String::from_utf8(combined.stderr).unwrap();
    assert!(stderr.contains("isolated_scope_required"), "{stderr}");
    let stdout = String::from_utf8(combined.stdout).unwrap();
    assert!(
        stdout.contains("separately") && stdout.contains("--out-dir"),
        "split-run hint: {stdout}"
    );
    assert!(!out.join("spec.check.json").exists());
    // Separate runs in separate directories preserve each outcome.
    let out1 = td.path().join("out1");
    let out2 = td.path().join("out2");
    scope_compile(&[d1.as_str()], &out1);
    scope_compile(&[d2.as_str()], &out2);
    let (c1, _) = mc_batch(&[d1.as_str()], &out1);
    let (c2, _) = mc_batch(&[d2.as_str()], &out2);
    assert_eq!(c1, Some(0));
    assert_eq!(c2, Some(0));
    assert_eq!(
        read_check_report(&out1, "spec")["outcome"],
        "no_counterexample"
    );
    assert_eq!(
        read_check_report(&out2, "spec")["outcome"],
        "counterexample_found"
    );
    // Identical claim names, different structured contents: the digests
    // bind contents, not paths — so they differ.
    let digest1 = scope_digest_of(&read_check_report(&out1, "spec"));
    let digest2 = scope_digest_of(&read_check_report(&out2, "spec"));
    assert_ne!(digest1, digest2);
    // Swapped reports are rejected: d1's invocation must not accept
    // d2's evidence, and the rejected report is never rewritten.
    std::fs::copy(out2.join("spec.check.json"), out1.join("spec.check.json")).unwrap();
    let before = std::fs::read_to_string(out1.join("spec.check.json")).unwrap();
    drop_props(&out1, &["spec"]);
    let (vcode, vjson) = verify_batch(&[d1.as_str()], &out1);
    assert_eq!(vcode, Some(1), "swapped evidence never verifies: {vjson}");
    let entry = &vjson["data"]["blocked"][0];
    assert_eq!(entry["model"]["state"], "stale");
    assert!(
        entry["model"]["detail"].as_str().unwrap().contains("scope"),
        "{entry}"
    );
    let after = std::fs::read_to_string(out1.join("spec.check.json")).unwrap();
    assert_eq!(before, after);
    // Multi-file dual-format lint remains valid (file-local identities).
    spk()
        .args(["lint", d1.as_str(), d2.as_str()])
        .assert()
        .success();
}

// ---- specodelic-gch: USAGE.md quick-start examples produce expected
// claim evidence ----
// (add-min-expr-kernel task 6.1; design D5 per-file gating). The fenced
// full-file examples in specs/USAGE.md are extracted live from the doc
// and pushed through the real compile → model-check path; every
// migrated kernel claim must surface in the fresh report with the
// expected three-valued status.

/// The fenced example with frontmatter id `id`, extracted live from
/// `specs/USAGE.md` — the doc itself is the fixture source, so doc
/// drift breaks this gate instead of silently passing. The scan
/// mirrors `scripts/check_doc_examples.py`: markdown fences whose
/// first non-blank line is `---` are runnable format artifacts.
fn usage_example(id: &str) -> String {
    let doc_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("specs/USAGE.md");
    let doc = std::fs::read_to_string(&doc_path).unwrap();
    let mut blocks: Vec<String> = Vec::new();
    let mut in_fence = false;
    let mut body: Vec<String> = Vec::new();
    for line in doc.lines() {
        if line.trim_start().starts_with("```") {
            if !in_fence {
                in_fence = true;
                body.clear();
            } else {
                in_fence = false;
                let non_blank: Vec<&String> =
                    body.iter().filter(|l| !l.trim().is_empty()).collect();
                if non_blank.first().is_some_and(|l| l.trim() == "---") {
                    blocks.push(body.join("\n"));
                }
            }
            continue;
        }
        if in_fence {
            body.push(line.to_string());
        }
    }
    assert!(!blocks.is_empty(), "USAGE.md must carry fenced examples");
    blocks
        .into_iter()
        .find(|b| {
            let mut lines = b.lines().skip(1); // past the opening ---
            while let Some(line) = lines.next() {
                if line.trim() == "---" {
                    break;
                }
                if let Some(fid) = line.trim().strip_prefix("id:") {
                    return fid.trim().trim_matches(['\'', '"']) == id;
                }
            }
            false
        })
        .unwrap_or_else(|| panic!("no fenced example with id {id} in specs/USAGE.md"))
}

/// The per-invariant (id, status) pairs of the first checked run.
fn checked_statuses(json: &serde_json::Value) -> Vec<(String, String)> {
    json["data"]["checked"][0]["invariant_statuses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            (
                s["id"].as_str().unwrap().to_string(),
                s["status"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
fn usage_quick_start_kernel_claims_verify() {
    // Task 6.1 (design D5): the quick-start example's data-dependent
    // structural facts are kernel claims — the migrated example must
    // lint clean and produce fresh expected statuses through the real
    // compile → model-check path.
    let td = tempfile::tempdir().unwrap();
    let out = td.path().join("out");
    let spec = td.path().join("order-cancel.md");
    std::fs::write(&spec, usage_example("order.cancel")).unwrap();
    // The doc example lints clean — zero exemptions.
    spk()
        .args(["lint", spec.to_str().unwrap()])
        .assert()
        .success();
    claim_compile(spec.to_str().unwrap(), &out);
    let (code, json) = claim_mc_json(spec.to_str().unwrap(), &out);
    assert_eq!(code, Some(0));
    assert_eq!(json["data"]["outcome"], "no_counterexample");
    let statuses = checked_statuses(&json);
    for (id, status) in [
        ("refund_traces_resolve", "verified"),
        ("refund_guard_reaches", "verified"),
    ] {
        assert!(
            statuses.contains(&(id.to_string(), status.to_string())),
            "missing claim evidence {id}:{status} in {statuses:?}"
        );
    }
    // The persisted report carries the identical statuses.
    let report = claim_report(&out, "order-cancel");
    assert_eq!(report["outcome"], "no_counterexample");
}

#[test]
fn usage_micro_example_kernel_claims_verify() {
    // Task 6.1 (design D5, example 2/2): the §3 worked micro-example's
    // data-dependent structural facts are kernel claims — same gate
    // shape as the quick-start, extracted live from the doc.
    let td = tempfile::tempdir().unwrap();
    let out = td.path().join("out");
    let spec = td.path().join("api-rate_limit.md");
    std::fs::write(&spec, usage_example("api.rate_limit")).unwrap();
    spk()
        .args(["lint", spec.to_str().unwrap()])
        .assert()
        .success();
    claim_compile(spec.to_str().unwrap(), &out);
    let (code, json) = claim_mc_json(spec.to_str().unwrap(), &out);
    assert_eq!(code, Some(0));
    assert_eq!(json["data"]["outcome"], "no_counterexample");
    let statuses = checked_statuses(&json);
    for (id, status) in [
        ("rate_traces_resolve", "verified"),
        ("exceed_guard_reaches", "verified"),
    ] {
        assert!(
            statuses.contains(&(id.to_string(), status.to_string())),
            "missing claim evidence {id}:{status} in {statuses:?}"
        );
    }
    // The persisted report carries the identical statuses.
    let report = claim_report(&out, "api-rate_limit");
    assert_eq!(report["outcome"], "no_counterexample");
}

// ---- specodelic-7yk: specodelic.md core format invariants produce
// expected kernel claim evidence ----
// (add-min-expr-kernel task 6.2; design D5 per-file gating). The
// format's own file is the fixture, read live from the repo — doc drift
// breaks this gate instead of silently passing. The four migrated
// invariants (total_refs, coverage, every_transition_valid,
// supersedes_acyclic) must surface as verified kernel claims in the
// fresh JSON envelope and the persisted report; the ineligible cells
// stay informal by recorded decision (research artifact 2026-10-08).

#[test]
fn specodelic_md_kernel_invariants_verify() {
    let td = tempfile::tempdir().unwrap();
    let out = td.path().join("out");
    let repo_spec = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("specs/specodelic.md");
    let spec = repo_spec.to_str().unwrap();
    // The format's own file lints clean — the migration gate (corpus
    // lint, as `just lint-specs` runs it: its cross-file refs resolve
    // corpus-wide, so a single-file lint would false-fail).
    let specs_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("specs");
    spk()
        .args(["lint", specs_dir.to_str().unwrap()])
        .assert()
        .success();
    // compile is corpus-wide for the same reason (compile's
    // precondition_satisfied remediation names it), then model-check
    // runs on specodelic.md as the sole invocation corpus.
    spk()
        .args([
            "compile",
            specs_dir.to_str().unwrap(),
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();
    let (code, json) = claim_mc_json(spec, &out);
    assert_eq!(code, Some(0));
    assert_eq!(json["data"]["outcome"], "no_counterexample");
    let statuses = checked_statuses(&json);
    for (id, status) in [
        ("total_refs", "verified"),
        ("coverage", "verified"),
        ("every_transition_valid", "verified"),
        ("supersedes_acyclic", "verified"),
    ] {
        assert!(
            statuses.contains(&(id.to_string(), status.to_string())),
            "missing claim evidence {id}:{status} in {statuses:?}"
        );
    }
    // The persisted report carries the identical statuses.
    let report = claim_report(&out, "specodelic");
    assert_eq!(report["outcome"], "no_counterexample");
}
