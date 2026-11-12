---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-q7uo-compile-pipe-unescape, pipeline-step:fix-review]
---

RO5U-FIXES: none required — zero Critical/High/Medium. Low 1 (backslash-pair-before-pipe inside a fragment string literal) is already documented in unescape_cell_pipes's doc comment and matches packs.rs's accepted ESC semantics; fixing it would require escape-sequence grammar the parser does not define — deferred of record. Low 2 (guard/emits cells) is out of the ticket's fragment-emission scope — deferred to a future ticket if the citation grammar ever grows pipes.
