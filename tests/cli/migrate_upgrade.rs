// `spk migrate --to-revision` CLI integration tests (add-revision-upgrade) —
// the corpus revision-upgrade path: a pre-Revision-18 corpus file is
// carried across the known revisions by the registered steps, every file
// that needs no step stays byte-identical, and the refusals (unknown
// revision, target below the current revision) never write.
use super::*;
use crate::common::UnwrapPanic;

/// The format revision the binary embeds — the registry's latest
/// revision (the corpus corpus is upgraded to it).
fn current_revision() -> u32 {
    let out = spk().arg("--version").arg("--json").output().unwrap_p();
    let text = String::from_utf8(out.stdout).unwrap_p();
    let marker = "Revision ";
    let at = text.find(marker).unwrap_p() + marker.len();
    text[at..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_p()
}

/// A pre-Revision-18 corpus file — the 0.6.0-era dual-format shape:
/// frontmatter `id: spec` with `[[spec…]]` refs (the shape Revision 18
/// retired). The id law derives the real id from the parent directory,
/// so this file upgrades to `legacy.cap`.
fn legacy_cap() -> String {
    "---\nid: spec\nkind: intent\nstatement: \"THE legacy cap SHALL hold\"\n---\n\n\
     ## Constraints\n\n\
     | id | kind | expr | traces_to |\n\
     |----|------|------|-----------|\n\
     | c1 | invariant | `holds` | [[spec]] |\n\
     | c2 | invariant | `also holds` | [[spec]] |\n\n\
     ## Model\n\n\
     ### States\n\n- `s1`\n\n\
     ### Transitions\n\n\
     | id | from | to | guard |\n\
     |----|------|----|-------|\n\
     | t1 | s1 | s1 | [[spec.c1]] |\n\n\
     ## Properties\n\n\
     | id | kind | derives_from | generator | predicate |\n\
     |----|------|--------------|-----------|-----------|\n\
     | p1 | unit | [[spec.c1]] | `g()` | `holds` |\n\
     | p2 | unit | [[spec.c2]] | `h()` | `also holds` |\n\n\
     ## Requirements\n\n\
     ### Requirement: Legacy cap\nThe legacy cap SHALL hold.\n"
        .to_string()
}

/// A corpus file that needs no step: a non-`spec.md` file's id was
/// stem-derived before Revision 18 too, so this file is already at the
/// target's shape. Its bytes (CRLF line endings, no final terminator)
/// must survive the upgrade untouched.
fn stable_notes() -> String {
    "---\r\nid: notes\r\nkind: intent\r\nstatement: \"THE notes SHALL hold\"\r\n---\r\n\r\n\
     ## Constraints\r\n\r\n\
     | id | kind | expr | traces_to |\r\n\
     |----|------|------|-----------|\r\n\
     | c1 | invariant | `holds` | [[notes]] |\r\n\r\n\
     ## Model\r\n\r\n\
     ### States\r\n\r\n- `s1`\r\n\r\n\
     ### Transitions\r\n\r\n\
     | id | from | to | guard |\r\n\
     |----|------|----|-------|\r\n\
     | t1 | s1 | s1 | [[notes.c1]] |\r\n\r\n\
     ## Properties\r\n\r\n\
     | id | kind | derives_from | generator | predicate |\r\n\
     |----|------|--------------|-----------|-----------|\r\n\
     | p1 | unit | [[notes.c1]] | `g()` | `holds` |\r\n\
     ## Requirements\r\n\r\n\
     ### Requirement: Notes\r\nThe notes SHALL hold."
        .to_string()
}

/// A corpus authored at Revision 17: one capability file still carrying
/// the pre-Revision-18 shape (the registered step transforms it) and one
/// file that needs no step (its bytes must survive untouched).
fn write_corpus(root: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let cap = root.join("legacy-cap");
    std::fs::create_dir_all(&cap).unwrap_p();
    let legacy = cap.join("spec.md");
    std::fs::write(&legacy, legacy_cap()).unwrap_p();
    let notes = cap.join("notes.md");
    std::fs::write(&notes, stable_notes()).unwrap_p();
    (legacy, notes)
}

/// The corpus fixture upgrades to the current revision: the lagging
/// capability file is carried across the registered step, the file that
/// needs no step stays byte-identical, and the upgraded corpus lints
/// with zero findings.
#[test]
fn upgrade_old_corpus_upgrades_to_target_revision() {
    let dir = tempfile::tempdir().unwrap_p();
    let (legacy, notes) = write_corpus(dir.path());
    let notes_bytes = std::fs::read(&notes).unwrap_p();
    let target = current_revision();
    spk()
        .args([
            "migrate",
            "--to-revision",
            &target.to_string(),
            "--write",
            dir.path().to_str().unwrap_p(),
        ])
        .assert()
        .success()
        .stdout(contains("\"source_revision\":17"))
        .stdout(contains("\"target_revision\":21"))
        .stdout(contains("\"files_changed\":1"));
    let upgraded = std::fs::read_to_string(&legacy).unwrap_p();
    assert!(upgraded.contains("id: legacy.cap\n"), "{upgraded}");
    assert!(upgraded.contains("[[legacy.cap.c1]]"), "{upgraded}");
    assert!(!upgraded.contains("[[spec"), "{upgraded}");
    // The file that needed no step is byte-identical — CRLF endings and
    // the absent final terminator survive.
    assert_eq!(std::fs::read(&notes).unwrap_p(), notes_bytes);
    // The upgraded corpus lints clean.
    spk()
        .arg("lint")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(contains("\"issues\":[]"));
}

/// The upgrade is a dry run by default: the disk is untouched and the
/// envelope carries the would-be content.
#[test]
fn upgrade_dry_run_writes_nothing() {
    let dir = tempfile::tempdir().unwrap_p();
    let (legacy, _) = write_corpus(dir.path());
    let before = std::fs::read(&legacy).unwrap_p();
    let target = current_revision();
    spk()
        .args([
            "migrate",
            "--to-revision",
            &target.to_string(),
            dir.path().to_str().unwrap_p(),
        ])
        .assert()
        .success()
        .stdout(contains("\"dry_run\":true"))
        .stdout(contains("legacy.cap"));
    assert_eq!(std::fs::read(&legacy).unwrap_p(), before);
}

/// The upgrade reads and writes only the files under the supplied paths:
/// a legacy file outside them is untouched, and no file is created
/// beside the upgraded ones.
#[test]
fn upgrade_never_touches_files_outside_supplied_paths() {
    let dir = tempfile::tempdir().unwrap_p();
    let corpus = dir.path().join("corpus");
    let (legacy, _) = write_corpus(&corpus);
    let outside_dir = dir.path().join("outside");
    std::fs::create_dir_all(&outside_dir).unwrap_p();
    let outside = outside_dir.join("spec.md");
    std::fs::write(&outside, legacy_cap()).unwrap_p();
    let outside_bytes = std::fs::read(&outside).unwrap_p();
    let target = current_revision();
    spk()
        .args([
            "migrate",
            "--to-revision",
            &target.to_string(),
            "--write",
            corpus.to_str().unwrap_p(),
        ])
        .assert()
        .success();
    // The file outside the supplied path is untouched — it still carries
    // the pre-Revision-18 shape.
    assert_eq!(std::fs::read(&outside).unwrap_p(), outside_bytes);
    assert!(
        std::fs::read_to_string(&legacy)
            .unwrap_p()
            .contains("id: legacy.cap")
    );
    // No file is created beside the upgraded ones.
    let names: Vec<String> = std::fs::read_dir(corpus.join("legacy-cap"))
        .unwrap_p()
        .map(|e| e.unwrap_p().file_name().to_string_lossy().to_string())
        .collect();
    assert!(!names.iter().any(|n| n == "layer.md"), "{names:?}");
}

/// An unknown target revision is refused with the known range and a
/// remediation hint — and nothing is written.
#[test]
fn upgrade_unknown_target_revision_is_refused() {
    let dir = tempfile::tempdir().unwrap_p();
    let (legacy, _) = write_corpus(dir.path());
    let before = std::fs::read(&legacy).unwrap_p();
    spk()
        .args([
            "migrate",
            "--to-revision",
            "999",
            "--write",
            dir.path().to_str().unwrap_p(),
        ])
        .assert()
        .failure()
        .code(2)
        .stdout(contains("999"))
        .stdout(contains("2 to 21"))
        .stdout(contains("--version --json"));
    assert_eq!(std::fs::read(&legacy).unwrap_p(), before);
}

/// An unknown source revision is refused the same way.
#[test]
fn upgrade_unknown_source_revision_is_refused() {
    let dir = tempfile::tempdir().unwrap_p();
    let (legacy, _) = write_corpus(dir.path());
    let before = std::fs::read(&legacy).unwrap_p();
    spk()
        .args([
            "migrate",
            "--to-revision",
            "21",
            "--from-revision",
            "999",
            "--write",
            dir.path().to_str().unwrap_p(),
        ])
        .assert()
        .failure()
        .code(2)
        .stdout(contains("999"))
        .stdout(contains("2 to 21"));
    assert_eq!(std::fs::read(&legacy).unwrap_p(), before);
}

/// A target below the corpus's current revision is refused — the message
/// names the current revision and the file that still lags — and
/// nothing is written.
#[test]
fn upgrade_target_below_current_revision_is_refused() {
    let dir = tempfile::tempdir().unwrap_p();
    let (legacy, _) = write_corpus(dir.path());
    let before = std::fs::read(&legacy).unwrap_p();
    spk()
        .args([
            "migrate",
            "--to-revision",
            "16",
            "--write",
            dir.path().to_str().unwrap_p(),
        ])
        .assert()
        .failure()
        .code(2)
        .stdout(contains("17"))
        .stdout(contains("spec.md"));
    assert_eq!(std::fs::read(&legacy).unwrap_p(), before);
}

/// A file whose id cannot be derived refuses the whole upgrade: the
/// file that would have changed stays byte-identical.
#[test]
fn upgrade_failed_upgrade_writes_nothing() {
    let dir = tempfile::tempdir().unwrap_p();
    let (legacy, _) = write_corpus(dir.path());
    let legacy_before = std::fs::read(&legacy).unwrap_p();
    // A spec.md under a directory named `spec` — the derived id would be
    // `spec` back, so there is nothing to derive.
    let bad_dir = dir.path().join("spec");
    std::fs::create_dir_all(&bad_dir).unwrap_p();
    let bad = bad_dir.join("spec.md");
    std::fs::write(&bad, legacy_cap()).unwrap_p();
    let bad_before = std::fs::read(&bad).unwrap_p();
    let target = current_revision();
    spk()
        .args([
            "migrate",
            "--to-revision",
            &target.to_string(),
            "--write",
            dir.path().to_str().unwrap_p(),
        ])
        .assert()
        .failure()
        .code(2)
        .stdout(contains("cannot be derived"));
    assert_eq!(std::fs::read(&legacy).unwrap_p(), legacy_before);
    assert_eq!(std::fs::read(&bad).unwrap_p(), bad_before);
}

/// The declared source revision drives the upgrade: `--from-revision 17`
/// applies the registered step across Revision 18 even though the
/// corpus's own detection agrees.
#[test]
fn upgrade_declared_source_revision_is_applied() {
    let dir = tempfile::tempdir().unwrap_p();
    let (legacy, _) = write_corpus(dir.path());
    spk()
        .args([
            "migrate",
            "--to-revision",
            "18",
            "--from-revision",
            "17",
            "--write",
            dir.path().to_str().unwrap_p(),
        ])
        .assert()
        .success()
        .stdout(contains("\"source_revision\":17"))
        .stdout(contains("\"target_revision\":18"));
    assert!(
        std::fs::read_to_string(&legacy)
            .unwrap_p()
            .contains("id: legacy.cap")
    );
}

/// A file that needs no step is byte-identical after the upgrade — its
/// CRLF line endings and its absent final terminator survive the run
/// untouched.
#[test]
fn upgrade_unchanged_files_are_byte_identical() {
    let dir = tempfile::tempdir().unwrap_p();
    let (_, notes) = write_corpus(dir.path());
    let before = std::fs::read(&notes).unwrap_p();
    let target = current_revision();
    spk()
        .args([
            "migrate",
            "--to-revision",
            &target.to_string(),
            "--write",
            dir.path().to_str().unwrap_p(),
        ])
        .assert()
        .success()
        .stdout(contains("\"files_changed\":1"));
    assert_eq!(std::fs::read(&notes).unwrap_p(), before);
    // The CRLF endings and the absent final terminator are still there.
    let after = std::fs::read(&notes).unwrap_p();
    assert!(after.windows(2).any(|w| w == b"\r\n"), "CRLF endings lost");
    assert!(!after.ends_with(b"\n"), "a final terminator appeared");
}

/// The two modes of the verb conflict: `--rekey` is the Revision 18
/// re-key, `--to-revision` is the corpus upgrade. Silently ignoring one
/// of them is a trap, so the combination is refused and nothing is
/// written.
#[test]
fn upgrade_rekey_and_to_revision_conflict_is_refused() {
    let dir = tempfile::tempdir().unwrap_p();
    let (legacy, _) = write_corpus(dir.path());
    let before = std::fs::read(&legacy).unwrap_p();
    let target = current_revision();
    spk()
        .args([
            "migrate",
            "--rekey",
            "--to-revision",
            &target.to_string(),
            "--write",
            dir.path().to_str().unwrap_p(),
        ])
        .assert()
        .failure()
        .code(2)
        .stdout(contains("--rekey"));
    assert_eq!(std::fs::read(&legacy).unwrap_p(), before);
}

/// `--write` and `--dry-run` conflict too — the upgrade previews unless
/// the write is asked for explicitly.
#[test]
fn upgrade_write_and_dry_run_conflict_is_refused() {
    let dir = tempfile::tempdir().unwrap_p();
    let (legacy, _) = write_corpus(dir.path());
    let before = std::fs::read(&legacy).unwrap_p();
    let target = current_revision();
    spk()
        .args([
            "migrate",
            "--to-revision",
            &target.to_string(),
            "--write",
            "--dry-run",
            dir.path().to_str().unwrap_p(),
        ])
        .assert()
        .failure()
        .code(2)
        .stdout(contains("--write"));
    assert_eq!(std::fs::read(&legacy).unwrap_p(), before);
}

/// The upgrade is idempotent: a second run over the upgraded corpus
/// changes nothing.
#[test]
fn upgrade_second_run_changes_nothing() {
    let dir = tempfile::tempdir().unwrap_p();
    let (legacy, notes) = write_corpus(dir.path());
    let target = current_revision();
    let args = [
        "migrate",
        "--to-revision",
        &target.to_string(),
        "--write",
        dir.path().to_str().unwrap_p(),
    ];
    spk().args(args).assert().success();
    let upgraded = std::fs::read(&legacy).unwrap_p();
    let notes_bytes = std::fs::read(&notes).unwrap_p();
    spk()
        .args(args)
        .assert()
        .success()
        .stdout(contains("\"files_changed\":0"));
    assert_eq!(std::fs::read(&legacy).unwrap_p(), upgraded);
    assert_eq!(std::fs::read(&notes).unwrap_p(), notes_bytes);
}
