//! Purpose: executable-fragment grammar and emission tests for `spk
//! compile` (Revision 15/16 behavior, specs `compile` +
//! `linter-failure_shape`), relocated from `src/compile.rs`'s in-module
//! test block (specodelic-ez9a.1: pretender's `file_lines` ratchet is
//! shrink-only and the shared-fragment machinery needs the headroom).
//! Responsibilities: pin fragment extraction (fragment_of_strict), the
//! closed tag set, mention-vs-opt-in positions, emission verbatim-ness,
//! IR invariant extraction, and the no_semantic_drift round trip —
//! against the pub compile API only. Rationale: behavior-locked tests
//! belong beside the other integration suites once a source file sits
//! at its size ratchet; nothing here reaches crate-private items.

use specodelic::compile::{
    ConstraintsDoc, FragmentLang, InvariantIr, compile_spec, extract_model_ir, fragment_of_strict,
    model_to_tla, properties_to_proptest,
};
use specodelic::spec::{Spec, parse_str};

mod common;
use common::{UnwrapPanic, UnwrapPanicErr};

#[test]
fn guard_rust_marker_still_rejected_labeled() {
    // Scenario: Executable guard fragments stay rejected (delta
    // spec, fragment_guard_rejected_stands) — a **rust:** guard
    // fails labeled at fragment_extraction, never silently ignored.
    let text = FRAGMENT_SAMPLE
        .replace(
            "| t1 | s1 | s2 | [[demo.frags.a]] |",
            "| t1 | s1 | s2 | `**rust:** v0.len() >= 1` |",
        )
        .replace("`word()`", "`word(), num()`");
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
    assert!(e.message.contains("guard"), "{}", e.message);
}

// --- specodelic-rjb: executable predicate fragments (Revision 15) ---

const FRAGMENT_SAMPLE: &str = "---\nid: demo.frags\nkind: intent\nstatement: \"THE system SHALL work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `**rust:** state != \"blackhole\"` | [[demo.frags]] |\n| b | effect | `y fires` | [[demo.frags]] |\n\n## Model\n\n### States\n\n- s1\n- s2\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s2 | [[demo.frags.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p1 | unit | [[demo.frags.a]] | `word()` | `**rust:** v0.len() >= 1` |\n| p2 | unit | [[demo.frags.a]] | `word(), num()` | `**rust:** v0.len() >= 1 && v1.len() >= 1` |\n";

fn fragment_sample() -> Spec {
    parse_str(FRAGMENT_SAMPLE).unwrap_p()
}

/// Prose-only counterpart of FRAGMENT_SAMPLE: the same model with the
/// marker cells replaced by prose, so placeholder emission and empty
/// invariant extraction stay pinned without reaching the in-module
/// SAMPLE (which stayed in `src/compile.rs` for the tests there).
fn prose_sample() -> Spec {
    let text = FRAGMENT_SAMPLE
        .replace(
            "`**rust:** state != \"blackhole\"`",
            "`the run never reaches the blackhole state`",
        )
        .replace("`**rust:** v0.len() >= 1`", "`check(v0) == ok`")
        .replace(
            "`**rust:** v0.len() >= 1 && v1.len() >= 1`",
            "`check(v0) == ok && check(v1) == ok`",
        );
    parse_str(&text).unwrap_p()
}

#[test]
fn fragment_body_emitted_verbatim() {
    let src = properties_to_proptest(&fragment_sample());
    assert!(
            src.contains("assert!(\n            v0.len() >= 1,\n            \"predicate `p1` violated (executable fragment)\",\n        );"),
            "fragment must be the assertion body, verbatim: {src}"
        );
    assert!(
        src.contains("assert!(\n            v0.len() >= 1 && v1.len() >= 1,"),
        "multi-generator fragment binds v0 and v1: {src}"
    );
    // The fragment rows carry no placeholder.
    assert!(!src.contains("todo_predicate!(\"**rust:**"));
}

#[test]
fn fragment_values_bind_generators_as_strings() {
    let src = properties_to_proptest(&fragment_sample());
    assert!(
        src.contains("impl Strategy<Value = String>"),
        "generators yield plain Strings (GenVal retired, Revision 15): {src}"
    );
    assert!(!src.contains("GenVal"), "GenVal must be gone: {src}");
}

#[test]
fn prose_predicate_stays_placeholder() {
    // Pure widening: a cell without the marker compiles exactly as
    // before (specodelic.md Revision 15).
    let src = properties_to_proptest(&prose_sample());
    assert!(src.contains("todo_predicate!"));
    assert!(!src.contains("executable fragment"));
}

#[test]
fn law_fragment_rejected_labeled() {
    // specodelic-sd1: the fixture opts in via the canonical fragment
    // position (span-opening marker) — a mid-cell prose marker is a
    // mention, not an opt-in.
    let text = FRAGMENT_SAMPLE.replace(
        "| p1 | unit | [[demo.frags.a]] | `word()` | `**rust:** v0.len() >= 1` |",
        "| p1 | law | [[demo.frags.a]] | `word()` | `**rust:** v0.len() >= 1` |",
    );
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
    assert!(e.message.contains("law"), "{}", e.message);
}

#[test]
fn hygiene_violation_rejected_labeled_names_token() {
    for (token, frag) in [
        ("unsafe", "unsafe { 1 == 1 }"),
        ("std::fs", "std::fs::metadata(\"x\").is_ok()"),
        ("Command", "Command::new(\"sh\") == never"),
    ] {
        let text =
            FRAGMENT_SAMPLE.replace("`**rust:** v0.len() >= 1`", &format!("`**rust:** {frag}`"));
        let spec = parse_str(&text).unwrap_p();
        let e = compile_spec(&spec).unwrap_p_err();
        assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
        assert!(
            e.message.contains(token),
            "message must name the token {token}: {}",
            e.message
        );
    }
}

#[test]
fn guard_fragment_rejected_labeled() {
    let text = FRAGMENT_SAMPLE.replace(
        "| t1 | s1 | s2 | [[demo.frags.a]] |",
        "| t1 | s1 | s2 | `**rust:** true` |",
    );
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
    assert!(e.message.contains("t1"), "{}", e.message);
}

#[test]
fn non_invariant_fragment_rejected_labeled() {
    let text = FRAGMENT_SAMPLE.replace(
        "| b | effect | `y fires` | [[demo.frags]] |",
        "| b | effect | `**rust:** true` | [[demo.frags]] |",
    );
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
    assert!(e.message.contains("b"), "{}", e.message);
}

#[test]
fn double_marker_rejected_labeled() {
    // specodelic-sd1: two markers in FRAGMENT POSITION (each opening
    // its own code span) are still the labeled at-most-one failure —
    // a mid-span second occurrence is a mention, not a second marker.
    let text = FRAGMENT_SAMPLE.replace(
        "`**rust:** v0.len() >= 1`",
        "`**rust:** v0.len() >= 1`, `**rust:** v0.len() >= 2`",
    );
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
}

#[test]
fn empty_fragment_rejected_labeled() {
    let text = FRAGMENT_SAMPLE.replace("`**rust:** v0.len() >= 1`", "`**rust:**`");
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
}

// --- specodelic-lf3: closed language-tag grammar (Revision 16) ---

#[test]
fn unknown_tag_rejected_labeled() {
    // unknown_tag_rejected: `**go:**` in fragment position is a labeled
    // extraction failure naming the tag AND the closed set — never
    // prose, never silence, never a Rust fall-through.
    let text = FRAGMENT_SAMPLE.replace("`**rust:** v0.len() >= 1`", "`**go:** len(v0) >= 1`");
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
    assert!(e.message.contains("go"), "names the tag: {}", e.message);
    assert!(
        e.message.contains("rust") && e.message.contains("py") && e.message.contains("ts"),
        "names the closed set: {}",
        e.message
    );
}

#[test]
fn py_tag_extracts_at_grammar_level() {
    // one_tag_extracts_per_cell: acceptance is grammar-level and
    // emitter-independent — extraction must succeed for `**py:**` even
    // though compile later gates it (missing_emitter_labeled is the
    // compile-stage failure, not the extraction).
    let cell = "`**py:** len(v0) >= 1`";
    let f = fragment_of_strict(cell).unwrap_p().unwrap_p();
    assert_eq!(f.lang.tag(), "py");
    assert_eq!(f.text, "len(v0) >= 1");
}

#[test]
fn py_no_emitter_fails_labeled() {
    // no_emitter_labeled_failure: a **py:** fragment reaching compile
    // fails labeled with stage compile.emission_failure and a
    // remediation naming the py-emitter follow-up — never a silent
    // fall-through to Rust emission, never a vacuous artifact.
    let text = FRAGMENT_SAMPLE.replace("`**rust:** v0.len() >= 1`", "`**py:** len(v0) >= 1`");
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "compile.emission_failure", "{}", e.message);
    assert!(
        e.message.contains("add-py-fragment-emission"),
        "remediation names the follow-up: {}",
        e.message
    );
    assert!(
        e.message.contains("rust"),
        "names the one executable tag: {}",
        e.message
    );
}

#[test]
fn ts_no_emitter_fails_labeled() {
    let text = FRAGMENT_SAMPLE.replace(
        "`**rust:** v0.len() >= 1 && v1.len() >= 1`",
        "`**ts:** v0.length >= 1 && v1.length >= 1`",
    );
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "compile.emission_failure", "{}", e.message);
    assert!(
        e.message.contains("add-ts-fragment-emission"),
        "remediation names the follow-up: {}",
        e.message
    );
}

#[test]
fn expr_cell_same_tag_grammar() {
    // expr_tag_same_grammar: an invariant-kind Constraint expr cell
    // accepts the same closed-set grammar — gated at compile by the
    // emitter, not rejected as a non-invariant/unknown-shape row.
    let text = FRAGMENT_SAMPLE.replace(
        "`**rust:** state != \"blackhole\"`",
        "`**py:** state != \"blackhole\"`",
    );
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "compile.emission_failure", "{}", e.message);
}

#[test]
fn guard_py_marker_still_rejected_labeled() {
    // Guards reject ANY fragment-position marker, per tag — the
    // program-counter model has no data binding, and an unknown tag
    // there is likewise labeled, never silence.
    let text = FRAGMENT_SAMPLE
        .replace(
            "| t1 | s1 | s2 | [[demo.frags.a]] |",
            "| t1 | s1 | s2 | `**py:** v0.len() >= 1` |",
        )
        .replace("`word()`", "`word(), num()`");
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
    assert!(e.message.contains("guard"), "{}", e.message);
}

#[test]
fn guard_unknown_tag_marker_rejected_labeled() {
    let text = FRAGMENT_SAMPLE
        .replace(
            "| t1 | s1 | s2 | [[demo.frags.a]] |",
            "| t1 | s1 | s2 | `**go:** v0.len() >= 1` |",
        )
        .replace("`word()`", "`word(), num()`");
    let spec = parse_str(&text).unwrap_p();
    let e = compile_spec(&spec).unwrap_p_err();
    assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
}

#[test]
fn midspan_py_marker_is_mention() {
    // mention_not_extraction: a mid-span **py:** occurrence compiles
    // byte-identically to the same cell without it (sd1 rule per tag).
    let predicate_with = "`a row mentioning **py:** mid-span, plus the placeholder v0.len() >= 1`";
    let predicate_without = "`a row mentioning py mid-span, plus the placeholder v0.len() >= 1`";
    let text_with = FRAGMENT_SAMPLE
        .replace("`**rust:** v0.len() >= 1`", predicate_with)
        .replace(
            "`**rust:** v0.len() >= 1 && v1.len() >= 1`",
            "`v0.len() >= 1 && v1.len() >= 1`",
        )
        .replace(
            "`**rust:** state != \"blackhole\"`",
            "`state != \"blackhole\"`",
        );
    let text_without = text_with.replace(predicate_with, predicate_without);
    let spec_with = parse_str(&text_with).unwrap_p();
    let spec_without = parse_str(&text_without).unwrap_p();
    assert!(
        fragment_of_strict("a row mentioning **py:** mid-span").is_ok_and(|f| f.is_none()),
        "mid-span occurrence never extracts"
    );
    compile_spec(&spec_with).unwrap_p();
    let src_with = properties_to_proptest(&spec_with);
    let src_without = properties_to_proptest(&spec_without);
    assert_eq!(
        src_with.replace("**py:**", "py"),
        src_without,
        "mid-span occurrence never extracts (artifact-identical modulo the mention text)"
    );
}

#[test]
fn rust_back_compat_byte_identical() {
    // rust_back_compat: the pre-change Rust extraction path is pure —
    // the widened scanner must extract exactly the Revision 15
    // fragment for the canonical fixtures, and every Revision 15 test
    // above must pass unchanged (CI proves the rest).
    let cell = "`**rust:** v0.len() >= 1`";
    let f = fragment_of_strict(cell).unwrap_p().unwrap_p();
    assert_eq!(f.lang.tag(), "rust");
    assert_eq!(f.text, "v0.len() >= 1");
    let bare = "**rust:** v0.len() >= 1";
    let f = fragment_of_strict(bare).unwrap_p().unwrap_p();
    assert_eq!(f.text, "v0.len() >= 1");
}

#[test]
fn bold_prose_label_is_not_a_marker() {
    // The tag shape is [a-z][a-z0-9_-]* — an uppercase-initial bold
    // label (`**Note:**`) in fragment position is prose, never an
    // unknown-tag failure: the closed-set grammar must not criminalize
    // ordinary bold prose.
    let cell = "`**Note:** keep the placeholder honest`";
    assert!(
        fragment_of_strict(cell).is_ok_and(|f| f.is_none()),
        "non-tag-shaped bold label is prose"
    );
}

// --- specodelic-sd1: marker mentions are not opt-ins ---

/// The two corpus mention shapes: a **rust:** occurrence MID-SPAN in a
/// defining row's expr cell (compile.md's predicate_fragment_opt_in et
/// al.) and one in PROSE between code spans (verify.md's
/// fragments_reach_verified predicate).
const MENTION_SAMPLE: &str = "---\nid: demo.mention\nkind: intent\nstatement: \"THE system SHALL work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `every row compiles verbatim as a **rust:** fragment when the row opts in, else the placeholder` | [[demo.mention]] |\n\n## Model\n\n### States\n\n- s1\n- s2\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s2 | [[demo.mention.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[demo.mention.a]] | `word()` | `check(file) == verified` — a passing **rust:** predicate and a clean executable-invariant run verify a real file |\n";

#[test]
fn marker_mentions_never_extract() {
    // specodelic-sd1: an occurrence of **rust:** that is not in
    // fragment position (cell start, or opening the fragment's code
    // span) is a mention of the mechanism, never an opt-in — the
    // extractor must not read the prose after it as a fragment. The
    // corpus rows that DEFINE the marker carry such mentions in their
    // own cells; before this fix the defining-row prose was extracted
    // as Rust and the scratch runs hard-failed.
    let spec = parse_str(MENTION_SAMPLE).unwrap_p();
    // Mid-span mention → no executable invariant in the IR.
    let ir = extract_model_ir(&spec);
    assert!(
        ir.invariants.is_empty(),
        "mid-span mention must not extract an invariant: {ir:?}"
    );
    // Prose mention between spans → the honest placeholder body.
    let src = properties_to_proptest(&spec);
    assert!(
        !src.contains("assert!("),
        "prose mention must not extract: {src}"
    );
    assert!(src.contains("todo_predicate!("));
}

#[test]
fn span_opening_optin_extracts_mention_alongside_ignored() {
    // The Revision 15 opt-in form still extracts when the cell ALSO
    // mentions the marker in prose outside the fragment span — the
    // mention is not a second marker, the cell compiles cleanly.
    let text = FRAGMENT_SAMPLE.replace(
        "`**rust:** v0.len() >= 1`",
        "`**rust:** v0.len() >= 1` — mentioned again as a **rust:** fragment in prose",
    );
    let spec = parse_str(&text).unwrap_p();
    let src = properties_to_proptest(&spec);
    assert!(
        src.contains("assert!(\n            v0.len() >= 1,"),
        "span-opening opt-in must still extract: {src}"
    );
}

#[test]
fn executable_invariants_extract_into_the_ir() {
    let ir = extract_model_ir(&fragment_sample());
    assert_eq!(
        ir.invariants,
        vec![InvariantIr {
            id: "a".into(),
            fragment: "state != \"blackhole\"".into(),
        }],
        "invariant fragments ride the ModelIR"
    );
    assert!(extract_model_ir(&prose_sample()).invariants.is_empty());
}

#[test]
fn module_carries_the_fragment_manifest() {
    // rerun_on_model_change: the manifest comment makes a fragment
    // edit change the artifact hash.
    let tla = model_to_tla(&fragment_sample());
    assert!(
        tla.contains("\\* INVARIANT a **rust:** state != \"blackhole\""),
        "manifest comment must carry id + fragment verbatim: {tla}"
    );
    assert!(!model_to_tla(&prose_sample()).contains("INVARIANT"));
}

#[test]
fn no_semantic_drift_roundtrip_is_byte_identical() {
    let spec = prose_sample();
    let first = compile_spec(&spec).unwrap_p();
    let second = compile_spec(&spec).unwrap_p();
    assert_eq!(first.toml, second.toml);
    assert_eq!(first.props, second.props);
    // re-parse + re-emit == original
    let doc: ConstraintsDoc = toml::from_str(&first.toml).unwrap_p();
    assert_eq!(toml::to_string(&doc).unwrap_p(), first.toml);
}
