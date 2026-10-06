# guard-citation-semantics

Can the guard citation algebra (`[[a]] ∧ [[b]]`, `¬(...)`) — closed, typed (citations resolve to invariant-kind Constraints or States), used consistently across all 30 guard cells — acquire defined semantics and join the verified surface? Guards currently travel into TLA+ as annotations "never a verdict" (`src/model_check.rs:165`). The blocker `fragment_guard_rejected` (`specs/compile.md:30`) is fragment-scoped and must not be misread as blocking the citation algebra.
