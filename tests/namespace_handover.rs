//! Regression tests for the identity-namespace handover between a
//! scaffolded `layer.md` sidecar and the deployed spec it is attached
//! to (specodelic-ez9a.2).
//!
//! The bug of record: the layer scaffold derives the error-label
//! namespace from the parent spec id instead of the sidecar's own file
//! id, and the archive companion's `assemble` → `rekey_sidecar` path
//! rewrote only the frontmatter `id:` line and the `[[sidecar_id.*]]` /
//! `[[sidecar_id]]` wiki-links — a BARE label occurrence (the
//! error-contract label form `<owning-file-id>.<variant_head>(…)` in an
//! expr/predicate cell) was deployed un-rekeyed, so the deployed spec
//! carried a label naming the sidecar's id instead of its own.
//!
//! The two tests pin both halves of the handover:
//!   1. the scaffold states the error-label namespace as THIS file's own
//!      id (never the parent capability's id), and
//!   2. an error-labeled sidecar deploys with the DEPLOYED id's
//!      namespace — no `.layer.` remnant anywhere in the deployed spec.

use specodelic::archive_companion;
use specodelic::migrate;
use std::fs;
use std::path::Path;

/// A plain openspec delta (Revision 19): no frontmatter, no layer
/// tables — openspec's native merge input.
const PLAIN_DELTA: &str = concat!(
    "## ADDED Requirements\n\n",
    "### Requirement: One holds\nThe system SHALL hold one.\n\n",
    "#### Scenario: One\n- **WHEN** x\n- **THEN** y\n"
);

/// A lint-clean `layer.md` sidecar carrying ONE error constraint whose
/// label is namespaced by the SIDECAR'S OWN file id (`c.layer.`) — the
/// natural reading of the error contract while the file IS the sidecar.
const ERROR_SIDECAR: &str = concat!(
    "---\nid: c.layer\nkind: intent\n",
    "statement: \"WHEN the c layer is elaborated, THE author SHALL write the real requirement.\"\n",
    "---\n\n",
    "## Constraints\n\n",
    "| id | kind | expr | traces_to |\n",
    "|----|------|------|-----------|\n",
    "| k1 | invariant | `true` | [[c.layer]] |\n",
    "| boom_failure | effect | `c.layer.boom_failure(detail) — the boom stage failed because detail` | [[c.layer]] |\n\n",
    "## Model\n\n",
    "### States\n\n",
    "- `draft`\n",
    "- `boom_failed` (emits: [[c.layer.boom_failure]])\n\n",
    "### Transitions\n\n",
    "| id | from | to | guard |\n",
    "|----|------|----|-------|\n",
    "| t1 | draft | draft | [[c.layer.k1]] |\n",
    "| t2 | draft | boom_failed | [[c.layer.k1]] |\n\n",
    "## Properties\n\n",
    "| id | kind | derives_from | generator | predicate |\n",
    "|----|------|--------------|-----------|-----------|\n",
    "| p1 | unit | [[c.layer.k1]] | `todo()` | `true` |\n",
    "| p2 | unit | [[c.layer.boom_failure]] | `todo()` | `label == \"c.layer.boom_failure(detail)\"` |\n"
);

/// Build an archive dir with the delta + sidecar pair for capability
/// `c` (the shape openspec's archive leaves behind).
fn archive_with_sidecar(sidecar: &str) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let archive = root
        .path()
        .join("openspec/changes/archive/2026-11-12-errns");
    let dir = archive.join("specs/c");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("spec.md"), PLAIN_DELTA).unwrap();
    fs::write(dir.join("layer.md"), sidecar).unwrap();
    root
}

/// The scaffolded sidecar of a plain delta under `widget-cap/`: the
/// sidecar's own file id is the parent-directory id plus the `.layer`
/// suffix (Revision 19) — `widget.cap.layer`.
fn scaffolded_sidecar() -> migrate::WrapOutcome {
    migrate::migrate(
        "## ADDED Requirements\n\n### Requirement: Widget\nThe system SHALL wiget.\n",
        Path::new("widget-cap/spec.md"),
    )
    .unwrap()
}

/// The scaffold states the error-label namespace as THIS file's own id
/// (`widget.cap.layer.`) — never the parent capability's id
/// (`widget.cap.`): a label derived from the parent id would deploy
/// wrong-namespace error labels.
#[test]
fn scaffold_states_the_error_label_namespace_of_its_own_file_id() {
    let sidecar = scaffolded_sidecar();
    assert_eq!(sidecar.sidecar_id, "widget.cap.layer");
    // The rule is stated, with the sidecar's own id spelled out.
    assert!(
        sidecar.content.contains("file-id-namespaced"),
        "the scaffold must state the error-label naming law: {}",
        sidecar.content
    );
    assert!(
        sidecar.content.contains("widget.cap.layer."),
        "the scaffold must state the namespace as the sidecar's OWN file id: {}",
        sidecar.content
    );
    // Every occurrence of the parent capability's id namespace is
    // followed by the sidecar arm — the scaffold never derives an
    // identity (or label) namespace from the parent spec id.
    let mut from = 0usize;
    while let Some(at) = sidecar.content[from..].find("widget.cap.") {
        let at = from + at;
        let rest = &sidecar.content[at + "widget.cap.".len()..];
        assert!(
            rest.starts_with("layer"),
            "the parent id namespace must never stand alone at byte {at}: {}",
            sidecar.content
        );
        from = at + "widget.cap.".len();
    }
}

/// An error-labeled sidecar deploys with the DEPLOYED id's namespace:
/// the bare label `c.layer.boom_failure(detail)` is re-keyed to
/// `c.boom_failure(detail)` (the deployed spec's own id owns the
/// label), and no `.layer.` remnant survives anywhere in the deployed
/// spec.
#[test]
fn error_labeled_sidecar_deploys_with_the_deployed_id_namespace() {
    let root = archive_with_sidecar(ERROR_SIDECAR);
    let archive = root
        .path()
        .join("openspec/changes/archive/2026-11-12-errns");

    let restored = archive_companion::restore(root.path(), &archive).unwrap();
    assert_eq!(restored.len(), 1, "one capability deployed");

    let deployed = fs::read_to_string(root.path().join("openspec/specs/c/spec.md")).unwrap();
    // The deployed spec owns the label: its own id namespaces it.
    assert!(
        deployed.contains("`c.boom_failure(detail)"),
        "the deployed label must carry the deployed id's namespace: {deployed}"
    );
    assert!(
        deployed.contains("label == \"c.boom_failure(detail)\""),
        "the property must assert the deployed label: {deployed}"
    );
    // The emits edge points at the deployed file's own row.
    assert!(
        deployed.contains("[[c.boom_failure]]"),
        "the emits edge must name the deployed file's own row: {deployed}"
    );
    // No remnant of the sidecar's identity namespace survives.
    assert!(
        !deployed.contains(".layer."),
        "no sidecar-namespace remnant may survive the deploy: {deployed}"
    );
    assert!(
        !deployed.contains("c.layer"),
        "no sidecar id may survive the deploy: {deployed}"
    );
}

/// The namespace rewrite never mangles unrelated text: a label that
/// already carries the DEPLOYED namespace stays as it is (it is never
/// re-prefixed), and an id that merely starts with the sidecar's id
/// (`c.layered.x`) is left alone (only `c.layer.` — the namespace form
/// — is re-keyed).
#[test]
fn deploy_leaves_text_outside_the_sidecar_namespace_untouched() {
    let sidecar = ERROR_SIDECAR.replace(
        "| boom_failure | effect | `c.layer.boom_failure(detail) — the boom stage failed because detail` | [[c.layer]] |",
        "| boom_failure | effect | `c.boom_failure(detail) — the boom stage failed because detail` | [[c.layer]] |",
    )
    .replace(
        "| k1 | invariant | `true` | [[c.layer]] |",
        "| k1 | invariant | `true` (the c.layered.x id is a different id) | [[c.layer]] |",
    );
    let root = archive_with_sidecar(&sidecar);
    let archive = root
        .path()
        .join("openspec/changes/archive/2026-11-12-errns");

    archive_companion::restore(root.path(), &archive).unwrap();
    let deployed = fs::read_to_string(root.path().join("openspec/specs/c/spec.md")).unwrap();
    // The already-deployed namespace is untouched — never re-prefixed.
    assert!(
        deployed.contains("`c.boom_failure(detail)"),
        "an already-deployed label must stay as it is: {deployed}"
    );
    assert!(
        !deployed.contains("c.c.boom_failure"),
        "an already-deployed label must never be re-prefixed: {deployed}"
    );
    // An id that merely starts with the sidecar id is left alone.
    assert!(
        deployed.contains("c.layered.x"),
        "an id outside the sidecar namespace must not be rewritten: {deployed}"
    );
}
