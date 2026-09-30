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
use specodelic::{
    blocks, checklist, compile, graph, guide, human, lint, merge, model_check, rename, spec, verify,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Exit-code contract (specodelic-7rr item 2, CLARITY-pinned):
/// 0 = success (lint with zero findings counts); 1 = the stage produced
/// findings / a tool-level failure; 2 = invocation error — nothing was
/// processed (path not found, no spec files matched, unreadable input).
/// clap's own argument-parse failures also exit 2, so the mapping is
/// uniform. Documented here, in README.md, and in specs/USAGE.md.
fn exit_code_footer() -> String {
    format!(
        "{}\n\nExit codes:\n  0 = success\n  1 = findings or tool-level failure\n  2 = invocation error (path not found, no spec files matched)",
        genesis::guide::Verbosity::help_footer()
    )
}

#[derive(Parser)]
#[command(
    name = "specodelic",
    version = VERSION,
    about = "Specodelic — lint, compile, verify, and refactor the four-layer markdown spec format",
    // Without an explicit long_about, clap promotes the flattened
    // genesis CliFormat struct's doc comment above Usage in `--help`
    // (gh#6.3: internal dev docs leaked into user help).
    long_about = "Specodelic — lint, compile, verify, and refactor the four-layer markdown spec format",
    after_help = exit_code_footer(),
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[command(flatten)]
    verbose: CliVerbosity,

    #[command(flatten)]
    format: CliFormat,
}

#[derive(clap::ValueEnum, Clone)]
enum HooksAction {
    /// Wire `spk lint openspec` into the repo's pre-commit chain
    Install,
    /// Remove the specodelic managed block from the hook chain
    Uninstall,
}

/// The model-check backend selection (specs/model_check.md, Backends).
#[derive(clap::ValueEnum, Clone, PartialEq)]
enum ModelBackend {
    /// Embedded stateright BFS exploration (default — exhaustive runs
    /// report exploration_only, never no_counterexample)
    Stateright,
    /// The TLA+ TLC reference engine — JVM subprocess over the compiled
    /// module; requires --tlc-jar
    Tlc,
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
        /// Model-check backend (specs/model_check.md: stateright embedded
        /// default; tlc = the JVM reference engine over the compiled .tla)
        #[arg(long, value_enum, default_value_t = ModelBackend::Stateright)]
        backend: ModelBackend,
        /// Path to tla2tools.jar (required for --backend tlc; the JVM
        /// binary comes from PATH or SPK_TLC_JAVA)
        #[arg(long)]
        tlc_jar: Option<String>,
    },
    /// Run compiled proptest blocks and gate on the model_check outcome
    Verify {
        /// Spec files to verify
        paths: Vec<String>,
        /// Directory holding the compiled artifacts (must match compile's
        /// out-dir — verify never re-compiles)
        #[arg(long, default_value = "specodelic")]
        out_dir: String,
    },
    /// Rename a spec row id, updating the definition and every [[link]]
    /// atomically (all-or-nothing, verified against the linters)
    Rename {
        /// Current id (e.g. compile.compile_is_total)
        old_id: String,
        /// New id
        new_id: String,
        /// Files or directories to operate on (defaults to ./specs)
        paths: Vec<String>,
    },
    /// Advise on tidy-first splits for high unrelated fan-in nodes
    Refactor {
        /// Files or directories (defaults to ./specs)
        paths: Vec<String>,
    },
    /// Detect id collisions and dangling renames before a branch merge
    Merge {
        /// Incoming (branch) spec tree directory
        #[arg(long)]
        branch: Option<String>,
        /// Common-ancestor spec tree directory (omit for a conservative
        /// check where every id shared by both tips counts as new)
        #[arg(long)]
        base: Option<String>,
        /// Current spec tree (defaults to ./specs)
        paths: Vec<String>,
    },
    /// Run the full lint → compile → model_check → verify pipeline (not yet implemented — stub)
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
    /// Wire or unwire the specodelic dual-format gate in the repo's
    /// hook chain (lefthook managed block — never claims core.hooksPath)
    Hooks {
        /// Whether to wire (install) or unwire (uninstall) the gate
        #[arg(value_enum)]
        action: HooksAction,
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
fn cmd_init(
    _force: bool,
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
            let payload = serde_json::json!({
                "file": blocks::BLOCK_FILE,
                "block": action,
                "format_revision": guide::FORMAT_REVISION,
            });
            let out: Output<serde_json::Value> = Output::success(payload.clone()).with_next_step(
                "agents in this repo now see the spec rules; check specs with: spk lint",
            );
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
fn cmd_hooks(
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

/// Outcome of the install-time gate dry-run.
struct GateDryRun {
    passed: bool,
    summary: String,
}

/// Summarize the gate dry-run output for the envelope. The gate runs in
/// its default (JSON envelope) mode on a pipe — a single-line envelope;
/// recognize it and summarize semantically (issue count / lint findings).
/// Any other output: the last meaningful line, truncated.
fn summarize_gate_output(raw: &str, passed: bool) -> String {
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
fn gate_dry_run(root: &std::path::Path) -> GateDryRun {
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
        Commands::Lint { paths } => cmd_lint(paths, format, verbosity, stdout, stderr),
        Commands::Graph { paths } => cmd_graph(paths, format, verbosity, stdout, stderr),
        Commands::Compile { paths, out_dir } => {
            cmd_compile(paths, out_dir, format, verbosity, stdout, stderr)
        }
        Commands::ModelCheck {
            paths,
            out_dir,
            max_depth,
            max_states,
            timeout_secs,
            backend,
            tlc_jar,
        } => cmd_model_check(
            paths,
            CheckTarget {
                out_dir: out_dir.clone(),
                bound: model_check::Bound {
                    max_depth: *max_depth,
                    max_states: *max_states,
                    timeout_secs: *timeout_secs,
                },
                backend: backend.clone(),
                tlc_jar: tlc_jar.clone(),
            },
            format,
            verbosity,
            stdout,
            stderr,
        ),
        Commands::Verify { paths, out_dir } => {
            cmd_verify(paths, out_dir, format, verbosity, stdout, stderr)
        }
        Commands::Rename {
            old_id,
            new_id,
            paths,
        } => cmd_rename(old_id, new_id, paths, format, verbosity, stdout, stderr),
        Commands::Refactor { .. } => {
            let out: Output<serde_json::Value> =
                Output::failure("not yet implemented — specced in specs/refactor.md")
                    .with_next_step("track progress: bd ready");
            emit_report(out, None, format, verbosity, stdout, stderr);
            1
        }
        Commands::Merge {
            branch,
            base,
            paths,
        } => cmd_merge(
            branch.as_deref(),
            base.as_deref(),
            paths,
            format,
            verbosity,
            stdout,
            stderr,
        ),
        Commands::Orchestrate { .. } => {
            let out: Output<serde_json::Value> =
                Output::failure("not yet implemented — specced in specs/orchestrate.md (lint → compile → model_check → verify)")
                    .with_next_step("run the stages individually: specodelic lint specs && specodelic graph specs");
            emit_report(out, None, format, verbosity, stdout, stderr);
            1
        }
        Commands::New { id, file } => {
            cmd_new(id, file.as_deref(), format, verbosity, stdout, stderr)
        }
        Commands::Explain { topic } => {
            cmd_explain(topic.as_deref(), format, verbosity, stdout, stderr)
        }
        Commands::Doctor => cmd_doctor(format, verbosity, stdout, stderr),
        Commands::Init { force } => cmd_init(*force, format, verbosity, stdout, stderr),
        Commands::Feedback {
            kind,
            dry_run,
            from_last_error,
            title,
        } => cmd_feedback(kind, *dry_run, *from_last_error, title.as_deref()),
        Commands::Hooks { action } => cmd_hooks(
            matches!(action, HooksAction::Install),
            format,
            verbosity,
            stdout,
            stderr,
        ),
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            match generate_completions(&mut cmd, *shell) {
                Ok(()) => 0,
                Err(e) => {
                    let out: Output<serde_json::Value> =
                        Output::failure(format!("completions generation failed: {e}"));
                    emit_report(out, None, format, verbosity, stdout, stderr);
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

/// The ingestion size cap (hostile-input gate, specodelic-suz): a
/// spec file is prose — corpus files are ~10-50 KB; anything beyond
/// this bound is not a spec, and reading it is an unbounded-memory risk.
const MAX_INPUT_BYTES: u64 = 2 * 1024 * 1024;

/// Corpus discovery (gh#2.2, specodelic-ag5): consumer repos keep their
/// specs under an openspec/ tree (openspec/changes/*/specs/…), while bare
/// `spk lint`/`spk graph` scan `specs/` — so the natural invocation finds
/// nothing and the working one is undiscoverable. When cwd carries an
/// openspec/ tree, the zero-files failures and doctor name it instead of
/// leaving trial-and-error.
fn openspec_tree_present() -> bool {
    std::path::Path::new("openspec").is_dir()
}

/// Parse a batch, skipping non-spec files (no frontmatter) with a note.
/// A `*.checklist.md` file is not a spec — it's the external-completeness
/// manifest (specs/linter-external_completeness.md, mp1 row 10 decision):
/// parsed leniently into [`checklist::Checklist`] and returned alongside
/// the specs for `spk lint`'s optional non-gating pass. Other commands
/// ignore checklists (external_completeness never gates a stage).
fn parse_batch(
    paths: &[String],
    verbosity: Verbosity,
) -> (Vec<Spec>, Vec<checklist::Checklist>, Vec<String>) {
    let mut specs = vec![];
    let mut checklists = vec![];
    let mut notes = vec![];
    for f in collect_specs(paths) {
        // Hostile-input gate (specodelic-suz): only regular files of a
        // bounded size are ever read. A FIFO named *.md blocks forever;
        // a char device (/dev/zero) reads unbounded and OOM-kills; an
        // oversized regular file bloats memory. All are labeled, name
        // the offending path, and are skipped — never read.
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
        let text = match std::fs::read_to_string(&f) {
            Ok(t) => t,
            Err(e) => {
                notes.push(format!("{}: unreadable ({e})", f.display()));
                continue;
            }
        };
        // A checklist manifest declares the external-completeness pass —
        // intercepted before the frontmatter check (it intentionally has
        // none) and never left on the floor as a "skipped" note.
        if checklist::is_checklist_path(&f) {
            checklists.push(checklist::parse_str(f.clone(), &text));
            continue;
        }
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
    (specs, checklists, notes)
}

fn emit_report<T: serde::Serialize + std::fmt::Debug>(
    out: Output<T>,
    human_text: Option<String>,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) {
    if format == OutputFormat::Human {
        if let Some(text) = &human_text {
            writeln!(stdout, "{text}").ok();
        }
        out.with_verbosity(Verbosity::MAX + 1)
            .emit(VERSION, format, verbosity, stdout, stderr)
            .ok();
    } else {
        out.emit(VERSION, format, verbosity, stdout, stderr).ok();
    }
}

fn cmd_lint(
    paths: &[String],
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let (specs, checklists, notes) = parse_batch(paths, verbosity);
    if specs.is_empty() && checklists.is_empty() {
        // Never a silent ok:true on zero files — that's a false green
        // (beads specodelic-6pi). A declared checklist alone still gets
        // linted (checklist_well_formed needs no specs), so the failure
        // only fires when NEITHER was found.
        let (msg, hint) = if notes.iter().any(|n| n.contains("parse error")) {
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
        for n in &notes {
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
    for n in &notes {
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
    if failures > 0 { 1 } else { 0 }
}

fn cmd_graph(
    paths: &[String],
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let (specs, _checklists, notes) = parse_batch(paths, verbosity);
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

fn cmd_rename(
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
fn read_tree(dir: &str) -> (Vec<(String, String)>, Vec<String>) {
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

fn cmd_merge(
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

fn cmd_compile(
    paths: &[String],
    out_dir: &str,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let (specs, _checklists, notes) = parse_batch(paths, verbosity);
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
/// artifact directory, the stated bound, and the backend selection,
/// grouped to keep the shared emit plumbing (cli/format/verbosity/
/// streams) within the arg lint.
struct CheckTarget {
    out_dir: String,
    bound: model_check::Bound,
    backend: ModelBackend,
    tlc_jar: Option<String>,
}

/// Run `spk model-check` — the model_check step (specs/model_check.md).
/// Never re-compiles: consumes the compiled `<stem>.tla` artifact from
/// `out_dir`, runs the native stateright backend within the stated
/// bound, and persists a `<stem>.check.json` run report carrying the
/// artifact's SHA-256 so `verify` can detect stale clean results.
fn cmd_model_check(
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
    let (specs, _checklists, notes) = parse_batch(paths, verbosity);
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
        let result = match &tlc_paths {
            Some(tlc) => model_check::run_tlc(&ir, &tla_path, &tla_artifact, &target.bound, tlc),
            None => model_check::run(&ir, &tla_artifact, &target.bound),
        };
        match result {
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
fn cmd_verify(
    paths: &[String],
    out_dir: &str,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let dir = std::path::Path::new(out_dir);
    let (specs, _checklists, notes) = parse_batch(paths, verbosity);
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

    let runner = verify::CargoRunner;
    let mut verified: Vec<serde_json::Value> = vec![];
    let mut blocked: Vec<serde_json::Value> = vec![];
    for spec in &specs {
        let file = spec
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| format!("<{}>", spec.intent.id));
        let stem = artifact_stem(spec);
        let properties = verify::evaluate_properties_gate(spec, dir, &runner);
        let model = verify::evaluate_model_gate(dir, &stem);
        let v = verify::verdict(&properties.state, &model);
        let entry = serde_json::json!({
            "file": file,
            "id": spec.intent.id,
            "status": v.status,
            "message": v.message,
            "hint": v.hint,
            "properties": props_gate_json(&properties),
            "model": model_gate_json(&model),
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
fn props_gate_json(gate: &verify::PropertiesGate) -> serde_json::Value {
    let state = match &gate.state {
        verify::PropsGateState::Pass => "pass",
        verify::PropsGateState::Failed(_) => "failed",
        verify::PropsGateState::Stale => "stale",
        verify::PropsGateState::MissingArtifact => "missing_artifact",
        verify::PropsGateState::Uncompilable(_) => "uncompilable",
        verify::PropsGateState::RunnerUnavailable(_) => "runner_unavailable",
    };
    let detail = match &gate.state {
        verify::PropsGateState::Failed(s)
        | verify::PropsGateState::Uncompilable(s)
        | verify::PropsGateState::RunnerUnavailable(s) => Some(s.clone()),
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
fn model_gate_json(gate: &verify::ModelGateState) -> serde_json::Value {
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

<!-- guard: must cite an invariant Constraint by a file-qualified
[[wiki-link]], e.g. [[{id}.c1]] — typed, not conventional (advisory
constraints can never gate a transition). Bare ids and bare text do not
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

fn cmd_doctor(
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

    // Corpus discovery (gh#2.2): name where the specs actually live so
    // the working invocation is never trial-and-error.
    let discovery = if std::path::Path::new("specs").is_dir() {
        "ok (specs/)".to_string()
    } else if openspec_tree_present() {
        let n = collect_specs(&["openspec".to_string()]).len();
        format!("found openspec/ ({n} spec file(s)) — lint it with: spk lint openspec")
    } else {
        "no corpus — pass a directory containing *.md specs; hidden and build dirs are skipped"
            .to_string()
    };
    checks.push(("corpus discovery".into(), discovery));

    let payload = serde_json::json!({
        "mode": mode,
        "checks": checks,
        "format_revision": guide::FORMAT_REVISION,
    });
    let out = Output::success(payload.clone());

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
    if !block_current {
        // Advisory rides the warnings channel — the primary next-step
        // stays the workspace-appropriate one (spk new / spk lint).
        out = out.with_warning("SPECODELIC block missing or stale in AGENTS.md — run: spk init");
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
