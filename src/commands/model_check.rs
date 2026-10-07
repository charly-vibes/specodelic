// Command handlers (split from main.rs — specodelic-g17 file_lines ratchet).
use crate::{ModelBackend, emit_report, parse_batch};

use genesis::guide::{Output, OutputFormat, Verbosity};
use specodelic::{citation_corpus, human, lint, model_check, orchestrate, verify};

/// The scope law's labeled failure as one envelope: the label names the
/// violation kind in the message (stderr channel), the remediation hint
/// rides the next-step channel (design D9).
fn scope_violation_output(v: &citation_corpus::ScopeViolation) -> Output<serde_json::Value> {
    Output::failure(format!("{}: {}", v.label, v.message)).with_next_step(v.hint.clone())
}

/// The artifact filename stem for a spec: the file stem when on disk,
/// else the intent id with `.` → `-`.
/// The model-check invocation parameters beyond the spec paths — the
/// artifact directory, the stated bound, and the backend selection,
/// grouped to keep the shared emit plumbing (cli/format/verbosity/
/// streams) within the arg lint.
pub(crate) struct CheckTarget {
    pub(crate) out_dir: String,
    pub(crate) bound: model_check::Bound,
    pub(crate) backend: ModelBackend,
    pub(crate) tlc_jar: Option<String>,
}

/// Run `spk model-check` — the model_check step (specs/model_check.md).
/// Never re-compiles: consumes the compiled `<stem>.tla` artifact from
/// `out_dir`, runs the native stateright backend within the stated
/// bound, and persists a `<stem>.check.json` run report carrying the
/// artifact's SHA-256 so `verify` can detect stale clean results.
pub(crate) fn cmd_model_check(
    paths: &[String],
    target: CheckTarget,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let out_dir = target.out_dir.as_str();
    // Backend selection first: an unusable backend configuration is an
    // invocation error (specodelic-7rr), never a run or a verdict.
    let tlc_paths = match target.backend {
        ModelBackend::Tlc => {
            let Some(jar) = target.tlc_jar.clone() else {
                let out: Output<serde_json::Value> =
                    Output::failure("--backend tlc requires --tlc-jar <tla2tools.jar>")
                        .with_next_step(
                            "download tla2tools.jar from https://github.com/tlaplus/tlaplus/releases and pass it via --tlc-jar (the JVM binary comes from PATH or SPK_TLC_JAVA)",
                        );
                emit_report(out, None, format, verbosity, stdout, stderr);
                return 2;
            };
            Some(model_check::TlcPaths {
                // SPK_TLC_JAVA is a test/ops seam for the JVM binary —
                // documented, read at invocation time.
                java: std::env::var("SPK_TLC_JAVA")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|_| std::path::PathBuf::from("java")),
                jar: std::path::PathBuf::from(jar),
            })
        }
        ModelBackend::Stateright => {
            if target.tlc_jar.is_some() {
                eprintln!("warning: --tlc-jar is only used by --backend tlc; ignoring it");
            }
            None
        }
    };
    let (specs, _checklists, notes, _parse_errors) = parse_batch(paths, verbosity);
    if specs.is_empty() {
        let mut out: Output<serde_json::Value> = Output::failure("no spec files to model-check")
            .with_next_step(
                "pass spec files or a directory; each must have compiled artifacts (run: specodelic compile <files>)",
            );
        // Hostile-input/parse notes name themselves even here (suz Ro5, CORR-001).
        for n in &notes {
            out = out.with_warning(n.clone());
        }
        emit_report(out, None, format, verbosity, stdout, stderr);
        // Invocation error: nothing to check (specodelic-7rr item 2).
        return 2;
    }

    // Scope law (design D9): the labeled failure fires before any write —
    // no evaluation and no report may precede it.
    if let Err(v) = citation_corpus::check_corpus_scope(&specs) {
        emit_report(
            scope_violation_output(&v),
            None,
            format,
            verbosity,
            stdout,
            stderr,
        );
        return 2;
    }

    let mut checked: Vec<serde_json::Value> = vec![];
    let mut failed: Vec<serde_json::Value> = vec![];
    let warnings: Vec<String> = notes;

    // The ONE claim-gated pipeline (design D4): the native CLI and the
    // orchestrate stage share this exact path — backend pass, corpus
    // resolution, kernel claims, one merge, one aggregate — so the
    // aggregate verdict cannot drift between report views. Only the
    // CLI's rich checked-entry rendering differs.
    for run in
        orchestrate::run_claim_gated_model_check(&specs, out_dir, &target.bound, tlc_paths.as_ref())
    {
        match run.outcome {
            Ok(c) => {
                // CLI output and persisted report carry identical
                // statuses (design D9): every merged entry — exec
                // outcomes, resolved citations, kernel claims — with
                // the labeled reason attached for unknown statuses.
                checked.push(serde_json::json!({
                    "file": c.file,
                    "id": c.id,
                    "outcome": c.report.outcome,
                    "backend": c.report.backend,
                    "bound": c.report.bound,
                    "invariants_checked": c.report.invariants_checked,
                    "invariant_statuses": c.statuses_json,
                    "violated_invariant_id": c.report.violated_invariant_id,
                    "trace": c.report.trace,
                    "states_explored": c.report.states_explored,
                    "artifact_sha256": c.report.artifact_sha256,
                    "written": c.written,
                }));
            }
            Err((stage, message)) => {
                failed.push(serde_json::json!({
                    "file": run.file,
                    "id": run.id,
                    "stage": stage,
                    "message": message,
                }));
            }
        }
    }

    let mut payload = serde_json::json!({
        "files_checked": checked.len(),
        "files_failed": failed.len(),
        "checked": checked,
        "failed": failed,
    });
    // Meter contract: `.data.outcome` for the single-file case.
    if checked.len() == 1 {
        payload["outcome"] = checked[0]["outcome"].clone();
    }
    let mut out = Output::success(payload.clone());
    for w in &warnings {
        out = out.with_warning(w.clone());
    }
    if failed.is_empty() {
        out = out.with_next_step("run: specodelic verify (consumes the *.check.json reports)");
    } else {
        out = out.with_next_step(
            "fix the labeled failures (missing artifacts: run specodelic compile first)",
        );
    }
    emit_report(
        out,
        Some(human::model_check(&payload)),
        format,
        verbosity,
        stdout,
        stderr,
    );
    if failed.is_empty() { 0 } else { 1 }
}

/// Run `spk verify` — the verified transition (specs/verify.md).
/// Never re-compiles and never re-runs the model checker: consumes the
/// `*_props.rs` artifact (staleness-checked against the current spec,
/// then really executed block by block) and the `<stem>.check.json` run
/// report (staleness-checked against the current `<stem>.tla`).
/// `verified` is the conjunction — either gate alone fails, and every
/// blocking stage is named with a remediation hint.
/// The orchestrate invocation parameters beyond the spec paths —
/// artifact directory, the model_check bound, and the backend
/// selection, grouped to keep the shared emit plumbing within the arg
/// lint (the CheckTarget pattern).
pub(crate) struct OrchestrateTarget {
    pub(crate) out_dir: String,
    pub(crate) bound: model_check::Bound,
    pub(crate) backend: ModelBackend,
    pub(crate) tlc_jar: Option<String>,
}

/// Run `spk orchestrate` — the full pipeline (specs/orchestrate.md):
/// lint's Checker Ownership checkers in dependency order, then compile,
/// model_check, verify in sequence, halting at the first stage that
/// fails. Rename is never part of a run; external_completeness runs
/// when a checklist is declared and never gates.
pub(crate) fn cmd_orchestrate(
    paths: &[String],
    target: OrchestrateTarget,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let out_dir = target.out_dir.as_str();
    // Backend selection first: an unusable backend configuration is an
    // invocation error (specodelic-7rr), never a run or a verdict.
    let tlc_paths = match target.backend {
        ModelBackend::Tlc => {
            let Some(jar) = target.tlc_jar.as_deref() else {
                let out: Output<serde_json::Value> =
                    Output::failure("--backend tlc requires --tlc-jar <tla2tools.jar>")
                        .with_next_step(
                            "download tla2tools.jar from https://github.com/tlaplus/tlaplus/releases and pass it via --tlc-jar (the JVM binary comes from PATH or SPK_TLC_JAVA)",
                        );
                emit_report(out, None, format, verbosity, stdout, stderr);
                return 2;
            };
            Some(model_check::TlcPaths {
                java: std::env::var("SPK_TLC_JAVA")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|_| std::path::PathBuf::from("java")),
                jar: std::path::PathBuf::from(jar),
            })
        }
        ModelBackend::Stateright => None,
    };
    let (specs, checklists, notes, parse_errors) = parse_batch(paths, verbosity);
    // EDGE-001 (Ro5 over specodelic-8kk): orchestrate requires at least
    // one spec file even when a checklist is declared — every stage
    // would otherwise pass vacuously over an empty file set (the 6pi
    // false-green class). Checklist-only repos are `spk lint`'s
    // territory: external_completeness is a lint checker, not a stage.
    if specs.is_empty() {
        let mut out: Output<serde_json::Value> = Output::failure(
            "no spec files found — nothing to orchestrate",
        )
        .with_next_step(if checklists.is_empty() {
            "pass files or directories containing *.md specs with YAML frontmatter (directories are searched recursively)"
        } else {
            "the declared checklist has no spec files to gate — orchestrate drives the four-stage pipeline over spec files; run `specodelic lint` for a checklist-only repo"
        });
        for n in notes.iter().chain(&parse_errors) {
            out = out.with_warning(n.clone());
        }
        emit_report(out, None, format, verbosity, stdout, stderr);
        // Invocation error (specodelic-7rr item 2): nothing was processed.
        return 2;
    }
    // Scope law (design D9): the labeled failure fires before any write —
    // no compile/model-check/verify stage may precede it.
    if let Err(v) = citation_corpus::check_corpus_scope(&specs) {
        emit_report(
            scope_violation_output(&v),
            None,
            format,
            verbosity,
            stdout,
            stderr,
        );
        return 2;
    }
    let orchestration = orchestrate::orchestrate(
        &specs,
        &checklists,
        &orchestrate::ParseInput {
            notes,
            parse_errors,
        },
        out_dir,
        &target.bound,
        &orchestrate::Backends { tlc: tlc_paths },
    );
    let payload = serde_json::to_value(&orchestration).unwrap_or_default();
    let mut out = Output::success(payload.clone());
    for w in lint::advisory_findings(&specs) {
        out = out.with_warning(format!("{} [{}] {}", w.file, w.rule_id, w.message));
    }
    if orchestration.overall == "succeeded" {
        out = out.with_next_step(
            "verified is not cached — re-run orchestrate after any edit to the spec or its artifacts",
        );
    } else {
        out = out.with_next_step(
            "inspect the first failed (or skipped-after-failure) stage in .data.stages — each stage's detail names the exact findings",
        );
    }
    emit_report(
        out,
        Some(human::orchestrate(&payload)),
        format,
        verbosity,
        stdout,
        stderr,
    );
    if orchestration.overall == "succeeded" {
        0
    } else {
        1
    }
}

pub(crate) fn cmd_verify(
    paths: &[String],
    out_dir: &str,
    timeout_secs: u64,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let dir = std::path::Path::new(out_dir);
    let (specs, _checklists, notes, _parse_errors) = parse_batch(paths, verbosity);
    if specs.is_empty() {
        let mut out: Output<serde_json::Value> = Output::failure("no spec files to verify")
            .with_next_step(
                "pass spec files or a directory; each must have compiled artifacts (run: specodelic compile <files>)",
            );
        // Hostile-input/parse notes name themselves even here (suz Ro5, CORR-001).
        for n in &notes {
            out = out.with_warning(n.clone());
        }
        emit_report(out, None, format, verbosity, stdout, stderr);
        // Invocation error: nothing to verify (specodelic-7rr item 2).
        return 2;
    }

    // Scope law (design D9): the labeled failure fires before any write —
    // no property execution may precede it.
    if let Err(v) = citation_corpus::check_corpus_scope(&specs) {
        emit_report(
            scope_violation_output(&v),
            None,
            format,
            verbosity,
            stdout,
            stderr,
        );
        return 2;
    }

    // 0 = unbounded (the one documented escape hatch for legitimately
    // long suites); the default bounds a hanging predicate.
    let runner = verify::CargoRunner {
        timeout_secs: (timeout_secs > 0).then_some(timeout_secs),
    };
    let mut verified: Vec<serde_json::Value> = vec![];
    let mut blocked: Vec<serde_json::Value> = vec![];
    // The ONE both-gates evaluation loop (design D4): the native CLI and
    // the orchestrate stage share this exact path — same gate evaluation,
    // same verdict — so the aggregate verdict cannot drift between report
    // views. Only the CLI's rich entry rendering (hint + gate states)
    // differs.
    for fv in verify::evaluate_file_verdicts(&specs, dir, &runner) {
        let v = fv.verdict;
        let entry = serde_json::json!({
            "file": fv.file,
            "id": fv.id,
            "status": v.status,
            "message": v.message,
            "hint": v.hint,
            "properties": props_gate_json(&fv.properties),
            "model": model_gate_json(&fv.model),
        });
        if v.status == "verified" {
            verified.push(entry);
        } else {
            blocked.push(entry);
        }
    }

    let mut payload = serde_json::json!({
        "files_verified": verified.len(),
        "files_blocked": blocked.len(),
        "verified": verified,
        "blocked": blocked,
    });
    // Meter contract: `.data.status` for the single-file case —
    // "verified" exactly when both gates hold, else the blocking stage.
    let first = verified.first().or_else(|| blocked.first());
    if let Some(first) = first {
        payload["status"] = first["status"].clone();
        payload["message"] = first["message"].clone();
        payload["hint"] = first["hint"].clone();
    }
    let mut out = Output::success(payload.clone());
    for w in &notes {
        out = out.with_warning(w.clone());
    }
    if blocked.is_empty() {
        out = out.with_next_step(
            "verified is not cached — re-run verify after any edit to the spec or its artifacts",
        );
    } else {
        let hint = blocked[0]["hint"].as_str().unwrap_or_default().to_string();
        out = out.with_next_step(hint);
    }
    emit_report(
        out,
        Some(human::verify(&payload)),
        format,
        verbosity,
        stdout,
        stderr,
    );
    if blocked.is_empty() { 0 } else { 1 }
}

/// The properties gate as envelope JSON: state name plus per-block
/// results (id, case, pass, and the shrunk-input detail for failures).
pub(crate) fn props_gate_json(gate: &verify::PropertiesGate) -> serde_json::Value {
    let state = match &gate.state {
        verify::PropsGateState::Pass => "pass",
        verify::PropsGateState::Failed(_) => "failed",
        verify::PropsGateState::Stale => "stale",
        verify::PropsGateState::MissingArtifact => "missing_artifact",
        verify::PropsGateState::Uncompilable(_) => "uncompilable",
        verify::PropsGateState::RunnerUnavailable(_) => "runner_unavailable",
        verify::PropsGateState::TimedOut { .. } => "timed_out",
    };
    let detail = match &gate.state {
        verify::PropsGateState::Failed(s)
        | verify::PropsGateState::Uncompilable(s)
        | verify::PropsGateState::RunnerUnavailable(s) => Some(s.clone()),
        verify::PropsGateState::TimedOut { secs, detail } => Some(format!(
            "{secs}s wall-clock bound hit; partial output: {detail}"
        )),
        _ => None,
    };
    serde_json::json!({
        "state": state,
        "detail": detail,
        "blocks": gate.blocks.iter().map(|b| serde_json::json!({
            "id": b.id,
            "case": b.case,
            "fn": b.fn_name,
            "passed": b.passed,
            "detail": b.detail,
        })).collect::<Vec<_>>(),
    })
}

/// The model gate as envelope JSON: state name, the outcome when the
/// report was readable, and the staleness detail.
pub(crate) fn model_gate_json(gate: &verify::ModelGateState) -> serde_json::Value {
    match gate {
        verify::ModelGateState::Clean => serde_json::json!({"state": "clean"}),
        verify::ModelGateState::NotClean { outcome } => serde_json::json!({
            "state": "not_clean",
            "outcome": outcome,
        }),
        verify::ModelGateState::Stale { detail } => {
            serde_json::json!({"state": "stale", "detail": detail})
        }
        verify::ModelGateState::Missing { detail } => {
            serde_json::json!({"state": "missing", "detail": detail})
        }
    }
}
