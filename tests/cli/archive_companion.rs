//! `spk archive-companion` end-to-end (CLI) tests: the Stage-2
//! envelope surface over the tempdir fixtures — dry-run planning,
//! fail-closed sidecar refusal, and the unknown-change label. The
//! test names are bound by the espectacular contract TOMLs under
//! `.espectacular/archive-companion/` — do not rename them casually.

use assert_cmd::Command;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap()
}

/// Fixture: a fake repo root (a `.git` dir is all repo_root() needs to
/// anchor) with an already-archived change; the sibling `layer.md`
/// sidecar rides along (the Stage-2 shape, Revision 19). `sidecar`
/// controls whether the sidecar exists at all.
fn fixture(id: &str, delta: &str, sidecar: Option<&str>) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join(".git")).unwrap();
    let delta_path = root.path().join(format!(
        "openspec/changes/archive/2026-10-01-{id}/specs/c/spec.md"
    ));
    std::fs::create_dir_all(delta_path.parent().unwrap()).unwrap();
    std::fs::write(&delta_path, delta).unwrap();
    if let Some(sidecar) = sidecar {
        std::fs::write(delta_path.with_file_name("layer.md"), sidecar).unwrap();
    }
    root
}

/// A plain openspec delta (Revision 19): no frontmatter, no tables.
const PLAIN_DELTA: &str =
    "## ADDED Requirements\n\n### Requirement: One holds\nThe system SHALL hold one.\n";

/// A lint-clean `layer.md` sidecar: frontmatter (`id: c.layer`) + the
/// three specodelic tables.
const SIDECAR: &str = concat!(
    "---\nid: c.layer\nkind: intent\n",
    "statement: \"WHEN the c layer is elaborated, THE author SHALL write the real requirement.\"\n",
    "---\n\n",
    "## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n",
    "| k1 | invariant | `true` | [[c.layer]] |\n\n",
    "## Model\n\n### States\n\n- `draft`\n\n### Transitions\n\n",
    "| id | from | to | guard |\n|----|------|----|-------|\n",
    "| t1 | draft | draft | [[c.layer.k1]] |\n\n",
    "## Properties\n\n",
    "| id | kind | derives_from | generator | predicate |\n",
    "|----|------|--------------|-----------|-----------|\n",
    "| p1 | unit | [[c.layer.k1]] | `todo()` | `true` |\n"
);

/// archive-companion dry-run over an already-archived change: exit 0,
/// envelope lists the planned restore AND the consumed sidecar, and
/// NOTHING is written (specodelic-fzo, dry_run_never_mutates).
#[test]
fn archive_companion_dry_run_reports_without_writing() {
    let root = fixture("dryx", PLAIN_DELTA, Some(SIDECAR));
    let out = spk()
        .args(["archive-companion", "dryx", "--dry-run", "--json"])
        .current_dir(root.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{stdout}");
    assert_eq!(json["data"]["dry_run"], serde_json::json!(true));
    assert_eq!(json["data"]["openspec_ran"], serde_json::json!(false));
    assert_eq!(
        json["data"]["restored"],
        serde_json::json!(["openspec/specs/c/spec.md"])
    );
    assert_eq!(
        json["data"]["sidecars"],
        serde_json::json!(["openspec/changes/archive/2026-10-01-dryx/specs/c/layer.md"])
    );
    assert!(
        !root.path().join("openspec/specs").exists(),
        "dry-run must not create deployed spec paths"
    );
}

/// archive-companion fails closed on a sidecar-less archived delta
/// (specodelic-s8fg, fail-closed on sidecar-less deltas — the
/// successor of the Stage-1 frontmatterless-delta refusal): exit 1,
/// the delta is named with the wrap-path hint, and no deployed spec
/// is written.
#[test]
fn archive_companion_refuses_frontmatterless_delta() {
    let root = fixture("plain", PLAIN_DELTA, None);
    let out = spk()
        .args(["archive-companion", "plain", "--json"])
        .current_dir(root.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("unrestorable"),
        "refusal must name the failure: {stderr}"
    );
    assert!(
        stderr.contains("spk migrate"),
        "refusal must carry the wrap-path hint: {stderr}"
    );
    assert!(
        !root.path().join("openspec/specs").exists(),
        "a refusal must not write any deployed spec"
    );
}

/// archive-companion over an unknown change id: labeled error naming
/// the id and the `openspec list` hint.
#[test]
fn archive_companion_unknown_change_is_labeled_error() {
    let root = fixture("other", PLAIN_DELTA, Some(SIDECAR));
    let out = spk()
        .args(["archive-companion", "ghost-change", "--json"])
        .current_dir(root.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("ghost-change"), "{stderr}");
    assert!(stderr.contains("openspec list"), "{stderr}");
}
