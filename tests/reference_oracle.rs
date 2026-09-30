//! Purpose: independent reference oracle for the spk checkers — the
//! corpus-reconciliation acceptance leg of beads specodelic-vv8 S2.
//!
//! Responsibilities:
//! - Implement a MINI STRUCTURAL CHECKER written from the spec text alone
//!   (`specs/linter-frontmatter.md`, `specs/linter-referential_integrity.md`,
//!   the Reference Typing table and `id_matches_file`/`every_transition_valid`
//!   invariants in `specs/specodelic.md`) with its OWN naive frontmatter /
//!   table / `[[wiki-link]]` parser and its OWN resolution algorithm — no
//!   use of `spec::parse_str`, `lint`, or `graph` internals, so a bug in the
//!   tool's parser cannot hide in the oracle.
//! - CORPUS GATE: the repo's own `specs/*.md` and `openspec/specs/*/spec.md`
//!   must pass BOTH the oracle (this file) and spk lint (the matrix) — the
//!   corpus-reconciliation acceptance leg.
//! - AGREEMENT CROSS-CHECK: on small fixtures, oracle and spk agree — both
//!   fire or both stay clean. The oracle's scope is exactly the surfaces spk
//!   owns (frontmatter family, referential integrity incl. Reference Typing,
//!   transition validity, within-file id uniqueness); schema-shape kind
//!   closure (`constraint_kind_closed` / `property_kind_closed`) is
//!   deliberately OUT of scope — spk does not enforce it yet (beads
//!   specodelic-7h8), and an oracle stricter than spk would break the
//!   both-fire-or-neither contract.
//!
//! Rationale: the 2026-09-28 adversarial review's reference checker existed
//! only outside the repo; this files it in as the acceptance gate. Oracle
//! rule names are namespaced `oracle.*` — they are findings of the
//! independent checker, never spk rule ids. Anti-goal: do not weaken an
//! oracle rule to make the corpus pass; a corpus failure is either an
//! oracle bug (fix the oracle) or a real corpus gap (file a bead).

use specodelic::checklist::Checklist;
use specodelic::graph;
use specodelic::lint;
use specodelic::spec::parse_str;
use std::path::Path;
use std::path::PathBuf;

// ===========================================================================
// The oracle — an independent mini structural checker from the spec text
// ===========================================================================

mod oracle {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;
    use std::path::Path;

    // ---------- independent parse ----------

    #[derive(Debug, Clone, PartialEq)]
    pub struct Fm {
        pub id: Option<String>,
        pub kind: Option<String>,
        pub statement: Option<String>,
    }

    /// One table row: `id`, optional `kind` attribute cell, and the rest of
    /// the columns raw (backticks/quotes stripped).
    #[derive(Debug)]
    pub struct Row {
        pub id: String,
        pub kind: Option<String>,
        pub cells: BTreeMap<String, String>,
    }

    #[derive(Debug, Default)]
    pub struct File {
        pub path: String,
        pub fm: Option<Fm>,
        pub constraints: Vec<Row>,
        /// `(id, emits-cell)` bullets under `### States`.
        pub states: Vec<(String, Option<String>)>,
        pub transitions: Vec<Row>,
        pub properties: Vec<Row>,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct Finding {
        pub rule: &'static str,
        pub detail: String,
    }

    fn finding(rule: &'static str, detail: impl Into<String>) -> Finding {
        Finding {
            rule,
            detail: detail.into(),
        }
    }

    fn cell(raw: &str) -> String {
        raw.trim()
            .trim_matches('`')
            .trim()
            .trim_matches('"')
            .trim()
            .to_string()
    }

    fn is_separator_row(cells: &[&str]) -> bool {
        cells.iter().all(|c| {
            let t = c.trim().trim_matches(':');
            !t.is_empty() && t.chars().all(|ch| ch == '-')
        })
    }

    fn split_row(line: &str) -> Vec<String> {
        line.trim()
            .trim_start_matches('|')
            .trim_end_matches('|')
            .split('|')
            .map(|c| c.trim().to_string())
            .collect()
    }

    /// Parse a table block into `(header, rows)` where each row maps header
    /// column → cleaned cell.
    fn parse_table(lines: &[&str]) -> (Vec<String>, Vec<BTreeMap<String, String>>) {
        let mut table_lines = Vec::new();
        for l in lines {
            if l.trim().starts_with('|') {
                table_lines.push(l.trim());
            } else if !table_lines.is_empty() {
                break;
            }
        }
        let mut iter = table_lines.iter();
        let header: Vec<String> = iter.next().map(|h| split_row(h)).unwrap_or_default();
        let mut rows = Vec::new();
        for l in iter {
            let cells = split_row(l);
            if is_separator_row(&cells.iter().map(|s| s.as_str()).collect::<Vec<_>>()) {
                continue;
            }
            let mut row = BTreeMap::new();
            for (i, col) in header.iter().enumerate() {
                row.insert(
                    col.to_lowercase(),
                    cells.get(i).cloned().unwrap_or_default(),
                );
            }
            rows.push(row);
        }
        (header, rows)
    }

    fn rows_with_ids(raw_rows: Vec<BTreeMap<String, String>>) -> Vec<Row> {
        raw_rows
            .into_iter()
            .filter_map(|mut cells| {
                let id = cells.remove("id")?;
                let kind = cells.remove("kind").map(|k| cell(&k));
                let cells = cells.into_iter().map(|(k, v)| (k, cell(&v))).collect();
                Some(Row {
                    id: cell(&id),
                    kind,
                    cells,
                })
            })
            .collect()
    }

    /// Naive parse of one file: frontmatter (leading `---` block), the three
    /// layer tables (`## Constraints`, `## Properties`), and the model
    /// (`## Model` → `### States` bullets + `### Transitions` table).
    /// Sections other than these are skipped — prose is never inspected.
    pub fn parse_file(path: &str, text: &str) -> File {
        let mut file = File {
            path: path.to_string(),
            ..Default::default()
        };

        let mut lines = text.lines().peekable();

        // Frontmatter: leading `---` fenced block with `key: value` lines.
        if text.starts_with("---") {
            lines.next(); // opening ---
            let mut id = None;
            let mut kind = None;
            let mut statement = None;
            for l in lines.by_ref() {
                if l.trim() == "---" {
                    break;
                }
                if let Some((k, v)) = l.split_once(':') {
                    let v = cell(v);
                    match k.trim() {
                        "id" => id = Some(v),
                        "kind" => kind = Some(v),
                        "statement" => statement = Some(v),
                        _ => {}
                    }
                }
            }
            file.fm = Some(Fm {
                id,
                kind,
                statement,
            });
        }

        // Body sections. `## X` switches the h2; `### Y` switches the h3.
        let body: Vec<&str> = lines.collect();
        let mut h2 = String::new();
        let mut h3 = String::new();
        let mut i = 0usize;
        while i < body.len() {
            let l = body[i];
            if let Some(h) = l.strip_prefix("## ") {
                h2 = h.trim().to_string();
                h3.clear();
                i += 1;
                continue;
            }
            if let Some(h) = l.strip_prefix("### ") {
                h3 = h.trim().to_string();
                i += 1;
                continue;
            }
            if l.trim().starts_with('|') {
                let (header, rows) = parse_table(&body[i..]);
                if !header.is_empty() {
                    let rows = rows_with_ids(rows);
                    // Only a table whose rows actually carry `id` cells is a
                    // layer table — e.g. specodelic.md's `### Reference
                    // Typing` table lives inside the `## Constraints`
                    // section but is meta-documentation, not constraints.
                    if !rows.is_empty() {
                        match (h2.as_str(), h3.as_str()) {
                            ("Constraints", _) => file.constraints = rows,
                            ("Properties", _) => file.properties = rows,
                            ("Model", "Transitions") => file.transitions = rows,
                            _ => {}
                        }
                    }
                }
                // Skip past the consumed table lines.
                while i < body.len() && body[i].trim().starts_with('|') {
                    i += 1;
                }
                continue;
            }
            if h2 == "Model" && h3 == "States" {
                let t = l.trim();
                if let Some(b) = t.strip_prefix("- ") {
                    let id = b
                        .split('`')
                        .nth(1)
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| cell(b));
                    let emits = b
                        .find("(emits:")
                        .and_then(|p| b[p..].split('`').nth(1).map(|s| s.to_string()));
                    file.states.push((id, emits));
                }
            }
            i += 1;
        }
        file
    }

    // ---------- resolution (specs/linter-referential_integrity.md) ----------

    /// What a `[[target]]` points at. `row: None` = the file's Intent.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Target {
        pub file: String,
        pub row: Option<String>,
        /// Schema kind of the resolved row: Intent/Constraint/State/
        /// Transition/Property, plus the row's own `kind` attribute.
        pub kind: &'static str,
        pub attr: Option<String>,
    }

    #[derive(Debug)]
    pub enum Resolved {
        Found(Target),
        /// Unresolvable — the spec's `ref_resolves` rejects these.
        Unresolved,
        /// Dotless target naming no row and no file: format prose, not a
        /// reference (the metasyntactic skip).
        NotARef,
    }

    fn state_kind_of_row(file: &File, row: &str) -> Option<(&'static str, Option<String>)> {
        for c in &file.constraints {
            if c.id == row {
                return Some(("Constraint", c.kind.clone()));
            }
        }
        for p in &file.properties {
            if p.id == row {
                return Some(("Property", p.kind.clone()));
            }
        }
        if file.states.iter().any(|(id, _)| id == row) {
            return Some(("State", None));
        }
        if file.transitions.iter().any(|t| t.id == row) {
            return Some(("Transition", None));
        }
        // Model section anchors are valid targets of the resolution
        // algorithm (`specs/linter-referential_integrity.md`, step 3).
        if row == "model.state" {
            return Some(("State", None));
        }
        if row == "model.transition" {
            return Some(("Transition", None));
        }
        None
    }

    /// The spec-pinned resolution algorithm: (1) exact file id wins;
    /// (2) bare-local row; (3) every dot split, last to first, remainder =
    /// row | section anchor | row.member (first segment is the row).
    pub fn resolve(
        file_ids: &BTreeSet<String>,
        files: &BTreeMap<String, &File>,
        source: &File,
        target: &str,
    ) -> Resolved {
        // 1. Exact file id.
        if file_ids.contains(target) {
            return Resolved::Found(Target {
                file: target.to_string(),
                row: None,
                kind: "Intent",
                attr: None,
            });
        }
        // 2. Bare-local row.
        if !target.contains('.') {
            if let Some((kind, attr)) = state_kind_of_row(source, target) {
                return Resolved::Found(Target {
                    file: source
                        .fm
                        .as_ref()
                        .and_then(|f| f.id.clone())
                        .unwrap_or_default(),
                    row: Some(target.to_string()),
                    kind,
                    attr,
                });
            }
            return Resolved::NotARef;
        }
        // 2½. Own-file qualification (self-containment law): a target
        // prefixed with the SOURCE file's own id resolves within that file.
        // This is load-bearing for the `id: spec` delta files — every
        // openspec capability spec shares the file id `spec`, so the
        // corpus-wide index cannot disambiguate them; the naming law makes
        // each one self-contained instead.
        let source_id = source
            .fm
            .as_ref()
            .and_then(|f| f.id.clone())
            .unwrap_or_default();
        if !source_id.is_empty() {
            if let Some(rest) = target.strip_prefix(&format!("{source_id}.")) {
                if let Some((kind, attr)) = state_kind_of_row(source, rest) {
                    return Resolved::Found(Target {
                        file: source_id,
                        row: Some(rest.to_string()),
                        kind,
                        attr,
                    });
                }
                if let Some((first, _)) = rest.split_once('.') {
                    if let Some((kind, attr)) = state_kind_of_row(source, first) {
                        return Resolved::Found(Target {
                            file: source_id,
                            row: Some(first.to_string()),
                            kind,
                            attr,
                        });
                    }
                }
            }
        }
        // 3. Every dot split, last to first.
        let parts: Vec<&str> = target.split('.').collect();
        for split in (1..parts.len()).rev() {
            let prefix = parts[..split].join(".");
            if !file_ids.contains(&prefix) {
                continue;
            }
            let remainder = parts[split..].join(".");
            let Some(target_file) = files.get(&prefix) else {
                continue;
            };
            if let Some((kind, attr)) = state_kind_of_row(target_file, &remainder) {
                return Resolved::Found(Target {
                    file: prefix,
                    row: Some(remainder),
                    kind,
                    attr,
                });
            }
            // row.member: the row is the single segment right after the
            // split; the member path may itself be dotted.
            if let Some((first, _)) = remainder.split_once('.') {
                if let Some((kind, attr)) = state_kind_of_row(target_file, first) {
                    return Resolved::Found(Target {
                        file: prefix,
                        row: Some(first.to_string()),
                        kind,
                        attr,
                    });
                }
            }
        }
        Resolved::Unresolved
    }

    /// Extract every `[[wiki-link]]` from a typed cell.
    fn links(cell: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = cell;
        while let Some(p) = rest.find("[[") {
            let after = &rest[p + 2..];
            if let Some(q) = after.find("]]") {
                out.push(after[..q].to_string());
                rest = &after[q + 2..];
            } else {
                break;
            }
        }
        out
    }

    // ---------- checks ----------

    /// Frontmatter gate (`specs/linter-frontmatter.md`) + `id_matches_file`
    /// (`specs/specodelic.md`): id present + `[a-z][a-z0-9_.]*`, kind ==
    /// intent, statement non-empty, id == stem(path) with `-` → `.`.
    /// Files with no frontmatter are exempt non-spec files.
    fn frontmatter_findings(file: &File, out: &mut Vec<Finding>) {
        let Some(fm) = &file.fm else { return };
        let id = match &fm.id {
            Some(id) if !id.is_empty() => id,
            _ => {
                out.push(finding(
                    "oracle.frontmatter_id",
                    format!("{}: missing id", file.path),
                ));
                return;
            }
        };
        let mut chars = id.chars();
        let valid = chars.next().is_some_and(|c| c.is_ascii_lowercase())
            && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.');
        if !valid {
            out.push(finding(
                "oracle.frontmatter_id",
                format!("{}: malformed id {id:?}", file.path),
            ));
        }
        if fm.kind.as_deref() != Some("intent") {
            out.push(finding(
                "oracle.frontmatter_kind",
                format!("{}: kind is {:?}, want \"intent\"", file.path, fm.kind),
            ));
        }
        if fm
            .statement
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .is_empty()
        {
            out.push(finding(
                "oracle.frontmatter_statement",
                format!("{}: missing/empty statement", file.path),
            ));
        }
        let stem = Path::new(&file.path)
            .file_stem()
            .map(|s| s.to_string_lossy().replace('-', "."))
            .unwrap_or_default();
        if *id != stem {
            out.push(finding(
                "oracle.frontmatter_id_matches_file",
                format!("{}: id {id:?} != stem-derived {stem:?}", file.path),
            ));
        }
    }

    /// `unique_within_file` (`specs/linter-referential_integrity.md`): no
    /// two rows in a file share the same local id.
    fn duplicate_findings(file: &File, out: &mut Vec<Finding>) {
        let mut seen: Vec<(String, &'static str)> = Vec::new();
        let mut mark = |id: &str, layer: &'static str, file: &File, out: &mut Vec<Finding>| {
            if seen.iter().any(|(s, _)| s == id) {
                out.push(finding(
                    "oracle.duplicate_row_id",
                    format!("{}: duplicate row id {id:?}", file.path),
                ));
            } else {
                seen.push((id.to_string(), layer));
            }
        };
        for r in &file.constraints {
            mark(&r.id, "constraint", file, out);
        }
        for r in &file.properties {
            mark(&r.id, "property", file, out);
        }
        for (id, _) in &file.states {
            mark(id, "state", file, out);
        }
        for t in &file.transitions {
            mark(&t.id, "transition", file, out);
        }
    }

    /// The Reference Typing table (`specs/specodelic.md`) read as a
    /// constraint, plus `ref_resolves` and `every_transition_valid`.
    fn reference_findings(
        file: &File,
        file_ids: &BTreeSet<String>,
        files: &BTreeMap<String, &File>,
        out: &mut Vec<Finding>,
    ) {
        // Typed cells per carrier row: (field, cell, carrier id, carrier
        // schema kind). `supersedes` appears on Constraint and Property
        // rows only (Reference Typing, `supersedes` row); `guard` only on
        // Transitions.
        let mut typed: Vec<(&str, String, String, &'static str)> = Vec::new();
        let mut law_rows: BTreeSet<String> = BTreeSet::new();
        for p in &file.properties {
            if p.kind.as_deref() == Some("law") {
                law_rows.insert(p.id.clone());
            }
        }

        for c in &file.constraints {
            for f in ["traces_to", "supersedes", "satisfies", "observes"] {
                if let Some(v) = c.cells.get(f) {
                    typed.push((f, v.clone(), c.id.clone(), "Constraint"));
                }
            }
        }
        for p in &file.properties {
            for f in ["derives_from", "supersedes"] {
                if let Some(v) = p.cells.get(f) {
                    typed.push((f, v.clone(), p.id.clone(), "Property"));
                }
            }
        }
        for t in &file.transitions {
            if let Some(v) = t.cells.get("guard") {
                typed.push(("guard", v.clone(), t.id.clone(), "Transition"));
            }
        }
        // States' optional `emits` bullet field (Moore output).
        for (id, emits) in &file.states {
            if let Some(v) = emits {
                typed.push(("emits", v.clone(), id.clone(), "State"));
            }
        }

        for (field, cellv, label, carrier_kind) in &typed {
            for target in links(cellv) {
                match resolve(file_ids, files, file, &target) {
                    Resolved::NotARef => {}
                    Resolved::Unresolved => {
                        out.push(finding(
                            "oracle.ref_unresolved",
                            format!(
                                "{}: {label}.{field} -> [[{target}]] does not resolve",
                                file.path
                            ),
                        ));
                    }
                    Resolved::Found(t) => {
                        if typing_ok(field, &t, carrier_kind, law_rows.contains(label)) {
                            continue;
                        }
                        out.push(finding(
                            "oracle.ref_kind_mismatch",
                            format!(
                                "{}: {label}.{field} -> [[{target}]] has kind {}{:?}, violates Reference Typing ({field})",
                                file.path,
                                t.kind,
                                t.attr
                            ),
                        ));
                    }
                }
            }
        }

        // `from`/`to` are bare state ids (not wiki-links) and must resolve
        // to states of the file's own model.
        let state_ids: BTreeSet<&str> = file.states.iter().map(|(id, _)| id.as_str()).collect();
        for t in &file.transitions {
            for field in ["from", "to"] {
                if let Some(v) = t.cells.get(field) {
                    if !v.is_empty() && !state_ids.contains(v.as_str()) {
                        out.push(finding(
                            "oracle.transition_unknown_state",
                            format!(
                                "{}: transition {} {field} -> {v:?} is not a declared state",
                                file.path, t.id
                            ),
                        ));
                    }
                }
            }
        }
    }

    /// The Reference Typing table, one arm per field.
    fn typing_ok(field: &str, t: &Target, carrier_kind: &str, carrier_is_law: bool) -> bool {
        match field {
            "traces_to" => t.kind == "Intent",
            // Constraint — or the same Property when the deriving row is a law.
            "derives_from" => t.kind == "Constraint" || (t.kind == "Property" && carrier_is_law),
            "guard" => {
                // specodelic.md Revision 12 (reconciling the hybrid
                // guard policy): a guard may cite an invariant
                // Constraint — or a State, the "has reached state X"
                // pattern (graph.md extract, refactor.md analyze,
                // orchestrate.md start_lint).
                t.kind == "Constraint" && t.attr.as_deref() == Some("invariant")
                    || t.kind == "State"
            }
            // Same kind as the row it appears on.
            "supersedes" => t.kind == carrier_kind,
            "emits" => t.kind == "Constraint" && t.attr.as_deref() == Some("effect"),
            "satisfies" => t.kind == "Constraint" && t.attr.as_deref() == Some("extension_point"),
            "observes" => t.kind == "Constraint" && t.attr.as_deref() == Some("effect"),
            _ => true, // unknown typed column: typing unconstrained here
        }
    }

    /// Run every oracle check over a repo (a set of parsed files).
    pub fn check_repo(files: &[File]) -> Vec<Finding> {
        let file_ids: BTreeSet<String> = files
            .iter()
            .filter_map(|f| f.fm.as_ref().and_then(|m| m.id.clone()))
            .collect();
        let index: BTreeMap<String, &File> = files
            .iter()
            .filter_map(|f| f.fm.as_ref().and_then(|m| m.id.clone()).map(|id| (id, f)))
            .collect();
        let mut out = Vec::new();
        for f in files {
            // Files with no frontmatter are not spec files (spk's parse
            // skips them entirely) — no rows, no checks. This keeps example
            // tables in prose governance docs (USAGE.md §0) out of scope.
            if f.fm.is_none() {
                continue;
            }
            frontmatter_findings(f, &mut out);
            duplicate_findings(f, &mut out);
        }
        for f in files {
            if f.fm.is_none() {
                continue;
            }
            reference_findings(f, &file_ids, &index, &mut out);
        }
        out
    }
}

// ===========================================================================
// Fixtures
// ===========================================================================

/// The clean single-file baseline.
const CLEAN: &str = r#"---
id: clean
kind: intent
statement: "WHEN a user submits a paid order, THE system SHALL record the order."
---

## Constraints

| id     | kind      | expr                    | traces_to        |
|--------|-----------|-------------------------|------------------|
| rec    | invariant | `orders are permanent`  | [[clean]]        |
| soft   | advisory  | `prefer idempotency`    | [[clean]]        |

## Model

### States
- `open`
- `done`

### Transitions

| id   | from  | to    | guard             |
|------|-------|-------|-------------------|
| ok   | open  | done  | [[clean.rec]]     |

## Properties

| id       | kind | derives_from    | generator     | predicate            |
|----------|------|-----------------|---------------|----------------------|
| rec_hold | unit | [[clean.rec]]   | `orders()`    | `prop_recorder()`    |
| soft_hold | unit | [[clean.soft]]  | `orders()`    | `prop_soft()`        |
"#;

struct AgreedCase {
    name: &'static str,
    files: Vec<(String, String)>,
    /// Oracle rules that MUST fire (subset check) — pins the category
    /// mapping between an oracle rule and the spk surface it mirrors.
    must_fire: &'static [&'static str],
}

/// Agreement fixtures: one single-surface mutation of CLEAN each. For every
/// case, the oracle fires exactly when spk fires (any lint issue/warning or
/// graph finding) — both fire or neither does.
fn agreement_cases() -> Vec<AgreedCase> {
    vec![
        AgreedCase {
            name: "clean_baseline",
            files: vec![("clean.md".into(), CLEAN.into())],
            must_fire: &[],
        },
        AgreedCase {
            name: "dangling_traces_to",
            // traces_to -> nonexistent row in a real file: resolves as a
            // dangling ref for both checkers (not the metasyntactic skip —
            // that is for dotless targets only).
            files: vec![(
                "clean.md".into(),
                CLEAN.replace("[[clean.rec]]   ", "[[clean.nope]]   "),
            )],
            must_fire: &["oracle.ref_unresolved"],
        },
        AgreedCase {
            name: "guard_on_advisory_constraint",
            files: vec![(
                "clean.md".into(),
                CLEAN.replace("| ok   | open  | done  | [[clean.rec]]     |", "| ok   | open  | done  | [[clean.soft]]     |"),
            )],
            must_fire: &["oracle.ref_kind_mismatch"],
        },
        AgreedCase {
            name: "transition_to_unknown_state",
            files: vec![(
                "clean.md".into(),
                CLEAN.replace("| ok   | open  | done  |", "| ok   | open  | void  |"),
            )],
            must_fire: &["oracle.transition_unknown_state"],
        },
        AgreedCase {
            name: "duplicate_constraint_id",
            files: vec![(
                "clean.md".into(),
                CLEAN.replace("| soft   | advisory", "| rec    | advisory"),
            )],
            must_fire: &["oracle.duplicate_row_id"],
        },
        AgreedCase {
            name: "missing_statement",
            files: vec![(
                "clean.md".into(),
                CLEAN.replace(
                    "statement: \"WHEN a user submits a paid order, THE system SHALL record the order.\"\n",
                    "",
                ),
            )],
            must_fire: &["oracle.frontmatter_statement"],
        },
        AgreedCase {
            name: "id_filename_mismatch",
            files: vec![("other.md".into(), CLEAN.into())],
            must_fire: &["oracle.frontmatter_id_matches_file"],
        },
    ]
}

// ===========================================================================
// Tests
// ===========================================================================

/// Oracle parser sanity: the independent parser must extract the same
/// structure a human reads off the fixture.
#[test]
fn oracle_parser_extracts_layers() {
    let f = oracle::parse_file("clean.md", CLEAN);
    let fm = f.fm.expect("frontmatter parsed");
    assert_eq!(fm.id.as_deref(), Some("clean"));
    assert_eq!(fm.kind.as_deref(), Some("intent"));
    assert_eq!(f.constraints.len(), 2);
    assert_eq!(f.constraints[0].id, "rec");
    assert_eq!(f.constraints[0].kind.as_deref(), Some("invariant"));
    assert_eq!(f.states, vec![("open".into(), None), ("done".into(), None)]);
    assert_eq!(f.transitions.len(), 1);
    assert_eq!(
        f.transitions[0].cells.get("guard").map(String::as_str),
        Some("[[clean.rec]]")
    );
    assert_eq!(f.properties.len(), 2);
    assert_eq!(
        f.properties[0]
            .cells
            .get("derives_from")
            .map(String::as_str),
        Some("[[clean.rec]]")
    );
}

/// Oracle parser sanity: `emits:` on a state bullet is captured.
#[test]
fn oracle_parser_extracts_state_emits() {
    let text = CLEAN.replace("- `done`\n", "- `done` (emits: `[[clean.rec]]`)\n");
    let f = oracle::parse_file("clean.md", &text);
    assert_eq!(
        f.states,
        vec![
            ("open".into(), None),
            ("done".into(), Some("[[clean.rec]]".into()))
        ]
    );
}

/// Agreement cross-check: on each fixture, oracle and spk agree — both fire
/// or both stay clean — and the pinned oracle rules do fire.
#[test]
fn oracle_agrees_with_spk_on_fixtures() {
    for case in agreement_cases() {
        let mut specs = Vec::new();
        let mut spk_rejected_parse = false;
        for (path, text) in &case.files {
            match parse_str(text).map(|mut s| {
                s.path = Some(PathBuf::from(path));
                s
            }) {
                Ok(s) => specs.push(s),
                // spk rejects at parse (e.g. missing statement is a parse
                // error, not a lint issue) — that IS the spk surface firing.
                Err(_) => spk_rejected_parse = true,
            }
        }
        let (issues_n, warnings_n, g) = if spk_rejected_parse {
            (1, 0, graph::build(&[]))
        } else {
            let checklists: Vec<Checklist> = Vec::new();
            let report = lint::lint_all(&specs, &checklists);
            let g = graph::build(&specs);
            (report.issues.len(), report.warnings.len(), g)
        };
        let spk_fires = spk_rejected_parse
            || issues_n > 0
            || warnings_n > 0
            || !g.dangling.is_empty()
            || !g.violations.is_empty()
            || !g.supersedes_cycles.is_empty();

        let files: Vec<oracle::File> = case
            .files
            .iter()
            .map(|(p, t)| oracle::parse_file(p, t))
            .collect();
        let findings = oracle::check_repo(&files);
        let oracle_fires = !findings.is_empty();

        assert_eq!(
            oracle_fires, spk_fires,
            "case {}: oracle vs spk disagreement — oracle: {findings:?}, spk issues: {issues_n}, warnings: {warnings_n}, graph: dangling={:?} typing={:?} cycles={:?}",
            case.name, g.dangling, g.violations, g.supersedes_cycles
        );
        for rule in case.must_fire {
            assert!(
                findings.iter().any(|f| f.rule == *rule),
                "case {}: expected oracle rule {rule} to fire; got {findings:?}",
                case.name
            );
        }
    }
}

/// Corpus gate: the repo's own corpus must pass the reference oracle with
/// zero findings.
#[test]
fn corpus_passes_reference_oracle() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    collect_corpus_specs(&manifest.join("specs"), &mut files);
    collect_openspec_capability_specs(&manifest.join("openspec/specs"), &mut files);
    assert!(
        files.len() > 20,
        "corpus walk found only {} files — walker broken?",
        files.len()
    );

    let parsed: Vec<oracle::File> = files
        .iter()
        .map(|(p, t)| oracle::parse_file(p, t))
        .collect();
    let findings = oracle::check_repo(&parsed);
    assert!(
        findings.is_empty(),
        "reference oracle found corpus violations:\n{}",
        findings
            .iter()
            .map(|f| format!("  {} — {}\n", f.rule, f.detail))
            .collect::<String>()
    );
}

/// Checklist manifests are exempt (external-completeness artifact, not a
/// four-layer spec) — the oracle must not report findings for them.
#[test]
fn checklist_files_are_exempt() {
    let text = "# A checklist\n\n- [ ] some item\n";
    let f = oracle::parse_file("x.checklist.md", text);
    assert!(f.fm.is_none());
    assert!(oracle::check_repo(&[f]).is_empty());
}

/// Spelled-out single-surface cases (one mutation each, off the matrix's
/// CLEAN baseline) so each oracle rule has a dedicated red test — these are
/// the rules the agreement table above only samples.
#[test]
fn oracle_rules_fire_individually() {
    let mk = |name: &str, text: String| oracle::parse_file(name, &text);

    // id_matches_file: id != stem(path) with '-' -> '.'.
    let f = vec![mk("other.md", CLEAN.to_string())];
    assert!(
        oracle::check_repo(&f)
            .iter()
            .any(|x| x.rule == "oracle.frontmatter_id_matches_file")
    );

    // ref_resolves: dotted target naming no file.
    let text = CLEAN.replace("[[clean.rec]]     |", "[[clean.nope.here]] |");
    let f = vec![mk("clean.md", text)];
    assert!(
        oracle::check_repo(&f)
            .iter()
            .any(|x| x.rule == "oracle.ref_unresolved")
    );

    // ref_kind_compatible: traces_to -> Constraint (Intent only).
    let text = CLEAN.replace(
        "| rec    | invariant | `orders are permanent`  | [[clean]]",
        "| rec    | invariant | `orders are permanent`  | [[clean.rec]]",
    );
    let f = vec![mk("clean.md", text)];
    assert!(
        oracle::check_repo(&f)
            .iter()
            .any(|x| x.rule == "oracle.ref_kind_mismatch")
    );

    // ref_kind_compatible: guard -> advisory constraint (invariant only).
    let text = CLEAN.replace(
        "| ok   | open  | done  | [[clean.rec]]     |",
        "| ok   | open  | done  | [[clean.soft]]     |",
    );
    let f = vec![mk("clean.md", text)];
    assert!(
        oracle::check_repo(&f)
            .iter()
            .any(|x| x.rule == "oracle.ref_kind_mismatch")
    );

    // ref_kind_compatible: emits -> invariant constraint (effect only).
    let text = CLEAN.replace("- `done`\n", "- `done` (emits: `[[clean.rec]]`)\n");
    let f = vec![mk("clean.md", text)];
    assert!(
        oracle::check_repo(&f)
            .iter()
            .any(|x| x.rule == "oracle.ref_kind_mismatch")
    );

    // ref_kind_compatible: unit Property -> State (Constraint, or law ->
    // law-Property; a State target is typed wrong for derives_from).
    let text = CLEAN.replace(
        "| rec_hold | unit | [[clean.rec]]",
        "| rec_hold | unit | [[clean.done]]",
    );
    let f = vec![mk("clean.md", text)];
    assert!(
        oracle::check_repo(&f)
            .iter()
            .any(|x| x.rule == "oracle.ref_kind_mismatch")
    );

    // every_transition_valid: `to` names no declared state.
    let text = CLEAN.replace("| ok   | open  | done  |", "| ok   | open  | void  |");
    let f = vec![mk("clean.md", text)];
    assert!(
        oracle::check_repo(&f)
            .iter()
            .any(|x| x.rule == "oracle.transition_unknown_state")
    );

    // duplicate row id within the file.
    let text = CLEAN.replace("| soft   | advisory", "| rec    | advisory");
    let f = vec![mk("clean.md", text)];
    assert!(
        oracle::check_repo(&f)
            .iter()
            .any(|x| x.rule == "oracle.duplicate_row_id")
    );

    // missing statement.
    let text = CLEAN.replace(
        "statement: \"WHEN a user submits a paid order, THE system SHALL record the order.\"\n",
        "",
    );
    let f = vec![mk("clean.md", text)];
    assert!(
        oracle::check_repo(&f)
            .iter()
            .any(|x| x.rule == "oracle.frontmatter_statement")
    );

    // malformed id (uppercase).
    let text = CLEAN.replace("id: clean", "id: Clean");
    let f = vec![mk("clean.md", text)];
    assert!(
        oracle::check_repo(&f)
            .iter()
            .any(|x| x.rule == "oracle.frontmatter_id")
    );

    // wrong kind.
    let text = CLEAN.replace("kind: intent", "kind: feature");
    let f = vec![mk("clean.md", text)];
    assert!(
        oracle::check_repo(&f)
            .iter()
            .any(|x| x.rule == "oracle.frontmatter_kind")
    );
}

// ---------------------------------------------------------------------------
// Corpus walking
// ---------------------------------------------------------------------------

fn collect_corpus_specs(dir: &Path, out: &mut Vec<(String, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_corpus_specs(&p, out);
        } else if p.extension().is_some_and(|x| x == "md") {
            let name = p
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            // Non-spec files (no frontmatter) are exempt by construction —
            // the oracle skips files without frontmatter. Checklists too.
            if name.ends_with(".checklist.md") {
                continue;
            }
            let text = std::fs::read_to_string(&p).unwrap_or_default();
            let rel = p
                .strip_prefix(Path::new(env!("CARGO_MANIFEST_DIR")))
                .unwrap_or(&p)
                .to_string_lossy()
                .into_owned();
            out.push((rel, text));
        }
    }
}

fn collect_openspec_capability_specs(dir: &Path, out: &mut Vec<(String, String)>) {
    // openspec/specs/<capability>/spec.md — exactly one level deep.
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let spec = e.path().join("spec.md");
        if spec.is_file() {
            let text = std::fs::read_to_string(&spec).unwrap_or_default();
            let rel = spec
                .strip_prefix(Path::new(env!("CARGO_MANIFEST_DIR")))
                .unwrap_or(&spec)
                .to_string_lossy()
                .into_owned();
            out.push((rel, text));
        }
    }
}
