# Which command when? — routing a task to the pipeline

The full reference is [commands.md](commands.md); this page answers the
earlier question — *I have X, what do I run?* — including which verbs
gate on which, and which findings are advisory.

## The spine first

The pipeline verbs form a gated chain — each stage refuses to run on a
file the earlier stages have not blessed:

```text
parse → lint → compile → model-check → verify
              (gates on lint)  (consumes      (gates on
                               artifacts)     model-check)
```

`spk compile` only runs on a file that passed lint and coverage
(`openspec/specs/compile/spec.md`, `precondition_satisfied`); `spk verify` never
re-compiles — it consumes the artifacts a prior compile wrote to the
same out-dir, so a stale compile is a rerun, never an acceptance;
`spk orchestrate` applies the same stage discipline across a whole
corpus run. `spk conform` stands *outside* this chain: it consumes and
classifies, but never gates a lifecycle transition.

## Routing table

| What you have / want | Run | Notes |
|---|---|---|
| A new spec | `spk new <id>`, then `spk lint` | The scaffold teaches its own tables — [quick start](installation.md) |
| A design question, not code | the [design canvas](design-canvas.md), running the loop in it | Specs first; implementation is a rendering |
| "Is this file format-valid?" | `spk lint` | The gate; findings self-describe — `spk explain lint-rules` |
| The machine to challenge the design | `spk compile`, then `spk model-check` | Bounded exploration; a counterexample is often the design lesson |
| Executable checks / generated test cases | `spk compile`, then `spk verify` | Only opted-in invariant claims are checked; prose never is |
| Traces from an existing system, oracle, or curated cases | `spk conform <file> --oracle scenarios.jsonl` | Open-world default; `--closed-world` makes an uncovered trace `forbidden` — [conform](conform.md) |
| Diagrams of the corpus | `spk graph` | Projections and derived views — [graph views](graph-views.md) |
| An id changed | `spk rename <old_id> <new_id>` | In-memory verified, atomic; rewrites links, filenames, checklists |
| Two branches both touched specs | `spk merge --branch <incoming>` | Collisions, dangling renames, blast radii — *before* joining trees |
| A file got too big / high fan-in | `spk refactor` | **Advisory only** — it flags split candidates, never edits |
| The whole pipeline over many files | `spk orchestrate` | Optional `--backend tlc --tlc-jar` — note TLC cannot execute Rust fragments |
| The workspace feels wrong | `spk doctor` | Workspace diagnosis with remediation hints |
| An OpenSpec delta to bring into dual format | `spk migrate <file> [--dry-run]` | Wraps the delta into the dual-format skeleton |
| The dual-format lint gate in pre-commit | `spk hooks install` | Marker-guarded managed block; never claims `core.hooksPath` |
| An OpenSpec change to archive, preserving the layer | `spk archive-companion <CHANGE_ID> [--dry-run]` | Fails closed on a delta lacking the dual-format layer |
| The format explained to an agent, no repo access | `spk explain [topic]`, `spk parse <file>`, `llms.txt` | Nine explain topics — [For agents](index.md#for-agents) |

## Two honest maturity notes

- `spk model-check` is bounded, and honest about it: exhaustive runs
  report `exploration_only`, never a fabricated clean; a backend that
  cannot execute Rust fragments reports `exploration_only`, never a
  fabricated pass ([model_check](specs/model_check/spec.md)).
- `spk refactor` advises — it never edits. Its findings are exit-0
  regardless ([refactor](specs/refactor/spec.md)).

More boundaries: [verification boundaries — what a green check actually buys you](verification-boundaries.md).
