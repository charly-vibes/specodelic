---
tags: [pipeline-run:tdd-ro5-2026-11-24-4yve-just-archive-change-accepts-the-id-id-invocation-form, pipeline-step:red]
---

RED: command=cargo test --test archive_change_recipe; expected failure=the archive-change recipe does not normalize an 'id=' prefix (no scripts/archive_change_id.sh, recipe body forwards the raw positional to the companion); observed=7 failed / 0 passed, all 7 failing on the missing normalizer (exit 127 'No such file or directory' for scripts/archive_change_id.sh; drift guard 'archive-change recipe must route the change id through scripts/archive_change_id.sh'), no setup breakage (test binary compiled, tempfile dev-dep present). Test ids: id_prefix_form_is_normalized, bare_form_is_preserved, empty_id_fails_with_remediation_hint, recipe_wires_the_normalizer, recipe_forwards_bare_id_to_companion, recipe_accepts_bare_positional_form, recipe_fails_on_empty_id_with_hint.
