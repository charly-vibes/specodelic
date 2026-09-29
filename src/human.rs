//! Human-readable renderings of command reports.
//!
//! Purpose: `--human` must read like a report, not a Rust `{:?}` Debug
//! dump (specodelic-7rr item 3). Responsibilities: one text formatter
//! per report verb (lint, graph, compile, model-check, verify, doctor),
//! rendered from the same report values the JSON envelope carries.
//! Rationale: both renderings derive from one source of truth — the
//! human story and the JSON story cannot drift, and Debug stays internal.

use crate::graph::GraphReport;
use crate::lint::Report as LintReport;
use crate::rename::RenameOutcome;

/// `spk rename` — what moved, and which files now carry the new id.
pub fn rename(outcome: &RenameOutcome) -> String {
    let mut out = format!(
        "rename: {} → {} across {} file(s)",
        outcome.old_id,
        outcome.new_id,
        outcome.writes.len()
    );
    for (path, _) in &outcome.writes {
        out.push_str(&format!("\n  rewritten: {}", path.display()));
    }
    if let Some(old) = &outcome.remove {
        out.push_str(&format!("\n  moved: {} → (new name above)", old.display()));
    }
    out
}

/// `spk lint` — one summary line, then one line per finding carrying the
/// self-describing rule id (`linter.<name>`, the agent-facing anchor).
pub fn lint(report: &LintReport) -> String {
    let mut out = format!(
        "lint: {} file(s) linted, {} finding(s)",
        report.files_linted,
        report.failures()
    );
    for issue in &report.issues {
        out.push_str(&format!(
            "\n  {} [{}] {}",
            issue.file, issue.rule_id, issue.message
        ));
    }
    out
}

/// `spk graph` — one summary line; dangling references, typing violations
/// and supersedes cycles are listed below it.
pub fn graph(report: &GraphReport) -> String {
    let mut out = format!(
        "graph: {} file(s), {} node(s), {} edge(s), {} dangling, {} typing violation(s), {} supersedes cycle(s)",
        report.files,
        report.nodes,
        report.edges.len(),
        report.dangling.len(),
        report.violations.len(),
        report.supersedes_cycles.len()
    );
    for d in &report.dangling {
        out.push_str(&format!("\n  dangling: {d}"));
    }
    for v in &report.violations {
        out.push_str(&format!(
            "\n  typing violation: {} -[[{}]]→ {} ({})",
            v.from, v.edge_kind, v.to, v.reason
        ));
    }
    for c in &report.supersedes_cycles {
        out.push_str(&format!("\n  supersedes cycle: {c}"));
    }
    out
}

/// `spk compile` — per-file: written artifact paths, or the labeled
/// failure stage. `payload` is the same value the JSON envelope carries.
pub fn compile(payload: &serde_json::Value) -> String {
    let mut out = format!(
        "compile: {} compiled, {} failed",
        payload["files_compiled"], payload["files_failed"]
    );
    if let Some(list) = payload["compiled"].as_array() {
        for c in list {
            let written = c["written"]
                .as_array()
                .map(|ws| {
                    ws.iter()
                        .filter_map(|w| w.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            out.push_str(&format!(
                "\n  {} [{}] → {}",
                c["file"].as_str().unwrap_or("?"),
                c["id"].as_str().unwrap_or("?"),
                written
            ));
        }
    }
    if let Some(list) = payload["failed"].as_array() {
        for f in list {
            out.push_str(&format!(
                "\n  {} [{}] FAILED at {}: {}",
                f["file"].as_str().unwrap_or("?"),
                f["id"].as_str().unwrap_or("?"),
                f["stage"].as_str().unwrap_or("?"),
                f["message"].as_str().unwrap_or("?")
            ));
        }
    }
    out
}

/// `spk model-check` — per-file: outcome, states explored, report path.
pub fn model_check(payload: &serde_json::Value) -> String {
    let mut out = format!(
        "model-check: {} checked, {} failed",
        payload["files_checked"], payload["files_failed"]
    );
    if let Some(list) = payload["checked"].as_array() {
        for c in list {
            out.push_str(&format!(
                "\n  {} [{}]: {} ({} states explored) → {}",
                c["file"].as_str().unwrap_or("?"),
                c["id"].as_str().unwrap_or("?"),
                c["outcome"].as_str().unwrap_or("?"),
                c["states_explored"],
                c["written"].as_str().unwrap_or("?")
            ));
        }
    }
    if let Some(list) = payload["failed"].as_array() {
        for f in list {
            out.push_str(&format!(
                "\n  {} [{}] FAILED at {}: {}",
                f["file"].as_str().unwrap_or("?"),
                f["id"].as_str().unwrap_or("?"),
                f["stage"].as_str().unwrap_or("?"),
                f["message"].as_str().unwrap_or("?")
            ));
        }
    }
    out
}

/// `spk verify` — per-file verdict; blocked files carry the blocking
/// stage message.
pub fn merge(report: &crate::merge::MergeReport) -> String {
    let mut out = format!("merge: {}", report.verdict);
    for f in &report.findings {
        out.push_str(&format!("\n  [{}] {}", f.kind, f.message));
    }
    if report.findings.is_empty() {
        out.push_str(" — no findings");
    }
    out
}

pub fn verify(payload: &serde_json::Value) -> String {
    let mut out = format!(
        "verify: {} verified, {} blocked",
        payload["files_verified"], payload["files_blocked"]
    );
    let render = |out: &mut String, entries: &Vec<serde_json::Value>, blocked: bool| {
        for e in entries {
            let line = format!(
                "\n  {} [{}]: {}",
                e["file"].as_str().unwrap_or("?"),
                e["id"].as_str().unwrap_or("?"),
                e["status"].as_str().unwrap_or("?")
            );
            out.push_str(&line);
            if blocked && let Some(m) = e["message"].as_str() {
                out.push_str(&format!(" — {m}"));
            }
        }
    };
    if let Some(list) = payload["verified"].as_array() {
        render(&mut out, list, false);
    }
    if let Some(list) = payload["blocked"].as_array() {
        render(&mut out, list, true);
    }
    out
}

/// `spk doctor` — checks rendered as `name: detail` lines.
pub fn doctor(payload: &serde_json::Value) -> String {
    let mut out = format!(
        "doctor: mode {} (format {})",
        payload["mode"].as_str().unwrap_or("?"),
        payload["format_revision"].as_str().unwrap_or("?")
    );
    if let Some(checks) = payload["checks"].as_array() {
        for check in checks {
            // Vec<(String, String)> serializes as a two-element array.
            let name = check[0].as_str().unwrap_or("?");
            let detail = check[1].as_str().unwrap_or("?");
            out.push_str(&format!("\n  {name}: {detail}"));
        }
    }
    out
}

/// `spk explain` topic list — one `id — title` line per topic.
pub fn explain_topics(payload: &serde_json::Value) -> String {
    let mut out = String::from("explain topics:");
    if let Some(topics) = payload["topics"].as_array() {
        for t in topics {
            out.push_str(&format!(
                "\n  {} — {}",
                t["id"].as_str().unwrap_or("?"),
                t["title"].as_str().unwrap_or("?")
            ));
        }
    }
    out
}

/// `spk init` — the managed-block outcome as one line.
pub fn init(payload: &serde_json::Value) -> String {
    format!(
        "{}: block {} — format {}",
        payload["file"].as_str().unwrap_or("?"),
        payload["block"].as_str().unwrap_or("?"),
        payload["format_revision"].as_str().unwrap_or("?")
    )
}

/// `spk hooks install|uninstall` — outcome, gate dry-run verdict.
pub fn hooks(payload: &serde_json::Value) -> String {
    let outcome = payload["outcome"].as_str().unwrap_or("?");
    match outcome {
        "wired" | "already_wired" => {
            let gate = &payload["gate_dry_run"];
            let gate_line = format!(
                "  gate dry-run: {} — {}",
                if gate["passed"].as_bool().unwrap_or(false) {
                    "passed"
                } else {
                    "FAILED"
                },
                gate["summary"].as_str().unwrap_or("?")
            );
            format!(
                "hooks: {} — {} (stage {})\n{}",
                outcome,
                payload["command"].as_str().unwrap_or("?"),
                payload["stage"].as_str().unwrap_or("?"),
                gate_line
            )
        }
        _ => format!("hooks: {outcome}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn lint_text_carries_rule_ids_and_counts() {
        let report = LintReport {
            files_linted: 2,
            issues: vec![crate::lint::Issue::new(
                "ears_syntax",
                "specs/x.md",
                "statement must be an EARS pattern",
            )],
        };
        let text = lint(&report);
        assert!(
            text.starts_with("lint: 2 file(s) linted, 1 finding(s)"),
            "{text}"
        );
        assert!(text.contains("specs/x.md [linter.ears_syntax]"), "{text}");
        assert!(text.contains("statement must be an EARS pattern"), "{text}");
    }

    #[test]
    fn graph_text_names_dangling_refs() {
        let report = GraphReport {
            files: 1,
            nodes: 3,
            edges: vec![],
            dangling: vec!["nowhere.void".to_string()],
            violations: vec![],
            supersedes_cycles: vec![],
            fan_in: Default::default(),
            fan_out: Default::default(),
        };
        let text = graph(&report);
        assert!(text.contains("1 dangling"), "{text}");
        assert!(text.contains("dangling: nowhere.void"), "{text}");
    }

    #[test]
    fn payload_formatters_render_from_envelope_values() {
        let compile_payload = json!({
            "files_compiled": 1,
            "files_failed": 1,
            "compiled": [{"file": "a.md", "id": "a.x", "written": ["a.toml"]}],
            "failed": [{"file": "b.md", "id": "b.y", "stage": "model_to_tla", "message": "boom"}],
        });
        let text = compile(&compile_payload);
        assert!(text.contains("compile: 1 compiled, 1 failed"), "{text}");
        assert!(text.contains("a.md [a.x] → a.toml"), "{text}");
        assert!(text.contains("FAILED at model_to_tla: boom"), "{text}");

        let mc_payload = json!({
            "files_checked": 1,
            "files_failed": 0,
            "checked": [{"file": "a.md", "id": "a.x", "outcome": "exploration_only", "states_explored": 7, "written": "a.check.json"}],
        });
        let text = model_check(&mc_payload);
        assert!(text.contains("exploration_only"), "{text}");
        assert!(text.contains("7 states explored"), "{text}");

        let v_payload = json!({
            "files_verified": 1,
            "files_blocked": 1,
            "verified": [{"file": "a.md", "id": "a.x", "status": "verified"}],
            "blocked": [{"file": "b.md", "id": "b.y", "status": "properties_failed", "message": "prop boom"}],
        });
        let text = verify(&v_payload);
        assert!(text.contains("a.md [a.x]: verified"), "{text}");
        assert!(text.contains("properties_failed — prop boom"), "{text}");

        let d_payload = json!({
            "mode": "self_hosting",
            "format_revision": "specodelic.md Revision 8",
            "checks": [["mode", "self_hosting"], ["specs/ directory", "ok"]],
        });
        let text = doctor(&d_payload);
        assert!(text.contains("doctor: mode self_hosting"), "{text}");
        assert!(text.contains("\n  specs/ directory: ok"), "{text}");
    }

    #[test]
    fn debug_markers_never_appear() {
        // The whole point (specodelic-7rr item 3): no Rust Debug shapes.
        for text in [
            lint(&LintReport::default()),
            graph(&GraphReport {
                files: 0,
                nodes: 0,
                edges: vec![],
                dangling: vec![],
                violations: vec![],
                supersedes_cycles: vec![],
                fan_in: Default::default(),
                fan_out: Default::default(),
            }),
            compile(&json!({"files_compiled": 0, "files_failed": 0})),
            model_check(&json!({"files_checked": 0, "files_failed": 0})),
            verify(&json!({"files_verified": 0, "files_blocked": 0})),
            doctor(&json!({"mode": "consumer", "checks": []})),
        ] {
            for marker in ["Object {", "Report {", "Issue {", "String("] {
                assert!(!text.contains(marker), "Debug marker {marker:?} in: {text}");
            }
        }
    }
}
