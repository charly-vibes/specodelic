# Tasks: add-parse-command

## 1. `spk parse` (TDD)

- [x] 1.1 Failing tests: envelope shape for a well-formed file — `ok`, `data` carries the full `Spec` IR (intent, constraints, states, transitions, properties, links), `hints` suggests `spk lint <file>`
- [x] 1.2 Failing tests: parse succeeds on a file that fails `spk lint` (no lint status in the envelope)
- [x] 1.3 Failing tests: labeled error envelope + non-zero exit for unparseable input and missing file
- [x] 1.4 Implement `src/parse.rs` (thin wrapper over `spec::parse` + serde) and register the subcommand in `src/main.rs`

## 2. Docs and dogfood

- [x] 2.1 `USAGE.md` entry; mention in the `guide` topics if the command catalog is listed there
- [x] 2.2 `spk lint specs/ openspec/changes/add-parse-command/specs` exits zero (dogfood the new capability delta)
- [x] 2.3 `openspec validate add-parse-command --strict` exits zero

## 3. Release coordination

- [ ] 3.1 Cut the release containing parse (and archive-companion, already on main); note it for espectacular's pin bump (their task group 1)