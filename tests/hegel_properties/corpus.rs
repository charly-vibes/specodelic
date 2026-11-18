//! Typed corpus model + Markdown renderer for the Hegel property suite
//! (specodelic-jnnf, per the bb98 scratchbook recommendation).
//!
//! The writer/rename properties cannot be driven by hand-built `Spec`
//! values — spans are produced by the parser — so every generated case is
//! a typed model rendered to text, parsed back by the real parser, and
//! checked against the model (`check_corpus`, the round-trip self-check)
//! before any property consumes it. Models are lint-clean by
//! construction: closed kind sets, unique ids, guards non-null, states
//! fully used, every constraint derived by exactly one property, and
//! every link target defined corpus-wide.

use specodelic::graph;
use specodelic::lint;
use specodelic::spec::{Link, Spec, parse_str};

/// The closed Constraint `kind` vocabulary (`linter.constraint_kind_closed`).
pub const CONSTRAINT_KINDS: [&str; 4] = ["invariant", "advisory", "effect", "extension_point"];

/// State vocabulary. No failure-terminal names, so the corpus never
/// enters `linter.terminal_states_emit`'s subject matter.
pub const STATE_NAMES: [&str; 4] = ["draft", "active", "review", "done"];

/// One Constraints-table row (rendered `| id | kind | expr | traces_to |`).
#[derive(Clone, Debug)]
pub struct CRow {
    pub local: String,
    /// Index into [`CONSTRAINT_KINDS`].
    pub kind: usize,
    pub expr: String,
    /// `Some(qualified_id)` — a cross-file trace target (resolves in the
    /// full corpus); `None` traces the file's own intent.
    pub cross_target: Option<String>,
}

/// One Transitions-table row.
#[derive(Clone, Debug)]
pub struct TRow {
    pub local: String,
    /// Index into the file's state list.
    pub from: usize,
    pub to: usize,
    /// `Some(i)` — the guard cites `[[<intent>.<constraints[i].local>]]`;
    /// `None` — the guard is prose (still non-null: `linter.guard_required`).
    pub guard: Option<usize>,
}

/// One Properties-table row (rendered `unit` kind).
#[derive(Clone, Debug)]
pub struct PRow {
    pub local: String,
    /// Index of the constraint this property derives from — coverage by
    /// construction (`linter.coverage` / `linter.no_orphan_property`).
    pub derives: usize,
    pub predicate: String,
}

/// One spec file's typed model.
#[derive(Clone, Debug)]
pub struct ModelFile {
    pub intent: String,
    pub system: String,
    pub verb: String,
    pub constraints: Vec<CRow>,
    /// State indices into [`STATE_NAMES`] (distinct, length ≥ 2).
    pub states: Vec<usize>,
    pub transitions: Vec<TRow>,
    pub properties: Vec<PRow>,
    /// Prose paragraphs; may mention ids as bare words (no link syntax) —
    /// the untouched-bytes oracle's canary.
    pub prose: Vec<String>,
    pub crlf: bool,
    /// Extra spaces per cell side (0 = tight `| c1 |`).
    pub pad: u32,
}

impl ModelFile {
    pub fn stem(&self) -> String {
        self.intent.replace('.', "-")
    }
    pub fn qualified(&self, local: &str) -> String {
        format!("{}.{}", self.intent, local)
    }
    pub fn constraint_qualified(&self, i: usize) -> String {
        self.qualified(&self.constraints[i].local)
    }
    fn traces_cell_body(&self, c: &CRow) -> String {
        match &c.cross_target {
            Some(t) => format!("[[{t}]]"),
            None => format!("[[{}]]", self.intent),
        }
    }
}

pub fn render(m: &ModelFile) -> String {
    let p = " ".repeat(1 + m.pad as usize);
    let cell = |text: &str| format!("{p}{text}{p}");

    let mut l: Vec<String> = Vec::new();
    l.push("---".into());
    l.push(format!("id: {}", m.intent));
    l.push("kind: intent".into());
    l.push(format!(
        "statement: \"THE {} SHALL {} while the ledger is open.\"",
        m.system, m.verb
    ));
    l.push("---".into());
    l.push(String::new());
    l.push(format!("# {} Specification", m.stem()));
    l.push(String::new());
    l.push("## Purpose".into());
    for para in &m.prose {
        l.push(String::new());
        l.push(para.clone());
    }
    l.push(String::new());
    l.push("## Constraints".into());
    l.push(String::new());
    l.push("| id | kind | expr | traces_to |".into());
    l.push("| --- | --- | --- | --- |".into());
    for c in &m.constraints {
        l.push(format!(
            "|{}|{}|{}|{}|",
            cell(&c.local),
            cell(CONSTRAINT_KINDS[c.kind]),
            cell(&format!("`{}`", c.expr)),
            cell(&m.traces_cell_body(c)),
        ));
    }
    l.push(String::new());
    l.push("## Model".into());
    l.push(String::new());
    l.push("### States".into());
    for s in &m.states {
        l.push(format!("- `{}`", STATE_NAMES[*s]));
    }
    l.push(String::new());
    l.push("### Transitions".into());
    l.push(String::new());
    l.push("| id | from | to | guard |".into());
    l.push("| --- | --- | --- | --- |".into());
    for t in m.transitions.iter() {
        l.push(format!(
            "|{}|{}|{}|{}|",
            cell(&t.local),
            cell(STATE_NAMES[m.states[t.from]]),
            cell(STATE_NAMES[m.states[t.to]]),
            cell(&m.guard_cell_text(t)),
        ));
    }
    l.push(String::new());
    l.push("## Properties".into());
    l.push(String::new());
    l.push("| id | kind | derives_from | generator | predicate |".into());
    l.push("| --- | --- | --- | --- | --- |".into());
    for (i, pr) in m.properties.iter().enumerate() {
        l.push(format!(
            "|{}|{}|{}|{}|{}|",
            cell(&pr.local),
            cell("unit"),
            cell(&m.derives_cell_of(pr)),
            cell(&format!("`gen_{}()`", i)),
            cell(&format!("`{}`", pr.predicate)),
        ));
    }
    let mut text = l.join("\n");
    text.push('\n');
    if m.crlf {
        text = text.replace('\n', "\r\n");
    }
    text
}

// ---------------------------------------------------------------------------
// Seed → model builders: deterministic, total, small.
// ---------------------------------------------------------------------------

pub const SYSTEMS: [&str; 4] = ["pipeline", "ledger", "notifier", "curator"];
pub const VERBS: [&str; 4] = [
    "apply the request",
    "record the verdict",
    "refresh the digest",
    "publish the artifact",
];
pub const EXPRS: [&str; 4] = [
    "holds under load",
    "stays within budget",
    "keeps the order stable",
    "maps every request",
];
pub const PREDICATES: [&str; 4] = [
    "result stays bounded",
    "verdict matches the model",
    "digest is reproducible",
    "artifact is emitted once",
];

/// Build one file's model deterministically from a seed.
///
/// `intent` must be unique in its corpus. `cross` targets constraint 0's
/// trace at a foreign intent id (the caller guarantees it resolves —
/// `traces_to` is Intent-typed by `ref_kind_compatible`).
pub fn build_file(intent: &str, seed: u64, cross: Option<String>) -> ModelFile {
    let pick = |bits: u32, n: usize| ((seed >> bits) % n as u64) as usize;
    let n_c = 1 + pick(0, 3);
    let constraints: Vec<CRow> = (0..n_c)
        .map(|i| CRow {
            local: format!("c{i}"),
            // c0 is always invariant: single_root_reachable reaches the
            // Model's states/transitions only through cited guard edges,
            // and ref_kind_compatible accepts invariant Constraints (or
            // States) as guard targets.
            kind: if i == 0 {
                0
            } else {
                pick(2 + 3 * i as u32, CONSTRAINT_KINDS.len())
            },
            expr: format!(
                "{} (variant {i})",
                EXPRS[pick(4 + 3 * i as u32, EXPRS.len())]
            ),
            cross_target: if i == 0 { cross.clone() } else { None },
        })
        .collect();
    let invariants: Vec<usize> = constraints
        .iter()
        .enumerate()
        .filter(|(_, c)| c.kind == 0)
        .map(|(i, _)| i)
        .collect();
    let ns = 2 + pick(1, 2);
    let start = pick(6, STATE_NAMES.len());
    let states: Vec<usize> = (0..ns).map(|k| (start + k) % STATE_NAMES.len()).collect();
    // Every transition cites an invariant Constraint: the guard edge is
    // the only path from the Model island to the intent row, and prose
    // guards would orphan the island (single_root_reachable).
    let guard_at =
        |i: usize| -> Option<usize> { Some(invariants[pick(10 + 2 * i as u32, invariants.len())]) };
    let mut transitions: Vec<TRow> = (0..ns - 1)
        .map(|i| TRow {
            local: format!("t{i}"),
            from: i,
            to: i + 1,
            guard: guard_at(i),
        })
        .collect();
    transitions.push(TRow {
        local: format!("t{}", ns - 1),
        from: ns - 1,
        to: 0,
        guard: guard_at(ns - 1),
    });
    let properties: Vec<PRow> = (0..n_c)
        .map(|i| PRow {
            local: format!("p{i}"),
            derives: i,
            predicate: format!(
                "{} (checks c{i})",
                PREDICATES[pick(14 + 2 * i as u32, PREDICATES.len())]
            ),
        })
        .collect();
    let system = SYSTEMS[pick(3, SYSTEMS.len())];
    let verb = VERBS[pick(5, VERBS.len())];
    let prose = vec![
        format!(
            "The {system} keeps {intent} wording intact: plain prose may mention \
             {intent} and c0 as ordinary words without link syntax."
        ),
        format!(
            "This paragraph exists so an edit that rewrote prose would be \
             visible: {intent} revision notes stay byte-identical under renames."
        ),
    ];
    ModelFile {
        intent: intent.to_string(),
        system: system.to_string(),
        verb: verb.to_string(),
        constraints,
        states,
        transitions,
        properties,
        prose,
        crlf: pick(2, 4) == 0,
        pad: pick(7, 3) as u32,
    }
}

/// A rendered corpus: models plus their rendered (path, text) trees.
pub struct Corpus {
    pub models: Vec<ModelFile>,
    /// `(rel path, text)` — the path is the filename only (`demo-alpha.md`).
    pub files: Vec<(String, String)>,
}

/// Build a 1–2 file corpus. File 1 (when present) traces constraint 0 at
/// file 0's INTENT — `ref_kind_compatible` requires `traces_to` to
/// resolve to an Intent kind, never a Constraint row.
pub fn build_corpus(seed: u64) -> Corpus {
    let n_files = 1 + (seed % 2);
    let mut models = Vec::new();
    let mut files = Vec::new();
    let intents = ["demo.alpha", "demo.beta"];
    for (i, intent) in intents.iter().enumerate().take(n_files as usize) {
        let cross = if i > 0 {
            Some("demo.alpha".to_string())
        } else {
            None
        };
        let m = build_file(intent, seed >> (8 * (i as u32 + 1)), cross);
        let text = render(&m);
        files.push((format!("{}.md", m.stem()), text));
        models.push(m);
    }
    Corpus { models, files }
}

// ---------------------------------------------------------------------------
// Round-trip self-check: parse + model equality + corpus-level gates.
// ---------------------------------------------------------------------------

/// Verify the rendered corpus: every file parses, every parsed element
/// equals the model, lint reports no issues, and no structured reference
/// dangles. `Err` names the first disagreement.
pub fn check_corpus(corpus: &Corpus) -> Result<(), String> {
    let mut specs: Vec<Spec> = Vec::new();
    for ((path, text), m) in corpus.files.iter().zip(&corpus.models) {
        let spec = parse_str(text).map_err(|e| format!("{path} does not parse: {e}"))?;
        check_file(path, text, &spec, m)?;
        specs.push(spec);
    }
    let report = lint::lint_corpus(&specs);
    if !report.issues.is_empty() {
        let first = &report.issues[0];
        return Err(format!("lint issue {}: {}", first.rule_id, first.message));
    }
    let g = graph::build(&specs);
    if !g.dangling.is_empty() {
        return Err(format!("dangling reference: {}", g.dangling[0]));
    }
    Ok(())
}

fn check_file(path: &str, text: &str, spec: &Spec, m: &ModelFile) -> Result<(), String> {
    if spec.intent.id != m.intent {
        return Err(format!(
            "{path}: intent id {} != {}",
            spec.intent.id, m.intent
        ));
    }
    check_constraints(path, spec, m)?;
    check_states(path, spec, m)?;
    check_transitions(path, spec, m)?;
    check_properties(path, spec, m)?;
    check_links(path, text, spec, m)
}

fn check_constraints(path: &str, spec: &Spec, m: &ModelFile) -> Result<(), String> {
    if spec.constraints.len() != m.constraints.len() {
        return Err(format!("{path}: constraint count drifted"));
    }
    for (row, c) in spec.constraints.iter().zip(&m.constraints) {
        if row.id != c.local {
            return Err(format!("{path}: constraint id {} != {}", row.id, c.local));
        }
        if row.kind.as_deref() != Some(CONSTRAINT_KINDS[c.kind]) {
            return Err(format!("{path}: constraint {} kind drifted", c.local));
        }
        expect_cell(path, &row.cells, "expr", &format!("`{}`", c.expr))?;
        expect_cell(path, &row.cells, "traces_to", &m.traces_cell_body(c))?;
    }
    Ok(())
}

fn check_states(path: &str, spec: &Spec, m: &ModelFile) -> Result<(), String> {
    if spec.states.len() != m.states.len() {
        return Err(format!("{path}: state count drifted"));
    }
    for (row, &si) in spec.states.iter().zip(&m.states) {
        if row.id != STATE_NAMES[si] {
            return Err(format!(
                "{path}: state id {} != {}",
                row.id, STATE_NAMES[si]
            ));
        }
    }
    Ok(())
}

fn check_transitions(path: &str, spec: &Spec, m: &ModelFile) -> Result<(), String> {
    if spec.transitions.len() != m.transitions.len() {
        return Err(format!("{path}: transition count drifted"));
    }
    for (t, tr) in spec.transitions.iter().zip(&m.transitions) {
        if t.id != tr.local {
            return Err(format!("{path}: transition id {} != {}", t.id, tr.local));
        }
        if t.from != STATE_NAMES[m.states[tr.from]] || t.to != STATE_NAMES[m.states[tr.to]] {
            return Err(format!("{path}: transition {} endpoints drifted", tr.local));
        }
        // guard_required: every transition carries a non-null guard, and
        // this renderer never emits an empty cell — the parsed guard is
        // always Some(rendered cell text).
        let want = m.guard_cell_text(tr);
        match &t.guard {
            Some(g) if g == &want => {}
            got => {
                return Err(format!(
                    "{path}: transition {} guard {got:?} != {want:?}",
                    tr.local
                ));
            }
        }
    }
    Ok(())
}

fn check_properties(path: &str, spec: &Spec, m: &ModelFile) -> Result<(), String> {
    if spec.properties.len() != m.properties.len() {
        return Err(format!("{path}: property count drifted"));
    }
    for (i, (row, pr)) in spec.properties.iter().zip(&m.properties).enumerate() {
        if row.id != pr.local {
            return Err(format!("{path}: property id {} != {}", row.id, pr.local));
        }
        if row.kind.as_deref() != Some("unit") {
            return Err(format!("{path}: property {} kind drifted", pr.local));
        }
        expect_cell(path, &row.cells, "derives_from", &m.derives_cell_of(pr))?;
        expect_cell(path, &row.cells, "generator", &format!("`gen_{i}()`"))?;
        expect_cell(
            path,
            &row.cells,
            "predicate",
            &format!("`{}`", pr.predicate),
        )?;
    }
    Ok(())
}

/// The structured-link multiset must equal the model's expected set,
/// exactly.
fn check_links(path: &str, text: &str, spec: &Spec, m: &ModelFile) -> Result<(), String> {
    let mut want_links: Vec<(String, String, String, String)> = Vec::new(); // (field, column, source, target)
    for c in &m.constraints {
        let target = match &c.cross_target {
            Some(t) => t.clone(),
            None => m.intent.clone(),
        };
        want_links.push((
            "constraints".into(),
            "traces_to".into(),
            c.local.clone(),
            target,
        ));
    }
    for tr in &m.transitions {
        if let Some(cidx) = tr.guard {
            want_links.push((
                "transitions".into(),
                "guard".into(),
                tr.local.clone(),
                m.constraint_qualified(cidx),
            ));
        }
    }
    for pr in &m.properties {
        want_links.push((
            "properties".into(),
            "derives_from".into(),
            pr.local.clone(),
            m.constraint_qualified(pr.derives),
        ));
    }
    let mut got_links: Vec<(String, String, String, String)> = spec
        .links
        .iter()
        .map(
            |Link {
                 target,
                 field,
                 column,
                 source,
                 ..
             }| {
                (
                    field.clone(),
                    column.clone(),
                    source.clone(),
                    target.clone(),
                )
            },
        )
        .collect();
    got_links.sort();
    want_links.sort();
    if got_links != want_links {
        return Err(format!("{path}: link set drifted (text={text:?})"));
    }
    Ok(())
}

fn expect_cell(
    path: &str,
    cells: &std::collections::BTreeMap<String, String>,
    col: &str,
    want: &str,
) -> Result<(), String> {
    match cells.get(col) {
        Some(v) if v == want => Ok(()),
        Some(v) => Err(format!("{path}: cell {col} = {v:?} != {want:?}")),
        None => Err(format!("{path}: cell {col} missing")),
    }
}

impl ModelFile {
    /// The exact rendered `guard` cell text for transition `t` (prose
    /// fallback when the guard is not a citation).
    fn guard_cell_text(&self, t: &TRow) -> String {
        match t.guard {
            Some(c) => format!("[[{}]]", self.constraint_qualified(c)),
            None => "approval recorded".to_string(),
        }
    }
    fn derives_cell_of(&self, pr: &PRow) -> String {
        format!("[[{}]]", self.constraint_qualified(pr.derives))
    }
}
