// (split from tests/cli.rs — specodelic-g17 file_lines ratchet)
use super::*;

// ---- specodelic-suz: hostile-input ingestion hardening ----

/// A FIFO named `*.md` must be rejected with a labeled message inside the
/// timeout — never ingested (a blocking read would hang the command
/// forever, reproduced 2026-09-28 with `mkfifo /tmp/x.md`).
#[cfg(unix)]
#[test]
fn lint_on_a_fifo_is_rejected_never_blocks() {
    let dir = tempfile::tempdir().unwrap();
    let fifo = dir.path().join("pipe.md");
    std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .unwrap()
        .success()
        .then_some(())
        .expect("mkfifo must succeed");
    let out = spk()
        .args(["lint", fifo.to_str().unwrap(), "--json"])
        .timeout(std::time::Duration::from_secs(10))
        .output()
        .expect("lint must finish — a FIFO must never block the read");
    // A rejected input is a labeled failure when nothing else was linted.
    assert!(!out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("not a regular file"),
        "labeled rejection required: {stdout}"
    );
    assert!(
        stdout.contains("pipe.md"),
        "the offending path must be named: {stdout}"
    );
}

/// A char device (`/dev/zero`-class) must never be ingested — the
/// unbounded read OOM-kills (confirmed mechanism, tested under ulimit).
#[cfg(unix)]
#[test]
fn lint_on_a_char_device_is_rejected_never_reads_unbounded() {
    let out = spk()
        .args(["lint", "/dev/zero", "--json"])
        .timeout(std::time::Duration::from_secs(10))
        .output()
        .expect("lint must finish — a device file must never be read unbounded");
    // A rejected input is a labeled failure when nothing else was linted.
    assert!(!out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("not a regular file"),
        "labeled rejection required: {stdout}"
    );
}

/// A regular file over the 2 MiB input cap (corpus files are ~10-50 KB)
/// must be rejected with a labeled message naming the cap — never read
/// unbounded into memory.
#[test]
fn lint_on_an_oversized_file_names_the_cap() {
    let dir = tempfile::tempdir().unwrap();
    let big = dir.path().join("big.md");
    let mut content = String::from(
        "---\nid: big\nkind: intent\nstatement: \"THE system SHALL be oversized\"\n---\n\n",
    );
    content.push_str(&"x".repeat(3 * 1024 * 1024));
    std::fs::write(&big, content).unwrap();
    let out = spk()
        .args(["lint", big.to_str().unwrap(), "--json"])
        .timeout(std::time::Duration::from_secs(10))
        .output()
        .unwrap();
    // A rejected input is a labeled failure when nothing else was linted.
    assert!(!out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("2 MiB"), "the cap must be named: {stdout}");
    assert!(
        stdout.contains("big.md"),
        "the offending path must be named: {stdout}"
    );
}

/// The hostile-input note surfaces on EVERY verb's empty failure path,
/// not just lint/graph — a rejected input must name itself wherever the
/// batch comes up empty (specodelic-suz Ro5 follow-up, CORR-001).
#[test]
fn hostile_note_rides_the_empty_failure_envelope_of_every_verb() {
    let dir = tempfile::tempdir().unwrap();
    let big = dir.path().join("big.md");
    let mut content = String::from(
        "---\nid: big\nkind: intent\nstatement: \"THE system SHALL be oversized\"\n---\n\n",
    );
    content.push_str(&"x".repeat(3 * 1024 * 1024));
    std::fs::write(&big, content).unwrap();
    for verb in ["compile", "model-check", "verify"] {
        let out = spk()
            .args([verb, big.to_str().unwrap(), "--json"])
            .timeout(std::time::Duration::from_secs(10))
            .output()
            .unwrap();
        assert!(!out.status.success(), "{verb}: must fail on zero specs");
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("2 MiB"),
            "{verb}: the cap must be named: {stdout}"
        );
        assert!(
            stdout.contains("big.md"),
            "{verb}: the offending path must be named: {stdout}"
        );
    }
}

/// A FIFO alongside a valid spec is skipped with a labeled note — the
/// valid spec still lints.
#[cfg(unix)]
#[test]
fn lint_skips_a_fifo_alongside_valid_specs_with_a_note() {
    let dir = tempfile::tempdir().unwrap();
    write_model_check_spec(&dir.path().join("ok.md"), "ok");
    let fifo = dir.path().join("pipe.md");
    std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .unwrap()
        .success()
        .then_some(())
        .expect("mkfifo must succeed");
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .timeout(std::time::Duration::from_secs(10))
        .output()
        .expect("lint must finish — a FIFO must never block the read");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("not a regular file") && stdout.contains("pipe.md"),
        "the FIFO is labeled and skipped: {stdout}"
    );
    assert!(
        stdout.contains("\"issues\":[]"),
        "no lint issues — the valid spec lints clean: {stdout}"
    );
    assert!(
        stdout.contains("\"files_linted\":1"),
        "the valid spec was ingested and linted: {stdout}"
    );
}

// ---- specodelic-7rr: output contract (exit codes, human formatters, ok:false) ----

/// Debug-rendering markers that must never appear in `--human` stdout:
/// the report verbs render real human text (per-verb formatters);
/// Rust's `{:?}` Debug dump stays internal (specodelic-7rr item 3).
fn assert_human_text(stdout: &str, context: &str) {
    for marker in [
        "Object {",
        "Report {",
        "Issue {",
        "GraphReport {",
        "String(",
    ] {
        assert!(
            !stdout.contains(marker),
            "{context}: --human must render human text, not Rust Debug — found {marker:?} in:\n{stdout}"
        );
    }
}

#[test]
#[ignore = "genesis-r13: Output::to_envelope must serialize ok:false on errors — activates when the fixed genesis (post-0.8.1) lands in Cargo.toml"]
fn error_envelope_serializes_ok_false() {
    // An invocation error must never read {"ok":true,"envelope_kind":"error"}
    // — machine consumers gate on `ok`.
    let dir = tempfile::tempdir().unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["ok"], serde_json::json!(false), "envelope: {json}");
    assert_eq!(json["envelope_kind"], serde_json::json!("error"));
}

#[test]
fn invocation_errors_exit_two() {
    // Exit-code contract (specodelic-7rr item 2, CLARITY-pinned):
    // 0 = success; 1 = findings/tool-level failure; 2 = invocation error
    // (nothing processed: path not found, no spec files matched).
    // Consumers must distinguish "your spec is bad" (1) from "you typoed
    // the path" (2).
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("README.md"), "# not a spec\n").unwrap();
    for verb in ["lint", "graph", "compile", "model-check", "verify"] {
        let out = spk()
            .args([verb, dir.path().to_str().unwrap(), "--json"])
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(2),
            "{verb} on a directory with no spec files must exit 2 (invocation error), got {:?}",
            out.status.code()
        );
        // ...and the envelope must not silently claim success either way.
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert!(
            stdout.contains("error"),
            "{verb}: envelope_kind must be error on an invocation error: {stdout}"
        );
    }
}

#[test]
fn lint_findings_exit_one() {
    // Findings are a tool-level failure (exit 1), distinct from the
    // invocation error (exit 2).
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad_spec.md");
    std::fs::write(
        &bad,
        "---\nid: bad_spec\nkind: intent\nstatement: \"the system should maybe work\"\n---\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn help_documents_exit_codes() {
    let out = spk().arg("--help").output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("Exit codes") && stdout.contains("invocation error"),
        "--help must document the exit-code mapping:\n{stdout}"
    );
}

#[test]
fn human_lint_is_text_not_debug() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad_spec.md");
    std::fs::write(
        &bad,
        "---\nid: bad_spec\nkind: intent\nstatement: \"the system should maybe work\"\n---\n",
    )
    .unwrap();
    let out = spk()
        .args(["lint", dir.path().to_str().unwrap(), "--human"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_human_text(&stdout, "lint");
    assert!(
        stdout.contains("linter.ears_syntax"),
        "findings carry the rule id: {stdout}"
    );
    assert!(
        stdout.contains("finding"),
        "a summary line names the counts: {stdout}"
    );
}

#[test]
fn human_graph_is_text_not_debug() {
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("gdemo.md");
    write_model_check_spec(&spec, "gdemo");
    let out = spk()
        .args(["graph", dir.path().to_str().unwrap(), "--human"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_human_text(&stdout, "graph");
    assert!(
        stdout.contains("dangling"),
        "the summary names dangling count: {stdout}"
    );
}

#[test]
fn human_compile_is_text_not_debug() {
    let td = tempfile::tempdir().unwrap();
    let (spec, out_dir) = compile_fixture(&td, "hcomp", "        let _ = v0;");
    let out = spk()
        .args(["compile", &spec, "--human", "--out-dir", &out_dir])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_human_text(&stdout, "compile");
    assert!(
        stdout.contains("compiled"),
        "summary names compiled count: {stdout}"
    );
}

#[test]
fn human_model_check_is_text_not_debug() {
    let td = tempfile::tempdir().unwrap();
    let (spec, out_dir) = compile_fixture(&td, "hmc", "        let _ = v0;");
    let out = spk()
        .args(["model-check", &spec, "--human", "--out-dir", &out_dir])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_human_text(&stdout, "model-check");
    assert!(
        stdout.contains("exploration_only") || stdout.contains("outcome"),
        "the run outcome appears in human text: {stdout}"
    );
}

#[test]
fn human_verify_is_text_not_debug() {
    let td = tempfile::tempdir().unwrap();
    let (spec, out_dir) = compile_fixture(&td, "hver", "        let _ = v0;");
    let out = spk()
        .args(["verify", &spec, "--human", "--out-dir", &out_dir])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_human_text(&stdout, "verify");
    assert!(
        stdout.contains("verified") || stdout.contains("blocked"),
        "the verdict appears in human text: {stdout}"
    );
}

#[test]
fn human_doctor_is_text_not_debug() {
    let out = spk().args(["doctor", "--human"]).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_human_text(&stdout, "doctor");
    assert!(
        stdout.contains("mode"),
        "doctor renders its checks as lines: {stdout}"
    );
}

// ---- specodelic-vpx: docs drift (STATUS revision, USAGE quick-start lint-clean) ----

#[test]
fn usage_quick_start_example_is_lint_clean() {
    // The §1 quick-start is a new user's first copy-paste — it must pass
    // the tool's own lint (specodelic-vpx): filename instruction present
    // (id_matches_file) and every constraint has a deriving property
    // (coverage).
    let usage = std::fs::read_to_string("specs/USAGE.md")
        .unwrap()
        // Windows runners check out CRLF (Git for Windows autocrlf);
        // normalize so the fence search below is line-ending-agnostic.
        .replace("\r\n", "\n");
    let start = usage
        .find("## 1. Quick start")
        .expect("USAGE §1 quick-start exists");
    let block_start = usage[start..]
        .find("```markdown\n")
        .expect("quick-start has a markdown block")
        + start
        + "```markdown\n".len();
    let block_end = usage[block_start..].find("\n```").expect("block is closed") + block_start;
    let body = &usage[block_start..block_end];

    // The example must name its file (the id↔filename law): order.cancel
    // lives in order-cancel.md.
    assert!(
        usage[start..block_start].contains("order-cancel.md"),
        "the quick-start prose must tell the reader to save the spec as order-cancel.md (id_matches_file law)"
    );

    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("order-cancel.md");
    std::fs::write(&spec, body).unwrap();
    let out = spk()
        .args(["lint", spec.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "quick-start spec must lint clean: {stdout}"
    );
}

// ---- specodelic-7rr residuals: explain/new/hooks human text + exit codes ----

#[test]
fn explain_unknown_topic_exits_two() {
    // Unknown topic = invalid invocation (clap argument errors also exit
    // 2) — distinct from tool-level failure (1).
    let out = spk()
        .args(["explain", "nosuchtopic", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn new_existing_file_exits_two() {
    // Overwriting an existing file is refused — an invocation error, not
    // a tool-level failure.
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("demo-thing.md");
    std::fs::write(&file, "exists").unwrap();
    let out = spk()
        .args([
            "new",
            "demo.thing",
            "--file",
            file.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn human_explain_lists_topics_not_debug() {
    let out = spk().args(["explain", "--human"]).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_human_text(&stdout, "explain list");
    assert!(
        stdout.contains("dual-format"),
        "the topic list renders each topic: {stdout}"
    );
}

#[test]
fn human_explain_unknown_topic_has_no_debug_junk() {
    let out = spk()
        .args(["explain", "nosuchtopic", "--human"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_human_text(&stdout, "explain unknown");
    assert!(
        !stdout.contains('"'),
        "no quoted Debug strings on stdout — the message rides stderr: {stdout:?}"
    );
}

#[test]
fn human_new_reports_unquoted() {
    let dir = tempfile::tempdir().unwrap();
    let out = spk()
        .args([
            "new",
            "demo.thing",
            "--file",
            &format!("{}/demo-thing.md", dir.path().display()),
            "--human",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("created"), "{stdout}");
    assert!(
        !stdout.contains('"'),
        "no quoted Debug string on stdout: {stdout:?}"
    );
}

#[test]
fn human_hooks_uninstall_renders_text() {
    let dir = hooks_fixture(true, true);
    let out = spk()
        .args(["hooks", "uninstall", "--human"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_human_text(&stdout, "hooks uninstall");
    assert!(
        stdout.contains("unwired") || stdout.contains("not_wired"),
        "the outcome appears as human text: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// rename (specodelic-ams, specs/rename.md)
// ---------------------------------------------------------------------------

/// Two-file fixture: `alpha` defines constraint `c1` (with a deriving
/// property and rationale prose that mentions "alpha" in words);
/// `beta` references `[[alpha.c1]]` cross-file (a Property's derives_from —
/// the typing-valid cross-file row ref: a Constraint's traces_to may only
/// target an Intent per the Reference Typing table).
fn write_rename_fixture(dir: &tempfile::TempDir) {
    std::fs::write(
        dir.path().join("alpha.md"),
        "---\nid: alpha\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n\
         ## Constraints\n\n\
         | id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | c1 | invariant | `x holds` | [[alpha]] |\n\n\
         Rationale: alpha is the root because alpha anchors the model.\n\n\
         ## Properties\n\n\
         | id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|-----------|\n\
         | p1 | unit | [[alpha.c1]] | `arbitrary_row()` | `check(x) == ok` |\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("beta.md"),
        "---\nid: beta\nkind: intent\nstatement: \"THE system SHALL reference\"\n---\n\n\
         ## Constraints\n\n\
         | id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | d1 | invariant | `z holds` | [[alpha]] |\n\n\
         ## Properties\n\n\
         | id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|-----------|\n\
         | pb1 | unit | [[alpha.c1]] | `arbitrary_row()` | `check(z) == ok` |\n",
    )
    .unwrap();
}

/// clean_rename_passes + old_id_fully_replaced + prose_mention_left_alone:
/// renaming a row rewrites the definition cell and every [[link]] (including
/// derives_from, cross-file), leaves zero old-id occurrences, and never
/// touches prose that mentions the old id in words.
#[test]
fn rename_of_a_row_rewrites_definition_and_all_refs() {
    let dir = tempfile::tempdir().unwrap();
    write_rename_fixture(&dir);
    let out = spk()
        .args([
            "rename",
            "alpha.c1",
            "alpha.c1_renamed",
            dir.path().join("alpha.md").to_str().unwrap(),
            dir.path().join("beta.md").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "clean rename must pass: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let alpha = std::fs::read_to_string(dir.path().join("alpha.md")).unwrap();
    let beta = std::fs::read_to_string(dir.path().join("beta.md")).unwrap();
    assert!(
        alpha.contains("| c1_renamed |"),
        "definition cell rewritten: {alpha}"
    );
    assert!(
        alpha.contains("[[alpha.c1_renamed]]"),
        "derives_from rewritten: {alpha}"
    );
    assert!(
        !alpha.contains("[[alpha.c1]]"),
        "zero old refs remain: {alpha}"
    );
    assert!(
        beta.contains("[[alpha.c1_renamed]]"),
        "cross-file ref rewritten: {beta}"
    );
    assert!(
        alpha.contains("Rationale: alpha is the root because alpha anchors"),
        "prose untouched: {alpha}"
    );
    // The renamed repo passes both verify-gate linters (referential_integrity
    // + graph_shape): zero dangling.
    let graph = spk()
        .args([
            "graph",
            dir.path().join("alpha.md").to_str().unwrap(),
            dir.path().join("beta.md").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        graph.status.success()
            && !String::from_utf8_lossy(&graph.stdout).contains("\"dangling\":[{"),
        "post-rename graph has zero dangling refs: {}",
        String::from_utf8_lossy(&graph.stdout)
    );
}

/// new_id_matches_filename: renaming a file's own Intent id also renames
/// the file per the naming law (`-` ⇔ `.`), and cross-file refs follow.
#[test]
fn rename_of_a_files_intent_id_renames_the_file() {
    let dir = tempfile::tempdir().unwrap();
    write_rename_fixture(&dir);
    let out = spk()
        .args([
            "rename",
            "alpha",
            "alpha.prime",
            dir.path().join("alpha.md").to_str().unwrap(),
            dir.path().join("beta.md").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "intent rename must pass: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!dir.path().join("alpha.md").exists(), "old filename gone");
    let renamed = std::fs::read_to_string(dir.path().join("alpha-prime.md")).unwrap();
    assert!(
        renamed.starts_with("---\nid: alpha.prime\n"),
        "frontmatter id rewritten: {renamed}"
    );
    let beta = std::fs::read_to_string(dir.path().join("beta.md")).unwrap();
    assert!(
        beta.contains("[[alpha.prime.c1]]"),
        "child refs follow the rename: {beta}"
    );
}

/// new_id_available + atomic_operation: a collision is a labeled failure
/// and every file is byte-identical to its pre-rename state.
#[test]
fn rename_to_an_existing_id_is_rejected_and_leaves_the_repo_byte_identical() {
    let dir = tempfile::tempdir().unwrap();
    write_rename_fixture(&dir);
    let before: Vec<_> = ["alpha.md", "beta.md"]
        .iter()
        .map(|f| (f, std::fs::read(dir.path().join(f)).unwrap()))
        .collect();
    let out = spk()
        .args([
            "rename",
            "alpha.c1",
            "p1", // collides with the property row id
            dir.path().join("alpha.md").to_str().unwrap(),
            dir.path().join("beta.md").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success(), "collision must fail");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("p1"),
        "the collision must be named: {stdout}"
    );
    for (f, bytes) in before {
        assert_eq!(
            std::fs::read(dir.path().join(f)).unwrap(),
            bytes,
            "{f} must be byte-identical after a failed rename"
        );
    }
}

/// An unknown old_id is a labeled failure — nothing is written.
#[test]
fn rename_of_an_unknown_id_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    write_rename_fixture(&dir);
    let out = spk()
        .args([
            "rename",
            "nope.such",
            "whatever",
            dir.path().join("alpha.md").to_str().unwrap(),
            dir.path().join("beta.md").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success(), "unknown id must fail");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("nope.such"),
        "the unknown id must be named: {stdout}"
    );
}
