---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-edrv-binds-nothing, pipeline-step:orient]
---

ORIENT: specodelic-edrv audit slice 3 binds_nothing — tasks 3.1 red + 3.2 green only. Read design.md (binds_nothing on warnings channel decision: advisory-only, file-scoped, never exit-nonzero), tasks.md slice 3, delta spec §Audit honesty channels (3 scenarios), delta layer rows binds_respect_sparse + honesty_channels, p_sparse + p_honesty. src/audit.rs at HEAD: slices 1-2 landed (audit_row/recount/sort_rows unified emission; AuditClass={Current,Stale,Unchecked,Contradicted,New}); binds_nothing NOT yet in enum. Citation-vs-declared-ids: substring-FREE whole-id match, cheap check, no graph fan-in (deferred open question). Supersedes model-binding half of specodelic-h5ks (catitas A2 vacuity advisories; bowling rev1 green-yet-vacuous) — h5ks invariant side stays separate, do not touch.
