//! The acset writer — the one module allowed to turn an instance edit back
//! into text (`openspec` change `add-acset-writer`, beads specodelic-p5b).
//!
//! Purpose: realize instance edits as markdown through the byte spans the
//! parser records (`source_spans_recorded`), returning results the caller
//! applies as its single transaction — never re-serializing a table, never
//! doing I/O. Responsibilities: identity emission first — `emit(parse(x),
//! no edit) == x` byte for byte for every file the parser accepts
//! (`emit_identity`), verified by `tests/acset_writer_identity.rs` over the
//! whole corpus; edit application (`apply`) follows in phase 4.
//! Rationale: without this seam every edit flows through hand-wired
//! rewriting in `rename.rs`, unspecified and unparityed; the writer makes
//! emission a specified, identity-gated seam so the typed core never costs
//! the format its byte-stable round trip.

use crate::spec::Spec;
use std::path::Path;

/// A writer failure — one labeled cause per the fleet error contract
/// (`specs/errors.md`): the label names the owning file's id (`spec`, the
/// capability intent), the detail says what failed, and the remediation
/// hint is non-empty. Internal cause — the consuming command wraps it in
/// its own envelope error; no new exit codes (design decision D1).
#[derive(Debug, Clone)]
pub struct WriterError {
    /// The label, e.g. `spec.span_failure` — asserted verbatim by the
    /// delta's `*_label_asserted` properties.
    pub label: String,
    /// What failed, with the offending detail.
    pub detail: String,
    /// Non-empty remediation hint.
    pub remediation: String,
}

impl std::fmt::Display for WriterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} — remediation: {}",
            self.label, self.detail, self.remediation
        )
    }
}

impl std::error::Error for WriterError {}

/// Identity emission: emit text for a parsed spec with no edit applied.
///
/// Never re-serializes — with no edit the emitted bytes ARE the source
/// bytes. What the call buys is the guarantee the later edit path rests
/// on: every span the parser recorded resolves in-bounds to exactly the
/// bytes its parsed element carries. A parser that drifts from its span
/// recording fails here (`spec.span_failure`) before any edit could
/// corrupt a file; unrecorded spans (`None`) are legitimate — the writer
/// simply has nothing to verify and no edit will ever target them.
pub fn emit(source: &str, spec: &Spec) -> Result<String, WriterError> {
    verify_spans(source, spec)?;
    Ok(source.to_string())
}

/// Verify every recorded span resolves in-bounds to its parsed element's
/// bytes. One labeled failure for the first disagreement — never a silent
/// partial check.
fn verify_spans(src: &str, spec: &Spec) -> Result<(), WriterError> {
    let check = |span: Option<crate::spec::Span>,
                 expected: &str,
                 what: String|
     -> Result<(), WriterError> {
        let Some(s) = span else {
            return Ok(());
        };
        let Some(covered) = src.get(s.start..s.end) else {
            return Err(span_failure(
                what,
                format!(
                    "span [{}, {}) is out of bounds for a {}-byte source",
                    s.start,
                    s.end,
                    src.len()
                ),
            ));
        };
        if covered != expected {
            return Err(span_failure(
                what,
                format!("recorded span covers {covered:?}, but the parsed element is {expected:?}"),
            ));
        }
        Ok(())
    };

    // Frontmatter intent id — `frontmatter_id_span` only records the span
    // when the raw value bytes ARE the parsed id, so this is a re-assert.
    check(
        spec.intent.id_span,
        &spec.intent.id,
        format!("intent id `{}`", spec.intent.id),
    )?;

    // Every structured row's id cell (Constraints, States, Properties).
    for r in spec
        .constraints
        .iter()
        .chain(&spec.states)
        .chain(&spec.properties)
    {
        check(r.id_span, &r.id, format!("row id `{}`", r.id))?;
        // Every column's cell span (task 4.2) — the any-cell rewrite
        // surface must resolve to the bytes its parsed value carries.
        for (col, s) in &r.cell_spans {
            let expected = r.cells.get(col).map(String::as_str).unwrap_or("");
            check(Some(*s), expected, format!("row `{}` cell `{col}`", r.id))?;
        }
    }

    // Transitions: id, from, to cells, plus every column's cell span.
    for t in &spec.transitions {
        check(t.id_span, &t.id, format!("transition id `{}`", t.id))?;
        check(
            t.from_span,
            &t.from,
            format!("transition `{}` from cell", t.id),
        )?;
        check(t.to_span, &t.to, format!("transition `{}` to cell", t.id))?;
        for (col, s) in &t.cell_spans {
            let expected = match col.as_str() {
                "id" => t.id.as_str(),
                "from" => t.from.as_str(),
                "to" => t.to.as_str(),
                _ => t.guard.as_deref().unwrap_or(""),
            };
            check(
                Some(*s),
                expected,
                format!("transition `{}` cell `{col}`", t.id),
            )?;
        }
    }

    // Wiki-links: the span covers the full `[[…]]` occurrence; the inner
    // text trimmed must equal the parsed target (the parser trims the
    // inner text, the raw bytes may carry padding an edit must preserve).
    for l in &spec.links {
        let Some(s) = l.span else { continue };
        let Some(covered) = src.get(s.start..s.end) else {
            return Err(span_failure(
                format!("link to `{}`", l.target),
                format!(
                    "span [{}, {}) is out of bounds for a {}-byte source",
                    s.start,
                    s.end,
                    src.len()
                ),
            ));
        };
        let inner = covered
            .strip_prefix("[[")
            .and_then(|c| c.strip_suffix("]]"));
        match inner {
            Some(inner) if inner.trim() == l.target => {}
            other => {
                return Err(span_failure(
                    format!("link to `{}`", l.target),
                    format!(
                        "recorded span covers {covered:?}, expected a [[…]] occurrence whose inner text is {:?} (got {other:?})",
                        l.target
                    ),
                ));
            }
        }
    }

    Ok(())
}

/// The `spec.span_failure` labeled cause: the writer could not extract a
/// recorded span because `detail`.
fn span_failure(what: String, detail: String) -> WriterError {
    WriterError {
        label: "spec.span_failure".into(),
        detail: format!("{what}: {detail}"),
        remediation: "reparse the file to refresh its recorded spans; if the \
                      span is persistently unresolvable the parser's span \
                      recording has drifted from its extraction — see the \
                      acset-writer identity gate"
            .into(),
    }
}

// ---------------------------------------------------------------------------
// Edit application (add-acset-writer task 4.2) — `apply(f, x)` realizes an
// instance edit through the recorded spans and returns the write-set.
// ---------------------------------------------------------------------------

/// The write-set: writes of `(path, full new contents)` plus removals —
/// data, never I/O; applying it is the caller's single transaction
/// (`write_set_atomic`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteSet {
    /// Files to write: the edited file's path and its FULL new contents.
    pub writes: Vec<(std::path::PathBuf, String)>,
    /// Paths to remove (an intent rename moves the file: one removal).
    pub removals: Vec<std::path::PathBuf>,
}

/// One edit: rename id `old` to `new` within a file. The shape mirrors
/// `rename.rs`'s per-file rewrite contract so the migration (phase 5) can
/// flip one rewrite family at a time behind the 1.1 parity snapshots:
///
/// - wiki-link targets matching `old` (or `old.<child>`) follow the
///   rename, in every file;
/// - when this file's own Intent id IS `old`, the frontmatter id is
///   rewritten and the file moves to the filename the new id implies
///   (`filename_follows_intent_id`);
/// - `local` rewrites cells and state bullets carrying the local id —
///   present only for the definition file of a row rename.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    Rename {
        /// The qualified id being renamed (links match this and its
        /// `.<child>` suffixes).
        old: String,
        /// The qualified replacement.
        new: String,
        /// `(local_old, local_new)` — the definition-file cell/bullet
        /// rewrite of a row rename.
        local: Option<(String, String)>,
    },
}

/// Apply `edit` to the file at `path` whose parsed form is `spec` and raw
/// bytes `source`. Performs no I/O; span coherence is verified first (the
/// identity gate), then every change is realized as a span replacement.
pub fn apply(edit: &Edit, path: &Path, source: &str, spec: &Spec) -> Result<WriteSet, WriterError> {
    let Edit::Rename { old, new, local } = edit;
    // rename_naturality at the writer: apply(id, x) == x.
    if old == new && local.as_ref().is_none_or(|(l, n)| l == n) {
        return Ok(WriteSet {
            writes: vec![],
            removals: vec![],
        });
    }
    // The replacement must be a usable id: it will live inside `[[…]]`
    // and `| … |` cells (rename.rs's shape law, mirrored here so an
    // unusable id is refused before any span is touched).
    validate_shape(new, "new")?;
    if let Some((_, ln)) = local {
        validate_shape(ln, "new local")?;
    }
    verify_spans(source, spec)?;

    // Collect replacements: (span, replacement bytes), non-overlapping.
    let mut repls: Vec<(crate::spec::Span, String)> = Vec::new();

    // Wiki-links: qualified targets follow the rename everywhere. The
    // emitted link is rebuilt from the trimmed target (the parser's
    // value). rename's exact-match law realized mechanically: only a
    // link whose raw bytes are exactly `[[target]]` follows — raw inner
    // padding (`[[ target ]]`) never matches rename's exact comparison,
    // so the writer leaves it untouched too (the link still RESOLVES
    // through its trimmed target corpus-wide, but a rename that would
    // strand it refuses at rename's verify gate, exactly as before the
    // writer existed).
    wiki_link_repls(spec, source, old, new, &mut repls);

    // The file's own Intent id: rewritten through its span, and the file
    // moves. A quoted id has no recordable span — the edit cannot be
    // realized (`apply_failure`), never a silent partial rewrite.
    let is_intent_rename = local.is_none() && spec.intent.id == *old;
    if is_intent_rename {
        match spec.intent.id_span {
            Some(s) => repls.push((s, new.clone())),
            None => {
                return Err(apply_failure(format!(
                    "intent id `{old}` is recorded without a rewritable span \
                     (quoted or multi-line frontmatter value)"
                )));
            }
        }
    }

    // Definition-file cells and bullets carrying the local id — rename's
    // any-cell rule, realized through per-column spans.
    if let Some((lo, ln)) = local {
        local_id_repls(spec, lo, ln, &mut repls)?;
    }

    // Overlap guard: spans from different families must be disjoint —
    // structural corruption here means the parser double-recorded.
    repls.sort_by_key(|(s, _)| s.start);
    for pair in repls.windows(2) {
        if pair[0].0.end > pair[1].0.start {
            return Err(span_failure(
                "edit realization".into(),
                format!(
                    "replacement spans overlap: [{}, {}) and [{}, {})",
                    pair[0].0.start, pair[0].0.end, pair[1].0.start, pair[1].0.end
                ),
            ));
        }
    }

    // Realize: splice the replacements into the source bytes — every
    // byte outside a replaced span, padding and terminators included,
    // passes through untouched (`untouched_bytes_preserved`).
    let out = splice_replacements(source, &repls);

    // Roundtrip gate: the emitted text must reparse to an instance that
    // realizes exactly the edit (`edit_application_faithful`); anything
    // else is a labeled `spec.roundtrip_failure`, never a silent emit.
    verify_roundtrip(edit, source, spec, &out)?;

    // The write-set: only files that actually change are written; an
    // intent rename always writes the new path and removes the old one.
    let writes = if out == *source && !is_intent_rename {
        vec![]
    } else {
        let target = if is_intent_rename {
            let new_name = format!("{}.md", new.replace('.', "-"));
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .join(new_name)
        } else {
            path.to_path_buf()
        };
        vec![(target, out)]
    };
    let removals = if is_intent_rename {
        vec![path.to_path_buf()]
    } else {
        vec![]
    };
    Ok(WriteSet { writes, removals })
}

/// Wiki-link replacements: qualified targets follow the rename
/// everywhere. The emitted link is rebuilt from the trimmed target (the
/// parser's value). rename's exact-match law realized mechanically: only
/// a link whose raw bytes are exactly `[[target]]` follows — raw inner
/// padding (`[[ target ]]`) never matches rename's exact comparison, so
/// the writer leaves it untouched too (the link still RESOLVES through
/// its trimmed target corpus-wide, but a rename that would strand it
/// refuses at rename's verify gate, exactly as before the writer
/// existed).
fn wiki_link_repls(
    spec: &Spec,
    source: &str,
    old: &str,
    new: &str,
    repls: &mut Vec<(crate::spec::Span, String)>,
) {
    for l in &spec.links {
        let Some(s) = l.span else { continue };
        if source[s.start..s.end] != format!("[[{}]]", l.target) {
            continue;
        }
        let follows = if l.target == old {
            Some(new.to_string())
        } else if l
            .target
            .strip_prefix(old)
            .is_some_and(|suffix| suffix.starts_with('.'))
        {
            Some(format!("{new}{}", &l.target[old.len()..]))
        } else {
            None
        };
        if let Some(nt) = follows {
            repls.push((s, format!("[[{nt}]]")));
        }
    }
}

/// Local-id replacements across the file: definition-table cells and
/// bullets carrying the local id, plus state bullets and
/// id/from/to transition cells — rename's any-cell rule through
/// per-column spans.
fn local_id_repls(
    spec: &Spec,
    lo: &str,
    ln: &str,
    repls: &mut Vec<(crate::spec::Span, String)>,
) -> Result<(), WriterError> {
    for r in &spec.constraints {
        collect_cell_repls(&r.cells, &r.cell_spans, lo, ln, repls)?;
    }
    for r in &spec.properties {
        collect_cell_repls(&r.cells, &r.cell_spans, lo, ln, repls)?;
    }
    // State bullets: the head id is the only editable surface.
    for s in &spec.states {
        if s.id == lo {
            match s.id_span {
                Some(sp) => repls.push((sp, ln.to_string())),
                None => {
                    return Err(apply_failure(format!(
                        "state bullet `{lo}` is recorded without a rewritable \
                         span (unrecognized bullet shape)"
                    )));
                }
            }
        }
    }
    for t in &spec.transitions {
        for (col, value) in [("id", &t.id), ("from", &t.from), ("to", &t.to)] {
            if value == lo
                && let Some(sp) = t.cell_spans.get(col)
            {
                repls.push((*sp, ln.to_string()));
            }
        }
    }
    Ok(())
}

/// Splice the (non-overlapping, start-sorted) replacements into the
/// source bytes — every byte outside a replaced span, padding and
/// terminators included, passes through untouched
/// (`untouched_bytes_preserved`).
fn splice_replacements(source: &str, repls: &[(crate::spec::Span, String)]) -> String {
    let mut out = String::with_capacity(source.len());
    let mut cursor = 0usize;
    for (s, text) in repls {
        out.push_str(&source[cursor..s.start]);
        out.push_str(text);
        cursor = s.end;
    }
    out.push_str(&source[cursor..]);
    out
}

/// Cell replacements for one table row: every column whose parsed value
/// equals the local id is rewritten through its recorded span (rename's
/// any-cell rule). A matching cell without a span cannot realize the edit.
fn collect_cell_repls(
    cells: &std::collections::BTreeMap<String, String>,
    cell_spans: &std::collections::BTreeMap<String, crate::spec::Span>,
    lo: &str,
    ln: &str,
    repls: &mut Vec<(crate::spec::Span, String)>,
) -> Result<(), WriterError> {
    for (col, value) in cells {
        if value == lo {
            match cell_spans.get(col) {
                Some(sp) => repls.push((*sp, ln.to_string())),
                None => {
                    return Err(apply_failure(format!(
                        "cell `{col}` carrying `{lo}` is recorded without a \
                         rewritable span"
                    )));
                }
            }
        }
    }
    Ok(())
}

/// The replacement must be a usable id: non-empty, no whitespace, no link
/// or table syntax characters (it will live inside `[[…]]` and `| … |`) —
/// rename.rs's shape law, mirrored (backticks and parens are allowed, as
/// in rename; the roundtrip gate catches the shapes they corrupt).
fn validate_shape(id: &str, what: &str) -> Result<(), WriterError> {
    if id.is_empty()
        || id.contains(|c: char| c.is_whitespace() || matches!(c, '[' | ']' | '|' | '#'))
    {
        return Err(apply_failure(format!(
            "{what} id {id:?} is not a usable id — empty, whitespace, or \
             link/table syntax characters"
        )));
    }
    Ok(())
}

/// The `spec.apply_failure` labeled cause: an edit names an id no recorded
/// span can realize because `detail`.
fn apply_failure(detail: String) -> WriterError {
    WriterError {
        label: "spec.apply_failure".into(),
        detail,
        remediation: "the edit cannot be realized through recorded spans — \
                      reparse the file, and if the shape persists, fix the \
                      id's spelling in the source first (quoted frontmatter \
                      ids and unrecognized bullet heads are not rewritable)"
            .into(),
    }
}

/// The `spec.roundtrip_failure` labeled cause: the emitted text reparses
/// to an instance other than the edit's target because `detail`.
fn roundtrip_failure(detail: String) -> WriterError {
    WriterError {
        label: "spec.roundtrip_failure".into(),
        detail,
        remediation: "the emitted text would not realize the edit — pick a \
                      replacement id that survives every context it is \
                      rewritten into (bullet heads, table cells, links) and \
                      retry"
            .into(),
    }
}

/// Roundtrip gate: reparse the emitted text and check the instance it
/// denotes realized exactly the edit — the renamed ids are the new ones,
/// no stale reference survives. A parse failure or a stale id is a
/// labeled refusal, never a silently corrupted emit.
fn verify_roundtrip(
    edit: &Edit,
    source: &str,
    before: &Spec,
    emitted: &str,
) -> Result<(), WriterError> {
    let Edit::Rename { old, new, local } = edit;
    let after = crate::spec::parse_str(emitted)
        .map_err(|e| roundtrip_failure(format!("emitted text does not reparse: {e}")))?;

    // The intent id realized the rename (an intent edit's own file).
    if local.is_none() && before.intent.id == *old && after.intent.id != *new {
        return Err(roundtrip_failure(format!(
            "intent id did not realize the rename: still `{}`",
            after.intent.id
        )));
    }

    // State bullets, transition endpoints, and any-cell matches realize
    // the local rename: no stale local id survives AND the new id appears
    // exactly where the old one stood (a replacement id that reparses to
    // a different token — a backticked or suffixed head — is caught by
    // the realized-count, not only the stale check).
    if let Some((lo, ln)) = local {
        let stale = |what: &str, n: usize| -> Result<(), WriterError> {
            if n > 0 {
                Err(roundtrip_failure(format!(
                    "{what} did not realize the rename: {n} stale `{lo}` occurrence(s)"
                )))
            } else {
                Ok(())
            }
        };
        let unrealized = |what: &str, before_n: usize, after_n: usize| -> Result<(), WriterError> {
            if before_n != after_n {
                Err(roundtrip_failure(format!(
                    "{what} did not realize the rename: `{lo}` stood at {before_n} \
                     place(s) but `{ln}` appears at {after_n}"
                )))
            } else {
                Ok(())
            }
        };
        stale(
            "state bullets",
            after.states.iter().filter(|s| s.id == *lo).count(),
        )?;
        unrealized(
            "state bullets",
            before.states.iter().filter(|s| s.id == *lo).count(),
            after.states.iter().filter(|s| s.id == *ln).count(),
        )?;
        let trans_endpoints = |sp: &Spec| -> usize {
            sp.transitions
                .iter()
                .filter(|t| t.id == *lo || t.from == *lo || t.to == *lo)
                .count()
        };
        stale("transitions", trans_endpoints(&after))?;
        let trans_new = |sp: &Spec, id: &str| -> usize {
            sp.transitions
                .iter()
                .filter(|t| t.id == id || t.from == id || t.to == id)
                .count()
        };
        unrealized(
            "transitions",
            trans_endpoints(before),
            trans_new(&after, ln),
        )?;
        let cells = |sp: &Spec| -> usize {
            sp.constraints
                .iter()
                .chain(&sp.properties)
                .filter(|r| r.cells.values().any(|v| v == lo))
                .count()
        };
        let cells_new = |sp: &Spec, id: &str| -> usize {
            sp.constraints
                .iter()
                .chain(&sp.properties)
                .filter(|r| r.cells.values().any(|v| v == id))
                .count()
        };
        stale("table cells", cells(&after))?;
        unrealized("table cells", cells(before), cells_new(&after, ln))?;
    }

    // Every link that followed the rename carries the new target; links
    // that did not match keep their original target. Compared as a
    // target MULTISET over the before-links — span matching misfires
    // when `new_id` itself has the `old_id.<child>` shape (`alpha` →
    // `alpha.prime`: the rewritten `[[alpha.prime]]` matches the
    // child-of-old pattern and looks unrewritten). The exact-raw-match
    // law applies: a padded link (`[[ target ]]`) never follows.
    let mut expected: Vec<String> = before
        .links
        .iter()
        .map(|l| match l.span {
            Some(s) if source[s.start..s.end] == format!("[[{}]]", l.target) => {
                let followed = l.target == *old
                    || l.target
                        .strip_prefix(old.as_str())
                        .is_some_and(|suffix| suffix.starts_with('.'));
                if followed && l.target == *old {
                    new.clone()
                } else if followed {
                    format!("{new}{}", &l.target[old.len()..])
                } else {
                    l.target.clone()
                }
            }
            _ => l.target.clone(),
        })
        .collect();
    expected.sort();
    let mut actual: Vec<String> = after.links.iter().map(|l| l.target.clone()).collect();
    actual.sort();
    if expected != actual {
        return Err(roundtrip_failure(
            "the emitted text's link targets do not match the rename's expected set".into(),
        ));
    }

    Ok(())
}
