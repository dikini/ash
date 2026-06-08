# TASK-1372: Cache — `.ash/law-cache.toml` implementation

## Status: 📝 Planned

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

- [ ] Cache file created and readable
- [ ] Results persist across runs
- [ ] Invalidation works on source change
- [ ] Test passes
- [ ] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1370](TASK-1370-runner-by-test-delegation.md)
