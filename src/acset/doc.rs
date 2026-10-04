//! The format doc's Reference Typing table, read as rows (`add-acset-core`
//! task 2.4 — the `schema_matches_typing_table` lint gate).
//!
//! Purpose: parse the Reference Typing section of the format doc
//! (captured verbatim into [`crate::spec::Spec::reference_typing_body`])
//! into typing rows and compare them, row for row, with the canonical
//! [`Schema`] — so the document and the code cannot drift silently: a lint
//! finding fires on divergence, in either direction. Rationale: the Schema
//! is the single data value (tasks 2.1–2.3); this module is its
//! document-side mirror, and the lint gate is the enforcement that keeps
//! the mirror aligned (design.md decision 3 — drift is lint-visible,
//! mirroring how the format's other closed value sets are policed).
//!
//! Parsing discipline (extractive, not interpretive): each cell's typing
//! scope is scanned for the closed object names — every occurrence is an
//! allowed-target claim — plus the refinement phrases the table's grammar
//! actually uses. Anything that yields no rows, or carries refinements the
//! reader cannot attribute, is an uninterpretable cell and becomes an
//! error: an unrecognized doc edit is a drift signal by construction,
//! never silence. Two use-mention conventions make prose-safe extraction
//! possible without reading full paragraphs:
//!
//! - Scope: only the cell's first sentence is scanned (up to the first
//!   `;` or `. `) — the typing claim lives there; later sentences explain
//!   (and freely name other objects, e.g. `observes`'s prose mentioning
//!   `Intent`).
//! - Quoting: a backticked adjective before an object name (`` `advisory`
//!   Constraint ``) is a mention, not a typing claim; only a bare kind
//!   word (`an invariant Constraint`) refines.
//!
//! Compared: (column, source, target) triples plus the stated refinements.
//! Not compared (code-side mechanics the table does not carry): violation
//! prose templates, source rules, endo-acyclicity flags.

use std::collections::{BTreeMap, BTreeSet};

use super::schema::{CLOSED_OBJECTS, Morphism, Schema};

/// One doc-side Reference Typing row: the typing claim a table row makes,
/// extracted. Refinement kinds are `Option`-carried because the doc states
/// them in prose; `None` means "none stated", not "none enforced".
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypingRow {
    /// The reference field (backticks stripped); a compound cell splits on
    /// `/` (`from` / `to` → two rows).
    pub column: String,
    /// The Appears-on object.
    pub source: String,
    /// The Must-resolve-to object for this row (a compound cell splits —
    /// one row per object it names).
    pub target: String,
    /// Target-side refinement kind stated by the cell (`kind == `effect``
    /// / `` `kind: profile` `` / a bare kind adjective).
    pub target_refinement: Option<String>,
    /// Source-side refinement kind stated by the cell ("the deriving row
    /// is itself a `law`") — carried by the same-kind (self) row.
    pub source_refinement: Option<String>,
}

/// The comparison key of a [`TypingRow`] — the full typing claim a row
/// carries, refinements included so a narrowing cannot drift silently.
fn row_key(r: &TypingRow) -> String {
    format!(
        "{}: {} → {}{}{}",
        r.column,
        r.source,
        r.target,
        r.target_refinement
            .as_ref()
            .map(|k| format!(" [target must be kind `{k}`]"))
            .unwrap_or_default(),
        r.source_refinement
            .as_ref()
            .map(|k| format!(" [source must be kind `{k}`]"))
            .unwrap_or_default(),
    )
}

/// The comparison key of a [`Morphism`] — the Schema-side twin of
/// [`row_key`]: at most one target-side and one source-side refinement.
fn morphism_key(m: &Morphism) -> String {
    format!(
        "{}: {} → {}{}{}",
        m.column,
        m.source.0,
        m.target.0,
        m.refinements
            .iter()
            .find(|r| r.side == super::schema::Side::Target)
            .map(|r| format!(" [target must be kind `{}`]", r.kind))
            .unwrap_or_default(),
        m.refinements
            .iter()
            .find(|r| r.side == super::schema::Side::Source)
            .map(|r| format!(" [source must be kind `{}`]", r.kind))
            .unwrap_or_default(),
    )
}

/// The recognized refinement kinds — the closed kind sets the schema's
/// refinements read (constraint kinds + property kinds). A bare word from
/// this set immediately before an object name is a typing claim.
fn refinement_kinds() -> Vec<&'static str> {
    let mut kinds = crate::guide::CONSTRAINT_KINDS.to_vec();
    kinds.extend_from_slice(crate::guide::PROPERTY_KINDS);
    kinds
}

/// Whole-word containment: `word` occurs in `hay` bounded by
/// non-alphanumerics (or the string edges).
fn contains_word(hay: &str, word: &str) -> bool {
    let mut rest = hay;
    while let Some(pos) = rest.find(word) {
        let before_ok = pos == 0
            || !rest
                .as_bytes()
                .get(pos.wrapping_sub(1))
                .copied()
                .map(|b| b.is_ascii_alphanumeric())
                .unwrap_or(false);
        let after = pos + word.len();
        let after_ok = after >= rest.len()
            || !rest
                .as_bytes()
                .get(after)
                .copied()
                .map(|b| b.is_ascii_alphanumeric())
                .unwrap_or(false);
        if before_ok && after_ok {
            return true;
        }
        rest = &rest[pos + word.len()..];
    }
    false
}

/// Every closed object named in the cell, in order of appearance,
/// deduplicated.
fn object_names(cell: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for name in CLOSED_OBJECTS {
        if contains_word(cell, name) && !out.iter().any(|s| s == name) {
            out.push(name.to_string());
        }
    }
    out
}

/// The cell's typing scope: everything before the first `;` or `. `
/// (or trailing `.`). The typing claim lives in the first sentence; the
/// rest is explanation.
fn typing_scope(cell: &str) -> &str {
    let bytes = cell.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b';' || (b == b'.' && (i + 1 == cell.len() || bytes[i + 1] == b' ')) {
            return &cell[..i];
        }
    }
    cell
}

/// The refinement value after `needle`, if present. Two span shapes, one
/// rule: the value is either the next backtick span (`kind == `effect``,
/// `itself a `law`` — needle outside the span) or the text up to the next
/// backtick (`kind: profile` — the needle sits inside the span). `
/// Quoted spans elsewhere never leak in: the value terminates at the first
/// backtick after the needle.
fn backtick_after(scope: &str, needle: &str) -> Option<String> {
    let rest = scope[scope.find(needle)? + needle.len()..].trim_start();
    // needle outside the span (`` `effect` ``): the value is between the
    // opening backtick and the next one. needle inside the span
    // (`` `kind: profile` ``): the value is up to the closing backtick.
    let after_open = rest.strip_prefix('`');
    let from = match after_open {
        Some(within) => within,
        None => rest,
    };
    let close = from.find('`')?;
    let kind = from[..close].trim();
    if kind.is_empty() {
        None
    } else {
        Some(kind.to_string())
    }
}

/// A bare (unquoted) refinement-kind word immediately before an object
/// name — `an invariant Constraint`. A backticked word (`` `advisory`
/// Constraint ``) is a mention, never a claim.
fn bare_kind_before(scope: &str, object: &str) -> Option<String> {
    let pos = scope.find(object)?;
    let before = scope[..pos].trim_end();
    let last = before.split_whitespace().last()?;
    if last.starts_with('`') || last.ends_with('`') {
        return None;
    }
    if refinement_kinds().contains(&last) {
        Some(last.to_string())
    } else {
        None
    }
}

/// Parse the verbatim Reference Typing section body into rows.
///
/// `Err` lists every cell the reader could not interpret — including a
/// changed table header — so an unrecognized doc edit surfaces as a lint
/// finding instead of a silent pass.
pub fn parse_typing_table(body: &str) -> Result<Vec<TypingRow>, Vec<String>> {
    let table: Vec<&str> = body
        .lines()
        .filter(|l| l.trim_start().starts_with('|'))
        .collect();
    if table.is_empty() {
        return Err(vec![
            "no Reference Typing table found in the section".to_string(),
        ]);
    }
    let header: Vec<String> = match crate::spec::parse_row(table[0]) {
        Ok(h) => h.iter().map(|c| c.trim().to_string()).collect(),
        Err(m) => return Err(vec![format!("Reference Typing header unparseable: {m}")]),
    };
    if header.len() < 3
        || header[0] != "Field"
        || header[1] != "Appears on"
        || header[2] != "Must resolve to"
    {
        return Err(vec![format!(
            "Reference Typing table header changed — expected \
             `Field | Appears on | Must resolve to`, found `{}`",
            header.join(" | ")
        )]);
    }

    let mut errs: Vec<String> = Vec::new();
    let mut rows: Vec<TypingRow> = Vec::new();
    for line in &table[1..] {
        if crate::spec::is_separator(line) {
            continue;
        }
        let cells = match crate::spec::parse_row(line) {
            Ok(c) => c,
            Err(m) => {
                errs.push(format!("Reference Typing row unparseable: {m}"));
                continue;
            }
        };
        if cells.len() < 3 {
            errs.push(format!(
                "Reference Typing row `{}` does not have three cells",
                line.trim()
            ));
            continue;
        }
        // Field cell: backticks stripped, `/` splits compound fields.
        let columns: Vec<String> = cells[0]
            .split('/')
            .map(|c| c.trim().trim_matches('`').trim().to_string())
            .filter(|c| !c.is_empty())
            .collect();
        if columns.is_empty() {
            errs.push(format!(
                "cannot interpret Reference Typing Field cell `{}` — no field named",
                cells[0]
            ));
            continue;
        }
        // Appears-on cell: the named objects; qualifier prose ("any file")
        // is ignored — only closed object names are claims.
        let sources = object_names(&cells[1]);
        if sources.is_empty() {
            errs.push(format!(
                "cannot interpret Reference Typing Appears-on cell `{}` — no known object named",
                cells[1]
            ));
            continue;
        }
        // Must-resolve-to cell: first-sentence scope, then extract.
        let scope = typing_scope(&cells[2]);
        let same_kind = scope.contains("same kind as the row it appears on");
        let targets = if same_kind {
            // The cell names the objects in a parenthetical restatement;
            // the claim is "target == source", derived per source below.
            Vec::new()
        } else {
            let t = object_names(scope);
            if t.is_empty() {
                errs.push(format!(
                    "cannot interpret Reference Typing Must-resolve-to cell `{}` — no known object named",
                    cells[2]
                ));
                continue;
            }
            t
        };

        // Refinements, attributed:
        // - cell-wide `kind == `X`` and `` `kind: X` `` spans,
        // - a bare kind adjective immediately before a target name,
        // - source-side "itself a `X`" for the same-kind (self) rows.
        let cell_wide =
            backtick_after(scope, "kind == ").or_else(|| backtick_after(scope, "kind: "));
        let mut per_target: BTreeMap<String, String> = BTreeMap::new();
        for t in &targets {
            if let Some(kind) = bare_kind_before(scope, t) {
                per_target.insert(t.clone(), kind);
            }
        }
        let source_refinement = backtick_after(scope, "itself a ");
        if cell_wide.is_some() && targets.len() > 1 {
            errs.push(format!(
                "cannot attribute the refinement of Reference Typing cell `{}` — a `kind ==` \
                 claim with multiple resolve-to objects ({}), which row does it narrow?",
                cells[2],
                targets.join(", ")
            ));
            continue;
        }
        if source_refinement.is_some() && !same_kind && !targets.contains(&sources[0]) {
            errs.push(format!(
                "cannot attribute the source refinement of Reference Typing cell `{}` — \
                 no same-kind resolve-to object to carry it",
                cells[2]
            ));
            continue;
        }
        for column in &columns {
            for source in &sources {
                let row_targets: Vec<String> = if same_kind {
                    vec![source.clone()]
                } else {
                    targets.clone()
                };
                for target in row_targets {
                    rows.push(TypingRow {
                        column: column.clone(),
                        source: source.clone(),
                        target: target.clone(),
                        target_refinement: per_target
                            .get(&target)
                            .cloned()
                            .or_else(|| cell_wide.clone()),
                        source_refinement: if target == *source {
                            source_refinement.clone()
                        } else {
                            None
                        },
                    });
                }
            }
        }
    }
    if errs.is_empty() { Ok(rows) } else { Err(errs) }
}

/// The drift gate: the doc's Reference Typing rows equal the [`Schema`]'s
/// morphisms row for row — triples and stated refinements, compared as
/// sets in both directions. `Err` names every divergence (and each side's
/// direction), each one a `schema_matches_typing_table` lint finding.
pub fn matches_typing_table(schema: &Schema, rows: &[TypingRow]) -> Result<(), Vec<String>> {
    let doc_keys: BTreeSet<String> = rows.iter().map(row_key).collect();
    let schema_keys: BTreeSet<String> = schema.morphisms.iter().map(morphism_key).collect();
    let mut drift: Vec<String> = Vec::new();
    for k in doc_keys.difference(&schema_keys) {
        drift.push(format!(
            "document row `{k}` has no Schema morphism — the code's Reference Typing is \
             behind the doc (schema_matches_typing_table)"
        ));
    }
    for k in schema_keys.difference(&doc_keys) {
        drift.push(format!(
            "Schema row `{k}` is missing from the doc's Reference Typing table — the doc is \
             behind the code (schema_matches_typing_table)"
        ));
    }
    if drift.is_empty() { Ok(()) } else { Err(drift) }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECTION: &str = "\
| Field         | Appears on  | Must resolve to |
|---------------|-------------|-----------------|
| `traces_to`   | Constraint  | Intent |
| `guard`       | Transition  | an invariant Constraint — or a State, the \"has reached state X\" pattern (Revision 12) — an `advisory` Constraint can never gate a transition, by typing, not by convention |
| `from` / `to` | Transition  | State |
| `supersedes`  | Constraint, Property | same kind as the row it appears on (Constraint→Constraint, Property→Property) |
| `emits`       | State       | Constraint, kind == `effect` only — a state's declared Moore-machine output |
| `uses`        | Constraint, any file | Intent of a `kind: profile` file (the pack's frontmatter id) |
";

    #[test]
    fn the_section_grammar_reads_today_s_table() {
        let rows = parse_typing_table(SECTION).expect("section parses");
        let keys: BTreeSet<String> = rows.iter().map(row_key).collect();
        // 6 doc rows → 9 typing rows (from/to split, guard/emits refinements).
        assert_eq!(keys.len(), 9, "{keys:#?}");
        assert!(keys.contains("traces_to: Constraint → Intent"));
        assert!(keys.contains("guard: Transition → Constraint [target must be kind `invariant`]"));
        assert!(keys.contains("guard: Transition → State"));
        assert!(keys.contains("from: Transition → State"));
        assert!(keys.contains("to: Transition → State"));
        assert!(keys.contains("supersedes: Constraint → Constraint"));
        assert!(keys.contains("supersedes: Property → Property"));
        assert!(keys.contains("emits: State → Constraint [target must be kind `effect`]"));
        assert!(keys.contains("uses: Constraint → Intent [target must be kind `profile`]"));
    }

    /// The use-mention discipline: `observes`'s prose names `Intent` in a
    /// later sentence — outside the typing scope, never a phantom target.
    #[test]
    fn later_sentence_object_mentions_are_not_claims() {
        let section = "\
| Field | Appears on | Must resolve to |
|-------|------------|-----------------|
| `observes` | Constraint, any file | Constraint, kind == `effect` only — a declared observable: an outbound pointer. The row carrying `observes` still has its own ordinary `traces_to` pointing at its own file's Intent |
";
        let rows = parse_typing_table(section).expect("section parses");
        assert_eq!(
            row_key(&rows[0]),
            "observes: Constraint → Constraint [target must be kind `effect`]"
        );
    }

    /// A cell whose scope names no object is uninterpretable — an edit the
    /// reader cannot read is a drift signal, never silence.
    #[test]
    fn an_uninterpretable_cell_is_an_error() {
        let section = "\
| Field | Appears on | Must resolve to |
|-------|------------|-----------------|
| `traces_to` | Constraint | something untyped |
";
        let errs = parse_typing_table(section).expect_err("uninterpretable cell");
        assert!(errs[0].contains("Must-resolve-to"));
    }

    /// A changed header is drift: the table's shape itself is part of the
    /// contract the Schema mirrors.
    #[test]
    fn a_changed_header_is_an_error() {
        let section = "\
| Field | Appears on | Resolves to |
|-------|------------|-------------|
| `traces_to` | Constraint | Intent |
";
        let errs = parse_typing_table(section).expect_err("header changed");
        assert!(errs[0].contains("header changed"));
    }

    /// The unit scenario (`schema_missing_one_row_of_the_typing_table`):
    /// a Schema missing one doc row fails, naming the row.
    #[test]
    fn schema_missing_one_row_of_the_typing_table() {
        let rows = parse_typing_table(SECTION).expect("section parses");
        let mut schema = crate::acset::schema::canonical();
        schema.morphisms.retain(|m| m.column != "traces_to");
        let drift = matches_typing_table(&schema, &rows)
            .expect_err("a schema missing a doc row must fail the comparison");
        assert!(drift.iter().any(|d| d.contains("traces_to")), "{drift:?}");
    }

    /// The mirror direction: the doc behind the code (a row removed from
    /// the table) also fails — the gate is bidirectional.
    #[test]
    fn doc_missing_one_row_of_the_schema() {
        let rows = parse_typing_table(SECTION)
            .expect("section parses")
            .into_iter()
            .filter(|r| r.column != "traces_to")
            .collect::<Vec<_>>();
        let drift = matches_typing_table(&crate::acset::schema::canonical(), &rows)
            .expect_err("a doc missing a schema row must fail the comparison");
        assert!(drift.iter().any(|d| d.contains("traces_to")), "{drift:?}");
    }
}
