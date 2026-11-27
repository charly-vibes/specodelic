# Quint formal-verification-tools survey — specodelic tool grading (2026-11-18)

Source: https://quint.sh/guides/formal-verification-tools (guide dated
2026-09-26, retrieved 2026-11). Grades verified against primary sources
(repo LICENSE files, crates.io license fields, veil.dev license statement)
in the same session. Provenance anchor for
`openspec/changes/add-kani-proof-harnesses/proposal.md` (track-1: tools
against specodelic) and the track-2 integration roadmap discussion.

## Grade card: open-source × Rust-integratability

| Tool | OSS grade | License (verified source) | Rust-integration grade | Shape |
|---|---|---|---|---|
| Stateright | A+ | MIT (stateright/stateright LICENSE, master) | A+ | Pure Rust crate, already in-process in spk's Cargo.toml |
| Kani | A | MIT (model-checking/kani LICENSE-MIT, dual Apache/MIT) | A | `cargo install kani-verifier` → `cargo kani`; native cargo workflow |
| Lean 4 | A | Apache-2.0 (leanprover/lean4 REUSE headers, master) | B− | Native binaries (elan/lake); contract is Lean source, not Rust; LSP+MCP |
| Veil | A− | Apache-2.0 (veil.dev statement; repo verse-lab/veil) | C+ | Only via Lean toolchain; research repo; ships agent skills |
| Quint | A | Apache-2.0 (informalsystems/quint LICENSE) | C+ | TypeScript/npm CLI subprocess; Node enters the dep graph |
| TLC | A | MIT (tlaplus/tlaplus LICENSE, master) | C | Java jar; already wired via `--tlc-jar` + `SPK_TLC_JAVA` |
| Apalache | A | Apache-2.0 (informalsystems/apalache LICENSE) | C | Scala/JVM fat jar; heaviest runtime footprint |
| Alloy | A− | MIT in practice (org.alloytools.alloy LICENSE header: "NOT VALID YET! currently under MIT") | C− | JVM + Gradle-built jar, no Rust bindings |
| Verus | A | MIT (verus-lang/verus LICENSE) | C+ | Real Rust syntax but forked rustc toolchain (vargo) — CI-invasive |
| Creusot | B+ | LGPL-2.1-or-later (creusot-contracts on crates.io) | B | cargo-native driver, but LGPL annotation dependency; Why3/OCaml backend |
| Prusti | B | MPL-2.0 (viperproject/prusti-dev LICENSE) | B− | Custom rustc driver + JVM Viper server; slowed maintenance |

## Decision taken from the grades

- Track 1 (verify spk internals): **Kani** selected — only candidate
  permissively licensed AND cargo-native for pure-function harness work.
  Verus/Prusti/Creusot struck (fork toolchains / copyleft / JVM).
- Track 2 (integrate engines for users): Apalache first (consumes spk's
  existing `.tla` emission), Lean/Veil for law discharge, Quint emitter and
  Alloy deferred; sequencing recorded in the same session's discussion.
- Standing note: Stateright A+/A+ is the baseline — embedded, MIT.
