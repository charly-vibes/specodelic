//! The kernel vocabulary pack — enablement of the `kernel.binding`
//! vocabulary (specodelic-04c; owner decision 2026-11-13).
//!
//! **Purpose:** prove that the `kernel.binding` vocabulary is enabled by
//! the declared-extension path — a `kind: profile` pack in namespace
//! `kernel` discovered by corpus scan alone — and not by a linter
//! carve-out, a config file, or a base-closed-set change.
//!
//! **Responsibilities:**
//! - a workspace fixture whose Constraints table carries a `kernel.binding`
//!   column lints **clean** because the corpus ships the kernel pack
//!   (`packs/kernel-pack.md`, id `kernel.pack`), and the
//!   discovered-pack list names it;
//! - a fixture **without** the column is untouched by the pack (no
//!   activation, no finding) — the pack-fiber-relative byte-identity
//!   law (`openspec/specs/packs/spec.md` `no_pack_no_change`);
//! - the corpus lint discovers the kernel pack by scan alone and stays
//!   issue-free.
//!
//! **Rationale:** `orphan_vocabulary` (openspec/specs/packs/spec.md
//! `orphan_vocabulary_labeled`) labels a file whose structured vocabulary
//! position carries a pack-qualified token whose namespace matches no
//! discovered pack — the column header `kernel.binding` is exactly such
//! a token (namespace `kernel`). The rule's own sanctioned remediation is
//! "add/enable a `kind: profile` pack file declaring the vocabulary";
//! this file is the executable proof that the remediation now works for
//! the kernel namespace (the pack itself: `packs/kernel-pack.md`, id
//! `kernel.pack` — the corpus tree holds capability specs only, so every
//! `kind: profile` file lives in `packs/`).
//!
//! The fixture workspace lives under the repository's `target/` directory
//! so the pack-discovery anchor (`git rev-parse --show-toplevel` of the
//! linted corpus) is the repository root — the real corpus pack is
//! discovered by scan alone, exactly as a consumer workspace discovers
//! its own packs. No config file, no registry, no environment variable.

#[path = "common/mod.rs"]
mod common;
use assert_cmd::Command;
use common::UnwrapPanic;
use serde_json::Value;

/// The CLI under test.
fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap_p()
}

/// A workspace fixture directory inside the repository (under `target/`,
/// which the pack-discovery scan skips — the fixture is a consumer of the
/// vocabulary, never a pack itself).
fn fixture_workspace(tag: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_p()
        .as_nanos();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("specodelic-tests")
        .join(format!("kernel-vocab-{tag}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap_p();
    dir
}

/// A lint-clean consumer spec; `binding_col` adds the `kernel.binding`
/// column to its Constraints table.
fn write_consumer(path: &std::path::Path, id: &str, binding_col: bool) {
    let cols = if binding_col {
        "| id | kind | expr | traces_to | kernel.binding |\n\
         |----|------|------|-----------|----------------|\n"
    } else {
        "| id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n"
    };
    let binding_cell = if binding_col {
        "`-k kernel_binding`"
    } else {
        ""
    };
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL carry a binding claim\"\n---\n\
             \n## Constraints\n\
             \n{cols}\
             | c1 | invariant | `holds` | [[{id}]] | {binding_cell} |\n\
             \n## Model\n\
             \n### States\n\
             \n- s1\n\
             - s2\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | t | s1 | s2 | [[{id}.c1]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p_c1 | unit | [[{id}.c1]] | `g()` | `x` |\n"
        ),
    )
    .unwrap_p();
}

/// The lint envelope of a workspace directory.
fn lint_json(dir: &std::path::Path) -> (Option<i32>, Value) {
    let out = spk()
        .args(["lint", dir.to_str().unwrap_p(), "--json"])
        .output()
        .unwrap_p();
    let stdout = String::from_utf8(out.stdout).unwrap_p();
    let json: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("lint envelope is not JSON ({e}): {stdout}"));
    (out.status.code(), json)
}

/// Every issue's rule id in the envelope.
fn issue_rules(json: &Value) -> Vec<String> {
    json["data"]["issues"]
        .as_array()
        .unwrap_p()
        .iter()
        .map(|i| i["rule_id"].as_str().unwrap_or_default().to_string())
        .collect()
}

/// Every warning message in the envelope.
fn warning_messages(json: &Value) -> Vec<String> {
    json["data"]["warnings"]
        .as_array()
        .unwrap_p()
        .iter()
        .map(|w| w["message"].as_str().unwrap_or_default().to_string())
        .collect()
}

/// The kernel pack's entry in a lint envelope's discovered-pack list.
fn kernel_pack(json: &Value) -> Value {
    let packs = json["data"]["packs"].as_array().unwrap_p();
    packs
        .iter()
        .find(|p| p["id"] == "kernel.pack")
        .cloned()
        .unwrap_or_else(|| panic!("the kernel pack must be discovered: {packs:?}"))
}

/// A path inside the repository.
fn repo_path(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

#[test]
fn kernel_binding_column_lints_clean_when_the_kernel_pack_is_discovered() {
    // The enablement: a file whose Constraints table carries the
    // `kernel.binding` column lints clean because the kernel vocabulary
    // pack is discovered by corpus scan alone — no config, no registry.
    let dir = fixture_workspace("enabled");
    write_consumer(&dir.join("demo.binding.md"), "demo.binding", true);
    let (code, json) = lint_json(&dir);
    let rules = issue_rules(&json);
    assert!(
        !rules.iter().any(|r| r == "orphan_vocabulary"),
        "the kernel vocabulary must not be orphaned once the kernel pack is \
         discovered (rules: {rules:?})"
    );
    assert_eq!(code, Some(0), "the workspace must lint clean: {json}");
    // The pack is discovered by scan alone — and it is the pack that
    // names the candidate the orphan rule would have demanded.
    let kernel = kernel_pack(&json);
    assert_eq!(
        kernel["base_pin"], 20,
        "the pack pins the current base revision"
    );
    assert_eq!(kernel["lifecycle"], "published");
    // The vocabulary use activates the pack advisory-first — the pack is
    // what the file's column matched.
    let warnings = warning_messages(&json);
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("kernel.pack") && w.contains("kernel.binding")),
        "the kernel pack must activate for the file using its vocabulary \
         (warnings: {warnings:?})"
    );
}

#[test]
fn file_without_the_binding_column_is_untouched_by_the_kernel_pack() {
    // A file using none of the pack's vocabulary keeps byte-identical
    // findings: no activation, no finding (the pack-fiber-relative
    // no_pack_no_change law).
    let dir = fixture_workspace("plain");
    write_consumer(&dir.join("demo.plain.md"), "demo.plain", false);
    let (code, json) = lint_json(&dir);
    assert_eq!(code, Some(0), "the workspace must lint clean: {json}");
    assert!(
        issue_rules(&json).is_empty(),
        "a file without the kernel vocabulary carries no findings: {json}"
    );
    let warnings = warning_messages(&json);
    assert!(
        !warnings.iter().any(|w| w.contains("kernel.pack")),
        "the kernel pack must not activate for a file using none of its \
         vocabulary (warnings: {warnings:?})"
    );
}

#[test]
fn the_kernel_pack_file_lints_clean_on_its_own() {
    // The pack artifact itself is clean: the six manifest facets are
    // present, non-empty and two-column shaped; the `## Kinds` rows are
    // pack-qualified (the narrowing rejection); the four layers are
    // wired (every constraint covered by a deriving property, every
    // transition guarded).
    let (code, json) = lint_json(&repo_path("packs/kernel-pack.md"));
    assert_eq!(code, Some(0), "the pack file must lint clean: {json}");
    assert!(
        issue_rules(&json).is_empty(),
        "the pack file carries no findings: {json}"
    );
}

#[test]
fn corpus_discovers_the_kernel_pack_by_scan_alone() {
    // The corpus itself discovers the pack — the discovery anchor is the
    // repository toplevel of the linted corpus, so no config file and no
    // registry are involved.
    let (code, json) = lint_json(&repo_path("openspec/specs"));
    assert_eq!(code, Some(0), "the corpus must stay issue-free: {json}");
    let kernel = kernel_pack(&json);
    assert_eq!(kernel["base_pin"], 20);
    let vocabulary = kernel["vocabulary"].as_array().unwrap_p();
    assert!(
        vocabulary.iter().any(|v| v == "kernel.binding"),
        "the pack must declare the kernel.binding vocabulary: {vocabulary:?}"
    );
}
