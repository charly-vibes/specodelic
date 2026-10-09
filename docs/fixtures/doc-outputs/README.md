# doc-outputs fixtures — replay inputs for the docs-accuracy gate

`scripts/check_quickstart_outputs.py` (run via `just doc-examples`) re-runs
every command whose output is captured verbatim in
`docs/src/installation.md` and `docs/src/examples/worked-example.md`, and
diffs the real output against the fenced ```text block in the doc.

These directories are the **inputs** for the worked-example scenarios:
each stage is the spec file exactly as the tutorial has built it at that
point of the narrative (`reservation-order.md`, per the naming law — the
linter's `linter.id_matches_file` rule pins the filename). The tutorial
shows the file only as fragments, so the replayable whole is checked in
here instead.

- `01-intent-only` … `05-model-fixed` — the five lint stages of the
  worked example (intent → constraints-with-typo → typo fixed →
  model draft with its four lessons → model fixed + effect row).
- `06-over-claim` — the finished file plus the executable
  `reserve_never_fails` invariant and its property (the counterexample
  stage).

The final-state scenarios (`lint`/`graph`/`compile`/`model-check`/`verify`
at zero findings) use `docs/src/examples/reservation-order.md` directly —
no duplicate copy.

These are inputs, not expected outputs: expected outputs live only in the
docs and are never duplicated here. Regenerating an expected block means
re-capturing from the real binary and pasting the fenced block into the
doc — never hand-editing either side to make the gate green.
