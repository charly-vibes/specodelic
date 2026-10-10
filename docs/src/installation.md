# Installation & Quick Start

## Install

```sh
# macOS (Homebrew)
brew tap charly-vibes/charly
brew install specodelic

# Linux / macOS (binary download — all five release targets, see checksums.txt)
# Grab the archive matching your platform from:
# https://github.com/charly-vibes/specodelic/releases/latest
#   specodelic_{VERSION}_linux_{amd64|arm64}.tar.gz · specodelic_{VERSION}_darwin_{amd64|arm64}.tar.gz
tar xz specodelic_*_linux_amd64.tar.gz && sudo mv specodelic spk /usr/local/bin/

# Windows (Scoop)
scoop bucket add charly https://github.com/charly-vibes/scoop-charly.git
scoop install specodelic

# Any platform (Cargo)
cargo install specodelic
```

(Rust stable toolchain; published on [crates.io](https://crates.io/crates/specodelic).)
Installs two binaries: `specodelic` and its alias `spk`.

## Quick start

All output below was captured verbatim from `specodelic 0.8.0`. On a TTY
the tool prints human-readable text (shown here); when stdout is piped it
emits a JSON envelope instead.

```sh
# Scaffold a spec — the template teaches its own tables
spk new order.cancel --file order.cancel.md
```

```text
created order.cancel.md
→ Run: specodelic lint order.cancel.md
```

(`--file` picks the target path; without it the scaffold lands in the
openspec tree at `openspec/specs/<cap>/spec.md`, which must already exist.)

```sh
# Lint it — findings carry their own rule id + semantics
spk lint order.cancel.md
```

```text
lint: 1 file(s) linted, 0 finding(s), 0 advisory warning(s)
→ Run: specodelic graph
```

**What a finding looks like.** A typical first edit — replacing the
file-qualified wiki-link in the `traces_to` column with bare text — is a
real mistake, and lint reports it:

```sh
spk lint order.cancel.md
```

```text
lint: 1 file(s) linted, 1 finding(s), 0 advisory warning(s)
  order.cancel [linter.single_root_reachable] 4 row(s) unreachable from any intent row — an orphaned island (traces_to/derives_from/guard/from-to/emits): order.cancel.c1, order.cancel.initial, order.cancel.p1, order.cancel.t1
→ Run: fix the reported invariants — each rule's semantics: spk explain lint-rules
```

That is `linter.single_root_reachable` hard-failing: every row must reach
its file's own intent row through own-file typed links, and bare text is
not a wiki-link — so `c1` loses its reachability path and drags the state,
transition, and property hanging off it down as one orphaned island.

```sh
# Derive the reference graph (fan-in/out, dangling links)
spk graph order.cancel.md
```

```text
graph: 1 file(s), 5 node(s), 5 edge(s), 0 dangling, 0 typing violation(s), 0 supersedes cycle(s), 0 external boundary(ies)
→ Run: specodelic lint
```

```sh
# Compile to artifacts: TOML + proptest scaffolding + TLA+ module
spk compile order.cancel.md --out-dir specodelic/
```

```text
compile: 1 compiled, 0 failed
  order.cancel.md [order.cancel] → specodelic/order.cancel.toml, specodelic/order.cancel_props.rs, specodelic/order.cancel.tla
→ Run: specodelic verify (consumes the *_props.rs artifacts)
```

```sh
# Model-check the compiled module (embedded stateright backend)
spk model-check order.cancel.md
```

```text
model-check: 1 checked, 0 failed
  order.cancel.md [order.cancel]: exploration_only (1 states explored) → specodelic/order.cancel.check.json
    claims: 0 evaluated, 1 unchecked, 0 blocking
      unchecked: c1
→ Run: specodelic verify (consumes the *.check.json reports)
```

## In an existing workspace

```sh
spk doctor        # diagnose the workspace (self-hosting vs consumer mode)
spk init          # write the SPECODELIC managed block into AGENTS.md
spk explain       # the embedded format guide — nine topics, no repo access needed
spk feedback      # send feedback about the tool
```

## Self-hosting

This repo's own corpus (`specs/`) is linted by the tool itself:

```sh
just lint-specs   # spk lint specs
just ci           # fmt-check + clippy + tests + release build + openspec gates
```
