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
    assert_eq!(json["data"]["format_revision"], "specodelic.md Revision 16");
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
    assert_eq!(json["data"]["format_revision"], "specodelic.md Revision 16");
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
    assert_eq!(json["data"]["format_revision"], "specodelic.md Revision 16");
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
