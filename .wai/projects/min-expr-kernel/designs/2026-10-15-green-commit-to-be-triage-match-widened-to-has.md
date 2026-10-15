---
tags: [pipeline-run:tdd-ro5-2026-10-15-specodelic-nvs9-migrate-wraps-modified-only-deltas-gh-16, pipeline-step:green]
---

GREEN: commit-to-be — triage match widened to (has_frontmatter, has_added||has_modified, has_mirror); copy anchor = ADDED when present else MODIFIED (fallback to whole-body span when the section precedes the cursor — removes a latent panic, no test change); mirror parts built conditionally: ADDED body byte-exact when present, MODIFIED body in modified form (54v single builder reused, conflict loop only has teeth vs ADDED). MODIFIED-only mirror keeps its own leading newline (byte-exact slice like the ADDED body) so the blank line under ## Requirements matches the ADDED convention. NoDeltaSection/NotADelta messages now name both accepted sections. just test: 38 result-ok lines, 0 failures. Narrow: cargo test --test cli -- migrate → 10 passed.
