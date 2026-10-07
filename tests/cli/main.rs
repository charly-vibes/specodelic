//! Integration tests — run the `specodelic` binary against the repo's own corpus.

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap()
}

fn write_bad_ears_spec(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!("---\nid: {id}\nkind: intent\nstatement: \"the system should maybe work\"\n---\n"),
    )
    .unwrap();
}

/// A lint-clean spec with a two-state model — the model-check fixture.
fn write_model_check_spec(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL be a model-check fixture\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | c1 | invariant | `holds` | [[{id}]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- s1\n\
             - s2\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | t | s1 | s2 | [[{id}.c1]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p | unit | [[{id}.c1]] | `g()` | `x` |\n"
        ),
    )
    .unwrap();
}
mod citation_resolution;
mod feedback_init;
mod hooks_hostile_output;
mod lint;
mod model_check;
mod orchestrate_verify;
mod parse_misc;
mod tlc_observability;

fn write_parse_fixture(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL parse as structured IR\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | c1 | invariant | `holds` | [[{id}]] |\n\
             | c2 | invariant | `always` | [[{id}.c1]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- s1\n\
             - s2\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | t | s1 | s2 | [[{id}.c1]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p1 | unit | [[{id}.c1]] | `g()` | `x` |\n\
             | p2 | unit | [[{id}.c2]] | `h()` | `y` |\n\
             | p3 | unit | [[{id}.c1]] | `k()` | `z` |\n"
        ),
    )
    .unwrap();
}

fn compile_fixture(td: &tempfile::TempDir, id: &str, body_line: &str) -> (String, String) {
    let spec = td.path().join(format!("{id}.md"));
    write_model_check_spec(&spec, id);
    let out = td.path().join("out");
    spk()
        .args([
            "compile",
            spec.to_str().unwrap(),
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();
    let props = out.join(format!("{id}_props.rs"));
    let src = std::fs::read_to_string(&props).unwrap();
    let translated = src
        .lines()
        .map(|l| {
            if l.trim().starts_with("todo_predicate!") {
                body_line.to_string()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&props, translated).unwrap();
    (
        spec.to_str().unwrap().to_string(),
        out.to_str().unwrap().to_string(),
    )
}

fn hooks_fixture(with_config: bool, with_openspec: bool) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    if with_config {
        std::fs::write(
            dir.path().join("lefthook.yml"),
            "pre-commit:\n  commands:\n    sibling-blockers:\n      run: scripts/guards/sibling-blockers.sh .\n",
        )
        .unwrap();
    }
    if with_openspec {
        std::fs::create_dir_all(dir.path().join("openspec")).unwrap();
    }
    dir
}
