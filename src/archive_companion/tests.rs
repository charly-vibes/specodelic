//! Tests for the archive-companion module (tempdirs, no openspec
//! process ever spawned — the runner seam, design D2/D7).

use std::fs;
use std::path::Path;

use super::*;

/// Build a repo-root-shaped temp tree from relative path → content.
fn tree(entries: &[(&str, &str)]) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    for (rel, content) in entries {
        let p = root.path().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    }
    root
}

fn tree_owned(entries: &[(String, &str)]) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    for (rel, content) in entries {
        let p = root.path().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    }
    root
}

const DUAL_DELTA: &str = "---\nid: spec\nkind: intent\nstatement: \"SHALL x.\"\n---\n\n# c\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n";

/// An archive dir with one dual-format delta for capability `c`.
fn archived_fixtures(id: &str) -> Vec<(String, &'static str)> {
    vec![(
        format!("openspec/changes/archive/2026-10-01-{id}/specs/c/spec.md"),
        DUAL_DELTA,
    )]
}

fn noop_runner() -> impl FnMut(&Path, &str) -> Result<(), String> {
    |_root, _id| Ok(())
}

fn panicking_runner() -> impl FnMut(&Path, &str) -> Result<(), String> {
    |_root, _id| panic!("openspec must not be invoked in this scenario")
}

#[test]
fn resolve_archive_dir_newest_wins() {
    let root = tree(&[
        (
            "openspec/changes/archive/2026-09-29-x/specs/c/spec.md",
            DUAL_DELTA,
        ),
        (
            "openspec/changes/archive/2026-10-01-x/specs/c/spec.md",
            DUAL_DELTA,
        ),
    ]);
    let dir = resolve_archive_dir(root.path(), "x").unwrap();
    assert_eq!(
        dir,
        root.path().join("openspec/changes/archive/2026-10-01-x")
    );
}

#[test]
fn resolve_archive_dir_none_is_labeled_error() {
    let root = tree(&[]);
    match resolve_archive_dir(root.path(), "missing") {
        Err(CompanionError::ChangeNotFound { change_id, .. }) => {
            assert_eq!(change_id, "missing");
        }
        other => panic!("expected ChangeNotFound, got {other:?}"),
    }
}

#[test]
fn verify_accepts_dual_format_delta() {
    let root = tree(&[("d/spec.md", DUAL_DELTA)]);
    verify_dual_format(&root.path().join("d/spec.md")).unwrap();
}

#[test]
fn verify_refuses_frontmatterless_delta() {
    let root = tree(&[("d/spec.md", "# plain openspec delta\n\n## Requirements\n")]);
    assert!(verify_dual_format(&root.path().join("d/spec.md")).is_err());
}

#[test]
fn verify_refuses_tableless_delta() {
    let root = tree(&[(
        "d/spec.md",
        "---\nid: spec\nkind: intent\nstatement: \"SHALL x.\"\n---\n\n# c\n",
    )]);
    assert!(verify_dual_format(&root.path().join("d/spec.md")).is_err());
}

#[test]
fn restore_copies_verbatim_and_creates_dirs() {
    let root = tree_owned(&archived_fixtures("a"));
    let archive = resolve_archive_dir(root.path(), "a").unwrap();
    let restored = restore(root.path(), &archive).unwrap();
    let target = root.path().join("openspec/specs/c/spec.md");
    assert_eq!(restored, vec![target.clone()]);
    assert_eq!(
        fs::read(&target).unwrap(),
        fs::read(archive.join("specs/c/spec.md")).unwrap(),
        "deployed spec must be byte-identical to the archived delta"
    );
}

#[test]
fn restore_empty_delta_set_is_success() {
    let root = tree(&[(
        "openspec/changes/archive/2026-10-01-b/proposal.md",
        "# docs",
    )]);
    let archive = resolve_archive_dir(root.path(), "b").unwrap();
    let restored = restore(root.path(), &archive).unwrap();
    assert!(restored.is_empty());
}

#[test]
fn restore_refuses_stripped_delta_before_any_copy() {
    let root = tree(&[
        (
            "openspec/changes/archive/2026-10-01-s/specs/c/spec.md",
            "# plain — no frontmatter\n",
        ),
        // A deployed spec that must NOT be touched by the refusal.
        ("openspec/specs/c/spec.md", "deployed original"),
    ]);
    // tree(&[…]) above takes &str entries; keep as-is
    let archive = resolve_archive_dir(root.path(), "s").unwrap();
    match restore(root.path(), &archive) {
        Err(CompanionError::UnverifiableDeltas { deltas, count }) => {
            assert_eq!(count, 1);
            assert!(deltas[0].ends_with("specs/c/spec.md"));
        }
        other => panic!("expected UnverifiableDeltas, got {other:?}"),
    }
    assert_eq!(
        fs::read_to_string(root.path().join("openspec/specs/c/spec.md")).unwrap(),
        "deployed original",
        "a refusal must not modify any deployed spec"
    );
}

#[test]
fn run_archives_active_change_then_restores() {
    let root = tree(&[("openspec/changes/live/specs/c/spec.md", DUAL_DELTA)]);
    // The fake runner simulates openspec: moves the change into the
    // archive dir, exactly what --skip-specs leaves behind.
    let root_path = root.path().to_path_buf();
    let mut runner = move |root: &Path, id: &str| -> Result<(), String> {
        let src = root.join("openspec/changes").join(id);
        let dst = root.join("openspec/changes/archive/2026-10-01-live");
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::rename(src, dst).unwrap();
        Ok(())
    };
    let outcome = run(root_path.as_path(), "live", false, &mut runner).unwrap();
    assert!(outcome.openspec_ran);
    assert!(!outcome.dry_run);
    assert_eq!(
        outcome.restored,
        vec![root_path.join("openspec/specs/c/spec.md")]
    );
    assert_eq!(
        fs::read(outcome.restored[0].clone()).unwrap(),
        fs::read(root_path.join("openspec/changes/archive/2026-10-01-live/specs/c/spec.md"))
            .unwrap()
    );
}

#[test]
fn run_already_archived_skips_openspec() {
    let root = tree_owned(&archived_fixtures("done"));
    let outcome = run(root.path(), "done", false, &mut panicking_runner()).unwrap();
    assert!(!outcome.openspec_ran);
    assert_eq!(outcome.restored.len(), 1);
}

#[test]
fn run_unknown_change_is_labeled_error() {
    let root = tree(&[]);
    assert!(matches!(
        run(root.path(), "ghost", false, &mut noop_runner()),
        Err(CompanionError::ChangeNotFound { .. })
    ));
}

#[test]
fn run_dry_run_never_invokes_or_writes() {
    let root = tree_owned(&archived_fixtures("dry"));
    let before = fs::read(
        root.path()
            .join("openspec/changes/archive/2026-10-01-dry/specs/c/spec.md"),
    )
    .unwrap();
    let outcome = run(root.path(), "dry", true, &mut panicking_runner()).unwrap();
    assert!(outcome.dry_run);
    assert!(!outcome.openspec_ran);
    assert_eq!(
        outcome.restored,
        vec![root.path().join("openspec/specs/c/spec.md")]
    );
    assert!(
        !root.path().join("openspec/specs").exists(),
        "dry-run must not create deployed spec paths"
    );
    assert_eq!(
        fs::read(
            root.path()
                .join("openspec/changes/archive/2026-10-01-dry/specs/c/spec.md")
        )
        .unwrap(),
        before
    );
}

#[test]
fn run_dry_run_over_active_change_reports_plan() {
    let root = tree(&[("openspec/changes/fresh/specs/c/spec.md", DUAL_DELTA)]);
    let outcome = run(root.path(), "fresh", true, &mut panicking_runner()).unwrap();
    assert!(outcome.dry_run);
    assert_eq!(
        outcome.restored,
        vec![root.path().join("openspec/specs/c/spec.md")]
    );
}

#[test]
fn run_dry_run_refuses_stripped_delta() {
    let root = tree(&[(
        "openspec/changes/stripped/specs/c/spec.md",
        "# no frontmatter\n",
    )]);
    assert!(matches!(
        run(root.path(), "stripped", true, &mut panicking_runner()),
        Err(CompanionError::UnverifiableDeltas { .. })
    ));
}

#[test]
fn run_open_spec_failure_is_labeled() {
    let root = tree(&[("openspec/changes/broken/specs/c/spec.md", DUAL_DELTA)]);
    let mut failing = |_root: &Path, _id: &str| -> Result<(), String> {
        Err("openspec: validation failed".to_string())
    };
    assert!(matches!(
        run(root.path(), "broken", false, &mut failing),
        Err(CompanionError::OpenSpecFailed { .. })
    ));
}

#[test]
fn run_open_spec_missing_is_labeled() {
    let root = tree(&[("openspec/changes/nobin/specs/c/spec.md", DUAL_DELTA)]);
    let mut missing = |_root: &Path, _id: &str| -> Result<(), String> {
        Err("spawn failed: No such file or directory".to_string())
    };
    assert!(matches!(
        run(root.path(), "nobin", false, &mut missing),
        Err(CompanionError::OpenSpecMissing { .. })
    ));
}
