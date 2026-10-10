//! Phase-5 lifecycle-independence and dogfood tests for `conform`
//! (openspec/changes/add-conform, tasks 5.1–5.3): conform is an
//! evaluation verb OUTSIDE the artifact lifecycle (delta
//! `lifecycle_outside`) — `spk orchestrate` over a spec with conform
//! available never invokes conform and the pipeline stages are
//! unchanged (`outside_lifecycle`); a conform run over a
//! `model_checked` artifact leaves the stage unchanged; and the
//! read-only dogfood property (`consumes_never_writes`): conform run
//! against a FIXTURE COPY of a real `specs/` corpus file leaves the
//! real corpus, the fixture, and the git worktree byte-identical.
//! Phase 5 expects NO source changes — these are conformance
//! constraints, not behavior to implement (tasks 5.2).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use assert_cmd::Command;

use specodelic::conform::{ConformReport, run};
use specodelic::spec;
use specodelic::{compile, orchestrate};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// A lint-clean, compilable model_check-shaped fixture (Properties
/// present so the model_check stage runs — the same shape orchestrate's
/// own unit tests use), with a rust-fragment executable claim so conform
/// has something to classify against.
const SPEC_TEXT: &str = r#"---
id: demo.lifecycle
kind: intent
statement: "THE system SHALL conform to the recorded oracle traces"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.lifecycle]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.lifecycle.locked]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[demo.lifecycle.locked]] | `g()` | `x` |
"#;

/// A corpus with a permitted trace AND a positively-contradicting one —
/// the conform run therefore produces a `forbidden` verdict (the exit-1
/// class), proving the lifecycle test exercises conform's real failure
/// path: even a forbidden-verdict run must not gate or block anything.
const CORPUS: &str = concat!(
    r#"{"id": "confirm-happy", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"status": "confirmed"}}]}"#,
    "\n",
    r#"{"id": "confirm-from-wrong-state", "setup": {"state": "confirmed"}, "trace": [{"action": "confirm"}]}"#,
    "\n",
);

/// Parse the fixture with its path set (artifact stem derives from it).
fn fixture_spec(dir: &Path) -> spec::Spec {
    let file = dir.join("demo.lifecycle.md");
    std::fs::write(&file, SPEC_TEXT).expect("fixture spec writes");
    let mut spec = spec::parse_str(SPEC_TEXT).expect("fixture spec parses");
    spec.path = Some(file.clone());
    spec
}

/// Compile the fixture's artifacts into `out_dir` (conform never
/// compiles — the gate requires current artifacts).
fn compile_fixture(spec: &spec::Spec, out_dir: &Path) {
    let compiled = compile::compile_spec(spec).expect("fixture spec compiles");
    compile::write_artifacts(spec, &compiled, out_dir.to_str().expect("utf8 out-dir"))
        .expect("fixture artifacts write");
}

/// Byte-exact snapshot of every file under `dir` (recursive).
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut map = BTreeMap::new();
    fn walk(dir: &Path, base: &Path, map: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).expect("dir reads") {
            let entry = entry.expect("dir entries");
            let path = entry.path();
            if path.is_dir() {
                walk(&path, base, map);
            } else {
                let rel = path.strip_prefix(base).expect("relative path");
                map.insert(rel.to_path_buf(), std::fs::read(&path).expect("file reads"));
            }
        }
    }
    walk(dir, dir, &mut map);
    map
}

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap()
}

// ---------------------------------------------------------------------------
// 5.1/5.2 — outside_lifecycle (delta `lifecycle_outside`)
// ---------------------------------------------------------------------------

/// `spk orchestrate` over a spec with conform available never invokes
/// conform: the stage list is EXACTLY the pipeline's five stages (parse,
/// lint, compile, model_check, verify) — no conform stage exists, and no
/// serialized orchestration detail mentions conform anywhere. Constraint
/// is conformance, not behavior: no orchestrate code change is expected.
#[test]
fn outside_lifecycle_orchestrate_never_invokes_conform() {
    let dir = tempfile::tempdir_in(std::env::temp_dir()).expect("tempdir");
    let spec = fixture_spec(dir.path());

    // Library path — the same machinery the CLI wraps.
    let o = orchestrate::orchestrate(
        std::slice::from_ref(&spec),
        &[],
        &orchestrate::ParseInput::default(),
        dir.path().join("out").to_str().expect("utf8 out-dir"),
        &specodelic::model_check::Bound::default(),
        &orchestrate::Backends::default(),
    );
    let names: Vec<&str> = o.stages.iter().map(|s| s.stage).collect();
    assert_eq!(
        names,
        vec!["parse", "lint", "compile", "model_check", "verify"],
        "pipeline stages are exactly the lifecycle stages — conform is not one"
    );
    let serialized = serde_json::to_string(&o).expect("orchestration serializes");
    assert!(
        !serialized.contains("conform"),
        "no orchestration stage or detail may reference conform: {serialized}"
    );

    // CLI path — the envelope a real `spk orchestrate` consumer sees.
    let out = spk()
        .args([
            "orchestrate",
            dir.path().to_str().expect("utf8 dir"),
            "--json",
            "--out-dir",
            dir.path().join("cli-out").to_str().expect("utf8 cli-out"),
        ])
        .output()
        .expect("orchestrate runs");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("envelope parses");
    let cli_stages: Vec<&str> = json["data"]["stages"]
        .as_array()
        .expect("stages array")
        .iter()
        .map(|s| s["stage"].as_str().expect("stage name"))
        .collect();
    assert_eq!(
        cli_stages,
        vec!["parse", "lint", "compile", "model_check", "verify"],
        "CLI stage list unchanged by conform's existence: {cli_stages:?}"
    );
    assert!(
        !stdout.contains("conform"),
        "the orchestrate envelope must not mention conform anywhere: {stdout}"
    );
}

/// A conform run over a `model_checked` artifact leaves the stage
/// unchanged: after a full orchestrate pass (compile + model_check —
/// the artifact is model_checked), a conform run that produces a
/// `forbidden` verdict (exit-1 class) writes NOTHING and neither
/// advances nor blocks the lifecycle — a re-run of orchestrate reports
/// exactly the same stages with the same statuses.
#[test]
fn conform_run_leaves_stage_unchanged() {
    let dir = tempfile::tempdir_in(std::env::temp_dir()).expect("tempdir");
    let spec = fixture_spec(dir.path());
    let out_dir = dir.path().join("out");
    compile_fixture(&spec, &out_dir);

    // Full pipeline pass — the artifact is now model_checked.
    let orchestrate_inputs = (
        orchestrate::ParseInput::default(),
        specodelic::model_check::Bound::default(),
        orchestrate::Backends::default(),
    );
    let before = orchestrate::orchestrate(
        std::slice::from_ref(&spec),
        &[],
        &orchestrate_inputs.0,
        out_dir.to_str().expect("utf8 out-dir"),
        &orchestrate_inputs.1,
        &orchestrate_inputs.2,
    );
    let stage_vec: Vec<(&str, String)> = before
        .stages
        .iter()
        .map(|s| (s.stage, s.status.clone()))
        .collect();

    // The artifact tree snapshot BEFORE the conform run.
    let artifacts_before_conform = snapshot(&out_dir);
    let spec_bytes_before =
        std::fs::read(spec.path.as_ref().expect("fixture path")).expect("spec reads");

    // Conform runs over the model_checked artifact — with a forbidden
    // verdict (the class that maps to exit 1) to prove even a failing
    // evaluation cannot gate or block the lifecycle.
    let report: ConformReport =
        run(&spec, &out_dir, CORPUS.as_bytes(), false).expect("conform runs");
    assert!(
        report
            .records
            .iter()
            .any(|r| r.verdict.as_str() == "forbidden"),
        "the contradiction corpus yields a forbidden verdict: {:?}",
        report.verdict_counts
    );

    // Nothing was written: the artifact tree AND the spec file are
    // byte-identical after the run.
    assert_eq!(
        snapshot(&out_dir),
        artifacts_before_conform,
        "conform leaves the artifact tree byte-identical"
    );
    let spec_bytes_after =
        std::fs::read(spec.path.as_ref().expect("fixture path")).expect("spec reads");
    assert_eq!(
        spec_bytes_before, spec_bytes_after,
        "conform never writes the spec file"
    );

    // The lifecycle is neither advanced nor blocked: a re-run of
    // orchestrate reports exactly the same stages and statuses.
    let after = orchestrate::orchestrate(
        std::slice::from_ref(&spec),
        &[],
        &orchestrate_inputs.0,
        out_dir.to_str().expect("utf8 out-dir"),
        &orchestrate_inputs.1,
        &orchestrate_inputs.2,
    );
    let stage_vec_after: Vec<(&str, String)> = after
        .stages
        .iter()
        .map(|s| (s.stage, s.status.clone()))
        .collect();
    assert_eq!(
        stage_vec, stage_vec_after,
        "conform never gates or blocks a lifecycle transition"
    );
}

// ---------------------------------------------------------------------------
// 5.3 — consumes_never_writes (dogfood, delta `read_only_evaluation`)
// ---------------------------------------------------------------------------

/// Read-only dogfood: conform run against a FIXTURE COPY of a real
/// `specs/` corpus file (`specs/errors.md` — the only real corpus file
/// that lints clean standalone AND compiles single-file) and a fixture
/// oracle corpus. The REAL corpus directory and the git worktree must be
/// byte-identical after the run, and conform must write nothing at all —
/// not even into its own out-dir (the report is returned, never
/// persisted as an artifact).
#[test]
fn consumes_never_writes() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let real_corpus = manifest.join("specs");

    // BEFORE: snapshot the real corpus (the REAL specs/ directory) and
    // the git worktree state.
    let corpus_before = snapshot(&real_corpus);
    let worktree_before = git_status_porcelain(&manifest);

    // Fixture: a byte-copy of a real corpus file in a temp dir, compiled
    // into a temp out-dir (never the real corpus).
    let dir = tempfile::tempdir_in(std::env::temp_dir()).expect("tempdir");
    let original = std::fs::read(real_corpus.join("errors.md")).expect("real corpus reads");
    std::fs::write(dir.path().join("errors.md"), &original).expect("fixture copy writes");
    let mut spec =
        spec::parse_str(&String::from_utf8(original.clone()).expect("real corpus is utf8"))
            .expect("real corpus file parses");
    spec.path = Some(dir.path().join("errors.md"));
    let out_dir = dir.path().join("out");
    compile_fixture(&spec, &out_dir);

    // The fixture oracle corpus — traces over the copied spec's Model.
    let corpus = concat!(
        r#"{"id": "publish-happy", "setup": {"state": "draft"}, "trace": [{"action": "publish", "observations": {"state": "published"}}]}"#,
        "\n",
        r#"{"id": "undeclared-retract", "setup": {"state": "draft"}, "trace": [{"action": "retract"}]}"#,
        "\n",
    );

    // The whole fixture tree before the conform run.
    let fixture_before = snapshot(dir.path());

    let report = run(&spec, &out_dir, corpus.as_bytes(), false)
        .expect("dogfood conform runs over the fixture copy");
    assert_eq!(report.records.len(), 2, "every trace receives a verdict");
    assert!(!report.evidence_scope.is_empty());

    // AFTER: the real corpus, the git worktree, the fixture copy, and
    // even the fixture's own out-dir are byte-identical — conform wrote
    // NOTHING anywhere.
    assert_eq!(
        snapshot(&real_corpus),
        corpus_before,
        "the REAL specs/ corpus is byte-identical after the dogfood run"
    );
    assert_eq!(
        git_status_porcelain(&manifest),
        worktree_before,
        "the git worktree is unchanged after the dogfood run"
    );
    assert_eq!(
        snapshot(dir.path()),
        fixture_before,
        "conform wrote nothing — not even the report — into the fixture tree"
    );
}

/// `git status --porcelain` in the repo (worktree drift detector).
fn git_status_porcelain(manifest: &Path) -> String {
    let out = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(manifest)
        .output()
        .expect("git status runs");
    String::from_utf8(out.stdout).expect("git status is utf8")
}
