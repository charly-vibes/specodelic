//! Python kernel agreement — D5 of the parked add-py-fragment-emission
//! design, promoted as its own slice (specodelic-y5x8; tasks 4.1-4.2).
//!
//! Purpose: pin a Python reference evaluator for the closed kernel
//! vocabulary against the same frozen corpus the Rust agreement test
//! reads (`tests/common/kernel_fixtures.rs`), so the production kernel
//! evaluator is cross-checked by an implementation that shares no code
//! with it. This is an agreement-test backend only — never a production
//! `--backend` selector, a user-facing Python emitter, or a Rust-to-Python
//! predicate translator.
//!
//! Responsibilities: for every fixture cell, shell out to
//! `python3 tests/python/kernel_reference.py` with
//! `{"specs": ["<verbatim markdown file>", ...], "expr": "<kernel text>"}`
//! on stdin, parse the single `{"status": ...}` JSON object on stdout, and
//! require that status to equal both the fixture's expected status and the
//! Rust backend's status. The corpus crosses the wire as raw file sources:
//! the Python side parses the markdown and the expression itself, so a
//! shared translated form cannot mask semantic drift. An absent `python3`
//! is a labeled setup failure naming the interpreter — the property never
//! skips.
//!
//! Rationale: a single evaluator cannot expose a shared misreading of the
//! kernel semantics; two independent implementations constrained to the
//! same expected statuses can. The empty-domain quantifier fixture lives
//! beside this test (the shared corpus predates it) and is exercised
//! through the exact same wire contract.
//!
//! RED today: `python3 tests/python/kernel_reference.py` exits 2 with the
//! labeled "not implemented (RED stage, specodelic-y5x8)" diagnostic, so
//! `cargo test --test kernel_agreement_py` fails in
//! `python_reference_agrees_on_every_kernel_fixture_cell` on the first
//! cell naming that diagnostic — a real failure, never a skip.

#[path = "common/mod.rs"]
mod common;

use common::kernel_fixtures::{Backend, FIXTURES, Outcome, extract};
use serde::Deserialize;
use specodelic::compile::ThreeValued;
use specodelic::spec::Spec;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The mandatory interpreter: when it is absent this property fails
/// setup with remediation, never a skip.
const PYTHON: &str = "python3";

const SCRIPT: &str = "tests/python/kernel_reference.py";

/// Empty-domain coverage the shared 36n corpus does not carry yet — the
/// bounded quantifiers must decide honestly over an unpopulated instance
/// (∀ vacuously verified, ∃ counterexample), and Python must agree with
/// Rust on both.
const EMPTY_DOMAIN_FIXTURE: &str = "tests/fixtures/kernel_agreement/empty_domain.json";

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn reference_script() -> PathBuf {
    repo_root().join(SCRIPT)
}

/// Fail setup, loudly, when the mandatory Python interpreter is absent.
fn require_python() {
    require_python_program(PYTHON);
}

fn require_python_program(program: &str) {
    let script = reference_script();
    assert!(
        script.is_file(),
        "kernel reference adapter missing at {script:?} — the Python kernel agreement \
         backend is part of the tree; restore {SCRIPT} (python3 must run it)"
    );
    match Command::new(program).arg("--version").output() {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            let code = output.status.code();
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr_text = stderr.trim();
            panic!(
                "`{program} --version` failed (status {code:?}): the Python kernel agreement \
                 backend requires a working python3. Install python3 (dnf/apt/brew) and \
                 re-run; this property never skips.\nstderr: {stderr_text}"
            )
        }
        Err(error) => panic!(
            "cannot run `{program} --version`: {error}. python3 is required for the \
             Python kernel agreement backend; install python3 (`dnf install python3`, \
             `apt-get install python3`, or `brew install python3`) and re-run. The \
             agreement property fails setup; it never skips."
        ),
    }
}

/// Round-trip one cell through the Python reference adapter and return its
/// three-valued status name.
fn evaluate_python(files: &[&str], expr: &str) -> String {
    let request = serde_json::json!({ "specs": files, "expr": expr });
    let payload = serde_json::to_vec(&request).expect("kernel reference request serializes");
    let script = reference_script();
    let mut child = Command::new(PYTHON)
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| {
            let script_display = script.display();
            panic!(
                "cannot start `{PYTHON} {script_display}`: {error}. Install python3; the \
                 agreement property never skips."
            )
        });
    if let Some(mut stdin) = child.stdin.take() {
        // A red-stage adapter may exit before draining stdin; the exit
        // status and stderr below carry the real failure.
        let _ = stdin.write_all(&payload);
    }
    let output = child
        .wait_with_output()
        .expect("python reference adapter terminates");
    let code = output.status.code();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout_text = stdout.trim();
    let stderr_text = stderr.trim();
    assert!(
        output.status.success(),
        "python kernel reference failed on `{expr}` (status {code:?}); it must consume \
         {{\"specs\": [...], \"expr\": \"...\"}} on stdin and print {{\"status\": \
         \"verified\" | \"counterexample\" | \"unknown\"}} on stdout.\nstdout:\n\
         {stdout_text}\nstderr:\n{stderr_text}"
    );
    let response: serde_json::Value = serde_json::from_str(stdout_text).unwrap_or_else(|error| {
        panic!(
            "python kernel reference printed non-JSON on stdout: {error}\n\
                 stdout:\n{stdout_text}\nstderr:\n{stderr_text}"
        )
    });
    let status = response
        .get("status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| {
            panic!(
                "python kernel reference response lacks a string `status`: {response}\n\
                 stderr:\n{stderr_text}"
            )
        });
    match status {
        "verified" | "counterexample" | "unknown" => status.to_string(),
        other => panic!(
            "python kernel reference returned an out-of-vocabulary status {other:?}; \
             the wire vocabulary is verified/counterexample/unknown"
        ),
    }
}

/// The rust seat, evaluated on the same corpus (shim over the shared
/// module's registry so the parity assertions read plainly).
fn rust_status(specs: &[Spec], expr: &str, fixture: &str, cell: &str) -> ThreeValued {
    match common::kernel_fixtures::evaluate(&Backend::Rust, specs, expr) {
        Outcome::Status(status) => status,
        Outcome::Unsupported(reason) => {
            panic!("rust backend must support every fixture cell ({fixture} :: {cell}): {reason}")
        }
    }
}

fn status_name(status: ThreeValued) -> &'static str {
    match status {
        ThreeValued::Verified => "verified",
        ThreeValued::Counterexample => "counterexample",
        ThreeValued::Unknown => "unknown",
    }
}

/// The main parity property: every fixture cell is evaluated by the Python
/// reference adapter and must agree with both the fixture's expected
/// status and the Rust backend's status.
#[test]
fn python_reference_agrees_on_every_kernel_fixture_cell() {
    require_python();
    let mut exercised = 0usize;
    for fixture in FIXTURES {
        let (specs, claims) = extract(fixture.files);
        assert_eq!(
            claims.len(),
            fixture.expected.len(),
            "fixture {} must extract exactly its expected claims",
            fixture.name
        );
        for (id, want) in fixture.expected {
            let (_, expr) = claims
                .iter()
                .find(|(cid, _)| cid.as_str() == *id)
                .unwrap_or_else(|| panic!("fixture {} claim {id} missing", fixture.name));
            let rust = rust_status(&specs, expr, fixture.name, id);
            let py = evaluate_python(fixture.files, expr);
            assert_eq!(
                py,
                status_name(*want),
                "python != expected on {} :: {id}",
                fixture.name
            );
            assert_eq!(
                py,
                status_name(rust),
                "python != rust on {} :: {id}",
                fixture.name
            );
            exercised += 1;
        }
    }
    let declared: usize = FIXTURES.iter().map(|f| f.expected.len()).sum();
    assert_eq!(
        exercised, declared,
        "every declared fixture cell must be exercised by the python parity property"
    );
}

#[derive(Deserialize)]
struct AgreementFixture {
    name: String,
    files: Vec<String>,
    expected: Vec<(String, String)>,
}

/// Empty-domain parity: bounded quantifiers over an unpopulated instance
/// decide honestly (∀ verified, ∃ counterexample) and both evaluators
/// agree. The shared corpus predates this cell; the fixture lives in
/// `tests/fixtures/kernel_agreement/`.
#[test]
fn python_reference_agrees_on_empty_domain_quantifiers() {
    require_python();
    let path = repo_root().join(EMPTY_DOMAIN_FIXTURE);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    let fixture: AgreementFixture = serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", path.display()));
    let files: Vec<&str> = fixture.files.iter().map(String::as_str).collect();
    let (specs, claims) = extract(&files);
    assert_eq!(
        claims.len(),
        fixture.expected.len(),
        "fixture {} must extract exactly its expected claims",
        fixture.name
    );
    for (id, want) in &fixture.expected {
        let (_, expr) = claims
            .iter()
            .find(|(cid, _)| cid == id)
            .unwrap_or_else(|| panic!("fixture {} claim {id} missing", fixture.name));
        let rust = rust_status(&specs, expr, &fixture.name, id);
        let py = evaluate_python(&files, expr);
        assert_eq!(
            py.as_str(),
            want.as_str(),
            "python != expected on {} :: {id}",
            fixture.name
        );
        assert_eq!(
            py,
            status_name(rust),
            "python != rust on {} :: {id}",
            fixture.name
        );
    }
}

/// The setup guard itself: an absent interpreter must panic with a labeled
/// remediation naming python3 — never skip the agreement property. Uses a
/// deliberately absent program name so the assertion is deterministic and
/// does not depend on the host's python3.
#[test]
fn missing_python3_fails_setup_with_labeled_remediation_never_skips() {
    let missing = "specodelic-test-missing-python3";
    let payload = std::panic::catch_unwind(|| require_python_program(missing))
        .expect_err("an absent interpreter must fail setup (panic), never skip");
    let message = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");
    for needle in ["python3", "install python3", "never skips"] {
        assert!(
            message.contains(needle),
            "missing-interpreter diagnostic must contain {needle:?}, got:\n{message}"
        );
    }
}
