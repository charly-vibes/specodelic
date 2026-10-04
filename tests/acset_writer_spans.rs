//! Span-presence tests (`add-acset-writer` task 2.1 — RED first).
//!
//! The writer edits only recorded spans (`source_spans_recorded`), so
//! parsing must record the byte span of every id cell, every `[[link]]`
//! occurrence, and every state bullet. Spans are additive fields on the
//! parsed structures, skipped in serialization (`spk parse --json` is
//! byte-stable); these tests pin the recording itself.

use specodelic::spec::{Span, parse_str};

/// One of every editable family, LF line endings.
fn full_family_spec(id: &str) -> String {
    format!(
        "---\nid: {id}\nkind: intent\nstatement: \"THE {id} SHALL hold one of every rewrite family\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `holds` | [[{id}]] |\n\n## Model\n\n### States\n- spanned\n- applied (emits: [[{id}.c1]])\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | spanned | applied | [[{id}.c1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | [[{id}.c1]] | `g()` | `x` |\n"
    )
}

/// The span must resolve to exactly `expected` bytes in the source —
/// the recording is byte-exact, not approximate.
fn assert_span(raw: &str, span: Option<Span>, expected: &str, what: &str) {
    let s = span.unwrap_or_else(|| panic!("{what}: span not recorded"));
    assert_eq!(
        &raw[s.start..s.end],
        expected,
        "{what}: span does not resolve to the expected bytes"
    );
}

#[test]
fn every_id_cell_and_link_and_bullet_carries_a_span() {
    let raw = full_family_spec("spans.a");
    let spec = parse_str(&raw).expect("fixture parses");

    // Frontmatter intent id.
    assert_span(&raw, spec.intent.id_span, "spans.a", "intent id");

    // Every structured row's id cell.
    for r in spec.constraints.iter().chain(&spec.properties) {
        assert_span(&raw, r.id_span, &r.id, &format!("row {} id cell", r.id));
    }
    for t in &spec.transitions {
        assert_span(&raw, t.id_span, &t.id, "transition id cell");
        assert_span(&raw, t.from_span, &t.from, "transition from cell");
        assert_span(&raw, t.to_span, &t.to, "transition to cell");
    }

    // Every state bullet's leading id token.
    for s in &spec.states {
        assert_span(&raw, s.id_span, &s.id, &format!("state bullet {}", s.id));
    }

    // Every `[[link]]` occurrence: the span covers the full brackets
    // and the inner text is the recorded target.
    assert!(!spec.links.is_empty(), "fixture must carry links");
    for l in &spec.links {
        let s = l.span.expect("link span not recorded");
        let covered = &raw[s.start..s.end];
        assert_eq!(
            covered,
            format!("[[{}]]", l.target),
            "link to {} must span exactly its [[…]] occurrence",
            l.target
        );
    }

    // Multiple links to the same target get DISTINCT spans.
    let raw2 = format!(
        "---\nid: dup.l\nkind: intent\nstatement: \"THE dup.l SHALL link twice\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `holds` | [[dup.l]] and [[dup.l]] |\n"
    );
    let spec2 = parse_str(&raw2).expect("parses");
    let spans: Vec<_> = spec2.links.iter().map(|l| l.span.unwrap()).collect();
    assert_eq!(spans.len(), 2, "both link occurrences recorded");
    assert_ne!(spans[0], spans[1], "distinct occurrences, distinct spans");
    assert_eq!(&raw2[spans[1].start..spans[1].end], "[[dup.l]]");
}

/// CRLF files: byte offsets must still resolve to the same content —
/// the `\r` lives at line end, after every recorded span.
#[test]
fn spans_survive_crlf() {
    let raw = full_family_spec("crlf.s").replace('\n', "\r\n");
    let spec = parse_str(&raw).expect("CRLF fixture parses");
    assert_span(&raw, spec.intent.id_span, "crlf.s", "intent id (CRLF)");
    for r in &spec.constraints {
        assert_span(&raw, r.id_span, &r.id, "constraint id cell (CRLF)");
    }
    for s in &spec.states {
        assert_span(&raw, s.id_span, &s.id, "state bullet (CRLF)");
    }
    for l in &spec.links {
        let sp = l.span.expect("link span (CRLF)");
        assert_eq!(
            &raw[sp.start..sp.end],
            format!("[[{}]]", l.target),
            "link span (CRLF)"
        );
    }
}

/// `spk parse --json` stays byte-stable: span fields are skipped in
/// serialization, so the Spec IR payload never grows (design D3).
#[test]
fn spans_are_not_serialized() {
    let raw = full_family_spec("ser.a");
    let spec = parse_str(&raw).expect("parses");
    let json = serde_json::to_string(&spec).expect("serializes");
    assert!(
        !json.contains("\"span\"") && !json.contains("id_span"),
        "span fields must be serde-skipped: {json}"
    );
}
