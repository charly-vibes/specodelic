// Purpose: CLI contract tests for `spk compile` (openspec/specs/compile/spec.md) —
// Responsibilities: pin the envelope/exit-code correspondence of the
// compile command: a labeled compile failure is an error-kind envelope
// with ok == false (openspec/specs/errors/spec.md envelope_error_kind +
// exit_code_mapping), never a success-shaped envelope riding exit 1.
// Rationale: specodelic-4v1 (F1) — failures ride success-shaped
// envelopes; see also tests/cli/model_check.rs for the model-check side.
use super::*;

/// A lint-clean spec with a Properties table whose predicate carries no
/// `**rust:` marker — the todo_predicate! placeholder path (gh-15).
fn write_props_spec(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL compile to proptest scaffolding\"\n---\n\
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
             | p | unit | [[{id}.c1]] | `g()` | `x` |\n"
        ),
    )
    .unwrap();
}

/// A lint-clean-shaped spec whose one kernel row uses a nonmember
/// kernel function — compile's precondition (lint) fails on it, a
/// labeled kernel-grammar failure.
fn write_kernel_grammar_broken_spec(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL carry a kernel claim\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | ka | invariant | `**kernel:** bogusfn(traces_to)` | [[{id}]] |\n\
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
             | p_ka | unit | [[{id}.ka]] | `word()` | `**rust:** v0.len() >= 1` |\n"
        ),
    )
    .unwrap();
}

#[test]
fn kernel_grammar_failure_is_an_error_envelope_with_exit_1() {
    // specodelic-4v1 (F1): exit 1 was already correct; the envelope lied
    // (ok:true / envelope_kind:ok with files_failed: 1). The
    // errors.md correspondence is bidirectional: exit 1 ⇔ ok:false.
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("kgf.md");
    write_kernel_grammar_broken_spec(&spec, "kgf");
    let result = spk()
        .args([
            "compile",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            td.path().join("out").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1), "labeled failure exits 1");
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(json["ok"], serde_json::json!(false), "envelope: {json}");
    assert_eq!(json["envelope_kind"], serde_json::json!("error"));
    // The payload survives the envelope flip — consumers still read the
    // labeled per-file failure.
    assert_eq!(json["data"]["files_failed"], serde_json::json!(1));
    assert_eq!(json["data"]["failed"][0]["stage"], "kernel_grammar");
    // remediation_hint_present: the hint channel is non-empty.
    assert!(
        json["hints"].as_array().is_some_and(|h| !h.is_empty()),
        "failure carries a remediation hint: {json}"
    );
}

#[test]
fn clean_compile_stays_a_success_envelope_with_exit_0() {
    // The flip must not overreach: a fully compiled batch keeps
    // ok:true / envelope_kind:ok / exit 0.
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("kgc.md");
    std::fs::write(
        &spec,
        "---\nid: kgc\nkind: intent\nstatement: \"THE kgc SHALL carry a kernel claim\"\n---\n\
          \n## Constraints\n\
          \n| id | kind | expr | traces_to |\n\
          |----|------|------|-----------|\n\
          | ka | invariant | `**kernel:** resolves(traces_to)` | [[kgc]] |\n\
          \n## Model\n\
          \n### States\n\
          \n- s1\n\
          - s2\n\
          \n### Transitions\n\
          \n| id | from | to | guard |\n\
          |----|------|----|-------|\n\
          | t | s1 | s2 | [[kgc.ka]] |\n\
          \n## Properties\n\
          \n| id | kind | derives_from | generator | predicate |\n\
          |----|------|--------------|-----------|------------|\n\
          | p_ka | unit | [[kgc.ka]] | `word()` | `**rust:** v0.len() >= 1` |\n",
    )
    .unwrap();
    let result = spk()
        .args([
            "compile",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            td.path().join("out").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(json["ok"], serde_json::json!(true));
    assert_eq!(json["envelope_kind"], serde_json::json!("ok"));
    assert_eq!(json["data"]["files_failed"], serde_json::json!(0));
}

#[test]
fn emitted_props_artifacts_are_rustfmt_clean() {
    // gh-15 (specodelic-w9ju): the generator's *_props.rs emission must be
    // rustfmt-clean by itself — recompiling after a spec amendment must not
    // churn ~14 files of pure formatting that trips downstream
    // `cargo fmt --check` gates. Emitted .rs artifacts pass
    // `rustfmt --check --edition 2021` as written; no post-pass needed.
    let td = tempfile::tempdir().unwrap();
    let spec = td.path().join("rfc.md");
    write_props_spec(&spec, "rfc");
    let out = td.path().join("out");
    let result = spk()
        .args([
            "compile",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(0), "compile must succeed");
    let props_path = std::fs::read_dir(&out)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "rs"))
        .expect("compile emits a *_props.rs artifact");

    // Byte-stable across a re-run: same input, byte-identical artifacts.
    let first = std::fs::read(&props_path).unwrap();
    let rerun = spk()
        .args([
            "compile",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(rerun.status.code(), Some(0));
    assert_eq!(
        first,
        std::fs::read(&props_path).unwrap(),
        "emission must be byte-stable across a re-run"
    );

    let fmt = std::process::Command::new("rustfmt")
        .args(["--check", "--edition", "2021", props_path.to_str().unwrap()])
        .output()
        .expect("rustfmt must be available to assert the fmt-clean contract");
    assert!(
        fmt.status.success(),
        "emitted {} is not rustfmt-clean:\n{}{}",
        props_path.display(),
        String::from_utf8_lossy(&fmt.stdout),
        String::from_utf8_lossy(&fmt.stderr)
    );
}
