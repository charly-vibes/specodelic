//! Kernel claims across the complete invocation corpus (specodelic-mui,
//! add-min-expr-kernel §3.7; design D9's complete command-path
//! evaluation).
//!
//! Every opted-in `**kernel:**` invariant claim must appear exactly once
//! in actual command output — the CLI JSON envelope and the persisted
//! `<stem>.check.json` report — with its three-valued status evaluated
//! over the complete explicit invocation corpus (KernelEnv over every
//! spec in the invocation, never per-file). Reversed file order yields
//! identical statuses; empty input is an invocation error; an incapable
//! backend (TLC) returns labeled unsupported results instead of omitting
//! the claims; and orchestrate shares the command path.
//!
//! These are CLI assertions — real `specodelic` invocations, never a
//! direct KernelEnv harness (the library-level chain lives in
//! `kernel_status.rs`; the out-of-module split honors the pretender
//! file_lines ratchet on `src/model_check.rs`).

use assert_cmd::Command;
use serde_json::Value;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap()
}

/// A kernel-claim fixture in the proven lint-clean shape: a Model
/// section (states/transitions, the guard citing a local row) plus one
/// coverage property row per constraint row. `rows` are verbatim
/// constraint rows; `props` the verbatim property rows; `guard_row`
/// names the row the transition cites.
fn write_kernel_fixture(
    path: &std::path::Path,
    id: &str,
    rows: &str,
    props: &str,
    guard_row: &str,
) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL carry kernel claim fixtures\"\n---\n\
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

/// The two coverage property rows for the ka/kb row pair.
fn ka_kb_props(id: &str) -> String {
    format!(
        "| p_ka | unit | [[{id}.ka]] | `g()` | `x` |\n| p_kb | unit | [[{id}.kb]] | `g()` | `x` |\n"
    )
}

/// Compile every spec into `out` (compile requires a linted-and-covered
/// file; the kernel-claim fixtures are lint-clean in this shape).
fn compile(specs: &[String], out: &std::path::Path) {
    let mut args = vec!["compile".to_string()];
    args.extend(specs.iter().cloned());
    args.extend(["--out-dir".to_string(), out.to_str().unwrap().to_string()]);
    spk().args(&args).assert().success();
}

/// Run model-check over the batch and return (exit code, parsed envelope).
fn model_check_json(specs: &[String], out: &std::path::Path) -> (Option<i32>, Value) {
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

/// The persisted report's per-invariant statuses as (id, status) pairs.
fn report_statuses(out: &std::path::Path, stem: &str) -> Vec<(String, String)> {
    let report: Value = serde_json::from_str(
        &std::fs::read_to_string(out.join(format!("{stem}.check.json"))).unwrap(),
    )
    .unwrap();
    statuses_of(&report["invariant_statuses"])
}

/// The CLI output's per-invariant statuses for one checked entry.
fn json_statuses(json: &Value, file_idx: usize) -> Vec<(String, String)> {
    statuses_of(&json["data"]["checked"][file_idx]["invariant_statuses"])
}

/// (id, status) pairs from a statuses array.
fn statuses_of(list: &Value) -> Vec<(String, String)> {
    list.as_array()
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

/// A claim id appears exactly once in the list (the §3.7 "exactly once"
/// meter — no omission, no duplication).
fn assert_exactly_once(list: &[(String, String)], id: &str, status: &str) {
    let hits: Vec<&(String, String)> = list.iter().filter(|(i, _)| i == id).collect();
    assert_eq!(
        hits.len(),
        1,
        "claim {id} must appear exactly once: {list:?}"
    );
    assert_eq!(hits[0].1, status, "status of {id}: {list:?}");
}

/// The unknown statuses' reason strings from one checked entry.
fn json_reasons(json: &Value, file_idx: usize) -> Vec<String> {
    json["data"]["checked"][file_idx]["invariant_statuses"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["status"] == "unknown")
        .filter_map(|s| s["reason"].as_str().map(String::from))
        .collect()
}

// ---- Scenario: mixed Rust/kernel corpus — every claim surfaces ----

#[test]
fn mixed_kernel_and_rust_claims_both_surface() {
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("mixed_demo.md");
    let rows = "| c1 | invariant | `**rust:** state != \"blackhole\"` | [[mixed_demo]] |\n\
                | ka | invariant | `**kernel:** |State| == 999` | [[mixed_demo]] |\n\
                | kb | invariant | `**kernel:** |State| == 2` | [[mixed_demo]] |\n";
    let props = format!(
        "| p_c1 | unit | [[mixed_demo.c1]] | `g()` | `x` |\n{}",
        ka_kb_props("mixed_demo")
    );
    write_kernel_fixture(&spec, "mixed_demo", rows, &props, "c1");
    let out = td.path().join("out");
    compile(&[spec.to_str().unwrap().to_string()], &out);
    let (code, json) = model_check_json(&[spec.to_str().unwrap().to_string()], &out);

    // The two-state model versus cardinality 999: the Rust claim passes
    // (s2 is reachable but blackhole is not) and the kernel claims are
    // honestly evaluated — 999 is refuted, 2 is verified.
    // specodelic-4v1: the refuted claim is a failing aggregate — exit 1.
    assert_eq!(code, Some(1));
    let cli = json_statuses(&json, 0);
    assert_exactly_once(&cli, "c1", "verified");
    assert_exactly_once(&cli, "ka", "counterexample");
    assert_exactly_once(&cli, "kb", "verified");

    // The persisted report carries the identical statuses.
    let report = report_statuses(&out, "mixed_demo");
    assert_exactly_once(&report, "c1", "verified");
    assert_exactly_once(&report, "ka", "counterexample");
    assert_exactly_once(&report, "kb", "verified");
}

// ---- Scenario: kernel-only corpus — claims still publish ----

#[test]
fn kernel_only_corpus_publishes_every_claim() {
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("konly_demo.md");
    let rows = "| ka | invariant | `**kernel:** |State| == 999` | [[konly_demo]] |\n\
                | kb | invariant | `**kernel:** |State| == 2` | [[konly_demo]] |\n";
    let props = ka_kb_props("konly_demo");
    write_kernel_fixture(&spec, "konly_demo", rows, &props, "ka");
    let out = td.path().join("out");
    compile(&[spec.to_str().unwrap().to_string()], &out);
    let (code, json) = model_check_json(&[spec.to_str().unwrap().to_string()], &out);

    // specodelic-4v1: the refuted claim is a failing aggregate — exit 1.
    assert_eq!(code, Some(1));
    let cli = json_statuses(&json, 0);
    assert_exactly_once(&cli, "ka", "counterexample");
    assert_exactly_once(&cli, "kb", "verified");
    let report = report_statuses(&out, "konly_demo");
    assert_exactly_once(&report, "ka", "counterexample");
    assert_exactly_once(&report, "kb", "verified");
}

// ---- Scenario: reversed file order — identical statuses ----

#[test]
fn reversed_file_order_yields_identical_statuses() {
    let td = tempfile::tempdir().unwrap();
    // |Intent| counts the corpus: == 2 is verified over the two-file
    // invocation (corpus-wide evaluation), == 1 is refuted. Per-file
    // evaluation would flip ka's status — the discriminator.
    let rows = "| ka | invariant | `**kernel:** |Intent| == 2` | [[corpus_a]] |\n\
                | kb | invariant | `**kernel:** |Intent| == 1` | [[corpus_a]] |\n";
    let a = td.path().join("corpus_a.md");
    let b = td.path().join("corpus_b.md");
    write_kernel_fixture(&a, "corpus_a", rows, &ka_kb_props("corpus_a"), "ka");
    write_kernel_fixture(&b, "corpus_b", rows, &ka_kb_props("corpus_b"), "ka");
    let out = td.path().join("out");
    let sa = a.to_str().unwrap().to_string();
    let sb = b.to_str().unwrap().to_string();
    compile(&[sa.clone(), sb.clone()], &out);

    let (code, forward) = model_check_json(&[sa.clone(), sb.clone()], &out);
    // specodelic-4v1: the refuted corpus-wide claim is a failing
    // aggregate — findings exit 1 in both file orders.
    assert_eq!(code, Some(1));
    let (code_rev, reversed) = model_check_json(&[sb, sa], &out);
    assert_eq!(code_rev, Some(1));

    // The corpus-wide statuses are identical in both orders — and they
    // are corpus statuses, not per-file ones.
    for (file_idx, stem) in [(0usize, "corpus_a"), (1, "corpus_b")] {
        assert_exactly_once(&json_statuses(&forward, file_idx), "ka", "verified");
        assert_exactly_once(&json_statuses(&forward, file_idx), "kb", "counterexample");
        assert_exactly_once(&report_statuses(&out, stem), "ka", "verified");
        assert_exactly_once(&report_statuses(&out, stem), "kb", "counterexample");
    }
    assert_eq!(json_statuses(&reversed, 0), json_statuses(&forward, 1));
    assert_eq!(json_statuses(&reversed, 1), json_statuses(&forward, 0));
}

// ---- Scenario: empty input — an invocation error ----

#[test]
fn empty_input_is_invocation_error() {
    let td = tempfile::tempdir().unwrap();
    let empty = td.path().join("empty");
    std::fs::create_dir(&empty).unwrap();
    let out = spk()
        .args([
            "model-check",
            empty.to_str().unwrap(),
            "--json",
            "--out-dir",
            td.path().join("out").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    // Invocation error: nothing to check — never a vacuous success.
    assert_eq!(out.status.code(), Some(2));
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["envelope_kind"], "error");
}

// ---- Scenario: incapable backend — labeled unsupported, never omitted ----

/// The TLC JVM test seam (the tlc_observability pattern): a fake `java`
/// answering the `-version` probe and printing canned run output, plus a
/// placeholder tla2tools.jar.
#[cfg(unix)]
fn tlc_seam(dir: &std::path::Path, run_output: &str) {
    use std::os::unix::fs::PermissionsExt;
    let shim = dir.join("fake-java.sh");
    let script = format!(
        "#!/bin/sh\nfor arg in \"$@\"; do case \"$arg\" in -version) echo 'TLC2 version 1.20.0 of 12 May 2024'; exit 0;; esac; done\ncat <<'TLCOUT'\n{run_output}\nTLCOUT\nexit 0\n"
    );
    std::fs::write(&shim, script).unwrap();
    std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::write(dir.join("tla2tools.jar"), b"placeholder").unwrap();
}

#[cfg(unix)]
#[test]
fn tlc_backend_kernel_claims_are_labeled_unsupported() {
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("konly_demo.md");
    let rows = "| ka | invariant | `**kernel:** |State| == 999` | [[konly_demo]] |\n\
                | kb | invariant | `**kernel:** |State| == 2` | [[konly_demo]] |\n";
    let props = ka_kb_props("konly_demo");
    write_kernel_fixture(&spec, "konly_demo", rows, &props, "ka");
    let out = td.path().join("out");
    compile(&[spec.to_str().unwrap().to_string()], &out);
    tlc_seam(
        td.path(),
        "Model checking completed. No error has been found.\n7 states generated, 7 distinct states found, 0 states left on queue.",
    );

    let result = spk()
        .args([
            "model-check",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            out.to_str().unwrap(),
            "--backend",
            "tlc",
            "--tlc-jar",
            td.path().join("tla2tools.jar").to_str().unwrap(),
        ])
        .env(
            "SPK_TLC_JAVA",
            td.path().join("fake-java.sh").to_str().unwrap(),
        )
        .output()
        .unwrap();
    // The TLC run itself completes; the kernel claims must still be
    // published — labeled unsupported, never omitted, never implied pass.
    // specodelic-4v1 boundary: unsupported (unknown) claims are
    // exploration_only — a NON-failure outcome; exit 0 stays.
    assert_eq!(result.status.code(), Some(0));
    let json: Value = serde_json::from_slice(&result.stdout).unwrap();
    let cli = json_statuses(&json, 0);
    assert_exactly_once(&cli, "ka", "unknown");
    assert_exactly_once(&cli, "kb", "unknown");
    let reasons = json_reasons(&json, 0);
    assert!(
        reasons.iter().any(|r| r.contains("unsupported")),
        "the unsupported label must name itself: {reasons:?}"
    );
    let report = report_statuses(&out, "konly_demo");
    assert_exactly_once(&report, "ka", "unknown");
    assert_exactly_once(&report, "kb", "unknown");
}

// ---- Scenario: orchestrate shares the command path ----

#[test]
fn orchestrate_stage_publishes_kernel_claims() {
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("konly_demo.md");
    let rows = "| ka | invariant | `**kernel:** |State| == 999` | [[konly_demo]] |\n\
                | kb | invariant | `**kernel:** |State| == 2` | [[konly_demo]] |\n";
    let props = ka_kb_props("konly_demo");
    write_kernel_fixture(&spec, "konly_demo", rows, &props, "ka");
    let out = td.path().join("out");
    let result = spk()
        .args([
            "orchestrate",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    // The native run is exploration_only (kernel claims are not executed
    // by the backend), so the stage honestly fails and verify skips —
    // but the stage detail must publish the kernel claims.
    assert_eq!(result.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&result.stdout).unwrap();
    let stages = json["data"]["stages"].as_array().unwrap();
    let mc = stages
        .iter()
        .find(|s| s["stage"] == "model_check")
        .unwrap_or_else(|| panic!("model_check stage missing: {stages:?}"));
    let cli = statuses_of(&mc["detail"]["checked"][0]["invariant_statuses"]);
    assert_exactly_once(&cli, "ka", "counterexample");
    assert_exactly_once(&cli, "kb", "verified");
    // The persisted report agrees.
    let report = report_statuses(&out, "konly_demo");
    assert_exactly_once(&report, "ka", "counterexample");
    assert_exactly_once(&report, "kb", "verified");
}
