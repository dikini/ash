# TASK-1822: Prove row requirements do not install authority

## Status: ✅ Complete

## Description

Add negative tests proving Phase 178 row summary and Core lowering work does not accidentally install providers, admission facts, handlers, host hooks, resources, roles, or workflow authority.

## Specification Reference

- [PLAN-178](../PLAN-178-SOURCE-TO-CORE-ROW-LOWERING-BRIDGE.md)
- [SPEC-096b](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-098c](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [NOTE-020](../../notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md)

## Dependencies

- TASK-1820 summary threading complete.
- TASK-1821 source-to-Core row lowering complete or partially implemented enough to test.

## Requirements

### Functional Requirements

1. Add negative tests proving operation row requirements do not create provider/capability authority.
2. Add negative tests proving role row requirements do not admit roles.
3. Add negative tests proving resource row requirements do not create resource ownership.
4. Add negative tests proving handler-looking row requirements do not install handler frames.
5. Add negative tests proving host/FFI-like operation rows do not call host hooks.
6. Add negative tests proving workflow/governance row requirements do not admit workflow authority.
7. Patch any leaked authority path found by these tests.

### Property Requirements

- Rows are requirements only.
- Authority/admission/provider/runtime installation must remain explicit and outside Phase 178.
- Negative tests must inspect actual runtime/admission/provenance surfaces where available, not just parse success.

## TDD Steps

### Step 1: Write failing non-authority tests

Add tests in `ash-engine`, `ash-typeck`, or `ash-core` according to the audited owner files.

### Step 2: Verify RED or existing PASS with coverage

If a test already passes, verify it truly covers the relevant authority surface. Otherwise confirm the failure demonstrates leakage.

### Step 3: Patch leaks if found

Patch only the leaking authority/admission path. Do not implement provider/admission wiring.

### Step 4: Verify GREEN

Run focused tests and affected crate tests.

## Dispatch

```yaml
agent: codex
reasoning: medium
toolsets: [terminal, file, rust-analyzer]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-engine
  - cargo test -p ash-typeck
  - cargo test -p ash-core
  - git diff --check
checklist:
  - [x] Operation rows do not grant provider/capability authority.
  - [x] Role rows do not admit roles.
  - [x] Resource rows do not create ownership.
  - [x] Handler/admission/host/workflow authority remains explicit and absent.
```

## Completion Evidence

- Added `crates/ash-engine/tests/task_1822_row_authority_neutrality.rs`.
- Verified row requirements do not register providers, select resources, select capability implementations, install runtime modules, fabricate workflow authority summaries, admit roles/capabilities, or call host observe/execute hooks during parse/check/execute.
- Verification: `cargo test -p ash-engine --test task_1822_row_authority_neutrality -- --nocapture`, `cargo test -p ash-engine`, `cargo test -p ash-typeck`, `cargo test -p ash-core`, `cargo fmt --check`, `git diff --check`, and `python3 tools/docs/validate_orientation_indexes.py --self-test`.

## Dependencies for Next Task

This task feeds TASK-1823 and TASK-1825.
