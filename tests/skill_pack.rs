//! First-party managed skill packs (add-skill-pack, specodelic-o1l3,
//! tasks.md §1 RED).
//!
//! The specodelic-design skill ships from this repo so the skill and the
//! binary evolve together (whisper `turu skill_pack` precedent). Phase 1
//! contract: `generate_pack` is deterministic, the pack layout is exactly
//! three files, every generated file carries the managed marker naming
//! the regenerate command, and `pack_content_hash` is SHA-256 over
//! sorted `path\0content` with a known fixture vector.
//!
//! Phase 2 (specodelic-ifm7) contract: the explain-derived tables are
//! rendered through the same code paths that serve `spk explain` (design
//! D3 — no table restatement), and every topic name, lint rule id, and
//! EARS pattern name mentioned in the hand-authored protocol resolves
//! against the in-binary tables. A dangling mention fails naming the
//! identifier — a hand-maintained list may be stale, a generated one
//! cannot.

use specodelic::ears::PATTERNS;
use specodelic::guide::TOPICS;
use specodelic::lint::{RULE_TABLE, lint_all};
use specodelic::skill_pack::{
    PackFiles, check_membership, generate_pack, pack_content_hash, render_body, topics_mentioned,
};
use specodelic::spec::parse_str;

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

// ---- Phase 2: explain-derived rendering (tasks.md §2.2, design D3) ----

#[test]
fn topics_line_renders_from_the_topic_list_table() {
    // The protocol's topics line is rendered from `guide::TOPICS` — the
    // same table `spk explain` (no argument) lists — never restated in
    // the source. Order and set must match the served table exactly (the
    // grounding drift class: the draft listed 8 of 9 topics).
    let pack = generate_pack("specodelic-design").expect("pack generates");
    let skill = &pack["SKILL.md"];
    let line = skill
        .lines()
        .find(|l| l.contains("topics:"))
        .expect("SKILL.md names the explaunable topics");
    let rendered = topics_mentioned(line);
    let served: Vec<&str> = TOPICS.iter().map(|(id, _)| *id).collect();
    assert_eq!(
        rendered, served,
        "the topics line must equal the served topic list, in serving order"
    );
    // and the source itself must not carry a hand-copied list: the line
    // was produced by the renderer, so the tracked source holds the
    // placeholder instead.
    let source = include_str!("../skill-packs/specodelic-design/SKILL.md");
    assert!(
        source.contains("<!-- spk:render:explain-topics -->"),
        "SKILL.md must render its topics line from the placeholder, not restate it"
    );
}

#[test]
fn format_reference_renders_the_explain_topics_it_summarizes() {
    // references/format.md renders EARS patterns, kind sets, the
    // typed-reference table, and the naming law through the same code
    // paths that serve `spk explain ears`/`kinds`/`references`/`format`
    // — byte-equal to the served bodies, never a hand-copied table.
    let pack = generate_pack("specodelic-design").expect("pack generates");
    let format_ref = &pack["references/format.md"];
    for topic in ["format", "ears", "kinds", "references"] {
        let body = specodelic::guide::topic_body(topic)
            .unwrap_or_else(|| panic!("topic `{topic}` must render"));
        assert!(
            format_ref.contains(&body),
            "references/format.md must render the served `{topic}` body verbatim"
        );
    }
    // no placeholder and no unfilled slot may survive rendering
    for content in pack.values() {
        assert!(!content.contains("<!-- spk:render:"), "placeholder leaked");
        assert!(!content.contains("{{"), "unfilled slot leaked");
    }
}

#[test]
fn ears_table_names_exactly_the_five_grammar_patterns() {
    // The rendered EARS table must name exactly the patterns the EARS
    // grammar classifies — the enum is the source of truth, the guide
    // prose cannot gain or lose a pattern without failing here.
    let pack = generate_pack("specodelic-design").expect("pack generates");
    let served: Vec<&str> = PATTERNS.iter().map(|p| p.display_name()).collect();
    let rendered = specodelic::skill_pack::pattern_name_mentions(&pack["references/format.md"]);
    assert_eq!(
        rendered, served,
        "the rendered EARS table must list exactly the five grammar patterns"
    );
}

#[test]
fn every_rule_mention_in_the_protocol_resolves() {
    // The self-check names the rules behind each hand-check in canonical
    // `linter.<rule>` form; every such mention must resolve against the
    // lint rule catalog (the grounding drift class: a rule cited in a
    // failure table but absent from the served list).
    let pack = generate_pack("specodelic-design").expect("pack generates");
    let skill = &pack["SKILL.md"];
    check_membership(skill).expect("every mention in the protocol resolves");
    // the gate must have shipped teeth: the self-check names rules
    let mentioned = skill
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|t| t.starts_with("linter."))
        .count();
    assert!(
        mentioned >= 5,
        "the self-check must name the governing rules in `linter.<rule>` form, found {mentioned}"
    );
    for token in skill.split('`').skip(1).step_by(2) {
        if let Some(rule) = token.strip_prefix("linter.") {
            assert!(
                RULE_TABLE.iter().any(|(name, _)| *name == rule),
                "rule `{rule}` mentioned in the protocol is not in the lint rule catalog"
            );
        }
    }
}

// ---- Phase 2: membership gate (tasks.md §2.1) ----

#[test]
fn dangling_topic_mention_fails_naming_the_identifier() {
    let err = check_membership("consult spk explain (topics: format, no-such-topic)")
        .expect_err("a dangling topic mention must fail");
    assert!(
        err.contains("no-such-topic"),
        "failure must name the identifier, got: {err}"
    );
}

#[test]
fn dangling_rule_mention_fails_naming_the_identifier() {
    let err = check_membership("treat the finding's rule id `linter.no_such_rule` as ground truth")
        .expect_err("a dangling rule mention must fail");
    assert!(
        err.contains("linter.no_such_rule"),
        "failure must name the identifier, got: {err}"
    );
}

#[test]
fn dangling_ears_pattern_fails_naming_the_identifier() {
    let fixture = "| Pattern | Shape |\n|---|---|\n| Fake-Pattern | `THE SYSTEM SHALL x` |";
    let err = check_membership(fixture).expect_err("a dangling EARS pattern must fail");
    assert!(
        err.contains("Fake-Pattern"),
        "failure must name the identifier, got: {err}"
    );
}

#[test]
fn resolvable_mentions_pass_the_membership_gate() {
    // the served tables themselves are members by construction
    let fixture = format!(
        "(topics: {}) — rule `linter.ears_syntax` — | Event-Driven | `WHEN t THE SYSTEM SHALL r` |",
        TOPICS
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>()
            .join(", ")
    );
    check_membership(&fixture).expect("resolvable mentions pass");
}

#[test]
fn metasyntactic_explain_mention_is_not_flagged() {
    // `spk explain <topic>` in prose is a meta-shape, not a topic mention
    check_membership("prefer `spk explain <topic>` for detail")
        .expect("metasyntactic mention passes");
}

// ---- phase 3 (specodelic-xoeu, tasks.md §3): example embedding + lint gate ----

/// The book example, read from disk at test time — the single authoring
/// surface for the worked example (design D2). Reading at test time (not
/// `include_str!`) is what makes the verbatim check a drift detector: the
/// assertion compares the compiled-in pack against what the repo
/// actually serves, so a corpus edit that dirties the example flips this
/// test (via the lint gate below) or the doc-examples gate — never
/// neither.
fn book_example() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/docs/src/examples/reservation-order.md"
    ))
    .expect("the book example exists at docs/src/examples/reservation-order.md")
}

#[test]
fn example_reference_embeds_the_book_example_verbatim() {
    // design D2: `references/example.md` is the book example verbatim
    // under the managed marker — never a copied (driftable) second
    // authoring surface.
    let files = generate_pack("specodelic-design").expect("known pack generates");
    let generated = files
        .get("references/example.md")
        .expect("the pack carries references/example.md");
    let expected = format!("{MANAGED_MARKER}\n\n{}", book_example());
    assert_eq!(
        generated, &expected,
        "references/example.md must be the managed marker + docs/src/examples/reservation-order.md, verbatim"
    );
}

#[test]
fn embedded_example_lints_clean_through_the_corpus_api() {
    // The shipped linter, over the embedded example, through the same
    // entry point `spk lint` (and so `just lint-specs`) runs — zero
    // findings and zero warnings, else the pack would ship a skill
    // teaching a spec that fails its own tool (tasks.md §3.1).
    let book = book_example();
    let spec = parse_str(&book).expect("the book example parses as a spec");
    let report = lint_all(&[spec], &[]);
    assert_eq!(
        report.failures(),
        0,
        "the embedded example must lint with zero findings: {:?}",
        report.issues
    );
    assert!(
        report.warnings.is_empty(),
        "the embedded example must lint with zero warnings: {:?}",
        report.warnings
    );
}
