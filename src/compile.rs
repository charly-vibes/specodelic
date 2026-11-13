//! The Compile functor — translate a linted-and-covered spec file's
//! structured layers into machine artifacts.
//!
//! Purpose: implement `openspec/specs/compile/spec.md` as one functor with three target
//! categories — Constraints → TOML (`constraint_table_to_toml`), Model →
//! backend-neutral IR (`ModelIR`; TLA+/Alloy emission is deferred to beads
//! `specodelic-mp1` row 6), Properties → proptest! sources
//! (`properties_to_proptest`). Responsibilities: gate on lint cleanliness
//! (`precondition_satisfied`), enforce `compile_is_total` (labeled single
//! failure, never a silent partial), `compile_preserves_ids`, and
//! `no_semantic_drift` (TOML round-trip byte-identical); plus the
//! artifact write path (`artifact_stem`/`write_artifacts`, shared with
//! the orchestrate driver — specodelic-8kk). Rationale: prose
//! is never parsed (`prose_untouched`) and ids are never minted, dropped,
//! or altered — the emitted artifacts are the machine half of the
//! self-hosting contract.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::lint::{self, Report};
use crate::spec::Spec;

// ---------------------------------------------------------------------------
// TOML translation (constraint_table_to_toml)
// ---------------------------------------------------------------------------

/// One Constraint row compiled field-for-field into a TOML entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConstraintToml {
    pub id: String,
    pub kind: String,
    pub expr: String,
    pub traces_to: String,
    /// The opaque `kernel.binding` claim-carrier text (add-min-expr-kernel,
    /// design D4): invariant-kind rows only, surfaced verbatim and never
    /// interpreted by the toolchain. `None` when the column is absent so
    /// pre-Revision artifacts stay byte-identical (pure widening); the
    /// claim path for external checkers is contract-TOML `flags`, not a
    /// toolchain registry (kernel_binding_extracted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding: Option<String>,
}

/// The TOML document shape: which spec it came from + one entry per row.
/// The fixed field order (serde struct order) is what makes re-emission
/// byte-identical under `no_semantic_drift`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConstraintsDoc {
    /// The source file's Intent id — anchors `compile_preserves_ids`.
    pub source: String,
    pub constraints: Vec<ConstraintToml>,
}

/// Compile the Constraints table to a TOML document: one entry per row,
/// `{id, kind, expr, traces_to}`, no lossy transformation.
pub fn constraints_to_toml(spec: &Spec) -> String {
    let doc = constraints_doc(spec);
    toml::to_string(&doc).expect("ConstraintsDoc serializes to TOML")
}

/// The opaque `kernel.binding` claim-carrier extraction (add-min-expr-kernel,
/// design D4, tasks.md §5.2): invariant-kind rows only; the cell's text is
/// carried verbatim — no parse, no validation, no interpretation of its
/// internals — with the backtick fence stripped exactly as the kernel expr
/// path shapes cells (`kernel_cell_content` precedent). An absent column
/// stays `None` so artifacts for pre-Revision specs are byte-identical
/// (pure widening).
fn kernel_binding_of_row(kind: Option<&str>, cell: Option<&String>) -> Option<String> {
    (kind == Some("invariant"))
        .then(|| cell.map(|c| c.trim().trim_matches('`').trim().to_string()))
        .flatten()
}

fn constraints_doc(spec: &Spec) -> ConstraintsDoc {
    ConstraintsDoc {
        source: spec.intent.id.clone(),
        constraints: spec
            .constraints
            .iter()
            .map(|r| ConstraintToml {
                id: r.id.clone(),
                kind: r.cells.get("kind").cloned().unwrap_or_default(),
                // The expr cell is where a `**rust:**` invariant fragment
                // lives; the toml carries cell content, not markdown
                // transport, so the pipe escape is undone here too
                // (specodelic-q7uo). Identity for cells without the escape.
                expr: r
                    .cells
                    .get("expr")
                    .map(|c| unescape_cell_pipes(c))
                    .unwrap_or_default(),
                traces_to: r.cells.get("traces_to").cloned().unwrap_or_default(),
                binding: kernel_binding_of_row(
                    r.cells.get("kind").map(String::as_str),
                    r.cells.get("kernel.binding"),
                ),
            })
            .collect(),
    }
}

// ---------------------------------------------------------------------------
// Model extraction (backend-neutral ModelIR — emission deferred to mp1 row 6)
// ---------------------------------------------------------------------------

/// A Model state (value in the future module's state range).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateIr {
    pub id: String,
}

/// A guarded transition — one disjunct of the future `Next` action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransitionIr {
    pub id: String,
    pub from: String,
    pub to: String,
    /// `None` when the guard cell is empty (`null` guard).
    pub guard: Option<String>,
}

/// An executable invariant fragment (specodelic.md Revision 15): the
/// constraint's id and the verbatim Rust expression after the `**rust:**`
/// marker in its `expr` cell. Consumed by `model_check`'s executable
/// scratch-crate run and carried in the `.tla` manifest comment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct InvariantIr {
    pub id: String,
    pub fragment: String,
}

/// Backend-neutral intermediate representation of the Model section.
/// Both candidate backends (native stateright interpreter, opt-in TLC)
/// consume this same structure; the follow-up change adds the emitter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelIr {
    pub states: Vec<StateIr>,
    pub transitions: Vec<TransitionIr>,
    /// `emits` → effect-Constraint mapping: state id → the referenced
    /// constraint. Exactly one entry per state with an `emits` field —
    /// no entry for the rest, never a default or null
    /// (`output_function_covers_emitting_states_only`).
    pub emits: BTreeMap<String, String>,
    /// The value the `.tla` emitter writes for each `emits` entry: the
    /// effect-Constraint's `expr` when the target is a local constraint
    /// row, the raw target otherwise (specodelic-8nt) — the model-check
    /// consistency gate compares the artifact's Output function against
    /// this verbatim, so value drift cannot pass silently.
    #[serde(default)]
    pub emits_values: BTreeMap<String, String>,
    /// Executable invariant fragments from the Constraints table — empty
    /// unless an invariant row opts in with the `**rust:**` marker
    /// (specodelic.md Revision 15). `compile_spec` validates fragments
    /// labeled before any artifact exists, so the lenient parse here
    /// never fires on a compiled artifact's input.
    #[serde(default)]
    pub invariants: Vec<InvariantIr>,
    /// Guard citation expressions from invariant-kind Constraints whose
    /// expr cell is a citation expression (`[[a]] ∧ [[b]]`, `¬[[a]]`,
    /// bare `[[a]]`) — carried verbatim, keyed by Constraint id
    /// (slice 1, specodelic-txo). Prose expr cells stay out: citations
    /// upgrade from inert annotations only when the cell IS a citation
    /// expression.
    #[serde(default)]
    pub guard_citations: BTreeMap<String, String>,
    /// Kernel expressions from invariant-kind Constraints whose expr
    /// cell opts into kernel translation (add-min-expr-kernel, Tier B) —
    /// carried verbatim, keyed by Constraint id. Prose expr cells stay
    /// out (pure widening): only cells the kernel grammar accepts land
    /// here; the labeled rejection happens in `validate_fragments`.
    #[serde(default)]
    pub guard_kernel: BTreeMap<String, String>,
}

// ---------------------------------------------------------------------------
// Guard citation algebra (slice 1, specodelic-txo — approach-02 increment)
// ---------------------------------------------------------------------------

/// The three-valued status of a citation evaluation: `verified`,
/// `counterexample`, or `unknown`. `unknown` is honest — it never
/// coerces to pass (dl/1 Kleene absorb).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreeValued {
    Verified,
    Counterexample,
    Unknown,
}

/// A parsed guard citation expression over the closed slice-1 grammar:
/// citations `[[id]]`, conjunction `∧`, negation `¬`. Prose is never a
/// citation expression (pure widening, prose untouched).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitationExpr {
    Cite(String),
    Not(Box<CitationExpr>),
    And(Box<CitationExpr>, Box<CitationExpr>),
}

/// Strip the surrounding whitespace/backticks from a Constraints
/// `expr` cell — the shared normalization used both by the parser and
/// by verbatim extraction (single source of truth for cell shaping).
fn strip_guard_cell(cell: &str) -> &str {
    cell.trim().trim_matches('`').trim()
}

/// Parse a Constraints `expr` cell as a citation expression over the
/// closed grammar — `Some` only when the whole cell (backticks and
/// whitespace stripped) is a citation expression; `None` for prose, so
/// prose cells stay inert exactly as before. A malformed cell that is
/// not a citation expression also returns `None` — it stays prose, not
/// a silent upgrade.
pub fn parse_citation_expr(cell: &str) -> Option<CitationExpr> {
    let s = strip_guard_cell(cell);
    if s.is_empty() {
        return None;
    }
    let mut terms: Vec<CitationExpr> = Vec::new();
    let mut rest = s;
    loop {
        rest = rest.trim_start();
        let negated = match rest.strip_prefix('¬') {
            Some(r) => {
                rest = r;
                true
            }
            None => false,
        };
        rest = rest.trim_start();
        let inner = rest.strip_prefix("[[")?;
        let close = inner.find("]]")?;
        let id = inner[..close].trim();
        // A citation id containing a bracket can never match a real
        // Constraints id (ids are filename stems) — the cell is malformed
        // and stays prose (RO5U F-1, specodelic-txo).
        if id.is_empty() || id.contains('[') || id.contains(']') {
            return None;
        }
        rest = &inner[close + 2..];
        let mut term = CitationExpr::Cite(id.to_string());
        if negated {
            term = CitationExpr::Not(Box::new(term));
        }
        terms.push(term);
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        rest = rest.strip_prefix('∧')?;
    }
    let mut terms = terms.into_iter();
    let first = terms.next()?;
    Some(terms.fold(first, |acc, t| {
        CitationExpr::And(Box::new(acc), Box::new(t))
    }))
}

/// Evaluate a citation expression against the known per-id outcomes —
/// Kleene three-valued composition: ∧ is counterexample-dominant and
/// absorbs unknown (verified ∧ unknown = unknown, never pass); ¬ flips
/// verified/counterexample and keeps unknown. A cited id with no known
/// outcome (prose property, nonexistent id) is undischargable →
/// `unknown`, never pass.
pub fn evaluate_citation(
    expr: &CitationExpr,
    outcomes: &BTreeMap<String, ThreeValued>,
) -> ThreeValued {
    match expr {
        CitationExpr::Cite(id) => outcomes.get(id).copied().unwrap_or(ThreeValued::Unknown),
        CitationExpr::Not(inner) => match evaluate_citation(inner, outcomes) {
            ThreeValued::Verified => ThreeValued::Counterexample,
            ThreeValued::Counterexample => ThreeValued::Verified,
            ThreeValued::Unknown => ThreeValued::Unknown,
        },
        CitationExpr::And(a, b) => {
            match (
                evaluate_citation(a, outcomes),
                evaluate_citation(b, outcomes),
            ) {
                (ThreeValued::Counterexample, _) | (_, ThreeValued::Counterexample) => {
                    ThreeValued::Counterexample
                }
                (ThreeValued::Verified, ThreeValued::Verified) => ThreeValued::Verified,
                _ => ThreeValued::Unknown,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Executable predicate fragments (specodelic.md Revision 15, specodelic-rjb)
// ---------------------------------------------------------------------------

/// The opt-in marker for the fully specified language: in a Property
/// `predicate` or invariant Constraint `expr` cell, the cell text after
/// `**rust:**`'s first fragment-position occurrence is the fragment — a
/// Rust boolean expression, emitted/evaluated verbatim. The other two
/// members of the closed tag set (`**py:**`, `**ts:**`) share the grammar
/// but gate compile until their emitters land (`no_emitter_labeled_failure`).
pub const FRAGMENT_MARKER: &str = "**rust:**";

/// The closed fragment-tag set (fragment_language_closed, Revision 16):
/// exactly the languages with a native-shrinking PBT library in the
/// intended consumer set (design D4). Unknown tag-shaped markers fail
/// labeled, never silence (unknown_tag_rejected).
pub const FRAGMENT_LANGUAGES: &[&str] = &["rust", "py", "ts"];

/// The language a fragment is tagged for — one of the closed set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FragmentLang {
    Rust,
    Py,
    Ts,
}

impl FragmentLang {
    /// The marker tag as written in the cell ("rust", "py", "ts").
    pub fn tag(self) -> &'static str {
        match self {
            FragmentLang::Rust => "rust",
            FragmentLang::Py => "py",
            FragmentLang::Ts => "ts",
        }
    }

    /// Parse a marker tag. The closed set itself lives in
    /// `law::FRAGMENT_TAGS` (Revision 21 — the one place the grammar
    /// and the emitters meet), so the two can never drift apart.
    pub(crate) fn parse(tag: &str) -> Option<Self> {
        if !crate::law::FRAGMENT_TAGS.contains(&tag) {
            return None;
        }
        match tag {
            "rust" => Some(FragmentLang::Rust),
            "py" => Some(FragmentLang::Py),
            "ts" => Some(FragmentLang::Ts),
            _ => None,
        }
    }
}

/// A marker occurrence: a known closed-set tag, or an unknown tag-shaped
/// one (the caller turns the latter into the labeled failure — the
/// scanner itself only reports what it saw).
#[derive(Debug, Clone, PartialEq, Eq)]
enum MarkerHit {
    Known(FragmentLang),
    Unknown(String),
}

/// A parsed fragment: its language tag and the verbatim cell text after
/// the marker. Only the Rust tag has an emitter at this Revision — py/ts
/// gate compile (validate_fragments), so the IR never carries them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    pub lang: FragmentLang,
    pub text: String,
}

/// A marker is `**` + a tag-shaped label + `:**` where the tag matches
/// `[a-z][a-z0-9_-]*`. The lowercase-initial shape is what separates a
/// marker from ordinary bold prose (`**Note:**` stays prose — it is not
/// tag-shaped, so it never trips unknown_tag_rejected). Returns the
/// parsed hit and the byte length of the full marker.
fn tag_marker_at(cell: &str, i: usize) -> Option<(MarkerHit, usize)> {
    let rest = cell[i..].strip_prefix("**")?;
    let tag_end = rest
        .char_indices()
        .take_while(|(_, c)| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_' || *c == '-')
        .last()
        .map(|(j, c)| j + c.len_utf8())
        .unwrap_or(0);
    if tag_end == 0 {
        return None;
    }
    let tag = &rest[..tag_end];
    rest[tag_end..].strip_prefix(":**")?;
    let hit = match FragmentLang::parse(tag) {
        Some(lang) => MarkerHit::Known(lang),
        None => MarkerHit::Unknown(tag.to_string()),
    };
    Some((hit, 2 + tag_end + 3))
}

/// `fragment_hygiene`'s banned tokens — defense-in-depth, not a sandbox:
/// fragments run on the invoking user's machine with the invoking user's
/// privileges, exactly like every other compiled artifact; this list
/// rejects the escape hatches a table cell should never legitimately need
/// (memory-unsafe code, compilation side doors, IO, processes, network,
/// environment reads, assembly). Substring match is deliberately
/// conservative — a false positive rejects a fragment at compile time
/// with a label naming the token, never a silent pass.
pub const FRAGMENT_BANNED: &[&str] = &[
    "unsafe",
    "extern",
    "include!",
    "include_str!",
    "include_bytes!",
    "std::fs",
    "std::process",
    "std::net",
    "std::env",
    "asm!",
    "Command",
];

/// Unescape the table-cell pipe escape (`\|` → `|`) at the
/// extraction/emission boundary (specodelic-q7uo, catitas A1): the parser
/// splits cells on unescaped pipes and keeps the escape sequence in the
/// cell text (guide.md's cell rule), so a fragment authored with `\|\|`,
/// `\|=`, or a closure `\|x\|` would otherwise emit Rust that does not
/// compile. The escape is markdown table transport, never fragment
/// content. A literal two-character replacement — the parser performs no
/// other unescaping, so this is the exact inverse of the cell rule (the
/// same shape as packs.rs's own cell-split protection, a different path).
/// Identity for text without the escape, so pre-existing artifacts stay
/// byte-identical.
fn unescape_cell_pipes(text: &str) -> String {
    text.replace("\\|", "|")
}

/// The shared body-shaping for executable rust bodies — inline
/// fragments (table cells) and `## Fragments` helper bodies alike
/// (task 3.1: one extraction discipline, two transports): trimmed,
/// with the table-cell pipe escape undone (identity for section text,
/// which never carries the escape — the fence body is not a cell).
fn shape_rust_body(text: &str) -> String {
    unescape_cell_pipes(text.trim())
}

/// Strict fragment parse: `Ok(None)` when the cell carries no marker in
/// fragment position (unchanged pre-Revision-15 behavior),
/// `Ok(Some(fragment))` for exactly one fragment-position marker with a
/// non-empty fragment after it, `Err(reason)` otherwise (more than one
/// fragment-position marker, or an empty fragment). Used by
/// `validate_fragments` (labeled) and, leniently via `fragment_of`, by
/// the IR/tla extraction — which only ever see compile-validated input.
pub fn fragment_of_strict(cell: &str) -> Result<Option<Fragment>, String> {
    // Only closed-set tags are markers at the grammar level; an unknown
    // tag-shaped occurrence is reported separately (unknown_tag_markers)
    // so the law-case grammar (`**identity:**` labels) and the fragment
    // grammar can share the `**word:**` shape without criminalizing each
    // other — the caller disambiguates by row kind.
    let mut positions: Vec<(usize, MarkerHit, usize)> = fragment_positions(cell)
        .into_iter()
        .filter(|(_, h, _)| matches!(h, MarkerHit::Known(_)))
        .collect();
    match positions.len() {
        0 => Ok(None),
        1 => {
            let (i, hit, marker_len) = positions.remove(0);
            let lang = match hit {
                MarkerHit::Known(lang) => lang,
                MarkerHit::Unknown(_) => unreachable!("filtered above"),
            };
            // The fragment runs to the cell's end or the code span's
            // closing backtick — cells author fragments as
            // `` `**lang:** <expr>` ``, so the closing backtick is part of
            // the cell, never part of the fragment (the expression
            // languages do not contain backticks).
            let after = &cell[i + marker_len..];
            let fragment = match after.find('`') {
                Some(j) => &after[..j],
                None => after,
            }
            .trim();
            if fragment.is_empty() {
                Err(format!(
                    "empty **{}:** fragment — write the code expression after the marker",
                    lang.tag()
                ))
            } else {
                Ok(Some(Fragment {
                    lang,
                    text: shape_rust_body(fragment),
                }))
            }
        }
        _ => Err(
            "more than one executable-fragment marker (**lang:**) in fragment position in one cell — at most one executable fragment per cell"
                .into(),
        ),
    }
}

/// Marker occurrences in FRAGMENT POSITION — the only occurrences that
/// opt a cell into executable translation (specodelic-sd1): at the very
/// start of the cell (after leading whitespace), or immediately after an
/// opening code-span backtick (`` `**lang:** <expr>` `` — the Revision 15
/// form). An occurrence anywhere else — mid-span (the corpus rows that
/// DEFINE the markers carry one, e.g. compile.md's
/// `predicate_fragment_opt_in`) or in prose between spans (verify.md's
/// `fragments_reach_verified`) — is a mention of the mechanism and never
/// extracts. Span scanning toggles on single backticks; the corpus does
/// not use multi-backtick spans. A marker after a CLOSING backtick
/// (`` `x` **rust:** y ``) is prose-position — a mention, not an opt-in.
/// Unknown tag-shaped markers in fragment position — the labeled-failure
/// scan for `unknown_tag_rejected`. Kind-blind; the CALLER exempts
/// law-kind predicate cells, where `**label:**` occurrences are the
/// law-case grammar (`spec.rs::law_case_labels`), never fragment markers.
/// Returns the unknown tag texts in cell order.
fn unknown_tag_markers(cell: &str) -> Vec<String> {
    fragment_positions(cell)
        .into_iter()
        .filter_map(|(_, h, _)| match h {
            MarkerHit::Unknown(tag) => Some(tag),
            MarkerHit::Known(_) => None,
        })
        .collect()
}

fn fragment_positions(cell: &str) -> Vec<(usize, MarkerHit, usize)> {
    let lead = cell.len() - cell.trim_start().len();
    let mut positions = vec![];
    let mut in_span = false;
    let mut prev: Option<char> = None;
    for (i, c) in cell.char_indices() {
        let prev_backtick = prev == Some('`');
        if c == '`' {
            in_span = !in_span;
        } else if let Some((hit, marker_len)) = tag_marker_at(cell, i) {
            let at_cell_start = i == lead;
            // The marker is span-opening when the backtick just before it
            // opened the span we are inside (`` `**rust:** … ``).
            let opens_span = in_span && prev_backtick;
            if at_cell_start || opens_span {
                positions.push((i, hit, marker_len));
            }
        }
        prev = Some(c);
    }
    positions
}

/// The lenient fragment parse for extraction contexts that only ever see
/// compile-validated input (`extract_model_ir`, `model_to_tla`): a
/// malformed cell yields `None` — compile itself never emits such an
/// artifact, so this cannot silently drop a real fragment.
pub fn fragment_of(cell: &str) -> Option<String> {
    fragment_of_strict(cell)
        .ok()
        .flatten()
        // The lenient path serves the Rust emitter's extraction contexts
        // (IR/tla) — py/ts are gated at validation, so they can never
        // reach here; filtering again keeps the "never a silent
        // fall-through to Rust" invariant robust against caller drift.
        .filter(|f| f.lang == FragmentLang::Rust)
        .map(|f| f.text)
}

fn hygiene_violation(fragment: &str) -> Option<&'static str> {
    FRAGMENT_BANNED
        .iter()
        .copied()
        .find(|t| fragment.contains(t))
}

fn fragment_error(row_id: &str, reason: impl std::fmt::Display) -> CompileError {
    CompileError {
        stage: "fragment_extraction".into(),
        message: format!(
            "row `{row_id}`: {reason} — fix or remove the **lang:** marker (executable predicate fragments, specodelic.md Revision 16)"
        ),
    }
}

/// The `no_emitter_labeled_failure` gate: a py/ts fragment is grammar-
/// valid but has no emitter — compile fails labeled, with the remediation
/// naming the per-language emitter follow-up, never a silent fall-through
/// to Rust emission and never a vacuous artifact.
fn emitter_gate_error(row_id: &str, lang: FragmentLang) -> CompileError {
    let (follow_up, lang) = match lang {
        FragmentLang::Py => ("add-py-fragment-emission", FragmentLang::Py),
        FragmentLang::Ts => ("add-ts-fragment-emission", FragmentLang::Ts),
        FragmentLang::Rust => unreachable!("rust has an emitter; the gate never fires for it"),
    };
    CompileError {
        stage: "compile.emission_failure".into(),
        message: format!(
            "row `{row_id}`: no emitter for **{}:** fragments — executable emission for {} is deferred of record (follow-up change: {follow_up}); **rust:** is the only executable tag at this Revision (no_emitter_labeled_failure, specodelic.md Revision 16)",
            lang.tag(),
            lang.tag()
        ),
    }
}

// ---------------------------------------------------------------------------
// Shared fragment definitions (## Fragments, add-shared-fragment-helpers)
// ---------------------------------------------------------------------------

/// One shared helper from the file's optional `## Fragments` section:
/// the `def <name>:` line's name plus the fenced rust body, captured
/// verbatim (extraction stays textual — design decision of record). A
/// fragment cell in the same file references a helper with the `@name`
/// token; compile inlines the referenced body at every use site as a
/// `({ <items> <name> })` block expression, so a predicate assertion and
/// an invariant fragment in the ModelIR carry their helpers identically.
#[derive(Debug, Clone, PartialEq, Eq)]
struct FragmentDef {
    name: String,
    body: String,
}

/// A labeled `## Fragments` failure — same stage as fragment extraction
/// (compile_is_total: one stage names the failure).
fn helper_error(reason: impl std::fmt::Display) -> CompileError {
    CompileError {
        stage: "fragment_extraction".into(),
        message: format!(
            "## Fragments: {reason} — fix the section or the @name reference (shared fragment definitions, add-shared-fragment-helpers)"
        ),
    }
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Parse the `## Fragments` body: `def <name>:` lines, each followed by
/// a fenced rust body. Lines that are neither defs nor fence content are
/// opaque prose (the linter's discipline — never parsed). Malformed defs
/// (bad name shape, missing fence or body, duplicate name) fail labeled.
fn fragment_defs(spec: &Spec) -> Result<Vec<FragmentDef>, CompileError> {
    let text = &spec.fragments_body;
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut defs: Vec<FragmentDef> = Vec::new();
    let mut lines = text.lines().peekable();
    while let Some(line) = lines.next() {
        let Some(rest) = line.trim().strip_prefix("def ") else {
            continue;
        };
        let name = rest.trim();
        let Some(name) = name.strip_suffix(':').map(str::trim) else {
            return Err(helper_error(format!(
                "malformed def line `{line}` — the shape is `def <name>:`"
            )));
        };
        let shape_ok = name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && name.chars().all(is_ident_char);
        if !shape_ok {
            return Err(helper_error(format!(
                "malformed helper name `{name}` — names are Rust identifiers ([A-Za-z_][A-Za-z0-9_]*)"
            )));
        }
        if defs.iter().any(|d| d.name == name) {
            return Err(helper_error(format!(
                "duplicate helper name `{name}` — references resolve to the single definition"
            )));
        }
        // Skip blank lines to the opening fence, then collect the body
        // to the closing fence.
        let mut body = String::new();
        let mut open = false;
        for l in lines.by_ref() {
            let t = l.trim();
            if !open {
                if t.starts_with("```") {
                    open = true;
                } else if !t.is_empty() {
                    return Err(helper_error(format!(
                        "def `{name}` has no fenced body — write ```rust fences around the helper's Rust items"
                    )));
                }
            } else if t.starts_with("```") {
                open = false;
                break;
            } else {
                body.push_str(l);
                body.push('\n');
            }
        }
        if open {
            return Err(helper_error(format!("def `{name}` has an unclosed fence")));
        }
        let body = shape_rust_body(&body);
        if body.is_empty() {
            return Err(helper_error(format!("def `{name}` has an empty body")));
        }
        defs.push(FragmentDef {
            name: name.to_string(),
            body,
        });
    }
    Ok(defs)
}

/// `@name` occurrences in a fragment or helper body: `@` at a word
/// boundary (never preceded by an identifier char) followed by an
/// identifier. Rust's pattern-binding `@` (`x @ Some(_)`) is always
/// preceded by an identifier and followed by a pattern, so it never
/// scans as a helper reference. The textual scan shares the hygiene
/// check's deliberately conservative imprecision (documented of record).
fn helper_refs(text: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find('@') {
        let boundary = rest[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !is_ident_char(c));
        let after = &rest[at + 1..];
        let name: String = after.chars().take_while(|&c| is_ident_char(c)).collect();
        let ident_start = name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
        if boundary && ident_start {
            let len = name.len();
            refs.push(name);
            rest = &after[len..];
        } else {
            rest = after;
        }
    }
    refs
}

/// Rewrite every `@name` token in `text` through `f`, keeping all other
/// bytes verbatim (single source of truth for the reference syntax —
/// validation and emission cannot drift on what counts as a reference).
fn rewrite_helper_refs(
    text: &str,
    mut f: impl FnMut(&str) -> Result<String, CompileError>,
) -> Result<String, CompileError> {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find('@') {
        let after = &rest[at + 1..];
        let name: String = after.chars().take_while(|&c| is_ident_char(c)).collect();
        let boundary = out.chars().next_back().is_none_or(|c| !is_ident_char(c));
        let ident_start = name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
        if boundary && ident_start {
            out.push_str(&rest[..at]);
            out.push_str(&f(&name)?);
            rest = &after[name.len()..];
        } else {
            out.push_str(&rest[..=at]);
            rest = after;
        }
    }
    out.push_str(rest);
    Ok(out)
}

/// Memoized helper expansion: a referenced def inlines as
/// `({ <body with its own @refs expanded> <name> })` — Rust items are
/// legal inside a block expression, so the rewrite is valid wherever the
/// fragment itself is (assertion operand, ModelIR invariant). Memoizing
/// keeps chains linear; the visiting stack turns mutual references into
/// the labeled cycle failure.
struct HelperExpander<'a> {
    defs: &'a [FragmentDef],
    memo: BTreeMap<String, String>,
}

impl<'a> HelperExpander<'a> {
    fn new(defs: &'a [FragmentDef]) -> Self {
        Self {
            defs,
            memo: BTreeMap::new(),
        }
    }

    fn def(&self, name: &str) -> Result<&'a FragmentDef, CompileError> {
        self.defs.iter().find(|d| d.name == name).ok_or_else(|| {
            helper_error(format!(
                "unknown fragment helper reference `@{name}` — no `def {name}:` in this file's `## Fragments` section"
            ))
        })
    }

    fn expand(&mut self, name: &str, stack: &mut Vec<String>) -> Result<String, CompileError> {
        if let Some(memo) = self.memo.get(name) {
            return Ok(memo.clone());
        }
        if let Some(pos) = stack.iter().position(|n| n == name) {
            let mut chain: Vec<String> = stack[pos..].to_vec();
            chain.push(name.to_string());
            return Err(helper_error(format!(
                "helper reference cycle: `{}` — inlining terminates by construction; break the cycle",
                chain.join("` -> `")
            )));
        }
        let body = self.def(name)?.body.clone();
        stack.push(name.to_string());
        let expanded = rewrite_helper_refs(&body, |r| {
            let inner = self.expand(r, stack)?;
            Ok(format!("({{{inner} {r}}})"))
        })?;
        stack.pop();
        let out = format!("({{{expanded}\n{name}}})");
        self.memo.insert(name.to_string(), out.clone());
        Ok(out)
    }
}

/// The fragment texts that may carry `@name` helper references: non-law
/// Property predicates and invariant-kind Constraint exprs that opted in
/// with the `**rust:**` marker, plus a law row's per-case bodies
/// (Revision 21 — a body is a fragment like any other, so the helper
/// rule reaches it; guards can never reach here — their rejection rows
/// run first in `validate_fragments`).
fn fragment_cell_texts(spec: &Spec) -> Vec<String> {
    let mut texts = Vec::new();
    for p in &spec.properties {
        let predicate = p.cells.get("predicate").map(String::as_str).unwrap_or("");
        if p.cells.get("kind").map(String::as_str) == Some("law") {
            texts.extend(
                crate::law::law_case_bodies(predicate)
                    .into_iter()
                    .map(|b| b.text),
            );
            continue;
        }
        if let Some(text) = fragment_of(predicate) {
            texts.push(text);
        }
    }
    for c in &spec.constraints {
        if c.cells.get("kind").map(String::as_str) == Some("invariant")
            && let Some(text) = fragment_of(c.cells.get("expr").map(String::as_str).unwrap_or(""))
        {
            texts.push(text);
        }
    }
    texts
}

/// Symbols the props emitter generates at the block's scope. A helper
/// colliding with one would silently shadow it inside the emitted block
/// — the labeled-failure list (design decision: collision, never shadow).
fn generated_symbol_names(spec: &Spec) -> BTreeSet<String> {
    let mut names: BTreeSet<String> = ["spec_gen".to_string(), "todo_predicate".to_string()]
        .into_iter()
        .collect();
    for p in &spec.properties {
        for b in blocks_for_row(p.id.clone(), p.cells.get("kind").map(String::as_str), p) {
            names.insert(b.fn_name);
        }
    }
    names
}

/// `v0`, `v1`, … — the strategy-binding namespace proptest! generates.
fn is_generated_binding(name: &str) -> bool {
    name.len() >= 2 && name.starts_with('v') && name[1..].chars().all(|c| c.is_ascii_digit())
}

/// The `## Fragments` half of fragment validation (runs first inside
/// `validate_fragments`, same stage): malformed defs, name collisions
/// with generated symbols, the inherited `FRAGMENT_BANNED` hygiene on
/// every helper body, unknown `@name` references from any fragment cell
/// or helper body, and reference cycles.
fn validate_fragment_helpers(spec: &Spec) -> Result<(), CompileError> {
    let defs = fragment_defs(spec)?;
    let generated = generated_symbol_names(spec);
    for d in &defs {
        if generated.contains(&d.name) {
            return Err(helper_error(format!(
                "helper `{}` collides with the generated symbol `{}` — rename the helper; a silent shadow is never allowed",
                d.name, d.name
            )));
        }
        if is_generated_binding(&d.name) {
            return Err(helper_error(format!(
                "helper `{}` collides with the generated symbol `{0}` (the strategy-binding namespace v0, v1, …) — rename the helper; a silent shadow is never allowed",
                d.name
            )));
        }
        if let Some(token) = hygiene_violation(&d.body) {
            return Err(helper_error(format!(
                "helper `{}` body contains banned token `{token}` (fragment_hygiene — helper bodies inherit the fragment token ban verbatim; the check is defense-in-depth, not a sandbox)",
                d.name
            )));
        }
    }
    let mut expander = HelperExpander::new(&defs);
    for text in fragment_cell_texts(spec)
        .iter()
        .chain(defs.iter().map(|d| &d.body))
    {
        for r in helper_refs(text) {
            expander.expand(&r, &mut Vec::new())?;
        }
    }
    Ok(())
}

/// Resolve and inline a fragment's `@name` helper references: each
/// referenced body expands at the use site (see `HelperExpander`), every
/// other byte is preserved. The fast path (no defs, no `@`) returns the
/// text unchanged — the pure-widening byte-preservation contract.
fn inline_helper_refs(spec: &Spec, fragment: &str) -> Result<String, CompileError> {
    let defs = fragment_defs(spec)?;
    if defs.is_empty() || !fragment.contains('@') {
        return Ok(fragment.to_string());
    }
    let mut expander = HelperExpander::new(&defs);
    rewrite_helper_refs(fragment, |r| expander.expand(r, &mut Vec::new()))
}

/// The lenient counterpart for extraction contexts that only ever see
/// compile-validated input — the same contract as `fragment_of`:
/// `compile_spec` has already failed labeled on the malformed section,
/// so an error here can only come from a direct unvalidated call, and
/// leaving the reference marker in the text fails visibly downstream
/// (the artifact does not compile), never silently drops it.
fn inline_helpers_lenient(spec: &Spec, fragment: &str) -> String {
    inline_helper_refs(spec, fragment).unwrap_or_else(|_| fragment.to_string())
}

/// Dead-helper warnings (the scenario's "unused definition warns"): a
/// def reachable from no fragment cell — directly or through other defs
/// — is dead weight; warned on the warnings channel, never a failure.
fn helper_warnings(spec: &Spec) -> Vec<String> {
    let Ok(defs) = fragment_defs(spec) else {
        return Vec::new();
    };
    if defs.is_empty() {
        return Vec::new();
    }
    let mut live: BTreeSet<String> = BTreeSet::new();
    let mut stack: Vec<String> = fragment_cell_texts(spec)
        .iter()
        .flat_map(|t| helper_refs(t))
        .collect();
    while let Some(name) = stack.pop() {
        if !live.insert(name.clone()) {
            continue;
        }
        if let Some(d) = defs.iter().find(|d| d.name == name) {
            stack.extend(helper_refs(&d.body));
        }
    }
    defs.iter()
        .filter(|d| !live.contains(&d.name))
        .map(|d| {
            format!(
                "dead helper `{}` in `## Fragments` — referenced by zero fragment cells; remove it or reference it as @{} (add-shared-fragment-helpers)",
                d.name, d.name
            )
        })
        .collect()
}

/// Every fragment-bearing cell in the spec is validated labeled before
/// any artifact is emitted (compile_is_total): single non-empty marker,
/// hygiene, and the three rejection rows — law-kind Property predicates,
/// non-invariant Constraint exprs, and Transition guards (no binding over
/// the program-counter model — deferred of record).
pub fn validate_fragments(spec: &Spec) -> Result<(), CompileError> {
    validate_fragment_helpers(spec)?;
    for p in &spec.properties {
        let predicate = p.cells.get("predicate").cloned().unwrap_or_default();
        let kind = p.cells.get("kind").cloned().unwrap_or_default();
        // unknown_tag_rejected: a tag-shaped marker outside the closed
        // set in fragment position fails labeled, naming the tag and the
        // closed set — never silence. Law-kind predicates are exempt:
        // their `**label:**` occurrences are the law-case grammar, not
        // fragment markers.
        if kind != "law" {
            // Revision 21: the per-case body grammar is law-only — a
            // case key on a non-law row is a labeled failure, checked
            // before the unknown-tag scan (the more specific defect
            // first).
            if let Some(failure) = crate::law::non_law_case_key_failure(&predicate) {
                return Err(fragment_error(&p.id, &failure.message));
            }
            if let Some(tag) = unknown_tag_markers(&predicate).first() {
                return Err(fragment_error(
                    &p.id,
                    format!(
                        "unknown fragment language tag `{tag}` — the closed tag set is {{rust, py, ts}} (fragment_language_closed)"
                    ),
                ));
            }
        }
        // Revision 21: a law-kind row's predicate carries one executable
        // body per named case (the per-case body grammar). Every defect
        // in that grammar — a fragment no case key introduces, a second
        // body for one case, a body naming a case the row does not
        // enumerate — is a labeled extraction failure. A case key on a
        // non-law row is rejected the same way: the grammar is law-only.
        if kind == "law" {
            let cell = crate::law::parse_law_cell(&predicate);
            if let Some(failure) = cell.failures.first() {
                return Err(fragment_error(&p.id, &failure.message));
            }
            for body in &cell.bodies {
                // The scanner only ever builds a body from a known tag,
                // so this arm is defense in depth — and it fails
                // labeled, never a silent fall-through to Rust.
                let Some(lang) = FragmentLang::parse(&body.tag) else {
                    return Err(fragment_error(
                        &p.id,
                        format!(
                            "the body of the case `{}` names the unknown tag `{}` — the closed tag set is {{rust, py, ts}} (fragment_language_closed)",
                            body.case, body.tag
                        ),
                    ));
                };
                if lang != FragmentLang::Rust {
                    return Err(emitter_gate_error(&p.id, lang));
                }
                if let Some(token) = hygiene_violation(&shape_rust_body(&body.text)) {
                    return Err(fragment_error(
                        &p.id,
                        format!(
                            "the body of the case `{}` contains banned token `{token}` (fragment_hygiene — fragments are compiled Rust, never a sandbox)",
                            body.case
                        ),
                    ));
                }
            }
            continue;
        }
        match fragment_of_strict(&predicate) {
            Err(reason) => return Err(fragment_error(&p.id, reason)),
            Ok(Some(fragment)) => {
                if fragment.lang != FragmentLang::Rust {
                    return Err(emitter_gate_error(&p.id, fragment.lang));
                }
                if let Some(token) = hygiene_violation(&fragment.text) {
                    return Err(fragment_error(
                        &p.id,
                        format!(
                            "fragment contains banned token `{token}` (fragment_hygiene — fragments are compiled Rust, never a sandbox)"
                        ),
                    ));
                }
            }
            Ok(None) => {}
        }
    }
    for c in &spec.constraints {
        let expr = c.cells.get("expr").cloned().unwrap_or_default();
        let kind = c.cells.get("kind").cloned().unwrap_or_default();
        // Kernel expressions (add-min-expr-kernel, specodelic-7ga): the
        // **kernel:** marker owns an invariant expr cell before the
        // fragment scan sees a tag-shaped `kernel` — a cell that opts
        // in and breaks the closed grammar fails labeled, naming the
        // offending atomic and the closed set
        // (kernel_grammar_violation_labeled); a cell that parses claims
        // the kernel path and skips the executable-fragment checks.
        if kind == "invariant" {
            match crate::kernel::parse_kernel_expr(&expr) {
                Err(e) => {
                    return Err(CompileError {
                        stage: "kernel_grammar".into(),
                        message: format!(
                            "row `{}`: {} (add-min-expr-kernel kernel_grammar_closed)",
                            c.id, e
                        ),
                    });
                }
                Ok(Some(_)) => continue,
                Ok(None) => {}
            }
        }
        if let Some(tag) = unknown_tag_markers(&expr).first() {
            return Err(fragment_error(
                &c.id,
                format!(
                    "unknown fragment language tag `{tag}` — the closed tag set is {{rust, py, ts}} (fragment_language_closed)"
                ),
            ));
        }
        match fragment_of_strict(&expr) {
            Err(reason) => return Err(fragment_error(&c.id, reason)),
            Ok(Some(fragment)) => {
                if fragment.lang != FragmentLang::Rust {
                    return Err(emitter_gate_error(&c.id, fragment.lang));
                }
                if kind != "invariant" {
                    return Err(fragment_error(
                        &c.id,
                        format!(
                            "**rust:** fragments are only defined for kind == invariant Constraints — `{}` is kind `{kind}`",
                            c.id
                        ),
                    ));
                }
                if let Some(token) = hygiene_violation(&fragment.text) {
                    return Err(fragment_error(
                        &c.id,
                        format!(
                            "fragment contains banned token `{token}` (fragment_hygiene — fragments are compiled Rust, never a sandbox)"
                        ),
                    ));
                }
            }
            Ok(None) => {}
        }
    }
    for t in &spec.transitions {
        if let Some(guard) = &t.guard
            // specodelic-sd1: only a marker in fragment position (a real
            // opt-in) rejects a guard — a mention of the marker in a
            // guard cell is prose, never an executable fragment. Any
            // fragment-position marker rejects, per tag: a known tag
            // (no binding) and an unknown tag alike fail labeled, never
            // silence.
            && (!matches!(fragment_of_strict(guard), Ok(None))
                || !unknown_tag_markers(guard).is_empty())
        {
            return Err(fragment_error(
                &t.id,
                "transition guards cannot carry **rust:** fragments — the program-counter model has no data binding a guard could constrain, so executable guards have no defined semantics in this Revision (decision of record, deferred)",
            ));
        }
    }
    Ok(())
}

/// Constraint id → its `expr` cell (empty when absent) — the single
/// source the `.tla` emitter and the IR's emitted values both read, so
/// the two can never diverge (specodelic-8nt).
fn constraint_exprs(spec: &Spec) -> BTreeMap<&str, &str> {
    spec.constraints
        .iter()
        .map(|c| {
            (
                c.id.as_str(),
                c.cells.get("expr").map(String::as_str).unwrap_or(""),
            )
        })
        .collect()
}

/// Extract the Model section into the IR without re-parsing prose.
pub fn extract_model_ir(spec: &Spec) -> ModelIr {
    let states = spec
        .states
        .iter()
        .map(|s| StateIr { id: s.id.clone() })
        .collect();
    let transitions = spec
        .transitions
        .iter()
        .map(|t| TransitionIr {
            id: t.id.clone(),
            from: t.from.clone(),
            to: t.to.clone(),
            guard: t.guard.clone(),
        })
        .collect();
    let mut emits = BTreeMap::new();
    let mut emits_values = BTreeMap::new();
    for s in &spec.states {
        let Some(raw) = s.cells.get("emits") else {
            continue;
        };
        // Prefer the typed [[link]] target over the raw cell text.
        let target = spec
            .links
            .iter()
            .find(|l| l.source == s.id && l.field == "states" && l.column == "emits")
            .map(|l| l.target.clone())
            .unwrap_or_else(|| raw.clone());
        // `file.row` → row id when the row is a local constraint; keep the
        // raw target when it points elsewhere.
        let row_id = target
            .rsplit_once('.')
            .map(|(_, r)| r.to_string())
            .unwrap_or_else(|| target.clone());
        let resolved = if spec.constraints.iter().any(|c| c.id == row_id) {
            row_id
        } else {
            target
        };
        emits.insert(s.id.clone(), resolved.clone());
        // The value the emitter writes into Output: the effect-Constraint's
        // expr when the target is a local constraint row, the raw target
        // otherwise — computed through the same exprs map model_to_tla
        // reads, so IR and artifact cannot diverge (specodelic-8nt).
        let exprs = constraint_exprs(spec);
        let value = exprs
            .get(resolved.as_str())
            .copied()
            .unwrap_or(resolved.as_str());
        emits_values.insert(s.id.clone(), value.to_string());
    }
    // Executable invariant fragments (Revision 15): kind == invariant rows
    // whose expr opts in. Lenient parse — compile_spec's labeled validation
    // has already rejected malformed fragments before any artifact exists.
    let invariants = spec
        .constraints
        .iter()
        .filter(|c| c.cells.get("kind").map(String::as_str) == Some("invariant"))
        .filter_map(|c| {
            fragment_of(c.cells.get("expr").map(String::as_str).unwrap_or("")).map(|fragment| {
                InvariantIr {
                    id: c.id.clone(),
                    // Shared helpers inline here too (add-shared-fragment-
                    // helpers): the block-expression rewrite keeps the IR
                    // fragment self-contained wherever model_check wraps it.
                    fragment: inline_helpers_lenient(spec, &fragment),
                }
            })
        })
        .collect();
    // Guard citations (slice 1, specodelic-txo): invariant rows whose
    // expr cell IS a citation expression (no fragment marker) carry
    // verbatim; prose cells stay out — inert as before.
    let guard_citations = spec
        .constraints
        .iter()
        .filter(|c| c.cells.get("kind").map(String::as_str) == Some("invariant"))
        .filter_map(|c| {
            let expr = c.cells.get("expr").map(String::as_str).unwrap_or("");
            // The parse itself is discarded: it proves the cell IS a
            // citation expression; the stripped text is carried verbatim.
            (fragment_of(expr).is_none() && parse_citation_expr(expr).is_some())
                .then(|| (c.id.clone(), strip_guard_cell(expr).to_string()))
        })
        .collect();
    // Kernel expressions (add-min-expr-kernel, Tier B, specodelic-7ga):
    // invariant rows whose expr cell opts into kernel translation carry
    // verbatim, keyed by id; prose cells stay out — pure widening.
    // Lenient parse: compile_spec's labeled validation has already
    // rejected an opted-in cell that breaks the closed grammar before
    // any artifact exists.
    let guard_kernel = spec
        .constraints
        .iter()
        .filter(|c| c.cells.get("kind").map(String::as_str) == Some("invariant"))
        .filter_map(|c| {
            let expr = c.cells.get("expr").map(String::as_str).unwrap_or("");
            // The parse itself is discarded: it proves the cell opted
            // in and is grammar-valid; the marker-stripped content is
            // carried verbatim (the same shaping the citation path
            // uses).
            (fragment_of(expr).is_none()
                && parse_citation_expr(expr).is_none()
                && crate::kernel::parse_kernel_expr(expr)
                    .ok()
                    .flatten()
                    .is_some())
            .then(|| crate::kernel::kernel_cell_content(expr))
            .flatten()
            .map(|text| (c.id.clone(), text))
        })
        .collect();
    ModelIr {
        states,
        transitions,
        emits,
        emits_values,
        invariants,
        guard_citations,
        guard_kernel,
    }
}

// ---------------------------------------------------------------------------
// proptest! emission (properties_to_proptest)
// ---------------------------------------------------------------------------

/// A generated proptest! block (one per property row, one per required
/// case for law-kind rows).
struct PropBlock {
    /// Rust identifier for the block fn (sanitized row id, + `_<case>` for laws).
    fn_name: String,
    /// The row's verbatim id (kept for `compile_preserves_ids`).
    id: String,
    /// Required case name for law rows (`None` for unit rows).
    case: Option<String>,
    /// Verbatim generator cell text.
    generator: String,
    /// Verbatim predicate cell text.
    predicate: String,
    /// The row's executable fragment when its predicate opts in with the
    /// `**rust:**` marker (specodelic.md Revision 15) — `None` keeps the
    /// un-translated placeholder.
    fragment: Option<String>,
    /// Generator function names referenced by the row, in order.
    gens: Vec<String>,
}

/// Compile the Properties table to a proptest! source file: one block per
/// row (`proptest_block_per_property`), one block per required case for
/// law-kind rows (`law_property_compiles_required_cases`).
///
/// Emission is compilable scaffolding, not a pseudo-code translation: the
/// strategy is a call into the generated `spec_gen` helpers (yielding
/// plain Strings — Revision 15 retired the `GenVal` wrapper) and the
/// predicate is carried verbatim through `todo_predicate!`, which compiles
/// here and only fails when executed — execution is `verify`'s (1pv) job.
/// A predicate opting in with the `**rust:**` marker (specodelic.md
/// Revision 15) is emitted verbatim as a real `assert!` body instead.
pub fn properties_to_proptest(spec: &Spec) -> String {
    let file_id = &spec.intent.id;
    let mut blocks: Vec<PropBlock> = spec
        .properties
        .iter()
        .flat_map(|p| blocks_for_row(p.id.clone(), p.kind.as_deref(), p))
        .collect();
    // Two distinct ids can sanitize to the same Rust fn name (e.g. `p.1`
    // and `p_1`) — disambiguate so the artifact never has duplicate fns.
    let mut used: BTreeSet<String> = BTreeSet::new();
    for b in &mut blocks {
        let base = b.fn_name.clone();
        let mut n = 2;
        while !used.insert(b.fn_name.clone()) {
            b.fn_name = format!("{base}_{n}");
            n += 1;
        }
    }
    // Shared fragment definitions (add-shared-fragment-helpers): a
    // predicate fragment's `@name` references inline each referenced
    // helper body at the use site — self-contained per-row blocks, the
    // inline-not-shared-module decision of record. The fast path leaves
    // markerless fragments byte-identical.
    for b in &mut blocks {
        if let Some(fragment) = b.fragment.as_mut() {
            *fragment = inline_helpers_lenient(spec, fragment);
        }
    }

    let mut out = String::new();
    out.push_str(&format!(
        "//! proptest scaffolding for `{file_id}` — generated by `spk compile`.\n\
         //! Source of truth: the spec file itself — edit there, not here.\n\
         //! Each block quotes its row's generator and predicate verbatim;\n\
         //! a predicate opting in with the **rust:** marker (executable\n\
         //! fragment, specodelic.md Revision 15) is emitted verbatim as the\n\
         //! block's assertion body; predicates not yet translated fail in\n\
         //! `verify` via `todo_predicate!`, never at artifact emission.\n\n"
    ));
    if blocks.is_empty() {
        // No property rows: an empty-but-present artifact beats an absent one.
        out.push_str("//! (no Property rows in the source spec)\n");
        return out;
    }
    // specodelic-8aq: the import is only needed by proptest! blocks — a
    // fully parameterless artifact stays warning-clean without it.
    if blocks.iter().any(|b| !b.gens.is_empty()) {
        out.push_str("use proptest::prelude::*;\n\n");
    }
    out.push_str(
        "/// Marker for predicates not yet translated to Rust: compiles here,\n\
         /// panics when executed — execution is `verify`'s job. A predicate\n\
         /// opting in with **rust:** compiles as a real assertion instead\n\
         /// (executable predicate fragment, specodelic.md Revision 15).\n\
         macro_rules! todo_predicate {\n    ($reason:expr) => {\n        todo!(\"{}\", $reason)\n    };\n}\n\n",
    );

    // spec_gen helpers: one per distinct generator referenced anywhere.
    let mut gens: Vec<&str> = vec![];
    for b in &blocks {
        for g in &b.gens {
            if !gens.contains(&g.as_str()) {
                gens.push(g);
            }
        }
    }
    if !gens.is_empty() {
        out.push_str(
            "/// Generated strategies — one per generator named in the spec rows.\n\
             /// Each yields a String (the generator's name; executable\n\
             /// fragments bind these directly — Revision 15).\n\
             pub mod spec_gen {\n    use super::*;\n\n",
        );
        for g in &gens {
            // A generator whose name is a FIXTURE name materializes
            // through the tool's fixture registry — a real concrete
            // input the real checker consumes (the fixture seam,
            // openspec/changes/add-verify-runner-seams design D2).
            // Every other generator keeps the placeholder emission
            // byte-for-byte: the generator's own name.
            if crate::fixtures::is_known(g) {
                out.push_str(&format!(
                    "    pub fn {g}() -> impl Strategy<Value = String> {{\n        Just(specodelic::fixtures::materialize({g:?}).expect(\"the {g} fixture materializes\"))\n    }}\n\n"
                ));
            } else {
                out.push_str(&format!(
                    "    pub fn {g}() -> impl Strategy<Value = String> {{\n        Just({g:?}.into())\n    }}\n\n"
                ));
            }
        }
        out.push_str("}\n\n");
    }

    // specodelic-8aq: a generator cell yielding no strategy params cannot
    // carry (pat in strategy) — proptest! requires one on every fn, so a
    // bare fn inside the macro is a parse error that left the scratch
    // crate uncompilable. Such blocks are emitted OUTSIDE the macro as
    // plain #[test] fns (same honest todo_predicate!/assert! bodies,
    // same metadata comments); blocks with strategies keep the proptest!
    // form. Blocks are emitted in row order, opening and closing the
    // macro around contiguous runs.
    let mut in_macro = false;
    for b in &blocks {
        if b.gens.is_empty() {
            if in_macro {
                out.push_str("}\n\n");
                in_macro = false;
            }
            out.push_str(&format!("// id: {}\n", b.id));
            if let Some(case) = &b.case {
                out.push_str(&format!("// case: {case}\n"));
            }
            out.push_str(&format!("// generator: {}\n", b.generator));
            out.push_str(&format!("// predicate: {}\n", b.predicate));
            out.push_str("#[test]\n");
            out.push_str(&format!("fn {}() {{\n", b.fn_name));
            push_predicate_body(&mut out, b);
            out.push_str("}\n\n");
        } else {
            if !in_macro {
                out.push_str("proptest! {\n");
                in_macro = true;
            }
            out.push_str(&format!("    // id: {}\n", b.id));
            if let Some(case) = &b.case {
                out.push_str(&format!("    // case: {case}\n"));
            }
            out.push_str(&format!("    // generator: {}\n", b.generator));
            out.push_str(&format!("    // predicate: {}\n", b.predicate));
            out.push_str("    #[test]\n");
            let args = b
                .gens
                .iter()
                .enumerate()
                .map(|(i, g)| format!("v{i} in spec_gen::{g}()"))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("    fn {}({}) {{\n", b.fn_name, args));
            push_predicate_body(&mut out, b);
            out.push_str("    }\n");
        }
    }
    if in_macro {
        out.push_str("}\n\n");
    }
    out
}

/// The body of an emitted property block — the verbatim executable
/// fragment as an `assert!` (specodelic.md Revision 15), or the honest
/// `todo_predicate!` placeholder that only fails when executed.
fn push_predicate_body(out: &mut String, b: &PropBlock) {
    if let Some(fragment) = &b.fragment {
        // Executable predicate fragment (specodelic.md Revision 15):
        // the Rust expression after the **rust:** marker, verbatim —
        // no mini-language. The assertion failure message names the
        // row id so the failing block is attributable.
        out.push_str(&format!(
            "        assert!(\n            {},\n            \"predicate `{}` violated (executable fragment)\",\n        );\n",
            fragment, b.id
        ));
    } else {
        out.push_str(&format!("        todo_predicate!({:?});\n", b.predicate));
    }
}

/// One or more blocks for a single Property row: unit rows yield exactly
/// one; law rows yield one per required case, read from the row's own
/// predicate per `law_requires_cases` (floor: identity, associativity).
fn blocks_for_row(id: String, kind: Option<&str>, row: &crate::spec::Row) -> Vec<PropBlock> {
    let generator = row.cells.get("generator").cloned().unwrap_or_default();
    let predicate = row.cells.get("predicate").cloned().unwrap_or_default();
    let gens = generator_names(&generator);
    let base = sanitize_ident(&id);
    match kind {
        Some("law") => {
            // Revision 21: the case's own body, when the cell carries
            // one; a case without a body keeps the honest placeholder.
            let bodies = law_case_fragments(&predicate);
            required_law_cases(&predicate)
                .into_iter()
                .map(|case| PropBlock {
                    fn_name: format!("{base}_{}", sanitize_ident(&case)),
                    id: id.clone(),
                    case: Some(case.clone()),
                    fragment: bodies.get(&case).cloned(),
                    generator: generator.clone(),
                    predicate: predicate.clone(),
                    gens: gens.clone(),
                })
                .collect()
        }
        _ => vec![PropBlock {
            fn_name: base,
            id,
            case: None,
            fragment: fragment_of(&predicate),
            generator,
            predicate,
            gens,
        }],
    }
}

/// The per-case bodies of a law row's predicate cell: the case label →
/// its body text, shaped like every other fragment (trimmed, the
/// table-cell pipe escape undone).
fn law_case_fragments(predicate: &str) -> BTreeMap<String, String> {
    crate::law::law_case_bodies(predicate)
        .into_iter()
        .map(|body| (body.case, shape_rust_body(&body.text)))
        .collect()
}

/// Generator identifiers in a generator cell: every `name(` occurrence.
fn generator_names(cell: &str) -> Vec<String> {
    let re = regex::Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").expect("static regex");
    let mut names: Vec<String> = vec![];
    for caps in re.captures_iter(cell) {
        let name = caps[1].to_string();
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

/// Required cases for a law row: `**case:**` labels in the predicate;
/// falls back to the `law_requires_cases` floor (identity, associativity).
fn required_law_cases(predicate: &str) -> Vec<String> {
    let cases = crate::law::law_case_labels(predicate);
    if cases.is_empty() {
        vec!["identity".into(), "associativity".into()]
    } else {
        cases
    }
}

/// Rust keywords — a sanitized identifier equal to one of these is a
/// keyword, not a fn name (`fn fn(…)` does not compile; hostile-input
/// gate, specodelic-suz). Strict + reserved keywords; weak keywords
/// (`union`, `gen`) are usable as fn names and excluded.
const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield",
];

/// The emitted-identifier cap (hostile-input gate, specodelic-suz): a
/// hostile 200-char id must not emit a 200-char fn name. Prefix above
/// the cap + a 16-hex FNV-1a suffix of the full id = 64 chars total.
const MAX_IDENT_LEN: usize = 64;

/// Sanitize a spec id into a valid Rust identifier (ids are never altered —
/// the verbatim id travels in the block's comment for id preservation).
/// Hostile-input gate (specodelic-suz): reserved words are prefixed, the
/// empty id becomes `_`, and ids beyond the cap are truncated with a
/// stable FNV-1a hash suffix so same-prefix ids never collide.
fn sanitize_ident(id: &str) -> String {
    let s: String = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let mut s = if s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("_{s}")
    } else {
        s
    };
    if s.is_empty() {
        return "_".into();
    }
    if RUST_KEYWORDS.contains(&s.as_str()) {
        s = format!("_{s}");
    }
    if s.len() > MAX_IDENT_LEN {
        let hash = fnv1a_64(id.as_bytes());
        s = format!("{}_{:016x}", &s[..MAX_IDENT_LEN - 17], hash);
    }
    s
}

/// FNV-1a 64-bit — stable across Rust releases (std's DefaultHasher is
/// keyed per-process and its algorithm may change; emitted artifacts are
/// byte-stable, so the suffix must be too).
fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

// ---------------------------------------------------------------------------
// TLA+ emission (model_to_tla — completes the functor's second category)
// ---------------------------------------------------------------------------

/// Compile the Model section to one TLA+ module, per `openspec/specs/compile/spec.md`'s
/// `model_to_tla`: always emitted, regardless of which model_check backend
/// (native stateright or TLC) later runs against the model. Each State
/// becomes a value in the state variable's range, each Transition one
/// disjunct of the `Next` action, and the `emits` mapping becomes an
/// `Output` function whose domain is exactly the emitting states — never a
/// default or null entry.
///
/// Spec-side guards and emits expressions are prose, not TLA+: they travel
/// verbatim in comments while the emitted disjuncts are valid TLA+ over the
/// state variable — the same scaffolding philosophy as the proptest
/// artifacts; semantic checking is `model_check`'s (nx7) domain.
pub fn model_to_tla(spec: &Spec) -> String {
    let ir = extract_model_ir(spec);
    let module = module_name(spec);
    // TLA+ module-header form: dashes + MODULE name + dashes on ONE line
    // (Specifying Systems §: a module's first line must be four or more
    // dashes, then MODULE, then the name, then four or more dashes). The
    // earlier three-line box was not a parseable header — opt-in TLC
    // could not open the module (specodelic-len).
    let mut out = String::new();
    out.push_str(&format!("---- MODULE {module} ----\n"));
    out.push_str(&format!(
        "\\* Generated by `spk compile` from spec `{}` — source of truth is the\n\\* spec file; edit there, not here. Extends nothing: the module is\n\\* self-contained so any engine can open it.\n\n",
        spec.intent.id
    ));

    // Executable-invariant manifest (specodelic.md Revision 15): id +
    // fragment verbatim as module comments. Purely a staleness key and a
    // human-reviewable record — no engine parses it — but a fragment edit
    // changes these bytes, so the artifact hash covers fragment edits
    // (model_check.md's rerun_on_model_change, reworded to name fragments).
    if ir.invariants.is_empty() {
        out.push_str("\\* No executable invariant fragments (Revision 15).\n\n");
    } else {
        out.push_str(
            "\\* Executable invariant fragments — id + verbatim **rust:** fragment;\n\\* carried in the module so the artifact hash covers fragment edits.\n",
        );
        for inv in &ir.invariants {
            out.push_str(&format!(
                "\\* INVARIANT {} **rust:** {}\n",
                inv.id, inv.fragment
            ));
        }
        out.push('\n');
    }

    // Each State becomes a value in the state variable's range.
    let state_values: Vec<String> = ir.states.iter().map(|s| format!("\"{}\"", s.id)).collect();
    let init = ir.states.first().map(|s| s.id.as_str()).unwrap_or("");
    out.push_str("\\* Each State becomes a value in the state variable's range.\n");
    out.push_str(&format!(
        "StateValues == {{{}}}\n\n",
        state_values.join(", ")
    ));
    out.push_str("VARIABLES vpc   \\* the state variable (program counter)\n\nvars == <<vpc>>\n\n");
    out.push_str("TypeOK == vpc \\in StateValues\n\n");

    // Init — the first listed state: the format has no explicit initial
    // marker, so the first State in the spec is the module's initial value
    // (recorded here and in the emitted comment so the choice is visible).
    out.push_str("\\* Initial state: first listed in the spec (the format has no explicit initial marker).\n");
    out.push_str(&format!("Init == vpc = \"{init}\"\n\n"));

    // Next: one disjunct per transition, guarded on from; the spec's own
    // guard text is quoted verbatim in the disjunct comment.
    out.push_str("Next ==\n");
    for t in &ir.transitions {
        let guard = t.guard.as_deref().unwrap_or("null guard");
        out.push_str(&format!(
            "  \\* {}: {} -> {} (guard: {})\n  \\/ vpc = \"{}\" /\\ vpc' = \"{}\"\n",
            t.id, t.from, t.to, guard, t.from, t.to
        ));
    }
    if ir.transitions.is_empty() {
        out.push_str("  \\* (no transitions in the source spec)\n  \\/ FALSE\n");
    }
    // Stuttering is an allowed step: the spec's guards travel as prose
    // comments, so an engine treats every disjunct above as always-enabled
    // and would flag a state with no outgoing transition as a deadlock.
    // Keeping the program counter unchanged closes that — opt-in TLC runs
    // clean on terminal states too (specodelic-len).
    out.push_str("  \\* stuttering: guards are prose (uninterpreted) — a terminal\n  \\* state must not read as an engine-side deadlock\n  \\/ UNCHANGED vpc\n");
    out.push('\n');

    // Output: Moore-machine output function — exactly the emitting states,
    // never a default or null entry; empty function when none emit.
    if ir.emits.is_empty() {
        out.push_str(
            "\\* No state carries an `emits` field — Output is simply empty.\nOutput == << >>\n",
        );
    } else {
        out.push_str("\\* One entry per state with an `emits` field — domain is exactly\n\\* the emitting states; each value is the effect-Constraint's expr.\nOutput ==\n");
        // Per `model_to_tla`, each entry maps the state value to the
        // effect-kind Constraint's `expr` (verbatim; the raw reference as
        // fallback when the target is not a local constraint row).
        let exprs = constraint_exprs(spec);
        let entries: Vec<String> = ir
            .emits
            .iter()
            .map(|(state, constraint)| {
                let value = exprs
                    .get(constraint.as_str())
                    .copied()
                    .unwrap_or(constraint);
                format!("\"{state}\" :> \"{value}\"")
            })
            .collect();
        out.push_str(&entries.join(" @@\n"));
        out.push('\n');
    }
    out.push('\n');
    out.push_str("============================================================================\n");
    out
}

/// TLA+ module identifier from the artifact stem: letters and digits
/// only, starting with a letter (TLA+ forbids underscores in module
/// names). The artifact stem, not the raw file stem — a `spec.md` file
/// (single tree) names its module from the id, like its artifacts.
fn module_name(spec: &Spec) -> String {
    let stem = artifact_stem(spec);
    let mut s: String = stem.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    if s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        s.insert(0, 'M');
    }
    if s.is_empty() {
        s = "Module".into();
    }
    s
}

// ---------------------------------------------------------------------------
// Pipeline contract
// ---------------------------------------------------------------------------

/// One complete compilation — every artifact due under the current backend
/// decision (TOML + proptest source + ModelIR; the model module lands after
/// `mp1` row 6). `compile_is_total`: either this, or a labeled failure.
#[derive(Debug, Clone, Serialize)]
pub struct Compiled {
    /// The `constraint_table_to_toml` artifact.
    pub toml: String,
    /// The `properties_to_proptest` artifact.
    pub props: String,
    /// The `model_to_tla` artifact — always emitted, regardless of which
    /// model_check backend later runs against it.
    pub tla: String,
    /// The extracted Model IR (stands in for the model module until the
    /// backend decision; preserves state/transition ids).
    pub model_ir: ModelIr,
    /// Non-fatal compile findings (add-shared-fragment-helpers): a
    /// `## Fragments` def reachable from no fragment cell — the
    /// dead-helper warning rides the existing warnings channel
    /// (`Written::warnings`), never the issues channel and never a
    /// failure. Empty for files without the section (pure widening).
    pub warnings: Vec<String>,
}

/// A labeled, single failure — never a silent partial result.
#[derive(Debug, Clone, Serialize)]
pub struct CompileError {
    /// Which stage failed: `precondition_satisfied`, `no_semantic_drift`,
    /// `compile_preserves_ids`.
    pub stage: String,
    pub message: String,
}

/// `precondition_satisfied` — the file must pass lint with zero issues
/// (linted and covered). Implemented by reusing the existing lint pass,
/// not a second implementation.
pub fn precondition_satisfied(spec: &Spec, report: &Report) -> Result<(), CompileError> {
    let path = spec
        .path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let file_id = &spec.intent.id;
    let issues: Vec<&lint::Issue> = report
        .issues
        .iter()
        .filter(|i| i.file == *file_id || (!path.is_empty() && i.file == path))
        .collect();
    if issues.is_empty() {
        return Ok(());
    }
    let names: Vec<&str> = issues.iter().map(|i| i.rule_id.as_str()).collect();
    Err(CompileError {
        stage: "precondition_satisfied".into(),
        message: format!(
            "file has {} lint finding(s) — compile requires a linted-and-covered file (fired: {}). Note: lint is corpus-wide, so a file that references other specs only passes in the whole-corpus context — run `spk compile specs`",
            issues.len(),
            names.join(", ")
        ),
    })
}

/// The artifact filename stem for a spec: the file stem when on disk,
/// else the intent id with `.` → `-`. A `spec.md` file derives its
/// identity from its parent directory (Revision 18 naming law,
/// specodelic-jf1) — a flat `spec` stem would collide across every
/// capability in the single tree, so the id in dash form names those
/// artifacts (which keeps every migrated corpus artifact's name stable:
/// id `linter.graph_shape` → `linter-graph_shape`, exactly the old
/// corpus file stem).
pub fn artifact_stem(spec: &Spec) -> String {
    if spec
        .path
        .as_ref()
        .and_then(|p| p.file_name().and_then(|s| s.to_str()))
        == Some("spec.md")
    {
        return spec.intent.id.replace('.', "-");
    }
    spec.path
        .as_ref()
        .and_then(|p| p.file_stem().and_then(|s| s.to_str()))
        .map(|s| s.to_string())
        .unwrap_or_else(|| spec.intent.id.replace('.', "-"))
}

/// The outcome of `write_artifacts`: the written artifact paths plus any
/// non-fatal warnings — the rustfmt fallback fact rides these so callers
/// surface it on their existing warning channel.
pub struct Written {
    /// Written artifact paths (`<stem>.toml`, `<stem>_props.rs`, `<stem>.tla`).
    pub paths: Vec<String>,
    /// Non-fatal warnings (e.g. rustfmt unavailable — unformatted bytes
    /// written, command still succeeded).
    pub warnings: Vec<String>,
}

/// Write a compiled spec's three artifacts (`<stem>.toml`,
/// `<stem>_props.rs`, `<stem>.tla`) into `out_dir` (created on demand);
/// returns written paths plus non-fatal warnings; a write failure is a
/// labeled error. Flat-layout form shared with the `spk compile`
/// command path — orchestrate routes through `write_artifacts_into`.
///
/// The `*.rs` artifact is made rustfmt-clean here (gh-15): formatted
/// with rustfmt (edition 2021) before writing, so no downstream
/// `cargo fmt --check` gate churns on formatting-only diffs; the
/// `.toml`/`.tla` companions are not rustfmt's business. A missing or
/// failing rustfmt is a graceful labeled fallback — unformatted bytes
/// written, command succeeds, fact carried in `Written::warnings`.
pub fn write_artifacts(spec: &Spec, compiled: &Compiled, out_dir: &str) -> Result<Written, String> {
    write_artifacts_into(std::path::Path::new(out_dir), spec, compiled)
}

/// Write into an EXPLICIT artifact dir — the `artifact_dir` routing
/// decision (gh-13) — so orchestrate's written paths and every later
/// consumer of the layout agree.
pub fn write_artifacts_into(
    dir: &std::path::Path,
    spec: &Spec,
    compiled: &Compiled,
) -> Result<Written, String> {
    let stem = artifact_stem(spec);
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("could not create out-dir {}: {e}", dir.display()))?;
    let toml_path = dir.join(format!("{stem}.toml"));
    let props_path = dir.join(format!("{stem}_props.rs"));
    let tla_path = dir.join(format!("{stem}.tla"));
    let (props, mut warnings) = match rustfmt_clean(compiled.props.clone()) {
        Ok(formatted) => (formatted, Vec::new()),
        Err(e) => (
            compiled.props.clone(),
            vec![format!(
                "wrote unformatted {} ({e}) — install rustfmt so compile emits fmt-clean artifacts",
                props_path.display()
            )],
        ),
    };
    // Compile's own non-fatal findings (the dead-helper warning) ride
    // the same channel as the rustfmt fallback — one warnings channel,
    // never the issues channel (add-shared-fragment-helpers).
    warnings.splice(0..0, compiled.warnings.iter().cloned());
    std::fs::write(&toml_path, &compiled.toml)
        .map_err(|e| format!("could not write {}: {e}", toml_path.display()))?;
    std::fs::write(&props_path, &props)
        .map_err(|e| format!("could not write {}: {e}", props_path.display()))?;
    std::fs::write(&tla_path, &compiled.tla)
        .map_err(|e| format!("could not write {}: {e}", tla_path.display()))?;
    Ok(Written {
        paths: vec![
            toml_path.display().to_string(),
            props_path.display().to_string(),
            tla_path.display().to_string(),
        ],
        warnings,
    })
}

/// Format generated Rust source with rustfmt (edition 2021) via stdin.
/// `Ok(formatted)` on success; `Err(reason)` when rustfmt is unavailable
/// or rejects the source — the caller falls back to the unformatted
/// bytes (gh-15: a missing rustfmt must never turn compile into a hard
/// failure).
fn rustfmt_clean(source: String) -> Result<String, String> {
    use std::io::Write as _;
    use std::process::{Command, Stdio};
    let mut child = Command::new("rustfmt")
        .args(["--edition", "2021", "--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("rustfmt unavailable: {e}"))?;
    let mut stdin = child.stdin.take().expect("stdin is piped");
    // Write stdin from a thread (the std::process::Child docs pattern) so
    // a chatty rustfmt can never deadlock against a large source: the
    // writer proceeds independently and drops the handle at EOF.
    let writer = std::thread::spawn(move || {
        stdin
            .write_all(source.as_bytes())
            .map_err(|e| format!("rustfmt stdin: {e}"))
    });
    let fed = writer.join().expect("stdin writer thread");
    let out = child.wait_with_output().expect("rustfmt child runs");
    fed?;
    if !out.status.success() {
        return Err(format!(
            "rustfmt failed ({}): {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("rustfmt output: {e}"))
}

/// Compile one spec file through the full contract. Idempotent and
/// byte-stable: the same input yields byte-identical artifacts.
pub fn compile_spec(spec: &Spec) -> Result<Compiled, CompileError> {
    // fragment_extraction — every fragment-bearing cell is validated
    // labeled before any artifact is emitted (compile_is_total; Revision 15).
    validate_fragments(spec)?;
    // model_to_tla needs a non-empty state range; lint's `model_present`
    // gate covers this for CLI runs, but the lib contract is total — a
    // spec without states fails labeled, never with a malformed module.
    if spec.states.is_empty() {
        return Err(CompileError {
            stage: "model_to_tla".into(),
            message: "Model section has no states — nothing to compile into a module".into(),
        });
    }
    let toml = constraints_to_toml(spec);
    let model_ir = extract_model_ir(spec);
    let props = properties_to_proptest(spec);
    let tla = model_to_tla(spec);
    let warnings = helper_warnings(spec);

    // no_semantic_drift — re-parsing the TOML and re-emitting it yields
    // output identical to the original compile.
    let doc: ConstraintsDoc = toml::from_str(&toml).map_err(|e| CompileError {
        stage: "no_semantic_drift".into(),
        message: format!("emitted TOML failed to re-parse: {e}"),
    })?;
    let re_emitted = toml::to_string(&doc).map_err(|e| CompileError {
        stage: "no_semantic_drift".into(),
        message: format!("re-emission failed: {e}"),
    })?;
    if re_emitted != toml {
        return Err(CompileError {
            stage: "no_semantic_drift".into(),
            message: "re-parsed TOML re-emits to different bytes".into(),
        });
    }

    // compile_preserves_ids — every source id appears, unchanged, in at
    // least one artifact. Compared as structured id sets, not substrings:
    // constraint ids from the TOML doc, the verbatim `// id:` lines in the
    // props artifact, the intent id in the TOML `source` field, and
    // state/transition ids in the ModelIR.
    let mut artifact_ids: BTreeSet<String> = BTreeSet::new();
    artifact_ids.insert(doc.source.clone());
    for c in &doc.constraints {
        artifact_ids.insert(c.id.clone());
    }
    for line in props.lines() {
        if let Some(id) = line.trim().strip_prefix("// id: ") {
            artifact_ids.insert(id.trim().to_string());
        }
    }
    let missing: Vec<String> = spec
        .defined_ids()
        .into_iter()
        .filter(|id| !artifact_ids.contains(id) && !ir_contains_id(&model_ir, id))
        .collect();
    if !missing.is_empty() {
        return Err(CompileError {
            stage: "compile_preserves_ids".into(),
            message: format!("ids dropped in translation: {}", missing.join(", ")),
        });
    }

    Ok(Compiled {
        toml,
        props,
        tla,
        model_ir,
        warnings,
    })
}

fn ir_contains_id(ir: &ModelIr, id: &str) -> bool {
    ir.states.iter().any(|s| s.id == id)
        || ir.transitions.iter().any(|t| t.id == id)
        || ir.emits.keys().any(|k| k == id)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests;
