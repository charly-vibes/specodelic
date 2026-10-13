//! The docs stay thin pointers to the skill pack's single authoring
//! surfaces (add-skill-pack, `no-duplicate-protocol-in-the-book`).
//!
//! The protocol prose lives in `skill-packs/specodelic-design/SKILL.md`
//! and the worked example in `docs/src/examples/reservation-order.md`;
//! no book page may carry a copy of either beyond the thin-pointer
//! page's quoted one-line summary. This is the executable form of that
//! scenario: a distinctive protocol sentence appearing outside the
//! tracked source fails here, so a copy-pasted "reading convenience"
//! cannot creep into the book.

use std::path::Path;

/// Distinctive lines of the protocol prose — the conversation stage
/// list. A book page containing these is a duplicate authoring surface,
/// not a pointer.
const PROTOCOL_MARKERS: &[&str] = &[
    "1. **Listen (no format yet).**",
    "2. **Intent.**",
    "3. **Constraints.**",
];

/// The worked example's verbatim fingerprint: the full contiguous
/// Constraints table of `reservation-order.md`. A second verbatim copy
/// of the example contains this whole block; the narrative page
/// (worked-example.md) builds the spec in stages and never reproduces
/// the finished table contiguously.
fn example_fingerprint() -> String {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/src/examples/reservation-order.md"),
    )
    .expect("the book example exists");
    let start = text
        .find("| all_or_nothing")
        .expect("the example carries its first constraint row");
    let end = text
        .find("\n\n## Model")
        .expect("the Constraints table ends before the Model");
    text[start..end].to_string()
}

fn book_pages(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut pages = Vec::new();
    let stack = vec![dir.to_path_buf()];
    let mut stack = stack;
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                pages.push(path);
            }
        }
    }
    pages.sort();
    pages
}

#[test]
fn no_book_page_duplicates_the_protocol_prose_or_the_worked_example() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let book = Path::new(manifest).join("docs/src");
    let canonical_example = book.join("examples/reservation-order.md");
    let fingerprint = example_fingerprint();
    let mut offenders = Vec::new();
    for page in book_pages(&book) {
        // The example's own home is the authoring surface, never a copy.
        if page == canonical_example {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&page) else {
            continue;
        };
        for marker in PROTOCOL_MARKERS {
            if text.contains(marker) {
                offenders.push(format!(
                    "{} carries protocol prose: {marker:?}",
                    page.strip_prefix(manifest).unwrap_or(&page).display()
                ));
            }
        }
        if text.contains(&fingerprint) {
            offenders.push(format!(
                "{} carries a verbatim copy of the worked example",
                page.strip_prefix(manifest).unwrap_or(&page).display()
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "the book must stay a thin pointer — single authoring surfaces live in \
         skill-packs/ and docs/src/examples/: {offenders:#?}"
    );
}

#[test]
fn the_thin_pointer_page_names_the_procedure_and_the_install_command() {
    // The pointer must actually point: the thin page links the tracked
    // protocol source and names `spk skill install`.
    let page = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/src/design-conversations.md"),
    )
    .expect("the thin-pointer book page exists");
    assert!(
        page.contains("skill-packs/specodelic-design/SKILL.md"),
        "the thin page links the tracked protocol source"
    );
    assert!(
        page.contains("spk skill install"),
        "the thin page names the install command"
    );
}
