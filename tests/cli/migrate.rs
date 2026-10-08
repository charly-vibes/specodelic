// `spk migrate` CLI integration tests (split from parse_misc.rs — the
// file_lines ratchet; mixed-delta coverage specodelic-54v).
use super::*;

/// (specodelic-54v) A mixed delta (ADDED + MODIFIED) migrates and the
/// migrated file lints clean — the mirror aggregates both bodies
/// per-requirement, matching linter.requirement_drift (CHANGELOG #93).
/// Regression: migrate used to mirror the ADDED body only, so the fresh
/// file failed lint immediately (migrate exit 0 → lint exit 1).
#[test]
fn migrate_mixed_delta_lints_clean() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("spec.md");
    std::fs::write(
        &file,
        "## ADDED Requirements\n\n### Requirement: One\none holds\n\n## MODIFIED Requirements\n\n### Requirement: Two\ntwo holds\n",
    )
    .unwrap();
    spk().arg("migrate").arg(&file).assert().success();
    let migrated = std::fs::read_to_string(&file).unwrap();
    assert!(migrated.contains("## Requirements\n\n### Requirement: One"));
    assert!(migrated.contains("### Requirement: Two"));
    // The freshly migrated file must not immediately fail lint.
    spk()
        .arg("lint")
        .arg(&file)
        .assert()
        .success()
        .stdout(contains("\"issues\":[]"));
}

/// (specodelic-54v) The same requirement heading in both ADDED and
/// MODIFIED cannot be mirrored faithfully (one heading, two texts) —
/// migrate refuses labeled with a remediation hint, file untouched.
#[test]
fn migrate_duplicate_requirement_heading_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("spec.md");
    let original = "## ADDED Requirements\n\n### Requirement: One\none holds\n\n## MODIFIED Requirements\n\n### Requirement: One\nmodified holds\n";
    std::fs::write(&file, original).unwrap();
    spk()
        .arg("migrate")
        .arg(&file)
        .assert()
        .failure()
        .stdout(contains("requirement `One` appears in both"))
        .stdout(contains("linter.requirement_drift"));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), original);
}
