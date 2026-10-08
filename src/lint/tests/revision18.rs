//! Revision 18 (specodelic-mcy) test suite: the id-derivation law
//! (`spec.md` takes its id from its parent directory), the retirement of
//! `id: spec` (`dual_format_valid` no longer polices the id; `total_refs`
//! resolves corpus-wide), and the folded scope law. Kept as its own
//! submodule to stay under the library file_lines ratchet.

use super::*;

#[test]
fn dual_format_file_with_wrong_id_fires_id_matches_file() {
    // Revision 18 (specodelic-mcy): dual_format_valid no longer
    // polices the id — a mismatched id on a dual-format file is the
    // NAMING LAW's finding (id_matches_file), not dual_format_valid's.
    let spec = spec_at(
        "---\nid: other.thing\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## ADDED Requirements\n\n### Requirement: Something\nThe system SHALL do the thing.\n\n## Requirements\n\n### Requirement: Something\nThe system SHALL do the thing.\n",
        "spec.md",
    );
    let report = lint_corpus(&[spec]);
    assert!(
        !report
            .issues
            .iter()
            .any(|i| i.rule_id == "linter.dual_format_valid"),
        "dual_format_valid must not police the id: {:?}",
        report.issues
    );
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "linter.id_matches_file"),
        "a spec.md with a non-derived id fires id_matches_file: {:?}",
        report.issues
    );
}

#[test]
fn modified_delta_with_non_spec_id_fires_dual_format_valid() {
    // Revision 18 (specodelic-mcy): dual_format_valid no longer
    // polices the id — a real-id delta file under its capability
    // directory lints clean. The naming law (id_matches_file) is
    // the only id gate a delta file answers to.
    let spec = spec_at(
        "---\nid: demo.thing\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## MODIFIED Requirements\n\n### Requirement: One\none holds\n\n## Requirements\n\n### Requirement: One\none holds\n",
        "demo-thing/spec.md",
    );
    let report = lint_all(&[spec], &checklist_fixtures());
    assert!(
        !report.issues.iter().any(|i| {
            i.rule_id == "linter.dual_format_valid"
                || i.rule_id == "linter.requirement_drift"
                || i.rule_id == "linter.id_matches_file"
        }),
        "a real-id delta file under its capability dir must lint clean: {:?}",
        report.issues
    );
}

#[test]
fn spec_md_derives_expected_id_from_parent_dir() {
    // Revision 18 (specodelic-mcy): a file named spec.md derives its
    // expected id from the PARENT DIRECTORY name (`-` maps to `.`,
    // `_` is literal) — single-tree spec authoring needs no `id: spec`.
    let spec = spec_at(
        "---\nid: ge.cli\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | [[ge.cli]] |\n\n## Model\n\n### States\n\n- s1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[ge.cli.c]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[ge.cli.c]] | `g()` | `x` |\n",
        "openspec/specs/ge-cli/spec.md",
    );
    let report = lint_all(&[spec], &checklist_fixtures());
    assert!(
        !report
            .issues
            .iter()
            .any(|i| i.rule_id == "linter.id_matches_file"),
        "spec.md under ge-cli/ must accept id ge.cli: {:?}",
        report.issues
    );
}

#[test]
fn spec_md_with_legacy_id_spec_fires_id_matches_file() {
    // Revision 18 (specodelic-mcy): `id: spec` under a real
    // capability directory is now a naming-law violation — the wall
    // test from the issue: exactly one finding, id_matches_file.
    let spec = spec_at(
        "---\nid: spec\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | [[spec]] |\n\n## Model\n\n### States\n\n- s1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[spec.c]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[spec.c]] | `g()` | `x` |\n",
        "openspec/specs/ge-cli/spec.md",
    );
    let report = lint_all(&[spec], &checklist_fixtures());
    let findings: Vec<_> = report
        .issues
        .iter()
        .filter(|i| i.rule_id == "linter.id_matches_file")
        .collect();
    assert_eq!(
        findings.len(),
        1,
        "exactly one id_matches_file finding: {:?}",
        report.issues
    );
}

#[test]
fn spec_md_without_parent_dir_keeps_stem_derivation() {
    // A bare `spec.md` (no parent directory) falls back to the stem:
    // expected id stays `spec`.
    let spec = spec_at(
        "---\nid: demo.thing\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | [[demo.thing]] |\n\n## Model\n\n### States\n\n- s1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[demo.thing.c]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[demo.thing.c]] | `g()` | `x` |\n",
        "spec.md",
    );
    let report = lint_all(&[spec], &checklist_fixtures());
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "linter.id_matches_file"),
        "bare spec.md with a non-spec id must fire id_matches_file: {:?}",
        report.issues
    );
}

#[test]
fn spec_md_underscore_dir_is_literal() {
    // `_` in the parent directory name is literal within the id
    // segment; `-` maps to the namespace dot.
    let spec = spec_at(
        "---\nid: graph_shape\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | [[graph_shape]] |\n\n## Model\n\n### States\n\n- s1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[graph_shape.c]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[graph_shape.c]] | `g()` | `x` |\n",
        "openspec/specs/graph_shape/spec.md",
    );
    let report = lint_all(&[spec], &checklist_fixtures());
    assert!(
        !report
            .issues
            .iter()
            .any(|i| i.rule_id == "linter.id_matches_file"),
        "dir graph_shape must map to id graph_shape (`_` literal): {:?}",
        report.issues
    );
}

#[test]
fn real_id_dual_format_files_resolve_cross_file() {
    let a = spec_at(
        "---\nid: demo.alpha\nkind: intent\nstatement: \"THE alpha SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| ca | invariant | `x` | [[demo.alpha]] |\n\n## Model\n\n### States\n\n- s1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[demo.alpha.ca]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| pa | unit | [[demo.alpha.ca]] | `g()` | `x` |\n\n## Requirements\n\n### Requirement: A\nThe system SHALL hold.\n",
        "demo-alpha/spec.md",
    );
    let b = spec_at(
        "---\nid: demo.beta\nkind: intent\nstatement: \"THE beta SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| cb | invariant | `the cited alpha row holds` | [[demo.beta]] |\n\n## Model\n\n### States\n\n- s2\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s2 | s2 | [[demo.alpha.ca]] ∧ [[demo.beta.cb]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| pb | unit | [[demo.beta.cb]] | `g()` | `x` |\n\n## Requirements\n\n### Requirement: B\nThe system SHALL hold.\n",
        "demo-beta/spec.md",
    );
    let report = lint_corpus(&[a, b]);
    assert!(
        report.issues.is_empty(),
        "cross-file ref between real-id dual-format files must resolve: {:?}",
        report.issues
    );
}

#[test]
fn same_id_files_do_not_collide_in_reference_resolution() {
    let make = |c: &str, p: &str, path: &str| {
        spec_at(
            &format!(
                "---\nid: demo.cap\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| {c} | invariant | `x` | [[demo.cap]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[demo.cap.{c}]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| {p} | unit | [[demo.cap.{c}]] | `g()` | `x` |\n"
            ),
            path,
        )
    };
    let a = make("ca", "pa", "changes/x/specs/demo-cap/spec.md");
    let b = make("cb", "pb", "openspec/specs/demo-cap/spec.md");
    let report = lint_corpus(&[a, b]);
    assert!(
        report.issues.is_empty(),
        "same-id files must not dangle each other's rows: {:?}",
        report.issues
    );
}

#[test]
fn real_id_files_resolve_corpus_wide_and_typos_still_dangle() {
    let a = spec_at(
        "---\nid: demo.a\nkind: intent\nstatement: \"THE a SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| local_a | invariant | `x` | [[demo.a]] |\n| cross | invariant | `cites beta's row` | [[demo.a]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[demo.a.local_a]] ∧ [[demo.b.row_in_b]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| pa | unit | [[demo.a.local_a]] | `g()` | `x` |\n| pc | unit | [[demo.a.cross]] | `g()` | `y` |\n",
        "demo-a/spec.md",
    );
    let b = spec_at(
        "---\nid: demo.b\nkind: intent\nstatement: \"THE b SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| row_in_b | invariant | `z` | [[demo.b]] |\n\n## Model\n\n### States\n\n- `s2`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s2 | s2 | [[demo.b.row_in_b]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| pb | unit | [[demo.b.row_in_b]] | `g()` | `z` |\n",
        "demo-b/spec.md",
    );
    let report = lint_corpus(&[a.clone(), b.clone()]);
    assert!(
        report.issues.is_empty(),
        "cross-file refs between real-id files resolve corpus-wide: {:?}",
        report.issues
    );
    // A typo'd file segment still dangles (no id:spec collision to
    // hide behind).
    let typo = spec_at(
        "---\nid: demo.c\nkind: intent\nstatement: \"THE c SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| cc | invariant | `x` | [[demo.c]] |\n\n## Model\n\n### States\n\n- s3\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s3 | s3 | [[demo.ccc.cc]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| pc | unit | [[demo.c.cc]] | `g()` | `x` |\n",
        "demo-c/spec.md",
    );
    let report = lint_corpus(&[a, b, typo]);
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "linter.total_refs" && i.message.contains("demo.ccc.cc")),
        "a typo'd file segment must still dangle: {:?}",
        report.issues
    );
}
