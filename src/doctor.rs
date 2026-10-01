//! `spk doctor` check suite on the genesis doctor framework (specodelic-sok).
//!
//! Purpose: shared diagnostic scaffolding — named checks, structured
//! report, exit codes, and `--fix` dispatch with verify-after-fix — so
//! the CLI layer only maps the report onto the envelope. Responsibilities:
//! the three *issue* checks (specs/ directory, beads, SPECODELIC block)
//! as [`DoctorCheck`] impls; the report run ([`run_checks`]); the
//! workspace *facts* (mode, core-spec presence, corpus discovery) as
//! detail helpers; and the payload assembly that keeps the doctor
//! capability contract (openspec/specs/doctor/spec.md).
//!
//! Rationale (framework fit): genesis [`DoctorRunner`] classifies any
//! `Severity::Warning`/`Advisory` finding as `CheckStatus::Warn` and only
//! `Severity::Error` as `Fail`, so every check below speaks `Warning` and
//! the report can never fail — the capability's warn-never-fail invariant
//! holds by construction, not by discipline. Facts (mode, discovery) are
//! not checks — a consumer workspace is a legitimate state, not a
//! finding — so they render from helpers into the envelope payload while
//! the report carries exactly the repairable/missing-things.

use std::path::Path;

use genesis::doctor::{DoctorCheck, DoctorReport, DoctorRunner};
use genesis::suite_linter::{LintResult, Severity};

use crate::blocks;
use crate::guide;

/// Workspace mode: self-hosting when the repo carries the format's own
/// core spec, consumer otherwise (add-embedded-aix-guide task 5.1).
pub fn workspace_mode(root: &Path) -> &'static str {
    if root.join("specs/specodelic.md").is_file() {
        "self_hosting"
    } else {
        "consumer"
    }
}

/// The `mode` payload row — a fact about the workspace, never a finding.
pub fn mode_detail(root: &Path) -> String {
    match workspace_mode(root) {
        "self_hosting" => "self_hosting — the corpus lives here".to_string(),
        _ => "consumer — the format is provided by the installed binary".to_string(),
    }
}

/// The `core format spec` payload row — self-hosting expects it; a
/// consumer corpus legitimately lacks it (the embedded guide serves the
/// format instead), so the consumer note is informational.
pub fn core_spec_detail(root: &Path) -> String {
    match workspace_mode(root) {
        "self_hosting" => "ok (specs/specodelic.md)".to_string(),
        _ => format!(
            "not present (consumer mode — the binary embeds {})",
            guide::FORMAT_REVISION
        ),
    }
}

/// The `corpus discovery` payload row (gh#2.2): name where the specs
/// actually live so the working invocation is never trial-and-error.
pub fn discovery_detail(root: &Path) -> String {
    if root.join("specs").is_dir() {
        "ok (specs/)".to_string()
    } else if root.join("openspec").is_dir() {
        let n = count_md_files(&root.join("openspec"));
        format!("found openspec/ ({n} spec file(s)) — lint it with: spk lint openspec")
    } else {
        "no corpus — pass a directory containing *.md specs; hidden and build dirs are skipped"
            .to_string()
    }
}

/// Count `*.md` files under `dir` recursively, skipping hidden, `target/`
/// and `node_modules/` directories (same skip policy as the CLI's
/// `collect_specs` walk). Used for the openspec-tree discovery detail.
fn count_md_files(dir: &Path) -> usize {
    fn skipped(name: &std::ffi::OsStr) -> bool {
        match name.to_str() {
            Some(n) => n.starts_with('.') || n == "target" || n == "node_modules",
            None => true,
        }
    }
    let mut count = 0usize;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                if p.file_name().is_some_and(|n| !skipped(n)) {
                    stack.push(p);
                }
            } else if p.extension().is_some_and(|e| e == "md") {
                count += 1;
            }
        }
    }
    count
}

// ── Checks (issue detection; empty = pass) ────────────────────────────

/// `specs/` presence — the natural corpus root.
pub struct SpecsDirCheck;

impl DoctorCheck for SpecsDirCheck {
    fn name(&self) -> &'static str {
        "specs/ directory"
    }
    fn description(&self) -> &'static str {
        "check that a specs/ corpus directory exists"
    }
    fn run(&self, root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        if root.join("specs").is_dir() {
            Ok(vec![])
        } else {
            Ok(vec![LintResult::with_fix(
                "missing — start a corpus with: spk new",
                Severity::Warning,
                "spk new <intent.id>",
            )])
        }
    }
}

/// Beads issue tracker initialized (`.beads/config.yaml`).
pub struct BeadsCheck;

impl DoctorCheck for BeadsCheck {
    fn name(&self) -> &'static str {
        "beads"
    }
    fn description(&self) -> &'static str {
        "check that the beads issue tracker is initialized (.beads/config.yaml)"
    }
    fn run(&self, root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        if root.join(".beads/config.yaml").is_file() {
            Ok(vec![])
        } else {
            Ok(vec![LintResult::with_fix(
                "not initialized — run: bd init",
                Severity::Warning,
                "bd init",
            )])
        }
    }
}

/// The AGENTS.md managed block — missing/stale is the one finding the
/// doctor can repair itself (`--fix` → `blocks::inject_into`, the same
/// code path `spk init` uses, idempotent and content-preserving).
pub struct BlockCheck;

impl BlockCheck {
    /// Classify the block state: `Ok(())` current, `Err(message)` for
    /// missing/stale/revision-less (each names the `spk init` remediation).
    pub fn state(root: &Path) -> Result<(), String> {
        let agents = root.join(blocks::BLOCK_FILE);
        if !blocks::has_block(&agents) {
            return Err(
                "missing — agents in this repo can't see the spec rules; run: spk init".to_string(),
            );
        }
        match blocks::block_format_revision(&agents) {
            Some(rev) => {
                let embedded = guide::revision_number(guide::FORMAT_REVISION).unwrap_or(0);
                if rev < embedded {
                    Err(format!(
                        "stale — block names Revision {rev}, binary embeds {} ; run: spk init",
                        guide::FORMAT_REVISION
                    ))
                } else {
                    Ok(())
                }
            }
            None => Err("present but names no revision — run: spk init to refresh".to_string()),
        }
    }

    /// The current-block payload row ("ok (Revision N ≥ embedded)").
    pub fn ok_detail(root: &Path) -> String {
        let rev = blocks::block_format_revision(&root.join(blocks::BLOCK_FILE))
            .unwrap_or_else(|| guide::revision_number(guide::FORMAT_REVISION).unwrap_or(0));
        format!("ok (Revision {rev} ≥ embedded)")
    }
}

impl DoctorCheck for BlockCheck {
    fn name(&self) -> &'static str {
        "SPECODELIC block"
    }
    fn description(&self) -> &'static str {
        "check that AGENTS.md carries a current SPECODELIC managed block"
    }
    fn run(&self, root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        match Self::state(root) {
            Ok(()) => Ok(vec![]),
            Err(message) => Ok(vec![LintResult::with_fix(
                message,
                Severity::Warning,
                "spk init",
            )]),
        }
    }
    fn auto_fixable(&self) -> bool {
        true
    }
    fn fix(&self, root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        blocks::inject_into(&root.join(blocks::BLOCK_FILE))?;
        Ok(vec![])
    }
}

// ── Runner ────────────────────────────────────────────────────────────

/// Run the check suite (in the doctor's canonical order). `fix`
/// dispatches auto-fixable checks (the SPECODELIC block) with the
/// runner's verify-after-fix — a fix that doesn't cure the finding is
/// reported as a failure, honestly.
pub fn run_checks(root: &Path, fix: bool) -> Result<DoctorReport, Box<dyn std::error::Error>> {
    let runner = DoctorRunner::new(vec![
        Box::new(SpecsDirCheck),
        Box::new(BeadsCheck),
        Box::new(BlockCheck),
    ])
    .with_tool_name("specodelic");
    runner.run(root, fix)
}

/// The report entry for `name`, if it found something (Warn/Fail).
fn report_finding<'a>(
    report: &'a DoctorReport,
    name: &str,
) -> Option<&'a genesis::doctor::CheckEntry> {
    report
        .checks
        .iter()
        .find(|c| c.name == name && c.status.is_issue())
}

/// Assemble the doctor envelope payload — same shape and strings as the
/// pre-framework hand-rolled version (`mode`, `checks` as
/// `[name, detail]` pairs, `format_revision`) so human rendering and the
/// doctor capability scenarios stay stable. Facts render from the detail
/// helpers; issue findings render from the report (post-`--fix` a
/// repaired check shows its current fact, not the stale finding).
pub fn payload(root: &Path, report: &DoctorReport) -> serde_json::Value {
    let specs_row = match report_finding(report, "specs/ directory") {
        Some(e) => e.message.clone(),
        None => "ok".to_string(),
    };
    let beads_row = match report_finding(report, "beads") {
        Some(e) => e.message.clone(),
        None => "ok (.beads/config.yaml)".to_string(),
    };
    let block_row = match report_finding(report, "SPECODELIC block") {
        Some(e) => e.message.clone(),
        None => BlockCheck::ok_detail(root),
    };
    let checks: Vec<(String, String)> = vec![
        ("mode".into(), mode_detail(root)),
        ("specs/ directory".into(), specs_row),
        ("core format spec".into(), core_spec_detail(root)),
        ("beads".into(), beads_row),
        ("SPECODELIC block".into(), block_row),
        ("corpus discovery".into(), discovery_detail(root)),
    ];
    serde_json::json!({
        "mode": workspace_mode(root),
        "checks": checks,
        "format_revision": guide::FORMAT_REVISION,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks;
    use genesis::doctor::DoctorCheck;
    use genesis::suite_linter::Severity;

    fn fixture() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn self_hosting_root(dir: &std::path::Path) -> std::path::PathBuf {
        let specs = dir.join("specs");
        std::fs::create_dir_all(&specs).unwrap();
        std::fs::write(
            specs.join("specodelic.md"),
            "---\nid: specodelic\nkind: intent\nstatement: \"THE format SHALL be described\"\n---\n\n## Revision 12\n",
        )
        .unwrap();
        dir.to_path_buf()
    }

    // -- mode detection (constraint mode_detection) ------------------------

    #[test]
    fn mode_is_self_hosting_when_core_spec_present() {
        let dir = fixture();
        let root = self_hosting_root(dir.path());
        assert_eq!(workspace_mode(&root), "self_hosting");
        assert!(mode_detail(&root).starts_with("self_hosting"));
    }

    #[test]
    fn mode_is_consumer_otherwise() {
        let dir = fixture();
        assert_eq!(workspace_mode(dir.path()), "consumer");
    }

    // -- beads check --------------------------------------------------------

    #[test]
    fn beads_check_passes_when_initialized() {
        let dir = fixture();
        std::fs::create_dir_all(dir.path().join(".beads")).unwrap();
        std::fs::write(dir.path().join(".beads/config.yaml"), "x: 1\n").unwrap();
        assert!(BeadsCheck.run(dir.path()).unwrap().is_empty());
    }

    #[test]
    fn beads_check_warns_when_missing() {
        let dir = fixture();
        let results = BeadsCheck.run(dir.path()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].severity, Severity::Warning);
        assert!(results[0].message.contains("bd init"));
    }

    // -- SPECODELIC block check (auto-fixable) ------------------------------

    #[test]
    fn block_check_warns_when_missing_and_names_the_fix() {
        let dir = fixture();
        let results = BlockCheck.run(dir.path()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].severity, Severity::Warning);
        assert!(results[0].message.contains("spk init"));
    }

    #[test]
    fn block_check_passes_when_current() {
        let dir = fixture();
        let path = dir.path().join(blocks::BLOCK_FILE);
        std::fs::write(&path, "# My repo\n").unwrap();
        blocks::inject_into(&path).unwrap();
        assert!(BlockCheck.run(dir.path()).unwrap().is_empty());
    }

    #[test]
    fn block_check_warns_when_stale() {
        let dir = fixture();
        // a block naming a prehistoric revision → stale vs embedded Revision 12
        std::fs::write(
            dir.path().join(blocks::BLOCK_FILE),
            "# repo\n\n<!-- SPECODELIC:START -->\nembedded format revision: specodelic.md Revision 1\n<!-- SPECODELIC:END -->\n",
        )
        .unwrap();
        let results = BlockCheck.run(dir.path()).unwrap();
        assert_eq!(results.len(), 1);
        assert!(
            results[0].message.contains("stale"),
            "{}",
            results[0].message
        );
    }

    #[test]
    fn block_check_is_auto_fixable_and_fix_injects() {
        let dir = fixture();
        assert!(BlockCheck.auto_fixable());
        assert!(!BlockCheck.run(dir.path()).unwrap().is_empty());
        BlockCheck.fix(dir.path()).unwrap();
        assert!(BlockCheck.run(dir.path()).unwrap().is_empty());
        let text = std::fs::read_to_string(dir.path().join(blocks::BLOCK_FILE)).unwrap();
        assert!(text.contains("SPECODELIC:START"));
    }

    // -- facts: corpus discovery --------------------------------------------

    #[test]
    fn discovery_passes_when_specs_dir_exists() {
        let dir = fixture();
        std::fs::create_dir_all(dir.path().join("specs")).unwrap();
        assert_eq!(discovery_detail(dir.path()), "ok (specs/)");
    }

    #[test]
    fn discovery_passes_on_openspec_tree_and_counts_files() {
        let dir = fixture();
        let nested = dir.path().join("openspec/specs/thing");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("spec.md"), "---\nid: spec\n---\n").unwrap();
        // hidden dirs are skipped
        std::fs::create_dir_all(dir.path().join("openspec/.hidden")).unwrap();
        std::fs::write(dir.path().join("openspec/.hidden/x.md"), "x").unwrap();
        let detail = discovery_detail(dir.path());
        assert!(
            detail.contains("found openspec/ (1 spec file(s))"),
            "detail: {detail}"
        );
    }

    #[test]
    fn discovery_advises_when_no_corpus() {
        let dir = fixture();
        assert!(discovery_detail(dir.path()).contains("no corpus"));
    }

    // -- payload assembly (capability contract: same shape) ------------------

    #[test]
    fn payload_matches_the_hand_rolled_shape() {
        let dir = fixture();
        let root = self_hosting_root(dir.path());
        std::fs::create_dir_all(root.join(".beads")).unwrap();
        std::fs::write(root.join(".beads/config.yaml"), "x: 1\n").unwrap();
        let path = root.join(blocks::BLOCK_FILE);
        std::fs::write(&path, "# My repo\n").unwrap();
        blocks::inject_into(&path).unwrap();
        let report = run_checks(&root, false).unwrap();
        let p = payload(&root, &report);
        assert_eq!(p["mode"], "self_hosting");
        assert_eq!(p["format_revision"], guide::FORMAT_REVISION);
        let checks: Vec<&serde_json::Value> = p["checks"].as_array().unwrap().iter().collect();
        assert_eq!(checks.len(), 6, "all six rows present");
        let rows: Vec<String> = checks
            .iter()
            .map(|c| format!("{}: {}", c[0].as_str().unwrap(), c[1].as_str().unwrap()))
            .collect();
        let joined = rows.join("\n");
        assert!(
            joined.contains("mode: self_hosting — the corpus lives here"),
            "{joined}"
        );
        assert!(joined.contains("specs/ directory: ok"), "{joined}");
        assert!(
            joined.contains("core format spec: ok (specs/specodelic.md)"),
            "{joined}"
        );
        assert!(
            joined.contains("beads: ok (.beads/config.yaml)"),
            "{joined}"
        );
        assert!(
            joined.contains("SPECODELIC block: ok (Revision 12 ≥ embedded)"),
            "{joined}"
        );
        assert!(joined.contains("corpus discovery: ok (specs/)"), "{joined}");
    }

    #[test]
    fn payload_renders_findings_for_broken_workspaces() {
        let dir = fixture();
        let report = run_checks(dir.path(), false).unwrap();
        let p = payload(dir.path(), &report);
        let joined = serde_json::to_string(&p["checks"]).unwrap();
        assert!(joined.contains("start a corpus"), "{joined}");
        assert!(joined.contains("bd init"), "{joined}");
        assert!(joined.contains("spk init"), "{joined}");
        assert!(joined.contains("no corpus"), "{joined}");
    }

    // -- runner integration: never fails, fix dispatch works ----------------

    #[test]
    fn runner_report_never_fails_on_any_workspace_shape() {
        // empty consumer dir: worst case — warnings but zero failures
        let dir = fixture();
        let report = run_checks(dir.path(), false).unwrap();
        assert_eq!(report.summary.fail, 0, "doctor never fails");
        assert!(report.summary.warn > 0, "empty workspace has findings");
        assert_eq!(report.exit_code(), 0);
    }

    #[test]
    fn runner_report_is_healthy_on_a_set_up_self_hosting_repo() {
        let dir = fixture();
        self_hosting_root(dir.path());
        std::fs::create_dir_all(dir.path().join(".beads")).unwrap();
        std::fs::write(dir.path().join(".beads/config.yaml"), "x: 1\n").unwrap();
        let path = dir.path().join(blocks::BLOCK_FILE);
        std::fs::write(&path, "# My repo\n").unwrap();
        blocks::inject_into(&path).unwrap();
        let report = run_checks(dir.path(), false).unwrap();
        assert!(report.is_healthy(), "{report:?}");
        assert_eq!(report.exit_code(), 0);
    }

    #[test]
    fn consumer_report_has_zero_failures_even_when_not_healthy() {
        // a consumer workspace is legitimate — its missing things warn,
        // but the invariant that matters is: never a failure, exit 0
        let dir = fixture();
        std::fs::create_dir_all(dir.path().join("specs")).unwrap();
        let report = run_checks(dir.path(), false).unwrap();
        assert_eq!(report.summary.fail, 0);
        assert_eq!(report.exit_code(), 0);
    }

    #[test]
    fn runner_fix_dispatch_repairs_the_block() {
        let dir = fixture();
        let report = run_checks(dir.path(), true).unwrap();
        let block = report
            .checks
            .iter()
            .find(|c| c.name == "SPECODELIC block")
            .expect("block check present");
        assert!(
            block.status.is_pass(),
            "post-fix the block check passes: {block:?}"
        );
        assert!(
            dir.path().join(blocks::BLOCK_FILE).is_file(),
            "--fix wrote the block"
        );
        // and the payload renders the repaired fact, not the old finding
        let p = payload(dir.path(), &report);
        let checks = p["checks"].as_array().unwrap();
        let block_row = checks
            .iter()
            .find(|c| c[0].as_str() == Some("SPECODELIC block"))
            .unwrap();
        assert!(
            block_row[1].as_str().unwrap().starts_with("ok (Revision"),
            "post-fix payload: {block_row:?}"
        );
    }
}
