//! The README front-door contract (dulce-de-leche standardization §5.1,
//! specodelic-wvzt): a cold reader decides whether to use the tool, runs
//! it once, and reaches the FAQ without opening another file.
//!
//! Only the mechanically checkable parts are asserted here — section
//! presence and order, the first-screen FAQ/positioning links, no
//! repository-development recipes in the quick start, and no broken
//! relative links. Prose quality is not a test.

use std::path::{Path, PathBuf};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn readme() -> String {
    std::fs::read_to_string(manifest_dir().join("README.md")).expect("README.md exists")
}

/// The front-door sections, in the order §5.1 requires them. `## Install`
/// already existed; the rest are the contract's additions.
const FRONT_DOOR_SECTIONS: &[&str] = &[
    "## When to use",
    "## Install",
    "## Quick start",
    "## Common workflows",
    "## Commands",
    "## FAQ",
];

fn heading_line(text: &str, heading: &str) -> usize {
    text.lines()
        .position(|line| line.trim_end() == heading)
        .unwrap_or_else(|| panic!("README carries `{heading}`"))
}

/// Markdown inline-link targets, skipping absolute URLs and anchors.
fn relative_links(text: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find("](") {
        rest = &rest[open + 2..];
        let Some(close) = rest.find(')') else { break };
        let target = rest[..close].trim();
        if !target.is_empty()
            && !target.starts_with('#')
            && !target.starts_with("mailto:")
            && !target.contains("://")
        {
            links.push(target.split('#').next().unwrap_or(target).to_string());
        }
        rest = &rest[close + 1..];
    }
    links
}

#[test]
fn readme_carries_the_front_door_sections_in_order() {
    let text = readme();
    let mut previous = 0;
    for heading in FRONT_DOOR_SECTIONS {
        let line = heading_line(&text, heading);
        assert!(
            line > previous,
            "`{heading}` must follow the previous front-door section (line {line} <= {previous})"
        );
        previous = line;
    }
}

#[test]
fn the_first_screen_links_the_faq_and_the_positioning_statement() {
    let text = readme();
    let first_section = text
        .lines()
        .position(|line| line.starts_with("## "))
        .expect("README has sections");
    let screen = text
        .lines()
        .take(first_section)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        screen.contains("docs/src/faq.md"),
        "the FAQ page must be one click from the README first screen:\n{screen}"
    );
    assert!(
        screen.contains("openspec/USAGE.md#positioning"),
        "the positioning statement must be linked from the README first screen:\n{screen}"
    );
}

#[test]
fn the_quick_start_holds_no_repository_development_recipe() {
    let text = readme();
    let start = heading_line(&text, "## Quick start");
    let end = text
        .lines()
        .enumerate()
        .skip(start + 1)
        .find(|(_, line)| line.starts_with("## "))
        .map(|(index, _)| index)
        .expect("Quick start is followed by another section");
    let quick_start = text
        .lines()
        .skip(start)
        .take(end - start)
        .collect::<Vec<_>>()
        .join("\n");
    for recipe in ["just ", "cargo build", "mdbook "] {
        assert!(
            !quick_start.contains(recipe),
            "`{recipe}` is a repository-development recipe; it belongs under ## Development"
        );
    }
}

#[test]
fn no_relative_readme_link_is_broken() {
    let root = manifest_dir();
    for target in relative_links(&readme()) {
        let resolved: &Path = &root.join(&target);
        assert!(
            resolved.exists(),
            "README links `{target}`, which does not resolve from the repository root"
        );
        // `docs/src/specs/` is the mdbook build's copy of the corpus
        // (gitignored, `.gitignore` line for docs/src/specs/). It exists
        // in a built checkout and 404s for every GitHub reader, so an
        // existence check alone cannot catch it.
        assert!(
            !target.starts_with("docs/src/specs/"),
            "README links `{target}` inside the generated book copy; link the tracked source instead"
        );
    }
}
