// (split from tests/cli/model_check.rs — the file_lines ratchet; the
// trust-ledger slice-4 CLI tests: version, scope binding, and the
// three views (tasks 4.1–4.9, plus the 2.6 kind table).

use super::model_check::{
    claim_compile, claim_mc_json, claim_props, claim_report, claim_verify_json, drop_props,
    mc_batch, read_check_report, scope_compile, verify_batch, write_check_report, write_claim_spec,
    write_scope_pair,
};
use super::spk;
use crate::common::UnwrapPanic;

// ---- add-verification-trust-ledger slice 4: version, scope binding, ----
// ---- and the three views (tasks 4.1–4.9, plus the 2.6 kind table)   ----
//
// The trust ledger is derived, versioned, and scope-bound (layer.md
// trust_base_derived / trust_base_versioned / trust_base_views_agree):
// a stored report's ledger is never the source of truth — verify
// recomputes it from the live corpus and refuses a mismatched, absent
// or unrecognized-version ledger as stale with the same rerun hint the
// unchecked-set comparison uses, and the ledger rides CLI JSON, human
// output, and the persisted report identically.

/// A trust-ledger fixture: one verified Rust claim (ka), one prose-only
/// invariant (kp), one advisory Constraint (adv), and a transition guard
/// citing the prose-only row — an assumption the citation algebra cannot
/// resolve. Coverage rows for every constraint keep the file lint-clean.
fn write_trust_spec(path: &std::path::Path, id: &str) {
    let rows = format!(
        "| ka | invariant | `**rust:** state != \"blackhole\"` | [[{id}]] |\n\
         | kp | invariant | prose invariant the corpus never checks | [[{id}]] |\n\
         | adv | advisory | `advisory guidance only` | [[{id}]] |\n"
    );
    let props = claim_props(id, &["ka", "kp", "adv"]);
    write_claim_spec(path, id, &rows, &props, "kp");
}

/// The trust ledger of [`write_trust_spec`] as (array, id, kind, source)
/// triples — unverified claims first (kind-sorted), then assumptions.
fn expected_trust_ledger(id: &str) -> serde_json::Value {
    serde_json::json!({
        "trust_base_schema_version": 1,
        "unverified_claims": [
            {"id": "kp", "kind": "prose_only_invariant", "source": id},
            {"id": "adv", "kind": "advisory_constraint", "source": id},
        ],
        "assumed": [
            {"id": "t", "kind": "prose_only_guard", "source": id},
        ],
    })
}

/// 4.5 (+ 4.7's seams): CLI JSON, human output, and the saved report
/// name the same members with the same kinds — no view reports an empty
/// trust base while another names an entry.
#[test]
fn trust_ledger_rides_every_view_and_the_persisted_report() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("tlv.md");
    write_trust_spec(&spec, "tlv");
    let out = td.path().join("out");
    claim_compile(spec.to_str().unwrap_p(), &out);
    let (code, json) = claim_mc_json(spec.to_str().unwrap_p(), &out);
    assert_eq!(code, Some(0));
    let checked = &json["data"]["checked"][0];
    let tb = &checked["trust_base"];
    assert_eq!(
        tb,
        &expected_trust_ledger("tlv"),
        "CLI JSON carries the derived ledger: {checked}"
    );

    // The persisted report: the same ledger, same members, same kinds.
    let report = claim_report(&out, "tlv");
    assert_eq!(report["trust_base"], *tb, "persisted report: {report}");

    // Verify's view: the recomputed ledger rides the entry, and the
    // model gate does NOT refuse the stored ledger it agrees with
    // (props dropped so the properties gate is deterministically
    // missing_artifact — this fixture pins the model gate).
    drop_props(&out, &["tlv"]);
    let (vcode, vjson) = claim_verify_json(spec.to_str().unwrap_p(), &out);
    assert_eq!(vcode, Some(1));
    let entry = vjson["data"]["blocked"]
        .as_array()
        .unwrap_p()
        .iter()
        .find(|e| e["id"] == "tlv")
        .unwrap_p();
    assert_eq!(
        entry["trust_base"],
        expected_trust_ledger("tlv"),
        "verify's entry carries the same ledger: {entry}"
    );
    assert_eq!(entry["model"]["state"], "clean", "{entry}");

    // Human output names the same members with the same kinds.
    let human = spk()
        .args([
            "model-check",
            spec.to_str().unwrap_p(),
            "--human",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    let stdout = String::from_utf8(human.stdout).unwrap_p();
    for needle in [
        "advisory_constraint",
        "prose_only_invariant",
        "prose_only_guard",
        "adv",
        "kp",
        "t",
    ] {
        assert!(stdout.contains(needle), "human names `{needle}`: {stdout}");
    }
    let v_human = spk()
        .args([
            "verify",
            spec.to_str().unwrap_p(),
            "--human",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    let v_stdout = String::from_utf8(v_human.stdout).unwrap_p();
    assert!(
        v_stdout.contains("prose_only_guard"),
        "verify's human view names the assumption: {v_stdout}"
    );
}

/// 4.1: a stored report with NO trust base is stale with the rerun
/// hint — never read as an empty ledger — and the claim schema version
/// stays 1 with the stored report unmodified.
#[test]
fn stored_report_without_trust_base_is_stale_with_rerun_hint() {
    let td = tempfile::tempdir().unwrap_p();
    let (a, _b) = write_scope_pair(td.path());
    let out = td.path().join("out");
    scope_compile(&[a.as_str()], &out);
    mc_batch(&[a.as_str()], &out);
    // Simulate a pre-trust-ledger report: no trust base at all.
    let mut old = read_check_report(&out, "scp_a");
    old.as_object_mut().unwrap_p().remove("trust_base");
    write_check_report(&out, "scp_a", &old);
    let before = std::fs::read_to_string(out.join("scp_a.check.json")).unwrap_p();
    drop_props(&out, &["scp_a"]);
    let (vcode, vjson) = verify_batch(&[a.as_str()], &out);
    assert_eq!(
        vcode,
        Some(1),
        "an absent ledger is never evidence: {vjson}"
    );
    let entry = &vjson["data"]["blocked"][0];
    assert_eq!(entry["model"]["state"], "stale");
    let detail = entry["model"]["detail"].as_str().unwrap_p();
    assert!(detail.contains("trust base"), "names the gap: {detail}");
    assert!(
        detail.contains("rerun model-check"),
        "carries the rerun hint: {detail}"
    );
    // Never rewritten, and the claim schema version is untouched.
    let after = std::fs::read_to_string(out.join("scp_a.check.json")).unwrap_p();
    assert_eq!(before, after, "the stored report is never rewritten");
    assert_eq!(read_check_report(&out, "scp_a")["claim_schema_version"], 1);
}

/// 4.2: an unrecognized `trust_base_schema_version` is refused stale,
/// same hint.
#[test]
fn unrecognized_trust_base_schema_version_requires_rerun() {
    let td = tempfile::tempdir().unwrap_p();
    let (a, _b) = write_scope_pair(td.path());
    let out = td.path().join("out");
    scope_compile(&[a.as_str()], &out);
    mc_batch(&[a.as_str()], &out);
    let mut future = read_check_report(&out, "scp_a");
    future["trust_base"]["trust_base_schema_version"] = serde_json::json!(99);
    write_check_report(&out, "scp_a", &future);
    drop_props(&out, &["scp_a"]);
    let (vcode, vjson) = verify_batch(&[a.as_str()], &out);
    assert_eq!(vcode, Some(1), "unrecognized ledger versions never verify");
    let entry = &vjson["data"]["blocked"][0];
    assert_eq!(entry["model"]["state"], "stale");
    let detail = entry["model"]["detail"].as_str().unwrap_p();
    assert!(detail.contains("trust base"), "{detail}");
    assert!(detail.contains("rerun model-check"), "{detail}");
}

/// 4.3: a stored ledger that disagrees with the recomputation is
/// refused with the unchecked-set comparison's rerun hint, and the
/// stored report is left unmodified.
#[test]
fn ledger_mismatch_is_refused_and_report_unmodified() {
    let td = tempfile::tempdir().unwrap_p();
    let (a, _b) = write_scope_pair(td.path());
    let out = td.path().join("out");
    scope_compile(&[a.as_str()], &out);
    mc_batch(&[a.as_str()], &out);
    let mut tampered = read_check_report(&out, "scp_a");
    tampered["trust_base"]["unverified_claims"]
        .as_array_mut()
        .unwrap_p()
        .push(serde_json::json!({
            "id": "ghost",
            "kind": "prose_only_invariant",
            "source": "scp_a"
        }));
    write_check_report(&out, "scp_a", &tampered);
    let before = std::fs::read_to_string(out.join("scp_a.check.json")).unwrap_p();
    drop_props(&out, &["scp_a"]);
    let (vcode, vjson) = verify_batch(&[a.as_str()], &out);
    assert_eq!(vcode, Some(1), "a disagreeing ledger is never accepted");
    let entry = &vjson["data"]["blocked"][0];
    assert_eq!(entry["model"]["state"], "stale");
    let detail = entry["model"]["detail"].as_str().unwrap_p();
    assert!(
        detail.contains("rerun model-check over the same file set"),
        "the unchecked-set comparison's rerun hint: {detail}"
    );
    let after = std::fs::read_to_string(out.join("scp_a.check.json")).unwrap_p();
    assert_eq!(before, after, "the stored report is left unmodified");
}

/// 4.4: the same corpus presented in a different file order — the
/// recomputed ledger is identical member for member and the persisted
/// report is byte-identical.
#[test]
fn reordered_corpus_yields_byte_identical_reports_and_ledger() {
    let td = tempfile::tempdir().unwrap_p();
    let a = td.path().join("tlr_a.md");
    let b = td.path().join("tlr_b.md");
    write_trust_spec(&a, "tlr_a");
    write_trust_spec(&b, "tlr_b");
    let a = a.to_str().unwrap_p().to_string();
    let b = b.to_str().unwrap_p().to_string();
    let fwd = td.path().join("fwd");
    let rev = td.path().join("rev");
    scope_compile(&[a.as_str(), b.as_str()], &fwd);
    mc_batch(&[a.as_str(), b.as_str()], &fwd);
    scope_compile(&[b.as_str(), a.as_str()], &rev);
    mc_batch(&[b.as_str(), a.as_str()], &rev);
    // The ledger is corpus-scoped: every file's report carries the ONE
    // invocation ledger, members naming their own file.
    let corpus_ledger = serde_json::json!({
        "trust_base_schema_version": 1,
        "unverified_claims": [
            {"id": "kp", "kind": "prose_only_invariant", "source": "tlr_a"},
            {"id": "kp", "kind": "prose_only_invariant", "source": "tlr_b"},
            {"id": "adv", "kind": "advisory_constraint", "source": "tlr_a"},
            {"id": "adv", "kind": "advisory_constraint", "source": "tlr_b"},
        ],
        "assumed": [
            {"id": "t", "kind": "prose_only_guard", "source": "tlr_a"},
            {"id": "t", "kind": "prose_only_guard", "source": "tlr_b"},
        ],
    });
    for stem in ["tlr_a", "tlr_b"] {
        let fwd_report = read_check_report(&fwd, stem);
        let rev_report = read_check_report(&rev, stem);
        // The persisted report CARRIES the derived ledger (not an absent
        // one), and the two orders agree member for member.
        assert_eq!(
            fwd_report["trust_base"], corpus_ledger,
            "{stem}: the report carries the derived ledger: {fwd_report}"
        );
        assert_eq!(
            fwd_report["trust_base"], rev_report["trust_base"],
            "{stem}: the ledger is identical member for member"
        );
        let fwd_bytes = std::fs::read(fwd.join(format!("{stem}.check.json"))).unwrap_p();
        let rev_bytes = std::fs::read(rev.join(format!("{stem}.check.json"))).unwrap_p();
        assert_eq!(
            fwd_bytes, rev_bytes,
            "{stem}: the persisted report is byte-identical across file orders"
        );
    }
}

/// 4.6: more than ten members — the human output names the count per
/// kind and the first ten members in the deterministic order, and says
/// the list is truncated; the JSON keeps every member. Eleven advisory
/// rows keep the truncated ids out of the unchecked-claim lines, so the
/// absence assertion is precise.
#[test]
fn human_view_counts_per_kind_and_truncates_over_ten_members() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("tlt.md");
    let mut rows = String::new();
    let mut names: Vec<String> = vec![];
    for i in 1..=11 {
        names.push(format!("adv{i:02}"));
        rows.push_str(&format!(
            "| adv{i:02} | advisory | `guidance {i}` | [[tlt]] |\n"
        ));
    }
    rows.push_str("| ka | invariant | `**rust:** state != \"blackhole\"` | [[tlt]] |\n");
    names.push("ka".to_string());
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let props = claim_props("tlt", &names);
    write_claim_spec(&spec, "tlt", &rows, &props, "ka");
    let out = td.path().join("out");
    claim_compile(spec.to_str().unwrap_p(), &out);
    let (code, json) = claim_mc_json(spec.to_str().unwrap_p(), &out);
    assert_eq!(code, Some(0));
    let tb = &json["data"]["checked"][0]["trust_base"];
    assert_eq!(
        tb["unverified_claims"].as_array().unwrap_p().len(),
        11,
        "the JSON keeps every member: {tb}"
    );
    assert_eq!(tb["assumed"].as_array().unwrap_p().len(), 0);

    let human = spk()
        .args([
            "model-check",
            spec.to_str().unwrap_p(),
            "--human",
            "--out-dir",
            out.to_str().unwrap_p(),
        ])
        .output()
        .unwrap_p();
    let stdout = String::from_utf8(human.stdout).unwrap_p();
    // The count per kind, and the truncation is stated.
    assert!(stdout.contains("11"), "names the per-kind count: {stdout}");
    assert!(
        stdout.contains("truncated"),
        "states the truncation: {stdout}"
    );
    for i in 1..=10 {
        assert!(
            stdout.contains(&format!("adv{i:02}")),
            "adv{i:02}: {stdout}"
        );
    }
    assert!(
        !stdout.contains("adv11"),
        "only the first ten members render: {stdout}"
    );
}

/// 4.8: an empty trust base is serialized as an empty object with both
/// arrays present, never omitted — the absent-vs-empty distinction
/// through the stored-report path.
#[test]
fn empty_trust_base_is_serialized_never_omitted() {
    let td = tempfile::tempdir().unwrap_p();
    let (a, _b) = write_scope_pair(td.path());
    let out = td.path().join("out");
    scope_compile(&[a.as_str()], &out);
    mc_batch(&[a.as_str()], &out);
    // The stored report carries an EMPTY ledger, not an absent one.
    let raw = std::fs::read_to_string(out.join("scp_a.check.json")).unwrap_p();
    assert!(raw.contains("\"trust_base\""), "serialized: {raw}");
    let report = read_check_report(&out, "scp_a");
    let tb = &report["trust_base"];
    assert!(tb.is_object(), "the ledger is an object: {report}");
    assert_eq!(tb["trust_base_schema_version"], 1);
    assert_eq!(tb["unverified_claims"], serde_json::json!([]));
    assert_eq!(tb["assumed"], serde_json::json!([]));
    // The empty ledger is accepted — never refused as absent.
    drop_props(&out, &["scp_a"]);
    let (vcode, vjson) = verify_batch(&[a.as_str()], &out);
    let entry = &vjson["data"]["blocked"][0];
    assert_ne!(
        entry["model"]["state"], "stale",
        "an empty ledger is not an absent one: {entry}"
    );
    assert_eq!(
        entry["trust_base"]["unverified_claims"],
        serde_json::json!([])
    );
    let _ = vcode;
}

// The ledger is not a verdict (layer.md "The ledger is not a verdict"
// scenario): the release-facing docs state that a non-empty ledger reports
// what was not checked and never blocks or grants `verified`, that an empty
// one says only that the corpus declares no unchecked claim and no
// machine-uninterpreted guard, and that neither says anything about the
// implementation's behaviour.
#[test]
fn release_docs_state_the_ledger_is_not_a_verdict() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let root = std::path::Path::new(manifest);
    let readme: String = std::fs::read_to_string(root.join("README.md"))
        .expect("README.md readable")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for needle in [
        "trust base",
        "prose-only invariants",
        "foreign extension-point",
        "advisory constraints",
        "prose-only guards",
        "never blocks or grants `verified`",
        "no unchecked claim and no machine-uninterpreted guard",
        "says anything about the implementation's behaviour",
    ] {
        assert!(readme.contains(needle), "README misses {needle:?}");
    }
    let status: String = std::fs::read_to_string(root.join("docs/src/status.md"))
        .expect("status.md readable")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        status.contains("trust base naming what was not checked"),
        "status.md must name the trust base"
    );
    assert!(
        status.contains("never blocks or grants `verified`"),
        "status.md must state the ledger is not a verdict"
    );
}
