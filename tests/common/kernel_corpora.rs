//! Purpose: single home of the kernel fixture corpora consumed by the
//! Tier B integration tests — `tests/kernel_grounding.rs` (per-atomic
//! grounding agreement) and `tests/kernel_status.rs` (three-valued
//! status chain end-to-end).
//!
//! Responsibilities: hold each corpus constant verbatim (byte-identical
//! to its pre-move definition, ids unchanged) so every existing
//! assertion keeps its meaning; give the Python follow-up
//! (specodelic-l8l / add-py-fragment-emission) one import surface for
//! kernel shapes alongside `kernel_fixtures` (the backend-agreement
//! corpus, specodelic-36n).
//!
//! Rationale: the corpora were duplicated inline across the two test
//! files (§4.3 TIDY of add-min-expr-kernel, specodelic-36t); a pure
//! move — no fixture text, id, or expected-status edits — keeps the
//! Rust interim gate (design D2) at full strength while making the
//! shapes importable for the py promotion path. Cases are shrink-only:
//! added or refined, never dropped or rewritten, so promotion to a
//! `law_requires_cases`-shaped Property row (l8l) inherits the exact
//! named cases.
//!
//! Note on near-duplicates: some shapes here resemble
//! `kernel_fixtures.rs` entries (cyclic pair, dangling, duplicates,
//! chain, absorbing) but use distinct ids and slightly different
//! table shapes. Unifying them would change what the existing
//! assertions mean, so they stay separate on purpose — the
//! agreement corpus is frozen by the 36n property, these corpora are
//! frozen by the grounding/status tests.

/// A two-file corpus with a supersedes cycle between its constraints.
pub const CYCLIC: &[&str] = &[
    "---\nid: demo.cyc.a\nkind: intent\nstatement: \"THE a SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | supersedes |\n|----|------|------|-----------|------------|\n| ca | invariant | `holds a` | [[demo.cyc.a]] | [[demo.cyc.b.cb]] |\n",
    "---\nid: demo.cyc.b\nkind: intent\nstatement: \"THE b SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | supersedes |\n|----|------|------|-----------|------------|\n| cb | invariant | `holds b` | [[demo.cyc.b]] | [[demo.cyc.a.ca]] |\n",
];

/// A two-file corpus, acyclic supersedes, with one dangling traces_to.
pub const DANGLING: &[&str] = &[
    "---\nid: demo.dng.a\nkind: intent\nstatement: \"THE a SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | supersedes |\n|----|------|------|-----------|------------|\n| da | invariant | `holds a` | [[demo.dng.a]] | [[demo.dng.b.db]] |\n",
    "---\nid: demo.dng.b\nkind: intent\nstatement: \"THE b SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | supersedes |\n|----|------|------|-----------|------------|\n| db | invariant | `holds b` | [[demo.ghost.missing]] |  |\n",
];

/// A corpus where two constraints resolve to the SAME intent — the
/// duplicate-target shape `unique` refutes (injectivity).
pub const DUPLICATES: &[&str] = &[
    "---\nid: demo.dup\nkind: intent\nstatement: \"THE dup SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| d1 | invariant | `holds d1` | [[demo.dup]] |\n| d2 | invariant | `holds d2` | [[demo.dup]] |\n",
];

/// A corpus with a clean chain na → nb → nc through supersedes — the
/// reachability shape.
pub const CHAIN: &[&str] = &[
    "---\nid: demo.chn.a\nkind: intent\nstatement: \"THE a SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | supersedes |\n|----|------|------|-----------|------------|\n| na | invariant | `holds na` | [[demo.chn.a]] | [[demo.chn.b.nb]] |\n",
    "---\nid: demo.chn.b\nkind: intent\nstatement: \"THE b SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | supersedes |\n|----|------|------|-----------|------------|\n| nb | invariant | `holds nb` | [[demo.chn.b]] | [[demo.chn.c.nc]] |\n",
    "---\nid: demo.chn.c\nkind: intent\nstatement: \"THE c SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| nc | invariant | `holds nc` | [[demo.chn.c]] |\n",
];

/// An acyclic corpus whose supersedes references all resolve — the
/// `verified` shape.
pub const CLEAN: &str = "---\nid: demo.st.clean\nkind: intent\nstatement: \"THE clean SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | supersedes |\n|----|------|------|-----------|------------|\n| a1 | invariant | `**kernel:** acyclic(supersedes)` | [[demo.st.clean]] |  |\n| a2 | invariant | `**kernel:** resolves(traces_to)` | [[demo.st.clean]] |  |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | [[demo.st.clean.a1]] | `arb()` | `true` |\n";

/// A corpus with a dangling traces_to — the `counterexample` shape.
pub const BROKEN: &str = "---\nid: demo.st.broken\nkind: intent\nstatement: \"THE broken SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| b1 | invariant | `**kernel:** resolves(traces_to)` | [[demo.ghost.missing]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | [[demo.st.broken.b1]] | `arb()` | `true` |\n";

/// A corpus whose claim cites a row that does not exist — the honest
/// `unknown` shape (reachability from a seed that is not an instance
/// id cannot be discharged).
pub const UNKNOWABLE: &str = "---\nid: demo.st.unk\nkind: intent\nstatement: \"THE unk SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| u1 | invariant | `**kernel:** reachable(demo.ghost.nowhere, demo.st.unk.u1, supersedes)` | [[demo.st.unk]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | [[demo.st.unk.u1]] | `arb()` | `true` |\n";

/// The unknown-absorption shape: a composite of a verifiable claim and
/// an undischargable one — the composite is unknown, NEVER pass.
pub const ABSORBING: &str = "---\nid: demo.st.abs\nkind: intent\nstatement: \"THE abs SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a1 | invariant | `**kernel:** acyclic(supersedes) ∧ reachable(demo.ghost.nowhere, demo.st.abs.a1, supersedes)` | [[demo.st.abs]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | [[demo.st.abs.a1]] | `arb()` | `true` |\n";
