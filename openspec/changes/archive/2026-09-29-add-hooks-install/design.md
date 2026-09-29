# Design: add-hooks-install

## Context

genesis-vibes 0.8 shipped `genesis::git_hooks` with exactly the
primitives the AGENTS.md hard blockers demand: `resolve_hooks_dir()`
(respects `core.hooksPath`), an ordered sigil table detecting
`Owner::Bd`, `framework()` detection, and marker-guarded
`install()`/`uninstall()` that refuse foreign hooks. It also ships
`lefthook::ensure_wired()` (managed-block injection into a lefthook
stage) and `lefthook::is_wired()` (stage-scoped verification).

In this repo (and any beads-managed charly-family repo), the hook chain
is: git → `core.hooksPath` → `.beads/hooks/pre-commit` (bd shim,
sigil `bd`) → lefthook → `lefthook.yml` stage commands. The sanctioned
extension point is the lefthook config.

## Goals / Non-Goals

**Goals**
- Wire `spk lint openspec` as a pre-commit command via a managed block.
- Purely additive: never drop, reorder, or wrap existing entries.
- Idempotent install/uninstall; envelope-reported outcome; remediation
  hints on every error path.
- Keep the hook self-contained: no `openspec` CLI, no `just`, no python
  at hook time (those stay CI-side in `just ci`).

**Non-Goals**
- Claiming `core.hooksPath` or writing `.git/hooks/*` / `.beads/hooks/*`
  (hard blockers).
- Supporting husky or prek wiring (detect and refuse with a hint).
- Creating a lefthook config from scratch (inherited genesis D4
  non-goal — D4 = never create a hook config from scratch;
  `MissingLefthookConfig` error instead).
- Repo-specific CI gates (`openspec validate --strict`, section-sync
  script) — those remain `just ci` recipes.

## Decisions

### Decision 1: two-case anchor — inside the existing `commands:` mapping

`lefthook::ensure_wired` inserts the block directly after the stage
anchor line. Its documented content shape is a full stage body
(`  commands:\n    gate:\n      run: ...`). Empirically verified
(lefthook 1.13.6): a stage section that already has `commands:` plus an
injected second `commands:` key fails to parse —

```
│  Error: yaml: unmarshal errors:
│    line 5: mapping key "commands" already defined at line 2
```

That failure would kill the *entire* hook chain, including beads' own
gates — worse than not wiring at all, and a violation of the chain
preservation constraint. Therefore:

Indentation is never assumed — it is inferred (Rule-of-5 CORR-001:
YAML allows any consistent indent; a 2-space wrapper injected into a
4-space-indented stage produces mixed-indent keys in one mapping, a
parse error of exactly the class we are avoiding):

- **Children indent** = indentation of the stage section's first child
  key line (2 spaces when the stage is empty).
- **`commands:` found at children indent** → insert the command entry
  (with YAML-comment markers) immediately after the `commands:` line,
  at children-indent + 2 — inside the existing mapping. No
  end-of-mapping scan needed.
- **No `commands:` at children indent** → format the full wrapper at
  children-indent and delegate to genesis `lefthook::ensure_wired`
  (the content is caller-supplied, so no genesis change is needed).
- **Stage present but children indent unascertainable** (e.g. quoted
  or mis-indented `commands:` key) → refuse without modification
  (honest_anchor), mirroring genesis D6 (D6 = refuse to anchor
  unrecognizable structure).

The injected block in both cases (markers as YAML comments):

```yaml
  commands:
    # <!-- SPK:START -->
    specodelic-gates:
      run: spk lint openspec
    # <!-- SPK:END -->
    sibling-blockers:
      run: ...
```

Marker strings use `BlockDef::with_markers("# <!-- SPK:START -->",
"# <!-- SPK:END -->")` — bare `<!--` is not valid YAML; the `# ` prefix
makes every marker line a comment, so the config stays valid regardless
of parser strictness. The two-space example below is the common case;
the actual indent is inferred per the rule above.

A red-test fixture matrix covers the indentation and byte-format edges
(Rule-of-5 EDGE-003): a 4-space-indented config, a config with no
trailing newline, and a CRLF config.

**Alternatives considered**
- Full-wrapper injection always (genesis test shape): produces the
  duplicate key — rejected, breaks the chain.
- Appending a managed block into bd's `.beads/hooks/pre-commit` shim:
  modifies a foreign file; bd regenerates its shims and would wipe it —
  rejected.
- Installing our own hook file: `pre-commit` is the hook name and bd
  owns it; genesis `install()` correctly refuses (Foreign) — there is no
  second pre-commit slot in git.
- Upstreaming first: a genesis `ensure_command_wired()` (command-level
  anchor) is the clean long-term home for the inside-mapping insertion;
  filed as a follow-up upstream ticket rather than blocking this
  capability on a genesis release cycle. Consolidate when genesis picks
  it up.

### Decision 2: gate command is fixed — `spk lint openspec`

The wired command runs the freshly-installed `spk` binary itself (a
real second binary: `Cargo.toml` `[[bin]] name = "spk"`) over the
repo's openspec tree: it enforces `dual_format_valid` and the full
specodelic lint on every commit, with zero external dependencies at
hook time. `openspec validate --strict` and the section-sync script stay
CI-only (they need the `openspec` CLI / python and are repo-protocol
gates, not specodelic gates). `spk lint` on a missing/empty tree fails
loudly with a hint (specodelic-6pi semantics), so a mis-wired repo
fails commits visibly rather than silently.

`install` pre-checks that an `openspec/` directory exists and errors
with a remediation hint otherwise — fail at wiring time, not at commit
time. Install then runs the gate once as a dry-run and reports the
outcome over the envelope (Rule-of-5 EDGE-001: a repo whose deltas are
not yet dual-format would otherwise be trapped failing every commit
with no warning) — a failing dry-run is a warning on the envelope's
warnings channel carrying the failure summary and an
`spk hooks uninstall` escape hint, never an install failure.

### Decision 3: framework gate

Use `genesis::git_hooks::framework()`: lefthook → proceed; prek or
husky → labeled error naming the detected framework and hinting manual
wiring (no block injection into configs we can't anchor); none →
`MissingLefthookConfig` error with a hint. Never create a config.

### Decision 4: uninstall by managed-block strip

`spk hooks uninstall` removes exactly the lines between the block
markers (inclusive). Because markers are whole-line YAML comments,
stripping them from either wiring case leaves valid YAML and leaves the
rest of the file byte-identical. Unwired → success no-op (reported as
`not_wired`), never an error.

## Risks / Trade-offs

- **Local inside-mapping insertion is anchored string surgery** —
  genesis deliberately excluded this ("no bespoke string surgery") in
  favor of stage-level injection. Mitigation: the insertion point is a
  single, strictly-defined anchor (the `commands:` line at the stage's
  inferred children indent); unanchorable structures (quoted/indented
  `commands:` key, unascertainable children indent) refuse without
  modification, mirroring genesis D6 (D6 = refuse to anchor
  unrecognizable structure). Upstream follow-up consolidates it into
  genesis.
- **Hook runs the whole corpus lint every commit** — corpus is small
  (18 files, lint < 1s); full lint is deterministic and simple. Revisit
  changed-files-only wiring if it ever hurts.
- **`spk` must be on PATH at commit time** — the gate invokes `spk`
  unqualified, like lefthook's own binaries. Remediation hint on install
  output states this assumption.

## Migration Plan

None — additive capability. Existing repos keep their hooks untouched
until they opt in by running `spk hooks install`.

## Open Questions

None — the empirical duplicate-key test settled the wiring shape.