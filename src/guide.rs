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

/// The embedded primer (`src/guide.md`) — six topics' machine-facing
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
