// (split from tests/cli.rs — specodelic-g17 file_lines ratchet)
use super::*;

// ---- specodelic-ug3: the opt-in TLC backend ----

/// The TLC JVM test seam: a fake `java` (SPK_TLC_JAVA) that answers the
/// `-version` probe instantly and prints canned run output otherwise, plus
/// a placeholder tla2tools.jar. Real-TLC agreement is external evidence
/// (the conformance suite, vv8) — the CLI contract is what's pinned here.
#[cfg(unix)]
fn tlc_seam(dir: &tempfile::TempDir, run_output: &str, exit_code: i32) {
    use std::os::unix::fs::PermissionsExt;
    let shim = dir.path().join("fake-java.sh");
    let script = format!(
        "#!/bin/sh\nfor arg in \"$@\"; do case \"$arg\" in -version) echo 'TLC2 version 1.20.0 of 12 May 2024'; exit 0;; esac; done\ncat <<'TLCOUT'\n{run_output}\nTLCOUT\nexit {exit_code}\n"
    );
    std::fs::write(&shim, script).unwrap();
    std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::write(dir.path().join("tla2tools.jar"), b"placeholder").unwrap();
}

#[cfg(unix)]
fn write_tlc_module_and_report(spec: &std::path::Path, out_dir: &std::path::Path) {
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
}

#[cfg(unix)]
#[test]
fn model_check_tlc_backend_without_jar_is_an_invocation_error() {
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("mc_demo.md");
    write_model_check_spec(&spec, "mc_demo");
    let out_dir = dir.path().join("specodelic");
    write_tlc_module_and_report(&spec, &out_dir);
    let out = spk()
        .args([
            "model-check",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--backend",
            "tlc",
        ])
        .output()
        .unwrap();
    // Invocation error (specodelic-7rr): invalid invocations exit 2.
    assert_eq!(out.status.code(), Some(2));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["envelope_kind"], "error");
    // Failure messages ride the warnings channel (genesis convention).
    let message = json["warnings"][0]["message"].as_str().unwrap();
    assert!(
        message.contains("--tlc-jar"),
        "the remediation must name the flag: {message}"
    );
}

#[cfg(unix)]
#[test]
fn model_check_tlc_backend_missing_checker_binary_is_a_labeled_error() {
    // MUST (specodelic-ug3): a missing checker binary is an ERROR — never
    // a no_counterexample result.
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("mc_demo.md");
    write_model_check_spec(&spec, "mc_demo");
    let out_dir = dir.path().join("specodelic");
    write_tlc_module_and_report(&spec, &out_dir);
    std::fs::write(dir.path().join("tla2tools.jar"), b"placeholder").unwrap();
    let out = spk()
        .args([
            "model-check",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--backend",
            "tlc",
            "--tlc-jar",
            dir.path().join("tla2tools.jar").to_str().unwrap(),
        ])
        .env(
            "SPK_TLC_JAVA",
            dir.path().join("no-such-java").to_str().unwrap(),
        )
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let failed = &json["data"]["failed"][0];
    assert_eq!(failed["stage"], "missing_checker");
    assert!(
        failed["message"].as_str().unwrap().contains("SPK_TLC_JAVA"),
        "the remediation must name the JVM seam: {}",
        failed["message"]
    );
}

#[cfg(unix)]
#[test]
fn model_check_tlc_backend_reports_a_comparable_run_report() {
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("mc_demo.md");
    write_model_check_spec(&spec, "mc_demo");
    let out_dir = dir.path().join("specodelic");
    write_tlc_module_and_report(&spec, &out_dir);
    tlc_seam(
        &dir,
        "Model checking completed. No error has been found.\n7 states generated, 7 distinct states found, 0 states left on queue.",
        0,
    );
    let out = spk()
        .args([
            "model-check",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--backend",
            "tlc",
            "--tlc-jar",
            dir.path().join("tla2tools.jar").to_str().unwrap(),
        ])
        .env(
            "SPK_TLC_JAVA",
            dir.path().join("fake-java.sh").to_str().unwrap(),
        )
        .output()
        .unwrap();
    // specodelic-4v1 boundary: exploration_only is a NON-failure
    // outcome — exit 0, success envelope, while the report stays
    // comparable across backends.
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["ok"], serde_json::json!(true));
    let checked = &json["data"]["checked"][0];
    assert_eq!(checked["backend"]["engine"], "tlc");
    assert_eq!(checked["backend"]["version"], "1.20.0");
    // Zero corpus invariants are executable (prose — Decision 3): even the
    // reference engine's completed run is exploration_only, never
    // no_counterexample — the two backends' reports are comparable.
    assert_eq!(checked["outcome"], "exploration_only");
    assert_eq!(checked["states_explored"], 7);
    // The persisted report carries the same attribution.
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out_dir.join("mc_demo.check.json")).unwrap())
            .unwrap();
    assert_eq!(report["backend"]["engine"], "tlc");
    assert_eq!(report["artifact_sha256"], checked["artifact_sha256"]);
}

#[cfg(unix)]
#[test]
fn model_check_tlc_backend_depth_cutoff_reports_timed_out() {
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("mc_demo.md");
    write_model_check_spec(&spec, "mc_demo");
    let out_dir = dir.path().join("specodelic");
    write_tlc_module_and_report(&spec, &out_dir);
    tlc_seam(
        &dir,
        "The behavior up to this point is error-free.\n2 states generated, 2 distinct states found.",
        0,
    );
    let out = spk()
        .args([
            "model-check",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--backend",
            "tlc",
            "--tlc-jar",
            dir.path().join("tla2tools.jar").to_str().unwrap(),
        ])
        .env(
            "SPK_TLC_JAVA",
            dir.path().join("fake-java.sh").to_str().unwrap(),
        )
        .output()
        .unwrap();
    // specodelic-4v1 boundary: timed_out is a NON-failure outcome —
    // exit 0, success envelope.
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["ok"], serde_json::json!(true));
    assert_eq!(json["data"]["outcome"], "timed_out");
}

// ---------------------------------------------------------------------------
// Corpus discovery (gh#2.2, specodelic-ag5): a consumer repo keeps its specs
// under an openspec/ tree — bare `spk lint` must not dead-end at "no spec
// files found" without naming the tree.
// ---------------------------------------------------------------------------

/// A minimal valid spec file for the consumer-repo fixtures.
fn consumer_repo_spec(id: &str) -> String {
    format!(
        "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n\
         ## Constraints\n\n\
         | id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | c1 | invariant | `holds` | [[{id}]] |\n\
         \n## Model\n\n\
         ### States\n\n- s1\n- s2\n\n\
         ### Transitions\n\n\
         | id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | t | s1 | s2 | [[{id}.c1]] |\n\
         \n## Properties\n\n\
         | id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|------------|\n\
         | p | unit | [[{id}.c1]] | `g()` | `x` |\n"
    )
}

/// Consumer-repo shape: no specs/ dir, a parseable spec under
/// openspec/changes/<change>/specs/<cap>/spec.md.
fn openspec_consumer_repo(dir: &std::path::Path) {
    let spec = dir.join("openspec/changes/add-x/specs/x").join("spec.md");
    std::fs::create_dir_all(spec.parent().unwrap()).unwrap();
    std::fs::write(&spec, consumer_repo_spec("spec")).unwrap();
}

#[test]
fn bare_lint_in_an_openspec_consumer_repo_hints_at_the_tree() {
    let dir = tempfile::tempdir().unwrap();
    openspec_consumer_repo(dir.path());
    let out = spk()
        .args(["lint", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("openspec"),
        "the zero-files failure names the openspec tree: {stdout}"
    );
}

#[test]
fn bare_graph_in_an_openspec_consumer_repo_hints_at_the_tree() {
    let dir = tempfile::tempdir().unwrap();
    openspec_consumer_repo(dir.path());
    let out = spk()
        .args(["graph", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("openspec"),
        "the zero-files failure names the openspec tree: {stdout}"
    );
}

#[test]
fn doctor_names_the_openspec_corpus_in_a_consumer_repo() {
    let dir = tempfile::tempdir().unwrap();
    openspec_consumer_repo(dir.path());
    let out = spk()
        .args(["doctor", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("corpus discovery") && stdout.contains("lint openspec"),
        "doctor discovers the openspec corpus: {stdout}"
    );
}

#[test]
fn doctor_reports_a_missing_corpus_with_the_discovery_rule() {
    let dir = tempfile::tempdir().unwrap();
    let out = spk()
        .args(["doctor", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("corpus discovery"),
        "doctor carries the discovery check: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// Bare row refs in id:spec dual-format files (gh#2.1, specodelic-15g,
// Option A): within a self-contained file a dotless target naming an own
// row has exactly one possible meaning — the local row — so it resolves
// (canonical `spec.<row>`) instead of vanishing into the metasyntactic
// skip. Dotful spellings are unchanged (ambiguous with `file.row`); the
// gh#5 self-file hint stays for them.
// ---------------------------------------------------------------------------

/// An id:spec dual-format file: constraint c1; a property that derives
/// from it BARE (`[[c1]]`); a second constraint tracing to it bare
/// (traces_to must resolve to an Intent — this is a typing violation).
fn dual_file_with_bare_refs() -> String {
    "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n\
     ## Constraints\n\n\
     | id | kind | expr | traces_to |\n\
     |----|------|------|-----------|\n\
     | c1 | invariant | `x` | |\n\
     | c2 | invariant | `y` | [[c1]] |\n\
     \n## Model\n\n\
     ### States\n\n- `s1`\n\n\
     ### Transitions\n\n\
     | id | from | to | guard |\n\
     |----|------|----|-------|\n\
     | t | s1 | s1 | [[c1]] |\n\n\
     ## Properties\n\n\
     | id | kind | derives_from | generator | predicate |\n\
     |----|------|--------------|-----------|------------|\n\
     | p | unit | [[c1]] | `g()` | `x` |\n"
        .to_string()
}

fn write_dual(dir: &std::path::Path, body: &str) -> std::path::PathBuf {
    let path = dir.join("spec.md");
    std::fs::write(&path, body).unwrap();
    path
}

#[test]
fn bare_row_ref_in_a_dual_format_file_resolves_to_an_edge() {
    let dir = tempfile::tempdir().unwrap();
    let f = write_dual(dir.path(), &dual_file_with_bare_refs());
    let out = spk()
        .args(["graph", f.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let edges: Vec<&serde_json::Value> = json["data"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "properties.derives_from")
        .collect();
    assert!(
        edges
            .iter()
            .any(|e| e["to"] == "spec.c1" && e["from"] == "spec.p"),
        "bare [[c1]] derives_from resolves to an edge: {}",
        json["data"]["edges"]
    );
    assert!(
        json["data"]["dangling"].as_array().unwrap().is_empty(),
        "no dangling from the bare form: {}",
        json["data"]["dangling"]
    );
}

#[test]
fn bare_row_ref_typing_violation_is_reported_not_swallowed() {
    let dir = tempfile::tempdir().unwrap();
    let f = write_dual(dir.path(), &dual_file_with_bare_refs());
    let out = spk()
        .args(["graph", f.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let violations = json["data"]["violations"].as_array().unwrap();
    assert!(
        violations
            .iter()
            .any(|v| v["edge_kind"] == "constraints.traces_to" && v["to"] == "spec.c1"),
        "the bare traces_to ref resolves and then violates typing — never silently skipped: {:?}",
        violations
    );
}

/// Revision 10 (specodelic-cxq): `derives_from` from a `law` Property may
/// resolve to another Property — the same-kind law-restates-law edge (e.g.
/// checker-file `*_naturality` laws deriving from
/// `specodelic.rename_naturality`). Must NOT be a typing violation.
#[test]
fn law_derives_from_law_is_allowed() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("t.md"),
        "---\nid: t\nkind: intent\nstatement: \"THE t SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `x` | [[t]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| law1 | law | [[t.c1]] | `g()` | **identity:** `f(a,a)==a` **naturality:** `f == f` |\n| law2 | law | [[t.law1]] | `g()` | **identity:** `f(a,a)==a` **naturality:** `f == f` |\n",
    )
    .unwrap();
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let violations = json["data"]["violations"].as_array().unwrap();
    let law_edges: Vec<&serde_json::Value> = json["data"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "properties.derives_from" && e["to"] == "t.law1")
        .collect();
    assert!(
        !law_edges.is_empty(),
        "law→law derives_from must be recorded as an edge: {:?}",
        json["data"]["edges"]
    );
    assert!(
        violations.iter().all(|v| v["from"] != "t.law2"),
        "law→law derives_from must not violate typing: {violations:?}"
    );
}

/// Revision 10 (specodelic-cxq): a `guard` may cite a State — the
/// "has reached state X" pattern (`graph.md` extract,
/// `refactor.md` analyze, `orchestrate.md` start_lint).
#[test]
fn guard_to_state_is_allowed() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("t.md"),
        "---\nid: t\nkind: intent\nstatement: \"THE t SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `x` | [[t]] |\n\n## Model\n\n### States\n\n- `s1`\n- `ready`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| go | s1 | ready | [[t.ready]] |\n",
    )
    .unwrap();
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let violations = json["data"]["violations"].as_array().unwrap();
    assert!(
        violations.iter().all(|v| v["from"] != "t.go"),
        "guard→State must not violate typing: {violations:?}"
    );
    let guard_edges: Vec<&serde_json::Value> = json["data"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "transitions.guard" && e["to"] == "t.ready")
        .collect();
    assert!(
        !guard_edges.is_empty(),
        "guard→State must be recorded as an edge"
    );
}

/// The invariant-only guard half still holds: a guard citing an advisory
/// Constraint stays a violation (typing did not loosen).
#[test]
fn guard_to_advisory_constraint_still_violates() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("t.md"),
        "---\nid: t\nkind: intent\nstatement: \"THE t SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | advisory | `x` | [[t]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| go | s1 | s1 | [[t.c1]] |\n",
    )
    .unwrap();
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let violations = json["data"]["violations"].as_array().unwrap();
    assert!(
        violations
            .iter()
            .any(|v| v["edge_kind"] == "transitions.guard" && v["from"] == "t.go"),
        "guard→advisory Constraint must still violate typing: {violations:?}"
    );
}

/// The unit-Property derives_from half still holds: a unit Property
/// deriving from another Property stays a violation.
#[test]
fn unit_derives_from_property_still_violates() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("t.md"),
        "---\nid: t\nkind: intent\nstatement: \"THE t SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `x` | [[t]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p1 | unit | [[t.c1]] | `g()` | `x` |\n| p2 | unit | [[t.p1]] | `g()` | `x` |\n",
    )
    .unwrap();
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let violations = json["data"]["violations"].as_array().unwrap();
    assert!(
        violations
            .iter()
            .any(|v| v["edge_kind"] == "properties.derives_from" && v["from"] == "t.p2"),
        "unit→Property derives_from must still violate typing: {violations:?}"
    );
}

#[test]
fn unknown_bare_targets_still_skip_as_metasyntactic() {
    // Option A is narrow: bare targets that name NO own row keep the
    // metasyntactic skip (template placeholders like [[...]] and docs
    // examples like [[x]] stay invisible).
    let dir = tempfile::tempdir().unwrap();
    let body = dual_file_with_bare_refs().replace("[[c1]]", "[[nope]]");
    let f = write_dual(dir.path(), &body);
    let out = spk()
        .args(["graph", f.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json["data"]["dangling"].as_array().unwrap().is_empty());
    let link_edges: Vec<&serde_json::Value> = json["data"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "properties.derives_from" || e["kind"] == "constraints.traces_to")
        .collect();
    assert!(
        link_edges.is_empty(),
        "an unknown bare target mints no reference edge: {:?}",
        link_edges
    );
}

#[test]
fn bare_row_ref_lints_clean_in_a_dual_format_file() {
    // lint and graph agree: the bare form resolves — no dangling finding,
    // and the guard (a structured traces_to cell) resolves too.
    let dir = tempfile::tempdir().unwrap();
    let f = write_dual(dir.path(), &dual_file_with_bare_refs());
    let out = spk()
        .args(["lint", f.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let dangling: Vec<&serde_json::Value> = json["data"]["issues"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["rule_id"] == "linter.total_refs")
        .collect();
    assert!(
        dangling.is_empty(),
        "bare own-row refs must not dangle: {}",
        json["data"]["issues"]
    );
}

// ---- specodelic-7l3: observability contracts (observes, advisory warning, boundary) ----

/// Publisher fixture: an invariant + an effect Constraint (emitted by a
/// state), lint-clean on its own.
const OBSERVABILITY_PUB: &str = "---\nid: pub\nkind: intent\nstatement: \"THE publisher SHALL emit output\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| inv_ok | invariant | `value is finite` | [[pub]] |\n| eff_out | effect | `output == {value}` | [[pub]] |\n\n## Model\n\n### States\n\n- `idle`\n- `emitting` `emits: [[pub.eff_out]]`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| emit | idle | emitting | [[pub.inv_ok]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_inv | unit | [[pub.inv_ok]] | `g()` | `x` |\n| p_eff | unit | [[pub.eff_out]] | `g()` | `x` |\n";

/// Consumer fixture: an invariant row observing a published effect
/// cross-file (the `observes` optional column).
const OBSERVABILITY_CON: &str = "---\nid: con\nkind: intent\nstatement: \"THE consumer SHALL observe the publisher output\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | observes |\n|----|------|------|-----------|----------|\n| watch | invariant | `output seen` | [[con]] | [[pub.eff_out]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[con.watch]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_w | unit | [[con.watch]] | `g()` | `x` |\n";

fn write_observability_pair(dir: &std::path::Path, consumer_observes: &str) {
    let con = OBSERVABILITY_CON.replace("[[pub.eff_out]] |", consumer_observes);
    std::fs::write(dir.join("pub.md"), OBSERVABILITY_PUB).unwrap();
    std::fs::write(dir.join("con.md"), con).unwrap();
}

#[test]
fn observes_cross_file_edge_extracts_and_types_clean() {
    // The observes column is a typed outbound reference: cross-file
    // observation extracts exactly one edge and types clean (no
    // violation, no dangling).
    let dir = tempfile::tempdir().unwrap();
    write_observability_pair(dir.path(), "[[pub.eff_out]] |");
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let violations = json["data"]["violations"].as_array().unwrap();
    assert!(
        violations.is_empty(),
        "observes → effect must type clean: {violations:?}"
    );
    let edge = json["data"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["kind"] == "constraints.observes")
        .expect("the observes edge must be extracted");
    assert_eq!(edge["from"], "con.watch");
    assert_eq!(edge["to"], "pub.eff_out");
}

#[test]
fn observes_wrong_target_rejected_with_hint() {
    // observes must resolve to an effect Constraint — pointing it at an
    // invariant row is a labeled typing violation, never a recorded edge.
    let dir = tempfile::tempdir().unwrap();
    write_observability_pair(dir.path(), "[[pub.inv_ok]] |");
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let violations = json["data"]["violations"].as_array().unwrap();
    let v = violations
        .iter()
        .find(|v| v["edge_kind"] == "constraints.observes")
        .expect("observes → invariant must be a labeled violation");
    assert!(
        v["reason"].as_str().unwrap().contains("effect Constraint"),
        "the reason must name the rule and the actual target kind: {v:?}"
    );
}

/// specodelic-2q8 (spike p2a, openspec/research/2026-10-01-contract-wiring-spike):
/// a dangling `satisfies` reference must be interface-shaped — it names the
/// consumer row, the consumption column, the missing contract row, and BOTH
/// remediations (publish the row / fix the id) without claiming which applies.
#[test]
fn satisfies_dangling_message_is_interface_shaped() {
    let dir = tempfile::tempdir().unwrap();
    let producer = "---\nid: producer\nkind: intent\nstatement: \"THE producer SHALL publish the hook contract\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| hook_shape | extension_point | `hook fn` | [[producer]] |\n\n## Model\n\n### States\n\n- `s`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s | s | [[producer.hook_shape]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[producer.hook_shape]] | `g()` | `x` |\n";
    // Spike p2a shape: the consumer satisfies a row the producer never
    // published (`producer.nope`).
    let consumer = "---\nid: consumer\nkind: intent\nstatement: \"THE consumer SHALL install hooks\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | satisfies |\n|----|------|------|-----------|-----------|\n| hooks_installed | effect | `hooks ok` | [[consumer]] | [[producer.nope]] |\n\n## Model\n\n### States\n\n- `s`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s | s | [[consumer.hooks_installed]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[consumer.hooks_installed]] | `g()` | `x` |\n";
    std::fs::write(dir.path().join("producer.md"), producer).unwrap();
    std::fs::write(dir.path().join("consumer.md"), consumer).unwrap();
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let dangling = json["data"]["dangling"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d.as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    for part in [
        "consumer.hooks_installed (satisfies)",
        "[[producer.nope]]",
        "no published contract row producer.nope exists",
        "publish it in the producer's file",
        "fix the id",
    ] {
        assert!(
            dangling.contains(part),
            "dangling satisfies entry must be interface-shaped ({part:?}): {dangling}"
        );
    }
}

/// specodelic-2q8 symmetry: a dangling `observes` reference gets the same
/// interface-shaped treatment as a dangling `satisfies`.
#[test]
fn observes_dangling_message_is_interface_shaped() {
    let dir = tempfile::tempdir().unwrap();
    // The pair fixture with the observes cell pointed at a row nobody
    // published — the observes ghost-row case.
    write_observability_pair(dir.path(), "[[ghost.row]] |");
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let dangling = json["data"]["dangling"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d.as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    for part in [
        "con.watch (observes)",
        "[[ghost.row]]",
        "no published contract row ghost.row exists",
        "publish it in the producer's file",
        "fix the id",
    ] {
        assert!(
            dangling.contains(part),
            "dangling observes entry must be interface-shaped ({part:?}): {dangling}"
        );
    }
}

#[test]
fn mutual_observation_is_not_a_cycle() {
    // observes is a claim, not a dependency — two files mutually
    // observing each other's effects stay acyclic (D4).
    let dir = tempfile::tempdir().unwrap();
    // pub's eff_out is observed by con.watch; con's eff_back is observed
    // by pub.watch — a crossed mutual pair. Acyclic's edge set
    // (traces_to ∪ derives_from ∪ guard) never sees it.
    let a = "---\nid: pub\nkind: intent\nstatement: \"THE publisher SHALL emit output\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | observes |\n|----|------|------|-----------|----------|\n| inv_ok | invariant | `value is finite` | [[pub]] | |\n| eff_out | effect | `output == {value}` | [[pub]] | [[con.eff_back]] |\n\n## Model\n\n### States\n\n- `idle`\n- `emitting` `emits: [[pub.eff_out]]`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| emit | idle | emitting | [[pub.inv_ok]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_inv | unit | [[pub.inv_ok]] | `g()` | `x` |\n| p_eff | unit | [[pub.eff_out]] | `g()` | `x` |\n";
    let b = "---\nid: con\nkind: intent\nstatement: \"THE consumer SHALL observe the publisher output\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | observes |\n|----|------|------|-----------|----------|\n| inv_seen | invariant | `output seen` | [[con]] | |\n| eff_back | effect | `ack == {value}` | [[con]] | [[pub.eff_out]] |\n\n## Model\n\n### States\n\n- `idle`\n- `acking` `emits: [[con.eff_back]]`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| ack | idle | acking | [[con.inv_seen]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_inv | unit | [[con.inv_seen]] | `g()` | `x` |\n| p_eff | unit | [[con.eff_back]] | `g()` | `x` |\n";
    std::fs::write(dir.path().join("pub.md"), a).unwrap();
    std::fs::write(dir.path().join("con.md"), b).unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        json["data"]["issues"].as_array().unwrap().is_empty(),
        "mutual observation must not trip acyclic: {}",
        json["data"]["issues"]
    );
    assert!(
        json["warnings"].as_array().unwrap().is_empty(),
        "both effects are observed — no warnings either: {}",
        json["warnings"]
    );
}

#[test]
fn unobserved_effect_warns_exit_zero() {
    // A declared output nobody observes is an advisory warning on the
    // warnings channel — exit 0, never a failure (design D5; the doctor
    // knowledge-currency precedent).
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("pub.md"), OBSERVABILITY_PUB).unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "an unobserved effect is advisory, never a failure"
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let warnings = json["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|w| {
            let m = w["message"].as_str().unwrap_or_default();
            m.contains("linter.observability") && m.contains("pub.eff_out")
        }),
        "the unobserved effect must be warned with rule id + row id: {warnings:?}"
    );
    assert!(
        json["data"]["issues"].as_array().unwrap().is_empty(),
        "the warning must not ride the issues channel: {}",
        json["data"]["issues"]
    );
}

#[test]
fn graph_classifies_extension_point_host_as_external_boundary() {
    // A file hosting ≥1 extension_point Constraint is an external
    // boundary — derived from published contracts, never authored.
    let dir = tempfile::tempdir().unwrap();
    let pub_ext = OBSERVABILITY_PUB.replace(
        "| eff_out | effect | `output == {value}` | [[pub]] |",
        "| contract | extension_point | `interface == Output` | [[pub]] |\n| eff_out | effect | `output == {value}` | [[pub]] |",
    );
    std::fs::write(dir.path().join("pub.md"), pub_ext).unwrap();
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let boundaries = json["data"]["external_boundaries"]
        .as_array()
        .expect("external_boundaries must be in the graph payload");
    assert!(
        boundaries.iter().any(|b| b == "pub"),
        "pub hosts an extension_point row — it is an external boundary: {boundaries:?}"
    );
}

/// No-tag-drift (linter-observability spec): the boundary classification is
/// DERIVED from published contracts, never authored — remove the
/// extension_point rows and the classification disappears with them; there
/// is no stale tag left to clean up.
#[test]
fn graph_no_extension_point_rows_yield_no_external_boundary() {
    let dir = tempfile::tempdir().unwrap();
    // OBSERVABILITY_PUB declares only an effect row — no extension_point.
    std::fs::write(dir.path().join("pub.md"), OBSERVABILITY_PUB).unwrap();
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let boundaries = json["data"]["external_boundaries"]
        .as_array()
        .expect("external_boundaries must be in the graph payload");
    assert!(
        boundaries.is_empty(),
        "no extension_point rows → no external boundary, no stale tag: {boundaries:?}"
    );
}
