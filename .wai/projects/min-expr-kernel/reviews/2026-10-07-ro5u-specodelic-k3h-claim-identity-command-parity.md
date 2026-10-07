# RO5U review — specodelic-k3h §3.8 TIDY (claim identity + command parity)

Scope reviewed: src/kernel.rs (from_tlc_presence, merge_corpus_statuses, header),
src/commands/model_check.rs, src/orchestrate.rs (call-site slimming),
tests/claim_parity.rs (3 characterization fixtures).

## Critical
- none.

## High
- none.

## Medium
- M1 kernel.rs now borrows citation_corpus::ResolvedStatus in
  merge_corpus_statuses (cross-module shape coupling). Accepted: citation_corpus.rs
  is outside the ticket's allowed-file scope; kernel.rs is the sanctioned home per
  the mui deviation. Fix shape for later: shared ClaimIdentity trait once
  citation_corpus.rs is editable.
- M2 struct-shape mirror (ResolvedStatus vs CorpusClaimStatus) remains at the
  field level. Deferred with reason → beads specodelic-dzn (P3).

## Low
- L1 from_tlc_presence(bool) bare-bool param; call sites read fine. Tracked.
- L2 claims iterated twice (push + reasons map) to preserve the historical
  chain order exactly. Tracked; negligible.

## Behavior-preservation evidence
- BTreeMap insert order identical (resolved then claims; later wins).
- statuses_json built from the same post-merge invariant_statuses.
- exec/citation/claim append order unchanged; persisted reports byte-identical
  across CLI and orchestrate paths (pinned by claim_parity fixtures).
- Deliberate-broken intermediate (orchestrate kernel-claim push removed) failed
  all 3 parity tests → recorded → restored.

## Verdict: PASS
