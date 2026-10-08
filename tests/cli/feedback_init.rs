// (split from tests/cli.rs — specodelic-g17 file_lines ratchet)
use super::*;

// ---- specodelic-ze4: spk feedback + spk init (managed block) ----

#[test]
fn feedback_dry_run_previews_issue_without_filing() {
    // dry-run must print the redacted body + the would-file command,
    // never invoke gh, and exit 0; content comes from piped stdin
    // (genesis handle_feedback contract) — --from-last-error is the
    // other source
    let dir = tempfile::tempdir().unwrap();
    let out = spk()
        .args(["feedback", "bug", "--dry-run"])
        .current_dir(dir.path())
        .env("NO_COLOR", "1")
        .write_stdin("spk lint crashes on empty dirs\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "dry-run never fails");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Would file"), "preview: {stderr}");
    assert!(
        stderr.contains("charly-vibes/specodelic"),
        "target repo: {stderr}"
    );
}

#[test]
fn feedback_title_flag_overrides_derived_title() {
    // genesis 0.8 FeedbackArgs.title: a user-supplied --title must override
    // the stdin-derived title and land verbatim in the would-file command
    let dir = tempfile::tempdir().unwrap();
    let out = spk()
        .args([
            "feedback",
            "bug",
            "--dry-run",
            "--title",
            "lint crashes on empty dirs",
        ])
        .current_dir(dir.path())
        .env("NO_COLOR", "1")
        .write_stdin("body text\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "dry-run never fails");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("--title \"[bug] lint crashes on empty dirs\""),
        "supplied title in would-file command: {stderr}"
    );
}

#[test]
fn feedback_rejects_unknown_kind_with_hint() {
    let out = spk().args(["feedback", "bugg"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("bug"), "valid kinds suggested: {stderr}");
}

#[test]
fn init_injects_specodelic_block_into_agents_md() {
    // spk init writes/refreshes a <!-- SPECODELIC:START/END --> block in
    // AGENTS.md carrying the lint rule catalog + format_revision
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("AGENTS.md"), "# My repo\n\nAgent notes.\n").unwrap();
    let out = spk()
        .args(["init", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let agents = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(
        agents.contains("<!-- SPECODELIC:START -->"),
        "block injected: {agents}"
    );
    assert!(agents.contains("<!-- SPECODELIC:END -->"));
    // existing content is preserved (block prepends, never replaces)
    assert!(agents.contains("# My repo"));
    assert!(agents.contains("Agent notes."));
    // block content is self-describing: rule catalog + revision + commands
    assert!(agents.contains("linter.ears_syntax"));
    assert!(agents.contains("linter.frontmatter_valid"));
    assert!(agents.contains("specodelic.md Revision 17"));
    assert!(agents.contains("spk lint"));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["data"]["block"], "injected");
    assert_eq!(json["data"]["file"], "AGENTS.md");
}

#[test]
fn init_updates_stale_block_in_place() {
    // second run updates the block in place: content between the markers
    // is refreshed, surrounding text preserved, exactly one block
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("AGENTS.md"), "# My repo\n\n<!-- SPECODELIC:START -->\nstale specodelic content\n<!-- SPECODELIC:END -->\n").unwrap();
    let out = spk()
        .args(["init", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let agents = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(
        !agents.contains("stale specodelic content"),
        "refreshed: {agents}"
    );
    assert!(agents.contains("# My repo"));
    assert_eq!(agents.matches("SPECODELIC:START").count(), 1);
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["data"]["block"], "updated");
}

#[test]
fn init_creates_agents_md_when_missing() {
    let dir = tempfile::tempdir().unwrap();
    let out = spk()
        .args(["init", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let agents = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(agents.contains("<!-- SPECODELIC:START -->"));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["data"]["block"], "created");
}

// ---- specodelic-las: init writes the .genesis/tools.toml manifest ----

#[test]
fn init_registers_in_genesis_tools_manifest() {
    // spk init declares specodelic's presence in .genesis/tools.toml so
    // orchestrators discover it without hardcoding (genesis::discovery)
    let dir = tempfile::tempdir().unwrap();
    spk()
        .args(["init", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let manifest = std::fs::read_to_string(dir.path().join(".genesis/tools.toml")).unwrap();
    assert!(
        manifest.contains("[tools.specodelic]"),
        "manifest: {manifest}"
    );
    assert!(
        manifest.contains("AGENTS.md"),
        "detector names the marker: {manifest}"
    );
}

#[test]
fn init_manifest_registration_is_idempotent_and_merges() {
    let dir = tempfile::tempdir().unwrap();
    // a sibling tool registered first (hand-written manifest)
    std::fs::create_dir_all(dir.path().join(".genesis")).unwrap();
    std::fs::write(
        dir.path().join(".genesis/tools.toml"),
        "[tools.wai]\ndescription = \"Workflow manager\"\ndetector = { type = \"directory\", path = \".wai\" }\n",
    )
    .unwrap();
    for _ in 0..2 {
        let out = spk()
            .args(["init", "--json"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0));
    }
    let manifest = std::fs::read_to_string(dir.path().join(".genesis/tools.toml")).unwrap();
    assert!(
        manifest.contains("[tools.wai]"),
        "sibling intact: {manifest}"
    );
    assert_eq!(
        manifest.matches("[tools.specodelic]").count(),
        1,
        "no duplicate entries: {manifest}"
    );
}

#[test]
fn init_on_unwritable_genesis_dir_still_succeeds_with_warning() {
    // ANTI-GOAL: init never fails on an unwritable .genesis beyond a
    // warnings-channel note — the AGENTS.md block is the primary payload
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".genesis"), "not a directory\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            dir.path().join(".genesis"),
            std::fs::Permissions::from_mode(0o444),
        )
        .unwrap();
    }
    let out = spk()
        .args(["init", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "init succeeds regardless");
    let agents = std::fs::read_to_string(dir.path().join("AGENTS.md"));
    assert!(agents.is_ok(), "AGENTS.md block still written");
}

#[test]
fn doctor_reports_missing_or_stale_block() {
    // doctor: fresh block → silent ok; missing block → hint to run spk init
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("AGENTS.md"), "# My repo\n").unwrap();
    let out = spk()
        .args(["doctor", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "advisory, never fails");
    let stdout_str = String::from_utf8(out.stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout_str).unwrap();
    let hints: Vec<String> = json["hints"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["command"].as_str().unwrap_or_default().to_string())
        .chain(json["data"]["next_step"].as_str().map(|s| s.to_string()))
        .collect();
    // the doctor must point at `spk init` when the block is absent/stale
    // (advisory: warnings channel in JSON, stderr footer in human mode)
    let out2 = spk()
        .args(["doctor", "--human"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8(out2.stderr).unwrap();
    let stdout = String::from_utf8(out2.stdout).unwrap();
    assert!(
        stderr.contains("spk init") || stdout.contains("spk init"),
        "doctor should suggest spk init when block missing: {stderr} | {stdout}"
    );
    let json2: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let warnings: Vec<String> = json2["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["message"].as_str().unwrap_or_default().to_string())
        .collect();
    assert!(
        warnings.iter().any(|w| w.contains("spk init")),
        "warning names the remediation: {warnings:?}"
    );
    let _ = hints;
}

// ---- specodelic-sok: doctor --fix (genesis::doctor framework) ----

#[test]
fn doctor_fix_writes_the_managed_block_and_verifies() {
    // --fix on a workspace with no block: the block check auto-fixes via
    // blocks::inject_into, the runner verifies post-fix, and the payload
    // renders the repaired fact
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("AGENTS.md"), "# My repo\n").unwrap();
    let out = spk()
        .args(["doctor", "--fix", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "doctor never fails");
    let agents = dir.path().join("AGENTS.md");
    let text = std::fs::read_to_string(&agents).unwrap();
    assert!(text.contains("SPECODELIC:START"), "block written: {text}");
    assert!(text.contains("# My repo"), "surrounding content preserved");
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(json["ok"], true, "success envelope");
    // payload renders the repaired state, not the stale finding
    let checks = json["data"]["checks"].as_array().unwrap();
    let block_row = checks
        .iter()
        .find(|c| c[0].as_str() == Some("SPECODELIC block"))
        .unwrap();
    assert!(
        block_row[1].as_str().unwrap().starts_with("ok (Revision"),
        "post-fix payload: {block_row:?}"
    );
}

#[test]
fn doctor_fix_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    for _ in 0..2 {
        let out = spk()
            .args(["doctor", "--fix", "--json"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0));
    }
    let text = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert_eq!(
        text.matches("SPECODELIC:START").count(),
        1,
        "exactly one block after two --fix runs"
    );
}

#[test]
fn doctor_without_fix_still_advises_but_does_not_write() {
    let dir = tempfile::tempdir().unwrap();
    let out = spk()
        .args(["doctor", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(
        !dir.path().join("AGENTS.md").exists(),
        "no --fix → no writes"
    );
}

// ---- specodelic-4le: update-availability notice (hermetic) ----

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[test]
fn doctor_warns_when_a_newer_version_is_on_crates_io() {
    // hermetic via XDG_CACHE_HOME + genesis's fresh-cache short-circuit:
    // the seeded cache says 9.9.9 is out → the doctor's warnings channel
    // names it, exit 0
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache/genesis/update-check");
    std::fs::create_dir_all(&cache).unwrap();
    std::fs::write(
        cache.join("specodelic.json"),
        format!(
            r#"{{"checked_at": {}, "latest": "9.9.9", "published_at": null, "ttl_secs": 604800}}"#,
            now_secs()
        ),
    )
    .unwrap();
    let out = spk()
        .args(["doctor", "--json"])
        .current_dir(dir.path())
        .env("XDG_CACHE_HOME", dir.path().join("cache"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "advisory, never fails");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let warnings: Vec<String> = json["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["message"].as_str().unwrap().to_string())
        .collect();
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("9.9.9") && w.contains("cargo install")),
        "update notice on the warnings channel: {warnings:?}"
    );
}

#[test]
fn doctor_is_silent_when_the_cached_version_is_current() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache/genesis/update-check");
    std::fs::create_dir_all(&cache).unwrap();
    let current = env!("CARGO_PKG_VERSION");
    std::fs::write(
        cache.join("specodelic.json"),
        format!(
            r#"{{"checked_at": {}, "latest": "{current}", "published_at": null, "ttl_secs": 604800}}"#,
            now_secs()
        ),
    )
    .unwrap();
    let out = spk()
        .args(["doctor", "--json"])
        .current_dir(dir.path())
        .env("XDG_CACHE_HOME", dir.path().join("cache"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let warnings: Vec<String> = json["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["message"].as_str().unwrap().to_string())
        .collect();
    assert!(
        !warnings.iter().any(|w| w.contains("available")),
        "no update notice when current: {warnings:?}"
    );
}

#[test]
fn doctor_survives_an_unreachable_crates_io() {
    // transport failure must be silent — doctor still exits 0 and the
    // stale-cache backoff write lands in the hermetic cache dir
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache/genesis/update-check");
    std::fs::create_dir_all(&cache).unwrap();
    // stale entry forces a fetch → unreachable API → silent backoff
    std::fs::write(
        cache.join("specodelic.json"),
        format!(
            r#"{{"checked_at": {}, "latest": "0.0.1", "published_at": null, "ttl_secs": 604800}}"#,
            now_secs() - 604800 - 1
        ),
    )
    .unwrap();
    let out = spk()
        .args(["doctor", "--json"])
        .current_dir(dir.path())
        .env("XDG_CACHE_HOME", dir.path().join("cache"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "network can never fail doctor");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["ok"], true);
}

// ---- specodelic-cxr: spk hooks install / uninstall ----

/// Wire a fake `spk` on PATH so the install-time gate dry-run is
/// deterministic: "pass" exits 0, "fail" exits 1 with a lint summary.
fn fake_spk(dir: &std::path::Path, behavior: &str) -> std::path::PathBuf {
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let script = match behavior {
        "pass" => "#!/bin/sh\necho 'linted 12 files, 0 issues'\n",
        _ => "#!/bin/sh\necho 'linter.dual_format_valid: openspec/changes/x/spec.md' >&2\nexit 1\n",
    };
    std::fs::write(bin.join("spk"), script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(bin.join("spk"), std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    bin
}

/// A minimal repo fixture: `.git` marker, optional lefthook config,
/// optional openspec/ tree (the install pre-check target).
// (comment, not a doc: this describes the fixture module's helpers, not
// the `with_path` item that follows)
fn with_path(bin: &std::path::Path) -> String {
    format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

#[test]
fn hooks_install_wires_gate_and_reports_envelope() {
    let dir = hooks_fixture(true, true);
    let pathdir = tempfile::tempdir().unwrap();
    let bin = fake_spk(pathdir.path(), "pass");
    let out = spk()
        .args(["hooks", "install", "--json"])
        .current_dir(dir.path())
        .env("PATH", with_path(&bin))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"outcome\""), "envelope data: {stdout}");
    assert!(stdout.contains("wired"), "envelope data: {stdout}");
    let config = std::fs::read_to_string(dir.path().join("lefthook.yml")).unwrap();
    assert!(
        config.contains(
            "  commands:\n    # <!-- SPK:START -->\n    specodelic-gates:\n      run: spk lint openspec\n    # <!-- SPK:END -->\n    sibling-blockers:"
        ),
        "entry inside the existing commands mapping:\n{config}"
    );
    assert_eq!(config.matches("commands:").count(), 1, "no duplicate key");
    assert!(
        stdout.contains("gate_dry_run"),
        "dry-run reported: {stdout}"
    );
}

#[test]
fn hooks_install_warns_but_succeeds_when_gate_fails() {
    let dir = hooks_fixture(true, true);
    let pathdir = tempfile::tempdir().unwrap();
    let bin = fake_spk(pathdir.path(), "fail");
    let out = spk()
        .args(["hooks", "install", "--json"])
        .current_dir(dir.path())
        .env("PATH", with_path(&bin))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "install must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("spk hooks uninstall"),
        "escape hint in warnings: {stdout}"
    );
    assert!(
        std::fs::read_to_string(dir.path().join("lefthook.yml"))
            .unwrap()
            .contains("specodelic-gates:"),
        "still wired"
    );
}

#[test]
fn hooks_install_errors_on_missing_config_and_creates_none() {
    let dir = hooks_fixture(false, true);
    let pathdir = tempfile::tempdir().unwrap();
    let bin = fake_spk(pathdir.path(), "pass");
    let out = spk()
        .args(["hooks", "install", "--json"])
        .current_dir(dir.path())
        .env("PATH", with_path(&bin))
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no supported hook framework config"),
        "labeled error: {stderr}"
    );
    assert!(!dir.path().join("lefthook.yml").exists(), "never created");
}

#[test]
fn hooks_install_requires_openspec_dir() {
    let dir = hooks_fixture(true, false);
    let out = spk()
        .args(["hooks", "install", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("openspec"),
        "error names the missing openspec tree: {stderr}"
    );
    assert!(
        !std::fs::read_to_string(dir.path().join("lefthook.yml"))
            .unwrap()
            .contains("specodelic-gates:"),
        "config untouched when the pre-check fails"
    );
}

#[test]
fn hooks_uninstall_on_unwired_repo_is_a_noop() {
    let dir = hooks_fixture(true, true);
    let before = std::fs::read_to_string(dir.path().join("lefthook.yml")).unwrap();
    let out = spk()
        .args(["hooks", "uninstall", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "no-op is success; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("not_wired"), "envelope: {stdout}");
    let after = std::fs::read_to_string(dir.path().join("lefthook.yml")).unwrap();
    assert_eq!(before, after, "config unchanged");
}

#[test]
fn hooks_uninstall_strips_only_the_block() {
    let dir = hooks_fixture(true, true);
    let pathdir = tempfile::tempdir().unwrap();
    let bin = fake_spk(pathdir.path(), "pass");
    spk()
        .args(["hooks", "install", "--json"])
        .current_dir(dir.path())
        .env("PATH", with_path(&bin))
        .output()
        .unwrap()
        .status
        .success()
        .then_some(())
        .expect("install must succeed");
    let wired = std::fs::read_to_string(dir.path().join("lefthook.yml")).unwrap();
    let out = spk()
        .args(["hooks", "uninstall", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("unwired"), "envelope: {stdout}");
    let after = std::fs::read_to_string(dir.path().join("lefthook.yml")).unwrap();
    let restored: String = wired
        .lines()
        .filter(|l| {
            let t = l.trim();
            !(t.contains("SPK:START")
                || t.contains("SPK:END")
                || t.contains("specodelic-gates:")
                || t.contains("run: spk lint openspec"))
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        after.lines().collect::<Vec<_>>().join("\n"),
        restored,
        "only block lines removed:\n{after}"
    );
}
