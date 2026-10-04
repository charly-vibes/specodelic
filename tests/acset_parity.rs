//! Parity harness for the acset refactoring epic (beads specodelic-84i).
//!
//! Phase 1 (tasks 1.1): a byte-comparison oracle over checked-in fixture
//! corpora that pin graph::build's current behaviour — the Reference Typing
//! violation classes (specs/specodelic.md's Reference Typing table),
//! the typing-allowed edges, and the dangling shapes (generic vs the
//! interface-shaped consumption message, specodelic-2q8). Each fixture is a
//! minimal one-or-two-spec file authored to trigger exactly one named shape.
//!
//! Phase 2 (task 1.2): a parity property over `arbitrary_corpus()` proptest
//! cases — `edges(from_specs(c)) == graph::build(c).edges` — pinning that the
//! extracted edge set is invariant under the acset constructor. RED first: it
//! references a `from_specs` constructor that does not exist yet (compile
//! error is an acceptable observed-red for a missing constructor).

use specodelic::graph;
use specodelic::spec::Spec;

/// Parse the given fixture directory recursively for `.md` files, sorted,
/// skipping dot/target/node_modules dirs (mirrors main.rs's collect_specs).
fn collect_fixture(dir: &str) -> Vec<Spec> {
    fn walk(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                walk(&p, files);
            } else if p.extension().and_then(|e| e.to_str()) == Some("md") {
                files.push(p);
            }
        }
    }
    let mut files = Vec::new();
    walk(std::path::Path::new(dir), &mut files);
    files
        .iter()
        .map(|p| Spec::from_file(p).expect("fixture parses"))
        .collect()
}

/// Serialize a GraphReport to a canonical byte form — the snapshot oracle.
/// Deterministic: BTreeMap-driven field order, no hash-map iteration.
fn oracle_bytes(r: &graph::GraphReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("files: {}\n", r.files));
    out.push_str(&format!("nodes: {}\n", r.nodes));
    for e in &r.edges {
        out.push_str(&format!("edge: {} -[{}]-> {}\n", e.from, e.kind, e.to));
    }
    for d in &r.dangling {
        out.push_str(&format!("dangling: {d}\n"));
    }
    for v in &r.violations {
        out.push_str(&format!(
            "violation: {} -[{}]-> {}: {}\n",
            v.from, v.edge_kind, v.to, v.reason
        ));
    }
    for c in &r.supersedes_cycles {
        out.push_str(&format!("supersedes_cycle: {c}\n"));
    }
    for b in &r.external_boundaries {
        out.push_str(&format!("external_boundary: {b}\n"));
    }
    for (k, v) in &r.fan_in {
        out.push_str(&format!("fan_in: {k} = {v}\n"));
    }
    for (k, v) in &r.fan_out {
        out.push_str(&format!("fan_out: {k} = {v}\n"));
    }
    out
}

const TYPING_ALLOWED: &str = "tests/fixtures/typing_allowed";
const TYPING_VIOLATIONS: &str = "tests/fixtures/typing_violations";
const DANGLING: &str = "tests/fixtures/dangling";
const SMALL: &str = "tests/fixtures/small";
const ZERO_FILE: &str = "tests/fixtures/zero_file";
const SINGLE_INTENT: &str = "tests/fixtures/single_intent";

fn assert_oracle(dir: &str, expected: &str) {
    let specs = collect_fixture(dir);
    let bytes = oracle_bytes(&graph::build(&specs));
    assert!(
        bytes == expected,
        "snapshot drift for {dir}:\n--- got ---\n{bytes}\n--- expected ---\n{expected}"
    );
}

/// The zero-file directory (a corpus with zero `.md` files) and the bare
/// single-intent corpus (frontmatter only, no layers): both must be clean —
/// zero specs parsed / one intent parsed, no violations, dangling, cycles,
/// or boundaries. The zero-file corpus is a degenerate input, not a fixture
/// that triggers a shape — the graph over an empty corpus must be empty.
#[test]
fn zero_file_and_single_intent_corpora_are_clean() {
    assert!(
        collect_fixture(ZERO_FILE).is_empty(),
        "zero-file corpus must have no .md files"
    );
    let single = collect_fixture(SINGLE_INTENT);
    assert_eq!(single.len(), 1);
    let r = graph::build(&single);
    assert!(
        r.violations.is_empty(),
        "single-intent corpus must trigger no violation"
    );
    assert!(
        r.dangling.is_empty(),
        "single-intent corpus must have no dangling links"
    );
    assert!(r.supersedes_cycles.is_empty());
    assert!(r.external_boundaries.is_empty());
}

/// A corpus that triggers NO violation, NO dangling, NO cycle — the clean
/// baseline against which every typed shape is compared.
#[test]
fn small_clean_corpus_is_clean() {
    let specs = collect_fixture(SMALL);
    assert_eq!(specs.len(), 1);
    let r = graph::build(&specs);
    assert!(
        r.violations.is_empty(),
        "small corpus must trigger no violation"
    );
    assert!(
        r.dangling.is_empty(),
        "small corpus must have no dangling links"
    );
    assert!(
        r.supersedes_cycles.is_empty(),
        "small corpus must have no supersedes cycle"
    );
    assert!(
        r.external_boundaries.is_empty(),
        "small corpus must be no external boundary"
    );
}

/// Today's typing-allowed edges (from a typing-allowed fixture), pinned by
/// byte comparison — drift here means graph::build's edge extraction changed.
#[test]
fn typing_allowed_edges_match_oracle() {
    assert_oracle(TYPING_ALLOWED, include_str!("snapshots/typing_allowed.txt"));
}

/// Today's typing-forbidden edges (from a typing-violations fixture), pinned
/// by byte comparison — every Reference Typing violation class specs/
/// specodelic.md names is represented.
#[test]
fn typing_violations_match_oracle() {
    assert_oracle(
        TYPING_VIOLATIONS,
        include_str!("snapshots/typing_violations.txt"),
    );
}

/// Today's dangling shapes (from a dangling fixture), pinned by byte
/// comparison — the generic shape and the interface-shaped consumption
/// message are both represented.
#[test]
fn dangling_shapes_match_oracle() {
    assert_oracle(DANGLING, include_str!("snapshots/dangling.txt"));
}

/// A corpus that triggers EVERY Reference Typing violation class named in
/// specs/specodelic.md's Reference Typing table, plus a supersedes cycle —
/// authors assert on named shapes to confirm the corpus actually triggers
/// them (the snapshot pins today's byte form; the assertions pin the corpus).
#[test]
fn typing_violations_fixture_covers_named_shapes() {
    let specs = collect_fixture(TYPING_VIOLATIONS);
    let r = graph::build(&specs);
    let reasons: Vec<&str> = r.violations.iter().map(|v| v.reason.as_str()).collect();
    // Every violation class, named.
    assert!(
        reasons
            .iter()
            .any(|x| x.contains("traces_to must resolve to an Intent"))
    );
    assert!(
        reasons
            .iter()
            .any(|x| x.contains("derives_from must resolve to a Constraint"))
    );
    assert!(
        reasons
            .iter()
            .any(|x| x.contains("guard must resolve to an invariant Constraint"))
    );
    assert!(
        reasons
            .iter()
            .any(|x| x.contains("emits must resolve to an effect Constraint"))
    );
    assert!(
        reasons
            .iter()
            .any(|x| x.contains("satisfies must resolve to an extension_point"))
    );
    assert!(
        reasons
            .iter()
            .any(|x| x.contains("observes must resolve to an effect Constraint"))
    );
    assert!(
        reasons
            .iter()
            .any(|x| x.contains("supersedes must target the same kind"))
    );
    // The supersedes cycle is represented.
    assert!(
        !r.supersedes_cycles.is_empty(),
        "supersedes cycle must be represented"
    );
    // And the violations are anchored to their named from-shapes: the
    // real builder carries the typed-reference column in the edge KIND
    // (the `-[constraints.traces_to]->` slot), never in the from anchor
    // (which is the bare row id) — assert the kind slot instead.
    let vkinds: Vec<&str> = r.violations.iter().map(|v| v.edge_kind.as_str()).collect();
    assert!(vkinds.iter().any(|x| x.ends_with("constraints.traces_to")));
    assert!(
        vkinds
            .iter()
            .any(|x| x.ends_with("properties.derives_from"))
    );
    assert!(vkinds.iter().any(|x| x.ends_with("transitions.guard")));
    assert!(vkinds.iter().any(|x| x.ends_with("states.emits")));
}

/// A corpus that triggers both dangling shapes — the generic shape (non-
/// consumption column) and the interface-shaped consumption message (the
/// `satisfies`/`observes` columns, specodelic-2q8) — plus the transitions
/// from/to dangling shapes. The snapshot pins today's byte form; the
/// assertions pin the corpus.
#[test]
fn dangling_fixture_covers_named_shapes() {
    let specs = collect_fixture(DANGLING);
    let r = graph::build(&specs);
    // Generic shape: `file_id → [[unresolved_target]]` — no column, no row.
    assert!(r.dangling.iter().any(|d| d.starts_with("c1 → [[")));
    assert!(r.dangling.iter().any(|d| d.contains("satisfies")));
    // Interface-shaped consumption message (specodelic-2q8): names BOTH
    // remediations — publish or fix.
    assert!(
        r.dangling
            .iter()
            .any(|d| d.contains("publish it in the producer's file or fix the id"))
    );
    // Transitions from/to dangling shapes.
    assert!(r.dangling.iter().any(|d| d.contains("transitions.from")));
    assert!(r.dangling.iter().any(|d| d.contains("transitions.to")));
}

/// A corpus of allowed edges across every layer — the types that resolve and
/// are recorded as edges. The snapshot pins today's byte form; the assertions
/// pin the corpus (every named edge kind is represented).
#[test]
fn typing_allowed_fixture_covers_named_edges() {
    let specs = collect_fixture(TYPING_ALLOWED);
    let r = graph::build(&specs);
    assert!(
        r.violations.is_empty(),
        "typing-allowed corpus must trigger no violation"
    );
    assert!(
        r.dangling.is_empty(),
        "typing-allowed corpus must have no dangling links"
    );
    let kinds: Vec<&str> = r.edges.iter().map(|e| e.kind.as_str()).collect();
    assert!(kinds.iter().any(|x| x.ends_with("traces_to")));
    assert!(kinds.iter().any(|x| x.ends_with("derives_from")));
    assert!(kinds.iter().any(|x| x.ends_with("guard")));
    assert!(kinds.iter().any(|x| x.ends_with("emits")));
    assert!(kinds.iter().any(|x| x.ends_with("satisfies")));
    assert!(kinds.iter().any(|x| x.ends_with("observes")));
    assert!(kinds.iter().any(|x| x.ends_with("supersedes")));
}

// ---------------------------------------------------------------------------
// Phase 2: parity property over arbitrary corpora (task 1.2 — RED first).
// ---------------------------------------------------------------------------

/// A hand-built in-memory corpus: intent file + constraints + properties +
/// states + transitions, with typed reference cells. Only the intent id and
/// the links matter for the parity property — the table rows are the layers
/// the edges point at.
fn in_memory_corpus() -> Vec<Spec> {
    use specodelic::spec::Spec as _;
    let _ = 0; // silence unused
    todo!("in-memory corpus: intent file + rows + links (task 1.2 RED)")
}

/// `from_specs` does not exist yet — the RED constructor the parity property
/// references. Its contract: build the acset from a corpus of parsed specs,
/// such that `edges(from_specs(c)) == graph::build(c).edges` for every
/// corpus.
fn from_specs(specs: &[Spec]) -> specodelic::acset::Acset {
    let _ = specs;
    todo!("from_specs: acset constructor (task 1.2 RED)")
}

/// Parity property (task 1.2): the acset's edge set is invariant under the
/// acset constructor — `edges(from_specs(c)) == graph::build(c).edges` for
/// every hand-built corpus. RED: from_specs is todo!() and the acset module
/// does not exist yet, so this test fails to compile / todo!-panics.
#[test]
fn parity_property_edges_from_specs_matches_graph_build() {
    let specs = in_memory_corpus();
    let acset_edges = from_specs(&specs).edges();
    let graph_edges = graph::build(&specs).edges.clone();
    assert_eq!(acset_edges, graph_edges);
}

// ---------------------------------------------------------------------------
// Task 2.4: the lint-time drift gate (`schema_matches_typing_table`) —
// the format doc's Reference Typing table vs the canonical Schema, row for
// row. RED first: the gate API does not exist yet (compile-fail red, the
// 1.2 convention).
// ---------------------------------------------------------------------------

/// The real format doc, parsed.
fn format_doc() -> Spec {
    Spec::from_file(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("specs/specodelic.md"),
    )
    .expect("the format doc parses")
}

/// GREEN state: the real doc's Reference Typing table equals `canonical()`
/// row for row — the gate the lint pass runs.
#[test]
fn real_doc_typing_table_matches_canonical_schema() {
    let doc = format_doc();
    let rows = specodelic::acset::doc::parse_typing_table(&doc.reference_typing_body)
        .expect("the format doc's Reference Typing table parses");
    specodelic::acset::doc::matches_typing_table(&specodelic::acset::schema::canonical(), &rows)
        .expect("the format doc and the canonical Schema must agree row for row");
}

/// The unit scenario (`schema_missing_one_row_of_the_typing_table`): a
/// Schema missing one row of the doc's table fails the comparison, naming
/// the missing row.
#[test]
fn schema_missing_one_row_of_the_typing_table() {
    let doc = format_doc();
    let rows = specodelic::acset::doc::parse_typing_table(&doc.reference_typing_body)
        .expect("the format doc's Reference Typing table parses");
    let mut schema = specodelic::acset::schema::canonical();
    schema.morphisms.retain(|m| m.column != "traces_to");
    let drift =
        specodelic::acset::doc::matches_typing_table(&schema, &rows)
            .expect_err("a schema missing a doc row must fail the comparison");
    assert!(
        drift.iter().any(|d| d.contains("traces_to")),
        "the drift report must name the missing row: {drift:?}"
    );
}

/// Lint integration: a corpus carrying the format doc with one typing row
/// removed reports exactly one `linter.schema_matches_typing_table` issue
/// naming the vanished field.
#[test]
fn lint_reports_drift_when_the_doc_loses_a_row() {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("specs/specodelic.md"),
    )
    .expect("the format doc reads");
    let doctored: String = text
        .lines()
        .filter(|l| !l.starts_with("| `traces_to`"))
        .map(|l| format!("{l}\n"))
        .collect();
    let spec = specodelic::spec::parse_str(&doctored).expect("doctored doc parses");
    let report = specodelic::lint::lint_corpus(&[spec]);
    let findings: Vec<_> = report
        .issues
        .iter()
        .filter(|i| i.rule_id == "linter.schema_matches_typing_table")
        .collect();
    assert_eq!(
        findings.len(),
        1,
        "exactly one drift finding: {:#?}",
        report.issues
    );
    assert!(
        findings[0].message.contains("traces_to"),
        "the finding must name the vanished field: {}",
        findings[0].message
    );
}

/// Scope: a corpus that does not carry the format doc is out of the gate's
/// scope — no-op, never fabricated expected rows.
#[test]
fn lint_without_the_format_doc_is_a_no_op() {
    let spec = specodelic::spec::parse_str(
        "---\nid: other.thing\nkind: intent\nstatement: \"THE thing SHALL hold\"\n---\n\
         \n## Constraints\n\
         \n| id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | c1 | invariant | `x` | [[other.thing]] |\n\
         \n## Model\n\
         \n### States\n\
         \n- `s1`\n\
         \n### Transitions\n\
         \n| id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | t | s1 | s1 | [[other.thing.c1]] |\n",
    )
    .expect("fixture parses");
    let report = specodelic::lint::lint_corpus(&[spec]);
    assert!(
        report
            .issues
            .iter()
            .all(|i| i.rule_id != "linter.schema_matches_typing_table"),
        "the gate must not fire without the format doc: {:#?}",
        report.issues
    );
}
