//! Edit application through recorded spans (`add-acset-writer` task 4.x).
//!
//! `apply(f, x)` realizes an instance edit as text: every change flows
//! through a recorded byte span — never a re-serialized table — and the
//! result is a write-set of `(path, full new contents)` plus optional
//! removals, with no I/O (applying it is the caller's single transaction).
//! The edit shape mirrors `rename.rs`'s per-file rewrite contract exactly
//! (qualified link targets everywhere; local cell/bullet rewrites only in
//! the definition file), so phase 5 can migrate one rewrite family at a
//! time behind the 1.1 parity snapshots.
//!
//! Fixture families (delta Properties):
//! - `crlf_survives_edit` — terminators byte-exact through an edit
//! - `prose_survives_edit` — prose mentioning the old id as a word survives
//! - `padding_not_realigned` — padding preserved, column may misalign
//! - `intent_rename_moves_file` — new path per `-` ⇔ `.`, one removal
//! - `writer_does_no_io` — apply succeeds on a path that cannot exist
//! - `text_realizes_instance` — `from_specs(apply(f, x))` equals the edit
//! - `apply_failure_label_asserted` — edits no recorded span can realize

use std::path::{Path, PathBuf};

use specodelic::acset::writer::{self, Edit};
use specodelic::spec::parse_str;

/// Definition file with one row of every editable family (same shape as
/// the 1.1 parity fixtures).
fn full_family_spec(id: &str) -> String {
    format!(
        "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL hold one of every rewrite family\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `holds` | [[{id}]] |\n\n## Model\n\n### States\n- spanned\n- applied (emits: [[{id}.c1]])\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | spanned | applied | [[{id}.c1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | [[{id}.c1]] | `g()` | `x` |\n"
    )
}

fn row_rename(old_local: &str, new_local: &str, intent: &str) -> Edit {
    Edit::Rename {
        old: format!("{intent}.{old_local}"),
        new: format!("{intent}.{new_local}"),
        local: Some((old_local.into(), new_local.into())),
    }
}

fn intent_rename(old: &str, new: &str) -> Edit {
    Edit::Rename {
        old: old.into(),
        new: new.into(),
        local: None,
    }
}

fn parse_ok(source: &str) -> specodelic::spec::Spec {
    parse_str(source).expect("fixture parses")
}

/// Row rename: the id cell, the state-bullet emits link, the transition
/// guard link, and the derives_from link follow; nothing else changes.
#[test]
fn row_rename_rewrites_cells_and_links_through_spans() {
    let source = full_family_spec("fam.a");
    let spec = parse_ok(&source);
    let out = writer::apply(
        &row_rename("c1", "c2", "fam.a"),
        Path::new("fam-a.md"),
        &source,
        &spec,
    )
    .expect("row rename applies");
    assert!(out.removals.is_empty(), "row rename never moves a file");
    assert_eq!(out.writes.len(), 1, "one write: the edited file");
    assert_eq!(out.writes[0].0, PathBuf::from("fam-a.md"));
    assert_eq!(
        out.writes[0].1,
        full_family_spec("fam.a").replace("c1", "c2"),
        "every c1 occurrence (cell head, links) follows; nothing else moves"
    );
}

/// A bare (non-link) cross-column cell carrying the local id — the
/// `derives_from` cell rename's any-cell rule covers — is rewritten
/// through its recorded cell span, padding untouched.
#[test]
fn bare_derives_from_cell_follows_local_rename() {
    let source = format!(
        "{}\n| p2 | unit | c1 | `g()` | `x` |\n",
        full_family_spec("fam.a")
    );
    let spec = parse_ok(&source);
    let out = writer::apply(
        &row_rename("c1", "c2", "fam.a"),
        Path::new("fam-a.md"),
        &source,
        &spec,
    )
    .expect("row rename applies");
    assert_eq!(out.writes.len(), 1);
    assert!(
        out.writes[0].1.contains("| p2 | unit | c2 | `g()` | `x` |"),
        "the bare derives_from cell follows the rename: {}",
        out.writes[0].1
    );
}

/// CRLF survives an edit byte-exactly — the agnostic snapshot compares
/// could never catch a flip, so pin the terminators explicitly.
#[test]
fn crlf_survives_edit() {
    let lf = full_family_spec("crlf.a");
    let source = lf.replace('\n', "\r\n");
    let spec = parse_ok(&source);
    let out = writer::apply(
        &row_rename("c1", "c2", "crlf.a"),
        Path::new("crlf-a.md"),
        &source,
        &spec,
    )
    .expect("row rename applies");
    assert_eq!(out.writes.len(), 1);
    let emitted = &out.writes[0].1;
    assert_eq!(
        emitted.matches("\r\n").count(),
        source.matches("\r\n").count(),
        "every CRLF terminator survives"
    );
    assert!(
        !emitted.replace("\r\n", "").contains('\n'),
        "no bare LF may appear: {emitted:?}"
    );
    assert_eq!(
        emitted.as_str(),
        lf.replace("c1", "c2").replace('\n', "\r\n"),
        "the CRLF edit is exactly the LF edit with terminators preserved"
    );
}

/// Prose mentioning the old id as a word survives byte for byte — only
/// recorded spans (cells, links, bullets, the intent id) are edited.
#[test]
fn prose_survives_edit() {
    let prose_line = "Prose may say c1 and fam.a as plain words — untouched.\n";
    let source = full_family_spec("fam.a")
        .replace("## Constraints", &format!("{prose_line}\n## Constraints"));
    let spec = parse_ok(&source);
    for edit in [
        row_rename("c1", "c2", "fam.a"),
        intent_rename("fam.a", "fam.z"),
    ] {
        let out =
            writer::apply(&edit, Path::new("fam-a.md"), &source, &spec).expect("edit applies");
        let (_, text) = &out.writes[0];
        assert!(
            text.contains(prose_line),
            "prose must survive {edit:?}: {text}"
        );
    }
}

/// Padding is not realigned: the writer replaces only the cell content's
/// span, so surrounding padding bytes are preserved verbatim and the
/// column may become unaligned (width_padding_policy — realignment is a
/// separate explicit operation, never an edit side effect).
#[test]
fn padding_not_realigned() {
    let source = full_family_spec("fam.a").replace("| c1 | invariant |", "| c1   | invariant |");
    let spec = parse_ok(&source);
    let out = writer::apply(
        &row_rename("c1", "c1_renamed_long", "fam.a"),
        Path::new("fam-a.md"),
        &source,
        &spec,
    )
    .expect("row rename applies");
    assert_eq!(out.writes.len(), 1);
    assert!(
        out.writes[0]
            .1
            .contains("| c1_renamed_long   | invariant |"),
        "padding preserved verbatim, not realigned: {}",
        out.writes[0].1
    );
}

/// An edit to a file's own Intent id maps the file to the filename the
/// new id implies (`-` ⇔ `.`) and returns exactly one removal.
#[test]
fn intent_rename_moves_file() {
    let source = full_family_spec("fam.a");
    let spec = parse_ok(&source);
    let out = writer::apply(
        &intent_rename("fam.a", "fam.z"),
        Path::new("some/dir/fam-a.md"),
        &source,
        &spec,
    )
    .expect("intent rename applies");
    assert_eq!(
        out.removals,
        vec![PathBuf::from("some/dir/fam-a.md")],
        "exactly one removal: the old path"
    );
    assert_eq!(out.writes.len(), 1);
    assert_eq!(
        out.writes[0].0,
        PathBuf::from("some/dir/fam-z.md"),
        "the write lands on the mapped filename"
    );
    // Frontmatter id and link targets follow; the statement's prose
    // mention of the old id is untouched (prose_untouched_by_rename).
    assert_eq!(
        out.writes[0].1,
        full_family_spec("fam.a")
            .replace("id: fam.a", "id: fam.z")
            .replace("[[fam.a", "[[fam.z")
    );
}

/// No matching span, no change, no write — a file that does not carry the
/// edited id is a no-op (rename calls apply on every corpus file).
#[test]
fn edit_missing_from_file_is_a_no_op() {
    let source = full_family_spec("fam.b");
    let spec = parse_ok(&source);
    let out = writer::apply(
        &Edit::Rename {
            old: "fam.a.c1".into(),
            new: "fam.a.c2".into(),
            local: None,
        },
        Path::new("fam-b.md"),
        &source,
        &spec,
    )
    .expect("unrelated edit applies");
    assert!(out.writes.is_empty() && out.removals.is_empty());
}

/// The writer performs no I/O: apply succeeds with a path that cannot
/// exist — the write-set is data; applying it is the caller's transaction.
#[test]
fn writer_does_no_io() {
    let source = full_family_spec("fam.a");
    let spec = parse_ok(&source);
    let nowhere = Path::new("/nonexistent/dir/fam-a.md");
    let out = writer::apply(&intent_rename("fam.a", "fam.z"), nowhere, &source, &spec)
        .expect("apply must not touch the filesystem");
    assert_eq!(out.writes[0].0, PathBuf::from("/nonexistent/dir/fam-z.md"));
}

/// edit_application_faithful: parsing the emitted text yields exactly the
/// renamed instance — ids, links, and the moved filename all agree.
#[test]
fn text_realizes_instance() {
    // Row rename: the instance's row id and every qualified link follow.
    let source = full_family_spec("fam.a");
    let spec = parse_ok(&source);
    let out = writer::apply(
        &row_rename("c1", "c2", "fam.a"),
        Path::new("fam-a.md"),
        &source,
        &spec,
    )
    .expect("applies");
    let renamed = parse_ok(&out.writes[0].1);
    assert_eq!(renamed.constraints[0].id, "c2");
    assert!(
        renamed.links.iter().any(|l| l.target == "fam.a.c2"),
        "qualified links realize the new id"
    );
    assert!(
        !renamed.links.iter().any(|l| l.target.contains("c1")),
        "no stale targets survive"
    );

    // Intent rename: the parsed intent id IS the new id.
    let out = writer::apply(
        &intent_rename("fam.a", "fam.z"),
        Path::new("fam-a.md"),
        &source,
        &spec,
    )
    .expect("applies");
    let renamed = parse_ok(&out.writes[0].1);
    assert_eq!(renamed.intent.id, "fam.z");
    assert!(
        renamed.links.iter().any(|l| l.target == "fam.z.c1"),
        "child references follow the parent's rename"
    );
}

/// apply_failure: an edit the recorded spans cannot realize — a quoted
/// frontmatter id has no recordable span — is a labeled error with a
/// remediation hint, never a silent partial rewrite.
#[test]
fn apply_failure_label_asserted_on_unspanned_intent_id() {
    let source = full_family_spec("fam.a").replace("id: fam.a", "id: \"fam.a\"");
    let spec = parse_ok(&source);
    let err = writer::apply(
        &intent_rename("fam.a", "fam.z"),
        Path::new("fam-a.md"),
        &source,
        &spec,
    )
    .expect_err("an unspanned intent id cannot realize the edit");
    assert_eq!(err.label, "spec.apply_failure");
    assert!(
        !err.remediation.is_empty(),
        "the fleet error contract requires a non-empty remediation hint"
    );
}

/// apply_failure, second shape: a state bullet the span recorder could not
/// span (unterminated backtick head) but whose id the edit names.
#[test]
fn apply_failure_label_asserted_on_unspanned_bullet() {
    let source = full_family_spec("fam.a").replace("- spanned\n", "- `unclosed\n- spanned\n");
    let spec = parse_ok(&source);
    let err = writer::apply(
        &row_rename("unclosed", "closed", "fam.a"),
        Path::new("fam-a.md"),
        &source,
        &spec,
    )
    .expect_err("an unspanned bullet cannot realize the edit");
    assert_eq!(err.label, "spec.apply_failure");
    assert!(!err.remediation.is_empty());
}

// ---------------------------------------------------------------------------
// Task 4.3: the writer edit laws — identity, composition, and roundtrip
// faithfulness (`edit_application_faithful`, `writer_edit_law`), plus the
// `spec.roundtrip_failure` label for emissions that reparse to an instance
// other than the edit's target.
// ---------------------------------------------------------------------------

use proptest::prelude::*;

/// rename_naturality at the writer: apply(id, x) == x — an edit whose
/// replacement equals its target writes nothing and removes nothing.
#[test]
fn identity_edit_is_a_no_op() {
    let source = full_family_spec("fam.a");
    let spec = parse_ok(&source);
    let out = writer::apply(
        &Edit::Rename {
            old: "fam.a.c1".into(),
            new: "fam.a.c1".into(),
            local: Some(("c1".into(), "c1".into())),
        },
        Path::new("fam-a.md"),
        &source,
        &spec,
    )
    .expect("identity edit applies");
    assert!(out.writes.is_empty() && out.removals.is_empty());
}

/// writer_edit_law's composition: apply(g, apply(f, x)) == apply(compose(g,
/// f), x) for chained renames whose intermediate id does not pre-exist
/// (the precondition rename.rs's collision gate guarantees). The composed
/// edit chains the qualified ids and the local ids.
#[test]
fn edits_compose() {
    let source = full_family_spec("fam.a");
    let spec = parse_ok(&source);
    let f = row_rename("c1", "c2", "fam.a");
    let g = row_rename("c2", "c3", "fam.a");
    let composed = row_rename("c1", "c3", "fam.a");
    let step1 = writer::apply(&f, Path::new("fam-a.md"), &source, &spec).expect("f applies");
    let (p1, t1) = &step1.writes[0];
    let mid_spec = parse_ok(t1);
    let step2 = writer::apply(&g, p1, t1, &mid_spec).expect("g applies");
    let (p2, t2) = &step2.writes[0];
    let one_shot =
        writer::apply(&composed, Path::new("fam-a.md"), &source, &spec).expect("composed applies");
    assert_eq!(one_shot.writes[0].0, *p2, "the same file is written");
    assert_eq!(
        one_shot.writes[0].1, *t2,
        "one step == two steps, byte for byte"
    );
}

/// roundtrip_failure: a replacement id that passes shape validation but
/// corrupts a bullet head (`` `x `` reparses as head token `x`, dropping
/// the backtick) makes the emitted text reparse to an instance other than
/// the edit's target — the writer refuses with the labeled error instead
/// of emitting it.
#[test]
fn roundtrip_failure_label_asserted() {
    let source = full_family_spec("fam.a");
    let spec = parse_ok(&source);
    let err = writer::apply(
        &row_rename("spanned", "`x", "fam.a"),
        Path::new("fam-a.md"),
        &source,
        &spec,
    )
    .expect_err("a corrupting replacement must not be emitted");
    assert_eq!(err.label, "spec.roundtrip_failure");
    assert!(
        !err.remediation.is_empty(),
        "the fleet error contract requires a non-empty remediation hint"
    );
}

// edit_application_faithful as the delta's proptest: for generated files
// and an arbitrary (valid) local rename, the emitted text reparses to
// exactly the renamed instance — the picked row's id renamed everywhere
// it appears (cell, links), prose and padding untouched, and no stale
// reference to the old id surviving.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn roundtrip_faithful(
        n_rows in 1usize..4,
        padded in prop::bool::ANY,
        crlf in prop::bool::ANY,
        pick in 0usize..3,
    ) {
        let id = "fam.a";
        let pick = pick % n_rows;
        let old_local = format!("r{pick}");
        let new_local = format!("renamed{pick}");
        let cell = |c: &str| {
            if padded { format!(" {c}  ") } else { format!(" {c} ") }
        };
        let mut file = format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL round trip\"\n---\n\nProse may say r0 as a word — untouched.\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n"
        );
        for i in 0..n_rows {
            file.push_str(&format!(
                "|{}|{}|{}|{}|\n",
                cell(&format!("r{i}")),
                cell("invariant"),
                cell("`holds`"),
                cell(&format!("[[{id}.r{i}]]")),
            ));
        }
        file.push_str(&format!(
            "\n## Model\n\n### States\n- st{i} (emits: [[{id}.r0]])\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | st0 | st1 | [[{id}.r{pick}]] |\n",
            i = pick,
            pick = pick,
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
        let edit = Edit::Rename {
            old: format!("{id}.{old_local}"),
            new: format!("{id}.{new_local}"),
            local: Some((old_local.clone(), new_local.clone())),
        };
        let out = writer::apply(&edit, Path::new("fam-a.md"), &file, &spec)
            .map_err(|e| TestCaseError::fail(format!("apply failed: {e}")))?;
        let (_, text) = &out.writes[0];
        let reparsed = parse_str(text)
            .map_err(|e| TestCaseError::fail(format!("emitted text does not reparse: {e}")))?;

        // The picked row's id is the new one; every other row keeps its id.
        for (i, r) in reparsed.constraints.iter().enumerate() {
            let expected: &str = if i == pick { &new_local } else { &format!("r{i}") };
            prop_assert_eq!(&r.id, expected);
        }
        // No stale qualified reference to the old id survives.
        prop_assert!(
            reparsed
                .links
                .iter()
                .all(|l| l.target != format!("{id}.{old_local}")),
            "stale link target after rename"
        );
        // The renamed row's own trace link follows.
        let want = format!("[[{id}.{new_local}]]");
        prop_assert!(
            reparsed
                .constraints
                .iter()
                .any(|r| r.cells.get("traces_to").is_some_and(|c| c.contains(&want)))
        );
        // Prose is untouched, byte for byte — including its mention of
        // r0 as a plain word.
        let prose = if crlf {
            "Prose may say r0 as a word — untouched.".to_string()
        } else {
            "Prose may say r0 as a word — untouched.\n".to_string()
        };
        prop_assert!(text.contains(&prose), "prose must survive: {text}");
    }
}
