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
use specodelic::{checklist, graph, guide, model_check, spec, verify};

mod commands;

use commands::corpus::{cmd_compile, cmd_graph, cmd_lint, cmd_merge, cmd_parse, cmd_rename};
use commands::manage::{
    cmd_archive_companion, cmd_doctor, cmd_explain, cmd_feedback, cmd_hooks, cmd_init, cmd_migrate,
    cmd_new, cmd_refactor, exit_code_footer,
};
use commands::model_check::{
    CheckTarget, OrchestrateTarget, cmd_model_check, cmd_orchestrate, cmd_verify,
};

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

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
pub(crate) enum ModelBackend {
    /// Embedded stateright BFS exploration (default — exhaustive runs
    /// report exploration_only, never no_counterexample)
    Stateright,
    /// The TLA+ TLC reference engine — JVM subprocess over the compiled
    /// module; requires --tlc-jar
    Tlc,
}

/// The graph output projection selection (`--format edges|dot|mermaid`,
/// add-graph-views D8).
#[derive(clap::ValueEnum, Clone)]
enum GraphFormat {
    /// Raw six-column TSV edge list (add-graph-views D2/D3)
    Edges,
    /// Graphviz DOT text projection — visual grammar: solid = state
    /// machine, dashed = guards, bold = `emits`, dotted = traceability,
    /// red dashed = dangling/violations (add-graph-views task 1.5, D8)
    Dot,
    /// Mermaid flowchart text projection — same visual grammar via arrow
    /// types and `linkStyle` (add-graph-views task 1.5, D8)
    Mermaid,
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
        /// Output projection instead of the JSON envelope: `edges` writes
        /// a sorted six-column TSV to stdout — columns: source_id,
        /// source_kind, field, target_id, target_kind, annotation.
        /// Endpoints are canonical node ids (intent ids, qualified row
        /// ids); display labels are normalized away (add-graph-views D2).
        /// Every recorded edge is one row (multiplicity preserved); every
        /// typing violation is one annotation row (empty source id/kind,
        /// `violation:<edge_kind>` in the field column, the full reason
        /// text — tab-escaped — in the annotation column, per D3's open
        /// question decided for full reasons). `dot`/`mermaid` write
        /// plain-text graph projections (byte-stable re-runs; visual
        /// grammar: solid = state machine, dashed = guards, bold =
        /// `emits`, dotted = traceability, red dashed =
        /// dangling/violations — violations and dangling references are
        /// annotated elements, never silently cleaned, D3). Rendering
        /// stays external (D8): pipe into `dot -Tsvg`, `graph-easy`, or
        /// paste the mermaid into an `mmdc`/viz-js target — no `--render`.
        /// Overrides `--json`/
        /// `--human`: the raw text goes straight to stdout — the one
        /// documented exception to Output::emit, scoped to this flag so
        /// awk/jq pipelines consume it without an envelope parser. Zero
        /// spec files exit 0 with empty output.
        #[arg(long)]
        format: Option<GraphFormat>,
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
        /// Wall-clock bound in seconds on the verify runner's cargo test
        /// run — a hanging predicate is killed and reported as a labeled
        /// timeout, never a silent hang. 0 runs unbounded.
        #[arg(long, default_value_t = verify::DEFAULT_VERIFY_TIMEOUT_SECS)]
        timeout_secs: u64,
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
        /// Fan-in count that counts as "high" (threshold_is_per_repo_setting:
        /// configured per invocation, never a number fixed in specs/refactor.md)
        #[arg(long)]
        high_fan_in: Option<usize>,
        /// Comma-separated row ids a proposed changeset would touch, enabling
        /// the narrow_diff_heuristic (strict-subset, no-coherent-dependency flag)
        #[arg(long)]
        changeset: Option<String>,
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
    /// Run the full pipeline: lint (Checker Ownership order) → compile
    /// → model_check → verify, gating each stage exactly as
    /// specs/orchestrate.md specifies (rename is never part of a run)
    Orchestrate {
        /// Files or directories (defaults to ./specs)
        paths: Vec<String>,
        /// Directory for the compiled artifacts (default `specodelic/`,
        /// committed — byte-stable output makes reruns diff-visible)
        #[arg(long, default_value = "specodelic")]
        out_dir: String,
        /// Stated depth bound for the model_check stage
        #[arg(long, default_value_t = 100)]
        max_depth: u32,
        /// Optional state-count budget for the model_check stage
        #[arg(long)]
        max_states: Option<u64>,
        /// Optional wall-clock budget (seconds) for the model_check stage
        #[arg(long)]
        timeout_secs: Option<u64>,
        /// Model-check backend (specs/model_check.md: stateright embedded
        /// default; tlc = the JVM reference engine over the compiled .tla)
        #[arg(long, value_enum, default_value_t = ModelBackend::Stateright)]
        backend: ModelBackend,
        /// Path to tla2tools.jar (required for --backend tlc)
        #[arg(long)]
        tlc_jar: Option<String>,
    },
    /// Wrap an existing openspec delta file in place into the dual-format
    /// four-layer skeleton (frontmatter + scaffold layers + byte-identical
    /// ## Requirements mirror); already-migrated files are refused
    Migrate {
        /// Delta file to wrap in place (must contain ## ADDED Requirements)
        file: String,
        /// Print the resulting content without writing the file
        #[arg(long)]
        dry_run: bool,
    },
    /// Scaffold a new spec file from the four-layer template
    New {
        /// Intent id for the new spec (e.g. order.cancel)
        id: String,
        /// Target file (defaults to <id with . → ->.md)
        #[arg(short, long)]
        file: Option<String>,
    },
    /// Export the parsed Spec IR for one file as a json envelope
    /// (syntax-only — succeeds independently of lint status; no lint
    /// status is embedded; exactly one file per invocation)
    Parse {
        /// Spec file to parse (exactly one)
        file: String,
    },
    /// Explain the Specodelic format — embedded guide, works offline
    Explain {
        /// Topic to explain; omit to list the topics
        topic: Option<String>,
    },
    /// Diagnose the Specodelic workspace setup
    Doctor {
        /// Auto-repair what the doctor can repair (the SPECODELIC block
        /// in AGENTS.md); repaired checks are verified after the fix
        #[arg(short, long)]
        fix: bool,
    },
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
    /// Archive an openspec change while preserving its dual-format
    /// layer (openspec archive --skip-specs + verbatim deploy of the
    /// archived deltas; specodelic-fzo / GH#7)
    ArchiveCompanion {
        /// The openspec change id to archive
        change_id: String,
        /// Resolve and verify the restore plan without invoking openspec
        /// or writing anything
        #[arg(long)]
        dry_run: bool,
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

/// `graph --format <projection>` (add-graph-views D2/D3/D8): the raw
/// projection text straight to stdout — the one documented exception to
/// Output::emit, scoped to this flag. The transform's scope gate
/// (intentless corpora) is the Python layer's beat; the tool-level
/// projection stays well-formed per specs/graph.md's parsed-not-linted
/// note: zero spec files exit 0 with empty output.
fn cmd_graph_projection(
    paths: &[String],
    format: &GraphFormat,
    stdout: &mut impl std::io::Write,
) -> i32 {
    let (specs, _checklists, _notes, _parse_errors) = parse_batch(paths, Verbosity::Quiet);
    if specs.is_empty() {
        return 0;
    }
    let text = match format {
        GraphFormat::Edges => graph::edges_tsv(&specs),
        GraphFormat::Dot => graph::dot_projection(&specs),
        GraphFormat::Mermaid => graph::mermaid_projection(&specs),
    };
    let _ = write!(stdout, "{text}");
    0
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
        Commands::Parse { file } => cmd_parse(file, format, verbosity, stdout, stderr),
        Commands::Graph {
            paths,
            format: Some(format),
        } => cmd_graph_projection(paths, format, stdout),
        Commands::Graph { paths, .. } => cmd_graph(paths, format, verbosity, stdout, stderr),
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
        Commands::Verify {
            paths,
            out_dir,
            timeout_secs,
        } => cmd_verify(
            paths,
            out_dir,
            *timeout_secs,
            format,
            verbosity,
            stdout,
            stderr,
        ),
        Commands::Rename {
            old_id,
            new_id,
            paths,
        } => cmd_rename(old_id, new_id, paths, format, verbosity, stdout, stderr),
        Commands::Refactor {
            paths,
            high_fan_in,
            changeset,
        } => cmd_refactor(
            paths,
            *high_fan_in,
            changeset.as_deref(),
            format,
            verbosity,
            stdout,
            stderr,
        ),
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
        Commands::Orchestrate {
            paths,
            out_dir,
            max_depth,
            max_states,
            timeout_secs,
            backend,
            tlc_jar,
        } => cmd_orchestrate(
            paths,
            OrchestrateTarget {
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
        Commands::New { id, file } => {
            cmd_new(id, file.as_deref(), format, verbosity, stdout, stderr)
        }
        Commands::Migrate { file, dry_run } => {
            cmd_migrate(file, *dry_run, format, verbosity, stdout, stderr)
        }
        Commands::Explain { topic } => {
            cmd_explain(topic.as_deref(), format, verbosity, stdout, stderr)
        }
        Commands::Doctor { fix } => cmd_doctor(*fix, format, verbosity, stdout, stderr),
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
        Commands::ArchiveCompanion { change_id, dry_run } => {
            cmd_archive_companion(change_id, *dry_run, format, verbosity, stdout, stderr)
        }
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
pub(crate) fn collect_specs(paths: &[String]) -> Vec<std::path::PathBuf> {
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
pub(crate) const MAX_INPUT_BYTES: u64 = 2 * 1024 * 1024;

/// Corpus discovery (gh#2.2, specodelic-ag5): consumer repos keep their
/// specs under an openspec/ tree (openspec/changes/*/specs/…), while bare
/// `spk lint`/`spk graph` scan `specs/` — so the natural invocation finds
/// nothing and the working one is undiscoverable. When cwd carries an
/// openspec/ tree, the zero-files failures and doctor name it instead of
/// leaving trial-and-error.
pub(crate) fn openspec_tree_present() -> bool {
    std::path::Path::new("openspec").is_dir()
}

/// Parse a batch, skipping non-spec files (no frontmatter) with a note.
/// A `*.checklist.md` file is not a spec — it's the external-completeness
/// manifest (specs/linter-external_completeness.md, mp1 row 10 decision):
/// parsed leniently into [`checklist::Checklist`] and returned alongside
/// the specs for `spk lint`'s optional non-gating pass. Other commands
/// ignore checklists (external_completeness never gates a stage).
/// The batch parse outcome: specs, declared checklists, labeled skip
/// notes, and STRUCTURED parse errors (Ro5 CORR-001 over
/// specodelic-8kk — gates read the fourth element, never note prose;
/// a file whose path contains "parse error" must not flip a gate).
pub(crate) fn parse_batch(
    paths: &[String],
    verbosity: Verbosity,
) -> (
    Vec<Spec>,
    Vec<checklist::Checklist>,
    Vec<String>,
    Vec<String>,
) {
    let mut specs = vec![];
    let mut checklists = vec![];
    let mut notes = vec![];
    let mut parse_errors: Vec<String> = vec![];
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
            Err(e) => parse_errors.push(format!("{}: parse error: {e}", f.display())),
        }
    }
    let _ = verbosity;
    (specs, checklists, notes, parse_errors)
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
