//! specodelic — the Specodelic format library shared by the `specodelic` CLI (binary `specodelic`, alias `spk`).
//!
//! Purpose: model the four-layer markdown spec format (Intent /
//! Constraints / Model / Properties) so the CLI commands and tests all
//! speak the same parse. Responsibilities: expose `spec` (the parser),
//! `ears` (the statement grammar), `lint` (the invariants), `graph` (the
//! derived reference graph), `compile` (the Constraints→TOML /
//! Model→IR / Properties→proptest! functor), `fixtures` (the named
//! property fixtures a compiled property cell may materialize), and
//! `guide` (the closed
//! value sets + format revision + embedded primer). Rationale: keep the domain core in a
//! library so the binary stays thin and the corpus can be dogfooded from
//! integration tests.

pub mod acset;
pub mod archive_companion;
pub mod audit;
pub mod blocks;
pub mod checklist;
pub mod citation_corpus;
pub mod compile;
pub mod conform;
pub mod doctor;
pub mod ears;
pub mod graph;
pub mod guide;
pub mod hooks;
pub mod human;
pub mod kernel;
pub mod lint;
pub mod merge;
pub mod migrate;
pub mod model_check;
pub mod orchestrate;
pub mod packs;
pub mod parse;
pub mod refactor;
pub mod rename;
pub mod skill_pack;
pub mod spec;
#[cfg(test)]
pub mod test_panic;
pub mod update_notice;
pub mod verify;

/// The named property fixtures a compiled property cell may draw on.
///
/// Purpose: turn a generator NAME into a concrete input. A corpus
/// property row's generator cell names a fixture
/// (`property_row_with_one_property()`), and the compiled artifact's
/// strategy materializes it through this registry — so the row's
/// predicate fragment runs against a real spec instead of against the
/// generator's own name.
/// Responsibilities: hold the registry of named fixtures, resolve a
/// name to its fixture, and materialize a fixture's concrete spec text.
/// Rationale: the materialization happens inside the compiled artifact,
/// which runs in the verify runner's scratch crate — so the fixtures
/// must live in the library that scratch crate depends on (the
/// tool-dependency seam, openspec/changes/add-verify-runner-seams design
/// D1/D2). Every fixture is a real four-layer spec text: the real
/// parser consumes it and the real checkers judge it — no test doubles,
/// no re-implemented checkers.
pub mod fixtures {
    /// One named fixture: the concrete spec text a property cell's
    /// generator yields.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Fixture {
        /// The fixture's name — the generator name that materializes it.
        pub name: &'static str,
        /// One line: what the fixture is for.
        pub summary: &'static str,
        /// The fixture's concrete spec text — a real four-layer spec file.
        pub text: &'static str,
    }

    /// A spec file with exactly one unit Property row — the input for
    /// "a property row compiles to at least one proptest! block".
    const PROPERTY_ROW_WITH_ONE_PROPERTY: &str = r#"---
id: fixture.property_row
kind: intent
statement: "THE fixture SHALL carry exactly one unit Property row."
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|------------|
| c1 | invariant | `holds` | [[fixture.property_row]] |

## Model

### States

- s1
- s2

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t | s1 | s2 | [[fixture.property_row.c1]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p_one_property_row | unit | [[fixture.property_row.c1]] | `sample_row()` | `count(compile(row).blocks) >= 1` |
"#;

    /// A spec file whose transition names a target its States list does
    /// not declare — the input for the linter's
    /// `linter.every_transition_valid` rule.
    const SPEC_FILE_WITH_UNDECLARED_TRANSITION_TARGET: &str = r#"---
id: fixture.undeclared_target
kind: intent
statement: "THE fixture SHALL name a transition target its States list does not declare."
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|------------|
| c1 | invariant | `holds` | [[fixture.undeclared_target]] |

## Model

### States

- s1

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t | s1 | s_never_declared | [[fixture.undeclared_target.c1]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p_declared_target | unit | [[fixture.undeclared_target.c1]] | `sample_row()` | `every transition target is a declared state` |
"#;

    /// The registry — the one list of named fixtures.
    const REGISTRY: &[Fixture] = &[
        Fixture {
            name: "property_row_with_one_property",
            summary: "a spec file with exactly one unit Property row",
            text: PROPERTY_ROW_WITH_ONE_PROPERTY,
        },
        Fixture {
            name: "spec_file_with_undeclared_transition_target",
            summary: "a spec file whose transition target its States list never declares",
            text: SPEC_FILE_WITH_UNDECLARED_TRANSITION_TARGET,
        },
    ];

    /// The registry — every named fixture, in registry order.
    pub fn registry() -> &'static [Fixture] {
        REGISTRY
    }

    /// The fixture named `name`, if the registry carries one.
    pub fn lookup(name: &str) -> Option<&'static Fixture> {
        REGISTRY.iter().find(|f| f.name == name)
    }

    /// The fixture named `name`'s concrete spec text — what a compiled
    /// property cell's strategy yields.
    pub fn materialize(name: &str) -> Option<String> {
        lookup(name).map(|f| f.text.to_string())
    }

    /// Every fixture name in the registry.
    pub fn names() -> Vec<&'static str> {
        REGISTRY.iter().map(|f| f.name).collect()
    }

    /// Is `name` a fixture name? The compiler's branch condition: a
    /// generator whose name is a fixture name materializes through the
    /// registry; every other generator keeps its placeholder emission.
    pub fn is_known(name: &str) -> bool {
        lookup(name).is_some()
    }
}

/// The scoped verdict a CI gate reads: the properties stage alone.
///
/// Purpose: `openspec/specs/verify/spec.md`'s properties-only scope
/// requirement — `properties_failed` exactly when a compiled block
/// failed, never for a missing or stale model run.
/// Responsibilities: hold the scope selector, map the properties gate
/// state to the scoped verdict, and share ONE implementation of the
/// properties branch with the full verdict.
/// Rationale: the scope lives at the library level (a library consumer
/// and the `just verify-properties` recipe read the same mapping), and
/// the shared branch means the scoped verdict and the full verdict can
/// never drift on what "properties failed" means. The `verified`
/// transition keeps its own both-gates-required semantics — the scoped
/// verdict never reports `verified`.
pub mod scope_verdict {
    use crate::verify::{ModelGateState, PropsGateState, Verdict};

    /// The properties branch of a verdict: the blocking verdict when the
    /// properties gate is not a pass, `None` when it passed (the caller
    /// decides on the model gate). ONE implementation — the full verdict
    /// and the properties-only scope verdict read the same mapping, so the
    /// two can never drift on what "properties failed" means.
    pub(crate) fn properties_verdict(properties: &PropsGateState) -> Option<Verdict> {
        let (status, message, hint) = match properties {
        PropsGateState::Pass => return None,
        PropsGateState::Failed(summary) => (
            "properties_failed",
            format!("{summary} — a partial pass is a failure of the property"),
            "fix the failing predicates (see each block's detail for the shrunk failing input)".to_string(),
        ),
        PropsGateState::Stale => (
            "stale_properties_artifact",
            "the on-disk *_props.rs artifact predates the current spec — properties from a stale artifact never count".to_string(),
            "run: specodelic compile <files> to regenerate the artifact".to_string(),
        ),
        PropsGateState::MissingArtifact => (
            "missing_properties_artifact",
            "no compiled proptest artifact for this spec — verify consumes compile's output, it never re-compiles".to_string(),
            "run: specodelic compile <files>".to_string(),
        ),
        PropsGateState::Uncompilable(output) => (
            "properties_uncompilable",
            "the *_props.rs artifact did not compile as a Rust test crate".to_string(),
            format!("fix the artifact's Rust (first errors): {output}"),
        ),
        PropsGateState::RunnerUnavailable(detail) => (
            "runner_unavailable",
            "the properties gate executes the compiled proptest! blocks and needs the Rust toolchain".to_string(),
            format!("install the Rust toolchain so cargo is on PATH ({detail})"),
        ),
        PropsGateState::TimedOut { secs, detail } => (
            "properties_timed_out",
            format!(
                "the cargo test run exceeded its {secs}s wall-clock bound and was killed — a hanging (likely pathological) predicate is an unbounded DoS on verify, and no block verdicts exist yet"
            ),
            format!(
                "raise --timeout-secs (or pass --timeout-secs 0 to run unbounded) and re-run; the partial output names the block that hung: {detail}"
            ),
        ),
    };
        Some(Verdict {
            status,
            message,
            hint,
        })
    }

    /// The scope a verdict is computed in.
    ///
    /// `BothGates` is the full `verified` transition
    /// (`both_gates_required`); `PropertiesOnly` is the scoped CI verdict —
    /// the properties gate alone, for a recipe that gates on the properties
    /// stage without the model stage hiding it.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum VerifyScope {
        /// Both gates: the full `verified` transition.
        BothGates,
        /// The properties gate alone — never reports `verified`.
        PropertiesOnly,
    }

    /// The verdict for `scope`. The scope selects WHICH gates the verdict
    /// reads; it never weakens a gate: the properties-only scope reports the
    /// properties verdict alone and can never report `verified`.
    pub fn verdict_in_scope(
        properties: &PropsGateState,
        model: &ModelGateState,
        scope: VerifyScope,
    ) -> Verdict {
        match scope {
            VerifyScope::BothGates => crate::verify::verdict(properties, model),
            VerifyScope::PropertiesOnly => properties_only_verdict(properties),
        }
    }

    /// The properties-only scope verdict: `properties_failed` exactly when a
    /// compiled block failed, and the properties gate's own status for every
    /// other state.
    ///
    /// The model gate is NOT read here: a missing or stale model run never
    /// produces `properties_failed` — a scoped CI gate reads the properties
    /// stage, not the model stage. And this verdict NEVER reports `verified`:
    /// the `verified` transition keeps its own semantics
    /// (`both_gates_required`), so a properties-only pass can never be
    /// mistaken for a verified file.
    pub fn properties_only_verdict(properties: &PropsGateState) -> Verdict {
        match properties_verdict(properties) {
        Some(blocker) => blocker,
        None => Verdict {
            status: "properties_passed",
            message: "every compiled proptest! block passed — the properties-only scope does not evaluate the model gate, so this file is NOT verified".to_string(),
            hint: "run: specodelic model-check <files>, then specodelic verify <files> for the full both-gates verdict".to_string(),
        },
    }
    }
}
