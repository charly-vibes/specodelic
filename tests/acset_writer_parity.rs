//! Baseline rename write-set snapshots (`add-acset-writer` task 1.1).
//!
//! The writer (`acset-writer` capability, slice S3 of the acset epic)
//! will take over the byte-level emission rename performs today. These
//! snapshots pin rename.rs's write-set bytes **before** any parser or
//! writer change exists — the parity oracle every later phase must
//! reproduce byte for byte (per rewrite family), per design decision
//! D6. Fixtures are in-memory (same shape as the rename unit tests)
//! and cover every rewrite family:
//!
//! - exact id cells in Constraints / Properties / Transition tables
//! - wiki-link rewrites in prose and in cross-file references
//! - state bullets (`- old_id (emits: …)`)
//! - a file's own Intent-id rename (write-set carries the new path,
//!   one removal for the old path)
//! - CRLF variants of the table fixture (explicit preservation
//!   assertions — the snapshot compare is line-ending-agnostic, which
//!   alone could NOT catch a CRLF→LF flip; see task 4.1)

use std::path::PathBuf;

use specodelic::rename;

/// Canonical byte form of a rename outcome — the snapshot oracle.
/// Writes sorted by path display string; no hash-map iteration.
fn oracle_bytes(out: &rename::RenameOutcome) -> String {
    let mut writes: Vec<(String, &str)> = out
        .writes
        .iter()
        .map(|(p, t)| (p.display().to_string(), t.as_str()))
        .collect();
    writes.sort();
    let mut s = String::new();
    s.push_str(&format!("old: {}\nnew: {}\n", out.old_id, out.new_id));
    match &out.remove {
        Some(r) => s.push_str(&format!("remove: {}\n", r.display())),
        None => s.push_str("remove: none\n"),
    }
    for (p, t) in &writes {
        s.push_str(&format!("--- write: {p} ---\n{t}\n"));
    }
    s
}

/// CRLF is invisible to the agnostic snapshot compare — pin it
/// explicitly: every `\r\n` in the fixture's write-set must survive
/// byte-exactly (same line terminators in and out).
fn assert_crlf_preserved(fixture: &str, out: &rename::RenameOutcome) {
    let crlf_in = fixture.matches("\r\n").count();
    assert!(crlf_in > 0, "fixture must actually contain CRLF");
    let crlf_out: usize = out
        .writes
        .iter()
        .map(|(_, t)| t.matches("\r\n").count())
        .sum();
    assert_eq!(
        crlf_in, crlf_out,
        "CRLF terminators must survive the rename byte-exactly"
    );
}

/// Definition file with one row of each editable family: a Constraints
/// id cell + trace link, a Transition from/to cell pair, a State
/// bullet, and a Properties derives_from cell (plain + link form).
fn full_family_spec(id: &str) -> String {
    format!(
        "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL hold one of every rewrite family\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `holds` | [[{id}]] |\n\n## Model\n\n### States\n- spanned\n- applied (emits: [[{id}.c1]])\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | spanned | applied | [[{id}.c1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | [[{id}.c1]] | `g()` | `x` |\n"
    )
}

const CRATE: &str = env!("CARGO_MANIFEST_DIR");

/// Snapshot every rewrite family's write-set (LF fixtures).
#[test]
fn baseline_write_set_snapshots() {
    // Family 1+2+4: row rename in the definition file (c1 cell, state
    // bullet, transition from/to), links follow in a second file.
    let def = full_family_spec("fam.a");
    let other = "---\nid: fam.b\nkind: intent\nstatement: \"THE fam.b SHALL reference fam.a\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `holds` | [[fam.a.c1]] and [[fam.a]] |\n";
    let files = vec![
        (PathBuf::from("fam-a.md"), def.clone()),
        (PathBuf::from("fam-b.md"), other.to_string()),
    ];
    let out = rename::run(&files, "fam.a.c1", "fam.a.c2").expect("row rename succeeds");
    oracle_snapshot(CRATE, "rename_row_family", &oracle_bytes(&out));

    // Family 5: the file's own Intent id — write-set carries the new
    // path, one removal for the old path.
    let out = rename::run(&files, "fam.a", "fam.z").expect("intent rename succeeds");
    oracle_snapshot(CRATE, "rename_intent_family", &oracle_bytes(&out));

    // No-op guard: identity rename writes nothing (rename_naturality).
    let out = rename::run(&files, "fam.a", "fam.a").expect("identity rename succeeds");
    assert!(out.writes.is_empty() && out.remove.is_none());
}

/// CRLF variant of the row-rename family: the snapshot pins the
/// *shape*, the explicit assertion pins the terminators.
#[test]
fn baseline_crlf_row_family() {
    let lf = full_family_spec("crlf.a");
    let crlf = lf.replace('\n', "\r\n");
    let files = vec![(PathBuf::from("crlf-a.md"), crlf.clone())];
    let out = rename::run(&files, "crlf.a.c1", "crlf.a.c2").expect("rename succeeds");
    oracle_snapshot(CRATE, "rename_crlf_row_family", &oracle_bytes(&out));
    assert_crlf_preserved(&crlf, &out);
}

/// Committed snapshot store: tests/snapshots/rename_baseline/<name>.txt,
/// compared line-ending-agnostically (Windows runners check snapshots
/// out with CRLF — same convention as tests/acset_parity.rs).
fn oracle_snapshot(crate_dir: &str, name: &str, bytes: &str) {
    let dir = std::path::Path::new(crate_dir).join("tests/snapshots/rename_baseline");
    std::fs::create_dir_all(&dir).expect("snapshot dir");
    let path = dir.join(format!("{name}.txt"));
    if std::env::var("RENAME_SNAPSHOT_UPDATE").is_ok() {
        // Store normalized: the compare is line-ending-agnostic, and CRLF
        // preservation is pinned by the explicit assertion, not here.
        std::fs::write(&path, bytes.replace("\r\n", "\n")).expect("write snapshot");
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "snapshot {} missing ({e}) — run with RENAME_SNAPSHOT_UPDATE=1 to pin",
            path.display()
        )
    });
    let expected = expected.replace("\r\n", "\n");
    let bytes = bytes.replace("\r\n", "\n");
    assert!(
        bytes == expected,
        "snapshot drift for {name}:\n--- got ---\n{bytes}\n--- expected ---\n{expected}"
    );
}
