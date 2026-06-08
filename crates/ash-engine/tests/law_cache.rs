//! Integration tests for the `.ash/law-cache.toml` cache substrate.

use ash_engine::law_cache::{LawCache, LawCacheResult};

#[test]
fn law_cache_saves_and_loads_from_project_ash_directory() {
    let dir = tempfile::tempdir().unwrap();
    let mut cache = LawCache::new();

    cache.record_result(
        "monad_left_identity",
        "source-hash-1",
        LawCacheResult::Valid,
        Some(42),
    );
    cache.save_to_project_root(dir.path()).unwrap();

    let cache_path = dir.path().join(".ash/law-cache.toml");
    assert!(
        cache_path.exists(),
        "cache should be saved under .ash/law-cache.toml"
    );

    let loaded = LawCache::load_from_project_root(dir.path()).unwrap();
    let entry = loaded
        .lookup_current("monad_left_identity", "source-hash-1")
        .expect("matching source hash should load current law result");

    assert_eq!(entry.law_name, "monad_left_identity");
    assert_eq!(entry.source_hash, "source-hash-1");
    assert_eq!(entry.result, LawCacheResult::Valid);
    assert_eq!(entry.seed, Some(42));
    assert!(entry.timestamp_unix_secs > 0);
}

#[test]
fn law_cache_persists_broken_and_untested_results_across_runs() {
    let dir = tempfile::tempdir().unwrap();
    let mut cache = LawCache::new();

    cache.record_result("associativity", "hash-a", LawCacheResult::Broken, Some(7));
    cache.record_result("right_identity", "hash-b", LawCacheResult::Untested, None);
    cache.save_to_project_root(dir.path()).unwrap();

    let loaded = LawCache::load_from_project_root(dir.path()).unwrap();

    assert_eq!(
        loaded
            .lookup_current("associativity", "hash-a")
            .map(|entry| entry.result),
        Some(LawCacheResult::Broken)
    );
    assert_eq!(
        loaded
            .lookup_current("right_identity", "hash-b")
            .map(|entry| entry.result),
        Some(LawCacheResult::Untested)
    );
}

#[test]
fn law_cache_source_hash_mismatch_invalidates_stale_entry() {
    let mut cache = LawCache::new();
    cache.record_result(
        "associativity",
        "old-source-hash",
        LawCacheResult::Tested,
        Some(99),
    );

    assert!(
        cache.invalidate_if_source_changed("associativity", "new-source-hash"),
        "source hash mismatch should remove stale law result"
    );
    assert!(
        cache
            .lookup_current("associativity", "old-source-hash")
            .is_none()
    );
    assert!(
        cache
            .lookup_current("associativity", "new-source-hash")
            .is_none()
    );
}

#[test]
fn missing_law_cache_loads_as_empty_cache() {
    let dir = tempfile::tempdir().unwrap();

    let loaded = LawCache::load_from_project_root(dir.path()).unwrap();

    assert!(loaded.entries().is_empty());
}
