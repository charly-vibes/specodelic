//! `emission_tests` — extracted from src/audit.rs to hold the file_lines
//! ratchet; zero behavior change (specodelic-3yt9 coverage lives here).

use super::fixtures::{shell, spec};
use super::*;

/// The oracle-side fixture spec: declares `confirm` AND `reopen`
/// (the integration fixture's shape at tests/audit_oracle_rows.rs;
/// no gate runs here, so no Properties table is needed).
const SPEC: &str = r#"---
id: demo.audit
kind: intent
statement: "THE system SHALL keep the ledger consistent"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.audit]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.audit.locked]] |
| reopen | confirmed | held | [[demo.audit.locked]] |
"#;

/// The three oracle corpus lines spread the full class range:
/// permitted, uncovered (underspecified open-world), and a positive
/// contradiction.
const CORPUS_LINES: &[&str] = &[
    r#"{"id": "confirm-happy", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"status": "confirmed"}}]}"#,
    r#"{"id": "undo-undeclared", "setup": {"state": "held"}, "trace": [{"action": "undo"}]}"#,
    r#"{"id": "confirm-from-wrong-state", "setup": {"state": "confirmed"}, "trace": [{"action": "confirm"}]}"#,
];

/// The oracle report built over the fixture corpus.
fn oracle_report(corpus_lines: &[&str]) -> AuditReport {
    let spec = spec(SPEC);
    let corpus_bytes = format!("{}\n", corpus_lines.join("\n"));
    let corpus = conform::parse_corpus(&corpus_bytes).expect("fixture corpus parses");
    build_report(&spec, corpus_bytes.as_bytes(), &corpus, false, false)
}

fn pack_check(pack: &str, subject: &str) -> PackCheck {
    PackCheck {
        pack_id: pack.to_string(),
        subject_id: subject.to_string(),
        declared_rules: vec![format!("{pack}.t")],
        violations: vec![],
    }
}

/// The unified emission surface's core contract across BOTH row
/// kinds: every emitted row carries the BOTH-layer payload — the
/// conform verdict AND the class that is exactly its
/// [`AuditClass::of`] layer — with the row key verbatim (oracle ids
/// straight from the corpus, pack keys `pack:<pack_id>:<subject_id>`).
#[test]
fn emission_is_both_layered_across_row_kinds() {
    let report = oracle_report(CORPUS_LINES);
    let expected: &[(&str, Verdict, AuditClass)] = &[
        (
            "confirm-from-wrong-state",
            Verdict::Forbidden,
            AuditClass::Contradicted,
        ),
        ("confirm-happy", Verdict::Permitted, AuditClass::Current),
        (
            "undo-undeclared",
            Verdict::Underspecified,
            AuditClass::Unchecked,
        ),
    ];
    assert_eq!(
        report.records.len(),
        expected.len(),
        "one row per corpus line"
    );
    for (row, (id, verdict, class)) in report.records.iter().zip(expected) {
        assert_eq!(&row.scenario_id, id, "the oracle row key is the corpus id");
        assert_eq!(&row.conform_verdict, verdict, "the verdict rides through");
        assert_eq!(&row.class, class, "the class is the verdict's audit layer");
        assert_eq!(
            AuditClass::of(row.conform_verdict),
            row.class,
            "the both-layer invariant holds for oracle row {id}"
        );
    }
    // The pack branches emit through the same contract: the empty
    // checked-set rides `agree` (its layer IS unchecked), the
    // violation rides `forbidden` (its layer IS contradicted).
    let empty = pack_check_row(&pack_check("packs.a", "spec_x"));
    assert_eq!(empty.scenario_id, "pack:packs.a:spec_x");
    assert_eq!(empty.conform_verdict, Verdict::Agree);
    assert_eq!(AuditClass::of(empty.conform_verdict), empty.class);
    assert_eq!(empty.class, AuditClass::Unchecked);
    let mut violated = pack_check("packs.a", "spec_x");
    violated.violations = vec![PackViolation {
        rule_id: "packs.a.v".to_string(),
        message: "boom".to_string(),
    }];
    let violated = pack_check_row(&violated);
    assert_eq!(violated.scenario_id, "pack:packs.a:spec_x");
    assert_eq!(violated.conform_verdict, Verdict::Forbidden);
    assert_eq!(AuditClass::of(violated.conform_verdict), violated.class);
    assert_eq!(violated.class, AuditClass::Contradicted);
}

/// The two count-map compute sites agree: recomputing
/// [`fold_rows`]' maps from scratch over [`build_report`]'s own
/// records reproduces exactly the maps [`build_report`] produced —
/// seeded garbage counts never survive — and pack rows fold onto
/// those counts additively.
#[test]
fn count_maps_agree_between_the_compute_sites() {
    let built = oracle_report(CORPUS_LINES);
    let mut reseeded = shell();
    reseeded.records = built.records.clone();
    // deliberately stale maps: prior-diff leftovers that must not
    // survive a from-scratch recompute
    reseeded.class_counts.insert("stale".to_string(), 99);
    reseeded.verdict_counts.insert("unknown".to_string(), 99);
    let recomputed = fold_rows(reseeded, vec![]);
    assert_eq!(recomputed.class_counts, built.class_counts);
    assert_eq!(recomputed.verdict_counts, built.verdict_counts);
    // pack rows fold onto the recomputed counts additively: the
    // pack's unchecked row joins the oracle's unchecked row
    let merged = fold_rows(
        recomputed,
        pack_check_rows(&[pack_check("packs.a", "spec_x")]),
    );
    assert_eq!(merged.class_counts.get("unchecked"), Some(&2));
    assert_eq!(merged.class_counts.get("contradicted"), Some(&1));
    assert_eq!(merged.verdict_counts.get("agree"), Some(&1));
    assert_eq!(merged.verdict_counts.get("underspecified"), Some(&1));
}

/// Row order is key-sorted across BOTH paths: [`build_report`]
/// sorts its walk output even when the corpus is fed out of id
/// order, and [`fold_rows`] interleaves oracle and pack rows
/// into one key-sorted sequence across kinds.
#[test]
fn row_order_is_key_sorted_across_kinds_in_both_paths() {
    let shuffled: Vec<&str> = CORPUS_LINES.iter().rev().copied().collect();
    let built = oracle_report(&shuffled);
    let oracle_ids: Vec<&str> = built
        .records
        .iter()
        .map(|r| r.scenario_id.as_str())
        .collect();
    let mut sorted = oracle_ids.clone();
    sorted.sort();
    assert_eq!(oracle_ids, sorted, "oracle rows come out key-sorted");
    let merged = fold_rows(
        built,
        pack_check_rows(&[
            pack_check("packs.m", "spec_z"),
            pack_check("packs.a", "spec_b"),
        ]),
    );
    let ids: Vec<&str> = merged
        .records
        .iter()
        .map(|r| r.scenario_id.as_str())
        .collect();
    let mut expected = ids.clone();
    expected.sort();
    assert_eq!(ids, expected, "the merged rows interleave in key order");
    assert_eq!(
        ids.iter().filter(|id| id.starts_with("pack:")).count(),
        2,
        "both pack rows ride in the merged order"
    );
}

/// The cross-kind row-key contract: an oracle corpus id is arbitrary
/// user input, so one spelled exactly like a bind row key
/// (`bind:<owner-id>`) must never leave TWO merged rows sharing a
/// `scenario_id`. The prior diff keys rows by scenario id and `find`
/// picks the first match, so a duplicate silently drops a row from
/// the comparison; the fix must either emit a deterministic
/// disambiguated bind key or refuse labeled naming the collision —
/// never merge a duplicate. Advisory-only: both rows still ride the
/// report and the exit contract is untouched.
#[test]
fn oracle_row_id_colliding_with_a_bind_row_key_never_duplicates_scenario_ids() {
    // `locked`'s fragment (`status != "open"`) cites no declared
    // model element, so this spec contributes the vacuity row keyed
    // exactly `bind:locked` — the corpus id used below.
    let spec = spec(SPEC);
    let corpus_lines = [
        r#"{"id": "bind:locked", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"status": "confirmed"}}]}"#,
    ];
    let merged = fold_rows(oracle_report(&corpus_lines), bind_rows(&spec));
    let ids: Vec<&str> = merged
        .records
        .iter()
        .map(|r| r.scenario_id.as_str())
        .collect();
    // Rename direction (the design depends on it): the oracle row
    // keeps its corpus id verbatim and the FOLDED bind row yields
    // with the first free suffix — never the other way around, and
    // no other suffix scheme.
    assert_eq!(
        ids,
        vec!["bind:locked", "bind:locked~2"],
        "the oracle row keeps `bind:locked` and the folded bind row yields with `~2`"
    );
    let Some(oracle_row) = merged
        .records
        .iter()
        .find(|r| r.scenario_id == "bind:locked")
    else {
        panic!("the corpus id stays on a row");
    };
    assert_eq!(
        oracle_row.class,
        AuditClass::Current,
        "the row keeping the corpus id is the oracle row, not the renamed bind row"
    );
    let Some(renamed_bind_row) = merged
        .records
        .iter()
        .find(|r| r.scenario_id == "bind:locked~2")
    else {
        panic!("the folded bind row is present under the suffixed key");
    };
    assert_eq!(
        renamed_bind_row.class,
        AuditClass::BindsNothing,
        "the suffixed key belongs to the folded binds_nothing row"
    );
    let mut seen = BTreeSet::new();
    let duplicates: Vec<&str> = ids.iter().copied().filter(|id| !seen.insert(*id)).collect();
    assert!(
        duplicates.is_empty(),
        "two audit rows share a scenario_id ({duplicates:?}) — an oracle id spelled like a bind row key must be disambiguated or refused labeled, never merged as a duplicate (the diff lookup would silently pick the first); keys: {ids:?}"
    );
    // A silent drop is not a fix: both the oracle row and the
    // vacuity advisory row survive the merge.
    assert_eq!(
        merged.records.len(),
        2,
        "the oracle row and the bind row both ride the report: {ids:?}"
    );
    assert!(
        merged
            .records
            .iter()
            .any(|r| r.class == AuditClass::BindsNothing),
        "the binds_nothing advisory row still surfaces: {merged:?}"
    );
    // Deterministic across runs: repeated assembly over identical
    // inputs yields the same keys (a counter or random suffix is
    // not a fix).
    let again = fold_rows(oracle_report(&corpus_lines), bind_rows(&spec));
    let again_ids: Vec<&str> = again
        .records
        .iter()
        .map(|r| r.scenario_id.as_str())
        .collect();
    assert_eq!(
        ids, again_ids,
        "the merged keys are deterministic across runs"
    );
}

/// The pack half of the same rule: a folded pack row keyed
/// `pack:<pack_id>:<subject_id>` collides with an oracle corpus id
/// spelled exactly like it, so the pack row yields the first free
/// `~<n>` suffix (`~2`) while the oracle row keeps its id verbatim —
/// no row dropped or merged, both classes preserved, counts intact,
/// and the keys deterministic across repeated folds.
#[test]
fn pack_row_key_collision_disambiguates_the_folded_row() {
    // The oracle row's corpus id IS the folded pack row's key.
    let corpus_lines = [
        r#"{"id": "pack:packs.a:spec_x", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"status": "confirmed"}}]}"#,
    ];
    let mut violated = pack_check("packs.a", "spec_x");
    violated.violations = vec![PackViolation {
        rule_id: "packs.a.v".to_string(),
        message: "boom".to_string(),
    }];
    let merged = fold_rows(
        oracle_report(&corpus_lines),
        pack_check_rows(&[violated.clone()]),
    );
    let ids: Vec<&str> = merged
        .records
        .iter()
        .map(|r| r.scenario_id.as_str())
        .collect();
    // Rename direction: the oracle row keeps the collided id and the
    // FOLDED pack row takes `~2`, never the other way around.
    assert_eq!(
        ids,
        vec!["pack:packs.a:spec_x", "pack:packs.a:spec_x~2"],
        "the oracle row keeps `pack:packs.a:spec_x` and the folded pack row yields with `~2`"
    );
    let Some(oracle_row) = merged
        .records
        .iter()
        .find(|r| r.scenario_id == "pack:packs.a:spec_x")
    else {
        panic!("the corpus id stays on a row");
    };
    assert_eq!(
        oracle_row.class,
        AuditClass::Current,
        "the row keeping the corpus id is the oracle row, not the renamed pack row"
    );
    let Some(renamed_pack_row) = merged
        .records
        .iter()
        .find(|r| r.scenario_id == "pack:packs.a:spec_x~2")
    else {
        panic!("the folded pack row is present under the suffixed key");
    };
    assert_eq!(
        renamed_pack_row.class,
        AuditClass::Contradicted,
        "the suffixed key belongs to the folded pack row, its class preserved"
    );
    assert_eq!(
        renamed_pack_row.conform_verdict,
        Verdict::Forbidden,
        "the folded pack row keeps its verdict under the renamed key"
    );
    let mut seen = BTreeSet::new();
    let duplicates: Vec<&str> = ids.iter().copied().filter(|id| !seen.insert(*id)).collect();
    assert!(
        duplicates.is_empty(),
        "two audit rows share a scenario_id ({duplicates:?}) — a pack row key colliding with an oracle id must be suffixed, never merged as a duplicate; keys: {ids:?}"
    );
    // A silent drop is not a fix: both rows survive and are counted.
    assert_eq!(
        merged.records.len(),
        2,
        "the oracle row and the folded pack row both ride the report: {ids:?}"
    );
    assert_eq!(merged.class_counts.get("current"), Some(&1));
    assert_eq!(merged.class_counts.get("contradicted"), Some(&1));
    // Deterministic across runs: repeated assembly over identical
    // inputs yields the same keys (a counter or random suffix is
    // not a fix).
    let again = fold_rows(oracle_report(&corpus_lines), pack_check_rows(&[violated]));
    let again_ids: Vec<&str> = again
        .records
        .iter()
        .map(|r| r.scenario_id.as_str())
        .collect();
    assert_eq!(
        ids, again_ids,
        "the merged keys are deterministic across runs"
    );
}
