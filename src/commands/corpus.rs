// Command handlers (split from main.rs — specodelic-g17 file_lines ratchet).
use crate::{MAX_INPUT_BYTES, collect_specs, emit_report, openspec_tree_present, parse_batch};

use genesis::guide::{Output, OutputFormat, Verbosity};
use specodelic::{compile, graph, human, lint, merge, rename};

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
    // Advisory findings (specs/linter-observability.md) ride the
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
            "resolve the typing violations / supersedes cycles (see specs/specodelic.md's Reference Typing table and specs/linter-graph_shape.md)",
        )
    } else {
        out.with_next_step(
            "resolve the dangling references (see specs/linter-referential_integrity.md)",
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
                    Ok(files) => {
                        // Two files with the same stem (different dirs)
                        // would silently overwrite each other's artifacts.
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
                            "status": "compiled",
                            "artifacts": {
                                "toml": c.toml,
                                "props": c.props,
                                "tla": c.tla,
                            },
                            "model_ir": c.model_ir,
                            "written": files,
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
