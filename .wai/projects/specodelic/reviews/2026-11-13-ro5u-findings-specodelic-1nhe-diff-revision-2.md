---
reviews: 2026-11-13-ro5u-specodelic-1nhe-critical-none-high-1.md
verdict: pass
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-1nhe-law-kind-per-case-fragment-bodies, pipeline-step:ro5u-review]
---

# RO5U findings — specodelic-1nhe diff (Revision 21, law-case per-case bodies)

## Critical / High

1. **The helper rule did not reach per-case bodies** —
   `src/compile.rs::fragment_cell_texts` skipped law rows (they could
   not carry fragments before Revision 21), so an `@name` inside a
   per-case body was neither validated nor counted as live: an unknown
   helper name was emitted verbatim into an artifact that cannot
   compile (not a labeled failure), and a helper used only by a body was
   reported dead on compile's warnings channel. **Fixed**: law rows now
   contribute their per-case bodies to `fragment_cell_texts`; three
   tests pin it (inlining at the use site, an unknown reference failing
   labeled at `fragment_extraction`, a body-only helper not reported
   dead).

## Medium

2. **Two sources of truth for the closed tag set** — `law::FRAGMENT_TAGS`
   and `compile::FragmentLang::parse` could drift. **Fixed**:
   `FragmentLang::parse` now defers to `law::FRAGMENT_TAGS`.
3. **A silent fall-through** — `FragmentLang::parse(&body.tag)
   .unwrap_or(FragmentLang::Rust)` would have fallen through to Rust
   for an unknown tag, against the never-silent-fall-through
   discipline. **Fixed**: the arm fails labeled, naming the tag and the
   closed set.
4. **A rule id that exists nowhere in the corpus** — the labeled
   failures named `law_case_bodies`; the corpus's own rule is
   `compile.fragment_law_rejected`. **Fixed**: the messages name the
   corpus rule.
5. **Two silent-ignore holes in the case grammar** — an empty body and a
   case key that introduces no body both compiled silently. **Fixed**:
   both are labeled extraction failures naming the case, with tests.

## Low

6. **The emitted `// predicate:` comment quotes the whole cell** (every
   case's bodies, not only the case's own). **Kept on purpose**: the
   comment's contract is "the cell, verbatim", and narrowing it would
   change the artifact shape for every law row (byte-compatibility).

## Out of scope (observed, not this change's work)

7. The tracked `specodelic/verify.toml`, `specodelic/verify_props.rs`
   and `specodelic/linter-referential_integrity.*` artifacts are stale
   against the corpus — rows added by other tickets were never
   regenerated. A whole-corpus regeneration would have fixed them; those
   files were reverted rather than folded into this change.

