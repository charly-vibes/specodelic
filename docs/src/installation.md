# Installation & Quick Start

## Install

```sh
cargo install specodelic
```

(Rust stable toolchain; published on [crates.io](https://crates.io/crates/specodelic).)

## Quick start

```sh
# Scaffold a spec — the template teaches its own tables
spk new order.cancel

# Lint it — findings carry their own rule id + semantics
spk lint order.cancel.md

# Derive the reference graph (fan-in/out, dangling links)
spk graph specs

# Compile to artifacts: TOML + proptest scaffolding + TLA+ module
spk compile order.cancel.md --out-dir specodelic/

# Model-check the compiled module (embedded stateright backend)
spk model-check order.cancel.md
```

## In an existing workspace

```sh
spk doctor        # diagnose the workspace (self-hosting vs consumer mode)
spk init          # write the SPECODELIC managed block into AGENTS.md
spk explain       # the embedded format guide — seven topics, no repo access needed
spk feedback      # send feedback about the tool
```

## Self-hosting

This repo's own corpus (`specs/`) is linted by the tool itself:

```sh
just lint-specs   # spk lint specs
just ci           # fmt-check + clippy + tests + release build + openspec gates
```
