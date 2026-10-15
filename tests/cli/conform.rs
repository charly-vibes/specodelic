// CLI wiring tests for `spk conform` (openspec/changes/add-conform
// phase 4, tasks 4.1/4.2): the report envelope over the standard
// conventions, the recorded `--closed-world` declaration, the
// open-world-never-forbidden property asserted at CLI level, the
// exit-code contract (0 = no forbidden/unsupported; 1 = any; 2 =
// invocation error / gate refusal with zero verdict records), and the
// read-only help contract.
use super::*;

/// A lint-clean, compilable single-file fixture (rust-fragment invariant
/// so the covering claim is executable — mirrors the phase-3 gate
/// fixture's shape with a distinct id).
const SPEC_TEXT: &str = r#"---
id: demo.cli_conform
kind: intent
statement: "THE system SHALL conform to the recorded oracle traces"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.cli_conform]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.cli_conform.locked]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[demo.cli_conform.locked]] | `g()` | `x` |
"#;

/// The same spec with edited structured content (a second transition) —
/// artifacts compiled before this edit are stale for the gate.
const SPEC_TEXT_EDITED: &str = r#"---
id: demo.cli_conform
kind: intent
statement: "THE system SHALL conform to the recorded oracle traces"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.cli_conform]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.cli_conform.locked]] |
| reopen | confirmed | held | [[demo.cli_conform.locked]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[demo.cli_conform.locked]] | `g()` | `x` |
"#;

const PERMITTED_LINE: &str = r#"{"id": "confirm-happy", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"status": "confirmed"}}]}"#;
/// `undo` is no declared transition — uncovered (underspecified
/// open-world, forbidden only under `--closed-world`).
const UNCOVERED_LINE: &str =
    r#"{"id": "undo-undeclared", "setup": {"state": "held"}, "trace": [{"action": "undo"}]}"#;
/// A declared transition exercised from a state it is not declared
/// from — positive contradiction, forbidden in ANY invocation mode.
const CONTRADICTION_LINE: &str = r#"{"id": "confirm-from-wrong-state", "setup": {"state": "confirmed"}, "trace": [{"action": "confirm"}]}"#;

/// Compile the fixture spec into a temp out-dir (the gate requires
/// current artifacts — conform never re-compiles).
fn compiled_out_dir(dir: &std::path::Path, spec: &std::path::Path) -> String {
    let out_dir = dir.join("out");
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
    out_dir.to_str().unwrap().to_string()
}

fn conform_json(
    spec: &std::path::Path,
    corpus: &std::path::Path,
    extra: &[&str],
) -> (Option<i32>, serde_json::Value) {
    let dir = spec.parent().unwrap().to_path_buf();
    let out_dir = compiled_out_dir(dir.as_path(), spec);
    let out = spk()
        .args([
            "conform",
            spec.to_str().unwrap(),
            "--oracle",
            corpus.to_str().unwrap(),
            "--out-dir",
            &out_dir,
            "--json",
        ])
        .args(extra)
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    (out.status.code(), json)
}

// ---- task 4.1: report envelope, recorded declaration, open-world ----

#[test]
fn conform_emits_report_envelope_open_world_never_forbidden() {
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("demo.cli_conform.md");
    std::fs::write(&spec, SPEC_TEXT).unwrap();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{PERMITTED_LINE}\n{UNCOVERED_LINE}\n")).unwrap();

    let (code, json) = conform_json(&spec, &corpus, &[]);
    // Coverage gaps never fail: permitted + underspecified → exit 0.
    assert_eq!(code, Some(0), "open-world gaps are non-failing: {json}");
    assert_eq!(json["ok"], serde_json::json!(true), "{json}");
    let data = &json["data"];
    // The conform-local report schema rides the envelope's data.
    assert_eq!(data["report_schema_version"], 1);
    assert_eq!(data["closed_world"], serde_json::json!(false));
    assert!(
        data["evidence_scope"]
            .as_str()
            .is_some_and(|s| !s.is_empty()),
        "evidence_scope present in the JSON view: {json}"
    );
    // Records sorted by scenario id; the uncovered trace is
    // underspecified — never forbidden without the declaration
    // (open_world_never_forbidden at CLI level).
    let records = data["records"].as_array().unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["scenario_id"], "confirm-happy");
    assert_eq!(records[0]["verdict"], "permitted");
    assert_eq!(records[1]["scenario_id"], "undo-undeclared");
    assert_eq!(records[1]["verdict"], "underspecified");
    for record in records {
        assert_ne!(
            record["verdict"], "forbidden",
            "open-world runs never produce forbidden: {json}"
        );
        assert_eq!(record["closed_world"], serde_json::json!(false));
    }
    // Counts surfaced: the coverage gap is visible, not hidden.
    assert_eq!(data["verdict_counts"]["permitted"], 1);
    assert_eq!(data["verdict_counts"]["underspecified"], 1);
}

#[test]
fn conform_closed_world_recorded_and_uncovered_forbidden() {
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("demo.cli_conform.md");
    std::fs::write(&spec, SPEC_TEXT).unwrap();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{UNCOVERED_LINE}\n")).unwrap();

    let (code, json) = conform_json(&spec, &corpus, &["--closed-world"]);
    // Uncovered + declared closed-world → forbidden → exit 1.
    assert_eq!(
        code,
        Some(1),
        "closed-world forbidden fails the run: {json}"
    );
    assert_eq!(json["ok"], serde_json::json!(false));
    let data = &json["data"];
    // The declaration is recorded in the header AND every record (D2).
    assert_eq!(data["closed_world"], serde_json::json!(true));
    let records = data["records"].as_array().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["verdict"], "forbidden");
    assert_eq!(records[0]["closed_world"], serde_json::json!(true));
    assert_eq!(data["verdict_counts"]["forbidden"], 1);
}

#[test]
fn conform_contradiction_forbidden_in_any_mode() {
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("demo.cli_conform.md");
    std::fs::write(&spec, SPEC_TEXT).unwrap();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{CONTRADICTION_LINE}\n")).unwrap();

    // NO --closed-world: a positive contradiction is still forbidden.
    let (code, json) = conform_json(&spec, &corpus, &[]);
    assert_eq!(code, Some(1), "contradiction fails any mode: {json}");
    let records = json["data"]["records"].as_array().unwrap();
    assert_eq!(records[0]["verdict"], "forbidden");
    assert_eq!(records[0]["closed_world"], serde_json::json!(false));
    // The reason names the contradicted executable claim id.
    assert!(
        records[0]["reason"].as_str().unwrap().contains("locked"),
        "reason names the contradicted claim: {json}"
    );
}

// ---- task 4.2: exit-code contract — exit 2 paths, zero records ----

#[test]
fn conform_invocation_error_exits_2_without_records() {
    let dir = tempfile::tempdir().unwrap();
    // Missing spec path.
    let out = spk()
        .args([
            "conform",
            dir.path().join("nope.md").to_str().unwrap(),
            "--oracle",
            dir.path().join("scenarios.jsonl").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(2),
        "missing spec is an invocation error"
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["ok"], serde_json::json!(false));
    assert!(
        json["data"]["records"].is_null(),
        "zero verdict records on refusal: {json}"
    );
    // Missing oracle path too.
    let spec = dir.path().join("demo.cli_conform.md");
    std::fs::write(&spec, SPEC_TEXT).unwrap();
    let out_dir = compiled_out_dir(dir.path(), &spec);
    let out = spk()
        .args([
            "conform",
            spec.to_str().unwrap(),
            "--oracle",
            dir.path().join("absent.jsonl").to_str().unwrap(),
            "--out-dir",
            &out_dir,
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(2),
        "missing oracle is an invocation error"
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["ok"], serde_json::json!(false));
    assert!(json["data"]["records"].is_null(), "zero records: {json}");
}

#[test]
fn conform_gate_refusal_exits_2_without_records() {
    let dir = tempfile::tempdir_in(std::env::temp_dir()).unwrap();
    let spec = dir.path().join("demo.cli_conform.md");
    std::fs::write(&spec, SPEC_TEXT).unwrap();
    let _out_dir = compiled_out_dir(dir.path(), &spec);
    // Edit the structured content WITHOUT recompiling → stale artifacts.
    std::fs::write(&spec, SPEC_TEXT_EDITED).unwrap();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{PERMITTED_LINE}\n")).unwrap();

    let out = spk()
        .args([
            "conform",
            spec.to_str().unwrap(),
            "--oracle",
            corpus.to_str().unwrap(),
            "--out-dir",
            dir.path().join("out").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "gate refusal stays exit 2");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["ok"], serde_json::json!(false), "{json}");
    assert!(
        json["data"]["records"].is_null(),
        "no verdict records from a gated-out invocation: {json}"
    );
    // The refusal names the failing stage's remediation (spk compile).
    let rendered = serde_json::to_string(&json).unwrap();
    assert!(
        rendered.contains("spk compile"),
        "refusal names the compile remediation: {json}"
    );
}

#[test]
fn conform_empty_corpus_yields_valid_empty_report_exit_0() {
    // Delta 'Empty corpus yields a valid empty report' at CLI level: a
    // zero-line corpus is a valid Ok report — zero records,
    // evidence_scope intact, exit 0 — never an error.
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("demo.cli_conform.md");
    std::fs::write(&spec, SPEC_TEXT).unwrap();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, "\n").unwrap();

    let (code, json) = conform_json(&spec, &corpus, &[]);
    assert_eq!(code, Some(0), "zero traces never fail: {json}");
    assert_eq!(json["ok"], serde_json::json!(true));
    let data = &json["data"];
    assert_eq!(data["report_schema_version"], 1);
    assert_eq!(data["records"], serde_json::json!([]));
    assert_eq!(data["verdict_counts"], serde_json::json!({}));
    assert!(
        data["evidence_scope"]
            .as_str()
            .is_some_and(|s| !s.is_empty()),
        "evidence_scope intact on the empty report: {json}"
    );
}

// ---- specodelic-1v30: gate corpus scope + structural-only mode ----

/// The conformed target of the sibling corpus: a property derives from
/// the SIBLING file's constraint row — a corpus-wide `[[ref]]` that a
/// single-spec lint slice would dangle (gh-17 friction 1).
const SIBLING_TARGET_TEXT: &str = r#"---
id: demo.cli_one
kind: intent
statement: "THE system SHALL conform to the recorded oracle traces"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.cli_one]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.cli_one.locked]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[demo.cli_one.locked]] | `g()` | `x` |
| p2 | unit | [[demo.cli_two.locked]] | `g()` | `y` |
"#;

/// The same target with its own REAL lint finding: a dangling ref to a
/// row that exists nowhere in the corpus — the gate must still refuse
/// (no false green from the corpus-widened scope).
fn sibling_target_dangling_text() -> String {
    SIBLING_TARGET_TEXT.replace(
        "| p2 | unit | [[demo.cli_two.locked]] | `g()` | `y` |",
        "| p2 | unit | [[demo.cli_absent.gone]] | `g()` | `y` |",
    )
}

/// The sibling spec declaring the referenced row — lint-clean on its own.
const SIBLING_OTHER_TEXT: &str = r#"---
id: demo.cli_two
kind: intent
statement: "THE sibling SHALL define the shared contract row"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.cli_two]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.cli_two.locked]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[demo.cli_two.locked]] | `g()` | `x` |
"#;

/// A prose-guarded spec: the covering claim's expr is prose, so a
/// conforming trace is `unknown` by default (closed-world honesty) and
/// `agree` only under the explicit `--structural-only` mode.
const PROSE_SPEC_TEXT: &str = r#"---
id: demo.cli_prose
kind: intent
statement: "THE system SHALL conform to the recorded oracle traces"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| healthy | invariant | the service stays healthy | [[demo.cli_prose]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.cli_prose.healthy]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[demo.cli_prose.healthy]] | `g()` | `x` |
"#;

/// A sibling file that FAILS to parse (no statement) — the corpus lint
/// then runs over a partial corpus, which must be surfaced as a labeled
/// warning, never a silent partial-corpus verdict (invariant 3).
const SIBLING_BROKEN_TEXT: &str = "---\nid: demo.cli_two\nkind: intent\n---\n";

/// Write the openspec corpus layout under `dir`: the target at
/// `openspec/specs/demo.cli_one/spec.md`, the sibling at
/// `openspec/specs/demo.cli_two/spec.md`.
fn write_corpus(dir: &std::path::Path, target_text: &str, sibling_text: &str) {
    let one = dir.join("openspec/specs/demo.cli_one");
    let two = dir.join("openspec/specs/demo.cli_two");
    std::fs::create_dir_all(&one).unwrap();
    std::fs::create_dir_all(&two).unwrap();
    std::fs::write(one.join("spec.md"), target_text).unwrap();
    std::fs::write(two.join("spec.md"), sibling_text).unwrap();
}

/// Run conform with cwd = `dir` (the corpus root's parent), compiling
/// `compile_path` into `out` first when given — compile's own gate is
/// single-file, so a corpus fixture must compile the CORPUS root — and
/// exercising cwd-anchored discovery like a real invocation.
fn conform_json_in(
    dir: &std::path::Path,
    spec_path: &str,
    corpus: &std::path::Path,
    compile_path: Option<&str>,
    extra: &[&str],
) -> (Option<i32>, serde_json::Value, String) {
    if let Some(compile_path) = compile_path {
        let out = spk()
            .current_dir(dir)
            .args([
                "compile",
                compile_path,
                "--json",
                "--out-dir",
                dir.join("out").to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "fixture target must compile: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let out = spk()
        .current_dir(dir)
        .args([
            "conform",
            spec_path,
            "--oracle",
            corpus.to_str().unwrap(),
            "--out-dir",
            dir.join("out").to_str().unwrap(),
            "--json",
        ])
        .args(extra)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    (out.status.code(), json, stdout)
}

/// The meter's green (gh-17 friction 1): a `[[ref]]` target living in a
/// SIBLING spec no longer dangles conform's input gate — the gate lints
/// the corpus the file belongs to, and the run exits 0.
#[test]
fn conform_sibling_ref_corpus_gate_green() {
    let dir = tempfile::tempdir().unwrap();
    write_corpus(dir.path(), SIBLING_TARGET_TEXT, SIBLING_OTHER_TEXT);
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{PERMITTED_LINE}\n")).unwrap();

    let (code, json, _stdout) = conform_json_in(
        dir.path(),
        "openspec/specs/demo.cli_one/spec.md",
        &corpus,
        Some("openspec"),
        &[],
    );
    assert_eq!(
        code,
        Some(0),
        "corpus-resolved sibling ref must not dangle the gate: {json}"
    );
    assert_eq!(json["ok"], serde_json::json!(true));
    assert_eq!(json["data"]["records"][0]["verdict"], "permitted");
}

/// No false green: the TARGET's own lint findings still refuse with the
/// same artifact_gate shape — even when the rest of the corpus is clean.
#[test]
fn conform_target_own_findings_still_refused() {
    let dir = tempfile::tempdir().unwrap();
    write_corpus(
        dir.path(),
        &sibling_target_dangling_text(),
        SIBLING_OTHER_TEXT,
    );
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{PERMITTED_LINE}\n")).unwrap();

    // No compile: the lint gate refuses before the artifact check (the
    // same stage order orchestrate applies: parse → lint → compile).
    let (code, json, _stdout) = conform_json_in(
        dir.path(),
        "openspec/specs/demo.cli_one/spec.md",
        &corpus,
        None,
        &[],
    );
    assert_eq!(code, Some(2), "own dangling ref refuses: {json}");
    assert_eq!(json["ok"], serde_json::json!(false));
    assert!(json["data"]["records"].is_null(), "zero records: {json}");
    let rendered = serde_json::to_string(&json).unwrap();
    assert!(rendered.contains("artifact_gate"), "{json}");
    assert!(rendered.contains("lint findings"), "{json}");
    assert!(rendered.contains("spk lint"), "{json}");
}

/// A SIBLING file that fails to PARSE makes the corpus lint partial —
/// surfaced as a labeled warning on the existing warnings channel, never
/// a silent partial-corpus verdict (invariant 3). The target itself is
/// clean and refs nothing cross-file, so the run still succeeds.
#[test]
fn conform_partial_corpus_surfaces_labeled_warning() {
    let dir = tempfile::tempdir().unwrap();
    write_corpus(dir.path(), SIBLING_TARGET_TEXT, SIBLING_BROKEN_TEXT);
    // Make the target corpus-self-contained: drop the cross-file ref so
    // the partial corpus is the ONLY defect in play.
    let target = SIBLING_TARGET_TEXT.replace(
        "| p2 | unit | [[demo.cli_two.locked]] | `g()` | `y` |",
        "| p2 | unit | [[demo.cli_one.locked]] | `g()` | `y` |",
    );
    std::fs::write(
        dir.path().join("openspec/specs/demo.cli_one/spec.md"),
        target,
    )
    .unwrap();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{PERMITTED_LINE}\n")).unwrap();

    let (code, json, stdout) = conform_json_in(
        dir.path(),
        "openspec/specs/demo.cli_one/spec.md",
        &corpus,
        Some("openspec"),
        &[],
    );
    assert_eq!(code, Some(0), "clean target still conforms: {json}");
    let rendered = format!("{stdout}{json}");
    assert!(
        rendered.to_lowercase().contains("partial corpus"),
        "partial-corpus lint must be a labeled warning, never silent: {rendered}"
    );
}

/// Friction 2, option (a): `--structural-only` flips the prose-guard
/// `unknown` to `agree` — the trace conforms to the declared transition
/// relation — while the reason STILL names the uninterpretable guard
/// (the record never claims the prose invariant was checked).
#[test]
fn conform_structural_only_agrees_on_prose_guard() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("demo.cli_prose.md"), PROSE_SPEC_TEXT).unwrap();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{PERMITTED_LINE}\n")).unwrap();

    let (code, json, _stdout) = conform_json_in(
        dir.path(),
        "demo.cli_prose.md",
        &corpus,
        Some("demo.cli_prose.md"),
        &["--structural-only"],
    );
    assert_eq!(code, Some(0), "agree never fails the run: {json}");
    let records = json["data"]["records"].as_array().unwrap();
    assert_eq!(records[0]["verdict"], "agree");
    let reason = records[0]["reason"].as_str().unwrap();
    assert!(
        reason.contains("healthy"),
        "the reason still names the uninterpretable guard: {reason}"
    );
    assert!(
        reason.contains("--structural-only"),
        "the reason states the mode's effect: {reason}"
    );
    assert!(
        reason.contains("never checked"),
        "the record never claims the prose invariant was checked: {reason}"
    );
}

/// The DEFAULT verdict space is unchanged: without the flag, the same
/// prose-guarded trace is still `unknown` (closed-world honesty).
#[test]
fn conform_default_mode_still_unknown_on_prose_guard() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("demo.cli_prose.md"), PROSE_SPEC_TEXT).unwrap();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{PERMITTED_LINE}\n")).unwrap();

    let (code, json, _stdout) = conform_json_in(
        dir.path(),
        "demo.cli_prose.md",
        &corpus,
        Some("demo.cli_prose.md"),
        &[],
    );
    assert_eq!(code, Some(0), "unknown never fails the run: {json}");
    let records = json["data"]["records"].as_array().unwrap();
    assert_eq!(records[0]["verdict"], "unknown");
    assert!(
        records[0]["reason"]
            .as_str()
            .unwrap()
            .contains("prose_claim_not_interpretable"),
        "{json}"
    );
}

/// The isolated-file fallback: with no corpus root discoverable, the
/// gate keeps today's single-spec slice — a cross-file ref dangles and
/// the gate refuses, the labeled documented behavior for an isolated
/// file (never an error).
#[test]
fn conform_isolated_file_fallback_unchanged() {
    // A bare temp dir with NO openspec ancestor: discovery falls back.
    let dir = tempfile::tempdir_in(std::env::temp_dir()).unwrap();
    std::fs::write(
        dir.path().join("demo.cli_one.md"),
        sibling_target_dangling_text(),
    )
    .unwrap();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{PERMITTED_LINE}\n")).unwrap();

    let (code, json, _stdout) = conform_json_in(dir.path(), "demo.cli_one.md", &corpus, None, &[]);
    assert_eq!(code, Some(2), "isolated dangling ref refuses: {json}");
    let rendered = serde_json::to_string(&json).unwrap();
    assert!(rendered.contains("lint findings"), "{json}");
}

// ---- task 4.2: the help contract — READ-ONLY, stated up front ----

#[test]
fn conform_help_states_read_only_evaluation() {
    spk()
        .args(["conform", "--help"])
        .assert()
        .success()
        .stdout(contains("READ-ONLY"));
}

#[test]
fn conform_help_declares_structural_only() {
    spk()
        .args(["conform", "--help"])
        .assert()
        .success()
        .stdout(contains("--structural-only"));
}
