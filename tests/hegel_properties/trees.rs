//! Merge-tree scenario constructions for the Hegel property suite
//! (specodelic-jnnf). Branch tips are `(rel path, text)` sets built from
//! corpus models with targeted, model-level mutations, then rendered —
//! never textual surgery on structure — so every tip stays parse-valid
//! by construction and lint-clean unless a scenario deliberately injects
//! a defect (dangling `[[ghost]]`, collision, cross-file divergence).

use super::corpus::{ModelFile, build_file, render};

pub type Tree = Vec<(String, String)>;

/// Render a model under a given rel path.
pub fn placed(path: &str, m: &ModelFile) -> (String, String) {
    (path.to_string(), render(m))
}

fn find_mut<'a>(path: &str, tree: &'a mut Tree) -> &'a mut String {
    tree.iter_mut()
        .find(|(p, _)| p == path)
        .map(|(_, t)| t)
        .unwrap_or_else(|| panic!("scenario bug: {path} absent from tree"))
}

/// Edit one constraint row's `expr` cell textually (same length class,
/// different words) — a file-level edit that preserves the parse and the
/// lint (expr cells are free prose in backticks).
pub fn edit_expr(tree: &mut Tree, path: &str, old_expr: &str, new_expr: &str) {
    let text = find_mut(path, tree);
    let needle = format!("`{old_expr}`");
    let replacement = format!("`{new_expr}`");
    assert!(
        text.contains(&needle),
        "scenario bug: {needle} absent from {path}"
    );
    *text = text.replace(&needle, &replacement);
}

/// Delete a file from the tree.
pub fn delete_file(tree: &mut Tree, path: &str) {
    let before = tree.len();
    tree.retain(|(p, _)| p != path);
    assert_eq!(tree.len(), before - 1, "scenario bug: {path} not deleted");
}

/// Add a fresh self-contained file (no cross-file references).
pub fn add_self_file(tree: &mut Tree, path: &str, intent: &str, seed: u64) {
    let m = build_file(intent, seed, None);
    tree.push(placed(path, &m));
}

/// Inject a dangling `[[ghost]]` reference into the first listed
/// constraint row's `traces_to` cell: its first `[[…]]` occurrence — own
/// or cross-file — becomes `[[ghost]]`.
pub fn inject_dangling_traces(tree: &mut Tree, path: &str, constraint_locals: &[&str]) {
    let text = find_mut(path, tree);
    for local in constraint_locals {
        let needle = format!("| {local} ");
        let Some(line_pos) = text.find(&needle) else {
            continue;
        };
        let line_end = text[line_pos..]
            .find('\n')
            .map(|e| line_pos + e)
            .unwrap_or(text.len());
        let line = &text[line_pos..line_end];
        let Some(open) = line.find("[[") else {
            continue;
        };
        let Some(close_rel) = line[open..].find("]]") else {
            continue;
        };
        let close = open + close_rel + 2;
        let patched = format!("{}[[ghost]]{}", &line[..open], &line[close..]);
        *text = format!("{}{}{}", &text[..line_pos], patched, &text[line_end..]);
        return;
    }
    panic!("scenario bug: no link-bearing row found in {path}");
}
