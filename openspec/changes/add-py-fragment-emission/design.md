# Design: a first non-Rust emitter and real generator semantics

## Test shape first

Compile fixtures use integer endpoints, empty/nonempty strings and lists,
nested choices, explicit singleton domains, named aliases, malformed input,
missing/cyclic names, a legacy generator, and mixed Rust/Python predicates.
Assert artifact determinism and the declared value domain before runner
implementation. CLI tests then execute passing/failing predicates, missing
Python/dependencies, timeout, stale registry, and partial-language failure.
Failures identify property ID/case and a native-shrunk input, with an explicit
unshrunk flag when shrinking did not finish. Never assert random engines
produce identical samples or identical minimal counterexamples.

## D1 — Explicit data grammar

A generator cell opts in only at fragment position with **gen:** followed by
one expression. V0 grammar:

    G := int(lo,hi) | string(min,max) | list(G,min,max)
       | one_of(G,G,...) | named(identifier)

Integer endpoints are inclusive signed 32-bit values. String/list lengths
are inclusive integers 0..1024; strings contain Unicode scalar values
(excluding surrogate code points). Bounds must be ordered; one_of has at
least two branches. Whitespace is insignificant; identifier matches
[A-Za-z_][A-Za-z0-9_]*. Empty lengths and equal integer bounds are deliberate
singleton domains, not inadequate placeholders. One expression binds v0;
list supports collection inputs. Additional argument-product syntax is
out of scope for v0. Marked malformed cells fail labeled; unmarked legacy
cells preserve prior emitted bytes. Mid-span mentions remain prose.

Named definitions live in .specodelic/generators.toml under the explicit
--project-root (default current working directory), with string values under
[generators] using the same G grammar. Resolve exact names, reject cycles,
missing names and expansion beyond 32 levels; never search parent folders
implicitly. These are named project data definitions, not a registry of
runners or executable imports. General callable generators remain a later
extension; do not approximate unknown names with Just(name).

### Generator types and predicate inputs

Infer T := Int | String | List(T) after named expansion. int yields Int,
string yields String, list(G,...) yields List(type(G)), and named keeps
its expansion's type. Every one_of branch must have exactly the same
recursive type; reject mismatch before emitting any artifact, with the
property ID, branch types and a remediation hint. There is no coercion,
tagged union or heterogeneous list in v0. Equal zero lengths do not erase
element types: list(int(0,1),0,0) is still List(Int).

Predicate v0 receives owned i32 / String / Vec<T> in Rust, int / str /
list[T] in Python, and number / string / Array<T> in TypeScript, recursively.
Bounds and Unicode scalar rules apply regardless of representation.
Conformance fixtures cover homogeneous choices, direct Int/String mismatch,
List(Int)/List(String) mismatch, aliases hiding a mismatch, and empty lists.
The compiler rejects all mixed-type cases consistently before dispatching
any language emitter.

## D2 — Shared IR, explicit adequacy

Rust and Python strategies consume one typed generator IR. Pin integer,
Unicode and collection domains with deterministic boundary fixtures and
membership tests; statistical diversity alone is not proof of adequacy.
Modern generator metadata records its canonical IR and registry dependency
fingerprint. Keep legacy emitted artifact bytes stable; the runner derives
legacy-placeholder status from the live spec and reports it separately.
Byte preservation covers unmarked legacy generators. An older compiler
already accepts **gen:** int(0,1) as a constant string scaffold; the new
opt-in intentionally changes that artifact and its v0 type. Migration:
update predicates to the declared type, recompile, rerun model-check and
verify. Remove the marker only to keep legacy scaffold bytes; that does
not satisfy the new adequacy gate. Malformed marked cells now fail labeled.
Legacy named-string scaffolds cannot produce properties_pass under the new
policy. Explicit singleton G domains and parameterless unit tests can pass.
Prose predicates still fail honestly. No claim of exhaustive generated-input
coverage is made. This change needs a documented verification migration.

## D3 — Per-language artifacts, complete execution

One deterministic artifact per present language: existing *_props.rs and
new *_props.py; a manifest records property ID, optional case, language,
function identity, source predicate, generator IR or legacy status, and
input fingerprints. Emit Python predicates as expressions over v0; reject
statement blocks and unsupported fragment positions before writing output.
Invariant **py:** expressions still fail labeled: this is a Properties
emitter, not a new executable application-state model checker. Preserve
law-kind fragment rejection. Untagged legacy law scaffolds retain prior
shape and cannot silently count as executable cases.

The dispatcher executes every expected artifact/row through PropertiesRunner;
missing languages, skipped blocks, duplicate IDs, failed tests, or inadequate
inputs block properties_pass. No language's successful result hides another's
failure. Verify staleness compares predicates, generator metadata, registry
content, artifacts and the claim report's scope. Editing a generated body
alone is not an approved source change; keep the existing translation path
explicitly documented instead of claiming provenance it cannot establish.

## D4 — Runner boundary

Use an explicit Python executable (--python, default python3) and invoke
pytest with Hypothesis in a per-run scratch directory, project root as the
known working context. Do not install dependencies automatically. Missing
pytest/Hypothesis is a labeled runner-unavailable result with install guidance.
Use structured pytest results keyed by generated IDs, not stdout substring
heuristics. The wall-clock budget covers startup and tests, with process-tree
cleanup so timeout cannot leave tests running. Native shrinking must finish
or the report labels its example unshrunk. Fragments remain trusted project
code under the invoking user's permissions, not sandboxed expressions.
Espectacular runner configuration is not reused for generated-scaffold policy.

## D5 — Backend agreement and limits

Existing 36n fixtures define expected statuses over finite acset snapshots.
Ship a Python reference evaluation adapter for that same closed kernel
vocabulary with parity tests for each atomic, Kleene composition, bounds,
empty domains, missing references and cycles. It is an agreement-test
backend, not a new production --backend selector or a general Rust-to-Python
translator. Activate the CI agreement path when this change lands; an absent
Python dependency in that CI job fails setup rather than skips the property.
Keep this work a separately testable feature slice inside the same change.

#### Promotion contract: shared fixtures → `law_requires_cases` (deferred of record — add-min-expr-kernel D2, tasks.md §4.3)

The fixture corpus is the promotion's frozen input; this is the contract
the l8l change picks up:

- **Single home**: the corpus lives in `tests/common/kernel_fixtures.rs`
  (agreement fixtures + backend registry, specodelic-36n) and
  `tests/common/kernel_corpora.rs` (grounding/status corpora,
  specodelic-36t). No per-test copies: tests import, never restate.
- **Trigger**: when this change's py reference adapter (D5) activates the
  agreement path, the l8l change promotes the agreement property from a
  CI test to a `law_requires_cases`-shaped Property row — the law
  requires its cases: every fixture cell (each v0 atomic, Kleene
  composition, bounds, empty domains, missing references, cycles) stays
  an explicit named case.
- **Shrink-only**: promotion adds or refines cases; it never drops,
  skips, or comments out an existing one. The `expected` statuses
  migrate verbatim into the row's case table; py expectations are added
  to the same fixture table when the emitter activates, and the
  cross-backend assertion covers them with no edits (the assertion is
  live, never commented out).
- **Mechanism**: promotion goes through the normal openspec change flow
  (l8l delta spec, dual-format archive recipe); espectacular stays
  read-only over `openspec/`, and per-scenario contract TOMLs are
  authored in the archive commit, never mid-change.

## D6 — Concrete user example

Use a small resumable-job fixture: lint its references, run declared model
claims, execute generated Python properties, and bind an existing application
test through the established espectacular contract path. Deliberately break
one application behavior and demonstrate its bound test fails. Explain which
stage catches which error; do not claim a passing spec proves arbitrary
application correctness. Record setup steps and expected diagnostics, not an
unmeasured productivity claim. Keep specs/ enforcement owned by specodelic.
