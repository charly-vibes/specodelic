//! Library-level mapping tests for `check`'s verify-stage rows
//! (openspec/changes/add-check-judge-verb tasks 1.1): one case per row of
//! design §D3's verify-stage table, constructed directly from the verify
//! gate states so no cargo execution is involved.

use specodelic::audit::AuditClass;
use specodelic::check;
use specodelic::conform::Verdict;
use specodelic::model_check::Outcome;
use specodelic::verify::{
    BlockResult, FileVerdict, ModelGateState, PropertiesGate, PropsGateState,
    Verdict as GateVerdict,
};

fn gate(status: &'static str) -> GateVerdict {
    GateVerdict {
        status,
        message: String::new(),
        hint: String::new(),
    }
}

fn file_verdict(
    properties: PropertiesGate,
    model: ModelGateState,
    status: &'static str,
) -> FileVerdict {
    FileVerdict {
        file: "demo.check_rows.md".to_string(),
        id: "demo.check_rows".to_string(),
        verdict: gate(status),
        properties,
        model,
    }
}

fn block(id: &str, case: Option<&str>, passed: bool, detail: Option<&str>) -> BlockResult {
    BlockResult {
        id: id.to_string(),
        case: case.map(String::from),
        fn_name: format!("{id}_test"),
        passed,
        detail: detail.map(String::from),
    }
}

fn passing(blocks: Vec<BlockResult>) -> PropertiesGate {
    PropertiesGate {
        state: PropsGateState::Pass,
        blocks,
    }
}

fn not_clean(outcome: Outcome, blocking: &[&str]) -> ModelGateState {
    ModelGateState::NotClean {
        outcome,
        blocking_claims: blocking.iter().map(|s| s.to_string()).collect(),
        unchecked_claim_ids: Vec::new(),
    }
}

#[test]
fn passed_block_is_permitted_current_evidence() {
    let fv = file_verdict(
        passing(vec![block("p", None, true, None)]),
        ModelGateState::Clean,
        "verified",
    );
    let rows = check::verify_rows(&fv);
    let row = rows
        .iter()
        .find(|r| r.scenario_id == "verify:prop:p")
        .expect("the passed block emits a row");
    assert_eq!(row.conform_verdict, Verdict::Permitted);
    assert_eq!(row.class, AuditClass::Current);
    assert_eq!(row.evaluated_claim_ids, vec!["p".to_string()]);
}

#[test]
fn failed_block_is_forbidden_contradicted_and_carries_the_detail() {
    let fv = file_verdict(
        PropertiesGate {
            state: PropsGateState::Failed("1 of 1 compiled blocks failed".to_string()),
            blocks: vec![block(
                "p",
                None,
                false,
                Some("shrunk input: 7 (assertion failed)"),
            )],
        },
        ModelGateState::Clean,
        "properties_failed",
    );
    let row = check::verify_rows(&fv)
        .into_iter()
        .find(|r| r.scenario_id == "verify:prop:p")
        .expect("the failed block emits a row");
    assert_eq!(row.conform_verdict, Verdict::Forbidden);
    assert_eq!(row.class, AuditClass::Contradicted);
    assert!(
        row.reason.contains("shrunk input: 7"),
        "the shrunk failing input rides the reason: {}",
        row.reason
    );
}

#[test]
fn law_case_blocks_keep_distinct_keys() {
    let fv = file_verdict(
        passing(vec![
            block("p", Some("identity"), true, None),
            block("p", Some("associativity"), false, Some("shape mismatch")),
        ]),
        ModelGateState::Clean,
        "properties_failed",
    );
    let rows = check::verify_rows(&fv);
    let keys: Vec<&str> = rows.iter().map(|r| r.scenario_id.as_str()).collect();
    assert!(keys.contains(&"verify:prop:p:identity"), "{keys:?}");
    assert!(keys.contains(&"verify:prop:p:associativity"), "{keys:?}");
}

#[test]
fn clean_model_gate_is_one_permitted_row() {
    let fv = file_verdict(passing(vec![]), ModelGateState::Clean, "verified");
    let rows = check::verify_rows(&fv);
    let row = rows
        .iter()
        .find(|r| r.scenario_id == "verify:model")
        .expect("clean gate emits one row");
    assert_eq!(row.conform_verdict, Verdict::Permitted);
    assert_eq!(row.class, AuditClass::Current);
}

#[test]
fn counterexample_claim_is_forbidden_and_named() {
    let fv = file_verdict(
        passing(vec![]),
        not_clean(Outcome::CounterexampleFound, &["ka"]),
        "model_not_clean",
    );
    let row = check::verify_rows(&fv)
        .into_iter()
        .find(|r| r.scenario_id == "verify:model:ka")
        .expect("the blocking claim is named in the key");
    assert_eq!(row.conform_verdict, Verdict::Forbidden);
    assert_eq!(row.class, AuditClass::Contradicted);
}

#[test]
fn exploration_only_is_unknown_unchecked_never_a_pass() {
    let fv = file_verdict(
        passing(vec![]),
        not_clean(Outcome::ExplorationOnly, &["ka"]),
        "model_not_clean",
    );
    let rows = check::verify_rows(&fv);
    let row = rows
        .iter()
        .find(|r| r.scenario_id == "verify:model:ka")
        .expect("the blocking claim is named in the key");
    assert_eq!(row.conform_verdict, Verdict::Unknown);
    assert_eq!(row.class, AuditClass::Unchecked);
    assert!(
        !rows.iter().any(|r| r.reason.contains("inconclusive")),
        "inconclusive is reserved and never spelled"
    );
}

#[test]
fn timed_out_model_run_without_claims_is_one_unknown_row() {
    let fv = file_verdict(
        passing(vec![]),
        not_clean(Outcome::TimedOut, &[]),
        "model_not_clean",
    );
    let row = check::verify_rows(&fv)
        .into_iter()
        .find(|r| r.scenario_id == "verify:model")
        .expect("an aggregate row is emitted");
    assert_eq!(row.conform_verdict, Verdict::Unknown);
    assert_eq!(row.class, AuditClass::Unchecked);
}

#[test]
fn stale_and_missing_model_evidence_emit_no_row() {
    let stale = file_verdict(
        passing(vec![]),
        ModelGateState::Stale {
            detail: "report predates the artifact".to_string(),
        },
        "stale_model_run",
    );
    let missing = file_verdict(
        passing(vec![]),
        ModelGateState::Missing {
            detail: "no report".to_string(),
        },
        "missing_model_run",
    );
    assert!(
        check::verify_rows(&stale)
            .iter()
            .all(|r| !r.scenario_id.starts_with("verify:model"))
    );
    assert!(
        check::verify_rows(&missing)
            .iter()
            .all(|r| !r.scenario_id.starts_with("verify:model"))
    );
}
