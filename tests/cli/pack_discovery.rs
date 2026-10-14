//! Pack discovery order-independence across repositories — property
//! `discovery_finds_all_packs` ("order-independent", specodelic-o5bh).
//! The fixtures live in `parse_misc.rs` (`write_pack_file`,
//! `write_fiber_kind_consumer`, `spk`).

use super::parse_misc::{write_fiber_kind_consumer, write_pack_file};
use super::*;

/// Discovery is order-independent across repositories (property
/// `discovery_finds_all_packs` — "order-independent"; specodelic-o5bh):
/// with the pack in repo P and its vocabulary used in repo C, both path
/// orders must discover the pack. The old last-path anchor
/// (`discovery_root`'s `.next_back()`) made `lint P C` fail with
/// `orphan_vocabulary` while `lint C P` passed.
#[test]
fn pack_discovery_is_order_independent_across_repos() {
    for order in [0usize, 1] {
        let dir = tempfile::tempdir().unwrap();
        let repo_p = dir.path().join("p");
        let repo_c = dir.path().join("c");
        std::fs::create_dir_all(&repo_p).unwrap();
        std::fs::create_dir_all(&repo_c).unwrap();
        for repo in [&repo_p, &repo_c] {
            std::process::Command::new("git")
                .args(["init", "-q", repo.to_str().unwrap()])
                .output()
                .unwrap();
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
                first.to_str().unwrap(),
                second.to_str().unwrap(),
                "--json",
            ])
            .output()
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(out.status.code(), Some(0), "order {order}: {json}");
        let issues = json["data"]["issues"].as_array().unwrap();
        assert!(
            !issues
                .iter()
                .any(|i| i["rule_id"] == "linter.orphan_vocabulary"),
            "order {order}: orphan_vocabulary must not fire when the pack repo is linted: {issues:?}"
        );
        let packs = json["data"]["packs"].as_array().expect("packs surfaced");
        assert!(
            packs.iter().any(|p| p["id"] == "bioimage"),
            "order {order}: pack must be discovered in both path orders: {packs:?}"
        );
    }
}
