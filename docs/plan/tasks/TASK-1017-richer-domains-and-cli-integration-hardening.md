# TASK-1017: Richer Domains and CLI Integration Hardening

## Status: Planned

## Description

Add richer finite domain families and harden synthesized/small-world CLI controls across ordinary source, structured snapshot, generated property, and world execution paths.

## Specification Reference

- [SPEC-077](../../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
- [PLAN-127](../PLAN-127-DESIGN-022-023-SYNTHESIZED-SMALLWORLD-COMPLETION.md)
- [DESIGN-023](../../design/DESIGN-023-SMALL-WORLD-EXPLORATION-SUBSTRATE.md)

## Requirements

1. Add bounded product and bounded list domains with explicit size caps.
2. Add role/capability inclusion-set worlds where metadata exposes finite roles/capabilities.
3. Add policy-context and obligation lifecycle state-machine descriptors where metadata is stable.
4. Preserve fail-closed behavior for uncapped or open domains.
5. Verify filters, source selection, fail-fast, seed, max-cases, max-worlds, timeout, human output, and JSON output across synthesized paths.

## TDD Steps

- RED: Add failing tests for each new finite domain and CLI integration behavior.
- GREEN: Implement domain enumeration and CLI behavior incrementally.

## Dispatch

Use direct implementation or sub-agents according to the active controller instruction for that session.

## Verification

- Focused domain and CLI tests.
- `cargo fmt --check`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli --test test_command -- --nocapture`
- `git diff --check`

## Completion Checklist

- [ ] Richer finite domains implemented with safe caps.
- [ ] Open domains defer before materialization.
- [ ] CLI behavior is consistent across synthesized paths.
- [ ] RED/GREEN evidence recorded.
