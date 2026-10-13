//! First-party managed skill packs (add-skill-pack, specodelic-o1l3,
//! tasks.md §1 RED).
//!
//! The specodelic-design skill ships from this repo so the skill and the
//! binary evolve together (whisper `turu skill_pack` precedent). Phase 1
//! contract: `generate_pack` is deterministic, the pack layout is exactly
//! three files, every generated file carries the managed marker naming
//! the regenerate command, and `pack_content_hash` is SHA-256 over
//! sorted `path\0content` with a known fixture vector.

use specodelic::skill_pack::{PackFiles, generate_pack, pack_content_hash, render_body};

/// The managed marker every generated file must start with (proposal:
/// manual edits are detectable by construction; design D5/D6).
const MANAGED_MARKER: &str =
    "<!-- MANAGED BY specodelic — DO NOT EDIT MANUALLY. Regenerate: spk skill install -->";

#[test]
fn generate_pack_is_deterministic() {
    let a = generate_pack("specodelic-design").expect("known pack generates");
    let b = generate_pack("specodelic-design").expect("known pack generates");
    assert_eq!(a, b, "two generate_pack calls must be byte-identical");
}

#[test]
fn pack_layout_is_exactly_three_files() {
    let files = generate_pack("specodelic-design").expect("known pack generates");
    let mut paths: Vec<&String> = files.keys().collect();
    paths.sort();
    let expected = ["SKILL.md", "references/example.md", "references/format.md"];
    let got: Vec<&str> = paths.iter().map(|p| p.as_str()).collect();
    assert_eq!(got, expected, "pack layout is SKILL.md + two references");
}

#[test]
fn every_file_starts_with_managed_marker() {
    let files = generate_pack("specodelic-design").expect("known pack generates");
    assert!(
        !files.is_empty(),
        "sanity: the pack contains at least one file"
    );
    for (path, content) in &files {
        assert!(
            content.starts_with(MANAGED_MARKER),
            "file {path:?} must start with the managed marker naming `spk skill install`"
        );
    }
}

#[test]
fn repo_sources_stay_marker_free() {
    // The tracked sources are compiled in via include_str!; the marker is
    // prepended at generate time (design D1/D2), so the source files
    // themselves must not carry it — otherwise the marker would double up.
    let skill = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/skill-packs/specodelic-design/SKILL.md"
    ));
    assert!(
        !skill.starts_with(MANAGED_MARKER),
        "tracked SKILL.md source must be marker-free"
    );
}

#[test]
fn pack_content_hash_is_sha256_over_sorted_path_nul_content() {
    // Known fixture vector (computed independently of the implementation):
    //   sha256("a.md\0hello\n" ++ "b/dir.md\0world")
    //   = 948deb3c35ca67183557a1806f69d0956df794e228014457d05220218be234f0
    let mut files = PackFiles::new();
    files.insert("b/dir.md".to_string(), "world".to_string());
    files.insert("a.md".to_string(), "hello\n".to_string());
    assert_eq!(
        pack_content_hash(&files),
        "948deb3c35ca67183557a1806f69d0956df794e228014457d05220218be234f0",
        "hash must be SHA-256 over sorted path + NUL + content, concatenated"
    );
    // Insertion order must not matter (BTreeMap sorts by path).
    let mut reversed = PackFiles::new();
    reversed.insert("a.md".to_string(), "hello\n".to_string());
    reversed.insert("b/dir.md".to_string(), "world".to_string());
    assert_eq!(pack_content_hash(&files), pack_content_hash(&reversed));
}

#[test]
fn hash_of_generated_pack_is_stable_across_calls() {
    // The staleness check (doctor, phase 5) recomputes both sides; the
    // generated pack's hash must therefore be call-stable by construction.
    let a = generate_pack("specodelic-design").expect("known pack generates");
    let b = generate_pack("specodelic-design").expect("known pack generates");
    assert_eq!(pack_content_hash(&a), pack_content_hash(&b));
}

#[test]
fn unknown_pack_is_labeled_error() {
    let err = generate_pack("nope").expect_err("unknown pack must fail");
    assert!(
        err.contains("nope"),
        "error must be labeled with the requested pack name, got: {err}"
    );
}

// --- render pipeline (tasks.md §1.3): placeholders are replaced or fail
// --- labeled — never silently passed through ---

#[test]
fn unknown_render_placeholder_is_labeled_failure() {
    let body = "before <!-- spk:render:no-such-table --> after";
    let err = render_body(body).expect_err("unknown placeholder must fail, never pass through");
    assert!(
        err.contains("no-such-table"),
        "failure must name the placeholder identifier, got: {err}"
    );
}

#[test]
fn unterminated_render_placeholder_is_labeled_failure() {
    let body = "text <!-- spk:render:ears with no close";
    let err =
        render_body(body).expect_err("unterminated placeholder must fail, never pass through");
    assert!(
        err.contains("ears"),
        "failure must name the placeholder identifier, got: {err}"
    );
}

#[test]
fn body_without_placeholders_passes_through_verbatim() {
    assert_eq!(
        render_body("plain text").expect("pass through"),
        "plain text"
    );
}

#[test]
fn tracked_sources_render_clean() {
    // No tracked source carries a dangling placeholder at HEAD: pack
    // generation must succeed (a drift-class guard in its own right).
    generate_pack("specodelic-design").expect("current sources must render cleanly");
}
