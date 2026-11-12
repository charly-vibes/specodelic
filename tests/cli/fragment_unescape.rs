// Purpose: regression tests for specodelic-q7uo (catitas A1) — the compile
// pipeline must unescape the table-cell pipe escape (`\|`) before fragment
// emission, so `||`, `|=`, and closures `|x|` are writable in `**rust:**`
// fragments.
// Responsibilities: pin (1) end-to-end behavior — a spec whose fragment
// cells carry escaped pipes compiles to artifacts whose emitted Rust
// contains literal pipes and passes both the verify cargo-test run and the
// model-check scratch-crate run; (2) byte-stability — inputs without the
// escape emit byte-identical artifacts (the unescape is identity there).
// Rationale: the parser splits cells on unescaped pipes and keeps the
// escape in the cell text (src/guide.md), so without this unescape a
// fragment authored `\|\|` arrived verbatim and the emitted Rust did not
// compile. The escape is markdown transport, not fragment content.
use super::*;
use crate::common::UnwrapPanic;

/// The escaped-pipe fixture: invariant fragments using `||`, a closure
/// `|c|`, and `|=`, plus property predicate fragments using `||` and a
/// closure. Every pipe is written escaped (`\|`) as guide.md requires for
/// a literal pipe in a table cell.
fn write_escaped_pipe_spec(path: &std::path::Path) {
    std::fs::write(
        path,
        "---\nid: frag_pipe\nkind: intent\nstatement: \"THE frag_pipe SHALL carry escaped-pipe fragments\"\n---\n\
         \n## Constraints\n\
         \n| id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | c_or | invariant | `**rust:** state == \"s1\" \\|\\| state == \"s2\"` | [[frag_pipe]] |\n\
         | c_cl | invariant | `**rust:** state.chars().any(\\|c\\| c == 's')` | [[frag_pipe]] |\n\
         | c_bor | invariant | `**rust:** { let mut bits = 1; bits |= 2; bits == 3 }` | [[frag_pipe]] |\n\
         \n## Model\n\
         \n### States\n\
         \n- s1\n\
         - s2\n\
         \n### Transitions\n\
         \n| id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | t | s1 | s2 | [[frag_pipe.c_or]] |\n\
         \n## Properties\n\
         \n| id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|------------|\n\
         | p_or | unit | [[frag_pipe.c_or]] | `word()` | `**rust:** !v0.is_empty() \\|\\| v0 == \"skip\"` |\n\
         | p_cl | unit | [[frag_pipe.c_cl]] | `word()` | `**rust:** v0.chars().any(\\|ch\\| ch == 'r')` |\n\
         | p_bor | unit | [[frag_pipe.c_bor]] | `word()` | `**rust:** { let mut seen = false; seen \\|= !v0.is_empty(); seen }` |\n",
    )
    .unwrap_p();
}

/// The no-escape fixture: the same shape with plain fragments — the
/// byte-stability pin (artifacts must equal the pre-change bytes exactly).
fn write_plain_spec(path: &std::path::Path) {
    std::fs::write(
        path,
        "---\nid: frag_plain\nkind: intent\nstatement: \"THE frag_plain SHALL carry plain fragments\"\n---\n\
         \n## Constraints\n\
         \n| id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | c1 | invariant | `**rust:** state == \"s1\"` | [[frag_plain]] |\n\
         \n## Model\n\
         \n### States\n\
         \n- s1\n\
         - s2\n\
         \n### Transitions\n\
         \n| id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | t | s1 | s2 | [[frag_plain.c1]] |\n\
         \n## Properties\n\
         \n| id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|------------|\n\
         | p | unit | [[frag_plain.c1]] | `word()` | `**rust:** v0.len() >= 1` |\n",
    )
    .unwrap_p();
}

fn compile(spec: &std::path::Path, out: &std::path::Path) {
    spk()
        .args([
            "compile",
            spec.to_str().unwrap_p(),
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .assert()
        .success();
}

#[test]
fn escaped_pipes_reach_emitted_rust_unescaped_and_verify() {
    // specodelic-q7uo: `\|` is markdown table transport, never fragment
    // content — the emitted artifacts must carry literal pipes and the
    // downstream Rust consumers (verify's cargo-test run over *_props.rs,
    // model-check's scratch crate) must compile and pass.
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("frag_pipe.md");
    write_escaped_pipe_spec(&spec);
    let out = td.path().join("out");
    compile(&spec, &out);

    let props = std::fs::read_to_string(out.join("frag_pipe_props.rs")).unwrap_p();
    assert!(
        props.contains("!v0.is_empty() || v0 == \"skip\""),
        "property fragment must carry literal `||`:\n{props}"
    );
    assert!(
        props.contains("v0.chars().any(|ch| ch == 'r')"),
        "property fragment must carry a literal closure:\n{props}"
    );
    assert!(
        props.contains("seen |= !v0.is_empty()"),
        "property fragment must carry literal `|`:\n{props}"
    );
    // The `// predicate:` metadata comments quote the raw cell verbatim
    // (that is their documenting job); the COMPILED lines must carry no
    // escape sequences.
    let compiled: String = props
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !compiled.contains('\\'),
        "no escape sequences survive in emitted Rust code:\n{compiled}"
    );

    let tla = std::fs::read_to_string(out.join("frag_pipe.tla")).unwrap_p();
    assert!(
        tla.contains("state == \"s1\" || state == \"s2\""),
        "tla manifest must carry the unescaped fragment:\n{tla}"
    );
    assert!(
        tla.contains("state.chars().any(|c| c == 's')"),
        "tla manifest must carry the unescaped closure:\n{tla}"
    );

    let toml_text = std::fs::read_to_string(out.join("frag_pipe.toml")).unwrap_p();
    assert!(
        !toml_text.contains('\\'),
        "constraints toml must carry no escape sequences:\n{toml_text}"
    );
    assert!(
        toml_text.contains("state == \"s1\" ||"),
        "constraints toml expr must carry the unescaped fragment:\n{toml_text}"
    );

    // The honest executable checks: model-check cargo-compiles the
    // scratch crate embedding the invariant fragments; verify
    // cargo-compiles *_props.rs and runs it (and gates on the
    // model-check report, so it runs second). Both must pass — pre-fix
    // both fail to compile.
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
    let mcjson: serde_json::Value = serde_json::from_slice(&mc.stdout).unwrap_p();
    assert_eq!(mc.status.code(), Some(0), "model-check must pass: {mcjson}");
    assert_eq!(mcjson["data"]["outcome"], "no_counterexample");
    let statuses = mcjson["data"]["checked"][0]["invariant_statuses"]
        .as_array()
        .unwrap_p();
    assert_eq!(
        statuses.len(),
        3,
        "all three executable invariants: {mcjson}"
    );
    for s in statuses {
        assert_eq!(s["status"], "verified", "claim {}: {}", s["id"], s);
    }

    let v = spk()
        .args([
            "verify",
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    let vjson: serde_json::Value = serde_json::from_slice(&v.stdout).unwrap_p();
    assert_eq!(v.status.code(), Some(0), "verify must pass: {vjson}");
    assert_eq!(vjson["data"]["status"], "verified");
}

#[test]
fn fragments_without_escape_emit_byte_identical_artifacts() {
    // specodelic-q7uo acceptance: legacy artifacts are byte-identical for
    // inputs without `\|`. The unescape is identity there; these pins are
    // the pre-change bytes, so any drift in the emission format fails here.
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("frag_plain.md");
    write_plain_spec(&spec);
    let out = td.path().join("out");
    compile(&spec, &out);

    let props = std::fs::read_to_string(out.join("frag_plain_props.rs")).unwrap_p();
    let tla = std::fs::read_to_string(out.join("frag_plain.tla")).unwrap_p();
    let toml_text = std::fs::read_to_string(out.join("frag_plain.toml")).unwrap_p();
    assert_eq!(
        props, EXPECTED_PLAIN_PROPS,
        "*_props.rs drifted for a fragment without escapes"
    );
    assert_eq!(
        tla, EXPECTED_PLAIN_TLA,
        "*.tla drifted for a fragment without escapes"
    );
    assert_eq!(
        toml_text, EXPECTED_PLAIN_TOML,
        "*.toml drifted for a fragment without escapes"
    );
}

// Pre-change artifact bytes for the frag_plain fixture (captured from the
// unfixed binary) — the byte-stability floor. Regenerate only if the
// emission format changes of record, never to accommodate a behavior fix.
const EXPECTED_PLAIN_PROPS: &str = r#"//! proptest scaffolding for `frag_plain` — generated by `spk compile`.
//! Source of truth: the spec file itself — edit there, not here.
//! Each block quotes its row's generator and predicate verbatim;
//! a predicate opting in with the **rust:** marker (executable
//! fragment, specodelic.md Revision 15) is emitted verbatim as the
//! block's assertion body; predicates not yet translated fail in
//! `verify` via `todo_predicate!`, never at artifact emission.

use proptest::prelude::*;

/// Marker for predicates not yet translated to Rust: compiles here,
/// panics when executed — execution is `verify`'s job. A predicate
/// opting in with **rust:** compiles as a real assertion instead
/// (executable predicate fragment, specodelic.md Revision 15).
macro_rules! todo_predicate {
    ($reason:expr) => {
        todo!("{}", $reason)
    };
}

/// Generated strategies — one per generator named in the spec rows.
/// Each yields a String (the generator's name; executable
/// fragments bind these directly — Revision 15).
pub mod spec_gen {
    use super::*;

    pub fn word() -> impl Strategy<Value = String> {
        Just("word".into())
    }
}

proptest! {
    // id: p
    // generator: `word()`
    // predicate: `**rust:** v0.len() >= 1`
    #[test]
    fn p(v0 in spec_gen::word()) {
        assert!(
            v0.len() >= 1,
            "predicate `p` violated (executable fragment)",
        );
    }
}
"#;
const EXPECTED_PLAIN_TLA: &str = r#"---- MODULE fragplain ----
\* Generated by `spk compile` from spec `frag_plain` — source of truth is the
\* spec file; edit there, not here. Extends nothing: the module is
\* self-contained so any engine can open it.

\* Executable invariant fragments — id + verbatim **rust:** fragment;
\* carried in the module so the artifact hash covers fragment edits.
\* INVARIANT c1 **rust:** state == "s1"

\* Each State becomes a value in the state variable's range.
StateValues == {"s1", "s2"}

VARIABLES vpc   \* the state variable (program counter)

vars == <<vpc>>

TypeOK == vpc \in StateValues

\* Initial state: first listed in the spec (the format has no explicit initial marker).
Init == vpc = "s1"

Next ==
  \* t: s1 -> s2 (guard: [[frag_plain.c1]])
  \/ vpc = "s1" /\ vpc' = "s2"
  \* stuttering: guards are prose (uninterpreted) — a terminal
  \* state must not read as an engine-side deadlock
  \/ UNCHANGED vpc

\* No state carries an `emits` field — Output is simply empty.
Output == << >>

============================================================================
"#;
const EXPECTED_PLAIN_TOML: &str = r#"source = "frag_plain"

[[constraints]]
id = "c1"
kind = "invariant"
expr = '`**rust:** state == "s1"`'
traces_to = "[[frag_plain]]"
"#;
