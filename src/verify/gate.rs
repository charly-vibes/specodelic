//! The stored-report trust gate (add-verification-trust-ledger): the
//! ledger's version check, its live recomputation-and-compare, and the
//! write-side report shape that attaches the ledger. Split from
//! `verify.rs` (the file_lines ratchet); the trust-base types themselves
//! stay in `trust_base.rs`.

use super::trust_base::{TRUST_BASE_SCHEMA_VERSION, TrustBase, derive_trust_base};
use super::{CLAIM_SCHEMA_VERSION, QualifiedClaim};
use crate::model_check::RunReport;

/// The trust base version gate (trust_base_versioned): a stored report
/// without a trust base, or carrying an unrecognized
/// `trust_base_schema_version`, is stale — an absent ledger is never
/// read as an empty one, and stored reports are never upgraded. Returns
/// the stored ledger (the caller compares it against the live
/// recomputation) or the stale detail.
pub(crate) fn stored_trust_base_gate(
    raw: &serde_json::Value,
) -> Result<&serde_json::Value, String> {
    let Some(stored_trust_base) = raw.get("trust_base") else {
        return Err("the stored report carries no trust base — an absent ledger is never an empty one; rerun model-check".to_string());
    };
    match stored_trust_base
        .get("trust_base_schema_version")
        .and_then(|v| v.as_u64())
    {
        Some(v) if v == TRUST_BASE_SCHEMA_VERSION as u64 => Ok(stored_trust_base),
        Some(other) => Err(format!(
            "the stored report's trust base schema version {other} is not supported (current {TRUST_BASE_SCHEMA_VERSION}) — stored reports are never upgraded or repaired; rerun model-check"
        )),
        None => Err(
            "the stored trust base carries no trust_base_schema_version — rerun model-check"
                .to_string(),
        ),
    }
}

/// The trust ledger folds into the SAME agreement comparison
/// (`assurance_views_agree` / `trust_base_derived`): one recomputation of
/// [`derive_trust_base`] over the live corpus, compared against the
/// stored ledger — never a second agreement check, and the stored ledger
/// is never the source of truth. A disagreement uses the same rerun hint
/// the unchecked-set comparison uses, and the stored report is left
/// unmodified.
pub(crate) fn stored_trust_base_agrees(
    stored: &serde_json::Value,
    specs: &[crate::spec::Spec],
) -> Result<(), String> {
    let live_trust_base =
        serde_json::to_value(derive_trust_base(specs)).expect("the derived ledger serializes");
    if *stored != live_trust_base {
        return Err("the stored trust_base does not match the live recomputation from the current corpus — rerun model-check over the same file set".to_string());
    }
    Ok(())
}

/// The write-side report shape (D3): the run report's serialized JSON with
/// the claim-freshness fields injected — `claim_schema_version`, qualified
/// `claims`, `expected_claim_ids`, `unchecked_claim_ids`, and
/// `scope_sha256` — plus the derived `trust_base` (always serialized,
/// empty included). The ONE place these fields are attached to a
/// persisted report (the field values themselves come from
/// `report_claim_fields` and [`derive_trust_base`]); verify recomputes
/// all of them from the live inputs and never trusts the stored values.
pub fn fresh_report_json(
    report: &RunReport,
    claims: &[QualifiedClaim],
    expected_claim_ids: &[String],
    unchecked_claim_ids: &[String],
    scope_sha256: &str,
    trust_base: &TrustBase,
) -> serde_json::Value {
    let mut json = serde_json::to_value(report).expect("RunReport serializes");
    let obj = json
        .as_object_mut()
        .expect("RunReport serializes to an object");
    obj.insert(
        "claim_schema_version".into(),
        serde_json::json!(CLAIM_SCHEMA_VERSION),
    );
    obj.insert(
        "claims".into(),
        serde_json::to_value(claims).expect("claims serialize"),
    );
    obj.insert(
        "expected_claim_ids".into(),
        serde_json::json!(expected_claim_ids),
    );
    obj.insert(
        "unchecked_claim_ids".into(),
        serde_json::json!(unchecked_claim_ids),
    );
    obj.insert("scope_sha256".into(), serde_json::json!(scope_sha256));
    obj.insert(
        "trust_base".into(),
        serde_json::to_value(trust_base).expect("the trust base serializes"),
    );
    json
}
