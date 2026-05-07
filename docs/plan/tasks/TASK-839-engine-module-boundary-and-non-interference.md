# TASK-839: Enforce module-local engine/import boundary and non-interference with existing semantic summaries

## Status: ✅ Complete

## Description

Enforce module-local engine/import boundary and non-interference with existing semantic summaries.

## Specification Reference

- [SPEC-061](../../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.5
- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)

## Dependencies

- ✅ Phase 112 / SPEC-060 complete.
- Depends on TASK-831 audit gate completion.
- Depends on TASK-838 source-equation normalizer integration completion.

## Dispatch

```
agent: hermes
provider: openai-codex
model: gpt-5.5
profile: default
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Objective

Enforce module-local engine/import boundary and non-interference with existing semantic summaries.

## Requirements

1. Verify ModuleFile/engine integration preserves local type-function definitions for same-module checking.
2. Reject or fence imported/public type-function normalization before SPEC-F.
3. Reject public ordinary aliases/signatures/interface surfaces that leak local computation heads before SPEC-F.
4. Prove ordinary type, sealed-domain, workflow, and normalizer fixture summaries remain non-regressed.

## Files

- Modify/create exact files identified by [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md) and the TASK-831 audit gate.
- Update `CHANGELOG.md` for completed implementation/tooling/docs-policy changes.

## TDD Steps

1. Write focused failing tests or docs/audit checks appropriate to task type.
2. Run the focused target and verify the expected failure or missing evidence.
3. Implement the minimal change for this task only.
4. Re-run the focused target and relevant non-regression tests.
5. Update docs/status evidence only after verification.

## Verification

```
strictness: clean
commands:
  - cargo test -p ash-engine --test task_839_type_function_module_boundary -- --nocapture
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Verify ModuleFile/engine integration preserves local type-function definitions for same-module checking.
  - [x] Reject or fence imported/public type-function normalization before SPEC-F.
  - [x] Reject public ordinary aliases/signatures/interface surfaces that leak local computation heads before SPEC-F.
  - [x] Prove ordinary type, sealed-domain, workflow, and normalizer fixture summaries remain non-regressed.
  - [x] focused tests/evidence recorded in this task file
  - [x] no SPEC-F/G/H scope creep
```


## Notes

Task type: Engine/Integration. Estimated effort: 5 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.

## Completion Evidence

- Added focused engine integration coverage in `crates/ash-engine/tests/task_839_type_function_module_boundary.rs` for:
  - ModuleFile metadata preserving local `type fn` definitions for same-module private alias checking.
  - public ordinary alias rejection when a local computation head leaks into exported representation.
  - public callable/workflow signature rejection when a local computation head leaks through the public surface.
  - imported semantic summaries continuing to transport ordinary public types while not serializing local type-function heads/equations.
- Preserved type-function declarations in `ash_parser::lower::LoweredTypeMetadata` for engine-local boundary checks without adding them to `ModuleSemanticSummary` export/import data.
- Added engine/module-loader public-boundary diagnostics for local type-function head leakage while keeping cross-module equation normalization unavailable before SPEC-F.

Verified commands:

```text
cargo test -p ash-engine --test task_839_type_function_module_boundary -- --nocapture
cargo test -p ash-engine --test task_785_modulefile_summary_exports -- --nocapture
cargo test -p ash-engine --test task_813_sealed_domain_non_interference -- --nocapture
cargo test -p ash-typeck --test task_838_type_function_normalizer -- --nocapture
cargo fmt --all --check
git diff --check
```
