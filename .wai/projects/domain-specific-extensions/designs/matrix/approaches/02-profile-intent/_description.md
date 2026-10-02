# Profile intent + manifest section

A domain pack is a four-layer spec file (`kind: profile` or similar) whose
manifest section declares: optional sections it introduces, append-only kind
growth (namespaced), new reference-typing rows, its checkers, and per-kind
required-case floors — against a base format_revision. `spk doctor`/lint
negotiate enabled packs. The pack is itself lintable (self-hosting).
