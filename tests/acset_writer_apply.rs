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
