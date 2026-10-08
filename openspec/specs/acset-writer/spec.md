---
id: acset.writer
kind: intent
statement: "WHEN an instance edit is realized as text, THE acset-writer capability SHALL emit markdown that differs from the source only in the cells and links the edit touches, so that the typed core never costs the format its byte-stable round trip."
---

# acset-writer Specification

## Purpose

The one module allowed to turn an instance edit back into text. It works
from recorded source spans and returns a write-set, never re-serializing
a table and never doing I/O itself. Without it, the typed acset core
(`acset-core`) can read and query the corpus but every edit still flows
through hand-wired rewriting in `src/rename.rs` — unspecified, unparityed,
and drifting from the format's byte-stability promises
(`rename.prose_untouched_by_rename`). This capability makes emission a
specified, identity-gated seam: `emit(parse(x), no edit) == x` byte for
byte over every file the parser accepts, and every edit touches only the
spans the edit names.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[acset.writer]] |
| source_spans_recorded | invariant | `parsing records the byte span of every id cell, every [[link]] occurrence, and every state bullet; the writer edits only recorded spans and never re-serializes a table` | [[acset.writer]] |
| emit_identity | invariant | `emit(parse(x), no edit) == x byte for byte, for every spec file the parser accepts` | [[acset.writer]] |
| untouched_bytes_preserved | invariant | `bytes outside the rewritten spans of edited rows are identical to the source text — including prose, CRLF terminators, trailing whitespace, and table padding (the mechanical realization of rename.md's prose_untouched_by_rename)` | [[acset.writer]] |
| width_padding_policy | invariant | `when an id edit changes a cell's width the writer leaves surrounding padding unchanged, so a column may become unaligned; realignment is a separate explicit formatting operation and never a side effect of an edit` | [[acset.writer]] |
| filename_follows_intent_id | invariant | `an edit to a file's own Intent id also produces the file rename implied by specodelic.md's id_matches_file rule` | [[acset.writer]] |
| write_set_atomic | invariant | `the writer returns a write-set of (path, full new contents) plus optional removals and performs no I/O; applying it is the caller's single transaction (the shape of rename.md's atomic_operation write-set)` | [[acset.writer]] |
| edit_application_faithful | invariant | `for a morphism f : I → I' and source text x, from_specs(apply(f, x)) equals I' — the emitted text realizes exactly the instance the edit denotes` | [[acset.writer]] |
| edits_compose | invariant | `apply(g, apply(f, x)) == apply(compose(g, f), x)` | [[acset.writer]] |
| span_failure | effect | `spec.span_failure(detail) — the writer could not extract a recorded span because detail; the error envelope satisfies the fleet error contract (error-kind envelope, labeled, remediation hint present, non-zero exit)` | [[acset.writer]] |
| apply_failure | effect | `spec.apply_failure(detail) — an edit names an id no recorded span can realize because detail; the error envelope satisfies the fleet error contract (error-kind envelope, labeled, remediation hint present, non-zero exit)` | [[acset.writer]] |
| roundtrip_failure | effect | `spec.roundtrip_failure(detail) — the emitted text reparses to an instance other than the edit's target because detail; the error envelope satisfies the fleet error contract (error-kind envelope, labeled, remediation hint present, non-zero exit)` | [[acset.writer]] |

## Model

### States
- `requested`
- `spanned`
- `applied`
- `verified`
- `span_failed` (emits: `[[acset.writer.span_failure]]`)
- `apply_failed` (emits: `[[acset.writer.apply_failure]]`)
- `roundtrip_failed` (emits: `[[acset.writer.roundtrip_failure]]`)

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| record | requested | spanned | [[acset.writer.source_spans_recorded]] ∧ [[acset.writer.emit_identity]]  |
| record_fail | requested | span_failed | `¬([[acset.writer.source_spans_recorded]] ∧ [[acset.writer.emit_identity]])`  |
| apply | spanned | applied | [[acset.writer.untouched_bytes_preserved]] ∧ [[acset.writer.width_padding_policy]] ∧ [[acset.writer.filename_follows_intent_id]] ∧ [[acset.writer.write_set_atomic]]  |
| apply_fail | spanned | apply_failed | `¬([[acset.writer.untouched_bytes_preserved]] ∧ [[acset.writer.width_padding_policy]] ∧ [[acset.writer.filename_follows_intent_id]] ∧ [[acset.writer.write_set_atomic]])`  |
| reparse | applied | verified | [[acset.writer.edit_application_faithful]] ∧ [[acset.writer.edits_compose]]  |
| reparse_fail | applied | roundtrip_failed | `¬([[acset.writer.edit_application_faithful]] ∧ [[acset.writer.edits_compose]])`  |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| process_lifecycle_checked | unit | [[acset.writer.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| span_covers_every_link | unit | [[acset.writer.source_spans_recorded]] | `arbitrary_spec_file()` | `card(link_spans) == card(links(parse(x)))` |
| identity_emission_exact | unit | [[acset.writer.emit_identity]] | `arbitrary_accepted_spec_file()` | `emit(parse(x)) == x` |
| crlf_survives_edit | unit | [[acset.writer.untouched_bytes_preserved]] | `crlf_file_with_one_renamed_id()` | `line_terminators(apply(f,x)) == line_terminators(x)` |
| prose_survives_edit | unit | [[acset.writer.untouched_bytes_preserved]] | `row_whose_prose_mentions_the_old_id_as_a_word()` | `prose_bytes(apply(f,x)) == prose_bytes(x)` |
| padding_not_realigned | unit | [[acset.writer.width_padding_policy]] | `rename_that_lengthens_an_id_in_an_aligned_table()` | `padding_bytes(apply(f,x)) == padding_bytes(x)` |
| intent_rename_moves_file | unit | [[acset.writer.filename_follows_intent_id]] | `rename_of_a_files_own_intent_id()` | `path(apply(f,x)) == mapped_filename(new_id)` and one removal is returned |
| writer_does_no_io | unit | [[acset.writer.write_set_atomic]] | `apply_with_io_calls_instrumented()` | `io_calls == 0` |
| text_realizes_instance | unit | [[acset.writer.edit_application_faithful]] | `arbitrary_spec_file(), arbitrary_id_rename()` | `from_specs(apply(f,x)) == rename(from_specs(x),a,b)` |
| writer_edit_law | law | [[acset.writer.edits_compose]] | `arbitrary_spec_file(), arbitrary_morphism_pair()` | **identity:** `apply(id, x) == x`  **associativity:** `apply(g, apply(f, x)) == apply(compose(g, f), x)` |
| span_failure_label_asserted | unit | [[acset.writer.span_failure]] | `span_failure_raised()` | `error_label == "spec.span_failure"` |
| apply_failure_label_asserted | unit | [[acset.writer.apply_failure]] | `apply_failure_raised()` | `error_label == "spec.apply_failure"` |
| roundtrip_failure_label_asserted | unit | [[acset.writer.roundtrip_failure]] | `roundtrip_failure_raised()` | `error_label == "spec.roundtrip_failure"` |

## ADDED Requirements

### Requirement: Span-preserving emission
The acset-writer SHALL emit markdown for an instance from recorded byte
spans alone — never re-serializing a table — such that emission without
an edit reproduces the source byte for byte, and an edit changes only
the cells and links the edit names.

#### Scenario: Identity emission is byte-exact
- **WHEN** the writer emits text for a parsed spec file with no edit applied
- **THEN** the emitted text equals the source byte for byte, including prose, line endings, and table padding

#### Scenario: Spans cover every editable element
- **WHEN** a spec file is parsed
- **THEN** every id cell, every `[[link]]` occurrence, and every state bullet carries a byte span the writer can rewrite

#### Scenario: Untouched bytes survive an edit
- **WHEN** an edit rewrites one id cell of a file whose prose mentions the old id as a word
- **THEN** the prose bytes, line terminators, and table padding outside the rewritten span are identical to the source

### Requirement: Edits realize typed morphisms
The acset-writer SHALL apply an edit (a morphism between instances) to
source text by returning a write-set of `(path, full new contents)` plus
optional removals, performing no I/O itself — applying the write-set is
the caller's single transaction.

#### Scenario: Text realizes the renamed instance
- **WHEN** a rename morphism f : I → I' is applied to source text x
- **THEN** parsing the emitted text yields exactly I'

#### Scenario: Edits compose
- **WHEN** two edits f and g are applied in sequence to x
- **THEN** the result equals applying their composition in one step

#### Scenario: Intent rename moves the file
- **WHEN** an edit renames a file's own Intent id
- **THEN** the write-set maps the file to the filename implied by the new id and returns one removal for the old path

#### Scenario: No realignment side effect
- **WHEN** an id edit changes a cell's width in an aligned table
- **THEN** surrounding padding is left unchanged — the column may become unaligned, and realignment is a separate explicit operation

#### Scenario: Writer performs no I/O
- **WHEN** the writer applies an edit
- **THEN** it returns a write-set and performs no file operations itself

### Requirement: Writer error contract
The acset-writer SHALL report its three failure modes — span extraction,
edit application, and roundtrip verification — as labeled errors carrying
remediation hints, surfaced through the consuming command's envelope per
the error contract.

#### Scenario: Span failure is labeled
- **WHEN** the writer cannot extract a recorded span it needs (e.g. the source changed under the parse)
- **THEN** the failure is labeled `acset.writer.span_failure` with a remediation hint

#### Scenario: Apply failure is labeled
- **WHEN** an edit names an id no recorded span can realize
- **THEN** the failure is labeled `acset.writer.apply_failure` with a remediation hint

#### Scenario: Roundtrip failure is labeled
- **WHEN** the emitted text reparses to an instance other than the edit's target
- **THEN** the failure is labeled `acset.writer.roundtrip_failure` with a remediation hint

## Requirements

### Requirement: Span-preserving emission
The acset-writer SHALL emit markdown for an instance from recorded byte
spans alone — never re-serializing a table — such that emission without
an edit reproduces the source byte for byte, and an edit changes only
the cells and links the edit names.

#### Scenario: Identity emission is byte-exact
- **WHEN** the writer emits text for a parsed spec file with no edit applied
- **THEN** the emitted text equals the source byte for byte, including prose, line endings, and table padding

#### Scenario: Spans cover every editable element
- **WHEN** a spec file is parsed
- **THEN** every id cell, every `[[link]]` occurrence, and every state bullet carries a byte span the writer can rewrite

#### Scenario: Untouched bytes survive an edit
- **WHEN** an edit rewrites one id cell of a file whose prose mentions the old id as a word
- **THEN** the prose bytes, line terminators, and table padding outside the rewritten span are identical to the source

### Requirement: Edits realize typed morphisms
The acset-writer SHALL apply an edit (a morphism between instances) to
source text by returning a write-set of `(path, full new contents)` plus
optional removals, performing no I/O itself — applying the write-set is
the caller's single transaction.

#### Scenario: Text realizes the renamed instance
- **WHEN** a rename morphism f : I → I' is applied to source text x
- **THEN** parsing the emitted text yields exactly I'

#### Scenario: Edits compose
- **WHEN** two edits f and g are applied in sequence to x
- **THEN** the result equals applying their composition in one step

#### Scenario: Intent rename moves the file
- **WHEN** an edit renames a file's own Intent id
- **THEN** the write-set maps the file to the filename implied by the new id and returns one removal for the old path

#### Scenario: No realignment side effect
- **WHEN** an id edit changes a cell's width in an aligned table
- **THEN** surrounding padding is left unchanged — the column may become unaligned, and realignment is a separate explicit operation

#### Scenario: Writer performs no I/O
- **WHEN** the writer applies an edit
- **THEN** it returns a write-set and performs no file operations itself

### Requirement: Writer error contract
The acset-writer SHALL report its three failure modes — span extraction,
edit application, and roundtrip verification — as labeled errors carrying
remediation hints, surfaced through the consuming command's envelope per
the error contract.

#### Scenario: Span failure is labeled
- **WHEN** the writer cannot extract a recorded span it needs (e.g. the source changed under the parse)
- **THEN** the failure is labeled `acset.writer.span_failure` with a remediation hint

#### Scenario: Apply failure is labeled
- **WHEN** an edit names an id no recorded span can realize
- **THEN** the failure is labeled `acset.writer.apply_failure` with a remediation hint

#### Scenario: Roundtrip failure is labeled
- **WHEN** the emitted text reparses to an instance other than the edit's target
- **THEN** the failure is labeled `acset.writer.roundtrip_failure` with a remediation hint
