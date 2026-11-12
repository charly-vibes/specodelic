# Experiment E2 — Kernel coverage of fragments (specodelic-4und)

**Label: n=1-author.** One agent authored the re-expressions, the probes, and
the parity judgment; the bundle's own author-confounded mutants are the
fixtures (same confounder the PARTIAL report discloses, §9). Measurements
below were taken by running the repo's own binary on scratch specs; no
product code, tests, or corpus files were edited.

**Status:** measurement complete; gates `add-shared-fragment-helpers`
(specodelic-ez9a.1). Falsifiable rule applied in §7.

## 1. Provenance

| Item | Value |
|---|---|
| Ticket | `specodelic-4und` (blocks `specodelic-ez9a.1`) |
| Bundle | `/var/home/sasha/cotorreo/raw/_conversations/claude/56d084d5-6e1c-407f-afba-b2fe244f101f/artifacts/specodelic_bundle_v3.tar.gz` |
| Bundle SHA-256 | `c854d5904d7380c390a9cf4f24a4f77caf44e2a32b970894050427b2938cbab6` |
| Extraction dir | `/var/tmp/4und-bundle/` (not committed) |
| Bundle tool | specodelic 0.12.1, tag `v0.12.1` → `6bd389f` (bundle `repo_commit.txt`) |
| Probe tool | repo `target/debug/specodelic` 0.12.1, rustc 1.98.1, `TMPDIR=/var/tmp/specodelic-xch1` |
| Scratch work | `/var/tmp/4und-probes/` (generator `gen_probes.py` + 13 emitted fixtures; not committed) |
| Framing cite | `specodelic_research_report_PARTIAL_v2.md` (catitas study interim report; findings O1–O16, I1–I7 cited inline) |

Bundle-contents deviation from the brief: `harness/` actually contains 15
Python files — `k08.py` exists (unlisted in the brief) and **no `mutB*.py`
exists** (brief listed them). Everything else named is present. The
inventory below enumerates what actually exists.

## 2. Method (tdd-ro5 honest mapping)

The repo-contract pipeline `tdd-ro5` was executed stepwise via
`wai pipeline next` (run `tdd-ro5-2026-11-12-specodelic-4und-kernel-coverage-of-fragments`).
The phase names do not map onto a measurement ticket; the honest mapping,
recorded in the step artifacts:

| Pipeline step | What it meant here |
|---|---|
| orient / plan | bundle extraction, ticket + kernel/compile/model_check specs read, probe design |
| red | the detection probes: each bundle fixture mutation must refute the re-expressed kernel claim (probe "red" = exit≠0, claim `counterexample`) |
| green | baseline fixtures must stay clean with **zero checking fragments** (all claims `verified`) |
| refactor | none — no production code exists to refactor |
| ro5u-review / fixes | self-review of the measurement evidence; fixes landed in this report's text |
| quality-ledger / report | this file |

No red→green product cycle was fabricated; nothing in `src/`, `tests/`,
`openspec/` was touched. Probe execution command (per case):
`specodelic orchestrate specs` with cwd=`/var/tmp/4und-probes/<case>/`.

**Probe scaffolding disclosure:** every probe spec carries one zero-content
stub `**rust:** true` invariant (`c_alive`) that transition guards cite —
transition guard citations resolve against *exec* outcomes only
(`src/model_check.rs:994–1004`), so without an exec-backed cited claim the
guards would be undischargable. Deriving property rows repeat the stub to
satisfy lint `coverage`. The stubs contain no checking content; every
discriminating byte in the probes is a `**kernel:**` claim.

## 3. What the kernel grammar actually covers (repo HEAD)

`openspec/specs/kernel/spec.md` + `src/kernel.rs`: closed atomics
`{resolves, unique, acyclic, reachable, ==, <, <=, >, >=, ∀, ∃, ∧, ¬}` over
the finite acset instances I(k); objects `{Constraint, Intent, Property,
State, Transition}`; guard citations `[[a]] ∧ [[b]]` evaluate under the
citation algebra. Two measured boundaries that matter for re-expression:

1. **`reachable(a, b, m)` walks one named reference morphism**
   (`src/kernel.rs` `Atomic::Reachable` → `query::forward_closure`). The
   schema's morphisms are the reference-typing rows (`traces_to`,
   `derives_from`, `guard`, `from`, `to`, …). There is **no State→State
   morphism**, so v0 has *no transitive-closure reachability over the
   transition graph* — the exact thing `bfs_frag` computes.
2. **Id literals are string-compared, unresolvable, and unqualified by
   default.** `kind_index` qualifies row ids corpus-wide
   (`probe.traffic.t1`, Revision 18). A kernel literal must be written
   file-qualified (`t.to == probe.traffic.both_go`); an unqualified literal
   matches nothing and — measured in probe round 1 — makes
   `¬(∃ t: t.to == both_go)` evaluate **verified vacuously** (no resolution
   check on `Term::Id`, no unknown). This is a live honesty hazard for
   kernel adoption (finding F1).

## 4. Helper inventory and classification

Classification: **structural** (state-set membership, reachability,
guard/invariant shape, transition legality) vs **arithmetic/data** (math
over sequences/counters/bitmasks) vs **oracle/scoring** (expected-value
comparison) vs **scaffolding** (generators/runners — excluded from the
rule's share; they emit or execute checks but are not themselves checks).

| File | Helper | Class | Notes |
|---|---|---|---|
| `sk.py` | `build`, `orch`, `mutate` | scaffolding | spec generator, orchestrate runner, mutation runner |
| `fsm.py` | `bfs_frag` | **structural** | BFS reachability + forbidden-state fragment generator (the FSM kata checking core) |
| `fsm.py` | `build_fsm` | scaffolding | spec assembler |
| `k04.py` | fixtures a1/a2/a3 | mutations | traffic phase model |
| `k05.py` | `post`, `rev`, `split`, `lcg` | arithmetic | ledger posting, reversal, fair split, PRNG; fixtures m1–m6 |
| `k07.py` | `gen` | scaffolding | elevator model generator; fixtures e1–e4 |
| `k08.py` | `gen`, FORB name list | scaffolding + structural data | approval model; FORB = 30 forbidden published-state names (the membership set) |
| `k11.py`, `k11_rev3.py` | `eff`, `or`, `lcg` | arithmetic | bitmask deny/delegate evaluator; fixtures m1–m5 |
| `bowl.py` | `automaton` | scaffolding | bowling legality model generator |
| `bowl.py` | `oracle` | oracle/scoring | independent weight-based scorer |
| `bowl.py` | `walk_games` | arithmetic/data | random legal-game generator (scorer input) |
| `bowl.py` | `py_accepts` | **structural** | language membership via ε-closure (NFA acceptance) |
| `bowlspec.py` | `P_pins`, `P_strike`, `P_final` | **structural** | state-NAME-shape invariants: parse `e3_4_5` → frame/rolls, pin arithmetic per frame |
| `bowlspec.py` | `cases` | scaffolding | positive/negative example wrapper |
| `bowlspec.py` | `WALK` (`adj`, `play`, `last`, `silent`) | arithmetic/data | random walk over the model feeding the scorer |
| `bowlspec.py` | `SC`, `OR` | oracle/scoring | spec-side scorer + independent oracle, expected-value compare |
| `bowlspec.py` | `ACC` (`clo`, `accepts`) | **structural** | ε-closure membership, Rust-fragment twin of `py_accepts` |
| `scale.py` | `count` | **structural** | BFS enumeration of reachable bowling config space with score accumulation (state-space sizing) |
| `mutA*.py`, `mutC.py` | `M`/`L`/`Sx` tables, `run`, `detail` | scaffolding + mutations | cargo-test / spec-side mutation runners and result extraction |
| `rev6.py` | arm-C vs arm-A comparator | scaffolding | L6_early_over cross-arm comparison |

**Counts.** Check helpers in scope: **structural = 7** (`bfs_frag`,
`py_accepts`, `ACC`, `P_pins`, `P_strike`, `P_final`, `count`),
**oracle/scoring = 3** (`oracle`, `OR`, `SC`), **arithmetic/data = 6**
(`post`, `rev`, `split`, `lcg`, `eff`, `or` — `walk_games`/`WALK` are data
generators, not checks). Non-arithmetic check helpers **= 10**.

## 5. Re-expression: kernel forms used

All probes runnable; fixtures are byte-ported models from the harness
(k04/k08/k07 generators) with `**kernel:**` constraints replacing the
fragment checking content.

**S1 — forbidden-state edge (k04 `no_double_green`, k07 `never_moves_door_open`):**

```
| c_no_both_go | invariant | `**kernel:** ¬(∃ t ∈ Transition: t.to == probe.traffic.both_go)` | [[probe.traffic]] |
| c_never_moves_door_open | invariant | `**kernel:** ¬(∃ t ∈ Transition: t.to == probe.elevator.fault_moving_door_open)` | [[probe.elevator]] |
```

One-hop form of the BFS helper: "no transition enters the forbidden state".
Stronger than BFS-from-initial in general (also flags edges in from
unreachable sources); on all bundle fixtures the entering edge's source is
reachable, so detection coincides (review finding H2).

**S2 — forbidden source→target pair (k04 skip-yellow, k07 skip-floor):**

```
| c_no_skip_yellow | invariant | `**kernel:** ¬(∃ t ∈ Transition: t.from == probe.traffic.ns_go ∧ t.to == probe.traffic.all_red_1)` | [[probe.traffic]] |
```

Elevator skip-floor needed a 48-conjunct chain (∀ over 6 `f1_up_*` sources ×
8 `f3_*` targets — 4,269 claim bytes) because the grammar has `∧` but no
`∨`/sets; measured to fire (below).

**S3 — forbidden name-set membership (k08 `published_only_when_both_approved_current`):**

```
| c_publish_current | invariant | `**kernel:** ∀ t ∈ Transition: ¬(t.to == probe.approval.published_v1_no_no) ∧ ¬(t.to == probe.approval.published_v1_no_ok1) ∧ … (30 conjuncts)` | [[probe.approval]] |
```

The 30-name FORB enumeration the harness kept as data becomes the claim
itself (1,419 claim bytes). This is the k08 helper's per-state name check,
lifted to the transition table.

**Not expressible (grammar-impossible in v0), with reasons:**

- `py_accepts` / `ACC` — ε-closure membership is transitive closure + symbol
  matching; the closed set has single-morphism `reachable` over the
  *reference* graph only (§3.1). Probe `nonmember` confirmed the labeled
  rejection: `frame(s) < 10` fails at compile naming the atomic and the
  closed set (`kernel_grammar_closed`).
- `P_pins` / `P_strike` / `P_final` — decompose state NAMES (string prefix/
  split) and do pin arithmetic per component; ids are opaque to the kernel
  (no string projections).
- `count` — counts *reachable configurations with weights*; `|State|` is a
  kernel term but counts declared rows, not reachable configs.
- Liveness/deadlock (k04-a3 stuck, k07-e3 door-stuck) and data effects
  (k07-e4 lost requests) — no temporal/data-carrying form (same deferral as
  `fragment_guard_rejected`).

**Citation algebra measurement (the ticket's "pure [[a]] ∧ [[b]] already
executable" premise):** half-true, measured. `[[exec]] ∧ [[exec]]`
verifies with no fragments (`c_combo_exec`, `c_combo_dummy` → verified,
file clean). But `[[exec]] ∧ [[kernel]]` evaluates **unknown** ("missing
target … no same-run evidence" — citations resolve against exec outcomes
only, `src/model_check.rs:994`), and Rule 4 of `aggregate_required_claims`
turns one unknown into `exploration_only` — the file cannot go clean. So
citations compose *fragment* claims, not kernel claims (finding F2).

## 6. Measurements

### 6.1 Detection parity (orchestrate exit + claim status vs bundle-log helper outcomes)

| Fixture | Helper outcome (bundle logs) | Kernel probe outcome | Verdict |
|---|---|---|---|
| traffic base | exit 0, green | exit 0; all claims `verified` | parity ✓ |
| k04 a1 unsafe phase | **killed** (counterexample_found) | exit 1; `c_no_both_go` = counterexample | **MATCH** |
| k04 a2 skip yellow | **SURVIVED** | exit 1; `c_no_skip_yellow` = counterexample | kernel **EXCEEDS** |
| k04 a3 stuck cycle | **SURVIVED** (liveness) | exit 0 — no v0 kernel form exists | both miss (no form) |
| approval base | exit 0, green (16 states) | exit 0; `c_publish_current` `verified` | parity ✓ |
| k08 w1 stale approval | **killed** | exit 1; `c_publish_current` = counterexample | **MATCH** |
| k08 w2 one approval | **killed** | exit 1; counterexample | **MATCH** |
| k08 w3 override rejection | **killed** | exit 1; counterexample | **MATCH** |
| elevator base | green (69 states, 183 trans) | exit 0; all claims `verified` | parity ✓ |
| k07 e1 door-while-moving | **killed** | exit 1; `c_never_moves_door_open` = counterexample | **MATCH** |
| k07 e2 skip floor | **SURVIVED** | exit 1 with 48-conjunct pair chain | kernel **EXCEEDS** |
| k07 e3 door stuck | **SURVIVED** (liveness) | no form | both miss |
| k07 e4 lost requests | **SURVIVED** (data effect) | no form | both miss |

Helper-side kill/survive baselines: `logs/k04_results.json`,
`logs/k08_results.json` (bundle); k07 e1–e4 outcomes per PARTIAL report §3
control-centric table (I4: 5/5 caught were "named bad state unreachable",
5/5 survived were transition-ordering/liveness/adjacency/data).

Attribution note (review finding H1): the *violated leg* differs — the
helper kills via model_check reaching the bad state through the BFS
fragment property; the kernel kills via the claim aggregate (Rule 2:
refuted required claim → `counterexample_found`, no state trace, the
refuted claim id in `violated_invariant_id`). Detection (non-zero exit,
counterexample outcome) is identical; the debugging evidence is less
(claim id, not a state path).

### 6.2 Coverage of the structural share

- Kernel-expressible structural helpers: **1 of 7** (`bfs_frag`,
  partially — one-hop/pair-set/name-set forms cover its *detection role* on
  every bundle fixture it addresses; transitive closure itself is not in
  the grammar).
- **Structural share: 1/7 = 14.3%.** Non-arithmetic share (structural +
  oracle): **1/10 = 10%.**
- Sensitivity: `count` reclassified as arithmetic changes nothing (1/8);
  no plausible classification reaches 70% — the membership-closure and
  name-shape helpers are grammar-impossible, capping coverage at one helper
  under any split.

### 6.3 Residual fragment bytes (measured on the bundle's own artifacts)

From `work/k01b/C/v5/specs/bowling-game.md` (158,970 bytes total — the
largest single spec in the study), the fragments that **must remain
`**rust:**`** under full kernel adoption:

| Residual fragment | Bytes | Why not kernel |
|---|---|---|
| `c_pins` / `c_strike_end` / `c_final_frame` | 607 / 497 / 757 | state-name shape + pin arithmetic |
| `p_pins_pred` / `p_strike_pred` / `p_final_pred` | 878 / 684 / 1,066 | example wrappers around the above |
| `p_membership` (`ACC`/`py_accepts`) | **24,919** | ε-closure membership — transitive closure |
| structural residual subtotal | **29,408** | |
| `p_scoring` (`SC` + `OR` + `WALK` + examples) | 24,356 | scorer oracle + random-walk data |
| **residual total (bowling v5)** | **53,764** (~34% of the spec) | |

(Constraint+property duplication counted as emitted — the harness copies
helper text into both rows, O15/O16 of the PARTIAL report.) Cross-check
from the report: k05 rev2 ≈ 15.3 KB and k11 rev3 ≈ 13.6 KB are arithmetic
fragments end-to-end; all residual.

Counter-side (what the kernel displaces): probe traffic_base is 1,439 B
with zero checking fragments vs the harness A_base at 1,507 B *with* a
copied BFS fragment; elevator_base is 16,486 B vs the harness E2 spec at
27 KB (PARTIAL report I6); approval_base is 4,591 B. The FSM-kata checking
pattern — the thing `fsm.py` existed for — needs no fragment at all.

## 7. Rule application (verbatim from the ticket)

> *"if kernel/citation coverage of the structural share is high (>= 70% of
> non-arithmetic helpers), the report recommends rescoping
> add-shared-fragment-helpers to data-helpers only or superseding it; else
> the proposal stands as written."*

- Measured coverage: **14.3%** of the structural share (1/7), 10% of
  non-arithmetic helpers (1/10).
- 14.3% < 70% → **the proposal stands as written.** No rescope, no
  supersede. `add-shared-fragment-helpers` should proceed.

**Scope note for the implementation ticket** (input, not a rule outcome):
the kernel already displaces the entire FSM bad-state pattern (`bfs_frag`'s
role — matched parity on 5/5 helper-killed fixtures, exceeded on 2
survivors, at smaller spec size). The fragment helpers' real residual
target is: membership/closure (`p_membership`, 24.9 KB), name-shape
invariants (4.5 KB), scorer oracles (`p_scoring`, 24.4 KB), and the
arithmetic kata fragments (k05/k11, ~29 KB combined). Conditional legality
that depends on guard data (k07-e4-style) remains out of both worlds for
now.

## 8. Findings (for the format/tool backlog)

- **F1 (High):** kernel id literals are unresolvable and compared
  unqualified; a typo'd or unqualified literal yields *vacuous verified*,
  not unknown (measured live in probe round 1). Candidates: resolve-check
  `Term::Id` against the corpus → unknown with labeled reason, or a lint
  advisory on kernel cells whose literals resolve to nothing.
- **F2 (High):** the citation algebra composes exec outcomes only; a
  citation over a kernel-evaluated claim is undischargable and (Rule 4)
  blocks the file from clean. Cross-evaluator composition is unmeasured
  territory worth a design note before authors mix claim kinds.
- **F3 (Medium):** one-hop forbidden-edge semantics is stronger than
  BFS-from-initial (flags edges from unreachable sources). Document as the
  intended reading of the S1/S3 forms.
- **F4 (Medium):** pair-set/name-set claims are mechanical but verbose —
  48 conjuncts (4.3 KB) for elevator skip-floor, per-conjunct cost makes
  large enumerations rival the fragments they displace. A future `∨`/set
  atomic (decidability-gated per the widening law) would compress this.
- **F5 (Low):** the stub-property pattern (`**rust:** true` rows to satisfy
  `coverage` + guard citations) is the current honest way to keep a
  kernel-only file clean; worth a first-class "kernel property" story so
  authors don't cargo-cult stubs.

## 9. Quality ledger

**Changed:** one file — this report. Scratch under `/var/tmp/4und-*`
(generator + 13 probe fixtures + extracted bundle), nothing committed.

**Verified:** 25 `specodelic orchestrate specs` invocations across two
rounds (traffic ×8, approval ×8, elevator ×4, citeover ×2, nonmember ×2,
incl. the round-1 rerun after the qualified-literal fix) with per-case
`specodelic/*.check.json` claim-status extractions; byte measurements from bundle artifacts (`logs/*.json`,
`work/k01b/C/v5/specs/bowling-game.md`). Helper baselines quoted from
bundle logs, not re-run.

**Review:** RO5U self-review applied; findings H1 (violated-leg
attribution), H2 (one-hop vs BFS semantics), H3 (F1 hazard) addressed in
§6.1/§8; M1–M3 addressed in §6.3/§5. Remaining: none blocking.

## Report

**Commits**
- (this commit) `docs(4und): E2 kernel-coverage-of-fragments measurement report` — the experiment deliverable; gates add-shared-fragment-helpers; no code change

**Measurements**
- inventory n=15 harness files (7 structural, 3 oracle, 6 arithmetic check helpers; scaffolding excluded), structural share kernel-expressible = 1/7 = 14.3% (non-arithmetic 1/10 = 10%), residual fragment bytes = 53,764 in bowling v5 (29,408 structural + 24,356 scoring), detection parity 5/5 MATCH on helper-killed fixtures + 2 EXCEEDS (k04-a2, k07-e2), 4 no-form both-miss (a3, e3, e4 + B-product false-alarm class)

**Rule applied**
- ">=70% of non-arithmetic helpers → rescope/supersede" → measured 14.3% of structural share (10% non-arithmetic) → **proposal stands** (add-shared-fragment-helpers proceeds as written; kernel displaces only the FSM bad-state pattern — noted as implementation input)

**Deviations**
- brief listed `mutB*.py` — none exist in the bundle; `k08.py` exists unlisted. Inventory enumerates actual contents.
- Probe property rows use a documented zero-content `**rust:** true` stub (lint `coverage` + guard-citation scaffolding); all checking content is kernel claims.
- k07 e3/e4 and k04 a3 have no kernel form — recorded as no-form both-miss, not counted as parity runs.
- Pipeline red/green steps mapped to detection/baseline probe runs (measurement ticket); no product TDD cycle exists or was fabricated.

**Next**
- Orchestrator: close specodelic-4und with recommendation "proposal stands", and attach §7's scope note + §8 findings (F1/F2 first) as inputs to specodelic-ez9a.1 and the format backlog; F1/F2 are pre-existing tool behaviors, not 4und defects — file as separate issues if wanted.
