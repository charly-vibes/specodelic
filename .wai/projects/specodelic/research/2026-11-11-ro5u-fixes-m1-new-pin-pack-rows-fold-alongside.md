---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-8ebr-pack-check-rows, pipeline-step:fix-review]
---

RO5U-FIXES: (M1) new pin pack_rows_fold_alongside_oracle_rows — pack rows folded into a report CARRYING an oracle record with deliberately stale count maps; asserts recompute-from-scratch (stale/unknown seeded entries vanish, combined current+unchecked/permitted+agree correct, row-key order across kinds). (M2) PackViolation.message now surfaced: violation-row reason lists each finding as 'rule (message)' sorted+deduped (empty message degrades to bare rule id); evaluated_claim_ids stay plain deduped rule ids; test updated to pin the message verbatim. (L3) doc sentences on pack_check_rows + with_pack_rows: duplicate (pack_id,subject_id) keys are caller discipline, refused labeled downstream by the slice-1 --before prior-parse guard. Verification: cargo test --lib audit 13/13 (6 pack_row + 7 diff); cargo test --test audit_oracle_rows 14/14 (slice-1 CLI pins untouched, green).
