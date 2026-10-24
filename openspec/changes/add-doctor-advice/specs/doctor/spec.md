## ADDED Requirements

### Requirement: Doctor advice profiler

`spk doctor --advice` SHALL profile each file's model with signals
computable from the file itself and SHALL emit advisory routing guidance
derived from those signals on the information/warnings channel, with exit 0
and without gating any lifecycle transition or writing to the corpus. The
signals are: absolute state count against a documented threshold, the share
of guard-cited claims whose expr carries no executable fragment
(`**rust:**`/`**kernel:**`), and fan-in concentration (reusing the graph
capability's existing fan-in signal). "Data-dominated" guidance is a
heuristic named as such — the format declares no data dimensions, so
growth-over-revisions is not computable here.

#### Scenario: Control-dominated model gets the loop advice

- **WHEN** a file's model sits under the state-count threshold with a high
  executable-guard share and low fan-in
- **THEN** the advice recommends the lint → compile → model-check loop and
  names the signals that support it

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
