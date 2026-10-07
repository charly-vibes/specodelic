# Rule of 5 Review - Final Report

**Current status:** all four findings were addressed in the proposals after
the user requested fixes. The review below is the original assessment; see
the resolution and validation ledger at the end for the current disposition.

**Work Reviewed:** the working-tree proposal, design, tasks and capability
snapshots for `add-min-expr-kernel`, `define-verification-claim-gates`,
`add-py-fragment-emission`, `add-ts-fragment-emission`, and `add-graph-views`.
Reviewed against base 29f81ef, repository conventions, implementation, and
the proposal tracking records. This is a proposal review, not implementation
approval or a claim that proposed behavior already exists.

**Convergence:** completed Stage 5; no early convergence at Stages 2–4.
The final pass found no additional issues. Outstanding findings still need
revision and re-review; discovery saturation is not acceptance.

## Summary

Total Issues by Severity:

- CRITICAL: 0
- HIGH: 2 — should fix before implementation
- MEDIUM: 2 — resolve the scope/interface decisions before task execution
- LOW: 0

## Top findings

1. **CORR-001 — Conflicting compatibility requirements.** Python and
   TypeScript full compile snapshots promise unchanged output even for
   explicit generator opt-ins whose output must change.
2. **EDGE-001 — Heterogeneous generator values lack a type contract.**
   The recursive grammar admits mixed choices without defining Rust's
   predicate input representation or rejecting incompatible branches.
3. **DRAFT-001 — Schema renderer has no specified export interface.**
   The Python script must consume a Rust schema value that its declared
   inputs do not expose.

## STAGE 1: DRAFT

Assessment: The separation of command integration, claim acceptance,
generator execution and graph presentation is sound. Test shapes precede
implementation and refactoring is separately tracked. One existing graph
design amendment has not been carried through to its producer interface.

Major Issues:

**[DRAFT-001] MEDIUM — UNVERIFIED by TypeSafe (below severity threshold)**

Location: `openspec/changes/add-graph-views/design.md:52–57,107–128`;
`openspec/changes/add-graph-views/tasks.md:49–56,70–77`;
`src/graph.rs:48–67`.

Description: The stdlib Python prototype consumes edge TSV and a JSON
envelope, but the schema view must now derive from
`acset::schema::canonical()`. The design expressly excludes guide JSON
from that derivation, and current GraphReport has no schema payload.
Task 2.6 jumps directly to rendering and perturbing constructed Rust
Schema values without defining their transport into Python. Related
stale task wording: RED 2.1 expects reference fields with allowed targets,
while GREEN 2.2 lists kinds, row shapes and revision only.

Recommendation: Specify one JSON export generated directly from canonical
Schema, including its format revision. Add a producer contract test and
feed two serialized constructed schemas to the script test. Reconcile
tasks 2.1/2.2 with the selected interface; retain the prohibition on
duplicating the typing table. This does not require promoting the renderer
to Rust or adding a Python dependency.

Shape Quality: GOOD

## STAGE 2: CORRECTNESS

Issues Found:

**[CORR-001] HIGH — VERIFIED, confidence 0.88**

Location: `openspec/changes/add-py-fragment-emission/specs/compile/spec.md:77–94`
and `openspec/changes/add-ts-fragment-emission/specs/compile/spec.md:77–94`;
generator requirement in
`openspec/changes/add-py-fragment-emission/specs/property-generators/spec.md:46–56`.
The same compile wording is repeated in each Requirements mirror.

Description: The inherited Pure widening scenario says any pre-Revision
artifact recompiles identically. The new requirement instead replaces
marked integer generator name strings with real integers. Later prose
narrows preservation to unmarked legacy inputs but leaves the universal
scenario intact, so an implementation cannot satisfy both contracts.

Evidence: The current 0.5.2 compiler successfully accepts a generator cell
`**gen:** int(0,1)` and emits `Strategy<Value = String>` containing
`Just("int".into())`. The new generator contract requires integer values
for this same syntax. This is a concrete already-compilable input, not
merely a hypothetical future syntax collision.

Reproduction: `/tmp/spk-proposal-review-s3qdtymc/kernel.md` and
`out/kernel_props.rs`; command:

```sh
target/debug/specodelic compile /tmp/spk-proposal-review-s3qdtymc/kernel.md \
  --out-dir /tmp/spk-proposal-review-s3qdtymc/out
```

Recommendation: Scope the retained compatibility requirement and its
scenarios to unmarked legacy inputs. Explicitly exclude newly opted-in
generator semantics and document migration for previously accepted marked
cells. Keep the requirement identity, update both mirrors, and propagate
the corrected baseline to the downstream TypeScript snapshot. Add paired
fixtures proving legacy byte stability and marked-generator replacement.

Correctness Quality: FAIR

Convergence check: new CRITICAL 0; new issues 1; new issue rate versus
Stage 1 100%; status CONTINUE. Final measured HIGH-finding FP rate: 0%.

## STAGE 3: CLARITY

Issues Found:

**[CLAR-001] MEDIUM — UNVERIFIED by TypeSafe (below severity threshold)**

Location: `openspec/changes/define-verification-claim-gates/design.md:36–50`;
`openspec/changes/add-min-expr-kernel/design.md:140–154`;
`openspec/project.md:28–33`; `src/acset/instance.rs:286–297`.

Description: The report design sorts by intent ID and unconditionally
rejects duplicate intent IDs. Kernel integration similarly rejects
duplicate corpus identities without defining their scope. The repository
explicitly permits multiple dual-format `id: spec` files with file-local
references, and the acset layer recognizes that convention. The proposals
do not state whether these corpora are excluded from command evaluation
or how their claim identities and report scopes remain distinct.

Impact: An implementer could reject valid dual-format input or accidentally
merge same-spelled claims while satisfying a different part of the design.
This is an unresolved command-scope contract, not a demonstrated regression
in an existing successful multi-file verification workflow.

Recommendation: Define ordinary corpus identity versus dual-format local
identity explicitly. Either specify independent evaluation/report namespaces
for these files, preserving local-only references, or explicitly exclude
combined dual-format execution and document the supported invocation.
Add a two-file `id: spec` fixture with the same local constraint ID and
different outcomes; pin the chosen diagnostic or isolated results and digest
behavior. Keep ordinary duplicate-intent rejection intact.

Clarity Quality: GOOD

Convergence check: new CRITICAL 0; new issues 1; new issue rate versus
Stage 2 100%; status CONTINUE. Final measured HIGH-finding FP rate: 0%.

## STAGE 4: EDGE CASES

Issues Found:

**[EDGE-001] HIGH — VERIFIED, confidence 0.93**

Location: `openspec/changes/add-py-fragment-emission/design.md:17–50` and
`specs/property-generators/spec.md:46–60` within the same change.
TypeScript inherits this shared contract through design decision 1.

Description: `G := ... | one_of(G,G,...)` allows branches of different
types, but the proposed typed IR and predicate variable `v0` have no
specified unification rule or tagged representation.

Scenario: `one_of(int(0,1),string(1,2))`, including the same combination
hidden behind named aliases or nested inside `list(...)`, satisfies the
written grammar and arity rules.

Impact: Authors cannot know how to write a Rust predicate for this accepted
generator. Emitters may reject different inputs or choose incompatible
representations, undermining the shared Rust/Python/TypeScript contract.
The finding is about an undefined public contract, not an assertion that
heterogeneous generation is impossible to implement.

Recommendation: For v0, define recursive type inference and require choice
branches to unify after named expansion, with labeled rejection before
emission. Document integer/string/list predicate types and add direct,
aliased and nested mixed-type rejection fixtures plus homogeneous success
fixtures. Alternatively specify a tagged sum and its mappings in every
language; that is a larger design choice.

Edge Case Coverage: FAIR

Convergence check: new CRITICAL 0; new issues 1; new issue rate versus
Stage 3 100%; status CONTINUE. Final measured HIGH-finding FP rate: 0%.

## STAGE 5: EXCELLENCE

Final Polish Issues: none additional. Rechecked ownership, full-snapshot
archive sequencing, duplicate findings, scope boundaries and testability.

The apparent kernel/policy ordering cycle was not retained as a finding:
kernel tasks 3.6–3.8 can land, then policy implementation, then corpus
migration and archival. The task-level prerequisites distinguish this from
requiring the entire kernel change to finish before any policy work.
The Python kernel reference evaluator is explicitly an agreement-test
backend, so its lack of a production selector is also not a defect.

Excellence Assessment:

- Structure: GOOD
- Correctness: FAIR
- Clarity: GOOD
- Edge Cases: FAIR
- Overall: FAIR

Production Ready: NO — proposals need revision before implementation.

## Validation and verification

- Strict OpenSpec validation: 25 passed, 0 failed.
- Section synchronization: passed.
- Concrete compile reproduction: exit 0, marked integer syntax produces
  the legacy constant string strategy on the current implementation.
- Both HIGH findings passed mechanical location/quotation checks, then
  one batched TypeSafe verification request. Returned model jev-1.13.0;
  CORR-001 verified at 0.88, EDGE-001 verified at 0.93.
- Measured false-positive rate for eligible findings: 0 / (2 + 0) = 0%.
  This small-sample measure excludes the two MEDIUM findings and does not
  certify overall review completeness. They remain UNVERIFIED by TypeSafe.
- Request/response and reproduction artifacts reside under
  `/tmp/spk-proposal-review-s3qdtymc/`; no credentials are stored there.

No proposal, tracker, runtime source, deployed spec or test changes were
made during this review. No implementation tests or contracts were invented.

## Recommended Actions

1. Fix the compatibility contradiction in Python and TypeScript snapshots.
2. Define generator branch typing and add the corresponding test shapes.
3. Specify the schema export and reconcile the graph tasks.
4. Resolve dual-format claim/report scope and add an explicit fixture.
5. Re-run strict validation and section sync after editing, then re-review
   the affected contracts before creating atomic implementation tickets.

## Verdict

**NEEDS_REVISION**

The proposed improvements address demonstrated value and correctness gaps.
Their organization is workable, but two behavioral contracts need correction
and two scope/interface decisions remain insufficiently specified.

## Resolution and validation ledger — 2026-10-06

User instruction: fix all findings. These are specification fixes; pending
implementation tasks remain unchecked.

| Finding | Disposition | Updated contract and test shape |
|---------|-------------|---------------------------------|
| CORR-001 | Addressed | Python/TypeScript full compile snapshots narrow preservation to unmarked legacy generators, retain requirement/scenario identities, and explicitly test marked integer replacement. Python proposal/design/tasks document predicate migration and regenerated evidence. Language-emitter wording preserves Rust extraction, with artifact preservation delegated to the narrowed requirement. |
| EDGE-001 | Addressed | Generator capability and design define recursive Int/String/List(T) inference, alias expansion, equal branch types, and native predicate types. Direct, aliased, nested and zero-length-list mismatches fail before emission. TypeScript uses the same rules and conformance fixtures. |
| DRAFT-001 | Addressed | Graph proposal/design/tasks/spec define guide --schema --json as a versioned direct canonical Schema export, including refinements and flags. Producer tests pin data, consumer tests use serialized constructed schemas, malformed input fails before writes. Stale ordinary guide JSON task/proposal wording is reconciled. |
| CLAR-001 | Addressed | Kernel and claim-gate contracts distinguish ordinary corpus IDs from dual-format file-local IDs. Dual-format evaluation is single-file only, with labeled preflight and split-run guidance; multi-file lint remains supported. Independent opposite-claim runs and swapped-report rejection are explicit fixtures. |

Changed — proposal/design/task documents and synchronized capability deltas
across the five reviewed changes, plus this review and the proposal handoff.
No runtime source, deployed capabilities, domain specs, tracker status or
implementation task completion changed in this follow-up.

Review — repeated the five lenses against the edits: ownership/shape,
compatibility and typing correctness, producer/consumer interface clarity,
alias/empty-list/report-swap boundaries, then cross-document consistency.
No additional unresolved finding was identified in this focused re-review.
The earlier TypeSafe results validate the original HIGH findings, not the
fixes; no new HIGH/CRITICAL finding required a verification batch.

Verified — strict OpenSpec validation: 25 passed, 0 failed; section
synchronization passed; specodelic lint: 50 files, 0 blocking findings,
18 existing advisory warnings; ah check: 0 structural issues (execution
skipped); git diff --check passed. Tests specified by these proposals
remain future implementation work; structural validation does not execute
them.

Risks — combined dual-format command execution is explicitly deferred.
Marked generators intentionally change semantics, with migration documented.
The selected interfaces and policies remain proposed until implemented.

Next — use the revised test shapes when splitting approved implementation
work into separate feature and refactoring tickets. No outstanding action
remains from these four proposal-review findings.

**Updated verdict: READY FOR IMPLEMENTATION PLANNING.** This is a reviewed
proposal set, not evidence that the proposed features have shipped.
