//! # packs.rs — the domain-pack mechanism (specs/packs.md, Revision 14)
//!
//! **Purpose:** make specodelic's extension mechanism first-class — a
//! domain pack is a declared, discoverable, versioned artifact in the
//! corpus itself (`kind: profile` + six manifest tables), not a
//! convention followed by hand.
//!
//! **Responsibilities:**
//! - parse pack manifests (the six per-facet closed tables) and validate
//!   their structure (`pack_shape` — the append-only lint rule);
//! - discover packs by corpus scan anchored at the git repository
//!   toplevel (the lint target directory's common parent when no git
//!   root exists) — no config file, no registry outside the corpus;
//! - activate pack checking advisory-first: vocabulary use triggers the
//!   pack's advisory (per-pack attribution); a declared `uses` edge
//!   upgrades to declared enablement and enables the revision-skew
//!   advisory; orphan vocabulary is a labeled failure naming the
//!   candidate pack and both remediations — two halves
//!   (specodelic-erd): a `uses` edge with no matching discovered pack
//!   (concrete candidate), and a pack-qualified (dotted) token in a
//!   structured kind/field position whose namespace matches no
//!   discovered pack (prefix-derived candidate).
//!
//! **Rationale (v1 scoping decisions, pinned by tests):**
//! - *Orphan detection is typed, not heuristic*: a bare dotted token in
//!   prose cannot be told apart from a row id (`compile.extraction_failure`
//!   is not vocabulary), so the mechanical orphan signal is two halves —
//!   the declared `uses` edge, and (specodelic-erd) pack-qualified tokens
//!   in *structured* kind/field positions only (kind cells, table column
//!   headers). Prose stays unscanned, so the signal stays
//!   false-positive-free: files using no pack vocabulary lint
//!   byte-identically (we would rather under- than over-report).
//! - *Lifecycle current-state*: the Model's States list either names the
//!   full three-state machine (`draft`, `published`, `deprecated` — the
//!   steady state is `published`) or a single state naming exactly where
//!   the pack stands. Deterministic, testable, and requires no new
//!   frontmatter field (base closed sets freeze).
//! - Manifest row shapes (fixed per facet, v1): every table is two
//!   columns — Sections `section | row_shape`; Kinds `kind | vocabulary`;
//!   References `field | resolves_to`; Checkers `rule | semantics`;
//!   Floors `kind | required_cases`; Requires `dep | revision`.
//! - Pack checkers ship as *declarations* (honest-empty): an activated
//!   pack reports its declared rules and an empty checked-set — no
//!   fabricated discoveries. Vocabulary checking proper arrives with the
//!   standard packs (follow-on tickets).

use crate::guide;
use crate::lint::{Issue, Report};
use crate::spec::Spec;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The six manifest facets, in declaration order (append-only).
const MANIFEST_TABLES: &[&str] = &[
    "## Sections",
    "## Kinds",
    "## References",
    "## Checkers",
    "## Floors",
    "## Requires",
];

/// Base closed-set members a pack's `## Kinds` table may never claim
/// unqualified — the narrowing rejection (append-only law).
const BASE_KINDS: &[&str] = &[
    "intent",
    "profile",
    "invariant",
    "advisory",
    "effect",
    "extension_point",
    "unit",
    "law",
];

/// One discovered pack.
#[derive(Debug, Clone, Serialize)]
pub struct PackInfo {
    /// The pack's frontmatter id (the `uses` edge's target).
    pub id: String,
    /// Lifecycle state: `draft` | `published` | `deprecated`.
    pub lifecycle: String,
    /// Vocabulary tokens the pack declares (kinds, sections, fields).
    pub vocabulary: Vec<String>,
    /// The `## Requires` base pin (Revision number), when declared.
    pub base_pin: Option<u32>,
}

/// A pack's parsed manifest.
#[derive(Debug, Clone, Default)]
pub struct Manifest {
    pub id: String,
    pub lifecycle: String,
    pub sections: Vec<String>,
    pub kinds: Vec<String>,
    pub references: Vec<String>,
    pub checkers: Vec<String>,
    pub floors: Vec<String>,
    pub requires: Vec<(String, String)>,
}

impl Manifest {
    /// The vocabulary tokens this pack declares — kinds, section names,
    /// and reference fields (the words a consumer's use of the pack can
    /// match).
    pub fn vocabulary(&self) -> Vec<String> {
        let mut v = self.kinds.clone();
        v.extend(self.sections.iter().cloned());
        v.extend(self.references.iter().cloned());
        v
    }

    /// The declared base-format revision pin (the `base` row of
    /// `## Requires`), as a `Revision N` number.
    pub fn base_pin(&self) -> Option<u32> {
        self.requires
            .iter()
            .find(|(dep, _)| dep == "base")
            .and_then(|(_, rev)| guide::revision_number(rev))
    }
}

/// Extract the markdown table that immediately follows a `## <heading>`
/// line, as raw `| ... |` rows (cell count preserved for shape checks).
///
/// Deliberately NOT folded onto the spanned parse path (design decision
/// D5, evaluated in `add-acset-writer` task 6.1, 2026-10-04): the
/// spanned parser only extracts the format's own tables —
/// `TableKind::{Constraints, Properties, Transitions}` under known
/// headings (`src/spec.rs` `on_h2`) — while the six manifest facets
/// (`## Sections` … `## Requires`) are pack vocabulary outside `Spec`'s
/// structure; their rows never reach the parser, spanned or otherwise.
/// A fold would need new parser surface (arbitrary-heading spanned
/// tables) plus escaped-pipe semantics parity — far beyond this
/// change's blast radius, for a file kind the writer never edits. Kept
/// as a second, independent table parse until a manifest-in-`Spec`
/// capability change exists; this comment is the decision of record.
fn table_after(lines: &[&str], heading: &str) -> Option<Vec<Vec<String>>> {
    let start = lines.iter().position(|l| l.trim() == heading)?;
    let mut rows = vec![];
    for line in &lines[start + 1..] {
        let t = line.trim();
        if t.is_empty() {
            if rows.is_empty() {
                continue; // skip the blank line between heading and table
            }
            break;
        }
        if t.starts_with("|") && t.ends_with("|") {
            // escaped pipes (`\|`) are cell content, not separators —
            // protect them before splitting
            const ESC: &str = "\u{0}PIPE\u{0}";
            let protected = t.replace("\\|", ESC);
            let cells: Vec<String> = protected
                .trim_start_matches('|')
                .trim_end_matches('|')
                .split('|')
                .map(|c| c.trim().replace(ESC, "\\|").to_string())
                .collect();
            // skip separator rows (---)
            if cells
                .iter()
                .all(|c| c.chars().all(|ch| ch == '-' || ch == ' '))
            {
                continue;
            }
            if cells.iter().all(|c| !c.is_empty()) {
                rows.push(cells);
            }
        } else if rows.is_empty() {
            continue; // prose between heading and table
        } else {
            break;
        }
    }
    // markdown tables carry a header row before the separator — drop it
    if !rows.is_empty() {
        rows.remove(0);
    }
    Some(rows)
}

/// Parse a pack file's manifest. `pack` must have frontmatter kind
/// `profile` (callers gate on that).
pub fn parse_manifest(pack: &Spec) -> Manifest {
    let raw_text = raw(pack);
    let lines: Vec<&str> = raw_text.lines().collect();
    let mut m = Manifest {
        id: pack.intent.id.clone(),
        lifecycle: lifecycle_of(pack),
        ..Default::default()
    };
    m.sections = table_after(&lines, "## Sections")
        .unwrap_or_default()
        .iter()
        .filter(|r| r.len() >= 2)
        .map(|r| r[0].clone())
        .collect();
    m.kinds = table_after(&lines, "## Kinds")
        .unwrap_or_default()
        .iter()
        .filter(|r| r.len() >= 2)
        .map(|r| r[0].clone())
        .collect();
    m.references = table_after(&lines, "## References")
        .unwrap_or_default()
        .iter()
        .filter(|r| r.len() >= 2)
        .map(|r| r[0].clone())
        .collect();
    m.checkers = table_after(&lines, "## Checkers")
        .unwrap_or_default()
        .iter()
        .filter(|r| r.len() >= 2)
        .map(|r| r[0].clone())
        .collect();
    m.floors = table_after(&lines, "## Floors")
        .unwrap_or_default()
        .iter()
        .filter(|r| r.len() >= 2)
        .map(|r| r[0].clone())
        .collect();
    m.requires = table_after(&lines, "## Requires")
        .unwrap_or_default()
        .iter()
        .filter(|r| r.len() >= 2)
        .map(|r| (r[0].clone(), r[1].clone()))
        .collect();
    m
}

/// The pack's current lifecycle state (v1 rule — see module header):
/// the full three-state machine declares the steady state `published`;
/// a single-state Model names exactly where the pack stands.
fn lifecycle_of(pack: &Spec) -> String {
    let names: Vec<&str> = pack.states.iter().map(|s| s.id.as_str()).collect();
    if names.len() == 1 {
        return names[0].to_string();
    }
    if names.len() == 3
        && names.contains(&"draft")
        && names.contains(&"published")
        && names.contains(&"deprecated")
    {
        return "published".into();
    }
    "published".into()
}

/// Word-boundary containment: does `haystack` use `token` as a word?
fn contains_word(haystack: &str, token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    let mut from = 0;
    while let Some(pos) = haystack[from..].find(token) {
        let abs = from + pos;
        let before = haystack[..abs].chars().next_back();
        let after = haystack[abs + token.len()..].chars().next();
        let boundary = |c: Option<char>| {
            c.map(|c| !(c.is_alphanumeric() || c == '_'))
                .unwrap_or(true)
        };
        if boundary(before) && boundary(after) {
            return true;
        }
        from = abs + 1;
    }
    false
}

/// Discover packs workspace-wide: scan `.md` files under the discovery
/// root (git toplevel, else the common parent of the linted specs) for
/// `kind: profile` frontmatter. No config file, no registry.
/// The namespace prefix of a pack-qualified (dotted) token —
/// `data.dataset` → `data`. Identifier-shaped segments only (alphabetic
/// first char): a dotted prose token never yields one, so the
/// vocabulary-orphan signal stays structured-position-only and
/// false-positive-free (specodelic-erd).
fn namespace_of(token: &str) -> Option<String> {
    let ident = |s: &str| {
        s.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
            && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    };
    let mut segs = token.split('.');
    let ns = segs.next()?;
    let head = segs.next()?; // dotted — at least two segments
    if ident(ns) && ident(head) && segs.all(ident) {
        Some(ns.to_string())
    } else {
        None
    }
}

pub fn discover(specs: &[Spec]) -> Vec<PackInfo> {
    let mut packs: Vec<PackInfo> = scan_workspace(specs)
        .into_iter()
        .map(|(_, m)| PackInfo {
            id: m.id.clone(),
            lifecycle: m.lifecycle.clone(),
            vocabulary: m.vocabulary(),
            base_pin: m.base_pin(),
        })
        .collect();
    packs.sort_by(|a, b| a.id.cmp(&b.id));
    packs.dedup_by(|a, b| a.id == b.id);
    packs
}

/// The workspace scan behind [`discover`]: every `kind: profile` file
/// under the discovery root with its parsed manifest (the full manifest
/// is needed by the fiber-kind walkers — `PackInfo` surfaces only the
/// joined vocabulary).
fn scan_workspace(specs: &[Spec]) -> Vec<(PathBuf, Manifest)> {
    let root = discovery_root(specs);
    let mut packs = vec![];
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if path.is_dir() {
                if name != ".git" && name != "target" && name != "node_modules" {
                    stack.push(path);
                }
                continue;
            }
            if name.ends_with(".md")
                && path.is_file()
                && let Ok(src) = std::fs::read_to_string(&path)
                && is_profile_frontmatter(&src)
                && let Ok(mut spec) = crate::spec::parse_str(&src)
            {
                spec.path = Some(path.clone());
                packs.push((path, parse_manifest(&spec)));
            }
        }
    }
    packs.sort_by(|a, b| a.1.id.cmp(&b.1.id));
    packs.dedup_by(|a, b| a.1.id == b.1.id);
    packs
}

/// The discovery anchor: the git repository toplevel when one exists,
/// else the common parent of the linted spec paths (the lint target
/// directory's parent in the single-file case).
fn discovery_root(specs: &[Spec]) -> PathBuf {
    // probe git AT the linted corpus — the workspace the specs live in,
    // never the process cwd (a consumer-repo lint from elsewhere must
    // still anchor correctly)
    let probe_dir = specs
        .iter()
        .filter_map(|s| {
            s.path
                .as_ref()
                .and_then(|p| p.parent().map(Path::to_path_buf))
        })
        .next_back();
    if let Some(dir) = probe_dir
        && let Ok(out) = std::process::Command::new("git")
            .args(["-C", &dir.to_string_lossy(), "rev-parse", "--show-toplevel"])
            .output()
        && out.status.success()
    {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() {
            return PathBuf::from(s);
        }
    }
    specs
        .iter()
        .filter_map(|s| {
            s.path
                .as_ref()
                .and_then(|p| p.parent().map(Path::to_path_buf))
        })
        .next_back()
        .unwrap_or_else(|| PathBuf::from("."))
}

/// The pack's raw file text — manifest tables and vocabulary scans read
/// the file itself (the parser keeps only the four standard layers).
fn raw(spec: &Spec) -> String {
    spec.path
        .as_deref()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default()
}

/// Cheap frontmatter probe: does the file declare `kind: profile`?
fn is_profile_frontmatter(src: &str) -> bool {
    let Some(rest) = src.strip_prefix("---") else {
        return false;
    };
    let Some(end) = rest.find("\n---") else {
        return false;
    };
    rest[..end].lines().any(|l| {
        let t = l.trim();
        t == "kind: profile"
    })
}

/// The pack pass over a linted corpus — appends `pack_shape` findings to
/// the issues channel, activation/skew/deprecation advisories to the
/// warnings channel, and the discovered packs to the report data. Called
/// at the end of [`crate::lint::lint_corpus`]; inert (no findings, no
/// data) when the workspace declares no `kind: profile` files and no
/// `uses` edges — files without packs lint byte-identically.
pub fn pack_pass(specs: &[Spec], report: &mut Report) {
    // 1. pack_shape over every profile file (and non-profile manifest claims)
    for spec in specs {
        if spec.intent.kind == "profile" {
            shape_findings(spec, report);
        } else {
            nonprofile_manifest_findings(spec, report);
        }
    }

    // 2. discovery — inert unless at least one pack exists
    let packs = discover(specs);
    if !packs.is_empty() {
        report.packs = packs.clone();
    }
    let by_id: BTreeMap<String, PackInfo> =
        packs.iter().map(|p| (p.id.clone(), p.clone())).collect();

    // 3. consumers: activation + skew + orphan (typed, via `uses` edges
    // and — specodelic-erd — vocabulary positions)
    // a pack's namespace is its id's first segment — dotless ids name
    // their namespace directly (`bioimage`)
    let pack_namespaces: std::collections::BTreeSet<String> = packs
        .iter()
        .map(|p| p.id.split('.').next().unwrap_or(&p.id).to_string())
        .collect();
    for spec in specs {
        // pack self-exemption: a pack file is never a consumer
        if spec.intent.kind == "profile" {
            continue;
        }
        // namespaces whose concrete uses-edge orphan already fired for
        // this file — the prefix-derived vocabulary finding must not
        // duplicate them
        let mut orphan_namespaces = std::collections::BTreeSet::new();
        // declared enablement: every `uses`-column link
        for link in &spec.links {
            if link.column != "uses" {
                continue;
            }
            uses_edge_findings(spec, link, &by_id, specs, &mut orphan_namespaces, report);
        }
        // vocabulary-triggered orphan (specodelic-erd): the "no pack
        // discovered" half of `orphan_vocabulary_labeled` — see
        // vocab_orphan_tokens for the prose-stays-unscanned rule.
        let vocab_orphans = vocab_orphan_tokens(spec, &pack_namespaces);
        emit_vocab_orphan_findings(spec, vocab_orphans, &orphan_namespaces, report);
        // implicit activation: vocabulary use (per-pack attribution,
        // overlapping vocabulary activates every declaring pack)
        implicit_activation_warnings(spec, &packs, report);
    }
}

/// Non-profile files must not declare manifest tables — a manifest
/// declares a pack, and only pack files may declare one.
fn nonprofile_manifest_findings(spec: &Spec, report: &mut Report) {
    let raw_text = raw(spec);
    let lines: Vec<&str> = raw_text.lines().collect();
    for heading in MANIFEST_TABLES {
        if lines.iter().any(|l| l.trim() == *heading) {
            report.issues.push(Issue::new(
                "pack_shape",
                spec.intent.id.clone(),
                format!(
                    "manifest table `{heading}` present but frontmatter kind is `{}` not `profile` — a manifest declares a pack, and only pack files may declare one; make the file `kind: profile` or remove the manifest tables",
                    spec.intent.kind
                ),
            ));
        }
    }
}

/// One declared `uses`-column link: activation advisory (with skew and
/// lifecycle naming), or the typed uses-edge orphan finding.
fn uses_edge_findings(
    spec: &Spec,
    link: &crate::spec::Link,
    by_id: &BTreeMap<String, PackInfo>,
    specs: &[Spec],
    orphan_namespaces: &mut std::collections::BTreeSet<String>,
    report: &mut Report,
) {
    match by_id.get(&link.target) {
        Some(pack) => {
            // declared mode: skew advisory (warnings channel, exit 0)
            if let Some(pin) = pack.base_pin {
                let corpus = corpus_revision(specs);
                if pin < corpus {
                    report.warnings.push(Issue::new(
                        "skew_advisory",
                        spec.intent.id.clone(),
                        format!(
                            "pack `{}` pins base Revision {pin}, workspace corpus is at Revision {corpus} — revision skew is advisory (never failing); consider updating the pack's `## Requires` base pin",
                            pack.id
                        ),
                    ));
                }
            }
            // lifecycle naming: draft status / deprecation named
            if pack.lifecycle == "draft" {
                report.warnings.push(Issue::new(
                    "observability",
                    spec.intent.id.clone(),
                    format!(
                        "pack `{}` activated for `{}` (declared uses edge) — draft status: this pack is not yet published, its findings are advisory-first",
                        pack.id, spec.intent.id
                    ),
                ));
            } else if pack.lifecycle == "deprecated" {
                report.warnings.push(Issue::new(
                    "observability",
                    spec.intent.id.clone(),
                    format!(
                        "pack `{}` activated for `{}` (declared uses edge) — deprecated: this pack is deprecated, its vocabulary still checks and findings name the deprecation",
                        pack.id, spec.intent.id
                    ),
                ));
            } else {
                report.warnings.push(Issue::new(
                    "observability",
                    spec.intent.id.clone(),
                    format!(
                        "pack `{}` activated for `{}` (declared uses edge) — declared rules: {}; empty checked-set (no violations to report)",
                        pack.id,
                        spec.intent.id,
                        if pack.vocabulary.is_empty() {
                            "none declared".to_string()
                        } else {
                            format!("{}", pack.vocabulary.len())
                        }
                    ),
                ));
            }
        }
        None => {
            if let Some(ns) = namespace_of(&link.target) {
                orphan_namespaces.insert(ns);
            }
            report.issues.push(Issue::new(
                "orphan_vocabulary",
                spec.intent.id.clone(),
                format!(
                    "orphan vocabulary: `uses` edge targets `{}`, but no `kind: profile` pack with that id is discovered — candidate pack `{}`; remediations: add/enable a `kind: profile` pack file declaring it, or fix the vocabulary (remove or retype the `uses` edge)",
                    link.target, link.target
                ),
            ));
        }
    }
}

/// Vocabulary-triggered orphan collection (specodelic-erd): the "no pack
/// discovered" half of `orphan_vocabulary_labeled` — a
/// pack-qualified (dotted) token in a structured kind/field
/// position whose namespace matches no discovered pack. Prose
/// stays unscanned (a dotted token in prose —
/// `compile.extraction_failure` — is not vocabulary), so the
/// signal stays false-positive-free and files using no pack
/// vocabulary lint byte-identically (no_pack_no_change).
fn vocab_orphan_tokens(
    spec: &Spec,
    pack_namespaces: &std::collections::BTreeSet<String>,
) -> BTreeMap<String, Vec<String>> {
    let mut vocab_orphans: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut scan = |tok: &str, out: &mut BTreeMap<String, Vec<String>>| {
        if let Some(ns) = namespace_of(tok)
            && !pack_namespaces.contains(ns.as_str())
        {
            out.entry(ns).or_default().push(tok.to_string());
        }
    };
    scan(&spec.intent.kind, &mut vocab_orphans);
    for c in &spec.constraints {
        if let Some(k) = c.kind.as_deref() {
            scan(k, &mut vocab_orphans);
        }
        for h in c.cells.keys() {
            scan(h, &mut vocab_orphans);
        }
    }
    for p in &spec.properties {
        if let Some(k) = p.kind.as_deref() {
            scan(k, &mut vocab_orphans);
        }
        for h in p.cells.keys() {
            scan(h, &mut vocab_orphans);
        }
    }
    vocab_orphans
}

/// Emit the collected vocabulary-orphan findings; namespaces whose
/// concrete uses-edge orphan already fired are skipped (no duplicates).
fn emit_vocab_orphan_findings(
    spec: &Spec,
    mut vocab_orphans: BTreeMap<String, Vec<String>>,
    orphan_namespaces: &std::collections::BTreeSet<String>,
    report: &mut Report,
) {
    for (ns, mut tokens) in vocab_orphans {
        if orphan_namespaces.contains(&ns) {
            continue; // the concrete uses-edge finding already names the pack
        }
        tokens.sort();
        tokens.dedup();
        let shown = tokens
            .iter()
            .map(|t| format!("`{t}`"))
            .collect::<Vec<_>>()
            .join(", ");
        report.issues.push(Issue::new(
            "orphan_vocabulary",
            spec.intent.id.clone(),
            format!(
                "orphan vocabulary: pack-qualified token(s) {shown} used by `{}`, but no `kind: profile` pack in namespace `{ns}` is discovered — candidate pack `{ns}.*` (prefix-derived: only the namespace is known); remediations: add/enable a `kind: profile` pack file declaring the vocabulary, or fix the vocabulary (un-qualify or retype the token)",
                spec.intent.id
            ),
        ));
    }
}

/// Implicit activation: vocabulary use (per-pack attribution,
/// overlapping vocabulary activates every declaring pack).
fn implicit_activation_warnings(spec: &Spec, packs: &[PackInfo], report: &mut Report) {
    for pack in packs {
        if spec.intent.id == pack.id {
            continue; // self-exemption
        }
        let raw_text = raw(spec);
        let hits: Vec<&String> = pack
            .vocabulary
            .iter()
            .filter(|tok| contains_word(&raw_text, tok))
            .collect();
        if hits.is_empty() {
            continue;
        }
        let shown = hits
            .iter()
            .map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let status_note = match pack.lifecycle.as_str() {
            "draft" => " — draft status: findings are advisory-first and name it",
            "deprecated" => " — deprecated: findings name the deprecation",
            _ => "",
        };
        report.warnings.push(Issue::new(
            "observability",
            spec.intent.id.clone(),
            format!(
                "pack `{}` activated for `{}` (vocabulary match: {shown}) — empty checked-set (no violations to report){status_note}",
                pack.id, spec.intent.id
            ),
        ));
    }
}

/// The workspace corpus's format revision: the latest Revision heading
/// in `specs/specodelic.md` under the discovery root, else the embedded
/// FORMAT_REVISION.
fn corpus_revision(specs: &[Spec]) -> u32 {
    let root = discovery_root(specs);
    let candidates = [
        root.join("specs").join("specodelic.md"),
        root.join("specodelic.md"),
    ];
    for c in candidates {
        if let Ok(src) = std::fs::read_to_string(&c)
            && let Some(rev) = guide::latest_revision(&src)
        {
            return rev;
        }
    }
    guide::revision_number(guide::FORMAT_REVISION).unwrap_or(0)
}

/// The fiber kinds active for one file (specodelic-ung): the `## Kinds`
/// tokens of every discovered pack the file activates — by a declared
/// `uses` edge or by vocabulary match (advisory-first, same activation
/// rule as [`pack_pass`]). The base∪fiber closed-set walkers in
/// `linter-schema_shape` extend their closed sets with these tokens:
/// the effective set is never narrower than the base set, and a kind no
/// active pack declares stays outside it (the labeled finding fires).
pub fn active_fiber_kinds(spec: &Spec, specs: &[Spec]) -> Vec<String> {
    let declared_uses: std::collections::BTreeSet<&str> = spec
        .links
        .iter()
        .filter(|l| l.column == "uses")
        .map(|l| l.target.as_str())
        .collect();
    let raw_text = raw(spec);
    let mut out = vec![];
    for (_, m) in scan_workspace(specs) {
        let active = declared_uses.contains(m.id.as_str())
            || m.vocabulary()
                .iter()
                .any(|tok| contains_word(&raw_text, tok));
        if active {
            out.extend(m.kinds.iter().cloned());
        }
    }
    out
}

/// `pack_shape` over one pack file: the six manifest tables must be
/// present with well-formed rows; `## Kinds` rows must be pack-qualified
/// (never a base closed-set name — the narrowing rejection).
fn shape_findings(pack: &Spec, report: &mut Report) {
    let raw_text = raw(pack);
    let lines: Vec<&str> = raw_text.lines().collect();
    for heading in MANIFEST_TABLES {
        match table_after(&lines, heading) {
            None => report.issues.push(Issue::new(
                "pack_shape",
                pack.intent.id.clone(),
                format!(
                    "missing manifest table `{heading}` — the pack_shape law requires all six facets (specs/packs.md manifest_six_tables); every pack declares exactly what it introduces"
                ),
            )),
            Some(rows) => {
                if rows.is_empty() {
                    report.issues.push(Issue::new(
                        "pack_shape",
                        pack.intent.id.clone(),
                        format!(
                            "manifest table `{heading}` is empty — a declared facet declares nothing; remove the table or declare its rows"
                        ),
                    ));
                }
                if rows.iter().any(|r| r.len() != 2) {
                    report.issues.push(Issue::new(
                        "pack_shape",
                        pack.intent.id.clone(),
                        format!(
                            "manifest facet `{}` has a row not matching its fixed two-column shape — every manifest table's row shape is fixed (specs/packs.md manifest_six_tables)",
                            heading.trim_start_matches("## ")
                        ),
                    ));
                }
            }
        }
    }
    // narrowing rejection: Kinds rows must be pack-qualified
    for row in table_after(&lines, "## Kinds").unwrap_or_default() {
        if row.len() < 2 {
            continue; // shape finding already emitted above
        }
        let kind = &row[0];
        if BASE_KINDS.contains(&kind.as_str()) || !kind.contains('.') {
            report.issues.push(Issue::new(
                "pack_shape",
                pack.intent.id.clone(),
                format!(
                    "narrowing rejected: pack `## Kinds` row `{kind}` must be pack-qualified (e.g. `{}.tolerance`) and must never name a base closed-set member unqualified — pack vocabulary is fiber-relative, the append-only law forbids narrowing any base set (specs/packs.md base_closed_sets_frozen, append_only_packs)",
                    pack.intent.id
                ),
            ));
        }
    }
}
