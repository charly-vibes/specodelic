//! `spk migrate` — wrap an existing plain openspec delta into the
//! Revision 19 pair: the delta stays byte-plain, the specodelic layer is
//! scaffolded into a sibling `layer.md` sidecar (gh#6 item 1, beads
//! specodelic-c32; reworked by add-delta-layer-sidecar / Revision 19).
//!
//! The pure seam: [`migrate`] returns the sidecar content, the CLI arm
//! (src/commands/manage.rs) does file IO and envelope emission. The old
//! dual-format wrap — frontmatter + layers + `## Requirements` mirror
//! inserted into the delta itself — retired with the mirror: plain
//! deltas cannot carry the mirror without failing openspec validation.
//! Design decisions now live in openspec/changes/add-delta-layer-sidecar/
//! design.md; the carried-over decisions keep their force: D3 (merge
//! adds only missing pieces — an existing sidecar passes through
//! untouched, enforced at the CLI layer), D4 (the scaffold is wired so
//! it lints clean as written), D5 (no backup files — `--dry-run` is the
//! preview path). `--rekey` (add-migrate-rekey) keeps the Revision 18
//! semantics unchanged: an `id: spec` dual-format file re-keyed in
//! place, every `[[spec.*]]` ref re-keyed, every other byte untouched.
//!
//! The third face of the verb is the corpus revision upgrade
//! (add-revision-upgrade): the core-owned [`REVISIONS`] registry records
//! every known format revision — its date, its change note and the
//! optional migration step a corpus file needs to cross it — and
//! [`plan_upgrade`] carries a file set from the corpus's source revision
//! to a target revision. The registry is thin by design: only Revision 18
//! (the id law) carries a step; every other revision was additive and
//! upgrades as a byte-stable no-op. The upgrade never rewrites a plain
//! delta and never creates a file — the wrap path's job stays untouched.

use std::fmt;

/// Why a file (or pair) cannot be migrated (specs/migrate,
/// refuse_already_migrated).
#[derive(Debug, PartialEq, Clone)]
pub enum MigrateError {
    /// The file already carries frontmatter — it is a specodelic file,
    /// not a plain delta (the dual-format wrap retired with the mirror).
    AlreadyMigrated,
    /// `## Requirements` without a delta section — a plain spec.
    NotADelta,
    /// Neither `## ADDED Requirements` nor `## MODIFIED Requirements`
    /// present — nothing to wrap.
    NoDeltaSection,
    /// A `## ADDED Requirements` heading with no trailing newline and no
    /// body — the degenerate one-line delta (specodelic-7i7): a bare
    /// heading is not a section, so there is nothing to wrap.
    BareDeltaHeading,
    /// `--rekey` on a bare `spec.md` with no parent directory: the
    /// naming law has nothing to derive the real id from (Revision 18).
    NoDerivableId,
}

impl fmt::Display for MigrateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrateError::AlreadyMigrated => write!(
                f,
                "file already carries frontmatter — a specodelic file, not a plain delta"
            ),
            MigrateError::NotADelta => write!(
                f,
                "file has ## Requirements but no ## ADDED Requirements or ## MODIFIED Requirements — a plain spec, not an openspec delta"
            ),
            MigrateError::NoDeltaSection => {
                write!(
                    f,
                    "no ## ADDED Requirements or ## MODIFIED Requirements section found — nothing to wrap"
                )
            }
            MigrateError::NoDerivableId => write!(
                f,
                "the real id cannot be derived (bare spec.md, or parent dir named `spec`) — move the file under its capability directory"
            ),
            MigrateError::BareDeltaHeading => write!(
                f,
                "the ## ADDED Requirements heading has no trailing newline or body — a bare heading is not a section"
            ),
        }
    }
}

/// Outcome of a wrap migration: the sibling `layer.md` sidecar content
/// to create beside the delta — which the transform never touches.
#[derive(Debug, PartialEq, Clone)]
pub struct WrapOutcome {
    /// The sidecar's frontmatter id (parent-directory id + `.layer`).
    pub sidecar_id: String,
    /// The full `layer.md` content.
    pub content: String,
    /// Names of the layer sections generated (the scaffold for a fresh
    /// sidecar carries all three).
    pub inserted_layers: Vec<&'static str>,
}

/// Outcome of a `--rekey` migration: the new content plus what was done.
#[derive(Debug, PartialEq, Clone)]
pub struct MigrateOutcome {
    /// The full rewritten file content.
    pub content: String,
    /// True when the id/refs were rewritten (false = idempotent no-op).
    pub rekeyed: Option<bool>,
    /// Always false for a rekey (kept for envelope continuity).
    pub inserted_frontmatter: bool,
    /// Always empty for a rekey (kept for envelope continuity).
    pub inserted_layers: Vec<&'static str>,
    /// Always false for a rekey (kept for envelope continuity).
    pub inserted_mirror: bool,
}

/// Inserted scaffold rows carry this prefix — the author's hand-finish
/// checklist is "replace every scaffold_* id with the real thing".
pub const SCAFFOLD_PREFIX: &str = "scaffold_";

/// `--rekey` transform (add-migrate-rekey): rewrite an `id: spec`
/// dual-format file to its Revision 18 real id. The naming law derives
/// the id from the parent directory (spec.md) or the stem (anything
/// else); `[[spec]]`/`[[spec.` refs re-key to the derived id; every
/// other byte stays untouched. A file already carrying its real id is
/// a no-op (idempotent sweeps); a plain delta or a bare spec.md is
/// refused, never rewritten.
pub fn rekey(text: &str, path: &std::path::Path) -> Result<MigrateOutcome, MigrateError> {
    // Shape refusals first — mirror the wrap path's triage.
    if !text.starts_with("---\n") && !text.starts_with("---\r\n") {
        return Err(MigrateError::NotADelta);
    }
    let derived = crate::lint::expected_id_from_path(path);
    if derived == "spec" || derived.is_empty() {
        // Nothing to re-key TO: a bare spec.md (no parent dir) or a
        // spec.md under a directory named `spec` both derive `spec` back
        // — refusing beats a self-referential rewrite.
        return Err(MigrateError::NoDerivableId);
    }
    let fm_close = text[4..]
        .find("\n---\n")
        .map(|i| i + 4)
        .map(|i| i + 5)
        .ok_or(MigrateError::NotADelta)?;
    let frontmatter = &text[..fm_close];
    let id_line = frontmatter
        .lines()
        .find(|l| l.trim_start().starts_with("id:"))
        .ok_or(MigrateError::NotADelta)?;
    let current_id = id_line.trim_start()[3..].trim();
    if current_id != "spec" {
        // Idempotence: already re-keyed (or never was id:spec) — a
        // no-op, never rewritten, so directory-wide sweeps are safe.
        return Ok(MigrateOutcome {
            rekeyed: Some(false),
            content: text.to_string(),
            inserted_frontmatter: false,
            inserted_layers: vec![],
            inserted_mirror: false,
        });
    }
    // A no-op when the derived id equals the current one is impossible
    // here (current == "spec", derived != "spec"), so a real rewrite
    // always follows: rekey the id line, then the body refs.
    let new_fm = frontmatter.replacen(&format!("id: {current_id}"), &format!("id: {derived}"), 1);
    let body = &text[fm_close..];
    let body = body.replace("[[spec]]", &format!("[[{derived}]]"));
    let body = body.replace("[[spec.", &format!("[[{derived}."));
    let content = format!("{new_fm}{body}");
    Ok(MigrateOutcome {
        rekeyed: Some(true),
        content,
        inserted_frontmatter: false,
        inserted_layers: vec![],
        inserted_mirror: false,
    })
}

/// The first known format revision — the corpus's revision history
/// starts at Revision 2.
pub const FIRST_REVISION: u32 = 2;

/// The latest known format revision — the one this binary embeds.
pub const LATEST_REVISION: u32 = 20;

/// One known format revision in the core-owned registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevisionEntry {
    /// The revision number.
    pub number: u32,
    /// The date the format corpus records for the revision (revisions
    /// 2 … 15 carry no date in the corpus).
    pub date: Option<&'static str>,
    /// A one-line change note.
    pub note: &'static str,
    /// The migration step a corpus file needs to cross this revision —
    /// `None` when the revision required no corpus rewrite.
    pub step: Option<RevisionStep>,
}

/// The mechanical transform a revision needs. A revision whose change was
/// additive (a new rule, a reworded constraint, a tool-side change)
/// carries no step at all: crossing it is a byte-stable no-op.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevisionStep {
    /// Revision 18's id law: the frontmatter `id: spec` becomes the
    /// parent-directory-derived id and every `[[spec…]]` ref re-keys to
    /// it. Every other byte of the file stays untouched.
    IdLaw,
}

/// The core-owned registry of the known format revisions.
///
/// Each entry records the revision number, the date the format corpus
/// records for it, a one-line change note, and an OPTIONAL migration
/// step. The registry is thin by design: only Revision 18 — the id law
/// — needs a corpus rewrite; the other revisions were additive,
/// rewordings, or changes to the tool's own semantics, so crossing them
/// changes no file.
pub const REVISIONS: &[RevisionEntry] = &[
    RevisionEntry {
        number: 2,
        date: None,
        note: "the linter's four gaps folded into the constraint/model/property split",
        step: None,
    },
    RevisionEntry {
        number: 3,
        date: None,
        note: "Rule-of-5 review of the checker corpus — the id_matches_file bug fixed",
        step: None,
    },
    RevisionEntry {
        number: 4,
        date: None,
        note: "the closed kind sets for Constraint and Property rows",
        step: None,
    },
    RevisionEntry {
        number: 5,
        date: None,
        note: "the polyglot-ecosystem sweep — quality signals, adapter contracts, feedback loops",
        step: None,
    },
    RevisionEntry {
        number: 6,
        date: None,
        note: "the category-theory sweep — grounded expressions, Moore-machine reducers",
        step: None,
    },
    RevisionEntry {
        number: 7,
        date: None,
        note: "the multiple-dispatch sweep — extension points, protocols, multimethods",
        step: None,
    },
    RevisionEntry {
        number: 8,
        date: None,
        note: "the model-checking backend became pluggable — the corpus language leaves Alloy",
        step: None,
    },
    RevisionEntry {
        number: 9,
        date: None,
        note: "declared outputs became observable claims — `observes` separates watched effects",
        step: None,
    },
    RevisionEntry {
        number: 10,
        date: None,
        note: "guard typing decided — a guard may be prose or cite a row",
        step: None,
    },
    RevisionEntry {
        number: 11,
        date: None,
        note: "`derives_from` gains the same-kind law edge",
        step: None,
    },
    RevisionEntry {
        number: 12,
        date: None,
        note: "`guard` may cite a State as well as a Constraint",
        step: None,
    },
    RevisionEntry {
        number: 13,
        date: None,
        note: "law rows enumerate their required cases as named labels in their own predicate",
        step: None,
    },
    RevisionEntry {
        number: 14,
        date: None,
        note: "domain packs — the extension mechanism becomes first-class",
        step: None,
    },
    RevisionEntry {
        number: 15,
        date: None,
        note: "executable predicate fragments — the verified gate becomes reachable",
        step: None,
    },
    RevisionEntry {
        number: 16,
        date: Some("2026-10-04"),
        note: "closed language-tag fragments — rust, py and ts",
        step: None,
    },
    RevisionEntry {
        number: 17,
        date: Some("2026-10-08"),
        note: "the min-expr kernel — decidable semantics for the data-dependent cells",
        step: None,
    },
    RevisionEntry {
        number: 18,
        date: Some("2026-10-08"),
        note: "single-tree spec authoring — the id law derives spec.md from its parent directory",
        step: Some(RevisionStep::IdLaw),
    },
    RevisionEntry {
        number: 19,
        date: Some("2026-10-24"),
        note: "change deltas go plain — the layer rides beside as a layer.md sidecar",
        step: None,
    },
    RevisionEntry {
        number: 20,
        date: Some("2026-11-12"),
        note: "the guard row names the executable regime",
        step: None,
    },
];

/// The known revision range, as the refusal messages name it.
pub fn known_revision_range() -> String {
    format!("{FIRST_REVISION} to {LATEST_REVISION}")
}

/// The registry entry for a revision, when the revision is known.
pub fn known_revision(number: u32) -> Option<&'static RevisionEntry> {
    REVISIONS.iter().find(|e| e.number == number)
}

/// Why an upgrade cannot run (specs/migrate, the upgrade refusals).
#[derive(Debug, PartialEq, Clone)]
pub enum UpgradeError {
    /// The requested target revision is outside the known range.
    UnknownTargetRevision(u32),
    /// The declared source revision is outside the known range.
    UnknownSourceRevision(u32),
    /// The target revision is below the corpus's current revision — an
    /// upgrade never moves a corpus backwards.
    TargetBelowCurrent {
        /// The requested target revision.
        target: u32,
        /// The corpus's current revision.
        current: u32,
        /// The file that still carries the older shape, when one is known.
        lagging: Option<std::path::PathBuf>,
    },
    /// The file's real id cannot be derived (a bare `spec.md`, or a
    /// parent directory named `spec`).
    NoDerivableId(std::path::PathBuf),
    /// The file's frontmatter has no closing fence — the id line cannot
    /// be read.
    UnreadableFrontmatter(std::path::PathBuf),
}

impl fmt::Display for UpgradeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpgradeError::UnknownTargetRevision(n) => write!(
                f,
                "unknown target revision {n} — the known format revisions are {}",
                known_revision_range()
            ),
            UpgradeError::UnknownSourceRevision(n) => write!(
                f,
                "unknown source revision {n} — the known format revisions are {}",
                known_revision_range()
            ),
            UpgradeError::TargetBelowCurrent {
                target,
                current,
                lagging,
            } => {
                write!(
                    f,
                    "target revision {target} is below the corpus's current revision {current} — an upgrade never moves a corpus backwards"
                )?;
                if let Some(path) = lagging {
                    write!(f, " (the oldest file is {})", path.display())?;
                }
                Ok(())
            }
            UpgradeError::NoDerivableId(path) => write!(
                f,
                "{}: the real id cannot be derived (bare spec.md, or parent dir named `spec`)",
                path.display()
            ),
            UpgradeError::UnreadableFrontmatter(path) => write!(
                f,
                "{}: the frontmatter has no closing fence — the id line cannot be read",
                path.display()
            ),
        }
    }
}

/// The per-file outcome of an upgrade plan.
#[derive(Debug, PartialEq, Clone)]
pub struct UpgradeFile {
    /// The file's path.
    pub path: std::path::PathBuf,
    /// The file's content after the upgrade (unchanged when no step
    /// applied).
    pub content: String,
    /// True when the registered steps changed the file's bytes.
    pub changed: bool,
    /// The revisions whose steps applied to this file.
    pub applied: Vec<u32>,
}

/// A file the upgrade left alone, with the reason.
#[derive(Debug, PartialEq, Clone)]
pub struct SkippedFile {
    /// The file's path.
    pub path: std::path::PathBuf,
    /// Why the file was skipped.
    pub reason: &'static str,
}

/// The upgrade plan: the resolved revisions plus the per-file outcomes.
#[derive(Debug, PartialEq, Clone)]
pub struct UpgradePlan {
    /// The corpus's source revision (declared, or detected).
    pub source_revision: u32,
    /// The target revision.
    pub target_revision: u32,
    /// The corpus files, with their post-upgrade content.
    pub files: Vec<UpgradeFile>,
    /// The files the upgrade skipped.
    pub skipped: Vec<SkippedFile>,
}

/// The revision a corpus file satisfies, derived from its shape.
///
/// `None` when the file is not a corpus spec file (no frontmatter). A
/// file that still declares `id: spec` predates Revision 18 — it
/// satisfies Revision 17 and no later. A file that already carries a
/// real id satisfies every registered step.
pub fn file_revision(text: &str) -> Option<u32> {
    frontmatter_id(text)?;
    for entry in REVISIONS {
        if let Some(step) = entry.step
            && step.pending(text)
        {
            return Some(entry.number - 1);
        }
    }
    Some(LATEST_REVISION)
}

impl RevisionStep {
    /// True when the file still carries the step's pre-shape. A step is
    /// shape-guarded: a file that does not is not rewritten by it.
    fn pending(self, text: &str) -> bool {
        match self {
            RevisionStep::IdLaw => frontmatter_id(text).as_deref() == Some("spec"),
        }
    }

    /// The step's transform: the file's new content.
    fn apply(self, text: &str, path: &std::path::Path) -> Result<String, UpgradeError> {
        match self {
            RevisionStep::IdLaw => apply_id_law(text, path),
        }
    }
}

/// The Revision 18 id law: the frontmatter `id: spec` becomes the
/// naming law's derived id and every `[[spec…]]` ref re-keys to it.
/// Every other byte of the file — line endings and the final terminator
/// included — stays untouched.
fn apply_id_law(text: &str, path: &std::path::Path) -> Result<String, UpgradeError> {
    let derived = crate::lint::expected_id_from_path(path);
    if derived.is_empty() || derived == "spec" {
        // Nothing to re-key TO: a bare spec.md (no parent dir) and a
        // spec.md under a directory named `spec` both derive `spec`
        // back — refusing beats a self-referential rewrite.
        return Err(UpgradeError::NoDerivableId(path.to_path_buf()));
    }
    let Some(close) = frontmatter_close(text) else {
        return Err(UpgradeError::UnreadableFrontmatter(path.to_path_buf()));
    };
    let frontmatter = &text[..close];
    let body = &text[close..];
    let new_frontmatter = replace_frontmatter_id(frontmatter, &derived);
    let body = body.replace("[[spec]]", &format!("[[{derived}]]"));
    let body = body.replace("[[spec.", &format!("[[{derived}."));
    Ok(format!("{new_frontmatter}{body}"))
}

/// The byte offset just past the frontmatter's closing fence, when the
/// text opens with one. Both LF and CRLF fences are recognized.
fn frontmatter_close(text: &str) -> Option<usize> {
    let open = if text.starts_with("---\n") {
        4
    } else if text.starts_with("---\r\n") {
        5
    } else {
        return None;
    };
    let mut offset = 0;
    for line in text[open..].split_inclusive('\n') {
        let len = line.len();
        if line.trim_end() == "---" {
            return Some(open + offset + len);
        }
        offset += len;
    }
    None
}

/// The frontmatter `id:` value of a spec file, when it has one.
fn frontmatter_id(text: &str) -> Option<String> {
    let close = frontmatter_close(text)?;
    let frontmatter = &text[..close];
    let id_line = frontmatter
        .lines()
        .find(|l| l.trim_start().starts_with("id:"))
        .map(|l| l.trim_start()[3..].trim())?;
    Some(id_line.trim_matches('"').trim_matches('\'').to_string())
}

/// The frontmatter with its `id:` line rewritten — every other byte
/// (indentation, line endings, the closing fence) stays untouched.
fn replace_frontmatter_id(frontmatter: &str, derived: &str) -> String {
    let mut out = String::with_capacity(frontmatter.len() + derived.len());
    let mut replaced = false;
    for line in frontmatter.split_inclusive('\n') {
        let stripped = line.trim_end_matches(['\r', '\n']);
        if !replaced && stripped.trim_start().starts_with("id:") {
            let indent = &stripped[..stripped.len() - stripped.trim_start().len()];
            let eol = &line[stripped.len()..];
            out.push_str(indent);
            out.push_str("id: ");
            out.push_str(derived);
            out.push_str(eol);
            replaced = true;
            continue;
        }
        out.push_str(line);
    }
    out
}

/// Plan the upgrade of a file set across the known format revisions.
///
/// The source revision is `declared_source` when given, and otherwise
/// the oldest revision any file in the set still satisfies (a file still
/// declaring `id: spec` satisfies Revision 17; a file already carrying
/// a real id satisfies every registered step). The registered steps
/// between the source revision and the target revision apply in
/// ascending revision order, each shape-guarded: a file that does not
/// carry a step's pre-shape is not rewritten by it.
pub fn plan_upgrade(
    files: &[(std::path::PathBuf, String)],
    declared_source: Option<u32>,
    target: u32,
) -> Result<UpgradePlan, UpgradeError> {
    if known_revision(target).is_none() {
        return Err(UpgradeError::UnknownTargetRevision(target));
    }
    if let Some(source) = declared_source
        && known_revision(source).is_none()
    {
        return Err(UpgradeError::UnknownSourceRevision(source));
    }
    // The corpus's current revision: the oldest one any file satisfies.
    let detected = files
        .iter()
        .filter_map(|(path, text)| file_revision(text).map(|rev| (rev, path.clone())))
        .min_by_key(|(rev, _)| *rev);
    let detected_revision = detected.as_ref().map(|(rev, _)| *rev);
    let lagging = detected.as_ref().map(|(_, path)| path.clone());
    let source = declared_source.unwrap_or_else(|| detected_revision.unwrap_or(LATEST_REVISION));
    if target < source {
        return Err(UpgradeError::TargetBelowCurrent {
            target,
            current: source,
            lagging,
        });
    }
    // The steps to cross, in ascending revision order.
    let steps: Vec<(u32, RevisionStep)> = REVISIONS
        .iter()
        .filter(|entry| entry.number > source && entry.number <= target)
        .filter_map(|entry| entry.step.map(|step| (entry.number, step)))
        .collect();
    let mut planned = Vec::new();
    let mut skipped = Vec::new();
    for (path, text) in files {
        match upgrade_file(path, text, &steps)? {
            Some(file) => planned.push(file),
            None => skipped.push(SkippedFile {
                path: path.clone(),
                reason: "not a corpus spec file (no frontmatter)",
            }),
        }
    }
    Ok(UpgradePlan {
        source_revision: source,
        target_revision: target,
        files: planned,
        skipped,
    })
}

/// The per-file upgrade: the registered steps that still apply, in
/// ascending revision order. `None` when the file is not a corpus spec
/// file (no frontmatter) — it is skipped, never rewritten; a file that
/// no step touches keeps its bytes exactly.
fn upgrade_file(
    path: &std::path::Path,
    text: &str,
    steps: &[(u32, RevisionStep)],
) -> Result<Option<UpgradeFile>, UpgradeError> {
    if file_revision(text).is_none() {
        return Ok(None);
    }
    let mut content = text.to_string();
    let mut applied = Vec::new();
    for (revision, step) in steps {
        if !step.pending(&content) {
            continue;
        }
        content = step.apply(&content, path)?;
        applied.push(*revision);
    }
    Ok(Some(UpgradeFile {
        path: path.to_path_buf(),
        changed: content != text,
        content,
        applied,
    }))
}

/// Wrap a plain openspec delta into the Revision 19 pair: the delta text
/// is returned untouched in spirit (the transform never rewrites it —
/// the CLI writes nothing back to the delta file) and the specodelic
/// layer is scaffolded into the sibling `layer.md` sidecar content.
///
/// Refusal triage (D3's missing-half form): a file already carrying
/// frontmatter is a specodelic file, not a plain delta
/// ([`MigrateError::AlreadyMigrated`]); a `## Requirements` section
/// without a delta heading is a plain spec, not a delta
/// ([`MigrateError::NotADelta`]); neither delta section present is
/// nothing to wrap ([`MigrateError::NoDeltaSection`]); a bare
/// `## ADDED Requirements` heading at EOF (specodelic-7i7) is not a
/// section ([`MigrateError::BareDeltaHeading`]).
pub fn migrate(text: &str, path: &std::path::Path) -> Result<WrapOutcome, MigrateError> {
    // specodelic-7i7: a bare `## ADDED Requirements` heading at EOF (no
    // trailing newline) is not a section — refuse labeled.
    if has_bare_heading(text, "## ADDED Requirements")
        || has_bare_heading(text, "## MODIFIED Requirements")
    {
        return Err(MigrateError::BareDeltaHeading);
    }
    let has_added = section_span(text, "## ADDED Requirements").is_some();
    // specodelic-nvs9 (gh-16): a MODIFIED section alone is a wrappable
    // delta — the triage accepts ADDED, MODIFIED, or both.
    let has_modified = section_span(text, "## MODIFIED Requirements").is_some();
    let has_requirements = section_span(text, "## Requirements").is_some();
    let has_frontmatter = text.starts_with("---\n") || text.starts_with("---\r\n");
    match (has_frontmatter, has_added || has_modified, has_requirements) {
        // Already a specodelic file: frontmatter present ⇒ refuse.
        (true, _, _) => return Err(MigrateError::AlreadyMigrated),
        // Plain spec: Requirements without a delta heading ⇒ refuse.
        (_, false, true) => return Err(MigrateError::NotADelta),
        // Nothing to wrap (neither ADDED nor MODIFIED present).
        (_, false, false) => return Err(MigrateError::NoDeltaSection),
        _ => {}
    }

    // The sidecar id: the parent-directory id (the capability dir the
    // delta sits in) plus a `.layer` suffix — the naming law's sidecar
    // arm (Revision 19). A bare delta file (no parent dir) falls back to
    // the stem law, matching expected_id_from_path.
    let parent_id = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .map(|d| d.replace('-', "."))
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("spec")
                .replace('-', ".")
        });
    let sidecar_id = format!("{parent_id}.layer");

    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut inserted_layers: Vec<&'static str> = Vec::new();
    let mut content = generated_frontmatter(eol, &sidecar_id);
    for layer in ["## Constraints", "## Model", "## Properties"] {
        content.push_str(&scaffold_layer(layer, eol, &sidecar_id));
        inserted_layers.push(match layer {
            "## Constraints" => "Constraints",
            "## Model" => "Model",
            _ => "Properties",
        });
    }

    Ok(WrapOutcome {
        sidecar_id,
        content,
        inserted_layers,
    })
}

/// True when a `## <heading>` line occurs at a line start but is not
/// terminated by a newline (LF or CRLF) — a bare heading at EOF, the
/// degenerate one-line delta (specodelic-7i7). Such a heading opens no
/// section; `migrate` refuses labeled instead of misreading it.
fn has_bare_heading(text: &str, heading: &str) -> bool {
    let mut from = 0;
    while let Some(i) = text[from..].find(heading) {
        let at = from + i;
        let at_line_start = at == 0 || text.as_bytes()[at - 1] == b'\n';
        if at_line_start && !text[at + heading.len()..].starts_with(['\n', '\r']) {
            return true;
        }
        from = at + heading.len();
    }
    false
}

/// Byte span of a `## <name>` section: from the heading line's start to
/// the next `## ` heading (exclusive) or EOF. Indices are relative to
/// `text`.
fn section_span(text: &str, heading: &str) -> Option<(usize, usize)> {
    let hay = text;
    // Heading at a line start: either the very beginning or after \n.
    // A heading must be a complete line (terminated by \n or \r\n) to
    // open a section — a bare heading not terminated by a newline (the
    // degenerate one-line delta, specodelic-7i7) falls through to the
    // needle search, which can never match it: the caller refuses
    // labeled instead of misreading it.
    let start = if hay.starts_with(heading)
        && (hay[heading.len()..].starts_with('\n') || hay[heading.len()..].starts_with("\r\n"))
    {
        0
    } else {
        let needle = format!("\n{heading}\n");
        hay.find(&needle)? + 1
    };
    let body_after = start + heading.len() + 1; // past heading + its newline
    let end = text[body_after..]
        .find("\n## ")
        .map(|i| body_after + i + 1)
        .unwrap_or(text.len());
    Some((start, end))
}

/// Generated sidecar frontmatter: the parent-dir + `.layer` id (the
/// naming law's sidecar arm, Revision 19), EARS scaffold statement (D4)
/// — the author replaces it.
fn generated_frontmatter(eol: &str, id: &str) -> String {
    format!(
        "---{eol}\
         id: {id}{eol}\
         kind: intent{eol}\
         statement: \"WHEN the migrated delta is elaborated, THE author SHALL replace this scaffold statement with the real requirement.\"{eol}\
         ---{eol}{eol}"
    )
}

/// Wired scaffold for one layer (D4): rows reference each other and the
/// sidecar's own intent so the file lints clean before the author writes
/// anything real.
fn scaffold_layer(layer: &str, eol: &str, id: &str) -> String {
    let p = SCAFFOLD_PREFIX;
    let self_ref = format!("[[{id}.{p}constraint]]");
    match layer {
        "## Constraints" => format!(
            "## Constraints{eol}{eol}\
             | id | kind | expr | traces_to |{eol}\
             |----|------|------|-----------|{eol}\
             | {p}constraint | invariant | `true` | [[{id}]] |{eol}{eol}"
        ),
        "## Model" => format!(
            "## Model{eol}{eol}\
             ### States{eol}{eol}\
             - `draft`{eol}{eol}\
             ### Transitions{eol}{eol}\
             | id | from | to | guard |{eol}\
             |----|------|----|-------|{eol}\
             | {p}transition | draft | draft | {self_ref} |{eol}{eol}"
        ),
        _ => format!(
            "## Properties{eol}{eol}\
             | id | kind | derives_from | generator | predicate |{eol}\
             |----|------|--------------|-----------|-----------|{eol}\
             | {p}property | unit | {self_ref} | `todo()` | `true` |{eol}{eol}"
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::test_panic::UnwrapPanic;

    use super::*;

    fn plain_delta() -> String {
        "## ADDED Requirements\n\n### Requirement: Widget\nThe system SHALL wiget.\n".to_string()
    }

    /// Fixture path: a spec.md under a capability directory — the
    /// sidecar id derives from the parent dir plus `.layer` (Revision 19).
    fn spec_path() -> std::path::PathBuf {
        std::path::PathBuf::from("demo-cap/spec.md")
    }

    // --- refusals ---

    #[test]
    fn frontmatter_carrying_file_is_refused() {
        let dual = "---\nid: spec\nkind: intent\nstatement: \"THE x SHALL x.\"\n---\n\n## ADDED Requirements\n\n### Requirement: W\nThe system SHALL w.\n";
        assert_eq!(
            migrate(dual, &spec_path()),
            Err(MigrateError::AlreadyMigrated)
        );
    }

    #[test]
    fn plain_spec_is_refused() {
        let plain = "# Title\n\n## Requirements\n\n### Requirement: R\nThe system SHALL r.\n";
        assert_eq!(migrate(plain, &spec_path()), Err(MigrateError::NotADelta));
    }

    #[test]
    fn no_delta_section_is_refused() {
        let prose = "# Title\n\nSome prose, no sections at all.\n";
        assert_eq!(
            migrate(prose, &spec_path()),
            Err(MigrateError::NoDeltaSection)
        );
    }

    #[test]
    fn degenerate_one_line_delta_is_refused_labeled() {
        let bare = "## ADDED Requirements";
        assert_eq!(
            migrate(bare, &spec_path()),
            Err(MigrateError::BareDeltaHeading)
        );
        let bare_modified = "## MODIFIED Requirements\n\n### Requirement: W\nThe system SHALL w.\n\n## ADDED Requirements";
        assert_eq!(
            migrate(bare_modified, &spec_path()),
            Err(MigrateError::BareDeltaHeading)
        );
    }

    // --- the wrap ---

    #[test]
    fn wraps_plain_delta_into_a_sidecar_leaving_the_delta_alone() {
        let delta = plain_delta();
        let out = migrate(&delta, &spec_path()).unwrap_p();
        // The sidecar id is the parent-dir id + .layer (Revision 19).
        assert_eq!(out.sidecar_id, "demo.cap.layer");
        assert!(
            out.content.contains("id: demo.cap.layer\n"),
            "{}",
            out.content
        );
        // The sidecar carries the layer skeleton — and no mirror, no
        // delta grammar.
        assert!(out.content.contains("## Constraints"));
        assert!(out.content.contains("## Model"));
        assert!(out.content.contains("## Properties"));
        assert!(!out.content.contains("## ADDED Requirements"));
        assert!(!out.content.contains("## Requirements"));
        // The transform never touches the delta text.
        assert_eq!(delta, plain_delta(), "delta bytes unchanged");
        assert_eq!(
            out.inserted_layers,
            vec!["Constraints", "Model", "Properties"]
        );
    }

    #[test]
    fn sidecar_id_ignores_the_delta_filename() {
        // A non-spec.md delta still sits in the capability directory —
        // the sidecar id derives from the PARENT DIRECTORY, not the stem.
        let out = migrate(&plain_delta(), std::path::Path::new("specs/cap-a/delta.md")).unwrap_p();
        assert_eq!(out.sidecar_id, "cap.a.layer");
    }

    #[test]
    fn crlf_delta_gets_crlf_sidecar() {
        let delta =
            "## ADDED Requirements\r\n\r\n### Requirement: Widget\r\nThe system SHALL wiget.\r\n";
        let out = migrate(delta, &spec_path()).unwrap_p();
        assert!(out.content.contains("\r\n"), "CRLF preserved in sidecar");
        assert!(!out.content.contains("\n\n"), "no LF blank lines leak");
    }

    #[test]
    fn no_delta_section_message_names_both_accepted_sections() {
        let err = migrate("# nothing\n", &spec_path()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("## ADDED Requirements") && msg.contains("## MODIFIED Requirements"));
    }

    #[test]
    fn scaffold_lints_clean_through_the_real_linter() {
        let delta = plain_delta();
        let out = migrate(&delta, &spec_path()).unwrap_p();
        let spec = crate::spec::parse_str(&out.content).unwrap_p();
        assert_eq!(spec.intent.id, "demo.cap.layer");
        let report = crate::lint::lint_corpus(&[spec]);
        assert!(
            report.issues.is_empty(),
            "sidecar scaffold must lint clean: {:?}",
            report.issues.iter().map(|i| &i.message).collect::<Vec<_>>()
        );
    }
}

#[cfg(test)]
mod rekey_tests {
    use super::*;
    use crate::test_panic::UnwrapPanic;

    fn spec_path() -> std::path::PathBuf {
        std::path::PathBuf::from("openspec/specs/ge-cli/spec.md")
    }

    /// A 0.6.0-era dual-format file: `id: spec` + `[[spec.*]]` refs.
    fn legacy_file() -> String {
        "---\nid: spec\nkind: intent\nstatement: \"THE ge cli SHALL hold\"\n---\n\n\
         ## Constraints\n\n\
         | id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | c1 | invariant | `x` | [[spec]] |\n\
         | c2 | invariant | `y` | [[spec.c1]] |\n\n\
         ## Model\n\n\
         ### States\n\n- `s1`\n\n\
         ### Transitions\n\n\
         | id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | t | s1 | s1 | [[spec.c1]] |\n\n\
         ## Properties\n\n\
         | id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|------------|\n\
         | p | unit | [[spec.c1]] | `g()` | `x` |\n\n\
         ## Requirements\n\n\
         ### Requirement: Ge cli\nThe ge cli SHALL hold.\n"
            .to_string()
    }

    #[test]
    fn rekey_rewrites_id_and_refs() {
        let out = rekey(&legacy_file(), &spec_path()).unwrap_p();
        assert!(out.content.contains("id: ge.cli\n"), "{}", out.content);
        assert!(out.content.contains("[[ge.cli]]"));
        assert!(out.content.contains("[[ge.cli.c1]]"));
        assert!(
            !out.content.contains("[[spec"),
            "no spec-qualified ref survives"
        );
        // Everything else byte-identical: strip the id line and refs, the
        // skeleton is untouched.
        assert!(out.content.contains("| c1 | invariant | `x` |"));
        assert!(
            out.content
                .contains("### Requirement: Ge cli\nThe ge cli SHALL hold.")
        );
    }

    #[test]
    fn rekey_refuses_plain_delta() {
        let delta = "## ADDED Requirements\n\n### Requirement: W\nThe system SHALL w.\n";
        let err = rekey(delta, &spec_path()).unwrap_err();
        assert_eq!(err, MigrateError::NotADelta);
    }

    #[test]
    fn rekey_refuses_bare_spec_md() {
        let err = rekey(&legacy_file(), std::path::Path::new("spec.md")).unwrap_err();
        assert!(matches!(err, MigrateError::NoDerivableId));
    }

    #[test]
    fn rekey_idempotent_second_run_is_noop() {
        let out = rekey(&legacy_file(), &spec_path()).unwrap_p();
        // Second run: id is already real — a no-op.
        let second = rekey(&out.content, &spec_path()).unwrap_p();
        assert_eq!(second.content, out.content, "no-op never rewrites");
        assert!(second.rekeyed.is_none() || !second.rekeyed.unwrap_p());
    }

    #[test]
    fn rekey_preserves_prose_mentions() {
        let file = legacy_file().replace(
            "### Requirement: Ge cli\n",
            "### Requirement: Ge cli\nSee spec.c1 prose mention — not a wiki-link.\n",
        );
        let out = rekey(&file, &spec_path()).unwrap_p();
        assert!(
            out.content.contains("See spec.c1 prose mention"),
            "prose (non-bracketed) mentions stay untouched: {}",
            out.content
        );
    }

    #[test]
    fn rekey_dry_run_semantics_via_outcome() {
        // The command layer owns --dry-run; the library transform is pure.
        let out = rekey(&legacy_file(), &spec_path()).unwrap_p();
        assert!(out.rekeyed.unwrap_p());
        assert!(!out.inserted_frontmatter);
        assert!(out.inserted_layers.is_empty());
        assert!(!out.inserted_mirror);
    }
}

#[cfg(test)]
mod revision_tests {
    use super::*;
    use crate::test_panic::UnwrapPanic;

    /// A pre-Revision-18 corpus file — the 0.6.0-era dual-format shape.
    fn legacy_file() -> String {
        "---\nid: spec\nkind: intent\nstatement: \"THE legacy cap SHALL hold\"\n---\n\n\
         ## Constraints\n\n\
         | id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | c1 | invariant | `holds` | [[spec]] |\n\n\
         ## Model\n\n\
         ### States\n\n- `s1`\n\n\
         ### Transitions\n\n\
         | id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | t1 | s1 | s1 | [[spec.c1]] |\n\n\
         ## Properties\n\n\
         | id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|-----------|\n\
         | p1 | unit | [[spec.c1]] | `g()` | `holds` |\n"
            .to_string()
    }

    fn legacy_path() -> std::path::PathBuf {
        std::path::PathBuf::from("corpus/legacy-cap/spec.md")
    }

    /// The registry covers the corpus's revision history without gaps.
    #[test]
    fn registry_covers_the_known_revision_range() {
        let numbers: Vec<u32> = REVISIONS.iter().map(|e| e.number).collect();
        let expected: Vec<u32> = (FIRST_REVISION..=LATEST_REVISION).collect();
        assert_eq!(numbers, expected);
        assert_eq!(known_revision(FIRST_REVISION).map(|e| e.number), Some(2));
        assert!(known_revision(FIRST_REVISION - 1).is_none());
        assert!(known_revision(LATEST_REVISION + 1).is_none());
    }

    /// The registry's latest revision is the one the binary embeds.
    #[test]
    fn registry_latest_matches_the_embedded_format_revision() {
        let embedded = crate::guide::revision_number(crate::guide::FORMAT_REVISION).unwrap_p();
        assert_eq!(embedded, LATEST_REVISION);
    }

    /// Every entry carries a change note; the dated entries are the ones
    /// the format corpus dates.
    #[test]
    fn registry_entries_carry_a_note() {
        for entry in REVISIONS {
            assert!(
                !entry.note.is_empty(),
                "revision {} has no note",
                entry.number
            );
        }
        let dated: Vec<u32> = REVISIONS
            .iter()
            .filter(|e| e.date.is_some())
            .map(|e| e.number)
            .collect();
        assert_eq!(dated, vec![16, 17, 18, 19, 20]);
    }

    /// Revision 18 is the only revision that needs a corpus rewrite —
    /// the registry is thin by design.
    #[test]
    fn only_the_id_law_carries_a_step() {
        let with_steps: Vec<u32> = REVISIONS
            .iter()
            .filter(|e| e.step.is_some())
            .map(|e| e.number)
            .collect();
        assert_eq!(with_steps, vec![18]);
    }

    /// A file's revision is detected from its shape.
    #[test]
    fn file_revision_detects_the_pre_18_shape() {
        assert_eq!(file_revision(&legacy_file()), Some(17));
        let modern = legacy_file().replace("id: spec", "id: legacy.cap");
        assert_eq!(file_revision(&modern), Some(LATEST_REVISION));
        assert_eq!(file_revision("# just prose\n"), None);
    }

    /// The plan applies the registered steps in ascending revision order.
    #[test]
    fn plan_applies_the_registered_steps_in_order() {
        let files = vec![(legacy_path(), legacy_file())];
        let plan = plan_upgrade(&files, None, LATEST_REVISION).unwrap_p();
        assert_eq!(plan.source_revision, 17);
        assert_eq!(plan.target_revision, LATEST_REVISION);
        assert_eq!(plan.files.len(), 1);
        assert_eq!(plan.files[0].applied, vec![18]);
        assert!(plan.files[0].changed);
        assert!(plan.files[0].content.contains("id: legacy.cap\n"));
        assert!(plan.files[0].content.contains("[[legacy.cap.c1]]"));
    }

    /// A file that needs no step keeps its bytes — CRLF endings and the
    /// absent final terminator included.
    #[test]
    fn plan_leaves_an_unchanged_file_byte_identical() {
        let modern =
            "---\r\nid: notes\r\nkind: intent\r\nstatement: \"THE notes SHALL hold\"\r\n---\r\n";
        let files = vec![(
            std::path::PathBuf::from("corpus/notes.md"),
            modern.to_string(),
        )];
        let plan = plan_upgrade(&files, None, LATEST_REVISION).unwrap_p();
        assert!(!plan.files[0].changed);
        assert_eq!(plan.files[0].content, modern);
    }

    /// The refusals: an unknown revision, and a target below the
    /// corpus's current revision.
    #[test]
    fn plan_refuses_unknown_and_downgrade_requests() {
        let files = vec![(legacy_path(), legacy_file())];
        assert_eq!(
            plan_upgrade(&files, None, 999),
            Err(UpgradeError::UnknownTargetRevision(999))
        );
        assert_eq!(
            plan_upgrade(&files, Some(999), LATEST_REVISION),
            Err(UpgradeError::UnknownSourceRevision(999))
        );
        assert!(matches!(
            plan_upgrade(&files, None, 16),
            Err(UpgradeError::TargetBelowCurrent {
                target: 16,
                current: 17,
                ..
            })
        ));
    }

    /// The upgrade never rewrites a plain delta — the wrap path's job.
    #[test]
    fn plan_skips_a_file_that_is_not_a_corpus_spec() {
        let delta = "## ADDED Requirements\n\n### Requirement: W\nThe system SHALL w.\n";
        let files = vec![(
            std::path::PathBuf::from("corpus/delta.md"),
            delta.to_string(),
        )];
        let plan = plan_upgrade(&files, None, LATEST_REVISION).unwrap_p();
        assert!(plan.files.is_empty());
        assert_eq!(plan.skipped.len(), 1);
    }
}
