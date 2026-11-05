// (split from tests/cli.rs — specodelic-g17 file_lines ratchet)
use super::*;
use crate::common::UnwrapPanic;

// ---- specodelic-8kk: spk orchestrate (openspec/specs/orchestrate/spec.md) ----

#[test]
fn orchestrate_reports_every_stage_and_skips_after_failure() {
    // The METER: .data lists every stage with status; the native
    // backend's exploration_only model_check honestly fails, verify is
    // skipped (never reported as failed), overall failed, exit 1.
    let td = tempfile::tempdir().unwrap_p();
    write_model_check_spec(&td.path().join("pipe-demo.md"), "pipe.demo");
    let out = spk()
        .args([
            "orchestrate",
            td.path().to_str().unwrap_p(),
            "--json",
            "--out-dir",
            td.path().join("out").to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(out.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_p();
    let data = &json["data"];
    assert_eq!(data["overall"], "failed");
    let stages = data["stages"].as_array().unwrap_p();
    let names: Vec<&str> = stages
        .iter()
        .map(|s| s["stage"].as_str().unwrap_p())
        .collect();
    assert_eq!(
        names,
        vec!["parse", "lint", "compile", "model_check", "verify"]
    );
    assert_eq!(stages[3]["status"], "failed");
    assert_eq!(stages[4]["status"], "skipped");
    assert!(stages[4]["reason"].is_string());
    // The lint stage reports every Checker Ownership checker + the
    // not-applicable external-completeness pass.
    let checkers = stages[1]["detail"]["checkers"].as_array().unwrap_p();
    let names: Vec<&str> = checkers
        .iter()
        .map(|c| c["checker"].as_str().unwrap_p())
        .collect();
    assert_eq!(
        names,
        vec![
            "linter.frontmatter",
            "linter.referential_integrity",
            "linter.graph_shape",
            "linter.model_shape",
            "linter.failure_shape",
            "linter.ears_syntax",
            "linter.schema_shape",
            "linter.external_completeness",
        ]
    );
}

#[test]
fn orchestrate_skips_dependents_on_lint_failure() {
    // MUST: the first failing stage halts its dependents (skipped, not
    // failed); exit 1.
    let td = tempfile::tempdir().unwrap_p();
    write_bad_ears_spec(&td.path().join("pipe-bad.md"), "pipe.bad");
    let out = spk()
        .args([
            "orchestrate",
            td.path().to_str().unwrap_p(),
            "--json",
            "--out-dir",
            td.path().join("out").to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(out.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_p();
    let stages = json["data"]["stages"].as_array().unwrap_p();
    assert_eq!(stages[1]["status"], "failed");
    assert_eq!(stages[2]["status"], "skipped");
    assert_eq!(stages[3]["status"], "skipped");
    assert_eq!(stages[4]["status"], "skipped");
    // Human output renders one line per stage (--human: pipes default
    // to JSON envelopes).
    let out = spk()
        .args(["orchestrate", td.path().to_str().unwrap_p(), "--human"])
        .output()
        .unwrap_p();
    let text = String::from_utf8(out.stdout).unwrap_p();
    assert!(text.contains("orchestrate: failed"), "{text}");
    assert!(text.contains("lint: failed"), "{text}");
}

#[test]
fn orchestrate_empty_corpus_is_an_invocation_error() {
    let td = tempfile::tempdir().unwrap_p();
    let out = spk()
        .args(["orchestrate", td.path().to_str().unwrap_p(), "--json"])
        .output()
        .unwrap_p();
    // Invocation error (specodelic-7rr): exit 2, error envelope.
    assert_eq!(out.status.code(), Some(2));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_p();
    assert_eq!(json["envelope_kind"], "error");
}

#[test]
fn orchestrate_checklist_only_corpus_is_an_invocation_error() {
    // EDGE-001 (Ro5 over specodelic-8kk): a declared checklist with zero
    // spec files must never orchestrate to a vacuous `succeeded` — every
    // stage would pass over an empty file set (the 6pi false-green
    // class). Invocation error, exit 2.
    let td = tempfile::tempdir().unwrap_p();
    std::fs::write(
        td.path().join("pipe.checklist.md"),
        "# Checklist\n\n## Items\n\n| id | requirement |\n|----|-------------|\n| i1 | something |\n",
    )
    .unwrap_p();
    let out = spk()
        .args([
            "orchestrate",
            td.path().to_str().unwrap_p(),
            "--json",
            "--out-dir",
            td.path().join("out").to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(out.status.code(), Some(2));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_p();
    assert_eq!(json["envelope_kind"], "error");
    let hint = json["hints"][0]["command"].as_str().unwrap_or("");
    assert!(
        hint.contains("checklist-only") || hint.contains("specodelic lint"),
        "hint names the checklist-only path: {hint}"
    );
}

#[test]
fn orchestrate_tlc_without_jar_is_an_invocation_error() {
    let td = tempfile::tempdir().unwrap_p();
    write_model_check_spec(&td.path().join("pipe-demo.md"), "pipe.demo");
    let out = spk()
        .args([
            "orchestrate",
            td.path().to_str().unwrap_p(),
            "--json",
            "--backend",
            "tlc",
        ])
        .output()
        .unwrap_p();
    assert_eq!(out.status.code(), Some(2));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_p();
    let message = json["warnings"][0]["message"].as_str().unwrap_p();
    assert!(message.contains("--tlc-jar"), "{message}");
}

#[test]
fn lint_failure_hint_does_not_reference_a_notes_field() {
    // gh#4: the format has no Notes field — the remediation hint must
    // point at what exists (spk explain lint-rules).
    let dir = tempfile::tempdir().unwrap_p();
    std::fs::write(
        dir.path().join("bad-spec.md"),
        "---\nid: bad.spec\nkind: intent\nstatement: \"the system should maybe work\"\n---\n",
    )
    .unwrap_p();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap_p(), "--json"])
        .output()
        .unwrap_p();
    let stdout = String::from_utf8(out.stdout).unwrap_p();
    assert!(
        !stdout.contains("in Notes"),
        "hint must not cite the nonexistent Notes field"
    );
    assert!(
        stdout.contains("explain lint-rules"),
        "hint must point at the rule catalog"
    );
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
    // `compile`, `model-check`, and `verify` are implemented
    // (specodelic-lnq/nx7/1pv); orchestrate keeps its stub until its
    // ticket lands.
    spk().args(["orchestrate"]).assert().failure();
}

// ---- specodelic-1pv: the verify step (openspec/specs/verify/spec.md) ----

/// A lint-clean spec with no Properties rows — the properties gate
/// passes vacuously, so the fast verify paths never invoke cargo.
fn write_verify_spec(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE fixture SHALL verify cleanly\"\n---\n\
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
             | t | s1 | s2 | [[{id}.c1]] |\n"
        ),
    )
    .unwrap_p();
}

/// A lint-clean spec with one unit property — compile requires coverage,
/// so the properties gate always has a block to execute. `body_line`
/// replaces the artifact's `todo_predicate!(...)` body: a hand-translation
/// (metadata fingerprint unchanged, so the artifact stays current).

#[test]
fn verify_reports_missing_properties_artifact() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("vfix.md");
    write_verify_spec(&spec, "vfix");
    spk()
        .args([
            "verify",
            spec.to_str().unwrap_p(),
            "--out-dir",
            td.path().join("out").to_str().unwrap_p(),
        ])
        .assert()
        .failure()
        .stdout(contains("missing_properties_artifact"));
}

#[test]
fn verify_requires_a_current_model_run() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = compile_fixture(&td, "vfix", "        let _ = v0;");
    spk()
        .args(["verify", &spec, "--out-dir", &out])
        .assert()
        .failure()
        .stdout(contains("missing_model_run"));
}

#[test]
fn verify_rejects_native_backend_exploration_only() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = compile_fixture(&td, "vfix", "        let _ = v0;");
    // specodelic-4v1 boundary: exploration_only is a NON-failure
    // model-check outcome (exit 0); verify's model gate is what rejects
    // it downstream.
    spk()
        .args(["model-check", &spec, "--out-dir", &out])
        .assert()
        .success();
    // The native backend executes no invariant predicates — its
    // exploration_only outcome can never satisfy the model gate
    // (both_gates_required; verify.md's single_gate_insufficient).
    spk()
        .args(["verify", &spec, "--out-dir", &out, "--json"])
        .assert()
        .failure()
        .stdout(contains("model_not_clean"))
        .stdout(contains("exploration_only"));
}

#[test]
fn verify_rejects_stale_model_run() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = compile_fixture(&td, "vfix", "        let _ = v0;");
    // specodelic-4v1 boundary: exploration_only is a NON-failure
    // model-check outcome (exit 0); verify's staleness check is what
    // rejects the stored report downstream.
    spk()
        .args(["model-check", &spec, "--out-dir", &out])
        .assert()
        .success();
    // The compiled module changed after the run — the stored clean-ish
    // report predates the artifact and fails closed as stale.
    let tla = td.path().join("out").join("vfix.tla");
    std::fs::write(&tla, "MODULE vfix edited").unwrap_p();
    spk()
        .args(["verify", &spec, "--out-dir", &out, "--json"])
        .assert()
        .failure()
        .stdout(contains("stale_model_run"));
}

/// A kernel-claim spec whose required claims all verify over the v0
/// snapshot — the honest no_counterexample shape (no fabricated
/// reports: claim-schema evidence is bound to the run, specodelic-68m.3).
// ---- specodelic-68m.5: the orchestrate model_check stage is the same
// claim view as the native CLI (design D5 visible_scope) ----

#[test]
fn orchestrate_model_check_stage_carries_the_same_claim_view() {
    // Unknown kernel claim + prose-only unchecked: the stage's compact
    // entries carry the SAME required/unchecked/claim sets the native
    // CLI envelope and the persisted report state.
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("ovp.md");
    std::fs::write(
        &spec,
        "---\nid: ovp\nkind: intent\nstatement: \"THE ovp SHALL carry stage claim-view fixtures\"\n---\n\
         \n## Constraints\n\
         \n| id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | ka | invariant | `**kernel:** reachable(demo.ghost.nowhere, ovp.ka, supersedes)` | [[ovp]] |\n\
         | kb | invariant | `**rust:** state != \"blackhole\"` | [[ovp]] |\n\
         | kp | invariant | prose guard holds across states | [[ovp]] |\n\
         \n## Model\n\
         \n### States\n\
         \n- s1\n\
         - s2\n\
         \n### Transitions\n\
         \n| id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | t | s1 | s2 | [[ovp.ka]] |\n\
         \n## Properties\n\
         \n| id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|------------|\n\
         | p_ka | unit | [[ovp.ka]] | `word()` | `**rust:** v0.len() >= 1` |\n\
         | p_kb | unit | [[ovp.kb]] | `word()` | `**rust:** v0.len() >= 1` |\n\
         | p_kp | unit | [[ovp.kp]] | `word()` | `**rust:** v0.len() >= 1` |\n",
    )
    .unwrap_p();
    let out = td.path().join("out");
    let json: serde_json::Value = serde_json::from_slice(
        &Command::cargo_bin("specodelic")
            .unwrap_p()
            .args([
                "orchestrate",
                spec.to_str().unwrap_p(),
                "--json",
                "--out-dir",
                out.to_str().unwrap_p(),
            ])
            .output()
            .unwrap_p()
            .stdout,
    )
    .unwrap_p();
    let stage = json["data"]["stages"]
        .as_array()
        .unwrap_p()
        .iter()
        .find(|s| s["stage"] == "model_check")
        .unwrap_p()
        .clone();
    assert_eq!(
        stage["status"], "failed",
        "unknown claim is not clean: {stage}"
    );
    let checked = &stage["detail"]["checked"][0];
    let expected: Vec<String> = checked["expected_claim_ids"]
        .as_array()
        .unwrap_p()
        .iter()
        .map(|v| v.as_str().unwrap_p().to_string())
        .collect();
    assert_eq!(expected, vec!["ka".to_string(), "kb".to_string()]);
    let unchecked: Vec<String> = checked["unchecked_claim_ids"]
        .as_array()
        .unwrap_p()
        .iter()
        .map(|v| v.as_str().unwrap_p().to_string())
        .collect();
    assert_eq!(unchecked, vec!["kp".to_string()]);
    let claims = checked["claims"].as_array().unwrap_p();
    let ka = claims.iter().find(|c| c["id"] == "ka").unwrap_p();
    assert_eq!(ka["status"], "unknown");
    // And the persisted report agrees with the stage view.
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("ovp.check.json")).unwrap_p())
            .unwrap_p();
    assert_eq!(report["expected_claim_ids"], checked["expected_claim_ids"]);
    assert_eq!(
        report["unchecked_claim_ids"],
        checked["unchecked_claim_ids"]
    );
    assert_eq!(report["claims"], checked["claims"]);
}

fn write_kernel_verified_spec(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL carry kernel-claim fixtures\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | ka | invariant | `**kernel:** acyclic(supersedes)` | [[{id}]] |\n\
             | kb | invariant | `**kernel:** resolves(traces_to)` | [[{id}]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- s1\n\
             - s2\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | t | s1 | s2 | [[{id}.ka]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p_ka | unit | [[{id}.ka]] | `word()` | `**rust:** v0.len() >= 1` |\n\
             | p_kb | unit | [[{id}.kb]] | `word()` | `**rust:** v0.len() >= 1` |\n"
        ),
    )
    .unwrap_p();
}

#[test]
fn verify_accepts_clean_report_with_current_artifacts() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("vfix.md");
    write_kernel_verified_spec(&spec, "vfix");
    let out = td.path().join("out");
    spk()
        .args([
            "compile",
            spec.to_str().unwrap_p(),
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .assert()
        .success();
    spk()
        .args([
            "model-check",
            spec.to_str().unwrap_p(),
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .assert()
        .success();
    // Every required kernel claim verified over a completed bounded
    // exploration — the METER case: .data.status == "verified" exactly
    // under the conjunction, from real evidence (specodelic-68m.3: a
    // syntactically valid forged report is not cryptographically
    // trusted evidence, so this fixture no longer fabricates one).
    spk()
        .args([
            "verify",
            spec.to_str().unwrap_p(),
            "--out-dir",
            out.to_str().unwrap_p(),
            "--json",
        ])
        .assert()
        .success()
        .stdout(contains("\"status\":\"verified\""));
}

#[test]
fn orchestrate_refuses_combined_same_identity_scope_before_any_stage() {
    // H2 (RO5U), revised Revision 18 (specodelic-mcy): the scope law is
    // the identity law — orchestrate's preflight refuses a same-id pair
    // before lint, compile, model_check or verify write anything.
    let td = tempfile::tempdir().unwrap_p();
    let write_dup = |path: &std::path::Path| {
        std::fs::write(
            path,
            "---\nid: dup_id\nkind: intent\nstatement: \"THE ord SHALL hold\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | c1 | invariant | `holds` | [[dup_id]] |\n",
        )
        .unwrap_p();
    };
    write_dup(&td.path().join("dup_one.md"));
    write_dup(&td.path().join("dup_two.md"));
    let out = td.path().join("out");
    let result = spk()
        .args([
            "orchestrate",
            td.path().join("dup_one.md").to_str().unwrap_p(),
            td.path().join("dup_two.md").to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_ne!(result.status.code(), Some(0));
    let stderr = String::from_utf8(result.stderr).unwrap_p();
    assert!(stderr.contains("duplicate_corpus_identity"), "{stderr}");
    // Before ANY stage writes: no artifacts, no reports.
    assert!(!out.exists());
}
#[test]
fn verify_executes_failing_predicate_blocks() {
    // Cargo-backed honest-execution path: the translated predicate body
    // fails for real, and the failure is reported as properties_failed
    // with the block's captured output — never a skip.
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = compile_fixture(&td, "vfix", "        panic!(\"forced failure for 1pv\");");
    spk()
        .args(["verify", &spec, "--out-dir", &out, "--json"])
        .timeout(std::time::Duration::from_secs(600))
        .assert()
        .failure()
        .stdout(contains("properties_failed"))
        .stdout(contains("forced failure for 1pv"));
}

// ---- specodelic-8aq: parameterless property rows must compile and fail honestly ----

/// A lint-clean spec whose property row's generator cell yields no strategy
/// params (no `name(` occurrences — the corpus shape behind specodelic-8aq,
/// e.g. linter-schema_shape's `(a, b)` where-prose).
fn write_parameterless_spec(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL fail parameterless properties honestly\"\n---\n\
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
             | p | unit | [[{id}.c1]] | `(a, b)` where one was removed | `check holds` |\n"
        ),
    )
    .unwrap_p();
}

#[test]
fn parameterless_property_rows_fail_honestly_not_uncompilable() {
    // specodelic-8aq: a generator cell with no strategy params used to emit
    // `fn p()` INSIDE proptest! — a parse error, so the scratch crate never
    // compiled and the verdict was properties_uncompilable (worse than the
    // honest todo-panic failure it replaced). The block must compile and
    // fail honestly via todo_predicate! (properties_failed).
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("p8aq.md");
    write_parameterless_spec(&spec, "p8aq");
    let out = td.path().join("out");
    spk()
        .args([
            "compile",
            spec.to_str().unwrap_p(),
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .assert()
        .success();
    spk()
        .args([
            "verify",
            spec.to_str().unwrap_p(),
            "--out-dir",
            out.to_str().unwrap_p(),
            "--json",
        ])
        .timeout(std::time::Duration::from_secs(600))
        .assert()
        .failure()
        .stdout(contains("properties_failed"))
        .stdout(contains("properties_uncompilable").not());
}

// ---- specodelic-rjb: executable predicate fragments (Revision 15) ----

/// A lint-clean spec with an executable predicate fragment, an executable
/// invariant, and a fragment-free sibling property — the end-to-end demo
/// (verify.md's fragments_reach_verified).
fn write_fragment_spec(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL verify end-to-end with executable fragments\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | c1 | invariant | `**rust:** state != \"blackhole\"` | [[{id}]] |\n\
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
             | p | unit | [[{id}.c1]] | `word()` | `**rust:** v0.len() >= 1` |\n"
        ),
    )
    .unwrap_p();
}

#[test]
fn fragments_reach_verified_end_to_end() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("fmdemo.md");
    write_fragment_spec(&spec, "fmdemo");
    let out = td.path().join("out");
    spk()
        .args([
            "compile",
            spec.to_str().unwrap_p(),
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .assert()
        .success();
    // The executable-invariant run: real no_counterexample from a real
    // scratch-crate BFS (engine native-bfs), invariants_checked = [c1].
    let mc = spk()
        .args([
            "model-check",
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(mc.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&mc.stdout).unwrap_p();
    let checked = &json["data"]["checked"][0];
    assert_eq!(json["data"]["outcome"], "no_counterexample");
    assert_eq!(checked["backend"]["engine"], "native-bfs");
    assert_eq!(checked["invariants_checked"], serde_json::json!(["c1"]));
    // Both gates now hold — the verdict the gate could never reach
    // before Revision 15.
    spk()
        .args([
            "verify",
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .assert()
        .success()
        .stdout(contains("\"status\":\"verified\""));
}

#[test]
fn fragment_invariant_violation_reports_minimal_trace() {
    // counterexample_names_violated_invariant + counterexample_is_minimal
    // on a real fragment: the invariant forbids reaching s2; BFS finds
    // the shortest violating path (s1 -> s2) and names the constraint id.
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("fmcx.md");
    write_fragment_spec(&spec, "fmcx");
    let text = std::fs::read_to_string(&spec).unwrap_p();
    std::fs::write(
        &spec,
        text.replace(
            "`**rust:** state != \"blackhole\"`",
            "`**rust:** state != \"s2\"`",
        ),
    )
    .unwrap_p();
    let out = td.path().join("out");
    spk()
        .args([
            "compile",
            spec.to_str().unwrap_p(),
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .assert()
        .success();
    let mc = spk()
        .args([
            "model-check",
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    let json: serde_json::Value = serde_json::from_slice(&mc.stdout).unwrap_p();
    let checked = &json["data"]["checked"][0];
    assert_eq!(json["data"]["outcome"], "counterexample_found");
    assert_eq!(checked["violated_invariant_id"], "c1");
    assert_eq!(checked["trace"], serde_json::json!(["s1", "s2"]));
}

#[test]
fn fragment_compile_failures_are_labeled() {
    // Hygiene: a fragment carrying a banned token fails compile labeled
    // (fragment_extraction naming the token), never a silent artifact.
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("fmhyg.md");
    write_fragment_spec(&spec, "fmhyg");
    let text = std::fs::read_to_string(&spec).unwrap_p();
    std::fs::write(
        &spec,
        text.replace(
            "`**rust:** v0.len() >= 1`",
            "`**rust:** std::fs::metadata(\"x\").is_ok()`",
        ),
    )
    .unwrap_p();
    let out = spk()
        .args([
            "compile",
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            td.path().join("out").to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(out.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_p();
    let failed = &json["data"]["failed"][0];
    assert_eq!(failed["stage"], "fragment_extraction");
    assert!(failed["message"].as_str().unwrap_p().contains("std::fs"));
}

#[test]
fn fragment_that_does_not_compile_is_a_labeled_model_check_error() {
    // An invariant fragment with a syntax error passes hygiene but cannot
    // build — the scratch run fails labeled (fragment_compile), never a
    // verdict. (Predicate fragments compile at verify time — the props
    // artifact — not here; a bad predicate is properties_uncompilable.)
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("fmbad.md");
    write_fragment_spec(&spec, "fmbad");
    let text = std::fs::read_to_string(&spec).unwrap_p();
    std::fs::write(
        &spec,
        text.replace(
            "`**rust:** state != \"blackhole\"`",
            "`**rust:** state != ] ]`",
        ),
    )
    .unwrap_p();
    let out = td.path().join("out");
    spk()
        .args([
            "compile",
            spec.to_str().unwrap_p(),
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .assert()
        .success();
    let mc = spk()
        .args([
            "model-check",
            spec.to_str().unwrap_p(),
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(mc.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&mc.stdout);
    assert!(stdout.contains("fragment_compile"), "{stdout}");
}

// ---- specodelic-4v1: failures ride success-shaped envelopes (F1) ----
// openspec/specs/errors/spec.md's envelope_error_kind row: a failing stage's report is
// an error-kind envelope with ok == false — exit 1 riding ok:true lied
// to consumers gating on .ok.

#[test]
fn verify_blocked_is_an_error_envelope_with_exit_1() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("v4v.md");
    write_verify_spec(&spec, "v4v");
    // No compiled artifacts → the properties gate blocks (the same
    // shape verify_reports_missing_properties_artifact pins, minus the
    // substring check: here we pin the envelope kind).
    let result = spk()
        .args([
            "verify",
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            td.path().join("out").to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(result.status.code(), Some(1), "blocked verify exits 1");
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap_p();
    assert_eq!(json["ok"], serde_json::json!(false), "envelope: {json}");
    assert_eq!(json["envelope_kind"], serde_json::json!("error"));
    // The payload survives: the blocking status is still .data.status.
    assert_eq!(json["data"]["status"], "missing_properties_artifact");
    assert!(
        json["hints"].as_array().is_some_and(|h| !h.is_empty()),
        "failure carries a remediation hint: {json}"
    );
}

#[test]
fn orchestrate_failed_overall_is_an_error_envelope_with_exit_1() {
    let td = tempfile::tempdir().unwrap_p();
    write_model_check_spec(&td.path().join("o4v.md"), "o4v");
    let result = spk()
        .args([
            "orchestrate",
            td.path().to_str().unwrap_p(),
            "--json",
            "--out-dir",
            td.path().join("out").to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(result.status.code(), Some(1), "failed overall exits 1");
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap_p();
    assert_eq!(json["ok"], serde_json::json!(false), "envelope: {json}");
    assert_eq!(json["envelope_kind"], serde_json::json!("error"));
    // The payload survives: the stage list is still .data.stages.
    assert_eq!(json["data"]["overall"], "failed");
    assert!(
        json["hints"].as_array().is_some_and(|h| !h.is_empty()),
        "failure carries a remediation hint: {json}"
    );
}

#[test]
fn orchestrate_succeeded_stays_a_success_envelope_with_exit_0() {
    // The flip must not overreach: a fully succeeded run (the
    // all-verified kernel-claim fixture, same shape as
    // verify_accepts_clean_report_with_current_artifacts) keeps
    // ok:true / envelope_kind:ok / exit 0.
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("o4s.md");
    write_kernel_verified_spec(&spec, "o4s");
    let out = td.path().join("out");
    let result = spk()
        .args([
            "orchestrate",
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(result.status.code(), Some(0), "fixture must succeed");
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap_p();
    assert_eq!(json["ok"], serde_json::json!(true));
    assert_eq!(json["envelope_kind"], serde_json::json!("ok"));
    assert_eq!(json["data"]["overall"], "succeeded");
}

// ---- specodelic-yl39 (gh-13): corpus-wide orchestrate with duplicate stems ----

/// A lint-clean fixture whose filename stem matches its intent id
/// (naming law) with a distinct Model section — state/transition names
/// are parameters, so each file's `.tla` describes a different
/// automaton and a collapsed artifact set is observable.
fn same_stem_fixture(id: &str, s1: &str, s2: &str, t: &str) -> String {
    format!(
        "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL be a same-stem fixture\"\n---\n\
         \n## Constraints\n\
         \n| id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | c1 | invariant | `holds` | [[{id}]] |\n\
         \n## Model\n\
         \n### States\n\
         \n- {s1}\n\
         - {s2}\n\
         \n### Transitions\n\
         \n| id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | {t} | {s1} | {s2} | [[{id}.c1]] |\n\
         \n## Properties\n\
         \n| id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|------------|\n\
         | p | unit | [[{id}.c1]] | `g()` | `x` |\n"
    )
}

#[test]
fn orchestrate_duplicate_stems_route_per_file_never_share_never_stale() {
    // gh-13: corpus files whose artifact stems collide (same file stem
    // in different dirs) used to compile into ONE flat out-dir — later
    // file wins, stem-collision warnings — and model_check then read
    // the single surviving `.tla` for every file → stale_artifact for
    // all but the last file. (The CLI refuses duplicate INTENT ids
    // up-front — duplicate_corpus_identity — so the collision class
    // that remains is the library path and any future law relaxation;
    // this test pins the structural guarantee there.) The routing
    // decision is shared (compile stage + backend pass + persisted
    // report), collide-only: duplicate stems get deterministic per-file
    // dirs keyed by source path.
    use specodelic::model_check;
    use specodelic::orchestrate::{Backends, ParseInput, orchestrate};
    use specodelic::spec::parse_str;

    let td = tempfile::tempdir().unwrap_p();
    let corpus = td.path().join("corpus");
    std::fs::create_dir_all(corpus.join("alpha")).unwrap_p();
    std::fs::create_dir_all(corpus.join("beta")).unwrap_p();
    std::fs::write(
        corpus.join("alpha/pipeline.md"),
        same_stem_fixture("pipeline", "s1", "s2", "t"),
    )
    .unwrap_p();
    std::fs::write(
        corpus.join("beta/pipeline.md"),
        same_stem_fixture("pipeline", "m1", "m2", "u"),
    )
    .unwrap_p();
    let specs: Vec<specodelic::spec::Spec> = [
        (
            "alpha/pipeline.md",
            same_stem_fixture("pipeline", "s1", "s2", "t"),
        ),
        (
            "beta/pipeline.md",
            same_stem_fixture("pipeline", "m1", "m2", "u"),
        ),
    ]
    .iter()
    .map(|(rel, body)| {
        let mut s = parse_str(body).unwrap_p();
        s.path = Some(corpus.join(rel));
        s
    })
    .collect();
    let out_dir = td.path().join("out");

    let run = || {
        orchestrate(
            &specs,
            &[],
            &ParseInput::default(),
            out_dir.to_str().unwrap_p(),
            &model_check::Bound::default(),
            &Backends::default(),
        )
    };
    let first = run();
    let stage = |o: &specodelic::orchestrate::Orchestration, name: &str| -> serde_json::Value {
        serde_json::to_value(
            o.stages
                .iter()
                .find(|s| s.stage == name)
                .unwrap_or_else(|| panic!("stage {name} missing")),
        )
        .unwrap_p()
    };

    // Invariant 1+2: zero stem collisions — the compile stage routes
    // duplicates into per-file dirs; the "later file wins" warning is
    // unreachable by construction.
    let compile_stage = stage(&first, "compile");
    assert_eq!(compile_stage["status"], "passed", "{compile_stage}");
    let warnings = compile_stage["detail"]["warnings"].as_array().unwrap_p();
    assert!(
        !warnings
            .iter()
            .any(|w| w.as_str().unwrap_p().contains("stem collision")),
        "no stem-collision warnings: {warnings:?}"
    );
    // The written paths are real, distinct, and both files' `.tla`
    // artifacts coexist with their own Model content.
    let compiled = compile_stage["detail"]["compiled"].as_array().unwrap_p();
    assert_eq!(compiled.len(), 2, "{compile_stage}");
    let collect_written = |c: &serde_json::Value| -> Vec<String> {
        c["written"]
            .as_array()
            .unwrap_p()
            .iter()
            .map(|p| p.as_str().unwrap_p().to_string())
            .collect()
    };
    let mut tla_paths: Vec<String> = vec![];
    for c in compiled {
        let written = collect_written(c);
        assert_eq!(written.len(), 3, "{c}");
        for p in &written {
            assert!(std::path::Path::new(p).exists(), "artifact {p} not written");
        }
        tla_paths.push(written[2].clone());
    }
    assert_ne!(tla_paths[0], tla_paths[1], "artifact paths must differ");
    let tla_a = std::fs::read(&tla_paths[0]).unwrap_p();
    let tla_b = std::fs::read(&tla_paths[1]).unwrap_p();
    assert_ne!(tla_a, tla_b, "both files' modules must coexist");

    // gh-13 symptom 2: the backend pass reads each file's OWN module —
    // no stale_artifact, no missing_artifact (the stages agree on
    // routing; verdict-vs-CLAR-003 exploration_only is out of scope).
    let mc_stage = stage(&first, "model_check");
    let failures = mc_stage["detail"]["failed"].as_array().unwrap_p();
    assert!(
        failures
            .iter()
            .all(|f| f["stage"] != "stale_artifact" && f["stage"] != "missing_artifact"),
        "no stale/missing artifacts: {failures:?}"
    );
    // The persisted `.check.json` reports follow the same routing: one
    // per file, each next to its own module.
    let reports: Vec<String> = compiled
        .iter()
        .map(|c| {
            collect_written(c)
                .iter()
                .find(|p| p.ends_with(".tla"))
                .unwrap_p()
                .replace(".tla", ".check.json")
        })
        .collect();
    for r in &reports {
        assert!(std::path::Path::new(r).exists(), "report {r} not persisted");
    }
    assert_ne!(reports[0], reports[1]);

    // Invariant 3: byte-stable re-run — same routing every run.
    let second = run();
    let compile_stage2 = stage(&second, "compile");
    let compiled2 = compile_stage2["detail"]["compiled"].as_array().unwrap_p();
    let written_again: Vec<Vec<String>> = compiled2.iter().map(collect_written).collect();
    let written_first: Vec<Vec<String>> = compiled.iter().map(collect_written).collect();
    assert_eq!(written_first, written_again, "routing must be stable");
}
