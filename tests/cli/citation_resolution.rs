//! Citation resolution over the command invocation corpus (specodelic-hb4,
//! add-min-expr-kernel §3.6, design D9): bare names resolve within the
//! citing file, qualified names resolve exactly against same-run evidence,
//! never by suffix, never by filesystem discovery. Scope guards fail
//! labeled before any write. Each fixture asserts exact statuses in both
//! the CLI JSON output and the persisted run report.

use super::*;
use crate::common::UnwrapPanic;

/// An ordinary spec whose row sets are given verbatim — the two-state
/// model is shared, so an exec fragment `state != "s2"` is a real
/// counterexample and `state != "blackhole"` is really verified.
fn write_rows_spec(path: &std::path::Path, id: &str, constraint_rows: &str, property_rows: &str) {
    // The transition guard must cite an existing invariant row — the
    // first constraint row (citation-only fixtures have no c1 row).
    let guard_row = constraint_rows
        .split('|')
        .nth(1)
        .map(|c| c.trim().to_string())
        .unwrap_or_else(|| "c1".to_string());
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL carry citation fixtures\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             {constraint_rows}\
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
             {property_rows}", guard_row = guard_row),
    )
    .unwrap_p();
}

/// The passing exec invariant every fixture cites.
fn verified_row_for(id: &str) -> String {
    format!("| c1 | invariant | `**rust:** state != \"blackhole\"` | [[{id}]] |\n")
}

/// The failing exec invariant — s2 is reachable, so this is a real
/// counterexample.
fn counterexample_row_for(id: &str) -> String {
    format!("| c1 | invariant | `**rust:** state != \"s2\"` | [[{id}]] |\n")
}

/// The coverage property row one per cited constraint (linter.coverage).
fn property_rows_for(id: &str, rows: &[&::std::primitive::str]) -> String {
    rows.iter()
        .map(|row| format!("| p_{row} | unit | [[{id}.{row}]] | `g()` | `x` |\n"))
        .collect()
}

/// Compile a batch of specs together (qualified cross-file refs resolve
/// corpus-wide at lint) into `out`.
fn compile_batch(specs: &[String], out: &std::path::Path) {
    let mut args = vec!["compile".to_string()];
    args.extend(specs.iter().cloned());
    args.extend(["--out-dir".to_string(), out.to_str().unwrap_p().to_string()]);
    spk().args(&args).assert().success();
}

/// Run model-check over a batch and return (exit code, parsed envelope).
fn model_check_json(specs: &[String], out: &std::path::Path) -> (Option<i32>, serde_json::Value) {
    let mut args = vec!["model-check".to_string()];
    args.extend(specs.iter().cloned());
    args.extend([
        "--json".to_string(),
        "--out-dir".to_string(),
        out.to_str().unwrap_p().to_string(),
    ]);
    let result = spk().args(&args).output().unwrap_p();
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap_p();
    (result.status.code(), json)
}

/// The persisted report's per-invariant statuses as (id, status) pairs.
fn report_statuses(out: &std::path::Path, stem: &str) -> Vec<(String, String)> {
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out.join(format!("{stem}.check.json"))).unwrap_p(),
    )
    .unwrap_p();
    report["invariant_statuses"]
        .as_array()
        .unwrap_p()
        .iter()
        .map(|s| {
            (
                s["id"].as_str().unwrap_p().to_string(),
                s["status"].as_str().unwrap_p().to_string(),
            )
        })
        .collect()
}

/// The CLI output's per-invariant statuses for one checked entry.
fn json_statuses(json: &serde_json::Value, file_idx: usize) -> Vec<(String, String)> {
    json["data"]["checked"][file_idx]["invariant_statuses"]
        .as_array()
        .unwrap_p()
        .iter()
        .map(|s| {
            (
                s["id"].as_str().unwrap_p().to_string(),
                s["status"].as_str().unwrap_p().to_string(),
            )
        })
        .collect()
}

/// The unknown statuses' reason strings from one checked entry.
fn json_reasons(json: &serde_json::Value, file_idx: usize) -> Vec<String> {
    json["data"]["checked"][file_idx]["invariant_statuses"]
        .as_array()
        .unwrap_p()
        .iter()
        .filter(|s| s["status"] == "unknown")
        .filter_map(|s| s["reason"].as_str().map(String::from))
        .collect()
}

fn assert_status(list: &[(String, String)], id: &str, status: &str) {
    let entry = list.iter().find(|(i, _)| i == id);
    assert_eq!(
        entry.map(|(_, s)| s.as_str()),
        Some(status),
        "status of {id}: {list:?}"
    );
}

/// A dual-format id:spec delta with one exec invariant and a local
/// qualified citation — `passing` picks the verified fragment,
/// `!passing` the counterexample fragment.
fn write_dual_spec(dir: &std::path::Path, passing: bool) {
    std::fs::create_dir_all(dir).unwrap_p();
    // Revision 18: spec.md derives its id from the parent directory.
    let fid = dir.file_name().and_then(|s| s.to_str()).unwrap_or("d1");
    let fragment = if passing {
        "state != \"blackhole\""
    } else {
        "state != \"s2\""
    };
    std::fs::write(
        dir.join("spec.md"),
        format!(
            "---\nid: {fid}\nkind: intent\nstatement: \"THE delta SHALL be a dual-format model-check fixture\"\n---\n\
             \n# {fid} Specification\n\
             \n## Purpose\n\
             \nMinimal dual-format model-check fixture.\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | c1 | invariant | `**rust:** {fragment}` | [[{fid}]] |\n\
             | c2 | invariant | `[[{fid}.c1]]` | [[{fid}]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- s1\n\
             - s2\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | t | s1 | s2 | [[{fid}.c1]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p_c1 | unit | [[{fid}.c1]] | `word()` | `**rust:** v0.len() >= 1` |\n\
             | p_c2 | unit | [[{fid}.c2]] | `word()` | `**rust:** v0.len() >= 1` |\n\
             \n## ADDED Requirements\n\
             \n### Requirement: Isolated delta scope\nThe delta SHALL be model-checked only as the sole parsed input.\n\
             \n## Requirements\n\
             \n### Requirement: Isolated delta scope\nThe delta SHALL be model-checked only as the sole parsed input.\n"
        ),
    )
    .unwrap_p();
}

// ---- Scenario: Qualified and bare citations agree locally ----

#[test]
fn qualified_and_bare_citations_agree_on_local_exec_evidence() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("cbdemo.md");
    let rows = format!(
        "{}| c2 | invariant | `[[c1]]` | [[cbdemo]] |\n| c3 | invariant | `[[cbdemo.c1]]` | [[cbdemo]] |\n",
        verified_row_for("cbdemo")
    );
    write_rows_spec(
        &spec,
        "cbdemo",
        &rows,
        &property_rows_for("cbdemo", &["c1", "c2", "c3"]),
    );
    let out = td.path().join("out");
    compile_batch(&[spec.to_str().unwrap_p().to_string()], &out);
    let (code, json) = model_check_json(&[spec.to_str().unwrap_p().to_string()], &out);
    assert_eq!(code, Some(0));
    // Baseline (29f81ef): the qualified spelling resolved unknown while
    // the bare spelling verified — both must agree on the same exec
    // evidence now.
    let statuses = json_statuses(&json, 0);
    assert_status(&statuses, "c2", "verified");
    assert_status(&statuses, "c3", "verified");
    // The persisted report carries the identical statuses.
    let report = report_statuses(&out, "cbdemo");
    assert_status(&report, "c2", "verified");
    assert_status(&report, "c3", "verified");
}

// ---- Scenario: no cross-file suffix matches ----

#[test]
fn qualified_citations_resolve_exactly_never_by_suffix() {
    let td = tempfile::tempdir().unwrap_p();
    let res = td.path().join("res.md");
    write_rows_spec(
        &res,
        "res",
        &verified_row_for("res"),
        &property_rows_for("res", &["c1"]),
    );
    let mres = td.path().join("m.res.md");
    write_rows_spec(
        &mres,
        "m.res",
        &counterexample_row_for("m.res"),
        &property_rows_for("m.res", &["c1"]),
    );
    let cite = td.path().join("suffix_cite.md");
    write_rows_spec(
        &cite,
        "suffix_cite",
        "| c2 | invariant | `[[res.c1]]` | [[suffix_cite]] |\n\
         | c3 | invariant | `[[m.res.c1]]` | [[suffix_cite]] |\n",
        &property_rows_for("suffix_cite", &["c2", "c3"]),
    );
    let specs: Vec<String> = [res, mres, cite]
        .iter()
        .map(|p| p.to_str().unwrap_p().to_string())
        .collect();
    let out = td.path().join("out");
    compile_batch(&specs, &out);
    let (code, json) = model_check_json(&specs, &out);
    // specodelic-4v1: the resolved-but-refuted citation claim
    // (m.res.c1's counterexample) is a failing aggregate — exit 1.
    assert_eq!(code, Some(1));
    // Exact keys only: `[[res.c1]]` is file `res`, never a suffix match
    // against `m.res`; the identically named rows keep their own outcomes.
    let idx = 2; // suffix_cite is the third checked entry
    let statuses = json_statuses(&json, idx);
    assert_status(&statuses, "c2", "verified");
    assert_status(&statuses, "c3", "counterexample");
    let report = report_statuses(&out, "suffix_cite");
    assert_status(&report, "c2", "verified");
    assert_status(&report, "c3", "counterexample");
}

// ---- Scenario: same local IDs across distinct ordinary intents ----

#[test]
fn shared_local_ids_stay_file_local_and_qualified_resolves_cross_file() {
    let td = tempfile::tempdir().unwrap_p();
    let a = td.path().join("shrd_a.md");
    write_rows_spec(
        &a,
        "shrd_a",
        &format!(
            "{}| c2 | invariant | `[[c1]]` | [[shrd_a]] |\n| c3 | invariant | `[[shrd_b.c1]]` | [[shrd_a]] |\n",
            verified_row_for("shrd_a")
        ),
        &property_rows_for("shrd_a", &["c1", "c2", "c3"]),
    );
    let b = td.path().join("shrd_b.md");
    write_rows_spec(
        &b,
        "shrd_b",
        &format!(
            "{}| c2 | invariant | `[[c1]]` | [[shrd_b]] |\n",
            counterexample_row_for("shrd_b")
        ),
        &property_rows_for("shrd_b", &["c1", "c2"]),
    );
    let specs: Vec<String> = [a, b]
        .iter()
        .map(|p| p.to_str().unwrap_p().to_string())
        .collect();
    let out = td.path().join("out");
    compile_batch(&specs, &out);
    let (code, json) = model_check_json(&specs, &out);
    // specodelic-4v1: a refuted citation is a failing aggregate — exit 1.
    assert_eq!(code, Some(1));
    // Bare `[[c1]]` in shrd_a resolves to shrd_a's own verified row —
    // never confused with shrd_b's counterexample of the same local id.
    let statuses_a = json_statuses(&json, 0);
    assert_status(&statuses_a, "c2", "verified");
    // Cross-file qualified citation resolves from the same run's evidence.
    assert_status(&statuses_a, "c3", "counterexample");
    let statuses_b = json_statuses(&json, 1);
    assert_status(&statuses_b, "c2", "counterexample");
    // Persisted reports agree with the CLI output, per file.
    assert_status(&report_statuses(&out, "shrd_a"), "c2", "verified");
    assert_status(&report_statuses(&out, "shrd_a"), "c3", "counterexample");
    assert_status(&report_statuses(&out, "shrd_b"), "c2", "counterexample");
}

// ---- Scenario: dependency order is explicit, either file order ----

#[test]
fn citation_chains_resolve_in_dependency_order_either_input_order() {
    let td = tempfile::tempdir().unwrap_p();
    let a = td.path().join("chain_a.md");
    write_rows_spec(
        &a,
        "chain_a",
        &format!(
            "{}| c2 | invariant | `[[c1]]` | [[chain_a]] |\n| c3 | invariant | `[[c2]]` | [[chain_a]] |\n",
            verified_row_for("chain_a")
        ),
        &property_rows_for("chain_a", &["c1", "c2", "c3"]),
    );
    let b = td.path().join("chain_b.md");
    write_rows_spec(
        &b,
        "chain_b",
        "| c9 | invariant | `[[chain_a.c3]]` | [[chain_b]] |\n",
        &property_rows_for("chain_b", &["c9"]),
    );
    let specs: Vec<String> = [a.to_str().unwrap_p(), b.to_str().unwrap_p()]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let out = td.path().join("out");
    compile_batch(&specs, &out);
    // Reversed file order yields identical statuses (same full corpus).
    for order in [vec![0usize, 1usize], vec![1usize, 0usize]] {
        let batch: Vec<String> = order.iter().map(|&i| specs[i].clone()).collect();
        let (code, json) = model_check_json(&batch, &out);
        assert_eq!(code, Some(0));
        let idx_a = order.iter().position(|&i| i == 0).unwrap_p();
        let statuses_a = json_statuses(&json, idx_a);
        assert_status(&statuses_a, "c2", "verified");
        // A citation of a citation resolves in dependency order.
        assert_status(&statuses_a, "c3", "verified");
        let statuses_b = json_statuses(&json, 1 - idx_a);
        assert_status(&statuses_b, "c9", "verified");
        assert_status(&report_statuses(&out, "chain_a"), "c3", "verified");
        assert_status(&report_statuses(&out, "chain_b"), "c9", "verified");
    }
}

// ---- Scenario: a cycle remains unknown with a reason ----

#[test]
fn citation_cycle_is_unknown_with_cycle_reason() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("cydemo.md");
    write_rows_spec(
        &spec,
        "cydemo",
        &format!(
            "{}| c2 | invariant | `[[c3]]` | [[cydemo]] |\n| c3 | invariant | `[[c2]]` | [[cydemo]] |\n",
            verified_row_for("cydemo")
        ),
        &property_rows_for("cydemo", &["c1", "c2", "c3"]),
    );
    let out = td.path().join("out");
    compile_batch(&[spec.to_str().unwrap_p().to_string()], &out);
    let (code, json) = model_check_json(&[spec.to_str().unwrap_p().to_string()], &out);
    // specodelic-4v1 boundary: unknown claims are exploration_only — a
    // NON-failure outcome; exit 0 stays.
    assert_eq!(code, Some(0));
    let statuses = json_statuses(&json, 0);
    assert_status(&statuses, "c2", "unknown");
    assert_status(&statuses, "c3", "unknown");
    // The reason names the cycle.
    let reasons = json_reasons(&json, 0);
    assert!(
        reasons.iter().any(|r| r.to_lowercase().contains("cycle")),
        "cycle reason present: {reasons:?}"
    );
    assert_status(&report_statuses(&out, "cydemo"), "c2", "unknown");
    assert_status(&report_statuses(&out, "cydemo"), "c3", "unknown");
}

// ---- Scenario: an absent target remains unknown with a reason ----

#[test]
fn missing_citation_target_is_unknown_with_reason() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("ghstdemo.md");
    write_rows_spec(
        &spec,
        "ghstdemo",
        &format!(
            "{}| c2 | invariant | `[[ghost]]` | [[ghstdemo]] |\n",
            verified_row_for("ghstdemo")
        ),
        &property_rows_for("ghstdemo", &["c1", "c2"]),
    );
    let out = td.path().join("out");
    compile_batch(&[spec.to_str().unwrap_p().to_string()], &out);
    let (code, json) = model_check_json(&[spec.to_str().unwrap_p().to_string()], &out);
    // specodelic-4v1 boundary: unknown claims are exploration_only — a
    // NON-failure outcome; exit 0 stays.
    assert_eq!(code, Some(0));
    let statuses = json_statuses(&json, 0);
    assert_status(&statuses, "c2", "unknown");
    let reasons = json_reasons(&json, 0);
    assert!(
        reasons.iter().any(|r| r.to_lowercase().contains("missing")),
        "missing-target reason present: {reasons:?}"
    );
    assert_status(&report_statuses(&out, "ghstdemo"), "c2", "unknown");
}

// ---- Scenario: property execution is not invented ----

#[test]
fn property_row_citation_is_unknown_without_property_execution() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("propcite.md");
    write_rows_spec(
        &spec,
        "propcite",
        &format!(
            "{}| c2 | invariant | `[[p_c1]]` | [[propcite]] |\n",
            verified_row_for("propcite")
        ),
        &property_rows_for("propcite", &["c1", "c2"]),
    );
    let out = td.path().join("out");
    compile_batch(&[spec.to_str().unwrap_p().to_string()], &out);
    let (code, json) = model_check_json(&[spec.to_str().unwrap_p().to_string()], &out);
    // specodelic-4v1 boundary: unknown claims are exploration_only — a
    // NON-failure outcome; exit 0 stays.
    assert_eq!(code, Some(0));
    // A Property row has no same-run invariant evidence: unknown, and
    // model-check never invokes a property runner for it (the only
    // backend in the entry is the model checker).
    let statuses = json_statuses(&json, 0);
    assert_status(&statuses, "c2", "unknown");
    assert_eq!(
        json["data"]["checked"][0]["backend"]["engine"],
        "native-bfs"
    );
    assert_status(&report_statuses(&out, "propcite"), "c2", "unknown");
}

// ---- Scenario: File-local identities do not merge into a corpus ----

#[test]
fn combined_same_identity_scope_fails_before_any_writes() {
    // Revision 18 (specodelic-mcy): the retired `id: spec` isolated-scope
    // rule is folded into the identity law — a combined invocation whose
    // inputs claim the SAME intent id fails duplicate_corpus_identity
    // before any writes.
    let td = tempfile::tempdir().unwrap_p();
    write_dual_spec(&td.path().join("d1"), true);
    let twin = {
        let dir = td.path().join("d1x");
        std::fs::create_dir_all(&dir).unwrap_p();
        let path = dir.join("spec.md");
        std::fs::copy(td.path().join("d1/spec.md"), &path).unwrap_p();
        path.to_str().unwrap_p().to_string()
    };
    let out = td.path().join("out");
    let result = spk()
        .args([
            "model-check",
            td.path().join("d1/spec.md").to_str().unwrap_p(),
            twin.as_str(),
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_ne!(result.status.code(), Some(0));
    let stderr = String::from_utf8(result.stderr).unwrap_p();
    assert!(
        stderr.contains("duplicate_corpus_identity"),
        "labeled scope failure: {stderr}"
    );
    // Before ANY writes: no run reports exist for either file.
    assert!(!out.join("spec.check.json").exists());
}

#[test]
fn combined_same_identity_fails_scope_for_check_and_verify() {
    let td = tempfile::tempdir().unwrap_p();
    let ordinary = td.path().join("ord_demo.md");
    write_rows_spec(
        &ordinary,
        "ord_demo",
        &verified_row_for("ord_demo"),
        &property_rows_for("ord_demo", &["c1"]),
    );
    let twin = td.path().join("ord_demo_twin.md");
    write_rows_spec(
        &twin,
        "ord_demo",
        &verified_row_for("ord_demo"),
        &property_rows_for("ord_demo", &["c1"]),
    );
    let ordinary = ordinary.to_str().unwrap_p().to_string();
    let twin = twin.to_str().unwrap_p().to_string();
    let out = td.path().join("out");
    for command in ["model-check", "verify"] {
        let result = spk()
            .args([
                command,
                ordinary.as_str(),
                twin.as_str(),
                "--json",
                "--out-dir",
                out.to_str().unwrap_p(),
            ])
            .output()
            .unwrap_p();
        assert_ne!(
            result.status.code(),
            Some(0),
            "{command} must refuse the same-identity scope"
        );
        let stderr = String::from_utf8(result.stderr).unwrap_p();
        assert!(
            stderr.contains("duplicate_corpus_identity"),
            "{command}: labeled scope failure: {stderr}"
        );
    }
    assert!(!out.join("ord_demo.check.json").exists());
}

#[test]
fn independent_dual_format_runs_preserve_opposite_outcomes() {
    let td = tempfile::tempdir().unwrap_p();
    write_dual_spec(&td.path().join("d1"), true);
    write_dual_spec(&td.path().join("d2"), false);
    let out1 = td.path().join("o1");
    let out2 = td.path().join("o2");
    for (dir, out) in [("d1", &out1), ("d2", &out2)] {
        spk()
            .args([
                "compile",
                td.path().join(dir).join("spec.md").to_str().unwrap_p(),
                "--out-dir",
                out.to_str().unwrap_p(),
            ])
            .assert()
            .success();
    }
    let (code1, json1) = model_check_json(
        &[td.path().join("d1/spec.md").to_str().unwrap_p().to_string()],
        &out1,
    );
    assert_eq!(code1, Some(0));
    let (code2, json2) = model_check_json(
        &[td.path().join("d2/spec.md").to_str().unwrap_p().to_string()],
        &out2,
    );
    // specodelic-4v1: the refuted run exits 1 (findings), the clean one 0.
    assert_eq!(code2, Some(1));
    // Each isolated run keeps its own outcome; the counterexample is not
    // laundered by the sibling file's absence or presence.
    assert_eq!(json1["data"]["outcome"], "no_counterexample");
    assert_eq!(json2["data"]["outcome"], "counterexample_found");
    // spec.c1 citations resolve locally in the single-file scope.
    // Artifacts for a spec.md file name by id (dash form) — d1/d2 here
    // (artifact_stem, specodelic-jf1).
    assert_status(&json_statuses(&json1, 0), "c2", "verified");
    assert_status(&report_statuses(&out1, "d1"), "c2", "verified");
    assert_status(&report_statuses(&out2, "d2"), "c2", "counterexample");
}

#[test]
fn orchestrate_mixed_same_identity_scope_fails_before_any_stage() {
    // H2 (RO5U), revised Revision 18 (specodelic-mcy): the scope law is
    // the identity law — orchestrate's preflight refuses a same-id pair
    // before the compile stage writes anything.
    let td = tempfile::tempdir().unwrap_p();
    let a = td.path().join("dup_one.md");
    let b = td.path().join("dup_two.md");
    write_rows_spec(
        &a,
        "dup_id",
        &verified_row_for("dup_id"),
        &property_rows_for("dup_id", &["c1"]),
    );
    write_rows_spec(
        &b,
        "dup_id",
        &verified_row_for("dup_id"),
        &property_rows_for("dup_id", &["c1"]),
    );
    let out = td.path().join("out");
    let result = spk()
        .args([
            "orchestrate",
            a.to_str().unwrap_p(),
            b.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_ne!(result.status.code(), Some(0));
    let stderr = String::from_utf8(result.stderr).unwrap_p();
    assert!(
        stderr.contains("duplicate_corpus_identity"),
        "labeled scope failure: {stderr}"
    );
    // Before ANY writes: no compile or report artifacts exist.
    assert!(!out.exists());
}

// ---- Scenario: ordinary duplicate intent ids are invalid ----

#[test]
fn ordinary_duplicate_intent_ids_fail_before_writes() {
    let td = tempfile::tempdir().unwrap_p();
    let a = td.path().join("dup_one.md");
    let b = td.path().join("dup_two.md");
    write_rows_spec(
        &a,
        "dup_id",
        &verified_row_for("dup_id"),
        &property_rows_for("dup_id", &["c1"]),
    );
    write_rows_spec(
        &b,
        "dup_id",
        &verified_row_for("dup_id"),
        &property_rows_for("dup_id", &["c1"]),
    );
    let out = td.path().join("out");
    let result = spk()
        .args([
            "model-check",
            a.to_str().unwrap_p(),
            b.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_ne!(result.status.code(), Some(0));
    let err = String::from_utf8(result.stderr).unwrap_p();
    assert!(
        err.contains("duplicate_corpus_identity"),
        "labeled duplicate failure: {err}"
    );
    assert!(!out.join("dup_one.check.json").exists());
    assert!(!out.join("dup_two.check.json").exists());
}

// ---- Multi-file lint keeps accepting these corpora ----

#[test]
fn multi_file_lint_accepts_shared_local_ids_and_dual_format_pairs() {
    let td = tempfile::tempdir().unwrap_p();
    let a = td.path().join("shrd_a.md");
    let b = td.path().join("shrd_b.md");
    write_rows_spec(
        &a,
        "shrd_a",
        &verified_row_for("shrd_a"),
        &property_rows_for("shrd_a", &["c1"]),
    );
    write_rows_spec(
        &b,
        "shrd_b",
        &counterexample_row_for("shrd_b"),
        &property_rows_for("shrd_b", &["c1"]),
    );
    write_dual_spec(&td.path().join("d1"), true);
    write_dual_spec(&td.path().join("d2"), false);
    // Ordinary files sharing local row ids: lint stays per-file.
    spk()
        .args(["lint", a.to_str().unwrap_p(), b.to_str().unwrap_p()])
        .assert()
        .success();
    // Two dual-format files: file-local identity support retained.
    spk()
        .args([
            "lint",
            td.path().join("d1/spec.md").to_str().unwrap_p(),
            td.path().join("d2/spec.md").to_str().unwrap_p(),
        ])
        .assert()
        .success();
}

// ---- Scenario: no implicit filesystem discovery ----

#[test]
fn qualified_citation_without_its_file_in_the_invocation_is_unknown() {
    let td = tempfile::tempdir().unwrap_p();
    let a = td.path().join("disc_a.md");
    write_rows_spec(
        &a,
        "disc_a",
        &format!(
            "{}| c2 | invariant | `[[disc_b.c1]]` | [[disc_a]] |\n",
            verified_row_for("disc_a")
        ),
        &property_rows_for("disc_a", &["c1", "c2"]),
    );
    let b = td.path().join("disc_b.md");
    write_rows_spec(
        &b,
        "disc_b",
        &verified_row_for("disc_b"),
        &property_rows_for("disc_b", &["c1"]),
    );
    // Both files compile together (corpus-wide lint), but the model-check
    // invocation names only disc_a: disc_b is on disk next to it and must
    // NOT be discovered implicitly — no evidence, so unknown.
    let out = td.path().join("out");
    compile_batch(
        &[
            a.to_str().unwrap_p().to_string(),
            b.to_str().unwrap_p().to_string(),
        ],
        &out,
    );
    let (code, json) = model_check_json(&[a.to_str().unwrap_p().to_string()], &out);
    // specodelic-4v1 boundary: unknown claims are exploration_only — a
    // NON-failure outcome; exit 0 stays.
    assert_eq!(code, Some(0));
    let statuses = json_statuses(&json, 0);
    assert_status(&statuses, "c2", "unknown");
    // No report may appear for the file that was never in the invocation.
    assert!(!out.join("disc_b.check.json").exists());
    assert_status(&report_statuses(&out, "disc_a"), "c2", "unknown");
}

// ---- specodelic-9o1h: cross-evaluator citation composition (E2 F2) ----
// A citation target resolves against the same invocation's EXEC outcomes
// AND the corpus-static KERNEL claim statuses (E2 finding F2: the
// algebra resolved EXEC outcomes only, so `[[exec]] ∧ [[kernel]]` was
// undischargable and Rule 4 of the claim aggregate held the file at
// exploration_only — it could never go clean).
//
// The composition is one-way — the closed kernel grammar has no `[[..]]`
// references, so a kernel claim can never cite back and no new cycle
// rule is needed — and every non-composed leg (exec-only citations,
// citation-of-citation dependency order, cycles, missing targets,
// property rows) keeps its exact status and reason.

/// A lint-clean fixture with verbatim constraint rows, one coverage
/// property per row, and the transition guard citing `guard_row`.
fn write_kernel_citation_spec(
    path: &std::path::Path,
    id: &str,
    rows: &str,
    props: &str,
    guard_row: &str,
) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL carry cross-evaluator citation fixtures\"\n---\n\
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
             |----|------|--------------|-----------|-----------|\n\
             {props}"
        ),
    )
    .unwrap_p();
}

/// One coverage property row per named constraint row.
fn kernel_citation_props(id: &str, rows: &[&str]) -> String {
    rows.iter()
        .map(|r| {
            format!("| p_{r} | unit | [[{id}.{r}]] | `word()` | `**rust:** v0.len() >= 1` |\n")
        })
        .collect()
}

/// The persisted report's aggregate outcome for one stem.
fn report_outcome(out: &std::path::Path, stem: &str) -> String {
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out.join(format!("{stem}.check.json"))).unwrap_p(),
    )
    .unwrap_p();
    report["outcome"].as_str().unwrap_p().to_string()
}

/// The reason string of one status entry (empty when unlabeled).
fn status_reason(entry: &serde_json::Value, id: &str) -> String {
    entry["invariant_statuses"]
        .as_array()
        .unwrap_p()
        .iter()
        .find(|s| s["id"] == id)
        .unwrap_or_else(|| panic!("no status entry for {id}: {entry}"))["reason"]
        .as_str()
        .unwrap_or_else(|| panic!("status {id} carries no reason: {entry}"))
        .to_string()
}

/// The (id, status) pairs of one checked entry's statuses.
fn checked_statuses(entry: &serde_json::Value) -> Vec<(String, String)> {
    entry["invariant_statuses"]
        .as_array()
        .unwrap_p()
        .iter()
        .map(|s| {
            (
                s["id"].as_str().unwrap_p().to_string(),
                s["status"].as_str().unwrap_p().to_string(),
            )
        })
        .collect()
}

/// Run verify over one spec and return (exit code, parsed envelope).
fn verify_json(spec: &str, out: &std::path::Path) -> (Option<i32>, serde_json::Value) {
    let result = spk()
        .args([
            "verify",
            spec,
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap_p();
    (result.status.code(), json)
}

/// Run orchestrate over one spec and return the parsed envelope.
fn orchestrate_json(spec: &str, out: &std::path::Path) -> serde_json::Value {
    let result = spk()
        .args([
            "orchestrate",
            spec,
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    serde_json::from_slice(&result.stdout).unwrap_p()
}

/// The constraint rows of the F2 acceptance fixture: one verified exec
/// row, one verified kernel row, and one citation row over both.
fn exec_and_kernel_rows(id: &str) -> String {
    format!(
        "| c1 | invariant | `**rust:** state != \"blackhole\"` | [[{id}]] |\n\
         | ka | invariant | `**kernel:** |State| == 2` | [[{id}]] |\n\
         | kc | invariant | `[[c1]] ∧ [[ka]]` | [[{id}]] |\n"
    )
}

#[test]
fn exec_and_kernel_citation_verifies_and_the_file_goes_clean() {
    // THE F2 acceptance case: `[[exec]] ∧ [[kernel]]` composes across
    // evaluators — both legs verify, so the citation row verifies, every
    // required claim is discharged, and the file reports the clean
    // aggregate. Before the composition the citation was unknown ("no
    // same-run evidence") and the file was exploration_only — it could
    // never go clean.
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("cxv.md");
    write_kernel_citation_spec(
        &spec,
        "cxv",
        &exec_and_kernel_rows("cxv"),
        &kernel_citation_props("cxv", &["c1", "ka", "kc"]),
        "c1",
    );
    let specs = vec![spec.to_str().unwrap_p().to_string()];
    let out = td.path().join("out");
    compile_batch(&specs, &out);
    let (code, json) = model_check_json(&specs, &out);
    assert_eq!(code, Some(0), "a clean aggregate is a success: {json}");
    assert_eq!(json["data"]["outcome"], "no_counterexample");
    let statuses = checked_statuses(&json["data"]["checked"][0]);
    assert!(
        statuses.contains(&("c1".to_string(), "verified".to_string())),
        "the exec leg verifies: {statuses:?}"
    );
    assert!(
        statuses.contains(&("ka".to_string(), "verified".to_string())),
        "the kernel leg verifies: {statuses:?}"
    );
    assert!(
        statuses.contains(&("kc".to_string(), "verified".to_string())),
        "the composed citation verifies: {statuses:?}"
    );
    // Every required claim is discharged — the aggregate is clean.
    let expected: Vec<String> = json["data"]["checked"][0]["expected_claim_ids"]
        .as_array()
        .unwrap_p()
        .iter()
        .map(|s| s.as_str().unwrap_p().to_string())
        .collect();
    assert_eq!(expected, vec!["c1", "ka", "kc"]);
    // The persisted report carries the same statuses and outcome.
    assert_eq!(report_outcome(&out, "cxv"), "no_counterexample");
    assert_status(&report_statuses(&out, "cxv"), "kc", "verified");
    // The verify gate accepts the clean aggregate end to end.
    let (vcode, vjson) = verify_json(spec.to_str().unwrap_p(), &out);
    assert_eq!(vcode, Some(0), "{vjson}");
    assert_eq!(vjson["data"]["status"], "verified");
}

#[test]
fn unknown_kernel_claim_leaves_the_citing_citation_unknown_with_the_kernel_reason() {
    // Kleene absorb across evaluators: an undischargable kernel claim
    // keeps the citing citation unknown — never a pass — and the
    // composed reason NAMES the kernel cause (the unresolvable literal),
    // not a generic "missing target".
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("cxk.md");
    let rows = "| c1 | invariant | `**rust:** state != \"blackhole\"` | [[cxk]] |\n\
                | ka | invariant | `**kernel:** ∃ t ∈ Transition: t.to == s2` | [[cxk]] |\n\
                | kc | invariant | `[[c1]] ∧ [[ka]]` | [[cxk]] |\n";
    write_kernel_citation_spec(
        &spec,
        "cxk",
        rows,
        &kernel_citation_props("cxk", &["c1", "ka", "kc"]),
        "c1",
    );
    let specs = vec![spec.to_str().unwrap_p().to_string()];
    let out = td.path().join("out");
    compile_batch(&specs, &out);
    let (code, json) = model_check_json(&specs, &out);
    // specodelic-4v1 boundary: an unknown required claim is
    // exploration_only — a NON-failure outcome; exit 0 stays.
    assert_eq!(code, Some(0));
    assert_eq!(json["data"]["outcome"], "exploration_only");
    let checked = &json["data"]["checked"][0];
    let statuses = checked_statuses(checked);
    assert!(
        statuses.contains(&("kc".to_string(), "unknown".to_string())),
        "the citing citation stays unknown: {statuses:?}"
    );
    let reason = status_reason(checked, "kc");
    assert!(
        reason.contains("`s2`"),
        "the composed reason names the kernel cause: {reason}"
    );
    assert!(
        !reason.contains("missing target"),
        "the citation resolved against the kernel claim — it is not a missing target: {reason}"
    );
    assert_eq!(report_outcome(&out, "cxk"), "exploration_only");
}

#[test]
fn counterexample_kernel_claim_makes_the_citing_citation_a_counterexample() {
    // Kleene dominance across evaluators: a refuted kernel claim makes
    // the citing citation a counterexample, and Rule 2 of the aggregate
    // names the CITATION row (the refuted required claim) — with no
    // invented state trace.
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("cxc.md");
    let rows = "| c1 | invariant | `**rust:** state != \"blackhole\"` | [[cxc]] |\n\
                | ka | invariant | `**kernel:** |State| == 999` | [[cxc]] |\n\
                | kc | invariant | `[[c1]] ∧ [[ka]]` | [[cxc]] |\n";
    write_kernel_citation_spec(
        &spec,
        "cxc",
        rows,
        &kernel_citation_props("cxc", &["c1", "ka", "kc"]),
        "c1",
    );
    let specs = vec![spec.to_str().unwrap_p().to_string()];
    let out = td.path().join("out");
    compile_batch(&specs, &out);
    let (code, json) = model_check_json(&specs, &out);
    // specodelic-4v1: a refuted required claim is a failing aggregate.
    assert_eq!(code, Some(1));
    assert_eq!(json["data"]["outcome"], "counterexample_found");
    let checked = &json["data"]["checked"][0];
    let statuses = checked_statuses(checked);
    assert!(
        statuses.contains(&("kc".to_string(), "counterexample".to_string())),
        "the citing citation is a counterexample: {statuses:?}"
    );
    // Rule 2 names the citation row — the refuted required claim.
    assert_eq!(checked["violated_invariant_id"], "kc");
    assert_eq!(report_outcome(&out, "cxc"), "counterexample_found");
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("cxc.check.json")).unwrap_p())
            .unwrap_p();
    assert_eq!(report["violated_invariant_id"], "kc");
    // No invented state trace: the citation counterexample carries claim
    // evidence, not a fabricated BFS path.
    assert!(report.get("trace").is_none() || report["trace"].is_null());
}

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
    std::fs::write(&shim, script).unwrap_p();
    std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).unwrap_p();
    std::fs::write(dir.join("tla2tools.jar"), b"placeholder").unwrap_p();
}

#[cfg(unix)]
#[test]
fn unsupported_kernel_claim_keeps_the_citing_citation_unknown_on_tlc() {
    // The incapable-backend leg: TLC cannot evaluate kernel claims, so
    // they come back labeled unsupported (unknown). The citing citation
    // stays unknown with a composed reason naming the unsupported label
    // — never verified, never a pass.
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("cxt.md");
    let rows = "| ka | invariant | `**kernel:** |State| == 2` | [[cxt]] |\n\
                | kc | invariant | `[[ka]]` | [[cxt]] |\n";
    write_kernel_citation_spec(
        &spec,
        "cxt",
        rows,
        &kernel_citation_props("cxt", &["ka", "kc"]),
        "ka",
    );
    let specs = vec![spec.to_str().unwrap_p().to_string()];
    let out = td.path().join("out");
    compile_batch(&specs, &out);
    tlc_seam(
        td.path(),
        "Model checking completed. No error has been found.\n7 states generated, 7 distinct states found, 0 states left on queue.",
    );
    let result = spk()
        .args([
            "model-check",
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
            "--backend",
            "tlc",
            "--tlc-jar",
            td.path().join("tla2tools.jar").to_str().unwrap_p(),
        ])
        .env(
            "SPK_TLC_JAVA",
            td.path().join("fake-java.sh").to_str().unwrap_p(),
        )
        .output()
        .unwrap_p();
    // specodelic-4v1 boundary: unsupported (unknown) claims are
    // exploration_only — a NON-failure outcome; exit 0 stays.
    assert_eq!(result.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap_p();
    let checked = &json["data"]["checked"][0];
    let statuses = checked_statuses(checked);
    assert!(
        statuses.contains(&("ka".to_string(), "unknown".to_string())),
        "the kernel claim is labeled unsupported: {statuses:?}"
    );
    assert!(
        statuses.contains(&("kc".to_string(), "unknown".to_string())),
        "the citing citation stays unknown: {statuses:?}"
    );
    let reason = status_reason(checked, "kc");
    assert!(
        reason.contains("unsupported"),
        "the composed reason names the unsupported label: {reason}"
    );
}

#[test]
fn cli_and_orchestrate_views_agree_on_cross_evaluator_composition() {
    // Both views share the ONE command path (design D4): the CLI
    // model-check envelope and the orchestrate model_check stage publish
    // the same statuses and the same outcome for one fixture.
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("cxv2.md");
    write_kernel_citation_spec(
        &spec,
        "cxv2",
        &exec_and_kernel_rows("cxv2"),
        &kernel_citation_props("cxv2", &["c1", "ka", "kc"]),
        "c1",
    );
    let specs = vec![spec.to_str().unwrap_p().to_string()];
    let out = td.path().join("out");
    compile_batch(&specs, &out);
    let (_, json) = model_check_json(&specs, &out);
    let cli_statuses = checked_statuses(&json["data"]["checked"][0]);
    assert!(
        cli_statuses.contains(&("kc".to_string(), "verified".to_string())),
        "the CLI view verifies the composed citation: {cli_statuses:?}"
    );

    let orchestrated = td.path().join("orch");
    let ojson = orchestrate_json(spec.to_str().unwrap_p(), &orchestrated);
    let stages = ojson["data"]["stages"].as_array().unwrap_p();
    let mc = stages
        .iter()
        .find(|s| s["stage"] == "model_check")
        .unwrap_or_else(|| panic!("model_check stage missing: {stages:?}"));
    assert_eq!(mc["status"], "passed", "the stage verdict: {mc}");
    let stage_statuses = checked_statuses(&mc["detail"]["checked"][0]);
    assert_eq!(
        stage_statuses, cli_statuses,
        "both views publish the same statuses"
    );
    assert_eq!(
        mc["detail"]["checked"][0]["outcome"], json["data"]["checked"][0]["outcome"],
        "both views publish the same outcome"
    );
}

#[test]
fn qualified_cross_file_citation_composes_the_other_files_kernel_claim() {
    // The qualified spelling resolves against the OTHER file's kernel
    // claim status of the same invocation — the cross-file leg of the
    // composition.
    let td = tempfile::tempdir().unwrap_p();
    let b = td.path().join("cxk_b.md");
    write_kernel_citation_spec(
        &b,
        "cxk_b",
        "| kb | invariant | `**kernel:** acyclic(supersedes)` | [[cxk_b]] |\n",
        &kernel_citation_props("cxk_b", &["kb"]),
        "kb",
    );
    let a = td.path().join("cxk_a.md");
    write_kernel_citation_spec(
        &a,
        "cxk_a",
        "| c1 | invariant | `**rust:** state != \"blackhole\"` | [[cxk_a]] |\n\
         | kc | invariant | `[[cxk_b.kb]]` | [[cxk_a]] |\n",
        &kernel_citation_props("cxk_a", &["c1", "kc"]),
        "c1",
    );
    let specs: Vec<String> = [a, b]
        .iter()
        .map(|p| p.to_str().unwrap_p().to_string())
        .collect();
    let out = td.path().join("out");
    compile_batch(&specs, &out);
    let (code, json) = model_check_json(&specs, &out);
    assert_eq!(code, Some(0), "both files are clean: {json}");
    // The citing file resolves the other file's kernel claim.
    let statuses_a = json_statuses(&json, 0);
    assert_status(&statuses_a, "kc", "verified");
    assert_status(&statuses_a, "c1", "verified");
    // The kernel claim itself still appears exactly once, in its own
    // file — a citation row is never merged into it.
    let statuses_b = json_statuses(&json, 1);
    assert_eq!(
        statuses_b.iter().filter(|(i, _)| i == "kb").count(),
        1,
        "the kernel claim appears exactly once: {statuses_b:?}"
    );
    assert_status(&statuses_b, "kb", "verified");
    // The persisted reports agree with the CLI output.
    assert_status(&report_statuses(&out, "cxk_a"), "kc", "verified");
    assert_status(&report_statuses(&out, "cxk_b"), "kb", "verified");
}

#[test]
fn non_composed_citation_legs_keep_their_statuses_and_reasons() {
    // The regression pins: the composition adds an evidence source, it
    // never rewrites the other legs. Exec-only citations, citation-of-
    // citation dependency order, the cycle reason, the missing-target
    // reason and the property-row reason are all unchanged, and the
    // kernel claim still appears exactly once.
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("cxr.md");
    let rows = "| c1 | invariant | `**rust:** state != \"blackhole\"` | [[cxr]] |\n\
                | ka | invariant | `**kernel:** |State| == 2` | [[cxr]] |\n\
                | c2 | invariant | `[[c1]]` | [[cxr]] |\n\
                | c3 | invariant | `[[c2]]` | [[cxr]] |\n\
                | kg | invariant | `[[ghost]]` | [[cxr]] |\n\
                | kp | invariant | `[[p_c1]]` | [[cxr]] |\n\
                | cy1 | invariant | `[[cy2]]` | [[cxr]] |\n\
                | cy2 | invariant | `[[cy1]]` | [[cxr]] |\n";
    write_kernel_citation_spec(
        &spec,
        "cxr",
        rows,
        &kernel_citation_props("cxr", &["c1", "ka", "c2", "c3", "kg", "kp", "cy1", "cy2"]),
        "c1",
    );
    let specs = vec![spec.to_str().unwrap_p().to_string()];
    let out = td.path().join("out");
    compile_batch(&specs, &out);
    let (code, json) = model_check_json(&specs, &out);
    // The unknown legs (cycle, missing target, property row) keep the
    // aggregate at exploration_only — a NON-failure outcome; exit 0.
    assert_eq!(code, Some(0));
    let statuses = json_statuses(&json, 0);
    assert_status(&statuses, "c1", "verified");
    assert_status(&statuses, "c2", "verified");
    assert_status(&statuses, "c3", "verified");
    assert_status(&statuses, "ka", "verified");
    assert_status(&statuses, "kg", "unknown");
    assert_status(&statuses, "kp", "unknown");
    assert_status(&statuses, "cy1", "unknown");
    assert_status(&statuses, "cy2", "unknown");
    // The kernel claim appears exactly once — the citation row is its
    // own row.
    assert_eq!(
        statuses.iter().filter(|(i, _)| i == "ka").count(),
        1,
        "the kernel claim appears exactly once: {statuses:?}"
    );
    let reasons = json_reasons(&json, 0);
    assert!(
        reasons.iter().any(|r| r.to_lowercase().contains("cycle")),
        "the cycle reason is unchanged: {reasons:?}"
    );
    assert!(
        reasons.iter().any(|r| r.to_lowercase().contains("missing")),
        "the missing-target reason is unchanged: {reasons:?}"
    );
    assert!(
        reasons
            .iter()
            .any(|r| r.contains("same-run invariant evidence")),
        "the property-row reason is unchanged: {reasons:?}"
    );
    // The persisted report carries the same statuses.
    let report = report_statuses(&out, "cxr");
    assert_status(&report, "c2", "verified");
    assert_status(&report, "c3", "verified");
    assert_status(&report, "ka", "verified");
    assert_status(&report, "kg", "unknown");
    assert_status(&report, "cy1", "unknown");
}
