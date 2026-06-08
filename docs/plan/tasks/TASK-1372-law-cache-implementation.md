# TASK-1372: Cache — `.ash/law-cache.toml` implementation

## Status: ✅ Complete

## Description

Implement dedicated law test result cache separate from `ash.lock`.

## Requirements

1. Create `LawCache` struct with:
   - Law name
   - Source hash
   - Result (valid/tested/broken/untested)
   - Seed and timestamp
2. Serialize to `.ash/law-cache.toml`
3. Deserialize on load
4. Invalidate on source hash mismatch

## Acceptance Criteria

- [x] Cache file created and readable
- [x] Results persist across runs
- [x] Invalidation works on source change
- [x] Test passes
- [x] No regressions

## Verification

- `cargo test -p ash-engine --test law_cache -- --nocapture` — 4 passed
- `cargo fmt --check` — passed
- `cargo clippy -p ash-engine --all-targets --all-features -- -D warnings` — passed
- `cargo check --workspace` — passed
- `git diff --check` — passed

## Completion Notes

- Added `ash_engine::law_cache` with `LawCache`, `LawCacheEntry`, `LawCacheResult`, and `LawCacheError`.
- Implemented `.ash/law-cache.toml` load/save with TOML serialization, missing-file-as-empty behavior, and `.ash` directory creation.
- Cached law entries record declared law name, source hash, result state, optional seed, and Unix timestamp.
- Added source-hash invalidation via `invalidate_if_source_changed` and current-only lookup via `lookup_current`.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1370](TASK-1370-runner-by-test-delegation.md)
