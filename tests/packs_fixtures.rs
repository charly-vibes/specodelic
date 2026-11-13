//! The packs spec's property fixtures — the registry the 16 property
//! rows of `openspec/specs/packs/spec.md` materialize through.
//!
//! Purpose: the scoped migration of the packs corpus file
//! (specodelic-lf3) makes every property row name a fixture; this file
//! proves the registry actually carries those inputs: every generator
//! name the corpus's property table names resolves to a pack fixture,
//! every pack fixture is a real `kind: profile` file the real parser
//! accepts, and the corpus harness really produces the behaviors the
//! migrated predicates assert (discovery, activation, orphan, skew,
//! narrowing, fiber kinds).
//!
//! Responsibilities: assert the corpus↔registry correspondence, assert
//! each pack fixture parses as a profile file with the four layers, and
//! assert the corpus harness's behaviors — the ones the migrated
//! predicates read.
//!
//! Rationale: the corpus's property cells are executable only if the
//! generator names they carry materialize real inputs — a name with no
//! fixture would silently yield the generator's own name (a placeholder
//! input), which is the failure mode the migration exists to remove.

use specodelic::fixtures;
use specodelic::fixtures::packs::{
    consumer_spec, fiber_kind_consumer, pack_file, pack_file_in_state, pack_file_with_kinds,
};
use specodelic::spec::parse_str;

#[path = "common/mod.rs"]
mod common;
use common::UnwrapPanic;

/// The corpus file whose property rows the registry serves.
const PACKS_SPEC: &str = "openspec/specs/packs/spec.md";

/// The generator names in a generator cell — the same `name(` shape the
/// compiler's `generator_names` extracts (a test may restate it; the
/// compiler's own is private).
fn generator_names(cell: &str) -> Vec<String> {
    let mut names = vec![];
    let mut rest = cell;
    while let Some(open) = rest.find('(') {
        let head = &rest[..open];
        let name: String = head
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect::<Vec<char>>()
            .into_iter()
            .rev()
            .collect();
        if !name.is_empty()
            && name.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
            && !names.contains(&name)
        {
            names.push(name);
        }
        rest = &rest[open + 1..];
    }
    names
}

#[test]
fn every_property_row_generator_resolves_to_a_pack_fixture() {
    // The corpus↔registry correspondence: every generator name the
    // packs spec's property table names must materialize a real pack
    // fixture — otherwise the row's compiled strategy yields the
    // generator's own name (a placeholder input).
    let text = std::fs::read_to_string(PACKS_SPEC).unwrap_p();
    let spec = parse_str(&text).unwrap_p();
    assert_eq!(spec.properties.len(), 16, "the packs file's 16 rows");
    let mut missing: Vec<String> = vec![];
    for row in &spec.properties {
        let cell = row.cells.get("generator").cloned().unwrap_or_default();
        for name in generator_names(&cell) {
            if fixtures::packs::lookup(&name).is_none() && !missing.contains(&name) {
                missing.push(name);
            }
        }
    }
    assert!(
        missing.is_empty(),
        "every generator name must resolve to a registered pack fixture; missing: {missing:?}"
    );
}

#[test]
fn every_pack_fixture_is_a_real_file_with_four_layers() {
    // The registry's own contract: each pack fixture's text is a real
    // file the real parser accepts, carrying the four standard layers.
    // The pack files are `kind: profile`; the consumer fixtures are the
    // `kind: intent` files that use (or fail to use) a pack.
    for name in fixtures::packs::names() {
        let text = fixtures::materialize(name).unwrap_p();
        let spec = parse_str(&text).unwrap_p();
        assert!(
            spec.intent.kind == "profile" || spec.intent.kind == "intent",
            "the fixture `{name}` carries the kind `{}`",
            spec.intent.kind
        );
        assert!(!spec.constraints.is_empty(), "the fixture `{name}`");
        assert!(!spec.states.is_empty(), "the fixture `{name}`");
        assert!(!spec.transitions.is_empty(), "the fixture `{name}`");
        assert!(!spec.properties.is_empty(), "the fixture `{name}`");
    }
}

#[test]
fn the_corpus_harness_discovers_packs_in_file_order() {
    // Discovery: the corpus's real pack files are found by the real
    // workspace scan, and the result does not depend on the order the
    // files are linted in.
    let alpha = pack_file("alpha.pack", 20);
    let beta = pack_file("beta.pack", 20);
    let corpus = fixtures::packs::Corpus::new(&[
        ("alpha.pack.md", alpha.as_str()),
        ("beta.pack.md", beta.as_str()),
    ]);
    let ids: Vec<String> = corpus.discovered().iter().map(|p| p.id.clone()).collect();
    assert_eq!(ids, vec!["alpha.pack".to_string(), "beta.pack".to_string()]);
    let mut reversed = corpus.specs().to_vec();
    reversed.reverse();
    let reversed_ids: Vec<String> = specodelic::packs::discover(&reversed)
        .iter()
        .map(|p| p.id.clone())
        .collect();
    assert_eq!(reversed_ids, ids, "discovery must be order-independent");
}

#[test]
fn the_corpus_harness_activates_a_pack_for_a_vocabulary_consumer() {
    // Activation: a consumer that uses the pack's vocabulary activates
    // it — the advisory names the pack and the empty checked-set.
    let pack = pack_file("data.pack", 20);
    let consumer = consumer_spec("consumer", "", "data.pack.tolerance applies here");
    let corpus = fixtures::packs::Corpus::new(&[
        ("data.pack.md", pack.as_str()),
        ("consumer.md", consumer.as_str()),
    ]);
    let report = corpus.lint();
    assert!(report.issues.is_empty(), "{:?}", report.issues);
    let warnings = corpus.warning_messages();
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("pack `data.pack` activated") && w.contains("empty checked-set")),
        "no activation advisory: {warnings:?}"
    );
}

#[test]
fn the_corpus_harness_reports_a_labeled_orphan() {
    // Orphan: a `uses` edge to a pack no workspace declares is a
    // labeled failure naming the candidate pack and both remediations.
    let consumer = consumer_spec("consumer", "[[ghost.pack]]", "plain prose only");
    let corpus = fixtures::packs::Corpus::new(&[("consumer.md", consumer.as_str())]);
    let issues = corpus.issue_messages();
    let orphan = issues
        .iter()
        .find(|i| i.contains("orphan_vocabulary"))
        .expect("the orphan finding fires");
    assert!(orphan.contains("ghost.pack"), "{orphan}");
    assert!(orphan.contains("kind: profile"), "{orphan}");
    assert!(orphan.contains("add/enable"), "{orphan}");
    assert!(orphan.contains("fix the vocabulary"), "{orphan}");
}

#[test]
fn the_corpus_harness_reports_the_skew_advisory_on_the_warnings_channel() {
    // Skew: an older base pin is an advisory (never a failure) naming
    // both revisions.
    let pack = pack_file("data.pack", 10);
    let consumer = consumer_spec("consumer", "[[data.pack]]", "plain prose only");
    let corpus = fixtures::packs::Corpus::new(&[
        ("data.pack.md", pack.as_str()),
        ("consumer.md", consumer.as_str()),
    ]);
    let report = corpus.lint();
    assert!(report.issues.is_empty(), "{:?}", report.issues);
    let skew = report
        .warnings
        .iter()
        .find(|w| w.rule_id == "linter.skew_advisory")
        .expect("the skew advisory fires");
    assert!(skew.message.contains("Revision 10"), "{}", skew.message);
    assert!(skew.message.contains("Revision 20"), "{}", skew.message);
}

#[test]
fn the_corpus_harness_rejects_a_narrowing_pack() {
    // Narrowing: a `## Kinds` row naming a base closed-set member is
    // the append-only rejection.
    let pack = pack_file_with_kinds("data.pack", 20, &["invariant"]);
    let corpus = fixtures::packs::Corpus::new(&[("data.pack.md", pack.as_str())]);
    let issues = corpus.issue_messages();
    let shape = issues
        .iter()
        .find(|i| i.contains("narrowing rejected"))
        .expect("the narrowing rejection fires");
    assert!(shape.contains("invariant"), "{shape}");
    assert!(shape.contains("append_only_packs"), "{shape}");
}

#[test]
fn the_corpus_harness_types_a_fiber_kind_only_while_its_pack_is_active() {
    // Fiber kinds: the base closed-set walkers accept a pack-qualified
    // kind while the declaring pack is active, and the labeled finding
    // fires when it is not.
    let pack = pack_file_with_kinds("data.pack", 20, &["data.dataset"]);
    let consumer = fiber_kind_consumer("consumer", "[[data.pack]]", "data.dataset");
    let with_pack = fixtures::packs::Corpus::new(&[
        ("data.pack.md", pack.as_str()),
        ("consumer.md", consumer.as_str()),
    ]);
    let without_pack = fixtures::packs::Corpus::new(&[("consumer.md", consumer.as_str())]);
    assert!(
        !with_pack
            .issue_messages()
            .iter()
            .any(|i| i.contains("constraint_kind_closed")),
        "{:?}",
        with_pack.issue_messages()
    );
    assert!(
        without_pack
            .issue_messages()
            .iter()
            .any(|i| i.contains("constraint_kind_closed")),
        "{:?}",
        without_pack.issue_messages()
    );
}

#[test]
fn the_corpus_harness_reads_each_lifecycle_state() {
    // Lifecycle: the pack's Model section names where the pack stands.
    let published = pack_file("data.pack", 20);
    let draft = pack_file_in_state("draft.pack", 20, "draft");
    let deprecated = pack_file_in_state("old.pack", 20, "deprecated");
    let corpus = fixtures::packs::Corpus::new(&[
        ("data.pack.md", published.as_str()),
        ("draft.pack.md", draft.as_str()),
        ("old.pack.md", deprecated.as_str()),
    ]);
    let lifecycles: Vec<String> = corpus
        .discovered()
        .iter()
        .map(|p| format!("{}:{}", p.id, p.lifecycle))
        .collect();
    assert_eq!(
        lifecycles,
        vec![
            "data.pack:published".to_string(),
            "draft.pack:draft".to_string(),
            "old.pack:deprecated".to_string(),
        ]
    );
}
