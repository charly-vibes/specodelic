# os1x-shape fixture

Snapshot of the `/tmp/os1x` matrix (add-delta-layer-sidecar tasks.md 5.2):
a self-contained mini-corpus used to assert the three post-Revision-19
shapes are strict-valid under the openspec 1.14.x validator — the
validator whose dual-delta complaints forced the 0.19.0 pin
(specodelic-uv3v; task 6 retires the pin).

Layout (relative paths are the assertion surface):

- `openspec/specs/cap-a/spec.md` — deployed dual spec (frontmatter +
  Constraints/Model/Properties tables + `## Requirements` mirror).
- `openspec/changes/t-delta/specs/cap-b/spec.md` — plain delta +
  `## Purpose`, no frontmatter, no mirror.
- `openspec/changes/t-delta/specs/cap-a/layer.md` — the sidecar layer
  (`id: cap.a.layer`, parent-dir-derived naming arm).
- `openspec/changes/t-delta/specs/cap-a/spec.md` — plain delta next to
  its sidecar (the 5.1 conversion shape).

Deviation from the volatile source matrix: its `cap-a` change delta was
still the pre-Revision-19 dual delta, which 1.14.x strict REJECTS (the
very reason the pin existed). The checked-in fixture shows that delta in
its post-5.1 plain+sidecar shape instead, with SHALL-bearing requirement
text, so all items validate clean. The gate lives in
`scripts/test_os1x_shape_fixture.py` (runs via `just sync-sections-test`).
