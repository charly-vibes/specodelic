//! SPECODELIC managed block — `spk init` self-registration in consumer repos.
//!
//! Purpose: let `spk` write (and keep current) a `<!--
//! SPECODELIC:START/END -->` block in a repo's `AGENTS.md` so agents
//! working there get the spec rules without reading this repo. The block
//! carries the lint rule catalog (rendered from the same [`RULE_TABLE`]
//! findings name, so they can never disagree), the embedded
//! `format_revision`, and the core commands. Responsibilities: own the
//! block definition + content rendering + inject/read; the `spk init` CLI
//! verb and the `spk doctor` currency check sit in `main.rs` and call
//! into here. Rationale: this is the same managed-block convention wai
//! and espectacular already use (genesis::managed_block) — one injector,
//! marker-guarded, idempotent updates in place.

use genesis::managed_block::{BlockDef, BlockInjector, BlockRegistry, InjectResult};

/// The block name — markers are `<!-- SPECODELIC:START -->` /
/// `<!-- SPECODELIC:END -->` (genesis `BlockDef::new` convention).
pub const BLOCK_NAME: &str = "SPECODELIC";

/// The file `spk init` manages by default (repo-level agent standing
/// instructions — same file wai/openspec manage blocks in).
pub const BLOCK_FILE: &str = "AGENTS.md";

/// Build the injector with the SPECODELIC block registered.
pub fn injector() -> BlockInjector {
    let mut registry = BlockRegistry::new();
    registry.register(BlockDef::new(BLOCK_NAME));
    BlockInjector::new(registry)
}

/// Render the block content: what an agent in this repo must know to
/// write lint-clean specs, derived entirely from closed sets + constants
/// (no free-prose drift possible).
pub fn block_content() -> String {
    use crate::guide::FORMAT_REVISION;
    use crate::lint::RULE_TABLE;

    let rules: Vec<String> = RULE_TABLE
        .iter()
        .map(|(name, semantics)| format!("- `{}` — {}", crate::lint::rule_id(name), semantics))
        .collect();

    format!(
        r#"
## Specodelic — spec format rules (managed block)

This repo's `specs/`-style markdown spec files (YAML frontmatter +
fixed-schema tables) are linted by `spk` (crates.io: specodelic).
Write specs so `spk lint` passes; embedded format revision: {rev}

### Lint rules (every violation names its `rule_id`)

{rules}

### Commands

- `spk lint <dir>` — check the invariants (fails with a hint on zero files)
- `spk graph <dir>` — typed reference graph (state edges, typing
  violations, supersedes cycles; blast-radius lands later)
- `spk compile <files>` — emit TOML / proptest / TLA+ artifacts
- `spk model-check <files>` — run the model checker against compiled
  output (stateright; reports land as `*.check.json`)
- `spk explain [topic]` — the embedded format primer (works offline)
- `spk doctor` — diagnose workspace + block currency
- `spk feedback bug --dry-run` — file an issue against upstream

Refresh this block after upgrading: `spk init --force`.
"#,
        rev = FORMAT_REVISION,
        rules = rules.join("\n"),
    )
}

/// Inject (or refresh) the block into `path`. Thin wrapper so main.rs
/// maps [`InjectResult`] to envelope data + never touches non-block
/// content.
pub fn inject_into(path: &std::path::Path) -> std::io::Result<InjectResult> {
    injector().inject(path, BLOCK_NAME, &block_content())
}

/// Whether `path` carries the block.
pub fn has_block(path: &std::path::Path) -> bool {
    injector().has_block(path, BLOCK_NAME)
}

#[cfg(test)]
mod discovery_tests {
    use super::*;
    use genesis::discovery;

    // -- specodelic-las: spk init registers in .genesis/tools.toml -------

    #[test]
    fn register_creates_manifest_with_specodelic_entry() {
        let dir = tempfile::tempdir().unwrap();
        genesis::discovery::register(
            dir.path(),
            "specodelic",
            "Specodelic spec format linter + pipeline (spk)",
            "file",
            "AGENTS.md",
        )
        .unwrap();
        let manifest = discovery::read_manifest(dir.path()).expect("manifest written");
        let entry = manifest.tools.get("specodelic").expect("entry present");
        assert_eq!(entry.detector.detector_type, "file");
        assert_eq!(entry.detector.path, "AGENTS.md");
    }

    #[test]
    fn register_is_idempotent_and_merges_with_siblings() {
        let dir = tempfile::tempdir().unwrap();
        // a sibling tool registered first
        discovery::register(dir.path(), "wai", "Workflow manager", "directory", ".wai").unwrap();
        // register specodelic twice
        for _ in 0..2 {
            genesis::discovery::register(
                dir.path(),
                "specodelic",
                "Specodelic spec format linter + pipeline (spk)",
                "file",
                "AGENTS.md",
            )
            .unwrap();
        }
        let manifest = discovery::read_manifest(dir.path()).unwrap();
        assert_eq!(
            manifest.tools.len(),
            2,
            "merge, never clobber: {manifest:?}"
        );
        assert!(manifest.tools.contains_key("wai"));
        assert!(manifest.tools.contains_key("specodelic"));
        // scan sees specodelic detected via its marker
        let tools = discovery::scan(dir.path());
        let spk = tools.iter().find(|t| t.name == "specodelic").unwrap();
        // detector: AGENTS.md must exist for `detected: true`
        std::fs::write(dir.path().join("AGENTS.md"), "x").unwrap();
        let tools = discovery::scan(dir.path());
        let spk = tools.iter().find(|t| t.name == "specodelic").unwrap();
        assert!(spk.detected, "AGENTS.md present → detected");
    }
}

/// The block's declared format revision, parsed from its content —
/// `None` when the block is absent or names no revision. Scans every
/// block line: the revision may appear in a heading or in prose
/// ("embedded format revision: specodelic.md Revision 8").
pub fn block_format_revision(path: &std::path::Path) -> Option<u32> {
    let content = injector().read_block(path, BLOCK_NAME)?;
    content
        .lines()
        .filter_map(crate::guide::revision_number)
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_content_lists_every_rule_and_revision() {
        let content = block_content();
        for (name, _) in crate::lint::RULE_TABLE {
            assert!(content.contains(&crate::lint::rule_id(name)));
        }
        assert!(content.contains(crate::guide::FORMAT_REVISION));
        assert!(content.contains("spk lint"));
        // the command catalog stays current: every implemented pipeline
        // command is advertised to consumer-repo agents
        assert!(content.contains("spk model-check"));
    }

    #[test]
    fn inject_is_idempotent_and_refreshes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("AGENTS.md");
        std::fs::write(&path, "# My repo\n").unwrap();
        assert!(matches!(
            inject_into(&path).unwrap(),
            InjectResult::Prepended
        ));
        assert!(has_block(&path));
        // refresh: updated in place, single block, content preserved
        assert!(matches!(inject_into(&path).unwrap(), InjectResult::Updated));
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text.matches("SPECODELIC:START").count(), 1);
        assert!(text.contains("# My repo"));
        // currency parse: the block names the current revision
        assert_eq!(
            block_format_revision(&path),
            Some(crate::guide::revision_number(crate::guide::FORMAT_REVISION).unwrap())
        );
    }
}
