//! The embedded format guide — closed sets, format revision, primer prose.
//!
//! Purpose: single-source every closed value set the format enforces
//! (intent kinds, constraint kinds, property kinds, reference-typing
//! pairs) plus the format-revision marker, so parser, linter, compiler,
//! and the embedded guide all consume one definition. Responsibilities:
//! expose `pub const` slices for the closed sets, `FORMAT_REVISION`
//! mirroring the corpus revision this binary implements, and `GUIDE_MD`
//! (the `include_str!`-ed `src/guide.md` primer rendered by `spk
//! explain`). Rationale: one source makes closed-set drift
//! unrepresentable — the same design bias as the format itself
//! (append_only_variants); the primer cannot disagree with the enforced
//! sets because the sets are not duplicated in its prose.

/// The format revision this binary embeds/implements — mirrors the
/// latest `## Revision N` heading in `specs/specodelic.md`. Updated by
/// hand when the corpus revision bumps; a corpus-lint style drift test
/// (task 6.1) compares this numerically against the corpus so staleness
/// fails CI, not consumers.
pub const FORMAT_REVISION: &str = "specodelic.md Revision 8";

/// The closed set of Intent `kind` values (frontmatter).
pub const INTENT_KINDS: &[&str] = &["intent"];

/// The closed set of Constraint row `kind` values
/// (`constraint_kind_closed`).
pub const CONSTRAINT_KINDS: &[&str] = &["invariant", "advisory", "effect", "extension_point"];

/// The closed set of Property row `kind` values (`property_kind_closed`).
pub const PROPERTY_KINDS: &[&str] = &["unit", "law"];

/// One row of the Reference Typing table (`ref_kind_compatible`): which
/// `kind` a reference field may point at.
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct RefTyping {
    /// The reference field, e.g. `traces_to`.
    pub field: &'static str,
    /// Which row kind carries the field, e.g. `Constraint`.
    pub appears_on: &'static str,
    /// What the reference must resolve to, e.g. `Intent` or
    /// `Constraint, kind == invariant only`.
    pub resolves_to: &'static str,
}

/// The Reference Typing table — every typed foreign key of the format.
pub const REFERENCE_TYPING: &[RefTyping] = &[
    RefTyping {
        field: "traces_to",
        appears_on: "Constraint",
        resolves_to: "Intent",
    },
    RefTyping {
        field: "derives_from",
        appears_on: "Property",
        resolves_to: "Constraint",
    },
    RefTyping {
        field: "guard",
        appears_on: "Transition",
        resolves_to: "Constraint, kind == `invariant` only",
    },
    RefTyping {
        field: "from / to",
        appears_on: "Transition",
        resolves_to: "State",
    },
    RefTyping {
        field: "supersedes",
        appears_on: "Constraint, Property",
        resolves_to: "same kind as the row it appears on",
    },
    RefTyping {
        field: "emits",
        appears_on: "State",
        resolves_to: "Constraint, kind == `effect` only",
    },
    RefTyping {
        field: "satisfies",
        appears_on: "Constraint",
        resolves_to: "Constraint, kind == `extension_point` only",
    },
];

/// The embedded primer (`src/guide.md`) — seven topics' machine-facing
/// prose, rendered by `spk explain` (task 2.2). Conventions:
///
/// - Topics are delimited by `<!-- topic: <id> -->` markers; the topic
///   ids are `format`, `ears`, `kinds`, `references`, `lifecycle`,
///   `lint-rules`.
/// - `{{placeholder}}` slots are filled at render time from the
///   constants above (and, for `lint-rules`, from the lint rule table):
///   `{{intent_kinds}}`, `{{constraint_kinds}}`, `{{property_kinds}}`,
///   `{{reference_typing}}`, `{{lint_rules}}`. The closed sets are
///   therefore never duplicated in prose — a placeholder that survives
///   into rendered output is a bug, and a unit test asserts no topic
///   body contains `{{`.
pub const GUIDE_MD: &str = include_str!("guide.md");

/// The guide topics as `(id, title)` pairs, in serving order. Append-only:
/// new topics are added at the end, never renumbered (same OCP bias as the
/// format's tables).
pub const TOPICS: &[(&str, &str)] = &[
    ("format", "Format overview — the four layers"),
    ("ears", "The five EARS statement patterns"),
    ("kinds", "Closed kind value sets"),
    ("references", "Reference typing — typed foreign keys"),
    ("lifecycle", "The artifact lifecycle and stage gates"),
    ("lint-rules", "Lint rule catalog"),
    (
        "dual-format",
        "The spec/openspec dual-format protocol — one file, two parsers, migration recipe",
    ),
];

/// Render a topic's markdown body: slice [`GUIDE_MD`] at the
/// `<!-- topic: id -->` markers and fill the `{{placeholder}}` slots from
/// the constants above. `None` when the topic id is unknown.
pub fn topic_body(topic: &str) -> Option<String> {
    let marker = format!("<!-- topic: {topic} -->");
    let start = GUIDE_MD.find(&marker)?;
    let after = &GUIDE_MD[start + marker.len()..];
    let end = after.find("\n<!-- topic: ").unwrap_or(after.len());
    Some(fill_placeholders(after[..end].trim()))
}

/// Fill the primer's placeholder slots. Every closed set renders from the
/// constants — the enforced values and the rendered guide cannot disagree
/// (design Decision 2).
fn fill_placeholders(body: &str) -> String {
    body.replace("{{intent_kinds}}", &code_list(INTENT_KINDS))
        .replace("{{constraint_kinds}}", &code_list(CONSTRAINT_KINDS))
        .replace("{{property_kinds}}", &code_list(PROPERTY_KINDS))
        .replace("{{reference_typing}}", &reference_typing_rows())
        .replace("{{lint_rules}}", &lint_rules_catalog())
}

/// Backtick-join a closed set: `` `invariant`, `advisory`, ... ``.
fn code_list(values: &[&str]) -> String {
    values
        .iter()
        .map(|v| format!("`{v}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Render [`REFERENCE_TYPING`] as markdown table rows (with header).
fn reference_typing_rows() -> String {
    let mut rows = vec![
        "| Field | Appears on | Must resolve to |".to_string(),
        "|-------|------------|-----------------|".to_string(),
    ];
    for r in REFERENCE_TYPING {
        rows.push(format!(
            "| `{}` | {} | {} |",
            r.field, r.appears_on, r.resolves_to
        ));
    }
    rows.join("\n")
}

/// Render the lint rule catalog from [`crate::lint::RULE_TABLE`] — the
/// same table every finding's `rule_id`/`rule_semantics` come from, so
/// the rendered catalog can never disagree with what the linter emits
/// (task 3.2).
fn lint_rules_catalog() -> String {
    let mut rows = vec![
        "| Rule id | Requires |".to_string(),
        "|---------|----------|".to_string(),
    ];
    for (name, semantics) in crate::lint::RULE_TABLE {
        rows.push(format!(
            "| `{}` | {} |",
            crate::lint::rule_id(name),
            semantics
        ));
    }
    rows.join("\n")
}

/// Extract the trailing revision number from a `## Revision N` heading
/// (or any string carrying `Revision N`), numerically. Returns `None` for
/// headings without a trailing integer.
pub fn revision_number(heading: &str) -> Option<u32> {
    let rest = heading.trim().rsplit_once("Revision ")?.1;
    rest.split_whitespace().next()?.parse().ok()
}

/// The numerically largest `## Revision N` heading in a corpus file, or
/// `None` when it declares no revisions (the doctor currency check then
/// skips with an informational note — never an error).
pub fn latest_revision(text: &str) -> Option<u32> {
    text.lines()
        .filter(|l| l.trim_start().starts_with("## "))
        .filter_map(revision_number)
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_bodies_render_for_all_seven_topics() {
        // every declared topic renders non-empty (design Decision 2's
        // prose-drift test)
        assert_eq!(TOPICS.len(), 7);
        for (id, _title) in TOPICS {
            let body = topic_body(id).unwrap_or_else(|| panic!("topic `{id}` missing"));
            assert!(!body.trim().is_empty(), "topic `{id}` body is empty");
            // a surviving {{placeholder}} means a closed set leaked into
            // prose or a slot was never filled
            assert!(
                !body.contains("{{"),
                "topic `{id}` has unfilled placeholders"
            );
            assert!(
                !body.contains("<!-- topic: "),
                "topic `{id}` leaked a marker"
            );
        }
    }

    #[test]
    fn references_topic_documents_file_qualified_ref_syntax() {
        // gh#3: refs are file-qualified wiki-links `[[<file-id>.<row-id>]]`;
        // bare ids and bare text do not resolve. A dual-format delta cites
        // its own rows with the `spec.` self-file prefix.
        let refs = topic_body("references").unwrap();
        assert!(
            refs.contains("file-qualified"),
            "references topic must state the file-qualification law"
        );
        assert!(
            refs.contains("[[<file-id>.<row-id>]]"),
            "references topic must show the general wiki-link shape"
        );
        assert!(
            refs.contains("[[spec."),
            "references topic must document the `spec.` self-file prefix for dual-format deltas"
        );
        assert!(
            refs.to_lowercase().contains("bare"),
            "references topic must state that bare ids / bare text do not resolve"
        );
        // The claim must match actual behavior (Ro5 CORR-001): bare TEXT
        // produces no reference (orphan), but a bare dotless [[row-id]] is
        // skipped as metasyntactic — it silently vanishes from the graph,
        // nothing "fails".
        assert!(
            refs.contains("metasyntactic"),
            "references topic must describe the metasyntactic skip accurately"
        );
        assert!(
            !refs.contains("both fail"),
            "references topic must not claim the bare bracketed form fails"
        );
    }

    #[test]
    fn format_topic_points_at_the_file_qualified_ref_law() {
        // gh#3: the format topic's layer walkthrough mentions the guard
        // cell; it must not teach the bare `[[id]]` form.
        let format = topic_body("format").unwrap();
        assert!(
            !format.contains("by `[[id]]`"),
            "format topic must not advertise bare-id refs"
        );
    }

    #[test]
    fn dual_format_topic_documents_self_intent_trace_and_drift_rule() {
        // gh#5.3: `[[spec]]` (the file's own frontmatter id) is the
        // intended traces_to target for self-contained deltas.
        // gh#4: drift between the requirement sections is lint-enforced.
        let dual = topic_body("dual-format").unwrap();
        assert!(
            dual.contains("[[spec]]"),
            "dual-format topic must document the self-intent traces_to target"
        );
        assert!(
            dual.contains("requirement_drift"),
            "dual-format topic must state that spk lint enforces mirror sync"
        );
    }

    #[test]
    fn format_topic_documents_escaped_pipes() {
        // gh#6.5: cells containing a literal pipe must escape it.
        let format = topic_body("format").unwrap();
        assert!(
            format.contains("\\|"),
            "format topic must document escaped pipes in table cells"
        );
    }

    #[test]
    fn unknown_topic_renders_none() {
        assert!(topic_body("nope").is_none());
    }

    #[test]
    fn closed_sets_render_from_constants() {
        // kinds topic: the rendered kind sets equal the enforced constants
        let kinds = topic_body("kinds").unwrap();
        for k in CONSTRAINT_KINDS {
            assert!(
                kinds.contains(&format!("`{k}`")),
                "constraint kind `{k}` not rendered"
            );
        }
        for k in PROPERTY_KINDS {
            assert!(
                kinds.contains(&format!("`{k}`")),
                "property kind `{k}` not rendered"
            );
        }
        // references topic: the rendered table has one row per typing pair
        let refs = topic_body("references").unwrap();
        for r in REFERENCE_TYPING {
            assert!(
                refs.contains(&format!("`{}`", r.field)),
                "field `{}` not rendered",
                r.field
            );
            assert!(
                refs.contains(r.resolves_to),
                "target `{}` not rendered",
                r.resolves_to
            );
        }
    }

    #[test]
    fn primer_prose_does_not_duplicate_closed_sets() {
        // the embedded prose never lists the sets bare — only via
        // placeholders (design Decision 2)
        assert!(!GUIDE_MD.contains("{invariant, advisory, effect, extension_point}"));
        assert!(!GUIDE_MD.contains("{unit, law}"));
    }

    #[test]
    fn revision_comparison_is_numeric() {
        // double-digit ordering: 10 > 9 numerically (the drift guard must
        // not compare lexicographically)
        assert!(revision_number("Revision 10").unwrap() > revision_number("Revision 9").unwrap());
        assert_eq!(revision_number("## Revision 8"), Some(8));
        assert_eq!(revision_number("## Model"), None);
        assert_eq!(revision_number("## Revision abc"), None);
    }

    #[test]
    fn format_revision_matches_corpus() {
        // guide-drift guard (task 6.1): FORMAT_REVISION must name the
        // latest Revision N heading in specs/specodelic.md — the suite
        // fails here, naming both revisions, when the corpus bumps
        // without the constant following.
        let corpus = std::fs::read_to_string("specs/specodelic.md")
            .expect("specs/specodelic.md must be readable from the crate root");
        let corpus_rev =
            latest_revision(&corpus).expect("specs/specodelic.md declares no Revision headings");
        let format_rev = revision_number(FORMAT_REVISION)
            .unwrap_or_else(|| panic!("FORMAT_REVISION `{FORMAT_REVISION}` carries no Revision N"));
        assert_eq!(
            corpus_rev, format_rev,
            "format knowledge drift: corpus is at `Revision {corpus_rev}` but the binary embeds `Revision {format_rev}` ({FORMAT_REVISION}) — bump FORMAT_REVISION in src/guide.rs",
        );
    }
}
