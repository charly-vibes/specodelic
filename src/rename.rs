//! The atomic rename operation (`specs/rename.md`).
//!
//! Purpose: construct `I'` from `(I, old_id, new_id)` — the concrete
//! operation the corpus' `rename_naturality` laws are about.
//! Responsibilities: locate the defining row (Intent frontmatter or a
//! table row), validate the request (no collision, filename law for
//! Intent renames), rewrite wiki-link syntax and structured id cells
//! in memory, and verify the would-be result against the linters
//! (`linter.referential_integrity` via zero dangling + re-parse) BEFORE
//! any byte is written. Rationale: `atomic_operation` is the point —
//! the caller receives the full set of writes (and one removal for an
//! Intent rename) and applies it only after every check has passed, so
//! a failed rename leaves the repo byte-identical by construction, and
//! prose is untouched because only `[[…]]` link syntax and exact id
//! cells/bullets are ever rewritten (`prose_untouched_by_rename`).

use std::path::PathBuf;

use crate::graph;
use crate::spec::{Spec, parse_str};

/// Everything the caller must write to make the rename real, produced
/// only after validation and verification succeeded.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct RenameOutcome {
    pub old_id: String,
    pub new_id: String,
    /// (path, full new contents) for every file the rename touches.
    pub writes: Vec<(PathBuf, String)>,
    /// The old file to remove when a file's own Intent id was renamed
    /// (`writes` carries the new path per the `-` ⇔ `.` naming law).
    pub remove: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenameError {
    /// `old_id` matches no row or Intent in the corpus.
    UnknownId(String),
    /// `new_id` already exists — `rename.new_id_available` violated;
    /// the payload names the holder.
    Collision { new_id: String, holder: String },
    /// `new_id` cannot be an id (empty, whitespace, link/table syntax).
    InvalidNewId(String),
    /// `old_id` matches more than one defining row (corpus already
    /// violates `unique_across_repo` — refusing to guess).
    Ambiguous(String),
    /// The post-rename repo fails the verify gate (re-parse or
    /// `linter.referential_integrity`): nothing has been written.
    VerifyFailed { details: Vec<String> },
}

/// Apply the rename purely in memory: read-only over the input files,
/// output a write set that leaves the repo fully renamed and lint-clean.
pub fn run(
    files: &[(PathBuf, String)],
    old_id: &str,
    new_id: &str,
) -> Result<RenameOutcome, RenameError> {
    validate_shape(new_id)?;
    if old_id == new_id {
        // rename_naturality's identity law: rename(I, a, a) == I.
        return Ok(RenameOutcome {
            old_id: old_id.into(),
            new_id: new_id.into(),
            writes: vec![],
            remove: None,
        });
    }

    let specs: Vec<Spec> = files
        .iter()
        .filter_map(|(p, t)| {
            parse_str(t).ok().map(|mut s| {
                s.path = Some(p.clone());
                s
            })
        })
        .collect();

    // Locate the definition: the file whose Intent id is `old_id`, or
    // whose row's QUALIFIED id (`intent.id` + `.` + local row id —
    // table cells carry the local id, links carry the qualified one)
    // is `old_id`.
    let mut owners: Vec<(&Spec, Option<String>)> = vec![]; // None = intent, Some(local) = row
    for spec in &specs {
        if spec.intent.id == old_id {
            owners.push((spec, None));
        }
        for rid in &spec.defined_ids() {
            if *rid != spec.intent.id && format!("{}.{}", spec.intent.id, rid) == old_id {
                owners.push((spec, Some(rid.clone())));
            }
        }
    }
    match owners.len() {
        0 => return Err(RenameError::UnknownId(old_id.into())),
        1 => {}
        _ => return Err(RenameError::Ambiguous(old_id.into())),
    }
    let (owner, local) = (&owners[0].0, owners[0].1.clone());

    // new_id must not collide with any existing id anywhere (intents by
    // their id, rows by their qualified id).
    for spec in &specs {
        let mut ids: Vec<String> = vec![spec.intent.id.clone()];
        for rid in &spec.defined_ids() {
            if *rid != spec.intent.id {
                ids.push(format!("{}.{}", spec.intent.id, rid));
            }
        }
        if ids.iter().any(|i| *i == new_id) {
            return Err(RenameError::Collision {
                new_id: new_id.into(),
                holder: spec
                    .path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| spec.intent.id.clone()),
            });
        }
    }

    // A row rename stays within its file's namespace: new_id must be
    // `intent.id.<new local>` — moving a row across files is a merge,
    // not a rename.
    let local_pair = match &local {
        Some(local_old) => {
            let parent = format!("{}.", owner.intent.id);
            if !new_id.starts_with(&parent) || new_id.len() <= parent.len() {
                return Err(RenameError::InvalidNewId(format!(
                    "{new_id} — a row rename stays in its file's namespace ({parent}<new-local>)"
                )));
            }
            Some((local_old.as_str(), &new_id[parent.len()..]))
        }
        None => None,
    };

    // Rewrite every file; collect the ones that actually change. Only
    // the definition file gets local-id (cell/bullet) rewrites; every
    // file gets wiki-link rewrites.
    let is_intent_rename = local.is_none();
    let mut writes = vec![];
    for (path, raw) in files {
        let in_owner = owner.path.as_ref().is_some_and(|p| p == path);
        let new_text = rewrite_text(
            raw,
            old_id,
            new_id,
            if in_owner { local_pair } else { None },
        );
        if new_text != *raw {
            writes.push((path.clone(), new_text));
        }
    }

    // Intent renames move the file per the naming law (`-` ⇔ `.`).
    let mut remove = None;
    if is_intent_rename {
        let old_path = owner
            .path
            .clone()
            .expect("definition file must have a path");
        let new_name = format!("{}.md", new_id.replace('.', "-"));
        let new_path = old_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(new_name);
        // The rewritten definition file becomes the new file; the old
        // path dies. Its rewrite is carried by the new path.
        let owner_text = files
            .iter()
            .find(|(p, _)| *p == old_path)
            .expect("owner in files")
            .1
            .clone();
        writes.retain(|(p, _)| *p != old_path);
        writes.push((new_path, rewrite_text(&owner_text, old_id, new_id, None)));
        remove = Some(old_path);
    }

    // Verify gate (rename.verify → accept): re-parse the would-be repo
    // and re-run linter.referential_integrity — zero dangling or the
    // rename is rejected before a single write. A pure rename cannot
    // introduce a cycle (structure is preserved), so dangling is the
    // gate that can actually fire (specs/rename.md Notes).
    let mut details = vec![];
    let mut new_specs = vec![];
    for (path, text) in &writes {
        match parse_str(text) {
            Ok(mut s) => {
                s.path = Some(path.clone());
                new_specs.push(s);
            }
            Err(e) => details.push(format!("{}: post-rename parse failed: {e}", path.display())),
        }
    }
    for (path, text) in files {
        if remove.as_ref().is_some_and(|r| r == path) {
            continue; // the moved file is represented by its new path
        }
        if !writes.iter().any(|(p, _)| p == path)
            && let Ok(mut s) = parse_str(text)
        {
            s.path = Some(path.clone());
            new_specs.push(s);
        }
    }
    if details.is_empty() {
        let report = graph::build(&new_specs);
        for d in &report.dangling {
            details.push(format!("dangling reference after rename: {d}"));
        }
    }
    if !details.is_empty() {
        return Err(RenameError::VerifyFailed { details });
    }

    Ok(RenameOutcome {
        old_id: old_id.into(),
        new_id: new_id.into(),
        writes,
        remove,
    })
}

/// new_id must be usable as an id: non-empty, no whitespace, no link or
/// table syntax characters (it will live inside `[[…]]` and `| … |`).
fn validate_shape(new_id: &str) -> Result<(), RenameError> {
    if new_id.is_empty()
        || new_id.contains(|c: char| c.is_whitespace() || matches!(c, '[' | ']' | '|' | '#'))
    {
        return Err(RenameError::InvalidNewId(new_id.into()));
    }
    Ok(())
}

/// Rewrite one file's text: wiki-link targets (`[[old_id]]` and
/// `[[old_id.child…]]`), plus — only in the definition file — the
/// frontmatter `id:` line, exact-id table cells, and exact-id state
/// bullets (row renames rewrite the LOCAL id in cells; links carry the
/// qualified one). Everything else — prose, expressions, predicates —
/// passes through byte-identical.
fn rewrite_text(raw: &str, old_id: &str, new_id: &str, local: Option<(&str, &str)>) -> String {
    let mut out = String::with_capacity(raw.len());
    for line in raw.lines() {
        let trimmed = line.trim_start();
        let rewritten: String = if local.is_none() && trimmed == format!("id: {old_id}") {
            // Frontmatter Intent id (the definition file's own).
            line.replacen(old_id, new_id, 1)
        } else if trimmed.starts_with('|') {
            rewrite_table_row(line, old_id, new_id, local)
        } else if trimmed.starts_with("- ") {
            rewrite_state_bullet(line, old_id, new_id, local)
        } else {
            rewrite_links(line, old_id, new_id)
        };
        out.push_str(&rewritten);
        out.push('\n');
    }
    // lines() drops a missing trailing newline only when absent —
    // strip the one we added if the original had no final newline.
    if !raw.ends_with('\n') {
        out.truncate(out.trim_end_matches('\n').len());
    }
    out
}

/// Rewrite `[[old_id]]` → `[[new_id]]` and `[[old_id.x]]` →
/// `[[new_id.x]]` (child refs follow their parent's rename);
/// `[[other.old_id_prefix…]]` targets are left alone.
fn rewrite_links(line: &str, old_id: &str, new_id: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(open) = rest.find("[[") {
        let (before, after) = rest.split_at(open + 2);
        out.push_str(before);
        rest = after;
        match rest.find("]]") {
            Some(close) => {
                let (target, tail) = rest.split_at(close);
                let link_new = if target == old_id {
                    Some(new_id.to_string())
                } else if target.starts_with(&format!("{old_id}.")) {
                    Some(format!("{}.{}", new_id, &target[old_id.len() + 1..]))
                } else {
                    None
                };
                out.push_str(&link_new.unwrap_or_else(|| target.to_string()));
                out.push_str("]]");
                rest = &tail[2..];
            }
            None => {
                // Unterminated link — prose-adjacent; leave untouched.
                out.push_str(rest);
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Exact-match id cells only (`| local |`) in the definition file: the
/// defining row's id cell, plain-id derives_from cells, and transition
/// from/to cells (state renames). Prose cells, expressions, and cells
/// that merely mention the id pass through untouched. Every cell still
/// gets wiki-link rewrites.
fn rewrite_table_row(
    line: &str,
    old_id: &str,
    new_id: &str,
    local: Option<(&str, &str)>,
) -> String {
    let targets = local.map(|(lo, _)| lo).into_iter().collect::<Vec<_>>();
    if !line.split('|').any(|c| targets.contains(&c.trim())) {
        return rewrite_links(line, old_id, new_id);
    }
    let mut out = String::with_capacity(line.len());
    for (i, cell) in line.split('|').enumerate() {
        if i > 0 {
            out.push('|');
        }
        let t = cell.trim();
        if local.is_some_and(|(lo, _ln)| t == lo && !t.is_empty()) {
            // Preserve the cell's padding exactly.
            let start = cell.len() - cell.trim_start().len();
            let end = cell.len() - cell.trim_end().len();
            out.push_str(&" ".repeat(start));
            out.push_str(local.map(|(_, ln)| ln).unwrap_or(new_id));
            out.push_str(&" ".repeat(end));
        } else {
            out.push_str(&rewrite_links(cell, old_id, new_id));
        }
    }
    out
}

/// A States bullet whose leading id token is exactly `old_id`
/// (`- old_id` / `- old_id (emits: …)`).
fn rewrite_state_bullet(
    line: &str,
    old_id: &str,
    new_id: &str,
    local: Option<(&str, &str)>,
) -> String {
    let Some((lo, ln)) = local else {
        return rewrite_links(line, old_id, new_id);
    };
    let trimmed = line.trim_start();
    let token = trimmed
        .trim_start_matches("- ")
        .split([' ', '('])
        .next()
        .unwrap_or("");
    if token != lo {
        return rewrite_links(line, old_id, new_id);
    }
    let prefix_len = line.len() - trimmed.len();
    line[prefix_len..].replacen(lo, ln, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// rename_naturality's identity law: rename(I, a, a) == I — a no-op
    /// that writes nothing.
    #[test]
    fn identity_rename_writes_nothing() {
        let files = vec![(
            std::path::PathBuf::from("a.md"),
            "---\nid: a\nkind: intent\nstatement: \"THE system SHALL x\"\n---\n".to_string(),
        )];
        let out = run(&files, "a", "a").unwrap();
        assert!(out.writes.is_empty() && out.remove.is_none());
    }

    /// An id with link/table syntax cannot be a new id (it would corrupt
    /// the structured cells it lands in).
    #[test]
    fn invalid_new_ids_are_rejected_before_any_lookup() {
        let files = vec![];
        for bad in ["", "a b", "a[[b]]", "a|b"] {
            assert!(
                matches!(run(&files, "a", bad), Err(RenameError::InvalidNewId(_))),
                "{bad:?} must be rejected"
            );
        }
    }
}
