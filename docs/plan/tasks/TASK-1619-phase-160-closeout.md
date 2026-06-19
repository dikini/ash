# TASK-1619: Close out Phase 160 with verification and documentation

## Status: 📝 Planned

## Description

Close out Phase 160 by running full verification gates, updating all status surfaces, and ensuring the phase is ready for handoff.

## Specification Reference

- [PLAN-160](../PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)

## Dependencies

- ✅ TASK-1610: Record/Tuple values
- ✅ TASK-1611: Field access primitives
- ✅ TASK-1612: Constructor tags
- ✅ TASK-1613: Match dispatch
- ✅ TASK-1614: Mutual recursion desugaring
- ✅ TASK-1615: Serde serialization extension
- ✅ TASK-1616: Speculative fixtures
- ✅ TASK-1617: Expanded operational semantics
- ✅ TASK-1618: Reference documentation

## Requirements

### Functional Requirements

1. Run full verification gates for all affected crates
2. Update CHANGELOG.md with Phase 160 entry
3. Update PLAN-INDEX.md with Phase 160 status
4. Verify all task files have correct status
5. Ensure no regressions in Phase 159 tests

## Verification Gates

```bash
# Full workspace tests
cargo test --all

# Clippy (all targets, all features)
cargo clippy --all-targets --all-features -- -D warnings

# Formatting
cargo fmt --check

# Documentation
cargo doc --no-deps

# Git diff check
git diff --check

# Phase 160 specific tests
cargo test -p ash-core -p ash-interp --test task_1610_cps_ir
cargo test -p ash-core -p ash-interp --test task_1611_cps_ir
cargo test -p ash-core -p ash-interp --test task_1612_cps_ir
cargo test -p ash-core -p ash-interp --test task_1613_cps_ir
cargo test -p ash-core -p ash-interp --test task_1614_cps_ir
cargo test -p ash-core -p ash-interp --test task_1615_cps_ir
cargo test -p ash-core -p ash-interp --test task_1616_cps_ir

# Speculative fixture execution
cargo test -p ash-interp --test task_1616_cps_ir -- --nocapture
```

## Status Surface Updates

### PLAN-INDEX.md

Add Phase 160 row to progress table:
```markdown
| [160](PLAN-160-CPS-IR-RUNTIME-EXPANSION.md) | 10 | 10 | ✅ Complete |
```

Add Phase 160 section with task table (all tasks marked ✅ Complete).

### CHANGELOG.md

Add under `[Unreleased]`:
```markdown
### Added
- Record and tuple values in CPS IR (TASK-1610)
- Field access primitives `RecordGet` and `TupleGet` (TASK-1611)
- Constructor name atoms for sum type discrimination (TASK-1612)
- Match dispatch term for pattern matching (TASK-1613)
- Mutual recursion desugaring support via tuple-of-lambdas (TASK-1614)
- Serde-based serialization extension for new forms (TASK-1615)
- Speculative test fixtures for upper-language lowering patterns (TASK-1616)
- Expanded operational semantics document (SPEC-099c) (TASK-1617)
- Reference documentation for expanded CPS IR (TASK-1618)
```

### Task Files

Update all TASK-161x files to `Status: ✅ Complete` with verification evidence.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo test --all
  - cargo clippy --all-targets --all-features -- -D warnings
  - cargo fmt --check
  - cargo doc --no-deps
  - git diff --check
checklist:
  - [ ] All tests pass
  - [ ] No clippy warnings
  - [ ] Formatting clean
  - [ ] Documentation builds
  - [ ] CHANGELOG.md updated
  - [ ] PLAN-INDEX.md updated
  - [ ] All task files marked complete
  - [ ] No regressions in Phase 159 tests
```

## Notes

- This is the final task of Phase 160. All previous tasks must be complete before starting.
- If any test fails, identify the failing task and reopen it for remediation.
- The closeout should produce a clean worktree ready for merge to main.
- Do not modify pre-existing Phase 159 files unless fixing a regression.
