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
//! and `citation_algebra.rs`).

use assert_cmd::Command;
use serde_json::Value;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap()
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
    .unwrap();
}

/// Compile every spec into `out` (compile requires a linted-and-covered
/// file; the parity fixtures are lint-clean in this shape).
fn compile(specs: &[String], out: &std::path::Path) {
    let mut args = vec!["compile".to_string()];
    args.extend(specs.iter().cloned());
    args.extend(["--out-dir".to_string(), out.to_str().unwrap().to_string()]);
    spk().args(&args).assert().success();
}

/// Run the CLI model-check over the batch: (exit code, parsed envelope).
fn cli_model_check(specs: &[String], out: &std::path::Path) -> (Option<i32>, Value) {
    let mut args = vec!["model-check".to_string()];
    args.extend(specs.iter().cloned());
    args.extend([
        "--json".to_string(),
        "--out-dir".to_string(),
        out.to_str().unwrap().to_string(),
    ]);
    let result = spk().args(&args).output().unwrap();
    let json: Value = serde_json::from_slice(&result.stdout).unwrap();
    (result.status.code(), json)
}

/// Run orchestrate over the batch: (exit code, parsed envelope).
fn orchestrate(specs: &[String], out: &std::path::Path) -> (Option<i32>, Value) {
    let mut args = vec!["orchestrate".to_string()];
    args.extend(specs.iter().cloned());
    args.extend([
        "--json".to_string(),
        "--out-dir".to_string(),
        out.to_str().unwrap().to_string(),
    ]);
    let result = spk().args(&args).output().unwrap();
    let json: Value = serde_json::from_slice(&result.stdout).unwrap();
    (result.status.code(), json)
}

/// The orchestrate model_check stage detail (panics if absent — the
/// stage is load-bearing for command parity).
fn model_check_stage(json: &Value) -> &Value {
    json["data"]["stages"]
        .as_array()
        .unwrap()
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
        .unwrap()
        .iter()
        .filter(|s| s["id"] == *id)
        .collect();
    assert_eq!(hits.len(), 1, "claim {id} must appear exactly once");
    assert_eq!(hits[0]["status"], status, "status of {id}: {list:?}");
}

/// The persisted report's raw payload (byte string) for a stem.
fn report_payload(out: &std::path::Path, stem: &str) -> String {
    std::fs::read_to_string(out.join(format!("{stem}.check.json"))).unwrap()
}

// ---- Scenario: mixed corpus — CLI and orchestrate publish identical
// ---- statuses, reasons, and payloads ----

#[test]
fn mixed_claims_cli_and_orchestrate_parity() {
    let td = tempfile::tempdir().unwrap();
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
    let s = spec.to_str().unwrap().to_string();
    let cli_out = td.path().join("cli_out");
    let orch_out = td.path().join("orch_out");
    let specs = vec![s.clone()];
    compile(&specs, &cli_out);
    let (cli_code, cli_json) = cli_model_check(&specs, &cli_out);
    let (orch_code, orch_json) = orchestrate(&[s], &orch_out);

    // The two-state model: the Rust claim is verified, the 999-cardinality
    // kernel claim is a real counterexample, the 2-cardinality claim is
    // verified, and the missing citation target is unknown with a reason.
    assert_eq!(cli_code, Some(0));
    let cli_entries = status_entries(&cli_json, 0);
    assert_exactly_once(cli_entries, "c1", "verified");
    assert_exactly_once(cli_entries, "ka", "counterexample");
    assert_exactly_once(cli_entries, "kb", "verified");
    assert_exactly_once(cli_entries, "cx", "unknown");

    // Orchestrate's model_check stage: identical entries — statuses,
    // reasons, order — from the shared command path.
    assert_eq!(orch_code, Some(1));
    let orch_stage = model_check_stage(&orch_json);
    assert_eq!(orch_stage["status"], "passed", "model_check stage clean");
    let orch_entries = &orch_stage["detail"]["checked"][0]["invariant_statuses"];
    assert_eq!(cli_entries, orch_entries, "CLI and orchestrate entries");

    // The persisted reports are byte-identical across the two paths.
    assert_eq!(
        report_payload(&cli_out, "parity_demo"),
        report_payload(&orch_out, "parity_demo"),
        "persisted report payloads must not diverge"
    );
    // The persisted report's statuses agree with the command output.
    let report: Value = serde_json::from_str(&report_payload(&cli_out, "parity_demo")).unwrap();
    assert_exactly_once(&report["invariant_statuses"], "cx", "unknown");
}

// ---- Scenario: clean corpus — parity holds on the passing path ----

#[test]
fn clean_corpus_cli_and_orchestrate_parity() {
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("clean_demo.md");
    let rows = "| c1 | invariant | `**rust:** state != \"blackhole\"` | [[clean_demo]] |\n\
                | ka | invariant | `**kernel:** |State| == 2` | [[clean_demo]] |\n";
    let props = "| p_c1 | unit | [[clean_demo.c1]] | `g()` | `x` |\n\
                 | p_ka | unit | [[clean_demo.ka]] | `g()` | `x` |\n";
    write_parity_fixture(&spec, "clean_demo", rows, props, "c1");
    let s = spec.to_str().unwrap().to_string();
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
    let td = tempfile::tempdir().unwrap();
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
    let sa = a.to_str().unwrap().to_string();
    let sb = b.to_str().unwrap().to_string();
    let cli_out = td.path().join("cli_out");
    let orch_out = td.path().join("orch_out");
    let specs = vec![sa.clone(), sb.clone()];
    compile(&specs, &cli_out);
    let (cli_code, cli_json) = cli_model_check(&specs, &cli_out);
    let (orch_code, orch_json) = orchestrate(&[sa, sb], &orch_out);

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
            .unwrap()
            .iter()
            .find(|s| s["id"] == "cx")
            .unwrap();
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
