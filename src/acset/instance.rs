//! The typed instance builder (openspec change `add-acset-core`, tasks
//! 3.x): a parsed corpus loaded into a typed instance — interned, dense
//! id-sets per schema object and a partial morphism vector per Reference
//! Typing row — in which an unresolved reference is a representable value
//! (`None`) that is never dropped and never a parse error.
//!
//! Parity context (design.md decision 2): the old `graph::build` path stays
//! authoritative until `adapter_graph_equivalent` holds over the whole
//! corpus, including its tolerated dirty shapes — duplicate ids resolve
//! first-wins (`or_insert`) and surface only as a named collision report,
//! never the hard failure the old path never emitted (Rule-of-5 CORR-001).

use crate::graph::Edge;
use crate::spec::Spec;

/// A link that resolved to no id — stored as a value (`None` in the
/// morphism vector), never dropped, never a parse error
/// (`dangling_is_a_value`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dangling {
    /// The anchor the link was emitted from: `file (intent)` for
    /// frontmatter links, `file.row` for table links.
    pub from: String,
    /// The typed reference column the link sat in.
    pub kind: String,
    /// The unresolved target spelling.
    pub target: String,
}

/// A Reference Typing violation: a link whose resolved target object is
/// outside the morphism's allowed targets — reported, never stored as a
/// morphism value (`forbidden_edge_not_stored`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// The anchor the forbidden edge would have started at.
    pub from: String,
    /// The resolved target the forbidden edge would have pointed at.
    pub to: String,
    /// The typed reference field, e.g. `constraints.traces_to`.
    pub edge_kind: String,
    /// The labeled reason — the Schema's violation prose.
    pub reason: String,
}

/// The typed instance over a parsed corpus: one interned id-set per schema
/// object, one partial morphism vector per Reference Typing row, plus the
/// build-time reports (dangling values, typing violations, duplicate-id
/// collisions).
pub struct Instance {
    _private: (),
}

impl Instance {
    /// Build the typed instance for a parsed corpus. Infallible: every
    /// shape the old path accepts — dangling links, duplicate ids — is a
    /// value here, not a failure.
    pub fn from_specs(_specs: &[Spec]) -> Self {
        todo!("typed instance builder (task 3.2 GREEN)")
    }

    /// Links that resolved AND passed Reference Typing — the stored
    /// morphism values (`no_link_dropped`'s first bucket).
    pub fn stored(&self) -> usize {
        todo!("task 3.2 GREEN")
    }

    /// The unresolved links, stored as values (`dangling_is_a_value`).
    pub fn dangling(&self) -> &[Dangling] {
        todo!("task 3.2 GREEN")
    }

    /// The Reference Typing violations — labeled, never stored as values.
    pub fn violations(&self) -> &[Violation] {
        todo!("task 3.2 GREEN")
    }

    /// The named duplicate-id collision report — surfaced, never a hard
    /// failure the old path did not emit (`duplicate_id_first_wins_parity`).
    pub fn collisions(&self) -> &[String] {
        todo!("task 3.2 GREEN")
    }

    /// The corpus's edge set, derived from the instance — the parity
    /// property compares it, edge for edge, with `graph::build`'s
    /// (`adapter_graph_equivalent`, task 3.3).
    pub fn edges(&self) -> Vec<Edge> {
        todo!("task 3.2 GREEN")
    }

    /// The canonical serialization — byte-identical for the same corpus
    /// regardless of input file order (`rebuild_is_byte_stable`).
    pub fn serialize(&self) -> String {
        todo!("task 3.2 GREEN")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::parse_str;

    /// Parse an inline corpus text (the same parser `Spec::from_file` runs).
    fn parse(text: &str) -> Spec {
        parse_str(text).expect("test corpus parses")
    }

    /// `dangling_is_a_value` (acset-core properties): one link resolving to
    /// no id — the morphism value is None, the build succeeds.
    #[test]
    fn dangling_is_a_value() {
        let c1 = parse(
            r#"---
id: c1
kind: intent
statement: "points at a ghost"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| r | invariant | `true` | [[c1.ghost_row]] |
"#,
        );
        let c2 = parse("---\nid: c2\nkind: intent\nstatement: \"another intent\"\n---\n");
        // The build succeeds — a dangling reference is a value, not an error.
        let inst = Instance::from_specs(&[c1, c2]);
        assert_eq!(inst.dangling().len(), 1, "the ghost link dangles");
        assert!(
            inst.dangling()[0].target.contains("ghost_row"),
            "the dangling value names the unresolved target"
        );
        assert_eq!(inst.stored(), 0, "nothing stored — the only link dangles");
    }

    /// `no_link_dropped` (delta scenario): stored + dangling + violations
    /// == links — every typed-reference link lands in exactly one bucket.
    #[test]
    fn no_link_dropped() {
        // Three typed-reference links, one per bucket: `[[c1.other]]` is a
        // typing violation (traces_to must reach an Intent), `[[c2]]` stores
        // (Constraint → Intent allowed), `[[c1.ghost_row]]` dangles.
        let c1 = parse(
            r#"---
id: c1
kind: intent
statement: "one link of each outcome"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| r | invariant | `true` | [[c1.other]] |
| other | invariant | `true` | [[c1.ghost_row]] |
| src | invariant | `true` | [[c2]] |
"#,
        );
        let c2 = parse("---\nid: c2\nkind: intent\nstatement: \"the intent target\"\n---\n");
        let inst = Instance::from_specs(&[c1, c2]);
        assert_eq!(inst.violations().len(), 1, "the constraint target violates");
        assert_eq!(inst.dangling().len(), 1, "the ghost link dangles");
        assert_eq!(inst.stored(), 1, "the Intent-targeted edge stores");
        assert_eq!(
            inst.stored() + inst.dangling().len() + inst.violations().len(),
            3,
            "stored + dangling + violations == links — none dropped"
        );
    }

    /// `forbidden_edge_not_stored` (delta scenario): a reference whose
    /// target object is outside the morphism's allowed targets is recorded
    /// as a typing violation and stores no morphism value.
    #[test]
    fn forbidden_edge_not_stored() {
        let c1 = parse(
            r#"---
id: c1
kind: intent
statement: "traces_to must reach an Intent, not a constraint row"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| r | invariant | `true` | [[c1.other]] |
| other | invariant | `true` |  |
"#,
        );
        let inst = Instance::from_specs(&[c1]);
        assert_eq!(inst.violations().len(), 1, "the forbidden link is reported");
        assert!(
            inst.violations()[0].reason.contains("traces_to"),
            "the violation names the morphism: {:?}",
            inst.violations()[0]
        );
        assert_eq!(inst.stored(), 0, "no morphism value stored for it");
        assert!(
            inst.dangling().is_empty(),
            "the target resolves — it is a violation, not a dangling link"
        );
    }

    /// `rebuild_is_byte_stable` (delta scenario): the same corpus built
    /// with input files in two different orders serializes byte-identically.
    #[test]
    fn rebuild_is_byte_stable() {
        let c1 = parse(
            r#"---
id: c1
kind: intent
statement: "clean baseline, first file"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| r | invariant | `true` | [[c2]] |
"#,
        );
        let c2 = parse(
            r#"---
id: c2
kind: intent
statement: "the other file, pointing back"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| s | invariant | `true` | [[c1]] |
"#,
        );
        let forward = Instance::from_specs(&[c1.clone(), c2.clone()]);
        let shuffled = Instance::from_specs(&[c2, c1]);
        assert_eq!(
            forward.serialize(),
            shuffled.serialize(),
            "shuffled input order rebuilds byte-identically"
        );
    }

    /// `duplicate_id_first_wins_parity` (delta scenario): a corpus with two
    /// rows sharing the file-qualified id — which the existing builder
    /// accepts first-wins via `kind_index`'s `or_insert` — resolves the
    /// same way in the builder, with the collision NAMED in the collision
    /// report, never a hard failure the old path did not emit.
    #[test]
    fn duplicate_id_first_wins_parity() {
        // Two files with the SAME intent id `c`: the first defines row `r`
        // as an invariant Constraint, the second as a unit Property.
        // kind_index keeps the FIRST (`Constraint("invariant")`), so a guard
        // link to `c.r` is allowed — the builder must resolve identically.
        let first = parse(
            r#"---
id: c
kind: intent
statement: "first file — the first-wins winner"
---

## Constraints

| id | kind | expr |
|----|------|------|
| r | invariant | `true` |
"#,
        );
        let second = parse(
            r#"---
id: c
kind: intent
statement: "duplicate id — must not erase or fail"
---

## Properties

| id | kind | generator | predicate |
|----|------|-----------|-----------|
| r | unit | `arb()` | `true` |
"#,
        );
        let user = parse(
            r#"---
id: u
kind: intent
statement: "guards the duplicated id's row"
---

## Model

### States
- `alive`
- `dead`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | alive | dead | [[c.r]] |
"#,
        );
        let specs = [first, second, user];
        let inst = Instance::from_specs(&specs);
        // First-wins parity: the guard link resolves against the FIRST
        // file's invariant constraint, so the guard edge stores — exactly
        // what graph::build's kind_index or_insert yields.
        assert_eq!(
            inst.edges(),
            crate::graph::build(&specs).edges,
            "the builder resolves the duplicate-id corpus identically to the old path"
        );
        // … and the collision is NAMED — never silently swallowed.
        assert_eq!(
            inst.collisions().len(),
            1,
            "one collision report for the duplicated id: {:?}",
            inst.collisions()
        );
        assert!(
            inst.collisions()[0].contains("c.r"),
            "the collision names the duplicated file-qualified id"
        );
    }
}
