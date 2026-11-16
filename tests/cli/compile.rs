// Purpose: CLI contract tests for `spk compile` (openspec/specs/compile/spec.md) —
// Responsibilities: pin the envelope/exit-code correspondence of the
// compile command: a labeled compile failure is an error-kind envelope
// with ok == false (openspec/specs/errors/spec.md envelope_error_kind +
// exit_code_mapping), never a success-shaped envelope riding exit 1.
// Rationale: specodelic-4v1 (F1) — failures ride success-shaped
// envelopes; see also tests/cli/model_check.rs for the model-check side.
// The corpus stem-scope refusal (specodelic-523w) is pinned here too:
// duplicate artifact stems are an up-front labeled invocation error, never
// a last-write-wins success.
use super::*;
use crate::common::UnwrapPanic;

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
    .unwrap_p();
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
    .unwrap_p();
}

/// Place one lint-clean compile fixture at `<root>/<subdir>/<file>` —
/// the corpus-shape builder the stem-scope tests below share.
fn write_corpus_spec(root: &std::path::Path, subdir: &str, file: &str, id: &str) {
    let dir = root.join(subdir);
    std::fs::create_dir_all(&dir).unwrap_p();
    write_props_spec(&dir.join(file), id);
}

#[test]
fn kernel_grammar_failure_is_an_error_envelope_with_exit_1() {
    // specodelic-4v1 (F1): exit 1 was already correct; the envelope lied
    // (ok:true / envelope_kind:ok with files_failed: 1). The
    // errors.md correspondence is bidirectional: exit 1 ⇔ ok:false.
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("kgf.md");
    write_kernel_grammar_broken_spec(&spec, "kgf");
    let result = spk()
        .args([
            "compile",
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            td.path().join("out").to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(result.status.code(), Some(1), "labeled failure exits 1");
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap_p();
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
    let td = tempfile::tempdir().unwrap_p();
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
    .unwrap_p();
    let result = spk()
        .args([
            "compile",
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            td.path().join("out").to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(result.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap_p();
    assert_eq!(json["ok"], serde_json::json!(true));
    assert_eq!(json["envelope_kind"], serde_json::json!("ok"));
    assert_eq!(json["data"]["files_failed"], serde_json::json!(0));
}

#[test]
fn duplicate_artifact_stem_is_a_labeled_scope_refusal_with_no_writes() {
    // specodelic-523w: a corpus whose files share one artifact stem used
    // to compile into one flat out-dir with the later file silently
    // overwriting the earlier one — a stem-collision warning rode the
    // warnings channel while the exit stayed 0. The scope guard now
    // refuses the collision up front — its own label,
    // `duplicate_artifact_stem`, naming both colliding files plus a
    // remediation hint, exit 2, and not one artifact on disk.
    //
    // This corpus is deliberately PRE-lint (the guard fires before lint
    // by design): ids `pipeline_alpha`/`pipeline_beta` do not satisfy
    // lint's `id_matches_file`, which pins an ordinary file's id to its
    // stem (`pipeline` here). Defense-in-depth — `compile::artifact_stem`
    // names artifacts from the identity a lint-clean file must already
    // carry: the filename stem for an ordinary file, the id-dash form
    // (`.`→`-`) for a `spec.md` whose id comes from its parent
    // directory. So a lint-clean corpus with a shared stem also shares
    // an intent id, the `duplicate_corpus_identity` class sibling
    // command paths refuse up front; the same-stem/distinct-id shape
    // pinned here is only reachable before lint, and the guard keeps the
    // refusal its own labeled one instead of an incidental lint finding
    // or a silent last-write-wins.
    let td = tempfile::tempdir().unwrap_p();
    let corpus = td.path().join("corpus");
    write_corpus_spec(&corpus, "alpha", "pipeline.md", "pipeline_alpha");
    write_corpus_spec(&corpus, "beta", "pipeline.md", "pipeline_beta");
    let out = td.path().join("out");
    let result = spk()
        .args([
            "compile",
            corpus.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(
        result.status.code(),
        Some(2),
        "a duplicate-stem corpus is an invocation error, never a last-write-wins success"
    );
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap_p();
    assert_eq!(
        json["ok"],
        serde_json::json!(false),
        "refusal envelope: {json}"
    );
    assert_eq!(json["envelope_kind"], serde_json::json!("error"));
    let message = json["warnings"]
        .as_array()
        .map(|ws| {
            ws.iter()
                .filter_map(|w| w["message"].as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    assert!(
        message.contains("duplicate_artifact_stem"),
        "the labeled scope failure names its kind: {json}"
    );
    assert!(
        message.matches("pipeline.md").count() >= 2,
        "the label names both colliding files: {json}"
    );
    assert!(
        json["hints"].as_array().is_some_and(|h| !h.is_empty()),
        "refusal carries a remediation hint: {json}"
    );
    // Never a silent partial: the refusal precedes every artifact write.
    assert!(
        !out.exists(),
        "no artifacts may be written before the refusal"
    );
}

#[test]
fn repeated_path_argument_is_not_a_stem_collision() {
    // `spk compile alpha.md alpha.md` collects one file twice — same path,
    // same bytes, same artifacts. That is not a collision: the guard skips
    // a repeat of the recorded path and the run stays exit 0, never a
    // misleading `duplicate_artifact_stem` refusal.
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("alpha.md");
    write_props_spec(&spec, "alpha");
    let out = td.path().join("out");
    let result = spk()
        .args([
            "compile",
            spec.to_str().unwrap_p(),
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(
        result.status.code(),
        Some(0),
        "a repeated path is one file, not a collision: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap_p();
    assert_eq!(json["ok"], serde_json::json!(true), "envelope: {json}");
    assert_eq!(json["envelope_kind"], serde_json::json!("ok"));
    assert_eq!(json["data"]["files_failed"], serde_json::json!(0));
    let warnings = json["warnings"]
        .as_array()
        .map(|ws| {
            ws.iter()
                .filter_map(|w| w["message"].as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    assert!(
        !warnings.contains("duplicate_artifact_stem"),
        "the repeated path must not trip the scope guard: {json}"
    );
    assert!(
        out.join("alpha_props.rs").is_file(),
        "the file still compiles and writes its artifacts: {json}"
    );
}

#[test]
fn refusal_keeps_hostile_input_notes_in_the_envelope() {
    // The refusal must not swallow the parse/hostile-input notes the
    // adjacent empty-corpus refusal re-attaches (suz Ro5, CORR-001): a
    // corpus carrying BOTH a stem collision and a skipped non-ingestable
    // file keeps the skip note in the error envelope's warnings.
    let td = tempfile::tempdir().unwrap_p();
    let corpus = td.path().join("corpus");
    write_corpus_spec(&corpus, "alpha", "pipeline.md", "pipeline_alpha");
    write_corpus_spec(&corpus, "beta", "pipeline.md", "pipeline_beta");
    // A regular file past the 2 MiB ingestion cap is skipped with a
    // labeled note (never read) — the deterministic hostile-input note.
    std::fs::write(corpus.join("huge.md"), vec![b'x'; 2 * 1024 * 1024 + 1]).unwrap_p();
    let out = td.path().join("out");
    let result = spk()
        .args([
            "compile",
            corpus.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(
        result.status.code(),
        Some(2),
        "the collision still refuses: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap_p();
    assert_eq!(
        json["ok"],
        serde_json::json!(false),
        "refusal envelope: {json}"
    );
    assert_eq!(json["envelope_kind"], serde_json::json!("error"));
    let message = json["warnings"]
        .as_array()
        .map(|ws| {
            ws.iter()
                .filter_map(|w| w["message"].as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    assert!(
        message.contains("duplicate_artifact_stem"),
        "the labeled scope failure survives: {json}"
    );
    assert!(
        message.contains("huge.md") && message.contains("skipped"),
        "the hostile-input skip note rides the refusal envelope: {json}"
    );
    assert!(
        !out.exists(),
        "no artifacts may be written before the refusal"
    );
}

#[test]
fn distinct_artifact_stems_compile_every_file_with_exit_0() {
    // The scope guard is collide-only: a corpus whose files have distinct
    // artifact stems compiles every file exactly as before. Ids equal
    // their filename stems (`id_matches_file`, `-` ⇔ `.`), so this pair
    // is honestly lint-clean — exit 0, never the guard's exit 2.
    let td = tempfile::tempdir().unwrap_p();
    let corpus = td.path().join("corpus");
    write_corpus_spec(&corpus, "alpha", "one.md", "one");
    write_corpus_spec(&corpus, "beta", "two.md", "two");
    let out = td.path().join("out");
    let result = spk()
        .args([
            "compile",
            corpus.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(
        result.status.code(),
        Some(0),
        "distinct stems stay a clean run: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap_p();
    assert_eq!(json["ok"], serde_json::json!(true), "envelope: {json}");
    assert_eq!(json["envelope_kind"], serde_json::json!("ok"));
    assert_eq!(json["data"]["files_compiled"], serde_json::json!(2));
    assert_eq!(json["data"]["files_failed"], serde_json::json!(0));
    for artifact in ["one_props.rs", "two_props.rs"] {
        assert!(
            out.join(artifact).is_file(),
            "{artifact} written for its own stem: {json}"
        );
    }
}

#[test]
fn emitted_props_artifacts_are_rustfmt_clean() {
    // gh-15 (specodelic-w9ju): the generator's *_props.rs emission must be
    // rustfmt-clean by itself — recompiling after a spec amendment must not
    // churn ~14 files of pure formatting that trips downstream
    // `cargo fmt --check` gates. Emitted .rs artifacts pass
    // `rustfmt --check --edition 2021` as written; no post-pass needed.
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("rfc.md");
    write_props_spec(&spec, "rfc");
    let out = td.path().join("out");
    let result = spk()
        .args([
            "compile",
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(result.status.code(), Some(0), "compile must succeed");
    let props_path = std::fs::read_dir(&out)
        .unwrap_p()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "rs"))
        .unwrap_p();

    // Byte-stable across a re-run: same input, byte-identical artifacts.
    let first = std::fs::read(&props_path).unwrap_p();
    let rerun = spk()
        .args([
            "compile",
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert_eq!(rerun.status.code(), Some(0));
    assert_eq!(
        first,
        std::fs::read(&props_path).unwrap_p(),
        "emission must be byte-stable across a re-run"
    );

    let fmt = std::process::Command::new("rustfmt")
        .args([
            "--check",
            "--edition",
            "2021",
            props_path.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    assert!(
        fmt.status.success(),
        "emitted {} is not rustfmt-clean:\n{}{}",
        props_path.display(),
        String::from_utf8_lossy(&fmt.stdout),
        String::from_utf8_lossy(&fmt.stderr)
    );
}
