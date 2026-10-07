//! Backend-agreement property — phase 4 of add-min-expr-kernel
//! (specodelic-36n, tasks.md §4.1/§4.2; design D2).
//!
//! The same fixture corpus (tests/common/kernel_fixtures.rs) is
//! evaluated through every registered backend and the per-cell
//! three-valued status must be identical across backends (⟦−⟧py ≅
//! ⟦−⟧rust over shared fixtures) and match the fixture's expected
//! status. Rust exists today; the py backend arrives with
//! specodelic-l8l — its seat is already registered and must report
//! labeled `Unsupported` (naming l8l), never a fabricated verdict.
//! When l8l's emitter lands it activates the seat and the
//! cross-backend equality assertion below covers it with no edits —
//! the assertion is live, never commented out.

#[path = "common/mod.rs"]
mod common;

use common::kernel_fixtures::{BACKENDS, Backend, FIXTURES, Outcome, extract};
use specodelic::compile::ThreeValued;

#[test]
fn every_cell_matches_its_expected_status_on_the_rust_backend() {
    for fixture in FIXTURES {
        let (specs, claims) = extract(fixture.files);
        assert_eq!(
            claims.len(),
            fixture.expected.len(),
            "fixture {} must extract exactly its expected claims",
            fixture.name
        );
        for (id, want) in fixture.expected {
            let (_, text) = claims
                .iter()
                .find(|(cid, _)| cid == id)
                .unwrap_or_else(|| panic!("fixture {} claim {id} missing", fixture.name));
            match evaluate_rust(&specs, text) {
                Outcome::Status(got) => assert_eq!(
                    got, *want,
                    "backend disagreement with the fixture's expected status: \
                     {} :: {}",
                    fixture.name, id
                ),
                Outcome::Unsupported(reason) => panic!(
                    "rust backend must support every fixture cell ({} :: {}): {reason}",
                    fixture.name, id
                ),
            }
        }
    }
}

#[test]
fn supporting_backends_agree_on_every_cell() {
    // The cross-backend assertion is live from day one: over the
    // registered seats, every cell's status must be identical across
    // the backends that support it. Today only rust supports, so the
    // loop trivially holds — but it runs, and when l8l's py emitter
    // activates the seat, agreement is asserted here with no edits.
    for fixture in FIXTURES {
        let (specs, claims) = extract(fixture.files);
        for (id, text) in &claims {
            let mut seen: Vec<(&str, ThreeValued)> = vec![];
            for backend in BACKENDS {
                match common::kernel_fixtures::evaluate(backend, &specs, text) {
                    Outcome::Status(status) => seen.push((backend.name(), status)),
                    Outcome::Unsupported(_) => {} // excluded from agreement, checked below
                }
            }
            assert!(
                !seen.is_empty(),
                "at least one backend must support each cell ({} :: {id})",
                fixture.name
            );
            for (name, status) in &seen[1..] {
                assert_eq!(
                    status, &seen[0].1,
                    "backend disagreement on {} :: {id}: {name} says {status:?}, \
                     {} says {:?}",
                    fixture.name, seen[0].0, seen[0].1
                );
            }
        }
    }
}

#[test]
fn py_seat_is_honest_pending_l8l() {
    // The activation seat must never fabricate a verdict: until l8l's
    // emitter lands, every cell reports labeled `Unsupported` naming
    // the change that will replace it. If this test fails with a real
    // status, l8l has landed — move the py expectations into the
    // fixture table and let `supporting_backends_agree_on_every_cell`
    // take over.
    for fixture in FIXTURES {
        let (specs, claims) = extract(fixture.files);
        for (id, text) in &claims {
            match common::kernel_fixtures::evaluate(&Backend::Py, &specs, text) {
                Outcome::Status(status) => panic!(
                    "py seat fabricated a verdict on {} :: {id} ({status:?}) — \
                     l8l activation must land the real expectations, not a stub",
                    fixture.name
                ),
                Outcome::Unsupported(reason) => {
                    assert!(
                        !reason.is_empty(),
                        "unsupported must be labeled ({} :: {id})",
                        fixture.name
                    );
                    assert!(
                        reason.contains("l8l"),
                        "the unsupported label must name its activation change \
                         ({} :: {id}): {reason}",
                        fixture.name
                    );
                }
            }
        }
    }
}

/// The rust seat, evaluated on the fixture corpus (shim over the shared
/// module's evaluator so the expected-status test reads plainly).
fn evaluate_rust(specs: &[specodelic::spec::Spec], text: &str) -> Outcome {
    common::kernel_fixtures::evaluate(&Backend::Rust, specs, text)
}
