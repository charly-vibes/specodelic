// Command handlers (split from main.rs — specodelic-g17 file_lines ratchet).
use crate::commands::{self, as_failure};
use crate::{MAX_INPUT_BYTES, collect_specs, emit_report, openspec_tree_present, parse_batch};

use genesis::guide::{Output, OutputFormat, Verbosity};
use specodelic::{compile, conform, graph, human, lint, merge, rename};

pub(crate) fn cmd_lint(
    paths: &[String],
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let (specs, checklists, notes, parse_errors) = parse_batch(paths, verbosity);
    if specs.is_empty() && checklists.is_empty() {
        // Never a silent ok:true on zero files — that's a false green
        // (beads specodelic-6pi). A declared checklist alone still gets
        // linted (checklist_well_formed needs no specs), so the failure
        // only fires when NEITHER was found.
        let (msg, hint) = if !parse_errors.is_empty() {
            (
                "spec files failed to parse",
                "fix the frontmatter/tables reported above",
            )
        } else {
            (
                "no spec files found — nothing was linted",
                if openspec_tree_present() {
                    "found an openspec/ tree — try: specodelic lint openspec"
                } else {
                    "pass files or directories containing *.md specs with YAML frontmatter (directories are searched recursively; hidden and build dirs are skipped)"
                },
            )
        };
        let mut out: Output<serde_json::Value> = Output::failure(msg).with_next_step(hint);
        // The labeled notes surface even on the nothing-linted path — a
        // hostile-input rejection (FIFO, device, oversized) or a parse
        // error must name itself, never vanish into a generic failure
        // (specodelic-suz).
        for n in notes.iter().chain(&parse_errors) {
            out = out.with_warning(n.clone());
        }
        emit_report(out, None, format, verbosity, stdout, stderr);
        // Invocation error (specodelic-7rr item 2): nothing was processed.
        return 2;
    }
    let report = lint::lint_all(&specs, &checklists);
    let payload = serde_json::to_value(&report).unwrap_or_default();
    let failures = report.failures();
    let mut out = Output::success(payload.clone());
    for n in notes.iter().chain(&parse_errors) {
        out = out.with_warning(n.clone());
    }
    // Advisory findings (openspec/specs/linter-observability/spec.md) ride the
    // warnings channel — exit 0, never a failure; the issues channel is
    // for invariant violations only.
    for w in &report.warnings {
        out = out.with_warning(format!("{} [{}] {}", w.file, w.rule_id, w.message));
    }
    if failures > 0 {
        out = out.with_next_step(
            "fix the reported invariants — each rule's semantics: spk explain lint-rules",
        );
    } else if !parse_errors.is_empty() {
        // specodelic-in9: parse errors are never a silent pass — even when
        // the rest of the batch linted clean, the stage fails so the
        // pre-commit gate cannot let a corrupted spec commit.
        out = out.with_next_step("fix the parse error reported above — the file was NOT linted");
    } else {
        out = out.with_next_step("run: specodelic graph");
    }
    emit_report(
        out,
        Some(human::lint(&report)),
        format,
        verbosity,
        stdout,
        stderr,
    );
    if failures > 0 || !parse_errors.is_empty() {
        1
    } else {
        0
    }
}

/// `spk parse <file>` — the parsed `Spec` IR as a json envelope
/// (specodelic-9rv). Syntax-only: succeeds on lint-dirty files and embeds
/// no lint status (design D3 — consumers chain `spk lint` themselves);
/// the hint names the lint command so the chain stays discoverable.
/// Unparseable or missing input is a labeled error envelope with a
/// remediation hint and non-zero exit (D4).
pub(crate) fn cmd_parse(
    file: &str,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    match specodelic::parse::parse_file(std::path::Path::new(file)) {
        Ok(spec) => {
            // The parsed Spec is emitted directly (D1): every structured
            // field, no filtering — consumers decide what they need.
            let payload = serde_json::to_value(&spec).unwrap_or_default();
            let out: Output<serde_json::Value> =
                Output::success(payload).with_next_step(format!("run: specodelic lint {file}"));
            emit_report(
                out,
                Some(human::parse(&spec)),
                format,
                verbosity,
                stdout,
                stderr,
            );
            0
        }
        Err(err) => {
            let out: Output<serde_json::Value> =
                Output::failure(err.to_string()).with_next_step(err.hint());
            emit_report(out, None, format, verbosity, stdout, stderr);
            1
        }
    }
}

pub(crate) fn cmd_graph(
    paths: &[String],
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let (specs, _checklists, notes, _parse_errors) = parse_batch(paths, verbosity);
    if specs.is_empty() {
        // Never a silent empty graph on zero files — a typoed path would
        // read as a fully-resolved corpus (specodelic-6pi precedent), and
        // the exit code must say invocation error, not success (7rr item 2).
        let hint = if openspec_tree_present() {
            "found an openspec/ tree — try: specodelic graph openspec"
        } else {
            "pass files or directories containing *.md specs with YAML frontmatter"
        };
        let mut out: Output<serde_json::Value> =
            Output::failure("no spec files found — nothing was graphed").with_next_step(hint);
        for n in &notes {
            out = out.with_warning(n.clone());
        }
        emit_report(out, None, format, verbosity, stdout, stderr);
        return 2;
    }
    let report = graph::build(&specs);
    let payload = serde_json::to_value(&report).unwrap_or_default();
    let mut out = Output::success(payload.clone());
    for n in &notes {
        out = out.with_warning(n.clone());
    }
    let clean = report.dangling.is_empty()
        && report.violations.is_empty()
        && report.supersedes_cycles.is_empty();
    out = if clean {
        out.with_next_step("every reference resolves — run: specodelic lint")
    } else if !report.violations.is_empty() || !report.supersedes_cycles.is_empty() {
        out.with_next_step(
            "resolve the typing violations / supersedes cycles (see openspec/specs/specodelic/spec.md's Reference Typing table and openspec/specs/linter-graph_shape/spec.md)",
        )
    } else {
        out.with_next_step(
            "resolve the dangling references (see openspec/specs/linter-referential_integrity/spec.md)",
        )
    };
    emit_report(
        out,
        Some(human::graph(&report)),
        format,
        verbosity,
        stdout,
        stderr,
    );
    if clean { 0 } else { 1 }
}

pub(crate) fn cmd_rename(
    old_id: &str,
    new_id: &str,
    paths: &[String],
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    // Same hostile-input gate as parse_batch (specodelic-suz): only
    // regular files under the cap are ever read.
    let mut files: Vec<(std::path::PathBuf, String)> = vec![];
    let mut notes: Vec<String> = vec![];
    for f in collect_specs(paths) {
        let meta = match std::fs::metadata(&f) {
            Ok(m) => m,
            Err(e) => {
                notes.push(format!("{}: unreadable ({e})", f.display()));
                continue;
            }
        };
        if !meta.is_file() {
            notes.push(format!(
                "{}: skipped (not a regular file — only regular files are ingested, never a FIFO, device, or other special file)",
                f.display()
            ));
            continue;
        }
        if meta.len() > MAX_INPUT_BYTES {
            notes.push(format!(
                "{}: skipped (exceeds the 2 MiB input cap — corpus files are ~10-50 KB; split or move the file)",
                f.display()
            ));
            continue;
        }
        match std::fs::read_to_string(&f) {
            Ok(t) => files.push((f, t)),
            Err(e) => notes.push(format!("{}: unreadable ({e})", f.display())),
        }
    }
    if files.is_empty() {
        let mut out: Output<serde_json::Value> = Output::failure("no spec files to rename")
            .with_next_step("pass spec files or a directory (defaults to ./specs)");
        for n in &notes {
            out = out.with_warning(n.clone());
        }
        emit_report(out, None, format, verbosity, stdout, stderr);
        return 2;
    }

    match rename::run(&files, old_id, new_id) {
        Ok(outcome) => {
            // Apply: write every new/changed file first, then remove the
            // moved file's old path last — between the two there is never
            // a moment where the definition is missing.
            let mut changed: Vec<String> = vec![];
            for (path, text) in &outcome.writes {
                if let Err(e) = std::fs::write(path, text) {
                    let out: Output<serde_json::Value> = Output::failure(format!(
                        "rename could not write {}: {e} — earlier files may already be rewritten; re-run after fixing the permission issue",
                        path.display()
                    ))
                    .with_next_step("make the target writable, then re-run the same rename");
                    emit_report(out, None, format, verbosity, stdout, stderr);
                    return 1;
                }
                changed.push(path.display().to_string());
            }
            if let Some(old) = &outcome.remove
                && let Err(e) = std::fs::remove_file(old)
            {
                let out: Output<serde_json::Value> = Output::failure(format!(
                        "rename could not remove {}: {e} — the new file exists; remove the old one manually",
                        old.display()
                    ))
                    .with_next_step("delete the stale old file, then re-run spk graph to confirm zero dangling");
                emit_report(out, None, format, verbosity, stdout, stderr);
                return 1;
            }
            let payload = serde_json::json!({
                "old_id": outcome.old_id,
                "new_id": outcome.new_id,
                "files_changed": changed,
            });
            let mut out = Output::success(payload).with_next_step(format!(
                "run: specodelic lint {} && specodelic graph {}",
                paths.first().map(String::as_str).unwrap_or("specs"),
                paths.first().map(String::as_str).unwrap_or("specs")
            ));
            for n in &notes {
                out = out.with_warning(n.clone());
            }
            emit_report(
                out,
                Some(human::rename(&outcome)),
                format,
                verbosity,
                stdout,
                stderr,
            );
            0
        }
        Err(e) => {
            let (msg, hint) = match &e {
                rename::RenameError::UnknownId(id) => (
                    format!("{id} matches no row or intent id in the corpus"),
                    "check the id with: specodelic graph (every defined id is a node)",
                ),
                rename::RenameError::Collision { new_id, holder } => (
                    format!("{new_id} already exists — defined in {holder}"),
                    "pick a new_id that is not in the corpus (rename.new_id_available)",
                ),
                rename::RenameError::InvalidNewId(id) => (
                    format!("{id:?} is not a usable id (empty, whitespace, or link/table syntax)"),
                    "ids are dotted identifiers like compile.compile_is_total",
                ),
                rename::RenameError::Ambiguous(id) => (
                    format!(
                        "{id} matches more than one row — the corpus already violates unique_across_repo"
                    ),
                    "fix the duplicate ids first (specodelic lint reports them)",
                ),
                rename::RenameError::VerifyFailed { .. } => (
                    "rename rejected at the verify gate — nothing was written".to_string(),
                    "fix the reported issues; the repo is byte-identical to before",
                ),
            };
            let mut out: Output<serde_json::Value> = Output::failure(msg).with_next_step(hint);
            if let rename::RenameError::VerifyFailed { details } = &e {
                for d in details {
                    out = out.with_warning(d.clone());
                }
            }
            emit_report(out, None, format, verbosity, stdout, stderr);
            1
        }
    }
}

/// Read one spec tree for the merge check, returning (relative path,
/// text) pairs plus hostile-input notes. Same gate as `cmd_rename`
/// (specodelic-suz): only regular files under the cap are ever read.
pub(crate) fn read_tree(dir: &str) -> (Vec<(String, String)>, Vec<String>) {
    let root = std::path::PathBuf::from(dir);
    let mut files: Vec<(String, String)> = vec![];
    let mut notes: Vec<String> = vec![];
    for f in collect_specs(std::slice::from_ref(&dir.to_string())) {
        let rel = match f.strip_prefix(&root) {
            Ok(r) => r.to_string_lossy().into_owned(),
            Err(_) => f.display().to_string(),
        };
        let meta = match std::fs::metadata(&f) {
            Ok(m) => m,
            Err(e) => {
                notes.push(format!("{}: unreadable ({e})", f.display()));
                continue;
            }
        };
        if !meta.is_file() {
            notes.push(format!(
                "{}: skipped (not a regular file — only regular files are ingested)",
                f.display()
            ));
            continue;
        }
        if meta.len() > MAX_INPUT_BYTES {
            notes.push(format!(
                "{}: skipped (exceeds the 2 MiB input cap)",
                f.display()
            ));
            continue;
        }
        match std::fs::read_to_string(&f) {
            Ok(t) => files.push((rel, t)),
            Err(e) => notes.push(format!("{}: unreadable ({e})", f.display())),
        }
    }
    (files, notes)
}

pub(crate) fn cmd_merge(
    branch: Option<&str>,
    base: Option<&str>,
    paths: &[String],
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let Some(branch_dir) = branch else {
        let out: Output<serde_json::Value> = Output::failure(
            "spk merge needs the incoming tree: --branch <dir> (and ideally --base <ancestor-tree>)",
        )
        .with_next_step(
            "pass the two branch tips' spec directories: spk merge --branch ../other/specs --base ../base/specs specs",
        );
        emit_report(out, None, format, verbosity, stdout, stderr);
        return 2;
    };
    let current_tree = paths
        .first()
        .cloned()
        .unwrap_or_else(|| "specs".to_string());
    let (a_files, notes_a) = read_tree(&current_tree);
    let (b_files, notes_b) = read_tree(branch_dir);
    let (base_files, notes_base) = match base {
        Some(b) => read_tree(b),
        None => (vec![], vec![]),
    };
    // Without an ancestor, every id defined on both tips reads as
    // "minted independently" (no_new_id_collision has nothing to compare
    // against) — say so explicitly instead of letting the collision flood
    // read as a verdict about the branches.
    let mut notes_no_base: Vec<String> = vec![];
    if base.is_none() {
        notes_no_base.push(
            "no --base given: ids defined on both branches are treated as independently \
             minted (collision) — pass --base <ancestor-tree> to compare against the \
             common ancestor"
                .to_string(),
        );
    }
    if b_files.is_empty() {
        let mut out: Output<serde_json::Value> = Output::failure(
            "no spec files found in the incoming tree (--branch) — nothing to merge",
        )
        .with_next_step("pass the incoming branch's spec directory: spk merge --branch <dir>");
        for n in notes_b.iter().chain(&notes_a) {
            out = out.with_warning(n.clone());
        }
        emit_report(out, None, format, verbosity, stdout, stderr);
        return 2;
    }

    let report = merge::run(&base_files, &a_files, &b_files);
    let payload = serde_json::to_value(&report).unwrap_or_default();
    let mut out = Output::success(payload.clone());
    let next = match report.verdict.as_str() {
        "merged" => "run: specodelic graph to confirm zero dangling after joining".to_string(),
        "needs_review" => {
            "review the flagged blast radii / rename replays, resolve, then re-run spk merge"
                .to_string()
        }
        _ => "resolve the findings in the branch trees, then re-run spk merge".to_string(),
    };
    out = out.with_next_step(next);
    for n in notes_a
        .iter()
        .chain(&notes_base)
        .chain(&notes_no_base)
        .chain(&notes_b)
    {
        out = out.with_warning(n.clone());
    }
    emit_report(
        out,
        Some(human::merge(&report)),
        format,
        verbosity,
        stdout,
        stderr,
    );
    if report.verdict == "merged" { 0 } else { 1 }
}

pub(crate) fn cmd_compile(
    paths: &[String],
    out_dir: &str,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let (specs, _checklists, notes, _parse_errors) = parse_batch(paths, verbosity);
    if specs.is_empty() {
        let mut out: Output<serde_json::Value> =
            Output::failure("no spec files to compile").with_next_step(
                "pass spec files or a directory (defaults to ./specs); parse errors, if any, are reported as warnings",
            );
        // Hostile-input/parse notes name themselves even here (suz Ro5, CORR-001).
        for n in &notes {
            out = out.with_warning(n.clone());
        }
        emit_report(out, None, format, verbosity, stdout, stderr);
        // Invocation error: nothing to compile (specodelic-7rr item 2).
        return 2;
    }

    // precondition_satisfied — the gate reuses the existing lint pass; a
    // file compiles only when lint reports zero issues for it.
    let report = lint::lint_corpus(&specs);

    let mut compiled: Vec<serde_json::Value> = vec![];
    let mut failed: Vec<serde_json::Value> = vec![];
    let mut warnings: Vec<String> = notes;
    let mut seen_stems: std::collections::BTreeMap<String, String> = Default::default();
    for spec in &specs {
        let file = spec
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| format!("<{}>", spec.intent.id));
        if let Err(e) = compile::precondition_satisfied(spec, &report) {
            failed.push(serde_json::json!({
                "file": file,
                "id": spec.intent.id,
                "stage": e.stage,
                "message": e.message,
            }));
            continue;
        }
        match compile::compile_spec(spec) {
            Ok(c) => {
                let written = compile::write_artifacts(spec, &c, out_dir);
                match written {
                    Ok(result) => {
                        // The rustfmt fallback fact rides the existing
                        // warning channel (write_artifacts stays soft).
                        warnings.extend(result.warnings);
                        // Two files with the same stem (different dirs)
                        // would silently overwrite each other's artifacts.
                        let stem = compile::artifact_stem(spec);
                        if let Some(first) = seen_stems.get(&stem) {
                            warnings.push(format!(
                                "stem collision: `{stem}` artifacts from {first} and {file} share one out-dir — the later file wins",
                            ));
                        }
                        seen_stems.insert(stem, file.clone());
                        // Envelope/disk note (gh-15): the payload carries
                        // the semantic artifact string while the on-disk
                        // _props.rs is the rustfmt-clean version — same
                        // semantics, formatting-only byte difference.
                        compiled.push(serde_json::json!({
                            "file": file,
                            "id": spec.intent.id,
                            "status": "compiled",
                            "artifacts": {
                                "toml": c.toml,
                                "props": c.props,
                                "tla": c.tla,
                            },
                            "model_ir": c.model_ir,
                            "written": result.paths,
                        }));
                    }
                    Err(e) => {
                        // compile_is_total: a write failure is a labeled
                        // failure, never a silent partial on disk.
                        failed.push(serde_json::json!({
                            "file": file,
                            "id": spec.intent.id,
                            "stage": "write_artifacts",
                            "message": e,
                        }));
                    }
                }
            }
            Err(e) => {
                failed.push(serde_json::json!({
                    "file": file,
                    "id": spec.intent.id,
                    "stage": e.stage,
                    "message": e.message,
                }));
            }
        }
    }

    let payload = serde_json::json!({
        "files_compiled": compiled.len(),
        "files_failed": failed.len(),
        "compiled": compiled,
        "failed": failed,
    });
    // specodelic-4v1 (F1): a labeled compile failure never rides a
    // success-shaped envelope (openspec/specs/errors/spec.md envelope_error_kind +
    // exit_code_mapping) — exit 1 was already correct, the ok bit lied.
    let failed_run = !failed.is_empty();
    let mut out = Output::success(payload.clone());
    for w in &warnings {
        out = out.with_warning(w.clone());
    }
    if failed.is_empty() {
        out = out.with_next_step("run: specodelic verify (consumes the *_props.rs artifacts)");
    } else {
        out = out.with_next_step(
            "fix the labeled stage failures (lint findings first — compile requires a linted-and-covered file)",
        );
    }
    let out = if failed_run {
        as_failure(
            out,
            "compile failed: labeled per-file failures (see .data.failed)",
        )
    } else {
        out
    };
    emit_report(
        out,
        Some(human::compile(&payload)),
        format,
        verbosity,
        stdout,
        stderr,
    );
    if failed.is_empty() { 0 } else { 1 }
}

// ---------------------------------------------------------------------------
// conform (moved from main.rs — specodelic-1v30: the handler grows the
// gate's corpus discovery and the --structural-only mode; the CLI-side
// corpus context lives beside the other corpus-walking handlers)
// ---------------------------------------------------------------------------

/// The conform invocation parameters beyond the spec path — the oracle
/// corpus path, the recorded closed-world declaration, the artifact
/// directory, and the opt-in structural-only mode (gh-17 friction 2),
/// grouped to keep the shared emit plumbing (cli/format/verbosity/
/// streams) within the arg lint (the CheckTarget precedent).
pub(crate) struct ConformTarget {
    pub(crate) oracle: String,
    pub(crate) closed_world: bool,
    pub(crate) structural_only: bool,
    pub(crate) out_dir: String,
}

/// Discover the openspec corpus root the spec file belongs to (gh-17
/// friction 1): walk UP from the file's own location and stop at the
/// first ancestor directory carrying an `openspec` tree — the corpus
/// context `spk lint <openspec-root>` sees. Deterministic per file
/// location, no env dependence; `None` is the labeled isolated-file
/// fallback (the gate lints the single-spec slice, never an error).
fn discover_openspec_root(spec_path: &str) -> Option<std::path::PathBuf> {
    let file = std::path::Path::new(spec_path);
    let start = std::fs::canonicalize(file).unwrap_or_else(|_| {
        std::env::current_dir()
            .map(|cwd| cwd.join(file))
            .unwrap_or_else(|_| file.to_path_buf())
    });
    let mut dir = start.parent()?.to_path_buf();
    loop {
        let candidate = dir.join("openspec");
        if candidate.is_dir() {
            return Some(candidate);
        }
        dir = dir.parent()?.to_path_buf();
    }
}

/// `spk conform <spec-path> --oracle <scenarios.jsonl>` (add-conform
/// phase 4): the READ-ONLY external-oracle evaluation over
/// `conform::run_with` — the artifact gate (design D5) linted in the
/// DISCOVERED corpus context (gh-17 friction 1), mechanical
/// classification (D3, with the opt-in structural-only mode, gh-17
/// friction 2), and the persisted report. Exit codes follow the
/// delta's exit_code_verdict_mapping over openspec/specs/errors/spec.md
/// `exit_code_mapping`: 0 when no verdict is forbidden or unsupported
/// (unknown/underspecified/agree never fail — their counts surface in
/// both views), 1 when any forbidden/unsupported verdict fails the run
/// (an error-kind envelope — the published ok:false contract,
/// envelope_error_kind), 2 on invocation errors and gate refusals —
/// with ZERO verdict records emitted from any refusal path.
pub(crate) fn cmd_conform(
    spec_path: &str,
    target: ConformTarget,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let ConformTarget {
        oracle,
        closed_world,
        structural_only,
        out_dir,
    } = target;
    // Invocation errors precede everything: zero verdict records.
    let corpus_bytes = match std::fs::read(&oracle) {
        Ok(bytes) => bytes,
        Err(e) => {
            let out: Output<serde_json::Value> =
                Output::failure(format!("cannot read the scenario corpus {oracle}: {e}")).with_next_step(
                    "pass an existing JSONL scenario corpus file — one {id, setup?, trace} object per line (--oracle)",
                );
            emit_report(out, None, format, verbosity, stdout, stderr);
            return 2;
        }
    };
    let paths = vec![spec_path.to_string()];
    let (specs, _checklists, notes, parse_errors) = parse_batch(&paths, verbosity);
    if !parse_errors.is_empty() {
        // An unparsed spec never reaches the gate (the same discipline
        // orchestrate applies before lint) — labeled, exit 2, no records.
        let mut out: Output<serde_json::Value> = Output::failure(
            "the spec file failed to parse — verdicts are never derived from an unparsed file",
        );
        for e in &parse_errors {
            out = out.with_warning(e.clone());
        }
        out = out.with_next_step("fix the frontmatter/tables named above");
        emit_report(out, None, format, verbosity, stdout, stderr);
        return 2;
    }
    if specs.is_empty() {
        let mut out: Output<serde_json::Value> =
            Output::failure("no spec file to conform — nothing was evaluated");
        // Hostile-input/parse notes name themselves even here (suz).
        for n in &notes {
            out = out.with_warning(n.clone());
        }
        out = out.with_next_step("pass exactly one spec file (*.md with YAML frontmatter)");
        emit_report(out, None, format, verbosity, stdout, stderr);
        return 2;
    }
    if specs.len() > 1 {
        let out: Output<serde_json::Value> = Output::failure(format!(
            "conform evaluates exactly one spec file — the given path named {}",
            specs.len()
        ))
        .with_next_step("pass a single spec file path");
        emit_report(out, None, format, verbosity, stdout, stderr);
        return 2;
    }
    // The gate's corpus context (gh-17 friction 1): discover the corpus
    // root the file belongs to and parse it with the SAME batch
    // machinery the lint command uses — never a second walker. The gate
    // verdict keys on THIS file's findings within that corpus lint.
    let (corpus_ctx, gate_warnings): (Option<conform::CorpusContext>, Vec<String>) =
        match discover_openspec_root(spec_path) {
            Some(root) => {
                let root_str = root.to_string_lossy().to_string();
                let (corpus_specs, _cl, _notes, corpus_parse_errors) =
                    parse_batch(std::slice::from_ref(&root_str), verbosity);
                let mut warnings = Vec::new();
                if !corpus_parse_errors.is_empty() {
                    // Labeled partial-corpus fact (invariant 3): the
                    // corpus lint resolves refs against a partial corpus,
                    // so referential findings may be false — surfaced on
                    // the existing warnings channel, never silent.
                    warnings.push(format!(
                        "conform's input gate linted a PARTIAL corpus: {} sibling file(s) failed to parse — referential findings may be false ({})",
                        corpus_parse_errors.len(),
                        corpus_parse_errors.join("; ")
                    ));
                }
                (
                    Some(conform::CorpusContext {
                        specs: corpus_specs,
                        parse_error_notes: corpus_parse_errors,
                    }),
                    warnings,
                )
            }
            None => {
                // The labeled isolated-file fallback (gh-17): the gate
                // lints the single-spec slice — cross-file refs may
                // dangle; labeled, never an error.
                (
                    None,
                    vec![
                        "no corpus root discovered — the input gate linted the file in ISOLATION; cross-file references may dangle (pass the file inside its openspec tree, or run: spk lint <corpus-root>)"
                            .to_string(),
                    ],
                )
            }
        };
    match conform::run_with(
        &specs[0],
        std::path::Path::new(&out_dir),
        &corpus_bytes,
        closed_world,
        structural_only,
        corpus_ctx.as_ref(),
    ) {
        Ok(report) => {
            // The exit mapping keys off the taxonomy's own spellings —
            // never a string literal that could drift from
            // conform::Verdict's serde form.
            let violation_count =
                |v: conform::Verdict| report.verdict_counts.get(v.as_str()).copied().unwrap_or(0);
            let forbidden = violation_count(conform::Verdict::Forbidden);
            let unsupported = violation_count(conform::Verdict::Unsupported);
            if forbidden + unsupported > 0 {
                // Findings-or-failure (exit 1): the error-kind envelope
                // is the published contract (openspec/specs/errors/spec.md
                // envelope_error_kind) — a failed conformance run never
                // rides a success-shaped envelope. The human view is the
                // same one the success branch renders.
                let mut out = commands::as_failure(
                    Output::success(report.clone()),
                    format!(
                        "{forbidden} forbidden and {unsupported} unsupported verdict(s) — the oracle disagrees with the declared model"
                    ),
                );
                for w in &gate_warnings {
                    out = out.with_warning(w.clone());
                }
                // RO5U M2 (eczv.4 tidy): the failure envelope renders
                // through the SAME path as the success envelope —
                // conform's single report renderer — never the generic
                // main.rs emit_report. One path; the bytes are unchanged.
                conform::emit_output(out, format, verbosity, stdout, stderr).ok();
                1
            } else {
                conform::emit_report_with_warnings(
                    &report,
                    &gate_warnings,
                    format,
                    verbosity,
                    stdout,
                    stderr,
                )
                .ok();
                0
            }
        }
        Err(e) => {
            // Gate refusals and corpus refusals stay exit 2 (delta
            // exit_code_verdict_mapping) with zero verdict records; the
            // message already carries the failing stage, the hint names
            // its remediation command.
            let hint = if e.message.contains("spk lint") {
                "run: spk lint <file>"
            } else if e.message.contains("spk compile") {
                "run: spk compile <file>"
            } else if e.stage == "corpus_parse" {
                "fix the scenario record — one JSON object per line: {id, setup?, trace}"
            } else {
                "address the labeled refusal above"
            };
            let mut out: Output<serde_json::Value> =
                Output::failure(format!("{}: {}", e.stage, e.message)).with_next_step(hint);
            for w in &gate_warnings {
                out = out.with_warning(w.clone());
            }
            emit_report(out, None, format, verbosity, stdout, stderr);
            2
        }
    }
}
