//! External-checklist manifests (specs/linter-external_completeness.md).
//!
//! Purpose: parse the dedicated `*.checklist.md` artifact — the manifest
//! format decided in mp1 row 10: a checklist is an artifact OUTSIDE the
//! four-layer spec shape (no frontmatter, exempt like AGENTS.md), so the
//! Reference Typing table and `𝒦` stay fixed. Responsibilities: detect
//! checklist paths, parse the flat `## Items` list and the `## Mapping`
//! table (columns exactly `item`/`status`/`mapped_ids`/`rationale`), and
//! collect well-formedness defects for the linter. Rationale: a lenient
//! parse plus strict lint keeps `checklist_well_formed` testable — a
//! manifest that fails to parse must still surface as findings, never
//! vanish into a silent checklist.

use std::path::{Path, PathBuf};

/// A parsed checklist manifest. Lenient by design: structural problems
/// ride [`Checklist::defects`] (for the `checklist_well_formed` rule to
/// emit) instead of failing the parse — the parse itself never fails.
#[derive(Debug, Clone, Default)]
pub struct Checklist {
    /// Where the manifest lives — the `Issue.file` of its findings.
    pub path: PathBuf,
    /// Well-formed `- **<id>**: <description>` items, in file order.
    pub items: Vec<ChecklistItem>,
    /// Mapping rows, in file order.
    pub mapping: Vec<MappingRow>,
    /// Well-formedness defects: nested items, missing ids, duplicate
    /// item ids, bad id charsets, missing sections, a mapping table
    /// with wrong columns or cell counts, and mapping rows naming an
    /// undeclared item. Emited verbatim as `checklist_well_formed`
    /// findings.
    pub defects: Vec<String>,
}

/// One checklist item: a stable id plus a plain-text description.
#[derive(Debug, Clone)]
pub struct ChecklistItem {
    pub id: String,
    pub description: String,
}

/// One mapping row: which checklist item, what claim, what backs it.
#[derive(Debug, Clone)]
pub struct MappingRow {
    /// The checklist item id this row claims (`m.item`).
    pub item: String,
    /// `covered` or `waived` — anything else is `every_item_accounted`'s
    /// beat (the constraint text fixes the status set).
    pub status: String,
    /// Constraint/property ids the claim rests on (`m.mapped_ids`);
    /// empty for waivers.
    pub mapped_ids: Vec<String>,
    /// Waiver rationale prose (`m.rationale`); empty for covered rows.
    pub rationale: String,
}

/// A path declares a checklist when its file stem ends in `.checklist`
/// (e.g. `release.checklist.md`) — presence of the file IS the
/// declaration (a repo with none has nothing external to be incomplete
/// relative to: `not_applicable`, not a vacuous pass).
pub fn is_checklist_path(path: &Path) -> bool {
    path.file_stem()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.ends_with(".checklist"))
}

/// The exact mapping-table columns (mp1 row 10 decision of record) —
/// unknown or missing columns are rejected, never guessed.
const MAPPING_COLUMNS: [&str; 4] = ["item", "status", "mapped_ids", "rationale"];

/// An item id is a row id: alphanumeric segments joined by `.`/`_`/`-`.
fn valid_item_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// Trim a cell and strip ONE layer of surrounding backticks, the same
/// leniency the mapping-table header row gets.
fn strip_backticks(cell: &str) -> String {
    let cell = cell.trim();
    cell.strip_prefix('`')
        .and_then(|c| c.strip_suffix('`'))
        .unwrap_or(cell)
        .trim()
        .to_string()
}

/// Parse one `## Items` bullet. Returns `Some(defect)` when the line is
/// a (nested or malformed) item attempt, `None` when it isn't an item
/// line at all (blank, prose) and should be ignored.
fn parse_item_line(
    line: &str,
    trimmed: &str,
    items: &mut Vec<ChecklistItem>,
    seen: &mut std::collections::BTreeSet<String>,
) -> Option<String> {
    let is_attempt = trimmed.starts_with("- ");
    if !is_attempt {
        return None;
    }
    // Flat list: an indented bullet under another item is a nested item
    // (the checker spec's `a_nested_item` generator).
    if line.starts_with(char::is_whitespace) {
        return Some(format!(
            "nested checklist item (items must be a flat list): {trimmed}"
        ));
    }
    let rest = trimmed.strip_prefix("- ").unwrap_or(trimmed);
    let Some(body) = rest.strip_prefix("**").and_then(|b| b.split_once("**")) else {
        return Some(format!(
            "malformed item — expected `- **<id>**: <description>`, got: {trimmed}"
        ));
    };
    let (id, after) = body;
    let Some(description) = after.strip_prefix(':') else {
        return Some(format!(
            "malformed item — expected `- **<id>**: <description>`, got: {trimmed}"
        ));
    };
    let description = description.trim();
    if !valid_item_id(id) {
        return Some(format!(
            "item id `{id}` is not a stable row id (alphanumeric segments joined by `.`, `_`, `-`)"
        ));
    }
    if description.is_empty() {
        return Some(format!("item `{id}` is missing its plain-text description"));
    }
    if !seen.insert(id.to_string()) {
        return Some(format!(
            "duplicate checklist item id `{id}` — item ids must be stable and unique"
        ));
    }
    items.push(ChecklistItem {
        id: id.to_string(),
        description: description.to_string(),
    });
    None
}

/// Split a `mapped_ids` cell into ids: comma-separated, each optionally
/// wrapped in `[[...]]` (the corpus link spelling — strip, don't parse).
fn split_mapped_ids(cell: &str) -> Vec<String> {
    cell.split(',')
        .map(|id| {
            let id = id.trim();
            id.strip_prefix("[[")
                .and_then(|i| i.strip_suffix("]]"))
                .unwrap_or(id)
                .trim()
                .to_string()
        })
        .filter(|id| !id.is_empty())
        .collect()
}

/// Parse a checklist manifest.
pub fn parse_str(path: PathBuf, text: &str) -> Checklist {
    // Tolerate a UTF-8 BOM (Rule-of-5 EDGE-001): otherwise the first
    // `## ` header goes unrecognized and the failure misattributes to a
    // misleading orphan-row finding instead of naming the BOM.
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut cl = Checklist {
        path,
        ..Default::default()
    };
    let mut section = "";
    // Section-visibility flags: `section` alone holds only the LAST
    // header seen, so a manifest with `## Mapping` but no `## Items`
    // (or vice versa) would otherwise never report its missing half —
    // a false green on the exact failure class this checker exists for
    // (Rule-of-5 CORR-001, specodelic-b15).
    let mut saw_items = false;
    let mut saw_mapping = false;
    let mut seen_items = std::collections::BTreeSet::new();
    // Mapping-table state: the header row sets the columns; the
    // separator row is skipped; data rows follow.
    let mut in_mapping_table = false;
    for line in text.lines() {
        let trimmed = line.trim();
        // Deeper headers (`### Note`, `#### …`) are prose inside a
        // section — never section switches (Rule-of-5 EDGE-003: a
        // `### ` line inside the Mapping region must not reset parsing).
        if trimmed.starts_with("###") {
            continue;
        }
        if let Some(header) = trimmed.strip_prefix("## ") {
            section = match header.trim() {
                "Items" => {
                    saw_items = true;
                    "items"
                }
                "Mapping" => {
                    saw_mapping = true;
                    "mapping"
                }
                _ => "",
            };
            in_mapping_table = false;
            continue;
        }
        match section {
            "items" => {
                if let Some(defect) = parse_item_line(line, trimmed, &mut cl.items, &mut seen_items)
                {
                    cl.defects.push(defect);
                }
            }
            "mapping" => {
                let Some(row) = trimmed.strip_prefix('|') else {
                    continue;
                };
                let cells: Vec<String> = row
                    .strip_suffix('|')
                    .unwrap_or(row)
                    .split('|')
                    .map(|c| c.trim().to_string())
                    .collect();
                if !in_mapping_table {
                    // The header row pins the table shape.
                    let names: Vec<String> = cells
                        .iter()
                        .map(|c| c.trim_start_matches('`').trim_end_matches('`').to_string())
                        .collect();
                    if names != MAPPING_COLUMNS {
                        cl.defects.push(format!(
                            "the mapping table columns must be exactly item/status/mapped_ids/rationale — got: {cells:?}"
                        ));
                    }
                    in_mapping_table = true;
                    continue;
                }
                // The `---` separator row.
                if cells.iter().all(|c| c.chars().all(|ch| ch == '-')) {
                    continue;
                }
                if cells.len() != MAPPING_COLUMNS.len() {
                    cl.defects.push(format!(
                        "mapping row must have four columns, got {}: {trimmed}",
                        cells.len()
                    ));
                    continue;
                }
                cl.mapping.push(MappingRow {
                    // Status/item cells tolerate surrounding backticks,
                    // same leniency the header row already gets.
                    item: strip_backticks(&cells[0]),
                    status: strip_backticks(&cells[1]),
                    mapped_ids: split_mapped_ids(&cells[2]),
                    rationale: cells[3].clone(),
                });
            }
            _ => {}
        }
    }
    // Missing sections are defects whenever they were never seen —
    // regardless of what the file ends inside.
    if !saw_items {
        cl.defects
            .push("checklist has no `## Items` section".to_string());
    }
    if !saw_mapping {
        cl.defects
            .push("checklist has no `## Mapping` section".to_string());
    }
    // An empty manifest is not a checklist: sections seen, zero items
    // declared — nothing can be consulted against it, so a silent pass
    // would be a false green (Rule-of-5 EDGE-002).
    if saw_items && cl.items.is_empty() {
        cl.defects
            .push("checklist declares no items — a checklist nothing can be consulted against is not a checklist".to_string());
    }
    // A mapping row claiming an undeclared item is a structural defect:
    // the manifest references machinery it never listed (checked at the
    // end, when the declared item set is complete).
    for row in &cl.mapping {
        if !cl.items.iter().any(|i| i.id == row.item) {
            cl.defects.push(format!(
                "mapping row references undeclared checklist item `{}`",
                row.item
            ));
        }
    }
    cl
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Checklist {
        parse_str(PathBuf::from("release.checklist.md"), text)
    }

    fn well_formed() -> &'static str {
        "# Release checklist\n\
         \n## Items\n\
         \n- **auth.session**: logged-in sessions expire\n\
         - **auth.token**: tokens are single-use\n\
         \n## Mapping\n\
         \n| item | status | mapped_ids | rationale |\n\
         |------|--------|------------|-----------|\n\
         | auth.session | covered | [[auth.session.expire]] | |\n\
         | auth.token | waived | | single-use tokens live outside this repo |\n"
    }

    #[test]
    fn detects_checklist_paths() {
        assert!(is_checklist_path(Path::new("release.checklist.md")));
        assert!(is_checklist_path(Path::new("a/b/ship.checklist.md")));
        assert!(!is_checklist_path(Path::new("checklist.md")));
        assert!(!is_checklist_path(Path::new("spec.md")));
        assert!(!is_checklist_path(Path::new("specodelic.md")));
    }

    #[test]
    fn parses_well_formed_manifest() {
        let cl = parse(well_formed());
        assert!(cl.defects.is_empty(), "no defects: {:?}", cl.defects);
        assert_eq!(cl.items.len(), 2);
        assert_eq!(cl.items[0].id, "auth.session");
        assert_eq!(cl.items[0].description, "logged-in sessions expire");
        assert_eq!(cl.mapping.len(), 2);
        assert_eq!(cl.mapping[0].item, "auth.session");
        assert_eq!(cl.mapping[0].status, "covered");
        assert_eq!(cl.mapping[0].mapped_ids, vec!["auth.session.expire"]);
        assert_eq!(cl.mapping[1].status, "waived");
        assert_eq!(
            cl.mapping[1].rationale,
            "single-use tokens live outside this repo"
        );
    }

    #[test]
    fn mapped_ids_accept_bare_and_wikilinked_comma_lists() {
        let cl = parse(
            "## Items\n\n- **a.b**: x\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n\
             |------|--------|------------|-----------|\n\
             | a.b | covered | [[x.c1]], y.p2 | |\n",
        );
        assert!(cl.defects.is_empty(), "{:?}", cl.defects);
        assert_eq!(cl.mapping[0].mapped_ids, vec!["x.c1", "y.p2"]);
    }

    #[test]
    fn nested_item_is_a_defect() {
        let cl = parse(
            "## Items\n\n- **a.b**: x\n  - **a.c**: nested\n\n## Mapping\n\n\
             | item | status | mapped_ids | rationale |\n\
             |------|--------|------------|-----------|\n\
             | a.b | covered | [[x.c1]] | |\n\
             | a.c | waived | | because |\n",
        );
        assert!(
            cl.defects.iter().any(|d| d.contains("nested")),
            "{:?}",
            cl.defects
        );
        // The malformed item never becomes an accounted item.
        assert_eq!(cl.items.len(), 1);
    }

    #[test]
    fn item_missing_its_id_is_a_defect() {
        let cl = parse(
            "## Items\n\n- plain bullet without an id\n\n## Mapping\n\n\
             | item | status | mapped_ids | rationale |\n\
             |------|--------|------------|-----------|\n\
             | a.b | covered | [[x.c1]] | |\n",
        );
        assert!(
            cl.defects
                .iter()
                .any(|d| d.contains("expected `- **<id>**: <description>`")),
            "{:?}",
            cl.defects
        );
    }

    #[test]
    fn item_missing_its_description_is_a_defect() {
        let cl = parse(
            "## Items\n\n- **a.b**:\n\n## Mapping\n\n\
             | item | status | mapped_ids | rationale |\n\
             |------|--------|------------|-----------|\n\
             | a.b | covered | [[x.c1]] | |\n",
        );
        assert!(
            cl.defects
                .iter()
                .any(|d| d.contains("a.b") && d.contains("description")),
            "{:?}",
            cl.defects
        );
    }

    #[test]
    fn duplicate_item_id_is_a_defect() {
        let cl = parse(
            "## Items\n\n- **a.b**: x\n- **a.b**: y\n\n## Mapping\n\n\
             | item | status | mapped_ids | rationale |\n\
             |------|--------|------------|-----------|\n\
             | a.b | covered | [[x.c1]] | |\n",
        );
        assert!(
            cl.defects.iter().any(|d| d.contains("duplicate")),
            "{:?}",
            cl.defects
        );
    }

    #[test]
    fn missing_sections_are_defects() {
        let cl = parse("# just a title\n");
        assert!(
            cl.defects
                .iter()
                .any(|d| d.contains("no `## Items` section")),
            "{:?}",
            cl.defects
        );
        assert!(
            cl.defects
                .iter()
                .any(|d| d.contains("no `## Mapping` section")),
            "{:?}",
            cl.defects
        );
    }

    /// Rule-of-5 CORR-001 regression: a manifest that ends inside a
    /// named section must still report its missing half — the old
    /// last-section guard silently passed a Mapping-only manifest.
    #[test]
    fn partial_section_manifests_report_their_missing_half() {
        let cl = parse("## Mapping\n");
        assert!(
            cl.defects
                .iter()
                .any(|d| d.contains("no `## Items` section")),
            "Mapping-only manifest must name its missing Items: {:?}",
            cl.defects
        );
        let cl = parse("## Items\n\n- **a.b**: x\n");
        assert!(
            cl.defects
                .iter()
                .any(|d| d.contains("no `## Mapping` section")),
            "Items-only manifest must name its missing Mapping: {:?}",
            cl.defects
        );
    }

    #[test]
    fn wrong_mapping_columns_are_a_defect() {
        let cl = parse(
            "## Items\n\n- **a.b**: x\n\n## Mapping\n\n\
             | item | status | ids | rationale |\n\
             |------|--------|-----|-----------|\n\
             | a.b | covered | [[x.c1]] | |\n",
        );
        assert!(
            cl.defects
                .iter()
                .any(|d| d.contains("exactly item/status/mapped_ids/rationale")),
            "{:?}",
            cl.defects
        );
    }

    #[test]
    fn mapping_row_with_wrong_cell_count_is_a_defect() {
        let cl = parse(
            "## Items\n\n- **a.b**: x\n\n## Mapping\n\n\
             | item | status | mapped_ids | rationale |\n\
             |------|--------|------------|-----------|\n\
             | a.b | covered | [[x.c1]] |\n",
        );
        assert!(
            cl.defects
                .iter()
                .any(|d| d.contains("four columns") && d.contains("a.b")),
            "{:?}",
            cl.defects
        );
    }

    #[test]
    fn mapping_row_for_undeclared_item_is_a_defect() {
        let cl = parse(
            "## Items\n\n- **a.b**: x\n\n## Mapping\n\n\
             | item | status | mapped_ids | rationale |\n\
             |------|--------|------------|-----------|\n\
             | ghost.item | covered | [[x.c1]] | |\n",
        );
        assert!(
            cl.defects
                .iter()
                .any(|d| d.contains("undeclared checklist item") && d.contains("ghost.item")),
            "{:?}",
            cl.defects
        );
    }

    /// Rule-of-5 EDGE-001: a BOM must not blind the parser — the
    /// manifest parses clean instead of misattributing to an orphan row.
    #[test]
    fn bom_is_tolerated() {
        let cl = parse(
            "\u{feff}## Items\n\n- **a.b**: x\n\n## Mapping\n\n\
                        | item | status | mapped_ids | rationale |\n\
                        |------|--------|------------|-----------|\n\
                        | a.b | waived | | because |\n",
        );
        assert!(cl.defects.is_empty(), "{:?}", cl.defects);
        assert_eq!(cl.items.len(), 1);
        assert_eq!(cl.mapping[0].status, "waived");
    }

    /// Rule-of-5 EDGE-002: sections present but zero items declared —
    /// a silent pass would be a false green.
    #[test]
    fn empty_manifest_declaring_no_items_is_a_defect() {
        let cl = parse("## Items\n\n## Mapping\n\n");
        assert!(
            cl.defects.iter().any(|d| d.contains("declares no items")),
            "{:?}",
            cl.defects
        );
    }

    /// Rule-of-5 EDGE-003: a `### ` line inside the Mapping region is
    /// prose, never a section switch — rows after it still parse.
    #[test]
    fn deeper_header_inside_mapping_does_not_reset_parsing() {
        let cl = parse(
            "## Items\n\n- **a.b**: x\n\n## Mapping\n\n\
             | item | status | mapped_ids | rationale |\n\
             |------|--------|------------|-----------|\n\
             ### note: statuses reviewed quarterly\n\
             | a.b | covered | [[x.c1]] | |\n",
        );
        assert!(cl.defects.is_empty(), "{:?}", cl.defects);
        assert_eq!(cl.mapping.len(), 1);
    }

    /// Rule-of-5 CLAR-002: status cells tolerate surrounding backticks,
    /// the same leniency the header row gets.
    #[test]
    fn backticked_status_cells_are_accepted() {
        let cl = parse(
            "## Items\n\n- **a.b**: x\n\n## Mapping\n\n\
             | item | status | mapped_ids | rationale |\n\
             |------|--------|------------|-----------|\n\
             | a.b | `waived` | | because |\n",
        );
        assert!(cl.defects.is_empty(), "{:?}", cl.defects);
        assert_eq!(cl.mapping[0].status, "waived");
    }
}
