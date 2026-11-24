//! CLI contracts for the corpus-aware artifact gate (specodelic-qbox).
//!
//! `spk check` used to lint the judged file in ISOLATION: every
//! cross-file reference dangled, the gate refused with
//! `artifact_gate: the file has lint findings`, and no corpus capability
//! could ever be judged. The gate must lint the file WITHIN the corpus
//! discovered from the file's own location — and when no corpus can be
//! discovered, the isolation must be labeled on the envelope's warnings
//! channel (a warning, never a silent partial verdict).

use crate::common::UnwrapPanic;
use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

/// A spec whose constraint cites a SIBLING capability (`[[beta]]`): in
/// isolation the reference dangles; inside a corpus it resolves.
fn write_alpha_spec(path: &std::path::Path) {
    std::fs::write(
        path,
        "---\nid: alpha\nkind: intent\nstatement: \"WHEN the artifact gate lints a spec file that lives inside an openspec corpus, THE check capability SHALL resolve the file's cross-file references against the discovered corpus.\"\n---\n\
             \n# alpha Specification\n\
             \n## Purpose\n\
             \nThe judged file of the corpus-aware gate fixture.\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | resolves_in_corpus | invariant | `the gate lints the judged file within the corpus discovered from the file's own location, so a cross-file reference such as [[beta.c1]] resolves against its sibling` | [[alpha]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- `gated`\n\
             \n- `judged`\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | judge | gated | judged | [[alpha.resolves_in_corpus]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p_resolves | unit | [[alpha.resolves_in_corpus]] | `arbitrary_corpus()` | `the judged file's cross-file references resolve ∧ no dangling reference is reported` |\n"
    )
    .unwrap_p();
}

/// The sibling capability the judged file cites.
fn write_beta_spec(path: &std::path::Path) {
    std::fs::write(
        path,
        "---\nid: beta\nkind: intent\nstatement: \"WHEN a sibling capability declares a constraint, THE beta capability SHALL expose that constraint for corpus-wide reference resolution.\"\n---\n\
             \n# beta Specification\n\
             \n## Purpose\n\
             \nThe sibling the judged file cites.\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | c1 | invariant | `the sibling constraint is declared once and stays referenceable from the corpus` | [[beta]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- `declared`\n\
             \n- `referenceable`\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | expose | declared | referenceable | [[beta.c1]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p_declared | unit | [[beta.c1]] | `arbitrary_sibling()` | `the constraint is declared exactly once ∧ referenceable corpus-wide` |\n"
    )
    .unwrap_p();
}

/// A two-file corpus under `<tmp>/openspec/specs/…` — the shape the
/// discovery walks up to.
fn write_corpus(root: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let alpha_dir = root.join("openspec/specs/alpha");
    let beta_dir = root.join("openspec/specs/beta");
    std::fs::create_dir_all(&alpha_dir).unwrap_p();
    std::fs::create_dir_all(&beta_dir).unwrap_p();
    let alpha = alpha_dir.join("spec.md");
    let beta = beta_dir.join("spec.md");
    write_alpha_spec(&alpha);
    write_beta_spec(&beta);
    (alpha, beta)
}

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap_p()
}

/// The corpus case: the judged file's cross-file reference resolves
/// against the discovered corpus, so the gate passes the lint stage and
/// the run REACHES the verify stage — it never refuses at `artifact_gate`.
#[test]
fn corpus_file_reaches_the_verify_stage() {
    let tmp = tempfile::tempdir().unwrap_p();
    let (alpha, beta) = write_corpus(tmp.path());
    let out_dir = tmp.path().join("out");
    std::fs::create_dir_all(&out_dir).unwrap_p();

    spk()
        .args([
            "compile",
            alpha.to_str().unwrap_p(),
            beta.to_str().unwrap_p(),
            "--out-dir",
            out_dir.to_str().unwrap_p(),
        ])
        .assert()
        .success();

    let out = spk()
        .args([
            "check",
            alpha.to_str().unwrap_p(),
            "--out-dir",
            out_dir.to_str().unwrap_p(),
            "--json",
        ])
        .output()
        .unwrap_p();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        !stdout.contains("artifact_gate"),
        "the gate must not refuse at artifact_gate for a file inside its corpus — the cross-file reference resolves there; got:\n{stdout}"
    );
    assert!(
        stdout.contains("verify_model"),
        "the run must reach the verify stage; got:\n{stdout}"
    );
}

/// The isolated-file fallback: with no corpus root discoverable the gate
/// still lints the file alone (the reference dangles), but the isolation
/// is LABELED on the warnings channel — a warning, never a silent
/// partial verdict.
#[test]
fn isolated_file_is_labeled_as_isolated() {
    let tmp = tempfile::tempdir().unwrap_p();
    let stray_dir = tmp.path().join("stray/alpha");
    std::fs::create_dir_all(&stray_dir).unwrap_p();
    let stray = stray_dir.join("spec.md");
    write_alpha_spec(&stray);

    let out = spk()
        .args(["check", stray.to_str().unwrap_p(), "--json"])
        .output()
        .unwrap_p();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        stdout.contains("ISOLATION"),
        "the isolated-file fallback must be labeled on the envelope's warnings channel; got:\n{stdout}"
    );
}

/// The corpus case never depends on the working directory or the
/// environment: the corpus is discovered from the FILE's own location.
#[test]
fn corpus_discovery_is_location_decided_not_cwd_decided() {
    let tmp = tempfile::tempdir().unwrap_p();
    let (alpha, beta) = write_corpus(tmp.path());
    let out_dir = tmp.path().join("out");
    std::fs::create_dir_all(&out_dir).unwrap_p();
    spk()
        .args([
            "compile",
            alpha.to_str().unwrap_p(),
            beta.to_str().unwrap_p(),
            "--out-dir",
            out_dir.to_str().unwrap_p(),
        ])
        .assert()
        .success();

    // Run from an unrelated working directory: the file's own location
    // still decides the corpus.
    let elsewhere = tempfile::tempdir().unwrap_p();
    let out = spk()
        .current_dir(elsewhere.path())
        .args([
            "check",
            alpha.to_str().unwrap_p(),
            "--out-dir",
            out_dir.to_str().unwrap_p(),
            "--json",
        ])
        .output()
        .unwrap_p();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        !stdout.contains("artifact_gate"),
        "the corpus is discovered from the file's own location, not the cwd; got:\n{stdout}"
    );
}

/// The refusal envelope keeps its labeled shape while carrying the
/// corpus fact: the gate's message names the stage, and the hint still
/// points at the remediation.
#[test]
fn gate_refusal_keeps_its_labeled_shape() {
    let tmp = tempfile::tempdir().unwrap_p();
    let stray_dir = tmp.path().join("stray/alpha");
    std::fs::create_dir_all(&stray_dir).unwrap_p();
    let stray = stray_dir.join("spec.md");
    write_alpha_spec(&stray);

    spk()
        .args(["check", stray.to_str().unwrap_p(), "--json"])
        .assert()
        .failure()
        .stdout(contains("artifact_gate").and(contains("run: spk lint")));
}
