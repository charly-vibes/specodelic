//! The Specodelic format model — parse the four layers of a spec file.
//!
//! One spec file = YAML frontmatter (Intent) + a `## Constraints` table +
//! a `## Model` section (`### States` list, `### Transitions` table) +
//! a `## Properties` table. Prose is never parsed (see
//! `specs/specodelic.md`, `prose_untouched`).
//!
//! This parser is deliberately dumb: frontmatter and fixed-schema markdown
//! tables only, `[[wiki-link]]` references from structured cells. It is the
//! substrate every specodelic command builds on.

use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Why a file failed to parse.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("{0}:{1}: {2}")]
    Cell(String, usize, String),
    #[error("{0}: {1}")]
    File(String, String),
}

/// A byte span in the source text a spec was parsed from — `[start, end)`.
/// The writer edits only recorded spans (acset-writer's
/// `source_spans_recorded`); spans are mechanical bookkeeping and are
/// skipped in serialization, so `spk parse --json` stays byte-stable
/// (add-acset-writer design D3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// A row from one of the three structured tables.
#[derive(Debug, Clone, Serialize)]
pub struct Row {
    /// Row id (the `id` column), e.g. `order.cancel.refund_bounded`.
    pub id: String,
    /// The row's own `kind` cell (Constraints: `invariant`/`advisory`/`effect`;
    /// Properties: `unit`/`law`).
    pub kind: Option<String>,
    /// All cells of the row, keyed by column header.
    pub cells: BTreeMap<String, String>,
    /// Byte span of the row's `id` cell content in the source text
    /// (add-acset-writer task 2.2). Not serialized.
    #[serde(skip_serializing)]
    pub id_span: Option<Span>,
}

/// A transition row from the Model's `### Transitions` table.
#[derive(Debug, Clone, Serialize)]
pub struct Transition {
    pub id: String,
    pub from: String,
    pub to: String,
    /// `None` when the guard cell is empty (`null` guard).
    pub guard: Option<String>,
    /// Byte spans of the id/from/to cells in the source text — a state
    /// rename rewrites the from/to cells through them. Not serialized.
    #[serde(skip_serializing)]
    pub id_span: Option<Span>,
    #[serde(skip_serializing)]
    pub from_span: Option<Span>,
    #[serde(skip_serializing)]
    pub to_span: Option<Span>,
}

/// Parsed frontmatter (the Intent layer).
#[derive(Debug, Clone, Serialize)]
pub struct Intent {
    pub id: String,
    pub kind: String,
    pub statement: String,
    /// Any extra frontmatter fields (e.g. `checked_against_core`).
    pub extra: BTreeMap<String, serde_yaml_ng::Value>,
    /// Byte span of the frontmatter `id:` value in the source text
    /// (add-acset-writer task 2.2). Not serialized.
    #[serde(skip_serializing)]
    pub id_span: Option<Span>,
}

/// A parsed spec file.
#[derive(Debug, Clone, Serialize)]
pub struct Spec {
    /// Path the file was parsed from, if known.
    pub path: Option<PathBuf>,
    pub intent: Intent,
    /// `## Constraints` rows.
    pub constraints: Vec<Row>,
    /// `### States` bullets: `id` plus optional `emits` cell.
    pub states: Vec<Row>,
    /// `### Transitions` rows.
    pub transitions: Vec<Transition>,
    /// `## Properties` rows.
    pub properties: Vec<Row>,
    /// True when the raw text carries an `## ADDED Requirements` heading
    /// (the openspec delta half of a dual-format file).
    pub has_added_requirements: bool,
    /// True when the raw text carries a `## Requirements` heading (the
    /// specodelic capability half of a dual-format file).
    pub has_requirements_section: bool,
    /// Body of `## ADDED Requirements` (lines rstripped, joined) — the
    /// drift check compares it against [`Spec::requirements_body`] (gh#4).
    pub added_requirements_body: String,
    /// True when the raw text carries a `## MODIFIED Requirements`
    /// heading (update-law-named-cases: the repo's first MODIFIED
    /// delta — the mirror rules treat it exactly like the ADDED one).
    pub has_modified_requirements: bool,
    /// Body of `## MODIFIED Requirements` (lines rstripped, joined).
    pub modified_requirements_body: String,
    /// Body of `## Requirements` (lines rstripped, joined).
    pub requirements_body: String,
    /// Body of `### Reference Typing` captured verbatim (add-acset-core
    /// task 2.4) — the `schema_matches_typing_table` lint gate compares
    /// this section against [`crate::acset::schema::canonical`], so the
    /// format doc and the code cannot drift. Empty when the file has no
    /// such section (only the format doc itself carries one).
    pub reference_typing_body: String,
    /// All `[[wiki-links]]` found in *structured* fields (frontmatter and
    /// table cells) — never prose. Each link is the raw inner text, which may
    /// be a file id (`specodelic`), a row id (`specodelic.model_present`), or
    /// a section anchor (`model.state`).
    pub links: Vec<Link>,
}

/// One `[[...]]` reference found in a structured field.
#[derive(Debug, Clone, Serialize)]
pub struct Link {
    /// Inner text without brackets, e.g. `specodelic.model_present`.
    pub target: String,
    /// Which structured field it was found in: `frontmatter`, or the table
    /// name (`constraints`, `states`, `transitions`, `properties`).
    pub field: String,
    /// Column the link appeared in (empty for frontmatter).
    pub column: String,
    /// Source id it is anchored to (row id, or the intent id for frontmatter).
    pub source: String,
    /// Byte span of the full `[[…]]` occurrence in the source text
    /// (add-acset-writer task 2.2). Not serialized.
    #[serde(skip_serializing)]
    pub span: Option<Span>,
}

impl Spec {
    /// Parse a spec file from disk.
    pub fn from_file(path: &Path) -> Result<Spec, ParseError> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| ParseError::File(path.display().to_string(), e.to_string()))?;
        let mut spec = parse_str(&text).map_err(|e| match e {
            ParseError::Cell(_, line, msg) => {
                ParseError::Cell(path.display().to_string(), line, msg)
            }
            other => other,
        })?;
        spec.path = Some(path.to_path_buf());
        Ok(spec)
    }

    /// The Intent id, e.g. `order.cancel`.
    pub fn id(&self) -> &str {
        &self.intent.id
    }

    /// Every id *defined* in this file (intent + all table rows + states).
    pub fn defined_ids(&self) -> Vec<String> {
        let mut ids = vec![self.intent.id.clone()];
        for r in self
            .constraints
            .iter()
            .chain(&self.properties)
            .chain(&self.states)
        {
            ids.push(r.id.clone());
        }
        for t in &self.transitions {
            ids.push(t.id.clone());
        }
        ids
    }

    /// Is this link's target defined *within this file*? Section anchors
    /// (`model.state`, `model.transition`) count as internally defined —
    /// they address the Model section's rows, not a row id.
    pub fn resolves_locally(&self, target: &str) -> bool {
        if target == "model.state" || target == "model.transition" {
            return true;
        }
        // A link resolves locally if the target equals the file id or one of
        // the file's row ids, or extends the file id / a row id with `.member`.
        if target == self.intent.id {
            return true;
        }
        for id in self.defined_ids() {
            if target == id || target.strip_prefix(&format!("{id}.")).is_some() {
                return true;
            }
        }
        false
    }
}

/// Case labels in a law-kind Property row's predicate: every
/// `**name:**` occurrence (specodelic.md Revision 13's
/// machine-findable case form). Shared by compile's block expansion
/// and the lint `law_cases` rule so the compiler and the linter
/// cannot disagree on what a case is.
pub(crate) fn law_case_labels(predicate: &str) -> Vec<String> {
    let re = regex::Regex::new(r"\*\*([a-zA-Z][a-zA-Z _-]*?):\*\*").expect("static regex");
    re.captures_iter(predicate)
        .map(|c| c[1].trim().to_string())
        .filter(|c| !c.is_empty())
        .collect()
}

/// Parse a spec from a string (path-less; used by tests and stdin).
pub fn parse_str(text: &str) -> Result<Spec, ParseError> {
    // Byte offset of each line's start — the coordinate system for every
    // recorded span (add-acset-writer task 2.2).
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
        .collect();
    let mut lines = text.lines().enumerate().peekable();

    // --- Frontmatter: leading `---` fenced block ---
    let mut fm_raw = String::new();
    let mut in_fm = false;
    let mut saw_any = false;
    loop {
        match lines.peek() {
            None => break,
            Some((_, l)) => {
                let t = l.trim_end();
                if !in_fm && t == "---" {
                    in_fm = true;
                    saw_any = true;
                    lines.next();
                    continue;
                }
                if in_fm && t == "---" {
                    lines.next();
                    break;
                }
                if in_fm {
                    fm_raw.push_str(t);
                    fm_raw.push('\n');
                    lines.next();
                    continue;
                }
                // blank lines before frontmatter are fine
                if t.trim().is_empty() {
                    lines.next();
                    continue;
                }
                break;
            }
        }
    }
    if !saw_any {
        return Err(ParseError::File(
            "<input>".into(),
            "no YAML frontmatter found — every spec file starts with a `---` fenced Intent block"
                .into(),
        ));
    }
    let fm: BTreeMap<String, serde_yaml_ng::Value> = serde_yaml_ng::from_str(&fm_raw)
        .map_err(|e| ParseError::File("<frontmatter>".into(), e.to_string()))?;
    let get = |k: &str| -> Option<String> { fm.get(k).map(v_to_string) };
    let intent = Intent {
        id_span: None,
        id: get("id")
            .ok_or_else(|| ParseError::File("<frontmatter>".into(), "missing `id` field".into()))?,
        kind: get("kind").ok_or_else(|| {
            ParseError::File("<frontmatter>".into(), "missing `kind` field".into())
        })?,
        statement: get("statement").ok_or_else(|| {
            ParseError::File("<frontmatter>".into(), "missing `statement` field".into())
        })?,
        extra: fm
            .into_iter()
            .filter(|(k, _)| !matches!(k.as_str(), "id" | "kind" | "statement"))
            .collect(),
    };

    let mut spec = Spec {
        path: None,
        intent,
        constraints: vec![],
        states: vec![],
        transitions: vec![],
        properties: vec![],
        has_added_requirements: false,
        has_requirements_section: false,
        added_requirements_body: String::new(),
        has_modified_requirements: false,
        modified_requirements_body: String::new(),
        requirements_body: String::new(),
        reference_typing_body: String::new(),
        links: vec![],
    };
    spec.links.extend(frontmatter_links(text, &spec.intent.id));
    spec.intent.id_span = frontmatter_id_span(text, &spec.intent.id);

    // --- Body sections ---
    let mut current_table: TableKind = TableKind::None;
    let mut headers: Vec<String> = vec![];
    let mut in_states = false;
    // Inside `### Reference Typing` — capture the section verbatim for the
    // schema drift gate (add-acset-core task 2.4).
    let mut in_reference_typing = false;
    // Which dual-format requirement section (if any) we are inside —
    // bodies are captured verbatim (line-rstripped) for the drift check.
    let mut dual_section = 0u8; // 0 none · 1 ADDED Requirements · 2 Requirements · 3 MODIFIED Requirements

    for (n, line) in lines {
        let lineno = n + 1;
        let t = line.trim();

        // Capture the requirement-section bodies (every line except the
        // `## ` headings themselves; `### Requirement:` lines included).
        if !t.starts_with("## ") {
            match dual_section {
                1 => {
                    spec.added_requirements_body.push_str(line.trim_end());
                    spec.added_requirements_body.push('\n');
                }
                2 => {
                    spec.requirements_body.push_str(line.trim_end());
                    spec.requirements_body.push('\n');
                }
                3 => {
                    spec.modified_requirements_body.push_str(line.trim_end());
                    spec.modified_requirements_body.push('\n');
                }
                _ => {}
            }
        }

        if let Some(heading) = t.strip_prefix("## ") {
            let h = heading.trim();
            dual_section = match h {
                "ADDED Requirements" => 1,
                "Requirements" => 2,
                "MODIFIED Requirements" => 3,
                _ => 0,
            };
            current_table = match h {
                "Constraints" => TableKind::Constraints,
                "Properties" => TableKind::Properties,
                "Model" => TableKind::Model,
                _ => TableKind::None,
            };
            // Dual-format markers (spec-integration protocol): the
            // openspec delta half and the capability-spec half.
            if h == "ADDED Requirements" {
                spec.has_added_requirements = true;
            }
            if h == "MODIFIED Requirements" {
                spec.has_modified_requirements = true;
            }
            if h == "Requirements" {
                spec.has_requirements_section = true;
            }
            in_states = false;
            in_reference_typing = false;
            headers = vec![];
            continue;
        }
        if let Some(heading) = t.strip_prefix("### ") {
            let h = heading.trim();
            in_states = h == "States";
            in_reference_typing = h == "Reference Typing";
            headers = vec![];
            current_table = match (current_table, h) {
                (TableKind::Model, "Transitions") => TableKind::Transitions,
                (TableKind::Model, "States") | (TableKind::Transitions, "States") => {
                    TableKind::Model
                }
                // Any other `###` subsection (e.g. Reference Typing,
                // Checker Ownership) is not one of the three structured
                // tables — leave table context.
                _ => TableKind::None,
            };
            continue;
        }

        // Reference Typing capture: headings above already `continue`d, so
        // every line reaching here is section body — kept verbatim (the
        // same rstripped discipline as the dual-section bodies).
        if in_reference_typing {
            spec.reference_typing_body.push_str(line.trim_end());
            spec.reference_typing_body.push('\n');
        }

        match current_table {
            TableKind::None | TableKind::Model => {
                // Model prose and States bullets handled below.
                if in_states && t.starts_with("- ") {
                    push_state_row(&mut spec, line, line_starts[n]);
                }
            }
            TableKind::Constraints | TableKind::Properties | TableKind::Transitions => {
                if t.starts_with('|') && !is_separator(t) {
                    if headers.is_empty() {
                        headers = parse_row(t)
                            .map_err(|m| ParseError::Cell("<table>".into(), lineno, m))?;
                        continue;
                    }
                    push_table_row(
                        &mut spec,
                        line,
                        line_starts[n],
                        current_table,
                        &headers,
                        lineno,
                    )?;
                } else if t.is_empty() {
                    continue;
                } else if headers.is_empty() && !t.starts_with('|') {
                    // prose between heading and table — leave the table open
                    continue;
                }
            }
        }
    }
    Ok(spec)
}

/// Parse one `### States` bullet line into a `Row` and push it (with
/// its recorded id span and spanned emits links) onto the spec
/// (add-acset-writer task 2.2; extracted from parse_str for the
/// pretender cyclomatic ratchet).
fn push_state_row(spec: &mut Spec, line: &str, line_start: usize) {
    let t = line.trim();
    let bullet = t.trim_start_matches("- ").trim();
    let (id, emits) = parse_state_bullet(bullet);
    let mut cells = BTreeMap::new();
    cells.insert("id".into(), id.clone());
    let (id_span, emits_at) = state_bullet_spans(line, line_start);
    if let Some(e) = emits {
        cells.insert("emits".into(), e.clone());
        // emits is a typed reference to a Constraint — extract
        // the [[link]] rather than treating the raw cell as
        // the target.
        let base = emits_at.and_then(|at| {
            line.get(at - line_start..)
                .and_then(|rest| rest.find(e.as_str()))
                .map(|rel| at + rel)
        });
        if let Some(b) = base {
            spec.links
                .extend(collect_links_spanned(&e, b, "states", "emits", &id));
        } else {
            spec.links.extend(collect_links(&e, "states", "emits", &id));
        }
    }
    spec.states.push(Row {
        id,
        kind: None,
        cells,
        id_span,
    });
}

/// Parse one data row of a structured table into its `Row`/`Transition`
/// and push it (with recorded id/from/to spans and spanned cell links)
/// onto the spec (add-acset-writer task 2.2; extracted from parse_str
/// for the pretender cyclomatic ratchet).
fn push_table_row(
    spec: &mut Spec,
    line: &str,
    line_start: usize,
    current_table: TableKind,
    headers: &[String],
    lineno: usize,
) -> Result<(), ParseError> {
    let cells_spanned = parse_row_spans(line, line_start)
        .map_err(|m| ParseError::Cell("<table>".into(), lineno, m))?;
    let cells_raw: Vec<String> = cells_spanned.iter().map(|(c, _)| c.clone()).collect();
    let cell_span = |name: &str| -> Option<Span> {
        headers
            .iter()
            .position(|h| h == name)
            .and_then(|i| cells_spanned.get(i))
            .map(|(_, s)| *s)
    };
    let get_col = |name: &str| -> Option<String> {
        headers
            .iter()
            .position(|h| h == name)
            .and_then(|i| cells_raw.get(i).cloned())
    };
    let id = get_col("id").unwrap_or_default();
    if id.is_empty() {
        return Err(ParseError::Cell(
            "<table>".into(),
            lineno,
            "row has no `id` cell".into(),
        ));
    }
    let kind = get_col("kind");
    let mut cells = BTreeMap::new();
    for (i, h) in headers.iter().enumerate() {
        if let Some(v) = cells_raw.get(i) {
            cells.insert(h.clone(), v.clone());
        }
    }
    // collect links from every cell
    for (h, v) in &cells {
        if h == "id" || h == "kind" {
            continue;
        }
        // Spanned when the cell's bytes can be located in the raw line;
        // unspanned fallback otherwise (the writer reports span_failure
        // when it needs one).
        if let Some(i) = headers.iter().position(|x| x == h)
            && let Some((_, s)) = cells_spanned.get(i)
        {
            spec.links.extend(collect_links_spanned(
                v,
                s.start,
                table_field(current_table),
                h,
                &id,
            ));
        } else {
            spec.links
                .extend(collect_links(v, table_field(current_table), h, &id));
        }
    }
    let id_span = cell_span("id");
    match current_table {
        TableKind::Constraints => spec.constraints.push(Row {
            id,
            kind,
            cells,
            id_span,
        }),
        TableKind::Properties => spec.properties.push(Row {
            id,
            kind,
            cells,
            id_span,
        }),
        TableKind::Transitions => spec.transitions.push(Transition {
            id,
            from: get_col("from").unwrap_or_default(),
            to: get_col("to").unwrap_or_default(),
            guard: get_col("guard").filter(|g| !g.trim().is_empty()),
            id_span,
            from_span: cell_span("from"),
            to_span: cell_span("to"),
        }),
        _ => unreachable!(),
    }
    Ok(())
}

fn v_to_string(v: &serde_yaml_ng::Value) -> String {
    match v {
        serde_yaml_ng::Value::String(s) => s.clone(),
        serde_yaml_ng::Value::Number(n) => n.to_string(),
        serde_yaml_ng::Value::Bool(b) => b.to_string(),
        other => serde_yaml_ng::to_string(other)
            .unwrap_or_default()
            .trim()
            .to_string(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TableKind {
    None,
    Constraints,
    Properties,
    Transitions,
    Model,
}

fn table_field(k: TableKind) -> &'static str {
    match k {
        TableKind::Constraints => "constraints",
        TableKind::Properties => "properties",
        TableKind::Transitions => "transitions",
        _ => "",
    }
}

pub(crate) fn is_separator(line: &str) -> bool {
    line.replace(['|', '-', ':', ' '], "").is_empty() && line.contains('-')
}

/// Split a markdown table line into cells, honoring backtick fences.
pub(crate) fn parse_row(line: &str) -> Result<Vec<String>, String> {
    let t = line.trim();
    let t = t.strip_prefix('|').ok_or("row does not start with `|`")?;
    let t = t.strip_suffix('|').unwrap_or(t);
    let mut cells = vec![];
    let mut cur = String::new();
    let mut in_backtick = false;
    for c in t.chars() {
        match c {
            '`' => {
                in_backtick = !in_backtick;
                cur.push(c);
            }
            '|' if !in_backtick => {
                cells.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    cells.push(cur.trim().to_string());
    Ok(cells)
}

/// Parse a States bullet: ``- `active` `` or ``- `cancel_requested` (emits: `[[x.y]]`)``.
/// Ids are backticked or bare; `emits` is a parenthesized suffix.
fn parse_state_bullet(bullet: &str) -> (String, Option<String>) {
    let bullet = bullet.trim().trim_start_matches('`').trim();
    // forms: "`id`" / "id" / "`id` (emits: X)" / "id — emits X"
    let (head, rest) = match bullet.find(['`', ' ', '(']) {
        Some(0) => {
            // backticked id: up to closing backtick
            if let Some(end) = bullet[1..].find('`') {
                (&bullet[1..1 + end], Some(bullet[1 + end + 1..].trim()))
            } else {
                (bullet, None)
            }
        }
        Some(_) | None => match bullet.split_once(char::is_whitespace) {
            Some((a, b)) => (a, Some(b.trim())),
            None => (bullet, None),
        },
    };
    let id = head.trim().trim_matches('`').to_string();
    let emits = rest.and_then(|r| {
        let r = r.trim_start_matches(['(', '—', '-', ':']).trim();
        let r = r
            .strip_prefix("emits:")
            .or_else(|| r.strip_prefix("emits"))
            .unwrap_or(r)
            .trim();
        let r = r.trim_end_matches(')').trim();
        let r = r.trim_matches('`').trim();
        (!r.is_empty()).then(|| r.to_string())
    });
    (id, emits)
}

/// Extract every `[[link]]` from a structured cell, with backtick awareness
/// not needed — `[[...]]` is unambiguous. (Unspanned fallback: used when a
/// cell's bytes cannot be located in the raw line.)
fn collect_links(cell: &str, field: &str, column: &str, source: &str) -> Vec<Link> {
    collect_links_spanned(cell, 0, field, column, source)
        .into_iter()
        .map(|mut l| {
            l.span = None;
            l
        })
        .collect()
}

/// [`collect_links`] with byte spans: `cell_base` is the cell content's
/// absolute offset in the source text; each recorded span covers the
/// full `[[…]]` occurrence (add-acset-writer task 2.2).
fn collect_links_spanned(
    cell: &str,
    cell_base: usize,
    field: &str,
    column: &str,
    source: &str,
) -> Vec<Link> {
    let mut links = vec![];
    let mut rest = cell;
    let mut consumed = 0usize;
    while let Some(start) = rest.find("[[") {
        let after = &rest[start + 2..];
        match after.find("]]") {
            Some(end) => {
                let target = after[..end].trim().to_string();
                if !target.is_empty() {
                    links.push(Link {
                        target,
                        field: field.into(),
                        column: column.into(),
                        source: source.into(),
                        span: Some(Span {
                            start: cell_base + consumed + start,
                            end: cell_base + consumed + start + 2 + end + 2,
                        }),
                    });
                }
                let adv = start + 2 + end + 2;
                rest = &after[end + 2..];
                consumed += adv;
            }
            None => break,
        }
    }
    links
}

/// Split a markdown table row into cells WITH byte spans — the spanned
/// counterpart of [`parse_row`]. `base` is the line's absolute start
/// offset in the source text; each span covers the cell's trimmed
/// content (what an id edit rewrites — padding around it is untouched,
/// `width_padding_policy`).
pub(crate) fn parse_row_spans(line: &str, base: usize) -> Result<Vec<(String, Span)>, String> {
    let t = line.trim();
    let t_off = line.len() - line.trim_start().len();
    let t = t.strip_prefix('|').ok_or("row does not start with `|`")?;
    let t_off = t_off + 1;
    let t = t.strip_suffix('|').unwrap_or(t);
    let mut cells: Vec<(String, Span)> = vec![];
    let mut buf = String::new();
    let mut first: Option<usize> = None;
    let mut in_backtick = false;
    let mut push_cell = |buf: &str, first: Option<usize>, pos: usize| {
        let content = buf.trim();
        let span = match first {
            Some(f) => Span {
                start: base + t_off + f,
                end: base + t_off + f + content.len(),
            },
            None => Span {
                start: base + pos,
                end: base + pos,
            },
        };
        cells.push((content.to_string(), span));
    };
    for (bi, c) in t.char_indices() {
        match c {
            '`' => {
                if first.is_none() {
                    first = Some(bi);
                }
                in_backtick = !in_backtick;
                buf.push(c);
            }
            '|' if !in_backtick => {
                push_cell(&buf, first, t_off + bi);
                buf.clear();
                first = None;
            }
            _ => {
                if first.is_none() && !c.is_whitespace() {
                    first = Some(bi);
                }
                buf.push(c);
            }
        }
    }
    push_cell(&buf, first, t_off + t.len());
    Ok(cells)
}

/// Byte span of a States bullet's leading id token in the raw line
/// (mirrors [`parse_state_bullet`]'s head logic), plus the absolute
/// offset where the emits region begins (for link spans). `base` is the
/// line's absolute start offset. `(None, None)` when the shape is
/// unrecognized — the writer reports `span_failure` if an edit needs it.
fn state_bullet_spans(line: &str, base: usize) -> (Option<Span>, Option<usize>) {
    let trimmed = line.trim_start();
    let lead = line.len() - trimmed.len();
    let Some(rest) = trimmed.strip_prefix("- ") else {
        return (None, None);
    };
    let off = |i: usize| base + lead + 2 + i;
    let content_off = rest.len() - rest.trim_start().len();
    let c = &rest[content_off..];
    if let Some(stripped) = c.strip_prefix('`') {
        let inner = content_off + 1;
        match stripped.find('`') {
            Some(end) => (
                Some(Span {
                    start: off(inner),
                    end: off(inner + end),
                }),
                Some(off(inner + end + 1)),
            ),
            None => (None, None),
        }
    } else {
        let tok_end = c.find([' ', '(']).unwrap_or(c.len());
        if tok_end == 0 {
            return (None, None);
        }
        (
            Some(Span {
                start: off(content_off),
                end: off(content_off + tok_end),
            }),
            Some(off(content_off + tok_end)),
        )
    }
}

/// Byte span of the frontmatter `id:` value in the raw text — `None`
/// unless the raw value bytes are exactly the parsed id (a quoted or
/// multi-line value is not a span the writer can safely rewrite).
fn frontmatter_id_span(text: &str, id: &str) -> Option<Span> {
    let mut in_fm = false;
    let mut off = 0usize;
    for seg in text.split_inclusive('\n') {
        let line = seg.strip_suffix('\n').unwrap_or(seg);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let this_off = off;
        off += seg.len();
        let t = line.trim();
        if !in_fm {
            if t == "---" {
                in_fm = true;
            }
            continue;
        }
        if t == "---" {
            break;
        }
        let Some(value) = t.strip_prefix("id:") else {
            continue;
        };
        let lead = line.len() - line.trim_start().len();
        let value_off_in_line = lead + 3 + (value.len() - value.trim_start().len());
        let v = value.trim();
        return (v == id).then(|| Span {
            start: this_off + value_off_in_line,
            end: this_off + value_off_in_line + v.len(),
        });
    }
    None
}

/// Every `[[link]]` in the frontmatter region with absolute byte spans
/// (the spanned counterpart of the old `collect_links(&fm_raw, …)` —
/// `fm_raw` is a reconstructed string whose offsets do not map back to
/// the source, so the raw region is scanned directly).
fn frontmatter_links(text: &str, source: &str) -> Vec<Link> {
    let mut links = vec![];
    let mut in_fm = false;
    let mut off = 0usize;
    for seg in text.split_inclusive('\n') {
        let line = seg.strip_suffix('\n').unwrap_or(seg);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let this_off = off;
        off += seg.len();
        let t = line.trim();
        if !in_fm {
            if t == "---" {
                in_fm = true;
            }
            continue;
        }
        if t == "---" {
            break;
        }
        let lead = line.len() - line.trim_start().len();
        links.extend(collect_links_spanned(
            t,
            this_off + lead,
            "frontmatter",
            "",
            source,
        ));
    }
    links
}
