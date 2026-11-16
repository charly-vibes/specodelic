//! Content advisories (openspec/specs/linter-content_advisories/spec.md):
//! whole-cell placeholders, tautological predicates, and the two vacuity
//! shapes (specodelic-h5ks).
//!
//! Every class is advisory. They ride [`Report::warnings`] — the success
//! envelope's warnings channel, exit 0 — and never [`Report::issues`], so a
//! corpus with filler content still lints green. The placeholder/tautology
//! checker judges WHOLE cells only: detection is a prefix/equality test on
//! the stripped cell body, never a substring search inside an executable
//! fragment (a `**rust:**` cell's interior is legitimate code —
//! `todo_predicate!` and a string containing `TODO` stay silent).
//!
//! The vacuity advisories judge the FRAGMENT SURFACE instead, and both are
//! citation checks against the file's declared model-element ids — never
//! coverage or reachability. A `**rust:**` fragment that cites no declared
//! state/transition/constraint/property binds nothing (green yet vacuous:
//! the bowling-rev1 shape — fragments that never called the scorer); the
//! finding is FILE-SCOPED, one summary per file naming every offending
//! row. Citation is textual and deliberately narrow: string/char literals
//! and `//` comments are stripped for identifier coincidence (except a
//! whole unescaped literal that exactly names a declared `Model/States`
//! id, which IS a citation — an invariant guarding a declared state is
//! genuinely bound, while an undeclared state literal never binds and is
//! `vacuous_invariant`'s surface), a path-qualified `::` call exempts the
//! fragment (it invokes production code), and one-/two-character ids
//! never satisfy the identifier check. An invariant fragment that forbids
//! a state literal not declared in `Model/States` is structurally vacuous:
//! the bad state cannot exist. Declared-but-unreachable bad states are
//! legitimate design pressure — a phase model may keep a defensive
//! invariant over a phase its correct runs never enter — so they must
//! never warn (the citation target is the declaration, never the
//! transition relation's reach).

use std::collections::BTreeSet;

use crate::spec::{Row, Spec};

use super::rules::file_label;
use super::{Issue, Report, declared_rows, transition_ids};

/// `linter.placeholder_content` — the bare rule name.
const PLACEHOLDER_RULE: &str = "placeholder_content";
/// `linter.tautological_predicate` — the bare rule name.
const TAUTOLOGY_RULE: &str = "tautological_predicate";
/// `linter.fragment_binds_nothing` — the bare rule name (specodelic-h5ks).
const BINDS_NOTHING_RULE: &str = "fragment_binds_nothing";
/// `linter.vacuous_invariant` — the bare rule name (specodelic-h5ks).
const VACUOUS_INVARIANT_RULE: &str = "vacuous_invariant";

/// The executable invariant fragment's state binder
/// (`fn inv_N(state: &str) -> bool` in the model-check scratch crate):
/// only `state` comparisons are judged for the forbidden-state check.
const STATE_BINDER: &str = "state";

/// The `spk new` stub intent statement, in both the ellipsis-character and
/// three-ASCII-dots spellings.
const STUB_STATEMENTS: [&str; 2] = ["THE system SHALL …", "THE system SHALL ..."];

/// The whole-cell placeholder words (`placeholder_classes`): each must be
/// followed by the end of the cell or a separator and an optional note —
/// `TODOESCAPE_IS_FINE` and a mid-cell `TODO` are not placeholders.
const PLACEHOLDER_WORDS: [&str; 3] = ["TODO", "TBD", "FIXME"];

/// The `**rust:**` fragment marker a cell may lead with. A cell that begins
/// with it is executable code, so its interior is never scanned for
/// placeholder words.
const RUST_MARKER: &str = "**rust:**";

/// `scaffold_*` row ids are the `spk new` scaffold's untouched rows — only
/// in a file that still carries the scaffold signature (see
/// [`carries_scaffold_signature`]).
const SCAFFOLD_PREFIX: &str = "scaffold_";

/// Warn — never fail — on every whole-cell placeholder, tautological
/// predicate, and vacuous invariant in the corpus. One warning per
/// offending cell (or offending `scaffold_*` row id, or undeclared
/// forbidden state), and ONE file-scoped summary for fragments that
/// bind nothing; nothing is ever pushed to `report.issues`.
pub(crate) fn lint_content_advisories(specs: &[Spec], report: &mut Report) {
    for spec in specs {
        let file = file_label(spec);
        let scaffold = carries_scaffold_signature(spec);

        inspect_statement(report, &file, &spec.intent.statement);

        for row in &spec.constraints {
            // Only an INVARIANT expr must be able to fail: an `effect` or
            // `advisory` expr is a label/hint, so the tautology class
            // (spec `tautology_class`) never judges it.
            let judge_tautology = row.kind.as_deref() == Some("invariant");
            inspect_row_cell(report, &file, "constraint", row, "expr", judge_tautology);
        }
        for row in &spec.properties {
            inspect_row_cell(report, &file, "property", row, "predicate", true);
        }
        inspect_vacuity(spec, &file, report);
        if !scaffold {
            continue;
        }
        for row in &spec.constraints {
            warn_scaffold_id(report, &file, "constraint", &row.id);
        }
        for row in &spec.properties {
            warn_scaffold_id(report, &file, "property", &row.id);
        }
        for row in &spec.states {
            warn_scaffold_id(report, &file, "state", &row.id);
        }
        for transition in &spec.transitions {
            warn_scaffold_id(report, &file, "transition", &transition.id);
        }
    }
}

/// Whether the file still carries a scaffold signature: its intent
/// statement is a whole-cell placeholder, or at least one structured
/// content cell (constraint `expr` / property `predicate`) is a
/// placeholder or a tautology.
///
/// Both real producers qualify: the `spk new` stub/TODO cells, and
/// `spk migrate`'s sidecar — `scaffold_constraint` / `scaffold_transition`
/// / `scaffold_property` ids whose cells are bare `true`, beside a prose
/// statement that is not itself a placeholder. That is why the signature
/// cannot be "stub statement only".
///
/// `placeholder_classes` gates the row-id class on this signature: a real
/// row that merely NAMES the scaffold (the corpus's own `scaffold_teaches`,
/// `scaffold_lints_clean`) is documentation, not filler.
fn carries_scaffold_signature(spec: &Spec) -> bool {
    is_placeholder(strip_code_span(&spec.intent.statement))
        || spec
            .constraints
            .iter()
            .any(|row| cell_is_scaffold_content(row, "expr"))
        || spec
            .properties
            .iter()
            .any(|row| cell_is_scaffold_content(row, "predicate"))
}

/// Is `row`'s `column` cell a whole-cell placeholder or a tautology (code
/// span stripped)? Kind-agnostic on purpose: the signature says "this file
/// is scaffold", not "this cell is wrong" — the per-cell judging below
/// applies the invariant-only tautology gate.
fn cell_is_scaffold_content(row: &Row, column: &str) -> bool {
    row.cells.get(column).is_some_and(|cell| {
        let body = strip_code_span(cell);
        is_placeholder(body) || is_tautology(strip_rust_marker(body))
    })
}

/// Judge the intent statement as a whole-cell placeholder only. The
/// tautology class (`tautology_class`) covers a property predicate and an
/// invariant constraint expr — never a statement, whose EARS grammar is
/// judged elsewhere.
fn inspect_statement(report: &mut Report, file: &str, statement: &str) {
    let body = strip_code_span(statement);
    if is_placeholder(body) {
        report.warnings.push(Issue::new(
            PLACEHOLDER_RULE,
            file,
            format!("intent statement is a whole-cell placeholder: {body}"),
        ));
    }
}

/// Judge one structured content cell (`expr` on a constraint, `predicate`
/// on a property): the placeholder class always, the tautology class only
/// when `judge_tautology` (an invariant constraint expr, or a property
/// predicate). The message names the row id — the anchor an author needs
/// to fix the cell — and the offending text.
fn inspect_row_cell(
    report: &mut Report,
    file: &str,
    row_kind: &str,
    row: &Row,
    column: &str,
    judge_tautology: bool,
) {
    let Some(cell) = row.cells.get(column) else {
        return;
    };
    let body = strip_code_span(cell);
    if is_placeholder(body) {
        report.warnings.push(Issue::new(
            PLACEHOLDER_RULE,
            file,
            format!(
                "{row_kind} `{}` {column} is a whole-cell placeholder: {body}",
                row.id
            ),
        ));
    } else if judge_tautology && is_tautology(strip_rust_marker(body)) {
        report.warnings.push(Issue::new(
            TAUTOLOGY_RULE,
            file,
            format!("{row_kind} `{}` {column} is tautological: {body}", row.id),
        ));
    }
}

/// Warn on an untouched scaffold row: its id carries the `scaffold_`
/// placeholder prefix (`placeholder_classes`) inside a file that still
/// carries the scaffold signature, whatever the row's own cells say.
fn warn_scaffold_id(report: &mut Report, file: &str, row_kind: &str, id: &str) {
    if id.starts_with(SCAFFOLD_PREFIX) {
        report.warnings.push(Issue::new(
            PLACEHOLDER_RULE,
            file,
            format!("{row_kind} id `{id}` carries the `scaffold_` placeholder prefix"),
        ));
    }
}

/// The two vacuity advisories (specodelic-h5ks), over the executable
/// fragment surface only:
///
/// - `fragment_binds_nothing` — every invariant constraint expr and every
///   non-law property predicate that extracts a `**rust:**` fragment must
///   cite at least one declared model element (a state/transition/
///   constraint/property row id of ≥3 characters; the intent id is the
///   file's own identity, not a model element). The finding is
///   FILE-SCOPED: one summary warning per file names the count and every
///   offending row id. Citation is a textual proxy (see
///   [`fragment_cites_declared_id`]): literals and line comments are
///   stripped for identifier coincidence, except a whole unescaped
///   literal exactly naming a declared `Model/States` id — an executable
///   invariant comparing the `state` binder against a declared state is
///   genuinely bound to the model, while an undeclared state literal is
///   not (that is `vacuous_invariant`'s surface) — and a path-qualified
///   `::` call exempts the fragment.
///   Law-kind predicates are skipped: their `**label:**` occurrences are
///   the law-case grammar, never fragment markers (the
///   validate_fragments discipline, mirrored from audit's evidence-unit
///   collection).
/// - `vacuous_invariant` — an invariant fragment's `state != "<literal>"`
///   must name a state declared under `Model/States`. An UNDECLARED
///   forbidden state cannot exist, so the claim can never fail. A state
///   that IS declared but is never entered by the transition relation is
///   NOT flagged: a legitimate phase-style model may keep a defensive
///   invariant over a phase its correct runs never reach, and this check
///   is citation-versus-declared-ids — never reachability from any seed
///   state, never coverage-of-declared-ids. That is the study's
///   declared-bad-state nuance, and the reason the advisory must never
///   nag an author off good design pressure.
///
/// Both are warnings-channel findings only: `Issue`s pushed to
/// `report.issues` would make the run exit nonzero, which neither
/// advisory may ever do.
fn inspect_vacuity(spec: &Spec, file: &str, report: &mut Report) {
    let declared = declared_model_ids(spec);
    let states: BTreeSet<&str> = spec.states.iter().map(|s| s.id.as_str()).collect();

    // Extracted fragments, collected immutably: the invariant pass needs
    // them twice (the forbidden-state judge and the binding summary), so
    // one extraction serves both.
    let invariant_fragments: Vec<(&Row, String)> = spec
        .constraints
        .iter()
        .filter(|row| row.kind.as_deref() == Some("invariant"))
        .filter_map(|row| {
            row.cells
                .get("expr")
                .and_then(|value| crate::compile::fragment_of(value))
                .map(|fragment| (row, fragment))
        })
        .collect();
    // A law's per-case bodies are the law-case grammar, not one fragment
    // cell (audit skips them for the same reason).
    let property_fragments: Vec<(&Row, String)> = spec
        .properties
        .iter()
        .filter(|row| row.kind.as_deref() != Some("law"))
        .filter_map(|row| {
            row.cells
                .get("predicate")
                .and_then(|value| crate::compile::fragment_of(value))
                .map(|fragment| (row, fragment))
        })
        .collect();

    for (row, fragment) in &invariant_fragments {
        warn_undeclared_forbidden_states(report, file, row, fragment, &states);
    }

    let bindings_nothing: Vec<&str> = invariant_fragments
        .into_iter()
        .chain(property_fragments)
        .filter(|(_, fragment)| !fragment_cites_declared_id(fragment.as_str(), &declared, &states))
        .map(|(row, _)| row.id.as_str())
        .collect();
    warn_binds_nothing(report, file, &bindings_nothing);
}

/// The file's declared model-element ids — the citation target set:
/// every state/transition/constraint/property row id (the ONE shared
/// walk lint's graph projections already use; transitions are a typed
/// table apart from the `Row` walk). The intent id is deliberately
/// absent: a fragment citing only its own file id binds nothing.
fn declared_model_ids(spec: &Spec) -> BTreeSet<&str> {
    declared_rows(spec)
        .map(|(_, row)| row.id.as_str())
        .chain(transition_ids(spec))
        .collect()
}

/// The file-scoped vacuity advisory (specodelic-h5ks): ONE warning per
/// file summarizing every fragment that cites no declared model
/// element, naming each offending row id. The advisory is deliberately
/// file-scoped, not per-row: a fragment-heavy file that never touches
/// the model produces one actionable summary, while the row-id list
/// keeps every offender visible (the fix-review shrank a 53-row list
/// this way). Per-row detail has no separate verbose surface in the
/// library form — no flag is added for it.
fn warn_binds_nothing(report: &mut Report, file: &str, offending: &[&str]) {
    if offending.is_empty() {
        return;
    }
    let ids = offending
        .iter()
        .map(|id| format!("`{id}`"))
        .collect::<Vec<_>>()
        .join(", ");
    report.warnings.push(Issue::new(
        BINDS_NOTHING_RULE,
        file,
        format!(
            "{} fragment(s) cite no declared model element — a vacuous binding (green yet vacuous: nothing was checked); name a declared state/transition/constraint/property (≥3 characters) to bind each, or drop the fragment — advisory only, a model whose declared elements are simply not exercised is never flagged: {ids}",
            offending.len()
        ),
    ));
}

/// Warn on each `state != "<literal>"` whose literal is not a declared
/// state: the bad state cannot exist, so the invariant is structurally
/// vacuous. A declared-but-unreachable state stays silent — see
/// [`inspect_vacuity`] for the phase-model nuance.
fn warn_undeclared_forbidden_states(
    report: &mut Report,
    file: &str,
    row: &Row,
    fragment: &str,
    states: &BTreeSet<&str>,
) {
    let undeclared: BTreeSet<String> = forbidden_state_literals(fragment)
        .into_iter()
        .filter(|literal| !states.contains(literal.as_str()))
        .collect();
    for literal in undeclared {
        report.warnings.push(Issue::new(
            VACUOUS_INVARIANT_RULE,
            file,
            format!(
                "constraint `{}` invariant forbids state `{literal}`, which is not declared in Model/States — the bad state cannot exist, so the invariant is structurally vacuous (advisory only: a DECLARED but unreachable phase state is legitimate design pressure and never warns)",
                row.id
            ),
        ));
    }
}

/// The minimum length (in characters) of a declared id that can satisfy
/// the citation check. A one- or two-character row id (`t`, `p`, `s1`)
/// occurs inside fragments by accident — requiring a meaningful
/// identifier (`≥3` characters) keeps the textual proxy from declaring
/// a fragment bound merely because a single letter happens to appear.
const MIN_CITATION_TOKEN_LEN: usize = 3;

/// Whether `fragment` cites any declared id — a deliberately TEXTUAL
/// proxy, documented as such:
///
/// - whole-token matching only, never substring: `unlocked` does not
///   cite `lock`; dotted references cite a declared id through their
///   segments (`.` is a token boundary);
/// - string and char literals plus `//` line comments are stripped
///   before IDENTIFIER matching, so a declared-looking name inside `"…"`
///   or a commented-out expression is not an identifier coincidence.
///   ONE literal exception: a whole unescaped string literal whose
///   content exactly equals a DECLARED `Model/States` id counts as a
///   citation — a fragment comparing the `state` binder against a
///   declared state is genuinely bound to the model. Literal equality is
///   exact and length-independent (state ids like `s1` are fine here;
///   the ≥3-character floor is about identifier tokens only). An
///   UNDECLARED state literal never counts: that is the
///   `vacuous_invariant` rule's surface, so such a fragment still binds
///   nothing and both advisories may name it;
/// - a fragment containing a path-qualified call (`::`) is EXEMPT: it
///   invokes real production code, so it is checking something and must
///   never be labeled vacuous (invoking `specodelic::…` is a binding to
///   behaviour even when no row id appears textually);
/// - tokens shorter than [`MIN_CITATION_TOKEN_LEN`] never satisfy the
///   identifier check (see that constant's rationale).
///
/// This is not semantic analysis: it cannot know whether the cited row
/// is *relevant* to the fragment. That is the point of the warning — it
/// surfaces a fragment that binds nothing textual at all.
fn fragment_cites_declared_id(
    fragment: &str,
    declared: &BTreeSet<&str>,
    states: &BTreeSet<&str>,
) -> bool {
    let (tokens, has_path_call, literals) = citation_tokens(fragment);
    if has_path_call {
        return true;
    }
    tokens
        .into_iter()
        .any(|token| token.chars().count() >= MIN_CITATION_TOKEN_LEN && declared.contains(token))
        || literals.into_iter().any(|literal| states.contains(literal))
}

/// The identifier tokens and unescaped string-literal contents a
/// fragment's CODE carries — char literals and `//` line comments are
/// skipped outright — plus whether that code contains a path-qualified
/// `::` call/use. Literal CONTENTS are returned (quotes excluded, escaped
/// literals omitted) because a state literal can itself cite a declared
/// state; they are never token-matched. Raw strings and block comments
/// are outside this narrow scan (fragments use neither); the scan is a
/// citation proxy, not a parser.
fn citation_tokens(fragment: &str) -> (Vec<&str>, bool, Vec<&str>) {
    let mut tokens: Vec<&str> = Vec::new();
    let mut literals: Vec<&str> = Vec::new();
    let mut has_path_call = false;
    let mut rest = fragment;
    while let Some(c) = rest.chars().next() {
        if c == '/' && rest.starts_with("//") {
            rest = rest.find('\n').map_or("", |i| &rest[i..]);
            continue;
        }
        if c == '"' {
            let (literal, tail) = take_string_literal(rest);
            let content = literal
                .strip_prefix('"')
                .and_then(|inner| inner.strip_suffix('"'));
            if let Some(content) = content.filter(|inner| !inner.contains('\\')) {
                literals.push(content);
            }
            rest = tail;
            continue;
        }
        if c == '\''
            && let Some(len) = char_literal_len(rest)
        {
            rest = &rest[len..];
            continue;
        }
        if c == ':' && rest[1..].starts_with(':') {
            has_path_call = true;
            rest = &rest[2..];
            continue;
        }
        if c.is_ascii_alphanumeric() || c == '_' {
            let end = rest
                .char_indices()
                .find(|(_, ch)| !(ch.is_ascii_alphanumeric() || *ch == '_'))
                .map_or(rest.len(), |(i, _)| i);
            tokens.push(&rest[..end]);
            rest = &rest[end..];
            continue;
        }
        rest = &rest[c.len_utf8()..];
    }
    (tokens, has_path_call, literals)
}

/// Split off the double-quoted string literal at the start of `rest` —
/// `(literal including quotes, tail)`; an unterminated literal swallows
/// the remainder (the fragment is already compile-validated upstream).
fn take_string_literal(rest: &str) -> (&str, &str) {
    let mut escaped = false;
    for (i, c) in rest.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' => escaped = true,
            '"' => return (&rest[..i + 1], &rest[i + 1..]),
            _ => {}
        }
    }
    (rest, "")
}

/// Byte length of the Rust char literal at the start of `rest` —
/// `'x'`, `'\n'`, `'\u{1F600}'` — or `None` when the `'` is a lifetime
/// apostrophe rather than a literal to strip.
fn char_literal_len(rest: &str) -> Option<usize> {
    let mut chars = rest.char_indices();
    let (_, quote) = chars.next()?;
    if quote != '\'' {
        return None;
    }
    let (_, first) = chars.next()?;
    if first != '\\' {
        let after = 1 + first.len_utf8();
        return rest[after..].starts_with('\'').then_some(after + 1);
    }
    let (_, escaped) = rest[2..].char_indices().next()?;
    let scan_from = 2 + escaped.len_utf8();
    let mut depth = 0usize;
    let mut prev = escaped;
    for (i, c) in rest[scan_from..].char_indices() {
        match c {
            '{' if prev == 'u' => depth += 1,
            '}' if depth > 0 => depth -= 1,
            '\'' if depth == 0 => return Some(scan_from + i + c.len_utf8()),
            _ => {}
        }
        prev = c;
    }
    None
}

/// The fragment with `//` line comments removed — the string-aware scan
/// keeps a `//` inside a literal (`"https://…"`) intact, so only a real
/// comment can hide a state comparison. Comments are line-scoped (the
/// narrow scan's scope, never a multi-line string body).
fn strip_line_comments(fragment: &str) -> String {
    fragment
        .split('\n')
        .map(|line| &line[..line_comment_start(line).unwrap_or(line.len())])
        .collect::<Vec<_>>()
        .join("\n")
}

/// The byte index where a `//` comment starts on `line`, or `None` —
/// string-aware, so `"https://…"` is not mistaken for a comment. A
/// `fold` state machine, not a mutable accumulator.
fn line_comment_start(line: &str) -> Option<usize> {
    let (_, _, at) = line.char_indices().fold(
        (false, false, None::<usize>),
        |(in_string, escaped, at), (i, c)| {
            if at.is_some() {
                return (in_string, escaped, at);
            }
            if in_string {
                return match c {
                    '\\' if !escaped => (true, true, None),
                    '"' if !escaped => (false, false, None),
                    _ => (true, false, None),
                };
            }
            if c == '"' {
                return (true, false, None);
            }
            if c == '/' && line[i..].starts_with("//") {
                return (false, false, Some(i));
            }
            (false, false, None)
        },
    );
    at
}

/// The string literals `fragment` forbids through `state != "<lit>"`
/// (or its mirror `"<lit>" != state`). `//` line comments are removed
/// first, so a commented-out `// state != "aborted"` never warns while
/// a `//` inside a literal stays intact. Narrow by design: only the
/// exactly-named `state` binder's string inequality is judged — other
/// binders, `model.state`/`state.field`/`crate::state` paths,
/// arithmetic, calls, `==`, and escaped/raw literals stay silent.
fn forbidden_state_literals(fragment: &str) -> Vec<String> {
    let code = strip_line_comments(fragment);
    let code = code.as_str();
    unescaped_literal_spans(code)
        .into_iter()
        .filter(|(start, end)| {
            let quote = *start - 1;
            !code[*start..*end].contains('\\')
                && !matches!(code[..quote].chars().next_back(), Some('r' | 'b'))
                && is_forbidden_state_comparison(code, quote, *end)
        })
        .map(|(start, end)| code[start..end].to_string())
        .collect()
}

/// The inner byte spans of the unescaped double-quoted string literals
/// in `code` (the quote characters excluded), in source order. A `scan`
/// state machine tracks `\\` escapes without a mutable accumulator; an
/// escaped literal still yields its span (the caller's narrow scan
/// rejects it), and an unterminated literal yields none.
fn unescaped_literal_spans(code: &str) -> Vec<(usize, usize)> {
    code.char_indices()
        .scan(
            (None::<usize>, false),
            |(open, escaped), (i, c)| -> Option<Option<(usize, usize)>> {
                if let Some(start) = *open {
                    if *escaped {
                        *escaped = false;
                        return Some(None);
                    }
                    match c {
                        '\\' => *escaped = true,
                        '"' => {
                            *open = None;
                            return Some(Some((start, i)));
                        }
                        _ => {}
                    }
                    return Some(None);
                }
                if c == '"' {
                    *open = Some(i + 1);
                }
                Some(None)
            },
        )
        .flatten()
        .collect()
}

/// Is the string literal spanning `quote_start..=quote_end` the operand
/// of a `state != …` (or `… != state`) comparison?
fn is_forbidden_state_comparison(fragment: &str, quote_start: usize, quote_end: usize) -> bool {
    let before = &fragment[..quote_start];
    if let Some(operand) = before.trim_end().strip_suffix("!=") {
        return ends_with_state_binder(operand);
    }
    let after = &fragment[quote_end + 1..];
    if let Some(operand) = after.trim_start().strip_prefix("!=") {
        return starts_with_state_binder(operand);
    }
    false
}

/// Does `text` end with the bare `state` identifier (word boundary
/// before it)? `my_state`, `model.state`, `crate::state`, and
/// `state_extra` are other names, not the binder.
fn ends_with_state_binder(text: &str) -> bool {
    let before = text.trim_end();
    match before.strip_suffix(STATE_BINDER) {
        Some(prefix) => !prefix
            .ends_with(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == ':'),
        None => false,
    }
}

/// Does `text` start with the bare `state` identifier (word boundary
/// after it)?
fn starts_with_state_binder(text: &str) -> bool {
    let after = text.trim_start();
    match after.strip_prefix(STATE_BINDER) {
        Some(rest) => !rest
            .starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == ':'),
        None => false,
    }
}

/// Strip surrounding whitespace and ONE surrounding backtick code span, so
/// `` `TODO — note` `` and `TODO — note` judge identically.
fn strip_code_span(cell: &str) -> &str {
    let trimmed = cell.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('`') && trimmed.ends_with('`') {
        trimmed[1..trimmed.len() - 1].trim()
    } else {
        trimmed
    }
}

/// Strip a leading `**rust:**` fragment marker plus the whitespace after it
/// (tautology judging only — a fragment may be exactly `true`).
fn strip_rust_marker(body: &str) -> &str {
    body.strip_prefix(RUST_MARKER)
        .map(str::trim)
        .unwrap_or(body)
}

/// A whole-cell placeholder: the `spk new` stub statement, or a cell that
/// opens with `TODO`/`TBD`/`FIXME`. A `**rust:**` cell is executable code —
/// only the stub form (whose whole cell IS the placeholder) is judged
/// inside one.
fn is_placeholder(body: &str) -> bool {
    if is_stub_statement(body) {
        return true;
    }
    if body.starts_with(RUST_MARKER) {
        return false;
    }
    PLACEHOLDER_WORDS
        .iter()
        .any(|word| opens_with_placeholder_word(body, word))
}

/// Is the whole cell the untouched `spk new` stub intent statement?
fn is_stub_statement(body: &str) -> bool {
    STUB_STATEMENTS.contains(&body)
}

/// `word` followed by end-of-cell or a separator (`:`, `-`, `—`) or
/// whitespace before an optional note. Case-sensitive by design: `TODOES`
/// and `todo` are prose, not markers.
fn opens_with_placeholder_word(body: &str, word: &str) -> bool {
    let Some(rest) = body.strip_prefix(word) else {
        return false;
    };
    match rest.chars().next() {
        None => true,
        Some(c) => matches!(c, ':' | '-' | '—') || c.is_whitespace(),
    }
}

/// The narrow tautology class: the whole body is exactly `true`/`⊤`, or one
/// identifier compared equal to itself. `x == y` and `x == x + 1` are
/// deliberately outside it — judging meaning is not this checker's beat.
fn is_tautology(body: &str) -> bool {
    matches!(body, "true" | "TRUE" | "True" | "⊤") || is_self_comparison(body)
}

/// Exactly `A == A` where `A` is one identifier (`[A-Za-z_][A-Za-z0-9_]*`).
/// No calls, arithmetic, or compound expressions — an extra `==` or any
/// non-identifier operand stays silent.
fn is_self_comparison(body: &str) -> bool {
    let mut parts = body.split("==");
    let (Some(lhs), Some(rhs)) = (parts.next(), parts.next()) else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }
    let lhs = lhs.trim();
    let rhs = rhs.trim();
    lhs == rhs && is_identifier(lhs)
}

/// `[A-Za-z_][A-Za-z0-9_]*` — ASCII only, no calls or indexing.
fn is_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
