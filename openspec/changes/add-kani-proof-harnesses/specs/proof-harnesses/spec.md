# Spec Delta

## Purpose
Give specodelic's verdict-deciding internals the same machine-checked
treatment the tool applies to user corpora: the laws the code claims in
doc-comments and specs become Kani-checked obligations, so the ladder's
foundations are themselves verified, not merely sampled by tests.

## ADDED Requirements

### Requirement: The guard-citation algebra is machine-checked

The three-valued verdict algebra used to compose guard citations (conjunction
with counterexample-dominance and unknown-absorption, negation flipping
verified/counterexample and preserving unknown) SHALL be checked by the
harness suite for commutativity, associativity, counterexample-dominance,
verified-identity, and involution of double negation, over the ENTIRE value
domain — a complete proof, not a sample. A violation of any law is a harness
failure naming the law.

#### Scenario: Counterexample dominance holds for every pairing
- **WHEN** either conjunct of a composed citation evaluates to counterexample
- **THEN** the composition evaluates to counterexample regardless of the other
  conjunct's value, for all value combinations the domain admits

#### Scenario: Double negation restores the verdict
- **WHEN** a composed citation is negated twice
- **THEN** the result equals the original verdict for every value the domain
  admits, including unknown

### Requirement: Guard-expression parsing is total and admits only well-formed cells

The guard-expression parser SHALL return no verdict-cell expression for any
input whose citation ids contain brackets (malformed cells stay prose) and
SHALL complete without panic on ANY input cell whatsoever.

#### Scenario: Bracketed id stays prose
- **WHEN** a cell parses to a citation whose id would contain a bracket
- **THEN** the parser yields no expression, leaving the cell prose

#### Scenario: Arbitrary input cannot panic
- **WHEN** the parser runs on an arbitrary input cell within the harness bound
- **THEN** it terminates with a result, never a panic or out-of-bounds slice

### Requirement: The acyclicity engine agrees with a reference oracle within the bound

The cycle-detection engine that gates the acyclic reference graph SHALL, for
arbitrary edge sets within the harness unwinding bound, report exactly the
cycle set a naive reference checker written in the harness reports — same
cycles, no invented cycles, no missed cycles.

#### Scenario: Planted cycle is found identically by both
- **WHEN** the harness generates an arbitrary bounded edge set containing at
  least one reference cycle
- **THEN** the engine and the reference checker report the same cycle set

#### Scenario: Acyclic graph yields empty from both
- **WHEN** the harness generates an arbitrary bounded edge set with no cycle
- **THEN** both the engine and the reference checker report no cycles

### Requirement: Reachability classification is total and exclusive

The tiered reachability classification (own-file reachable, cross-file-only
advisory, or failure) SHALL assign every row exactly one tier for arbitrary
bounded spec graphs — no row unclassified, no row classified twice.

#### Scenario: Every row lands in exactly one tier
- **WHEN** the harness generates an arbitrary bounded spec graph
- **THEN** each row receives exactly one tier assignment and the union of
  assignments covers all rows

### Requirement: The harness suite runs at declared points and never in the commit gate

The harness suite SHALL be runnable via a dedicated recipe (`just kani`),
SHALL NOT execute as part of the existing per-commit quality gate (`just ci`),
and SHALL leave no harness code in release builds.

#### Scenario: Dedicated recipe runs the suite
- **WHEN** `just kani` is invoked
- **THEN** every harness family runs and reports pass or a counterexample
  naming the failing law

#### Scenario: The commit gate stays kani-free
- **WHEN** `just ci` runs
- **THEN** no Kani verification executes and gate wall-clock is unchanged

### Requirement: Bounded harness results are labeled honestly

The suite SHALL label each family's result scope: the verdict-algebra family
as exhaustive over its domain, the graph and parser families as bounded by
their stated unwinding limits — and no bounded result SHALL ever be presented
as an unbounded proof.

#### Scenario: Graph family reports its bound
- **WHEN** a graph-family harness completes within its unwinding bound
- **THEN** the result is labeled bounded at that bound, never as a general proof
