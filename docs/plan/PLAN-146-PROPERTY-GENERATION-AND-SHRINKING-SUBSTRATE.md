# PLAN-146: Property Generation and Shrinking Substrate

**Status:** ✅ Complete
**Spec:** [SPEC-082: Property Generation and Shrinking Substrate](../spec/SPEC-082-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md)
**Depends on:** [PLAN-145: Law Test Evidence Substrate](PLAN-145-LAW-TEST-EVIDENCE-SUBSTRATE.md)
**Task range:** TASK-1456 through TASK-1465
**Estimated effort:** 40h

## Overview

Build the generator, binding, counterexample, and shrinking substrate that makes `ash test` property evidence useful from an Ash executable without Cargo/Rust tooling in the author/executor path.

## Goals

- [ ] Make generated property cases bind into laws and authored property tests.
- [ ] Produce stable counterexample/replay artifacts.
- [ ] Add deterministic shrinking for first-slice value families.

## Non-Goals

- coverage reporting
- mutation testing
- flake quarantine
- distributed orchestration
- proof-producing synthesis
- unbounded arbitrary source-world generation

## Orchestrator Guidance

- Create a dedicated worktree before implementation, for example `.worktrees/phase-146-property-generation-and-shrinking-substrate`.
- Load `rust-skills`, `ash-language-feature-spec-writing`, `test-driven-development`, `systematic-debugging`, and `verification-before-completion` before code work.
- Use rust-analyzer MCP/LSP for Rust symbol tracing before broad text search.
- Keep tasks small and sequential where schema/result formats are dependencies.
- Require direct `$ASH_UNDER_TEST test ...` evidence for user-facing runner behavior; Rust tests alone are bridge evidence.
- Update `CHANGELOG.md` and relevant `reference/tools/test.md` wording in the closeout task.

## Task Plan

| Task | Title | Estimate | Status |
|---|---|---:|---|
| [TASK-1456](tasks/TASK-1456-property-generation-shrinking-audit.md) | Audit current property generation and shrinking gaps | 4h | 📝 Planned |
| [TASK-1457](tasks/TASK-1457-generator-schema-and-binding-model.md) | Define generator schema and binding model | 4h | 📝 Planned |
| [TASK-1458](tasks/TASK-1458-primitive-property-generators.md) | Implement primitive property generators | 4h | 📝 Planned |
| [TASK-1459](tasks/TASK-1459-adt-container-property-generators.md) | Implement ADT/container property generators | 4h | 📝 Planned |
| [TASK-1460](tasks/TASK-1460-authored-property-binding-injection.md) | Inject generated bindings into Ash-authored property tests | 4h | 📝 Planned |
| [TASK-1461](tasks/TASK-1461-counterexample-artifact-schema.md) | Add counterexample artifact schema | 4h | 📝 Planned |
| [TASK-1462](tasks/TASK-1462-primitive-shrinker-core.md) | Implement primitive shrinking core | 4h | 📝 Planned |
| [TASK-1463](tasks/TASK-1463-adt-container-shrinking.md) | Implement ADT/container shrinking | 4h | 📝 Planned |
| [TASK-1464](tasks/TASK-1464-property-shrinking-final-surface-fixtures.md) | Add no-Cargo property/shrinking fixtures | 4h | 📝 Planned |
| [TASK-1465](tasks/TASK-1465-property-generation-shrinking-closeout.md) | Close out property generation/shrinking phase | 4h | 📝 Planned |

## Decision Gates

- D1: generator schema lands before generator implementations.
- D2: counterexample artifact format lands before shrinkers.
- D3: no task may claim shrinking until direct `$ASH_UNDER_TEST` evidence shows a smaller replayable case.

## Verification Strategy

Each implementation task must include:

1. Focused Rust tests for new parser/runner/schema behavior.
2. Focused Ash fixture tests where the behavior is user-facing.
3. Direct Ash-under-test command evidence:

   ```bash
   export ASH_UNDER_TEST=/absolute/path/to/candidate/ash
   "$ASH_UNDER_TEST" test fixtures/phase146-... --format json
   ```

4. `cargo fmt --check`, focused `cargo test`, and focused `cargo clippy` for touched crates.

The closeout task owns broad gates and documentation drift checks.
