//! The Orchestrator — drive the four-stage pipeline (parse → lint →
//! compile → model_check → verify) in the order specs/orchestrate.md
//! fixes.
//!
//! Purpose: realize specs/orchestrate.md's sequencing and gating as one
//! driver: every Checker Ownership checker runs in dependency order
//! (dependents of a failed checker are skipped — never invoked, never
//! reported as failed), independent branches report regardless, and the
//! compile/model_check/verify stages run in sequence with exactly the
//! guards specodelic.md specifies. Responsibilities: sequencing and
//! gating ONLY — every actual check is deferred to the module that
//! already owns it (lint, compile, model_check, verify). Rationale: a
//! pipeline stage that re-implemented a check could drift from its
//! owner; the orchestrator only decides *whether* a stage runs and
//! *what* its outcome was.

use serde::Serialize;
use std::collections::BTreeMap;

use crate::checklist::Checklist;
use crate::citation_corpus;
use crate::compile;
use crate::kernel;
use crate::lint::{self, Issue};
use crate::model_check;
use crate::spec::Spec;
use crate::verify;

/// A pipeline stage's outcome. `skipped` always carries the reason —
/// every skip is reported (specs/orchestrate.md, no-silent-skips).
#[derive(Serialize)]
pub struct Stage {
    pub stage: &'static str,
    /// `passed` | `failed` | `skipped`
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<serde_json::Value>,
}

/// The full orchestration report. Byte-stable for an unchanged repo
/// (`deterministic_rerun`): stages serialize in pipeline order and all
/// per-file collections keep the input order.
#[derive(Serialize)]
pub struct Orchestration {
    /// `succeeded` | `failed`
    pub overall: String,
    pub stages: Vec<Stage>,
}

/// The model-check backend selection: native stateright (default) or
/// the JVM TLC reference engine. Mirrors cmd_model_check's seam.
#[derive(Clone, Default)]
pub struct Backends {
    pub tlc: Option<model_check::TlcPaths>,
}

/// The parse stage's structured outcome (CORR-001, Ro5 over
/// specodelic-8kk): gating reads `parse_errors`, never note prose — a
/// file whose *path* happens to contain "parse error" must not flip
/// the gate. Notes stay labeled prose for the warnings channel.
#[derive(Clone, Default)]
pub struct ParseInput {
    pub notes: Vec<String>,
    pub parse_errors: Vec<String>,
}

fn skipped_stage(stage: &'static str, reason: String) -> Stage {
    Stage {
        stage,
        status: "skipped".into(),
        reason: Some(reason),
        detail: None,
    }
}

fn passed_stage(stage: &'static str, detail: serde_json::Value) -> Stage {
    Stage {
        stage,
        status: "passed".into(),
        reason: None,
        detail: Some(detail),
    }
}

fn failed_stage(stage: &'static str, detail: serde_json::Value) -> Stage {
    Stage {
        stage,
        status: "failed".into(),
        reason: None,
        detail: Some(detail),
    }
}

fn issues_json(issues: &[Issue]) -> serde_json::Value {
    serde_json::json!(
        issues
            .iter()
            .map(|i| serde_json::json!({
                "rule_id": i.rule_id,
                "file": i.file,
                "message": i.message,
            }))
            .collect::<Vec<_>>()
    )
}

/// One Checker Ownership checker's reported verdict.
fn checker_entry(checker: &'static str, issues: Vec<Issue>) -> serde_json::Value {
    let status = if issues.is_empty() {
        "passed"
    } else {
        "failed"
    };
    serde_json::json!({
        "checker": checker,
        "status": status,
        "issues": issues_json(&issues),
    })
}

fn checker_skipped(
    checker: &'static str,
    dependency: &str,
    dependency_status: &str,
) -> serde_json::Value {
    serde_json::json!({
        "checker": checker,
        "status": "skipped",
        "reason": format!(
            "not run — dependency {dependency} {dependency_status}; never run against input the failed checker has not validated"
        ),
        "issues": [],
    })
}

/// The lint stage: the seven Checker Ownership gate checkers in
/// dependency order. `linter.external_completeness` runs only when a
/// checklist is declared and never gates. The stage passes iff every
/// gate checker reports passed — a skipped checker means its dependency
/// failed, so the stage fails with the dependency's findings.
fn run_lint_stage(specs: &[Spec], checklists: &[Checklist]) -> Stage {
    // Branch A head: the first gate.
    let fm = lint::frontmatter_findings(specs);
    let fm_ok = fm.is_empty();
    let mut checkers = vec![checker_entry("linter.frontmatter", fm)];

    // Branch A: referential_integrity → graph_shape → model_shape.
    let (ref_entry, ref_ok) = if fm_ok {
        let issues = lint::referential_findings(specs);
        let ok = issues.is_empty();
        (checker_entry("linter.referential_integrity", issues), ok)
    } else {
        (
            checker_skipped(
                "linter.referential_integrity",
                "linter.frontmatter",
                "failed",
            ),
            false,
        )
    };
    checkers.push(ref_entry);

    let (gs_entry, gs_ok) = if ref_ok {
        let issues = lint::graph_shape_findings(specs);
        let ok = issues.is_empty();
        (checker_entry("linter.graph_shape", issues), ok)
    } else {
        let dep_status = if fm_ok { "failed" } else { "skipped" };
        (
            checker_skipped(
                "linter.graph_shape",
                "linter.referential_integrity",
                dep_status,
            ),
            false,
        )
    };
    checkers.push(gs_entry);

    let (ms_entry, ms_ok) = if gs_ok {
        let issues = lint::model_shape_findings(specs);
        let ok = issues.is_empty();
        (checker_entry("linter.model_shape", issues), ok)
    } else {
        let dep_status = if ref_ok { "failed" } else { "skipped" };
        (
            checker_skipped("linter.model_shape", "linter.graph_shape", dep_status),
            false,
        )
    };
    checkers.push(ms_entry);

    // Branch A tail: failure_shape — depends on model_shape (the
    // failure-shape walk reads the Model's states, transitions and
    // emits edges, so it needs the model proven well-formed first).
    let (fs_entry, fs_ok) = if ms_ok {
        let issues = lint::failure_shape_findings(specs);
        let ok = issues.is_empty();
        (checker_entry("linter.failure_shape", issues), ok)
    } else {
        let dep_status = if gs_ok { "failed" } else { "skipped" };
        (
            checker_skipped("linter.failure_shape", "linter.model_shape", dep_status),
            false,
        )
    };
    checkers.push(fs_entry);

    // Branch B: ears_syntax — depends only on frontmatter.
    let (ears_entry, ears_ok) = if fm_ok {
        let issues = lint::ears_findings(specs);
        let ok = issues.is_empty();
        (checker_entry("linter.ears_syntax", issues), ok)
    } else {
        (
            checker_skipped("linter.ears_syntax", "linter.frontmatter", "failed"),
            false,
        )
    };
    checkers.push(ears_entry);

    // Branch C: schema_shape — depends only on frontmatter. The engine
    // rules are the closed kind-set table-walkers (constraint_kind_closed
    // / property_kind_closed, specodelic-7h8) plus the structural
    // parser/cross-revision residue; see lint::schema_shape_findings.
    let (schema_entry, schema_ok) = if fm_ok {
        let issues = lint::schema_shape_findings(specs);
        (
            serde_json::json!({
                "checker": "linter.schema_shape",
                "status": if issues.is_empty() { "passed" } else { "failed" },
                "basis": "table-walking closed kind sets (constraint_kind_closed / property_kind_closed) plus structural parser/cross-revision residue",
                "issues": issues_json(&issues),
            }),
            issues.is_empty(),
        )
    } else {
        (
            checker_skipped("linter.schema_shape", "linter.frontmatter", "failed"),
            false,
        )
    };
    checkers.push(schema_entry);

    // external_completeness — runs only when a checklist is declared;
    // its outcome never blocks or delays any stage.
    let ec = if checklists.is_empty() {
        serde_json::json!({
            "checker": "linter.external_completeness",
            "status": "not_applicable",
            "issues": [],
        })
    } else {
        let (issues, declared) = lint::external_completeness_findings(specs, checklists);
        let status = if issues.is_empty() {
            "passed"
        } else {
            "failed"
        };
        serde_json::json!({
            "checker": "linter.external_completeness",
            "status": status,
            "declared": declared,
            "issues": issues_json(&issues),
        })
    };
    checkers.push(ec);

    let passed = fm_ok && ref_ok && gs_ok && ms_ok && fs_ok && ears_ok && schema_ok;
    let detail = serde_json::json!({ "checkers": checkers });
    if passed {
        passed_stage("lint", detail)
    } else {
        failed_stage("lint", detail)
    }
}

/// The compile stage: the coverage gate exactly (compile advances iff
/// linter.coverage reports passed — never looser, never stricter), then
/// per-file compilation through the compile functor's own contract.
fn run_compile_stage(specs: &[Spec], out_dir: &str) -> Stage {
    // compile_gate_matches_coverage: the gate is the coverage checker's
    // verdict, not a re-derivation.
    let cov = lint::coverage_findings(specs);
    if !cov.is_empty() {
        return failed_stage(
            "compile",
            serde_json::json!({
                "gate": "linter.coverage",
                "issues": issues_json(&cov),
            }),
        );
    }
    let mut compiled: Vec<serde_json::Value> = vec![];
    let mut failed: Vec<serde_json::Value> = vec![];
    let mut warnings: Vec<String> = vec![];
    let mut seen_stems: std::collections::BTreeMap<String, String> = Default::default();
    for spec in specs {
        let file = spec
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| format!("<{}>", spec.intent.id));
        match compile::compile_spec(spec) {
            Ok(c) => match compile::write_artifacts(spec, &c, out_dir) {
                Ok(written) => {
                    let stem = compile::artifact_stem(spec);
                    if let Some(first) = seen_stems.get(&stem) {
                        warnings.push(format!(
                            "stem collision: `{stem}` artifacts from {first} and {file} share one out-dir — the later file wins",
                        ));
                    }
                    seen_stems.insert(stem, file.clone());
                    compiled.push(serde_json::json!({
                        "file": file,
                        "id": spec.intent.id,
                        "written": written,
                    }));
                }
                Err(message) => failed.push(serde_json::json!({
                    "file": file,
                    "id": spec.intent.id,
                    "stage": "write_artifacts",
                    "message": message,
                })),
            },
            Err(e) => failed.push(serde_json::json!({
                "file": file,
                "id": spec.intent.id,
                "stage": e.stage,
                "message": e.message,
            })),
        }
    }
    let detail = serde_json::json!({
        "files_compiled": compiled.len(),
        "files_failed": failed.len(),
        "compiled": compiled,
        "failed": failed,
        "warnings": warnings,
    });
    if failed.is_empty() {
        passed_stage("compile", detail)
    } else {
        failed_stage("compile", detail)
    }
}

/// The model_check stage: consumes the compiled `.tla` the compile
/// stage just wrote, runs within the stated bound, persists the
/// `<stem>.check.json` run report, and passes only when every file's
/// outcome is `no_counterexample` — `exploration_only` is a completed
/// exploration with zero predicates executed, never a clean verdict
/// (model_check.md's CLAR-003 distinction).
/// One file's claim-gated model-check result (design D4): the
/// post-aggregate report plus the merged statuses JSON and the report
/// path it was persisted to. Each report view renders its own entry
/// from these shared fields.
pub struct CheckedFile {
    pub file: String,
    pub id: String,
    pub report: model_check::RunReport,
    pub statuses_json: Vec<serde_json::Value>,
    pub written: String,
}

/// One file's claim-gated model-check run: either the checked result or
/// the labeled (stage, message) failure — a backend error or a report
/// write error.
pub struct GateRun {
    pub file: String,
    pub id: String,
    pub outcome: Result<CheckedFile, (String, String)>,
}

/// The ONE claim-gated model-check pipeline (define-verification-claim-gates
/// design D4): backend pass, corpus citation resolution, the kernel corpus
/// pass, the single status merge (§3.8 TIDY), and the single claim-gate
/// aggregate — both the native CLI (`cmd_model_check`) and the orchestrate
/// stage consume this exact path, so the aggregate verdict cannot drift
/// between report views. Per-view entry rendering stays with each view
/// (the CLI's rich checked entry vs the orchestrate stage's compact one).
pub fn run_claim_gated_model_check(
    specs: &[Spec],
    out_dir: &str,
    bound: &model_check::Bound,
    tlc: Option<&model_check::TlcPaths>,
) -> Vec<GateRun> {
    // Pass 1 — per-file backend runs; pass 2 — corpus-wide citation
    // resolution applied to every report before persistence (design D9):
    // the native CLI and orchestrate share this exact path.
    let mut runs = citation_corpus::run_backend_pass(specs, out_dir, bound, tlc);
    let resolved_per_file = citation_corpus::apply_corpus_resolution(specs, &mut runs);
    // The kernel corpus pass (§3.7, design D9): every opted-in kernel
    // claim evaluated over the complete invocation corpus, or labeled
    // unsupported on the incapable backend, appended to the same
    // statuses the citations resolved into.
    let kernel_claims = kernel::evaluate_corpus_claims(
        specs,
        kernel::CorpusBackend::from_tlc_presence(tlc.is_some()),
    );
    // Scope/freshness binding (design D3 — specodelic-68m.3): every
    // file's report carries the SAME invocation digest — the structured
    // content of ALL parsed inputs plus the compiled artifacts consumed
    // for the invocation — plus that file's qualified claim records and
    // canonical expected/unchecked sets. Verify recomputes every one of
    // these from the live inputs; the stored values are never trusted.
    let classifications: Vec<verify::ClaimClassification> =
        specs.iter().map(verify::claim_classification).collect();
    let artifact_hashes: std::collections::BTreeMap<String, String> = runs
        .iter()
        .filter_map(|r| {
            r.result
                .as_ref()
                .ok()
                .map(|rep| (r.id.clone(), rep.artifact_sha256.clone()))
        })
        .collect();
    let scope_sha256 = verify::scope_digest(specs, &artifact_hashes);
    runs.into_iter()
        .enumerate()
        .map(|(idx, run)| {
            let file = run.file;
            let id = run.id;
            let outcome = match run.result {
                Ok(mut report) => {
                    // One shared merge (§3.8 TIDY): the persisted statuses
                    // and every view's entries come from the single
                    // identity helper — kernel claims ride the same
                    // statuses as the resolved citations, exactly once.
                    let statuses_json = kernel::merge_corpus_statuses(
                        &mut report,
                        &resolved_per_file[idx],
                        &kernel_claims[idx],
                    );
                    // The claim-gate aggregate (design D1/D2/D4): the
                    // merged required-claim statuses govern the verdict
                    // that is persisted and handed to verify.
                    kernel::aggregate_required_claims(&mut report);
                    // Qualified claim records (D3): one per evaluated
                    // invariant — the evaluator kind from the live
                    // classification, the evaluated status, and the
                    // labeled reason when the claim did not verify.
                    let mut reasons: BTreeMap<&str, String> = BTreeMap::new();
                    for r in &resolved_per_file[idx] {
                        if let Some(reason) = &r.reason {
                            reasons.insert(r.id.as_str(), reason.clone());
                        }
                    }
                    for k in &kernel_claims[idx] {
                        if let Some(reason) = &k.reason {
                            reasons.insert(k.id.as_str(), reason.clone());
                        }
                    }
                    let claims: Vec<verify::QualifiedClaim> = report
                        .invariant_statuses
                        .iter()
                        .map(|s| verify::QualifiedClaim {
                            id: s.id.clone(),
                            evaluator: classifications[idx]
                                .required
                                .iter()
                                .find(|req| req.id == s.id)
                                .map(|req| req.evaluator.to_string())
                                // A status outside the live
                                // classification cannot exist (both
                                // derive from the same IR); if it ever
                                // did, the record says so and verify
                                // fails closed on the evaluator match.
                                .unwrap_or_else(|| "unknown".to_string()),
                            status: s.status,
                            reason: reasons.get(s.id.as_str()).cloned(),
                        })
                        .collect();
                    let expected_claim_ids: Vec<String> = classifications[idx]
                        .required
                        .iter()
                        .map(|req| req.id.clone())
                        .collect();
                    let report_value = verify::fresh_report_json(
                        &report,
                        &claims,
                        &expected_claim_ids,
                        &classifications[idx].unchecked,
                        &scope_sha256,
                    );
                    let report_path =
                        std::path::Path::new(out_dir).join(format!("{}.check.json", run.stem));
                    let report_json =
                        serde_json::to_string_pretty(&report_value).expect("run report serializes");
                    match std::fs::write(&report_path, &report_json) {
                        Ok(()) => Ok(CheckedFile {
                            file: file.clone(),
                            id: id.clone(),
                            report,
                            statuses_json,
                            written: report_path.display().to_string(),
                        }),
                        Err(e) => Err((
                            "write_report".to_string(),
                            format!("could not write {}: {e}", report_path.display()),
                        )),
                    }
                }
                Err(e) => Err((e.stage, e.message)),
            };
            GateRun { file, id, outcome }
        })
        .collect()
}

/// The model_check stage: consumes the compiled `.tla` the compile
/// stage just wrote, runs within the stated bound, persists the
/// `<stem>.check.json` run report, and passes only when every file's
/// outcome is `no_counterexample` — `exploration_only` is a completed
/// exploration with zero predicates executed, never a clean verdict
/// (model_check.md's CLAR-003 distinction).
fn run_model_check_stage(
    specs: &[Spec],
    out_dir: &str,
    bound: &model_check::Bound,
    backends: &Backends,
) -> Stage {
    let mut checked: Vec<serde_json::Value> = vec![];
    let mut failed: Vec<serde_json::Value> = vec![];
    let mut all_clean = true;
    // The stage rides the ONE shared claim-gated pipeline (design D4) —
    // the exact path the native CLI takes; only the detail rendering
    // (the compact stage entry) differs from the CLI's.
    for run in run_claim_gated_model_check(specs, out_dir, bound, backends.tlc.as_ref()) {
        match run.outcome {
            Ok(c) => {
                // The stage's clean verdict is exactly the shared
                // aggregate's: `no_counterexample`, nothing else.
                let clean = matches!(c.report.outcome, model_check::Outcome::NoCounterexample);
                if !clean {
                    all_clean = false;
                }
                checked.push(serde_json::json!({
                    "file": c.file,
                    "id": c.id,
                    "outcome": c.report.outcome,
                    "clean": clean,
                    "backend": c.report.backend,
                    "invariant_statuses": c.statuses_json,
                    "states_explored": c.report.states_explored,
                    "written": c.written,
                }));
            }
            Err((stage, message)) => {
                all_clean = false;
                failed.push(serde_json::json!({
                    "file": run.file,
                    "id": run.id,
                    "stage": stage,
                    "message": message,
                }));
            }
        }
    }
    let detail = serde_json::json!({
        "files_checked": checked.len(),
        "files_failed": failed.len(),
        "checked": checked,
        "failed": failed,
    });
    if all_clean {
        passed_stage("model_check", detail)
    } else {
        failed_stage("model_check", detail)
    }
}

/// The verify stage: both gates required — the compiled proptest blocks
/// execute and pass, AND model_check's most recent run against the
/// current artifact is clean. The conjunction is verify::verdict's.
fn run_verify_stage(specs: &[Spec], out_dir: &str) -> Stage {
    let runner = verify::CargoRunner::default();
    let dir = std::path::Path::new(out_dir);
    let mut verified: Vec<serde_json::Value> = vec![];
    let mut blocked: Vec<serde_json::Value> = vec![];
    // The stage rides the ONE both-gates evaluation loop (design D4) —
    // the exact path the native CLI takes; only the compact detail
    // entry rendering differs from the CLI's.
    for fv in verify::evaluate_file_verdicts(specs, dir, &runner) {
        let entry = serde_json::json!({
            "file": fv.file,
            "id": fv.id,
            "status": fv.verdict.status,
            "message": fv.verdict.message,
        });
        if fv.verdict.status == "verified" {
            verified.push(entry);
        } else {
            blocked.push(entry);
        }
    }
    let detail = serde_json::json!({
        "files_verified": verified.len(),
        "files_blocked": blocked.len(),
        "verified": verified,
        "blocked": blocked,
    });
    if blocked.is_empty() {
        passed_stage("verify", detail)
    } else {
        failed_stage("verify", detail)
    }
}

/// Run the pipeline. `parse_notes` carries the parse stage's labeled
/// notes (hostile-input skips, non-spec skips, parse errors) — a spec
/// parse error is the only parse failure: a file that never reached
/// `parsed` can not start lint (start_lint's guard), while hostile-input
/// and non-spec skips are labeled warnings by design (specodelic-suz).
pub fn orchestrate(
    specs: &[Spec],
    checklists: &[Checklist],
    parse: &ParseInput,
    out_dir: &str,
    bound: &model_check::Bound,
    backends: &Backends,
) -> Orchestration {
    let parse_failed = !parse.parse_errors.is_empty();
    let parse_detail = serde_json::json!({
        "files_parsed": specs.len(),
        "notes": parse.notes,
        "parse_errors": parse.parse_errors,
    });
    let mut stages = vec![if parse_failed {
        failed_stage("parse", parse_detail)
    } else {
        passed_stage("parse", parse_detail)
    }];

    if parse_failed {
        stages.push(skipped_stage(
            "lint",
            "parse stage failed — every file must reach parsed before lint starts".into(),
        ));
        stages.push(skipped_stage(
            "compile",
            "parse stage failed — every file must reach parsed before lint starts".into(),
        ));
        stages.push(skipped_stage(
            "model_check",
            "parse stage failed — every file must reach parsed before lint starts".into(),
        ));
        stages.push(skipped_stage(
            "verify",
            "parse stage failed — every file must reach parsed before lint starts".into(),
        ));
    } else {
        let lint_stage = run_lint_stage(specs, checklists);
        let lint_ok = lint_stage.status == "passed";
        stages.push(lint_stage);
        if !lint_ok {
            stages.push(skipped_stage("compile", "lint stage failed".into()));
            stages.push(skipped_stage("model_check", "lint stage failed".into()));
            stages.push(skipped_stage("verify", "lint stage failed".into()));
        } else {
            let compile_stage = run_compile_stage(specs, out_dir);
            let compile_ok = compile_stage.status == "passed";
            stages.push(compile_stage);
            if !compile_ok {
                stages.push(skipped_stage("model_check", "compile stage failed".into()));
                stages.push(skipped_stage("verify", "compile stage failed".into()));
            } else {
                let mc_stage = run_model_check_stage(specs, out_dir, bound, backends);
                let mc_ok = mc_stage.status == "passed";
                stages.push(mc_stage);
                if !mc_ok {
                    stages.push(skipped_stage(
                        "verify",
                        "model_check stage failed — verify is never invoked before model_check reports model_checked".into(),
                    ));
                } else {
                    stages.push(run_verify_stage(specs, out_dir));
                }
            }
        }
    }

    let overall = if stages.iter().all(|s| s.status == "passed") {
        "succeeded"
    } else {
        "failed"
    };
    Orchestration {
        overall: overall.into(),
        stages,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec_from(text: &str, path: &str) -> Spec {
        let mut s = crate::spec::parse_str(text).unwrap();
        s.path = Some(std::path::PathBuf::from(path));
        s
    }

    fn mc_fixture(id: &str) -> String {
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL be a model-check fixture\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | c1 | invariant | `holds` | [[{id}]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- s1\n\
             - s2\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | t | s1 | s2 | [[{id}.c1]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p | unit | [[{id}.c1]] | `g()` | `x` |\n"
        )
    }

    fn stage<'a>(o: &'a Orchestration, name: &str) -> &'a Stage {
        o.stages
            .iter()
            .find(|s| s.stage == name)
            .unwrap_or_else(|| panic!("stage {name} missing"))
    }

    fn checker<'a>(stage: &'a Stage, name: &str) -> &'a serde_json::Value {
        stage.detail.as_ref().unwrap()["checkers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["checker"] == name)
            .unwrap_or_else(|| panic!("checker {name} missing"))
    }

    #[test]
    fn every_stage_reported_and_native_model_check_is_never_clean() {
        let td = tempfile::tempdir().unwrap();
        let spec = spec_from(&mc_fixture("pipe.demo"), "pipe-demo.md");
        let specs = vec![spec];
        let o = orchestrate(
            &specs,
            &[],
            &ParseInput::default(),
            td.path().to_str().unwrap(),
            &model_check::Bound::default(),
            &Backends::default(),
        );
        let names: Vec<&str> = o.stages.iter().map(|s| s.stage).collect();
        assert_eq!(
            names,
            vec!["parse", "lint", "compile", "model_check", "verify"]
        );
        assert_eq!(stage(&o, "parse").status, "passed");
        assert_eq!(stage(&o, "lint").status, "passed");
        assert_eq!(stage(&o, "compile").status, "passed");
        // The native backend executes zero invariant predicates — an
        // exhaustive run is exploration_only, never a clean verdict, so
        // the model_check stage honestly fails and verify is skipped.
        assert_eq!(stage(&o, "model_check").status, "failed");
        let mc = stage(&o, "model_check");
        assert_eq!(
            mc.detail.as_ref().unwrap()["checked"][0]["outcome"],
            "exploration_only"
        );
        assert_eq!(mc.detail.as_ref().unwrap()["checked"][0]["clean"], false);
        assert_eq!(stage(&o, "verify").status, "skipped");
        assert_eq!(o.overall, "failed");
    }

    #[test]
    fn failure_shape_checker_rides_the_lint_stage() {
        // specodelic-hhp decision (a): the failure-shape checker joins
        // the Checker Ownership table and the lint stage, after its
        // natural dependency linter.model_shape.
        let td = tempfile::tempdir().unwrap();
        let clean = spec_from(&mc_fixture("pipe.demo"), "pipe-demo.md");
        let o = orchestrate(
            &[clean],
            &[],
            &ParseInput::default(),
            td.path().to_str().unwrap(),
            &model_check::Bound::default(),
            &Backends::default(),
        );
        let lint = stage(&o, "lint");
        assert_eq!(checker(lint, "linter.failure_shape")["status"], "passed");

        // A mute failure terminal (no emits edge) fails the checker and
        // the stage.
        let td2 = tempfile::tempdir().unwrap();
        let mute_src = mc_fixture("pipe.demo")
            .replace(
                "| t | s1 | s2 | [[pipe.demo.c1]] |",
                "| t | s1 | s2 | [[pipe.demo.c1]] |\n| boom | s1 | failed | `the world ends` |",
            )
            .replace("- s2\n", "- s2\n- failed\n");
        let mute = spec_from(&mute_src, "pipe-demo.md");
        let o2 = orchestrate(
            &[mute],
            &[],
            &ParseInput::default(),
            td2.path().to_str().unwrap(),
            &model_check::Bound::default(),
            &Backends::default(),
        );
        let lint2 = stage(&o2, "lint");
        assert_eq!(lint2.status, "failed");
        let fs = checker(lint2, "linter.failure_shape");
        assert_eq!(fs["status"], "failed");
        assert!(
            !fs["issues"].as_array().unwrap().is_empty(),
            "the mute terminal is a finding: {fs}"
        );
    }

    #[test]
    fn upstream_failure_skips_dependents() {
        let td = tempfile::tempdir().unwrap();
        // Wrong kind: parses, but frontmatter_valid fires.
        let bad = spec_from(
            "---\nid: pipe.demo\nkind: feature\nstatement: \"THE system SHALL hold\"\n---\n",
            "pipe-demo.md",
        );
        let specs = vec![bad];
        let o = orchestrate(
            &specs,
            &[],
            &ParseInput::default(),
            td.path().to_str().unwrap(),
            &model_check::Bound::default(),
            &Backends::default(),
        );
        let lint = stage(&o, "lint");
        assert_eq!(lint.status, "failed");
        assert_eq!(checker(lint, "linter.frontmatter")["status"], "failed");
        for dependent in [
            "linter.referential_integrity",
            "linter.graph_shape",
            "linter.model_shape",
            "linter.failure_shape",
            "linter.ears_syntax",
            "linter.schema_shape",
        ] {
            let c = checker(lint, dependent);
            assert_eq!(c["status"], "skipped", "{dependent}");
            assert_eq!(
                c["issues"].as_array().unwrap().len(),
                0,
                "{dependent} was never invoked — it reports no findings"
            );
        }
        assert_eq!(stage(&o, "compile").status, "skipped");
        assert_eq!(stage(&o, "model_check").status, "skipped");
        assert_eq!(stage(&o, "verify").status, "skipped");
    }

    #[test]
    fn independent_branches_report_regardless() {
        let td = tempfile::tempdir().unwrap();
        // Branch B (ears) fails; branch A (referential → graph → model)
        // and branch C (schema) still run and report their own outcomes.
        let bad = spec_from(
            "---\nid: pipe.demo\nkind: intent\nstatement: \"the system should maybe work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `holds` | [[pipe.demo]] |\n\n## Model\n\n### States\n\n- s1\n- s2\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s2 | [[pipe.demo.c1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[pipe.demo.c1]] | `g()` | `x` |\n",
            "pipe-demo.md",
        );
        let specs = vec![bad];
        let o = orchestrate(
            &specs,
            &[],
            &ParseInput::default(),
            td.path().to_str().unwrap(),
            &model_check::Bound::default(),
            &Backends::default(),
        );
        let lint = stage(&o, "lint");
        assert_eq!(lint.status, "failed");
        assert_eq!(checker(lint, "linter.frontmatter")["status"], "passed");
        assert_eq!(
            checker(lint, "linter.referential_integrity")["status"],
            "passed"
        );
        assert_eq!(checker(lint, "linter.graph_shape")["status"], "passed");
        assert_eq!(checker(lint, "linter.model_shape")["status"], "passed");
        assert_eq!(checker(lint, "linter.failure_shape")["status"], "passed");
        assert_eq!(checker(lint, "linter.ears_syntax")["status"], "failed");
        assert_eq!(checker(lint, "linter.schema_shape")["status"], "passed");
    }

    #[test]
    fn dependency_skip_stays_within_its_branch() {
        let td = tempfile::tempdir().unwrap();
        // Duplicate row id: referential fails → graph_shape and
        // model_shape skip (branch A), while ears (branch B) still runs.
        let dup = spec_from(
            &mc_fixture("pipe.demo").replace(
                "| p | unit | [[pipe.demo.c1]] | `g()` | `x` |",
                "| c1 | unit | [[pipe.demo.c1]] | `g()` | `x` |",
            ),
            "pipe-demo.md",
        );
        let specs = vec![dup];
        let o = orchestrate(
            &specs,
            &[],
            &ParseInput::default(),
            td.path().to_str().unwrap(),
            &model_check::Bound::default(),
            &Backends::default(),
        );
        let lint = stage(&o, "lint");
        assert_eq!(checker(lint, "linter.frontmatter")["status"], "passed");
        assert_eq!(
            checker(lint, "linter.referential_integrity")["status"],
            "failed"
        );
        assert_eq!(checker(lint, "linter.graph_shape")["status"], "skipped");
        assert_eq!(checker(lint, "linter.model_shape")["status"], "skipped");
        assert_eq!(checker(lint, "linter.failure_shape")["status"], "skipped");
        assert_eq!(checker(lint, "linter.ears_syntax")["status"], "passed");
        assert_eq!(checker(lint, "linter.schema_shape")["status"], "passed");
    }

    #[test]
    fn coverage_failure_holds_compile() {
        let td = tempfile::tempdir().unwrap();
        // Constraint without a deriving property: all seven gate checkers
        // pass (coverage is NOT in the ownership table), the compile
        // stage halts on exactly the coverage checker's verdict.
        let cov = spec_from(
            "---\nid: pipe.demo\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `holds` | [[pipe.demo]] |\n\n## Model\n\n### States\n\n- s1\n- s2\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s2 | [[pipe.demo.c1]] |\n",
            "pipe-demo.md",
        );
        let specs = vec![cov];
        let o = orchestrate(
            &specs,
            &[],
            &ParseInput::default(),
            td.path().to_str().unwrap(),
            &model_check::Bound::default(),
            &Backends::default(),
        );
        assert_eq!(stage(&o, "lint").status, "passed");
        let compile = stage(&o, "compile");
        assert_eq!(compile.status, "failed");
        assert_eq!(compile.detail.as_ref().unwrap()["gate"], "linter.coverage");
        assert_eq!(stage(&o, "model_check").status, "skipped");
        assert_eq!(stage(&o, "verify").status, "skipped");
    }

    #[test]
    fn external_completeness_never_gates() {
        let td = tempfile::tempdir().unwrap();
        // A declared (failing) checklist: the checker reports failed,
        // yet lint passes and the pipeline proceeds past compile.
        let spec = spec_from(&mc_fixture("pipe.demo"), "pipe-demo.md");
        let checklist_path = td.path().join("pipe.checklist.md");
        std::fs::write(
            &checklist_path,
            "# Checklist\n\n## Items\n\n| id | requirement |\n|----|-------------|\n| i1 | something |\n",
        )
        .unwrap();
        let cl = crate::checklist::parse_str(
            checklist_path.clone(),
            &std::fs::read_to_string(&checklist_path).unwrap(),
        );
        let specs = vec![spec];
        let o = orchestrate(
            &specs,
            &[cl],
            &ParseInput::default(),
            td.path().to_str().unwrap(),
            &model_check::Bound::default(),
            &Backends::default(),
        );
        let lint = stage(&o, "lint");
        assert_eq!(lint.status, "passed", "external_completeness never gates");
        let ec = checker(lint, "linter.external_completeness");
        assert_eq!(ec["status"], "failed");
        assert_eq!(ec["declared"], 1);
        assert_eq!(stage(&o, "compile").status, "passed");
    }

    #[test]
    fn rerun_is_byte_identical() {
        let td = tempfile::tempdir().unwrap();
        let spec = spec_from(&mc_fixture("pipe.demo"), "pipe-demo.md");
        let specs = vec![spec];
        let out = td.path().to_str().unwrap().to_string();
        let run = || {
            orchestrate(
                &specs,
                &[],
                &ParseInput::default(),
                &out,
                &model_check::Bound::default(),
                &Backends::default(),
            )
        };
        let a = serde_json::to_string(&run()).unwrap();
        let b = serde_json::to_string(&run()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn note_prose_never_flips_the_parse_gate() {
        // CORR-001 (Ro5 over specodelic-8kk): gating reads structured
        // parse_errors, never note prose — a file whose PATH contains
        // "parse error" must not fail the parse stage.
        let td = tempfile::tempdir().unwrap();
        let spec = spec_from(&mc_fixture("pipe.demo"), "pipe-demo.md");
        let specs = vec![spec];
        let o = orchestrate(
            &specs,
            &[],
            &ParseInput {
                notes: vec!["my parse error notes.md: unreadable (boom)".to_string()],
                parse_errors: vec![],
            },
            td.path().to_str().unwrap(),
            &model_check::Bound::default(),
            &Backends::default(),
        );
        assert_eq!(stage(&o, "parse").status, "passed");
    }

    #[test]
    fn parse_error_holds_lint() {
        let td = tempfile::tempdir().unwrap();
        let spec = spec_from(&mc_fixture("pipe.demo"), "pipe-demo.md");
        let specs = vec![spec];
        let o = orchestrate(
            &specs,
            &[],
            &ParseInput {
                notes: vec![],
                parse_errors: vec!["broken.md: parse error: missing `kind` field".to_string()],
            },
            td.path().to_str().unwrap(),
            &model_check::Bound::default(),
            &Backends::default(),
        );
        assert_eq!(stage(&o, "parse").status, "failed");
        assert_eq!(stage(&o, "lint").status, "skipped");
        assert_eq!(stage(&o, "compile").status, "skipped");
        assert_eq!(o.overall, "failed");
    }

    #[test]
    fn verify_stage_passes_only_on_the_conjunction() {
        // Unit of run_verify_stage: fabricate a clean, current model run
        // report (real artifact sha — the staleness key) and a passing
        // proptest body; the stage passes. Missing the model leg (no
        // .check.json) blocks even with passing properties.
        let td = tempfile::tempdir().unwrap();
        let spec = spec_from(&mc_fixture("pipe.demo"), "pipe-demo.md");
        let specs = vec![spec];
        let out_dir = td.path().join("out");
        let out = out_dir.to_str().unwrap();
        // Compile + hand-translate the predicate to a passing body.
        let compile_stage = run_compile_stage(&specs, out);
        assert_eq!(compile_stage.status, "passed");
        let props_path = out_dir.join("pipe-demo_props.rs");
        let src = std::fs::read_to_string(&props_path).unwrap();
        let translated = src
            .lines()
            .map(|l| {
                if l.trim().starts_with("todo_predicate!") {
                    "        let _ = v0;".to_string()
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&props_path, translated).unwrap();
        // Fabricate the clean run report with the real artifact sha.
        let report_path = out_dir.join("pipe-demo.check.json");
        let mut report: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&report_path).unwrap_or_else(|_| {
                // model_check stage never ran; build the report by
                // running the model_check stage first.
                run_model_check_stage(
                    &specs,
                    out,
                    &model_check::Bound::default(),
                    &Backends::default(),
                );
                std::fs::read_to_string(&report_path).unwrap()
            }))
            .unwrap();
        report["outcome"] = serde_json::json!("no_counterexample");
        std::fs::write(&report_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
        let stage = run_verify_stage(&specs, out);
        assert_eq!(stage.status, "passed", "{:?}", stage.detail);
        assert_eq!(
            stage.detail.as_ref().unwrap()["verified"][0]["status"],
            "verified"
        );
    }
}
