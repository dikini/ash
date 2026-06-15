# SPEC-082: Property Generation and Shrinking Substrate

**Status:** Planned
**Date:** 2026-06-15
**Builds on:** [SPEC-081](SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md)
**Plan:** [PLAN-146: Property Generation and Shrinking Substrate](../plan/PLAN-146-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md)

## Summary

Build the generator, binding, counterexample, and shrinking substrate that makes `ash test` property evidence useful from an Ash executable without Cargo/Rust tooling in the author/executor path.

## Motivation

Phase 145 made empirical law evidence explicit but intentionally left several important `ash test` gaps visible. This specification defines the next scoped slice for those gaps while preserving the project rule that Ash law/test/proof authors and executors validate supported behavior through an Ash executable, not through Cargo or Rust test harnesses.

## Evidence Boundary

Implementation agents must distinguish three command classes:

1. **Implementation health** — Rust commands such as `cargo test`, `cargo clippy`, and `cargo fmt`; useful and required for implementers.
2. **Candidate Ash final surface** — direct invocations of an Ash-under-test executable:

   ```bash
   ${ASH_UNDER_TEST:?set Ash candidate binary} test fixtures/<phase-fixture> --format json
   ```

3. **Release/install parity** — ordinary `ash` on PATH after install/release catches up; closeout must either prove parity or record an explicit handoff.

`cargo run -p ash-cli -- test ...` is never final-surface evidence.

## Scope

### In Scope

- Generator schemas and generated binding injection for law/property parameters.
- Primitive and bounded ADT/container generation.
- Counterexample artifacts and deterministic replay.
- Primitive and container/ADT shrinking.
- No-Cargo final-surface fixtures for generated and shrunk cases.

### Non-Goals

- coverage reporting
- mutation testing
- flake quarantine
- distributed orchestration
- proof-producing synthesis
- unbounded arbitrary source-world generation

## Required Agent Skills

Implementation agents must load and follow:

- `rust-skills` for Rust code, public APIs, proptest coverage, error handling, and clippy-clean implementation.
- `ash-language-feature-spec-writing` for Ash surface/parser/typechecker/runner contracts and final-surface Ash examples.
- `test-driven-development` when implementing code slices: write failing Rust/Ash-facing tests before production changes.
- `verification-before-completion` before marking any task complete.
- `systematic-debugging` for any unexpected runner, parser, or property failure.

## Examples

```ash
law reverse_twice(xs: List<Int>): reverse(reverse(xs)) == xs
proof reverse_twice(xs: List<Int>) {
    by test property
}
```

Expected final-surface evidence example:

```bash
$ASH_UNDER_TEST test fixtures/phase146-property-shrinking --only-synthesized laws --format json --seed 42 --max-cases 50
```

A failing property row should include generated bindings and, after shrinking, a smaller counterexample.

## Result and Reporting Requirements

- JSON output must remain machine-readable and stable enough for later orchestration.
- Unsupported cases must be `deferred`, `untested`, or explicit errors; they must not be counted as passing evidence.
- Repro artifacts must include enough data for a direct `$ASH_UNDER_TEST test ...` replay when the phase owns execution behavior.
- Human output should summarize the new capability without hiding caveats.

## Implementation Tasks

- [TASK-1456](../plan/tasks/TASK-1456-property-generation-shrinking-audit.md): Audit current property generation and shrinking gaps
- [TASK-1457](../plan/tasks/TASK-1457-generator-schema-and-binding-model.md): Define generator schema and binding model
- [TASK-1458](../plan/tasks/TASK-1458-primitive-property-generators.md): Implement primitive property generators
- [TASK-1459](../plan/tasks/TASK-1459-adt-container-property-generators.md): Implement ADT/container property generators
- [TASK-1460](../plan/tasks/TASK-1460-authored-property-binding-injection.md): Inject generated bindings into Ash-authored property tests
- [TASK-1461](../plan/tasks/TASK-1461-counterexample-artifact-schema.md): Add counterexample artifact schema
- [TASK-1462](../plan/tasks/TASK-1462-primitive-shrinker-core.md): Implement primitive shrinking core
- [TASK-1463](../plan/tasks/TASK-1463-adt-container-shrinking.md): Implement ADT/container shrinking
- [TASK-1464](../plan/tasks/TASK-1464-property-shrinking-final-surface-fixtures.md): Add no-Cargo property/shrinking fixtures
- [TASK-1465](../plan/tasks/TASK-1465-property-generation-shrinking-closeout.md): Close out property generation/shrinking phase

## Changelog

### 2026-06-15

- Created this planning specification and registered PLAN-146 / TASK-1456 through TASK-1465.
