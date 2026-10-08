// (split from tests/cli.rs — specodelic-g17 file_lines ratchet)
use super::*;

// ---- specodelic-zo9: dual_format_valid lint rule ----

#[test]
fn lint_flags_half_format_dual_file_via_cli() {
    // A file with frontmatter + ## ADDED Requirements but no sibling
    // ## Requirements is half a dual-format file — the rule fires so the
    // spec-integration protocol is tool-enforced, not convention.
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("spec.md"),
        "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## ADDED Requirements\n\n### Requirement: Something\nThe system SHALL do the thing.\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("dual_format_valid"),
        "half-format file flagged: {stdout}"
    );
    assert_eq!(out.status.code(), Some(1));
}

// ---- specodelic-6pi: recursive spec collection, no silent empty success ----

#[test]
fn lint_finds_specs_in_nested_directories() {
    // collect_specs must recurse: a spec in a subdirectory is linted,
    // not silently ignored (beads specodelic-6pi).
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("domain").join("deep");
    std::fs::create_dir_all(&nested).unwrap();
    write_bad_ears_spec(&nested.join("nested_spec.md"), "nested_spec");
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("nested_spec.md"),
        "nested spec must be linted: {stdout}"
    );
    assert!(stdout.contains("ears_syntax"));
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn lint_skips_hidden_and_build_directories() {
    // Recursive collection must not sweep .git/, target/, node_modules/:
    // exactly the top-level spec is linted, junk specs are never read.
    let dir = tempfile::tempdir().unwrap();
    write_bad_ears_spec(&dir.path().join("top_spec.md"), "top_spec");
    for junk in [".git", "target", "node_modules"] {
        let j = dir.path().join(junk);
        std::fs::create_dir_all(&j).unwrap();
        let id = format!("junk_{}", junk.trim_start_matches('.'));
        write_bad_ears_spec(&j.join(format!("{id}.md")), &id);
    }
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert_eq!(json["data"]["files_linted"], 1, "only the top-level spec");
    let issues = json["data"]["issues"].as_array().unwrap();
    assert!(!issues.is_empty(), "top_spec's ears_syntax must fire");
    assert!(
        issues.iter().all(|i| i["file"]
            .as_str()
            .map(|f| f.contains("top_spec.md"))
            .unwrap_or(false)),
        "no finding may come from hidden/build dirs: {issues:?}"
    );
}

#[test]
fn lint_fails_with_hint_when_no_specs_found() {
    // A directory with no spec files must not yield a silent ok:true —
    // that's a false green (beads specodelic-6pi). Failure message rides
    // stderr even in JSON mode (convention of record).
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("README.md"), "# not a spec\n").unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("no spec files found"), "stderr: {stderr}");
    // JSON mode keeps the remediation hint inside the envelope (stdout);
    // the footer prints to stderr only in human mode (convention of record)
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("frontmatter"), "envelope hint: {stdout}");
    // Invocation error exits 2 (specodelic-7rr item 2) — distinct from
    // findings (1); the envelope kind agrees (error).
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn lint_fails_on_parse_error_even_when_other_specs_parse() {
    // specodelic-in9: a malformed frontmatter file in a directory that
    // also holds parseable specs rode the warnings channel of a success
    // envelope — exit 0 — so the pre-commit gate let a corrupted spec
    // commit (empirically verified with a gate-probe commit). A parse
    // error is a failure regardless of the rest of the batch.
    let dir = tempfile::tempdir().unwrap();
    let good = dir.path().join("good.md");
    write_parse_fixture(&good, "good");
    let broken = dir.path().join("broken.md");
    std::fs::write(&broken, "---\nid: [unclosed\nkind: intent\n---\nbody\n").unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_ne!(
        out.status.code(),
        Some(0),
        "parse error must fail the lint stage (exit 0 = gate passes)"
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("broken.md"),
        "the parse error must name the offending file: {stdout}"
    );

    // Single malformed file keeps its labeled invocation failure (exit 2).
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("broken.md"), "---\nid: [unclosed\n").unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn lint_corpus_is_fully_clean() {
    // The corpus is the first dogfood target: every spec file must parse,
    // every reference must resolve, and every constraint must be covered
    // by a deriving property (beads specodelic-qc8 closed the last gaps).
    let out = spk().args(["lint", "specs", "--json"]).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let data = &json["data"];
    assert_eq!(data["files_linted"], 22); // + errors.md + linter-failure_shape.md + packs.md (add-domain-pack-mechanism)
    let issues = data["issues"].as_array().unwrap();
    assert!(issues.is_empty(), "corpus lint findings: {issues:?}");
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn graph_corpus_is_fully_resolved() {
    let out = spk().args(["graph", "specs", "--json"]).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let data = &json["data"];
    assert!(data["dangling"].as_array().unwrap().is_empty());
    assert!(data["edges"].as_array().unwrap().len() > 300);
    // specodelic-cxq (2026-09-30): the corpus is reconciled with the
    // Reference Typing table — zero typing-forbidden edges, and the
    // graph therefore exits 0 (the old "violations tolerated until
    // cxq" escape hatch is gone).
    let violations = data["violations"].as_array().unwrap();
    assert!(
        violations.is_empty(),
        "corpus typing violations: {violations:?}"
    );
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn graph_real_id_files_resolve_corpus_wide_and_typos_dangle() {
    // Revision 18 (specodelic-mcy): the self-containment law retired
    // with `id: spec` — distinct real-id files resolve corpus-wide (a
    // cross-file ref between them produces an edge, no dangling), while
    // a typo'd file segment still dangles (the CORR-001 protection now
    // rests on real ids, not on a scoped index).
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("demo-a");
    let b = dir.path().join("demo-b");
    std::fs::create_dir_all(&a).unwrap();
    std::fs::create_dir_all(&b).unwrap();
    std::fs::write(
        a.join("spec.md"),
        "---\nid: demo.a\nkind: intent\nstatement: \"THE a SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| local_a | invariant | `x` | [[demo.a]] |\n| cross | invariant | `cites beta's row` | [[demo.a]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[demo.a.local_a]] ∧ [[demo.b.row_in_b]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| pa | unit | [[demo.a.local_a]] | `g()` | `x` |\n| pc | unit | [[demo.a.cross]] | `g()` | `x` |\n",
    )
    .unwrap();
    std::fs::write(
        b.join("spec.md"),
        "---\nid: demo.b\nkind: intent\nstatement: \"THE b SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| row_in_b | invariant | `z` | [[demo.b]] |\n\n## Model\n\n### States\n\n- `s2`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s2 | s2 | [[demo.b.row_in_b]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| pb | unit | [[demo.b.row_in_b]] | `g()` | `z` |\n",
    )
    .unwrap();
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        !stdout.contains("demo.b.row_in_b is not defined"),
        "cross-file ref between real-id files must resolve: {stdout}"
    );
    // Typo'd file segment still dangles.
    let c = dir.path().join("demo-c");
    std::fs::create_dir_all(&c).unwrap();
    std::fs::write(
        c.join("spec.md"),
        "---\nid: demo.c\nkind: intent\nstatement: \"THE c SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| cc | invariant | `x` | [[demo.c]] |\n\n## Model\n\n### States\n\n- `s3`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s3 | s3 | [[demo.ccc.cc]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| pc | unit | [[demo.c.cc]] | `g()` | `x` |\n",
    )
    .unwrap();
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("demo.ccc"),
        "typo'd file segment must dangle in the graph: {stdout}"
    );
}

/// Rev 7 of kinds.md — CORR-003 (RED first): intent_row_shape now
/// acknowledges optionally-declared frontmatter keys (currently
/// `checked_against_core`, per specs/AGENTS.md) beside the three base
/// fields. The linter's frontmatter_valid stays a subset check, so a
/// spec carrying the extra key must lint clean — this pins the
/// spec-text reading at tool level (a future stricter check would fail
/// here loudly, never silently).
#[test]
fn extra_frontmatter_key_lints_clean() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("extra.md"),
        "---\nid: extra\nkind: intent\nchecked_against_core: clear\nstatement: \"THE extra SHALL carry an optional key\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | [[extra]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[extra.c]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p | unit | [[extra.c]] | `g()` | `x` |\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "exit 0");
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    let issues = json["data"]["issues"].as_array().unwrap();
    assert!(
        issues.is_empty(),
        "optional frontmatter key must lint clean (subset reading): {issues:?}"
    );
    // The parser must actually capture the key as an extra field, not
    // silently drop it — this is what makes the spec-text assertion
    // meaningful rather than vacuous (an unparsed key would "pass" any
    // future strict check).
    let out = spk()
        .args([
            "parse",
            dir.path().join("extra.md").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert!(
        json["data"]["intent"]["extra"]["checked_against_core"].is_string(),
        "parser captures the extra frontmatter field: {json}"
    );
}

/// Rev 10 tiered own-file reachability (specodelic-erb, RED first):
/// `single_root_reachable` lands on the file's OWN intent through
/// own-file primary linkage; cross-file typed edges (guard citations of
/// foreign constraints, satisfies, observes) are outbound leaves, never
/// reachability paths. Tiered: a row whose ONLY tie is cross-file gets
/// an advisory warning (warnings channel), never a silent pass — while
/// a row with no path to ANY intent still hard-fails.
#[test]
fn cross_file_only_rows_warn_advisory_never_silent() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("base.md"),
        "---\nid: base\nkind: intent\nstatement: \"THE base SHALL anchor\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| base_invariant | invariant | `x` | [[base]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[base.base_invariant]] |\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("consumer.md"),
        "---\nid: consumer\nkind: intent\nstatement: \"THE consumer SHALL consume\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| orphan_c | invariant | `y` | |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[base.base_invariant]] |\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let issues = json["data"]["issues"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["rule_id"] == "linter.single_root_reachable")
        .cloned()
        .collect::<Vec<_>>();
    // Hard tier: the unlinked row still fails.
    assert_ne!(out.status.code(), Some(0), "orphan row must hard-fail");
    assert!(
        issues
            .iter()
            .any(|i| i["message"].as_str().unwrap().contains("orphan_c")),
        "row with no path to ANY intent must hard-fail: {issues:?}"
    );
    // The cross-file-only component {t, s1} must NOT hard-fail.
    assert!(
        !issues.iter().any(|i| i["message"]
            .as_str()
            .unwrap()
            .contains("[[base.base_invariant]]"))
            || !issues.iter().any(|i| {
                let m = i["message"].as_str().unwrap();
                (m.contains(" t") || m.contains("s1")) && !m.contains("orphan_c")
            }),
        "cross-file-only rows must not ride the issues channel: {issues:?}"
    );
    // Advisory tier: the same rows warn on the warnings channel.
    let warnings: Vec<String> = json["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["message"].as_str().unwrap_or_default().to_string())
        .collect();
    assert!(
        warnings.iter().any(|w| w.contains("single_root_reachable")
            && w.contains("t")
            && w.contains("s1")
            && w.contains("cross-file")),
        "cross-file-only rows must produce the advisory warning: {warnings:?}"
    );
}

/// add-error-contract task 1.1 (RED first): the corpus must publish the
/// cross-cutting output contract — specs/errors.md owns the three
/// `extension_point` rows, each pointing at its own intent and each
/// covered by a falsifying property (error_property_names_label's
/// coverage half is enforced today; the exact-label half is tier 2).
/// Fails before specs/errors.md exists — observed RED 2026-09-30.
#[test]
fn error_contract_rows_are_published() {
    let out = spk().args(["graph", "specs", "--json"]).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let edges = json["data"]["edges"].as_array().unwrap();
    for row in [
        "envelope_error_kind",
        "exit_code_mapping",
        "remediation_hint_present",
    ] {
        let id = format!("errors.{row}");
        assert!(
            edges.iter().any(|e| e["kind"] == "constraints.traces_to"
                && e["from"] == id
                && e["to"] == "errors"),
            "contract row {id} not published (no traces_to edge to its own intent)"
        );
        assert!(
            edges
                .iter()
                .any(|e| e["kind"] == "properties.derives_from" && e["to"] == id),
            "contract row {id} has no falsifying property"
        );
    }
}

/// add-error-contract task 2.1 (RED first): the per-file gaps named
/// mechanically from the graph. After phase 2 this holds:
/// - every failure terminal (inbound transitions.to, no outbound
///   transitions.from, name containing "fail") emits exactly ONE
///   file-owned labeled error Constraint (single_labeled_failure at
///   model altitude), and every label is property-covered;
/// - the exact label set per file matches the phase-2 plan (compile
///   splits extraction/emission; multi-class files split failure states);
/// - every failure transition with ≥1 guard citations cites exactly the
///   citation set of its success sibling (guard_negation_typed, D2);
/// - the only zero-citation failure guards are orchestrate's carved-out
///   stage-fails (the D2 carve-out is a checked list);
/// - no state carries ≥2 inbound failure transitions with differing
///   citation sets (failure_class_is_state, D2a).
#[test]
fn failure_terminals_emit_labeled_errors() {
    let out = spk().args(["graph", "specs", "--json"]).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let edges = json["data"]["edges"].as_array().unwrap();
    let e = |k: &str| -> Vec<(String, String)> {
        edges
            .iter()
            .filter(|x| x["kind"] == k)
            .map(|x| {
                (
                    x["from"].as_str().unwrap().to_string(),
                    x["to"].as_str().unwrap().to_string(),
                )
            })
            .collect()
    };
    let to_state = e("transitions.to");
    let from_state = e("transitions.from");
    let guard = e("transitions.guard");
    let emits = e("states.emits");
    let derives = e("properties.derives_from");

    let outbound: std::collections::BTreeSet<&str> =
        from_state.iter().map(|(_, s)| s.as_str()).collect();
    let mut inbound: std::collections::BTreeMap<&str, Vec<&str>> =
        std::collections::BTreeMap::new();
    for (f, t) in &to_state {
        inbound.entry(t.as_str()).or_default().push(f);
    }
    // failure terminals: terminal states whose STATE segment is named *fail*
    // (rsplit: the node id is <file-id>.<state-id> and a file id may itself
    // contain "failure" — linter.failure_shape)
    let is_fail_state = |node: &str| -> bool {
        node.rsplit_once('.')
            .map(|(_, st)| st.contains("fail"))
            .unwrap_or(false)
    };
    let failure_terminals: Vec<&str> = inbound
        .iter()
        .filter(|(s, _)| !outbound.contains(**s) && is_fail_state(s))
        .map(|(s, _)| *s)
        .collect();
    assert!(
        failure_terminals.len() >= 20,
        "expected the split corpus's failure terminals, got {failure_terminals:?}"
    );

    // exactly one labeled error per failure state, label property-covered
    for s in &failure_terminals {
        let targets: Vec<&String> = emits
            .iter()
            .filter(|(f, _)| f.as_str() == *s)
            .map(|(_, t)| t)
            .collect();
        assert_eq!(
            targets.len(),
            1,
            "failure terminal {s} must emit exactly one labeled error"
        );
        let label = targets[0];
        assert!(
            derives.iter().any(|(_, t)| t == label),
            "error label {label} has no falsifying property"
        );
    }

    // exact label set per file (phase-2 plan)
    let expected: &[(&str, &[&str])] = &[
        (
            "compile",
            &["compile.extraction_failure", "compile.emission_failure"],
        ),
        (
            "rename",
            &[
                "rename.validation_failure",
                "rename.apply_failure",
                "rename.post_check_failure",
            ],
        ),
        (
            "linter.coverage",
            &[
                "linter.coverage.count_failure",
                "linter.coverage.law_case_failure",
            ],
        ),
        (
            "linter.referential_integrity",
            &[
                "linter.referential_integrity.index_failure",
                "linter.referential_integrity.resolution_failure",
            ],
        ),
        (
            "linter.ears_syntax",
            &[
                "linter.ears_syntax.pattern_failure",
                "linter.ears_syntax.id_check_failure",
            ],
        ),
        (
            "linter.external_completeness",
            &[
                "linter.external_completeness.manifest_failure",
                "linter.external_completeness.mapping_failure",
                "linter.external_completeness.resolution_failure",
            ],
        ),
        ("linter.frontmatter", &["linter.frontmatter.check_failure"]),
        ("linter.graph_shape", &["linter.graph_shape.check_failure"]),
        (
            "linter.model_shape",
            &[
                "linter.model_shape.pairing_failure",
                "linter.model_shape.field_check_failure",
            ],
        ),
        (
            "linter.schema_shape",
            &[
                "linter.schema_shape.kind_check_failure",
                "linter.schema_shape.diff_failure",
                "linter.schema_shape.parser_audit_failure",
            ],
        ),
        ("graph", &["graph.extraction_failure"]),
        (
            "kinds",
            &["kinds.kind_assignment_failure", "kinds.shape_check_failure"],
        ),
        (
            "merge",
            &[
                "merge.collision_failure",
                "merge.reverification_failure",
                "merge.merge_aborted",
            ],
        ),
        (
            "orchestrate",
            &[
                "orchestrate.lint_stage_failure",
                "orchestrate.compile_stage_failure",
                "orchestrate.model_check_stage_failure",
                "orchestrate.verify_stage_failure",
            ],
        ),
        ("verify", &["verify.verification_failure"]),
    ];
    for (file, labels) in expected {
        let prefix = format!("{file}.");
        let mut got: Vec<&str> = emits
            .iter()
            .filter(|(f, _)| f.as_str().starts_with(&prefix))
            .map(|(_, t)| t.as_str())
            .collect();
        got.sort();
        let mut want: Vec<&str> = labels.to_vec();
        want.sort();
        assert_eq!(got, want, "emits label set mismatch for {file}");
    }

    // guard_negation_typed + failure_class_is_state, mechanically
    let from_of: std::collections::BTreeMap<&str, &str> = from_state
        .iter()
        .map(|(t, s)| (t.as_str(), s.as_str()))
        .collect();
    let to_of: std::collections::BTreeMap<&str, &str> = to_state
        .iter()
        .map(|(t, s)| (t.as_str(), s.as_str()))
        .collect();
    let cites = |t: &str| -> Vec<&str> {
        let mut v: Vec<&str> = guard
            .iter()
            .filter(|(f, _)| f.as_str() == t)
            .map(|(_, x)| x.as_str())
            .collect();
        v.sort();
        v
    };
    let mut by_state: std::collections::BTreeMap<&str, Vec<&str>> =
        std::collections::BTreeMap::new();
    for (t, s) in &to_of {
        if is_fail_state(s) {
            by_state.entry(*s).or_default().push(t);
        }
    }
    for (s, fails) in &by_state {
        // failure_class_is_state: differing citation sets must be split
        let mut sets: Vec<Vec<&str>> = fails.iter().map(|t| cites(t)).collect();
        sets.sort();
        sets.dedup();
        assert_eq!(
            sets.len(),
            1,
            "state {s} carries {}/{} differing failure citation sets — split the classes",
            sets.len(),
            fails.len()
        );
        for t in fails {
            let c = cites(t);
            if c.is_empty() {
                // the D2 carve-out is a checked list: only orchestrate's
                // stage-fails may stay prose
                assert!(
                    t.starts_with("orchestrate."),
                    "zero-citation failure guard {t} is outside the carve-out list"
                );
                continue;
            }
            // the success siblings from the same source state are what
            // the failure guard negates (a disjunction negates the union);
            // citation sets must be equal
            let src = from_of.get(t).copied().unwrap();
            let siblings: Vec<&str> = to_of
                .iter()
                .filter(|(id, to)| from_of.get(*id).copied() == Some(src) && !is_fail_state(to))
                .map(|(id, _)| *id)
                .collect();
            assert!(
                !siblings.is_empty(),
                "failure transition {t} has no success sibling"
            );
            let mut want: Vec<&str> = siblings.iter().flat_map(|s| cites(s)).collect();
            want.sort();
            want.dedup();
            assert_eq!(
                c,
                cites(siblings[0]),
                "failure transition {t} must cite exactly its negated sibling's citation set"
            );
        }
    }
}

// ---- specodelic-ct5: linter.failure_shape checker (specs/linter-failure_shape.md) ----
// The three rules are graph-decidable per file; these tests reuse the
// failure_terminals_emit_labeled_errors derivation (above) at single-file
// lint altitude: mute terminals, label collisions, and guard negation
// totals are lint findings (tier 2 of errors.md's enforcement_routed row).

/// A lint-clean tool file with one failure terminal that emits a labeled
/// error, guarded by the sibling's citation set — the shape every rule
/// below perturbs.
fn write_clean_failure_shape_spec(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE tool SHALL label its failures\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | inv | invariant | `x` | [[{id}]] |\n\
             | x_failure | effect | `{id}.x_failure(detail)` | [[{id}]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- s\n\
             - failed (emits: `[[{id}.x_failure]]`)\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | ok | s | s | [[{id}.inv]] |\n\
             | boom | s | failed | [[{id}.inv]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p_inv | unit | [[{id}.inv]] | `g()` | `x` |\n\
             | p_x | unit | [[{id}.x_failure]] | `g()` | `x` |\n"
        ),
    )
    .unwrap();
}

fn lint_issues(dir: &tempfile::TempDir) -> (Option<i32>, Vec<String>) {
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let issues = json["data"]["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["rule_id"].as_str().unwrap().to_string())
        .collect();
    (out.status.code(), issues)
}

/// terminal_states_emit: a failure terminal that emits nothing is a
/// finding — the corpus's pre-add-error-contract shape, now labeled
/// (mute_terminal_rejected).
#[test]
fn lint_mute_failure_terminal_flagged() {
    let dir = tempfile::tempdir().unwrap();
    write_clean_failure_shape_spec(&dir.path().join("d-mute.md"), "d.mute");
    // strip the emits cell from the failure state
    let p = dir.path().join("d-mute.md");
    let text = std::fs::read_to_string(&p).unwrap();
    std::fs::write(
        &p,
        text.replace("- failed (emits: `[[d.mute.x_failure]]`)", "- failed"),
    )
    .unwrap();

    let (code, issues) = lint_issues(&dir);
    assert_eq!(code, Some(1));
    assert_eq!(
        issues,
        vec!["linter.terminal_states_emit".to_string()],
        "exactly the mute-terminal finding, nothing else: {issues:?}"
    );
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let msg = json["data"]["issues"][0]["message"].as_str().unwrap();
    assert!(msg.contains("failed"), "names the mute state: {msg}");
    assert!(
        msg.contains("emits"),
        "carries a remediation hint (remediation_hint_present): {msg}"
    );
}

/// error_labels_unique: two error Constraints sharing a variant head
/// collide within the file (label_collision_rejected). The dotted row id
/// makes the second emitted label's head `x_failure` again — a future
/// format change dropping the namespacing law is caught here.
#[test]
fn lint_error_label_collision_flagged() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("d-lbl.md"),
        "---\nid: d.lbl\nkind: intent\nstatement: \"THE d SHALL label its failures\"\n---\n\
         \n## Constraints\n\
         \n| id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | inv | invariant | `x` | [[d.lbl]] |\n\
         | x_failure | effect | `d.lbl.x_failure(detail)` | [[d.lbl]] |\n\
         | sub.x_failure | effect | `d.lbl.sub.x_failure(detail)` | [[d.lbl]] |\n\
         \n## Model\n\
         \n### States\n\
         \n- s\n\
         - f1 (emits: `[[d.lbl.x_failure]]`)\n\
         - f2 (emits: `[[d.lbl.sub.x_failure]]`)\n\
         \n### Transitions\n\
         \n| id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | ok | s | s | [[d.lbl.inv]] |\n\
         | boom1 | s | f1 | [[d.lbl.inv]] |\n\
         | boom2 | s | f2 | [[d.lbl.inv]] |\n\
         \n## Properties\n\
         \n| id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|------------|\n\
         | p_inv | unit | [[d.lbl.inv]] | `g()` | `x` |\n\
         | p_x | unit | [[d.lbl.x_failure]] | `g()` | `x` |\n\
         | p_sub | unit | [[d.lbl.sub.x_failure]] | `g()` | `x` |\n",
    )
    .unwrap();
    let (code, issues) = lint_issues(&dir);
    assert_eq!(code, Some(1));
    assert!(
        issues.contains(&"linter.error_labels_unique".to_string()),
        "the collision is labeled: {issues:?}"
    );
}

/// guard_negation_total, zero-citation leg: a failure transition citing
/// nothing and off the carve-out list is a finding, never a silent pass
/// (zero_citation_flagged).
#[test]
fn lint_zero_citation_failure_guard_flagged() {
    let dir = tempfile::tempdir().unwrap();
    write_clean_failure_shape_spec(&dir.path().join("d-zc.md"), "d.zc");
    let p = dir.path().join("d-zc.md");
    let text = std::fs::read_to_string(&p).unwrap();
    std::fs::write(
        &p,
        text.replace(
            "| boom | s | failed | [[d.zc.inv]] |",
            "| boom | s | failed | `the world ends` |",
        ),
    )
    .unwrap();

    let (code, issues) = lint_issues(&dir);
    assert_eq!(code, Some(1));
    assert_eq!(
        issues,
        vec!["linter.guard_negation_total".to_string()],
        "exactly the zero-citation finding, nothing else: {issues:?}"
    );
}

/// guard_negation_total, carve-out leg: orchestrate's stage-fail
/// transitions stay prose-guarded — the carve-out is CHECKED membership
/// (file orchestrate + the four stage-fail transition ids), not assumed
/// (carved_out_guard_passes).
#[test]
fn lint_carved_out_stage_fail_passes() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("orchestrate.md"),
        "---\nid: orchestrate\nkind: intent\nstatement: \"THE orchestrate SHALL stage its checks\"\n---\n\
         \n## Constraints\n\
         \n| id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | inv | invariant | `x` | [[orchestrate]] |\n\
         | lint_stage_failure | effect | `orchestrate.lint_stage_failure(stage, detail)` | [[orchestrate]] |\n\
         \n## Model\n\
         \n### States\n\
         \n- stage\n\
         - lint_failed (emits: `[[orchestrate.lint_stage_failure]]`)\n\
         \n### Transitions\n\
         \n| id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | lint_ok | stage | stage | [[orchestrate.inv]] |\n\
         | lint_fail | stage | lint_failed | `¬lint_ok.guard` |\n\
         \n## Properties\n\
         \n| id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|------------|\n\
         | p_inv | unit | [[orchestrate.inv]] | `g()` | `x` |\n\
         | p_lsf | unit | [[orchestrate.lint_stage_failure]] | `g()` | `x` |\n",
    )
    .unwrap();
    let (code, issues) = lint_issues(&dir);
    assert_eq!(code, Some(0), "carve-out membership is checked: {issues:?}");
    assert!(issues.is_empty(), "no findings: {issues:?}");
}

/// guard_negation_total, mismatch leg: a failure transition citing a set
/// that differs from its success siblings' union is a finding —
/// citation-set inequality is graph-decidable
/// (negation_set_mismatch_flagged).
#[test]
fn lint_negation_set_mismatch_flagged() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("d-mm.md"),
        "---\nid: d.mm\nkind: intent\nstatement: \"THE d SHALL negate its guards\"\n---\n\
         \n## Constraints\n\
         \n| id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | inv | invariant | `x` | [[d.mm]] |\n\
         | other | invariant | `y` | [[d.mm]] |\n\
         | x_failure | effect | `d.mm.x_failure(detail)` | [[d.mm]] |\n\
         \n## Model\n\
         \n### States\n\
         \n- s\n\
         - failed (emits: `[[d.mm.x_failure]]`)\n\
         \n### Transitions\n\
         \n| id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | ok | s | s | [[d.mm.inv]] |\n\
         | boom | s | failed | [[d.mm.other]] |\n\
         \n## Properties\n\
         \n| id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|------------|\n\
         | p_inv | unit | [[d.mm.inv]] | `g()` | `x` |\n\
         | p_other | unit | [[d.mm.other]] | `g()` | `x` |\n\
         | p_x | unit | [[d.mm.x_failure]] | `g()` | `x` |\n",
    )
    .unwrap();
    let (code, issues) = lint_issues(&dir);
    assert_eq!(code, Some(1));
    assert_eq!(
        issues,
        vec!["linter.guard_negation_total".to_string()],
        "exactly the negation-mismatch finding, nothing else: {issues:?}"
    );
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let msg = json["data"]["issues"][0]["message"].as_str().unwrap();
    assert!(
        msg.contains("d.mm.inv"),
        "the finding names the expected citation set: {msg}"
    );
}

/// total_extraction (specs/graph.md): a Transition's `from`/`to` cells are
/// typed reference fields (→ State) and must yield exactly one edge each.
#[test]
fn graph_extracts_from_to_state_edges() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("t.md"),
        "---\nid: t\nkind: intent\nstatement: \"THE t SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `x` | [[t]] |\n\n## Model\n\n### States\n\n- `a`\n- `b`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| go | a | b | [[t.c1]] |\n| bad | a | zz | [[t.c1]] |\n",
    )
    .unwrap();
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let data = &json["data"];
    let edges = data["edges"].as_array().unwrap();
    for (kind, to) in [("transitions.from", "t.a"), ("transitions.to", "t.b")] {
        assert!(
            edges
                .iter()
                .any(|e| e["kind"] == kind && e["from"] == "t.go" && e["to"] == to),
            "missing {kind} state edge: {edges:?}"
        );
    }
    // An unknown state in from/to dangles — never silently dropped.
    let dangling = data["dangling"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d.as_str().unwrap())
        .collect::<Vec<_>>()
        .join("; ");
    assert!(
        dangling.contains("zz"),
        "unknown from/to state must dangle: {dangling}"
    );
}

/// edge_kind_matches_typing (specs/graph.md): the graph never records an
/// edge the Reference Typing table wouldn't allow — a Constraint's
/// `traces_to` must resolve to an Intent, so a traces_to→Constraint row
/// surfaces as a labeled violation instead of a silent edge.
#[test]
fn graph_typing_forbidden_edge_is_reported_not_recorded() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("t.md"),
        "---\nid: t\nkind: intent\nstatement: \"THE t SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `x` | [[t]] |\n| c2 | invariant | `y` | [[t.c1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p1 | unit | [[t.c1]] | `g()` | `x` |\n",
    )
    .unwrap();
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let data = &json["data"];
    let edges = data["edges"].as_array().unwrap();
    // Control: the legal derives_from edge is still recorded.
    assert!(
        edges
            .iter()
            .any(|e| e["kind"] == "properties.derives_from" && e["to"] == "t.c1"),
        "legal derives_from edge must be recorded: {edges:?}"
    );
    assert!(
        !edges
            .iter()
            .any(|e| e["kind"] == "constraints.traces_to" && e["to"] == "t.c1"),
        "typing-forbidden edge must not be recorded: {edges:?}"
    );
    let violations = data["violations"].as_array().unwrap();
    assert!(
        violations
            .iter()
            .any(|v| v["edge_kind"] == "constraints.traces_to" && v["to"] == "t.c1"),
        "forbidden edge must surface as a labeled violation: {violations:?}"
    );
    assert_eq!(out.status.code(), Some(1));
}

/// specodelic-huf: the Reference Typing table's **Appears on** column is
/// normative — `derives_from` appears on Property rows only. A Constraint
/// row carrying `derives_from` is out-of-format: a labeled typing
/// violation, never a recorded edge — and therefore never a member of any
/// acyclic edge set (linter-graph_shape.md's invariant is qualified to
/// the same edge set).
#[test]
fn constraint_row_derives_from_is_a_typing_violation() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("t.md"),
        "---\nid: t\nkind: intent\nstatement: \"THE t SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | derives_from |\n|----|------|------|-----------|--------------|\n| c1 | invariant | `x` | [[t]] | [[t.c2]] |\n| c2 | invariant | `y` | [[t]] | |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| go | s1 | s1 | [[t.c1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p1 | unit | [[t.c1]] | `g()` | `x` |\n",
    )
    .unwrap();
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let data = &json["data"];
    let edges = data["edges"].as_array().unwrap();
    // Control: the legal property→constraint derives_from edge is still
    // recorded.
    assert!(
        edges
            .iter()
            .any(|e| e["kind"] == "properties.derives_from" && e["to"] == "t.c1"),
        "legal derives_from edge must be recorded: {edges:?}"
    );
    assert!(
        !edges
            .iter()
            .any(|e| e["kind"] == "constraints.derives_from"),
        "a Constraint-row derives_from must not be recorded as an edge: {edges:?}"
    );
    let violations = data["violations"].as_array().unwrap();
    assert!(
        violations
            .iter()
            .any(|v| v["edge_kind"] == "constraints.derives_from" && v["to"] == "t.c2"),
        "out-of-format source must surface as a labeled violation: {violations:?}"
    );
    assert_eq!(out.status.code(), Some(1));
}

/// supersedes_dag (specs/linter-graph_shape.md) + total_extraction: the
/// `supersedes` column yields edges (Constraint→Constraint, Property→
/// Property), a supersedes cycle is reported rather than silently served,
/// and a cross-kind supersedes is a typing violation.
#[test]
fn graph_supersedes_edges_cycle_and_typing() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("t.md"),
        "---\nid: t\nkind: intent\nstatement: \"THE t SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | supersedes |\n|----|------|------|-----------|------------|\n| c1 | invariant | `x` | [[t]] | [[t.c2]] |\n| c2 | invariant | `y` | [[t]] | [[t.c1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate | supersedes |\n|----|------|--------------|-----------|------------|------------|\n| p1 | unit | [[t.c1]] | `g()` | `x` | [[t.c1]] |\n",
    )
    .unwrap();
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let data = &json["data"];
    let edges = data["edges"].as_array().unwrap();
    assert!(
        edges.iter().any(|e| e["kind"] == "constraints.supersedes"
            && e["from"] == "t.c1"
            && e["to"] == "t.c2"),
        "supersedes edge must be recorded: {edges:?}"
    );
    let cycles = data["supersedes_cycles"].as_array().unwrap();
    assert!(
        cycles
            .iter()
            .any(|c| c.as_str().unwrap().contains("t.c1") && c.as_str().unwrap().contains("t.c2")),
        "supersedes cycle must be reported: {cycles:?}"
    );
    let violations = data["violations"].as_array().unwrap();
    assert!(
        violations
            .iter()
            .any(|v| v["edge_kind"] == "properties.supersedes"),
        "cross-kind supersedes must be a typing violation: {violations:?}"
    );
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn lint_synthetically_bad_file_fails_with_rule_name() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad_spec.md");
    std::fs::write(
        &bad,
        "---\nid: bad_spec\nkind: intent\nstatement: \"the system should maybe work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[bad_spec]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s2 | |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[bad_spec.a]] | `g()` | `x` |\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    // ears_syntax (no SHALL) + guard_required (empty guard) both fire
    assert!(stdout.contains("ears_syntax"));
    assert!(stdout.contains("guard_required"));
    // self-describing findings: rule id + semantics in the JSON payload
    assert!(stdout.contains("linter.ears_syntax"));
    assert!(stdout.contains("rule_semantics"));
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn bad_ears_finding_carries_rule_id_and_semantics() {
    // spec scenario: agent reads a finding without repo access — the
    // JSON finding names the rule id and states what the rule requires
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad_spec.md");
    std::fs::write(
        &bad,
        "---\nid: bad_spec\nkind: intent\nstatement: \"the system should maybe work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[bad_spec]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s2 | |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[bad_spec.a]] | `g()` | `x` |\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    let ears = json["data"]["issues"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["rule_id"] == "linter.ears_syntax")
        .expect("bad-EARS fixture must yield a linter.ears_syntax finding");
    assert!(!ears["rule_semantics"].as_str().unwrap().is_empty());
    // the bare `rule` field was dropped in the review pass — rule_id is
    // the single naming (rule ids are stable, linter.<name>)
}

#[test]
fn human_output_shows_rule_id() {
    // spec scenario: human output shows the rule id
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad_spec.md");
    std::fs::write(
        &bad,
        "---\nid: bad_spec\nkind: intent\nstatement: \"the system should maybe work\"\n---\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--human"])
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(0));
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("linter.ears_syntax")
    );
}

#[test]
fn explain_lint_rules_renders_the_rule_catalog() {
    // `explain lint-rules` renders from the same table the linter emits
    // from — the rendered catalog is exactly the emittable rule id set
    let out = spk()
        .args(["explain", "lint-rules", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    let body = json["data"]["body"].as_str().unwrap();
    assert!(!body.contains("{{"));
    assert!(!body.contains("shipped with the self-describing lint findings change"));
    // the rendered catalog enumerates every rule id the linter can emit
    // (review EDGE-001b) — loop over the lib's RULE_TABLE directly
    for (name, _) in specodelic::lint::RULE_TABLE {
        let id = format!("linter.{name}");
        assert!(body.contains(&id), "catalog missing rule id {id}");
    }
}

#[test]
fn new_scaffolds_a_spec_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = spk()
        .args(["new", "demo.thing", "--file", "demo-thing.md"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let content = std::fs::read_to_string(dir.path().join("demo-thing.md")).unwrap();
    assert!(content.contains("id: demo.thing"));
    // the scaffold carries per-layer guidance comments that teach the
    // format in place (task 4.1); closed sets render from the guide
    // constants
    assert!(content.contains("<!-- Intent layer:"));
    assert!(content.contains("<!-- kind: one of invariant | advisory | effect | extension_point"));
    assert!(content.contains("<!-- kind: one of unit | law"));
    assert!(content.contains("<!-- guard: may cite an invariant Constraint"));
    // the scaffold is lintable shape-wise: model sections present with a
    // placeholder transition, Ubiquitous EARS statement — i.e. linting a
    // valid spec containing the guidance comments yields no findings
    // (task 4.2; the linter never parses prose/comments)
    let lint = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(lint.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["data"]["issues"].as_array().unwrap().len(), 0);
    assert_eq!(lint.status.code(), Some(0));
}

#[test]
fn new_scaffold_shows_file_qualified_ref_examples() {
    // gh#3: derives_from / guard cells take file-qualified wiki-links,
    // not bare ids — the scaffold's guidance comments must show the
    // concrete self-file form rendered from the new spec's own id.
    let dir = tempfile::tempdir().unwrap();
    let out = spk()
        .args(["new", "demo.thing", "--file", "demo-thing.md"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let content = std::fs::read_to_string(dir.path().join("demo-thing.md")).unwrap();
    assert!(
        content.contains("[[demo.thing."),
        "scaffold must render a file-qualified example ref from the spec's own id"
    );
    assert!(
        content.to_lowercase().contains("bare"),
        "scaffold guidance must state that bare ids / bare text do not resolve"
    );
    // Ro5 CORR-002: the placeholder guard cell must not itself demonstrate
    // the bare-id form the guidance just forbid.
    assert!(
        !content.contains("by [[id]]"),
        "scaffold must not show a bare [[id]] ref anywhere"
    );
}

#[test]
fn help_does_not_leak_flattened_struct_docs_and_labels_orchestrate() {
    // gh#6.3: the genesis CliFormat doc comment leaked above Usage.
    // gh#6.4: orchestrate was a stub — the subcommand list said so.
    // specodelic-8kk shipped it: the label must be gone, and the help
    // line must name the pipeline it drives.
    let out = spk().args(["--help"]).output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(
        !text.contains("Clap-derivable"),
        "help must not leak internal struct docs"
    );
    assert!(text.contains("Specodelic — lint, compile"));
    let orch = text
        .lines()
        .find(|l| l.trim_start().starts_with("orchestrate"))
        .expect("orchestrate listed in help");
    assert!(
        !orch.contains("not yet implemented"),
        "orchestrate is implemented — no stub label: {orch}"
    );
    assert!(orch.contains("pipeline"), "help names the pipeline: {orch}");
}
