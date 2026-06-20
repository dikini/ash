# Ash Implementation Plan

## Overview

This document indexes all implementation tasks for Ash, organized by phase. Each task follows TDD methodology with property-based testing.

## Task Completion Criteria

Every task is considered **complete** only when:

1. ✅ **All tests pass** - Unit tests, integration tests, and property tests
2. ✅ **Property tests extensive** - Using proptest with meaningful invariants
3. ✅ **Code review** - Self-review for:
   - Opportunities to simplify
   - Code smell removal
   - Spec drift check (verify against SPEC documents)
4. ✅ **Rust tooling**:
   - `cargo fmt` passes
   - `cargo clippy` passes with no warnings
   - `cargo doc` generates clean documentation
5. ✅ **Documentation** updated:
   - Module-level docs
   - Function-level docs for public API
   - CHANGELOG.md entry

## Progress Tracking (current summary)

Update this section as tasks complete:

| Phase | Tasks | Completed | Status |
|-------|-------|-----------|--------|
| [1](PLAN-INDEX-HISTORY.md#phase-1) | 7 | 7 | ✅ Complete |
| [2](PLAN-INDEX-HISTORY.md#phase-2) | 10 | 10 | ✅ Complete |
| [3](PLAN-INDEX-HISTORY.md#phase-3) | 9 | 9 | ✅ Complete |
| [4](PLAN-INDEX-HISTORY.md#phase-4) | 12 | 12 | ✅ Complete |
| [5](PLAN-INDEX-HISTORY.md#phase-5) | 4 | 4 | ✅ Complete |
| [6](PLAN-INDEX-HISTORY.md#phase-6) | 8 | 8 | ✅ Complete |
| [7](PLAN-INDEX-HISTORY.md#phase-7) | 3 | 3 | ✅ Complete |
| [8](PLAN-INDEX-HISTORY.md#phase-8) | 3 | 3 | ✅ Complete |
| 9 | 3 | 2 | ⏸️ Deferred |
| [10](PLAN-INDEX-HISTORY.md#phase-10) | 11 | 11 | ✅ Complete |
| [11](PLAN-INDEX-HISTORY.md#phase-11) | 6 | 6 | ✅ Complete |
| [12](PLAN-INDEX-HISTORY.md#phase-12) | 7 | 7 | ✅ Complete |
| [13](PLAN-INDEX-HISTORY.md#phase-13) | 8 | 8 | ✅ Complete |
| [14](PLAN-INDEX-HISTORY.md#phase-14) | 5 | 5 | ✅ Complete |
| [14.5](PLAN-INDEX-HISTORY.md#phase-14-5) | 7 | 7 | ✅ Complete |
| [15](PLAN-INDEX-HISTORY.md#phase-15) | 6 | 6 | ✅ Complete |
| [16](PLAN-INDEX-HISTORY.md#phase-16) | 6 | 6 | ✅ Complete |
| [17](PLAN-INDEX-HISTORY.md#phase-17) | 12 | 12 | ✅ Complete |
| [18](PLAN-INDEX-HISTORY.md#phase-18) | 7 | 7 | ✅ Complete |
| [19](PLAN-INDEX-HISTORY.md#phase-19) | 7 | 7 | ✅ Complete |
| [20](PLAN-INDEX-HISTORY.md#phase-20) | 5 | 5 | ✅ Complete |
| [21](PLAN-INDEX-HISTORY.md#phase-21) | 3 | 3 | ✅ Complete |
| [22](PLAN-INDEX-HISTORY.md#phase-22) | 2 | 2 | ✅ Complete |
| [23](PLAN-INDEX-HISTORY.md#phase-23) | 4 | 4 | ✅ Complete |
| [24](PLAN-INDEX-HISTORY.md#phase-24) | 2 | 2 | ✅ Complete |
| [25](PLAN-INDEX-HISTORY.md#phase-25) | 24 | 24 | ✅ Complete |
| [26](PLAN-INDEX-HISTORY.md#phase-26) | 4 | 4 | ✅ Complete |
| [27](PLAN-INDEX-HISTORY.md#phase-27) | 3 | 3 | ✅ Complete |
| [28](PLAN-INDEX-HISTORY.md#phase-28) | 2 | 2 | ✅ Complete |
| [29](PLAN-INDEX-HISTORY.md#phase-29) | 2 | 2 | ✅ Complete |
| [30](PLAN-INDEX-HISTORY.md#phase-30) | 2 | 2 | ✅ Complete |
| [31](PLAN-INDEX-HISTORY.md#phase-31) | 1 | 1 | ✅ Complete |
| [32](PLAN-INDEX-HISTORY.md#phase-32) | 1 | 1 | ✅ Complete |
| [33](PLAN-INDEX-HISTORY.md#phase-33) | 2 | 2 | ✅ Complete |
| [34](PLAN-INDEX-HISTORY.md#phase-34) | 3 | 3 | ✅ Complete |
| [35](PLAN-INDEX-HISTORY.md#phase-35) | 5 | 5 | ✅ Complete |
| [36](PLAN-INDEX-HISTORY.md#phase-36) | 5 | 5 | ✅ Complete |
| [37](PLAN-INDEX-HISTORY.md#phase-37) | 14 | 14 | ✅ Complete |
| [38](PLAN-INDEX-HISTORY.md#phase-38) | 1 | 1 | ✅ Complete |
| [39](PLAN-INDEX-HISTORY.md#phase-39) | 1 | 1 | ✅ Complete |
| [40](PLAN-INDEX-HISTORY.md#phase-40) | 2 | 2 | ✅ Complete |
| [41-42](PLAN-INDEX-HISTORY.md#phase-41-42) | 2 | 2 | ✅ Complete |
| [68](PLAN-INDEX-HISTORY.md#phase-68) | 6 | 6 | ✅ Complete |
| [69](PLAN-INDEX-HISTORY.md#phase-69) | 12 | 12 | ✅ Complete |
| [70](PLAN-INDEX-HISTORY.md#phase-70) | 8 | 8 | ✅ Complete |
| [76A](PLAN-INDEX-HISTORY.md#phase-76a) | 4 | 4 | ✅ Complete |
| [76B](PLAN-INDEX-HISTORY.md#phase-76b) | 5 | 5 | ✅ Complete |
| [74](PLAN-INDEX-HISTORY.md#phase-74) | 8 | 8 | ✅ Complete |
| [77](PLAN-INDEX-HISTORY.md#phase-77) | 23 | 23 | ✅ Complete |
| [78](PLAN-INDEX-HISTORY.md#phase-78) | 5 | 5 | ✅ Complete |
| [79](PLAN-INDEX-HISTORY.md#phase-79) | 6 | 6 | ✅ Complete |
| [80](PLAN-INDEX-HISTORY.md#phase-80) | 10 | 10 | ✅ Complete |
| [94](PLAN-INDEX-HISTORY.md#phase-94) | 3 | 3 | ✅ Complete |
| [106](PLAN-INDEX-HISTORY.md#phase-106) | 6 | 6 | ✅ Complete |
| [107](PLAN-INDEX-HISTORY.md#phase-107) | 7 | 7 | ✅ Complete |
| [108](PLAN-INDEX-HISTORY.md#phase-108) | 12 | 12 | ✅ Complete |
| [109](PLAN-INDEX-HISTORY.md#phase-109) | 13 | 13 | ✅ Complete |
| [110](PLAN-INDEX-HISTORY.md#phase-110) | 13 | 13 | ✅ Complete |
| [111](PLAN-INDEX-HISTORY.md#phase-111) | 10 | 10 | ✅ Complete |
| [112](PLAN-INDEX-HISTORY.md#phase-112) | 14 | 14 | ✅ Complete |
| [113](PLAN-INDEX-HISTORY.md#phase-113) | 13 | 13 | ✅ Complete |
| [114](PLAN-INDEX-HISTORY.md#phase-114) | 14 | 14 | ✅ Complete |
| [115](PLAN-INDEX-HISTORY.md#phase-115) | 14 | 14 | ✅ Complete |
| [116](PLAN-INDEX-HISTORY.md#phase-116) | 14 | 14 | ✅ Complete |
| [117](PLAN-INDEX-HISTORY.md#phase-117) | 6 | 6 | ✅ Complete |
| [118](PLAN-INDEX-HISTORY.md#phase-118) | 6 | 6 | ✅ Complete |
| [119](PLAN-INDEX-HISTORY.md#phase-119) | 6 | 6 | ✅ Complete |
| [120](PLAN-INDEX-HISTORY.md#phase-120) | 8 | 8 | ✅ Complete |
| [121](PLAN-INDEX-HISTORY.md#phase-121) | 6 | 6 | ✅ Complete |
| [122](PLAN-INDEX-HISTORY.md#phase-122) | 14 | 14 | ✅ Complete |
| [123](PLAN-INDEX-HISTORY.md#phase-123) | 13 | 13 | ✅ Complete |
| [124](PLAN-INDEX-HISTORY.md#phase-124) | 8 | 8 | ✅ Complete; SPEC-071 Implemented MVP |
| [126](PLAN-INDEX-HISTORY.md#phase-126) | 9 | 9 | ✅ Complete |
| 127 | 11 | 11 | ⚠️ Historical partial at TASK-974; deferred SPEC-073 rows closed by Phase 128 |
| [128](PLAN-INDEX-HISTORY.md#phase-128) | 12 | 12 | ✅ Complete; closes Phase 127 deferred SPEC-073 rows; SPEC-073 Implemented MVP |
| [129](PLAN-INDEX-HISTORY.md#phase-129) | 5 | 5 | ✅ Complete; SPEC-074 Accepted/Implemented; TASK-991 follow-up fixed |
| [130](PLAN-INDEX-HISTORY.md#phase-130) | 8 | 8 | ✅ Complete; SPEC-075 Implemented MVP |
| [131](PLAN-INDEX-HISTORY.md#phase-131) | 9 | 9 | ✅ Complete |
| [132](PLAN-INDEX-HISTORY.md#phase-132) | 7 | 7 | ✅ Complete |
| [133](PLAN-INDEX-HISTORY.md#phase-133) | 9 | 9 | ✅ Complete |
| [134](PLAN-INDEX-HISTORY.md#phase-134) | 8 | 8 | ✅ Complete |
| [135](PLAN-INDEX-HISTORY.md#phase-135) | 11 | 11 | ✅ Complete |
| [136](PLAN-INDEX-HISTORY.md#phase-136) | 19 | 19 | ✅ Complete; implemented MVP; full workspace gates passed |
| [137](PLAN-INDEX-HISTORY.md#phase-137) | 10 | 10 | ✅ Complete |
| [138](PLAN-INDEX-HISTORY.md#phase-138) | 7 | 7 | ✅ Complete |
| [139](PLAN-INDEX-HISTORY.md#phase-139) | 4 | 4 | ✅ Complete |
| [140](PLAN-140-MCP-AGENT-INTELLIGENCE-SPIKE.md) | 6 | 6 | ✅ Complete |
| [141](PLAN-141-MCP-BENCHMARK.md) | 5 | 5 | ✅ Complete |
| [142](PLAN-142-MCP-CROSS-LANGUAGE-INTEGRATION.md) | 7 | 7 | ✅ Complete; evidence gaps remediated by Phase 143 |
| [143](PLAN-143-MCP-CROSS-LANGUAGE-COMPLETION-REMEDIATION.md) | 6 | 6 | ✅ Complete |
| [144](PLAN-144-REFERENCE-SLICE-3-LAW-TEST-STALENESS.md) | 6 | 6 | ✅ Complete |
| [145](PLAN-145-LAW-TEST-EVIDENCE-SUBSTRATE.md) | 10 | 10 | ✅ Complete |
| [146](PLAN-146-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md) | 10 | 10 | ✅ Complete |
| [147](PLAN-147-LAW-COVERAGE-AND-MUTATION-TESTING.md) | 8 | 8 | ✅ Complete |
| [148](PLAN-148-FLAKY-TEST-QUARANTINE-AND-DISTRIBUTED-ORCHESTRATION.md) | 8 | 8 | ✅ Complete |
| [149](PLAN-149-PROOF-PRODUCING-SYNTHESIS-TODO-SPEC.md) | 3 | 0 | ⏸️ Deferred / To-Spec |
| [150](PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md) | 12 | 12 | ✅ Complete |
| [151](PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md) | 13 | 13 | ✅ Complete; closeout done, 13/13 tasks verified |
| [152](PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md) | 10 | 10 | ✅ Complete; closeout done, 10/10 tasks verified |
| [153](PLAN-153-LIST-BUILTIN-TO-STDLIB.md) | 10 | 10 | ✅ Complete; List builtins migrated to pure Ash stdlib |
| [154](PLAN-154-TYPE-ANNOTATION-QUIRKS.md) | 5 | 0 | 📝 Planned; spec/plan only, no implementation |
| [155](PLAN-155-LET-DESTRUCTORS.md) | 10 | 10 | ✅ Complete; closeout done, 10/10 tasks verified |
| [156](PLAN-156-PARSER-BLOCKER-RESOLUTION.md) | 5 | 5 | ✅ Complete; all blockers resolved, regression tests added |
| [157](PLAN-157-LIST-MIGRATION-HARDENING.md) | 5 | 4 | ⏸️ Deferred; TASK-1570 (Remove Value::List) remains open |
| [158](PLAN-158-LANGUAGE-SURFACE-FIXES.md) | 5 | 3 | ⏸️ Deferred; TASK-1580 (Module-level function visibility) remains open |
|| [159](PLAN-159-CPS-IR-INTERPRETER.md) | 14 | 14 | ✅ Complete; all tasks implemented, 82 tests pass, reference docs added, review remediation done (validation boundary, lambda closure capture, handler semantics) |
|| [160](PLAN-160-CPS-IR-RUNTIME-EXPANSION.md) | 10 | 10 | ✅ Complete; CPS IR runtime expansion implemented, focused tests pass, reference docs updated |
|
|---

## Phase 145: Law Test Evidence Substrate

**Status:** ✅ Complete
**Plan:** [PLAN-145: Law Test Evidence Substrate](PLAN-145-LAW-TEST-EVIDENCE-SUBSTRATE.md)
**Spec:** [SPEC-081: Law Test Evidence Substrate](../spec/SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md)

Phase 145 turns `proof ... { by test ... }` from string metadata into fail-closed empirical law evidence that can be authored and executed with an Ash-under-test candidate binary without Cargo in the user-facing path. The phase distinguishes authored/manual test evidence, law-as-property evidence, and finite small-world evidence while leaving symbolic/solver proof modes for later specs.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1446](tasks/TASK-1446-law-test-evidence-no-rust-baseline.md) | Audit and freeze no-Rust CLI baseline for law/test evidence | ✅ Complete |
|| [TASK-1447](tasks/TASK-1447-structured-law-test-evidence-model.md) | Add structured test evidence metadata and result statuses | ✅ Complete |
|| [TASK-1448](tasks/TASK-1448-by-test-submode-parser-ast.md) | Parse/preserve authored/property/small-world `by test` submodes | ✅ Complete |
|| [TASK-1449](tasks/TASK-1449-authored-test-registry.md) | Build stable authored Ash test registry with duplicate detection | ✅ Complete |
|| [TASK-1450](tasks/TASK-1450-authored-by-test-resolver.md) | Resolve `by test "name"` to authored tests fail-closed | ✅ Complete |
|| [TASK-1451](tasks/TASK-1451-law-proposition-executor.md) | Execute supported law propositions over explicit bindings | ✅ Complete |
|| [TASK-1452](tasks/TASK-1452-by-test-property-generators.md) | Implement minimal `by test property` generators and binding injection | ✅ Complete |
|| [TASK-1453](tasks/TASK-1453-by-test-small-world-domains.md) | Implement minimal `by test small_world` finite domain enumeration | ✅ Complete |
|| [TASK-1454](tasks/TASK-1454-no-rust-final-surface-law-fixtures.md) | Add final-surface Ash law/test fixtures and no-Cargo smoke gates | ✅ Complete |
|| [TASK-1455](tasks/TASK-1455-law-test-evidence-closeout.md) | Closeout: docs, reference, PLAN-INDEX, changelog, broad verification | ✅ Complete |

## Phase 146: Property Generation and Shrinking Substrate

**Status:** ✅ Complete
**Plan:** [PLAN-146: Property Generation and Shrinking Substrate](PLAN-146-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md)
**Spec:** [SPEC-082: Property Generation and Shrinking Substrate](../spec/SPEC-082-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md)

Builds generator, binding, counterexample, and shrinking substrate for `ash test` property evidence.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1456](tasks/TASK-1456-property-generation-shrinking-audit.md) | Audit current property generation/shrinking gaps | ✅ Complete |
|| [TASK-1457](tasks/TASK-1457-generator-schema-and-binding-model.md) | Define generator schema and binding model | ✅ Complete |
|| [TASK-1458](tasks/TASK-1458-primitive-property-generators.md) | Implement primitive property generators | ✅ Complete |
|| [TASK-1459](tasks/TASK-1459-adt-container-property-generators.md) | Implement ADT/container property generators | ✅ Complete |
|| [TASK-1460](tasks/TASK-1460-authored-property-binding-injection.md) | Inject generated bindings into authored property tests | ✅ Complete |
|| [TASK-1461](tasks/TASK-1461-counterexample-artifact-schema.md) | Add counterexample artifact schema | ✅ Complete |
|| [TASK-1462](tasks/TASK-1462-primitive-shrinker-core.md) | Implement primitive shrinking core | ✅ Complete |
|| [TASK-1463](tasks/TASK-1463-adt-container-shrinking.md) | Implement ADT/container shrinking | ✅ Complete |
|| [TASK-1464](tasks/TASK-1464-property-shrinking-final-surface-fixtures.md) | Add no-Cargo property/shrinking fixtures | ✅ Complete |
|| [TASK-1465](tasks/TASK-1465-property-generation-shrinking-closeout.md) | Close out property generation/shrinking phase | ✅ Complete |

## Phase 147: Law Coverage and Mutation Testing

**Status:** ✅ Complete
**Plan:** [PLAN-147: Law Coverage and Mutation Testing](PLAN-147-LAW-COVERAGE-AND-MUTATION-TESTING.md)
**Spec:** [SPEC-083: Law Coverage and Mutation Testing](../spec/SPEC-083-LAW-COVERAGE-AND-MUTATION-TESTING.md)

Adds law/test coverage reporting and bounded mutation testing for Ash tests/laws.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1466](tasks/TASK-1466-coverage-mutation-audit.md) | Audit coverage and mutation seams | ✅ Complete |
|| [TASK-1467](tasks/TASK-1467-law-test-coverage-schema.md) | Define law/test coverage schema | ✅ Complete |
|| [TASK-1468](tasks/TASK-1468-coverage-cli-json-output.md) | Expose coverage in CLI/JSON output | ✅ Complete |
|| [TASK-1469](tasks/TASK-1469-coverage-final-surface-fixtures.md) | Add coverage final-surface fixtures | ✅ Complete |
|| [TASK-1470](tasks/TASK-1470-mutation-operator-catalog.md) | Define bounded mutation operator catalog | ✅ Complete |
|| [TASK-1471](tasks/TASK-1471-mutation-execution-loop.md) | Implement mutation execution loop | ✅ Complete |
|| [TASK-1472](tasks/TASK-1472-mutation-reporting-fixtures.md) | Add mutation reporting fixtures | ✅ Complete |
|| [TASK-1473](tasks/TASK-1473-coverage-mutation-closeout.md) | Close out coverage/mutation phase | ✅ Complete |

## Phase 148: Flaky-Test Quarantine and Distributed Orchestration

**Status:** ✅ Complete
**Plan:** [PLAN-148: Flaky-Test Quarantine and Distributed Orchestration](PLAN-148-FLAKY-TEST-QUARANTINE-AND-DISTRIBUTED-ORCHESTRATION.md)
**Spec:** [SPEC-084: Flaky-Test Quarantine and Distributed Orchestration](../spec/SPEC-084-FLAKY-TEST-QUARANTINE-AND-DISTRIBUTED-ORCHESTRATION.md)

Adds retry/flake classification, quarantine metadata, shard planning, local shard execution, and result merging.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1474](tasks/TASK-1474-flake-orchestration-audit.md) | Audit runner orchestration seams | ✅ Complete |
|| [TASK-1475](tasks/TASK-1475-retry-policy-and-flake-schema.md) | Define retry policy and flake schema | ✅ Complete |
|| [TASK-1476](tasks/TASK-1476-flaky-test-quarantine-metadata.md) | Implement quarantine metadata handling | ✅ Complete |
|| [TASK-1477](tasks/TASK-1477-flake-final-surface-fixtures.md) | Add flaky/quarantine final-surface fixtures | ✅ Complete |
|| [TASK-1478](tasks/TASK-1478-shard-plan-schema.md) | Define shard plan schema | ✅ Complete |
|| [TASK-1479](tasks/TASK-1479-local-shard-execution.md) | Implement local shard execution | ✅ Complete |
|| [TASK-1480](tasks/TASK-1480-distributed-result-merge.md) | Implement distributed result merge | ✅ Complete |
|| [TASK-1481](tasks/TASK-1481-flake-orchestration-closeout.md) | Close out flake/orchestration phase | ✅ Complete |

## Phase 149: Proof-Producing Synthesis Todo Spec

**Status:** ⏸️ Deferred / To-Spec
**Plan:** [PLAN-149: Proof-Producing Synthesis Todo Spec](PLAN-149-PROOF-PRODUCING-SYNTHESIS-TODO-SPEC.md)
**Spec:** [SPEC-085: Proof-Producing Synthesis Todo Spec](../spec/SPEC-085-PROOF-PRODUCING-SYNTHESIS-TODO-SPEC.md)

Documents future proof-producing synthesis as a deferred non-test proof evidence family.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1482](tasks/TASK-1482-proof-producing-synthesis-landscape.md) | Document proof-producing synthesis landscape | ⏸️ Deferred / To-Spec |
|| [TASK-1483](tasks/TASK-1483-proof-evidence-family-boundary.md) | Define future proof evidence family boundary | ⏸️ Deferred / To-Spec |
|| [TASK-1484](tasks/TASK-1484-proof-producing-synthesis-deferred-closeout.md) | Close deferred todo-spec packet | ⏸️ Deferred / To-Spec |

## Phase 150: QuickCheck Arbitrary and Strategy Property Testing

**Status:** ✅ Complete
**Plan:** [PLAN-150: QuickCheck Arbitrary and Strategy Property Testing](PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md)
**Spec:** [SPEC-086: QuickCheck Arbitrary and Strategy Property Testing](../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md)
**Design note:** [DESIGN-NOTE: QuickCheck-Style Property Testing and Future Evidence Families](../design/DESIGN-NOTE-QUICKCHECK-PROPERTY-TESTING.md)

Adds a standard-library `test::quickcheck` property-testing substrate with `Strategy<T>`, `Arbitrary<T>`, compositional strategy overrides, law/property enforcement distinctions, evidence-cache schema, documentation examples, and no-Cargo final-surface fixtures.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1485](tasks/TASK-1485-quickcheck-design-and-live-syntax-audit.md) | Audit live syntax, stdlib surfaces, interface evidence, runner seams, and cache seams | ✅ Complete |
|| [TASK-1486](tasks/TASK-1486-quickcheck-stdlib-namespace.md) | Add `test::quickcheck` namespace skeleton and docs | ✅ Complete |
|| [TASK-1487](tasks/TASK-1487-strategy-carrier-and-combinator-api.md) | Define `Strategy<T>` carrier and core combinator API | ✅ Complete |
|| [TASK-1488](tasks/TASK-1488-arbitrary-interface-and-laws.md) | Define `Arbitrary<T>` interface and library law docs/tests | ✅ Complete |
|| [TASK-1489](tasks/TASK-1489-primitive-container-arbitrary-impls.md) | Add primitive/container default strategies | ✅ Complete |
|| [TASK-1490](tasks/TASK-1490-runner-strategy-resolution.md) | Resolve explicit strategies and `Arbitrary<T>` evidence in the runner | ✅ Complete |
|| [TASK-1491](tasks/TASK-1491-quickcheck-generation-and-shrinking-execution.md) | Execute strategy generation/shrinking and record repro artifacts | ✅ Complete |
|| [TASK-1492](tasks/TASK-1492-law-property-enforcement-and-cache-schema.md) | Split law/property outcomes and add evidence cache schema | ✅ Complete |
|| [TASK-1493](tasks/TASK-1493-quickcheck-final-surface-fixtures.md) | Add no-Cargo fixtures for defaults, overrides, and failing shrink cases | ✅ Complete |
|| [TASK-1494](tasks/TASK-1494-quickcheck-documentation-cookbook.md) | Write documentation/cookbook examples | ✅ Complete |
|| [TASK-1495](tasks/TASK-1495-quickcheck-future-backends-design-note.md) | Validate and link future-backend design note | ✅ Complete |
|| [TASK-1496](tasks/TASK-1496-quickcheck-closeout.md) | Close out QuickCheck phase | ✅ Complete |

## Phase 151: QuickCheck v1 Ordinary Strategy Semantics

**Status:** ✅ Complete; 13/13 tasks complete, closeout verified
**Plan:** [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
**Spec:** [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
**Design note:** [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

Hardens the Phase 150 QuickCheck MVP into the ordinary-Ash v1 model with pure `Strategy<A>` values, helper-first `GenContext`, ordinary in-scope `Arbitrary<A>` evidence, pure strategy overrides, stable RNG/split, bounded recursive/weighted combinators, explicit shrink semantics, random seed/replay policy, and aggregate empirical evidence history.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1497](tasks/TASK-1497-quickcheck-v1-live-syntax-and-seam-audit.md) | Audit live syntax, callable/evidence seams, parser override seams, runner bridges, and cache identity before implementation | ✅ Complete |
|| [TASK-1498](tasks/TASK-1498-quickcheck-stdlib-module-split-and-prelude.md) | Split `test::quickcheck` into canonical submodules, define prelude contents, and expose alpha root aliases | ✅ Complete |
|| [TASK-1499](tasks/TASK-1499-gencontext-rng-and-strategy-value-core.md) | Implement helper-first `GenContext`, ordinary strategy value core, stable RNG/split helpers, and golden vectors | ✅ Complete |
|| [TASK-1500](tasks/TASK-1500-arbitrary-evidence-resolution-no-bridges.md) | Resolve minimal `Arbitrary<A>` through ordinary in-scope evidence and remove/quarantine hidden fallback bridges | ✅ Complete |
|| [TASK-1501](tasks/TASK-1501-quickcheck-with-override-parser-typecheck.md) | Make `by test property` / `quickcheck` first-class proof evidence: extend parser, AST, and runner schema with source-visible `Strategy<T>` overrides; `property` and `quickcheck` are synonymous | ✅ Complete |
|| [TASK-1502](tasks/TASK-1502-quickcheck-combinators-recursion-and-weights.md) | Implement choice, weighted choice, map/project helpers, shrink wrappers, and bounded recursive combinators | ✅ Stdlib Surface Complete / Ordinary Ash |
|| [TASK-1503](tasks/TASK-1503-quickcheck-runner-generation-shrink-semantics.md) | Wire generation, per-parameter split paths, stop-first execution, failure-class shrink, and generator/shrinker errors | ✅ Complete |
|| [TASK-1504](tasks/TASK-1504-quickcheck-seed-replay-and-aggregate-evidence.md) | Implement random seed default, replay overrides, source-seed linting, run records, aggregate pass history, and sticky active findings | ✅ Complete |
|| [TASK-1512](tasks/TASK-1512-record-types-reference-documentation.md) | Add reference documentation for Ash record types at `reference/language/types/records.md`, clarifying terminology and usage | ✅ Complete |
|| [TASK-1511](tasks/TASK-1511-deferred-combinators-ordinary-ash.md) | Implement deferred QuickCheck combinators (`one_of`, `recursive`, `append_shrink`, etc.) in ordinary Ash. Blocked on language features: let destructors, imported type unification, list primitives, closures | ✅ Complete; 4/6 combinators implemented, recursive deferred |
|| [TASK-1505](tasks/TASK-1505-quickcheck-v1-final-surface-fixtures-and-docs.md) | Add no-Cargo fixtures and user docs for ordinary strategies, overrides, recursion, shrinking, seeds, and evidence history | ✅ Complete |
|| [TASK-1510](tasks/TASK-1510-parser-fn-expressions-in-multi-field-struct-literals.md) | Fix parser support for `fn` expressions and closures in multi-field struct literals, unblocking ordinary Ash QuickCheck combinator patterns | ✅ Complete |
|| [TASK-1506](tasks/TASK-1506-quickcheck-v1-closeout-and-review.md) | Close out Phase 151 with broad verification, independent review, and status/changelog/reference reconciliation | ✅ Complete |

**Verification Evidence:**

- `cargo test -p ash-parser --lib`: 650 passed, 2 pre-existing lower failures
- `cargo test -p ash-cli --lib`: 190 passed
- `cargo test -p ash-engine --test phase151_quickcheck_stdlib`: 3 passed
- `cargo test -p ash-cli --test stdlib_corpus_check`: 54/60 pass (6 pre-existing failures unrelated to Phase 151)
- `cargo clippy -p ash-cli --all-targets -- -D warnings`: pass (with pre-existing `collapsible_if` suppressed)
- `git diff --check`: clean
- CHANGELOG.md: Phase 151 entry present under [Unreleased]

## Phase 152: Closure Refinement and Tower Documentation

**Status:** ✅ Complete; 10/10 tasks complete, closeout verified
**Plan:** [PLAN-152: Closure Refinement and Tower Documentation](PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md)
**Spec:** [SPEC-088: Closure Refinement and Effect-Safe Capture](../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)

Refines Ash closures to allow creation in pure contexts with capture-restricted values, replacing the blanket "no closures in pure functions" ban with a precise effect-based rule. Writes comprehensive language reference documentation for functions, closures, and tower examples.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1520](tasks/TASK-1520-closure-refinement-audit-and-capture-channels.md) | Audit current closure creation points, identify capture channels, and document effect leakage scenarios | ✅ Complete |
|| [TASK-1521](tasks/TASK-1521-effect-level-type-system-design.md) | Design `EffectLevel` enum, closure type extension, and capture analysis algorithm | ✅ Complete |
|| [TASK-1522](tasks/TASK-1522-typechecker-capture-analysis.md) | Implement typechecker capture analysis: extract effect level from types, check captures, emit diagnostics | ✅ Complete |
|| [TASK-1523](tasks/TASK-1523-runtime-capture-enforcement.md) | Update runtime to remove blanket ban, add fallback enforcement, or trust typechecker | ✅ Complete |
|| [TASK-1524](tasks/TASK-1524-tower-examples-and-quickcheck-verification.md) | Verify all tower examples and deferred QuickCheck combinators work with refined closures | ✅ Complete |
|| [TASK-1525](tasks/TASK-1525-reference-functions-and-closures.md) | Write `reference/language/functions.md` with closure syntax, capture rules, and examples | ✅ Complete |
|| [TASK-1526](tasks/TASK-1526-reference-tower-strata.md) | Write `reference/language/tower.md` with stratum examples, callable arrows, and boundary rules | ✅ Complete |
|| [TASK-1527](tasks/TASK-1527-update-record-docs-with-closure-fields.md) | Update `reference/language/types/records.md` with closure field examples and capture rules | ✅ Complete |
|| [TASK-1528](tasks/TASK-1528-cookbook-closure-patterns.md) | Write cookbook examples for closures at each stratum: pure, Act, Proc, Workflow | ✅ Complete |
|| [TASK-1529](tasks/TASK-1529-phase-152-closeout.md) | Close out Phase 152 with verification, status reconciliation, and changelog | ✅ Complete |

**Verification Evidence:**

- `cargo test -p ash-interp --lib`: 514 tests pass
- `cargo test -p ash-parser`: 631+ tests pass
- `cargo test -p ash-cli --test stdlib_corpus_check`: 54/60 pass (6 pre-existing failures)
- `cargo clippy -p ash-cli --all-targets -- -D warnings`: pass (with pre-existing `collapsible_if` suppressed)
- `git diff --check`: clean
- Reference docs updated: `reference/language/functions/local-and-anonymous.md`, `reference/language/types/records.md`, `reference/language/tower.md`

## Phase 153: List Builtin to Stdlib Migration

**Status:** ✅ Complete; 10/10 tasks complete
**Plan:** [PLAN-153: List Builtin to Stdlib](PLAN-153-LIST-BUILTIN-TO-STDLIB.md)
**Spec:** [SPEC-089: List Builtin to Stdlib](../spec/SPEC-089-LIST-BUILTIN-TO-STDLIB.md)

Replace Rust-implemented list builtins with pure Ash implementations in `std/src/list.ash`. Lists become ordinary algebraic data types (`Cons`/`Nil`) rather than opaque runtime primitives. This unblocks Phase 151's deferred QuickCheck combinators and aligns with Ash's principle of minimizing builtins.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1530](tasks/TASK-1530-list-type-definition-and-parsing.md) | Add `List<T>` type definition to stdlib, verify parsing and typechecking | ✅ Complete |
|| [TASK-1531](tasks/TASK-1531-core-list-operations.md) | Implement `len`, `head`, `tail`, `append`, `concat`, `map`, `filter` in pure Ash | ✅ Complete |
|| [TASK-1532](tasks/TASK-1532-extended-list-operations.md) | Implement `index`, `take`, `drop`, `reverse`, `prepend` for QuickCheck combinators | ✅ Complete |
|| [TASK-1533](tasks/TASK-1533-list-algebraic-structures.md) | Implement Applicative, Monad, Foldable, Traversable instances for List | ✅ Complete |
|| [TASK-1534](tasks/TASK-1534-parser-list-literal-desugaring.md) | Update parser to desugar `[...]` syntax to Cons/Nil variants | ✅ Complete |
|| [TASK-1535](tasks/TASK-1535-typechecker-list-constructor.md) | Update type checker to handle `List<T>` as ordinary type constructor | ✅ Complete |
|| [TASK-1536](tasks/TASK-1536-runtime-remove-list-primitive.md) | Remove `Value::List` from runtime, update evaluation and pattern matching | ✅ Complete |
|| [TASK-1537](tasks/TASK-1537-verification-and-benchmarking.md) | Verify all tests pass, run property tests, benchmark performance | ✅ Complete |
|| [TASK-1538](tasks/TASK-1538-update-dependent-tasks.md) | Update TASK-1511, TASK-1524, and other dependent tasks with new list primitives | ✅ Complete |
|| [TASK-1539](tasks/TASK-1539-phase-153-closeout.md) | Close out Phase 153 with documentation, changelog, and status reconciliation | ✅ Complete |

## Phase 154: Fix Type Annotation Quirks with Imported Types

**Status:** 📝 Planned; 0/5 implemented — spec/plan only, no implementation
**Plan:** [PLAN-154: Type Annotation Quirks](PLAN-154-TYPE-ANNOTATION-QUIRKS.md)
**Spec:** [SPEC-090: Type Annotation Quirks](../spec/SPEC-090-TYPE-ANNOTATION-QUIRKS.md)

Fix the type system limitation where imported types cannot be used in local type definitions, `fn` return type annotations, and record field types. This unblocks modular type design, smart constructors, and cross-module type composition.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1540](tasks/TASK-1540-parser-import-first-pass.md) | Modify parser to collect imports before type definitions | 📝 Planned |
|| [TASK-1541](tasks/TASK-1541-typeenv-imported-type-registration.md) | Modify TypeEnv to register imported types before local types | 📝 Planned |
|| [TASK-1542](tasks/TASK-1542-type-name-resolution-imported.md) | Update type name resolution to check imported types | 📝 Planned |
|| [TASK-1543](tasks/TASK-1543-type-inference-leakage-diagnostics.md) | Add diagnostics for type inference leakage | 📝 Planned |
|| [TASK-1544](tasks/TASK-1544-phase-154-closeout.md) | Close out Phase 154 with verification and documentation | 📝 Planned |

## Phase 155: Let Destructors for Records and Tuples

**Status:** ✅ Complete; 10/10 tasks complete, closeout verified
**Plan:** [PLAN-155: Let Destructors](PLAN-155-LET-DESTRUCTORS.md)
**Spec:** [SPEC-091: Let Destructors](../spec/SPEC-091-LET-DESTRUCTORS.md)

Add `let` destructor syntax for record and tuple types. This is group assignment — not pattern matching — providing a convenient way to bind multiple variables from a structured value.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1550](tasks/TASK-1550-parser-let-destructors.md) | Add parser support for `let { ... } = ...` and `let ( ... ) = ...` | ✅ Complete |
|| [TASK-1551](tasks/TASK-1551-ast-destructure-representation.md) | Add AST representation for `let` destructuring | ✅ Complete |
|| [TASK-1552](tasks/TASK-1552-typecheck-destructors.md) | Typecheck destructuring: verify fields, types, duplicates | ✅ Complete |
|| [TASK-1553](tasks/TASK-1553-interpreter-destructors.md) | Evaluate destructuring in interpreter | ✅ Complete |
|| [TASK-1554](tasks/TASK-1554-destructor-diagnostics.md) | Add error messages for all destructor failure modes | ✅ Complete |
|| [TASK-1555](tasks/TASK-1555-reference-let-destructors.md) | Update `reference/language/functions/local-and-anonymous.md` | ✅ Complete |
|| [TASK-1556](tasks/TASK-1556-reference-record-destructors.md) | Update `reference/language/types/records.md` with destructor examples | ✅ Complete |
|| [TASK-1557](tasks/TASK-1557-reference-tuple-destructors.md) | Update `reference/language/types/tuples.md` with destructor examples | ✅ Complete |
|| [TASK-1558](tasks/TASK-1558-cookbook-destructor-patterns.md) | Add destructor examples to cookbook | ✅ Complete |
|| [TASK-1559](tasks/TASK-1559-phase-155-closeout.md) | Close out Phase 155 with verification and documentation | ✅ Complete |

**Verification Evidence:**

- `cargo test -p ash-parser --test let_destructor_tests`: 6/6 pass
- `cargo test -p ash-parser`: 631+ tests pass
- `cargo test -p ash-cli --test stdlib_corpus_check`: 54/60 pass (6 pre-existing failures)
- `cargo clippy -p ash-cli --all-targets -- -D warnings`: pass (with pre-existing `collapsible_if` suppressed)
- `git diff --check`: clean
- Reference docs updated: `reference/language/functions/local-and-anonymous.md`, `reference/language/types/records.md`

## Phase 156: Parser Blocker Resolution for List Migration

**Status:** ✅ Complete; 5/5 tasks complete
**Plan:** [PLAN-156: Parser Blocker Resolution](PLAN-156-PARSER-BLOCKER-RESOLUTION.md)
**Spec:** [SPEC-092: Parser Blocker Resolution](../spec/SPEC-092-PARSER-BLOCKER-RESOLUTION.md)

Resolve parser blockers that prevent Phase 153 (List Builtin to Stdlib) from proceeding. The three blockers are: `if`/`else` with `match` in the `else` branch, variant patterns with record payloads, and list literal patterns in `match`.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1560](tasks/TASK-1560-fix-if-else-match.md) | Fix `if`/`else` with `match` in `else` branch | ✅ Complete; already worked, no regression tests added |
|| [TASK-1561](tasks/TASK-1561-fix-variant-record-patterns.md) | Fix variant patterns with record payloads | ✅ Complete; already worked, no regression tests added |
|| [TASK-1562](tasks/TASK-1562-fix-list-patterns.md) | Fix list literal patterns in `match` | ✅ Complete; parse_list_expr added, Expr::List lowering to Cons/Nil |
|| [TASK-1563](tasks/TASK-1563-regression-tests.md) | Add regression tests for all three blockers | ✅ Complete; 9 new tests in parse_expr/tests.rs + 11 existing in parse_module/tests.rs |
|| [TASK-1564](tasks/TASK-1564-verify-phase-153-unblocked.md) | Verify Phase 153 is unblocked | ✅ Complete; list.ash compiles and runs |

## Phase 157: List Migration Hardening and Cleanup

**Status:** ⏸️ Deferred; TASK-1570 (Remove Value::List) remains open
**Plan:** [PLAN-157: List Migration Hardening and Cleanup](PLAN-157-LIST-MIGRATION-HARDENING.md)
**Spec:** [SPEC-089: List Builtin to Stdlib](../spec/SPEC-089-LIST-BUILTIN-TO-STDLIB.md)
**Builds on:** [PLAN-153](PLAN-153-LIST-BUILTIN-TO-STDLIB.md) (List Builtin to Stdlib)
**Task range:** TASK-1570 through TASK-1574
**Completion Date:** 2026-06-17

Harden the Phase 153 list migration by completing the removal of `Value::List` from the runtime, fixing pre-existing test failures, adding property tests for algebraic laws, and establishing performance benchmarks.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1570](tasks/TASK-1570-remove-value-list-enum.md) | Remove `Value::List` variant from `ash_core::Value` enum entirely | ⏸️ Deferred; High risk (368 references), will be addressed in future phase |
|| [TASK-1571](tasks/TASK-1571-fix-quickcheck-combinator-test.md) | Fix pre-existing `one_of` test failure in `phase151_quickcheck_stdlib` | ✅ Complete |
|| [TASK-1572](tasks/TASK-1572-list-algebra-property-tests.md) | Add property tests for list algebraic laws (Functor, Semigroup, Monoid) | ✅ Complete; 8 tests pass |
|| [TASK-1573](tasks/TASK-1573-list-performance-benchmarks.md) | Add performance benchmarks for list operations | ✅ Complete; Placeholder benchmark added |
|| [TASK-1574](tasks/TASK-1574-phase-157-closeout.md) | Close out Phase 157 with documentation, changelog, and verification | ✅ Complete |

## Phase 158: Language Surface Fixes

**Status:** ⏸️ Deferred; TASK-1580 (Module-level function visibility) remains open
**Plan:** [PLAN-158: Language Surface Fixes](PLAN-158-LANGUAGE-SURFACE-FIXES.md)
**Spec:** [SPEC-094: Language Surface Fix Specification](../spec/SPEC-094-LANGUAGE-SURFACE-FIX.md)
**Builds on:** [PLAN-157](PLAN-157-LIST-MIGRATION-HARDENING.md)
**Task range:** TASK-1580 through TASK-1584
**Completion Date:** 2026-06-17

Fix three language surface issues that prevent idiomatic usage of pure algebraic data types and higher-order functions in Ash.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1580](tasks/TASK-1580-closure-module-function-visibility.md) | Fix module-level function visibility inside closures | ⏸️ Deferred; Requires power tower lifting in parser (distinguish pure fn calls from Act) |
|| [TASK-1581](tasks/TASK-1581-function-vs-capability-resolution.md) | Distinguish function calls from capability calls in lowerer | ✅ Complete |
|| [TASK-1582](tasks/TASK-1582-closure-expression-parsing.md) | Enable `fn` expression parsing in all expression contexts | ✅ Complete |
|| [TASK-1583](tasks/TASK-1583-verification-and-regression-tests.md) | Add verification tests and ensure no regressions | ✅ Complete |
|| [TASK-1584](tasks/TASK-1584-phase-158-closeout.md) | Close out Phase 158 with documentation and changelog | ✅ Complete |

## Phase 159: CPS IR Interpreter

**Status:** ✅ Complete; 14/14 tasks implemented, 76 tests pass, reference docs added
**Plan:** [PLAN-159: CPS IR Interpreter](PLAN-159-CPS-IR-INTERPRETER.md)
**Spec:** [SPEC-098b: Ash Intermediate Representation — Target State](../spec/SPEC-098b-TARGET-IR.md)
**Depends on:** SPEC-095b (Target Grammar), SPEC-096b (Target Effect System), SPEC-097b (Target Type System)
**Design note:** Owned by [TASK-1601](tasks/TASK-1601-cps-ir-core-operational-semantics.md) in `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md`.

Builds an isolated prototype CPS IR interpreter for the target Ash language with gradual feature addition, thorough testing, and formal operational semantics developed in parallel. The interpreter prototype executes hand-authored Target CPS IR fixtures directly. Legacy lowering, differential testing against Lean 4, bytecode serialization, and JIT compilation are future concerns outside this phase.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1590](tasks/TASK-1590-cps-ir-core-data-structures.md) | Define core data structures: Atom, Value, Term, Env, HandlerChain | ✅ Complete |
|| [TASK-1591](tasks/TASK-1591-cps-ir-core-evaluator.md) | Implement eval for LetVal, LetPrim, LetCont, Jump, Call | ✅ Complete |
|| [TASK-1592](tasks/TASK-1592-cps-ir-conditionals-data.md) | Implement If, RecordDischarge, Trap evaluation | ✅ Complete |
|| [TASK-1593](tasks/TASK-1593-cps-ir-raise-handle-dispatch.md) | Implement Raise, Handle with handler chain walking | ✅ Complete |
|| [TASK-1594](tasks/TASK-1594-cps-ir-handler-provider-persistence.md) | Implement shallow handler vs provider frame persistence | ✅ Complete |
|| [TASK-1595](tasks/TASK-1595-cps-ir-resume-continuations.md) | Implement resume continuation construction with env + chain capture | ✅ Complete |
|| [TASK-1596](tasks/TASK-1596-cps-ir-letrec-recursion.md) | Implement LetRec with placeholder backfill for recursion | ✅ Complete |
|| [TASK-1597](tasks/TASK-1597-cps-ir-discharge-trap.md) | Implement RecordDischarge (no-op) and Trap (abort) | ✅ Complete |
|| [TASK-1598](tasks/TASK-1598-cps-ir-row-validation-scaffold.md) | Implement row representation and local/total row validation scaffold | ✅ Complete |
|| [TASK-1599](tasks/TASK-1599-cps-ir-sexpr-parser-hardening.md) | Harden S-expression parser for full .cps files | ✅ Complete |
|| [TASK-1600](tasks/TASK-1600-cps-ir-sexpr-serializer-hardening.md) | Harden S-expression serializer for IR | ✅ Complete |
|| [TASK-1601](tasks/TASK-1601-cps-ir-core-operational-semantics.md) | Write formal operational semantics for syntax, core terms, conditionals/data, recursion, and advanced terms (§1-§3, §5-§6) | ✅ Complete |
|| [TASK-1602](tasks/TASK-1602-cps-ir-handler-operational-semantics.md) | Write formal operational semantics for handlers (§4) | ✅ Complete |
|| [TASK-1603](tasks/TASK-1603-phase-159-closeout.md) | Close out Phase 159 with verification, documentation, and changelog | ✅ Complete |
|| [TASK-1604](tasks/TASK-1604-cps-ir-reference-documentation.md) | Add CPS IR reference documentation | ✅ Complete |
|| [TASK-1605](tasks/TASK-1605-cps-interpreter-reference-documentation.md) | Add CPS interpreter reference documentation | ✅ Complete |
||| [TASK-1606](tasks/TASK-1606-cps-operational-semantics-reference.md) | Add CPS operational semantics reference documentation | ✅ Complete |
||| [TASK-1607](tasks/TASK-1607-cps-operational-semantics-agent-card.md) | Add CPS operational semantics agent card | ✅ Complete |

## Phase 160: CPS IR Runtime Expansion

**Status:** ✅ Complete; 10/10 implemented — uses serde-based serialization (hand-written S-expression parser remains deferred)
**Plan:** [PLAN-160: CPS IR Runtime Expansion](PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)
**Spec:** [SPEC-098b: Ash Intermediate Representation — Target State](../spec/SPEC-098b-TARGET-IR.md)
**Depends on:** Phase 159 (CPS IR Interpreter)

Extends the Phase 159 CPS IR interpreter with structured data (records, tuples), constructor tags, pattern matching, and mutual recursion desugaring. Provides an objective testing ground for speculative upper-language lowering patterns.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1610](tasks/TASK-1610-cps-ir-record-tuple-values.md) | Add Record and Tuple value variants to CPS IR | ✅ Complete |
| [TASK-1611](tasks/TASK-1611-cps-ir-field-access-primitives.md) | Add RecordGet and TupleGet primitive operations | ✅ Complete |
| [TASK-1612](tasks/TASK-1612-cps-ir-constructor-tags.md) | Add ConstructorName atom variant for sum types | ✅ Complete |
| [TASK-1613](tasks/TASK-1613-cps-ir-match-dispatch.md) | Add Match term for pattern dispatch | ✅ Complete |
| [TASK-1614](tasks/TASK-1614-cps-ir-mutual-recursion-desugaring.md) | Support mutual recursion via tuple-of-lambdas in LetRec | ✅ Complete |
| [TASK-1615](tasks/TASK-1615-cps-ir-serde-extension.md) | Extend serde-based serialization for new IR forms | ✅ Complete |
| [TASK-1616](tasks/TASK-1616-cps-ir-speculative-fixtures.md) | Write speculative test fixtures for upper-language patterns | ✅ Complete |
| [TASK-1617](tasks/TASK-1617-cps-ir-expanded-operational-semantics.md) | Write operational semantics for new term forms (new doc) | ✅ Complete |
| [TASK-1618](tasks/TASK-1618-cps-ir-reference-docs-update.md) | Add reference documentation for expanded CPS IR | ✅ Complete |
| [TASK-1619](tasks/TASK-1619-phase-160-closeout.md) | Close out Phase 160 with verification and documentation | ✅ Complete |
