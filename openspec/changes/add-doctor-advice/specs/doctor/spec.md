## ADDED Requirements

### Requirement: Doctor advice profiler

`spk doctor --advice` SHALL profile each file's model with computable
signals — state-count growth attributable to data dimensions, guards citing
claims whose expr carries no executable fragment, and fan-in concentration —
and SHALL emit advisory routing guidance derived from those signals, on the
information/warnings channel, with exit 0 and without gating any lifecycle
transition or writing to the corpus.

#### Scenario: Control-dominated model gets the loop advice

- **WHEN** a file's model shows small state counts with no data-dimension
  growth and executable or structural guards
- **THEN** the advice recommends the lint → compile → model-check loop and
  names the signals that support it

#### Scenario: Data-dominated model gets the oracle advice

- **WHEN** a file's state counts grow multiplicatively with data dimensions,
  or guards cite claims with no executable fragment
- **THEN** the advice names the data/fragment signal and recommends putting
  the behavior in executable fragments or an external oracle with
  differential checking, stating that the spec layer cannot challenge it

#### Scenario: Advice is advisory

- **WHEN** `spk doctor --advice` runs on any corpus
- **THEN** exit status is unchanged by the advice's presence or absence, and
  the corpus is left byte-identical
