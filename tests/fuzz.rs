//! Purpose: structure-aware fuzzer for specodelic — beads specodelic-vv8 S3.
//!
//! Responsibilities:
//! - Generate valid spec skeletons (the same shape the corpus uses:
//!   frontmatter + Constraints + Model + Properties tables), then apply
//!   SEEDED row-level mutations from a fixed menu — every mutant is a pure
//!   function of its seed, so a crash reproduces exactly.
//! - LIB LEG (default `#[ignore]`d run): drive every library entry point
//!   (`spec::parse_str`, `lint::lint_all`, `graph::build`,
//!   `compile::compile_spec`, `model_check::run`, `merge::run`,
//!   `migrate::migrate`, `verify::blocks_from_source`) over each mutant
//!   corpus in-process. A panic fails the test naturally; labeled
//!   `Result::Err` values are expected and fine.
//! - CLI LEG (`cli_survives_mutants`): write the mutant corpus to a temp
//!   dir and run the real binary (`Command::cargo_bin`) over `lint`,
//!   `graph`, `compile`, `doctor` in `--json` mode, asserting the process
//!   always (a) terminates within the timeout (a hang = failure), (b) exits
//!   with a normal exit code — never killed by a signal — and (c) prints a
//!   well-formed JSON envelope object on stdout.
//! - Generator invariants run UNGATED (cheap): the skeleton parses and
//!   lints clean, generation is deterministic per seed, and the mutation
//!   engine actually mutates.
//!
//! Rationale: the 2026-09-28 adversarial review's fuzzer (6,800 command
//! runs, 0 panics/hangs/malformed JSON) existed only outside the repo; this
//! files the harness in so it doubles as the crash-freedom acceptance gate.
//! The whole file is `#[ignore]`d at the test level (expensive: hundreds of
//! subprocess runs) — run with `cargo test --test fuzz -- --ignored`; a
//! nightly CI job can crank the iteration count via `FUZZ_ITERS` /
//! `FUZZ_CLI_ITERS`. Anti-goal: the fuzzer asserts crash-freedom and
//! envelope well-formedness ONLY — never semantic correctness, which is the
//! conformance matrix's and the oracle's job; and it never patches the
//! generator to avoid a crash instead of reporting it.

use assert_cmd::Command;
use specodelic::compile;
use specodelic::graph;
use specodelic::lint;
use specodelic::merge;
use specodelic::migrate;
use specodelic::model_check;
use specodelic::spec::{Spec, parse_str};
use specodelic::verify;
use std::path::PathBuf;
use std::time::Duration;

/// Base seed — all iteration seeds derive from it, so any crash found by a
/// nightly run reproduces locally with the same `FUZZ_*` settings.
const BASE_SEED: u64 = 0x0005_FEC0_DE11_C110;

fn lib_iters() -> u64 {
    std::env::var("FUZZ_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200)
}

fn cli_iters() -> u64 {
    std::env::var("FUZZ_CLI_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(40)
}

// ===========================================================================
// Seeded PRNG — splitmix64. No external dep; determinism is the contract.
// ===========================================================================

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed ^ BASE_SEED)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

// ===========================================================================
// Structure-aware generator
// ===========================================================================

/// One generated corpus: `k` spec files (paths relative to a dir root,
/// contents). Files are named `f0.md`, `f1.md`, … with matching ids so the
/// skeleton is `id_matches_file`-clean.
struct Corpus(Vec<(String, String)>);

/// A valid, lint-clean two-file skeleton with a cross-file link — the base
/// every mutation degrades. Shape mirrors the verified model-check fixture
/// in `tests/cli.rs` (frontmatter + Constraints + Model + Properties).
fn skeleton(files: usize) -> Corpus {
    let mut out = Vec::with_capacity(files);
    for i in 0..files {
        let id = format!("f{i}");
        // The second file's c1 link points at the first file — exercises
        // cross-file resolution in graph/lint on every mutant.
        let other = format!("f{}", (i + 1) % files);
        let text = format!(
            "---\n\
             id: {id}\n\
             kind: intent\n\
             statement: \"THE {id} SHALL be a fuzz skeleton\"\n\
             ---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | c1 | invariant | `holds` | [[{id}]] |\n\
             | c2 | invariant | `ok` | [[{other}]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- s1\n\
             - s2\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | t1 | s1 | s2 | [[{id}.c1]] |\n\
             | t2 | s2 | s1 | [[{id}.c2]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p1 | unit | [[{id}.c1]] | `g()` | `x` |\n\
             | p2 | unit | [[{id}.c2]] | `g()` | `y` |\n"
        );
        out.push((format!("{id}.md"), text));
    }
    Corpus(out)
}

// ===========================================================================
// Mutation engine — seeded row-level menu. Every mutation is a pure function
// of the rng state, so a crashing mutant reproduces from its seed alone.
// ===========================================================================

/// Garbage strings injected by the text-level mutations — unicode, control
/// chars, table/markdown delimiters, and length bombs.
const GARBAGE: &[&str] = &[
    "```",
    "|\t|",
    "\u{0}",
    "\u{7}",
    "\u{1F600} é",
    "----",
    "#### ",
    "]]",
    "[[",
    "\u{7f}",
];

/// Kind-cell replacements — closed-set violations and empties.
const KIND_JUNK: &[&str] = &["bogus", "", "constraint", "invariant invariant"];

/// Link-target replacements — dangling refs, empties, whitespace bombs.
const LINK_JUNK: &[&str] = &["nonexistent", "", " ", "f0.f0.f0", "..", "f0"];

const MUTATION_COUNT: usize = 11;

/// Mutate the corpus: pick a random file, apply 1..=3 mutations from the
/// menu. All line/byte targets come from the rng — nothing environmental.
fn mutate(rng: &mut Rng, corpus: &Corpus) -> Corpus {
    let mut files = corpus.0.clone();
    let k = 1 + rng.below(3);
    for _ in 0..k {
        let fi = rng.below(files.len());
        apply_mutation(rng.below(MUTATION_COUNT), rng, &mut files[fi].1);
    }
    Corpus(files)
}

fn lines(text: &str) -> Vec<String> {
    text.split('\n').map(str::to_string).collect()
}

fn random_line(rng: &mut Rng, text: &str) -> Option<usize> {
    let ls = lines(text);
    if ls.is_empty() {
        None
    } else {
        Some(rng.below(ls.len()))
    }
}

/// Split at a random char-boundary-safe byte offset.
fn random_split<'a>(rng: &mut Rng, text: &'a str) -> (&'a str, &'a str) {
    let mut off = rng.below(text.len() + 1);
    while off < text.len() && !text.is_char_boundary(off) {
        off += 1;
    }
    text.split_at(off)
}

fn apply_mutation(which: usize, rng: &mut Rng, text: &mut String) {
    match which {
        // 0 — truncate at a random byte offset (char-boundary-safe).
        0 => {
            let (head, _) = random_split(rng, text);
            *text = head.to_string();
        }
        // 1 — delete a random line.
        1 => {
            if let Some(i) = random_line(rng, text) {
                let mut ls = lines(text);
                ls.remove(i);
                *text = ls.join("\n");
            }
        }
        // 2 — duplicate a random line (uniqueness bait for row ids).
        2 => {
            if let Some(i) = random_line(rng, text) {
                let mut ls = lines(text);
                let dup = ls[i].to_string();
                ls.insert(i, dup);
                *text = ls.join("\n");
            }
        }
        // 3 — swap two random lines.
        3 => {
            if let (Some(a), Some(b)) = (random_line(rng, text), random_line(rng, text)) {
                let mut ls = lines(text);
                ls.swap(a, b);
                *text = ls.join("\n");
            }
        }
        // 4 — replace a random line with garbage.
        4 => {
            if let Some(i) = random_line(rng, text) {
                let mut ls = lines(text);
                ls[i] = GARBAGE[rng.below(GARBAGE.len())].to_string();
                *text = ls.join("\n");
            }
        }
        // 5 — mangle a random `[[wiki-link]]` target.
        5 => {
            let starts: Vec<usize> = text.match_indices("[[").map(|(i, _)| i).collect();
            if !starts.is_empty() {
                let s = starts[rng.below(starts.len())];
                let end = text[s..].find("]]").map(|e| s + e).unwrap_or(text.len());
                let junk = LINK_JUNK[rng.below(LINK_JUNK.len())];
                text.replace_range(s..end, &format!("[[{junk}]]"));
            }
        }
        // 6 — corrupt a kind cell (closed-set violation / empty cell).
        6 => {
            let needles = [" invariant ", " unit "];
            let hits: Vec<(usize, usize)> = needles
                .iter()
                .flat_map(|n| {
                    text.match_indices(n)
                        .map(|(i, m)| (i, m.len()))
                        .collect::<Vec<_>>()
                })
                .collect();
            if !hits.is_empty() {
                let (s, len) = hits[rng.below(hits.len())];
                let junk = KIND_JUNK[rng.below(KIND_JUNK.len())];
                text.replace_range(s..s + len, junk);
            }
        }
        // 7 — duplicate a random table row.
        7 => {
            let rows: Vec<usize> = text
                .lines()
                .enumerate()
                .filter(|(_, l)| l.starts_with("| "))
                .map(|(i, _)| i)
                .collect();
            if !rows.is_empty() {
                let i = rows[rng.below(rows.len())];
                let mut ls = lines(text);
                let dup = ls[i].to_string();
                ls.insert(i, dup);
                *text = ls.join("\n");
            }
        }
        // 8 — empty the file entirely.
        8 => {
            *text = String::new();
        }
        // 9 — append garbage after the last line.
        9 => {
            let g = GARBAGE[rng.below(GARBAGE.len())];
            text.push_str(&format!("\n{g}"));
        }
        // 10 — inject a control/unicode char at a random boundary-safe offset.
        10 => {
            let (head, tail) = random_split(rng, text);
            let g = GARBAGE[rng.below(GARBAGE.len())];
            *text = format!("{head}{g}{tail}");
        }
        _ => unreachable!("mutation menu is closed"),
    }
}

fn mutant_for_seed(seed: u64) -> Corpus {
    let base = skeleton(2);
    let mut rng = Rng::new(seed);
    mutate(&mut rng, &base)
}

// ===========================================================================
// Lib leg — every entry point over every mutant, in-process.
// ===========================================================================

/// Drive all library entry points over one mutant corpus. Parse errors and
/// labeled failures are EXPECTED; panics are the bug class under test.
/// `other` is a second mutant of the same seed family — merge's three-way
/// input (base, current, incoming) needs a real pair to diff.
fn exercise_lib(corpus: &[(String, String)], other: &[(String, String)]) {
    // Whole-text consumers: parse, migrate, verify's block scanner.
    for (_, text) in corpus {
        let _ = parse_str(text);
        let _ = migrate::migrate(text, std::path::Path::new("spec.md"));
        let _ = verify::blocks_from_source(text);
    }

    // Structured consumers: only over what parses (the CLI collects parse
    // errors and continues the same way).
    let mut specs: Vec<Spec> = Vec::new();
    for (path, text) in corpus {
        if let Ok(mut s) = parse_str(text) {
            s.path = Some(PathBuf::from(path));
            specs.push(s);
        }
    }

    let report = lint::lint_all(&specs, &[]);
    let _ = &report;
    let _ = graph::build(&specs);

    let _ = merge::run(corpus, corpus, other);

    for spec in &specs {
        // Labeled failure fine (precondition, stateless, …) — never a panic.
        if let Ok(compiled) = compile::compile_spec(spec) {
            let bound = model_check::Bound {
                max_depth: 8,
                max_states: Some(512),
                timeout_secs: Some(2),
            };
            let tla = compiled.tla.clone().into_bytes();
            let _ = model_check::run(&compiled.model_ir, &tla, &bound);
        }
    }
}

/// The fuzz loop: `n` seeded mutants through every lib entry point.
fn fuzz_lib(n: u64) {
    for i in 0..n {
        let corpus = mutant_for_seed(BASE_SEED.wrapping_add(i));
        let other = mutant_for_seed(BASE_SEED.wrapping_add(i).wrapping_mul(31).wrapping_add(7));
        exercise_lib(&corpus.0, &other.0);
    }
}

// ===========================================================================
// CLI leg — the real binary over mutant corpora on disk.
// ===========================================================================

/// Run one spk subcommand over a mutant corpus dir; assert termination,
/// signal-free exit, and a well-formed JSON envelope on stdout. `dir` is
/// `None` for arg-less subcommands (doctor).
fn cli_must_survive(
    dir: Option<&std::path::Path>,
    args: &[&str],
    out_dir: Option<&std::path::Path>,
) {
    let mut cmd = Command::cargo_bin("specodelic").expect("binary built");
    cmd.args(args)
        .arg("--json")
        .timeout(Duration::from_secs(60));
    if let Some(d) = dir {
        cmd.arg(d);
    }
    if let Some(od) = out_dir {
        cmd.arg("--out-dir").arg(od);
    }
    let out = cmd
        .output()
        .expect("CLI terminates within 60s (timeout hit = hang = bug)");
    let code = out.status.code();
    assert!(
        code.is_some(),
        "CLI killed by signal ( {:?} ): {}",
        args,
        out.status
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("malformed JSON envelope from {args:?}: {e}\n{stdout}"));
    assert!(
        v.is_object() && v.get("envelope_kind").is_some(),
        "stdout is not a spk envelope: {stdout}"
    );
}

fn fuzz_cli(n: u64) {
    for i in 0..n {
        let corpus = mutant_for_seed(BASE_SEED.wrapping_add(1_000_000 + i));
        let dir = tempfile::tempdir().expect("tempdir");
        for (path, text) in &corpus.0 {
            std::fs::write(dir.path().join(path), text).expect("write mutant");
        }
        let out = tempfile::tempdir().expect("tempdir out");
        cli_must_survive(Some(dir.path()), &["lint"], None);
        cli_must_survive(Some(dir.path()), &["graph"], None);
        cli_must_survive(Some(dir.path()), &["compile"], Some(out.path()));
        cli_must_survive(None, &["doctor"], None);
        let _ = out; // keep both temp dirs alive for the run
        drop(dir);
    }
}

// ===========================================================================
// Generator invariants — cheap, run ungated.
// ===========================================================================

#[test]
fn skeleton_parses_and_lints_clean() {
    let corpus = skeleton(2).0;
    let mut specs = Vec::new();
    for (path, text) in &corpus {
        let mut s = parse_str(text).expect("skeleton parses");
        s.path = Some(PathBuf::from(path));
        specs.push(s);
    }
    let report = lint::lint_all(&specs, &[]);
    assert!(
        report.issues.is_empty() && report.warnings.is_empty(),
        "skeleton must lint clean: {:?}",
        report.issues
    );
    let g = graph::build(&specs);
    assert!(
        g.dangling.is_empty() && g.violations.is_empty() && g.supersedes_cycles.is_empty(),
        "skeleton graph must be clean: dangling={:?} violations={:?} cycles={:?}",
        g.dangling,
        g.violations,
        g.supersedes_cycles
    );
}

#[test]
fn generation_is_deterministic_per_seed() {
    for seed in [0u64, 1, 42, 1337] {
        let a = mutant_for_seed(seed);
        let b = mutant_for_seed(seed);
        assert_eq!(a.0, b.0, "seed {seed} must regenerate byte-identically");
    }
}

#[test]
fn mutations_change_the_skeleton() {
    let base = skeleton(2).0;
    let mut changed = 0;
    for seed in 0..50u64 {
        let corpus = mutant_for_seed(seed);
        if corpus.0 != base {
            changed += 1;
        }
    }
    assert_eq!(
        changed, 50,
        "every seeded mutant must differ from the skeleton — \
         the mutation engine is a stub or degenerate"
    );
}

// ===========================================================================
// The fuzz legs — #[ignore]d: run explicitly with -- --ignored.
// ===========================================================================

#[test]
#[ignore = "expensive in-process fuzz loop — cargo test --test fuzz -- --ignored"]
fn lib_survives_mutants() {
    fuzz_lib(lib_iters());
}

#[test]
#[ignore = "spawns ~4*n CLI processes — cargo test --test fuzz -- --ignored; nightly cranks FUZZ_CLI_ITERS"]
fn cli_survives_mutants() {
    fuzz_cli(cli_iters());
}
