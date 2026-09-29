//! `specodelic` (alias `spk`) — the Specodelic CLI.
//!
//! Purpose: lint, compile, verify, graph, and refactor Specodelic spec
//! files (the four-layer markdown format in `specs/specodelic.md`).
//! Responsibilities: wire clap verbs to the domain modules, emit every
//! result through genesis's `Output`/envelope so humans and agents get the
//! same story, and keep the command set aligned with the specced tools
//! (`compile.md`, `verify.md`, `graph.md`, `rename.md`, `merge.md`,
//! `orchestrate.md`, ...). Rationale: one file = one spec keeps the format
//! self-hosting; this binary is the machine half of that contract.

use clap::CommandFactory;
use clap::{Parser, Subcommand};
use genesis::cli::generate_completions;
use genesis::envelope::{Envelope, EnvelopeKind};
use genesis::guide::{CliFormat, CliVerbosity, Output, OutputFormat, Verbosity};

use specodelic::spec::Spec;
use specodelic::{blocks, compile, graph, guide, lint, model_check, spec};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(
    name = "specodelic",
    version = VERSION,
    about = "Specodelic — lint, compile, verify, and refactor the four-layer markdown spec format",
    after_help = genesis::guide::Verbosity::help_footer()
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[command(flatten)]
    verbose: CliVerbosity,

    #[command(flatten)]
    format: CliFormat,
}

#[derive(Subcommand)]
enum Commands {
    /// Lint spec files against the Specodelic invariants
    Lint {
        /// Files or directories to lint (defaults to ./specs)
        paths: Vec<String>,
    },
    /// Derive the typed reference graph from all spec files
    Graph {
        /// Files or directories (defaults to ./specs)
        paths: Vec<String>,
    },
    /// Translate a linted spec file into TOML / proptest / TLA+ artifacts
    Compile {
        /// Spec files to compile
        paths: Vec<String>,
        /// Directory for the written artifacts (default `specodelic/`,
        /// committed — byte-stable output makes reruns diff-visible)
        #[arg(long, default_value = "specodelic")]
        out_dir: String,
    },
    /// Run the model checker (stateright default, TLC opt-in) against compiled output
    ModelCheck {
        /// Spec files whose compiled module to check
        paths: Vec<String>,
        /// Directory holding the compiled artifacts (must match compile's
        /// out-dir — model_check never re-compiles)
        #[arg(long, default_value = "specodelic")]
        out_dir: String,
        /// Stated depth bound (exhaustive_within_bound — part of the report)
        #[arg(long, default_value_t = 100)]
        max_depth: u32,
        /// Optional state-count budget
        #[arg(long)]
        max_states: Option<u64>,
        /// Optional wall-clock budget in seconds
        #[arg(long)]
        timeout_secs: Option<u64>,
    },
    /// Run compiled proptest blocks and gate on the model_check outcome
    Verify {
        /// Spec files to verify
        paths: Vec<String>,
    },
    /// Rename a spec row id, updating the definition and every [[link]]
    Rename {
        /// Current id (e.g. compile.compile_is_total)
        old_id: String,
        /// New id
        new_id: String,
    },
    /// Advise on tidy-first splits for high unrelated fan-in nodes
    Refactor {
        /// Files or directories (defaults to ./specs)
        paths: Vec<String>,
    },
    /// Detect id collisions and dangling renames before a branch merge
    Merge {
        /// Branch being merged in (defaults to current diff)
        #[arg(long)]
        branch: Option<String>,
    },
    /// Run the full lint → compile → model_check → verify pipeline
    Orchestrate {
        /// Files or directories (defaults to ./specs)
        paths: Vec<String>,
    },
    /// Scaffold a new spec file from the four-layer template
    New {
        /// Intent id for the new spec (e.g. order.cancel)
        id: String,
        /// Target file (defaults to <id with . → ->.md)
        #[arg(short, long)]
        file: Option<String>,
    },
    /// Explain the Specodelic format — embedded guide, works offline
    Explain {
        /// Topic to explain; omit to list the topics
        topic: Option<String>,
    },
    /// Diagnose the Specodelic workspace setup
    Doctor,
    /// Write/refresh the SPECODELIC managed block in AGENTS.md (agent
    /// facing: lint rules + format revision + core commands)
    Init {
        /// Overwrite even if a newer block version was hand-edited
        #[arg(short, long)]
        force: bool,
    },
    /// File an issue against the upstream repo via gh
    Feedback {
        /// Kind of feedback: bug, feature, question, or chore
        kind: String,
        /// Print the issue body and gh command without submitting
        #[arg(long)]
        dry_run: bool,
        /// Read the last error from scratch to auto-populate the body
        #[arg(long)]
        from_last_error: bool,
        /// Override the issue title (wins over derived titles)
        #[arg(long)]
        title: Option<String>,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

fn main() {
    // Handle `--version --json` before normal parsing (genesis convention).
    // Hand-rolled (design Decision 4): the payload carries `format_revision`,
    // domain data genesis's fixed helper cannot express.
    if print_version_json() {
        return;
    }
    let cli = Cli::parse();
    let verbosity = cli.verbose.verbosity();
    let format = cli.format.format();

    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();

    let exit_code = run(&cli, format, verbosity, &mut stdout, &mut stderr);
    std::process::exit(exit_code);
}

/// `--version --json` handled pre-parse (genesis convention) with a
/// domain-extended payload: `{name, version, format_revision}` — genesis's
/// `maybe_print_version_json` prints a fixed `{name, version}` payload with
/// no extension point, so the envelope is hand-rolled here (design
/// Decision 4). Plain `--version` falls through to clap.
fn print_version_json() -> bool {
    let args: Vec<String> = std::env::args().collect();
    let has_version = args.iter().any(|a| a == "--version" || a == "-V");
    let has_json = args.iter().any(|a| a == "--json" || a == "-j");
    if !(has_version && has_json) {
        return false;
    }
    let envelope = Envelope::success(
        VERSION,
        EnvelopeKind::Version,
        serde_json::json!({
            "name": "specodelic",
            "version": VERSION,
            "format_revision": guide::FORMAT_REVISION,
        }),
        vec![],
        vec![],
    );
    println!("{}", serde_json::to_string(&envelope).unwrap());
    true
}

/// `spk explain [TOPIC]` — serve the embedded format guide through the
/// envelope. Bare call lists topics; a known topic returns
/// `{topic, format_revision, body}`; an unknown topic fails with a hint
/// listing the valid topics (design Decision 3). Works offline: the body
/// is embedded via `include_str!`, no repo access needed.
fn cmd_explain(
    topic: Option<&str>,
    cli: &Cli,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let Some(topic) = topic else {
        let topics: Vec<serde_json::Value> = guide::TOPICS
            .iter()
            .map(|(id, title)| serde_json::json!({ "id": id, "title": title }))
            .collect();
        let out: Output<serde_json::Value> =
            Output::success(serde_json::json!({ "topics": topics }))
                .with_next_step("run: specodelic explain <topic> — try: specodelic explain format");
        emit(&out, cli, format, verbosity, stdout, stderr);
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
                emit(&out, cli, format, verbosity, stdout, stderr);
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
            emit(&out, cli, format, verbosity, stdout, stderr);
            1
        }
    }
}

/// `spk init` — write/refresh the SPECODELIC managed block in AGENTS.md
/// (specodelic-ze4). Idempotent: injects when missing, updates in place
/// when present; surrounding content is never touched. `--force` is
/// accepted for explicit refresh intent (the injector always rewrites the
/// block body — force exists so scripts record the intent).
fn cmd_init(
    _force: bool,
    cli: &Cli,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let path = std::path::Path::new(blocks::BLOCK_FILE);
    match blocks::inject_into(path) {
        Ok(result) => {
            let action = match result {
                genesis::managed_block::InjectResult::Created => "created",
                genesis::managed_block::InjectResult::Prepended => "injected",
                genesis::managed_block::InjectResult::Updated => "updated",
            };
            let out: Output<serde_json::Value> = Output::success(serde_json::json!({
                "file": blocks::BLOCK_FILE,
                "block": action,
                "format_revision": guide::FORMAT_REVISION,
            }))
            .with_next_step(
                "agents in this repo now see the spec rules; check specs with: spk lint",
            );
            emit(&out, cli, format, verbosity, stdout, stderr);
            0
        }
        Err(e) => {
            let out: Output<serde_json::Value> = Output::failure(format!(
                "could not write {}: {e}",
                blocks::BLOCK_FILE
            ))
            .with_next_step("check directory permissions, or pass an explicit path once AGENTS.md support lands elsewhere");
            emit(&out, cli, format, verbosity, stdout, stderr);
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
fn cmd_feedback(kind: &str, dry_run: bool, from_last_error: bool, title: Option<&str>) -> i32 {
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

fn run(
    cli: &Cli,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    match &cli.command {
        Commands::Lint { paths } => cmd_lint(paths, cli, format, verbosity, stdout, stderr),
        Commands::Graph { paths } => cmd_graph(paths, cli, format, verbosity, stdout, stderr),
        Commands::Compile { paths, out_dir } => {
            cmd_compile(paths, out_dir, cli, format, verbosity, stdout, stderr)
        }
        Commands::ModelCheck {
            paths,
            out_dir,
            max_depth,
            max_states,
            timeout_secs,
        } => cmd_model_check(
            paths,
            CheckTarget {
                out_dir: out_dir.clone(),
                bound: model_check::Bound {
                    max_depth: *max_depth,
                    max_states: *max_states,
                    timeout_secs: *timeout_secs,
                },
            },
            cli,
            format,
            verbosity,
            stdout,
            stderr,
        ),
        Commands::Verify { .. } => {
            let out: Output<serde_json::Value> =
                Output::failure("not yet implemented — specced in specs/verify.md")
                    .with_next_step("track progress: bd ready");
            out.emit(VERSION, format, verbosity, stdout, stderr).ok();
            1
        }
        Commands::Rename { .. } | Commands::Refactor { .. } | Commands::Merge { .. } => {
            let out: Output<serde_json::Value> =
                Output::failure("not yet implemented — specced in specs/rename.md, specs/refactor.md, and specs/merge.md")
                    .with_next_step("track progress: bd ready");
            out.emit(VERSION, format, verbosity, stdout, stderr).ok();
            1
        }
        Commands::Orchestrate { .. } => {
            let out: Output<serde_json::Value> =
                Output::failure("not yet implemented — specced in specs/orchestrate.md (lint → compile → model_check → verify)")
                    .with_next_step("run the stages individually: specodelic lint specs && specodelic graph specs");
            out.emit(VERSION, format, verbosity, stdout, stderr).ok();
            1
        }
        Commands::New { id, file } => {
            cmd_new(id, file.as_deref(), cli, format, verbosity, stdout, stderr)
        }
        Commands::Explain { topic } => {
            cmd_explain(topic.as_deref(), cli, format, verbosity, stdout, stderr)
        }
        Commands::Doctor => cmd_doctor(cli, format, verbosity, stdout, stderr),
        Commands::Init { force } => cmd_init(*force, cli, format, verbosity, stdout, stderr),
        Commands::Feedback {
            kind,
            dry_run,
            from_last_error,
            title,
        } => cmd_feedback(kind, *dry_run, *from_last_error, title.as_deref()),
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            match generate_completions(&mut cmd, *shell) {
                Ok(()) => 0,
                Err(e) => {
                    let out: Output<serde_json::Value> =
                        Output::failure(format!("completions generation failed: {e}"));
                    out.emit(VERSION, format, verbosity, stdout, stderr).ok();
                    1
                }
            }
        }
    }
}

/// Collect spec files from paths (files, or directories searched
/// recursively for `.md`, skipping hidden and build directories:
/// `.git`, `target`, `node_modules`, anything dot-prefixed).
fn collect_specs(paths: &[String]) -> Vec<std::path::PathBuf> {
    /// Directories never descended into during recursive collection
    /// (beads specodelic-6pi: recursion must not sweep VCS/build junk).
    fn skipped_dir(p: &std::path::Path) -> bool {
        match p.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.starts_with('.') || name == "target" || name == "node_modules",
            None => true,
        }
    }
    fn walk(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                if !skipped_dir(&p) {
                    walk(&p, files);
                }
            } else if p.extension().is_some_and(|e| e == "md") {
                files.push(p);
            }
        }
    }
    let roots: Vec<std::path::PathBuf> = if paths.is_empty() {
        vec!["specs".into()]
    } else {
        paths.iter().map(std::path::PathBuf::from).collect()
    };
    let mut files = vec![];
    for root in roots {
        if root.is_dir() {
            // The root itself is never skipped (the user named it);
            // only nested directories are filtered.
            walk(&root, &mut files);
        } else {
            files.push(root);
        }
    }
    files
}

/// Parse a batch, skipping non-spec files (no frontmatter) with a note.
fn parse_batch(paths: &[String], verbosity: Verbosity) -> (Vec<Spec>, Vec<String>) {
    let mut specs = vec![];
    let mut notes = vec![];
    for f in collect_specs(paths) {
        let text = match std::fs::read_to_string(&f) {
            Ok(t) => t,
            Err(e) => {
                notes.push(format!("{}: unreadable ({e})", f.display()));
                continue;
            }
        };
        // Non-spec files (STATUS.md, USAGE.md, ...) have no frontmatter — skip.
        if !text.trim_start().starts_with("---") {
            notes.push(format!(
                "{}: skipped (no frontmatter — not a spec file)",
                f.display()
            ));
            continue;
        }
        match spec::parse_str(&text) {
            Ok(mut s) => {
                s.path = Some(f);
                specs.push(s);
            }
            Err(e) => notes.push(format!("{}: parse error: {e}", f.display())),
        }
    }
    let _ = verbosity;
    (specs, notes)
}

fn emit<T: serde::Serialize + std::fmt::Debug>(
    out: &Output<T>,
    cli: &Cli,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) {
    out.emit(VERSION, format, verbosity, stdout, stderr).ok();
    let _ = cli;
}

fn cmd_lint(
    paths: &[String],
    cli: &Cli,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let (specs, notes) = parse_batch(paths, verbosity);
    if specs.is_empty() {
        // Never a silent ok:true on zero files — that's a false green
        // (beads specodelic-6pi).
        let (msg, hint) = if notes.iter().any(|n| n.contains("parse error")) {
            (
                "spec files failed to parse",
                "fix the frontmatter/tables reported above",
            )
        } else {
            (
                "no spec files found — nothing was linted",
                "pass files or directories containing *.md specs with YAML frontmatter (directories are searched recursively; hidden and build dirs are skipped)",
            )
        };
        let out: Output<serde_json::Value> = Output::failure(msg).with_next_step(hint);
        emit(&out, cli, format, verbosity, stdout, stderr);
        return 1;
    }
    let report = lint::lint_corpus(&specs);
    let payload = serde_json::to_value(&report).unwrap_or_default();
    let failures = report.failures();
    let mut out = Output::success(payload);
    for n in &notes {
        out = out.with_warning(n.clone());
    }
    if failures > 0 {
        out = out.with_next_step("fix the reported invariants, or cite the governing constraint in Notes if it's a false positive");
    } else {
        out = out.with_next_step("run: specodelic graph");
    }
    emit(&out, cli, format, verbosity, stdout, stderr);
    if failures > 0 { 1 } else { 0 }
}

fn cmd_graph(
    paths: &[String],
    cli: &Cli,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let (specs, notes) = parse_batch(paths, verbosity);
    let report = graph::build(&specs);
    let payload = serde_json::to_value(&report).unwrap_or_default();
    let mut out = Output::success(payload);
    for n in &notes {
        out = out.with_warning(n.clone());
    }
    if report.dangling.is_empty() {
        out = out.with_next_step("every reference resolves — run: specodelic lint");
    } else {
        out = out.with_next_step(
            "resolve the dangling references (see specs/linter-referential_integrity.md)",
        );
    }
    emit(&out, cli, format, verbosity, stdout, stderr);
    if report.dangling.is_empty() { 0 } else { 1 }
}

fn cmd_compile(
    paths: &[String],
    out_dir: &str,
    cli: &Cli,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let (specs, notes) = parse_batch(paths, verbosity);
    if specs.is_empty() {
        let out: Output<serde_json::Value> =
            Output::failure("no spec files to compile").with_next_step(
                "pass spec files or a directory (defaults to ./specs); parse errors, if any, are reported as warnings",
            );
        emit(&out, cli, format, verbosity, stdout, stderr);
        return 1;
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
                let written = write_artifacts(spec, &c, out_dir);
                match written {
                    Ok(files) => {
                        // Two files with the same stem (different dirs)
                        // would silently overwrite each other's artifacts.
                        let stem = artifact_stem(spec);
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
    let mut out = Output::success(payload);
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
    emit(&out, cli, format, verbosity, stdout, stderr);
    if failed.is_empty() { 0 } else { 1 }
}

/// The artifact filename stem for a spec: the file stem when on disk,
/// else the intent id with `.` → `-`.
fn artifact_stem(spec: &Spec) -> String {
    spec.path
        .as_ref()
        .and_then(|p| p.file_stem().and_then(|s| s.to_str()))
        .map(str::to_string)
        .unwrap_or_else(|| spec.intent.id.replace('.', "-"))
}

/// The model-check invocation parameters beyond the spec paths — the
/// artifact directory plus the stated bound, grouped to keep the shared
/// emit plumbing (cli/format/verbosity/streams) within the arg lint.
struct CheckTarget {
    out_dir: String,
    bound: model_check::Bound,
}

/// Run `spk model-check` — the model_check step (specs/model_check.md).
/// Never re-compiles: consumes the compiled `<stem>.tla` artifact from
/// `out_dir`, runs the native stateright backend within the stated
/// bound, and persists a `<stem>.check.json` run report carrying the
/// artifact's SHA-256 so `verify` can detect stale clean results.
fn cmd_model_check(
    paths: &[String],
    target: CheckTarget,
    cli: &Cli,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let out_dir = target.out_dir.as_str();
    let (specs, notes) = parse_batch(paths, verbosity);
    if specs.is_empty() {
        let out: Output<serde_json::Value> = Output::failure("no spec files to model-check")
            .with_next_step(
                "pass spec files or a directory; each must have compiled artifacts (run: specodelic compile <files>)",
            );
        emit(&out, cli, format, verbosity, stdout, stderr);
        return 1;
    }

    let mut checked: Vec<serde_json::Value> = vec![];
    let mut failed: Vec<serde_json::Value> = vec![];
    let warnings: Vec<String> = notes;
    for spec in &specs {
        let file = spec
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| format!("<{}>", spec.intent.id));
        let stem = artifact_stem(spec);
        let tla_path = std::path::Path::new(out_dir).join(format!("{stem}.tla"));
        // The compiled module is the run's input — a missing artifact is
        // a labeled error with a remediation hint, never a run.
        let tla_artifact = match std::fs::read(&tla_path) {
            Ok(bytes) => bytes,
            Err(e) => {
                failed.push(serde_json::json!({
                    "file": file,
                    "id": spec.intent.id,
                    "stage": "missing_artifact",
                    "message": format!(
                        "no compiled module at {}: {e} — model-check consumes compile's output, it never re-compiles (run: specodelic compile <files>)",
                        tla_path.display()
                    ),
                }));
                continue;
            }
        };
        let ir = compile::extract_model_ir(spec);
        match model_check::run(&ir, &tla_artifact, &target.bound) {
            Ok(report) => {
                let report_path = std::path::Path::new(out_dir).join(format!("{stem}.check.json"));
                let report_json =
                    serde_json::to_string_pretty(&report).expect("RunReport serializes");
                if let Err(e) = std::fs::write(&report_path, &report_json) {
                    failed.push(serde_json::json!({
                        "file": file,
                        "id": spec.intent.id,
                        "stage": "write_report",
                        "message": format!(
                            "could not write {}: {e}",
                            report_path.display()
                        ),
                    }));
                    continue;
                }
                checked.push(serde_json::json!({
                    "file": file,
                    "id": spec.intent.id,
                    "outcome": report.outcome,
                    "backend": report.backend,
                    "bound": report.bound,
                    "invariants_checked": report.invariants_checked,
                    "states_explored": report.states_explored,
                    "artifact_sha256": report.artifact_sha256,
                    "written": report_path.display().to_string(),
                }));
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
    let mut out = Output::success(payload);
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
    emit(&out, cli, format, verbosity, stdout, stderr);
    if failed.is_empty() { 0 } else { 1 }
}

/// Write `<stem>.toml`, `<stem>_props.rs`, and `<stem>.tla` into `out_dir`. Byte-stable
/// output: the same input always produces the same bytes, so committed
/// artifacts make reruns diff-visible. Returns the written paths.
fn write_artifacts(
    spec: &Spec,
    compiled: &compile::Compiled,
    out_dir: &str,
) -> Result<Vec<String>, String> {
    let stem = artifact_stem(spec);
    std::fs::create_dir_all(out_dir)
        .map_err(|e| format!("could not create out-dir {out_dir}: {e}"))?;
    let toml_path = std::path::Path::new(out_dir).join(format!("{stem}.toml"));
    let props_path = std::path::Path::new(out_dir).join(format!("{stem}_props.rs"));
    let tla_path = std::path::Path::new(out_dir).join(format!("{stem}.tla"));
    std::fs::write(&toml_path, &compiled.toml)
        .map_err(|e| format!("could not write {}: {e}", toml_path.display()))?;
    std::fs::write(&props_path, &compiled.props)
        .map_err(|e| format!("could not write {}: {e}", props_path.display()))?;
    std::fs::write(&tla_path, &compiled.tla)
        .map_err(|e| format!("could not write {}: {e}", tla_path.display()))?;
    Ok(vec![
        toml_path.display().to_string(),
        props_path.display().to_string(),
        tla_path.display().to_string(),
    ])
}

fn cmd_new(
    id: &str,
    file: Option<&str>,
    cli: &Cli,
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
        emit(&out, cli, format, verbosity, stdout, stderr);
        return 1;
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

## Model

### States

<!-- `emits: [[<effect-constraint>]]` may follow a state that outputs. -->

- `initial`

### Transitions

<!-- guard: must cite an invariant Constraint by [[id]] — typed, not
conventional (advisory constraints can never gate a transition). Every
state must appear as a from or to of at least one transition. -->

| id | from | to | guard |
|----|------|----|-------|
| t1 | initial | initial | TODO — cite a Constraint by [[id]] |

## Properties

<!-- kind: one of {property_kinds}. Every property must cite a constraint
in derives_from; a `law` predicate requires **identity:** and
**associativity:** case labels (law_requires_cases). -->

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|

<!-- Delete each guidance comment as you fill the layer in.
Full format guide: spk explain -->
"#,
        id.split('.').next_back().unwrap_or(id)
    );
    match std::fs::write(&path, template) {
        Ok(()) => {
            let out =
                Output::success(format!("created {}", path.display())).with_next_step(format!(
                    "fill in the four layers, then run: specodelic lint {0}",
                    path.display()
                ));
            emit(&out, cli, format, verbosity, stdout, stderr);
            0
        }
        Err(e) => {
            let out: Output<serde_json::Value> =
                Output::failure(format!("could not write {}: {e}", path.display()));
            emit(&out, cli, format, verbosity, stdout, stderr);
            1
        }
    }
}

fn cmd_doctor(
    cli: &Cli,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    // Workspace mode: self-hosting when the repo carries the format's own
    // core spec, consumer otherwise (add-embedded-aix-guide task 5.1).
    let core = std::path::Path::new("specs/specodelic.md");
    let mode = if core.is_file() {
        "self_hosting"
    } else {
        "consumer"
    };
    let mut checks: Vec<(String, String)> = vec![(
        "mode".into(),
        if mode == "self_hosting" {
            "self_hosting — the corpus lives here".into()
        } else {
            "consumer — the format is provided by the installed binary".into()
        },
    )];
    checks.push((
        "specs/ directory".into(),
        if std::path::Path::new("specs").is_dir() {
            "ok".into()
        } else {
            "missing — start a corpus with: spk new".into()
        },
    ));
    checks.push((
        "core format spec".into(),
        if mode == "self_hosting" {
            "ok (specs/specodelic.md)".into()
        } else {
            // consumers don't carry the corpus — informational, never a
            // failure (the embedded guide serves the format instead)
            format!(
                "not present (consumer mode — the binary embeds {})",
                guide::FORMAT_REVISION
            )
        },
    ));
    checks.push((
        "beads".into(),
        if std::path::Path::new(".beads/config.yaml").is_file() {
            "ok (.beads/config.yaml)".into()
        } else {
            "not initialized — run: bd init".into()
        },
    ));

    // Managed-block currency (specodelic-ze4): AGENTS.md should carry the
    // SPECODELIC block so agents see the rules without reading upstream.
    // Advisory: missing/stale → hint to run `spk init`, never a failure.
    let agents = std::path::Path::new(blocks::BLOCK_FILE);
    let (block_check, block_current) = if !blocks::has_block(agents) {
        (
            "missing — agents in this repo can't see the spec rules; run: spk init".to_string(),
            false,
        )
    } else {
        match blocks::block_format_revision(agents) {
            Some(rev) => {
                let embedded = guide::revision_number(guide::FORMAT_REVISION).unwrap_or(0);
                if rev < embedded {
                    (
                        format!(
                            "stale — block names Revision {rev}, binary embeds {} ; run: spk init",
                            guide::FORMAT_REVISION
                        ),
                        false,
                    )
                } else {
                    (
                        "ok (Revision {rev} ≥ embedded)".replace("{rev}", &rev.to_string()),
                        true,
                    )
                }
            }
            None => (
                "present but names no revision — run: spk init to refresh".into(),
                false,
            ),
        }
    };
    checks.push(("SPECODELIC block".into(), block_check));

    let out = Output::success(serde_json::json!({
        "mode": mode,
        "checks": checks,
        "format_revision": guide::FORMAT_REVISION,
    }));

    // Knowledge currency (task 5.2): whenever a local specs/specodelic.md
    // exists, compare its latest `Revision N` heading (numerically)
    // against the binary's embedded FORMAT_REVISION — a vendored consumer
    // corpus can lag or lead. Warn — never fail.
    let mut out = if core.is_file() {
        currency_check(core, out)
    } else {
        out
    };
    if mode == "consumer" {
        if !std::path::Path::new("specs").is_dir() {
            out = out.with_next_step("start a corpus with: spk new <intent.id>");
        } else {
            out = out.with_next_step("run: specodelic lint specs");
        }
    } else {
        out = out.with_next_step("run: specodelic lint");
    }
    if !block_current {
        // Advisory rides the warnings channel — the primary next-step
        // stays the workspace-appropriate one (spk new / spk lint).
        out = out.with_warning("SPECODELIC block missing or stale in AGENTS.md — run: spk init");
    }
    emit(&out, cli, format, verbosity, stdout, stderr);
    0
}

/// Knowledge-currency check (task 5.2): compare the local corpus' latest
/// `Revision N` heading (numerically) against the binary's embedded
/// `guide::FORMAT_REVISION`. Warn — never fail — when the corpus is
/// newer; skip with an informational note when the corpus carries no
/// revision headings.
fn currency_check(
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
