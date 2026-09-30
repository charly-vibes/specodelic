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
