# Verification improvement ticket breakdown

Source: the five reviewed and corrected OpenSpec proposals. User authorized ticket creation after the fixes. Base: 29f81ef918d556d70e52ab37ab2bcee072287666.

The numbered list is the planned breakdown. Existing umbrella tickets are reused as epics; refactoring tickets are separate per AGENTS.md even though the generic issue skill prefers merging them into feature slices. This maps reviewed task cycles, so no additional granularity approval is needed. All implementation remains undone.

1. **Python properties verify real generated inputs** (`py`)
   - Lane: REVIEW; complexity: XL; type: epic.
   - Gate: `openspec validate add-py-fragment-emission --strict`.
   - Blocked by: None.
   - Story and scope: Coordinate the linked feature and separate cleanup slices; close only after all children and final validation are complete. Do not execute this epic as one task.
   - Traceability: add-py-fragment-emission: umbrella.
   - Slice rationale: Coordinates separately executable child slices; not scheduled as an atomic task.

2. **TypeScript properties share complete verification** (`ts`)
   - Lane: REVIEW; complexity: XL; type: epic.
   - Gate: `openspec validate add-ts-fragment-emission --strict`.
   - Blocked by: None.
   - Story and scope: Coordinate the linked feature and separate cleanup slices; close only after all children and final validation are complete. Do not execute this epic as one task.
   - Traceability: add-ts-fragment-emission: umbrella.
   - Slice rationale: Coordinates separately executable child slices; not scheduled as an atomic task.

3. **Readers can inspect faithful corpus diagrams** (`graph`)
   - Lane: REVIEW; complexity: XL; type: epic.
   - Gate: `openspec validate add-graph-views --strict`.
   - Blocked by: None.
   - Story and scope: Coordinate the linked feature and separate cleanup slices; close only after all children and final validation are complete. Do not execute this epic as one task.
   - Traceability: add-graph-views: umbrella.
   - Slice rationale: Coordinates separately executable child slices; not scheduled as an atomic task.

4. **Verified means every required current claim passed** (`claims`)
   - Lane: REVIEW; complexity: XL; type: epic.
   - Gate: `openspec validate define-verification-claim-gates --strict`.
   - Blocked by: None.
   - Story and scope: Coordinate the linked feature and separate cleanup slices; close only after all children and final validation are complete. Do not execute this epic as one task.
   - Traceability: define-verification-claim-gates: umbrella.
   - Slice rationale: Coordinates separately executable child slices; not scheduled as an atomic task.

5. **Citations resolve the intended invariant in every command** (`citations`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test --test cli`.
   - Blocked by: None.
   - Story and scope: Resolve local/qualified invariant citations from same-run evidence across command output and persisted reports. Enforce ordinary unique intent IDs and isolated dual-format command scope.
   - Traceability: add-min-expr-kernel: 3.6.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

6. **Every opted-in kernel claim appears in command results** (`kernel_cli`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test --test cli`.
   - Blocked by: citations.
   - Story and scope: Run the existing kernel evaluator over the complete explicit input corpus in CLI and orchestrate; persist every claim status and expose unsupported backend claims.
   - Traceability: add-min-expr-kernel: 3.7.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

7. **Keep claim identity and command results consistent during cleanup** (`kernel_tidy`)
   - Lane: AFK; complexity: M; type: task.
   - Gate: `cargo test`.
   - Blocked by: kernel_cli.
   - Story and scope: Extract shared identity/evaluation helpers only after command fixtures pass; preserve CLI/orchestrate/report parity.
   - Traceability: add-min-expr-kernel: 3.8.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

8. **False or unknown required claims block verification** (`aggregate`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test --test cli`.
   - Blocked by: kernel_tidy.
   - Story and scope: Implement the reviewed aggregation truth table across model-check, verify and orchestrate, keeping prose explicitly unchecked.
   - Traceability: define-verification-claim-gates: 1.1, 1.2.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

9. **Keep aggregate verdicts consistent across report views** (`aggregate_tidy`)
   - Lane: AFK; complexity: M; type: task.
   - Gate: `cargo test`.
   - Blocked by: aggregate.
   - Story and scope: Consolidate aggregate rendering after the truth-table fixtures pass; do not change acceptance policy.
   - Traceability: define-verification-claim-gates: 1.3.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

10. **Verification rejects evidence from another input scope** (`freshness`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test --test cli`.
   - Blocked by: aggregate_tidy.
   - Story and scope: Bind report schema v1 to current structured content, required IDs and consumed artifact hashes; recompute live scope in verify.
   - Traceability: define-verification-claim-gates: 2.1, 2.2.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

11. **Keep freshness decisions identical across commands** (`freshness_tidy`)
   - Lane: AFK; complexity: M; type: task.
   - Gate: `cargo test`.
   - Blocked by: freshness.
   - Story and scope: Share the canonical scope encoder after stale-evidence tests pass; preserve schema encoding and report compatibility.
   - Traceability: define-verification-claim-gates: 2.3.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

12. **Users see the same claim blockers in every report** (`assurance`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test --test cli`.
   - Blocked by: freshness_tidy.
   - Story and scope: Show evaluated/unchecked/blocking claims consistently in human, JSON and saved output; align version/capability docs to release metadata.
   - Traceability: define-verification-claim-gates: 3.1, 3.2.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

13. **Keep assurance guidance consistent after documentation cleanup** (`assurance_tidy`)
   - Lane: AFK; complexity: M; type: task.
   - Gate: `just ci`.
   - Blocked by: assurance.
   - Story and scope: Remove stale repeated assurance claims after release documentation checks pass; retain canonical references.
   - Traceability: define-verification-claim-gates: 3.3.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

14. **Kernel fixture outcomes are checked on every CI run** (`agreement`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test --test kernel_grounding --test kernel_status --test ci_wiring && just ci`.
   - Blocked by: kernel_tidy.
   - Story and scope: Execute finite acset fixtures in Rust against explicit expected statuses now; leave an activation contract for Python parity in l8l.
   - Traceability: add-min-expr-kernel: 4.1, 4.2.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

15. **Keep shared kernel fixtures stable for the Python follow-up** (`agreement_tidy`)
   - Lane: AFK; complexity: M; type: task.
   - Gate: `cargo test`.
   - Blocked by: agreement.
   - Story and scope: Move agreement fixtures to a shared test module and document named-case law promotion for Python without weakening the Rust interim gate.
   - Traceability: add-min-expr-kernel: 4.3.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

16. **External checker claims preserve opaque binding text** (`binding`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test`.
   - Blocked by: kernel_tidy.
   - Story and scope: Extract kernel.binding on invariant constraints verbatim and expose it through the established external claim carrier; no registry or executable interpretation.
   - Traceability: add-min-expr-kernel: 5.1, 5.2.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

17. **Keep external claim guidance on the established binding path** (`binding_tidy`)
   - Lane: AFK; complexity: M; type: task.
   - Gate: `just ci`.
   - Blocked by: binding.
   - Story and scope: Place binding guidance beside existing espectacular documentation, preserving the new extraction behavior and read-only sibling boundaries.
   - Traceability: add-min-expr-kernel: 5.3.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

18. **Quick-start kernel examples produce expected claim evidence** (`usage`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `just lint-doc-examples && just lint-specs && cargo test --test cli`.
   - Blocked by: assurance_tidy, agreement_tidy, binding_tidy.
   - Story and scope: Migrate USAGE fenced examples one at a time and execute extracted fixtures through compile/model-check with the expected claims.
   - Traceability: add-min-expr-kernel: 6.1.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

19. **Core format invariants execute as declared kernel claims** (`corpus`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `just lint-specs && cargo test --test cli`.
   - Blocked by: usage.
   - Story and scope: Migrate eligible specodelic.md equational and bounded-quantified invariants per-file, with fresh report evidence and revision discipline.
   - Traceability: add-min-expr-kernel: 6.2.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

20. **Keep migrated corpus claims lint-clean without new exemptions** (`corpus_tidy`)
   - Lane: AFK; complexity: M; type: task.
   - Gate: `just lint-specs && just lint-doc-examples && cargo test --test cli`.
   - Blocked by: corpus.
   - Story and scope: Review and tidy migrated corpus wording; record why no new advisory class is required. Preserve every executable claim and evidence fixture.
   - Traceability: add-min-expr-kernel: 6.3.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

21. **Kernel capabilities retain complete contracts when archived** (`kernel_close`)
   - Lane: AFK; complexity: L; type: chore.
   - Gate: `just ci && ah check && openspec validate add-min-expr-kernel --strict`.
   - Blocked by: corpus_tidy, agreement_tidy, binding_tidy.
   - Story and scope: Complete revision/artifact synchronization and genuine scenario contracts, preserve full capability snapshots, then archive with the dual-format recipe. Bind contracts alongside each owning feature, not fabricated at closure.
   - Traceability: add-min-expr-kernel: 7.1, 7.2, 7.3, 7.4.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

22. **Strict verification policy ships with current contracts and guidance** (`claims_close`)
   - Lane: AFK; complexity: L; type: chore.
   - Gate: `just ci && ah check --changes define-verification-claim-gates && openspec validate define-verification-claim-gates --strict`.
   - Blocked by: assurance_tidy, kernel_close.
   - Story and scope: Synchronize domain revisions/artifacts, bind every new scenario to real tests and archive the additive capability after kernel archive.
   - Traceability: define-verification-claim-gates: 4.1, 4.2, 4.3.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

23. **Marked generators give Rust predicates real typed values** (`generators`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test`.
   - Blocked by: claims_close, agreement_tidy.
   - Story and scope: Parse the bounded gen grammar and explicit named registry, infer recursive types, and emit real Rust strategies without changing unmarked legacy bytes.
   - Traceability: add-py-fragment-emission: 1.1, 1.2.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

24. **Keep generator metadata identical across emitters** (`generators_tidy`)
   - Lane: AFK; complexity: M; type: task.
   - Gate: `cargo test`.
   - Blocked by: generators.
   - Story and scope: Extract shared typed metadata after Rust domain/compatibility fixtures pass; do not add vocabulary.
   - Traceability: add-py-fragment-emission: 1.3.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

25. **Python unit properties pass or report a shrunk failure** (`python_run`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test --test cli`.
   - Blocked by: generators_tidy.
   - Story and scope: Emit deterministic Python artifacts and a manifest, execute pytest/Hypothesis through PropertiesRunner and attribute results to exact property IDs.
   - Traceability: add-py-fragment-emission: 2.1, 2.2.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

26. **Keep Rust and Python artifact emission consistent** (`python_tidy`)
   - Lane: AFK; complexity: M; type: task.
   - Gate: `cargo test`.
   - Blocked by: python_run.
   - Story and scope: Consolidate emitter helpers after both language fixtures pass, preserving manifest and artifact determinism.
   - Traceability: add-py-fragment-emission: 2.3.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

27. **Partial or inadequate property execution cannot verify** (`property_gate`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test --test cli`.
   - Blocked by: python_tidy.
   - Story and scope: Enforce complete mixed-language dispatch, typed-input adequacy and registry/artifact freshness; bound startup/tests and terminate descendant processes on timeout.
   - Traceability: add-py-fragment-emission: 3.1, 3.2.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

28. **Keep property failures identical across reporting paths** (`property_tidy`)
   - Lane: AFK; complexity: M; type: task.
   - Gate: `cargo test`.
   - Blocked by: property_gate.
   - Story and scope: Share all-block reporting without changing adequacy, timeout or freshness outcomes.
   - Traceability: add-py-fragment-emission: 3.3.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

29. **Rust and Python agree on every declared kernel fixture** (`python_agreement`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `just ci`.
   - Blocked by: generators_tidy, agreement_tidy.
   - Story and scope: Add the Python reference evaluator for the closed kernel test vocabulary and activate mandatory parity in CI; no production backend selector.
   - Traceability: add-py-fragment-emission: 4.1, 4.2.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

30. **Keep backend agreement fixtures shared without skipped cases** (`python_agreement_tidy`)
   - Lane: AFK; complexity: M; type: task.
   - Gate: `cargo test`.
   - Blocked by: python_agreement.
   - Story and scope: Remove duplicated fixture plumbing after Rust/Python parity passes; preserve mandatory coverage and named-case promotion path.
   - Traceability: add-py-fragment-emission: 4.3.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

31. **A runnable example distinguishes spec checks from application tests** (`walkthrough`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `just ci && ah check --run-tests`.
   - Blocked by: property_tidy, python_agreement_tidy.
   - Story and scope: Ship the resumable-job walkthrough from lint through model claims and generated Python properties to an existing external application-test binding.
   - Traceability: add-py-fragment-emission: 5.1.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

32. **Python verification retains legacy contracts and migration guidance** (`python_close`)
   - Lane: AFK; complexity: L; type: chore.
   - Gate: `just ci && ah check --changes add-py-fragment-emission && openspec validate add-py-fragment-emission --strict`.
   - Blocked by: walkthrough.
   - Story and scope: Rebase compile snapshot after kernel deployment, publish marked/legacy generator migration, synchronize revision literals and bind real scenarios before dual-format archive.
   - Traceability: add-py-fragment-emission: 5.2, 5.3, 5.4.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

33. **TypeScript predicates receive the shared declared value domains** (`typescript_emit`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test`.
   - Blocked by: python_close.
   - Story and scope: Emit deterministic fast-check artifacts using shared recursive types, signed 32-bit integers and Unicode scalar lengths.
   - Traceability: add-ts-fragment-emission: 1.1, 1.2.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

34. **Keep TypeScript property manifests aligned with other languages** (`typescript_emit_tidy`)
   - Lane: AFK; complexity: M; type: task.
   - Gate: `cargo test`.
   - Blocked by: typescript_emit.
   - Story and scope: Reuse shared manifest helpers with identical generated artifacts and domain fixtures.
   - Traceability: add-ts-fragment-emission: 1.3.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

35. **Mixed Rust Python and TypeScript runs verify only when complete** (`typescript_run`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test --test cli`.
   - Blocked by: typescript_emit_tidy.
   - Story and scope: Run installed project-local Vitest/fast-check; collect exact block results and native shrink metadata with freshness and process-tree timeout handling.
   - Traceability: add-ts-fragment-emission: 2.1, 2.2.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

36. **Keep mixed-language execution outcomes consistent during cleanup** (`typescript_run_tidy`)
   - Lane: AFK; complexity: M; type: task.
   - Gate: `cargo test`.
   - Blocked by: typescript_run.
   - Story and scope: Remove duplicate runner plumbing while preserving project resolution, all-block accounting and failure attribution.
   - Traceability: add-ts-fragment-emission: 2.3.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

37. **TypeScript support preserves the deployed Python and kernel contracts** (`typescript_close`)
   - Lane: AFK; complexity: L; type: chore.
   - Gate: `just ci && ah check --changes add-ts-fragment-emission && openspec validate add-ts-fragment-emission --strict`.
   - Blocked by: typescript_run_tidy.
   - Story and scope: Run shared conformance, rebase full compile snapshot after Python archive, update revision/docs and genuine scenario bindings, then use dual-format archive.
   - Traceability: add-ts-fragment-emission: 3.1, 3.2, 3.3.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

38. **Raw graph edges preserve every node and violation** (`edges`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test --test cli`.
   - Blocked by: None.
   - Story and scope: Expose deterministic six-column TSV preserving qualified row IDs, duplicate edge instances and every violation; document escaping and output precedence. Own raw-projection portion of 1.7.
   - Traceability: add-graph-views: 1.1, 1.2, 1.3, 1.7.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

39. **Keep edge projections byte-stable after formatting cleanup** (`edges_tidy`)
   - Lane: AFK; complexity: M; type: task.
   - Gate: `just ci`.
   - Blocked by: edges.
   - Story and scope: Extract projection formatting after its CLI fixtures pass; preserve six columns, duplicate multiplicity and annotations.
   - Traceability: add-graph-views: 1.4.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

40. **Users can pipe graph projections into DOT and Mermaid renderers** (`native_views`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test --test cli`.
   - Blocked by: edges_tidy.
   - Story and scope: Emit native dot/mermaid string projections with the specified visual grammar and deterministic output.
   - Traceability: add-graph-views: 1.5.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

41. **Wiring diagrams expose declared producer-consumer connections** (`wiring`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test --test cli`.
   - Blocked by: native_views.
   - Story and scope: Project constraints.satisfies to file-level wiring, drop self-loops and label missing wiring.
   - Traceability: add-graph-views: 1.6.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

42. **Schema diagrams render the canonical typing rules without duplication** (`schema_view`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test --test cli && python3 -m unittest discover -s scripts -p test_graph_views.py`.
   - Blocked by: None.
   - Story and scope: Deliver guide value-set JSON and the canonical Schema JSON export through the Python schema diagram. Own schema portions of 2.3; use production exporter for constructed perturbation fixtures.
   - Traceability: add-graph-views: 2.1, 2.2, 2.3, 2.6.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

43. **State diagrams preserve distinct states and show guarded transitions** (`state_view`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `cargo test --test cli && python3 -m unittest discover -s scripts -p test_graph_views.py`.
   - Blocked by: edges_tidy, schema_view.
   - Story and scope: Render per-file state diagrams from graph artifacts only. Own rendering portion of 1.7 and corpus/scope portion of 2.3; validate script scope before output.
   - Traceability: add-graph-views: 1.7, 2.3, 2.4.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

44. **Traceability diagrams show file dependencies and distinct-source fan-in** (`trace_view`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `python3 -m unittest discover -s scripts -p test_graph_views.py`.
   - Blocked by: state_view.
   - Story and scope: Collapse only file-level projections to owning intents and annotate fan-in by distinct source intents.
   - Traceability: add-graph-views: 2.5.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

45. **Keep derived diagrams unchanged when prose or renderer structure changes** (`views_tidy`)
   - Lane: AFK; complexity: M; type: task.
   - Gate: `python3 -m unittest discover -s scripts -p test_graph_views.py`.
   - Blocked by: trace_view, wiring.
   - Story and scope: Share rendering helpers in a separate refactor; pin prose-only perturbation to identical diagram output.
   - Traceability: add-graph-views: 2.7.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

46. **Readers can rebuild diagrams and follow embedded render recipes** (`graph_docs`)
   - Lane: AFK; complexity: L; type: feature.
   - Gate: `just docs-graphs && just ci`.
   - Blocked by: views_tidy.
   - Story and scope: Wire docs-graphs to docs build using one binary for exports, untracked intermediate/generated files and an appended graph-views primer.
   - Traceability: add-graph-views: 3.1, 3.2, 3.3, 3.4.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

47. **Graph views demonstrate faithful output on real corpora** (`graph_close`)
   - Lane: REVIEW; complexity: L; type: chore.
   - Gate: `just ci && openspec validate add-graph-views --strict`.
   - Blocked by: graph_docs.
   - Story and scope: Exercise this repo and read-only sibling bajan corpus, record real wiring/violation behavior, verify existing bajan-ac8 rather than filing a duplicate, then complete graph proposal gates.
   - Traceability: add-graph-views: 4.1, 4.2, 4.3, 4.4.
   - Slice rationale: Demonstrates the named outcome with the specified input/output fixtures; cleanup preserves the already passing outcome in a separate commit.

## Published ticket index

40 new tickets created; 7 existing work records reused and decomposed;
one additional existing CI ticket received its missing blocker.

| Ticket | Outcome | Direct blockers |
|--------|---------|-----------------|
| `specodelic-l8l` | Python properties verify real generated inputs | specodelic-68m.7, specodelic-36t |
| `specodelic-aby` | TypeScript properties share complete verification | specodelic-l8l |
| `specodelic-gre` | Readers can inspect faithful corpus diagrams | None |
| `specodelic-68m` | Verified means every required current claim passed | specodelic-k3h |
| `specodelic-hb4` | Citations resolve the intended invariant in every command | None |
| `specodelic-mui` | Every opted-in kernel claim appears in command results | specodelic-hb4 |
| `specodelic-k3h` | Keep claim identity and command results consistent during cleanup | specodelic-mui |
| `specodelic-68m.1` | False or unknown required claims block verification | specodelic-k3h |
| `specodelic-68m.2` | Keep aggregate verdicts consistent across report views | specodelic-68m.1 |
| `specodelic-68m.3` | Verification rejects evidence from another input scope | specodelic-68m.2 |
| `specodelic-68m.4` | Keep freshness decisions identical across commands | specodelic-68m.3 |
| `specodelic-68m.5` | Users see the same claim blockers in every report | specodelic-68m.4 |
| `specodelic-68m.6` | Keep assurance guidance consistent after documentation cleanup | specodelic-68m.5 |
| `specodelic-36n` | Kernel fixture outcomes are checked on every CI run | specodelic-k3h |
| `specodelic-36t` | Keep shared kernel fixtures stable for the Python follow-up | specodelic-36n |
| `specodelic-bf5` | External checker claims preserve opaque binding text | specodelic-k3h |
| `specodelic-ats` | Keep external claim guidance on the established binding path | specodelic-bf5 |
| `specodelic-gch` | Quick-start kernel examples produce expected claim evidence | specodelic-68m.6, specodelic-36t, specodelic-ats |
| `specodelic-7yk` | Core format invariants execute as declared kernel claims | specodelic-gch |
| `specodelic-5c4` | Keep migrated corpus claims lint-clean without new exemptions | specodelic-7yk |
| `specodelic-bxk` | Kernel capabilities retain complete contracts when archived | specodelic-5c4, specodelic-36t, specodelic-ats |
| `specodelic-68m.7` | Strict verification policy ships with current contracts and guidance | specodelic-68m.6, specodelic-bxk |
| `specodelic-l8l.1` | Marked generators give Rust predicates real typed values | specodelic-68m.7, specodelic-36t |
| `specodelic-l8l.2` | Keep generator metadata identical across emitters | specodelic-l8l.1 |
| `specodelic-l8l.3` | Python unit properties pass or report a shrunk failure | specodelic-l8l.2 |
| `specodelic-l8l.4` | Keep Rust and Python artifact emission consistent | specodelic-l8l.3 |
| `specodelic-l8l.5` | Partial or inadequate property execution cannot verify | specodelic-l8l.4 |
| `specodelic-l8l.6` | Keep property failures identical across reporting paths | specodelic-l8l.5 |
| `specodelic-l8l.7` | Rust and Python agree on every declared kernel fixture | specodelic-l8l.2, specodelic-36t |
| `specodelic-l8l.8` | Keep backend agreement fixtures shared without skipped cases | specodelic-l8l.7 |
| `specodelic-l8l.9` | A runnable example distinguishes spec checks from application tests | specodelic-l8l.6, specodelic-l8l.8 |
| `specodelic-l8l.10` | Python verification retains legacy contracts and migration guidance | specodelic-l8l.9 |
| `specodelic-aby.1` | TypeScript predicates receive the shared declared value domains | specodelic-l8l.10 |
| `specodelic-aby.2` | Keep TypeScript property manifests aligned with other languages | specodelic-aby.1 |
| `specodelic-aby.3` | Mixed Rust Python and TypeScript runs verify only when complete | specodelic-aby.2 |
| `specodelic-aby.4` | Keep mixed-language execution outcomes consistent during cleanup | specodelic-aby.3 |
| `specodelic-aby.5` | TypeScript support preserves the deployed Python and kernel contracts | specodelic-aby.4 |
| `specodelic-gre.1` | Raw graph edges preserve every node and violation | None |
| `specodelic-gre.2` | Keep edge projections byte-stable after formatting cleanup | specodelic-gre.1 |
| `specodelic-gre.3` | Users can pipe graph projections into DOT and Mermaid renderers | specodelic-gre.2 |
| `specodelic-gre.4` | Wiring diagrams expose declared producer-consumer connections | specodelic-gre.3 |
| `specodelic-gre.5` | Schema diagrams render the canonical typing rules without duplication | None |
| `specodelic-gre.6` | State diagrams preserve distinct states and show guarded transitions | specodelic-gre.2, specodelic-gre.5 |
| `specodelic-gre.7` | Traceability diagrams show file dependencies and distinct-source fan-in | specodelic-gre.6 |
| `specodelic-gre.8` | Keep derived diagrams unchanged when prose or renderer structure changes | specodelic-gre.7, specodelic-gre.4 |
| `specodelic-gre.9` | Readers can rebuild diagrams and follow embedded render recipes | specodelic-gre.8 |
| `specodelic-gre.10` | Graph views demonstrate faithful output on real corpora | specodelic-gre.9 |

The preserved TypeScript epic dependency on the Python epic is intentional.
Policy implementation precedes corpus migration; policy archival follows
kernel archival. This separates those milestones without a dependency cycle.

## Completion ledger — finalized 2026-10-07

Changed — created 40 tickets; reused seven existing records (three became
coordination epics; four kernel tickets were narrowed to their owned tasks).
Added the missing Python closure blocker to existing CI ticket specodelic-lf3.
Fifteen cleanup tickets remain separate from feature work. All 75 unchecked
proposal tasks have owners; graph tasks 1.7 and 2.3 explicitly split their
raw/schema versus rendering fixture responsibilities.

Dependencies — 54 new blocking links and 32 new parent-child links.
The complete graph has no cycles. All ticket metadata records concrete file
paths, the base commit, lane, complexity, plan tasks and meter command.

Verified — all 47 mapped issue records passed the metadata/body/dependency
checks and bd lint with zero template warnings. OpenSpec strict validation:
25 passed; section synchronization and git diff --check passed. ah check:
zero structural issues; execution skipped. No runtime code was changed.

Review — checked task coverage, duplicate existing work, dependency order,
separate feature/cleanup ownership, file-header criteria and anti-goals.
No closed implementation history was reopened. Implementation checkboxes
remain unchecked and no feature is claimed shipped.

Persistence — tracked .beads/issues.jsonl contains 165 records. Exported and
merged only the 48 records created/updated during ticketing; unrelated
tracked records were preserved byte-for-byte. No tracker configuration,
remote synchronization, hook ownership, push or deployment changed.

Risks — file metadata exposes overlapping paths; do not execute overlapping
ready slices concurrently without isolation. New module/test paths are
proposed paths, not claims that those files already exist. The graph epic
may appear ready, but it is coordination-only; claim an executable child.

Next — ready implementation starts are specodelic-hb4 (citation resolution),
specodelic-gre.1 (raw graph edges), and specodelic-gre.5 (schema diagrams).
No additional approval step was inserted for ticket creation.
