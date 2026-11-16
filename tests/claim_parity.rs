//! Command parity for corpus claims and resolved citations (specodelic-k3h,
//! add-min-expr-kernel §3.8 TIDY; design D9).
//!
//! Characterization fixtures pinning the CURRENT behavior the §3.8
//! extraction must preserve: the same input through the native CLI
//! (`model-check`) and the orchestrate `model_check` stage yields
//! identical statuses, identical unknown-reason labels, identical
//! persisted `.check.json` payloads, and the pinned exit statuses. The
//! refactoring ticket extracts the duplicated corpus-status merge blocks
//! into one shared helper; these fixtures are the meter — 0 changes to
//! accepted inputs, output payloads, or exit statuses.
//!
//! These are CLI assertions — real `specodelic` invocations, never a
//! direct harness (the library-level chains live in `kernel_status.rs`
//! and `citation_algebra.rs`). The one exception is
//! `corpus_status_record_shape_oracle` (specodelic-dzn): a library-level
//! oracle pinning the exact serialized status-record shape the CLI
//! fixtures assert behaviorally, so the shared-type unification of the
//! kernel and citation status records cannot drift a field, a key
//! spelling, or the reason-presence rule.

#[path = "common/mod.rs"]
mod common;
use assert_cmd::Command;
use common::UnwrapPanic;
use serde_json::Value;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap_p()
}

/// A parity fixture in the proven lint-clean shape: a Model section
/// (states/transitions, the guard citing a local row) plus one coverage
/// property row per constraint row. `rows` are verbatim constraint rows;
/// `props` the verbatim property rows; `guard_row` names the row the
/// transition cites.
fn write_parity_fixture(
    path: &std::path::Path,
    id: &str,
    rows: &str,
    props: &str,
    guard_row: &str,
) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL carry claim parity fixtures\"\n---\n\
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
    .unwrap_p();
}

/// Compile every spec into `out` (compile requires a linted-and-covered
/// file; the parity fixtures are lint-clean in this shape).
fn compile(specs: &[String], out: &std::path::Path) {
    let mut args = vec!["compile".to_string()];
    args.extend(specs.iter().cloned());
    args.extend(["--out-dir".to_string(), out.to_str().unwrap_p().to_string()]);
    spk().args(&args).assert().success();
}

/// Run the CLI model-check over the batch: (exit code, parsed envelope).
fn cli_model_check(specs: &[String], out: &std::path::Path) -> (Option<i32>, Value) {
    let mut args = vec!["model-check".to_string()];
    args.extend(specs.iter().cloned());
    args.extend([
        "--json".to_string(),
        "--out-dir".to_string(),
        out.to_str().unwrap_p().to_string(),
    ]);
    let result = spk().args(&args).output().unwrap_p();
    let json: Value = serde_json::from_slice(&result.stdout).unwrap_p();
    (result.status.code(), json)
}

/// Run orchestrate over the batch: (exit code, parsed envelope).
fn orchestrate(specs: &[String], out: &std::path::Path) -> (Option<i32>, Value) {
    let mut args = vec!["orchestrate".to_string()];
    args.extend(specs.iter().cloned());
    args.extend([
        "--json".to_string(),
        "--out-dir".to_string(),
        out.to_str().unwrap_p().to_string(),
    ]);
    let result = spk().args(&args).output().unwrap_p();
    let json: Value = serde_json::from_slice(&result.stdout).unwrap_p();
    (result.status.code(), json)
}

/// The orchestrate model_check stage detail (panics if absent — the
/// stage is load-bearing for command parity).
fn model_check_stage(json: &Value) -> &Value {
    json["data"]["stages"]
        .as_array()
        .unwrap_p()
        .iter()
        .find(|s| s["stage"] == "model_check")
        .unwrap_or_else(|| panic!("model_check stage missing: {:?}", json["data"]["stages"]))
}

/// The full per-invariant status entries (id/status/reason) of one
/// checked entry — the command-output payload shape the merge must
/// preserve verbatim.
fn status_entries(json: &Value, file_idx: usize) -> &Value {
    &json["data"]["checked"][file_idx]["invariant_statuses"]
}

/// A claim id appears exactly once in the list (the "exactly once"
/// meter — no omission, no duplication).
fn assert_exactly_once(list: &Value, id: &str, status: &str) {
    let hits: Vec<&Value> = list
        .as_array()
        .unwrap_p()
        .iter()
        .filter(|s| s["id"] == *id)
        .collect();
    assert_eq!(hits.len(), 1, "claim {id} must appear exactly once");
    assert_eq!(hits[0]["status"], status, "status of {id}: {list:?}");
}

/// The persisted report's raw payload (byte string) for a stem.
fn report_payload(out: &std::path::Path, stem: &str) -> String {
    std::fs::read_to_string(out.join(format!("{stem}.check.json"))).unwrap_p()
}

// ---- Scenario: record-shape oracle — the kernel path and the citation
// ---- path carry ONE claim-report record ----

// The labeled reasons the two paths attach to a not-verified outcome.
const CITATION_REASON: &str = "citation target [[ghost]] is missing from the invocation corpus";
const CLAIM_REASON: &str = "claim kb evaluated to counterexample over the invocation corpus";

/// Record oracle (specodelic-dzn): a kernel claim and a resolved
/// citation carrying the SAME outcome (id, status, reason) carry one
/// record shape, and both paths serialize exactly two shapes —
///
/// * the command-output record: `id`, `status`, and `reason` only when
///   the status is not verified (never an empty/`null` reason);
/// * the persisted-report record: `id` and `status` only — the reason is
///   dropped because `RunReport` keeps its deployed status-only schema.
///
/// Pinned as exact bytes (serde_json's map orders keys) so a refactor
/// cannot rename, reorder, add or drop a field — the CLI fixtures above
/// assert behavior, this oracle asserts the record shape they consume.
#[test]
fn corpus_status_record_shape_oracle() {
    use specodelic::citation_corpus::ResolvedStatus;
    use specodelic::compile::ThreeValued;
    use specodelic::kernel::{CorpusClaimStatus, merge_corpus_statuses};
    use specodelic::model_check::{Backend, Bound, Outcome, RunReport};

    // Same corpus outcome on both paths: unknown, naming its cause.
    let citation_cx = ResolvedStatus {
        id: "cx".to_string(),
        status: ThreeValued::Unknown,
        reason: Some(CITATION_REASON.to_string()),
    };
    let claim_cx = CorpusClaimStatus {
        id: "cx".to_string(),
        status: ThreeValued::Unknown,
        reason: Some(CITATION_REASON.to_string()),
    };
    // The command-output record: exact bytes, key order and all.
    let output = r#"{"id":"cx","reason":"citation target [[ghost]] is missing from the invocation corpus","status":"unknown"}"#;
    assert_eq!(
        serde_json::to_string(&citation_cx.output_json()).unwrap_p(),
        output,
        "citation command-output record shape"
    );
    assert_eq!(
        serde_json::to_string(&claim_cx.output_json()).unwrap_p(),
        output,
        "kernel command-output record shape"
    );
    assert_eq!(
        citation_cx.output_json(),
        claim_cx.output_json(),
        "both paths publish the identical record"
    );

    // Same corpus outcome, verified: no reason field at all.
    let citation_ok = ResolvedStatus {
        id: "ka".to_string(),
        status: ThreeValued::Verified,
        reason: None,
    };
    let claim_ok = CorpusClaimStatus {
        id: "ka".to_string(),
        status: ThreeValued::Verified,
        reason: None,
    };
    for record in [&citation_ok, &claim_ok] {
        assert_eq!(
            serde_json::to_string(&record.output_json()).unwrap_p(),
            r#"{"id":"ka","status":"verified"}"#,
            "verified command-output record shape"
        );
        assert!(
            record.output_json().get("reason").is_none(),
            "a verified record never carries a reason key: {record:?}"
        );
    }

    // The persisted-report record: id + status, reason dropped on BOTH
    // paths (the deployed `RunReport.invariant_statuses` schema).
    let report_cx = r#"{"id":"cx","status":"unknown"}"#;
    assert_eq!(
        serde_json::to_string(&citation_cx.invariant_status()).unwrap_p(),
        report_cx,
        "citation persisted-report record shape"
    );
    assert_eq!(
        serde_json::to_string(&claim_cx.invariant_status()).unwrap_p(),
        report_cx,
        "kernel persisted-report record shape"
    );
    assert_eq!(
        citation_cx.invariant_status(),
        claim_cx.invariant_status(),
        "both paths persist the identical record"
    );

    // The merged command output over one corpus outcome: resolved
    // citation `cx` (unknown, reason), kernel claims `ka` (verified) and
    // `kb` (counterexample, reason) — resolved rows first (already in the
    // report), kernel claims appended, reasons attached only where
    // present, and the report keeping its status-only schema.
    let claim_kb = CorpusClaimStatus {
        id: "kb".to_string(),
        status: ThreeValued::Counterexample,
        reason: Some(CLAIM_REASON.to_string()),
    };
    let mut report = RunReport {
        backend: Backend {
            engine: specodelic::model_check::BACKEND_ENGINE.to_string(),
            version: "0.1.0".to_string(),
        },
        bound: Bound::default(),
        outcome: Outcome::CounterexampleFound,
        invariants_checked: vec![],
        invariant_statuses: vec![citation_cx.invariant_status()],
        violated_invariant_id: Some("kb".to_string()),
        trace: None,
        artifact_sha256: "0".repeat(64),
        states_explored: 0,
    };
    let entries = merge_corpus_statuses(
        &mut report,
        std::slice::from_ref(&citation_cx),
        &[claim_ok, claim_kb],
    );
    assert_eq!(
        serde_json::to_string(&entries).unwrap_p(),
        concat!(
            r#"[{"id":"cx","reason":"citation target [[ghost]] is missing from the invocation corpus","status":"unknown"},"#,
            r#"{"id":"ka","status":"verified"},"#,
            r#"{"id":"kb","reason":"claim kb evaluated to counterexample over the invocation corpus","status":"counterexample"}]"#
        ),
        "merged command-output entries"
    );
    assert_eq!(
        serde_json::to_string(&report.invariant_statuses).unwrap_p(),
        r#"[{"id":"cx","status":"unknown"},{"id":"ka","status":"verified"},{"id":"kb","status":"counterexample"}]"#,
        "merged persisted-report statuses stay status-only"
    );
}

// ---- Scenario: mixed corpus — CLI and orchestrate publish identical
// ---- statuses, reasons, and payloads ----

#[test]
fn mixed_claims_cli_and_orchestrate_parity() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("parity_demo.md");
    let rows = "| c1 | invariant | `**rust:** state != \"blackhole\"` | [[parity_demo]] |\n\
                | ka | invariant | `**kernel:** |State| == 999` | [[parity_demo]] |\n\
                | kb | invariant | `**kernel:** |State| == 2` | [[parity_demo]] |\n\
                | cx | invariant | `[[ghost]]` | [[parity_demo]] |\n";
    let props = "| p_c1 | unit | [[parity_demo.c1]] | `g()` | `x` |\n\
                 | p_ka | unit | [[parity_demo.ka]] | `g()` | `x` |\n\
                 | p_kb | unit | [[parity_demo.kb]] | `g()` | `x` |\n\
                 | p_cx | unit | [[parity_demo.cx]] | `g()` | `x` |\n";
    write_parity_fixture(&spec, "parity_demo", rows, props, "c1");
    let s = spec.to_str().unwrap_p().to_string();
    let cli_out = td.path().join("cli_out");
    let orch_out = td.path().join("orch_out");
    let specs = vec![s.clone()];
    compile(&specs, &cli_out);
    let (cli_code, cli_json) = cli_model_check(&specs, &cli_out);
    let (orch_code, orch_json) = orchestrate(&[s], &orch_out);

    // The two-state model: the Rust claim is verified, the 999-cardinality
    // kernel claim is a real counterexample, the 2-cardinality claim is
    // verified, and the missing citation target is unknown with a reason.
    // specodelic-4v1: the refuted claim is a failing aggregate — the CLI
    // exit code now agrees with the orchestrate stage's verdict (both 1).
    assert_eq!(cli_code, Some(1));
    let cli_entries = status_entries(&cli_json, 0);
    assert_exactly_once(cli_entries, "c1", "verified");
    assert_exactly_once(cli_entries, "ka", "counterexample");
    assert_exactly_once(cli_entries, "kb", "verified");
    assert_exactly_once(cli_entries, "cx", "unknown");

    // Orchestrate's model_check stage: identical entries — statuses,
    // reasons, order — from the shared command path. The refuted
    // kernel claim governs the aggregate (specodelic-68m.1), so the
    // stage is FAILED — the old policy let a refuted claim coexist
    // with a clean stage, the exact gap define-verification-claim-gates
    // closes.
    assert_eq!(orch_code, Some(1));
    let orch_stage = model_check_stage(&orch_json);
    assert_eq!(orch_stage["status"], "failed", "model_check stage clean");
    let orch_entries = &orch_stage["detail"]["checked"][0]["invariant_statuses"];
    assert_eq!(cli_entries, orch_entries, "CLI and orchestrate entries");

    // The persisted reports are byte-identical across the two paths.
    assert_eq!(
        report_payload(&cli_out, "parity_demo"),
        report_payload(&orch_out, "parity_demo"),
        "persisted report payloads must not diverge"
    );
    // The persisted report's statuses agree with the command output.
    let report: Value = serde_json::from_str(&report_payload(&cli_out, "parity_demo")).unwrap_p();
    assert_exactly_once(&report["invariant_statuses"], "cx", "unknown");
}

// ---- Scenario: clean corpus — parity holds on the passing path ----

#[test]
fn clean_corpus_cli_and_orchestrate_parity() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("clean_demo.md");
    let rows = "| c1 | invariant | `**rust:** state != \"blackhole\"` | [[clean_demo]] |\n\
                | ka | invariant | `**kernel:** |State| == 2` | [[clean_demo]] |\n";
    let props = "| p_c1 | unit | [[clean_demo.c1]] | `g()` | `x` |\n\
                 | p_ka | unit | [[clean_demo.ka]] | `g()` | `x` |\n";
    write_parity_fixture(&spec, "clean_demo", rows, props, "c1");
    let s = spec.to_str().unwrap_p().to_string();
    let cli_out = td.path().join("cli_out");
    let orch_out = td.path().join("orch_out");
    let specs = vec![s.clone()];
    compile(&specs, &cli_out);
    let (cli_code, cli_json) = cli_model_check(&specs, &cli_out);
    let (orch_code, orch_json) = orchestrate(&[s], &orch_out);

    assert_eq!(cli_code, Some(0));
    let cli_entries = status_entries(&cli_json, 0);
    assert_exactly_once(cli_entries, "c1", "verified");
    assert_exactly_once(cli_entries, "ka", "verified");

    assert_eq!(orch_code, Some(1)); // verify stage fails downstream; pinned
    let orch_stage = model_check_stage(&orch_json);
    assert_eq!(orch_stage["status"], "passed", "model_check stage clean");
    assert_eq!(
        cli_entries,
        &orch_stage["detail"]["checked"][0]["invariant_statuses"]
    );
    assert_eq!(
        report_payload(&cli_out, "clean_demo"),
        report_payload(&orch_out, "clean_demo")
    );
}

// ---- Scenario: two-file corpus — per-file merge identity holds ----

#[test]
fn two_file_corpus_cli_and_orchestrate_parity() {
    let td = tempfile::tempdir().unwrap_p();
    let a = td.path().join("par_a.md");
    let b = td.path().join("par_b.md");
    // Each file carries a corpus-wide kernel claim (|Intent| == 2 over
    // the two-file invocation) and a file-local missing-target citation
    // with the SAME local id — bare names stay file-local, so the merge
    // must keep the per-file identity aligned with the file order.
    let rows = "| ka | invariant | `**kernel:** |Intent| == 2` | [[par_a]] |\n\
                | cx | invariant | `[[ghost]]` | [[par_a]] |\n";
    let props = "| p_ka | unit | [[par_a.ka]] | `g()` | `x` |\n\
                 | p_cx | unit | [[par_a.cx]] | `g()` | `x` |\n";
    write_parity_fixture(&a, "par_a", rows, props, "ka");
    let rows_b = rows.replace("[[par_a]]", "[[par_b]]");
    let props_b = props.replace("par_a.", "par_b.");
    write_parity_fixture(&b, "par_b", &rows_b, &props_b, "ka");
    let sa = a.to_str().unwrap_p().to_string();
    let sb = b.to_str().unwrap_p().to_string();
    let cli_out = td.path().join("cli_out");
    let orch_out = td.path().join("orch_out");
    let specs = vec![sa.clone(), sb.clone()];
    compile(&specs, &cli_out);
    let (cli_code, cli_json) = cli_model_check(&specs, &cli_out);
    let (orch_code, orch_json) = orchestrate(&[sa, sb], &orch_out);

    // specodelic-4v1 boundary: the unknown citation claims make the
    // aggregate exploration_only — a NON-failure outcome; exit 0 stays
    // while orchestrate's stage still fails downstream (pinned).
    assert_eq!(cli_code, Some(0));
    assert_eq!(orch_code, Some(1)); // verify stage fails downstream; pinned
    let orch_stage = model_check_stage(&orch_json);
    for file_idx in 0..2 {
        let cli = status_entries(&cli_json, file_idx);
        let orch = &orch_stage["detail"]["checked"][file_idx]["invariant_statuses"];
        // Corpus-wide kernel claim verified in both files; the file-local
        // unknown carries the same reason through both command paths.
        assert_exactly_once(cli, "ka", "verified");
        assert_exactly_once(cli, "cx", "unknown");
        assert_eq!(cli, orch, "per-file parity for file index {file_idx}");
        let cx = cli
            .as_array()
            .unwrap_p()
            .iter()
            .find(|s| s["id"] == "cx")
            .unwrap_p();
        assert!(
            cx["reason"]
                .as_str()
                .map(|r| r.contains("missing"))
                .unwrap_or(false),
            "unknown reason label preserved: {cx}"
        );
    }
    for stem in ["par_a", "par_b"] {
        assert_eq!(
            report_payload(&cli_out, stem),
            report_payload(&orch_out, stem)
        );
    }
}

// ---- Scenario: a citation of a refuted rust fragment carries a reason ----

#[test]
fn refuted_fragment_citation_persists_a_reason() {
    // specodelic-60e: `bad` is a **rust:** invariant the executable run
    // refutes and `cx` cites it. The citation resolves to
    // counterexample — and claim_report_schema requires a reason on
    // every not-verified claim, in BOTH the command output and the
    // persisted qualified claim record.
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("cite_reason.md");
    let rows = "| bad | invariant | `**rust:** state != \"s2\"` | [[cite_reason]] |\n\
                | cx | invariant | `[[bad]]` | [[cite_reason]] |\n";
    let props = "| p_bad | unit | [[cite_reason.bad]] | `g()` | `x` |\n\
                 | p_cx | unit | [[cite_reason.cx]] | `g()` | `x` |\n";
    write_parity_fixture(&spec, "cite_reason", rows, props, "bad");
    let s = spec.to_str().unwrap_p().to_string();
    let out = td.path().join("out");
    let specs = vec![s];
    compile(&specs, &out);
    let (code, json) = cli_model_check(&specs, &out);
    assert_eq!(code, Some(1), "the refuted claim fails the aggregate");

    let entries = status_entries(&json, 0);
    let cx = entries
        .as_array()
        .unwrap_p()
        .iter()
        .find(|s| s["id"] == "cx")
        .unwrap_p();
    assert_eq!(
        cx["status"], "counterexample",
        "the citation follows the refuted target: {entries}"
    );
    let output_reason = cx["reason"]
        .as_str()
        .unwrap_or_else(|| panic!("citation counterexample has no reason: {cx}"));
    assert!(!output_reason.is_empty(), "the reason is non-empty");
    assert!(
        output_reason.contains("bad"),
        "the command-output reason names the refuted invariant: {output_reason}"
    );

    // The persisted report's qualified claim record carries the same
    // status and an equally honest reason.
    let report: Value = serde_json::from_str(&report_payload(&out, "cite_reason")).unwrap_p();
    let claim = report["claims"]
        .as_array()
        .unwrap_p()
        .iter()
        .find(|c| c["id"] == "cx")
        .unwrap_p();
    assert_eq!(
        claim["status"], "counterexample",
        "the persisted claim follows the refuted target: {claim}"
    );
    let persisted_reason = claim["reason"]
        .as_str()
        .unwrap_or_else(|| panic!("persisted citation counterexample has no reason: {claim}"));
    assert!(!persisted_reason.is_empty(), "the reason is non-empty");
    assert!(
        persisted_reason.contains("bad"),
        "the persisted reason names the refuted invariant: {persisted_reason}"
    );
}
