//! Pack discovery order-independence across repositories — property
//! `discovery_finds_all_packs` ("order-independent", specodelic-o5bh).
//! The fixtures live in `parse_misc.rs` (`write_pack_file`,
//! `write_fiber_kind_consumer`, `spk`).

use super::parse_misc::{write_fiber_kind_consumer, write_pack_file};
use super::*;
use crate::common::UnwrapPanic;

/// Discovery is order-independent across repositories (property
/// `discovery_finds_all_packs` — "order-independent"; specodelic-o5bh):
/// with the pack in repo P and its vocabulary used in repo C, both path
/// orders must discover the pack. The old last-path anchor
/// (`discovery_root`'s `.next_back()`) made `lint P C` fail with
/// `orphan_vocabulary` while `lint C P` passed.
#[test]
fn pack_discovery_is_order_independent_across_repos() {
    for order in [0usize, 1] {
        let dir = tempfile::tempdir().unwrap_p();
        let repo_p = dir.path().join("p");
        let repo_c = dir.path().join("c");
        std::fs::create_dir_all(&repo_p).unwrap_p();
        std::fs::create_dir_all(&repo_c).unwrap_p();
        for repo in [&repo_p, &repo_c] {
            std::process::Command::new("git")
                .args(["init", "-q", repo.to_str().unwrap_p()])
                .output()
                .unwrap_p();
        }
        write_pack_file(&repo_p.join("bioimage.md"), "bioimage", 14);
        write_fiber_kind_consumer(
            &repo_c.join("consumer.md"),
            "consumer",
            "",
            "bioimage.tolerance",
        );
        let (first, second) = if order == 0 {
            (repo_p.as_path(), repo_c.as_path())
        } else {
            (repo_c.as_path(), repo_p.as_path())
        };
        let out = spk()
            .args([
                "lint",
                first.to_str().unwrap_p(),
                second.to_str().unwrap_p(),
                "--json",
            ])
            .output()
            .unwrap_p();
        let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_p();
        assert_eq!(out.status.code(), Some(0), "order {order}: {json}");
        let issues = json["data"]["issues"].as_array().unwrap_p();
        assert!(
            !issues
                .iter()
                .any(|i| i["rule_id"] == "linter.orphan_vocabulary"),
            "order {order}: orphan_vocabulary must not fire when the pack repo is linted: {issues:?}"
        );
        let packs = json["data"]["packs"].as_array().unwrap_p();
        assert!(
            packs.iter().any(|p| p["id"] == "bioimage"),
            "order {order}: pack must be discovered in both path orders: {packs:?}"
        );
    }
}
