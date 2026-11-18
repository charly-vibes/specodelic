---
sut_path: /var/home/sasha/para/areas/dev/gh/charly/specodelic
commit: c76e2b0c
updated: 2026-11-18
external_references:
  - path: /var/home/sasha/para/areas/dev/gh/charly/reports/hegel-eval-2026-11-16/README.md
    why: Establishes that the decision under evaluation is a local test runner (Hegel), not fault injection, and that the crate's generated machinery stays on proptest.
  - path: /var/home/sasha/para/areas/dev/gh/charly/specodelic/.wai/resources/research/2026-11-18-antithesis-skills-method-inventory.md
    why: Its non-goals explicitly exclude the Docker/Antithesis harness machinery for single-crate work; this file records that exclusion for this SUT.
  - path: https://hegel.dev/
    why: The local engine that replaces a containerized Antithesis deployment for these in-process properties.
---

# Deployment topology

## Conclusion

**There is no Antithesis container topology for this SUT, and the harness path
is out of scope for this pass.** `specodelic` is a single-crate Rust
library/CLI. The in-scope laws are pure functions over in-memory
`(path, text)` corpora. There is no service, listener, peer protocol, database,
shared filesystem, or multi-process state for a container network to isolate or
perturb.

An Antithesis deployment for this crate would be one container running a test
driver plus one container running the same crate — with no network, no
replication, and no fault to inject between them. That topology would add
scheduling nondeterminism and container overhead without exploring any state
space the local generator does not already explore.

## Minimal equivalent topology

| Component | Instances | Why |
|---|---|---|
| test process (`cargo test` / `cargo nextest` / Hegel harness) | 1 per property run | Runs the in-process property; no network |
| tempdir fixture | 0 or 1 per case | Only for tests that choose to exercise `write_artifacts_into` or the CLI; the law properties are in-memory |
| built `spk` binary | 0 | Not needed for library properties; `tests/merge_flow.rs` uses it for end-to-end coverage, which is outside this catalog |
| `rustfmt` subprocess | 0 or 1 per compile-write case | `compile_spec` does not invoke it; `write_artifacts_into` does, and that seam is excluded from byte determinism |
| network / containers / volumes | 0 | Nothing to connect |

## Why the Antithesis harness adds nothing here

The Antithesis strengths are timing-sensitive races, concurrency interleavings,
partial failures, and crash/restart. None is present in the in-scope seams:

- `rename::run`, `writer::emit`/`apply`, `merge::run`, and
  `compile::compile_spec` take references and return values; there is no shared
  mutable state.
- The only thread is `rustfmt_clean`'s stdin feeder, used after compilation in
  the write path; it is not a race target and its output is an environment
  input rather than a property.
- There is no durable state to crash-recover: a caller applies a write-set, but
  that boundary is outside the crate's law functions.
- The failure classes the properties target are **data-shape** failures
  (keyword-colliding ids, add/add paths, span drift, non-deterministic
  iteration), which generated inputs reach directly.

Fault injection would therefore explore container behavior, not `specodelic`
behavior. The honest equivalent is a single local test process with a
deterministic generator and shrinker — Hegel in `tests/`, or proptest.

## What would change this conclusion

- A property about concurrent `spk` invocations writing the same out-dir. That
  is a filesystem-race property at the CLI boundary, not a library law, and it
  would need a multi-process topology (or at least threads). It is not in this
  catalog.
- A property about crash recovery of a partially applied write-set. The crate
  deliberately returns data and performs no I/O, so the crash model belongs to
  the caller.
- A remote registry or service dependency added to `compile`/`verify`. Neither
  exists today.

## Assumptions and Open Questions

- [ASSUMPTION] The decision under evaluation is "which local property runner and
  which properties", not "should this crate be tested under Antithesis". The
  Hegel evaluation report scopes the decision that way.
- Open: if a future pass targets CLI subprocess contracts (exit codes,
  envelopes, concurrent out-dir writes), a minimal one-container topology plus a
  driver becomes meaningful. It should be designed then, not pre-built for a
  pure library. (needs human input only if the scope expands)