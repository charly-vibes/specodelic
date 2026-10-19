# Subagent brief: specodelic-s8fg — task 6: validate-gate retirement (pin was only for the dual shape)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-s8fg` (already in_progress) and is hands-off: you own the
implementation end-to-end for THIS run's slice — **tasks.md section 6 only
(6.1–6.2)**. The orchestrator will verify with real gates — never claim work
the diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=domain-specific-extensions`
- Read, in order:
  - `openspec/changes/add-delta-layer-sidecar/tasks.md` section 6 (your
    scope; sections 1–5 are landed — do NOT redo them; 7 is NOT yours).
  - `openspec/changes/add-delta-layer-sidecar/design.md` — Decision 4 (the
    pin's job, minus the pin: the fixture-proved risk is now a
    validation-time concern 1.x enforces on every plain delta directly) and
    the rollout order (your task is rollout step 4).
  - `scripts/spec_validate.py` — the version-guarded gate you are
    COLLAPSING: `PINNED_VERSION = "0.19.0"`, direct-command-when-0.19.x /
    npx-fallback-otherwise, header docstring explains the dual-format-era
    rationale (specodelic-uv3v / GH-20).
  - `scripts/test_spec_validate.py` — the gate's python tests (TDD target:
    they must name 1.x as the validator FIRST after your change).
  - `.github/workflows/ci.yml` lines ~48–49: `npm install -g
    @fission-ai/openspec@0.19.0` — bump to the pinned 1.x version.
  - `openspec/decisions/2026-10-19-openspec-validate-pin.md` — the decision
    record you are superseding; match its format.
  - `wai search "validate pin"` — accumulated patterns.
- What 1.x version to pin: the corpus validated green under
  `@fission-ai/openspec@1.14.2` (task 5's fixture gate uses it via npx —
  see `scripts/test_os1x_shape_fixture.py`). Pin THAT exact version in
  ci.yml and in the collapsed script; record it in the decision record.

## What to build (tasks.md 6.1–6.2, TDD: RED first)

### 6.1 — Collapse the gate to a plain 1.x invocation

- RED: `scripts/test_spec_validate.py` — the gate test names 1.x as the
  validator first: assert the script invokes `openspec validate --all
  --strict` with NO version guard, NO npx fallback (a plain, single
  invocation; dev-machine `openspec` must BE the pinned 1.x — that is the
  contract the dev-machine install requirement documents). The old
  version-guard/fallback tests flip to retirement semantics or are
  replaced — the version-sniffing behavior no longer exists, so tests that
  asserted the fallback must now assert its absence.
- GREEN: collapse `scripts/spec_validate.py` to a plain 1.x invocation:
  no version sniffing, no npx fallback, no PINNED_VERSION constant unless
  it is still needed to DOCUMENT the expected version (prefer a docstring
  note over a runtime constant). Exit code stays the validator's exit
  code (CI safe). Keep the `--no-interactive` flag if 1.x accepts it.
- justfile `openspec-validate` target: unchanged unless it references the
  fallback; check and update if needed.
- ci.yml wiring: the `Install openspec (pinned...)` step bumps
  `@fission-ai/openspec@0.19.0` → `@fission-ai/openspec@<1.x pin>`; the
  step NAME and any comments updated (the pinned version is now 1.x); the
  validate step that runs the script stays.
- `tests/ci_wiring.rs`: update any assertion pinning 0.19.0 / the
  version-guard to the new shape (grep `0.19.0` and `spec_validate` there).
- Dev-machine install requirement: document (in the decision record and,
  if there is an existing docs seam for dev setup — e.g. the decision
  record itself or a README/contributing note if one already exists) that
  contributors must `npm install -g @fission-ai/openspec@<pin>` so the
  bare gate works. Do NOT create new doc pages — use the decision record
  as the home unless a natural existing seam exists.

### 6.2 — Decision record + GH-20

- New `openspec/decisions/2026-10-19-validate-pin-retired.md` (match the
  repo's decision-record format; use today's date slug) explicitly
  superseding `2026-10-19-openspec-validate-pin.md`: WHY the pin existed
  (dual-format delta shape 0.19 accepted / 1.x rejected), what changed
  (Rev 19: plain deltas + layer.md sidecars; 1.14.x strict 52/52 in
  task 5), and the new contract (single 1.x invocation; dev machines
  install the pin; CI installs the pin).
- Update the superseded record with a superseded-by pointer line (repo
  convention — check how other superseded decisions mark it, e.g.
  `2026-10-19-delta-section-strip.md`).
- GH-20: add a landing note comment via `gh issue comment 20 --repo <org
  from gh repo view>` (or `gh issue view 20` first to see context) — the
  org-wide sweep is unblocked: corpus is 1.x-strict-valid, the pin is
  retired, validators converge on the pinned 1.x. If `gh` is
  unavailable/auth fails, SKIP the comment and say so in your report (do
  not fabricate).

## Known gotchas (from session recall — read before coding)

- **edit tool atomicity**: multi-edit[] calls roll back ALL edits when one
  oldText misses — verify each edit landed.
- **TMPDIR quota**: set `TMPDIR=/var/tmp/<your-session-slug>` before long
  test runs.
- **cli test binary rebuild lag**: `spk()`-style tests can run a stale
  binary mid-iteration — rebuild before concluding RED/GREEN.
- The `openspec` CLI on this machine is globally 0.19.0 (`/home/linuxbrew/.linuxbrew/bin/openspec`);
  1.14.x runs via `npx @fission-ai/openspec@1.14.2`. After your change the
  SCRIPT must invoke plain `openspec` (no fallback) — for local
  verification of the 1.x pin you can prepend the npx-cached 1.14.x to
  PATH or `npm i -g @fission-ai/openspec@1.14.2` if acceptable; document
  what you did. CI installs the pin itself, so the bare invocation is
  correct there.
- **`spk` binary staleness**: the installed spk can lag the repo; if
  pre-commit lint flags sidecars, `cargo install --path .` first.
- Commit hygiene: stage only files YOU authored.

## Hard scope guard

- Allowed files: `scripts/spec_validate.py`, `scripts/test_spec_validate.py`,
  `.github/workflows/ci.yml`, `tests/ci_wiring.rs`,
  `openspec/decisions/2026-10-19-validate-pin-retired.md` (new),
  `openspec/decisions/2026-10-19-openspec-validate-pin.md` (superseded-by
  pointer), `justfile` (ONLY if the openspec-validate target needs it).
- Never edit: `openspec/specs/` (READ-ONLY), `openspec/changes/` (READ-ONLY
  — the change's own tasks.md checkbox ticks for section 6 are ALLOWED),
  `.espectacular/`, `pretender.toml`, `specs/`, root `AGENTS.md`,
  `docs/src/**` (no new doc pages), `.wai/**`, other workflows.
- Respect shrink-only ratchets: `pretender check` before committing.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **beads**: you do NOT close the ticket — leave status to the orchestrator.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **ah/espectacular**: read-only over `openspec/`.
- **New source files** need Purpose/Responsibilities/Rationale headers.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags.

## Gates to run before reporting

- `just ci` → green (all recipes; guard-espectacular is green on main)
- `spk lint openspec` → 0 findings
- `ah check` → green, baseline counts unchanged
- The collapsed gate itself: `python3 scripts/spec_validate.py` against
  the 1.x validator → exit 0
- `pretender check` → gate mode passes

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `just ci` → <result>
- `spk lint openspec` → <result>
- `ah check` → <result>
- `python3 scripts/spec_validate.py` (1.x) → <result>
- `pretender check` → <result>
- `gh issue comment 20` → <done | skipped + why>

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">
