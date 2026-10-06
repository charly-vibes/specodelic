# Design: language-neutral property binding

## Context

The originating discussion mapped five Rust coupling points for the
Properties leg. The Rule-of-5 review (converged Stage 5; CRITICAL/HIGH
findings verified through the TypeSafe pass, measured FP rate 0%; three
findings flagged REVIEW_REQUIRED at low Jev confidence — D1, D3, D4
below restate the directly-verified evidence) corrected the scope:

| Layer | Owner | Status |
|---|---|---|
| Fragment grammar (`**lang:**` tag), generator vocabulary, artifact emission | specodelic (spec-format) | owns the format; espectacular stays read-only over it |
| Language-neutral runner binding, capability detection, per-scenario contract tests | espectacular — **already built** (0.9.2) | `[[tests.cargo]]` ×157, `[[tests.shell]]` ×3, `[runners]` config, doctor pytest/property capabilities |
| Corpus self-verification (`CargoRunner`, scratch crate) | specodelic, unchanged | `specs/` enforcement is specodelic's alone |

Grounding evidence (verified against the live workspace):
`.espectacular/config.toml` `[runners] cargo = ["cargo", "test"]`;
contract runner blocks surveyed with `rg -o '^\[\[tests\.[a-z]+\]\]'`;
`ah doctor` output recommending `enable_capability` for `pytest`
("detected via environment") and `property` ("detected via manifest");
`ah explain scenario-scoped-tests` documenting pytest node-id / `-k` and
vitest `--testNamePattern` binding forms.

## Decisions

### D1 — Migration fork decided: binding moves, compilation stays (EDGE-001)

"Move into espectacular/ah" resolves to **(a)** property-derived
contracts binding to non-cargo property tests — permitted, consistent
with shipped capabilities. Reading **(b)** — moving `**rust:**` fragment
compilation or any `specs/`-corpus enforcement into `ah` — is rejected:
AGENTS.md sibling-tool constraints make espectacular read-only over
`openspec/` and bar it from enforcing anything against the `specs/`
domain corpus. (Jev confidence 0.66 < 0.8 gate → REVIEW_REQUIRED; the
underlying AGENTS.md text is directly quoted above and was read from
file — treating as settled.)

### D2 — Tag set is closed and per-cell; no file-level language default

The tag set is `{rust, py, ts}` — exactly the languages where a PBT
library with native shrinking exists in the intended consumer set — and
is declared closed (`fragment_language_closed`), mirroring
`property_kind_closed` discipline. Language selection stays **per-cell**
(the marker already is): a file's artifact language is a compile-time
*consequence* of the tags present, cross-tool binding is a contract-level
`flags` choice, and a frontmatter default would introduce a second
selection mechanism with no demonstrated need. Revisit if a corpus
actually mixes tags densely.

### D3 — No parallel registry; the two registries serve different layers (CORR-002)

specodelic keeps `PropertiesRunner` with `CargoRunner` as the only
adapter; per-language adapters arrive *with* their emitters (one
follow-up per language). Espectacular's `[runners]` config keeps
governing contract execution. The layers are: **executing generated
scaffolds** (specodelic) vs **binding existing tests to scenarios**
(espectacular). They must not be merged or cross-wired — a shared
registry would couple scaffold execution policy to contract-binding
policy that have different wall-clock, caching, and toolchain stories.
(Jev 0.60 → REVIEW_REQUIRED; config file and doctor output read
directly — treating as settled.)

### D4 — Shrinking contract per language

`failure_reports_shrunk_counterexample` holds for every language:
hypothesis and fast-check shrink natively, so the floor is "the
language's native PBT shrinker"; a framework without shrinking is out of
the tag set until it grows one, and an unshrunk report (e.g. shrink
budget exhausted) must be *flagged as unshrunk* in the failure text —
never presented as minimal. Binding-level (espectacular) tests inherit
their framework's contract; nothing new is specified here.

### D5 — Generator vocabulary deferred, direction recorded

The hybrid option: a small language-neutral core vocabulary
(`int(range)`, `string`, `list(...)`, `one_of(...)`) declared in-format,
plus named domain generators resolved per project. Deferred because the
exemplars in this change bind hand-written tests (no scaffold, no
generator problem), and multiplying vacuous `Just(name)` generators
across languages — today's state — would be worse than not generalizing.
The follow-up that lands a first emitter (`py`) must land the vocabulary
decision in the same change, not before it and not after.

### D6 — Boundary line drawn at tracing vs semantics (EDGE-002)

Espectacular's `property-untraced` finding is bookkeeping (a Properties
row no scenario traces) and may grow similar tracing findings. It must
NOT grow semantic findings — e.g. a hypothetical `property-uncompiled`
would enforce compile.md's semantics from `ah`, crossing the
sibling-tool boundary. Restated in the docs task so the line survives
tool churn.

### D7 — Unknown tag is a labeled failure, never silence (sd1 discipline)

Extending the format's standing rule (malformed fragments and
mispositioned markers are labeled extraction failures, never silently
ignored): a tag outside the closed set (e.g. `**go:**`) fails extraction
naming the tag and the closed set; a fragment on a tag whose emitter
does not exist yet (`py`, `ts`) fails with a remediation hint naming the
follow-up. A mid-span occurrence remains a mention — the
defining-rows-mention rule carries over per tag verbatim.

## Risks

- **Grammar widening races the lf3 migration** on the same predicate
  cells — mitigated by sequencing (churn, not correctness).
- **Exemplar contract drift**: pytest selection by node id is exact;
  `-k` expressions are substring-y and can over-match — prefer node ids
  per `ah explain scenario-scoped-tests`.
- **`hypothesis` availability** in this environment is asserted by
  doctor's manifest detection but unproven here — task 3 verifies before
  authoring; if absent, the exemplar falls back to `[[tests.shell]]`
  with a pinned `uv run` invocation, which the escape hatch already
  supports.