//! Drift guard for the `just archive-change` recipe (specodelic-4yve).
//!
//! The corpus documents the invocation as `just archive-change id=<change-id>`
//! (deployed specs' `process_lifecycle` rows, change `tasks.md` files), but
//! `just` treats `id=` as part of the positional value — the companion used
//! to receive the literal `id=<change-id>` and fail closed with a confusing
//! `ChangeNotFound`. The recipe must accept BOTH forms and forward the bare
//! change id to `cargo run -q -- archive-companion <id>`.
//!
//! Contract (specodelic-4yve):
//! 1. The id normalizer strips a leading `id=` prefix.
//! 2. A bare id is passed through untouched.
//! 3. An empty id fails with a remediation hint (non-zero exit).
//! 4. The deployed recipe body wires the normalizer (a recipe that stops
//!    calling it is a finding).
//! 5. End-to-end: the recipe body, run with a stub `cargo` on PATH, forwards
//!    the BARE id and never the literal `id=...` form.
//! 6. End-to-end: an empty id fails the recipe with the hint.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo_root().join(rel)).unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
}

/// The recipe body of `archive-change` in the justfile: the shebang line and
/// every indented line that follows the recipe header.
fn archive_change_recipe_body() -> String {
    let justfile = read("justfile");
    let mut lines = justfile.lines();
    let header = lines
        .find(|l| l.trim() == "archive-change id:")
        .expect("justfile must declare an `archive-change id:` recipe");
    let mut body = Vec::new();
    for line in lines {
        if line.trim().is_empty() || line.starts_with(' ') || line.starts_with('\t') {
            body.push(line);
            continue;
        }
        break;
    }
    let body = body.join("\n");
    assert!(
        body.contains("archive-companion"),
        "archive-change recipe must delegate to the archive companion"
    );
    body
}

/// Run the normalizer script the recipe wires, capturing stdout/stderr/status.
fn run_normalizer(arg: &str) -> (String, String, Option<i32>) {
    let out = Command::new("bash")
        .arg(repo_root().join("scripts/archive_change_id.sh"))
        .arg(arg)
        .current_dir(repo_root())
        .output()
        .expect("normalizer script must be runnable");
    (
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.code(),
    )
}

#[test]
fn id_prefix_form_is_normalized() {
    let (stdout, _, code) = run_normalizer("id=add-check-claim-coverage");
    assert_eq!(code, Some(0), "normalizer must succeed on the id= form");
    assert_eq!(stdout, "add-check-claim-coverage");
}

#[test]
fn bare_form_is_preserved() {
    let (stdout, _, code) = run_normalizer("add-check-claim-coverage");
    assert_eq!(code, Some(0));
    assert_eq!(stdout, "add-check-claim-coverage");
}

#[test]
fn empty_id_fails_with_remediation_hint() {
    let (_, stderr, code) = run_normalizer("");
    assert_ne!(code, Some(0), "an empty id must fail");
    assert!(
        stderr.contains("just archive-change"),
        "the failure must carry a remediation hint, got: {stderr}"
    );
}

/// The deployed recipe must call the normalizer — otherwise the `id=` form
/// regresses silently even though the normalizer still works.
#[test]
fn recipe_wires_the_normalizer() {
    let body = archive_change_recipe_body();
    assert!(
        body.contains("archive_change_id.sh"),
        "archive-change recipe must route the change id through scripts/archive_change_id.sh, got:\n{body}"
    );
}

/// End-to-end: the recipe body forwards the BARE id to the companion.
#[test]
fn recipe_forwards_bare_id_to_companion() {
    let (stdout, stderr, code) = run_recipe_with_stub_cargo("id=add-check-claim-coverage");
    assert_eq!(code, Some(0), "recipe must succeed, stderr: {stderr}");
    assert!(
        stdout.contains("archive-companion add-check-claim-coverage"),
        "the companion must receive the bare change id, got: {stdout}"
    );
    assert!(
        !stdout.contains("id=add-check-claim-coverage"),
        "the literal id= form must never reach the companion, got: {stdout}"
    );
}

/// End-to-end: the bare positional form keeps working.
#[test]
fn recipe_accepts_bare_positional_form() {
    let (stdout, stderr, code) = run_recipe_with_stub_cargo("add-check-claim-coverage");
    assert_eq!(code, Some(0), "recipe must succeed, stderr: {stderr}");
    assert!(
        stdout.contains("archive-companion add-check-claim-coverage"),
        "the bare form must still reach the companion, got: {stdout}"
    );
}

/// End-to-end: an empty id fails the recipe with the hint.
#[test]
fn recipe_fails_on_empty_id_with_hint() {
    let (_, stderr, code) = run_recipe_with_stub_cargo("");
    assert_ne!(code, Some(0), "the recipe must fail on an empty id");
    assert!(
        stderr.contains("just archive-change"),
        "the failure must carry a remediation hint, got: {stderr}"
    );
}

/// Run the deployed recipe body with a stub `cargo` on PATH; the stub echoes
/// its arguments so the test can observe what the companion was called with.
fn run_recipe_with_stub_cargo(arg: &str) -> (String, String, Option<i32>) {
    let tmp = tempfile::tempdir().expect("temp dir");
    let stub_dir = tmp.path().join("bin");
    fs::create_dir_all(&stub_dir).expect("stub dir");
    let stub = stub_dir.join("cargo");
    fs::write(&stub, "#!/usr/bin/env bash\nprintf 'cargo %s\\n' \"$*\"\n").expect("write stub");
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o755)).expect("chmod stub");

    let body = archive_change_recipe_body();
    // `just` substitutes the recipe parameter into `{{id}}` and the justfile's
    // own directory into `{{justfile_directory()}}` before running the body;
    // the stub run reproduces those substitutions verbatim.
    let body = body.replace("{{id}}", arg).replace(
        "{{justfile_directory()}}",
        repo_root().to_string_lossy().as_ref(),
    );
    let script = tmp.path().join("recipe.sh");
    fs::write(&script, body).expect("write recipe");

    let path = format!(
        "{}:{}",
        stub_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = Command::new("bash")
        .arg(&script)
        .arg(arg)
        .env("PATH", path)
        .current_dir(repo_root())
        .output()
        .expect("recipe body must be runnable");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.code(),
    )
}
