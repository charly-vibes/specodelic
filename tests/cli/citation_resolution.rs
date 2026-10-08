//! Citation resolution over the command invocation corpus (specodelic-hb4,
//! add-min-expr-kernel §3.6, design D9): bare names resolve within the
//! citing file, qualified names resolve exactly against same-run evidence,
//! never by suffix, never by filesystem discovery. Scope guards fail
//! labeled before any write. Each fixture asserts exact statuses in both
//! the CLI JSON output and the persisted run report.

use super::*;

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
    .unwrap();
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
    args.extend(["--out-dir".to_string(), out.to_str().unwrap().to_string()]);
    spk().args(&args).assert().success();
}

/// Run model-check over a batch and return (exit code, parsed envelope).
fn model_check_json(specs: &[String], out: &std::path::Path) -> (Option<i32>, serde_json::Value) {
    let mut args = vec!["model-check".to_string()];
    args.extend(specs.iter().cloned());
    args.extend([
        "--json".to_string(),
        "--out-dir".to_string(),
        out.to_str().unwrap().to_string(),
    ]);
    let result = spk().args(&args).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    (result.status.code(), json)
}

/// The persisted report's per-invariant statuses as (id, status) pairs.
fn report_statuses(out: &std::path::Path, stem: &str) -> Vec<(String, String)> {
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out.join(format!("{stem}.check.json"))).unwrap(),
    )
    .unwrap();
    report["invariant_statuses"]
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

/// The CLI output's per-invariant statuses for one checked entry.
fn json_statuses(json: &serde_json::Value, file_idx: usize) -> Vec<(String, String)> {
    json["data"]["checked"][file_idx]["invariant_statuses"]
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

/// The unknown statuses' reason strings from one checked entry.
fn json_reasons(json: &serde_json::Value, file_idx: usize) -> Vec<String> {
    json["data"]["checked"][file_idx]["invariant_statuses"]
        .as_array()
        .unwrap()
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
    std::fs::create_dir_all(dir).unwrap();
    let fragment = if passing {
        "state != \"blackhole\""
    } else {
        "state != \"s2\""
    };
    std::fs::write(
        dir.join("spec.md"),
        format!(
            "---\nid: spec\nkind: intent\nstatement: \"THE delta SHALL be a dual-format model-check fixture\"\n---\n\
             \n# spec Specification\n\
             \n## Purpose\n\
             \nMinimal dual-format model-check fixture.\n\
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
             | p_c2 | unit | [[spec.c2]] | `word()` | `**rust:** v0.len() >= 1` |\n\
             \n## ADDED Requirements\n\
             \n### Requirement: Isolated delta scope\nThe delta SHALL be model-checked only as the sole parsed input.\n\
             \n## Requirements\n\
             \n### Requirement: Isolated delta scope\nThe delta SHALL be model-checked only as the sole parsed input.\n"
        ),
    )
    .unwrap();
}

// ---- Scenario: Qualified and bare citations agree locally ----

#[test]
fn qualified_and_bare_citations_agree_on_local_exec_evidence() {
    let td = tempfile::tempdir().unwrap();
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
    compile_batch(&[spec.to_str().unwrap().to_string()], &out);
    let (code, json) = model_check_json(&[spec.to_str().unwrap().to_string()], &out);
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
    let td = tempfile::tempdir().unwrap();
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
        .map(|p| p.to_str().unwrap().to_string())
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
    let td = tempfile::tempdir().unwrap();
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
        .map(|p| p.to_str().unwrap().to_string())
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
    let td = tempfile::tempdir().unwrap();
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
    let specs: Vec<String> = [a.to_str().unwrap(), b.to_str().unwrap()]
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
        let idx_a = order.iter().position(|&i| i == 0).unwrap();
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
    let td = tempfile::tempdir().unwrap();
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
    compile_batch(&[spec.to_str().unwrap().to_string()], &out);
    let (code, json) = model_check_json(&[spec.to_str().unwrap().to_string()], &out);
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
    let td = tempfile::tempdir().unwrap();
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
    compile_batch(&[spec.to_str().unwrap().to_string()], &out);
    let (code, json) = model_check_json(&[spec.to_str().unwrap().to_string()], &out);
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
    let td = tempfile::tempdir().unwrap();
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
    compile_batch(&[spec.to_str().unwrap().to_string()], &out);
    let (code, json) = model_check_json(&[spec.to_str().unwrap().to_string()], &out);
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
fn combined_dual_format_scope_fails_before_any_writes() {
    let td = tempfile::tempdir().unwrap();
    write_dual_spec(&td.path().join("d1"), true);
    write_dual_spec(&td.path().join("d2"), false); // opposite outcome
    let out = td.path().join("out");
    let result = spk()
        .args([
            "model-check",
            td.path().join("d1/spec.md").to_str().unwrap(),
            td.path().join("d2/spec.md").to_str().unwrap(),
            "--json",
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_ne!(result.status.code(), Some(0));
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(
        stderr.contains("isolated_scope_required"),
        "labeled scope failure: {stderr}"
    );
    // The remediation hint rides the envelope's hints channel.
    let stdout = String::from_utf8(result.stdout).unwrap();
    assert!(
        stdout.contains("separately") && stdout.contains("--out-dir"),
        "hint to run each file separately with its own out-dir: {stdout}"
    );
    // Before ANY writes: no run reports exist for either file.
    assert!(!out.join("spec.check.json").exists());
}

#[test]
fn dual_format_plus_ordinary_input_fails_scope_for_check_and_verify() {
    let td = tempfile::tempdir().unwrap();
    write_dual_spec(&td.path().join("d1"), true);
    let ordinary = td.path().join("ord_demo.md");
    write_rows_spec(
        &ordinary,
        "ord_demo",
        &verified_row_for("ord_demo"),
        &property_rows_for("ord_demo", &["c1"]),
    );
    let dual = td.path().join("d1/spec.md").to_str().unwrap().to_string();
    let ordinary = ordinary.to_str().unwrap().to_string();
    let out = td.path().join("out");
    for command in ["model-check", "verify"] {
        let result = spk()
            .args([
                command,
                dual.as_str(),
                ordinary.as_str(),
                "--json",
                "--out-dir",
                out.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert_ne!(
            result.status.code(),
            Some(0),
            "{command} must refuse the mixed scope"
        );
        let stderr = String::from_utf8(result.stderr).unwrap();
        assert!(
            stderr.contains("isolated_scope_required"),
            "{command}: labeled scope failure: {stderr}"
        );
        let stdout = String::from_utf8(result.stdout).unwrap();
        assert!(
            stdout.contains("separately"),
            "{command}: hint to run each file separately: {stdout}"
        );
    }
    assert!(!out.join("spec.check.json").exists());
    assert!(!out.join("ord_demo.check.json").exists());
}

#[test]
fn independent_dual_format_runs_preserve_opposite_outcomes() {
    let td = tempfile::tempdir().unwrap();
    write_dual_spec(&td.path().join("d1"), true);
    write_dual_spec(&td.path().join("d2"), false);
    let out1 = td.path().join("o1");
    let out2 = td.path().join("o2");
    for (dir, out) in [("d1", &out1), ("d2", &out2)] {
        spk()
            .args([
                "compile",
                td.path().join(dir).join("spec.md").to_str().unwrap(),
                "--out-dir",
                out.to_str().unwrap(),
            ])
            .assert()
            .success();
    }
    let (code1, json1) = model_check_json(
        &[td.path().join("d1/spec.md").to_str().unwrap().to_string()],
        &out1,
    );
    assert_eq!(code1, Some(0));
    let (code2, json2) = model_check_json(
        &[td.path().join("d2/spec.md").to_str().unwrap().to_string()],
        &out2,
    );
    // specodelic-4v1: the refuted run exits 1 (findings), the clean one 0.
    assert_eq!(code2, Some(1));
    // Each isolated run keeps its own outcome; the counterexample is not
    // laundered by the sibling file's absence or presence.
    assert_eq!(json1["data"]["outcome"], "no_counterexample");
    assert_eq!(json2["data"]["outcome"], "counterexample_found");
    // spec.c1 citations resolve locally in the single-file scope.
    assert_status(&json_statuses(&json1, 0), "c2", "verified");
    assert_status(&report_statuses(&out1, "spec"), "c2", "verified");
    assert_status(&report_statuses(&out2, "spec"), "c2", "counterexample");
}

#[test]
fn orchestrate_mixed_dual_format_scope_fails_before_any_writes() {
    // H2 (RO5U): the delta names model-check, verify AND orchestrate —
    // orchestrate's guard fires before the compile stage writes anything.
    let td = tempfile::tempdir().unwrap();
    write_dual_spec(&td.path().join("d1"), true);
    let ordinary = td.path().join("ord_demo.md");
    write_rows_spec(
        &ordinary,
        "ord_demo",
        &verified_row_for("ord_demo"),
        &property_rows_for("ord_demo", &["c1"]),
    );
    let out = td.path().join("out");
    let result = spk()
        .args([
            "orchestrate",
            td.path().join("d1/spec.md").to_str().unwrap(),
            ordinary.to_str().unwrap(),
            "--json",
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_ne!(result.status.code(), Some(0));
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(
        stderr.contains("isolated_scope_required"),
        "labeled scope failure: {stderr}"
    );
    // Before ANY writes: no compile or report artifacts exist.
    assert!(!out.exists());
}

// ---- Scenario: ordinary duplicate intent ids are invalid ----

#[test]
fn ordinary_duplicate_intent_ids_fail_before_writes() {
    let td = tempfile::tempdir().unwrap();
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
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--json",
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_ne!(result.status.code(), Some(0));
    let err = String::from_utf8(result.stderr).unwrap();
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
    let td = tempfile::tempdir().unwrap();
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
        .args(["lint", a.to_str().unwrap(), b.to_str().unwrap()])
        .assert()
        .success();
    // Two dual-format files: file-local identity support retained.
    spk()
        .args([
            "lint",
            td.path().join("d1/spec.md").to_str().unwrap(),
            td.path().join("d2/spec.md").to_str().unwrap(),
        ])
        .assert()
        .success();
}

// ---- Scenario: no implicit filesystem discovery ----

#[test]
fn qualified_citation_without_its_file_in_the_invocation_is_unknown() {
    let td = tempfile::tempdir().unwrap();
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
            a.to_str().unwrap().to_string(),
            b.to_str().unwrap().to_string(),
        ],
        &out,
    );
    let (code, json) = model_check_json(&[a.to_str().unwrap().to_string()], &out);
    // specodelic-4v1 boundary: unknown claims are exploration_only — a
    // NON-failure outcome; exit 0 stays.
    assert_eq!(code, Some(0));
    let statuses = json_statuses(&json, 0);
    assert_status(&statuses, "c2", "unknown");
    // No report may appear for the file that was never in the invocation.
    assert!(!out.join("disc_b.check.json").exists());
    assert_status(&report_statuses(&out, "disc_a"), "c2", "unknown");
}
