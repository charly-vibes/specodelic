//! Tests for the gate-drill advisory lock (specodelic-do8, remediation (a)
//! of the specodelic-dz4 concurrent-session race, incident e7bb19e).
//!
//! Contract (openspec/decisions/2026-10-05-precommit-sweep-policy.md):
//! 1. `scripts/guards/gate-drill-lock.sh check` refuses commit/push while a
//!    LIVE drill lock is held (exit 1, stderr carries a remediation hint).
//! 2. A STALE lock (dead pid, unparsable owner, or cross-host lock older
//!    than GATE_DRILL_TTL) must not wedge commits: exit 0 with a warning.
//! 3. `scripts/guards/gate-drill.sh <cmd...>` refuses to run a drill in-tree
//!    without the lock (exit 2, hint names the lock/worktree options),
//!    acquires + releases the lock around the drill, and runs bare inside a
//!    linked git worktree (worktree = isolation, no lock needed).
//! 4. The guard is wired where commits happen: lefthook.yml pre-commit AND
//!    pre-push, plus a `guard-drill-lock` recipe in `just ci` — the same
//!    chaining pattern as sibling-blockers (no .beads/hooks edits, no
//!    core.hooksPath claims).
//!
//! Unix-only: the scripts are bash programs; on Windows the `bash` on PATH
//! is WSL's stub — the contract is exercised on the ubuntu CI leg.
#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn guard_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/guards/gate-drill-lock.sh")
}

fn drill_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/guards/gate-drill.sh")
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo_root().join(rel)).unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
}

/// A pid that is definitively dead: spawned, waited (reaped), and gone.
fn dead_pid() -> u32 {
    let mut child = Command::new("true").spawn().expect("spawn true");
    let pid = child.id();
    child.wait().expect("reap true");
    pid
}

/// A child that stays alive (and unreaped, hence kill -0-visible) as long as
/// the returned guard value is held. The `Child` field exists for its
/// Drop side effect (reaping) — it is never read.
#[allow(dead_code)]
struct LiveChild(std::process::Child);
fn live_pid() -> (u32, LiveChild) {
    let child = Command::new("sleep")
        .arg("30")
        .spawn()
        .expect("spawn sleep");
    let pid = child.id();
    (pid, LiveChild(child))
}

fn hostname() -> String {
    String::from_utf8_lossy(
        &Command::new("uname")
            .arg("-n")
            .output()
            .expect("uname")
            .stdout,
    )
    .trim()
    .to_string()
}

fn fixture_repo(name: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir_in(std::env::temp_dir()).expect("tempdir");
    let repo = dir.path().join(name);
    fs::create_dir_all(&repo).unwrap();
    let repo = repo.canonicalize().unwrap();
    Command::new("git")
        .args(["-C", repo.to_str().unwrap(), "init", "-q"])
        .status()
        .expect("git init");
    (dir, repo)
}

/// Write a drill lock owner file with the given fields.
fn write_lock(repo: &Path, host: &str, pid: u32, epoch: u64, note: &str) {
    let lock = repo.join(".beads/gate-drill.lock");
    fs::create_dir_all(&lock).unwrap();
    fs::write(lock.join("owner"), format!("{host}|{pid}|{epoch}|{note}\n")).unwrap();
}

/// Run gate-drill-lock.sh `args` against `repo`.
fn run_lock(args: &[&str], repo: &Path, extra_env: &[(&str, &str)]) -> (bool, String) {
    let out = Command::new("bash")
        .arg(guard_path())
        .args(args)
        .arg(repo)
        .envs(extra_env.iter().copied())
        .output()
        .expect("gate-drill-lock.sh should be runnable");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), combined)
}

/// Run gate-drill.sh with `cmd` against `repo` (cwd = repo).
fn run_drill(repo: &Path, cmd: &[&str], extra_env: &[(&str, &str)]) -> (Option<i32>, String) {
    let out = Command::new("bash")
        .arg(drill_path())
        .args(cmd)
        .current_dir(repo)
        .envs(extra_env.iter().copied())
        .output()
        .expect("gate-drill.sh should be runnable");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.code(), combined)
}

// --- contract 1: live lock blocks commit/push -------------------------------

#[test]
fn no_lock_check_passes_silently() {
    let (_dir, repo) = fixture_repo("no-lock");
    let (ok, out) = run_lock(&["check"], &repo, &[]);
    assert!(ok, "check with no lock must pass, got: {out}");
}

#[test]
fn live_same_host_lock_fails_check_with_hint() {
    let (_dir, repo) = fixture_repo("live-lock");
    let (pid, _child) = live_pid();
    write_lock(&repo, &hostname(), pid, epoch_now(), "test drill");
    let (ok, out) = run_lock(&["check"], &repo, &[]);
    assert!(
        !ok,
        "check must FAIL while a live drill lock is held (do8 meter), got: {out}"
    );
    assert!(
        out.contains(".beads/gate-drill.lock"),
        "failure must name the lockfile as remediation, got: {out}"
    );
    assert!(
        out.to_lowercase().contains("drill"),
        "failure must explain a gate drill is in progress, got: {out}"
    );
}

// --- contract 2: stale locks never wedge commits -----------------------------

#[test]
fn dead_pid_lock_is_stale_and_passes_check() {
    let (_dir, repo) = fixture_repo("dead-pid");
    write_lock(
        &repo,
        &hostname(),
        dead_pid(),
        epoch_now(),
        "crashed session",
    );
    let (ok, out) = run_lock(&["check"], &repo, &[]);
    assert!(
        ok,
        "stale (dead pid) lock must NOT block commits, got: {out}"
    );
    assert!(
        out.to_lowercase().contains("stale"),
        "stale lock must produce a warning, got: {out}"
    );
}

#[test]
fn unparsable_lock_is_stale_and_passes_check() {
    let (_dir, repo) = fixture_repo("unparsable");
    let lock = repo.join(".beads/gate-drill.lock");
    fs::create_dir_all(&lock).unwrap();
    fs::write(lock.join("owner"), "garbage-without-pipes").unwrap();
    let (ok, out) = run_lock(&["check"], &repo, &[]);
    assert!(ok, "unparsable lock must NOT block commits, got: {out}");
    assert!(out.to_lowercase().contains("stale"), "got: {out}");
}

#[test]
fn cross_host_lock_is_live_within_ttl_and_stale_beyond() {
    let (_dir, repo) = fixture_repo("cross-host");
    write_lock(
        &repo,
        "some-other-host.invalid",
        dead_pid(),
        epoch_now(),
        "remote session",
    );

    // Within TTL (default): live — even though the pid is meaningless on
    // this host (it belongs to another machine).
    let (ok, out) = run_lock(&["check"], &repo, &[]);
    assert!(
        !ok,
        "cross-host lock within TTL must count live, got: {out}"
    );

    // TTL 0: anything cross-host is stale.
    let (ok, out) = run_lock(&["check"], &repo, &[("GATE_DRILL_TTL", "0")]);
    assert!(ok, "cross-host lock beyond TTL must be stale, got: {out}");
    assert!(out.to_lowercase().contains("stale"), "got: {out}");
}

// --- contract 3: drill wrapper refuses / isolates / releases -----------------

#[test]
fn drill_in_tree_without_lock_runs_and_releases() {
    let (_dir, repo) = fixture_repo("drill-run");
    let (code, out) = run_drill(&repo, &["touch", "marker"], &[]);
    assert_eq!(
        code,
        Some(0),
        "drill should run after acquiring lock, got: {out}"
    );
    assert!(repo.join("marker").exists(), "drill cmd must actually run");
    assert!(
        !repo.join(".beads/gate-drill.lock").exists(),
        "lock must be released after the drill exits, got: {out}"
    );
}

#[test]
fn drill_in_tree_refuses_when_live_lock_held_exit_2() {
    let (_dir, repo) = fixture_repo("drill-refuse");
    let (pid, _child) = live_pid();
    write_lock(&repo, &hostname(), pid, epoch_now(), "other session");
    let (code, out) = run_drill(&repo, &["touch", "marker"], &[]);
    assert_eq!(
        code,
        Some(2),
        "drill must refuse with exit 2 when lock is live (do8 meter), got: {out}"
    );
    assert!(
        !repo.join("marker").exists(),
        "refused drill must not run its command"
    );
    assert!(
        out.to_lowercase().contains("worktree"),
        "refusal hint must mention the worktree alternative, got: {out}"
    );
}

#[test]
fn drill_runs_bare_in_linked_worktree() {
    let (_dir, repo) = fixture_repo("drill-main");
    let git = |args: &[&str]| {
        let st = Command::new("git")
            .args(["-C", repo.to_str().unwrap()])
            .args(args)
            .status()
            .expect("git");
        assert!(st.success(), "git {:?} failed", args);
    };
    git(&["config", "user.email", "t@example.com"]);
    git(&["config", "user.name", "t"]);
    fs::write(repo.join("seed.txt"), "seed").unwrap();
    git(&["add", "seed.txt"]);
    git(&["commit", "-q", "-m", "seed"]);

    // A live lock is held by another session: the worktree path must not
    // care (worktree = the isolation), the in-tree path would refuse.
    let (pid, _child) = live_pid();
    write_lock(&repo, &hostname(), pid, epoch_now(), "other session");

    let wt = _dir.path().join("drill-wt");
    git(&[
        "worktree",
        "add",
        "-q",
        wt.to_str().unwrap(),
        "-b",
        "drill-wt",
    ]);

    let (code, out) = run_drill(&wt, &["touch", "wt-marker"], &[]);
    assert_eq!(
        code,
        Some(0),
        "in a linked worktree the drill runs bare, no lock needed, got: {out}"
    );
    assert!(wt.join("wt-marker").exists());
}

#[test]
fn drill_propagates_command_failure_status() {
    let (_dir, repo) = fixture_repo("drill-fail");
    let (code, _out) = run_drill(&repo, &["false"], &[]);
    assert_eq!(code, Some(1), "drill must propagate the cmd's exit status");
}

// --- contract 4: wiring — commits and CI are guarded -------------------------

#[test]
fn lefthook_checks_drill_lock_on_commit_and_push() {
    let cfg = read("lefthook.yml");
    // Split into the pre-commit block (up to pre-push) and the pre-push
    // block (rest of file); both must carry the guard.
    let (_, after_pre_commit) = cfg
        .split_once("pre-commit:")
        .expect("lefthook.yml pre-commit block");
    let (pre_commit_block, pre_push_block) = after_pre_commit
        .split_once("pre-push:")
        .unwrap_or((after_pre_commit, ""));
    let needle = "gate-drill-lock";
    assert!(
        pre_commit_block.contains(needle),
        "lefthook.yml pre-commit must run the gate-drill-lock guard (specodelic-do8)"
    );
    assert!(
        pre_push_block.contains(needle),
        "lefthook.yml pre-push must run the gate-drill-lock guard (specodelic-do8)"
    );
}

#[test]
fn just_ci_runs_the_drill_lock_guard() {
    let jf = read("justfile");
    assert!(
        jf.contains("guard-drill-lock:"),
        "justfile must define a guard-drill-lock recipe (specodelic-do8)"
    );
    let ci = jf
        .lines()
        .find(|l| l.starts_with("ci:"))
        .expect("justfile ci recipe");
    assert!(
        ci.contains("guard-drill-lock"),
        "just ci must run guard-drill-lock (specodelic-do8)"
    );
}

#[test]
fn guard_does_not_touch_beads_hooks_or_claim_hookspath() {
    // House constraint (AGENTS.md): the interlock must chain via
    // lefthook.yml, never edit .beads/hooks shims or claim core.hooksPath.
    let guard = fs::read_to_string(guard_path()).unwrap();
    assert!(
        !guard.contains("git config"),
        "gate-drill-lock.sh must not touch git config (core.hooksPath claims are the pretender-class violation)"
    );
    // No lefthook COMMAND may target .beads/hooks — descriptive comments
    // about the shim chain are fine (the shims chain TO lefthook).
    let cfg = read("lefthook.yml");
    for line in cfg.lines().filter(|l| l.trim().starts_with("run:")) {
        assert!(
            !line.contains(".beads/hooks"),
            "lefthook commands must not target .beads/hooks shims: {line}"
        );
    }
}

fn epoch_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
