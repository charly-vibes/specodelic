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
use genesis::cli::{generate_completions, maybe_print_version_json};
use genesis::guide::{CliFormat, CliVerbosity, Output, OutputFormat, Verbosity};

use specodelic::spec::Spec;
use specodelic::{graph, lint, spec};

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
    /// Translate a linted spec file into TOML / TLA+ / proptest artifacts
    Compile {
        /// Spec files to compile
        paths: Vec<String>,
    },
    /// Run the model checker (stateright default, TLC opt-in) against compiled output
    ModelCheck {
        /// Spec files whose compiled module to check
        paths: Vec<String>,
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
    /// Diagnose the Specodelic workspace setup
    Doctor,
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

fn main() {
    // Handle `--version --json` before normal parsing (genesis convention).
    if maybe_print_version_json("specodelic", VERSION) {
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
        Commands::Compile { .. } | Commands::ModelCheck { .. } | Commands::Verify { .. } => {
            let out: Output<serde_json::Value> =
                Output::failure("not yet implemented — the pipeline is specced in specs/compile.md, specs/model_check.md, and specs/verify.md")
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
        Commands::Doctor => cmd_doctor(cli, format, verbosity, stdout, stderr),
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

/// Collect spec files from paths (files, or directories ending in `.md`,
/// skipping known non-spec files: no frontmatter).
fn collect_specs(paths: &[String]) -> Vec<std::path::PathBuf> {
    let roots: Vec<std::path::PathBuf> = if paths.is_empty() {
        vec!["specs".into()]
    } else {
        paths.iter().map(std::path::PathBuf::from).collect()
    };
    let mut files = vec![];
    for root in roots {
        if root.is_dir() {
            let mut dir_files: Vec<_> = std::fs::read_dir(&root)
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == "md"))
                .collect();
            dir_files.sort();
            files.extend(dir_files);
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
    if specs.is_empty() && notes.iter().any(|n| n.contains("parse error")) {
        let out: Output<serde_json::Value> = Output::failure("spec files failed to parse")
            .with_next_step("fix the frontmatter/tables reported above");
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
    let template = format!(
        "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL …\"\n---\n\n# {}\n\nOne paragraph of prose: why this exists, what it's not.\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n\n## Model\n\n### States\n\n- `initial`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | initial | initial | TODO — cite a Constraint by [[id]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n",
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
    let mut checks: Vec<(String, String)> = vec![];
    checks.push((
        "specs/ directory".into(),
        if std::path::Path::new("specs").is_dir() {
            "ok".into()
        } else {
            "missing — no spec corpus found".into()
        },
    ));
    let core = std::path::Path::new("specs/specodelic.md");
    checks.push((
        "core format spec".into(),
        if core.is_file() {
            "ok (specs/specodelic.md)".into()
        } else {
            "missing — expected specs/specodelic.md".into()
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
    let payload = serde_json::json!({ "checks": checks });
    let out = Output::success(payload).with_next_step("run: specodelic lint");
    emit(&out, cli, format, verbosity, stdout, stderr);
    0
}
