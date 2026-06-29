# TASK-1741: Specify and implement the narrow source-origin sidecar boundary for expansion products

## Status: ✅ Complete

## Summary

Specify and, if safe, implement a narrow source-origin sidecar boundary for notation and operator-section expansion products so generated surface forms can retain their expansion origin without overclaiming full Core provenance.

## Specification Reference

- PLAN-170: origin sidecar track
- SPEC-095c: surface origin and macro/notation metadata
- SPEC-098c: surface-to-Core lowering and source mapping
- PLAN-169 TASK-1733: operator-section elaboration span preservation

## Dependencies

- ✅ TASK-1736: Phase 170 packet created
- ✅ TASK-1737: Boundary audit informs where origin metadata can be preserved

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Full Core origin threading | PLAN-169 non-goal | Needed separate carrier decision | Partial | Specify boundary; implement only narrow surface-side metadata unless Core API change is explicitly approved | Tests prove metadata exists or docs record deferral |

## Requirements

1. Define whether origin sidecars live in surface AST, expanded module metadata, Core sidecars, or a separate map keyed by spans/node IDs.
2. Preserve the distinction between `OperatorSection`, `NotationExpansion`, and parser-original nodes.
3. Avoid changing Core public APIs unless the task records a T2 decision and review.
4. Add tests for generated forms from built-in sections and local notation sections.
5. Ensure diagnostics can still point at the original section/operator spelling.
6. Document any remaining Core-origin threading deferral honestly.

## TDD Steps

1. Write tests describing the chosen metadata surface.
2. Implement the narrow carrier or side map.
3. Update expansion and lowering as needed without changing unrelated semantics.
4. Verify diagnostics/source spans still work for unresolved sections.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1733_operator_section_elaboration
  - cargo test -p ash-parser --test task_1734_expanded_surface_lowering_gate
  - cargo test -p ash-parser
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Origin metadata boundary is specified.
  - [x] Implemented metadata, if any, has positive tests.
  - [x] Any Core-origin deferral is explicit and indexed in docs.
```

## Closeout evidence

- Design note: `docs/design/phase-170-expansion-origin-sidecar-boundary.md`.
- Implemented `ExpandedSurfaceModule::origins` with `ExpandedSurfaceOrigin` entries for generated surface nodes.
- Built-in operator-section expansion records `SurfaceOrigin::OperatorSection { section_span, operator_span }`.
- Local notation-section expansion records `SurfaceOrigin::NotationExpansion { notation_span, target }`.
- Core-origin threading remains explicitly deferred; no Core public API changes were made.
- Fresh verification:
  - `cargo test -p ash-parser --test task_1733_operator_section_elaboration -- --nocapture`
  - `cargo test -p ash-parser --test task_1734_expanded_surface_lowering_gate -- --nocapture`
  - `cargo test -p ash-parser`
  - `cargo check --workspace`
  - `cargo clippy -p ash-parser --all-targets --all-features -- -D warnings`
  - `cargo fmt --check`
  - `git diff --check`
