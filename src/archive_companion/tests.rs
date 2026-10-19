//! Tests for the archive-companion module (tempdirs, no openspec
//! process ever spawned — the runner seam, design D2/D7).
//!
//! Stage-2 shape (specodelic-s8fg, Revision 19): openspec applies the
//! plain delta (the native merge), the companion re-attaches the
//! dual-format layer from the archived `layer.md` sidecar. The dual-era
//! guards (self-containedness, existing-target coverage, post-strip
//! shape) are retired with their subject; the fail-closed guards here
//! protect the NEW failure shapes: sidecar-less/unlintable refusal
//! BEFORE the destructive archive, and the post-attach shape + lint
//! assertion with revert.

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

/// A plain openspec delta (Revision 19): `## Purpose` + delta section,
/// no frontmatter, no layer tables — openspec's native merge input.
const PLAIN_DELTA: &str = concat!(
    "## Purpose\nKeep the c capability thinkable-with.\n\n",
    "## ADDED Requirements\n\n",
    "### Requirement: One holds\nThe system SHALL hold one.\n\n",
    "#### Scenario: One\n- **WHEN** x\n- **THEN** y\n"
);

/// The deployed spec openspec's archive apply writes from that delta
/// (1.x shape: heading + Purpose + applied Requirements).
const APPLIED_DEPLOY: &str = concat!(
    "# c Specification\n\n",
    "## Purpose\nKeep the c capability thinkable-with.\n\n",
    "## Requirements\n\n",
    "### Requirement: One holds\nThe system SHALL hold one.\n\n",
    "#### Scenario: One\n- **WHEN** x\n- **THEN** y\n"
);

/// A lint-clean `layer.md` sidecar: frontmatter (`id: c.layer`) + the
/// three specodelic tables. Rows cite the sidecar's own intent (the
/// refs the companion re-keys to the deployed id at attach time).
pub const SIDECAR: &str = concat!(
    "---\nid: c.layer\nkind: intent\n",
    "statement: \"WHEN the c layer is elaborated, THE author SHALL write the real requirement.\"\n",
    "---\n\n",
    "## Constraints\n\n",
    "| id | kind | expr | traces_to |\n",
    "|----|------|------|-----------|\n",
    "| k1 | invariant | `true` | [[c.layer]] |\n\n",
    "## Model\n\n",
    "### States\n\n",
    "- `draft`\n\n",
    "### Transitions\n\n",
    "| id | from | to | guard |\n",
    "|----|------|----|-------|\n",
    "| t1 | draft | draft | [[c.layer.k1]] |\n\n",
    "## Properties\n\n",
    "| id | kind | derives_from | generator | predicate |\n",
    "|----|------|--------------|-----------|-----------|\n",
    "| p1 | unit | [[c.layer.k1]] | `todo()` | `true` |\n"
);

/// The expected deployed dual-format spec: sidecar frontmatter (id
/// re-keyed to `c`) + applied heading/Purpose + sidecar tables (refs
/// re-keyed) + applied Requirements.
const ASSEMBLED_DEPLOY: &str = concat!(
    "---\nid: c\nkind: intent\n",
    "statement: \"WHEN the c layer is elaborated, THE author SHALL write the real requirement.\"\n",
    "---\n\n",
    "# c Specification\n\n",
    "## Purpose\nKeep the c capability thinkable-with.\n\n",
    "## Constraints\n\n",
    "| id | kind | expr | traces_to |\n",
    "|----|------|------|-----------|\n",
    "| k1 | invariant | `true` | [[c]] |\n\n",
    "## Model\n\n",
    "### States\n\n",
    "- `draft`\n\n",
    "### Transitions\n\n",
    "| id | from | to | guard |\n",
    "|----|------|----|-------|\n",
    "| t1 | draft | draft | [[c.k1]] |\n\n",
    "## Properties\n\n",
    "| id | kind | derives_from | generator | predicate |\n",
    "|----|------|--------------|-----------|-----------|\n",
    "| p1 | unit | [[c.k1]] | `todo()` | `true` |\n\n",
    "## Requirements\n\n",
    "### Requirement: One holds\nThe system SHALL hold one.\n\n",
    "#### Scenario: One\n- **WHEN** x\n- **THEN** y\n"
);

/// An archive dir with one plain delta + sidecar pair for capability `c`.
fn archived_fixtures(id: &str) -> Vec<(String, &'static str)> {
    vec![
        (
            format!("openspec/changes/archive/2026-10-01-{id}/specs/c/spec.md"),
            PLAIN_DELTA,
        ),
        (
            format!("openspec/changes/archive/2026-10-01-{id}/specs/c/layer.md"),
            SIDECAR,
        ),
    ]
}

fn noop_runner() -> impl FnMut(&Path, &str) -> Result<(), String> {
    |_root, _id| Ok(())
}

fn panicking_runner() -> impl FnMut(&Path, &str) -> Result<(), String> {
    |_root, _id| panic!("openspec must not be invoked in this scenario")
}

/// Simulate openspec archive WITH spec application (Revision 19
/// default): the change dir moves to the archive and the applied
/// deployed spec is written — exactly what the real CLI does.
fn applying_runner() -> impl FnMut(&Path, &str) -> Result<(), String> {
    move |root: &Path, id: &str| -> Result<(), String> {
        let src = root.join("openspec/changes").join(id);
        let dst = root.join(format!("openspec/changes/archive/2026-10-01-{id}"));
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::rename(&src, &dst).unwrap();
        fs::create_dir_all(root.join("openspec/specs/c")).unwrap();
        fs::write(root.join("openspec/specs/c/spec.md"), APPLIED_DEPLOY).unwrap();
        Ok(())
    }
}

#[test]
fn resolve_archive_dir_newest_wins() {
    let root = tree(&[
        (
            "openspec/changes/archive/2026-09-29-x/specs/c/spec.md",
            PLAIN_DELTA,
        ),
        (
            "openspec/changes/archive/2026-10-01-x/specs/c/spec.md",
            PLAIN_DELTA,
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

// === 4.1: apply + re-attach assembly ===

#[test]
fn restore_assembles_dual_format_from_sidecar_and_applied_spec() {
    let root = tree_owned(&archived_fixtures("asm"));
    let archive = resolve_archive_dir(root.path(), "asm").unwrap();
    // openspec's apply already wrote the deployed spec from the delta
    fs::create_dir_all(root.path().join("openspec/specs/c")).unwrap();
    fs::write(root.path().join("openspec/specs/c/spec.md"), APPLIED_DEPLOY).unwrap();

    let restored = restore(root.path(), &archive).unwrap();
    assert_eq!(restored, vec![root.path().join("openspec/specs/c/spec.md")]);

    let deployed = fs::read_to_string(root.path().join("openspec/specs/c/spec.md")).unwrap();
    assert_eq!(deployed, ASSEMBLED_DEPLOY);
    // the archived delta stays byte-plain (openspec moved it verbatim;
    // the companion never rewrites it)
    assert_eq!(
        fs::read_to_string(archive.join("specs/c/spec.md")).unwrap(),
        PLAIN_DELTA
    );
    // the sidecar rides along in the archive, untouched
    assert_eq!(
        fs::read_to_string(archive.join("specs/c/layer.md")).unwrap(),
        SIDECAR
    );
}

#[test]
fn restore_new_capability_deploys_from_sidecar_tables() {
    // a cap with no pre-existing deployed spec at attach time: the
    // assembly is the sidecar layer alone (defensive; openspec's apply
    // normally writes the deployed file first)
    let root = tree_owned(&[
        (
            "openspec/changes/archive/2026-10-01-new/specs/c/spec.md".to_string(),
            PLAIN_DELTA,
        ),
        (
            "openspec/changes/archive/2026-10-01-new/specs/c/layer.md".to_string(),
            SIDECAR,
        ),
    ]);
    let archive = resolve_archive_dir(root.path(), "new").unwrap();
    let restored = restore(root.path(), &archive).unwrap();
    let deployed = fs::read_to_string(root.path().join("openspec/specs/c/spec.md")).unwrap();
    assert!(restored.len() == 1);
    // frontmatter + the three tables, no delta headers
    assert!(deployed.starts_with("---\nid: c\n"));
    assert!(deployed.contains("## Constraints"));
    assert!(deployed.contains("## Model"));
    assert!(deployed.contains("## Properties"));
    assert!(!deployed.contains("## ADDED Requirements"));
}

#[test]
fn restore_rekeys_sidecar_id_and_refs() {
    let root = tree_owned(&archived_fixtures("rekey"));
    let archive = resolve_archive_dir(root.path(), "rekey").unwrap();
    restore(root.path(), &archive).unwrap();
    let deployed = fs::read_to_string(root.path().join("openspec/specs/c/spec.md")).unwrap();
    assert!(deployed.contains("id: c\n"), "{deployed}");
    assert!(
        !deployed.contains("c.layer"),
        "stale layer id must be re-keyed: {deployed}"
    );
    assert!(deployed.contains("[[c]]"));
    assert!(deployed.contains("[[c.k1]]"));
}

// === 4.2: fail-closed guards ===

#[test]
fn restore_refuses_sidecarless_delta_before_any_write() {
    let root = tree(&[(
        "openspec/changes/archive/2026-10-01-s/specs/c/spec.md",
        PLAIN_DELTA,
    )]);
    let archive = resolve_archive_dir(root.path(), "s").unwrap();
    match restore(root.path(), &archive) {
        Err(CompanionError::SidecarRefusal { refused, .. }) => {
            assert_eq!(refused.len(), 1);
            assert!(refused[0].0.ends_with("specs/c/spec.md"));
            assert!(refused[0].1.contains("layer.md"), "{}", refused[0].1);
        }
        other => panic!("expected SidecarRefusal, got {other:?}"),
    }
    assert!(
        !root.path().join("openspec/specs").exists(),
        "a refusal must not create deployed spec paths"
    );
}

#[test]
fn run_refuses_sidecarless_active_change_before_invocation() {
    // a dual-format (not-yet-migrated) delta: no sidecar beside it —
    // the refusal must fire BEFORE the destructive openspec archive
    let root = tree(&[(
        "openspec/changes/live/specs/c/spec.md",
        "---\nid: spec\nkind: intent\nstatement: \"SHALL x.\"\n---\n\n## Constraints\n",
    )]);
    let outcome = run(root.path(), "live", false, &mut panicking_runner());
    match outcome {
        Err(CompanionError::SidecarRefusal { refused, .. }) => {
            assert_eq!(refused.len(), 1);
            assert!(refused[0].0.ends_with("specs/c/spec.md"));
        }
        other => panic!("expected SidecarRefusal, got {other:?}"),
    }
    assert!(
        root.path().join("openspec/changes/live").is_dir(),
        "the change dir must NOT be moved by a pre-invocation refusal"
    );
    assert!(
        !root.path().join("openspec/specs").exists(),
        "a refusal must not write any deployed spec"
    );
}

#[test]
fn run_refuses_unlintable_sidecar_before_invocation() {
    // sidecar parses as garbage (broken frontmatter): refused with the
    // findings, before anything moves
    let root = tree(&[
        ("openspec/changes/live/specs/c/spec.md", PLAIN_DELTA),
        (
            "openspec/changes/live/specs/c/layer.md",
            "not frontmatter at all",
        ),
    ]);
    match run(root.path(), "live", false, &mut panicking_runner()) {
        Err(CompanionError::SidecarRefusal { refused, .. }) => {
            assert_eq!(refused.len(), 1);
            assert!(refused[0].0.ends_with("layer.md"), "{:?}", refused[0].0);
            assert!(
                !refused[0].1.is_empty(),
                "the refusal must carry the findings"
            );
        }
        other => panic!("expected SidecarRefusal, got {other:?}"),
    }
    assert!(
        root.path().join("openspec/changes/live").is_dir(),
        "the change dir must NOT be moved by a pre-invocation refusal"
    );
}

#[test]
fn restore_refuses_sidecar_missing_a_table() {
    // parses, but no ## Properties table: it cannot restore the
    // three-table layer — refused as unlintable
    let root = tree_owned(&[
        (
            "openspec/changes/archive/2026-10-01-partial/specs/c/spec.md".to_string(),
            PLAIN_DELTA,
        ),
        (
            "openspec/changes/archive/2026-10-01-partial/specs/c/layer.md".to_string(),
            "---\nid: c.layer\nkind: intent\nstatement: \"SHALL x.\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n",
        ),
    ]);
    let archive = resolve_archive_dir(root.path(), "partial").unwrap();
    match restore(root.path(), &archive) {
        Err(CompanionError::SidecarRefusal { refused, .. }) => {
            assert!(refused[0].1.contains("Properties"), "{}", refused[0].1);
        }
        other => panic!("expected SidecarRefusal, got {other:?}"),
    }
}

#[test]
fn restore_post_attach_shape_assertion_refuses_delta_headers() {
    // if the deployed body still carries a delta header (e.g. openspec
    // did not apply the delta), the assembly would deploy a shape 1.x
    // strict rejects — refused, the write reverted
    let root = tree(&[
        (
            "openspec/changes/archive/2026-10-01-raw/specs/c/spec.md",
            PLAIN_DELTA,
        ),
        (
            "openspec/changes/archive/2026-10-01-raw/specs/c/layer.md",
            SIDECAR,
        ),
        ("openspec/specs/c/spec.md", PLAIN_DELTA),
    ]);
    let archive = resolve_archive_dir(root.path(), "raw").unwrap();
    match restore(root.path(), &archive) {
        Err(CompanionError::AttachViolation { cap, detail, .. }) => {
            assert_eq!(cap, "c");
            assert!(detail.contains("ADDED"), "detail: {detail}");
        }
        other => panic!("expected AttachViolation, got {other:?}"),
    }
    assert!(
        !fs::read_to_string(root.path().join("openspec/specs/c/spec.md"))
            .unwrap()
            .contains("## Constraints"),
        "the reverted deployed tree must not carry the assembled layer"
    );
}

#[test]
fn restore_post_attach_lint_failure_reverts_write() {
    // a sidecar whose tables violate coverage (constraint without a
    // deriving property): pre-guard passes (it parses, three tables),
    // the post-attach corpus lint catches it — revert + labeled error
    let dirty_sidecar = concat!(
        "---\nid: c.layer\nkind: intent\nstatement: \"SHALL x.\"\n---\n\n",
        "## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| k1 | invariant | `true` | [[c.layer]] |\n\n",
        "## Model\n\n### States\n\n- `draft`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | draft | draft | [[c.layer.k1]] |\n\n",
        "## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n"
    );
    let prior = "# c Specification\n\n## Requirements\n\n### Requirement: Old\nOld.\n";
    let root = tree(&[
        (
            "openspec/changes/archive/2026-10-01-dirty/specs/c/spec.md",
            PLAIN_DELTA,
        ),
        (
            "openspec/changes/archive/2026-10-01-dirty/specs/c/layer.md",
            dirty_sidecar,
        ),
        ("openspec/specs/c/spec.md", prior),
    ]);
    let archive = resolve_archive_dir(root.path(), "dirty").unwrap();
    match restore(root.path(), &archive) {
        Err(CompanionError::AttachViolation { cap, detail, .. }) => {
            assert_eq!(cap, "c");
            assert!(
                detail.contains("coverage") || detail.contains("derives"),
                "detail must name the lint findings: {detail}"
            );
        }
        other => panic!("expected AttachViolation, got {other:?}"),
    }
    assert_eq!(
        fs::read_to_string(root.path().join("openspec/specs/c/spec.md")).unwrap(),
        prior,
        "the violated write must be reverted to the prior deployed bytes"
    );
}

#[test]
fn run_reverts_archive_move_on_attach_violation() {
    // the destructive openspec archive ran, then attach failed: the
    // change dir must move BACK out of changes/archive (best-effort
    // restore of the pre-archive state)
    let root = tree(&[("openspec/changes/broken/specs/c/spec.md", PLAIN_DELTA)]);
    let root_path = root.path().to_path_buf();
    // runner moves the change but writes NO applied spec (deployed stays
    // a raw delta → attach shape violation)
    let mut runner = move |root: &Path, id: &str| -> Result<(), String> {
        let src = root.join("openspec/changes").join(id);
        let dst = root.join(format!("openspec/changes/archive/2026-10-01-{id}"));
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::rename(src, dst).unwrap();
        Ok(())
    };
    assert!(run(root_path.as_path(), "broken", false, &mut runner).is_err());
    assert!(
        root_path.join("openspec/changes/broken").is_dir(),
        "the change dir must be moved back out of the archive"
    );
    assert!(
        !root_path
            .join("openspec/changes/archive/2026-10-01-broken")
            .exists(),
        "the archive dir must not linger after the revert"
    );
}

#[test]
fn restore_recognizes_already_deployed_sidecarless_archive() {
    // Stage-1-era archive (plain delta, no sidecar) whose deployed
    // target already carries the dual-format layer: recognized as
    // converged, no-op success — never clobbered
    let root = tree(&[
        (
            "openspec/changes/archive/2026-10-01-r2/specs/c/spec.md",
            PLAIN_DELTA,
        ),
        ("openspec/specs/c/spec.md", ASSEMBLED_DEPLOY),
    ]);
    let archive = resolve_archive_dir(root.path(), "r2").unwrap();
    let restored = restore(root.path(), &archive).unwrap();
    assert!(restored.is_empty());
    assert_eq!(
        fs::read_to_string(root.path().join("openspec/specs/c/spec.md")).unwrap(),
        ASSEMBLED_DEPLOY
    );
}

// === 4.3: idempotent rerun + newest-archive-wins + dry-run ===

#[test]
fn run_rerun_converges_to_identical_deployed_bytes() {
    let root = tree(&[("openspec/changes/live/specs/c/spec.md", PLAIN_DELTA)]);
    // add the sidecar to the active change
    fs::write(
        root.path().join("openspec/changes/live/specs/c/layer.md"),
        SIDECAR,
    )
    .unwrap();
    let root_path = root.path().to_path_buf();

    let outcome = run(root_path.as_path(), "live", false, &mut applying_runner()).unwrap();
    assert!(outcome.openspec_ran);
    let deployed = root_path.join("openspec/specs/c/spec.md");
    let first = fs::read(&deployed).unwrap();
    assert_eq!(fs::read_to_string(&deployed).unwrap(), ASSEMBLED_DEPLOY);

    // rerun: openspec is NOT invoked (already archived); the re-attach
    // re-resolves from the archive and converges to identical bytes
    let outcome2 = run(root_path.as_path(), "live", false, &mut panicking_runner()).unwrap();
    assert!(!outcome2.openspec_ran);
    assert_eq!(
        fs::read(&deployed).unwrap(),
        first,
        "rerun must converge byte-identically"
    );
    assert_eq!(
        fs::read_to_string(&deployed).unwrap(),
        ASSEMBLED_DEPLOY,
        "rerun must still produce the fully assembled dual-format spec"
    );
}

#[test]
fn restore_newest_archive_wins() {
    let old_sidecar = SIDECAR.replace(
        "WHEN the c layer is elaborated",
        "OLDER MARKER: WHEN the c layer is elaborated",
    );
    let root = tree_owned(&[
        (
            "openspec/changes/archive/2026-09-29-n/specs/c/spec.md".to_string(),
            PLAIN_DELTA,
        ),
        (
            "openspec/changes/archive/2026-09-29-n/specs/c/layer.md".to_string(),
            old_sidecar.as_str(),
        ),
        (
            "openspec/changes/archive/2026-10-01-n/specs/c/spec.md".to_string(),
            PLAIN_DELTA,
        ),
        (
            "openspec/changes/archive/2026-10-01-n/specs/c/layer.md".to_string(),
            SIDECAR,
        ),
        ("openspec/specs/c/spec.md".to_string(), APPLIED_DEPLOY),
    ]);
    let outcome = run(root.path(), "n", false, &mut panicking_runner()).unwrap();
    assert_eq!(
        outcome.archive_dir,
        root.path().join("openspec/changes/archive/2026-10-01-n"),
        "the newest archive directory must win"
    );
    let deployed = fs::read_to_string(root.path().join("openspec/specs/c/spec.md")).unwrap();
    assert!(
        !deployed.contains("OLDER MARKER"),
        "the stale archive's layer must not leak into the deploy: {deployed}"
    );
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
    assert_eq!(
        outcome.sidecars,
        vec![
            root.path()
                .join("openspec/changes/archive/2026-10-01-dry/specs/c/layer.md")
        ]
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
    let root = tree(&[("openspec/changes/fresh/specs/c/spec.md", PLAIN_DELTA)]);
    fs::write(
        root.path().join("openspec/changes/fresh/specs/c/layer.md"),
        SIDECAR,
    )
    .unwrap();
    let outcome = run(root.path(), "fresh", true, &mut panicking_runner()).unwrap();
    assert!(outcome.dry_run);
    assert_eq!(
        outcome.restored,
        vec![root.path().join("openspec/specs/c/spec.md")]
    );
    assert_eq!(
        outcome.sidecars,
        vec![root.path().join("openspec/changes/fresh/specs/c/layer.md")]
    );
    assert!(
        !root.path().join("openspec/specs").exists(),
        "dry-run must not write the deployed spec"
    );
}

#[test]
fn run_dry_run_refuses_sidecarless_delta() {
    let root = tree(&[(
        "openspec/changes/stripped/specs/c/spec.md",
        "# no sidecar\n",
    )]);
    assert!(matches!(
        run(root.path(), "stripped", true, &mut panicking_runner()),
        Err(CompanionError::SidecarRefusal { .. })
    ));
}

#[test]
fn run_already_archived_skips_openspec() {
    let root = tree_owned(&archived_fixtures("done"));
    fs::create_dir_all(root.path().join("openspec/specs/c")).unwrap();
    fs::write(root.path().join("openspec/specs/c/spec.md"), APPLIED_DEPLOY).unwrap();
    let outcome = run(root.path(), "done", false, &mut panicking_runner()).unwrap();
    assert!(!outcome.openspec_ran);
    assert_eq!(outcome.restored.len(), 1);
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
fn run_unknown_change_is_labeled_error() {
    let root = tree(&[]);
    assert!(matches!(
        run(root.path(), "ghost", false, &mut noop_runner()),
        Err(CompanionError::ChangeNotFound { .. })
    ));
}

#[test]
fn run_open_spec_failure_is_labeled() {
    let root = tree(&[("openspec/changes/broken2/specs/c/spec.md", PLAIN_DELTA)]);
    fs::write(
        root.path()
            .join("openspec/changes/broken2/specs/c/layer.md"),
        SIDECAR,
    )
    .unwrap();
    let mut failing = |_root: &Path, _id: &str| -> Result<(), String> {
        Err("openspec: validation failed".to_string())
    };
    assert!(matches!(
        run(root.path(), "broken2", false, &mut failing),
        Err(CompanionError::OpenSpecFailed { .. })
    ));
}

#[test]
fn run_open_spec_missing_is_labeled() {
    let root = tree(&[("openspec/changes/nobin/specs/c/spec.md", PLAIN_DELTA)]);
    fs::write(
        root.path().join("openspec/changes/nobin/specs/c/layer.md"),
        SIDECAR,
    )
    .unwrap();
    let mut missing = |_root: &Path, _id: &str| -> Result<(), String> {
        Err("spawn failed: No such file or directory".to_string())
    };
    assert!(matches!(
        run(root.path(), "nobin", false, &mut missing),
        Err(CompanionError::OpenSpecMissing { .. })
    ));
}
