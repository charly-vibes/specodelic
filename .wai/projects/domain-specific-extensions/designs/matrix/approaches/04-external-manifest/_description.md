# External manifest file

Pack registration lives outside the spec corpus (e.g. a TOML manifest the CLI
reads). The spec corpus stays pure four-layer; tooling parses manifests for
namespacing and negotiation. Trades self-hosting for implementation simplicity.
