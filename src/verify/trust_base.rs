//! The trust ledger (add-verification-trust-ledger): what a verification
//! verdict rests on but did not check. One pure derivation over the corpus
//! rows, carried by every report surface, never stored as truth.

use serde::{Deserialize, Serialize};

use super::claim_classification;

// ---------------------------------------------------------------------------
// The trust ledger (add-verification-trust-ledger): what a verdict rests on
// but did not check — derived once from the corpus rows, carried by every
// report surface, never stored as truth.
// ---------------------------------------------------------------------------

/// The trust base's own version, carried inside the persisted trust base.
/// Deliberately NOT [`CLAIM_SCHEMA_VERSION`]: the ledger is a payload
/// addition, not a claim-semantics change — bumping the claim schema would
/// invalidate every stored report corpus-wide for no claim-meaning drift.
pub const TRUST_BASE_SCHEMA_VERSION: u32 = 1;

/// One trust-base member's kind. The two closed kind sets read as one
/// enum plus [`TrustMemberKind::is_assumed`]: `assumed` admits only
/// `prose_only_guard`; every other kind is an unverified claim
/// (`trust_base_partitioned`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustMemberKind {
    /// A prose-only invariant Constraint — asserted but unchecked (the
    /// same classification the unchecked claim set derives from).
    ProseOnlyInvariant,
    /// A row whose `satisfies` points at a foreign `extension_point` —
    /// conformance the corpus declares but nothing checks.
    ForeignExtensionPoint,
    /// An `advisory` Constraint — declared, shape-checked, gating nothing.
    AdvisoryConstraint,
    /// A transition whose guard is prose and cites nothing evaluable —
    /// relied upon but unchecked.
    ProseOnlyGuard,
}

impl TrustMemberKind {
    /// The closed kind-set table (2.6): the kinds each array admits, in
    /// ONE place — the derivation routes members through
    /// [`TrustMemberKind::is_assumed`], the JSON serializer routes the
    /// spellings through this enum's serde renames, and the human
    /// renderer reads kinds from the envelope value. No view keeps its
    /// own kind list.
    pub const UNVERIFIED_CLAIM_KINDS: [TrustMemberKind; 3] = [
        TrustMemberKind::ProseOnlyInvariant,
        TrustMemberKind::ForeignExtensionPoint,
        TrustMemberKind::AdvisoryConstraint,
    ];
    /// The `assumed` array's closed set: relied-upon-but-unchecked kinds.
    pub const ASSUMED_KINDS: [TrustMemberKind; 1] = [TrustMemberKind::ProseOnlyGuard];

    /// The closed set test: which array the kind belongs to. A kind is in
    /// exactly one set, so a member lands in exactly one array.
    pub fn is_assumed(self) -> bool {
        Self::ASSUMED_KINDS.contains(&self)
    }
}

/// One entry of the trust base: the row (or transition) id, its kind, and
/// the file that declares it — corpus-scoped, so a member may come from
/// any file in the scope and names its source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustMember {
    pub id: String,
    pub kind: TrustMemberKind,
    pub source: String,
}

/// The trust base of a verdict: two disjoint arrays, always serialized,
/// empty included — an omitted ledger is not an empty one
/// (`empty_trust_base_meaning`). Run metadata (the model-check bound, the
/// artifact digest) is deliberately absent: the ledger is a function of
/// the corpus, never of the run's parameters (`run_limits_excluded`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TrustBase {
    pub trust_base_schema_version: u32,
    pub unverified_claims: Vec<TrustMember>,
    pub assumed: Vec<TrustMember>,
}

/// The `[[wiki-link]]` inner texts of a structured cell, in document
/// order — the shared citation scan for the guard classification.
fn citation_targets(cell: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = cell;
    while let Some(i) = rest.find("[[") {
        let after = &rest[i + 2..];
        match after.find("]]") {
            Some(j) => {
                out.push(&after[..j]);
                rest = &after[j + 2..];
            }
            None => break,
        }
    }
    out
}

/// Resolve a link target against the corpus the way `lint`'s index does
/// (last-dot split first): the qualified `(file id, row id)` plus the
/// row's kind cell — `"state"` for a State bullet, the section anchors
/// (`model.state` / `model.transition`) type as a state.
fn trust_resolve_target<'a>(
    specs: &'a [crate::spec::Spec],
    source_file: &'a str,
    target: &'a str,
) -> Option<(&'a str, String, String)> {
    if target == "model.state" || target == "model.transition" {
        return Some((source_file, target.to_string(), "state".to_string()));
    }
    // (file id, rest) candidates, last dot first; the bare arm last.
    let mut candidates: Vec<(&str, &str)> = target
        .match_indices('.')
        .map(|(i, _)| (&target[..i], &target[i + 1..]))
        .collect();
    candidates.reverse();
    candidates.push((source_file, target));
    for (file_id, rest) in candidates {
        let Some(spec) = specs.iter().find(|s| s.intent.id == file_id) else {
            continue;
        };
        // Member arms (`file.row.member`) canonicalize to the row.
        let row_id = rest.split('.').next().unwrap_or(rest);
        for row in &spec.constraints {
            if row.id == row_id {
                let kind = row.cells.get("kind").cloned().unwrap_or_default();
                return Some((file_id, row_id.to_string(), kind));
            }
        }
        if spec.states.iter().any(|s| s.id == row_id) {
            return Some((file_id, row_id.to_string(), "state".to_string()));
        }
    }
    None
}

/// Whether a transition's guard resolves to a condition the model-check
/// citation algebra can evaluate: a citation to an evaluable constraint
/// (executable invariant, citation expression, or kernel binding — the
/// same three ModelIr maps the backend evaluates) or to a State (the
/// "has reached state X" progress marker). Prose with no resolvable
/// condition is the assumption `prose_only_guard` names.
fn guard_resolves(
    specs: &[crate::spec::Spec],
    guard_cell: &str,
    source_file: &str,
    evaluable: &std::collections::BTreeSet<(String, String)>,
    states: &std::collections::BTreeSet<(String, String)>,
) -> bool {
    citation_targets(guard_cell).iter().any(|target| {
        trust_resolve_target(specs, source_file, target).is_some_and(|(file, row, kind)| {
            let key = (file.to_string(), row);
            (kind == "state" && states.contains(&key)) || evaluable.contains(&key)
        })
    })
}

/// A corpus-qualified id: (file id, row id) — the shape the citation
/// algebra's evaluable-constraint and State sets use.
type QualifiedSet = std::collections::BTreeSet<(String, String)>;

/// The evaluable-constraint ids and the State ids of the whole corpus,
/// qualified by file — the same sets the model-check backend's citation
/// algebra reads off the ModelIr (slice 3: the classification reads the
/// compiled IR, not the guard's text).
fn evaluable_and_states(specs: &[crate::spec::Spec]) -> (QualifiedSet, QualifiedSet) {
    let mut evaluable = std::collections::BTreeSet::new();
    let mut states = std::collections::BTreeSet::new();
    for spec in specs {
        let file = spec.intent.id.as_str();
        let ir = crate::compile::extract_model_ir(spec);
        for id in ir.evaluable_constraint_ids() {
            evaluable.insert((file.to_string(), id.to_string()));
        }
        for state in &spec.states {
            states.insert((file.to_string(), state.id.clone()));
        }
    }
    (evaluable, states)
}

/// Every ledger member the corpus declares, unsorted — the per-spec scan:
/// prose-only invariants, advisory rows, foreign `extension_point`
/// `satisfies` links, and prose-only transition guards.
fn corpus_members(
    specs: &[crate::spec::Spec],
    evaluable: &std::collections::BTreeSet<(String, String)>,
    states: &std::collections::BTreeSet<(String, String)>,
) -> Vec<TrustMember> {
    let mut members: Vec<TrustMember> = Vec::new();
    for spec in specs {
        let file = spec.intent.id.clone();
        // Prose-only invariants: the SAME unchecked set the claim path
        // derives — the ledger cannot disagree with it.
        for id in &claim_classification(spec).unchecked {
            members.push(TrustMember {
                id: id.clone(),
                kind: TrustMemberKind::ProseOnlyInvariant,
                source: file.clone(),
            });
        }
        for row in &spec.constraints {
            if row.cells.get("kind").map(String::as_str) == Some("advisory") {
                members.push(TrustMember {
                    id: row.id.clone(),
                    kind: TrustMemberKind::AdvisoryConstraint,
                    source: file.clone(),
                });
            }
        }
        for link in &spec.links {
            if link.column != "satisfies" {
                continue;
            }
            if let Some((target_file, _row, kind)) =
                trust_resolve_target(specs, &file, &link.target)
                && target_file != file
                && kind == "extension_point"
            {
                members.push(TrustMember {
                    id: link.source.clone(),
                    kind: TrustMemberKind::ForeignExtensionPoint,
                    source: file.clone(),
                });
            }
        }
        for transition in &spec.transitions {
            if let Some(guard) = &transition.guard
                && !guard_resolves(specs, guard, &file, evaluable, states)
            {
                members.push(TrustMember {
                    id: transition.id.clone(),
                    kind: TrustMemberKind::ProseOnlyGuard,
                    source: file.clone(),
                });
            }
        }
    }
    members
}

/// Derive the trust base from the corpus rows — the ONE derivation, pure
/// in the parsed specs: no CLI state, no filesystem access, no randomness.
/// Members sort by `(kind, source file id, row id)` so the same corpus in
/// a different file order yields an identical ledger. The prose-only
/// invariant members reuse [`claim_classification`]'s unchecked set — one
/// prose-only detector, never two (`trust_base_derived`).
pub fn derive_trust_base(specs: &[crate::spec::Spec]) -> TrustBase {
    let (evaluable, states) = evaluable_and_states(specs);
    let mut members = corpus_members(specs, &evaluable, &states);
    members.sort_by(|a, b| {
        (&a.kind, a.source.as_str(), a.id.as_str()).cmp(&(
            &b.kind,
            b.source.as_str(),
            b.id.as_str(),
        ))
    });
    // dedup_by drops only CONSECUTIVE equal members; the sort above is
    // total on (kind, source, id), so equal members are guaranteed
    // adjacent. A row with several satisfies links to foreign extension
    // points emits duplicate members pre-dedup — exactly one survives.
    members.dedup_by(|a, b| a.id == b.id && a.kind == b.kind && a.source == b.source);
    let mut base = TrustBase {
        trust_base_schema_version: TRUST_BASE_SCHEMA_VERSION,
        unverified_claims: Vec::new(),
        assumed: Vec::new(),
    };
    for member in members {
        if member.kind.is_assumed() {
            base.assumed.push(member);
        } else {
            base.unverified_claims.push(member);
        }
    }
    base
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::model_check::{self, Outcome, RunReport};
    use crate::test_panic::UnwrapPanic;

    use super::*;

    fn spec_of(text: &str) -> crate::spec::Spec {
        crate::spec::parse_str(text).unwrap_p()
    }

    const FIXTURE_TRUST: &str = "---\nid: tbase\nkind: intent\nstatement: \"THE tbase SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `the prose invariant holds` | [[tbase]] |\n| c2 | invariant | `**rust:** x > 0` | [[tbase]] |\n| a1 | advisory | `best effort` | [[tbase]] |\n\n## Model\n\n### States\n\n- s1\n- s2\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t_prose | s1 | s2 | the user is signed in |\n| t_rust | s1 | s2 | [[tbase.c2]] |\n| t_state | s2 | s1 | [[tbase.s1]] |\n| t_prose_cite | s2 | s1 | [[tbase.c1]] |\n";

    /// A file whose Constraint row `satisfies` a FOREIGN extension point,
    /// plus the foreign host that declares it.
    const FIXTURE_SATISFIES: &str = "---\nid: thext\nkind: intent\nstatement: \"THE thext SHALL conform\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | satisfies |\n|----|------|------|-----------|-----------|\n| c_s | effect | `the conformance claim` | [[thext]] | [[thost.c_point]] |\n| c_inv | invariant | `the prose invariant holds` | [[thext]] |  |\n\n## Model\n\n### States\n\n- s1\n- s2\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s2 | the user is signed in |\n";

    const FIXTURE_EXT_HOST: &str = "---\nid: thost\nkind: intent\nstatement: \"THE thost SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c_point | extension_point | `the host extends` | [[thost]] |\n\n## Model\n\n### States\n\n- h1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t_h | h1 | h1 | the host is ready |\n";

    fn trust_member<'a>(base: &'a TrustBase, id: &str) -> Option<&'a TrustMember> {
        base.unverified_claims
            .iter()
            .chain(base.assumed.iter())
            .find(|m| m.id == id)
    }

    /// 1.1 `trust_base_partitioned`: the prose-only invariant is an
    /// unverified claim, the prose guard is an assumption, and no member
    /// appears in both arrays.
    #[test]
    fn trust_base_partitioned() {
        let spec = spec_of(FIXTURE_TRUST);
        let base = derive_trust_base(std::slice::from_ref(&spec));
        let inv = trust_member(&base, "c1").expect("prose-only invariant listed");
        assert_eq!(inv.kind, TrustMemberKind::ProseOnlyInvariant);
        assert_eq!(inv.source, "tbase");
        let guard = trust_member(&base, "t_prose").expect("prose guard listed");
        assert_eq!(guard.kind, TrustMemberKind::ProseOnlyGuard);
        assert_eq!(guard.source, "tbase");
        for member in &base.unverified_claims {
            assert!(
                !base.assumed.contains(member),
                "member {} in both arrays",
                member.id
            );
        }
        assert_eq!(base.trust_base_schema_version, TRUST_BASE_SCHEMA_VERSION);
    }

    /// 1.2 `trust_base_derived`: the unchecked set and the
    /// `prose_only_invariant` members come from one classification — per
    /// file, the id sets are identical.
    #[test]
    fn prose_only_invariants_match_unchecked_set() {
        let spec = spec_of(FIXTURE_TRUST);
        let class = claim_classification(&spec);
        let base = derive_trust_base(std::slice::from_ref(&spec));
        let mut ledger_ids: Vec<&str> = base
            .unverified_claims
            .iter()
            .filter(|m| m.kind == TrustMemberKind::ProseOnlyInvariant)
            .map(|m| m.id.as_str())
            .collect();
        ledger_ids.sort();
        assert_eq!(
            ledger_ids,
            class
                .unchecked
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
        );
    }

    /// 1.3 `empty_trust_base_meaning` (serialization half): an empty
    /// ledger serializes with both arrays present, never omitted.
    #[test]
    fn empty_trust_base_serializes_both_arrays() {
        let empty = derive_trust_base(&[]);
        assert!(empty.unverified_claims.is_empty() && empty.assumed.is_empty());
        assert_eq!(empty.trust_base_schema_version, TRUST_BASE_SCHEMA_VERSION);
        let Ok(json) = serde_json::to_value(&empty) else {
            panic!("TrustBase serializes")
        };
        assert_eq!(json["trust_base_schema_version"], 1);
        assert!(json["unverified_claims"].is_array());
        assert!(json["assumed"].is_array());
    }

    /// 1.4 the sort order `(kind, source file id, row id)` holds across
    /// the concatenated arrays.
    #[test]
    fn trust_members_sort_by_kind_source_id() {
        let thext = spec_of(FIXTURE_SATISFIES);
        let thost = spec_of(FIXTURE_EXT_HOST);
        let base = derive_trust_base(&[thost, thext]);
        let all: Vec<&TrustMember> = base
            .unverified_claims
            .iter()
            .chain(base.assumed.iter())
            .collect();
        assert!(all.len() >= 3, "the corpus carries several members");
        for pair in all.windows(2) {
            let a = (&pair[0].kind, &pair[0].source, &pair[0].id);
            let b = (&pair[1].kind, &pair[1].source, &pair[1].id);
            assert!(a <= b, "members out of order: {a:?} before {b:?}");
        }
    }

    /// 2.1 a `satisfies` row pointing at a foreign `extension_point` is
    /// an unverified claim, never an assumption.
    #[test]
    fn foreign_extension_point_is_unverified_claim() {
        let thext = spec_of(FIXTURE_SATISFIES);
        let thost = spec_of(FIXTURE_EXT_HOST);
        let base = derive_trust_base(&[thext, thost]);
        let member = trust_member(&base, "c_s").expect("satisfies row listed");
        assert_eq!(member.kind, TrustMemberKind::ForeignExtensionPoint);
        assert_eq!(member.source, "thext");
        assert!(!base.assumed.iter().any(|m| m.id == "c_s"));
    }

    /// 2.2 an `advisory` Constraint is an unverified claim.
    #[test]
    fn advisory_constraint_is_unverified_claim() {
        let spec = spec_of(FIXTURE_TRUST);
        let base = derive_trust_base(std::slice::from_ref(&spec));
        let member = trust_member(&base, "a1").expect("advisory listed");
        assert_eq!(member.kind, TrustMemberKind::AdvisoryConstraint);
        assert_eq!(member.source, "tbase");
    }

    /// 2.3 `run_limits_excluded`: the bound and the artifact digest are
    /// run metadata beside the verdict — the report payload carries them,
    /// and neither value appears in either trust array.
    #[test]
    fn run_bound_and_digest_stay_out_of_the_ledger() {
        let spec = spec_of(FIXTURE_TRUST);
        let base = derive_trust_base(std::slice::from_ref(&spec));
        let report = RunReport {
            backend: model_check::Backend {
                engine: "test".into(),
                version: "0".into(),
            },
            bound: model_check::Bound::default(),
            outcome: Outcome::ExplorationOnly,
            invariants_checked: vec![],
            invariant_statuses: vec![],
            violated_invariant_id: None,
            trace: None,
            artifact_sha256: "cafebabe".into(),
            states_explored: 0,
        };
        let Ok(json) = serde_json::to_value(&report) else {
            panic!("RunReport serializes")
        };
        assert!(json.get("bound").is_some(), "bound is run metadata");
        assert!(
            json.get("artifact_sha256").is_some(),
            "digest is run metadata"
        );
        let Ok(ledger) = serde_json::to_string(&base) else {
            panic!("ledger serializes")
        };
        assert!(
            !ledger.contains("cafebabe") && !ledger.contains("max_depth"),
            "run metadata leaked into the ledger: {ledger}"
        );
    }

    /// 2.4 `trust_base_corpus_scoped`: a member from the other file in the
    /// scope carries that file's id — the ledger follows the corpus.
    #[test]
    fn ledger_is_corpus_scoped() {
        let thext = spec_of(FIXTURE_SATISFIES);
        let thost = spec_of(FIXTURE_EXT_HOST);
        let base = derive_trust_base(&[thext, thost]);
        let host_guard = base
            .assumed
            .iter()
            .find(|m| m.id == "t_h")
            .expect("foreign file's prose guard listed");
        assert_eq!(host_guard.source, "thost");
        // And the ledger of the citing file changes with the corpus: the
        // extension point member names the foreign conformance.
        assert!(trust_member(&base, "c_s").is_some());
    }

    /// 3.1 a guard the citation algebra resolves — citing an evaluable
    /// (`**rust:**`) constraint, or a State — is absent from `assumed`; a
    /// prose guard, and one citing only a prose-only invariant, is present.
    #[test]
    fn resolved_guards_are_not_assumptions() {
        let spec = spec_of(FIXTURE_TRUST);
        let base = derive_trust_base(std::slice::from_ref(&spec));
        assert!(
            !base.assumed.iter().any(|m| m.id == "t_rust"),
            "a guard citing an executable invariant resolves"
        );
        assert!(
            !base.assumed.iter().any(|m| m.id == "t_state"),
            "a State progress marker resolves"
        );
        assert!(
            base.assumed.iter().any(|m| m.id == "t_prose"),
            "prose with no citation is the assumption"
        );
        assert!(
            base.assumed.iter().any(|m| m.id == "t_prose_cite"),
            "citing a prose-only invariant resolves to nothing evaluable"
        );
    }

    /// 3.2/3.3 the guard classification reads the compiled ModelIr's
    /// evaluable sets, not the guard's text: removing the executable
    /// fragment from `c2` demotes `t_rust` to an assumption even though
    /// the guard text (`[[tbase.c2]]`) is unchanged.
    /// 2.6 one kind-set table: the two closed sets are declared once in
    /// this module, partition every kind exactly once, and serialize
    /// with the layer's kind spellings — no view keeps its own list.
    #[test]
    fn kind_table_partitions_every_kind_once() {
        let all = [
            TrustMemberKind::ProseOnlyInvariant,
            TrustMemberKind::ForeignExtensionPoint,
            TrustMemberKind::AdvisoryConstraint,
            TrustMemberKind::ProseOnlyGuard,
        ];
        let mut table: Vec<TrustMemberKind> = TrustMemberKind::UNVERIFIED_CLAIM_KINDS
            .iter()
            .chain(TrustMemberKind::ASSUMED_KINDS.iter())
            .copied()
            .collect();
        table.sort();
        let mut sorted_all = all.to_vec();
        sorted_all.sort();
        assert_eq!(
            table, sorted_all,
            "the table covers every kind exactly once"
        );
        for kind in all {
            assert_eq!(
                kind.is_assumed(),
                TrustMemberKind::ASSUMED_KINDS.contains(&kind),
                "is_assumed reads the table"
            );
        }
        // The spellings every view consumes (serde is the JSON table).
        let spellings: Vec<String> = all
            .iter()
            .map(|k| {
                serde_json::to_value(k)
                    .unwrap_p()
                    .as_str()
                    .unwrap_p()
                    .to_string()
            })
            .collect();
        assert_eq!(
            spellings,
            [
                "prose_only_invariant",
                "foreign_extension_point",
                "advisory_constraint",
                "prose_only_guard",
            ]
        );
    }

    #[test]
    fn guard_classification_reads_the_model_ir() {
        let demoted = FIXTURE_TRUST.replace("**rust:** x > 0", "the prose invariant two holds");
        let spec = spec_of(&demoted);
        let base = derive_trust_base(std::slice::from_ref(&spec));
        assert!(
            base.assumed.iter().any(|m| m.id == "t_rust"),
            "without an evaluable c2, the citing guard is an assumption"
        );
    }
}
