//! specodelic-mlc: the outbound-leaf carve-out has ONE normative home
//! (`openspec/specs/specodelic/spec.md`, `### Reference Typing`). Every
//! other spec file may mention it at most once, and that mention must be
//! a pointer naming `specodelic.md`, never a restatement.
//!
//! `linter-failure_shape` is exempt: its "outbound transitions" is the
//! Model-edge sense (a state with no outbound transition), not a typed
//! reference carve-out.

use std::path::Path;

const EXEMPT: &[&str] = &["linter-failure_shape"];

/// The rule's restatement vocabulary. Removing the word `outbound` is not
/// enough: a file that keeps one of these clauses is still restating the
/// carve-out instead of pointing at it.
const RESTATEMENT_PHRASES: &[&str] = &[
    "no reachability path",
    "no acyclic edge set",
    "never reachability path",
];

/// Count a line as a carve-out mention only when `outbound` stands alone:
/// identifier spellings (`lineage_edges_outbound`, `uses_is_outbound_leaf`,
/// `[[...outbound]]`) are names, not restatements.
fn mentions_carveout(line: &str) -> bool {
    let lower = line.to_lowercase();
    let is_ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
    for (i, _) in lower.match_indices("outbound") {
        let before = lower[..i].chars().next_back();
        let after = lower[i + "outbound".len()..].chars().next();
        if !before.is_some_and(is_ident) && !after.is_some_and(is_ident) {
            return true;
        }
    }
    false
}

#[test]
fn outbound_leaf_rule_has_a_single_normative_home() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("openspec/specs");
    let canonical = root.join("specodelic").join("spec.md");
    let text = std::fs::read_to_string(&canonical).expect("the format spec exists");
    assert!(
        text.contains("### Reference Typing"),
        "the normative section exists"
    );
    assert!(
        text.contains("Outbound-leaf carve-out (Revision 22)"),
        "the single normative carve-out statement is present"
    );

    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&root).expect("the corpus exists") {
        let entry = entry.expect("readable directory entry");
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        if name == "specodelic" || EXEMPT.contains(&name.as_str()) {
            continue;
        }
        let spec = dir.join("spec.md");
        let Ok(body) = std::fs::read_to_string(&spec) else {
            continue;
        };
        let lower = body.to_lowercase();
        for phrase in RESTATEMENT_PHRASES {
            if lower.contains(phrase) {
                offenders.push(format!("{name}: still restates the carve-out (`{phrase}`)"));
            }
        }
        let lines: Vec<&str> = body
            .lines()
            .filter(|line| mentions_carveout(line))
            .collect();
        if lines.len() > 1 {
            offenders.push(format!(
                "{name}: {} lines mention the carve-out; keep one pointer",
                lines.len()
            ));
        } else if let Some(line) = lines.first()
            && !line.contains("specodelic")
        {
            offenders.push(format!(
                "{name}: the single mention is a restatement, not a pointer to specodelic.md"
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "outbound-leaf restatements outside specodelic.md:\n{}",
        offenders.join("\n")
    );
}
