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
use specodelic::{acset, checklist, conform, graph, guide, model_check, spec, verify};

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

/// The graph view selection (`--view wiring`, add-graph-views task 1.6,
/// specodelic-5qj). A view selector over the shipped text projections;
/// v1 ships only the wiring view.
#[derive(clap::ValueEnum, Clone)]
enum GraphView {
    /// File-level producer→consumer projection of `constraints.satisfies`
    /// edges (specodelic-5qj)
    Wiring,
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
        /// `--view wiring` selects the wiring view over the chosen format
        /// (add-graph-views task 1.6, specodelic-5qj): the corpus's
        /// `constraints.satisfies` edges project to FILE level — each
        /// endpoint collapses to its owning intent/file, the drawn arrow
        /// follows the satisfies edge direction (consumer file → the
        /// producer's published contract, per the decision record's sample
        /// outputs), self-loops (a file satisfying its own contract) are
        /// dropped, and the remaining cross-file instances aggregate per
        /// distinct pair (the count rides as the edge label in
        /// dot/mermaid, the annotation column in the TSV). A corpus with
        /// files but zero remaining satisfies edges — including one whose
        /// satisfies edges are all self-loops — emits a labeled `no_wiring`
        /// element, never a silently clean diagram. Requires `--format`;
        /// composes with `edges`, `dot`, and `mermaid`. Overrides
        /// `--json`/
        /// `--human`: the raw text goes straight to stdout — the one
        /// documented exception to Output::emit, scoped to this flag so
        /// awk/jq pipelines consume it without an envelope parser. Zero
        /// spec files on a real path exit 0 with empty output; a
        /// nonexistent or unreadable path is an invocation error (exit 2,
        /// labeled envelope on stderr — a typo'd path is not a clean empty
        /// corpus, specodelic-0zk F3); typing violations are findings
        /// (exit 1 — matching the JSON envelope's exit, violations still
        /// riding along as annotation rows per D3, specodelic-0zk F8).
        #[arg(long)]
        format: Option<GraphFormat>,
        /// View selection over the text projections (see the format
        /// flag's help for the wiring semantics). Requires `--format` —
        /// there is no JSON wiring envelope in v1 to fall back to.
        #[arg(long, requires = "format")]
        view: Option<GraphView>,
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
    /// Evaluate an external oracle's recorded traces against a spec —
    /// READ-ONLY evaluation: conform classifies, it never generates,
    /// never writes artifacts, and never advances the lifecycle
    /// (openspec/changes/add-conform phase 4)
    Conform {
        /// The spec file to conform (exactly one)
        spec_path: String,
        /// JSONL scenario corpus from the external oracle — one
        /// {id, setup?, trace} object per line
        #[arg(long)]
        oracle: String,
        /// Declare closed-world mode: a trace nothing covers is
        /// forbidden, not underspecified — the declaration is recorded
        /// in every verdict record and the report header. Without it,
        /// absence of coverage is underspecified and never forbidden.
        #[arg(long)]
        closed_world: bool,
        /// Directory holding the compiled artifacts (must match compile's
        /// out-dir — conform never re-compiles)
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
    /// ## Requirements mirror); already-migrated files are refused.
    /// With --rekey, instead rewrite an `id: spec` dual-format file to its
    /// Revision 18 real id (naming law; idempotent no-op when already real)
    Migrate {
        /// Delta file to wrap in place (must contain ## ADDED Requirements)
        file: String,
        /// Print the resulting content without writing the file
        #[arg(long)]
        dry_run: bool,
        /// Rekey an `id: spec` dual-format file to its real parent-dir-derived id
        #[arg(long)]
        rekey: bool,
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
    /// Print the guide's closed value sets as a JSON envelope (kinds, row
    /// shapes, format revision); `--schema` selects the versioned acset
    /// Schema export instead (add-graph-views D4 — the schema view's
    /// transport interface)
    Guide {
        /// Export the versioned acset Schema payload instead of the value
        /// sets (consumed by scripts/graph_views.py's schema view)
        #[arg(long)]
        schema: bool,
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
/// note: zero spec files on a real path exit 0 with empty output.
///
/// Exit-code semantics follow the corpus-wide 0/1/2 mapping
/// (specs/errors.md `exit_code_mapping`, specodelic-0zk F3+F8): a
/// nonexistent/unreadable path is an invocation error (exit 2 — a typo'd
/// path must not read as an empty corpus), and typing violations are
/// findings (exit 1) while still riding along as annotation rows (D3).
fn cmd_graph_projection(
    paths: &[String],
    format: &GraphFormat,
    view: Option<&GraphView>,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let (specs, _checklists, _notes, _parse_errors) = parse_batch(paths, Verbosity::Quiet);
    if specs.is_empty() {
        // F3 (specodelic-0zk): a named root that never resolves is an
        // invocation error, not a clean empty corpus. `collect_specs`
        // pushes a nonexistent root through as a file, so parse_batch
        // never distinguishes it from a real empty path — check the roots
        // directly. At least one readable root keeps the parsed-not-linted
        // exit 0 (the zero_file fixture's contract).
        let roots = named_roots(paths);
        if roots.iter().all(|r| std::fs::metadata(r).is_err()) {
            // Projection stdout is the raw text channel — the labeled
            // refusal envelope routes to stderr (mirrors cmd_graph's
            // JSON-mode refusal, which exits 2 for the same input).
            let refusal: Output<()> = Output::failure(
                "no spec files found and no path resolves — the named path(s) do not exist or are unreadable",
            )
            .with_next_step("pass files or directories containing *.md specs with YAML frontmatter");
            let _ = refusal.emit(
                VERSION,
                OutputFormat::Json,
                Verbosity::Quiet,
                &mut Vec::new(),
                stderr,
            );
            return 2;
        }
        return 0;
    }
    // F8 (specodelic-0zk): violations flip the exit code like the JSON
    // envelope mode does, without changing the projection text — the
    // annotation rows still ride along (D3: a view is never cleaner than
    // the artifact).
    let violations = graph::build(&specs).violations;
    let text = match (format, view) {
        (GraphFormat::Edges, None) => graph::edges_tsv(&specs),
        (GraphFormat::Dot, None) => graph::dot_projection(&specs),
        (GraphFormat::Mermaid, None) => graph::mermaid_projection(&specs),
        (GraphFormat::Edges, Some(GraphView::Wiring)) => graph::wiring_tsv(&specs),
        (GraphFormat::Dot, Some(GraphView::Wiring)) => graph::wiring_dot(&specs),
        (GraphFormat::Mermaid, Some(GraphView::Wiring)) => graph::wiring_mermaid(&specs),
    };
    let _ = write!(stdout, "{text}");
    if violations.is_empty() { 0 } else { 1 }
}

/// `spk guide [--schema]` (add-graph-views 2.2, D4): the guide's closed
/// value sets (kinds, row shapes, format_revision) as a JSON envelope —
/// or, with `--schema`, the versioned acset Schema export that is the
/// Python schema view's sole structural input. Both emit through
/// genesis Output::emit; the reference-typing table stays out of both
/// payloads (guide typing constants are not a derivation source).
fn cmd_guide(
    schema: bool,
    format: OutputFormat,
    verbosity: Verbosity,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32 {
    let (payload, next_step, human): (serde_json::Value, &str, String) = if schema {
        let export = guide::schema_export(&acset::schema::canonical());
        let morphs = export["morphisms"].as_array().map(Vec::len).unwrap_or(0);
        let objects = export["objects"].as_array().map(Vec::len).unwrap_or(0);
        let text = format!(
            "schema export: {objects} objects, {morphs} morphisms — {} (derived from the lint-gated acset Schema; render with: scripts/graph_views.py schema <export.json>)",
            guide::FORMAT_REVISION
        );
        (
            export,
            "render the schema view: python3 scripts/graph_views.py schema <export.json>",
            text,
        )
    } else {
        (
            guide::value_set_payload(),
            "versioned schema export: specodelic guide --schema --json",
            format!(
                "guide value sets — format revision: {} (use --schema for the acset Schema export)",
                guide::FORMAT_REVISION
            ),
        )
    };
    let out = Output::success(payload).with_next_step(next_step);
    emit_report(out, Some(human), format, verbosity, stdout, stderr);
    0
}

/// The conform invocation parameters beyond the spec path — the oracle
/// corpus path, the recorded closed-world declaration, and the artifact
/// directory, grouped to keep the shared emit plumbing (cli/format/
/// verbosity/streams) within the arg lint (the CheckTarget precedent).
struct ConformTarget {
    oracle: String,
    closed_world: bool,
    out_dir: String,
}

/// `spk conform <spec-path> --oracle <scenarios.jsonl>` (add-conform
/// phase 4): the READ-ONLY external-oracle evaluation over
/// `conform::run` — the artifact gate (design D5), mechanical
/// classification (D3), and the persisted report. Exit codes follow the
/// delta's exit_code_verdict_mapping over specs/errors.md
/// `exit_code_mapping`: 0 when no verdict is forbidden or unsupported
/// (unknown/underspecified never fail — their counts surface in both
/// views), 1 when any forbidden/unsupported verdict fails the run (an
/// error-kind envelope — the published ok:false contract, envelope_error_kind),
/// 2 on invocation errors and gate refusals — with ZERO verdict records
/// emitted from any refusal path.
fn cmd_conform(
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
        let mut out: Output<serde_json::Value> =
            Output::failure("the spec file failed to parse — verdicts are never derived from an unparsed file");
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
        out = out.with_next_step(
            "pass exactly one spec file (*.md with YAML frontmatter)",
        );
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
    match conform::run(
        &specs[0],
        std::path::Path::new(&out_dir),
        &corpus_bytes,
        closed_world,
    ) {
        Ok(report) => {
            let forbidden = report.verdict_counts.get("forbidden").copied().unwrap_or(0);
            let unsupported = report
                .verdict_counts
                .get("unsupported")
                .copied()
                .unwrap_or(0);
            if forbidden + unsupported > 0 {
                // Findings-or-failure (exit 1): the error-kind envelope
                // is the published contract (specs/errors.md
                // envelope_error_kind) — a failed conformance run never
                // rides a success-shaped envelope. The human view is the
                // same one the success branch renders.
                let out = commands::as_failure(
                    Output::success(report.clone()),
                    format!(
                        "{forbidden} forbidden and {unsupported} unsupported verdict(s) — the oracle disagrees with the declared model"
                    ),
                );
                emit_report(
                    out,
                    Some(conform::human_view(&report)),
                    format,
                    verbosity,
                    stdout,
                    stderr,
                );
                1
            } else {
                conform::emit_report(&report, format, verbosity, stdout, stderr).ok();
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
            let out: Output<serde_json::Value> =
                Output::failure(format!("{}: {}", e.stage, e.message)).with_next_step(hint);
            emit_report(out, None, format, verbosity, stdout, stderr);
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
        Commands::Parse { file } => cmd_parse(file, format, verbosity, stdout, stderr),
        Commands::Graph {
            paths,
            format: Some(format),
            view,
        } => cmd_graph_projection(paths, format, view.as_ref(), stdout, stderr),
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
        Commands::Conform {
            spec_path,
            oracle,
            closed_world,
            out_dir,
        } => cmd_conform(
            spec_path,
            ConformTarget {
                oracle: oracle.clone(),
                closed_world: *closed_world,
                out_dir: out_dir.clone(),
            },
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
        Commands::Migrate {
            file,
            dry_run,
            rekey,
        } => cmd_migrate(file, *dry_run, *rekey, format, verbosity, stdout, stderr),
        Commands::Explain { topic } => {
            cmd_explain(topic.as_deref(), format, verbosity, stdout, stderr)
        }
        Commands::Guide { schema } => cmd_guide(*schema, format, verbosity, stdout, stderr),
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

/// The named roots of a batch: the paths as given, defaulting to
/// `./specs` (shared by collect_specs and the projection exit-code
/// check — specodelic-0zk).
pub(crate) fn named_roots(paths: &[String]) -> Vec<std::path::PathBuf> {
    if paths.is_empty() {
        vec!["specs".into()]
    } else {
        paths.iter().map(std::path::PathBuf::from).collect()
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
    let roots = named_roots(paths);
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
