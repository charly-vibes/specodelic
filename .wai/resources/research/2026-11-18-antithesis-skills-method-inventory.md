# Antithesis-skills methodology inventory (2026-11-18)

Source: https://github.com/antithesishq/antithesis-skills (reviewed from a shallow clone of main, HEAD `acd775db` 2026-11-17; the skills' metadata version string `2026-11-12 6ba10e2` is per-skill, not a repo commit). Experimental repo — skills "may change shape, move into the plugin, or be removed" (README); re-verify skill locations before rerunning.
Question: what is transferable to specodelic's spec→test correspondence, linter dogfooding, and review workflows?

## What the repo is

Four agent skills + getting-started docs for Antithesis-style reliability testing:

| Skill | Core move |
| --- | --- |
| `antithesis-research` | Ensemble property discovery over a SUT → scratchbook (property catalog, per-property evidence files, relationships, evaluation synthesis) |
| `antithesis-feature-workload` | Per-feature self-driving workload; dual-mode assertions; TDD-style stubs that go green as code lands |
| `antithesis-mutation-testing` | Validates the *oracle*, not the SUT: one realistic bug per safety property; confirm the property fires; five-verdict report |
| `antithesis-review-inputs` | Read-only review of input generation for untapped state-space exploration ("findings are ceilings, not defects") |

## Transfer map to this repo

### Already converged (don't re-file)

- **Property discovery on our own laws** → `specodelic-bb98` (P1) already schedules running `antithesis-research` here.
- **Seeded-defect replay** → `specodelic-b3nv` (P3) is exactly the warrant-probe idea.
- **"Coverage is not an adequacy measure"** → authority-lattice G15 (2026-11-16/17/18 research memos) already gates demotion on mutation score / replay / differential execution.

### New and transferable

1. **Vacuous-pass framing for `ah check --run_tests`.** "A green run shows nothing bad was observed — not that the property would have noticed." Espectacular proves contract tests *pass*; nothing proves a scenario's test would *fail* on a realistic violation. → filed as specodelic ticket (oracle-validity probes).
2. **Verdict taxonomy for oracle validation** (enriches `specodelic-b3nv`). Every target ends with exactly one verdict: `falsified` / `not mutatable` / `outstanding` / `withdrawn` / `refined`; qualifiers (`attempt cap reached`, `workload gap`, `dominated`…) never upgrade the verdict. Scope: only *safety-class* properties are mutated — `Sometimes` (liveness) and `Reachable` are validated by the baseline instead, since a green `Reachable` already proves the workload drove the system there. Anti-circularity: re-keying an assertion to the mutant that validates it proves nothing — record as *falsified after refinement*. Attribution: a kill counts only when evidence ties the divergence marker to the targeted failure — never two run-wide counts, never collateral.
3. **Mutant design discipline.** One *realistic developer mistake* per target (dropped guard, flipped comparator, off-by-one — never vandalism); subtle (needs a reachability prerequisite); non-fatal; must not disable its own reporting path. Three-part patch: announcement (build fingerprint) / marker (divergence reached) / the bug. Collateral failures are *predicted* in evidence, credited only when evidence shows a distinct violation.
4. **Green-baseline gate with fingerprint.** No mutant design until a green baseline at the *current tree hash*; recorded baselines expire on tree drift. Maps to: require `just ci` + `ah check` green at a recorded commit before any seeded-defect sweep.
5. **Discovery/evaluation separation.** "Discovery agents are biased toward finding properties; evaluation agents toward finding problems with the catalog. Running both in one stage degrades evaluation into post-hoc rationalization." Fresh-context reviewer sub-agent. Synthesizer (not finders) categorizes findings as **Gap** / **Bias** / **Refinement**.
6. **Attention focuses + wildcard lens.** Fixed lenses see what their descriptions tell them to look for; the wildcard gets only *one-line summaries of covered territory* (not the others' methods), runs last, deliberately unconstrained. A reusable prompt-engineering pattern for ensemble reviews (openspec change review, linter-rule discovery).
7. **Claims are leads, not facts** (`validating-claims.md`). "If you cannot quote the discriminating detail, you have not validated the claim, and it does not enter." Claimed guarantees become properties to *test*, never stated as holding. Direct echo of incident e7bb19e (commit message claiming a test binding the diff didn't contain).
8. **Scratchbook artifact discipline.** Provenance frontmatter (`sut_path`, `commit`, `updated`, `external_references` with per-entry `why`); extend-don't-rewrite; per-property evidence files; Open Questions lifecycle: untagged → `(partial: ...)` → `(needs human input)`, each transition requiring an auditable `### Investigation Log` entry.
9. **Budget/authorization pattern.** Opening interview → `interview.md` is *the only record of authorization* (run ceiling, concurrency, what may touch the tree); resumable `status.md` keyed on tree fingerprint; append-only `history.md`. Maps to long wai pipeline runs.
10. **"Untapped exploration" review framing** (`review-inputs`). Findings are ceilings, not defects; tiers high-signal / worth-considering / observation; reports include "what's working well". Same spirit as vampiro's advisory-only stance — a better report shape for advisory findings. Its anti-pattern catalog (fixed action sets, hardcoded ranges, no per-run subsetting) is a checklist for reviewing proptest *generator strategy* quality in `spk compile` output.

### Skill-authoring craft worth copying

- Each SKILL.md ends with a **Self-Review checklist** whose every criterion is externally auditable (file exists, tag backed by log entry, run id cited).
- Reference table with "When to read" per file; references are loaded lazily.
- "DO NOT PROCEED" preconditions up top; single-agent fallback for every ensemble path.

## Non-goals / what NOT to import

- Docker/Antithesis harness machinery (n/a here).
- 30–60 min ensemble runs for single-crate work — the ensemble pattern only pays on corpus-scale questions.
- Five-verdict bureaucracy for small changes — reserve for sweep-shaped instruments.

## Disposition

- specodelic-b3nv: notes appended (verdict ladder, attribution, anti-circularity, mutant design).
- specodelic-bb98: note appended (read order: research SKILL.md → property-discovery → property-evaluation → validating-claims; scratchbook format).
- New ticket filed: oracle-validity probes for espectacular contract tests (vacuous-pass check).
