# Subagent brief: specodelic-4und — experiment E2: kernel coverage of fragments

You are a subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-4und` and is hands-off: you own the experiment end-to-end. The
orchestrator verifies the report's evidence trail — never claim measurements
you did not take.

This is a MEASUREMENT / DECISION-INPUT ticket, not a product-code ticket.
Its output decides whether `add-shared-fragment-helpers` (bd:
`specodelic-ez9a.1`) rescopes or supersedes BEFORE any implementation.

## Orientation (do this first)

- `export WAI_PROJECT=specodelic`
- Read: `bd show specodelic-4und` (acceptance + falsifiable rule);
  `openspec/specs/kernel/spec.md` (kernel grammar: three-valued status,
  acyclic/decidable claims); `openspec/specs/specodelic/spec.md` (kernel
  section, `kernel.binding` claim carrier, pure `[[a]] ∧ [[b]]` citation
  algebra — executable WITHOUT fragments); `openspec/specs/compile/spec.md`
  and `openspec/specs/model_check/spec.md` (fragment emission, executable
  invariants); `src/compile.rs` `FRAGMENT_MARKER` machinery (what a
  fragment IS today).
- `wai search "fragment"`, `wai search "kernel"` — accumulated patterns.
- Inputs (all verified present):
  - Bundle: `/var/home/sasha/cotorreo/raw/_conversations/claude/56d084d5-6e1c-407f-afba-b2fe244f101f/artifacts/specodelic_bundle_v3.tar.gz`
    (also `specodelic_bundle_v2.tar.gz`, `manifest.json`,
    `specodelic_research_report_PARTIAL_v2.md`,
    `specodelic_bowling_semantic_report.md` in the same dir).
  - Extract to `/var/tmp/4und-bundle/` (`mkdir -p`, `tar xzf`). Do NOT
    commit the bundle or its extracted tree.
- Start and execute the executor pipeline (repo contract):
  `wai pipeline start tdd-ro5 --topic="specodelic-4und: kernel coverage of fragments"`
  Advance ONLY via `wai pipeline next` — never hand-edit run YAMLs. The
  tdd-ro5 phase names do not all map onto a measurement ticket: record the
  honest mapping (research = inventory, design = re-expression method,
  red/green = measure → parity probes, report = review) in the step
  artifacts; do not fabricate red/green test cycles.
- `export TMPDIR=/var/tmp/specodelic-xch1` (mkdir -p it) — MANDATORY for
  cargo invocations.

## What to do (the experiment)

1. **Inventory** every helper in the extracted `harness/` (k04.py, fsm.py,
   k11.py, k11_rev3.py, bowl.py, k07.py, sk.py, k05.py, bowlspec.py,
   scale.py, mutA_orig.py, mutA_import.py, mutA.py, mutB*.py, mutC.py,
   rev6.py — enumerate what actually exists). For each: purpose, one-line
   classification (structural — state-set membership, reachability,
   guard/invariant shape, transition legality; vs arithmetic/data — math
   over sequences/counters; vs oracle/scoring — expected-value comparison).
   Read the PARTIAL report for the study's own framing; cite it.
2. **Re-express** every STRUCTURAL helper against the kernel grammar +
   citation algebra: write the kernel-expressible form (constraint rows /
   citation claims / model transitions in the corpus language) as fenced
   probe specs or constraint snippets in the report (scratch files under
   /var/tmp; do NOT edit product code or the corpus).
3. **Measure** (numbers, not adjectives):
   - % of non-arithmetic helpers kernel-expressible (numerator/denominator
     explicit),
   - residual fragment bytes for the remainder (what still needs a
     `**rust:**` fragment and why),
   - detection parity: for each re-expressed helper, does the kernel form
     detect the same violations the helper/oracle detects on the bundle's
     own fixtures/mutations (mutA/B/C, rev6)? Document the probe used and
     its observed result; a parity check you could not run is recorded as
     not-run with the reason — never assumed.
4. **Apply the falsifiable rule** (ticket text, verbatim): if kernel/
   citation coverage of the structural share is high (>= 70% of
   non-arithmetic helpers), the report recommends rescoping
   `add-shared-fragment-helpers` to data-helpers only or superseding it;
   else the proposal stands as written. State the measured value, the
   rule, and the resulting recommendation explicitly.
5. **Deliverable**: write the report to
   `.wai/projects/specodelic/research/2026-11-12-4und-experiment-e2-kernel-coverage-report.md`
   with: provenance (bundle path + `sha256sum` of the tar.gz, extraction
   dir), the `n=1-author` label, per-helper table, measurements, parity
   evidence, rule application, and recommendation. Commit ONLY this report
   file (plus your wai pipeline artifacts are orchestrator-recorded;
   stage only files you authored). Keep scratch work in `/var/tmp/4und-*`.

## Hard scope guard

- Allowed files: the report file above; scratch dirs under `/var/tmp/`.
- Never edit: `src/**`, `tests/**`, `openspec/**`, `specs/`,
  `.espectacular/**`, `.beads/**` (orchestrator owns bd), `pretender.toml`,
  `.wai/resources/**`, run YAMLs, the corpus.
- If a probe genuinely requires running `spk` (lint/model-check/compile),
  use the built binary (`cargo run -q -- …` or target/debug/spk) on
  scratch spec files under /var/tmp — do not add tests.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`,
  run `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **beads**: issues live in `.beads/issues.jsonl`. Do NOT claim/close/
  update the ticket — the orchestrator owns bd state for this run.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Measurements**
- inventory n=…, structural share=…, kernel-expressible=…%, residual fragment bytes=…

**Rule applied**
- rule → measured value → recommendation (rescope to data-helpers | supersede | proposal stands)

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">