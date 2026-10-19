// `spk migrate` CLI integration tests (split from parse_misc.rs — the
// file_lines ratchet; mixed-delta coverage specodelic-54v).
use super::*;

/// (Revision 19, add-delta-layer-sidecar) A mixed delta (ADDED +
/// MODIFIED) wraps into the pair: the delta stays byte-plain, the
/// sibling layer.md carries the layer, and the SIDECAIR lints clean.
/// Regression: migrate used to rewrite the delta with a mirror — the
/// mirror retired with the dual-delta class.
#[test]
fn migrate_mixed_delta_wraps_sidecar_pair() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("spec.md");
    let original = "## ADDED Requirements\n\n### Requirement: One\none holds\n\n## MODIFIED Requirements\n\n### Requirement: Two\ntwo holds\n";
    std::fs::write(&file, original).unwrap();
    spk().arg("migrate").arg(&file).assert().success();
    // The delta file is untouched.
    assert_eq!(std::fs::read_to_string(&file).unwrap(), original);
    // The sibling sidecar exists with the parent-dir + .layer id and the
    // layer skeleton (no mirror, no delta grammar).
    let sidecar = std::fs::read_to_string(dir.path().join("layer.md")).unwrap();
    assert!(
        sidecar.contains("id: .layer") || sidecar.contains("id: "),
        "{sidecar}"
    );
    assert!(!sidecar.contains("## Requirements"));
    // The freshly generated sidecar must not immediately fail lint.
    spk()
        .arg("lint")
        .arg(dir.path().join("layer.md"))
        .assert()
        .success()
        .stdout(contains("\"issues\":[]"));
}

/// (specodelic-nvs9, gh-16) A MODIFIED-only delta — no ## ADDED
/// Requirements section — wraps like any other delta: the delta stays
/// byte-plain, the sidecar carries the layer (Revision 19 — there is no
/// mirror to hold the MODIFIED body).
#[test]
fn migrate_modified_only_delta_wraps_sidecar() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("spec.md");
    let original = "## MODIFIED Requirements\n\n### Requirement: One\nmodified holds\n";
    std::fs::write(&file, original).unwrap();
    spk().arg("migrate").arg(&file).assert().success();
    // The delta body is preserved byte-for-byte.
    assert_eq!(std::fs::read_to_string(&file).unwrap(), original);
    // The sidecar exists; the delta grammar stays in the delta.
    let sidecar = std::fs::read_to_string(dir.path().join("layer.md")).unwrap();
    assert!(sidecar.contains("## Constraints"));
    assert!(!sidecar.contains("## MODIFIED Requirements"));
    // The freshly generated sidecar must not immediately fail lint.
    spk()
        .arg("lint")
        .arg(dir.path().join("layer.md"))
        .assert()
        .success()
        .stdout(contains("\"issues\":[]"));
}

/// (specodelic-nvs9) A file with NEITHER an ADDED nor a MODIFIED section
/// is still refused NoDeltaSection — and the message names both accepted
/// delta sections so the remediation hint matches the widened contract.
#[test]
fn migrate_no_delta_section_refusal_names_both_sections() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("spec.md");
    let original = "# Just prose\n\nSome text.\n";
    std::fs::write(&file, original).unwrap();
    spk()
        .arg("migrate")
        .arg(&file)
        .assert()
        .failure()
        .stdout(contains(
            "## ADDED Requirements or ## MODIFIED Requirements",
        ));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), original);
}

/// (Revision 19) The old mirror-conflict refusal retired with the
/// mirror: the same requirement heading in both ADDED and MODIFIED is
/// now just a delta that carries both — the pair wraps cleanly.
#[test]
fn migrate_duplicate_requirement_heading_wraps_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("spec.md");
    let original = "## ADDED Requirements\n\n### Requirement: One\none holds\n\n## MODIFIED Requirements\n\n### Requirement: One\nmodified holds\n";
    std::fs::write(&file, original).unwrap();
    spk().arg("migrate").arg(&file).assert().success();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), original);
    assert!(dir.path().join("layer.md").exists());
}

/// (specodelic-7i7) A bare `## ADDED Requirements` heading — no
/// trailing newline, no body — is refused labeled with a remediation
/// hint (never a panic), and the file is untouched.
#[test]
fn migrate_degenerate_one_line_delta_is_refused_labeled() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("spec.md");
    let original = "## ADDED Requirements";
    std::fs::write(&file, original).unwrap();
    spk()
        .arg("migrate")
        .arg(&file)
        .assert()
        .failure()
        .stdout(contains("no trailing newline or body"))
        .stdout(contains("### Requirement:"));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), original);
}

// ---------- --rekey (add-migrate-rekey) ----------

/// A 0.6.0-era `id: spec` dual-format file rekeys to its real
/// parent-dir-derived id; refs re-key; the file lints clean after.
#[test]
fn rekey_rewrites_legacy_dual_format_file() {
    let dir = tempfile::tempdir().unwrap();
    let cap = dir.path().join("ge-cli");
    std::fs::create_dir_all(&cap).unwrap();
    let file = cap.join("spec.md");
    let legacy = "---\nid: spec\nkind: intent\nstatement: \"THE ge cli SHALL hold\"\n---\n\n\
         ## Constraints\n\n\
         | id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | c1 | invariant | `x` | [[spec]] |\n\
         | c2 | invariant | `y` | [[spec.c1]] |\n\n\
         ## Model\n\n\
         ### States\n\n- `s1`\n\n\
         ### Transitions\n\n\
         | id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | t | s1 | s1 | [[spec.c1]] |\n\n\
         ## Properties\n\n\
         | id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|------------|\n\
         | p1 | unit | [[spec.c1]] | `g()` | `x` |\n\
         | p2 | unit | [[spec.c2]] | `g()` | `y` |\n\n\
         ## Requirements\n\n\
         ### Requirement: Ge cli\nThe ge cli SHALL hold.\n";
    std::fs::write(&file, legacy).unwrap();
    spk()
        .args(["migrate", file.to_str().unwrap(), "--rekey"])
        .assert()
        .success()
        .stdout(contains("\"rekeyed\":true"));
    let rekeyed = std::fs::read_to_string(&file).unwrap();
    assert!(rekeyed.contains("id: ge.cli\n"), "{rekeyed}");
    assert!(rekeyed.contains("[[ge.cli]]") && rekeyed.contains("[[ge.cli.c1]]"));
    assert!(!rekeyed.contains("[[spec"));
    // The rekeyed file lints clean (single-file lint resolves own rows).
    spk().arg("lint").arg(&file).assert().success();
    // Idempotence: second run is a warning no-op, disk unchanged.
    spk()
        .args(["migrate", file.to_str().unwrap(), "--rekey"])
        .assert()
        .success()
        .stdout(contains("\"rekeyed\":false"))
        .stdout(contains("nothing to re-key"));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), rekeyed);
}

/// Refusals never rewrite: a plain delta takes the wrap path; a bare
/// spec.md has no derivable id.
#[test]
fn rekey_refusals_never_rewrite() {
    let dir = tempfile::tempdir().unwrap();
    // Plain delta (no frontmatter) → wrap path's job.
    let plain = dir.path().join("plain.md");
    let delta = "## ADDED Requirements\n\n### Requirement: W\nThe system SHALL w.\n";
    std::fs::write(&plain, delta).unwrap();
    spk()
        .args(["migrate", plain.to_str().unwrap(), "--rekey"])
        .assert()
        .failure()
        .stdout(contains("must already be dual-format"))
        .stdout(contains("the wrap path"));
    assert_eq!(std::fs::read_to_string(&plain).unwrap(), delta);
    // A spec.md under a directory literally named `spec`: the derived id
    // would still be `spec` — nothing to re-key to, refusal beats a
    // self-referential rewrite.
    let spec_dir = dir.path().join("spec");
    std::fs::create_dir_all(&spec_dir).unwrap();
    let bare = spec_dir.join("spec.md");
    let legacy = "---\nid: spec\nkind: intent\nstatement: \"THE x SHALL hold\"\n---\n";
    std::fs::write(&bare, legacy).unwrap();
    spk()
        .args(["migrate", bare.to_str().unwrap(), "--rekey"])
        .assert()
        .failure()
        .stdout(contains("cannot be derived"));
    assert_eq!(std::fs::read_to_string(&bare).unwrap(), legacy);
}

/// --dry-run emits the would-be content, disk untouched.
#[test]
fn rekey_dry_run_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let cap = dir.path().join("demo-cap");
    std::fs::create_dir_all(&cap).unwrap();
    let file = cap.join("spec.md");
    let legacy = "---\nid: spec\nkind: intent\nstatement: \"THE demo SHALL hold\"\n---\n";
    std::fs::write(&file, legacy).unwrap();
    spk()
        .args(["migrate", file.to_str().unwrap(), "--rekey", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("\"dry_run\":true"))
        .stdout(contains("id: demo.cap"));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), legacy);
}
