//! The claim-coverage section — what `spk check` judged, and what it did
//! not (openspec/changes/add-check-claim-coverage, scope A).
//!
//! Purpose: make the gap countable. Every declared invariant claim of the
//! judged file appears in the section with exactly one class from audit's
//! existing closed set, so an expected behaviour with no evidence behind it
//! is visible instead of invisible.
//! Responsibilities: derive the declared claims from the parsed spec;
//! place each evidence row on a claim by the declared placement rule (the
//! row names the claim, or names a property whose `derives_from` names the
//! claim); fold the placing rows into one class per claim (the strongest
//! among them); account for every row that places no claim in the residual
//! bucket; count the classes; carry the scope statement that says whose
//! evidence the classes describe.
//! Rationale: the section counts knowledge ABSENCE, so it must never
//! overstate what was checked. It therefore places rows only through the
//! corpus's own declared edge — never by name, slug or similarity — it
//! reuses audit's closed class set instead of inventing a vocabulary, and
//! it says out loud that `unchecked` means "no evidence from THIS run's
//! producers", not "no evidence exists" (scope A cannot read the sibling
//! contract surface).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::audit::{AuditClass, AuditReport};
use crate::spec::Spec;

/// The machine-readable id of the coverage scope: the classes describe the
/// evidence THIS run's producers produced. A consumer compares the id, not
/// the prose — the statement beside it is for the reader.
pub const SCOPE_ID: &str = "this_run_producers";

/// The scope statement the section carries in every view.
pub const SCOPE_STATEMENT: &str = "the coverage classes describe the evidence this run's producers produced — evidence outside this run's authority is not read, so `unchecked` means \"no evidence from this run\", never \"no evidence exists\"";

/// The scope the section's classes describe — a stable id for machines and
/// a statement for readers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageScope {
    /// The machine-readable scope id ([`SCOPE_ID`]).
    pub id: String,
    /// The human statement of the same fact.
    pub statement: String,
}

/// One declared claim and the class its evidence carries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimEntry {
    /// The file-qualified claim id — `<file id>.<claim id>`.
    pub claim_id: String,
    /// The claim's class: the strongest among the rows that place it.
    pub class: AuditClass,
}

/// One row that places no claim — accounted for, never dropped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResidualRow {
    /// The row's key (its producer-shaped id).
    pub row_key: String,
    /// The row's own class — the existing closed set, no new spelling.
    pub class: AuditClass,
}

/// The claim-coverage section: every declared invariant claim of the judged
/// file with its class, the rows that placed no claim, the class counts,
/// and the scope the classes describe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimCoverage {
    /// The judged file's id — the prefix of every file-qualified claim id.
    pub file_id: String,
    /// The scope the classes describe.
    pub scope: CoverageScope,
    /// The declared claims, in the corpus's declaration order.
    pub claims: Vec<ClaimEntry>,
    /// The rows that placed no claim.
    pub residuals: Vec<ResidualRow>,
    /// The class counts over the listed claims.
    pub class_counts: BTreeMap<String, usize>,
}

/// The class order the fold uses: the strongest class among the rows that
/// place a claim wins. A contradiction is a live failure and outranks
/// everything; a stale row is a regression and outranks an agreement; an
/// agreement outranks a novelty label (`new` — the row the prior report
/// never saw, whose agreement with the model is not in question); a vacuous
/// placement outranks absence.
pub fn class_rank(class: AuditClass) -> u8 {
    match class {
        AuditClass::Contradicted => 6,
        AuditClass::Stale => 5,
        AuditClass::Current => 4,
        AuditClass::New => 3,
        AuditClass::BindsNothing => 2,
        AuditClass::Unchecked => 1,
    }
}

/// The section for one judged file: the declared claims of the spec, placed
/// by the report's rows.
pub fn coverage(spec: &Spec, report: &AuditReport) -> ClaimCoverage {
    let file_id = spec.intent.id.clone();
    let claims = declared_claims(spec);
    let declared: BTreeSet<&str> = claims.iter().map(String::as_str).collect();
    let properties = property_claims(spec, &file_id, &declared);

    let mut placed: BTreeMap<&str, AuditClass> = BTreeMap::new();
    let mut residuals: Vec<ResidualRow> = Vec::new();

    for record in &report.records {
        let mut row_claims: Vec<&str> = Vec::new();
        for name in named_ids(record) {
            if let Some(claim) = resolve_claim(&name, &file_id, &declared) {
                row_claims.push(claim);
            } else if let Some(derived) = properties.get(name.as_str()) {
                row_claims.extend(derived.iter().copied());
            }
        }
        row_claims.sort_unstable();
        row_claims.dedup();
        if row_claims.is_empty() {
            residuals.push(ResidualRow {
                row_key: record.scenario_id.clone(),
                class: record.class,
            });
        } else {
            for claim in row_claims {
                let slot = placed.entry(claim).or_insert(record.class);
                if class_rank(record.class) > class_rank(*slot) {
                    *slot = record.class;
                }
            }
        }
    }

    let entries: Vec<ClaimEntry> = claims
        .iter()
        .map(|id| ClaimEntry {
            claim_id: format!("{file_id}.{id}"),
            class: placed
                .get(id.as_str())
                .copied()
                .unwrap_or(AuditClass::Unchecked),
        })
        .collect();

    let mut class_counts: BTreeMap<String, usize> = BTreeMap::new();
    for entry in &entries {
        *class_counts
            .entry(entry.class.as_str().to_string())
            .or_insert(0) += 1;
    }

    ClaimCoverage {
        file_id,
        scope: CoverageScope {
            id: SCOPE_ID.to_string(),
            statement: SCOPE_STATEMENT.to_string(),
        },
        claims: entries,
        residuals,
        class_counts,
    }
}

/// The section's verdict-line summary — the counts beside the existing
/// evidence counts, so a reader sees them where the verdict is read.
pub fn verdict_summary(coverage: &ClaimCoverage) -> String {
    let mut classes: Vec<String> = Vec::new();
    for (class, count) in &coverage.class_counts {
        classes.push(format!("{class} {count}"));
    }
    if classes.is_empty() {
        return "claims: none".to_string();
    }
    format!("claims: {}", classes.join(", "))
}

/// The section's own lines — the claims it lists and the rows it could not
/// place, one per line.
pub fn section_lines(coverage: &ClaimCoverage) -> Vec<String> {
    let mut lines = vec![format!("  claim_coverage: {}", coverage.scope.statement)];
    for entry in &coverage.claims {
        lines.push(format!("    {}: {}", entry.claim_id, entry.class.as_str()));
    }
    for residual in &coverage.residuals {
        lines.push(format!(
            "    residual {}: {} — places no declared claim",
            residual.row_key,
            residual.class.as_str()
        ));
    }
    lines
}

/// The declared invariant claims of the file, in declaration order. Only
/// invariants are listed: the other constraint kinds assert emissions and
/// judgements rather than behaviours evidence can contradict, and no
/// producer places a row on them.
fn declared_claims(spec: &Spec) -> Vec<String> {
    spec.constraints
        .iter()
        .filter(|row| row.kind.as_deref() == Some("invariant"))
        .map(|row| row.id.clone())
        .collect()
}

/// The property → claim map: each declared property's `derives_from` edge,
/// resolved to the declared claims of this file. One hop, through the
/// corpus's own declared edge — never a name, slug or similarity.
fn property_claims<'a>(
    spec: &Spec,
    file_id: &str,
    declared: &BTreeSet<&'a str>,
) -> BTreeMap<String, Vec<&'a str>> {
    let mut map: BTreeMap<String, Vec<&'a str>> = BTreeMap::new();
    for property in &spec.properties {
        let cell = property
            .cells
            .get("derives_from")
            .map(String::as_str)
            .unwrap_or("");
        let mut claims: Vec<&'a str> = Vec::new();
        for target in cell_targets(cell) {
            if let Some(claim) = resolve_claim(&target, file_id, declared) {
                claims.push(claim);
            }
        }
        claims.sort_unstable();
        claims.dedup();
        map.insert(property.id.clone(), claims);
    }
    map
}

/// The ids a row names: the ids it carries, plus the id its producer-shaped
/// key names (a model row's key names the claim it could not verify, a
/// property row's key names the property, a binding row's key names its
/// owner).
fn named_ids(record: &crate::audit::AuditRecord) -> Vec<String> {
    let mut ids = record.evaluated_claim_ids.clone();
    for prefix in ["verify:prop:", "verify:model:", "bind:"] {
        if let Some(rest) = record.scenario_id.strip_prefix(prefix) {
            let id = rest.split(':').next().unwrap_or("").trim();
            if !id.is_empty() {
                ids.push(id.to_string());
            }
            break;
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

/// The declared claim an id names: the bare id, or the file-qualified form
/// the corpus's links use. Nothing else resolves — a resemblance is not a
/// claim.
fn resolve_claim<'a>(name: &str, file_id: &str, declared: &BTreeSet<&'a str>) -> Option<&'a str> {
    if let Some(id) = declared.get(name) {
        return Some(id);
    }
    let qualified = format!("{file_id}.");
    if let Some(rest) = name.strip_prefix(qualified.as_str())
        && let Some(id) = declared.get(rest)
    {
        return Some(*id);
    }
    None
}

/// The claim ids a `derives_from` cell names: its wiki-link targets, or —
/// when the cell carries no link — the bare id it writes.
fn cell_targets(cell: &str) -> Vec<String> {
    let mut targets: Vec<String> = Vec::new();
    let mut rest = cell;
    while let Some(start) = rest.find("[[") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("]]") else { break };
        targets.push(after[..end].trim().to_string());
        rest = &after[end + 2..];
    }
    if targets.is_empty() {
        let bare = cell.trim().trim_matches('`').trim();
        if !bare.is_empty() {
            targets.push(bare.to_string());
        }
    }
    targets
}
