//! Cross-file citation resolution and command scope law (design D9,
//! add-min-expr-kernel §3.6 — specodelic-hb4).
//!
//! Purpose: resolve guard-citation expressions over the complete explicit
//! invocation corpus — bare names within the citing file, qualified names
//! as exact corpus keys — in deterministic dependency order, with cycles,
//! missing targets, and property rows reported `unknown` with a reason;
//! and enforce the command scope law shared by model-check, verify and
//! orchestrate: corpus intent ids are unique within the invocation
//! (`duplicate_corpus_identity`; Revision 18 retired the former
//! `id: spec` isolated-scope rule with `id: spec` itself).
//!
//! Responsibilities: `check_corpus_scope` (the pre-write scope guard);
//! `resolve_corpus_citations` (whole-corpus status resolution over
//! same-run evidence only — no filesystem discovery, never a suffix
//! match); `statuses_with_reasons` (the command-output shape).
//! Rationale: the command path (native CLI and orchestrate) must publish
//! one canonical status per citation invariant in both CLI output and
//! persisted reports; resolution lives beside — never inside — the
//! pinned per-run backend (`model_check.rs` is ratcheted), so per-run
//! semantics stay untouched and corpus composition is a command-layer
//! concern over the same citation algebra (`compile::evaluate_citation`).

use std::collections::{BTreeMap, BTreeSet};

use crate::compile::{self, CitationExpr, ThreeValued};
use crate::model_check::{self, InvariantStatus, RunReport, TlcPaths};
use crate::spec::Spec;

/// One file's backend run in a command invocation: identity (display
/// path, intent id, artifact stem) plus the run result.
pub struct FileRun {
    pub file: String,
    pub id: String,
    pub stem: String,
    pub result: Result<RunReport, model_check::ModelCheckError>,
}

/// Command-path pass 1 (design D9): run every file's backend — the
/// per-file semantics are exactly the library run (`model_check::run`,
/// `run_executable`, `run_tlc`); a missing compiled artifact is a
/// labeled error carried in the file's result, never a run.
pub fn run_backend_pass(
    specs: &[Spec],
    out_dir: &str,
    bound: &model_check::Bound,
    tlc: Option<&TlcPaths>,
) -> Vec<FileRun> {
    let mut runs = Vec::new();
    for spec in specs {
        let file = spec
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| format!("<{}>", spec.intent.id));
        let stem = compile::artifact_stem(spec);
        let tla_path = std::path::Path::new(out_dir).join(format!("{stem}.tla"));
        // The compiled module is the run's input — a missing artifact is
        // a labeled error with a remediation hint, never a run.
        let tla_artifact = match std::fs::read(&tla_path) {
            Ok(bytes) => bytes,
            Err(e) => {
                runs.push(FileRun {
                    file,
                    id: spec.intent.id.clone(),
                    stem,
                    result: Err(model_check::ModelCheckError {
                        stage: "missing_artifact".into(),
                        message: format!(
                            "no compiled module at {}: {e} — model-check consumes compile's output, it never re-compiles (run: specodelic compile <files>)",
                            tla_path.display()
                        ),
                    }),
                });
                continue;
            }
        };
        let ir = compile::extract_model_ir(spec);
        let result = match tlc {
            Some(tlc) => model_check::run_tlc(&ir, &tla_path, &tla_artifact, bound, tlc),
            // Executable invariants (Revision 15): fragment-bearing IR
            // goes through the scratch-crate run — no_counterexample
            // becomes producible; fragment-less IR keeps the in-process
            // run.
            None if ir.invariants.is_empty() => model_check::run(&ir, &tla_artifact, bound),
            None => model_check::run_executable(&ir, &tla_artifact, bound),
        };
        runs.push(FileRun {
            file,
            id: spec.intent.id.clone(),
            stem,
            result,
        });
    }
    runs
}

/// Command-path passes 2+3 core (design D9): resolve every citation
/// over the whole corpus from same-run evidence and apply the resolved
/// statuses to each successful report in place (replacing the per-file
/// local resolution), so persisted reports and command output carry
/// identical statuses. Returns the per-file resolved statuses with
/// reasons for the command output.
pub fn apply_corpus_resolution(specs: &[Spec], runs: &mut [FileRun]) -> Vec<Vec<ResolvedStatus>> {
    // Index alignment invariant: `run_backend_pass` pushes exactly one
    // `FileRun` per spec, in `specs` order, so `runs[idx]` always
    // corresponds to `specs[idx]`.

    let mut exec_maps: Vec<BTreeMap<String, ThreeValued>> = Vec::new();
    for (idx, run) in runs.iter().enumerate() {
        match &run.result {
            Ok(report) => {
                let ir = compile::extract_model_ir(&specs[idx]);
                exec_maps.push(
                    report
                        .invariant_statuses
                        .iter()
                        .filter(|s| !ir.guard_citations.contains_key(&s.id))
                        .map(|s| (s.id.clone(), s.status))
                        .collect(),
                );
            }
            Err(_) => exec_maps.push(BTreeMap::new()),
        }
    }
    let resolved_per_file = resolve_corpus_citations(specs, &exec_maps);
    for (idx, run) in runs.iter_mut().enumerate() {
        if let Ok(report) = &mut run.result {
            let ir = compile::extract_model_ir(&specs[idx]);
            report
                .invariant_statuses
                .retain(|s| !ir.guard_citations.contains_key(&s.id));
            for r in &resolved_per_file[idx] {
                report.invariant_statuses.push(r.invariant_status());
            }
        }
    }
    resolved_per_file
}

/// A scope-law violation: the labeled kind, the message naming the
/// offending inputs, and the remediation hint.
pub struct ScopeViolation {
    /// The stable label (`duplicate_corpus_identity`).
    pub label: &'static str,
    pub message: String,
    pub hint: String,
}

/// The command scope law (design D9, revised Revision 18 —
/// specodelic-mcy): intent ids must be unique across the invocation —
/// a collision fails `duplicate_corpus_identity` (repeated local row
/// ids across distinct intents are valid and stay file-local). The
/// former `id: spec` isolated-scope rule retired with `id: spec`
/// itself: dual-format files now carry real intent ids like everyone
/// else, and the identity-uniqueness law is the only scope law left.
pub fn check_corpus_scope(specs: &[Spec]) -> Result<(), ScopeViolation> {
    let mut seen: BTreeMap<&str, String> = BTreeMap::new();
    for spec in specs {
        let display = spec
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| format!("<{}>", spec.intent.id));
        if let Some(first) = seen.get(spec.intent.id.as_str()) {
            return Err(ScopeViolation {
                label: "duplicate_corpus_identity",
                message: format!(
                    "intent id `{}` is claimed by both {first} and {display} — ordinary corpus intent ids must be unique",
                    spec.intent.id
                ),
                hint: "rename one file (and its id:) so each ordinary intent id is unique; repeated local row ids across distinct intents are fine and stay file-local".to_string(),
            });
        }
        seen.insert(spec.intent.id.as_str(), display);
    }
    Ok(())
}

/// One resolved citation status: the Constraints-table id, the
/// three-valued status, and the reason when (and only when) the status
/// is `unknown` for a namable cause (cycle, missing target, property
/// row without same-run invariant evidence).
#[derive(Debug, Clone)]
pub struct ResolvedStatus {
    pub id: String,
    pub status: ThreeValued,
    pub reason: Option<String>,
}

impl ResolvedStatus {
    /// The report shape — status only, no reason (RunReport keeps its
    /// deployed schema; reasons surface in command output).
    pub fn invariant_status(&self) -> InvariantStatus {
        InvariantStatus {
            id: self.id.clone(),
            status: self.status,
        }
    }

    /// The command-output shape: id, status, and the unknown reason when
    /// present (design D9 — reasons surface in command output; the
    /// persisted report keeps its deployed status-only schema).
    pub fn output_json(&self) -> serde_json::Value {
        let mut entry = serde_json::json!({
            "id": self.id,
            "status": self.status,
        });
        if let Some(reason) = &self.reason {
            entry["reason"] = serde_json::json!(reason);
        }
        entry
    }
}

/// Resolve every citation invariant of every spec over the whole
/// invocation corpus (design D9). `exec_outcomes` carries each file's
/// same-run executable-invariant evidence keyed by local row id.
///
/// - Bare `[[row]]` resolves within the citing file.
/// - Qualified `[[file.row]]` is one exact key match against the corpus
///   (`<intent-id>`, `<row-id>`) — never a suffix match, never discovered
///   from the filesystem: only files in this invocation contribute
///   evidence.
/// - Citations of citations resolve in dependency order; a cycle leaves
///   every participant `unknown` with a cycle reason.
/// - Missing targets and property rows (no same-run invariant evidence)
///   are `unknown` with a reason — never a pass.
pub fn resolve_corpus_citations(
    specs: &[Spec],
    exec_outcomes: &[BTreeMap<String, ThreeValued>],
) -> Vec<Vec<ResolvedStatus>> {
    let mut resolver = Resolver::new(specs, exec_outcomes);
    (0..specs.len())
        .map(|i| {
            let rows: Vec<String> = resolver.citations[i].keys().cloned().collect();
            rows.iter()
                .map(|row| resolver.resolve_citation_row(i, row))
                .collect()
        })
        .collect()
}

/// One citation row pending resolution, keyed by (file index, row id) —
/// the node identity of the dependency graph.
type Node = (usize, String);

struct Resolver<'a> {
    specs: &'a [Spec],
    exec_outcomes: &'a [BTreeMap<String, ThreeValued>],
    /// Per file: the citation rows (id → verbatim cell).
    citations: Vec<BTreeMap<String, String>>,
    memo: BTreeMap<Node, ResolvedStatus>,
    visiting: BTreeSet<Node>,
}

impl<'a> Resolver<'a> {
    fn new(specs: &'a [Spec], exec_outcomes: &'a [BTreeMap<String, ThreeValued>]) -> Self {
        let citations: Vec<BTreeMap<String, String>> = specs
            .iter()
            .map(|spec| compile::extract_model_ir(spec).guard_citations)
            .collect();
        Resolver {
            specs,
            exec_outcomes,
            citations,
            memo: BTreeMap::new(),
            visiting: BTreeSet::new(),
        }
    }

    /// Resolve one citation row: parse its cell and evaluate the
    /// expression against corpus-aware evidence. The parse is proven at
    /// extraction time; a failed re-parse reports honest unknown (never a
    /// silent drop).
    fn resolve_citation_row(&mut self, file_idx: usize, row: &str) -> ResolvedStatus {
        let node = (file_idx, row.to_string());
        if let Some(resolved) = self.memo.get(&node) {
            return resolved.clone();
        }
        if !self.visiting.insert(node.clone()) {
            return ResolvedStatus {
                id: row.to_string(),
                status: ThreeValued::Unknown,
                reason: Some(format!("citation cycle through `{row}`")),
            };
        }
        let resolved = match self.citations[file_idx]
            .get(row)
            .and_then(|cell| compile::parse_citation_expr(cell))
        {
            Some(expr) => {
                let (status, reason) = self.evaluate(&expr, file_idx);
                ResolvedStatus {
                    id: row.to_string(),
                    status,
                    reason,
                }
            }
            None => ResolvedStatus {
                id: row.to_string(),
                status: ThreeValued::Unknown,
                reason: Some(
                    "citation cell no longer parses — honest unknown, never a silent drop"
                        .to_string(),
                ),
            },
        };
        self.visiting.remove(&node);
        self.memo.insert(node, resolved.clone());
        resolved
    }

    /// Kleene evaluation with reasons: ∧ is counterexample-dominant and
    /// absorbs unknown; ¬ flips verified/counterexample and keeps
    /// unknown. A reason rides along whenever the result stays unknown.
    fn evaluate(&mut self, expr: &CitationExpr, file_idx: usize) -> (ThreeValued, Option<String>) {
        match expr {
            CitationExpr::Cite(target) => self.resolve_target(target, file_idx),
            CitationExpr::Not(inner) => {
                let (status, reason) = self.evaluate(inner, file_idx);
                (
                    match status {
                        ThreeValued::Verified => ThreeValued::Counterexample,
                        ThreeValued::Counterexample => ThreeValued::Verified,
                        ThreeValued::Unknown => ThreeValued::Unknown,
                    },
                    reason.map(|r| format!("composed ¬ over unknown: {r}")),
                )
            }
            CitationExpr::And(a, b) => {
                let (sa, ra) = self.evaluate(a, file_idx);
                let (sb, rb) = self.evaluate(b, file_idx);
                match (sa, sb) {
                    (ThreeValued::Counterexample, _) | (_, ThreeValued::Counterexample) => {
                        (ThreeValued::Counterexample, None)
                    }
                    (ThreeValued::Verified, ThreeValued::Verified) => (ThreeValued::Verified, None),
                    _ => {
                        let reason = ra
                            .or(rb)
                            .map(|r| format!("composed ∧ over unknown: {r}"))
                            .unwrap_or_else(|| "composed ∧ over unknown evidence".to_string());
                        (ThreeValued::Unknown, Some(reason))
                    }
                }
            }
        }
    }

    /// Resolve one citation target. Bare names stay local to the citing
    /// file; qualified names are exact corpus keys. The reason names the
    /// cause: a missing target (no such invariant evidence in the run) or
    /// a property row (model-check never executes properties).
    fn resolve_target(&mut self, target: &str, file_idx: usize) -> (ThreeValued, Option<String>) {
        let owner = match target.rsplit_once('.') {
            // Qualified: exactly one corpus file owns the target — the
            // intent id must match in full, so a suffix of a longer file
            // id can never match and nothing outside the invocation is
            // ever discovered.
            Some((file, row)) => match self.specs.iter().position(|s| s.intent.id == file) {
                Some(idx) => (idx, row.to_string()),
                None => return (ThreeValued::Unknown, Some(self.missing_reason(target))),
            },
            // Bare: the citing file owns the target.
            None => (file_idx, target.to_string()),
        };
        let (idx, row) = owner;
        if let Some(status) = self.exec_outcomes[idx].get(&row) {
            return (*status, None);
        }
        if self.citations[idx].contains_key(&row) {
            let resolved = self.resolve_citation_row(idx, &row);
            let reason = resolved
                .reason
                .clone()
                .map(|r| format!("citation `{target}` is unknown: {r}"));
            return (resolved.status, reason);
        }
        if self.specs[idx].properties.iter().any(|p| p.id == row) {
            return (
                ThreeValued::Unknown,
                Some(format!(
                    "property row `{target}` has no same-run invariant evidence — model-check never executes properties"
                )),
            );
        }
        (ThreeValued::Unknown, Some(self.missing_reason(target)))
    }

    /// The unknown reason for a target with no same-run invariant
    /// evidence anywhere in this invocation.
    fn missing_reason(&self, target: &str) -> String {
        format!(
            "missing target `{target}`: no same-run evidence (only files in this invocation contribute)"
        )
    }
}
