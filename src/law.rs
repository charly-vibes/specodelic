//! The law-case grammar (specodelic.md Revision 21).
//!
//! Purpose: read a law-kind Property row's `predicate` cell ONCE, so the
//! linter's `law_cases` rule and the compiler's block expansion can never
//! disagree on what a case is. Revision 13 gave the row its machine-
//! findable case labels (`**name:**`); Revision 21 gives each named case
//! its own executable body (a case key, `**name::**`, followed by the
//! fragment marker in fragment position).
//!
//! Responsibilities: parse the case labels (the enumeration of record),
//! the case keys, and the per-case bodies; report the labeled failures
//! the grammar forbids (a fragment no case key introduces, a second body
//! for one case, a body naming a case the cell does not enumerate).
//!
//! Rationale: the case grammar is shared state between two checkers — one
//! seam, one parse. The closed fragment-tag set lives here because this is
//! the one place where a marker and a case label share a cell shape: a
//! known tag (`rust`, `py`, `ts`) is a fragment marker and NEVER a case
//! label, whatever its position.

/// The closed fragment-tag set (compile.md's `fragment_language_closed`):
/// exactly the languages with an executable emitter at this Revision, and
/// the tags the case grammar reserves — a `**rust:**` occurrence is a
/// fragment marker, never a case label.
pub const FRAGMENT_TAGS: &[&str] = &["rust", "py", "ts"];

/// One case body: the case it belongs to, the fragment tag it is tagged
/// for, and the body's text (the cell text after the marker, up to the
/// code span's closing backtick — the compiler shapes it like every
/// other fragment).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LawBody {
    /// The case the body belongs to — the label as the cell spells it.
    pub case: String,
    /// The fragment tag (`rust`, `py`, `ts`).
    pub tag: String,
    /// The body text, verbatim.
    pub text: String,
}

/// A labeled failure the law-case grammar forbids. Every failure names
/// the rule and, where one applies, the offending case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LawFailure {
    /// The offending case, when the failure names one.
    pub case: Option<String>,
    /// The failure message.
    pub message: String,
}

impl LawFailure {
    /// A failure that names a case.
    fn for_case(case: &str, message: impl Into<String>) -> Self {
        LawFailure {
            case: Some(case.to_string()),
            message: message.into(),
        }
    }
}

/// The parsed law predicate cell: the cases it enumerates, the bodies it
/// carries, and the failures it earns.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LawCell {
    /// The enumerated case labels, in cell order — the enumeration of
    /// record (Revision 13). A fragment marker is never one of them.
    pub cases: Vec<String>,
    /// The per-case bodies — one per case that carries one.
    pub bodies: Vec<LawBody>,
    /// The case keys the cell carries.
    pub keys: Vec<String>,
    /// The labeled failures the cell earns.
    pub failures: Vec<LawFailure>,
}

/// The case labels a law-kind row's predicate enumerates (Revision 13's
/// machine-findable `**name:**` form), read through the one shared
/// parser — the seam the linter's `law_cases` rule and the compiler's
/// block expansion both read.
pub fn law_case_labels(predicate: &str) -> Vec<String> {
    parse_law_cell(predicate).cases
}

/// The per-case bodies a law-kind row's predicate carries, in cell order.
pub fn law_case_bodies(predicate: &str) -> Vec<LawBody> {
    parse_law_cell(predicate).bodies
}

/// Parse a law-kind row's predicate cell: the enumerated cases, the
/// per-case bodies, and the labeled failures the grammar forbids.
pub fn parse_law_cell(predicate: &str) -> LawCell {
    let hits = scan(predicate);
    let mut cell = LawCell {
        cases: hits
            .iter()
            .filter_map(|h| match &h.kind {
                HitKind::Label(name) => Some(name.clone()),
                _ => None,
            })
            .collect(),
        bodies: vec![],
        keys: hits
            .iter()
            .filter_map(|h| match &h.kind {
                HitKind::Key(name) => Some(name.clone()),
                _ => None,
            })
            .collect(),
        failures: vec![],
    };
    collect_bodies(predicate, &hits, &mut cell);
    cell
}

/// The labeled failure a NON-law row earns when its cell carries a case
/// key: the per-case body grammar is law-only, so a unit row's case key
/// is a labeled failure, never a silently ignored marker.
pub fn non_law_case_key_failure(predicate: &str) -> Option<LawFailure> {
    let key = scan(predicate).into_iter().find_map(|h| match h.kind {
        HitKind::Key(name) => Some(name),
        _ => None,
    })?;
    Some(LawFailure::for_case(
        &key,
        format!(
            "the per-case body grammar is law-only — a case key (`**{key}::**`) on a non-law Property row is rejected (fragment_law_rejected, specodelic.md Revision 21)"
        ),
    ))
}

// ---------------------------------------------------------------------------
// The scanner
// ---------------------------------------------------------------------------

/// What a scan found at one position of the cell.
#[derive(Debug, Clone, PartialEq, Eq)]
enum HitKind {
    /// A case label: `**name:**` — the enumeration of record.
    Label(String),
    /// A case key: `**name::**` — introduces the case's body.
    Key(String),
    /// A fragment marker: `**tag:**` with a known tag, in fragment
    /// position.
    Marker(String),
}

/// One scan hit: its kind and the byte span it occupies.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Hit {
    kind: HitKind,
    start: usize,
    end: usize,
}

/// Scan the cell for the three occurrence shapes the law-case grammar
/// knows: case labels, case keys, and fragment markers.
///
/// The fragment-position rule is the one compile.md states for the
/// executable-fragment markers: an occurrence at the very start of the
/// cell, or one immediately opening the code span it lives in (a
/// backtick directly before it). An occurrence anywhere else is prose —
/// a mention of the mechanism, never an opt-in.
fn scan(cell: &str) -> Vec<Hit> {
    let lead = cell.len() - cell.trim_start().len();
    let mut hits: Vec<Hit> = vec![];
    let mut in_span = false;
    let mut prev: Option<char> = None;
    let mut i = 0usize;
    while i < cell.len() {
        let Some(c) = cell[i..].chars().next() else {
            break;
        };
        if c == '`' {
            in_span = !in_span;
            prev = Some(c);
            i += c.len_utf8();
            continue;
        }
        if cell[i..].starts_with("**")
            && let Some((kind, end)) = occurrence_at(cell, i)
        {
            let fragment_pos = i == lead || (in_span && prev == Some('`'));
            let keep = match &kind {
                // A known tag outside fragment position is a mention
                // — prose, never a body.
                HitKind::Marker(_) => fragment_pos,
                _ => true,
            };
            if keep {
                hits.push(Hit {
                    kind,
                    start: i,
                    end,
                });
            }
            prev = Some('*');
            i = end;
            continue;
        }
        prev = Some(c);
        i += c.len_utf8();
    }
    hits
}

/// The occurrence at `i`, if any: a case key (`**name::**`), a fragment
/// marker (`**tag:**` with a known tag), or a case label (`**name:**`).
///
/// The label shape is Revision 13's — a letter, then letters, spaces,
/// underscores or hyphens. The tag shape is the closed set's — a
/// lowercase letter, then lowercase letters, digits, underscores or
/// hyphens. A known tag wins over the label shape (a `**rust:**`
/// occurrence is a marker, never a case label); an unknown tag-shaped
/// occurrence falls back to the label shape, so `**identity:**` stays a
/// case label while `**go1:**` is neither.
fn occurrence_at(cell: &str, i: usize) -> Option<(HitKind, usize)> {
    let after = cell.get(i + 2..)?;
    let name_len = name_run_len(after)?;
    let name = &after[..name_len];
    let tail = &after[name_len..];
    if tail.starts_with("::**") {
        return Some((HitKind::Key(name.trim().to_string()), i + 2 + name_len + 4));
    }
    if tail.starts_with(":**") {
        let end = i + 2 + name_len + 3;
        if FRAGMENT_TAGS.contains(&name) {
            return Some((HitKind::Marker(name.to_string()), end));
        }
        return Some((HitKind::Label(name.trim().to_string()), end));
    }
    None
}

/// The length of the label-shaped name run at the start of `s`: a letter,
/// then letters, spaces, underscores or hyphens. `None` when `s` does
/// not start with a letter.
fn name_run_len(s: &str) -> Option<usize> {
    let mut chars = s.char_indices();
    let (_, first) = chars.next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    let mut len = first.len_utf8();
    for (i, c) in chars {
        if c.is_ascii_alphabetic() || c == ' ' || c == '_' || c == '-' {
            len = i + c.len_utf8();
        } else {
            break;
        }
    }
    Some(len)
}

// ---------------------------------------------------------------------------
// The bodies
// ---------------------------------------------------------------------------

/// Collect the per-case bodies: each fragment marker must be introduced
/// by a case key, the key must name a case the cell enumerates, the body
/// must be non-empty, and a case may carry at most one body. A case key
/// that introduces no body at all is a labeled failure too — a silently
/// ignored key is the marker-honesty violation this grammar exists to
/// prevent.
fn collect_bodies(cell: &str, hits: &[Hit], out: &mut LawCell) {
    let mut seen: Vec<String> = vec![];
    let mut claimed: Vec<String> = vec![];
    for (i, hit) in hits.iter().enumerate() {
        let HitKind::Marker(tag) = &hit.kind else {
            continue;
        };
        let Some(key) = nearest_key(hits, i) else {
            out.failures.push(LawFailure {
                case: None,
                message: "the executable fragment is not associated with a case label — a law-kind property row carries one body per case, written under its case key (`**identity::** `**rust:** …`)"
                    .into(),
            });
            continue;
        };
        let key_name = match &key.kind {
            HitKind::Key(name) => name.clone(),
            _ => continue,
        };
        claimed.push(key_name.clone());
        if !introduces(cell, key, hit) {
            out.failures.push(LawFailure {
                case: Some(key_name.clone()),
                message: format!(
                    "the executable fragment is not associated with the case `{key_name}` — a law-kind row writes a body immediately under its case key (`**{key_name}::** `**{tag}:** …`)"
                ),
            });
            continue;
        }
        let Some(label) = out
            .cases
            .iter()
            .find(|c| c.eq_ignore_ascii_case(&key_name))
            .cloned()
        else {
            out.failures.push(LawFailure::for_case(
                &key_name,
                format!(
                    "the body of a law-kind row names the case `{key_name}`, which the predicate does not enumerate — a body key must name one of the row's `**name:**` case labels"
                ),
            ));
            continue;
        };
        if seen.contains(&label) {
            out.failures.push(LawFailure::for_case(
                &label,
                format!(
                    "the law-kind case `{label}` carries two bodies — one body per case (fragment_law_rejected)"
                ),
            ));
            continue;
        }
        let text = body_text(cell, hit);
        if text.trim().is_empty() {
            out.failures.push(LawFailure::for_case(
                &label,
                format!(
                    "the body of the law-kind case `{label}` is empty — write the code expression after the marker"
                ),
            ));
            continue;
        }
        seen.push(label.clone());
        out.bodies.push(LawBody {
            case: label,
            tag: tag.clone(),
            text,
        });
    }
    for key in &out.keys {
        if !claimed.contains(key) {
            out.failures.push(LawFailure::for_case(
                key,
                format!(
                    "the case key `{key}` of a law-kind row introduces no body — write the fragment marker in fragment position under the key (`**{key}::** `**rust:** …`)"
                ),
            ));
        }
    }
}

/// The case key nearest before hit `i` — the key that introduces the
/// marker, if any.
fn nearest_key(hits: &[Hit], i: usize) -> Option<&Hit> {
    hits[..i]
        .iter()
        .rev()
        .find(|h| matches!(h.kind, HitKind::Key(_)))
}

/// Does the marker at `marker` follow the case key at `key` directly —
/// whitespace, then the backtick that opens the body's code span?
fn introduces(cell: &str, key: &Hit, marker: &Hit) -> bool {
    let gap = &cell[key.end..marker.start];
    match gap.strip_suffix('`') {
        Some(rest) => rest.trim().is_empty(),
        None => false,
    }
}

/// The body's text: the cell text after the marker, up to the code span's
/// closing backtick (or the cell's end).
fn body_text(cell: &str, marker: &Hit) -> String {
    let after = &cell[marker.end..];
    match after.find('`') {
        Some(j) => after[..j].to_string(),
        None => after.to_string(),
    }
}
