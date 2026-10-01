//! Update-availability notice for `spk doctor` (specodelic-4le).
//!
//! Purpose: surface newer specodelic releases on crates.io as an
//! advisory warning on the doctor envelope's warnings channel — the same
//! advisory discipline as the knowledge-currency warning. Responsibilities:
//! one call ([`update_notice`]) mapping a genesis
//! `update_check::UpdateInfo` to the notice string (or `None` when
//! current / uncheckable).
//!
//! Rationale: binaries, not libs, notify (genesis-2ex contract) — the
//! check carries our own crate name + version. `doctor` is the only
//! wired surface: a startup-time check would tax every invocation; the
//! doctor runs rarely and is advisory by nature. All transport failures
//! are silent (`check_with` returns `None`) — a network problem can
//! never fail the doctor or clutter its output.

use genesis::update_check;
use std::path::Path;

/// The doctor's update-availability warning, when a newer stable
/// release exists. Advisory mapping only: `None` when the installed
/// version is current (genesis already returns `None` for that case at
/// check time — this re-check guards direct calls with hand-built infos).
pub fn update_notice(info: &update_check::UpdateInfo) -> Option<String> {
    if info.latest == info.current {
        return None;
    }
    Some(update_check::notice(info))
}

/// Run the availability check itself — production entry point.
///
/// Reads/writes `<cache_dir>/specodelic.json` (7-day TTL) and contacts
/// crates.io only on a stale/absent cache. Every failure mode
/// (offline, rate limit, bad response) is swallowed by the genesis
/// contract: `None`, silently.
pub fn check_for_update(cache_dir: &Path) -> Option<update_check::UpdateInfo> {
    update_check::check_with(
        "specodelic",
        env!("CARGO_PKG_VERSION"),
        cache_dir,
        update_check::CRATES_IO_API,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn now_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Seed a FRESH cache entry — genesis's fresh-cache short-circuit
    /// answers without any HTTP, making every test here hermetic.
    ///
    /// Genesis semantics: `latest == current` → `None` ("up to date"),
    /// `latest != current` → `Some(UpdateInfo)` — asserted here.
    fn seed_update(cache_dir: &Path, latest: &str) -> Option<update_check::UpdateInfo> {
        let cache_dir = cache_dir.join("cache");
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::write(
            cache_dir.join("specodelic.json"),
            format!(
                r#"{{"checked_at": {}, "latest": "{latest}", "published_at": null, "ttl_secs": 604800}}"#,
                now_secs()
            ),
        )
        .unwrap();
        check_for_update(&cache_dir)
    }

    // -- notice construction (hermetic — cache seeding, no network) --------

    #[test]
    fn update_notice_rides_warnings_when_newer_version_exists() {
        let dir = tempfile::tempdir().unwrap();
        let info = seed_update(dir.path(), "9.9.9").expect("newer version → Some");
        let warning = update_notice(&info);
        assert!(warning.is_some(), "newer version → warning");
        let msg = warning.unwrap();
        assert!(msg.contains("9.9.9"), "names the newer version: {msg}");
        assert!(msg.contains("cargo install"), "actionable: {msg}");
    }

    #[test]
    fn no_notice_when_current() {
        let dir = tempfile::tempdir().unwrap();
        // genesis design: latest == current → None ("up to date"),
        // BEFORE notice() is ever consulted
        assert!(
            seed_update(dir.path(), env!("CARGO_PKG_VERSION")).is_none(),
            "current version → no UpdateInfo at all"
        );
    }

    #[test]
    fn transport_failure_is_silent_never_an_error() {
        // unreachable API base (nothing listens on port 1) → None, no
        // panic, no error — the doctor must never fail on the network
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("cache");
        std::fs::create_dir_all(&cache).unwrap();
        let info = update_check::check_with(
            "specodelic",
            env!("CARGO_PKG_VERSION"),
            &cache,
            "http://127.0.0.1:1/api/v1/crates",
        );
        assert!(info.is_none(), "offline → silent None");
    }

    #[test]
    fn stale_cache_triggers_backoff_write_not_error() {
        // a stale cache entry with an unreachable API: check re-issues
        // the fetch, fails silently, writes a backoff entry — next call
        // short-circuits on the backoff (also hermetic, zero HTTP after)
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("cache");
        std::fs::create_dir_all(&cache).unwrap();
        let stale = now_secs() - 604800 - 1; // older than the 7-day TTL
        std::fs::write(
            cache.join("specodelic.json"),
            format!(
                r#"{{"checked_at": {stale}, "latest": "0.0.1", "published_at": null, "ttl_secs": 604800}}"#
            ),
        )
        .unwrap();
        let info = update_check::check_with(
            "specodelic",
            env!("CARGO_PKG_VERSION"),
            &cache,
            "http://127.0.0.1:1/api/v1/crates",
        );
        assert!(info.is_none(), "transport failure → None");
        // backoff entry persisted (genesis stale-cache preservation: the
        // prior latest rides along) — the next call reads it fresh
        let entry: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(cache.join("specodelic.json")).unwrap())
                .unwrap();
        let checked_at = entry["checked_at"].as_u64().unwrap();
        assert!(
            checked_at > stale,
            "checked_at advanced to the backoff write"
        );
    }
}
