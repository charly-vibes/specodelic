// (split from tests/cli.rs — specodelic-g17 file_lines ratchet)
use super::*;

// ---- external_completeness (specs/linter-external_completeness.md, mp1 row 10) ----

/// A lint-clean two-constraint spec a checklist can map to.
fn write_mappable_spec(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL be mappable\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | c1 | invariant | `holds` | [[{id}]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- s1\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | t | s1 | s1 | [[{id}.c1]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p1 | unit | [[{id}.c1]] | `g()` | `x` |\n"
        ),
    )
    .unwrap();
}

#[test]
fn declared_checklist_with_violations_fails_lint() {
    // A declared checklist whose items go unconsulted / waiver without
    // rationale / duplicated claim — the external-completeness rules
    // fire through the CLI and the exit code is 1.
    let dir = tempfile::tempdir().unwrap();
    write_mappable_spec(&dir.path().join("x-file.md"), "x.file");
    std::fs::write(
        dir.path().join("ship.checklist.md"),
        "## Items\n\n- **unmapped**: never claimed\n- **norationale**: waived in silence\n\n\
         ## Mapping\n\n| item | status | mapped_ids | rationale |\n\
         |------|--------|------------|-----------|\n\
         | norationale | waived | | |\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "failures must exit 1");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let rules: Vec<&str> = json["data"]["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["rule_id"].as_str().unwrap())
        .collect();
    assert!(rules.contains(&"linter.every_item_accounted"), "{rules:?}");
    assert!(rules.contains(&"linter.waiver_has_rationale"), "{rules:?}");
}

#[test]
fn fully_mapped_checklist_lints_clean() {
    // A fully accounted checklist — covered rows resolving to real
    // rows, waiver carrying a rationale — is zero findings, exit 0.
    let dir = tempfile::tempdir().unwrap();
    write_mappable_spec(&dir.path().join("x-file.md"), "x.file");
    std::fs::write(
        dir.path().join("ship.checklist.md"),
        "## Items\n\n- **a.first**: sessions expire\n- **a.second**: out of scope\n\n\
         ## Mapping\n\n| item | status | mapped_ids | rationale |\n\
         |------|--------|------------|-----------|\n\
         | a.first | covered | [[x.file.c1]], [[x.file.p1]] | |\n\
         | a.second | waived | | tracked elsewhere |\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "clean checklist exits 0");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        json["data"]["issues"].as_array().unwrap().is_empty(),
        "{json:?}"
    );
}

#[test]
fn checklist_file_is_never_reported_as_skipped() {
    // The declaration convention: a *.checklist.md is intentional — it
    // must not surface a "skipped (no frontmatter)" note.
    let dir = tempfile::tempdir().unwrap();
    write_mappable_spec(&dir.path().join("x-file.md"), "x.file");
    std::fs::write(
        dir.path().join("ship.checklist.md"),
        "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n\
         | item | status | mapped_ids | rationale |\n\
         |------|--------|------------|-----------|\n\
         | a.first | covered | [[x.file.c1]] | |\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        json["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|w| !w.as_str().unwrap().contains("skipped")),
        "checklist files are intentional, never skipped-noise: {json:?}"
    );
}

#[test]
fn lint_payload_reports_declared_checklist_count() {
    // Honest-gate payload (Ro5 EXCL-002): the consumer can tell an
    // empty pass from a skipped one — checklists_declared is 1 here.
    let dir = tempfile::tempdir().unwrap();
    write_mappable_spec(&dir.path().join("x-file.md"), "x.file");
    std::fs::write(
        dir.path().join("ship.checklist.md"),
        "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n\
         | item | status | mapped_ids | rationale |\n\
         |------|--------|------------|-----------|\n\
         | a.first | covered | [[x.file.c1]] | |\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["data"]["checklists_declared"], 1, "{json:?}");
}

#[test]
fn bare_mapped_id_gets_the_dotted_spelling_hint() {
    // Ro5 CLAR-003: a single-segment mapped id can never be a row
    // reference — the finding must teach the file_id.row_id spelling.
    let dir = tempfile::tempdir().unwrap();
    write_mappable_spec(&dir.path().join("x-file.md"), "x.file");
    std::fs::write(
        dir.path().join("ship.checklist.md"),
        "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n\
         | item | status | mapped_ids | rationale |\n\
         |------|--------|------------|-----------|\n\
         | a.first | covered | c1 | |\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let msgs: Vec<&str> = json["data"]["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["message"].as_str().unwrap())
        .collect();
    assert!(
        msgs.iter()
            .any(|m| m.contains("dotted file_id.row_id spelling")),
        "{msgs:?}"
    );
}

// --- spk migrate (specodelic-c32, gh#6 item 1) ---

fn tempdir() -> tempfixture::TempDir {
    tempfixture::tempdir()
}

mod tempfixture {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    pub struct TempDir(pub PathBuf);
    impl TempDir {
        pub fn path(&self) -> &std::path::Path {
            &self.0
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    static N: AtomicUsize = AtomicUsize::new(0);
    pub fn tempdir() -> TempDir {
        let n = N.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let p = std::env::temp_dir().join(format!("spk-migrate-{pid}-{n}"));
        std::fs::create_dir_all(&p).unwrap();
        TempDir(p)
    }
}

/// End-to-end (task 4.1): plain delta → migrate → rewritten; second run refuses.
#[test]
fn migrate_wraps_delta_then_refuses_second_run() {
    let dir = tempdir();
    let file = dir.path().join("spec.md");
    std::fs::write(
        &file,
        "## ADDED Requirements\n\n### Requirement: Widget\nThe system SHALL wiget.\n",
    )
    .unwrap();
    spk()
        .arg("migrate")
        .arg(&file)
        .assert()
        .success()
        .stdout(contains("\"inserted_frontmatter\":true"))
        .stdout(contains("\"inserted_mirror\":true"));
    let migrated = std::fs::read_to_string(&file).unwrap();
    // Generated id is always `spec` — the dual-format naming law.
    assert!(migrated.starts_with("---\nid: spec\nkind: intent\n"));
    assert!(migrated.contains("## Requirements\n\n### Requirement: Widget"));
    // Idempotence: the second run is a refusal, file untouched.
    spk()
        .arg("migrate")
        .arg(&file)
        .assert()
        .failure()
        .stdout(contains("already dual-format"));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), migrated);
}

/// (task 4.2) The migrated scaffold lints clean through the real binary.
#[test]
fn migrated_scaffold_lints_clean() {
    let dir = tempdir();
    let file = dir.path().join("spec.md");
    std::fs::write(
        &file,
        "## ADDED Requirements\n\n### Requirement: Widget\nThe system SHALL wiget.\n",
    )
    .unwrap();
    spk().arg("migrate").arg(&file).assert().success();
    spk()
        .arg("lint")
        .arg(&file)
        .assert()
        .success()
        .stdout(contains("\"issues\":[]"));
}

/// (task 4.3) --dry-run prints the content, writes nothing.
#[test]
fn dry_run_leaves_disk_untouched() {
    let dir = tempdir();
    let file = dir.path().join("spec.md");
    let original = "## ADDED Requirements\n\n### Requirement: W\nThe system SHALL w.\n";
    std::fs::write(&file, original).unwrap();
    spk()
        .arg("migrate")
        .arg("--dry-run")
        .arg(&file)
        .assert()
        .success()
        .stdout(contains("## Requirements"));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), original);
}

/// Naming law: a generated id `spec` only lints when the file is named
/// spec.md — a differently-named file gets an explicit warning.
#[test]
fn non_spec_md_filename_warns_naming_law() {
    let dir = tempdir();
    let file = dir.path().join("other-name.md");
    std::fs::write(
        &file,
        "## ADDED Requirements\n\n### Requirement: W\nThe system SHALL w.\n",
    )
    .unwrap();
    spk()
        .arg("migrate")
        .arg(&file)
        .assert()
        .success()
        .stdout(contains("must be named spec.md"));
}

// ---------- refactor advisor (specodelic-3l7, specs/refactor.md) ----------

/// A minimal lint-tolerable spec: one constraint whose traces_to cell cites
/// each target in `refs`.
fn write_refactor_spec(path: &std::path::Path, id: &str, refs: &[&str]) {
    let traces = refs
        .iter()
        .map(|r| format!("[[{r}]]"))
        .collect::<Vec<_>>()
        .join(", ");
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL hold\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | c1 | invariant | `holds` | {traces} |\n"
        ),
    )
    .unwrap();
}

/// A spec with several constraint rows (`c1..cn`), for narrow-diff fixtures.
fn write_multi_row_spec(path: &std::path::Path, id: &str, n: usize, derives_from: Option<&str>) {
    // Every constraint traces its own intent (traces_to → Intent: legal
    // typing under the Reference Typing table). The optional Property p1
    // derives_from `target` (Property→Constraint: legal typing) — the
    // cross-row dependency edge a narrow-diff changeset can carry.
    let rows = (1..=n)
        .map(|i| format!("| c{i} | invariant | `holds` | [[{id}]] |"))
        .collect::<Vec<_>>()
        .join("\n");
    let props = derives_from
        .map(|target| {
            format!(
                "\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p1 | unit | [[{target}]] | `g()` | `x` |\n"
            )
        })
        .unwrap_or_default();
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL hold\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n{rows}\n{props}"
        ),
    )
    .unwrap();
}

/// unrelated_fan_in_flagged + related_fan_in_not_flagged
/// (specs/refactor.md Properties): a node referenced from two disjoint
/// top-level namespaces is flagged (threshold 2), a node referenced only
/// from within its own namespace subtree is clean — and the advisory
/// never gates (exit 0 either way).
#[test]
fn refactor_flags_high_unrelated_fan_in_and_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    let p = |name: &str| dir.path().join(name);
    write_refactor_spec(&p("hub.md"), "alpha.hub", &["alpha.hub"]);
    // Dependents cite the hub's INTENT ([[alpha.hub]]): traces_to → Intent
    // is the legal typing — a traces_to → Constraint edge is typing-
    // forbidden and the graph never records it (edge_kind_matches_typing).
    write_refactor_spec(&p("beta.md"), "beta.one", &["alpha.hub"]);
    write_refactor_spec(&p("gamma.md"), "gamma.one", &["alpha.hub"]);
    write_refactor_spec(&p("leaf.md"), "alpha.leaf", &["alpha.hub"]);
    write_refactor_spec(&p("dhub.md"), "delta.hub", &["delta.hub"]);
    write_refactor_spec(&p("ddep.md"), "delta.dep", &["delta.hub.c1"]);
    write_refactor_spec(&p("ddep2.md"), "delta.dep2", &["delta.hub.c1"]);
    let out = spk()
        .args([
            "refactor",
            dir.path().to_str().unwrap(),
            "--high-fan-in",
            "2",
            "--json",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "advisory never gates: {stdout}");
    let findings = json["data"]["findings"].as_array().unwrap();
    let hub = findings
        .iter()
        .find(|f| f["node_id"] == "alpha.hub")
        .unwrap_or_else(|| panic!("alpha.hub must be flagged: {stdout}"));
    // 3 incoming (beta, gamma, alpha.leaf) — 2 unrelated namespaces.
    assert_eq!(hub["dependent_count"], 3);
    assert_eq!(hub["unrelated_namespace_count"], 2);
    assert_eq!(hub["suggested_split"], true);
    // The emitted_finding_shape contract: exactly the four spec'd fields.
    assert_eq!(
        hub.as_object().unwrap().len(),
        4,
        "finding carries exactly node_id/dependent_count/unrelated_namespace_count/suggested_split: {hub}"
    );
    assert!(
        !findings.iter().any(|f| f["node_id"] == "delta.hub"),
        "coherent (same-namespace) fan-in is clean: {stdout}"
    );
}

/// related_fan_in_not_flagged: only within-namespace dependents → clean.
#[test]
fn refactor_coherent_fan_in_is_clean() {
    let dir = tempfile::tempdir().unwrap();
    let p = |name: &str| dir.path().join(name);
    write_refactor_spec(&p("dhub.md"), "delta.hub", &["delta.hub"]);
    write_refactor_spec(&p("ddep.md"), "delta.dep", &["delta.hub.c1"]);
    write_refactor_spec(&p("ddep2.md"), "delta.dep2", &["delta.hub.c1"]);
    write_refactor_spec(&p("ddep3.md"), "delta.dep3", &["delta.hub.c1"]);
    let out = spk()
        .args(["refactor", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{stdout}");
    assert!(
        json["data"]["findings"].as_array().unwrap().is_empty(),
        "same-namespace fan-in never flags: {stdout}"
    );
}

/// threshold_read_from_config: same node, two configs → different outcome.
/// The high-fan-in value is per-invocation configuration, never a constant.
#[test]
fn refactor_threshold_follows_config() {
    let dir = tempfile::tempdir().unwrap();
    let p = |name: &str| dir.path().join(name);
    write_refactor_spec(&p("hub.md"), "alpha.hub", &["alpha.hub"]);
    // Legal typing only: dependents cite the hub's intent node.
    write_refactor_spec(&p("beta.md"), "beta.one", &["alpha.hub"]);
    write_refactor_spec(&p("gamma.md"), "gamma.one", &["alpha.hub"]);
    let path = dir.path().to_str().unwrap();
    let high = spk()
        .args(["refactor", path, "--high-fan-in", "2", "--json"])
        .output()
        .unwrap();
    let loose = spk()
        .args(["refactor", path, "--high-fan-in", "5", "--json"])
        .output()
        .unwrap();
    let flags = |out: &std::process::Output| {
        let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        !json["data"]["findings"].as_array().unwrap().is_empty()
    };
    assert!(flags(&high), "unrelated count 2 ≥ threshold 2 → flagged");
    assert!(
        !flags(&loose),
        "unrelated count 2 < threshold 5 → clean (decision follows config)"
    );
}

/// narrow_diff_flagged_low_fan_in: a changeset touching a strict subset of
/// a node's owned rows while depending on none of its others is flagged
/// even though fan-in alone wouldn't trigger (fan-in 1 < default 3).
#[test]
fn refactor_narrow_diff_flagged_regardless_of_fan_in() {
    let dir = tempfile::tempdir().unwrap();
    // n: five constraints (all tracing their own intent — legal typing),
    // m: one Property deriving from n.c1 (Property→Constraint, legal) —
    // n's fan-in is 1, below the default threshold of 3.
    write_multi_row_spec(&dir.path().join("n.md"), "n", 5, None);
    write_multi_row_spec(&dir.path().join("m.md"), "m", 1, Some("n.c1"));
    let out = spk()
        .args([
            "refactor",
            dir.path().to_str().unwrap(),
            "--changeset",
            "n.c1",
            "--json",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{stdout}");
    let findings = json["data"]["findings"].as_array().unwrap();
    let n = findings
        .iter()
        .find(|f| f["node_id"] == "n")
        .unwrap_or_else(|| panic!("narrow diff on n must be flagged: {stdout}"));
    assert_eq!(n["suggested_split"], true);
    assert_eq!(n["dependent_count"], 1, "fan-in alone wouldn't flag here");
}

/// narrow_diff heuristics' coherence clause: when the changeset's rows DO
/// depend on the node's other owned rows, the changeset needs the node
/// whole — no flag.
#[test]
fn refactor_narrow_diff_with_coherent_dependency_is_clean() {
    let dir = tempfile::tempdir().unwrap();
    // The changeset {n.c1, n.p1}: c1 plus the Property p1, which derives
    // from n.c3 — a row the changeset does NOT touch. The changeset
    // depends on n's other owned rows, so it needs the node whole.
    write_multi_row_spec(&dir.path().join("n.md"), "n", 5, Some("n.c3"));
    let out = spk()
        .args([
            "refactor",
            dir.path().to_str().unwrap(),
            "--changeset",
            "n.c1,n.p1",
            "--json",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{stdout}");
    assert!(
        json["data"]["findings"].as_array().unwrap().is_empty(),
        "changeset depending on the node's other rows is coherent: {stdout}"
    );
}

/// archive-companion fixture: a fake repo root (a `.git` dir is all
/// repo_root() needs to anchor) with an already-archived change.
fn archive_companion_fixture(id: &str, delta: &str) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join(".git")).unwrap();
    let delta_path = root.path().join(format!(
        "openspec/changes/archive/2026-10-01-{id}/specs/c/spec.md"
    ));
    std::fs::create_dir_all(delta_path.parent().unwrap()).unwrap();
    std::fs::write(&delta_path, delta).unwrap();
    root
}

const DUAL_DELTA_FIXTURE: &str = "---\nid: spec\nkind: intent\nstatement: \"SHALL x.\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n";

/// archive-companion dry-run over an already-archived change: exit 0,
/// envelope lists the planned restore, and NOTHING is written
/// (specodelic-fzo, dry_run_never_mutates).
#[test]
fn archive_companion_dry_run_reports_without_writing() {
    let root = archive_companion_fixture("dryx", DUAL_DELTA_FIXTURE);
    let out = spk()
        .args(["archive-companion", "dryx", "--dry-run", "--json"])
        .current_dir(root.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{stdout}");
    assert_eq!(json["data"]["dry_run"], serde_json::json!(true));
    assert_eq!(json["data"]["openspec_ran"], serde_json::json!(false));
    assert_eq!(
        json["data"]["restored"],
        serde_json::json!(["openspec/specs/c/spec.md"])
    );
    assert!(
        !root.path().join("openspec/specs").exists(),
        "dry-run must not create deployed spec paths"
    );
}

/// archive-companion fails closed on a frontmatterless archived delta
/// (specodelic-fzo, fail_closed_unverifiable): exit 1, the delta is
/// named, and no deployed spec is written.
#[test]
fn archive_companion_refuses_frontmatterless_delta() {
    let root = archive_companion_fixture("plain", "# plain openspec delta\n");
    let out = spk()
        .args(["archive-companion", "plain", "--json"])
        .current_dir(root.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("dual-format layer"),
        "refusal must name the failure: {stderr}"
    );
    assert!(
        stderr.contains("spk explain dual-format"),
        "refusal must carry the migration-recipe hint: {stderr}"
    );
    assert!(
        !root.path().join("openspec/specs").exists(),
        "a refusal must not write any deployed spec"
    );
}

/// archive-companion over an unknown change id: labeled error naming
/// the id and the `openspec list` hint.
#[test]
fn archive_companion_unknown_change_is_labeled_error() {
    let root = archive_companion_fixture("other", DUAL_DELTA_FIXTURE);
    let out = spk()
        .args(["archive-companion", "ghost-change", "--json"])
        .current_dir(root.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("ghost-change"), "{stderr}");
    assert!(stderr.contains("openspec list"), "{stderr}");
}

// ---- specodelic-9rv: spk parse — structured Spec IR export ----

// A well-formed four-layer spec with three Properties rows and links —
// the parse IR fixture.
//
// C-parse-ir + C-parse-envelope (tasks 1.1): a well-formed file parses
// into the full Spec IR under data, with a hint suggesting spk lint.
#[test]
fn parse_exports_full_spec_ir() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("parse_fixture.md");
    write_parse_fixture(&file, "parse_fixture");
    let out = spk()
        .args(["parse", file.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(env["ok"], serde_json::Value::Bool(true));
    // Shared envelope fields all present.
    for field in ["ok", "data", "warnings", "hints", "meta"] {
        assert!(
            env.get(field).is_some(),
            "envelope missing `{field}`: {stdout}"
        );
    }
    let data = &env["data"];
    // Full IR, no structured field omitted.
    assert_eq!(data["intent"]["id"], "parse_fixture");
    assert_eq!(data["constraints"].as_array().map(Vec::len), Some(2));
    assert_eq!(data["states"].as_array().map(Vec::len), Some(2));
    assert_eq!(data["transitions"].as_array().map(Vec::len), Some(1));
    assert_eq!(data["properties"].as_array().map(Vec::len), Some(3));
    assert_eq!(data["links"].as_array().map(Vec::len).unwrap_or(0), 6);
    // Properties rows carry structured cells, not generated text.
    assert_eq!(data["properties"][0]["id"], "p1");
    // Hint suggests linting the parsed file.
    let hints = env["hints"].as_array().cloned().unwrap_or_default();
    assert!(
        serde_json::to_string(&hints).unwrap().contains("spk lint")
            || serde_json::to_string(&hints).unwrap().contains("lint"),
        "hints must suggest spk lint: {hints:?}"
    );
    assert_eq!(out.status.code(), Some(0));
}

/// C-parse-lint-independent (task 1.2): parse succeeds on a lint-dirty
/// file and the envelope embeds no lint status.
#[test]
fn parse_is_lint_independent() {
    let dir = tempfile::tempdir().unwrap();
    // Parses fine but fails lint: the statement violates the EARS grammar.
    let file = dir.path().join("lint_dirty.md");
    write_bad_ears_spec(&file, "lint_dirty");
    let out = spk()
        .args(["parse", file.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(env["ok"], serde_json::Value::Bool(true), "{stdout}");
    assert_eq!(out.status.code(), Some(0));
    // No lint status anywhere in the envelope.
    let blob = serde_json::to_string(&env).unwrap().to_lowercase();
    assert!(
        !blob.contains("lint_ok") && !blob.contains("\"linted\"") && !blob.contains("lint_status"),
        "envelope must not embed lint status: {blob}"
    );
    assert_eq!(env["data"]["intent"]["id"], "lint_dirty");
}

/// C-parse-error (task 1.3): unparseable input and a nonexistent path
/// each yield a labeled error envelope with a hint and non-zero exit.
#[test]
fn parse_rejects_unparseable_and_missing_input() {
    let dir = tempfile::tempdir().unwrap();
    // Unparseable: broken YAML frontmatter.
    let bad = dir.path().join("broken.md");
    std::fs::write(&bad, "---\nid: [unclosed\nkind: intent\n---\nbody\n").unwrap();
    let out = spk()
        .args(["parse", bad.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(env["ok"], serde_json::Value::Bool(false), "{stdout}");
    let blob = serde_json::to_string(&env).unwrap();
    assert!(
        blob.contains("broken.md"),
        "error must name the file: {blob}"
    );
    assert!(
        !env["hints"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .is_empty()
            || !env["next_step"].is_null(),
        "error envelope must carry a remediation hint: {blob}"
    );
    assert_ne!(out.status.code(), Some(0));

    // Nonexistent path.
    let ghost = dir.path().join("ghost.md");
    let out = spk()
        .args(["parse", ghost.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(env["ok"], serde_json::Value::Bool(false), "{stdout}");
    let blob = serde_json::to_string(&env).unwrap();
    assert!(
        blob.contains("ghost.md"),
        "error must name the path: {blob}"
    );
    assert_ne!(out.status.code(), Some(0));
}

/// C-parse-single-file (task 1.3): exactly one path per invocation —
/// zero or multiple paths are a usage error.
#[test]
fn parse_accepts_exactly_one_path() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("one.md");
    write_parse_fixture(&file, "one");
    let other = dir.path().join("two.md");
    write_parse_fixture(&other, "two");

    // Zero paths.
    let out = spk().args(["parse", "--json"]).output().unwrap();
    assert_ne!(out.status.code(), Some(0), "zero paths must be rejected");
    // Two paths.
    let out = spk()
        .args([
            "parse",
            file.to_str().unwrap(),
            other.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(0), "two paths must be rejected");
}

// -- domain packs (specs/packs.md, Revision 14; specodelic-dcx) ------------

/// A well-formed `kind: profile` pack file — all six manifest tables,
/// full lifecycle machine, base pin at the current revision.
fn write_pack_file(path: &std::path::Path, id: &str, revision: u32) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: profile\nstatement: \"WHEN a workspace enables this pack, THE format SHALL provide the {id} vocabulary and its declared checkers\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | vocab_declared | invariant | `the six manifest tables declare the vocabulary` | [[{id}]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- draft\n\
             - published\n\
             - deprecated\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | publish | draft | published | [[{id}.vocab_declared]] |\n\
             | deprecate | published | deprecated | [[{id}.vocab_declared]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p | unit | [[{id}.vocab_declared]] | `g()` | `x` |\n\
             \n## Sections\n\
             \n| section | row_shape |\n\
             |---------|-----------|\n\
             | Data | `\\| name \\| dtype \\|` |\n\
             \n## Kinds\n\
             \n| kind | vocabulary |\n\
             |------|------------|\n\
             | {id}.tolerance | `an empirical bound` |\n\
             \n## References\n\
             \n| field | resolves_to |\n\
             |-------|-------------|\n\
             | measures | `a {id}.tolerance kind row` |\n\
             \n## Checkers\n\
             \n| rule | semantics |\n\
             |------|-----------|\n\
             | {id}.bound_present | `every {id}.tolerance row carries a bound` |\n\
             \n## Floors\n\
             \n| kind | required_cases |\n\
             |------|----------------|\n\
             | {id}.tolerance | `identity` |\n\
             \n## Requires\n\
             \n| dep | revision |\n\
             |-----|----------|\n\
             | base | specodelic.md Revision {revision} |\n"
        ),
    )
    .unwrap();
}

/// A lint-clean consumer intent spec, optionally carrying a `uses` column
/// and pack vocabulary in its expr.
fn write_consumer_spec(path: &std::path::Path, id: &str, uses: &str, vocab: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to | uses |\n\
             |----|------|------|-----------|------|\n\
             | c1 | invariant | `{vocab}` | [[{id}]] | {uses} |\n\
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
             | p | unit | [[{id}.c1]] | `g()` | `x` |\n"
        ),
    )
    .unwrap();
}

/// A consumer spec whose Properties and Constraints rows carry a
/// pack-qualified fiber kind in their base-table `kind` columns
/// (specodelic-ung).
fn write_fiber_kind_consumer(path: &std::path::Path, id: &str, uses: &str, fiber: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to | uses |\n\
             |----|------|------|-----------|------|\n\
             | c1 | {fiber} | `holds` | [[{id}]] | {uses} |\n\
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
             | p | {fiber} | [[{id}.c1]] | `g()` | `x` |\n"
        ),
    )
    .unwrap();
}

#[test]
fn fiber_kinds_typeable_when_pack_active_finding_without() {
    // specodelic-ung: the closed-set walkers' effective set is
    // base ∪ active-pack-fiber — a base-table kind column carrying a
    // pack-qualified fiber kind is accepted when the pack is active
    // (declared uses edge or vocabulary match), and the same token
    // fires the labeled finding when no pack declares it — never a
    // silent pass.
    let dir = tempfile::tempdir().unwrap();
    write_pack_file(&dir.path().join("bioimage.md"), "bioimage", 14);
    // declared uses edge → active
    write_fiber_kind_consumer(
        &dir.path().join("declared.md"),
        "declared",
        "[[bioimage]]",
        "bioimage.tolerance",
    );
    // no uses edge, but the kind token vocabulary-matches → active
    write_fiber_kind_consumer(
        &dir.path().join("implicit.md"),
        "implicit",
        "",
        "bioimage.tolerance",
    );
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "active fiber kinds accepted: {json}"
    );
    let issues = json["data"]["issues"].as_array().unwrap();
    assert!(
        !issues.iter().any(|i| {
            i["rule_id"] == "linter.property_kind_closed"
                || i["rule_id"] == "linter.constraint_kind_closed"
        }),
        "fiber kinds must not fire the closed-set walkers when the pack is active: {issues:?}"
    );

    // the same token with no pack discovered → the labeled finding fires
    let dir = tempfile::tempdir().unwrap();
    write_fiber_kind_consumer(
        &dir.path().join("orphan.md"),
        "orphan",
        "",
        "bioimage.tolerance",
    );
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_ne!(out.status.code(), Some(0));
    let issues = json["data"]["issues"].as_array().unwrap();
    assert!(
        issues
            .iter()
            .any(|i| i["rule_id"] == "linter.property_kind_closed"),
        "without the pack active the token stays outside the closed set: {issues:?}"
    );
}

#[test]
fn well_formed_pack_lints_clean_and_is_discovered() {
    let dir = tempfile::tempdir().unwrap();
    write_pack_file(&dir.path().join("bioimage.md"), "bioimage", 14);
    write_consumer_spec(&dir.path().join("consumer.md"), "consumer", "", "plain");
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{json}");
    let issues = json["data"]["issues"].as_array().unwrap();
    assert!(issues.is_empty(), "pack corpus lint findings: {issues:?}");
    let packs = json["data"]["packs"].as_array().expect("packs surfaced");
    assert_eq!(packs.len(), 1);
    assert_eq!(packs[0]["id"], "bioimage");
    assert_eq!(packs[0]["lifecycle"], "published");
}

#[test]
fn missing_manifest_table_is_a_labeled_pack_shape_finding() {
    let dir = tempfile::tempdir().unwrap();
    write_pack_file(&dir.path().join("bioimage.md"), "bioimage", 14);
    // drop the ## Floors table
    let text = std::fs::read_to_string(dir.path().join("bioimage.md")).unwrap();
    let stripped = text.split("## Floors").next().unwrap();
    std::fs::write(dir.path().join("bioimage.md"), stripped).unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_ne!(out.status.code(), Some(0));
    let issues = json["data"]["issues"].as_array().unwrap();
    assert!(
        issues.iter().any(|i| {
            i["rule_id"] == "linter.pack_shape" && i["message"].as_str().unwrap().contains("Floors")
        }),
        "no pack_shape finding naming Floors: {issues:?}"
    );
}

#[test]
fn malformed_manifest_row_is_a_labeled_pack_shape_finding() {
    let dir = tempfile::tempdir().unwrap();
    write_pack_file(&dir.path().join("bioimage.md"), "bioimage", 14);
    // Sections row with three cells (the facet's fixed row shape is 2)
    let text = std::fs::read_to_string(dir.path().join("bioimage.md")).unwrap();
    let bad = text.replace(
        "| Data | `\\| name \\| dtype \\|` |",
        "| Data | `x` | `extra` |",
    );
    assert_ne!(bad, text, "mutation must apply");
    std::fs::write(dir.path().join("bioimage.md"), bad).unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_ne!(out.status.code(), Some(0));
    let issues = json["data"]["issues"].as_array().unwrap();
    assert!(
        issues.iter().any(|i| {
            i["rule_id"] == "linter.pack_shape"
                && i["message"].as_str().unwrap().contains("Sections")
        }),
        "no pack_shape finding naming Sections: {issues:?}"
    );
}

#[test]
fn non_profile_file_claiming_a_manifest_is_a_finding() {
    let dir = tempfile::tempdir().unwrap();
    write_consumer_spec(&dir.path().join("consumer.md"), "consumer", "", "plain");
    // a kind: intent file carrying manifest headings
    let text = std::fs::read_to_string(dir.path().join("consumer.md")).unwrap();
    std::fs::write(
        dir.path().join("sneaky.md"),
        text.replace("id: consumer", "id: sneaky")
            .replace("THE system SHALL hold", "THE sneaky SHALL claim a manifest")
            + "\n## Sections\n\n| section | row_shape |\n|---------|-----------|\n| Data | `x` |\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_ne!(out.status.code(), Some(0));
    let issues = json["data"]["issues"].as_array().unwrap();
    assert!(
        issues.iter().any(|i| {
            i["rule_id"] == "linter.pack_shape"
                && i["message"].as_str().unwrap().contains("profile")
        }),
        "no pack_shape finding for non-profile manifest: {issues:?}"
    );
}

#[test]
fn pack_kinds_must_be_pack_qualified_narrowing_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    write_pack_file(&dir.path().join("bioimage.md"), "bioimage", 14);
    // a Kinds row claiming a base closed-set name unqualified
    let text = std::fs::read_to_string(dir.path().join("bioimage.md")).unwrap();
    let bad = text.replace(
        "| bioimage.tolerance | `an empirical bound` |",
        "| invariant | `an empirical bound` |",
    );
    assert_ne!(bad, text, "mutation must apply");
    std::fs::write(dir.path().join("bioimage.md"), bad).unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_ne!(out.status.code(), Some(0));
    let issues = json["data"]["issues"].as_array().unwrap();
    assert!(
        issues.iter().any(|i| {
            i["rule_id"] == "linter.pack_shape"
                && i["message"].as_str().unwrap().contains("append-only")
        }),
        "no narrowing finding: {issues:?}"
    );
}

#[test]
fn no_profile_files_means_pack_machinery_is_inert() {
    let dir = tempfile::tempdir().unwrap();
    write_consumer_spec(&dir.path().join("plain.md"), "plain", "", "plain");
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(0));
    // byte-identical guarantee: no packs key surfaces when nothing is discovered
    assert!(
        json["data"].get("packs").is_none(),
        "packs key surfaced with no profile files: {json}"
    );
    let warnings = json["warnings"].as_array().unwrap();
    assert!(
        !warnings
            .iter()
            .any(|w| w["message"].as_str().unwrap_or("").contains("pack")),
        "pack advisory with no packs: {warnings:?}"
    );
}

#[test]
fn vocabulary_use_activates_the_pack_advisory() {
    let dir = tempfile::tempdir().unwrap();
    write_pack_file(&dir.path().join("bioimage.md"), "bioimage", 14);
    write_consumer_spec(
        &dir.path().join("consumer.md"),
        "consumer",
        "",
        "bioimage.tolerance applies here",
    );
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{json}");
    let warnings = json["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|w| {
            w["message"].as_str().unwrap_or("").contains("bioimage")
                && w["message"].as_str().unwrap_or("").contains("consumer")
        }),
        "no activation advisory: {warnings:?}"
    );
}

#[test]
fn declared_uses_edge_enables_the_pack() {
    let dir = tempfile::tempdir().unwrap();
    write_pack_file(&dir.path().join("bioimage.md"), "bioimage", 14);
    write_consumer_spec(
        &dir.path().join("consumer.md"),
        "consumer",
        "[[bioimage]]",
        "plain",
    );
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{json}");
    let warnings = json["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|w| {
            let s = w["message"].as_str().unwrap_or("");
            s.contains("bioimage") && s.contains("declared uses edge")
        }),
        "no declared-mode advisory: {warnings:?}"
    );
}

#[test]
fn uses_edge_to_a_missing_pack_is_a_labeled_orphan_failure() {
    let dir = tempfile::tempdir().unwrap();
    write_consumer_spec(
        &dir.path().join("consumer.md"),
        "consumer",
        "[[ghost]]",
        "plain",
    );
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_ne!(out.status.code(), Some(0));
    let issues = json["data"]["issues"].as_array().unwrap();
    assert!(
        issues.iter().any(|i| {
            let m = i["message"].as_str().unwrap_or("");
            m.contains("ghost") && m.contains("kind: profile") && m.contains("fix")
        }),
        "no labeled orphan finding naming candidate pack + both remediations: {issues:?}"
    );
}

#[test]
fn vocabulary_used_without_any_pack_is_a_labeled_orphan_failure() {
    // specodelic-erd: the vocabulary half of orphan_vocabulary_labeled —
    // a pack-qualified kind token used with no pack discovered and no
    // `uses` edge produces the labeled orphan finding naming the
    // prefix-derived candidate pack and both remediations, never
    // silent.
    let dir = tempfile::tempdir().unwrap();
    write_fiber_kind_consumer(&dir.path().join("orphan.md"), "orphan", "", "data.dataset");
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_ne!(out.status.code(), Some(0));
    let issues = json["data"]["issues"].as_array().unwrap();
    let orphan = issues
        .iter()
        .find(|i| i["rule_id"] == "linter.orphan_vocabulary")
        .expect("no labeled orphan finding for pack vocabulary with no pack: {issues:?}");
    let m = orphan["message"].as_str().unwrap();
    assert!(
        m.contains("data.dataset") && m.contains("data.*") && m.contains("kind: profile"),
        "orphan finding must name the token, the prefix-derived candidate pack, and the pack kind: {m}"
    );
    assert!(
        m.contains("add/enable") && m.contains("fix the vocabulary"),
        "orphan finding must carry both remediations: {m}"
    );
}

#[test]
fn vocabulary_orphan_suppressed_when_uses_edge_names_the_pack() {
    // A concrete `uses`-edge orphan for the same namespace already names
    // the candidate pack — the prefix-derived vocabulary finding must
    // not duplicate it.
    let dir = tempfile::tempdir().unwrap();
    write_fiber_kind_consumer(
        &dir.path().join("orphan.md"),
        "orphan",
        "[[data.lineage]]",
        "data.dataset",
    );
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let issues = json["data"]["issues"].as_array().unwrap();
    let orphans: Vec<&serde_json::Value> = issues
        .iter()
        .filter(|i| i["rule_id"] == "linter.orphan_vocabulary")
        .collect();
    assert_eq!(
        orphans.len(),
        1,
        "one orphan finding per namespace, the concrete one: {issues:?}"
    );
    let m = orphans[0]["message"].as_str().unwrap();
    assert!(
        m.contains("data.lineage") && !m.contains("data.*"),
        "the concrete uses-edge finding must win: {m}"
    );
}

#[test]
fn vocabulary_orphan_absent_when_namespace_pack_discovered() {
    // A discovered pack in the token's namespace means the token is in
    // an active-namespace workspace — vocabulary matching and the
    // closed-set walkers handle it; no orphan fires.
    let dir = tempfile::tempdir().unwrap();
    // write_pack_file declares the `{id}.tolerance` kind; a dotted id
    // namespaces it (`data.lineage.tolerance` — namespace `data`).
    write_pack_file(&dir.path().join("data-lineage.md"), "data.lineage", 15);
    write_fiber_kind_consumer(
        &dir.path().join("consumer.md"),
        "consumer",
        "",
        "data.lineage.tolerance",
    );
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let issues = json["data"]["issues"].as_array().unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "active vocabulary is advisory: {issues:?}"
    );
    assert!(
        !issues
            .iter()
            .any(|i| i["rule_id"] == "linter.orphan_vocabulary"),
        "no orphan when the namespace's pack is discovered: {issues:?}"
    );
}

#[test]
fn overlapping_vocabulary_activates_both_packs() {
    let dir = tempfile::tempdir().unwrap();
    write_pack_file(&dir.path().join("alpha.md"), "alpha", 14);
    write_pack_file(&dir.path().join("beta.md"), "beta", 14);
    write_consumer_spec(
        &dir.path().join("consumer.md"),
        "consumer",
        "",
        "shared.tolerance applies here",
    );
    // both packs declare the identical token
    for (pack, id) in [("alpha.md", "alpha"), ("beta.md", "beta")] {
        let path = dir.path().join(pack);
        let text = std::fs::read_to_string(&path).unwrap();
        let needle = format!("| {id}.tolerance | `an empirical bound` |");
        let grown = text.replace(
            &needle,
            &format!("{needle}\n| shared.tolerance | `an empirical bound` |"),
        );
        assert_ne!(grown, text, "mutation must apply for {id}");
        std::fs::write(&path, grown).unwrap();
    }
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{json}");
    let warnings = json["warnings"].as_array().unwrap();
    for pack in ["alpha", "beta"] {
        assert!(
            warnings.iter().any(|w| w["message"]
                .as_str()
                .unwrap_or("")
                .contains(&format!("pack `{pack}` activated"))),
            "pack {pack} not activated: {warnings:?}"
        );
    }
}

#[test]
fn pack_file_linted_alone_is_self_exempt() {
    let dir = tempfile::tempdir().unwrap();
    write_pack_file(&dir.path().join("bioimage.md"), "bioimage", 14);
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{json}");
    let issues = json["data"]["issues"].as_array().unwrap();
    assert!(
        issues.is_empty(),
        "self-exempt pack has findings: {issues:?}"
    );
    // the pack's own vocabulary never activates anything — no advisories
    let warnings = json["warnings"].as_array().unwrap();
    assert!(
        !warnings
            .iter()
            .any(|w| w["message"].as_str().unwrap_or("").contains("activated")),
        "pack activated itself: {warnings:?}"
    );
}

#[test]
fn revision_skew_is_a_warning_not_a_failure() {
    let dir = tempfile::tempdir().unwrap();
    write_pack_file(&dir.path().join("bioimage.md"), "bioimage", 10);
    write_consumer_spec(
        &dir.path().join("consumer.md"),
        "consumer",
        "[[bioimage]]",
        "plain",
    );
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{json}");
    let warnings = json["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|w| {
            let s = w["message"].as_str().unwrap_or("");
            s.contains("bioimage") && s.contains("Revision 10") && s.contains("Revision 16")
        }),
        "no skew advisory naming both revisions: {warnings:?}"
    );
}

#[test]
fn draft_pack_findings_name_the_draft_status() {
    let dir = tempfile::tempdir().unwrap();
    write_pack_file(&dir.path().join("bioimage.md"), "bioimage", 14);
    // single-state draft model
    let text = std::fs::read_to_string(dir.path().join("bioimage.md")).unwrap();
    let draft = text
        .replace("- draft\n- published\n- deprecated\n", "- draft\n")
        .replace(
            "| publish | draft | published | [[bioimage.vocab_declared]] |\n| deprecate | published | deprecated | [[bioimage.vocab_declared]] |",
            "| self | draft | draft | [[bioimage.vocab_declared]] |",
        );
    assert_ne!(draft, text, "mutation must apply");
    std::fs::write(dir.path().join("bioimage.md"), draft).unwrap();
    write_consumer_spec(
        &dir.path().join("consumer.md"),
        "consumer",
        "",
        "bioimage.tolerance applies here",
    );
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{json}");
    let warnings = json["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|w| {
            let s = w["message"].as_str().unwrap_or("");
            s.contains("bioimage") && s.contains("draft")
        }),
        "draft status not named: {warnings:?}"
    );
}

#[test]
fn deprecated_pack_findings_name_the_deprecation() {
    let dir = tempfile::tempdir().unwrap();
    write_pack_file(&dir.path().join("bioimage.md"), "bioimage", 14);
    let text = std::fs::read_to_string(dir.path().join("bioimage.md")).unwrap();
    let dead = text
        .replace("- draft\n- published\n- deprecated\n", "- deprecated\n")
        .replace(
            "| publish | draft | published | [[bioimage.vocab_declared]] |\n| deprecate | published | deprecated | [[bioimage.vocab_declared]] |",
            "| self | deprecated | deprecated | [[bioimage.vocab_declared]] |",
        );
    assert_ne!(dead, text, "mutation must apply");
    std::fs::write(dir.path().join("bioimage.md"), dead).unwrap();
    write_consumer_spec(
        &dir.path().join("consumer.md"),
        "consumer",
        "",
        "bioimage.tolerance applies here",
    );
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{json}");
    let warnings = json["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|w| {
            let s = w["message"].as_str().unwrap_or("");
            s.contains("bioimage") && s.contains("deprecated")
        }),
        "deprecation not named: {warnings:?}"
    );
}

#[test]
fn subdirectory_lint_discovers_workspace_packs() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("work")).unwrap();
    write_pack_file(&dir.path().join("bioimage.md"), "bioimage", 14);
    write_consumer_spec(
        &dir.path().join("work/consumer.md"),
        "consumer",
        "",
        "bioimage.tolerance applies here",
    );
    // anchor the scan at the git toplevel — the tempdir becomes a repo
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let out = spk()
        .args(["lint", "work", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{json}");
    let packs = json["data"]["packs"].as_array().expect("packs surfaced");
    assert_eq!(packs.len(), 1, "workspace pack not discovered: {packs:?}");
    assert_eq!(packs[0]["id"], "bioimage");
}

#[test]
fn doctor_surfaces_discovered_packs() {
    // task 4.2: the pack mechanism is surfaced in `spk doctor` output —
    // the workspace's `kind: profile` files reported with their lifecycle
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("specs")).unwrap();
    write_pack_file(&dir.path().join("specs/bioimage.md"), "bioimage", 14);
    let out = spk()
        .args(["doctor", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(0));
    let packs = json["data"]["packs"].as_array().expect("packs surfaced");
    assert_eq!(packs.len(), 1);
    assert_eq!(packs[0]["id"], "bioimage");
}

// ---- kernel.binding claim carrier (add-min-expr-kernel §5.2, specodelic-bf5) ----

#[test]
fn compile_cli_surfaces_binding_in_the_toml_artifact() {
    // The claim-carrier surface end-to-end: an invariant-kind
    // Constraint's kernel.binding cell surfaces verbatim in the
    // `{stem}.toml` artifact the CLI writes (the material contract-TOML
    // `flags` authors bind against — node id / `-k` / [[tests.shell]]).
    // No registry is built and the contents are never interpreted.
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("bind_demo.md");
    // The pack-path enablement for the `kernel.binding` vocabulary
    // (specs/packs.md): the dotted column header is a pack-qualified
    // token in namespace `kernel`, and orphan_vocabulary's own
    // remediation is a discovered `kind: profile` pack in that
    // namespace. The fixture ships the minimal such pack so the compile
    // precondition passes exactly the way an enabled workspace would.
    write_pack_file(&dir.path().join("kernel.md"), "kernel", 16);
    std::fs::write(
        &spec,
        "---\nid: bind_demo\nkind: intent\nstatement: \"THE demo SHALL carry a binding claim\"\n---\n\
         \n## Constraints\n\
         \n| id | kind | expr | traces_to | kernel.binding |\n\
         |----|------|------|-----------|----------------|\n\
         | c1 | invariant | `holds` | [[bind_demo]] |  |\n\
         | b1 | invariant | `**kernel:** |State| == 2` | [[bind_demo]] | `-k kernel_binding and not slow` |\n\
         | b2 | invariant | `**kernel:** |Intent| == 1` | [[bind_demo]] | `printf '%s' 'a|b{c}\\\\' && exit 1 # {drop}` |\n\
         \n## Model\n\
         \n### States\n\
         \n- s1\n\
         - s2\n\
         \n### Transitions\n\
         \n| id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | t | s1 | s2 | [[bind_demo.c1]] |\n\
         \n## Properties\n\
         \n| id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|------------|\n\
         | p_c1 | unit | [[bind_demo.c1]] | `g()` | `x` |\n\
         | p_b1 | unit | [[bind_demo.b1]] | `g()` | `x` |\n\
         | p_b2 | unit | [[bind_demo.b2]] | `g()` | `x` |\n",
    )
    .unwrap();
    let out = dir.path().join("out");
    spk()
        .args([
            "compile",
            dir.path().to_str().unwrap(),
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();
    let toml = std::fs::read_to_string(out.join("bind_demo.toml")).unwrap();
    assert!(
        toml.contains("binding = \"-k kernel_binding and not slow\""),
        "ordinary binding surfaces verbatim in the CLI artifact: {toml}"
    );
    assert!(
        toml.contains("printf '%s' 'a|b{c}\\\\' && exit 1 # {drop}"),
        "arbitrary binding surfaces verbatim in the CLI artifact: {toml}"
    );
    // The row without binding text carries an empty carrier, not absence
    // (present column ⇒ every invariant row carries the cell).
    assert!(
        toml.contains("binding = \"\""),
        "empty binding cell surfaces as the empty string: {toml}"
    );
}

// ---- graph --format edges (add-graph-views tasks 1.1/1.2/1.3/1.7) ----

/// A spec whose Constraints row shares the intent id — the shape that
/// anchors extraction on the display label `id (intent)`
/// (add-graph-views D2's evidence shape).
fn write_self_anchored_spec(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL anchor on the intent\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | {id} | invariant | `row shares the intent id` | [[{id}]] |\n"
        ),
    )
    .unwrap();
}

/// A lint-clean spec with two states, one transition, one intent link,
/// and one duplicated reference instance — the raw-projection fixture
/// (task 1.7: distinct qualified IDs, both transition edges, multiplicity).
fn write_two_state_spec(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL be a two-state machine\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | c1 | invariant | `holds` | [[{id}]] [[{id}]] |\n\
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
    .unwrap();
}

/// A spec with one typing violation: a Constraint tracing to another
/// Constraint (traces_to must resolve to an Intent).
fn write_violation_spec(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL violate typing once\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | a | invariant | `fine` | [[{id}]] |\n\
             | b | invariant | `bad target kind` | [[{id}.a]] |\n"
        ),
    )
    .unwrap();
}

fn edges_rows(out: &[u8]) -> Vec<Vec<&str>> {
    std::str::from_utf8(out)
        .unwrap()
        .lines()
        .map(|l| l.split('\t').collect())
        .collect()
}

#[test]
fn edges_projection_emits_sorted_six_column_tsv() {
    let dir = tempfile::tempdir().unwrap();
    write_two_state_spec(&dir.path().join("two.md"), "two");
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--format", "edges"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "edges projection exits 0");
    let rows = edges_rows(&out.stdout);
    assert!(!rows.is_empty(), "a two-state corpus yields rows");
    for row in &rows {
        assert_eq!(row.len(), 6, "every row has exactly six columns: {row:?}");
    }
    let mut sorted = rows.clone();
    sorted.sort();
    assert_eq!(rows, sorted, "rows are emitted in sorted order");
}

#[test]
fn edges_projection_is_byte_identical_on_rerun() {
    let dir = tempfile::tempdir().unwrap();
    write_two_state_spec(&dir.path().join("two.md"), "two");
    let mut cmd = spk();
    cmd.args(["graph", dir.path().to_str().unwrap(), "--format", "edges"]);
    let first = cmd.output().unwrap();
    let second = cmd.output().unwrap();
    assert_eq!(first.stdout, second.stdout, "re-runs are byte-identical");
}

/// Characterization (add-graph-views task 1.4): pins the projection's
/// exact bytes for the two-state fixture — sort order (kind column sorts
/// `Constraint` rows before `Transition` rows, and `two\t` before
/// `two.t\t` since TAB < '.'), the six-column shape, and the empty
/// trailing annotation on every edge row. Guards the task 1.4 extraction
/// of the formatting machinery into a reusable unit: any drift in the
/// bytes fails here, not downstream in dot/mermaid consumers.
#[test]
fn edges_projection_pins_exact_tsv_bytes() {
    let dir = tempfile::tempdir().unwrap();
    write_two_state_spec(&dir.path().join("two.md"), "two");
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--format", "edges"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let expected = "\
two.c1\tConstraint\tconstraints.traces_to\ttwo\tIntent\t\n\
two.c1\tConstraint\tconstraints.traces_to\ttwo\tIntent\t\n\
two.t\tTransition\ttransitions.from\ttwo.s1\tState\t\n\
two.t\tTransition\ttransitions.guard\ttwo.c1\tConstraint\t\n\
two.t\tTransition\ttransitions.to\ttwo.s2\tState\t\n";
    assert_eq!(
        std::str::from_utf8(&out.stdout).unwrap(),
        expected,
        "edge projection bytes are pinned"
    );
}

#[test]
fn edges_projection_emits_canonical_ids_only() {
    // The self-anchored corpus makes extraction label the edge
    // `id (intent)`; the projection must emit the bare intent id (D2).
    let dir = tempfile::tempdir().unwrap();
    write_self_anchored_spec(&dir.path().join("self.md"), "self");
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--format", "edges"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = std::str::from_utf8(&out.stdout).unwrap();
    assert!(
        !text.contains(" ("),
        "no label-qualified endpoints like `{0} (intent)`: {text}",
        "self (intent)"
    );
    let rows = edges_rows(&out.stdout);
    assert!(
        rows.iter()
            .any(|r| r[0] == "self" && r[3] == "self" && r[2] == "constraints.traces_to"),
        "the intent-anchored edge carries canonical endpoints: {rows:?}"
    );
}

#[test]
fn edges_projection_zero_file_directory_exits_zero_with_empty_output() {
    let dir = tempfile::tempdir().unwrap();
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--format", "edges"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "zero files exit 0");
    assert!(
        out.stdout.is_empty(),
        "zero files emit an empty TSV: {:?}",
        std::str::from_utf8(&out.stdout)
    );
}

#[test]
fn edges_projection_single_intent_corpus_well_formed_rows() {
    let dir = tempfile::tempdir().unwrap();
    write_self_anchored_spec(&dir.path().join("one.md"), "one");
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--format", "edges"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let rows = edges_rows(&out.stdout);
    assert_eq!(
        rows,
        vec![vec![
            "one",
            "Intent",
            "constraints.traces_to",
            "one",
            "Intent",
            ""
        ]],
        "single-intent corpus yields exactly its one well-formed edge row"
    );
}

#[test]
fn violations_survive_projection() {
    let dir = tempfile::tempdir().unwrap();
    write_violation_spec(&dir.path().join("viol.md"), "viol");
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--format", "edges"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let rows = edges_rows(&out.stdout);
    let annotations: Vec<&Vec<&str>> = rows
        .iter()
        .filter(|r| r[2].starts_with("violation:"))
        .collect();
    assert_eq!(
        annotations.len(),
        1,
        "one annotation row per violation: {rows:?}"
    );
    let a = annotations[0];
    assert_eq!(a[0], "", "annotation rows carry an empty source id");
    assert_eq!(a[1], "", "annotation rows carry an empty source kind");
    assert_eq!(
        a[2], "violation:constraints.traces_to",
        "field names the violated edge kind"
    );
    assert_eq!(
        a[3], "viol.a",
        "annotation rows keep the canonical target id"
    );
    assert!(
        !a[5].is_empty(),
        "the annotation column carries the finding"
    );
}

#[test]
fn clean_corpus_no_annotations() {
    let dir = tempfile::tempdir().unwrap();
    write_two_state_spec(&dir.path().join("clean.md"), "clean");
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--format", "edges"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = std::str::from_utf8(&out.stdout).unwrap();
    assert!(
        !text.contains("violation:"),
        "a clean corpus emits zero annotation rows: {text}"
    );
}

#[test]
fn edges_retain_state_transition_edges_and_multiplicity() {
    let dir = tempfile::tempdir().unwrap();
    write_two_state_spec(&dir.path().join("two.md"), "two");
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--format", "edges"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let rows = edges_rows(&out.stdout);
    assert!(
        rows.iter()
            .any(|r| r[0] == "two.t" && r[2] == "transitions.from" && r[3] == "two.s1"),
        "the from edge keeps both distinct qualified ids: {rows:?}"
    );
    assert!(
        rows.iter()
            .any(|r| r[0] == "two.t" && r[2] == "transitions.to" && r[3] == "two.s2"),
        "the to edge keeps both distinct qualified ids: {rows:?}"
    );
    let dup = rows
        .iter()
        .filter(|r| r[0] == "two.c1" && r[2] == "constraints.traces_to" && r[3] == "two")
        .count();
    assert_eq!(
        dup, 2,
        "duplicate edge instances survive (multiplicity, D2): {rows:?}"
    );
}
