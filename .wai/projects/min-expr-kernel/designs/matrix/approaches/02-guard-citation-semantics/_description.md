# Guard citation semantics (near-term slice only)

Give the existing guard citation algebra (`[[a]] ∧ [[b]]`, `¬(...)`) defined semantics: a transition is enabled at state s iff s ∈ ⟦guard⟧ (meet in the subobject lattice Sub(S)), compiled into the TLA+ module instead of a comment. Nothing else changes — no kernel grammar, no data-dependent language. The smallest unverified thing that can become verified; does not touch the pc-model's data representation (`fragment_guard_rejected` is fragment-scoped, `specs/compile.md:30`).
