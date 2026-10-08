//! The embedded format guide — closed sets, format revision, primer prose.
//!
//! Purpose: single-source every closed value set the format enforces
//! (intent kinds, constraint kinds, property kinds, reference-typing
//! pairs) plus the format-revision marker, so parser, linter, compiler,
//! and the embedded guide all consume one definition. Responsibilities:
//! expose `pub const` slices for the closed sets, `FORMAT_REVISION`
//! mirroring the corpus revision this binary implements, `GUIDE_MD`
//! (the `include_str!`-ed `src/guide.md` primer rendered by `spk
//! explain`), the `spk guide` JSON payloads — the ordinary value-set
//! payload and the versioned acset Schema export (`schema_export`,
//! add-graph-views D4), which also serializes constructed perturbation
//! fixtures for the Python schema-view tests. Rationale: one source
//! makes closed-set drift unrepresentable — the same design bias as the
//! format itself (append_only_variants); the primer cannot disagree with
//! the enforced sets because the sets are not duplicated in its prose,
//! and the schema export projects the lint-gated Schema value directly
//! so reference-typing constants never become a derivation source.

/// The format revision this binary embeds/implements — mirrors the
/// latest `## Revision N` heading in `specs/specodelic.md`. Updated by
/// hand when the corpus revision bumps; a corpus-lint style drift test
/// (task 6.1) compares this numerically against the corpus so staleness
/// fails CI, not consumers.
pub const FORMAT_REVISION: &str = "specodelic.md Revision 18";

/// The closed set of Intent `kind` values (frontmatter). Revision 14
/// added `profile` — a domain pack file (see `specs/packs.md`); the set
/// freezes at {intent, profile} by policy, pack vocabulary is
/// fiber-relative and never grows this set.
pub const INTENT_KINDS: &[&str] = &["intent", "profile"];

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
        resolves_to: "invariant Constraint, or a State (the “has reached state X” pattern)",
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
    RefTyping {
        field: "uses",
        appears_on: "Constraint",
        resolves_to: "Intent of a `kind: profile` file (the pack's frontmatter id) — set-valued, declares explicit pack enablement (Revision 14)",
    },
    RefTyping {
        field: "observes",
        appears_on: "Constraint",
        resolves_to: "Constraint, kind == `effect` only",
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
    (
        "packs",
        "Domain packs — first-class, opt-in vocabulary extension (Revision 14)",
    ),
    (
        "graph-views",
        "Graph views — derived diagrams and one-pipe render recipes",
    ),
];

/// The graph-views primer body (add-graph-views task 3.4, D8) — appended
/// after the `src/guide.md` topics and embedded in THIS module so the
/// primer file's topic list stays byte-untouched (gre.9 scope guard: the
/// edit to shared guide code must stay strictly additive; new topic data
/// lives at the end of the module, never interleaved). Same marker
/// convention as [`GUIDE_MD`] so one slicing helper serves both sources.
/// Rendering stays 100 % external (D8): the recipes pipe the native text
/// projections into user-chosen renderers (`dot`, `graph-easy`, any
/// mermaid paste target) — the tool never shells out to a renderer.
pub const GRAPH_VIEWS_BODY: &str = "\
<!-- topic: graph-views -->
Graph views are derived diagrams: they are never more current or more
correct than the graph artifact they were rendered from. Regenerate
after every corpus edit; never hand-edit a rendered view.

The taxonomy, over one artifact set (`spk graph` edges TSV + JSON
envelope, plus `spk guide --schema --json` for the typing diagram):

- `--format edges` — the raw six-column TSV projection (the contract
every view below consumes): source row, edge field, target row,
kinds, annotation. Duplicate edge instances are retained.
- `--format dot` / `--format mermaid` — whole-corpus text projections;
visual grammar: solid = state-machine edges, dashed = guards,
dotted = traceability, bold = `emits`, red dashed = dangling or
typing violations.
- `--view wiring` — the file-level producer→consumer view over
`constraints.satisfies` edges; self-loops dropped; a corpus with zero
satisfies edges renders a labeled `no_wiring` note, never a silently
clean diagram.
- `graph_views.py schema` — the revision-labeled Reference Typing
diagram, rendered from the versioned acset Schema export alone.
- `graph_views.py states` — per-file state machines (guards dashed,
violations annotated red, fan-in over distinct sources).
- `graph_views.py trace` — file-level traceability (edges collapsed to
owning intents, cross-file dependencies deduped, fan-in annotated).

One-pipe render recipes — rendering is external and user-chosen (D8);
the tool never shells out to a renderer:

    spk graph specs --format dot | dot -Tsvg > graph.svg
    spk graph specs --format dot | graph-easy --from dot   # ASCII, in-terminal
    spk graph specs --format mermaid > graph.mmd           # paste into mmdc,
                                                           # mermaid.live, or a
                                                           # GitHub mermaid fence

Script views (the prototype lives in scripts/ — D1):

    spk graph specs --format edges > edges.tsv
    spk graph specs --json > graph.json
    spk guide --schema --json > schema.json
    python3 scripts/graph_views.py schema schema.json
    python3 scripts/graph_views.py states edges.tsv --graph graph.json
    python3 scripts/graph_views.py trace edges.tsv --graph graph.json

A corpus is in scope iff it lints clean of invariant findings and has
at least one intent file; out-of-scope corpora are refused with a
remediation hint before any output is written. `just docs-graphs`
regenerates all four views into docs/src/views/ at docs build time;
the rendered views are never committed.
";

/// Render a topic's markdown body: slice [`GUIDE_MD`] at the
/// `<!-- topic: id -->` markers and fill the `{{placeholder}}` slots from
/// the constants above. Topics appended after `guide.md`'s set (the
/// append-only TOPICS law) are served from their module-level bodies —
/// currently only [`GRAPH_VIEWS_BODY`] — via the same slice-and-fill
/// helper. `None` when the topic id is unknown.
pub fn topic_body(topic: &str) -> Option<String> {
    slice_topic(GUIDE_MD, topic)
        .or_else(|| slice_topic(GRAPH_VIEWS_BODY, topic))
        .map(|body| fill_placeholders(&body))
}

/// Slice one marked-up source at the `<!-- topic: id -->` markers:
/// everything between this topic's marker and the next (or end of
/// source), trimmed.
fn slice_topic(source: &str, topic: &str) -> Option<String> {
    let marker = format!("<!-- topic: {topic} -->");
    let start = source.find(&marker)?;
    let after = &source[start + marker.len()..];
    let end = after.find("\n<!-- topic: ").unwrap_or(after.len());
    Some(after[..end].trim().to_string())
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

/// The ordinary guide value-set payload (`spk guide`, JSON envelope):
/// kinds, row shapes and format_revision for value-set consumers.
/// The Reference Typing table is deliberately absent (add-graph-views
/// D4): reference typing remains the embedded guide's rendering surface
/// only, and the schema view consumes the `--schema` export below.
pub fn value_set_payload() -> serde_json::Value {
    serde_json::json!({
        "format_revision": FORMAT_REVISION,
        "intent_kinds": INTENT_KINDS,
        "constraint_kinds": CONSTRAINT_KINDS,
        "property_kinds": PROPERTY_KINDS,
        "row_shapes": {
            "constraint": ["id", "kind", "expr", "traces_to"],
            "property": ["id", "kind", "derives_from"],
            "state": ["id", "emits"],
            "transition": ["id", "from", "to", "guard"],
        },
    })
}

/// The versioned schema export (`spk guide --schema`, JSON envelope
/// data — add-graph-views 2.2, design D4): a direct projection of the
/// acset `Schema` value with schema_version 1, the format revision
/// marker, sorted object names and morphisms sorted by (source, name).
/// Each morphism carries name, column, source, target, refinements
/// (sorted by side then kind), source_rule and endo_acyclic — no prose
/// parsing, no reference-typing constant, no graph-node inference.
/// Takes any `&Schema` so constructed perturbation fixtures exercise
/// the same serialization as the production input
/// (`acset::schema::canonical()`).
pub fn schema_export(schema: &crate::acset::schema::Schema) -> serde_json::Value {
    use crate::acset::schema::{Side, SourceRule};
    let refinements = |rs: &[crate::acset::schema::Refinement]| -> serde_json::Value {
        let mut rows: Vec<(&str, &str)> = rs
            .iter()
            .map(|r| match r.side {
                Side::Source => ("source", r.kind),
                Side::Target => ("target", r.kind),
            })
            .collect();
        rows.sort(); // (side, kind) canonical order
        serde_json::json!(
            rows.into_iter()
                .map(|(side, kind)| serde_json::json!({ "side": side, "kind": kind }))
                .collect::<Vec<_>>()
        )
    };
    let source_rule = |rule: SourceRule| match rule {
        SourceRule::Unchecked => "unchecked",
        SourceRule::AppearsOn => "appears_on",
        SourceRule::SameKind => "same_kind",
    };
    let mut morphisms: Vec<serde_json::Value> = schema
        .morphisms
        .iter()
        .map(|m| {
            serde_json::json!({
                "name": m.name,
                "column": m.column,
                "source": m.source.0,
                "target": m.target.0,
                "refinements": refinements(&m.refinements),
                "source_rule": source_rule(m.source_rule),
                "endo_acyclic": m.endo_acyclic,
            })
        })
        .collect();
    // Defensive canonical order: the Schema value declares sorted
    // morphisms, but constructed fixtures must serialize the same way.
    morphisms.sort_by_key(|m| {
        (
            m["source"].as_str().unwrap_or_default().to_string(),
            m["name"].as_str().unwrap_or_default().to_string(),
        )
    });
    serde_json::json!({
        "schema_version": 1,
        "format_revision": FORMAT_REVISION,
        "objects": schema
            .objects
            .iter()
            .map(|o| o.0.as_str())
            .collect::<Vec<_>>(),
        "morphisms": morphisms,
    })
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
    fn lifecycle_topic_states_the_claim_gate_and_assurance_distinction() {
        // specodelic-68m.5 (design D5): the embedded guide's lifecycle
        // topic states the implemented claim gate — every opted-in
        // invariant claim verified, prose stays unchecked — and
        // distinguishes verification from application-test execution.
        let lifecycle = topic_body("lifecycle").unwrap();
        assert!(
            lifecycle.contains("every opted-in invariant claim verified"),
            "lifecycle topic must state the claim gate"
        );
        assert!(
            lifecycle.contains("not a substitute for the application's own test suite"),
            "lifecycle topic must distinguish verification from application tests"
        );
    }

    #[test]
    fn topic_bodies_render_for_all_topics() {
        // every declared topic renders non-empty (design Decision 2's
        // prose-drift test) — the count bumps with each append-only topic
        // (9th = graph-views, add-graph-views task 3.4)
        assert_eq!(TOPICS.len(), 9);
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
        // bare ids and bare text do not resolve. (Revision 18: the
        // `spec.` self-file prefix retired with `id: spec` — dual-format
        // files carry real ids.)
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
            refs.contains("[[ge.cli.c1]]"),
            "references topic must document the real-id file-qualified form for spec.md files"
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
        // gh#5.3, revised Revision 18: `[[<file-id>]]` (the file's own
        // real id) is the intended traces_to target.
        // gh#4: drift between the requirement sections is lint-enforced.
        let dual = topic_body("dual-format").unwrap();
        assert!(
            dual.contains("[[<file-id>]]"),
            "dual-format topic must document the self-intent traces_to target"
        );
        assert!(
            dual.contains("id: ge.cli"),
            "dual-format topic must document the parent-dir id derivation"
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
    fn guard_typing_admits_state_citation() {
        // specodelic.md Revision 12: the Reference Typing `guard` row
        // admits a State target (the "has reached state X" pattern)
        // beside the invariant Constraint — the shipped checker
        // (graph.rs) and the independent oracle already encoded this;
        // the embedded table is the format knowledge surface and must
        // not lag the corpus.
        let guard = REFERENCE_TYPING
            .iter()
            .find(|r| r.field == "guard")
            .expect("guard row");
        assert!(
            guard.resolves_to.contains("invariant"),
            "guard must still exclude advisory Constraints: {}",
            guard.resolves_to
        );
        assert!(
            guard.resolves_to.contains("State"),
            "guard must admit the State citation pattern: {}",
            guard.resolves_to
        );
    }

    #[test]
    fn primer_prose_does_not_duplicate_closed_sets() {
        // the embedded prose never lists the sets bare — only via
        // placeholders (design Decision 2)
        assert!(!GUIDE_MD.contains("{invariant, advisory, effect, extension_point}"));
        assert!(!GUIDE_MD.contains("{unit, law}"));
    }

    #[test]
    fn law_cases_machine_form_is_ratified_in_the_corpus() {
        // update-law-named-cases task 2.1: the law_requires_cases row
        // names the machine-findable case form (the one compile's
        // required_law_cases parses) and Revision 13 records the
        // ratification.
        let corpus = std::fs::read_to_string("specs/specodelic.md")
            .expect("specs/specodelic.md must be readable from the crate root");
        let row = corpus
            .lines()
            .find(|l| l.starts_with("| law_requires_cases"))
            .expect("law_requires_cases row must exist in specodelic.md");
        assert!(
            row.contains("**name:**"),
            "law_requires_cases must name the machine-findable **name:** case-label form"
        );
        assert!(
            corpus.contains("## Revision 13"),
            "Revision 13 must record the law-case ratification"
        );
    }

    #[test]
    fn pack_mechanism_sets_grown_per_revision_14() {
        // add-domain-pack-mechanism (specodelic-dcx): Revision 14 grows
        // INTENT_KINDS by `profile` (a pack file) and REFERENCE_TYPING by
        // `uses` (Constraint, any file → Intent of a kind: profile file).
        // Base closed sets freeze by policy from this Revision on — these
        // are the last two core growths this mechanism needs.
        assert!(
            INTENT_KINDS.contains(&"profile"),
            "INTENT_KINDS must contain `profile` (Revision 14)"
        );
        let uses = REFERENCE_TYPING
            .iter()
            .find(|r| r.field == "uses")
            .expect("Reference Typing must carry a `uses` row (Revision 14)");
        assert_eq!(uses.appears_on, "Constraint");
        assert!(
            uses.resolves_to.contains("profile"),
            "uses must resolve to the intent of a kind: profile file: {}",
            uses.resolves_to
        );
    }

    #[test]
    fn pack_mechanism_ratified_in_the_corpus() {
        // Revision 14 is the decision of record for the pack mechanism:
        // the corpus carries the grown rows and the append-only heading.
        let corpus = std::fs::read_to_string("specs/specodelic.md")
            .expect("specs/specodelic.md must be readable from the crate root");
        assert!(
            corpus.contains("## Revision 14"),
            "Revision 14 must record the pack-mechanism growth"
        );
        let frontmatter_row = corpus
            .lines()
            .find(|l| l.starts_with("| frontmatter_valid"))
            .expect("frontmatter_valid row must exist");
        assert!(
            frontmatter_row.contains("profile"),
            "frontmatter_valid must admit kind: profile (Revision 14)"
        );
        let uses_row = corpus
            .lines()
            .find(|l| l.contains("| `uses`        |"))
            .expect("uses Reference Typing row must exist");
        assert!(
            uses_row.contains("profile"),
            "the uses row must resolve to a kind: profile file's intent"
        );
        let packs = std::fs::read_to_string("specs/packs.md")
            .expect("specs/packs.md must be readable from the crate root");
        assert!(
            packs.contains("kind: profile"),
            "specs/packs.md must describe the profile pack artifact"
        );
    }

    #[test]
    fn predicate_fragments_ratified_in_the_corpus() {
        // specodelic-rjb (Revision 15): executable predicate fragments —
        // the `**rust:**` opt-in marker, verbatim Rust, compiled into the
        // proptest artifact and executed as scratch-crate invariants.
        let compile_md = std::fs::read_to_string("specs/compile.md")
            .expect("specs/compile.md must be readable from the crate root");
        let pred_row = compile_md
            .lines()
            .find(|l| l.starts_with("| predicate_fragment_opt_in"))
            .expect("predicate_fragment_opt_in row must exist in compile.md");
        assert!(
            pred_row.contains("**rust:**"),
            "predicate_fragment_opt_in must name the **rust:** marker form"
        );
        let inv_row = compile_md
            .lines()
            .find(|l| l.starts_with("| invariant_fragment_opt_in"))
            .expect("invariant_fragment_opt_in row must exist in compile.md");
        assert!(
            inv_row.contains("**rust:**"),
            "invariant_fragment_opt_in must name the **rust:** marker form"
        );
        let model_md = std::fs::read_to_string("specs/model_check.md")
            .expect("specs/model_check.md must be readable from the crate root");
        assert!(
            model_md
                .lines()
                .any(|l| l.starts_with("| executable_invariants_execute")),
            "model_check.md must specify executable invariant execution"
        );
        let corpus = std::fs::read_to_string("specs/specodelic.md")
            .expect("specs/specodelic.md must be readable from the crate root");
        assert!(
            corpus.contains("## Revision 15"),
            "Revision 15 must record the predicate-fragment decision"
        );
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
    fn value_set_payload_carries_kinds_row_shapes_and_revision() {
        // add-graph-views 2.1: `spk guide` (JSON envelope) exposes kinds,
        // row shapes and format_revision for value-set consumers — and,
        // per D4, leaves the Reference Typing table OUT of the payload
        // (reference typing remains the embedded guide's rendering
        // surface only; the schema view consumes the --schema export).
        let payload = value_set_payload();
        assert_eq!(payload["format_revision"], FORMAT_REVISION);
        assert_eq!(
            payload["intent_kinds"],
            serde_json::json!(INTENT_KINDS),
            "intent kinds must equal the enforced constant"
        );
        assert_eq!(
            payload["constraint_kinds"],
            serde_json::json!(CONSTRAINT_KINDS)
        );
        assert_eq!(payload["property_kinds"], serde_json::json!(PROPERTY_KINDS));
        assert_eq!(
            payload["row_shapes"]["constraint"],
            serde_json::json!(["id", "kind", "expr", "traces_to"])
        );
        assert_eq!(
            payload["row_shapes"]["property"],
            serde_json::json!(["id", "kind", "derives_from"])
        );
        assert_eq!(
            payload["row_shapes"]["transition"],
            serde_json::json!(["id", "from", "to", "guard"])
        );
        assert_eq!(
            payload["row_shapes"]["state"],
            serde_json::json!(["id", "emits"])
        );
        // D4: no reference-typing table in the ordinary guide payload.
        assert!(payload.get("reference_typing").is_none());
        assert!(
            !payload.to_string().contains("resolves_to"),
            "ordinary guide payload must not leak typing-table rows"
        );
    }

    #[test]
    fn schema_export_projects_canonical_schema() {
        // add-graph-views 2.1/2.2 (D4): the versioned schema export is
        // derived from the acset Schema value — schema_version 1,
        // format_revision, sorted objects, morphisms sorted by
        // (source, name), each carrying name, column, source, target,
        // sorted refinements, source_rule and endo_acyclic.
        let data = schema_export(&crate::acset::schema::canonical());
        assert_eq!(data["schema_version"], 1);
        assert_eq!(data["format_revision"], FORMAT_REVISION);
        assert_eq!(
            data["objects"],
            serde_json::json!(["Constraint", "Intent", "Property", "State", "Transition"]),
            "objects must be the closed five, sorted"
        );
        let morphs = data["morphisms"]
            .as_array()
            .expect("morphisms must be an array");
        let canonical = &crate::acset::schema::canonical().morphisms;
        assert_eq!(
            morphs.len(),
            canonical.len(),
            "export must carry one row per canonical morphism"
        );
        // canonical order: sorted by (source, name) — the Schema value's
        // own canonical sort inherited by every derived artifact.
        let mut keys: Vec<(&str, &str)> = morphs
            .iter()
            .map(|m| (m["source"].as_str().unwrap(), m["name"].as_str().unwrap()))
            .collect();
        let sorted = keys.clone();
        keys.sort();
        assert_eq!(keys, sorted, "morphisms must be sorted by (source, name)");
        for (exported, row) in morphs.iter().zip(canonical) {
            assert_eq!(exported["name"], row.name);
            assert_eq!(exported["column"], row.column);
            assert_eq!(exported["source"], row.source.0);
            assert_eq!(exported["target"], row.target.0);
            assert!(exported.get("refinements").is_some());
            assert!(exported.get("source_rule").is_some());
            assert!(exported.get("endo_acyclic").is_some());
        }
        // Spot-checks pin the projection semantics, not just the shape.
        let find = |name: &str, source: &str| {
            morphs
                .iter()
                .find(|m| m["name"] == name && m["source"] == source)
                .unwrap_or_else(|| panic!("{source}.{name} missing from export"))
        };
        let traces = find("traces_to", "Constraint");
        assert_eq!(traces["source_rule"], "unchecked");
        assert!(traces["endo_acyclic"].is_null());
        let law = find("derives_from_law", "Property");
        assert_eq!(
            law["refinements"],
            serde_json::json!([{ "side": "source", "kind": "law" }])
        );
        assert_eq!(law["endo_acyclic"], false);
        let supersedes = find("supersedes", "Constraint");
        assert_eq!(supersedes["endo_acyclic"], true);
        assert_eq!(supersedes["source_rule"], "same_kind");
        let guard = find("guard", "Transition");
        assert_eq!(
            guard["refinements"],
            serde_json::json!([{ "side": "target", "kind": "invariant" }])
        );
    }

    #[test]
    fn schema_export_is_deterministic() {
        // byte-identical re-runs (sorted, deterministic data — D4)
        let a = schema_export(&crate::acset::schema::canonical());
        let b = schema_export(&crate::acset::schema::canonical());
        assert_eq!(a, b);
    }

    #[test]
    fn schema_export_accepts_constructed_schemas() {
        // The exporter is shared: the same function serializes
        // constructed perturbation fixtures (2.3's consumer inputs) and
        // canonical()'s production export. Here: canonical minus the
        // `emits` morphism — a valid schema differing by ONE morphism.
        let mut constructed = crate::acset::schema::canonical();
        let before = constructed.morphisms.len();
        constructed
            .morphisms
            .retain(|m| !(m.name == "emits" && m.source.0 == "State"));
        assert_eq!(constructed.morphisms.len(), before - 1);
        let data = schema_export(&constructed);
        let names: Vec<&str> = data["morphisms"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["name"].as_str().unwrap())
            .collect();
        assert!(!names.contains(&"emits"), "dropped morphism must be gone");
        assert_eq!(
            data["objects"],
            serde_json::json!(["Constraint", "Intent", "Property", "State", "Transition"])
        );
    }

    #[test]
    fn python_schema_view_renders_constructed_export_delta() {
        // add-graph-views 2.3 schema portion / D4: the shared exporter's
        // output for two valid constructed schemas differing in ONE
        // morphism renders through the real script as a diagram
        // differing by exactly that edge — the producer↔consumer
        // contract, exercised end to end without a renderer edit.
        let mut constructed = crate::acset::schema::canonical();
        constructed
            .morphisms
            .retain(|m| !(m.name == "emits" && m.source.0 == "State"));
        let render = |schema: &crate::acset::schema::Schema| {
            let path = std::env::temp_dir().join(format!(
                "spk_schema_export_{}_{}.json",
                std::process::id(),
                schema.morphisms.len()
            ));
            std::fs::write(
                &path,
                serde_json::to_string(&serde_json::json!({
                    "ok": true,
                    "data": schema_export(schema)
                }))
                .unwrap(),
            )
            .unwrap();
            let out = std::process::Command::new("python3")
                .arg("scripts/graph_views.py")
                .arg("schema")
                .arg(&path)
                .output()
                .expect("python3 must run the schema view script");
            std::fs::remove_file(&path).ok();
            assert!(
                out.status.success(),
                "schema view failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            String::from_utf8_lossy(&out.stdout).to_string()
        };
        let base = render(&crate::acset::schema::canonical());
        let perturbed = render(&constructed);
        assert!(base.contains("emits"), "canonical export renders the edge");
        let base_lines: Vec<&str> = base.lines().filter(|l| !l.trim().is_empty()).collect();
        let pert_lines: Vec<&str> = perturbed.lines().filter(|l| !l.trim().is_empty()).collect();
        let removed: Vec<&&str> = base_lines
            .iter()
            .filter(|l| !pert_lines.contains(l))
            .collect();
        let added: Vec<&&str> = pert_lines
            .iter()
            .filter(|l| !base_lines.contains(l))
            .collect();
        assert!(added.is_empty(), "dropping one morphism must add no edge");
        assert_eq!(removed.len(), 1, "one morphism out = one edge out");
        assert!(
            removed[0].contains("emits"),
            "the removed edge is the one dropped"
        );
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

    /// The graph-views primer topic (add-graph-views task 3.4) must be
    /// APPENDED — the packs topic keeps its index, and the new topic is
    /// last, never renumbered (the TOPICS append-only law).
    #[test]
    fn graph_views_topic_is_appended_last() {
        assert_eq!(
            TOPICS.last().unwrap().0,
            "graph-views",
            "graph-views must be the last topic — append-only, never renumbered"
        );
        assert_eq!(
            TOPICS.iter().position(|(id, _)| *id == "graph-views"),
            Some(TOPICS.len() - 1),
            "graph-views must appear exactly once, at the end"
        );
    }

    /// The topic body carries the view taxonomy, the format flags and the
    /// one-pipe render recipes (task 3.4, D8): rendering stays external
    /// and user-chosen — dot, graph-easy for the terminal, mermaid paste
    /// targets.
    #[test]
    fn graph_views_topic_documents_taxonomy_flags_and_recipes() {
        let body = topic_body("graph-views").expect("graph-views topic must render");
        for fragment in [
            "--format edges",
            "--format dot",
            "--format mermaid",
            "--view wiring",
            "guide --schema --json",
            "| dot -Tsvg",
            "graph-easy",
        ] {
            assert!(
                body.contains(fragment),
                "graph-views topic must document `{fragment}` — the primer is \
                 the embedded render-recipe surface (D8)"
            );
        }
        // No placeholder survives rendering (the same law as every topic).
        assert!(!body.contains("{{"), "a placeholder survived rendering");
    }
}
