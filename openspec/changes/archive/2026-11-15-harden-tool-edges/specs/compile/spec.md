## ADDED Requirements

### Requirement: Duplicate artifact stems refused before any write

`spk compile` over a corpus SHALL refuse an invocation in which two distinct
collected files produce the same artifact stem (`compile::artifact_stem`) with
a `duplicate_artifact_stem` error envelope naming both colliding paths and the
remediation hint (rename one file, or compile in separate `--out-dir` runs),
exiting 2 before lint, before compilation, before every artifact write, and
before the out-dir is created, so a refused run leaves no artifacts.

#### Scenario: Duplicate stems refuse with no artifacts written

- **WHEN** two distinct files in one `spk compile` invocation share an
  artifact stem
- **THEN** the run exits 2 with an error envelope whose label is
  `duplicate_artifact_stem`, whose message names both colliding file
  paths, and whose hint names the rename / separate-`--out-dir`
  remediation — and the out-dir is never created

#### Scenario: Unique stems compile unchanged

- **WHEN** every collected file in the invocation has a distinct artifact
  stem
- **THEN** every file compiles and writes its artifacts with exit 0 — the
  guard is collide-only

#### Scenario: A repeated path is one file, not a collision

- **WHEN** the same path is collected twice by one invocation (repeated
  argument or overlapping roots)
- **THEN** the run does not refuse: the repeat is one file with one
  artifact set, exit 0, and the refusal label never appears

### Requirement: Scope refusal keeps the corpus notes

The duplicate-stem refusal SHALL re-attach every parse and hostile-input
note collected while building the corpus to the refusal envelope's
warnings channel, exactly as the adjacent empty-corpus refusal does — an
early refusal never swallows the note that names a skipped file.

#### Scenario: Hostile-input note survives the refusal

- **WHEN** a corpus carries both a stem collision and a non-ingestable
  file skipped with a labeled note
- **THEN** the error envelope reports the collision label and the skip
  note naming the skipped file, exits 2, and writes nothing
