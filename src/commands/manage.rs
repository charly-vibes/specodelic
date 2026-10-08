// Command handlers (split from main.rs — specodelic-g17 file_lines ratchet).

use crate::{VERSION, emit_report, openspec_tree_present, parse_batch};

use genesis::guide::{Output, OutputFormat, Verbosity};
use specodelic::guide;
use specodelic::{blocks, graph, human, migrate, refactor};

pub(crate) fn cmd_refactor(
    paths: &[String],
    high_fan_in: Option<usize>,
    changeset: Option<&str>,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let (specs, _checklists, notes, _parse_errors) = parse_batch(paths, verbosity);
    if specs.is_empty() {
        // Same never-silent-empty guard as graph (specodelic-6pi): a
        // typoed path must read as invocation error, not a clean corpus.
        let hint = if openspec_tree_present() {
            "found an openspec/ tree — try: specodelic refactor openspec"
        } else {
            "pass files or directories containing *.md specs with YAML frontmatter"
        };
        let mut out: Output<serde_json::Value> =
            Output::failure("no spec files found — nothing was analyzed").with_next_step(hint);
        for n in &notes {
            out = out.with_warning(n.clone());
        }
        emit_report(out, None, format, verbosity, stdout, stderr);
        return 2;
    }
    // fan_in_read_from_graph: every count comes from the derived graph,
    // never an independent markdown walk.
    let graph = graph::build(&specs);
    let threshold = high_fan_in.unwrap_or(refactor::DEFAULT_HIGH_FAN_IN);
    let changeset: std::collections::BTreeSet<String> = changeset
        .map(|s| {
            s.split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let report = refactor::analyze(&graph, &specs, threshold, &changeset);
    let payload = serde_json::to_value(&report).unwrap_or_default();
    let mut out = Output::success(payload.clone());
    for n in &notes {
        out = out.with_warning(n.clone());
    }
    // The advisor never gates (exit 0 either way): the finding is a
    // suggestion to split BEFORE the behavioral edit, nothing more.
    out = if report.findings.is_empty() {
        out.with_next_step("no split candidates — run: specodelic lint")
    } else {
        out.with_next_step(
            "split the flagged node first — the split itself is an ordinary edit, checked by the six existing linters (see specs/refactor.md)",
        )
    };
    emit_report(
        out,
        Some(human::refactor(&report)),
        format,
        verbosity,
        stdout,
        stderr,
    );
    0
}

/// Exit-code contract (specodelic-7rr item 2, CLARITY-pinned):
/// 0 = success (lint with zero findings counts); 1 = the stage produced
/// findings / a tool-level failure; 2 = invocation error — nothing was
/// processed (path not found, no spec files matched, unreadable input).
/// clap's own argument-parse failures also exit 2, so the mapping is
/// uniform. Documented here, in README.md, and in specs/USAGE.md.
pub(crate) fn exit_code_footer() -> String {
    format!(
        "{}\n\nExit codes:\n  0 = success\n  1 = findings or tool-level failure\n  2 = invocation error (path not found, no spec files matched)",
        genesis::guide::Verbosity::help_footer()
    )
}

/// `spk explain [TOPIC]` — serve the embedded format guide through the
/// envelope. Bare call lists topics; a known topic returns
/// `{topic, format_revision, body}`; an unknown topic fails with a hint
/// listing the valid topics (design Decision 3). Works offline: the body
/// is embedded via `include_str!`, no repo access needed.
pub(crate) fn cmd_explain(
    topic: Option<&str>,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let Some(topic) = topic else {
        let payload = serde_json::json!({
            "topics": guide::TOPICS
                .iter()
                .map(|(id, title)| serde_json::json!({ "id": id, "title": title }))
                .collect::<Vec<_>>(),
        });
        let out: Output<serde_json::Value> = Output::success(payload.clone())
            .with_next_step("run: specodelic explain <topic> — try: specodelic explain format");
        emit_report(
            out,
            Some(human::explain_topics(&payload)),
            format,
            verbosity,
            stdout,
            stderr,
        );
        return 0;
    };
    match guide::topic_body(topic) {
        Some(body) => {
            let payload = serde_json::json!({
                "topic": topic,
                "format_revision": guide::FORMAT_REVISION,
                "body": body,
            });
            let out = Output::success(payload)
                .with_next_step("apply the format, then check with: specodelic lint");
            // Decision 3: human mode prints the body itself, not the Debug
            // form of the payload — raise the output's verbosity threshold
            // so emit skips the data but still writes the next-step footer.
            if format == OutputFormat::Human {
                writeln!(stdout, "{body}").ok();
                out.with_verbosity(Verbosity::MAX + 1)
                    .emit(VERSION, format, verbosity, stdout, stderr)
                    .ok();
            } else {
                out.emit(VERSION, format, verbosity, stdout, stderr).ok();
            }
            0
        }
        None => {
            let valid = guide::TOPICS
                .iter()
                .map(|(id, _)| *id)
                .collect::<Vec<_>>()
                .join(" | ");
            let out: Output<serde_json::Value> =
                Output::failure(format!("unknown topic `{topic}` — valid topics: {valid}"))
                    .with_next_step("run: specodelic explain (no argument) to list the topics");
            // Unknown topic = invalid invocation (clap argument errors
            // also exit 2) — specodelic-7rr residual.
            emit_report(out, None, format, verbosity, stdout, stderr);
            2
        }
    }
}

/// `spk init` — write/refresh the SPECODELIC managed block in AGENTS.md
/// (specodelic-ze4). Idempotent: injects when missing, updates in place
/// when present; surrounding content is never touched. `--force` is
/// accepted for explicit refresh intent (the injector always rewrites the
/// block body — force exists so scripts record the intent).
pub(crate) fn cmd_init(
    _force: bool,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    // Discovery manifest (specodelic-las): declare specodelic's presence
    // in .genesis/tools.toml so orchestrators (wai) discover it without
    // hardcoding. Best-effort: a failure (unwritable dir) becomes a
    // warnings-channel note — the AGENTS.md block is the primary payload.
    let mut discovery_warnings: Vec<String> = Vec::new();
    if let Err(e) = genesis::discovery::register(
        std::path::Path::new("."),
        "specodelic",
        "Specodelic spec format linter + pipeline (spk)",
        "file",
        blocks::BLOCK_FILE,
    ) {
        discovery_warnings.push(format!(
            "could not register in .genesis/tools.toml ({e}) — tool discovery for this repo stays manual"
        ));
    }
    let path = std::path::Path::new(blocks::BLOCK_FILE);
    match blocks::inject_into(path) {
        Ok(result) => {
            let action = match result {
                genesis::managed_block::InjectResult::Created => "created",
                genesis::managed_block::InjectResult::Prepended => "injected",
                genesis::managed_block::InjectResult::Updated => "updated",
            };
            let payload = serde_json::json!({
                "file": blocks::BLOCK_FILE,
                "block": action,
                "format_revision": guide::FORMAT_REVISION,
            });
            let mut out: Output<serde_json::Value> = Output::success(payload.clone())
                .with_next_step(
                    "agents in this repo now see the spec rules; check specs with: spk lint",
                );
            for w in discovery_warnings {
                out = out.with_warning(w);
            }
            emit_report(
                out,
                Some(human::init(&payload)),
                format,
                verbosity,
                stdout,
                stderr,
            );
            0
        }
        Err(e) => {
            let out: Output<serde_json::Value> = Output::failure(format!(
                "could not write {}: {e}",
                blocks::BLOCK_FILE
            ))
            .with_next_step("check directory permissions, or pass an explicit path once AGENTS.md support lands elsewhere");
            emit_report(out, None, format, verbosity, stdout, stderr);
            1
        }
    }
}

/// `spk feedback` — file an issue against charly-vibes/specodelic via the
/// genesis unified feedback handler (espectacular pattern). Kind is
/// validated with typo suggestions; --dry-run previews without gh;
/// --from-last-error auto-populates from the scratch error record.
/// Report-only verb: always human-readable on stderr, never envelope JSON
/// (the envelope is for pipeline data; feedback is an interactive side
/// channel).
/// `spk hooks install|uninstall` — wire/unwire the specodelic
/// dual-format gate into the repo's hook chain (specodelic-cxr).
/// Wiring is purely additive marker-guarded lefthook surgery in
/// `specodelic::hooks`; this layer adds the repo-root resolution, the
/// fail-early `openspec/` pre-check (design Decision 2), the install
/// time gate dry-run (Rule-of-5 EDGE-001 — a failing gate is a warning
/// with an escape hint, never a commit trap), and the envelope.
pub(crate) fn cmd_hooks(
    install_action: bool,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let root = match genesis::git_hooks::repo_root() {
        Ok(root) => root,
        Err(err) => {
            let out: Output<serde_json::Value> = Output::failure(err.to_string())
                .with_next_step("run from inside the repository that owns the hook chain");
            emit_report(out, None, format, verbosity, stdout, stderr);
            return 1;
        }
    };

    if install_action {
        // Fail-early pre-check: a wired gate over a missing openspec
        // tree would fail every commit (design Decision 2).
        if !root.join("openspec").is_dir() {
            let out: Output<serde_json::Value> = Output::failure(format!(
                "no openspec/ tree at {} — the wired gate `spk lint openspec` would fail on every commit",
                root.display()
            ))
            .with_next_step(
                "adopt the openspec layout first (openspec init), then re-run: spk hooks install",
            );
            emit_report(out, None, format, verbosity, stdout, stderr);
            return 1;
        }
        let (outcome, config) = match specodelic::hooks::install(&root) {
            Ok(outcome) => {
                let config = ["lefthook.yml", "lefthook.yaml"]
                    .iter()
                    .find(|name| root.join(name).is_file())
                    .map(|n| n.to_string())
                    .unwrap_or_default();
                (outcome, config)
            }
            Err(err) => {
                let out: Output<serde_json::Value> = Output::failure(err.to_string())
                    .with_next_step(
                        "inspect the hook config manually, then re-run: spk hooks install",
                    );
                emit_report(out, None, format, verbosity, stdout, stderr);
                return 1;
            }
        };

        // Gate dry-run (design Decision 2 / EDGE-001): run the wired
        // command once so a repo whose tree would fail immediately gets
        // a warning + escape hint instead of a commit trap.
        let dry_run = gate_dry_run(&root);
        let payload = serde_json::json!({
            "command": specodelic::hooks::GATE_COMMAND,
            "stage": specodelic::hooks::STAGE,
            "config": config,
            "outcome": match outcome {
                specodelic::hooks::WireOutcome::Injected => "wired",
                specodelic::hooks::WireOutcome::AlreadyWired => "already_wired",
            },
            "gate_dry_run": serde_json::json!({
                "passed": dry_run.passed,
                "summary": dry_run.summary,
            }),
        });
        let mut out: Output<serde_json::Value> = Output::success(payload.clone()).with_next_step(
            "the gate runs on every commit — adjust anytime with: spk hooks uninstall",
        );
        if !dry_run.passed {
            out = out.with_warning(format!(
                "the gate dry-run failed: {} — the wired gate will fail on commits until the tree is fixed; escape hatch: spk hooks uninstall",
                dry_run.summary
            ));
        }
        emit_report(
            out,
            Some(human::hooks(&payload)),
            format,
            verbosity,
            stdout,
            stderr,
        );
        0
    } else {
        match specodelic::hooks::uninstall(&root) {
            Ok(outcome) => {
                let payload = serde_json::json!({
                    "outcome": match outcome {
                        specodelic::hooks::UnwireOutcome::Removed => "unwired",
                        specodelic::hooks::UnwireOutcome::NotWired => "not_wired",
                    },
                });
                emit_report(
                    Output::success(payload.clone()),
                    Some(human::hooks(&payload)),
                    format,
                    verbosity,
                    stdout,
                    stderr,
                );
                0
            }
            Err(err) => {
                let out: Output<serde_json::Value> = Output::failure(err.to_string())
                    .with_next_step(
                        "fix the managed-block markers manually, then re-run: spk hooks uninstall",
                    );
                emit_report(out, None, format, verbosity, stdout, stderr);
                1
            }
        }
    }
}

/// `spk archive-companion <CHANGE_ID>` — archive a change with the
/// dual-format layer preserved (specodelic-fzo / GH#7).
pub(crate) fn cmd_archive_companion(
    change_id: &str,
    dry_run: bool,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let root = match genesis::git_hooks::repo_root() {
        Ok(root) => root,
        Err(err) => {
            let out: Output<serde_json::Value> = Output::failure(err.to_string())
                .with_next_step("run from inside the repository that owns the openspec tree");
            emit_report(out, None, format, verbosity, stdout, stderr);
            return 1;
        }
    };

    let mut runner = |root: &std::path::Path, id: &str| {
        specodelic::archive_companion::openspec_archive_runner(root, id)
    };
    match specodelic::archive_companion::run(&root, change_id, dry_run, &mut runner) {
        Ok(outcome) => {
            let rel = |p: &std::path::Path| -> String {
                p.strip_prefix(&root).unwrap_or(p).display().to_string()
            };
            let payload = serde_json::json!({
                "change_id": outcome.change_id,
                "openspec_ran": outcome.openspec_ran,
                "archive_dir": rel(&outcome.archive_dir),
                "restored": outcome.restored.iter().map(|p| rel(p)).collect::<Vec<_>>(),
                "dry_run": outcome.dry_run,
            });
            let next_step = if outcome.dry_run {
                "re-run without --dry-run to archive and restore: spk archive-companion".to_string()
            } else if outcome.restored.is_empty() {
                "the change carried no spec deltas — nothing to restore".to_string()
            } else {
                "verify the gates: just lint-specs && openspec validate --all --strict".to_string()
            };
            let out: Output<serde_json::Value> =
                Output::success(payload.clone()).with_next_step(next_step);
            emit_report(
                out,
                Some(human::archive_companion(&payload)),
                format,
                verbosity,
                stdout,
                stderr,
            );
            0
        }
        Err(err) => {
            let out: Output<serde_json::Value> = Output::failure(err.to_string())
                .with_next_step("resolve the reported cause, then re-run: spk archive-companion");
            emit_report(out, None, format, verbosity, stdout, stderr);
            1
        }
    }
}

/// Outcome of the install-time gate dry-run.
pub(crate) struct GateDryRun {
    passed: bool,
    summary: String,
}

/// Summarize the gate dry-run output for the envelope. The gate runs in
/// its default (JSON envelope) mode on a pipe — a single-line envelope;
/// recognize it and summarize semantically (issue count / lint findings).
/// Any other output: the last meaningful line, truncated.
pub(crate) fn summarize_gate_output(raw: &str, passed: bool) -> String {
    for line in raw.lines().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line)
            && let Some(issues) = v
                .get("data")
                .and_then(|data| data.get("issues"))
                .and_then(|i| i.as_array())
        {
            return if issues.is_empty() {
                "clean (0 lint issues)".to_string()
            } else {
                let rules: Vec<String> = issues
                    .iter()
                    .filter_map(|i| i.get("rule_id").and_then(|r| r.as_str()).map(String::from))
                    .take(3)
                    .collect();
                format!(
                    "{} lint issue(s){}",
                    issues.len(),
                    if rules.is_empty() {
                        String::new()
                    } else {
                        format!(": {}", rules.join(", "))
                    }
                )
            };
        }
        let truncated = if line.len() > 200 {
            &line[line.len() - 200..]
        } else {
            line
        };
        return truncated.to_string();
    }
    if passed {
        "clean".to_string()
    } else {
        "failed".to_string()
    }
}

/// Run the wired gate command (`spk lint openspec`) once, from the repo
/// root. A missing `spk` binary on PATH is itself a dry-run failure —
/// the wired gate would fail at commit time (honest, not silent).
pub(crate) fn gate_dry_run(root: &std::path::Path) -> GateDryRun {
    match std::process::Command::new("spk")
        .args(["lint", "openspec"])
        .current_dir(root)
        .output()
    {
        Ok(output) => {
            let passed = output.status.success();
            let mut tail = String::from_utf8_lossy(&output.stderr).to_string();
            if tail.trim().is_empty() {
                tail = String::from_utf8_lossy(&output.stdout).to_string();
            }
            GateDryRun {
                passed,
                summary: summarize_gate_output(&tail, passed),
            }
        }
        Err(_) => GateDryRun {
            passed: false,
            summary: "spk not found on PATH — the wired gate would fail at commit time (install spk or adjust the gate)"
                .to_string(),
        },
    }
}

pub(crate) fn cmd_feedback(
    kind: &str,
    dry_run: bool,
    from_last_error: bool,
    title: Option<&str>,
) -> i32 {
    let args = genesis::feedback::FeedbackArgs {
        kind: kind.to_string(),
        dry_run,
        from_last_error,
        title: title.map(str::to_string),
    };
    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    match genesis::feedback::handle_feedback(
        &args,
        "spk",
        VERSION,
        "charly-vibes/specodelic",
        &project_root,
    ) {
        Ok(result) => {
            match result {
                genesis::feedback::gh::GhResult::Created { url, number } => {
                    eprintln!("filed issue #{number}: {url}");
                }
                genesis::feedback::gh::GhResult::FallbackUrl(url) => {
                    eprintln!("open: {url}");
                }
                genesis::feedback::gh::GhResult::LocalFile(path) => {
                    eprintln!(
                        "gh unavailable — issue body saved to {}; file it manually",
                        path.display()
                    );
                }
            }
            0
        }
        Err(msg) => {
            eprintln!("{msg}");
            2
        }
    }
}

/// Write `<stem>.toml`, `<stem>_props.rs`, and `<stem>.tla` into `out_dir`. Byte-stable
/// output: the same input always produces the same bytes, so committed
/// artifacts make reruns diff-visible. Returns the written paths.
pub(crate) fn cmd_migrate(
    file: &str,
    dry_run: bool,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) => {
            let out: Output<serde_json::Value> = Output::failure(format!(
                "{file}: unreadable ({e})"
            ))
            .with_next_step("pass a readable UTF-8 markdown file containing ## ADDED Requirements");
            emit_report(out, None, format, verbosity, stdout, stderr);
            return 2;
        }
    };
    // Generated frontmatter is always `id: spec` — the dual-format naming
    // law requires deltas to carry `id: spec` AND be named spec.md
    // (linter.dual_format_valid + id_matches_file). A different filename
    // cannot lint clean, so warn instead of silently generating an
    // id the linter will reject.
    let stem = std::path::Path::new(file)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    match migrate::migrate(&text) {
        Err(e) => {
            // Refusals are invocation errors, never tool failures
            // (specodelic-7rr precedent) — the file is never rewritten.
            let hint = match e {
                migrate::MigrateError::AlreadyMigrated => {
                    "run: spk lint <file> to verify the migrated file"
                }
                migrate::MigrateError::ConflictingDelta { .. } => {
                    "rename the requirement in one delta section (or hand-mirror the file), then re-run: spk migrate <file>"
                }
                _ => "a delta must carry an ## ADDED Requirements section",
            };
            let out: Output<serde_json::Value> =
                Output::failure(format!("{file}: {e}")).with_next_step(hint);
            emit_report(out, None, format, verbosity, stdout, stderr);
            2
        }
        Ok(outcome) => {
            let write_result = if !dry_run {
                std::fs::write(file, &outcome.content)
            } else {
                Ok(())
            };
            if let Err(e) = write_result {
                let out: Output<serde_json::Value> =
                    Output::failure(format!("{file}: write failed ({e})"))
                        .with_next_step("check file permissions");
                emit_report(out, None, format, verbosity, stdout, stderr);
                return 2;
            }
            // Hand-finish checklist (D4): the scaffold is a marked
            // skeleton, the author derives the real content.
            let mut steps = vec![
                "replace every scaffold_* id with the real row id".to_string(),
                "replace the scaffold intent statement with the real requirement".to_string(),
            ];
            if outcome.inserted_frontmatter {
                steps
                    .push("deltas conventionally carry id: spec — adjust if the generated frontmatter needs it".to_string());
            }
            let mut out: Output<serde_json::Value> = Output::success(serde_json::json!({
                "file": file,
                "dry_run": dry_run,
                "inserted_frontmatter": outcome.inserted_frontmatter,
                "inserted_layers": outcome.inserted_layers,
                "inserted_mirror": outcome.inserted_mirror,
                "content": outcome.content,
            }))
            .with_next_step(
                "run: spk lint <file> — the scaffold lints clean; keep it green as you fill it in",
            );
            if outcome.inserted_frontmatter && stem != "spec" {
                out = out.with_warning(format!(
                    "dual-format naming law: the file must be named spec.md (id `spec` matches the stem) — rename {file} to spec.md before linting"
                ));
            }
            for s in &steps {
                out = out.with_warning(s.clone());
            }
            emit_report(out, None, format, verbosity, stdout, stderr);
            0
        }
    }
}

pub(crate) fn cmd_new(
    id: &str,
    file: Option<&str>,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let default_name = id.replace('.', "-");
    let name = file.unwrap_or(&default_name);
    let filename = if name.ends_with(".md") {
        name.to_string()
    } else {
        format!("{name}.md")
    };
    let path = std::path::PathBuf::from(&filename);
    if path.exists() {
        let out: Output<serde_json::Value> =
            Output::failure(format!("{} already exists", path.display()))
                .with_next_step("edit the existing file instead");
        // Refusing to overwrite is an invocation error, not a tool
        // failure (specodelic-7rr residual) — distinct exit code.
        emit_report(out, None, format, verbosity, stdout, stderr);
        return 2;
    }
    // Scaffold with per-layer HTML-comment guidance (task 4.1). The
    // guidance teaches in-place, is trivially deleted, and the linter —
    // which never parses prose — is unaffected by it (design Decision 6).
    // Closed sets render from the guide constants, so the scaffold cannot
    // drift from the enforced sets.
    let constraint_kinds = guide::CONSTRAINT_KINDS.join(" | ");
    let property_kinds = guide::PROPERTY_KINDS.join(" | ");
    let template = format!(
        r#"---
id: {id}
kind: intent
statement: "THE system SHALL …"
---

# {}

One paragraph of prose: why this exists, what it's not.

<!-- Intent layer: the statement must match one of the five EARS patterns
(Ubiquitous / Event-Driven / State-Driven / Unwanted-Behavior /
Optional-Feature) and carry an imperative SHALL. Learn them with:
spk explain ears -->

## Constraints

<!-- kind: one of {constraint_kinds}. `invariant` is the only kind a
transition guard may cite; `effect` is the only kind a state's emits may
reference. Every constraint needs a deriving property (coverage). See:
spk explain kinds -->

| id | kind | expr | traces_to |
|----|------|------|-----------|
| c1 | invariant | TODO — what must always hold | [[{id}]] |

## Model

### States

<!-- `emits: [[{id}.<effect-constraint-id>]]` may follow a state that
outputs. Refs are file-qualified wiki-links — bare ids and bare text do
not resolve. -->

- `initial`

### Transitions

<!-- guard: may cite an invariant Constraint by a file-qualified
[[wiki-link]], e.g. [[{id}.c1]], or a State (the "has reached state X"
pattern) — typed, not conventional (advisory constraints can never gate
a transition). Bare ids and bare text do not
resolve. Every state must appear as a from or to of at least one
transition. -->

| id | from | to | guard |
|----|------|----|-------|
| t1 | initial | initial | [[{id}.c1]] |

## Properties

<!-- kind: one of {property_kinds}. Every property must cite a constraint
in derives_from — as a file-qualified wiki-link, e.g. [[{id}.c1]]
(bare ids and bare text do not resolve); a `law` predicate requires
**identity:** and **associativity:** case labels (law_requires_cases). -->

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[{id}.c1]] | TODO — a generator | TODO — the property holds |

<!-- Delete each guidance comment as you fill the layer in.
Full format guide: spk explain -->
"#,
        id.split('.').next_back().unwrap_or(id)
    );
    match std::fs::write(&path, template) {
        Ok(()) => {
            let message = format!("created {}", path.display());
            let out = Output::success(message.clone()).with_next_step(format!(
                "fill in the four layers, then run: specodelic lint {0}",
                path.display()
            ));
            emit_report(out, Some(message), format, verbosity, stdout, stderr);
            0
        }
        Err(e) => {
            let out: Output<serde_json::Value> =
                Output::failure(format!("could not write {}: {e}", path.display()));
            emit_report(out, None, format, verbosity, stdout, stderr);
            1
        }
    }
}

pub(crate) fn cmd_doctor(
    fix: bool,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    // The check suite runs on the genesis doctor framework (specodelic-sok):
    // issue checks (specs dir, beads, SPECODELIC block — the block is
    // auto-fixable) go through DoctorRunner with --fix dispatch and
    // verify-after-fix; workspace facts (mode, core spec, discovery)
    // render from detail helpers into the payload.
    let root = std::path::Path::new(".");
    let core = std::path::Path::new("specs/specodelic.md");
    let report = match specodelic::doctor::run_checks(root, fix) {
        Ok(report) => report,
        Err(e) => {
            let out: Output<serde_json::Value> =
                Output::failure(format!("doctor could not run: {e}"));
            emit_report(out, None, format, verbosity, stdout, stderr);
            return 1;
        }
    };
    let payload = specodelic::doctor::payload(root, &report);
    let out = Output::success(payload.clone());

    // --fix: surface the runner's verify-after-fix verdicts on warnings
    let mut out = out;
    if fix {
        for check in &report.checks {
            if check.message.starts_with("fixed") {
                out = out.with_warning(format!("{}: {}", check.name, check.message));
            }
        }
    }

    // Knowledge currency (task 5.2): whenever a local specs/specodelic.md
    // exists, compare its latest `Revision N` heading (numerically)
    // against the binary's embedded FORMAT_REVISION — a vendored consumer
    // corpus can lag or lead. Warn — never fail.
    let mut out = if core.is_file() {
        currency_check(core, out)
    } else {
        out
    };

    // Update availability (specodelic-4le): a newer stable specodelic on
    // crates.io rides the same advisory channel. Transport failures are
    // silent (genesis contract) — the doctor never fails on the network.
    // CI is skipped by genesis itself (no network calls in pipelines).
    if let Some(info) = specodelic::update_notice::check_for_update(&update_cache_dir())
        && let Some(msg) = specodelic::update_notice::update_notice(&info)
    {
        out = out.with_warning(msg);
    }
    let mode = payload["mode"].as_str().unwrap_or("consumer");
    if mode == "consumer" {
        if !std::path::Path::new("specs").is_dir() {
            if openspec_tree_present() {
                out = out.with_next_step("run: specodelic lint openspec");
            } else {
                out = out.with_next_step("start a corpus with: spk new <intent.id>");
            }
        } else {
            out = out.with_next_step("run: specodelic lint specs");
        }
    } else {
        out = out.with_next_step("run: specodelic lint");
    }
    if !fix {
        for check in &report.checks {
            // Advisory rides the warnings channel — the primary next-step
            // stays the workspace-appropriate one (spk new / spk lint).
            // With --fix the fixed/failed verdicts already surfaced above.
            if let (true, Some(fix_cmd)) = (check.status.is_issue(), &check.fix) {
                out = out.with_warning(format!("{}: {}", check.name, fix_cmd));
            }
        }
    }
    emit_report(
        out,
        Some(human::doctor(&payload)),
        format,
        verbosity,
        stdout,
        stderr,
    );
    0
}

/// The update-check cache directory: genesis's default (`$XDG_CACHE_HOME`
/// when set — it already IS the cache home — else `$HOME/.cache`, then
/// `genesis/update-check`). Isolated for tests.
pub(crate) fn update_cache_dir() -> std::path::PathBuf {
    match std::env::var("XDG_CACHE_HOME") {
        Ok(xdg) if !xdg.is_empty() => std::path::PathBuf::from(xdg),
        _ => {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            std::path::PathBuf::from(home).join(".cache")
        }
    }
    .join("genesis")
    .join("update-check")
}

/// Knowledge-currency check (task 5.2): compare the local corpus' latest
/// `Revision N` heading (numerically) against the binary's embedded
/// `guide::FORMAT_REVISION`. Warn — never fail — when the corpus is
/// newer; skip with an informational note when the corpus carries no
/// revision headings.
pub(crate) fn currency_check(
    core: &std::path::Path,
    out: Output<serde_json::Value>,
) -> Output<serde_json::Value> {
    let text = match std::fs::read_to_string(core) {
        Ok(text) => text,
        // unreadable corpus (review EDGE-001): skip the check, but say so
        // — a silent skip would make the doctor look current when it
        // simply couldn't see the corpus
        Err(e) => {
            return out.with_warning(format!(
                "could not read {}: knowledge-currency check skipped ({e})",
                core.display()
            ));
        }
    };
    match guide::latest_revision(&text) {
        None => out.with_warning(
            "corpus has no `Revision N` headings — knowledge-currency check skipped".to_string(),
        ),
        Some(latest) => {
            let embedded = guide::revision_number(guide::FORMAT_REVISION).unwrap_or(0);
            if latest > embedded {
                out.with_warning(format!(
                    "local corpus is at {} while this binary embeds {} — upgrade specodelic or re-read the guide with: spk explain",
                    latest, embedded
                ))
            } else {
                out
            }
        }
    }
}
