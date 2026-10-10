# conform — checking a spec against reality, not against itself

Every other verb in the pipeline evaluates the spec's own claims: `lint`
checks the file's shape, `compile` and `model-check` explore the model the
file declares, `verify` checks the file's opt-in claims. `spk conform` is
the one verb that brings in evidence from *outside* the spec: it
classifies traces recorded from an external oracle — a legacy system, a
curated set of cases, a reference implementation — against what the spec
declares and can actually execute.

## Usage

```console
$ spk conform <spec-file> --oracle scenarios.jsonl [--closed-world]
```

The scenario corpus is JSONL: one record per line, each carrying `id`,
an optional `setup` state, and a `trace` of actions with observations.
Anything else — unknown fields, malformed lines, missing or duplicate
ids — is refused with a remediation hint, never silently ignored.

## The verdict taxonomy

Every trace receives exactly one verdict from a closed set, each with a
labeled reason:

| Verdict | Meaning |
|---------|---------|
| `permitted` | a declared, executable claim covers the trace and the trace agrees with it |
| `forbidden` | the trace *violates* a declared executable claim (any invocation mode), or — only under `--closed-world` — nothing declared covers it |
| `underspecified` | no declared Model element or claim covers the trace (open-world default: a missing transition is a spec gap, never a prohibition) |
| `unknown` | a claim covers the trace but is prose-only — prose is never silently checked |
| `unsupported` | the covering claim's evaluator kind is not executable in this run |

Exit codes follow the same honesty discipline as the rest of the
pipeline: `0` = no `forbidden` or `unsupported` verdict; `1` = any such
verdict; `2` = invocation error or gate refusal. `unknown` and
`underspecified` are surfaced as counts but never fail the run — they are
spec-gap findings, not refutations.

## Gates and lifecycle

conform runs only over a file that parses, lints clean, and has current
compiled artifacts — the same stage discipline `orchestrate` applies
between stages. A stale artifact or a lint-dirty file is refused with the
remediation hint (`spk compile` / `spk lint`), and no verdicts are
derived from it.

conform is deliberately **outside the artifact lifecycle**: it never
advances `draft → parsed → linted → compiled → model_checked → verified`,
is never a stage of `spk orchestrate`, and a conform run — even one that
ends in `forbidden` verdicts — never gates or blocks a lifecycle
transition, and writes nothing: not the spec, not the artifacts, not the
report. It consumes, classifies, and reports.

## Evidence scope

The report — in both the JSON envelope and the `--human` view — carries a
fixed `evidence_scope` field. It is data, not documentation, so no
reformatting can strip it:

> agreement on the supplied corpus; not a proof of behavioral equality

A finite set of recorded traces can agree with the spec without the spec
describing the system. conform tells you exactly how far its evidence
reaches — and no further. For how that fits the other verbs' guarantees,
see [verification boundaries](verification-boundaries.md); for the
lifecycle stages conform stays outside of, see
[orchestrate](specs/orchestrate.md) and the
[command reference](commands.md).