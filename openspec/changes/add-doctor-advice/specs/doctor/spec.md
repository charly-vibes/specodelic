## ADDED Requirements

### Requirement: Doctor advice profiler

`spk doctor --advice` SHALL profile each file's model with signals
computable from the file itself and SHALL emit advisory routing guidance
derived from those signals on the information/warnings channel, with exit 0
and without gating any lifecycle transition or writing to the corpus. The
signals and their heuristic character are specified in the scenarios; the
format declares no data dimensions, so growth-over-revisions is not
computable here.

#### Scenario: Control-dominated model gets the loop advice

- **WHEN** a file's model sits under the documented state-count threshold
  with a high executable-guard share (few guard-cited claims lacking
  `**rust:**`/`**kernel:**` fragments) and low fan-in (reusing the graph
  capability's existing fan-in signal)
- **THEN** the advice recommends the lint → compile → model-check loop and
  names those signals with their values

#### Scenario: Data-heavy signals get the oracle advice

- **WHEN** a file's state count exceeds the threshold, or its guard-cited
  claims are predominantly without executable fragments
- **THEN** the advice names the state/fragment signal — labeled as a
  heuristic — and recommends putting the behavior in executable fragments or
  an external oracle with differential checking, stating that the spec layer
  cannot challenge it

#### Scenario: Clean signals say so explicitly

- **WHEN** every profiled file triggers no signal
- **THEN** the advice emits an explicit no-signals note (informational) —
  never silence

#### Scenario: Advice is advisory

- **WHEN** `spk doctor --advice` runs on any corpus
- **THEN** exit status is unchanged by the advice's presence or absence, and
  the corpus is left byte-identical

#### Scenario: Uncomputable growth is out of scope

- **WHEN** an implementation is tempted to derive "state growth" from a
  single file
- **THEN** it does not — growth-over-revisions needs two models and is
  `spk preserve`'s or a diff workflow's concern, not the profiler's
