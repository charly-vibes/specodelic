# Tasks

## 1. Wiring (maps to: The harness suite runs at declared points and never in the commit gate)

- [ ] 1.1 ESTABLISH: prove the separation property first — a `#[cfg(kani)]`-free
  debug and release build contains no harness symbols (scripted check);
  confirm `just ci` invokes no `cargo kani` (recipe inspection + run log).
  Green: the check script passes against the unmodified tree, establishing
  the invariant the rest of the change must preserve.
- [ ] 1.2 Add the `just kani` recipe (locked `cargo kani` invocation over the
  crate) and verify `just kani` on a stub harness reports pass/fail cleanly;
  `just ci` output byte-identical to before (spec row: dedicated recipe /
  commit gate stays kani-free).
- [ ] 1.3 Recipe toolchain guard: when `kani-verifier` is absent or its
  pinned release disagrees with the installed rustc, `just kani` fails with
  a remediation hint (install command + pinned release, design D4);
  exercise the missing-tool path once (spec row: dedicated recipe runs the
  suite — the hint is part of the recipe's report).
- [ ] 1.4 Document run points (pre-push, nightly) and the Kani release pin in
  `docs/src/verification-boundaries.md` (tool-trust row per proposal Impact).

## 2. Verdict-algebra harness (maps to: The guard-citation algebra is machine-checked)

- [ ] 2.1 Harness with mutation-kill red: exhaustive value-domain harness in
  `src/compile/kani.rs` under `#[cfg(kani)]` — commutativity, associativity,
  counterexample-dominance, verified-identity, double-negation involution
  over `kani::any` value tuples. RED = temporarily flip one evaluator arm
  (the mutant) and confirm the harness reports a counterexample naming the
  violated law, then revert (harness sensitivity proven; a red against the
  CLEAN tree is a D5 finding, separate fix). Green: harness passes on the
  clean tree and fired on the mutant (spec rows: counterexample dominance /
  double negation scenarios).
- [ ] 2.2 REFACTOR (separate commit): tidy the harness constants and law
  names into the module's bound-policy block per design D2; re-run `just
  kani`.

## 3. Guard-expr parser harness (maps to: Guard-expression parsing admits exactly the well-formed and never panics)

- [ ] 3.1 No-panic harness with mutation-kill red — arbitrary cell (≤128
  bytes, named constant) parses without panic or OOB slice; RED = remove a
  bounds guard temporarily, confirm the harness catches the panic, revert.
  Green: harness passes on the clean tree (spec row: arbitrary input cannot
  panic).
- [ ] 3.2 Bracketed-id admission harness with mutation-kill red — any cell
  whose citation id contains a bracket yields no expression; RED = tempora-
  rily accept bracketed ids, confirm the harness fires, revert. Green: passes
  on the clean tree, or D5 finding filed with minimized cell (spec row:
  bracketed id stays prose). (Round-trip fixed point deferred — see
  proposal What Changes.)
- [ ] 3.3 REFACTOR (separate commit): shared cell-shaping constant between
  parser harnesses; re-run `just kani`.

## 4. Acyclicity differential harness (maps to: The acyclicity engine agrees with a reference oracle within the bound)

- [ ] 4.1 Differential harness with mutation-kill red: naive exponential
  reference checker written inside the harness module (design D3); assert
  engine cycle set == reference cycle set on arbitrary bounded edge sets
  (≤7 nodes, ≤12 edges), including planted cycles and acyclic cases; RED =
  temporarily replace the engine with a no-op, confirm the harness fires,
  revert. Green: passes on the clean tree, or D5 finding (spec rows:
  planted cycle / acyclic graph scenarios).
- [ ] 4.2 Run the harness against both cycle surfaces (`find_cycles` in
  `src/lint/graph.rs` and `find_supersedes_cycles` in `src/graph.rs`);
  record per-surface results AND the per-surface normalization rule in the
  harness module doc — the surfaces differ in shape
  (`&BTreeSet<(String, String)>` → full cycle paths vs `&[Edge]` → member
  ids), so "same cycle set" is defined per surface, not assumed.
- [ ] 4.3 REFACTOR (separate commit): extract the shared bounded-graph
  generator; re-run `just kani`.

## 5. Reachability totality harness (maps to: Reachability classification is total and exclusive)

- [ ] 5.1 Tier-totality harness with mutation-kill red: arbitrary bounded
  spec graphs via minimal hand-built `Spec` values (design D2 — target the
  pure tier-classification core if Spec construction proves heavyweight);
  every row classified exactly once across the three tiers; RED = tempora-
  rily drop one tier assignment, confirm the harness fires, revert. Green:
  passes on the clean tree, or D5 finding (spec row: every row lands in
  exactly one tier).
- [ ] 5.2 REFACTOR (separate commit): align bound constants with family 4;
  re-run `just kani`.

## 6. Honest scope labeling (maps to: Bounded harness results are labeled honestly)

- [ ] 6.1 Harness module result labels: algebra family marked exhaustive,
  parser/graph families marked bounded at their named constants; `just kani`
  output surfaces the labels (spec row: graph family reports its bound).
- [ ] 6.2 RED: a check that every bounded family's label names its bound
  constant (grep-level script in the recipe); green when labels match D2's
  recorded bounds.

## 7. Gate discipline re-verification (maps to: The harness suite runs at declared points and never in the commit gate)

- [ ] 7.1 Re-run 1.1's separation check against the finished tree (release
  build clean, `just ci` unchanged, wall-clock delta ~zero); attach the run
  log evidence to the closing summary (evidence-citation standing rule).
- [ ] 7.2 Full `just ci` green; `just lint-specs` green (corpus untouched);
  `ah check` green with this change's overlay.
