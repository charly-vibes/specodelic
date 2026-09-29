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
}

/// A transition row from the Model's `### Transitions` table.
#[derive(Debug, Clone, Serialize)]
pub struct Transition {
    pub id: String,
    pub from: String,
    pub to: String,
    /// `None` when the guard cell is empty (`null` guard).
    pub guard: Option<String>,
}

/// Parsed frontmatter (the Intent layer).
#[derive(Debug, Clone, Serialize)]
pub struct Intent {
    pub id: String,
    pub kind: String,
    pub statement: String,
    /// Any extra frontmatter fields (e.g. `checked_against_core`).
    pub extra: BTreeMap<String, serde_yaml::Value>,
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
    /// Body of `## Requirements` (lines rstripped, joined).
    pub requirements_body: String,
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

/// Parse a spec from a string (path-less; used by tests and stdin).
pub fn parse_str(text: &str) -> Result<Spec, ParseError> {
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
    let fm: BTreeMap<String, serde_yaml::Value> = serde_yaml::from_str(&fm_raw)
        .map_err(|e| ParseError::File("<frontmatter>".into(), e.to_string()))?;
    let get = |k: &str| -> Option<String> { fm.get(k).map(v_to_string) };
    let intent = Intent {
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
        requirements_body: String::new(),
        links: vec![],
    };
    spec.links
        .extend(collect_links(&fm_raw, "frontmatter", "", &spec.intent.id));

    // --- Body sections ---
    let mut current_table: TableKind = TableKind::None;
    let mut headers: Vec<String> = vec![];
    let mut in_states = false;
    // Which dual-format requirement section (if any) we are inside —
    // bodies are captured verbatim (line-rstripped) for the drift check.
    let mut dual_section = 0u8; // 0 none · 1 ADDED Requirements · 2 Requirements

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
                _ => {}
            }
        }

        if let Some(heading) = t.strip_prefix("## ") {
            let h = heading.trim();
            dual_section = match h {
                "ADDED Requirements" => 1,
                "Requirements" => 2,
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
            if h == "Requirements" {
                spec.has_requirements_section = true;
            }
            in_states = false;
            headers = vec![];
            continue;
        }
        if let Some(heading) = t.strip_prefix("### ") {
            let h = heading.trim();
            in_states = h == "States";
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

        match current_table {
            TableKind::None | TableKind::Model => {
                // Model prose and States bullets handled below.
                if in_states && t.starts_with("- ") {
                    let bullet = t.trim_start_matches("- ").trim();
                    let (id, emits) = parse_state_bullet(bullet);
                    let mut cells = BTreeMap::new();
                    cells.insert("id".into(), id.clone());
                    if let Some(e) = emits {
                        cells.insert("emits".into(), e.clone());
                        // emits is a typed reference to a Constraint — extract
                        // the [[link]] rather than treating the raw cell as
                        // the target.
                        for l in collect_links(&e, "states", "emits", &id) {
                            spec.links.push(l);
                        }
                    }
                    spec.states.push(Row {
                        id,
                        kind: None,
                        cells,
                    });
                }
            }
            TableKind::Constraints | TableKind::Properties | TableKind::Transitions => {
                if t.starts_with('|') && !is_separator(t) {
                    if headers.is_empty() {
                        headers = parse_row(t)
                            .map_err(|m| ParseError::Cell("<table>".into(), lineno, m))?;
                        continue;
                    }
                    let cells_raw =
                        parse_row(t).map_err(|m| ParseError::Cell("<table>".into(), lineno, m))?;
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
                        spec.links
                            .extend(collect_links(v, table_field(current_table), h, &id));
                    }
                    match current_table {
                        TableKind::Constraints => spec.constraints.push(Row { id, kind, cells }),
                        TableKind::Properties => spec.properties.push(Row { id, kind, cells }),
                        TableKind::Transitions => {
                            spec.transitions.push(Transition {
                                id,
                                from: get_col("from").unwrap_or_default(),
                                to: get_col("to").unwrap_or_default(),
                                guard: get_col("guard").filter(|g| !g.trim().is_empty()),
                            });
                        }
                        _ => unreachable!(),
                    }
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

fn v_to_string(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        other => serde_yaml::to_string(other)
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

fn is_separator(line: &str) -> bool {
    line.replace(['|', '-', ':', ' '], "").is_empty() && line.contains('-')
}

/// Split a markdown table line into cells, honoring backtick fences.
fn parse_row(line: &str) -> Result<Vec<String>, String> {
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
/// not needed — `[[...]]` is unambiguous.
fn collect_links(cell: &str, field: &str, column: &str, source: &str) -> Vec<Link> {
    let mut links = vec![];
    let mut rest = cell;
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
                    });
                }
                rest = &after[end + 2..];
            }
            None => break,
        }
    }
    links
}
