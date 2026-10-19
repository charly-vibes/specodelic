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

/// A full dual-format delta: frontmatter + tables + a delta section and
/// its byte-identical mirror (the corpus's self-contained convention).
/// 1.x flags the delta sections in a deployed copy; Stage 1 strips them
/// at deploy time (specodelic-s8fg) while the mirror stays as the
/// deployed Requirements section.
const FULL_DELTA: &str = concat!(
    "---\nid: spec\nkind: intent\nstatement: \"SHALL x.\"\n---\n\n# c\n\n",
    "## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n",
    "\n## ADDED Requirements\n\n",
    "### Requirement: One holds\nThe system SHALL hold one.\n\n#### Scenario: One\n- **WHEN** x\n- **THEN** y\n\n",
    "## Requirements\n\n",
    "### Requirement: One holds\nThe system SHALL hold one.\n\n#### Scenario: One\n- **WHEN** x\n- **THEN** y\n"
);

/// The expected deployed shape after the Stage-1 strip: delta sections
/// gone, frontmatter + tables + mirror intact.
const STRIPPED_DEPLOY: &str = concat!(
    "---\nid: spec\nkind: intent\nstatement: \"SHALL x.\"\n---\n\n# c\n\n",
    "## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n",
    "\n## Requirements\n\n",
    "### Requirement: One holds\nThe system SHALL hold one.\n\n#### Scenario: One\n- **WHEN** x\n- **THEN** y\n"
);

/// The archived delta shape AFTER post-deploy normalization (the
/// openspec-archival shape the corpus linter expects: frontmatter-less).
const STRIPPED_DELTA: &str = "# c\n\n## Requirements\n";

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

// === Stage 1: deployed-copy delta-section strip (specodelic-s8fg) ===

#[test]
fn restore_strips_delta_sections_from_deployed_copy() {
    let root = tree(&[(
        "openspec/changes/archive/2026-10-01-s1/specs/c/spec.md",
        FULL_DELTA,
    )]);
    let archive = resolve_archive_dir(root.path(), "s1").unwrap();
    let restored = restore(root.path(), &archive).unwrap();
    assert_eq!(restored.len(), 1);
    let deployed = fs::read_to_string(root.path().join("openspec/specs/c/spec.md")).unwrap();
    assert_eq!(deployed, STRIPPED_DEPLOY);
    // the archived delta itself keeps the full dual-format layer (D4
    // lives on for the ARCHIVED copy; only the DEPLOYED copy is cleaned)
    assert_eq!(
        fs::read_to_string(archive.join("specs/c/spec.md")).unwrap(),
        FULL_DELTA
    );
}

#[test]
fn restore_strips_all_four_delta_section_titles() {
    let delta = concat!(
        "## ADDED Requirements\n\n### Requirement: A\na.\n\n",
        "## MODIFIED Requirements\n\n### Requirement: B\nb.\n\n",
        "## REMOVED Requirements\n\n### Requirement: C\n\n",
        "## RENAMED Requirements\n\n### Requirement: D\n- FROM: D\n- TO: E\n\n",
        "## Requirements\n\n### Requirement: A\na.\n"
    );
    let out = strip_delta_sections(delta);
    for title in ["ADDED", "MODIFIED", "REMOVED", "RENAMED"] {
        assert!(
            !out.contains(&format!("## {title}")),
            "title {title} survived"
        );
    }
    assert!(out.contains("## Requirements"));
    assert!(out.contains("### Requirement: A"));
}

#[test]
fn restore_strip_ignores_fenced_delta_headers() {
    let delta = "## Requirements\n\n### Requirement: One holds\nExample:\n\n```md\n## ADDED Requirements\n```\n";
    let out = strip_delta_sections(delta);
    assert!(
        out.contains("## ADDED Requirements"),
        "a delta header inside a code fence is content, not structure"
    );
}

#[test]
fn restore_refuses_delta_missing_mirror_coverage() {
    // self-containedness (corpus constraint delta_self_contained): every
    // ADDED/MODIFIED requirement must appear in the mirror
    let delta = concat!(
        "---\nid: spec\nkind: intent\nstatement: \"SHALL x.\"\n---\n\n## Constraints\n\n",
        "## ADDED Requirements\n\n### Requirement: One holds\nBody.\n\n",
        "## Requirements\n\n### Requirement: Something else\nBody.\n"
    );
    let root = tree(&[(
        "openspec/changes/archive/2026-10-01-g1/specs/c/spec.md",
        delta,
    )]);
    let archive = resolve_archive_dir(root.path(), "g1").unwrap();
    match restore(root.path(), &archive) {
        Err(CompanionError::DeltaNotSelfContained { missing, .. }) => {
            assert_eq!(missing, vec!["One holds".to_string()]);
        }
        other => panic!("expected DeltaNotSelfContained, got {other:?}"),
    }
    assert!(
        !root.path().join("openspec/specs").exists(),
        "a refusal must not create deployed spec paths"
    );
}

#[test]
fn restore_refuses_narrowing_existing_deployed_spec() {
    // the deployed target has a requirement the delta's full requirement
    // set (mirror ∪ REMOVED ∪ RENAMED) does not restate — verbatim deploy
    // would silently narrow the spec
    let existing = concat!(
        "## Requirements\n\n### Requirement: One holds\nOld.\n\n",
        "### Requirement: Two holds\nOld.\n"
    );
    let root = tree(&[
        (
            "openspec/changes/archive/2026-10-01-g2/specs/c/spec.md",
            FULL_DELTA,
        ),
        ("openspec/specs/c/spec.md", existing),
    ]);
    let archive = resolve_archive_dir(root.path(), "g2").unwrap();
    match restore(root.path(), &archive) {
        Err(CompanionError::UncoveredDeployedRequirements { missing, .. }) => {
            assert_eq!(missing, vec!["Two holds".to_string()]);
        }
        other => panic!("expected UncoveredDeployedRequirements, got {other:?}"),
    }
    assert_eq!(
        fs::read_to_string(root.path().join("openspec/specs/c/spec.md")).unwrap(),
        existing,
        "a refusal must leave the deployed spec untouched"
    );
}

#[test]
fn restore_accepts_full_restatement_over_existing_spec() {
    // corpus convention: the delta restates the capability's FULL
    // requirement set — every deployed name appears in the delta
    let existing = "## Requirements\n\n### Requirement: One holds\nOld.\n";
    let root = tree(&[
        (
            "openspec/changes/archive/2026-10-01-g3/specs/c/spec.md",
            FULL_DELTA,
        ),
        ("openspec/specs/c/spec.md", existing),
    ]);
    let archive = resolve_archive_dir(root.path(), "g3").unwrap();
    restore(root.path(), &archive).unwrap();
    let deployed = fs::read_to_string(root.path().join("openspec/specs/c/spec.md")).unwrap();
    assert_eq!(deployed, STRIPPED_DEPLOY);
}

#[test]
fn restore_strip_output_must_be_1x_clean() {
    // post-strip shape assertion (EDGE-001): a requirement header
    // outside the mirror and outside any delta section would survive the
    // strip as an invisible-to-openspec requirement — refuse
    let delta = concat!(
        "---\nid: spec\nkind: intent\nstatement: \"SHALL x.\"\n---\n\n## Constraints\n\n",
        "### Requirement: Stray holds\nBody.\n\n",
        "## Requirements\n\n### Requirement: Stray holds\nBody.\n"
    );
    let root = tree(&[(
        "openspec/changes/archive/2026-10-01-g4/specs/c/spec.md",
        delta,
    )]);
    let archive = resolve_archive_dir(root.path(), "g4").unwrap();
    match restore(root.path(), &archive) {
        Err(CompanionError::DeployedShapeViolation { detail, .. }) => {
            assert!(detail.contains("Stray holds"), "detail: {detail}");
        }
        other => panic!("expected DeployedShapeViolation, got {other:?}"),
    }
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
        fs::read_to_string(outcome.restored[0].clone()).unwrap(),
        DUAL_DELTA,
        "the deployed spec keeps the dual-format layer"
    );
    assert_eq!(
        outcome.stripped,
        vec![root_path.join("openspec/changes/archive/2026-10-01-live/specs/c/spec.md")]
    );
    assert!(
        !fs::read_to_string(
            root_path.join("openspec/changes/archive/2026-10-01-live/specs/c/spec.md")
        )
        .unwrap()
        .starts_with("---"),
        "the archived delta is frontmatter-less after the run"
    );
}

#[test]
fn run_strips_archived_delta_after_deploy() {
    let root = tree_owned(&archived_fixtures("norm"));
    let outcome = run(root.path(), "norm", false, &mut panicking_runner()).unwrap();
    let archived = root
        .path()
        .join("openspec/changes/archive/2026-10-01-norm/specs/c/spec.md");
    let deployed = root.path().join("openspec/specs/c/spec.md");
    assert_eq!(outcome.stripped, vec![archived.clone()]);
    let text = fs::read_to_string(&archived).unwrap();
    assert!(
        !text.starts_with("---"),
        "archived delta must be frontmatter-less after the run"
    );
    assert!(
        text.contains("# c"),
        "the delta body must survive the strip"
    );
    assert_eq!(
        fs::read_to_string(&deployed).unwrap(),
        DUAL_DELTA,
        "the deployed spec keeps its dual-format layer verbatim"
    );
}

#[test]
fn run_rerun_after_strip_is_noop_success() {
    let root = tree_owned(&archived_fixtures("rerun"));
    let deployed = root.path().join("openspec/specs/c/spec.md");
    run(root.path(), "rerun", false, &mut panicking_runner()).unwrap();
    assert_eq!(fs::read_to_string(&deployed).unwrap(), DUAL_DELTA);
    let outcome = run(root.path(), "rerun", false, &mut panicking_runner()).unwrap();
    assert!(!outcome.openspec_ran);
    assert!(
        outcome.restored.is_empty(),
        "re-run must not re-copy the stripped delta"
    );
    assert!(outcome.stripped.is_empty());
    assert_eq!(
        fs::read_to_string(&deployed).unwrap(),
        DUAL_DELTA,
        "re-run must recognize the already-deployed spec, never overwrite it with the stripped delta"
    );
}

#[test]
fn restore_recognizes_already_deployed_stripped_delta() {
    let root = tree(&[
        (
            "openspec/changes/archive/2026-10-01-r2/specs/c/spec.md",
            STRIPPED_DELTA,
        ),
        ("openspec/specs/c/spec.md", DUAL_DELTA),
    ]);
    let archive = resolve_archive_dir(root.path(), "r2").unwrap();
    let restored = restore(root.path(), &archive).unwrap();
    assert!(restored.is_empty());
    assert_eq!(
        fs::read_to_string(root.path().join("openspec/specs/c/spec.md")).unwrap(),
        DUAL_DELTA
    );
}

#[test]
fn restore_refuses_stripped_delta_without_deployed_target() {
    let root = tree(&[(
        "openspec/changes/archive/2026-10-01-r3/specs/c/spec.md",
        STRIPPED_DELTA,
    )]);
    let archive = resolve_archive_dir(root.path(), "r3").unwrap();
    assert!(matches!(
        restore(root.path(), &archive),
        Err(CompanionError::UnverifiableDeltas { .. })
    ));
    assert!(
        !root.path().join("openspec/specs").exists(),
        "a refusal must not create deployed spec paths"
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
