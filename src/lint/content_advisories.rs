//! Content advisories (openspec/specs/linter-content_advisories/spec.md):
//! whole-cell placeholders and tautological predicates.
//!
//! Both classes are advisory. They ride [`Report::warnings`] — the success
//! envelope's warnings channel, exit 0 — and never [`Report::issues`], so a
//! corpus with filler content still lints green. The checker judges WHOLE
//! cells only: detection is a prefix/equality test on the stripped cell
//! body, never a substring search inside an executable fragment (a
//! `**rust:**` cell's interior is legitimate code — `todo_predicate!` and a
//! string containing `TODO` stay silent).

use crate::spec::{Row, Spec};

use super::rules::file_label;
use super::{Issue, Report};

/// `linter.placeholder_content` — the bare rule name.
const PLACEHOLDER_RULE: &str = "placeholder_content";
/// `linter.tautological_predicate` — the bare rule name.
const TAUTOLOGY_RULE: &str = "tautological_predicate";

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

/// Warn — never fail — on every whole-cell placeholder and tautological
/// predicate in the corpus. One warning per offending cell (or offending
/// `scaffold_*` row id); nothing is ever pushed to `report.issues`.
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
