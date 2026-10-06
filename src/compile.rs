//! The Compile functor — translate a linted-and-covered spec file's
//! structured layers into machine artifacts.
//!
//! Purpose: implement `specs/compile.md` as one functor with three target
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

fn constraints_doc(spec: &Spec) -> ConstraintsDoc {
    ConstraintsDoc {
        source: spec.intent.id.clone(),
        constraints: spec
            .constraints
            .iter()
            .map(|r| ConstraintToml {
                id: r.id.clone(),
                kind: r.cells.get("kind").cloned().unwrap_or_default(),
                expr: r.cells.get("expr").cloned().unwrap_or_default(),
                traces_to: r.cells.get("traces_to").cloned().unwrap_or_default(),
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

    fn parse(tag: &str) -> Option<Self> {
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
                    text: fragment.to_string(),
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

/// Every fragment-bearing cell in the spec is validated labeled before
/// any artifact is emitted (compile_is_total): single non-empty marker,
/// hygiene, and the three rejection rows — law-kind Property predicates,
/// non-invariant Constraint exprs, and Transition guards (no binding over
/// the program-counter model — deferred of record).
pub fn validate_fragments(spec: &Spec) -> Result<(), CompileError> {
    for p in &spec.properties {
        let predicate = p.cells.get("predicate").cloned().unwrap_or_default();
        let kind = p.cells.get("kind").cloned().unwrap_or_default();
        // unknown_tag_rejected: a tag-shaped marker outside the closed
        // set in fragment position fails labeled, naming the tag and the
        // closed set — never silence. Law-kind predicates are exempt:
        // their `**label:**` occurrences are the law-case grammar, not
        // fragment markers.
        if kind != "law"
            && let Some(tag) = unknown_tag_markers(&predicate).first()
        {
            return Err(fragment_error(
                &p.id,
                format!(
                    "unknown fragment language tag `{tag}` — the closed tag set is {{rust, py, ts}} (fragment_language_closed)"
                ),
            ));
        }
        match fragment_of_strict(&predicate) {
            Err(reason) => return Err(fragment_error(&p.id, reason)),
            Ok(Some(fragment)) => {
                if fragment.lang != FragmentLang::Rust {
                    return Err(emitter_gate_error(&p.id, fragment.lang));
                }
                if kind == "law" {
                    return Err(fragment_error(
                        &p.id,
                        "law-kind properties cannot carry **rust:** fragments — each required case needs its own body, and one fragment cannot honestly serve several named cases",
                    ));
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
                    fragment,
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
         macro_rules! todo_predicate {\n    ($reason:expr) => { todo!(\"{}\", $reason) };\n}\n\n",
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
            out.push_str(&format!(
                "    pub fn {g}() -> impl Strategy<Value = String> {{\n        Just({g:?}.into())\n    }}\n\n"
            ));
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
        Some("law") => required_law_cases(&predicate)
            .into_iter()
            .map(|case| PropBlock {
                fn_name: format!("{base}_{}", sanitize_ident(&case)),
                id: id.clone(),
                case: Some(case),
                // Law rows never carry fragments (fragment_law_rejected —
                // compile_spec fails labeled before emission runs).
                fragment: None,
                generator: generator.clone(),
                predicate: predicate.clone(),
                gens: gens.clone(),
            })
            .collect(),
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
    let cases = crate::spec::law_case_labels(predicate);
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

/// Compile the Model section to one TLA+ module, per `specs/compile.md`'s
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

/// TLA+ module identifier from the file stem: letters and digits only,
/// starting with a letter (TLA+ forbids underscores in module names).
fn module_name(spec: &Spec) -> String {
    let stem = spec
        .path
        .as_ref()
        .and_then(|p| p.file_stem().and_then(|s| s.to_str()))
        .map(str::to_string)
        .unwrap_or_else(|| spec.intent.id.replace('.', "_"));
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
/// else the intent id with `.` → `-`.
pub fn artifact_stem(spec: &Spec) -> String {
    spec.path
        .as_ref()
        .and_then(|p| p.file_stem().and_then(|s| s.to_str()))
        .map(|s| s.to_string())
        .unwrap_or_else(|| spec.intent.id.replace('.', "-"))
}

/// Write a compiled spec's three artifacts (`<stem>.toml`,
/// `<stem>_props.rs`, `<stem>.tla`) into `out_dir` (created on demand).
/// Returns the written paths; a write failure is a labeled error, never
/// a silent partial on disk.
pub fn write_artifacts(
    spec: &Spec,
    compiled: &Compiled,
    out_dir: &str,
) -> Result<Vec<String>, String> {
    let stem = artifact_stem(spec);
    std::fs::create_dir_all(out_dir)
        .map_err(|e| format!("could not create out-dir {out_dir}: {e}"))?;
    let toml_path = std::path::Path::new(out_dir).join(format!("{stem}.toml"));
    let props_path = std::path::Path::new(out_dir).join(format!("{stem}_props.rs"));
    let tla_path = std::path::Path::new(out_dir).join(format!("{stem}.tla"));
    std::fs::write(&toml_path, &compiled.toml)
        .map_err(|e| format!("could not write {}: {e}", toml_path.display()))?;
    std::fs::write(&props_path, &compiled.props)
        .map_err(|e| format!("could not write {}: {e}", props_path.display()))?;
    std::fs::write(&tla_path, &compiled.tla)
        .map_err(|e| format!("could not write {}: {e}", tla_path.display()))?;
    Ok(vec![
        toml_path.display().to_string(),
        props_path.display().to_string(),
        tla_path.display().to_string(),
    ])
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
mod tests {
    use super::*;
    use crate::spec::parse_str;

    #[test]
    fn guard_rust_marker_still_rejected_labeled() {
        // Scenario: Executable guard fragments stay rejected (delta
        // spec, fragment_guard_rejected_stands) — a **rust:** guard
        // fails labeled at fragment_extraction, never silently ignored.
        let text = FRAGMENT_SAMPLE
            .replace(
                "| t1 | s1 | s2 | [[demo.frags.a]] |",
                "| t1 | s1 | s2 | `**rust:** v0.len() >= 1` |",
            )
            .replace("`word()`", "`word(), num()`");
        let spec = parse_str(&text).expect("parses");
        let e = compile_spec(&spec).expect_err("rust guard fragment must fail");
        assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
        assert!(e.message.contains("guard"), "{}", e.message);
    }

    #[test]
    fn guard_citations_extracted_from_invariant_rows_prose_excluded() {
        // Slice 1 (specodelic-txo): an invariant-kind Constraint whose
        // expr cell is a citation expression lands in the IR's
        // guard_citations; prose expr cells stay out (inert as before).
        let text = SAMPLE
            .replace(
                "| a | invariant | `x holds` |",
                "| a | invariant | `[[p1]] ∧ [[p2]]` |",
            )
            .replace(
                "| b | effect | `y fires` |",
                "| b | invariant | `x holds` |",
            );
        let spec = parse_str(&text).expect("parses");
        let ir = extract_model_ir(&spec);
        assert_eq!(
            ir.guard_citations.get("a").map(String::as_str),
            Some("[[p1]] ∧ [[p2]]"),
            "citation expr is carried verbatim"
        );
        assert!(
            !ir.guard_citations.contains_key("b"),
            "prose expr cells are not citations"
        );
        assert!(ir.invariants.is_empty(), "no fragments in this spec");
    }

    #[test]
    fn sanitize_ident_leading_digit_and_unicode_are_valid_rust_idents() {
        // specodelic-suz (pinning): a digit-leading id gets an underscore
        // prefix; unicode chars are replaced — both results are valid
        // Rust identifiers.
        assert_eq!(sanitize_ident("9lives"), "_9lives");
        assert_eq!(sanitize_ident("café"), "caf_");
        for s in [sanitize_ident("9lives"), sanitize_ident("café")] {
            let first = s.chars().next().unwrap();
            assert!(first == '_' || first.is_ascii_alphabetic());
            assert!(s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
        }
    }

    #[test]
    fn sanitize_ident_never_yields_a_rust_keyword_or_empty() {
        // specodelic-suz: a reserved word passes the char map untouched
        // and must still not become a fn name — `fn fn(…)` does not
        // compile; the empty id must not become `fn ()` either.
        for kw in [
            "fn", "match", "async", "await", "impl", "type", "self", "Self",
        ] {
            assert_eq!(sanitize_ident(kw), format!("_{kw}"), "keyword {kw}");
        }
        assert_eq!(sanitize_ident(""), "_");
    }

    #[test]
    fn sanitize_ident_caps_extremely_long_ids_and_stays_unique_deterministic() {
        // specodelic-suz: a hostile 200-char id must not emit a 200-char
        // fn name — capped, and two ids sharing a prefix beyond the cap
        // must not collide (the suffix hash disambiguates them).
        let prefix = "a".repeat(200);
        let a = sanitize_ident(&prefix);
        let b = sanitize_ident(&format!("{prefix}x"));
        assert!(a.len() <= 64, "capped: {}", a.len());
        assert!(b.len() <= 64, "capped: {}", b.len());
        assert_ne!(a, b, "same-prefix long ids must not collide");
        assert_eq!(a, sanitize_ident(&prefix), "deterministic");
    }

    const SAMPLE: &str = "---\nid: demo.thing\nkind: intent\nstatement: \"THE system SHALL work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x holds` | [[demo.thing]] |\n| b | effect | `y fires` | [[demo.thing]] |\n\n## Model\n\n### States\n\n- s1\n- s2 (emits: `[[demo.thing.b]]`)\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s2 | [[demo.thing.a]] |\n| t2 | s2 | s1 | `done` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | [[demo.thing.a]] | `arbitrary_row()` | `check(x) == ok` |\n| p2 | law | [[demo.thing.a]] | `arbitrary_row(), other()` | **identity:** `f(a) == a` **associativity:** `f(f(a)) == f(a)` |\n";

    fn sample() -> Spec {
        parse_str(SAMPLE).expect("sample parses")
    }

    // --- 1.2 / 1.3 Constraints → TOML ---

    #[test]
    fn toml_roundtrips_field_for_field() {
        let spec = sample();
        let toml = constraints_to_toml(&spec);
        let doc: ConstraintsDoc = toml::from_str(&toml).expect("re-parses");
        assert_eq!(doc.source, "demo.thing");
        assert_eq!(doc.constraints.len(), 2);
        let a = &doc.constraints[0];
        assert_eq!(a.id, "a");
        assert_eq!(a.kind, "invariant");
        assert_eq!(a.expr, "`x holds`");
        assert_eq!(a.traces_to, "[[demo.thing]]");
    }

    #[test]
    fn toml_ids_preserved_verbatim() {
        let spec = sample();
        let toml = constraints_to_toml(&spec);
        assert!(toml.contains("id = \"a\""));
        assert!(toml.contains("id = \"b\""));
    }

    // --- 2.1 / 2.2 Model → ModelIR ---

    #[test]
    fn ir_transition_count_matches() {
        let spec = sample();
        let ir = extract_model_ir(&spec);
        assert_eq!(ir.transitions.len(), 2);
        let t1 = &ir.transitions[0];
        assert_eq!(t1.from, "s1");
        assert_eq!(t1.to, "s2");
        assert_eq!(t1.guard.as_deref(), Some("[[demo.thing.a]]"));
    }

    #[test]
    fn ir_emits_covers_exactly_emitting_states() {
        let spec = sample();
        let ir = extract_model_ir(&spec);
        // exactly one entry — no entry for s1, never a default or null
        assert_eq!(ir.emits.len(), 1);
        assert_eq!(ir.emits.get("s2").map(String::as_str), Some("b"));
        assert!(!ir.emits.contains_key("s1"));
    }

    // --- 3.2 / 3.3 Properties → proptest! ---

    #[test]
    fn unit_property_yields_at_least_one_block() {
        let spec = sample();
        let src = properties_to_proptest(&spec);
        assert!(src.contains("proptest!"));
        assert!(src.contains("fn p1("));
        assert!(src.contains("todo_predicate!"));
    }

    #[test]
    fn law_expands_to_required_cases() {
        let spec = sample();
        let src = properties_to_proptest(&spec);
        // p1 (unit) → 1 block; p2 (law, identity+associativity) → 2 blocks
        assert_eq!(src.matches("#[test]").count(), 3);
        assert!(src.contains("fn p2_identity("));
        assert!(src.contains("fn p2_associativity("));
    }

    #[test]
    fn law_without_labels_falls_back_to_floor() {
        let spec = parse_str(
            "---\nid: demo.law\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[demo.law]] |\n\n## Model\n\n### States\n\n- s\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s | s | [[demo.law.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p | law | [[demo.law.a]] | `g()` | `f holds` |\n",
        )
        .unwrap();
        let src = properties_to_proptest(&spec);
        // floor: identity + associativity
        assert_eq!(src.matches("#[test]").count(), 2);
        assert!(src.contains("fn p_identity("));
        assert!(src.contains("fn p_associativity("));
    }

    #[test]
    fn generators_get_spec_gen_helpers() {
        let spec = sample();
        let src = properties_to_proptest(&spec);
        assert!(src.contains("pub fn arbitrary_row()"));
        assert!(src.contains("pub fn other()"));
        assert!(src.contains("v0 in spec_gen::arbitrary_row()"));
        assert!(src.contains("v1 in spec_gen::other()"));
    }

    // --- specodelic-8aq: parameterless property blocks ---

    const PARAMETERLESS_SAMPLE: &str = "---\nid: demo.par\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[demo.par]] |\n\n## Model\n\n### States\n\n- s\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s | s | [[demo.par.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[demo.par.a]] | `(a, b)` where one was removed | `x holds` |\n| l | law | [[demo.par.a]] |  | `f holds` |\n";

    #[test]
    fn parameterless_generator_rows_emit_plain_test_fns() {
        // specodelic-8aq: a generator cell yielding no strategy params (no
        // `name(` occurrences — the corpus shape, e.g. linter-schema_shape's
        // `(a, b)` where-prose) must NOT emit `fn p()` inside proptest! —
        // proptest! requires (pat in strategy) on every fn, so a bare fn is
        // a parse error that left the scratch crate properties_uncompilable.
        // The block goes outside the macro as a plain #[test] fn, honest via
        // todo_predicate!, with the metadata comments verify fingerprints.
        let spec = parse_str(PARAMETERLESS_SAMPLE).expect("parses");
        let src = properties_to_proptest(&spec);
        // No proptest! macro at all: every block in this fixture is
        // parameterless.
        assert!(
            !src.contains("proptest!"),
            "bare fn inside proptest! is a parse error: {src}"
        );
        // Plain #[test] fns at top level (unindented `fn`).
        assert!(src.contains("#[test]\nfn p("), "unit block: {src}");
        // Law-case splitting intact: one plain fn per required case.
        assert!(
            src.contains("#[test]\nfn l_identity("),
            "law identity: {src}"
        );
        assert!(
            src.contains("#[test]\nfn l_associativity("),
            "law associativity: {src}"
        );
        // Honest failure preserved — never a silent pass.
        assert_eq!(
            src.matches("todo_predicate!(").count(),
            3,
            "placeholder bodies: {src}"
        );
        // Metadata comments preserved for verify's staleness fingerprint.
        assert!(src.contains("// id: p"));
        assert!(src.contains("// id: l"));
        // verify's block discovery still finds every emitted block.
        let blocks = crate::verify::blocks_from_source(&src);
        assert_eq!(blocks.len(), 3, "all blocks discoverable: {src}");
    }

    #[test]
    fn parameterless_fragment_row_asserts_outside_proptest() {
        // A parameterless row that opts in with **rust:** keeps its verbatim
        // fragment body (fragment rows unchanged) — just outside proptest!.
        let text = PARAMETERLESS_SAMPLE.replace(
            "| p | unit | [[demo.par.a]] | `(a, b)` where one was removed | `x holds` |",
            "| p | unit | [[demo.par.a]] | `(a, b)` where one was removed | `**rust:** state != \"blackhole\"` |",
        );
        let spec = parse_str(&text).expect("parses");
        let src = properties_to_proptest(&spec);
        assert!(src.contains("#[test]\nfn p("), "plain fn: {src}");
        assert!(!src.contains("proptest!"), "no macro fn: {src}");
        assert!(
            src.contains("assert!(\n            state != \"blackhole\","),
            "verbatim fragment body: {src}"
        );
    }

    // --- model_to_tla ---

    #[test]
    fn tla_disjunct_count_matches_transitions_plus_stuttering() {
        let spec = sample();
        let src = model_to_tla(&spec);
        // one disjunct per transition + the closing stutter disjunct
        // (specodelic-len: terminal states must not be engine-side
        // deadlocks for opt-in TLC)
        assert_eq!(src.matches("\\/").count(), 3);
        assert!(src.contains("UNCHANGED vpc"));
        assert!(src.contains("vpc = \"s1\" /\\ vpc' = \"s2\""));
    }

    #[test]
    fn tla_module_header_is_single_line_tla_plus_form() {
        // specodelic-len: the header must be the TLA+ module-header form
        // (dashes + MODULE name + dashes on ONE line) so opt-in TLC can
        // parse the module — the previous three-line form is not a valid
        // TLA+ module header.
        let spec = sample();
        let src = model_to_tla(&spec);
        let header = src.lines().next().unwrap();
        assert!(header.starts_with("---- MODULE demothing ----"));
        assert!(header.trim_end().ends_with("----"));
        assert!(!src.contains("\nMODULE"));
    }

    #[test]
    fn tla_output_covers_emitting_states_only() {
        let spec = sample();
        let src = model_to_tla(&spec);
        // domain is exactly the emitting states — s1 has no entry; the
        // value is the effect-Constraint's `expr`, not its id
        assert!(src.contains("\"s2\" :> \"`y fires`\""));
        assert!(!src.contains("\"s1\" :>"));
    }

    #[test]
    fn tla_no_emits_means_empty_output() {
        let spec = parse_str(
            "---\nid: demo.quiet\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[demo.quiet]] |\n\n## Model\n\n### States\n\n- s\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s | s | [[demo.quiet.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[demo.quiet.a]] | `g()` | `x` |\n",
        )
        .unwrap();
        let src = model_to_tla(&spec);
        assert!(src.contains("Output == << >>"));
    }

    #[test]
    fn tla_module_name_sanitized_and_guard_verbatim() {
        let mut spec = sample();
        spec.path = Some(std::path::PathBuf::from("specs/demo-thing.md"));
        let src = model_to_tla(&spec);
        assert!(src.contains("MODULE demothing"));
        // the spec's own guard text travels verbatim in the disjunct comment
        assert!(src.contains("(guard: [[demo.thing.a]])"));
    }

    #[test]
    fn tla_artifact_is_byte_stable() {
        let spec = sample();
        assert_eq!(model_to_tla(&spec), model_to_tla(&spec));
    }

    // --- 4.2 contract invariants ---

    #[test]
    fn compile_refuses_stateless_spec_with_labeled_failure() {
        // lib-level totality: lint's model_present gate covers CLI runs,
        // but compile_spec itself must fail labeled, never emit a
        // malformed module.
        let spec = parse_str(
            "---\nid: demo.bare\nkind: intent\nstatement: \"THE system SHALL work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[demo.bare]] |\n",
        )
        .unwrap();
        let err = compile_spec(&spec).expect_err("stateless spec must fail");
        assert_eq!(err.stage, "model_to_tla");
    }

    #[test]
    fn colliding_fn_names_are_disambiguated() {
        // `p.1` and `p_1` sanitize to the same fn name — the second gets a
        // suffix instead of emitting duplicate fns.
        let spec = parse_str(
            "---\nid: demo.coll\nkind: intent\nstatement: \"THE system SHALL work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[demo.coll]] |\n\n## Model\n\n### States\n\n- s\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s | s | [[demo.coll.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p.1 | unit | [[demo.coll.a]] | `g()` | `x` |\n| p_1 | unit | [[demo.coll.a]] | `g()` | `x` |\n",
        )
        .unwrap();
        let src = properties_to_proptest(&spec);
        assert!(src.contains("fn p_1("));
        assert!(src.contains("fn p_1_2("));
        assert_eq!(src.matches("fn p_1(").count(), 1);
    }

    #[test]
    fn compile_preserves_every_source_id() {
        let spec = sample();
        let compiled = compile_spec(&spec).expect("compiles");
        let text = format!("{}{}{:?}", compiled.toml, compiled.props, compiled.model_ir);
        for id in spec.defined_ids() {
            assert!(
                text.contains(&id) || ir_contains_id(&compiled.model_ir, &id),
                "id `{id}` dropped"
            );
        }
    }

    // --- specodelic-rjb: executable predicate fragments (Revision 15) ---

    const FRAGMENT_SAMPLE: &str = "---\nid: demo.frags\nkind: intent\nstatement: \"THE system SHALL work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `**rust:** state != \"blackhole\"` | [[demo.frags]] |\n| b | effect | `y fires` | [[demo.frags]] |\n\n## Model\n\n### States\n\n- s1\n- s2\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s2 | [[demo.frags.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p1 | unit | [[demo.frags.a]] | `word()` | `**rust:** v0.len() >= 1` |\n| p2 | unit | [[demo.frags.a]] | `word(), num()` | `**rust:** v0.len() >= 1 && v1.len() >= 1` |\n";

    fn fragment_sample() -> Spec {
        parse_str(FRAGMENT_SAMPLE).expect("fragment sample parses")
    }

    #[test]
    fn fragment_body_emitted_verbatim() {
        let src = properties_to_proptest(&fragment_sample());
        assert!(
            src.contains("assert!(\n            v0.len() >= 1,\n            \"predicate `p1` violated (executable fragment)\",\n        );"),
            "fragment must be the assertion body, verbatim: {src}"
        );
        assert!(
            src.contains("assert!(\n            v0.len() >= 1 && v1.len() >= 1,"),
            "multi-generator fragment binds v0 and v1: {src}"
        );
        // The fragment rows carry no placeholder.
        assert!(!src.contains("todo_predicate!(\"**rust:**"));
    }

    #[test]
    fn fragment_values_bind_generators_as_strings() {
        let src = properties_to_proptest(&fragment_sample());
        assert!(
            src.contains("impl Strategy<Value = String>"),
            "generators yield plain Strings (GenVal retired, Revision 15): {src}"
        );
        assert!(!src.contains("GenVal"), "GenVal must be gone: {src}");
    }

    #[test]
    fn prose_predicate_stays_placeholder() {
        // Pure widening: a cell without the marker compiles exactly as
        // before (specodelic.md Revision 15).
        let src = properties_to_proptest(&sample());
        assert!(src.contains("todo_predicate!"));
        assert!(!src.contains("executable fragment"));
    }

    #[test]
    fn law_fragment_rejected_labeled() {
        // specodelic-sd1: the fixture opts in via the canonical fragment
        // position (span-opening marker) — a mid-cell prose marker is a
        // mention, not an opt-in.
        let text = FRAGMENT_SAMPLE.replace(
            "| p1 | unit | [[demo.frags.a]] | `word()` | `**rust:** v0.len() >= 1` |",
            "| p1 | law | [[demo.frags.a]] | `word()` | `**rust:** v0.len() >= 1` |",
        );
        let spec = parse_str(&text).expect("parses");
        let e = compile_spec(&spec).expect_err("law fragment must fail");
        assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
        assert!(e.message.contains("law"), "{}", e.message);
    }

    #[test]
    fn hygiene_violation_rejected_labeled_names_token() {
        for (token, frag) in [
            ("unsafe", "unsafe { 1 == 1 }"),
            ("std::fs", "std::fs::metadata(\"x\").is_ok()"),
            ("Command", "Command::new(\"sh\") == never"),
        ] {
            let text = FRAGMENT_SAMPLE
                .replace("`**rust:** v0.len() >= 1`", &format!("`**rust:** {frag}`"));
            let spec = parse_str(&text).expect("parses");
            let e = compile_spec(&spec).expect_err("banned token must fail");
            assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
            assert!(
                e.message.contains(token),
                "message must name the token {token}: {}",
                e.message
            );
        }
    }

    #[test]
    fn guard_fragment_rejected_labeled() {
        let text = FRAGMENT_SAMPLE.replace(
            "| t1 | s1 | s2 | [[demo.frags.a]] |",
            "| t1 | s1 | s2 | `**rust:** true` |",
        );
        let spec = parse_str(&text).expect("parses");
        let e = compile_spec(&spec).expect_err("guard fragment must fail");
        assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
        assert!(e.message.contains("t1"), "{}", e.message);
    }

    #[test]
    fn non_invariant_fragment_rejected_labeled() {
        let text = FRAGMENT_SAMPLE.replace(
            "| b | effect | `y fires` | [[demo.frags]] |",
            "| b | effect | `**rust:** true` | [[demo.frags]] |",
        );
        let spec = parse_str(&text).expect("parses");
        let e = compile_spec(&spec).expect_err("effect fragment must fail");
        assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
        assert!(e.message.contains("b"), "{}", e.message);
    }

    #[test]
    fn double_marker_rejected_labeled() {
        // specodelic-sd1: two markers in FRAGMENT POSITION (each opening
        // its own code span) are still the labeled at-most-one failure —
        // a mid-span second occurrence is a mention, not a second marker.
        let text = FRAGMENT_SAMPLE.replace(
            "`**rust:** v0.len() >= 1`",
            "`**rust:** v0.len() >= 1`, `**rust:** v0.len() >= 2`",
        );
        let spec = parse_str(&text).expect("parses");
        let e = compile_spec(&spec).expect_err("two fragment-position markers must fail");
        assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
    }

    #[test]
    fn empty_fragment_rejected_labeled() {
        let text = FRAGMENT_SAMPLE.replace("`**rust:** v0.len() >= 1`", "`**rust:**`");
        let spec = parse_str(&text).expect("parses");
        let e = compile_spec(&spec).expect_err("empty fragment must fail");
        assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
    }

    // --- specodelic-lf3: closed language-tag grammar (Revision 16) ---

    #[test]
    fn unknown_tag_rejected_labeled() {
        // unknown_tag_rejected: `**go:**` in fragment position is a labeled
        // extraction failure naming the tag AND the closed set — never
        // prose, never silence, never a Rust fall-through.
        let text = FRAGMENT_SAMPLE.replace("`**rust:** v0.len() >= 1`", "`**go:** len(v0) >= 1`");
        let spec = parse_str(&text).expect("parses");
        let e = compile_spec(&spec).expect_err("unknown tag must fail");
        assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
        assert!(e.message.contains("go"), "names the tag: {}", e.message);
        assert!(
            e.message.contains("rust") && e.message.contains("py") && e.message.contains("ts"),
            "names the closed set: {}",
            e.message
        );
    }

    #[test]
    fn py_tag_extracts_at_grammar_level() {
        // one_tag_extracts_per_cell: acceptance is grammar-level and
        // emitter-independent — extraction must succeed for `**py:**` even
        // though compile later gates it (missing_emitter_labeled is the
        // compile-stage failure, not the extraction).
        let cell = "`**py:** len(v0) >= 1`";
        let f = fragment_of_strict(cell)
            .expect("py marker in fragment position extracts")
            .expect("one fragment");
        assert_eq!(f.lang.tag(), "py");
        assert_eq!(f.text, "len(v0) >= 1");
    }

    #[test]
    fn py_no_emitter_fails_labeled() {
        // no_emitter_labeled_failure: a **py:** fragment reaching compile
        // fails labeled with stage compile.emission_failure and a
        // remediation naming the py-emitter follow-up — never a silent
        // fall-through to Rust emission, never a vacuous artifact.
        let text = FRAGMENT_SAMPLE.replace("`**rust:** v0.len() >= 1`", "`**py:** len(v0) >= 1`");
        let spec = parse_str(&text).expect("parses");
        let e = compile_spec(&spec).expect_err("py fragment must gate compile");
        assert_eq!(e.stage, "compile.emission_failure", "{}", e.message);
        assert!(
            e.message.contains("add-py-fragment-emission"),
            "remediation names the follow-up: {}",
            e.message
        );
        assert!(
            e.message.contains("rust"),
            "names the one executable tag: {}",
            e.message
        );
    }

    #[test]
    fn ts_no_emitter_fails_labeled() {
        let text = FRAGMENT_SAMPLE.replace(
            "`**rust:** v0.len() >= 1 && v1.len() >= 1`",
            "`**ts:** v0.length >= 1 && v1.length >= 1`",
        );
        let spec = parse_str(&text).expect("parses");
        let e = compile_spec(&spec).expect_err("ts fragment must gate compile");
        assert_eq!(e.stage, "compile.emission_failure", "{}", e.message);
        assert!(
            e.message.contains("add-ts-fragment-emission"),
            "remediation names the follow-up: {}",
            e.message
        );
    }

    #[test]
    fn expr_cell_same_tag_grammar() {
        // expr_tag_same_grammar: an invariant-kind Constraint expr cell
        // accepts the same closed-set grammar — gated at compile by the
        // emitter, not rejected as a non-invariant/unknown-shape row.
        let text = FRAGMENT_SAMPLE.replace(
            "`**rust:** state != \"blackhole\"`",
            "`**py:** state != \"blackhole\"`",
        );
        let spec = parse_str(&text).expect("parses");
        let e = compile_spec(&spec).expect_err("py expr fragment must gate compile");
        assert_eq!(e.stage, "compile.emission_failure", "{}", e.message);
    }

    #[test]
    fn guard_py_marker_still_rejected_labeled() {
        // Guards reject ANY fragment-position marker, per tag — the
        // program-counter model has no data binding, and an unknown tag
        // there is likewise labeled, never silence.
        let text = FRAGMENT_SAMPLE
            .replace(
                "| t1 | s1 | s2 | [[demo.frags.a]] |",
                "| t1 | s1 | s2 | `**py:** v0.len() >= 1` |",
            )
            .replace("`word()`", "`word(), num()`");
        let spec = parse_str(&text).expect("parses");
        let e = compile_spec(&spec).expect_err("py guard marker must fail");
        assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
        assert!(e.message.contains("guard"), "{}", e.message);
    }

    #[test]
    fn guard_unknown_tag_marker_rejected_labeled() {
        let text = FRAGMENT_SAMPLE
            .replace(
                "| t1 | s1 | s2 | [[demo.frags.a]] |",
                "| t1 | s1 | s2 | `**go:** v0.len() >= 1` |",
            )
            .replace("`word()`", "`word(), num()`");
        let spec = parse_str(&text).expect("parses");
        let e = compile_spec(&spec).expect_err("unknown-tag guard marker must fail");
        assert_eq!(e.stage, "fragment_extraction", "{}", e.message);
    }

    #[test]
    fn midspan_py_marker_is_mention() {
        // mention_not_extraction: a mid-span **py:** occurrence compiles
        // byte-identically to the same cell without it (sd1 rule per tag).
        let predicate_with =
            "`a row mentioning **py:** mid-span, plus the placeholder v0.len() >= 1`";
        let predicate_without =
            "`a row mentioning py mid-span, plus the placeholder v0.len() >= 1`";
        let text_with = FRAGMENT_SAMPLE
            .replace("`**rust:** v0.len() >= 1`", predicate_with)
            .replace(
                "`**rust:** v0.len() >= 1 && v1.len() >= 1`",
                "`v0.len() >= 1 && v1.len() >= 1`",
            )
            .replace(
                "`**rust:** state != \"blackhole\"`",
                "`state != \"blackhole\"`",
            );
        let text_without = text_with.replace(predicate_with, predicate_without);
        let spec_with = parse_str(&text_with).expect("parses");
        let spec_without = parse_str(&text_without).expect("parses");
        assert!(
            fragment_of_strict("a row mentioning **py:** mid-span").is_ok_and(|f| f.is_none()),
            "mid-span occurrence never extracts"
        );
        compile_spec(&spec_with).expect("mid-span mention compiles");
        let src_with = properties_to_proptest(&spec_with);
        let src_without = properties_to_proptest(&spec_without);
        assert_eq!(
            src_with.replace("**py:**", "py"),
            src_without,
            "mid-span occurrence never extracts (artifact-identical modulo the mention text)"
        );
    }

    #[test]
    fn rust_back_compat_byte_identical() {
        // rust_back_compat: the pre-change Rust extraction path is pure —
        // the widened scanner must extract exactly the Revision 15
        // fragment for the canonical fixtures, and every Revision 15 test
        // above must pass unchanged (CI proves the rest).
        let cell = "`**rust:** v0.len() >= 1`";
        let f = fragment_of_strict(cell)
            .expect("rust marker extracts")
            .expect("one fragment");
        assert_eq!(f.lang.tag(), "rust");
        assert_eq!(f.text, "v0.len() >= 1");
        let bare = "**rust:** v0.len() >= 1";
        let f = fragment_of_strict(bare)
            .expect("rust marker extracts at cell start")
            .expect("one fragment");
        assert_eq!(f.text, "v0.len() >= 1");
    }

    #[test]
    fn bold_prose_label_is_not_a_marker() {
        // The tag shape is [a-z][a-z0-9_-]* — an uppercase-initial bold
        // label (`**Note:**`) in fragment position is prose, never an
        // unknown-tag failure: the closed-set grammar must not criminalize
        // ordinary bold prose.
        let cell = "`**Note:** keep the placeholder honest`";
        assert!(
            fragment_of_strict(cell).is_ok_and(|f| f.is_none()),
            "non-tag-shaped bold label is prose"
        );
    }

    // --- specodelic-sd1: marker mentions are not opt-ins ---

    /// The two corpus mention shapes: a **rust:** occurrence MID-SPAN in a
    /// defining row's expr cell (compile.md's predicate_fragment_opt_in et
    /// al.) and one in PROSE between code spans (verify.md's
    /// fragments_reach_verified predicate).
    const MENTION_SAMPLE: &str = "---\nid: demo.mention\nkind: intent\nstatement: \"THE system SHALL work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `every row compiles verbatim as a **rust:** fragment when the row opts in, else the placeholder` | [[demo.mention]] |\n\n## Model\n\n### States\n\n- s1\n- s2\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s2 | [[demo.mention.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[demo.mention.a]] | `word()` | `check(file) == verified` — a passing **rust:** predicate and a clean executable-invariant run verify a real file |\n";

    #[test]
    fn marker_mentions_never_extract() {
        // specodelic-sd1: an occurrence of **rust:** that is not in
        // fragment position (cell start, or opening the fragment's code
        // span) is a mention of the mechanism, never an opt-in — the
        // extractor must not read the prose after it as a fragment. The
        // corpus rows that DEFINE the marker carry such mentions in their
        // own cells; before this fix the defining-row prose was extracted
        // as Rust and the scratch runs hard-failed.
        let spec = parse_str(MENTION_SAMPLE).expect("parses");
        // Mid-span mention → no executable invariant in the IR.
        let ir = extract_model_ir(&spec);
        assert!(
            ir.invariants.is_empty(),
            "mid-span mention must not extract an invariant: {ir:?}"
        );
        // Prose mention between spans → the honest placeholder body.
        let src = properties_to_proptest(&spec);
        assert!(
            !src.contains("assert!("),
            "prose mention must not extract: {src}"
        );
        assert!(src.contains("todo_predicate!("));
    }

    #[test]
    fn span_opening_optin_extracts_mention_alongside_ignored() {
        // The Revision 15 opt-in form still extracts when the cell ALSO
        // mentions the marker in prose outside the fragment span — the
        // mention is not a second marker, the cell compiles cleanly.
        let text = FRAGMENT_SAMPLE.replace(
            "`**rust:** v0.len() >= 1`",
            "`**rust:** v0.len() >= 1` — mentioned again as a **rust:** fragment in prose",
        );
        let spec = parse_str(&text).expect("parses");
        let src = properties_to_proptest(&spec);
        assert!(
            src.contains("assert!(\n            v0.len() >= 1,"),
            "span-opening opt-in must still extract: {src}"
        );
    }

    #[test]
    fn executable_invariants_extract_into_the_ir() {
        let ir = extract_model_ir(&fragment_sample());
        assert_eq!(
            ir.invariants,
            vec![InvariantIr {
                id: "a".into(),
                fragment: "state != \"blackhole\"".into(),
            }],
            "invariant fragments ride the ModelIR"
        );
        assert!(extract_model_ir(&sample()).invariants.is_empty());
    }

    #[test]
    fn module_carries_the_fragment_manifest() {
        // rerun_on_model_change: the manifest comment makes a fragment
        // edit change the artifact hash.
        let tla = model_to_tla(&fragment_sample());
        assert!(
            tla.contains("\\* INVARIANT a **rust:** state != \"blackhole\""),
            "manifest comment must carry id + fragment verbatim: {tla}"
        );
        assert!(!model_to_tla(&sample()).contains("INVARIANT"));
    }

    #[test]
    fn no_semantic_drift_roundtrip_is_byte_identical() {
        let spec = sample();
        let first = compile_spec(&spec).expect("compiles");
        let second = compile_spec(&spec).expect("compiles");
        assert_eq!(first.toml, second.toml);
        assert_eq!(first.props, second.props);
        // re-parse + re-emit == original
        let doc: ConstraintsDoc = toml::from_str(&first.toml).unwrap();
        assert_eq!(toml::to_string(&doc).unwrap(), first.toml);
    }
}
