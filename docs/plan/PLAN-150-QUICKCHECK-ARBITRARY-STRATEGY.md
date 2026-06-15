# PLAN-150: QuickCheck Arbitrary and Strategy Property Testing

**Status:** ✅ Complete
**Spec:** [SPEC-086: QuickCheck Arbitrary and Strategy Property Testing](../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md)
**Design note:** [DESIGN-NOTE: QuickCheck-Style Property Testing and Future Evidence Families](../design/DESIGN-NOTE-QUICKCHECK-PROPERTY-TESTING.md)
**Builds on:** [PLAN-146](PLAN-146-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md)
**Task range:** TASK-1485 through TASK-1496

## Goal

Move Phase 146's runner-owned generated bindings toward a standard-library QuickCheck-like property-testing model. The phase should introduce `test::quickcheck`, `Strategy<T>`, `Arbitrary<T>`, strategy overrides, law/property outcome distinctions, and law-evidence cache schema groundwork.

## Non-Goals

- no automatic `Arbitrary<T>` derivation,
- no full SmallCheck implementation,
- no solver/proof-producing synthesis,
- no coverage/mutation/distributed orchestration,
- no effectful generators,
- no unrestricted source-world generation.

## Decision Gates

| Gate | Decision | Owner task |
|---|---|---|
| D1 | Validate exact live syntax and runner callability for stdlib `test::quickcheck` APIs. | TASK-1485 |
| D2 | Decide first-slice `Strategy<T>` representation: opaque runner carrier vs library data wrapper with runner hooks. | TASK-1487 |
| D3 | Decide first-slice override syntax: metadata bridge only vs parser-level `by test quickcheck with { ... }`. | TASK-1490 |
| D4 | Decide first-slice cache behavior: schema-only, write-only, or read/write local cache. | TASK-1492 |
| D5 | Verify no-Cargo final surface before status closeout. | TASK-1493, TASK-1496 |

## Task Table

| Task | Description | Status |
|---|---|---|
| [TASK-1485](tasks/TASK-1485-quickcheck-design-and-live-syntax-audit.md) | Audit live syntax, stdlib surfaces, interface evidence, runner seams, and cache seams. | ✅ Complete |
| [TASK-1486](tasks/TASK-1486-quickcheck-stdlib-namespace.md) | Add `test::quickcheck` namespace skeleton and docs. | ✅ Complete |
| [TASK-1487](tasks/TASK-1487-strategy-carrier-and-combinator-api.md) | Define `Strategy<T>` carrier and core combinator API. | ✅ Complete |
| [TASK-1488](tasks/TASK-1488-arbitrary-interface-and-laws.md) | Define `Arbitrary<T>` interface and library law docs/tests. | ✅ Complete |
| [TASK-1489](tasks/TASK-1489-primitive-container-arbitrary-impls.md) | Add primitive/container default strategies. | ✅ Complete |
| [TASK-1490](tasks/TASK-1490-runner-strategy-resolution.md) | Resolve explicit strategies and `Arbitrary<T>` evidence in the runner. | ✅ Complete |
| [TASK-1491](tasks/TASK-1491-quickcheck-generation-and-shrinking-execution.md) | Execute strategy generation/shrinking and record repro artifacts. | ✅ Complete |
| [TASK-1492](tasks/TASK-1492-law-property-enforcement-and-cache-schema.md) | Split law/property outcomes and add evidence cache schema. | ✅ Complete |
| [TASK-1493](tasks/TASK-1493-quickcheck-final-surface-fixtures.md) | Add no-Cargo fixtures for defaults, overrides, and failing shrink cases. | ✅ Complete |
| [TASK-1494](tasks/TASK-1494-quickcheck-documentation-cookbook.md) | Write documentation/cookbook examples. | ✅ Complete |
| [TASK-1495](tasks/TASK-1495-quickcheck-future-backends-design-note.md) | Validate and link the future-backend design note. | ✅ Complete |
| [TASK-1496](tasks/TASK-1496-quickcheck-closeout.md) | Close out the phase and run broad verification. | ✅ Complete |

## Implementation Order

1. TASK-1485 blocks all implementation. It must replace any placeholder commands in later tasks if the live audit discovers changed syntax/seams.
2. TASK-1486 and TASK-1487 establish stdlib API and carrier shape.
3. TASK-1488 and TASK-1489 add default evidence and laws.
4. TASK-1490 and TASK-1491 connect runner resolution/execution.
5. TASK-1492 adds enforcement/cache semantics.
6. TASK-1493 proves final-surface behavior.
7. TASK-1494 and TASK-1495 produce user and future-design documentation.
8. TASK-1496 reconciles statuses and verifies the phase.

## Verification Strategy

Every implementation task must include:

- focused Rust tests for changed runner/type/stdlib-loading behavior,
- no-Cargo `$ASH_UNDER_TEST test ...` fixtures for user-facing behavior when applicable,
- `cargo fmt --check`,
- scoped `cargo check` / `cargo test` / `cargo clippy` gates for affected crates,
- `git diff --check`,
- docs link/trailing-whitespace checks for spec/plan/reference changes.

## Documentation Deliverables

- `reference/tools/test.md` quickcheck section,
- at least one cookbook/reference page or section with examples for:
  - defining `Arbitrary<T>`,
  - defining and composing `Strategy<T>`,
  - sorted-list and safe-expression overrides,
  - shrink traces,
  - law vs property semantics,
  - evidence cache invalidation,
  - future SmallCheck/solver/proof relationship.

## Risks

| Risk | Mitigation |
|---|---|
| Live interface/default-method support is not enough for the ideal API. | TASK-1485 must audit and choose a staged bridge; first slice may use docs + runner-known shims. |
| Strategy carrier becomes too magical. | Keep public API in `test::quickcheck`; document runner hooks as opaque implementation detail. |
| Law cache overclaims proof certainty. | Cache stores empirical evidence only; stale/missing is distinct from refuted. |
| Override syntax requires parser churn. | Use metadata bridge first if parser-level `with` syntax is too large. |
| Missing generator evidence accidentally passes. | Fail closed: defer/error for missing evidence; never count as pass. |

## Closeout Criteria

- All TASK-1485 through TASK-1496 files are complete with checked verification evidence.
- `PLAN-INDEX.md` and this plan agree on 12/12 task status.
- `CHANGELOG.md` has a Phase 150 entry.
- `docs/spec/README.md` links SPEC-086.
- Final-surface examples run through `$ASH_UNDER_TEST test ...` without `cargo run` as evidence.
- The design note is linked from spec, plan, and documentation/cookbook surfaces.

## Completion Notes

Implemented Phase 150 as the first QuickCheck-like property-testing slice. The final implementation chose the metadata bridge for strategy overrides rather than parser-level `with` syntax, because TASK-1485 confirmed the existing authored property metadata seam could deliver no-Cargo behavior without widening parser scope. The stdlib now exposes `test::quickcheck` as the documented user namespace; the runner materializes default bounded `Arbitrary<T>` representatives and explicit `Strategy<T>` overrides for supported primitive/list domains.

Verification evidence is recorded in TASK-1496.
