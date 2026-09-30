# Dogfood observations — `linter.observability` (task 5.2)

Recorded 2026-09-29, immediately after the check landed (commit
sequence of specodelic-7l3). This is the evidence base for the deferred
gating question (design D5 / Open Questions); it wires nothing — the
adoption-note discipline of beads `specodelic-4ae` applies.

## What was run

`spk lint specs --json` over the repo's own corpus, first run after the
check existed. Result: 19 files, 0 issues, 1 advisory warning, exit 0:

```
refactor [linter.observability] effect `refactor.advisory_finding_emitted`
has no `observes` reference targeting it — a declared output nobody
observes (advisory, exit 0) [hint: add an `observes` column on the row
that consumes this output, or remove the effect if it is unintentional]
```

## Observations

1. **The predicted first subject fired, and it is the right one.** The
   grounding Ro5 review predicted the corpus's live unobserved effect
   (`refactor.md`'s `found` state emitting `advisory_finding_emitted`)
   would be the first warning. It was. The finding is honest: the
   refactor tool's output shape is a real published behavior (its
   `emitted_finding_shape` property pins it), but nothing in the corpus
   observes it — the consumer is the CLI surface, which is not a spec
   file.
2. **The warning did not disturb any gate.** `just ci`, `just
   lint-specs` (exit 0), the pre-commit gate, and every downstream
   consumer stayed green. Advisory severity via the warnings channel
   works exactly as designed — no corpus file needed an `observes` edit
   to keep gates green.
3. **The warning text carries what a consumer needs.** Rule id, row id,
   severity, and a concrete remediation hint — no re-run needed to act.
4. **Friction observed: none yet — and that is itself informative.** The
   corpus has exactly one effect row, so the check's discriminating
   power is untested by real volume. The gating question ("should
   unobserved effects block?") cannot be answered from one warning on
   one spec. Defer, per D5, until corpora with many effect rows exist
   (bajan-style consumer repos are the candidates).

## Recommendation (for the follow-up change, when reopened)

Stay advisory until a corpus accumulates ≥10 effect rows with real
observer relationships. If gating lands, the natural home is a new
`orchestrate.md` stage gate, not a lint exit-code change — the warnings
channel keeps the lint contract stable. Waivers: revisit only when a
legitimate permanently-unobserved effect appears;
`linter-external_completeness.md`'s covered/waived structure remains the
candidate (design D6).