//! Identity emission gate (`add-acset-writer` tasks 3.1–3.2).
//!
//! `emit(parse(x), no edit) == x` byte for byte, for every spec file the
//! parser accepts (`acset-writer` capability, `emit_identity`). The writer
//! never re-serializes: with no edit it must reproduce the source exactly,
//! and the only way it can promise that later edits are safe is by
//! verifying every recorded span resolves in-bounds to the bytes its
//! parsed element carries. A parser that drifts from its span recording
//! fails here, corpus-wide, before any edit can corrupt a file.
//!
//! Three legs, mirroring the delta's Properties:
//! - fixtures over the whole `specs/` corpus (plus the checked-in fixture
//!   directories — extra parse shapes for free),
//! - an explicit CRLF fixture (the Windows-runner CRLF gotcha from
//!   `acset_parity` — identity here must be EXACT, no agnostic compare),
//! - the `identity_emission_exact` proptest over generated spec-shaped
//!   inputs (padding, backticks, padded links, quoted ids, CRLF).

use proptest::prelude::*;
use specodelic::acset::writer;
use specodelic::spec::{Spec, parse_str};

/// Walk `dir` recursively for `.md` files, sorted (mirrors main.rs's
/// collect_specs and acset_parity's collect_corpus).
fn walk_md(dir: &str, files: &mut Vec<std::path::PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            let d = p.display().to_string();
            walk_md(&d, files);
        } else if p.extension().and_then(|e| e.to_str()) == Some("md") {
            files.push(p);
        }
    }
}

/// `(path, source, spec)` for every parseable spec file under `dir` —
/// non-spec `.md` (no frontmatter: AGENTS.md, STATUS.md, the checklist
/// manifest) is skipped exactly as main.rs's collect does.
fn corpus(dir: &str) -> Vec<(std::path::PathBuf, String, Spec)> {
    let mut files = Vec::new();
    walk_md(dir, &mut files);
    files
        .iter()
        .filter_map(|p| {
            let text = std::fs::read_to_string(p).ok()?;
            if !text.trim_start().starts_with("---") {
                return None;
            }
            let spec = parse_str(&text).ok()?;
            Some((p.clone(), text, spec))
        })
        .collect()
}

/// Identity emission over the whole `specs/` corpus — byte for byte.
#[test]
fn identity_emission_over_the_real_corpus() {
    for (path, source, spec) in corpus("specs") {
        let emitted = writer::emit(&source, &spec)
            .unwrap_or_else(|e| panic!("{}: emit failed: {}", path.display(), e));
        assert_eq!(
            emitted,
            source,
            "identity emission drifted for {}",
            path.display()
        );
    }
}

/// The checked-in fixture directories carry parse shapes the corpus
/// doesn't (dangling refs, typing violations) — identity must hold there
/// too: every file the parser accepts, not only well-formed corpora.
#[test]
fn identity_emission_over_fixture_directories() {
    for dir in [
        "tests/fixtures/small",
        "tests/fixtures/single_intent",
        "tests/fixtures/typing_allowed",
        "tests/fixtures/typing_violations",
        "tests/fixtures/dangling",
        "tests/fixtures/zero_file",
    ] {
        for (path, source, spec) in corpus(dir) {
            let emitted = writer::emit(&source, &spec)
                .unwrap_or_else(|e| panic!("{}: emit failed: {}", path.display(), e));
            assert_eq!(
                emitted,
                source,
                "identity emission drifted for {}",
                path.display()
            );
        }
    }
}

/// One of every editable family, LF line endings — the same shape the
/// span tests pin, now end-to-end through emit.
fn full_family_spec(id: &str) -> String {
    format!(
        "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL hold one of every rewrite family\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `holds` | [[{id}]] |\n\n## Model\n\n### States\n- spanned\n- applied (emits: [[{id}.c1]])\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | spanned | applied | [[{id}.c1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | [[{id}.c1]] | `g()` | `x` |\n"
    )
}

/// CRLF identity must be EXACT — the agnostic compares elsewhere could
/// never catch a `\r\n` → `\n` flip, and the writer touches exactly the
/// bytes (padding, terminators) a flip would destroy.
#[test]
fn identity_emission_exact_under_crlf() {
    let source = full_family_spec("crlf.id").replace('\n', "\r\n");
    assert!(source.matches("\r\n").count() > 0, "fixture is CRLF");
    let spec = parse_str(&source).expect("CRLF fixture parses");
    let emitted = writer::emit(&source, &spec).expect("CRLF identity emission must not fail");
    assert_eq!(emitted, source, "CRLF bytes must survive emit unchanged");
}

// ---------------------------------------------------------------------------
// `identity_emission_exact` — the delta's proptest leg: generated
// spec-shaped inputs the parser accepts, identity byte for byte.
// ---------------------------------------------------------------------------

/// File-id-shaped identifiers (the parser's id shape: dot-namespaced
/// lowercase tokens).
fn arb_id() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,7}(\\.[a-z][a-z0-9_]{0,7}){0,2}"
}

// Proptest over generated spec files: arbitrary ids, optional backticks
// and padding in id cells, optional padded `[[ link ]]` inners, optional
// quoted frontmatter ids (whose span is legitimately unrecorded), and
// either line terminator — emit must reproduce every accepted input
// byte for byte.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    // `identity_emission_exact` — the delta's generator/predicate pair,
    // over generated spec files: arbitrary ids, optional backticks and
    // padding in id cells, optional padded `[[ link ]]` inners, optional
    // quoted frontmatter ids (whose span is legitimately unrecorded), and
    // either line terminator — emit must reproduce every accepted input
    // byte for byte.
    #[test]
    fn identity_emission_exact(
        id in arb_id(),
        n_rows in 1usize..4,
        padded_cells in prop::bool::ANY,
        backticked_ids in prop::bool::ANY,
        padded_links in prop::bool::ANY,
        quoted_intent_id in prop::bool::ANY,
        crlf in prop::bool::ANY,
    ) {
        let pad = |cell: &str| {
            if padded_cells { format!(" {cell}  ") } else { format!(" {cell} ") }
        };
        let id_cell = |cid: &str| {
            let raw = if backticked_ids { format!("`{cid}`") } else { cid.to_string() };
            pad(&raw)
        };
        let link = |target: &str| {
            if padded_links { format!("[[ {target} ]]") } else { format!("[[{target}]]") }
        };
        let intent_value = if quoted_intent_id {
            format!("\"{id}\"")
        } else {
            id.clone()
        };
        let mut file = format!(
            "---\nid: {intent_value}\nkind: intent\nstatement: \"THE {id} SHALL round trip\"\n---\n\nProse mentioning {id} as a word must survive untouched.\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n"
        );
        for i in 0..n_rows {
            file.push_str(&format!(
                "|{}|{}|{}|{}|\n",
                id_cell(&format!("c{i}")),
                pad("invariant"),
                pad("`holds`"),
                pad(&link(&id)),
            ));
        }
        file.push_str(&format!(
            "\n## Model\n\n### States\n- spanned\n- applied (emits: {link_target})\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | spanned | applied | {guard} |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | {derives} | `g()` | `x` |\n",
            link_target = link(&format!("{id}.c1")),
            guard = link(&format!("{id}.c0")),
            derives = link(&format!("{id}.c0")),
        ));
        if crlf {
            file = file.replace('\n', "\r\n");
        }
        let spec = if let Ok(s) = parse_str(&file) {
            s
        } else {
            prop_assume!(false, "generator produced an unparseable file");
            unreachable!()
        };
        let emitted = writer::emit(&file, &spec)
            .map_err(|e| TestCaseError::fail(format!("emit failed: {e}")))?;
        prop_assert_eq!(emitted, file);
    }
}

// ---------------------------------------------------------------------------
// Task 3.3: span-extraction failures surface as `spec.span_failure` —
// labeled, with a non-empty remediation hint (`span_failure_label_asserted`).
// The failure paths: a recorded span whose bytes no longer match its parsed
// element (source mutated after parse), and a span driven out of bounds by
// a shrunken source — emit must return the labeled error, never panic and
// never return doctored text.
// ---------------------------------------------------------------------------

/// The delta's exact label — the property asserts the string verbatim.
const SPAN_FAILURE_LABEL: &str = "spec.span_failure";

#[test]
fn span_failure_label_asserted_on_disagreement() {
    let source = full_family_spec("spanf.a");
    let spec = parse_str(&source).expect("fixture parses");
    // Mutate the source AFTER parsing: the row's recorded id-cell span now
    // covers bytes that are not the parsed id — the writer must refuse.
    let mutated = source.replace("| c1 |", "| cZ |");
    assert_ne!(mutated, source, "mutation must actually change the source");
    let err = writer::emit(&mutated, &spec).expect_err("span disagreement must fail");
    assert_eq!(err.label, SPAN_FAILURE_LABEL);
    assert!(
        err.detail.contains("c1"),
        "detail must name the disagreed element, got: {}",
        err.detail
    );
    assert!(
        !err.remediation.is_empty(),
        "the fleet error contract requires a non-empty remediation hint"
    );
}

#[test]
fn span_failure_label_asserted_on_out_of_bounds() {
    let source = full_family_spec("spanf.b");
    let spec = parse_str(&source).expect("fixture parses");
    // Shrink the source after parsing: recorded spans now point past EOF.
    let truncated = &source[..source.len() / 2];
    let err = writer::emit(truncated, &spec).expect_err("out-of-bounds span must fail");
    assert_eq!(err.label, SPAN_FAILURE_LABEL);
    assert!(
        !err.remediation.is_empty(),
        "the fleet error contract requires a non-empty remediation hint"
    );
}
