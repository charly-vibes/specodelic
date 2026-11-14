//! specodelic-3a8: the parse envelope keeps the dual-format keys after the
//! adapter extraction. Kept in its own module because `parse_misc.rs` is at
//! the pretender file-lines ratchet.

use super::*;

/// `spk parse --json` keeps the six dual-format keys and their values even
/// though they are now computed by the adapter rather than carried on
/// `Spec` — the external fleet envelope is unchanged.
#[test]
fn parse_json_keeps_dual_format_fields() {
    let dir = tempfile::tempdir().unwrap_p();
    let file = dir.path().join("spec.md");
    std::fs::write(
        &file,
        "---\nid: demo.cap\nkind: intent\nstatement: \"THE demo SHALL hold\"\n---\n\n## ADDED Requirements\n\n### Requirement: One\nadded body\n\n## Requirements\n\n### Requirement: Two\ncapability body\n",
    )
    .unwrap_p();
    let out = spk()
        .args(["parse", file.to_str().unwrap_p(), "--json"])
        .output()
        .unwrap_p();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_p();
    let data = &json["data"];
    assert_eq!(data["has_added_requirements"], true);
    assert_eq!(data["has_requirements_section"], true);
    assert_eq!(data["has_modified_requirements"], false);
    assert_eq!(
        data["added_requirements_body"],
        "\n### Requirement: One\nadded body\n\n"
    );
    assert_eq!(
        data["requirements_body"],
        "\n### Requirement: Two\ncapability body\n"
    );
    assert_eq!(data["modified_requirements_body"], "");

    // A plain spec still gets the historical keys, false/empty.
    let plain = dir.path().join("plain.md");
    std::fs::write(
        &plain,
        "---\nid: plain\nkind: intent\nstatement: \"THE plain SHALL hold\"\n---\n\nProse only.\n",
    )
    .unwrap_p();
    let out = spk()
        .args(["parse", plain.to_str().unwrap_p(), "--json"])
        .output()
        .unwrap_p();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_p();
    let data = &json["data"];
    assert_eq!(data["has_added_requirements"], false);
    assert_eq!(data["has_requirements_section"], false);
    assert_eq!(data["has_modified_requirements"], false);
    assert_eq!(data["added_requirements_body"], "");
    assert_eq!(data["requirements_body"], "");
    assert_eq!(data["modified_requirements_body"], "");
}
