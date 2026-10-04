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
    }

    // Transitions: id, from, to cells.
    for t in &spec.transitions {
        check(t.id_span, &t.id, format!("transition id `{}`", t.id))?;
        check(
            t.from_span,
            &t.from,
            format!("transition `{}` from cell", t.id),
        )?;
        check(t.to_span, &t.to, format!("transition `{}` to cell", t.id))?;
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
