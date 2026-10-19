# Tasks: add-delta-layer-sidecar

## 1. Format Revision 19 (specodelic layer of record)

- [ ] 1.1 Extend the naming law in `openspec/specs/specodelic/spec.md`
      (`id_matches_file` constraint + new `## Revision 19` section): a
      file named `layer.md` derives its expected id from its PARENT
      DIRECTORY name plus a `.layer` suffix (`specs/cap-a/layer.md` →
      `id: cap.a.layer`); the change-delta shape statement (plain delta +
      sidecar) records the dual-format delta class as retired.
- [ ] 1.2 Bump `FORMAT_REVISION` in the embedded guide (drift guard test
      fails first — RED); refresh the guide primer text that teaches the
      dual-format delta (wrap recipe → wrap-into-sidecar recipe).
- [ ] 1.3 Refresh the repo-root `AGENTS.md` managed block (`spk init
      --force`) and update the espectacular sibling-constraint bullet (the
      sanctioned write path for capability specs becomes: openspec archive
      with spec application + companion layer re-attachment); update the
      `openspec/USAGE.md`/primer wording that still teaches the mirror.

## 2. Lint-rule retirement (TDD: the retired shapes must be unwritable)

- [ ] 2.1 RED: fixture files carrying a delta-section + `## Requirements`
      mirror pairing report NO `requirement_drift`/`dual_format_valid`
      findings after retirement; GREEN: remove the delta-pairing arms of
      `dual_format_valid` and `requirement_drift` from the linter (deployed
      dual-format capability specs keep every other rule — corpus lint
      stays 0 findings).
- [ ] 2.2 Update the embedded `spk explain` topic bodies that describe the
      retired rules; membership gates (`check_membership`, skill-pack
      topics) updated to the new rule set.

## 3. migrate: wrap into sidecar

- [ ] 3.1 RED: `tests/migrate.rs` — wrap of a plain delta writes a
      sibling `layer.md` (parent-dir-derived `.layer` id, scaffold rows)
      and leaves the delta byte-unchanged; GREEN: rework `src/migrate.rs`
      wrap path (mirror generation removed, sidecar generation added).
- [ ] 3.2 Refusal classes updated: frontmatter-carrying files, deployed
      specs, and heading-less files refuse with exit 2; partially-migrated
      pairs pass existing sidecars through verbatim.
- [ ] 3.3 `--dry-run` envelope carries the sidecar content; `--rekey`
      behavior unchanged (existing tests stay green).

## 4. archive-companion: apply + re-attach

- [ ] 4.1 RED: `tests/archive_companion.rs` — a plain delta + sidecar
      change deploys a dual-format spec (applied `## Requirements` +
      sidecar frontmatter/tables) and the archived delta stays plain;
      GREEN: rework `src/archive_companion.rs` — invoke `openspec archive`
      with spec application, then re-attach the layer from the archived
      sidecar.
- [ ] 4.2 Fail-closed guards: sidecar-less or unlintable sidecar refuses
      BEFORE the openspec invocation (destructive); post-attach shape
      assertion (no delta headers, frontmatter + tables present, lints
      clean) reverts and fails on violation.
- [ ] 4.3 Idempotent rerun + newest-archive-wins + `--dry-run` re-verified
      against the new assembly (convergence to identical deployed bytes).

## 5. Corpus migration (the 3 remaining dual deltas)

- [ ] 5.1 Convert `add-graph-views`, `add-py-fragment-emission`,
      `add-ts-fragment-emission` deltas to plain deltas + `layer.md`
      sidecars (layer content = existing delta frontmatter/tables
      verbatim; requirement text byte-preserved from the current mirror);
      scenario headers stay verbatim (ah contract bindings).
- [ ] 5.2 Check in the 1.14.2 shape fixture (recreate the `/tmp/os1x`
      matrix: deployed dual spec, plain delta + Purpose, sidecar layer) as
      a checked-in fixture asserting 1.x-strict-validity of all three.
- [ ] 5.3 `openspec validate --all --strict` (1.14.x): 51/51 items pass
      with zero warnings; `spk lint openspec`: 0 findings; `just ci` green.

## 6. Validate-gate retirement (the pin was only for the dual shape)

- [ ] 6.1 Collapse `scripts/spec_validate.py` to a plain 1.x invocation
      (no version guard, no 0.19 npx fallback); justfile `openspec-validate`
      target and ci.yml wiring updated; `tests/ci_wiring.rs` and
      `tests/test_spec_validate.py` updated (TDD: gate test names 1.x as
      the validator first). The DETERMINISTIC version pin survives — ci.yml
      line 49 installs `@fission-ai/openspec@0.19.0` globally; bump that
      install to the pinned 1.x version (and document the dev-machine
      install requirement) so the gate runs the same validator everywhere.
- [ ] 6.2 Decision record `openspec/decisions/2026-10-DD-validate-pin-retired.md`
      superseding 2026-10-19-openspec-validate-pin.md; GH-20 updated with
      the landing note (org-wide sweep unblocked).

## 7. Archive the pending plain-shaped changes

- [ ] 7.0 Reconcile the cross-change MODIFIED conflict: the pending
      `update-id-derivation-spec-md` delta for spec-integration restates
      "Dual-format delta" with the RETIRED mirror-pairing wording ("pairing
      its carried delta sections with a sibling `## Requirements` mirror");
      if it archives after this change, openspec apply would deploy that
      wording over the new one. Rewrite its delta block to the Revision 19
      wording (or fold it into this change's delta) before it archives.
- [ ] 7.1 Land `update-id-derivation-spec-md` and `add-migrate-rekey`
      through the new companion (their deltas are already Stage-2 shaped);
      post-archive `openspec validate --all --strict` + `spk lint openspec`
      stay green.
- [ ] 7.2 Mirror-wording residue sweep: `openspec/specs/hooks/spec.md`
      ("deltas not yet migrated to dual" advisory),
      `openspec/specs/verification-claims/spec.md` deploy transition guard
      ("dual-format archive preserves every requirement"), and the docs
      pages (`docs/src/tool-relationships.md` "Why not plain openspec
      archive?", `docs/src/commands.md` archive section) reworded to the
      plain-delta + sidecar shape; `ah check` stays green (contract TOMLs
      bind scenario slugs, not constraint prose).
- [ ] 7.3 Close specodelic-s8fg with the meter trail; note
      specodelic-dlf/3le9 as folded-in (close them).
