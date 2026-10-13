//! `spk skill install` — the managed skill-pack install verb
//! (add-skill-pack tasks.md §4, specodelic-pxh5; design D5/D6).
//!
//! Contract (openspec/changes/add-skill-pack specs/skill-pack):
//! default target `~/.claude/skills` (parent dirs created), `--dir`
//! override; overwrite refusal for a non-marker target file (exit 2,
//! hint names the path, file untouched); marker-bearing overwrite
//! succeeds; exit 0/1/2 per the error contract; envelope lists the
//! written paths and `pack_content_hash`.
//!
//! Hermeticity: every test points `HOME` and `XDG_STATE_HOME` at a
//! fresh temp dir so the default target and the recorded-targets state
//! file never touch the developer's real skill directories.

use super::*;

/// Run `spk skill install` with hermetic HOME + XDG_STATE_HOME.
fn skill_install(home: &std::path::Path, state: &std::path::Path, dir: Option<&str>) -> Command {
    let mut cmd = spk();
    cmd.env("HOME", home).env("XDG_STATE_HOME", state);
    cmd.args(["skill", "install", "--json"]);
    if let Some(d) = dir {
        cmd.args(["--dir", d]);
    }
    cmd
}

/// The pack directory the verb writes under a target dir.
fn pack_dir(target: &std::path::Path) -> std::path::PathBuf {
    target.join("specodelic-design")
}

#[test]
fn install_into_an_empty_dir_writes_the_pack_and_lists_paths_and_hash() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("skills");
    let out = skill_install(
        tmp.path(),
        &tmp.path().join("state"),
        Some(target.to_str().unwrap()),
    )
    .output()
    .unwrap();
    assert!(
        out.status.success(),
        "install into an empty dir exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Exactly the pack layout, written under the target dir.
    let expected = ["SKILL.md", "references/example.md", "references/format.md"];
    for rel in expected {
        let path = pack_dir(&target).join(rel);
        let content =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} must be written: {e}"));
        assert!(
            content.starts_with("<!-- MANAGED BY specodelic"),
            "{rel} must start with the managed marker"
        );
    }

    // The envelope lists exactly those paths plus the pack hash.
    let stdout = String::from_utf8_lossy(&out.stdout);
    let envelope: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout must be a JSON envelope: {e}\n{stdout}"));
    assert!(
        envelope["ok"].as_bool().unwrap_or(false),
        "envelope ok: {stdout}"
    );
    let data = &envelope["data"];
    let mut got_paths: Vec<String> = data["paths"]
        .as_array()
        .expect("envelope carries paths")
        .iter()
        .map(|p| p.as_str().unwrap().to_string())
        .collect();
    got_paths.sort();
    assert_eq!(
        got_paths, expected,
        "envelope lists exactly the written paths"
    );
    assert!(
        data["pack_content_hash"].is_string(),
        "envelope carries pack_content_hash: {stdout}"
    );
}

#[test]
fn default_target_is_claude_skills_under_home_with_parents_created() {
    let tmp = tempfile::tempdir().unwrap();
    let out = skill_install(tmp.path(), &tmp.path().join("state"), None)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "default-target install exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let expected = tmp.path().join(".claude/skills/specodelic-design/SKILL.md");
    assert!(
        expected.is_file(),
        "default target ~/.claude/skills is created with parents: {}",
        expected.display()
    );
}

#[test]
fn overwrite_refusal_for_a_non_marker_target_file() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("skills");
    let skill = pack_dir(&target).join("SKILL.md");
    std::fs::create_dir_all(skill.parent().unwrap()).unwrap();
    let foreign = "# my own notes\n";
    std::fs::write(&skill, foreign).unwrap();

    let out = skill_install(
        tmp.path(),
        &tmp.path().join("state"),
        Some(target.to_str().unwrap()),
    )
    .output()
    .unwrap();
    let code = out.status.code().unwrap_or(-1);
    assert_eq!(code, 2, "overwrite refusal exits 2");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains(&skill.to_string_lossy().to_string())
            || stderr.contains(&skill.to_string_lossy().to_string()),
        "the refusal must name the conflicting path: {stdout} | {stderr}"
    );
    assert_eq!(
        std::fs::read_to_string(&skill).unwrap(),
        foreign,
        "the foreign file must be left untouched"
    );
}

#[test]
fn marker_bearing_overwrite_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("skills");
    let first = skill_install(
        tmp.path(),
        &tmp.path().join("state"),
        Some(target.to_str().unwrap()),
    )
    .output()
    .unwrap();
    assert!(first.status.success(), "first install exits 0");
    let second = skill_install(
        tmp.path(),
        &tmp.path().join("state"),
        Some(target.to_str().unwrap()),
    )
    .output()
    .unwrap();
    assert!(
        second.status.success(),
        "re-installing over marker-bearing files succeeds: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    let content = std::fs::read_to_string(pack_dir(&target).join("SKILL.md")).unwrap();
    assert!(
        content.starts_with("<!-- MANAGED BY specodelic"),
        "the overwritten file is still marker-bearing"
    );
}

#[test]
fn write_failure_exits_1() {
    let tmp = tempfile::tempdir().unwrap();
    // A plain file where the target directory must be created — the
    // write cannot succeed; the error contract calls it exit 1.
    let blocker = tmp.path().join("blocker");
    std::fs::write(&blocker, "not a directory").unwrap();
    let out = skill_install(
        tmp.path(),
        &tmp.path().join("state"),
        Some(blocker.to_str().unwrap()),
    )
    .output()
    .unwrap();
    assert_eq!(
        out.status.code().unwrap_or(-1),
        1,
        "a write failure exits 1: {} | {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn invalid_invocation_exits_2() {
    let out = spk()
        .env("HOME", tempfile::tempdir().unwrap().path())
        .args(["skill", "frobnicate"])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code().unwrap_or(-1),
        2,
        "an unknown skill action is an invalid invocation (exit 2)"
    );
}

// ---- phase 5 (specodelic-p94h, tasks.md §5): doctor staleness check ----

/// The `managed skill pack` payload row from `spk doctor`.
fn doctor_pack_row(
    home: &std::path::Path,
    state: &std::path::Path,
    cwd: &std::path::Path,
) -> (i32, String) {
    let out = spk()
        .env("HOME", home)
        .env("XDG_STATE_HOME", state)
        .args(["doctor", "--json"])
        .current_dir(cwd)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let envelope: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("doctor must emit JSON: {e}\n{stdout}"));
    let row = envelope["data"]["checks"]
        .as_array()
        .expect("doctor payload carries checks")
        .iter()
        .find(|r| r[0].as_str() == Some("managed skill pack"))
        .unwrap_or_else(|| panic!("doctor payload carries a managed skill pack row: {envelope}"))
        [1]
    .as_str()
    .unwrap()
    .to_string();
    (out.status.code().unwrap_or(-1), row)
}

#[test]
fn doctor_reports_missing_when_no_pack_is_installed() {
    let tmp = tempfile::tempdir().unwrap();
    let (code, detail) = doctor_pack_row(tmp.path(), &tmp.path().join("state"), tmp.path());
    assert_eq!(
        code, 0,
        "a missing pack must not change the doctor exit code"
    );
    assert!(
        detail.contains("missing"),
        "the row names the missing state: {detail}"
    );
    assert!(
        detail.contains("spk skill install"),
        "the missing row carries the install hint: {detail}"
    );
}

#[test]
fn doctor_reports_pass_over_a_fresh_install() {
    let tmp = tempfile::tempdir().unwrap();
    let out = skill_install(tmp.path(), &tmp.path().join("state"), None)
        .output()
        .unwrap();
    assert!(out.status.success());
    let (code, detail) = doctor_pack_row(tmp.path(), &tmp.path().join("state"), tmp.path());
    assert_eq!(code, 0);
    assert!(
        detail.contains("current") || detail.contains("pass"),
        "a fresh install is current: {detail}"
    );
}

#[test]
fn doctor_reports_stale_with_both_hashes_and_regenerate_command() {
    let tmp = tempfile::tempdir().unwrap();
    let out = skill_install(tmp.path(), &tmp.path().join("state"), None)
        .output()
        .unwrap();
    assert!(out.status.success());
    // Simulate an older-binary install: mutate the installed pack file.
    let skill = tmp.path().join(".claude/skills/specodelic-design/SKILL.md");
    let content = std::fs::read_to_string(&skill).unwrap();
    std::fs::write(&skill, format!("{content}\nstale mutation\n")).unwrap();
    let (code, detail) = doctor_pack_row(tmp.path(), &tmp.path().join("state"), tmp.path());
    assert_eq!(code, 0, "stale is reported but never fails the doctor run");
    assert!(
        detail.contains("stale"),
        "the row names the stale state: {detail}"
    );
    assert!(
        detail.contains("spk skill install"),
        "the stale row carries the regenerate command: {detail}"
    );
    // Both hashes are named and they differ.
    let hashes: Vec<&str> = detail
        .split_whitespace()
        .filter(|w| w.len() == 64 && w.chars().all(|c| c.is_ascii_hexdigit()))
        .collect();
    assert!(
        hashes.len() >= 2,
        "installed + expected hashes are named: {detail}"
    );
    assert_ne!(hashes[0], hashes[1], "the two hashes differ: {detail}");
}

#[test]
fn mutated_non_pack_files_do_not_flip_pass_to_stale() {
    let tmp = tempfile::tempdir().unwrap();
    let out = skill_install(tmp.path(), &tmp.path().join("state"), None)
        .output()
        .unwrap();
    assert!(out.status.success());
    // A foreign file dropped next to the pack must not count.
    std::fs::write(
        tmp.path()
            .join(".claude/skills/specodelic-design/notes.txt"),
        "unrelated\n",
    )
    .unwrap();
    let (code, detail) = doctor_pack_row(tmp.path(), &tmp.path().join("state"), tmp.path());
    assert_eq!(code, 0);
    assert!(
        detail.contains("current") || detail.contains("pass"),
        "non-pack files must not flip pass to stale: {detail}"
    );
}

#[test]
fn doctor_rechecks_recorded_dir_targets() {
    let tmp = tempfile::tempdir().unwrap();
    let alt = tmp.path().join("alt-skills");
    let out = skill_install(
        tmp.path(),
        &tmp.path().join("state"),
        Some(alt.to_str().unwrap()),
    )
    .output()
    .unwrap();
    assert!(out.status.success());
    let (code, detail) = doctor_pack_row(tmp.path(), &tmp.path().join("state"), tmp.path());
    assert_eq!(code, 0);
    assert!(
        detail.contains(&alt.to_string_lossy().to_string()),
        "the --dir target is re-checked by the doctor: {detail}"
    );
    assert!(
        detail.contains("current") || detail.contains("pass"),
        "the recorded install is current: {detail}"
    );
}
