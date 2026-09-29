//! Integration tests for `spk merge` (specs/merge.md, beads specodelic-7oq).
//!
//! METER from the ticket: tempdir spec trees per fixture, `spk merge
//! --json`, findings asserted on the envelope. The three MUSTs: an id
//! collision between branches is flagged; a rename left dangling by a
//! reference minted on the other branch is flagged; a clean merge
//! reports no findings.

use assert_cmd::Command;
use std::path::{Path, PathBuf};

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap()
}

/// A minimal lint-clean spec: frontmatter + Constraints + Model + a
/// deriving Property (the same shape as the linter's clean fixture).
fn write_spec(path: &Path, id: &str, statement: &str) {
    std::fs::write(path, clean_spec(id, statement)).unwrap();
}

/// The lint-clean fixture body for id `id`; `extra_constraint` optionally
/// appends a second constraint row (its own deriving property included).
fn clean_spec(id: &str, statement: &str) -> String {
    format!(
        "---\nid: {id}\nkind: intent\nstatement: \"{statement}\"\n---\n\n\
         ## Constraints\n\n\
         | id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | c1 | invariant | `holds` | [[{id}]] |\n\
         \n## Model\n\n\
         ### States\n\n- s1\n- s2\n\n\
         ### Transitions\n\n\
         | id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | t | s1 | s2 | [[{id}.c1]] |\n\
         \n## Properties\n\n\
         | id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|------------|\n\
         | p | unit | [[{id}.c1]] | `g()` | `x` |\n"
    )
}

/// A lint-clean spec that additionally references `target` from a
/// constraint's traces_to cell (structured cells reach the graph;
/// prose never does).
fn write_referring_spec(path: &Path, id: &str, target: &str) {
    let mut body = clean_spec(
        id,
        &format!("THE system SHALL reference [[{target}]] from {id}"),
    );
    let anchor = format!("| c1 | invariant | `holds` | [[{id}]] |\n");
    body = body.replacen(
        &anchor,
        &format!("{anchor}| c2 | invariant | `reaches` | [[{target}]] |\n"),
        1,
    );
    body.push_str(&format!("| p2 | unit | [[{id}.c2]] | `g2()` | `y` |\n"));
    std::fs::write(path, body).unwrap();
}

/// Base/A/B trees: `shared_ids` are written identically into all three;
/// the caller then adds branch-specific files.
fn setup(dir: &Path, shared_ids: &[&str]) -> (PathBuf, PathBuf, PathBuf) {
    let mk = |name: &str| {
        let d = dir.join(name);
        std::fs::create_dir_all(&d).unwrap();
        d
    };
    let (base, a, b) = (mk("base"), mk("a"), mk("b"));
    for id in shared_ids {
        for t in [&base, &a, &b] {
            write_spec(
                &t.join(format!("{id}.md")),
                id,
                "THE ancestor SHALL define the shared id",
            );
        }
    }
    (base, a, b)
}

fn run_merge(base: &Path, a: &Path, b: &Path) -> (String, Option<i32>) {
    let out = spk()
        .args([
            "merge",
            "--branch",
            b.to_str().unwrap(),
            "--base",
            base.to_str().unwrap(),
            a.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    (String::from_utf8(out.stdout).unwrap(), out.status.code())
}

/// MUST 1: both branches independently mint the same new id → flagged.
#[test]
fn id_collision_across_branches_is_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let (base, a, b) = setup(tmp.path(), &["shared"]);
    write_spec(&a.join("collide.md"), "collide", "A's body");
    write_spec(&b.join("collide.md"), "collide", "B's different body");

    let (stdout, code) = run_merge(&base, &a, &b);
    assert!(
        stdout.contains("id_collision"),
        "collision flagged: {stdout}"
    );
    assert_eq!(code, Some(1), "findings must fail the merge");
}

/// Property `inherited_id_not_falsely_flagged` + MUST 3: an id present
/// identically in the ancestor is NOT a collision; a clean merge exits 0.
#[test]
fn clean_merge_reports_no_findings() {
    let tmp = tempfile::tempdir().unwrap();
    let (base, a, b) = setup(tmp.path(), &["shared"]);
    write_spec(
        &a.join("only_a.md"),
        "only_a",
        "THE A side SHALL add its own spec",
    );
    write_spec(
        &b.join("only_b.md"),
        "only_b",
        "THE B side SHALL add its own spec",
    );

    let (stdout, code) = run_merge(&base, &a, &b);
    assert!(
        !stdout.contains("id_collision") && !stdout.contains("blast_radius_intersection"),
        "clean merge must not flag: {stdout}"
    );
    assert_eq!(code, Some(0), "clean merge exits 0: {stdout}");
}

/// MUST 2: A renames an id away; B mints a fresh reference to the old
/// name → flagged for rename replay.
#[test]
fn dangling_rename_across_branches_is_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let mk = |name: &str| {
        let d = tmp.path().join(name);
        std::fs::create_dir_all(&d).unwrap();
        d
    };
    let (base, a, b) = (mk("base"), mk("a"), mk("b"));
    // Ancestor defines `old_spec`; A renames it away (old file gone,
    // new_spec.md defines new_spec); B keeps old_spec and mints a fresh
    // reference to [[old_spec]] in a file A never touched.
    write_spec(
        &base.join("old_spec.md"),
        "old_spec",
        "THE ancestor SHALL define old_spec",
    );
    write_spec(
        &a.join("new_spec.md"),
        "new_spec",
        "THE rename SHALL define new_spec",
    );
    std::fs::copy(base.join("old_spec.md"), b.join("old_spec.md")).unwrap();
    write_referring_spec(&b.join("referrer.md"), "referrer", "old_spec");

    let (stdout, code) = run_merge(&base, &a, &b);
    assert!(stdout.contains("rename_replay"), "flagged: {stdout}");
    assert_eq!(code, Some(1));
}

/// Property `textually_clean_semantic_conflict_flagged`: A renames a row
/// of `m` (only m.md changes); B adds a NEW file referencing `m.c1` —
/// the branches never touch a common line, but the blast radii
/// intersect → needs_review.
#[test]
fn textually_clean_semantic_conflict_is_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let mk = |name: &str| {
        let d = tmp.path().join(name);
        std::fs::create_dir_all(&d).unwrap();
        d
    };
    let (base, a, b) = (mk("base"), mk("a"), mk("b"));

    // Ancestor: m.md defines m with row c1.
    let m = "---\nid: m\nkind: intent\nstatement: \"THE m SHALL have a row\"\n---\n\n\
        ## Constraints\n\n\
        | id | kind | expr | traces_to |\n\
        |----|------|------|-----------|\n\
        | c1 | invariant | `holds` | [[m]] |\n";
    std::fs::write(base.join("m.md"), m_with_row("c1", m)).unwrap();
    // Branch A renamed the row in place: c1 -> c1renamed.
    std::fs::write(
        a.join("m.md"),
        m_with_row("c1renamed", &m_with_row("c1", m)),
    )
    .unwrap();
    // Branch B keeps m.md untouched and mints a new ref to m.c1.
    std::fs::write(b.join("m.md"), m).unwrap();
    write_referring_spec(&b.join("referrer.md"), "referrer", "m.c1");
    let _ = (&base, &a, &b);

    let (stdout, code) = run_merge(&base, &a, &b);
    assert!(
        stdout.contains("blast_radius_intersection") || stdout.contains("rename_replay"),
        "semantic conflict flagged: {stdout}"
    );
    assert_eq!(code, Some(1));
}

/// Modify/delete: branch A edits the file, branch B deletes it. Git
/// marks the conflict; the check must never report a false clean
/// `merged` (Ro5 round over bc39f9d..main — the union used to keep A's
/// edited copy silently and relint the wrong tree).
#[test]
fn modify_delete_edited_in_a_deleted_in_b_is_flagged() {
    let dir = tempfile::tempdir().unwrap();
    let (base, a, b) = setup(dir.path(), &["x", "y"]);
    std::fs::write(
        a.join("x.md"),
        clean_spec("x", "THE system SHALL hold edited"),
    )
    .unwrap();
    std::fs::remove_file(b.join("x.md")).unwrap(); // B keeps only y.md

    let (stdout, code) = run_merge(&base, &a, &b);
    assert!(
        stdout.contains("textual_conflict"),
        "modify/delete conflict flagged: {stdout}"
    );
    assert!(
        stdout.contains("deleted on branch B"),
        "conflict names the deletion side: {stdout}"
    );
    assert_eq!(code, Some(1));
}

/// The mirror case: branch B edits, branch A deletes.
#[test]
fn modify_delete_edited_in_b_deleted_in_a_is_flagged() {
    let dir = tempfile::tempdir().unwrap();
    let (base, a, b) = setup(dir.path(), &["x", "y"]);
    std::fs::remove_file(a.join("x.md")).unwrap(); // A keeps only y.md
    std::fs::write(
        b.join("x.md"),
        clean_spec("x", "THE system SHALL hold edited"),
    )
    .unwrap();

    let (stdout, code) = run_merge(&base, &a, &b);
    assert!(
        stdout.contains("textual_conflict") && stdout.contains("deleted on branch A"),
        "modify/delete conflict flagged symmetrically: {stdout}"
    );
    assert_eq!(code, Some(1));
}

/// Merging without `--base` treats every id defined on both branches as
/// independently minted — the envelope must say so explicitly (a
/// warning), or the collision flood reads as a verdict about the
/// branches instead of about the missing ancestor.
#[test]
fn merge_without_base_warns_about_missing_ancestor() {
    let dir = tempfile::tempdir().unwrap();
    let (base, a, b) = setup(dir.path(), &["x"]);
    let _ = base;
    let out = spk()
        .args([
            "merge",
            "--branch",
            b.to_str().unwrap(),
            a.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("--base") && stdout.contains("ancestor"),
        "no-base merge explains the missing ancestor: {stdout}"
    );
}

/// The ancestor `m.md` body, with the constraint row id swapped for
/// `row_id` (the `original` param supplies the surrounding doc shape).
fn m_with_row(row_id: &str, _original: &str) -> String {
    format!(
        "---\nid: m\nkind: intent\nstatement: \"THE m SHALL be a merge fixture\"\n---\n\n\
         ## Constraints\n\n\
         | id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | {row_id} | invariant | `holds` | [[m]] |\n"
    )
}
