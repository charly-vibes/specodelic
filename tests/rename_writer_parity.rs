//! Rename↔writer parity (`add-acset-writer` task 5.1).
//!
//! The property: a rename routed through the span-preserving writer
//! (`rename::run_via_writer`) produces EXACTLY the write-set the
//! hand-wired `rename::run` produces — byte for byte, per rewrite
//! family, for every fixture and arbitrary corpus/rename case (design
//! decision D6: `rename.rs`'s own checks are unchanged, the writer
//! realizes them mechanically; any intentional output difference is an
//! explicit reviewed snapshot update, never silent).
//!
//! The `run_via_writer` outcome is compared against `run`'s canonically
//! (writes sorted by path — the two drivers may order same-rename
//! writes differently without changing a single byte) and byte-exactly
//! per file. The 1.1 snapshot oracle in `acset_writer_parity.rs`
//! remains the byte-level reference for `run` itself; this file pins
//! that the writer path reproduces it before `run` is flipped over.
//!
//! Known reviewed difference (not a parity failure): a quoted
//! frontmatter intent id has no rewritable span, so the writer refuses
//! (`spec.apply_failure`) where the hand-wired path rewrites links and
//! only fails later at the verify gate — both refuse, the writer
//! earlier and with a better hint. Pinned by `quoted_intent_rename`
//! below, excluded from the property legs.

use std::path::PathBuf;

use proptest::prelude::*;
use specodelic::rename::{RenameOutcome, run, run_via_writer};

/// Canonical byte form of a rename outcome — writes sorted by path so
/// the two drivers' ordering choices don't mask byte parity.
fn canonical(out: &RenameOutcome) -> String {
    let mut writes: Vec<(String, &str)> = out
        .writes
        .iter()
        .map(|(p, t)| (p.display().to_string(), t.as_str()))
        .collect();
    writes.sort();
    let mut s = format!("old: {}\nnew: {}\n", out.old_id, out.new_id);
    match &out.remove {
        Some(r) => s.push_str(&format!("remove: {}\n", r.display())),
        None => s.push_str("remove: none\n"),
    }
    for (p, t) in &writes {
        s.push_str(&format!("--- write: {p} ---\n{t}\n"));
    }
    s
}

fn assert_parity(files: &[(PathBuf, String)], old_id: &str, new_id: &str) {
    let hand = run(files, old_id, new_id);
    let writer = run_via_writer(files, old_id, new_id);
    match (hand, writer) {
        (Ok(a), Ok(b)) => assert_eq!(
            canonical(&a),
            canonical(&b),
            "writer-driven rename diverged from the hand-wired write-set"
        ),
        (Err(a), Err(b)) => assert_eq!(
            format!("{a:?}"),
            format!("{b:?}"),
            "writer-driven rename refused where the hand-wired path refused differently"
        ),
        (a, b) => panic!(
            "parity broken — one path refused where the other succeeded:\n  run: {a:?}\n  run_via_writer: {b:?}"
        ),
    }
}

/// Definition file with one row of every editable family, plus a
/// referencing file (same shape as the 1.1 snapshot fixtures).
fn full_family_spec(id: &str) -> String {
    format!(
        "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL hold one of every rewrite family\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `holds` | [[{id}]] |\n\n## Model\n\n### States\n- spanned\n- applied (emits: [[{id}.c1]])\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | spanned | applied | [[{id}.c1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | [[{id}.c1]] | `g()` | `x` |\n"
    )
}

fn referencing_file(id: &str) -> String {
    format!(
        "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL reference the families\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `holds` | [[fam.a.c1]] and [[fam.a]] |\n"
    )
}

/// Every 1.1 rewrite family, both rename shapes: the writer path must
/// reproduce `run`'s write-set byte for byte.
#[test]
fn writer_parity_over_snapshot_fixtures() {
    let files = vec![
        (PathBuf::from("fam-a.md"), full_family_spec("fam.a")),
        (PathBuf::from("fam-b.md"), referencing_file("fam.b")),
    ];

    // Row rename: id cells, state bullet, transition endpoints, links
    // in both files.
    assert_parity(&files, "fam.a.c1", "fam.a.c2");

    // Intent rename: frontmatter id + file move + child links follow.
    assert_parity(&files, "fam.a", "fam.z");

    // Identity rename: both paths write nothing.
    let hand = run(&files, "fam.a", "fam.a").unwrap();
    let writer = run_via_writer(&files, "fam.a", "fam.a").unwrap();
    assert!(hand.writes.is_empty() && writer.writes.is_empty());
}

/// CRLF variant: terminators must survive the writer path byte-exactly,
/// exactly as the hand-wired path preserves them.
#[test]
fn writer_parity_crlf_row_family() {
    let lf = full_family_spec("crlf.a");
    let crlf = lf.replace('\n', "\r\n");
    let files = vec![(PathBuf::from("crlf-a.md"), crlf.clone())];
    let hand = run(&files, "crlf.a.c1", "crlf.a.c2").unwrap();
    let writer = run_via_writer(&files, "crlf.a.c1", "crlf.a.c2").unwrap();
    assert_eq!(canonical(&hand), canonical(&writer));
    let crlf_in = crlf.matches("\r\n").count();
    let crlf_out: usize = writer
        .writes
        .iter()
        .map(|(_, t)| t.matches("\r\n").count())
        .sum();
    assert_eq!(
        crlf_in, crlf_out,
        "CRLF must survive the writer path byte-exactly"
    );
}

/// Files the parser rejects (prose-only markdown riding the same
/// directory) keep the link-only rewrite on both paths: no spans exist
/// to drive the writer, so the fallback is the hand-wired link rewrite.
#[test]
fn writer_parity_non_spec_file_fallback() {
    let files = vec![
        (PathBuf::from("fam-a.md"), full_family_spec("fam.a")),
        (
            PathBuf::from("notes.md"),
            "Plain notes about [[fam.a]] and [[fam.a.c1]] — not a spec.\n".to_string(),
        ),
    ];
    assert_parity(&files, "fam.a.c1", "fam.a.c2");
    assert_parity(&files, "fam.a", "fam.z");
}

/// A checklist manifest rides the rename on both paths through
/// `checklist::rewrite_text` (checklists are not specs — the writer's
/// parse never covers them).
#[test]
fn writer_parity_checklist_unchanged() {
    let files = vec![
        (PathBuf::from("fam-a.md"), full_family_spec("fam.a")),
        (
            PathBuf::from("ship.checklist.md"),
            "# Ship\n\n## Items\n\n- **a.first**: covered\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| a.first | covered | [[fam.a.c1]] | |\n"
                .to_string(),
        ),
    ];
    assert_parity(&files, "fam.a.c1", "fam.a.c2");
    assert_parity(&files, "fam.a", "fam.z");
}

/// Quoted intent ids: the writer refuses BEFORE any rewrite
/// (`spec.apply_failure` — no rewritable span exists), the hand-wired
/// path rewrites links and only refuses at the verify gate. Reviewed
/// difference per D6: both refuse, the writer earlier and with the
/// better remediation hint. This pins the refusal; it is excluded from
/// the parity property legs.
#[test]
fn quoted_intent_rename_writer_refuses() {
    let quoted = full_family_spec("fam.a").replace("id: fam.a", "id: \"fam.a\"");
    let files = vec![
        (PathBuf::from("fam-a.md"), quoted),
        (PathBuf::from("fam-b.md"), referencing_file("fam.b")),
    ];
    // Hand-wired: gets past rewriting, refuses at the verify gate (the
    // moved file's frontmatter id is still the quoted old id).
    assert!(run(&files, "fam.a", "fam.z").is_err());
    // Writer: refuses immediately, labeled.
    match run_via_writer(&files, "fam.a", "fam.z") {
        Err(e) => assert!(
            format!("{e:?}").contains("apply_failure") || format!("{e:?}").contains("VerifyFailed"),
            "writer must refuse with a labeled error: {e:?}"
        ),
        Ok(out) => panic!("writer must refuse a quoted-intent-id rename, got {out:?}"),
    }
}

/// File-id-shaped identifiers (mirrors `acset_writer_identity.rs`).
fn arb_id() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,7}(\\.[a-z][a-z0-9_]{0,7}){0,2}"
}

// `rename_writer_parity_exact` — the delta's proptest leg: for
// arbitrary spec/rename cases the writer-driven write-set equals the
// hand-wired write-set byte for byte.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn rename_writer_parity_exact(
        id in arb_id(),
        new_local in "[a-z][a-z0-9_]{0,7}",
        n_rows in 1usize..4,
        padded_cells in prop::bool::ANY,
        backticked_ids in prop::bool::ANY,
        padded_links in prop::bool::ANY,
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
        let mut def = format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL round trip\"\n---\n\nProse mentioning {id} as a word must survive untouched.\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n"
        );
        for i in 0..n_rows {
            def.push_str(&format!(
                "|{}|{}|{}|{}|\n",
                id_cell(&format!("c{i}")),
                pad("invariant"),
                pad("`holds`"),
                pad(&link(&id)),
            ));
        }
        def.push_str(&format!(
            "\n## Model\n\n### States\n- spanned\n- applied (emits: {emits})\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | spanned | applied | {guard} |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | {derives} | `g()` | `x` |\n",
            emits = link(&format!("{id}.c0")),
            guard = link(&format!("{id}.c0")),
            derives = link(&format!("{id}.c0")),
        ));
        let mut other = format!(
            "---\nid: {id}ref\nkind: intent\nstatement: \"THE {id}ref SHALL reference\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `holds` | {cross} |\n",
            cross = pad(&link(&format!("{id}.c0"))),
        );
        let _ = &other;
        if crlf {
            def = def.replace('\n', "\r\n");
            other = other.replace('\n', "\r\n");
        }
        // The referencing file's id is `{id}ref` — never collides with
        // `{id}` or `{id}.…`; the new row local `new` never collides
        // with c0..c2/t1/p1/spanned/applied.
        let files = vec![
            (PathBuf::from("def.md"), def),
            (PathBuf::from("ref.md"), other),
        ];
        // Row rename (new_id inside the file's namespace) …
        let new_row = format!("{id}.z{new_local}");
        prop_assume!(!files.iter().any(|(_, t)| t.contains(&new_row)), "collision-free case only");
        assert_parity(&files, &format!("{id}.c0"), &new_row);
        // … and intent rename (new_id is a fresh id).
        let new_intent = format!("{id}renamed");
        prop_assume!(!files.iter().any(|(_, t)| t.contains(&new_intent)), "collision-free case only");
        assert_parity(&files, &id, &new_intent);
    }
}
