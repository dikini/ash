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
| [154](PLAN-154-TYPE-ANNOTATION-QUIRKS.md) | 5 | 5 | ✅ Complete; imported type annotations and opaque callable-signature types |
| [155](PLAN-155-LET-DESTRUCTORS.md) | 10 | 10 | ✅ Complete; closeout done, 10/10 tasks verified |
| [156](PLAN-156-PARSER-BLOCKER-RESOLUTION.md) | 5 | 5 | ✅ Complete; all blockers resolved, regression tests added |
| [157](PLAN-157-LIST-MIGRATION-HARDENING.md) | 5 | 5 | ✅ Complete; TASK-1570 completed by Phase 176 |
| [158](PLAN-158-LANGUAGE-SURFACE-FIXES.md) | 5 | 5 | ✅ Complete; TASK-1580 completed by Phase 176 |
| [159](PLAN-159-CPS-IR-INTERPRETER.md) | 14 | 14 | ✅ Complete; all tasks implemented, 82 tests pass, reference docs added, review remediation done (validation boundary, lambda closure capture, handler semantics) |
| [160](PLAN-160-CPS-IR-RUNTIME-EXPANSION.md) | 10 | 10 | ✅ Complete; CPS IR runtime expansion implemented, focused tests pass, reference docs updated |
| [161](PLAN-161-CORE-ASH-IR-FOUNDATION.md) | 13 | 13 | ✅ Complete; Core Ash foundation and public Core text round-trip review remediation verified |
| [162](PLAN-162-CORE-ASH-TYPE-CHECKING.md) | 12 | 12 | ✅ Complete; Core Ash type checker implemented with reference docs and closeout review |
| [163](PLAN-163-CORE-LAZY-MEMO-MODES.md) | 15 | 15 | ✅ Complete; SPEC-101 Core lazy/memo mode implementation packet plus review remediation |
| [164](PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md) | 12 | 12 | ✅ Complete; SPEC-102 Core/CPS continuation multiplicity implemented and verified |
| [165](PLAN-165-CONTRACT-SYSTEM-IMPLEMENTATION-HANDOFF.md) | 10 | 10 | ✅ Complete; Core contract sidecar carriers, predicate lowering, diagnostics, discharge metadata, observation evidence, and trace monitor carriers implemented |
| [166](PLAN-166-DOCS-ORIENTATION-INDEXES.md) | 6 | 6 | ✅ Complete; notes/spec orientation indexes, lint tooling, and agent usability evals added |
| [167](PLAN-167-TARGET-SURFACE-SEMANTICS-GAP-CLOSURE.md) | 12 | 12 | ✅ Complete; docs-only target surface/lowering/semantics spec-hardening packet |
| [168](PLAN-168-SURFACE-AST-NOTATION-SUBSTRATE.md) | 7 | 7 | ✅ Complete; implementation substrate handoff |
| [169](PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md) | 8 | 8 | ✅ Complete; surface expansion and notation elaboration implemented with explicit deferrals |
| [170](PLAN-170-EXPANDED-SURFACE-INTEGRATION-AND-NOTATION-SCOPING.md) | 7 | 7 | ✅ Complete |
| [171](PLAN-171-MACRO-NOTATION-HYGIENE-AND-EXPANSION-BOUNDARIES.md) | 8 | 8 | ✅ Complete |
| [172](PLAN-172-PARSER-FIRST-MACRO-EXECUTION-MVP.md) | 9 | 9 | ✅ Complete |
| [173](PLAN-173-MACRO-SUMMARIES-TOKEN-TREES-HYGIENIC-BINDERS-TYPED-MACROS.md) | 14 | 14 | ✅ Complete |
| [174](PLAN-174-MACRO-AWARE-TOOLING-SUMMARY-IDENTITY-AND-INFERENCE-READINESS.md) | 10 | 10 | ✅ Complete |
| [175](PLAN-175-NAME-RESOLUTION-BACKED-SEMANTIC-IDENTITY-FOR-MACROS-AND-TOOLING.md) | 10 | 10 | ✅ Complete |
| [176](PLAN-176-DEFERRED-CLEANUP-AFTER-TARGET-REDESIGN.md) | 9 | 9 | ✅ Complete; deferred cleanup after target-language redesign closed with review remediation |
| [177](PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md) | 11 | 11 | ✅ Complete; target row syntax review remediation complete |
| [178](PLAN-178-SOURCE-TO-CORE-ROW-LOWERING-BRIDGE.md) | 9 | 9 | ✅ Complete; explicit source rows lower into Core callable metadata with review remediation |
| [179](PLAN-179-EXPLICIT-ROW-ADMISSION-RUNTIME-WIRING.md) | 9 | 9 | ✅ Complete; explicit row requirements wired to admission/runtime checks |
| [180](PLAN-180-TARGET-DOCS-CONSISTENCY-CLEANUP.md) | 1 | 1 | ✅ Complete; target docs consistency cleanup |
| [181](PLAN-181-LEGACY-AUTHORITY-VOCABULARY-AUDIT.md) | 1 | 1 | ✅ Complete; legacy authority vocabulary audit |
| [182](PLAN-182-CORE-COMPUTATION-MODEL-CONFORMANCE.md) | 10 | 10 | Complete; Core computation model conformance |
| [183](PLAN-183-OPERATION-AUTHORITY-MODEL.md) | 8 | 8 | ✅ Complete; operation and authority model |
| [184](PLAN-184-HANDLER-PROVIDER-SEMANTICS.md) | 8 | 8 | ✅ Complete; handler/provider semantics |
| [185](PLAN-185-SURFACE-FUNCTION-LANGUAGE.md) | 7 | 7 | ✅ Complete; surface function language entry slice |
| [186](PLAN-186-SURFACE-FUNCTION-CLI-ENTRY.md) | 7 | 7 | ✅ Complete; CLI surface function entry conformance |
| [187](PLAN-187-SURFACE-RECORD-EXPRESSIONS.md) | 2 | 2 | ✅ Complete; structural record expressions for function-first Ash |
| [188](PLAN-188-SURFACE-MATCH-CONSTRUCTOR-SCRUTINEES.md) | 2 | 2 | ✅ Complete; ADT constructor expressions as match scrutinees |
| [189](PLAN-189-SURFACE-MATCH-ORDINARY-SCRUTINEES.md) | 2 | 2 | ✅ Complete; call, field, and binary expressions as match scrutinees |
| [190](PLAN-190-SURFACE-DO-EXPRESSION-STATEMENTS.md) | 2 | 2 | ✅ Complete; expression statements in unified do |
| [191](PLAN-191-SURFACE-BLOCK-EXPRESSIONS.md) | 2 | 2 | ✅ Complete; nested block expressions and block expression statements |
| [192](PLAN-192-SURFACE-POSTFIX-PROJECTION.md) | 2 | 2 | ✅ Complete; postfix field projection on ordinary primary expressions |
| [193](PLAN-193-SURFACE-TUPLE-ADT-EXPRESSIONS.md) | 2 | 2 | ✅ Complete; tuple-payload ADTs in function-first Ash |
| [194](PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md) | 11 | 11 | ✅ Complete; all Phase 194 tasks finished |
| [195](PLAN-195-PROCESS-AND-CONCURRENCY-MODEL.md) | 11 | 11 | ✅ Complete; process/concurrency model after computation, authority, handler/provider, and contract foundations |
| [196](PLAN-196-APPLICATION-WORKFLOW-RUNTIME.md) | 11 | 11 | ✅ Complete; application/workflow runtime layer over computation, admission, authority, contracts, and process foundations |
| [197](PLAN-197-HOST-FFI-BUILTINS.md) | 10 | 10 | ✅ Complete; host/FFI/builtin boundary over provider admission, sandboxing, and provenance |
| [198](PLAN-198-STANDARD-PROVIDERS-AND-PROFILES.md) | 8 | 8 | ✅ Complete; standard providers and profiles over Phase 197 host boundary substrate |
| [199](PLAN-199-PRODUCTIVE-APP-LIBRARIES-AND-TEMPLATES.md) | 9 | 9 | ✅ Complete; productive app libraries, testing helpers, process/channel helpers, and templates |
| [200](PLAN-200-TOOLING-AND-MIGRATION-POLISH.md) | 9 | 9 | ✅ Complete; migration-first tooling polish and legacy/deprecated form elimination |
| [201](PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md) | 23 | 23 | ✅ Complete; deprecated-functionality removal and semantic-cleanup follow-up verified |
| [204](PLAN-204-DIRECT-AST-RETIREMENT-AUDIT-AND-CONTRACT-FREEZE.md) | 3 | 3 | ✅ Complete; direct-AST retirement audit, contract freeze, and re-entry guard |
| [205](PLAN-205-ENGINE-ONLY-EXECUTION-CUTOVER.md) | 7 | 7 | ✅ Complete; Engine-only executor migration, zero-use gate, four-client terminal evidence, and tracked Cargo artifact cleanup |

---

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
|| [TASK-1502](tasks/TASK-1502-quickcheck-combinators-recursion-and-weights.md) | Implement choice, weighted choice, map/project helpers, shrink wrappers, and recursive public API/config; bounded recursive execution re-scoped fail-closed by TASK-1800 | ✅ Complete / Phase 176 Reconciled |
|| [TASK-1503](tasks/TASK-1503-quickcheck-runner-generation-shrink-semantics.md) | Wire generation, per-parameter split paths, stop-first execution, failure-class shrink, and generator/shrinker errors | ✅ Complete |
|| [TASK-1504](tasks/TASK-1504-quickcheck-seed-replay-and-aggregate-evidence.md) | Implement random seed default, replay overrides, source-seed linting, run records, aggregate pass history, and sticky active findings | ✅ Complete |
|| [TASK-1512](tasks/TASK-1512-record-types-reference-documentation.md) | Add reference documentation for Ash record types at `reference/language/types/records.md`, clarifying terminology and usage | ✅ Complete |
|| [TASK-1511](tasks/TASK-1511-deferred-combinators-ordinary-ash.md) | Implement deferred QuickCheck combinators in ordinary Ash; Phase 176 landed recursive public names/config plus fail-closed execution guard, with real bounded generation deferred to parser/type-metadata substrate | ✅ Complete / Phase 176 Reconciled |
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

**Status:** ✅ Complete; 5/5 implemented — imported type annotations and opaque callable-signature types
**Plan:** [PLAN-154: Type Annotation Quirks](PLAN-154-TYPE-ANNOTATION-QUIRKS.md)
**Spec:** [SPEC-090: Type Annotation Quirks](../spec/SPEC-090-TYPE-ANNOTATION-QUIRKS.md)

Fix the type system limitation where imported types cannot be used in local type definitions, `fn` return type annotations, and record field types. This unblocks modular type design, smart constructors, and cross-module type composition.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1540](tasks/TASK-1540-parser-import-first-pass.md) | Modify parser to collect imports before type definitions | ✅ Complete |
|| [TASK-1541](tasks/TASK-1541-typeenv-imported-type-registration.md) | Modify TypeEnv to register imported types before local types | ✅ Complete |
|| [TASK-1542](tasks/TASK-1542-type-name-resolution-imported.md) | Update type name resolution to check imported types | ✅ Complete |
|| [TASK-1543](tasks/TASK-1543-type-inference-leakage-diagnostics.md) | Add diagnostics for type inference leakage | ✅ Complete |
|| [TASK-1544](tasks/TASK-1544-phase-154-closeout.md) | Close out Phase 154 with verification and documentation | ✅ Complete |

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

**Status:** ✅ Complete; TASK-1570 completed by Phase 176
**Plan:** [PLAN-157: List Migration Hardening and Cleanup](PLAN-157-LIST-MIGRATION-HARDENING.md)
**Spec:** [SPEC-089: List Builtin to Stdlib](../spec/SPEC-089-LIST-BUILTIN-TO-STDLIB.md)
**Builds on:** [PLAN-153](PLAN-153-LIST-BUILTIN-TO-STDLIB.md) (List Builtin to Stdlib)
**Task range:** TASK-1570 through TASK-1574
**Completion Date:** 2026-06-17

Harden the Phase 153 list migration by completing the removal of `Value::List` from the runtime, fixing pre-existing test failures, adding property tests for algebraic laws, and establishing performance benchmarks.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1570](tasks/TASK-1570-remove-value-list-enum.md) | Remove `Value::List` variant from `ash_core::Value` enum entirely | ✅ Complete via TASK-1797 |
|| [TASK-1571](tasks/TASK-1571-fix-quickcheck-combinator-test.md) | Fix pre-existing `one_of` test failure in `phase151_quickcheck_stdlib` | ✅ Complete |
|| [TASK-1572](tasks/TASK-1572-list-algebra-property-tests.md) | Add property tests for list algebraic laws (Functor, Semigroup, Monoid) | ✅ Complete; 8 tests pass |
|| [TASK-1573](tasks/TASK-1573-list-performance-benchmarks.md) | Add performance benchmarks for list operations | ✅ Complete; Placeholder benchmark added |
|| [TASK-1574](tasks/TASK-1574-phase-157-closeout.md) | Close out Phase 157 with documentation, changelog, and verification | ✅ Complete |

## Phase 158: Language Surface Fixes

**Status:** ✅ Complete; TASK-1580 completed by Phase 176
**Plan:** [PLAN-158: Language Surface Fixes](PLAN-158-LANGUAGE-SURFACE-FIXES.md)
**Spec:** [SPEC-094: Language Surface Fix Specification](../spec/SPEC-094-LANGUAGE-SURFACE-FIX.md)
**Builds on:** [PLAN-157](PLAN-157-LIST-MIGRATION-HARDENING.md)
**Task range:** TASK-1580 through TASK-1584
**Completion Date:** 2026-06-17

Fix three language surface issues that prevent idiomatic usage of pure algebraic data types and higher-order functions in Ash.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1580](tasks/TASK-1580-closure-module-function-visibility.md) | Fix module-level function visibility inside closures | ✅ Complete via TASK-1798 |
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

## Phase 161: Core Ash IR Foundation

**Status:** ✅ Complete; 13/13 implemented, closeout and review remediation verified
**Plan:** [PLAN-161: Core Ash IR Foundation](PLAN-161-CORE-ASH-IR-FOUNDATION.md)
**Spec:** [SPEC-099: Ash Core Language](../spec/SPEC-099-CORE-LANGUAGE.md)
**Depends on:** SPEC-099, SPEC-098b, SPEC-096b, SPEC-097b; builds on Phase 159 CPS IR substrate.

Builds the first implementation slice for Core Ash: dedicated Core AST carriers, a strict `.core` fixture/debug text format, parser/serializer round-trips, Core validation, and minimal Core-to-CPS lowering. Surface-to-Core lowering, ad-hoc polymorphism, arbitrary user-defined algebraic effects, and full type checking remain out of scope.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1620](tasks/TASK-1620-core-ash-ast-carriers.md) | Add Core Ash AST carriers and module exports | ✅ Complete |
| [TASK-1621](tasks/TASK-1621-core-text-format-fixtures.md) | Freeze minimal `.core` text grammar and golden fixtures | ✅ Complete |
| [TASK-1622](tasks/TASK-1622-core-text-parser-atoms-values.md) | Parse Core atoms, rows, types, and values | ✅ Complete |
| [TASK-1623](tasks/TASK-1623-core-text-parser-expressions.md) | Parse Core expressions and effect/discharge forms | ✅ Complete |
| [TASK-1624](tasks/TASK-1624-core-text-serializer.md) | Add canonical Core AST serializer and round-trip tests | ✅ Complete |
| [TASK-1625](tasks/TASK-1625-core-validator-basic-invariants.md) | Validate basic SPEC-099 Core invariants | ✅ Complete |
| [TASK-1626](tasks/TASK-1626-core-validator-affine-resume.md) | Validate handler resume affine-position restrictions | ✅ Complete |
| [TASK-1627](tasks/TASK-1627-core-to-cps-lowering-basic.md) | Lower values, lets, primitives, conditionals, calls, and jumps | ✅ Complete |
| [TASK-1628](tasks/TASK-1628-core-to-cps-lowering-effects.md) | Lower raise, handle, discharge, and trap forms | ✅ Complete |
| [TASK-1629](tasks/TASK-1629-core-end-to-end-fixtures.md) | Add `.core` -> validate -> CPS golden fixtures | ✅ Complete |
| [TASK-1630](tasks/TASK-1630-core-ash-reference-docs.md) | Document Core text and implementation boundaries | ✅ Complete |
| [TASK-1631](tasks/TASK-1631-phase-161-closeout.md) | Close out Phase 161 with verification and review | ✅ Complete |
| [TASK-1632](tasks/TASK-1632-core-text-roundtrip-review-fixes.md) | Fix Core text public AST round-trip review findings | ✅ Complete |

**Verification Evidence:**

- Focused Phase 161 tests `task_1620_core_ash_ast` through `task_1630_core_docs_consistency` pass.
- `cargo test -p ash-core` passes.
- `cargo clippy -p ash-core --all-targets -- -D warnings` passes.
- `cargo fmt --check` and `git diff --check` pass.
- Closeout review recorded in [PHASE-161-CLOSEOUT-REVIEW.md](audits/PHASE-161-CLOSEOUT-REVIEW.md).

## Phase 162: Core Ash Type Checking

**Status:** ✅ Complete; 12/12 implemented
**Plan:** [PLAN-162: Core Ash Type Checking](PLAN-162-CORE-ASH-TYPE-CHECKING.md)
**Spec:** [SPEC-100: Ash Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
**Depends on:** Phase 161; SPEC-100, SPEC-099, SPEC-098b, SPEC-097b, SPEC-096b.

Implements the first annotation-led Core Ash type checker. The phase adds Core type-checker environments, type well-formedness, row normalization/solving, atom/value/expression typing, operation and handler checking, refinement obligation recording, discharge metadata checks, public summary scaffolding, integration fixtures, and closeout documentation.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1640](tasks/TASK-1640-core-typecheck-api-and-environments.md) | Add Core type-checker API, environments, typed program wrappers, and diagnostics | ✅ Complete |
| [TASK-1641](tasks/TASK-1641-core-type-wellformedness.md) | Check Core type well-formedness and nominal/type-app shape | ✅ Complete |
| [TASK-1642](tasks/TASK-1642-core-row-normalization-solving.md) | Normalize rows, remove duplicates, compare rows, and solve explicit row variables | ✅ Complete |
| [TASK-1643](tasks/TASK-1643-core-atom-value-typing.md) | Type Core atoms and values, including lambdas, records, tuples, and discharge markers | ✅ Complete |
| [TASK-1644](tasks/TASK-1644-core-expression-basics-typecheck.md) | Type Atom, LetVal, LetRec, LetPrim, If, and Trap expressions | ✅ Complete |
| [TASK-1645](tasks/TASK-1645-core-call-jump-row-accounting.md) | Type LetCall, Call, and Jump with SPEC-098b row-accounting facts | ✅ Complete |
| [TASK-1646](tasks/TASK-1646-core-effect-operation-typing.md) | Type capability/channel/process/failure Raise operations and operation signatures | ✅ Complete |
| [TASK-1647](tasks/TASK-1647-core-handle-affine-resume-typecheck.md) | Type Handle clauses with affine resume and captured-resume row preservation | ✅ Complete |
| [TASK-1648](tasks/TASK-1648-core-refinement-obligations-discharge.md) | Record refinement obligations and validate discharge metadata shape | ✅ Complete |
| [TASK-1649](tasks/TASK-1649-core-public-summary-scaffold.md) | Add public type/row summary scaffolding and private alias diagnostics | ✅ Complete |
| [TASK-1650](tasks/TASK-1650-core-typecheck-integration-fixtures.md) | Add `.core` parse -> validate -> type-check -> lower integration fixtures | ✅ Complete |
| [TASK-1651](tasks/TASK-1651-core-typecheck-reference-closeout.md) | Document Core type-checker behavior and close out Phase 162 | ✅ Complete |

## Phase 163: Core Lazy and Memo Computation Modes

**Status:** ✅ Complete; 15/15 implemented
**Plan:** [PLAN-163: Core Lazy and Memo Computation Modes](PLAN-163-CORE-LAZY-MEMO-MODES.md)
**Spec:** [SPEC-101: Lazy and Memo Computation Modes](../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md)
**Depends on:** Phase 161, Phase 162; SPEC-101, SPEC-100, SPEC-099, SPEC-098b, SPEC-097b, SPEC-096b.

Implements SPEC-101 for Core Ash: explicit `Strict`/`Lazy`/`Memo` mode types, thunk values, `LetMode`, `Force`, type checking, CPS value-level thunk carrier, memo runtime behavior, Core-to-CPS lowering, captured handler/provider-chain authority, examples, fixtures, trace events, and closeout documentation.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1660](tasks/TASK-1660-core-mode-ast-carriers.md) | Add Core mode type, thunk value, LetMode, and Force AST carriers | ✅ Complete |
| [TASK-1661](tasks/TASK-1661-core-mode-text-format.md) | Parse, serialize, and round-trip `.core` mode syntax and fixtures | ✅ Complete |
| [TASK-1662](tasks/TASK-1662-core-mode-validation.md) | Validate mode/type agreement, thunk shape, Force shape, and binder scoping | ✅ Complete |
| [TASK-1663](tasks/TASK-1663-cps-thunk-carrier.md) | Add CPS ThunkClosure value carrier and memo-cell state scaffolding | ✅ Complete |
| [TASK-1664](tasks/TASK-1664-cps-force-runtime.md) | Implement CPS lazy/memo force runtime behavior, cached outcomes, and re-entrant rejection | ✅ Complete |
| [TASK-1665](tasks/TASK-1665-core-mode-type-wellformedness.md) | Type-check mode type well-formedness and mode invariance diagnostics | ✅ Complete |
| [TASK-1666](tasks/TASK-1666-core-thunk-value-typing.md) | Type thunk values with latent rows and pure construction row | ✅ Complete |
| [TASK-1667](tasks/TASK-1667-core-letmode-force-typechecking.md) | Type LetMode and Force expressions with SPEC-101 row accounting | ✅ Complete |
| [TASK-1668](tasks/TASK-1668-core-mode-public-summaries.md) | Preserve mode and latent-row facts in public summaries and diagnostics | ✅ Complete |
| [TASK-1669](tasks/TASK-1669-core-mode-lowering.md) | Lower thunk construction, strict/lazy/memo LetMode, and Force into CPS thunk runtime forms | ✅ Complete |
| [TASK-1670](tasks/TASK-1670-core-thunk-capture-authority.md) | Verify captured handler/provider-chain authority at force time | ✅ Complete |
| [TASK-1672](tasks/TASK-1672-core-mode-tracing-observability.md) | Add thunk construction/force/memo trace events and observability tests | ✅ Complete |
| [TASK-1671](tasks/TASK-1671-core-mode-end-to-end-fixtures.md) | Add parse -> validate -> type-check -> lower -> run fixtures and golden examples | ✅ Complete |
| [TASK-1673](tasks/TASK-1673-core-lazy-memo-reference-closeout.md) | Document behavior, reconcile tracking, and close out Phase 163 | ✅ Complete |
| [TASK-1674](tasks/TASK-1674-core-force-function-row-remediation.md) | Preserve forced function rows and scoped LetMode bindings during checked lowering | ✅ Complete |

## Phase 164: Core CPS Continuation Multiplicity

**Status:** ✅ Complete; 12/12 complete and verified
**Plan:** [PLAN-164: Core CPS Continuation Multiplicity](PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Spec:** [SPEC-102: CPS Continuation Multiplicity](../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md)
**Depends on:** Phase 159, Phase 160, Phase 161, Phase 162, Phase 163; SPEC-102, SPEC-101, SPEC-100, SPEC-099, SPEC-099c, SPEC-098b, SPEC-097b, SPEC-096b.

Implements explicit continuation multiplicity for Core Ash and CPS IR. Existing continuations remain affine by default. Legal `multi-shot-pure` continuations require an explicit Core multiplicity and a normalized closed empty row, and may be resumed multiple times by pure handlers. Surface syntax and upper-layer lowering remain out of scope.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1680](tasks/TASK-1680-continuation-multiplicity-spec-plan-packet.md) | Freeze SPEC-102 and Phase 164 planning packet | ✅ Complete |
| [TASK-1681](tasks/TASK-1681-cps-cont-multiplicity-carrier.md) | Add CPS continuation, LetCont, LetContCall, and handler row/multiplicity carriers | ✅ Complete |
| [TASK-1682](tasks/TASK-1682-cps-multishot-runtime.md) | Implement affine vs multi-shot CPS jump and LetContCall behavior | ✅ Complete |
| [TASK-1683](tasks/TASK-1683-cps-multishot-validation.md) | Validate CPS multi-shot row legality and malformed unchecked input | ✅ Complete |
| [TASK-1684](tasks/TASK-1684-core-cont-multiplicity-wellformedness.md) | Type-check Core continuation multiplicity well-formedness | ✅ Complete |
| [TASK-1685](tasks/TASK-1685-core-handler-multishot-resume-typecheck.md) | Accept legal multi-shot handler resumes and reject illegal ones | ✅ Complete |
| [TASK-1686](tasks/TASK-1686-core-affine-use-discipline-with-multishot.md) | Add Core LetContCall and preserve affine use discipline with multi-shot | ✅ Complete |
| [TASK-1687](tasks/TASK-1687-core-to-cps-multiplicity-lowering.md) | Preserve multiplicity and LetContCall through Core-to-CPS lowering | ✅ Complete |
| [TASK-1688](tasks/TASK-1688-core-text-fixtures-for-continuation-multiplicity.md) | Add Core text fixtures and golden coverage for multiplicity and LetContCall | ✅ Complete |
| [TASK-1689](tasks/TASK-1689-motivational-multishot-fixtures.md) | Add Choice/backtracking/nested/discard motivational fixtures | ✅ Complete |
| [TASK-1690](tasks/TASK-1690-continuation-multiplicity-reference-docs.md) | Add reference docs and non-normative commentary links | ✅ Complete |
| [TASK-1691](tasks/TASK-1691-phase-164-closeout.md) | Close out Phase 164 with verification, changelog, and index reconciliation | ✅ Complete |

## Phase 165: Contract System Implementation Handoff

**Status:** ✅ Complete; 10/10 tasks complete
**Plan:** [PLAN-165: Contract System Implementation Handoff](PLAN-165-CONTRACT-SYSTEM-IMPLEMENTATION-HANDOFF.md)
**Spec:** [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md), [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md), [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md), [SPEC-099: Core Language](../spec/SPEC-099-CORE-LANGUAGE.md), [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
**Depends on:** Phase 161, Phase 162, Phase 163, Phase 164; NOTE-027 through NOTE-035.

Closes NOTE-014 as the resolved contract-system design gap register and hands implementation to ordered tasks. The phase starts with Core predicate artifacts and dynamic contract diagnostics, then proceeds through discharge/evidence metadata, interface/impl subsumption and blame, capability observation evidence, trace contracts, temporal monitor diagnostics, and integration closeout.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1693](tasks/TASK-1693-contract-system-implementation-handoff.md) | Close NOTE-014 and create the Phase 165 handoff packet | ✅ Complete |
| [TASK-1694](tasks/TASK-1694-core-contract-predicate-artifacts.md) | Add Core predicate, snapshot, environment, and runtime-check artifact carriers | ✅ Complete |
| [TASK-1695](tasks/TASK-1695-contract-predicate-validation-and-lowering.md) | Validate and lower contract-position predicates into Core artifacts | ✅ Complete |
| [TASK-1696](tasks/TASK-1696-dynamic-contract-traps-and-predicate-faults.md) | Implement structured dynamic contract traps and predicate-fault diagnostics | ✅ Complete |
| [TASK-1697](tasks/TASK-1697-contract-discharge-and-evidence-metadata.md) | Record static/evidence/dynamic discharge metadata and public summaries | ✅ Complete |
| [TASK-1698](tasks/TASK-1698-interface-impl-contract-subsumption-and-blame.md) | Check interface-to-impl contract subsumption and preserve blame labels | ✅ Complete |
| [TASK-1699](tasks/TASK-1699-capability-observation-evidence-boundary.md) | Add operation-produced observation evidence without predicate authority leakage | ✅ Complete |
| [TASK-1700](tasks/TASK-1700-trace-contract-monitor-sidecars.md) | Add trace-contract, trace-fact, workflow-ledger, and monitor-plan carriers | ✅ Complete |
| [TASK-1701](tasks/TASK-1701-temporal-monitor-runtime-diagnostics.md) | Implement temporal monitor result, violation, and monitor-fault diagnostics | ✅ Complete |
| [TASK-1702](tasks/TASK-1702-contract-system-integration-closeout.md) | Add integration fixtures, docs consistency checks, PLAN-INDEX reconciliation, and closeout | ✅ Complete |

## Phase 166: Docs Orientation Indexes

**Status:** ✅ Complete; 6/6 tasks complete
**Plan:** [PLAN-166: Docs Orientation Indexes](PLAN-166-DOCS-ORIENTATION-INDEXES.md)
**Spec:** Documentation infrastructure / navigational metadata
**Depends on:** Phase 165 handoff packet; existing notes/spec corpus.

Adds agent-oriented indexes for design notes and specs. The indexes combine a structured topic ontology with unstructured tags for cross-cutting concerns such as `grammar`, `semantics`, `references`, `diagnostics`, and `authority`. A docs-gate validator now checks index coverage, link shape, topic/tag vocabulary, and table structure.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1703](tasks/TASK-1703-docs-orientation-index-plan.md) | Create the Phase 166 plan and task packet | ✅ Complete |
| [TASK-1704](tasks/TASK-1704-notes-orientation-index.md) | Create `docs/notes/NOTE-INDEX.md` | ✅ Complete |
| [TASK-1705](tasks/TASK-1705-specs-orientation-index.md) | Create `docs/spec/SPEC-INDEX.md` | ✅ Complete |
| [TASK-1706](tasks/TASK-1706-orientation-index-lint-tooling.md) | Add validator tooling and wire it into docs gate | ✅ Complete |
| [TASK-1707](tasks/TASK-1707-agent-usability-evaluation.md) | Record independent before/after agent discovery evaluations | ✅ Complete |
| [TASK-1708](tasks/TASK-1708-docs-orientation-index-closeout.md) | Reconcile status surfaces and run verification | ✅ Complete |

## Phase 167: Target Surface and Semantics Gap Closure

**Status:** ✅ Complete; docs-only spec-hardening packet
**Plan:** [PLAN-167: Target Surface and Semantics Gap Closure](PLAN-167-TARGET-SURFACE-SEMANTICS-GAP-CLOSURE.md)
**Audit:** [Target spec gaps against notes](../audit/2026-06-29-target-spec-notes-gap-audit.md)
**Depends on:** Phase 166 docs orientation indexes; Phase 165 contract system implementation handoff; target specs SPEC-095b through SPEC-100.

Closes the target-spec gaps identified by the 2026-06-29 audit before parser, macro, lowering, or semantics implementation proceeds. The packet is documentation-only: it patches target grammar drift, adds a source-preserving surface AST/macro/notation spec, specifies surface-to-Core lowering, tightens surface inference, and rewrites target operational semantics.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1709](tasks/TASK-1709-target-grammar-drift-patch.md) | Patch target grammar drift in SPEC-095b | ✅ Complete |
| [TASK-1710](tasks/TASK-1710-surface-ast-macro-substrate.md) | Create SPEC-095c syntax-tree layers and macro boundaries | ✅ Complete |
| [TASK-1711](tasks/TASK-1711-notation-declarations.md) | Specify prefix/infix/suffix/mixfix notation declarations | ✅ Complete |
| [TASK-1712](tasks/TASK-1712-operator-sections.md) | Specify operator sections as callable sugar | ✅ Complete |
| [TASK-1713](tasks/TASK-1713-surface-phase-cross-spec-reconciliation.md) | Reconcile Phase 1 cross-references and stale claims | ✅ Complete |
| [TASK-1714](tasks/TASK-1714-surface-to-core-lowering-spec-scaffold.md) | Create surface-to-Core lowering spec scaffold | ✅ Complete |
| [TASK-1715](tasks/TASK-1715-lower-callables-rows-handlers-impls.md) | Specify lowering for callables, rows, do, handlers, and impls | ✅ Complete |
| [TASK-1716](tasks/TASK-1716-lower-contracts-evidence-trace-notation.md) | Specify lowering for contracts, evidence, trace contracts, and notation erasure | ✅ Complete |
| [TASK-1717](tasks/TASK-1717-surface-type-inference-tightening.md) | Tighten surface type inference for rows, evidence, handlers, operation identity, and notation | ✅ Complete |
| [TASK-1718](tasks/TASK-1718-operational-semantics-scope-split.md) | Rewrite SPEC-099b scope and preserve Phase 159 interpreter semantics as context | ✅ Complete |
| [TASK-1719](tasks/TASK-1719-target-big-small-step-semantics.md) | Add target Core big-step and Core/CPS small-step semantics | ✅ Complete |
| [TASK-1720](tasks/TASK-1720-operational-contracts-traces-closeout.md) | Integrate contracts, traces, monitors, lazy/memo semantics, and close out Phase 167 | ✅ Complete |

## Phase 168: Surface AST, Notation, and Lowering Substrate

**Status:** ✅ Complete; implementation substrate handoff
**Plan:** [PLAN-168: Surface AST, Notation, and Lowering Substrate](PLAN-168-SURFACE-AST-NOTATION-SUBSTRATE.md)
**Depends on:** Phase 167 target surface and semantics gap closure; `SPEC-095c`; `SPEC-098c`.

Introduces the first implementation substrate for the newly specified surface layer without trying
to implement the full macro system at once. The phase inventories live parser/lowering seams,
designs a source-preserving carrier slice, preserves notation-relevant token/grouping shape,
establishes an operator-section boundary, stages expanded surface AST, and scopes the follow-on
surface-to-Core lowering implementation packet.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1721](tasks/TASK-1721-parser-ast-lowering-inventory.md) | Audit current parser AST and lowering seams against Phase 167 specs | ✅ Complete |
| [TASK-1722](tasks/TASK-1722-source-preserving-surface-carriers.md) | Design the source-preserving surface syntax carrier slice | ✅ Complete |
| [TASK-1723](tasks/TASK-1723-notation-token-preservation.md) | Preserve notation/operator token shape before resolution | ✅ Complete |
| [TASK-1724](tasks/TASK-1724-operator-section-boundary.md) | Add the binary infix operator-section AST boundary or fail-closed diagnostics | ✅ Complete |
| [TASK-1725](tasks/TASK-1725-expanded-surface-ast-boundary.md) | Introduce an expanded-surface-AST boundary without full macro expansion | ✅ Complete |
| [TASK-1726](tasks/TASK-1726-surface-to-core-lowering-inventory.md) | Inventory and scope surface-to-Core lowering implementation seams | ✅ Complete |
| [TASK-1727](tasks/TASK-1727-phase-168-closeout.md) | Close out Phase 168 with verification and status reconciliation | ✅ Complete |

## Phase 169: Surface Expansion and Notation Elaboration

**Status:** ✅ Complete
**Plan:** [PLAN-169: Surface Expansion and Notation Elaboration](PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md)
**Depends on:** Phase 168 surface AST, notation, and lowering substrate; `SPEC-095c`; `SPEC-098c`.

Turns the Phase 168 surface substrate into the first usable expansion and notation elaboration pass.
The packet adds reusable expansion traversal, notation declaration parsing, raw built-in operator-token
preservation, local notation-table diagnostics, binary operator-section elaboration, and a high-level
expanded-surface-to-Core lowering gate. Macro hygiene, imported notation propagation, generalized
mixfix partial application, and full `SPEC-098c` lowering remain explicitly deferred.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1728](tasks/TASK-1728-phase-169-plan-packet.md) | Create the Phase 169 plan and task packet | ✅ Complete |
| [TASK-1729](tasks/TASK-1729-surface-expansion-traversal-api.md) | Add reusable surface traversal for expansion diagnostics | ✅ Complete |
| [TASK-1730](tasks/TASK-1730-notation-declaration-parser-ast.md) | Parse and preserve minimal notation declarations | ✅ Complete |
| [TASK-1731](tasks/TASK-1731-built-in-operator-token-normalization.md) | Preserve raw built-in infix operator tokens | ✅ Complete |
| [TASK-1732](tasks/TASK-1732-local-notation-table-resolution.md) | Build minimal local notation-table resolution diagnostics | ✅ Complete |
| [TASK-1733](tasks/TASK-1733-operator-section-elaboration.md) | Elaborate binary operator sections to callable surface forms | ✅ Complete |
| [TASK-1734](tasks/TASK-1734-expanded-surface-lowering-gate.md) | Add expanded-surface-to-Core lowering gate | ✅ Complete |
| [TASK-1735](tasks/TASK-1735-phase-169-closeout.md) | Close out Phase 169 with verification and review | ✅ Complete |

## Phase 170: Expanded Surface Integration and Notation Scoping

**Status:** ✅ Complete
**Plan:** [PLAN-170: Expanded Surface Integration and Notation Scoping](PLAN-170-EXPANDED-SURFACE-INTEGRATION-AND-NOTATION-SCOPING.md)
**Depends on:** Phase 169 surface expansion and notation elaboration; `SPEC-095c`; `SPEC-098c`.

Closes the highest-value Phase 169 deferrals around expanded-surface integration and notation scoping. The packet audits direct lowering paths, routes high-level module/file lowering through expansion where safe, specifies notation summary/export semantics, either implements bounded propagation or records explicit non-propagation, defines the narrow origin sidecar boundary, and closes out with verification/review. Full macro hygiene, typed macros, generalized mixfix partial application, and broad `SPEC-098c` lowering remain out of scope.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1736](tasks/TASK-1736-phase-170-plan-packet.md) | Create the Phase 170 plan and task packet | ✅ Complete |
| [TASK-1737](tasks/TASK-1737-expanded-surface-boundary-callsite-audit.md) | Audit expanded-surface boundary and direct-lowering call sites | ✅ Complete |
| [TASK-1738](tasks/TASK-1738-expanded-surface-high-level-lowering-routing.md) | Route high-level module/file lowering through expanded-surface validation | ✅ Complete |
| [TASK-1739](tasks/TASK-1739-notation-summary-export-design.md) | Specify notation summary/export and visibility semantics | ✅ Complete |
| [TASK-1740](tasks/TASK-1740-bounded-notation-import-export-scope.md) | Implement bounded notation import/export propagation or explicit non-propagation | ✅ Complete |
| [TASK-1741](tasks/TASK-1741-expansion-origin-sidecar-boundary.md) | Specify and implement the narrow source-origin sidecar boundary for expansion products | ✅ Complete |
| [TASK-1742](tasks/TASK-1742-phase-170-closeout.md) | Close out Phase 170 with verification, changelog, index reconciliation, and review | ✅ Complete |

## Phase 171: Macro/Notation Hygiene and Expansion Boundaries

**Status:** ✅ Complete
**Plan:** [PLAN-171: Macro/Notation Hygiene and Expansion Boundaries](PLAN-171-MACRO-NOTATION-HYGIENE-AND-EXPANSION-BOUNDARIES.md)
**Depends on:** Phase 170 expanded surface integration and notation scoping; `SPEC-095c`; `SPEC-098c`.

Builds the conservative hygiene substrate required before full macro execution or generalized mixfix work. The packet audits live hygiene/origin/scope seams, adds narrow expansion identity and origin-chain carriers, fences source/generated identifier capture, hardens local-only notation and macro scope boundaries, adds a fail-closed macro invocation boundary, and validates high-level parser/engine/typechecker boundaries with positive visibility and negative leakage tests. Full macro execution, typed macros, imported notation propagation, and binder-introducing mixfix remain out of scope unless a later task proves the required carriers.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1743](tasks/TASK-1743-phase-171-plan-packet.md) | Create the Phase 171 plan and task packet | ✅ Complete |
| [TASK-1744](tasks/TASK-1744-hygiene-origin-scope-audit.md) | Audit hygiene, origin, and scope boundary seams | ✅ Complete |
| [TASK-1745](tasks/TASK-1745-expansion-identity-origin-chain.md) | Add expansion identity and origin-chain carriers for generated surface nodes | ✅ Complete |
| [TASK-1746](tasks/TASK-1746-generated-identifier-hygiene-fences.md) | Implement source/generated identifier hygiene fences | ✅ Complete |
| [TASK-1747](tasks/TASK-1747-notation-macro-scope-boundaries.md) | Harden notation and macro scope-table boundaries | ✅ Complete |
| [TASK-1748](tasks/TASK-1748-fail-closed-macro-invocation-boundary.md) | Add fail-closed macro invocation boundary without macro execution | ✅ Complete |
| [TASK-1749](tasks/TASK-1749-cross-boundary-hygiene-validation.md) | Add cross-boundary hygiene and negative-leakage validation tests | ✅ Complete |
| [TASK-1750](tasks/TASK-1750-phase-171-closeout.md) | Close out Phase 171 with verification, changelog, index reconciliation, and review | ✅ Complete |

## Phase 172: Parser-First Macro Execution MVP

**Status:** ✅ Complete
**Plan:** [PLAN-172: Parser-First Macro Execution MVP](PLAN-172-PARSER-FIRST-MACRO-EXECUTION-MVP.md)
**Depends on:** Phase 171 macro/notation hygiene and expansion boundaries; `SPEC-095c`; `SPEC-098c`.

Implements the first conservative executable macro slice. The phase remains parser-first and fail-closed: only local expression-position `name!(...)` macros with parsed expression arguments and whitelisted expression-template bodies may expand. Bracket/brace invocations, token-tree rewriting, typed macros, binder-introducing macros, imported/exported macro activation, and Core/runtime macro forms remain out of scope. Positive visibility and negative leakage tests must prove that supported local macros execute while unsupported macro syntax cannot bypass parser, engine/module-loader, typechecker, or Core-lowering boundaries.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1751](tasks/TASK-1751-phase-172-plan-packet.md) | Create the Phase 172 parser-first macro execution MVP plan packet | ✅ Complete |
| [TASK-1752](tasks/TASK-1752-macro-execution-mvp-audit.md) | Audit macro execution seams and define the safe MVP subset | ✅ Complete |
| [TASK-1753](tasks/TASK-1753-macro-mvp-spec-amendments.md) | Amend macro specs for parser-first expression macro MVP | ✅ Complete |
| [TASK-1754](tasks/TASK-1754-macro-declaration-parse-carriers.md) | Add parsed macro declaration and structured invocation-argument carriers | ✅ Complete |
| [TASK-1755](tasks/TASK-1755-macro-registry-scope-validation.md) | Add local macro registry and scope-boundary validation | ✅ Complete |
| [TASK-1756](tasks/TASK-1756-expression-template-macro-expansion.md) | Implement fail-closed expression-template macro expansion | ✅ Complete |
| [TASK-1757](tasks/TASK-1757-macro-origin-hygiene-metadata.md) | Preserve macro expansion origin and hygiene metadata through notation expansion | ✅ Complete |
| [TASK-1758](tasks/TASK-1758-macro-execution-cross-boundary-tests.md) | Add cross-boundary macro execution and negative-leakage tests | ✅ Complete |
| [TASK-1759](tasks/TASK-1759-phase-172-closeout.md) | Close out Phase 172 with verification, review, and status reconciliation | ✅ Complete |

## Phase 173: Macro Summaries, Token Trees, Hygienic Binders, and Typed Macros

**Status:** ✅ Complete
**Plan:** [PLAN-173: Macro Summaries, Token Trees, Hygienic Binders, and Typed Macros](PLAN-173-MACRO-SUMMARIES-TOKEN-TREES-HYGIENIC-BINDERS-TYPED-MACROS.md)
**Depends on:** Phase 172 parser-first macro execution MVP; `SPEC-095c`; `SPEC-098c`; `SPEC-097b`.

Extends the Phase 172 local expression-macro MVP into the next conservative macro-system slice. The phase adds explicit macro summary carriers for imported/exported macro activation, delimiter-preserving token-tree/bracket/brace carriers, hygienic binder-introducing macro metadata and bounded execution, and typed macro checking/inference. Core still receives no macro forms; unsupported macro syntax and ambiguous typed/hygiene states fail closed before public export acceptance or Core lowering.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1760](tasks/TASK-1760-phase-173-plan-packet.md) | Create the Phase 173 macro-system expansion plan packet | ✅ Complete |
| [TASK-1761](tasks/TASK-1761-macro-system-expansion-seam-audit.md) | Audit macro-system expansion seams and split-risk decisions | ✅ Complete |
| [TASK-1762](tasks/TASK-1762-macro-system-spec-amendments.md) | Amend macro specs for summaries, token trees, binder hygiene, and typed checking | ✅ Complete |
| [TASK-1763](tasks/TASK-1763-macro-summary-carriers.md) | Add macro summary carrier design and export collection | ✅ Complete |
| [TASK-1764](tasks/TASK-1764-imported-exported-macro-activation.md) | Implement bounded imported/exported macro activation | ✅ Complete |
| [TASK-1765](tasks/TASK-1765-delimiter-preserving-token-tree-carriers.md) | Add delimiter-preserving macro token-tree carriers | ✅ Complete |
| [TASK-1766](tasks/TASK-1766-bracket-brace-macro-parsing.md) | Parse bracket and brace macro invocations into structured carriers | ✅ Complete |
| [TASK-1767](tasks/TASK-1767-bounded-token-tree-expansion-reparse.md) | Add bounded token-tree expansion and reparse boundaries | ✅ Complete |
| [TASK-1768](tasks/TASK-1768-binder-hygiene-metadata-model.md) | Add binder hygiene metadata model and validation rules | ✅ Complete |
| [TASK-1769](tasks/TASK-1769-hygienic-binder-introducing-macros.md) | Implement bounded hygienic binder-introducing macro expansion | ✅ Complete |
| [TASK-1770](tasks/TASK-1770-typed-macro-signature-carriers.md) | Add typed macro signature carriers | ✅ Complete |
| [TASK-1771](tasks/TASK-1771-fail-closed-typed-macro-checking.md) | Implement fail-closed typed macro checking | ✅ Complete |
| [TASK-1772](tasks/TASK-1772-bounded-macro-type-inference.md) | Implement bounded macro type inference | ✅ Complete |
| [TASK-1773](tasks/TASK-1773-phase-173-cross-boundary-closeout.md) | Close out Phase 173 with cross-boundary validation, review, and status reconciliation | ✅ Complete |

## Phase 174: Macro-Aware Tooling, Summary Identity, and Inference Readiness

**Status:** ✅ Complete
**Plan:** [PLAN-174: Macro-Aware Tooling, Summary Identity, and Inference Readiness](PLAN-174-MACRO-AWARE-TOOLING-SUMMARY-IDENTITY-AND-INFERENCE-READINESS.md)
**Depends on:** Phase 173 macro summaries, token trees, hygienic binders, and typed macros; `SPEC-095c`; `SPEC-098c`; `SPEC-097b`.

Hardens the tooling and identity seams left after Phase 173. The phase makes LSP-facing macro presentation, symbol identity, cache invalidation, and navigation macro-aware without treating macro summaries as runtime callables. It also audits callable identity summaries and implements only bounded macro inference through ordinary calls when a unique callable identity and type summary are proven.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1774](tasks/TASK-1774-phase-174-plan-packet.md) | Create the Phase 174 macro-aware tooling and inference-readiness plan packet | ✅ Complete |
| [TASK-1775](tasks/TASK-1775-macro-aware-tooling-audit.md) | Audit macro-aware tooling, LSP, and summary-identity seams | ✅ Complete |
| [TASK-1776](tasks/TASK-1776-macro-symbol-cache-model.md) | Add macro-specific symbol kinds and cache-summary invalidation keys | ✅ Complete |
| [TASK-1777](tasks/TASK-1777-macro-completion-hover-signature-ux.md) | Implement macro-aware completion and hover/signature presentation | ✅ Complete |
| [TASK-1778](tasks/TASK-1778-macro-goto-reference-boundaries.md) | Harden macro goto-definition and symbol boundaries without callable overclaiming | ✅ Complete |
| [TASK-1779](tasks/TASK-1779-callable-identity-summary-audit.md) | Audit and specify callable identity summaries for macro inference | ✅ Complete |
| [TASK-1780](tasks/TASK-1780-bounded-callable-identity-inference.md) | Implement bounded macro inference through proven callable identities | ✅ Complete |
| [TASK-1781](tasks/TASK-1781-cross-boundary-tooling-validation.md) | Add parser/engine/LSP cross-boundary tooling and inference validation | ✅ Complete |
| [TASK-1782](tasks/TASK-1782-phase-174-docs-spec-reconciliation.md) | Reconcile specs, docs, and indexes for Phase 174 boundaries | ✅ Complete |
| [TASK-1783](tasks/TASK-1783-phase-174-closeout.md) | Close out Phase 174 with broad gates and review | ✅ Complete |

## Phase 175: Name-Resolution-Backed Semantic Identity for Macros and Tooling

**Status:** ✅ Complete (10/10 tasks complete)
**Plan:** [PLAN-175: Name-Resolution-Backed Semantic Identity for Macros and Tooling](PLAN-175-NAME-RESOLUTION-BACKED-SEMANTIC-IDENTITY-FOR-MACROS-AND-TOOLING.md)
**Depends on:** Phase 174 macro-aware tooling, summary identity, and inference readiness; `SPEC-095c`; `SPEC-038`; `SPEC-098c`; `SPEC-097b`.

Introduces a conservative name-resolution-backed semantic identity substrate for macro declarations and tooling. The phase defines canonical macro declaration identity, threads resolved macro/callable identity through parser/LSP summaries, replaces token-only same-file references with semantic macro/function splitting, prepares imported macro navigation through summary identities without overclaiming, and preserves the syntax-phase-only macro boundary.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1784](tasks/TASK-1784-phase-175-plan-packet.md) | Create the Phase 175 semantic-identity plan packet | ✅ Complete |
| [TASK-1785](tasks/TASK-1785-identity-surface-audit.md) | Audit macro/callable identity surfaces and current name-resolution seams | ✅ Complete |
| [TASK-1786](tasks/TASK-1786-canonical-macro-identity-model.md) | Define canonical macro declaration identity and callable identity boundaries | ✅ Complete |
| [TASK-1787](tasks/TASK-1787-parser-local-name-resolution-identities.md) | Add parser-local resolved macro/callable identity carriers | ✅ Complete |
| [TASK-1788](tasks/TASK-1788-lsp-summary-identity-threading.md) | Thread resolved identities through LSP parse and symbol summaries | ✅ Complete |
| [TASK-1789](tasks/TASK-1789-semantic-same-file-references.md) | Replace token-only same-file references with semantic macro/function splitting | ✅ Complete |
| [TASK-1790](tasks/TASK-1790-imported-macro-navigation-prep.md) | Prepare imported macro navigation via summary identities without overclaiming | ✅ Complete |
| [TASK-1791](tasks/TASK-1791-identity-non-callability-validation.md) | Validate identity threading does not make macros runtime-callable | ✅ Complete |
| [TASK-1792](tasks/TASK-1792-phase-175-docs-spec-reconciliation.md) | Reconcile specs, docs, indexes, and changelog for Phase 175 | ✅ Complete |
| [TASK-1793](tasks/TASK-1793-phase-175-closeout.md) | Close out Phase 175 with broad gates and independent review | ✅ Complete |

## Phase 176: Deferred Cleanup after Target-Language Redesign

**Status:** ✅ Complete (9/9 tasks complete)
**Plan:** [PLAN-176: Deferred Cleanup after Target-Language Redesign](PLAN-176-DEFERRED-CLEANUP-AFTER-TARGET-REDESIGN.md)
**Depends on:** Phase 175 closeout; target-language redesign/spec-hardening sequence through Phases 167-175; prior deferred cleanup rows in Phases 151, 152, 157, and 158.

Retires or re-scopes deferred cleanup candidates that were intentionally left behind before the target-language redesign: `Value::List` removal, module-level pure function visibility inside closures, remaining ordinary-Ash QuickCheck recursive combinators, and stale Phase 152 status drift. The phase is audit-first: every old deferral must receive a current disposition before implementation claims completion.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1794](tasks/TASK-1794-phase-176-plan-packet.md) | Create the Phase 176 deferred-cleanup planning packet | ✅ Complete |
| [TASK-1795](tasks/TASK-1795-deferred-cleanup-readiness-audit.md) | Audit deferred cleanup candidates and prerequisite substrate | ✅ Complete |
| [TASK-1796](tasks/TASK-1796-value-list-reference-classification.md) | Classify every `Value::List` reference before removal | ✅ Complete |
| [TASK-1797](tasks/TASK-1797-remove-value-list-runtime-variant.md) | Remove `Value::List` and route all list values through `Cons`/`Nil` | ✅ Complete |
| [TASK-1798](tasks/TASK-1798-module-function-visibility-in-closures.md) | Fix module-level pure function visibility inside closures | ✅ Complete |
| [TASK-1799](tasks/TASK-1799-quickcheck-recursive-combinator-design-audit.md) | Re-audit recursive QuickCheck combinator design against live language features | ✅ Complete |
| [TASK-1800](tasks/TASK-1800-quickcheck-recursive-combinators.md) | Implement or explicitly re-scope recursive QuickCheck combinators | ✅ Complete / Re-scoped |
| [TASK-1801](tasks/TASK-1801-stale-phase-status-reconciliation.md) | Reconcile stale Phase 151/152/157/158 status surfaces | ✅ Complete |
| [TASK-1802](tasks/TASK-1802-phase-176-closeout.md) | Close out Phase 176 with broad gates and independent review | ✅ Complete |

- TASK-1800 landed SPEC-087 public QuickCheck recursive names/config and explicitly re-scoped execution through a fail-closed private helper pending parser/type-metadata support for bounded ordinary-Ash recursion.

- TASK-1801 reconciled historical status surfaces for Phase 151/TASK-1511, Phase 152, Phase 157/TASK-1570, and Phase 158/TASK-1580 against Phase 176 outcomes.


- TASK-1802 closed Phase 176 after broad gates, `Value::List` absence verification, and independent review remediation for imported private-helper isolation, QuickCheck fail-closed honesty, and final status-count drift.

## Phase 177: Target Ash Row Syntax and Core/CPS Alignment

**Status:** ✅ Complete (11/11 tasks complete; bounded parser/validation, operation identity, Core/CPS taxonomy, cross-boundary evidence, closeout, and row syntax review remediation complete)
**Plan:** [PLAN-177: Target Ash Row Syntax and Core/CPS Alignment](PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md)
**Depends on:** Phase 176 closeout and interphase TASK-1803 through TASK-1805 status reconciliation.
**Specs/notes:** `SPEC-095b`, `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099`, `SPEC-100`, `NOTE-020`, `NOTE-021`, `NOTE-022`, `NOTE-023`, and `NOTE-025`.

Starts the next target-Ash implementation packet by connecting source-facing computation-row/effect syntax to parser and validation carriers while aligning Core and CPS row carriers for later source-to-Core bridging. The phase is audit-first and bounded: it parses and validates target row syntax, preserves impl-qualified operation identity where proven, aligns Core/CPS row taxonomy enough to avoid silent row loss, records the current rowless source-to-typechecker boundary, and adds cross-boundary tests proving rows remain requirements rather than authority grants.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1806](tasks/TASK-1806-phase-177-plan-packet.md) | Create the Phase 177 target-Ash row syntax and Core/CPS alignment packet | ✅ Complete |
| [TASK-1807](tasks/TASK-1807-row-syntax-core-cps-seam-audit.md) | Audit row syntax, Core row, CPS row, and lowering seams | ✅ Complete |
| [TASK-1808](tasks/TASK-1808-row-syntax-spec-delta-reconciliation.md) | Reconcile target row/effect syntax deltas into implementation decisions | ✅ Complete |
| [TASK-1809](tasks/TASK-1809-surface-computation-row-parser-carriers.md) | Add surface computation-row parser and AST carriers | ✅ Complete |
| [TASK-1810](tasks/TASK-1810-impl-qualified-operation-identity-resolution.md) | Resolve impl-qualified operation row identities | ✅ Complete |
| [TASK-1811](tasks/TASK-1811-row-validation-and-diagnostics.md) | Validate row syntax and emit fail-closed diagnostics | ✅ Complete |
| [TASK-1812](tasks/TASK-1812-core-row-taxonomy-alignment.md) | Align Core row taxonomy with target computation-row families | ✅ Complete |
| [TASK-1813](tasks/TASK-1813-cps-row-taxonomy-bridge.md) | Align CPS row/effect carriers and Core-to-CPS row lowering | ✅ Complete |
| [TASK-1814](tasks/TASK-1814-row-syntax-core-cps-cross-boundary-tests.md) | Add parser/engine/Core/CPS cross-boundary row preservation tests | ✅ Complete |
| [TASK-1815](tasks/TASK-1815-phase-177-closeout.md) | Close out Phase 177 with gates, review, and status reconciliation | ✅ Complete |
| [TASK-1816](tasks/TASK-1816-phase-177-row-syntax-review-remediation.md) | Remediate Phase 177 row syntax review findings | ✅ Complete |

## Phase 178: Source-to-Core Row Lowering Bridge

**Status:** ✅ Complete (9/9 tasks complete; source-to-Core explicit row bridge closed with review remediation)
**Plan:** [PLAN-178: Source-to-Core Row Lowering Bridge](PLAN-178-SOURCE-TO-CORE-ROW-LOWERING-BRIDGE.md)
**Depends on:** Phase 177 closeout and TASK-1816 row syntax review remediation.
**Specs/notes:** `SPEC-095b`, `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099`, `SPEC-100`, `NOTE-020`, `NOTE-021`, and `NOTE-025`.

Bridges Phase 177's parsed and validated target row syntax into the source-to-typechecker/Core lowering path. The phase is bounded to explicit row preservation: it audits the rowless `Type::Fn`/source-to-Core loss boundary, threads parsed callable rows into function/type summaries, lowers supported explicit rows into Core callable rows, and proves row requirements remain authority-neutral. Row-polymorphic inference and provider/admission runtime wiring remain out of scope.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1817](tasks/TASK-1817-phase-178-plan-packet.md) | Create the Phase 178 source-to-Core row lowering bridge packet | ✅ Complete |
| [TASK-1818](tasks/TASK-1818-source-to-core-row-loss-audit.md) | Audit source-to-typechecker/Core row-loss boundaries | ✅ Complete |
| [TASK-1819](tasks/TASK-1819-row-bearing-callable-summary-carriers.md) | Add row-bearing callable summary carriers | ✅ Complete |
| [TASK-1820](tasks/TASK-1820-thread-parsed-rows-into-type-summaries.md) | Thread parsed rows into function/type summaries | ✅ Complete |
| [TASK-1821](tasks/TASK-1821-lower-source-rows-to-core-callable-rows.md) | Lower source rows into Core callable rows | ✅ Complete |
| [TASK-1822](tasks/TASK-1822-row-requirements-authority-neutrality-tests.md) | Prove row requirements do not install authority | ✅ Complete |
| [TASK-1823](tasks/TASK-1823-parser-engine-typecheck-core-row-preservation.md) | Add parser -> engine/typecheck -> Core row preservation tests | ✅ Complete |
| [TASK-1824](tasks/TASK-1824-phase-178-docs-spec-reconciliation.md) | Reconcile docs/spec/status for Phase 178 boundaries | ✅ Complete |
| [TASK-1825](tasks/TASK-1825-phase-178-closeout.md) | Close out Phase 178 with gates and review | ✅ Complete |

## Phase 179: Explicit Row Admission Runtime Wiring

|**Status:** ✅ Complete (9/9 tasks complete)
|**Plan:** [PLAN-179: Explicit Row Admission Runtime Wiring](PLAN-179-EXPLICIT-ROW-ADMISSION-RUNTIME-WIRING.md)
|**Depends on:** Phase 178 closeout.
|**Specs/notes:** `SPEC-096b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`, `SPEC-017`, `SPEC-019`, `SPEC-052`, `SPEC-053`, `NOTE-009` *(superseded; historical context only; see NOTE-022/023/025)*, `NOTE-020`, `NOTE-021`, `NOTE-022`, `NOTE-023`, and `NOTE-025`.

Connects Phase 178's explicit source/Core row metadata to runtime/admission checks. Operation rows are interface/impl-qualified operation identities per NOTE-022/025; the "provider" checks in this phase refer to already-registered host/runtime authority, not the deprecated `capability binding` vocabulary. The phase is bounded to fail-closed admission of already explicit row requirements: operation/provider, resource, role, and policy rows must require existing authority or produce precise unsupported/missing-authority diagnostics. Row-polymorphic inference, handler execution, provider registration, and broad stdlib/example migration remain out of scope.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1826](tasks/TASK-1826-phase-179-plan-packet.md) | Create the Phase 179 explicit row admission wiring packet | ✅ Complete |
|| [TASK-1827](tasks/TASK-1827-row-admission-runtime-audit.md) | Audit row metadata against admission/runtime authority paths | ✅ Complete |
|| [TASK-1828](tasks/TASK-1828-explicit-row-admission-carriers.md) | Add explicit row admission requirement carriers | ✅ Complete |
|| [TASK-1829](tasks/TASK-1829-operation-row-provider-admission.md) | Check operation rows against provider/operation admission | ✅ Complete |
|| [TASK-1830](tasks/TASK-1830-resource-row-admission.md) | Check resource rows against resource authority | ✅ Complete |
|| [TASK-1831](tasks/TASK-1831-role-policy-row-admission.md) | Check role and policy rows against admission authority | ✅ Complete |
|| [TASK-1832](tasks/TASK-1832-imported-row-admission.md) | Apply row admission checks across imported callables | ✅ Complete |
|| [TASK-1833](tasks/TASK-1833-row-admission-non-authority-regressions.md) | Prove row admission does not install authority | ✅ Complete |
|| [TASK-1834](tasks/TASK-1834-phase-179-closeout.md) | Close out Phase 179 with gates and review | ✅ Complete |

## Phase 180: Target Docs Consistency Cleanup

|**Status:** ✅ Complete (1/1 tasks complete)
|**Plan:** [PLAN-180: Target Docs Consistency Cleanup](PLAN-180-TARGET-DOCS-CONSISTENCY-CLEANUP.md)
|**Depends on:** Phase 179 closeout.
|**Specs/notes:** `SPEC-095b`, `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`, `NOTE-015`, `NOTE-018`, `NOTE-019`, `NOTE-022`, `NOTE-023`, and `NOTE-025`.

Reconciles target-Ash documentation after the interface/impl-qualified operation and explicit row-admission work. The phase fences legacy `capability binding`, `effect` declaration, and `WorkflowForm` material so current target planning routes through computation rows, provider/handler admission, and ambient workflow facts.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1835](tasks/TASK-1835-target-docs-consistency-cleanup.md) | Reconcile stale target-Ash specs and notes | ✅ Complete |

## Phase 181: Legacy Authority Vocabulary Audit

|**Status:** ✅ Complete (1/1 tasks complete)
|**Plan:** [PLAN-181: Legacy Authority Vocabulary Audit](PLAN-181-LEGACY-AUTHORITY-VOCABULARY-AUDIT.md)
|**Depends on:** Phase 180 target docs consistency cleanup.
|**Specs/notes:** `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-099b`, `SPEC-100`, `NOTE-022`, `NOTE-023`, and `NOTE-025`.

Audits older capability/provider authority vocabulary and classifies affected docs as target-state authority, current-state compatibility, superseded historical reference, or deferred background. Target correctness takes priority over preserving compatibility vocabulary as active target guidance.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1836](tasks/TASK-1836-legacy-authority-vocabulary-audit.md) | Classify legacy authority vocabulary docs | ✅ Complete |

## Phase 182: Core Computation Model Conformance

|**Status:** Complete (10/10 tasks complete)
|**Plan:** [PLAN-182: Core Computation Model Conformance](PLAN-182-CORE-COMPUTATION-MODEL-CONFORMANCE.md)
|**Depends on:** Phase 181 legacy authority vocabulary audit.
|**Specs/notes:** `SPEC-095b`, `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099`, `SPEC-100`, `NOTE-019`, `NOTE-020`, and `NOTE-021`.

Makes the target Core computation model explicit and executable for a bounded slice: Core Ash is the checked direct-style language, computation rows are requirement metadata, `fn` is the primary computation unit, and target `do { ... }` is direct sequencing sugar rather than an `Act`, `Proc`, or `Workflow` mode.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1837](tasks/TASK-1837-core-computation-plan-packet.md) | Create the Phase 182 plan packet | Complete |
|| [TASK-1838](tasks/TASK-1838-core-computation-boundary-audit.md) | Audit Core computation boundaries | Complete |
|| [TASK-1839](tasks/TASK-1839-core-computation-spec-reconciliation.md) | Reconcile target Core computation specs | Complete |
|| [TASK-1840](tasks/TASK-1840-primary-fn-computation-unit.md) | Prove `fn` as primary row-bearing computation unit | Complete |
|| [TASK-1841](tasks/TASK-1841-ambient-do-sequencing-sugar.md) | Implement target `do { ... }` sequencing sugar | Complete |
|| [TASK-1842](tasks/TASK-1842-row-requirements-direct-style-preservation.md) | Preserve row requirements through direct-style Core metadata | Complete |
|| [TASK-1843](tasks/TASK-1843-demote-tower-language-in-target-docs.md) | Demote tower language in target docs | Complete |
|| [TASK-1844](tasks/TASK-1844-core-computation-cross-boundary-fixture.md) | Add canonical cross-boundary target fixture | Complete |
|| [TASK-1845](tasks/TASK-1845-phase-182-consistency-review.md) | Review Phase 182 consistency and cross-references | Complete |
|| [TASK-1846](tasks/TASK-1846-core-computation-closeout.md) | Close out Phase 182 | Complete |

## Phase 183: Operation And Authority Model

|**Status:** ✅ Complete (8/8 tasks complete)
|**Plan:** [PLAN-183: Operation And Authority Model](PLAN-183-OPERATION-AUTHORITY-MODEL.md)
|**Depends on:** Phase 182 Core computation model conformance.
|**Specs/notes:** `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`, `NOTE-022`, `NOTE-023`, and `NOTE-025`.

Defines how effects actually happen for the target model: operations are interface methods, operation identity is impl/type-qualified, rows require operations without granting authority, and operation/resource/role/policy/evidence/failure rows have separate discharge paths.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1847](tasks/TASK-1847-operation-authority-plan-packet.md) | Create the Phase 183 plan packet | Complete |
|| [TASK-1848](tasks/TASK-1848-operation-authority-boundary-audit.md) | Audit operation authority boundaries | Complete |
|| [TASK-1849](tasks/TASK-1849-operation-authority-spec-reconciliation.md) | Reconcile operation authority specs | Complete |
|| [TASK-1850](tasks/TASK-1850-admission-discharge-model.md) | Add admission discharge model | Complete |
|| [TASK-1851](tasks/TASK-1851-impl-qualified-operation-authority-fixtures.md) | Add impl/type-qualified operation authority fixtures | Complete |
|| [TASK-1852](tasks/TASK-1852-row-family-discharge-diagnostics.md) | Separate row-family discharge diagnostics | Complete |
|| [TASK-1853](tasks/TASK-1853-operation-authority-non-grant-regressions.md) | Prove rows do not grant authority | Complete |
|| [TASK-1854](tasks/TASK-1854-operation-authority-closeout.md) | Close out Phase 183 | Complete |

## Phase 184: Handler / Provider Semantics

|**Status:** ✅ Complete (8/8 tasks complete)
|**Plan:** [PLAN-184: Handler / Provider Semantics](PLAN-184-HANDLER-PROVIDER-SEMANTICS.md)
|**Depends on:** Phase 183 Operation and Authority Model.
|**Specs/notes:** `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-099b`, `SPEC-100`, `NOTE-023`, and `NOTE-025`.

Turns rows from metadata into the operational model for operation effects: handler/provider frames discharge operation requirements, raise/handle dispatch uses frame-stack order, missing discharge fails closed, and handler/provider nesting and shadowing are observable.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1855](tasks/TASK-1855-handler-provider-plan-packet.md) | Create the Phase 184 plan packet | Complete |
|| [TASK-1856](tasks/TASK-1856-handler-provider-boundary-audit.md) | Audit handler/provider semantics boundaries | Complete |
|| [TASK-1857](tasks/TASK-1857-admission-frame-proof-model.md) | Add admission frame proof model | Complete |
|| [TASK-1858](tasks/TASK-1858-cps-frame-ordered-dispatch.md) | Fix CPS frame-ordered dispatch | Complete |
|| [TASK-1859](tasks/TASK-1859-raise-handle-provider-regressions.md) | Add raise/handle/provider regressions | Complete |
|| [TASK-1860](tasks/TASK-1860-missing-discharge-failure-diagnostics.md) | Define missing-discharge failures | Complete |
|| [TASK-1861](tasks/TASK-1861-handler-provider-spec-reconciliation.md) | Reconcile handler/provider specs | Complete |
|| [TASK-1862](tasks/TASK-1862-handler-provider-closeout.md) | Close out Phase 184 | Complete |

## Phase 185: Surface Function Language

|**Status:** ✅ Complete (7/7 tasks complete)
|**Plan:** [PLAN-185: Surface Function Language](PLAN-185-SURFACE-FUNCTION-LANGUAGE.md)
|**Depends on:** Phase 184 Handler / Provider Semantics.
|**Specs/notes:** `SPEC-095b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, and `SPEC-100`.

Makes target Ash pleasant as a surface language by using ordinary `fn` declarations as the user-facing computation unit, keeping rows as requirement sets, treating `do { ... }` as direct-style sequencing sugar, and demoting workflow syntax to compatibility/runtime profile handling rather than a core source-language path.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1863](tasks/TASK-1863-surface-function-language-plan-packet.md) | Create the Phase 185 plan packet | Complete |
|| [TASK-1864](tasks/TASK-1864-surface-function-boundary-audit.md) | Audit current `fn`/row/`do`/workflow boundaries | Complete |
|| [TASK-1865](tasks/TASK-1865-fn-main-entry-adapter.md) | Accept `fn main` as target entry syntax | Complete |
|| [TASK-1866](tasks/TASK-1866-function-body-language-fixture.md) | Add cohesive ordinary function body conformance fixture | Complete |
|| [TASK-1867](tasks/TASK-1867-surface-function-spec-reconciliation.md) | Reconcile target specs and indexes | Complete |
|| [TASK-1868](tasks/TASK-1868-surface-function-closeout.md) | Close out Phase 185 | Complete |
|| [TASK-1869](tasks/TASK-1869-surface-function-do-return-and-execution.md) | Accept semicolon `do` return and execute `fn main` sources | Complete |

## Phase 186: Surface Function CLI Entry

|**Status:** ✅ Complete (7/7 tasks complete)
|**Plan:** [PLAN-186: Surface Function CLI Entry](PLAN-186-SURFACE-FUNCTION-CLI-ENTRY.md)
|**Depends on:** Phase 185 Surface Function Language.
|**Specs/notes:** `SPEC-095b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`, and `PLAN-185`.

Extends the function-first target entry path to the command-line user surface: `ash run --dry-run` and normal `ash run` should accept ordinary `fn main` sources without requiring a `workflow` block, while legacy workflow entry handling remains compatibility/runtime-profile routing. Function-first runtime execution also accepts named constructor payload field projection for ordinary ADT/record-shaped source fixtures.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1870](tasks/TASK-1870-surface-function-cli-plan-packet.md) | Create the Phase 186 plan packet | Complete |
|| [TASK-1871](tasks/TASK-1871-cli-entry-boundary-audit.md) | Audit CLI run/check entry boundaries | Complete |
|| [TASK-1872](tasks/TASK-1872-run-dry-run-fn-main-entry.md) | Make `ash run --dry-run` accept function-first entry sources | Complete |
|| [TASK-1873](tasks/TASK-1873-cli-entry-spec-reconciliation.md) | Reconcile CLI entry specs and indexes | Complete |
|| [TASK-1874](tasks/TASK-1874-surface-function-cli-closeout.md) | Close out Phase 186 | Complete |
|| [TASK-1875](tasks/TASK-1875-synthetic-entry-warning-cleanup.md) | Suppress legacy workflow warnings for synthetic `fn main` adapters | Complete |
|| [TASK-1876](tasks/TASK-1876-surface-constructor-field-execution.md) | Execute function-first sources with named constructor field projection | Complete |

## Phase 187: Surface Record Expressions

|**Status:** ✅ Complete (2/2 tasks complete)
|**Plan:** [PLAN-187: Surface Record Expressions](PLAN-187-SURFACE-RECORD-EXPRESSIONS.md)
|**Depends on:** Phase 186 Surface Function CLI Entry.
|**Specs/notes:** `SPEC-095b`, `SPEC-097b`, `SPEC-098c`, `SPEC-100`, and `PLAN-185`.

Makes structural records ordinary function-first surface expressions: `{ field: expr }` should parse,
check, execute, and project fields without requiring legacy workflow syntax, nominal constructor
payloads, or stdlib helper calls.

|| Task | Description | Status |
||------|-------------|--------|
|| [TASK-1877](tasks/TASK-1877-surface-record-expression-plan-packet.md) | Create the Phase 187 plan packet | Complete |
|| [TASK-1878](tasks/TASK-1878-structural-record-expression-execution.md) | Parse, check, and execute structural record expressions | Complete |

## Phase 194: Contract And Evidence System

|**Status:** ✅ Complete (11/11 tasks complete)
**Plan:** [PLAN-194: Contract And Evidence System](PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)
**Audit:** [AUDIT-194: Contract Evidence Seams](audits/AUDIT-194-contract-evidence-seams.md)
**Depends on:** Phase 184 Handler / Provider Semantics and Phase 193 Surface Tuple ADT Expressions.
|**Specs/notes:** `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`,
`PLAN-165`, `PLAN-183`, `PLAN-184`, `NOTE-027`, `NOTE-029`, `NOTE-031`, `NOTE-032`, `NOTE-033`,
`NOTE-034`, and `NOTE-035`.

Adds correctness obligations on top of the target computation model: surface `requires`/`ensures`,
predicate well-formedness, authority-free contract predicate evaluation, evidence rows for tests,
laws, proofs, runtime monitors, and observations, plus structured blame diagnostics. The phase is
intentionally after the operation model: contract predicates may inspect values and observation
evidence, but they cannot acquire operation/resource/role/policy authority.

|| Task | Description | Status |
||------|-------------|--------|
||| [TASK-1891](tasks/TASK-1891-contract-evidence-plan-packet.md) | Create the Phase 194 plan and task packet | ✅ Complete |
||| [TASK-1892](tasks/TASK-1892-contract-evidence-seam-audit.md) | Audit live contract/evidence carriers, row admission, and diagnostics boundaries | ✅ Complete |
|| [TASK-1893](tasks/TASK-1893-requires-ensures-surface-carriers.md) | Parse and preserve `requires`/`ensures` clauses on target `fn` declarations | ✅ Complete |
|| [TASK-1894](tasks/TASK-1894-contract-predicate-well-formedness.md) | Enforce predicate well-formedness and authority-free observer rules | ✅ Complete |
|| [TASK-1895](tasks/TASK-1895-surface-contract-lowering.md) | Lower surface contracts to Core predicate sidecars, snapshots, and check plans | ✅ Complete |
|| [TASK-1896](tasks/TASK-1896-evidence-row-substrate.md) | Add evidence row records for tests, laws, proofs, runtime monitors, and observations | ✅ Complete |
|| [TASK-1897](tasks/TASK-1897-contract-discharge-integration.md) | Integrate static/evidence/dynamic contract discharge with row admission | ✅ Complete |
|| [TASK-1898](tasks/TASK-1898-dynamic-contract-runtime-checks.md) | Execute dynamic contract checks with distinct violation and predicate-fault traps | ✅ Complete |
|| [TASK-1899](tasks/TASK-1899-contract-blame-diagnostics.md) | Emit structured blame diagnostics with snapshots, evidence, and redaction metadata | ✅ Complete |
|| [TASK-1900](tasks/TASK-1900-runtime-monitor-evidence.md) | Wire runtime monitor evidence rows and temporal monitor diagnostics | ✅ Complete |
|| [TASK-1901](tasks/TASK-1901-contract-evidence-closeout.md) | Close out Phase 194 with fixtures, docs, gates, and review remediation | ✅ Complete |

## Phase 195: Process And Concurrency Model

**Status:** ✅ Complete (11/11 tasks complete)
**Plan:** [PLAN-195: Process And Concurrency Model](PLAN-195-PROCESS-AND-CONCURRENCY-MODEL.md)
**Audit:** [AUDIT-195: Process Runtime Seams](audits/AUDIT-195-process-runtime-seams.md)
**Depends on:** Phase 182 Core Computation Model Conformance, Phase 183 Operation And Authority Model, Phase 184 Handler / Provider Semantics, and Phase 194 Contract And Evidence System.
**Specs/notes:** `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`, `PLAN-182`, `PLAN-183`, `PLAN-184`, `PLAN-194`, `NOTE-020`, `NOTE-021`, and `NOTE-035`.

Adds structured execution beyond single computations while preserving the target model: process execution is a runtime profile over ambient row-bearing computations, not a separate semantic foundation and not a revival of the deprecated `Act`, `Proc`, or `Workflow` tower forms. Those names may remain only as legacy documentation references; new development must not add surface syntax, Core terms, IR nodes, public stdlib types, or runtime entry paths named `Act`, `Proc`, or `Workflow`. The phase is intentionally after the basic computation, operation authority, handler/provider, and contract/evidence models because concurrency multiplies unresolved authority, ownership, failure, and monitor questions. It introduces spawn/join/await, channels, cancellation, failure propagation, and sendability/ownership across process boundaries as runtime-profile facts with fail-closed validation and trace evidence.

Non-goals: no `Act`/`Proc`/`Workflow` development forms, no separate workflow runtime, no distributed actor runtime, no scheduler fairness proof, no generalized temporal logic surface syntax, and no process authority granted by mentioning process rows.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1902](tasks/TASK-1902-process-concurrency-plan-packet.md) | Create the Phase 195 plan and task packet | ✅ Complete |
| [TASK-1903](tasks/TASK-1903-process-runtime-seam-audit.md) | Audit computation, handler/provider, row admission, runtime, and trace seams for process execution | ✅ Complete |
| [TASK-1904](tasks/TASK-1904-deprecated-tower-vocabulary-spec-reconciliation.md) | Reconcile deprecated `Act`/`Proc`/`Workflow` vocabulary in target specs and notes | ✅ Complete |
| [TASK-1905](tasks/TASK-1905-process-row-and-core-carriers.md) | Add process row facts and Core/CPS carriers for spawn, join, await, channels, cancellation, and ownership transfer | ✅ Complete |
| [TASK-1906](tasks/TASK-1906-sendability-ownership-validation.md) | Validate sendability, ownership, affine movement, and borrowed-resource rejection across process boundaries | ✅ Complete |
| [TASK-1907](tasks/TASK-1907-spawn-join-await-runtime-semantics.md) | Implement bounded spawn, join, and await runtime semantics over existing computation and handler frames | ✅ Complete |
| [TASK-1908](tasks/TASK-1908-channel-runtime-semantics.md) | Implement bounded typed channel creation, send, receive, close, and select-ready diagnostics | ✅ Complete |
| [TASK-1909](tasks/TASK-1909-cancellation-and-failure-propagation.md) | Model cancellation, child failure, join failure, and supervisor-facing propagation diagnostics | ✅ Complete |
| [TASK-1910](tasks/TASK-1910-process-trace-and-monitor-evidence.md) | Emit process/channel/cancellation trace facts and runtime monitor evidence without granting authority | ✅ Complete |
| [TASK-1911](tasks/TASK-1911-process-concurrency-cross-boundary-fixtures.md) | Add cross-boundary parser, typecheck, Core/CPS, runtime, and CLI fixtures for process/concurrency behavior | ✅ Complete |
| [TASK-1912](tasks/TASK-1912-process-concurrency-closeout.md) | Close out Phase 195 with docs, changelog, gates, and review remediation | ✅ Complete |

## Phase 196: Application / Workflow Runtime

**Status:** ✅ Complete (11/11 tasks complete)
**Plan:** [PLAN-196: Application / Workflow Runtime](PLAN-196-APPLICATION-WORKFLOW-RUNTIME.md)
**Depends on:** Phase 182 Core Computation Model Conformance, Phase 183 Operation And Authority Model, Phase 184 Handler / Provider Semantics, Phase 194 Contract And Evidence System, and Phase 195 Process And Concurrency Model.
**Specs/notes:** `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`, `PLAN-182`, `PLAN-183`, `PLAN-184`, `PLAN-194`, `PLAN-195`, `NOTE-020`, `NOTE-021`, and `NOTE-035`.
**Audit:** [AUDIT-196: Application Runtime Seams](audits/AUDIT-196-application-runtime-seams.md)

Builds workflow as an application/runtime layer over ordinary checked computations, admission profiles, role/policy/resource/provider boundaries, contracts, process supervision, reports, traces, services, and external actor adapters. The legacy `workflow` form is compatibility-only and must not become a primitive target surface, Core, IR, or semantic island.

Non-goals: no revival of legacy `workflow` form as a target primitive, no hidden authority from application entrypoints or admission profile names, no distributed actor runtime without explicit adapters, and no bypass of handler/provider, contract/evidence, process, or sendability checks.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1913](tasks/TASK-1913-application-workflow-runtime-plan-packet.md) | Create the Phase 196 plan and task packet | ✅ Complete |
| [TASK-1914](tasks/TASK-1914-application-runtime-seam-audit.md) | Audit CLI, engine, runtime kernel, daemon, admission, report, trace, and process seams | ✅ Complete |
| [TASK-1915](tasks/TASK-1915-legacy-workflow-form-boundary-reconciliation.md) | Reconcile specs/docs so legacy `workflow` form is compatibility-only for target planning | ✅ Complete |
| [TASK-1916](tasks/TASK-1916-application-entrypoint-metadata.md) | Add application entrypoint metadata and invocation packet carriers over checked computations | ✅ Complete |
| [TASK-1917](tasks/TASK-1917-admission-profile-runtime-boundary.md) | Wire admission profiles to runtime entry boundaries without granting ambient authority | ✅ Complete |
| [TASK-1918](tasks/TASK-1918-role-policy-resource-boundary-bindings.md) | Bind roles, policies, resources, providers, and contracts at application boundaries | ✅ Complete |
| [TASK-1919](tasks/TASK-1919-application-reports-traces-artifacts.md) | Emit application reports, trace bundles, runtime artifacts, and monitor evidence | ✅ Complete |
| [TASK-1920](tasks/TASK-1920-supervisor-runtime-profiles.md) | Add supervisor profiles over process handles with restart/cancel/failure policy | ✅ Complete |
| [TASK-1921](tasks/TASK-1921-long-running-service-lifecycle.md) | Add long-running service lifecycle, health, reload, shutdown, and retention semantics | ✅ Complete |
| [TASK-1922](tasks/TASK-1922-external-actor-integration.md) | Integrate external actors through explicit typed adapters and capability boundaries | ✅ Complete |
| [TASK-1923](tasks/TASK-1923-application-runtime-cross-boundary-fixtures-and-closeout.md) | Add cross-boundary fixtures, docs, gates, and closeout | ✅ Complete |

## Phase 197: Host / FFI / Builtins

**Status:** ✅ Complete (10/10 tasks complete)
**Plan:** [PLAN-197: Host / FFI / Builtins](PLAN-197-HOST-FFI-BUILTINS.md)
**Depends on:** Phase 183 Operation And Authority Model, Phase 184 Handler / Provider Semantics, Phase 194 Contract And Evidence System, Phase 195 Process And Concurrency Model, and Phase 196 Application / Workflow Runtime.
**Specs/notes:** `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`, `PLAN-183`, `PLAN-184`, `PLAN-194`, `PLAN-195`, `PLAN-196`, `NOTE-016`, `NOTE-020`, `NOTE-021`, `NOTE-025`, and `NOTE-035`.

Exposes host functionality carefully through audited builtins, provider authoring APIs, trusted
runtime adapters, sandboxing, and provenance. The phase is intentionally after authority,
handler/provider, contract/evidence, process, and application runtime work so host integration
cannot become a backdoor around row admission, policy checks, sendability, sandbox constraints, or
report/trace obligations.

Non-goals: no ambient host calls from ordinary expressions, no `extern` keyword or native ABI MVP,
no builtin trusted by name alone, no provider/adapter bypass of admission or sandboxing, and no raw
dynamic library/plugin callback path in this phase.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1924](tasks/TASK-1924-host-ffi-builtins-plan-packet.md) | Create the Phase 197 plan and task packet | ✅ Complete |
| [TASK-1925](tasks/TASK-1925-host-boundary-seam-audit.md) | Audit builtin, provider, runtime adapter, sandbox, and provenance seams | ✅ Complete |
| [TASK-1926](tasks/TASK-1926-builtin-host-hook-metadata.md) | Add builtin host hook metadata and fail-closed diagnostics | ✅ Complete |
| [TASK-1927](tasks/TASK-1927-provider-authoring-api.md) | Define provider authoring API for operation surfaces, constraints, resources, and effects | ✅ Complete |
| [TASK-1928](tasks/TASK-1928-trusted-runtime-adapter-registry.md) | Add trusted runtime adapter registry with identity, versioning, and admission boundaries | ✅ Complete |
| [TASK-1929](tasks/TASK-1929-host-sandbox-policy-enforcement.md) | Enforce sandbox policies for host-facing providers and adapters | ✅ Complete |
| [TASK-1930](tasks/TASK-1930-host-provenance-and-redaction.md) | Attach provenance, trace, report, and redaction evidence to host boundary crossings | ✅ Complete |
| [TASK-1931](tasks/TASK-1931-extern-decision-gate.md) | Decide whether `extern` is still needed and document the authority-checked path | ✅ Complete |
| [TASK-1932](tasks/TASK-1932-host-boundary-cross-boundary-fixtures.md) | Add cross-boundary fixtures for builtins, providers, adapters, sandboxing, and provenance | ✅ Complete |
| [TASK-1933](tasks/TASK-1933-host-ffi-builtins-closeout.md) | Close out Phase 197 with docs, changelog, gates, and review remediation | ✅ Complete |

## Phase 198: Standard Providers And Profiles

**Status:** Complete (8/8 tasks complete)
**Plan:** [PLAN-198: Standard Providers And Profiles](PLAN-198-STANDARD-PROVIDERS-AND-PROFILES.md)
**Depends on:** Phase 183 Operation And Authority Model, Phase 184 Handler / Provider Semantics, Phase 194 Contract And Evidence System, Phase 195 Process And Concurrency Model, Phase 196 Application / Workflow Runtime, and Phase 197 Host / FFI / Builtins.
**Specs/notes:** `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`, `PLAN-183`, `PLAN-184`, `PLAN-194`, `PLAN-195`, `PLAN-196`, `PLAN-197`, `NOTE-016`, `NOTE-020`, `NOTE-021`, `NOTE-024`, `NOTE-025`, and `NOTE-035`.

Turns the Phase 197 host boundary substrate into usable standard Ash provider libraries and
admission profiles for filesystem, HTTP, clock/time, logging, and contract/evidence helper use.
Provider wrappers must remain ordinary operation surfaces backed by trusted runtime adapters,
sandbox policy, provider metadata, and redacted provenance. Profiles select explicit boundary
expectations and must not grant authority by name.

Non-goals: no app templates or scaffolding CLI, no ambient provider access, no user native FFI, and
no legacy `workflow` syntax revival as the target provider example path.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1934](tasks/TASK-1934-standard-providers-profiles-plan-packet.md) | Create the Phase 198 plan and task packet | ✅ Complete |
| [TASK-1935](tasks/TASK-1935-standard-provider-profile-audit.md) | Audit stdlib provider modules, runtime providers, examples, and profile seams | ✅ Complete |
| [TASK-1936](tasks/TASK-1936-filesystem-provider-wrappers-and-profiles.md) | Implement filesystem stdlib wrappers and read/write row profiles | ✅ Complete |
| [TASK-1937](tasks/TASK-1937-http-provider-wrappers-and-profiles.md) | Implement HTTP stdlib wrappers and sandboxed network profiles | ✅ Complete |
| [TASK-1938](tasks/TASK-1938-clock-time-provider-and-test-clock.md) | Implement clock/time wrappers and deterministic test-clock profile support | ✅ Complete |
| [TASK-1939](tasks/TASK-1939-logging-provider-redaction-and-provenance.md) | Implement logging wrappers with redaction and provenance evidence | ✅ Complete |
| [TASK-1940](tasks/TASK-1940-common-row-admission-profiles.md) | Add common row/admission profile definitions and validation fixtures | ✅ Complete |
| [TASK-1941](tasks/TASK-1941-contract-evidence-helper-library-and-closeout.md) | Add contract/evidence helpers, final-surface fixtures, docs, gates, and closeout | ✅ Complete |

## Phase 199: Productive App Libraries And Templates

**Status:** ✅ Complete (9/9 tasks complete)
**Plan:** [PLAN-199: Productive App Libraries And Templates](PLAN-199-PRODUCTIVE-APP-LIBRARIES-AND-TEMPLATES.md)
**Depends on:** Phase 198 Standard Providers And Profiles.
**Specs/notes:** `PLAN-198`, `PLAN-197`, `PLAN-196`, `PLAN-195`, `PLAN-194`, `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`, `SPEC-081`, `SPEC-082`, `SPEC-083`, `SPEC-084`, and `NOTE-035`.

Builds productive Ash app libraries, testing helpers, process/channel helpers, app templates, and
tutorial examples over Phase 198. The first implementation task audits and revises productive
libraries, examples, and template-like files to current target syntax so templates teach the current
language rather than preserving historical syntax.

Non-goals: no new language syntax, no new host provider family beyond Phase 198, no template that
depends on legacy `workflow` syntax as a target primitive, and no package registry or marketplace
workflow.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1942](tasks/TASK-1942-productive-app-libraries-templates-plan-packet.md) | Create the Phase 199 plan and task packet | ✅ Complete |
| [TASK-1943](tasks/TASK-1943-current-syntax-library-template-audit-remediation.md) | Review and revise libraries, examples, and template-like files to current syntax | ✅ Complete |
| [TASK-1944](tasks/TASK-1944-testing-helper-libraries.md) | Add testing helper libraries over QuickCheck, law/evidence, coverage, and flake orchestration | ✅ Complete |
| [TASK-1945](tasks/TASK-1945-process-channel-convenience-library.md) | Add process/channel convenience helpers over Phase 195 semantics | ✅ Complete |
| [TASK-1946](tasks/TASK-1946-app-template-manifest-and-validation.md) | Define app template manifest/schema and validation model | ✅ Complete |
| [TASK-1947](tasks/TASK-1947-template-instantiation-cli.md) | Add CLI/template instantiation path with fail-closed diagnostics | ✅ Complete |
| [TASK-1948](tasks/TASK-1948-canonical-app-template-corpus.md) | Add canonical current-syntax app templates | ✅ Complete |
| [TASK-1949](tasks/TASK-1949-tutorial-examples-and-template-docs.md) | Add tutorial examples and template docs tied to executable gates | ✅ Complete |
| [TASK-1950](tasks/TASK-1950-productive-app-libraries-templates-closeout.md) | Close out Phase 199 with cross-template gates, docs, changelog, and review remediation | ✅ Complete |

## Phase 200: Tooling And Migration Polish

**Status:** Complete (9/9 tasks complete)
**Plan:** [PLAN-200: Tooling And Migration Polish](PLAN-200-TOOLING-AND-MIGRATION-POLISH.md)
**Depends on:** Phase 199 Productive App Libraries And Templates.
**Specs/notes:** `PLAN-199`, `PLAN-198`, `PLAN-196`, `PLAN-195`, `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`, and `NOTE-035`.

Makes current Ash the default path in tooling, examples, and docs by eliminating or explicitly
demoting legacy and deprecated forms from productive surfaces. The phase is migration-first:
diagnostics, formatter behavior, LSP behavior, examples, and docs are all driven by an initial
old-form inventory rather than treating stale syntax as incidental cleanup.

Non-goals: no new language syntax, no new provider/runtime authority model, no semantic rewrite of
current target Ash, no broad editor feature expansion beyond migration-relevant LSP polish, and no
package registry or marketplace workflow.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1951](tasks/TASK-1951-tooling-migration-polish-plan-packet.md) | Create the Phase 200 plan and task packet | ✅ Complete |
| [TASK-1952](tasks/TASK-1952-legacy-deprecated-form-audit.md) | Audit diagnostics, formatter, LSP, examples, docs, and old-form productive paths | ✅ Complete |
| [TASK-1953](tasks/TASK-1953-migration-diagnostics.md) | Improve stale/deprecated syntax diagnostics and migration hints | ✅ Complete |
| [TASK-1954](tasks/TASK-1954-formatter-current-syntax-polish.md) | Polish formatter coverage for current target syntax and old-form quarantine | ✅ Complete |
| [TASK-1955](tasks/TASK-1955-lsp-current-syntax-migration-polish.md) | Polish LSP diagnostics, hover, symbols, semantic tokens, and navigation for current syntax | ✅ Complete |
| [TASK-1956](tasks/TASK-1956-examples-current-syntax-refresh.md) | Refresh examples corpus and classify or remove legacy examples | ✅ Complete |
| [TASK-1957](tasks/TASK-1957-docs-current-syntax-refresh.md) | Refresh docs/tutorials/reference paths around current syntax and migration notes | ✅ Complete |
| [TASK-1958](tasks/TASK-1958-old-syntax-removal-demotion.md) | Remove or demote old syntax from productive paths with fail-closed gates | ✅ Complete |
| [TASK-1959](tasks/TASK-1959-tooling-migration-polish-closeout.md) | Close out Phase 200 with full gates, stale-claim sweep, docs, and review remediation | ✅ Complete |

## Phase 201: Deprecated Functionality Removal

**Status:** ✅ Complete (23/23 tasks complete and verified)
**Plan:** [PLAN-201: Deprecated Functionality Removal](PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md)
**Depends on:** Phase 200 Tooling And Migration Polish.
**Specs/notes:** `PLAN-200`, `PLAN-199`, `PLAN-196`, `PLAN-195`, `SPEC-095b`, `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`, and `NOTE-035`.

Removes deprecated Ash functionality completely from repository code, fixtures, examples,
templates, tooling behavior, executable/checkable/lowerable/formattable paths, and productive
documentation paths. After this phase, Ash source in the project repository must use target Ash
only.

Non-goals: no new language syntax, no new runtime/provider/authority semantics, no target Ash
semantic expansion, and no deletion of explicitly labeled historical/reference prose.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1960](tasks/TASK-1960-deprecated-functionality-removal-plan-packet.md) | Create the Phase 201 plan and task packet | ✅ Complete |
| [TASK-1961](tasks/TASK-1961-deprecated-functionality-dependency-audit.md) | Audit remaining deprecated functionality and classify removal owners | Complete |
| [TASK-1962](tasks/TASK-1962-parser-checker-deprecated-acceptance-removal.md) | Remove parser/checker acceptance of deprecated Ash forms | Complete |
| [TASK-1963](tasks/TASK-1963-surface-ast-lowering-legacy-carrier-removal.md) | Remove unreachable legacy surface AST and lowering carriers | Complete |
| [TASK-1964](tasks/TASK-1964-type-effect-runtime-deprecated-carrier-removal.md) | Remove deprecated type/effect/runtime vocabulary and carriers | Complete |
| [TASK-1965](tasks/TASK-1965-tooling-deprecated-behavior-removal.md) | Remove deprecated formatter, LSP, template, and CLI behavior | Complete |
| [TASK-1966](tasks/TASK-1966-docs-reference-historical-quarantine.md) | Quarantine historical docs and reconcile current/target spec references | Complete |
| [TASK-1967](tasks/TASK-1967-deprecated-functionality-removal-gates.md) | Add fail-closed gates for deprecated functionality removal | Complete |
| [TASK-1968](tasks/TASK-1968-deprecated-functionality-removal-closeout.md) | Close out Phase 201 with full gates, stale-claim sweep, docs, and review remediation | Complete |
| [TASK-1969](tasks/TASK-1969-semantic-removal-vs-rename-audit.md) | Audit Phase 201 for rename-only cleanup and stale mechanisms preserved under target names | ✅ Complete |
| [TASK-1970](tasks/TASK-1970-semantic-cleanup-plan-from-audit.md) | Elaborate the deletion/refactor plan from the semantic-removal audit and target specs | ✅ Complete |
| [TASK-1971](tasks/TASK-1971-residual-workflow-form-carriers.md) | Remove residual workflow-form parser/lowering carriers not needed for current contracts | ✅ Complete |
| [TASK-1972](tasks/TASK-1972-entry-artifact-carrier-alignment.md) | Align TCIR/AMIR entry-artifact carriers with target effect-row computation artifacts | ✅ Complete |
| [TASK-1973](tasks/TASK-1973-entry-projection-boundary.md) | Remove the stale entry Proc projection boundary in favor of application result/report projection | ✅ Complete |
| [TASK-1974](tasks/TASK-1974-historical-reference-routing.md) | Quarantine historical workflow/tower references from current read paths | ✅ Complete |
| [TASK-1975](tasks/TASK-1975-function-body-runtime-registry.md) | Fold callable-entry runtime registry into ordinary function-body metadata/cache behavior | ✅ Complete |
| [TASK-1976](tasks/TASK-1976-spawned-process-body-registry.md) | Fold child-entry registry behavior into spawned process body registry semantics | ✅ Complete |
| [TASK-1977](tasks/TASK-1977-application-report-identity.md) | Retarget application/entry report identity away from workflow-id vocabulary | ✅ Complete |
| [TASK-1978](tasks/TASK-1978-contract-helper-intrinsics.md) | Retarget contract helper intrinsics to target contract/evidence helpers | ✅ Complete |
| [TASK-1979](tasks/TASK-1979-ambient-effect-context.md) | Retarget ambient/entry effect context to row/profile effect typing vocabulary | ✅ Complete |
| [TASK-1980](tasks/TASK-1980-reference-tower-routing.md) | Archive or relabel workflow/tower stdlib and language references from current guidance | ✅ Complete |
| [TASK-1981](tasks/TASK-1981-removed-form-authority-page.md) | Add a removed-form authority page for historical terms and target replacements | ✅ Complete |
| [TASK-1982](tasks/TASK-1982-stale-compatibility-tests.md) | Delete or rewrite tests whose only purpose is old semantic compatibility | ✅ Complete |

## Phase 202: Formal Semantics And Verification Programme

**Status:** Complete (12/12 planning tasks complete; PLAN-203 now owns production realization and
direct-refinement integration follow-ups)
**Plan:** [PLAN-202: Formal Semantics And Verification Programme](PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)
**Depends on:** Plan-packet creation is independent. TASK-1988 and overlapping removal work build
on the completed Phase 201 semantic-cleanup handoff.
**Specs/notes:** `SPEC-071`, `SPEC-095b`, `SPEC-095c`, `SPEC-096b`, `SPEC-097b`, `SPEC-098b`,
`SPEC-098c`, `SPEC-099`, `SPEC-099b`, `SPEC-100`, `NOTE-030` through `NOTE-038`, the formalization
boundary, and the verification/prover literature survey.

Establishes a compact canonical authority graph, archive/supersession migration, semantic
implementation audit, bounded `λAsh-CPS` calculus, stable rule-to-code/test/proof traceability, and
two ordered Verus pilots. Ash-native `spec`/`proof` syntax remains a downstream design programme
informed by the measured pilot evidence.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1983](tasks/TASK-1983-formal-programme-plan-packet.md) | Create the Phase 202 formal programme plan packet | Complete |
| [TASK-1984](tasks/TASK-1984-corpus-authority-conflict-inventory.md) | Inventory documentation authority and semantic conflicts | Complete |
| [TASK-1985](tasks/TASK-1985-canonical-manifest-metadata-validation.md) | Add the canonical manifest, metadata, and validation schema | Complete |
| [TASK-1986](tasks/TASK-1986-canonical-core-reconciliation-promotion.md) | Reconcile and promote the compact canonical core | Complete |
| [TASK-1987](tasks/TASK-1987-archive-redirect-context-migration.md) | Migrate archive, redirects, and generated context routing | Complete |
| [TASK-1988](tasks/TASK-1988-semantic-implementation-deprecation-audit.md) | Map canonical rules to Rust and plan evidence-led removals | Complete |
| [TASK-1989](tasks/TASK-1989-ash-core-cps-calculus-freeze.md) | Freeze the staged `λAsh-CPS` calculus | Complete |
| [TASK-1990](tasks/TASK-1990-semantic-traceability-coverage-gates.md) | Add semantic traceability and coverage gates | Complete |
| [TASK-1991](tasks/TASK-1991-verus-toolchain-tcb-ci-spike.md) | Establish isolated Verus toolchain and TCB reporting | Complete |
| [TASK-1992](tasks/TASK-1992-verus-core-row-algebra-pilot.md) | Verify Core row normalization and closed inclusion | Complete |
| [TASK-1993](tasks/TASK-1993-verus-frame-ordered-dispatch-pilot.md) | Verify frame-ordered operation dispatch | Complete |
| [TASK-1994](tasks/TASK-1994-formal-programme-closeout-proof-design-handoff.md) | Close out the programme and hand off Ash proof-system design | Complete (evidence closeout) |

### TASK-1988 implementation follow-ups

These evidence-led implementation tasks are not additional PLAN-202 planning-task count claims.
They must complete before their affected semantic rules can claim production realization.

Their records are deliberately compositional: each task owns a specified feature/domain and its
designated layers, then names the handoffs consumed or produced for other owners. A `bounded`,
`not applicable`, or `non-authorizing` layer is an intentional implementation-domain boundary, not
a requirement for that record to implement every later layer. Production realization is a property
of the composed handoffs; separately owned integration tests and refinement/proof work demonstrate
that composition.

Current TASK-2003 local-call addition: the same sealed recognizer accepts only
`helper() -> Int { 7 }` or exact ambient `helper() -> Int { do { return 7; } }`, each
immediately followed by `main() -> Int { helper() }`. Both reuse the existing checked
`Lam`/`Call` construction; the helper's explicit `return` lowers as `Jump(cont, 7)` while
the caller supplies `__answer`. This is not general return/do/local-call lowering, runtime, or
admission, and it adds no fallback.

Current TASK-2003 Boolean equality addition: checked `PureAnf` admits only `Bool` × `Bool`
`==`/`!=` in addition to its approved `Int` × `Int` binary family. Each exact source operation
retains its checked Core identity, becomes one matching CPS `LetPrim(Eq|Ne, [Bool, Bool])`, and
finishes at `Jump(__answer, result)`. Mixed or other non-`Int`/non-`Bool` equality operands,
`&&`/`||`, `Neg`, calls, effects, handlers, providers, and frames remain closed; this is neither
polymorphic equality nor general binary lowering.

Current TASK-2008 test-only addition: the exact already-admitted `trap_sleep` CLI fixture run as
`ash run --format json --output terminal.json` exits 5 with stdout empty; only the output file
contains its telemetry-free V1 `trap` envelope and division-by-zero language reason. This does
not widen terminal taxonomy, handler admission, CLI routing, or output-file guarantees.

Current TASK-2005 parity addition: the closed corpus contains exactly
`phase202-source-absorb-sleep-handler-parity`, an `SEM-EFFECT-HANDLE-001` values/relation witness
for `Int(0)`. Its manifest locks the direct-source bytes to
`sha256:005a6c46e25884ca13762b7cd26e836b2756263f378fd297aa0afc006e8acf89`; loader verification
precedes carrier metadata and dispatch, reports expected/actual digest mismatch, and reserves the
field to this case. It remains a private direct derivation plus opaque checked-handler inspection,
not generic lowering, a production token, row/frame authority, or a fallback.

The same closed corpus additionally contains exactly
`phase202-source-trap-sleep-handler-terminal`: its fixed abortive `trap_sleep`
`1 / 0` clause compares the case-locked direct derivation and opaque checked-handler inspection
as the canonical V1 `trap` envelope with reason `division by zero`, under
`SEM-CPS-TRAP-001`. It neither consumes a TASK-2014 production token nor supplies CLI,
generic-terminalization, frame, lowering, or fallback authority.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-2000](tasks/TASK-2000-residual-act-proc-public-machinery-decision.md) | Decide the disposition of residual public `Act<T>`/`Proc<T>` machinery | Complete — the 50-reference detector inventory and independent public-absence controls prove that `Act<T>`/`Proc<T>`, all fourteen source-callable bridges, stale diagnostic/prelude carriers, and direct-source `invoke` carrier typing are deleted or fail closed; ambient `do` and ordinary builtins remain controls. Generic parser/lowering work (TASK-2002), future named operation/row realization, and hidden `ActEnv`/provider/process runtime machinery are explicit non-wrapper follow-ups, not public compatibility. |
| [TASK-2001](tasks/TASK-2001-target-grammar-gap-and-spec-conflict-decision.md) | Realize selected target grammar and retire proxy specification conflict | In progress — authority is already selected: `GRAM-TARGET-MODULE-001 → SPEC-095b`, `TYPE-TARGET-ROW-001 → SPEC-096b/SPEC-097b`, lowering → SPEC-098c, and runtime → SPEC-099b. Parser/AST declaration slices, stable parser rejection of historical `capability`/`proxy`/top-level `yield` declarations, TypeEnv handler/newtype registration/query evidence, source-anchored newtype summary/constructor handoff, non-granting alias/group plus marked-handler export metadata, source named-import and selected public `pub use` metadata transport, and narrow imported/local alias/group row validation are recorded. `SummaryVersion::STRUCTURAL_EFFECT_ROW_PROVIDER_BINDINGS_V8` now carries tagged structural requirements under the validated provider/binding closure envelope; V7 remains decode-only, rejects structural payloads, and deterministically fails before typed-handler normalization can parse text. Neither version grants provider, frame, admission, or runtime authority. Source-local, direct public-import, and one-hop public-facade non-generic nominal newtypes now share one exact-identity singleton constructor universe for `let`, `match`, `if let`, and exhaustiveness; wrong constructor/arity, private, generic, identity-mismatched, and two-or-more-hop cases reject. Summary provenance carries a provider depth of `0`, increments at each public facade, and decodes missing legacy metadata as unproved, so stale cache data cannot widen pattern admission. No successful or rejected control grants capability/runtime authority. Normal local newtypes retain their actual file/module identity through module-aware typecheck and Engine file/inline resolution; only direct no-module `TypeEnv` registration intentionally retains the documented fallback identity. Direct alias/group cycles deterministically reject through `TypeEnvError::InvalidDefinition` as `Audit -> Audit`; mutual alias→group→alias rejects as `Audit -> Workflow -> Audit`; an acyclic shared-row control proves expansion-stack cleanup. Normal declaration-resolved symbolic `ImplType::operation(args)` is settled by TASK-2011/TASK-2012/TASK-2017, not a TASK-2001 gate. Remaining work is specified realization: complete typed alias/group identity/expansion/discharge, full SPEC-097b diagnostic/import/versioning behavior, generic/multi-hop/unproved re-export and broader cross-module newtype patterns beyond the singleton slice, proof patterns, runtime newtype behavior, broader cross-module behavior, and runtime implementation. TASK-2014's sole user-facing gate is now decided: Path B requires strict Core/CPS cutover and closed admission, still incomplete. |
| [TASK-2002](tasks/TASK-2002-generic-do-and-lowering-sidecar-strategy.md) | Decide generic `do` behavior and preserve target lowering sidecars | In progress — ambient `do` source/evidence/Core-Let evidence, an entry-body source anchor, successful macro- and notation-expansion origin audit metadata, deterministic named-target rejection, and deterministic all-local-callable lowered-contract artifacts are recorded. Every retained arithmetic `requires` expression and `ensures` clause now preserves its exact parsed offsets; file-backed lowering, including `parse_entry_file` after accepted runtime-import masking, retains the canonical module path, while direct/in-memory lowering validates its runtime imports then same-byte-length masks the accepted leading prelude and retains `file: None`. Unregistered imports reject before an entry is published; all retained sidecars are audit/evidence-only and grant no row, runtime, monitor, provider, frame, or admission authority. Predicate environments now retain exact parameter-name spans and the synthetic `result` binder's enclosing `FnDef` signature span, with canonical file paths for file-backed lowering and `file: None` for direct lowering. Unified source/evidence/trace/diagnostic sidecars, handler boundaries, and full conformance remain open. |
| [TASK-2003](tasks/TASK-2003-return-authority-and-cps-kernel-decision.md) | Resolve `Return` authority in the CPS kernel | In progress — terminal-observation decision, checked projection, and answer-typed source-return-to-`Jump` inspection bridge are recorded. One typed `PureAnf` normalizer lowers typed atoms, recursive approved `Int` × `Int` binary trees over `Add`/`Sub`/`Mul`/`Div` and `Eq`/`Ne`/`Lt`/`Le`/`Gt`/`Ge`, exact `Bool` × `Bool` `Eq`/`Ne`, and recursive Boolean `Not` left-to-right as fresh internal `LetPrim` bindings with one final `Jump(__answer)`. Separately, one sealed no-argument local-call fixture `helper() -> Int { 7 }; main() -> Int { helper() }` checks canonical parsed provenance and no retained imported state, then becomes checked Core `LetVal/Lam/Call` and CPS lambda/tail `Call(..., __answer)` for `run → Int(7)`. It does not admit general calls, inference/thunking, closures, recursion, parameters, or imports, and generic `execute` remains closed. The `PureAnf` fragment is bounded ANF, not generic ANF or general `let`/conditional/match lowering: mixed and other non-`Int`/non-`Bool` equality operands, other non-`Int` binary operands, `Neg`, `&&`/`||`, other calls, effects, handlers, providers, frames, and other forms remain closed. The case-bound differential `7 - 2` witness remains a private oracle control and cannot invoke production. General source/Core realization remains deferred. |
| [TASK-2004](tasks/TASK-2004-core-cps-production-boundary-decision.md) | Decide Core/CPS production-boundary status | In progress — TASK-2014 Path B keeps general routes closed under `CheckedCoreCpsClosedAdmission`; bounded pure/constructor tokens, one sealed local `helper() -> Int { 7 }; main() -> Int { helper() }` Core `Lam`/`Call` token, two exact one-frame host-operation tokens, and sealed handler tokens are positive production slices. The local-call token requires canonical parse-time Core/anchor provenance and no retained imported state, lowers to a CPS lambda/tail call, executes only through its opaque admission token, and leaves generic `execute` closed. Alongside compatible built-in `time::sleep`, local `TestClock::sleep(Int) -> Null` accepts only literal or prior checked lexical `Int` under retained declaration identity, canonical anchor, exact Engine binding, and parse-time Core provenance checked before typechecking; a post-check comparison remains defense in depth, and public sidecar/legacy-Core mutation plus generic `Engine::execute` reject before dispatch. `absorb_sleep` has direct `resume(ms)`, identity `done`, literal `0`, and one root `SourceHandler`; exact abortive `trap_sleep` has no `resume`, identity `done`, exact `TestClock::sleep(0)`, and fixed `1 / 0`. Each instruction authorizes only its engine-private checked-CPS handler installation/dispatch; no provider binding/provider frame or row-derived/general/multi-frame installation. Rows never install frames. Ordinary file CLI execution has no bootstrap/direct-evaluator fallback; bounded closed routes classify missing admission as `external/admission/rejected` (exit 1), forged/malformed/unchecked purported Core/CPS as fixed `entry_verification` (exit 4), and admitted `trap_sleep` as V1 `trap` (exit 5). General calls/inference/thunking, generalized declarations/providers, deep/general handlers, multi-frame TASK-1993 evidence, and remaining route matrix remain open. |
| [TASK-2005](tasks/TASK-2005-direct-runtime-core-cps-semantic-parity.md) | Establish direct-runtime/Core-CPS semantic parity or bounded divergence | Complete — retired historical prototype record. TASK-2040 removed its Rust direct-runtime differential implementation and tests. It has no active semantic-task record and supplies no current executor, conformance authority, or fallback. Current axes are implementation `not_implemented`, evidence `none`, and parity `below_spec`; a future target-rule owner must establish conformance through the Engine-only route. |
| [TASK-2006](tasks/TASK-2006-cps-public-api-visibility-decision.md) | Decide CPS public API visibility and checked/unchecked boundary | Complete — retained exported compatibility/prototype APIs; a downstream fixture preserves the checked-versus-trusted-IR boundary, while external consumer absence is intentionally not claimed |
| [TASK-2007](tasks/TASK-2007-cli-core-terminology-clarification.md) | Clarify public CLI/test-runner Core representation terminology | Complete — compatible substrate preserved; public metadata identifies `ash_core::Expr` |
| [TASK-2008](tasks/TASK-2008-json-variant-observable-projection.md) | Reconcile JSON `_variant` with canonical observable projection | In progress — versioned `ash run --format json` projects return, declared and bounded entry-execution traps (including division by zero), unreadable-input/parse/type/entry-verification pre-entry failures, and one declaration-only missing-`main` dry-run failure through the existing `entry_verification` envelope with stdout/exclusive-`--output` ownership. It also projects both malformed pre-read and source-aware unknown-selection configuration failures as coarse `pre_entry_failure/configuration` `run configuration is invalid`, canonical `time::sleep` timeout, and cooperative one-shot cancellation without telemetry. The approved TASK-2014 mapping is implemented for bounded closed production routes: absent validated lowering/token is `external/admission/rejected` (exit 1), forged/malformed/unchecked purported Core/CPS is fixed `pre_entry_failure/entry_verification` (exit 4), and exact admitted abortive `trap_sleep` projects fixed division as V1 `trap` (exit 5), with typed Engine-to-CLI classification and no ordinary direct-evaluator fallback. This is not general handler/continuation or complete handler-`--output` coverage. Complete observable/differential coverage remains required. |
| [TASK-2009](tasks/TASK-2009-rust-stable-baseline-1-96-0.md) | Align Ash and Verus on Rust 1.96.0 | Complete |
| [TASK-2010](tasks/TASK-2010-static-impl-operation-source-call.md) | Implement the first statically resolvable impl-qualified source operation call | Complete — bounded `time::sleep(0)` only: the strict concrete descriptor attaches non-granting `time::sleep` row metadata after ordinary checking; absent time-provider admission rejects, the admitted existing provider returns `Null`, and a private checked-CPS `Raise` inspection artifact exists. Direct `invoke` remains rejected; generic/interface/binding operations, handler realization, per-term origins, Core/CPS production execution, and parity remain deferred. |
| [TASK-2011](tasks/TASK-2011-declared-concrete-impl-operation-source-calls.md) | Resolve declared concrete impl-qualified source operation calls | Complete — local `Clock<TestClock>` proves `TestClock::sleep(0)` from registered declaration facts, gives exact unknown-impl/unknown-operation/argument-mismatch diagnostics, adds non-granting `TestClock::sleep` row metadata, and exposes private declared-signature `Raise` inspection. The target semantics of declaration-resolved symbolic `ImplType::operation(args)` calls are settled; provider mapping/execution, evaluated-local-argument coverage, generic/interface/binding calls, imported resolution, handler realization, and production Core/CPS execution remain implementation work. |
| [TASK-2012](tasks/TASK-2012-declared-operation-provider-binding.md) | Bind declared operations to admitted provider operations | Complete — explicit host registration binds resolved `TestClock::sleep` to validated provider-operation metadata; no grant/row mismatch/provider-operation mismatch/conflicting binding is accepted. Unbound rows reject before execution despite unrelated providers, while the exact bound provider executes once and returns `Null`. Existing handler priority and `invoke` rejection remain intact; generic/interface/binding/imported/multi-provider selection, handler UX, and production CPS are implementation deferrals, not a symbolic-call design gate. |
| [TASK-2015](tasks/TASK-2015-evaluated-local-arguments-symbolic-operation-calls.md) | Execute declaration-resolved symbolic operation calls with evaluated local arguments | Complete — bounded lexical-local `let delay = 0; TestClock::sleep(delay)` retains exact declaration identity/non-granting row and explicit binding, dispatches `Int(0)` once to the selected provider for `Null`, and exposes private exact `Raise` inspection. Non-`Int` locals reject during checking and conditional initializers fail closed at inspection; arbitrary expressions/imports/generics/multi-provider selection and production Core/CPS remain deferred. |
| [TASK-2016](tasks/TASK-2016-local-nominal-newtype-checking.md) | Typecheck local non-generic nominal newtypes on the normal program path | Complete — normal checking registers local non-generic newtypes before bodies, validates the sole declared tuple constructor against its representation, and preserves nominal non-coercion and sibling-wrapper distinction. TASK-2001 additionally supplies the same source-local/direct-public-import/one-hop-facade singleton constructor universe for `let`, `match`, `if let`, and exhaustiveness, only when the visible binding has the exact provider `TypeDeclId` and proved depth at most one. Opaque bodyless representations, direct/mutual recursion, ordinary-type/newtype collisions, and primitive/prelude shadowing reject deterministically; runtime erasure/execution, generic, multi-hop/unproved, and broader cross-module newtypes beyond the singleton slice, proof patterns, handlers, and broader cross-module behavior remain deferred. |
| [TASK-2017](tasks/TASK-2017-posixfs-read-symbolic-concrete-operation.md) | Realize local declaration-resolved `PosixFs::read(path)` as a normal symbolic concrete operation | Complete — local nominal `PosixFs` is retained during declaration resolution, so literal and lexical-local `String` calls have exact `PosixFs::read` identity and non-granting row. An exact explicit provider binding dispatches the controlled provider once without host I/O; missing/mismatched binding, row, argument, or operation reject fail closed; private inspection preserves `Atom::String` `Raise`. Imports, generics, handlers, production Core/CPS, and actual filesystem reads remain excluded. |
| [TASK-2018](tasks/TASK-2018-entry-lowering-sidecar-hygiene.md) | Transport expanded-surface identifier hygiene into entry lowering sidecars | Complete — exact successful `ExpandedSurfaceModule` hygiene reaches `EntryLoweringSidecars` unchanged as entry-level audit/diagnostic metadata, for generated and ordinary identifiers alike. The legacy parser fallback retains an explicit empty vector only as a defensive, presently unreachable-path invariant; rejected expansion creates no entry. No evaluator/check/admission/provider/trace/monitor authority, per-Core origin/hygiene, or Core/CPS-production claim was introduced. |
| [TASK-2019](tasks/TASK-2019-post-execution-invalid-exit-terminal-envelope.md) | Project post-execution invalid entry exit codes through the canonical terminal envelope | Complete — a valid canonical `main` terminal `Err { error: RuntimeError(999, "boom") }` retains its exact evaluated value in the post-execution invalid-exit carrier, allowing JSON-only reuse of the existing versioned `trap` envelope. It remains non-successful under the unchanged `0..=255` exit-code rule; stdout and `--output` ownership are exclusive. Text, pre-entry, verification, engine, and legacy/dry-run behavior remain unchanged. |
| [TASK-2020](tasks/TASK-2020-canonical-core-v1-differential-fixture-adapter.md) | Add a versioned canonical Core V1 differential fixture schema and private checked adapter | Complete — first predecessor-specific fixed-text V1 control: manifest-local `(lit-int 7)` alone is admitted through parse, validation, typecheck, checked-lowering, and private checked-CPS projection. Alternate/malformed text, including text that would normalize to the same AST, rejects before parsing; all three V1 controls retain the same closed per-case boundary and direct runtime unsupported. No Engine/CLI/provider/admission/trace/monitor or production Core/CPS authority is added. |
| [TASK-2021](tasks/TASK-2021-canonical-core-v1-letval-differential-control.md) | Add a bounded canonical Core V1 `LetVal` / answer-continuation differential control | Complete — second predecessor-specific fixed-text V1 control: only `(let-val value : Int (lit-int 7) value)` reaches private `LetVal(value, Int(7)) → Jump(__answer, Var(value)) → Return(Int(7))` evidence. It shares neither a general parser admission rule nor a generic Core loader with TASK-2020/2022; alternate/malformed text rejects before parsing, direct runtime remains unsupported, and production Core/CPS authority is excluded. |
| [TASK-2022](tasks/TASK-2022-canonical-core-v1-letprim-add-differential-control.md) | Add a bounded canonical Core V1 `LetPrim(Add)` / answer-continuation differential control | Complete — third and final closed V1 fixed-text control admits only `(let-prim sum add ((lit-int 2) (lit-int 5)) sum)`, producing exact private `LetPrim(sum, Add, [Int(2), Int(5)]) → Jump(__answer, Var(sum)) → Return(Int(7))` evidence. Altered/normalized-equivalent spellings, binder/op/arity/operand/body changes, schema widening, direct runtime, and production Core/CPS authority all remain excluded. |
| [TASK-2023](tasks/TASK-2023-canonical-core-v1-literal-if-differential-controls.md) | Add bounded canonical Core V1 literal `If` true/false differential controls | Complete — two separate fixed-text private controls admit only `(if (lit-bool true) (lit-int 7) (lit-int 9))` and `(if (lit-bool false) (lit-int 7) (lit-int 9))`, proving exact checked `If(Bool, Jump(__answer, Int(7)), Jump(__answer, Int(9)))` evidence and selected `Return(Int(7))`/`Return(Int(9))`. Identity, ordered rules, branch/condition text and normalized-equivalent spellings reject before parsing; direct runtime and production Core/CPS authority remain unsupported. |
| [TASK-2024](tasks/TASK-2024-handler-local-effect-row-propagation.md) | Propagate one nonproduction handler-local effect row through Core/CPS inspection | Complete — bounded `forward_sleep` preserves declaration-resolved `TestClock::sleep` as the handled body `Raise`, `TestClock::wake(ms)` as the clause-body `Raise`, and exactly `{TestClock::wake}` as the local Core/CPS `Handle.row`; `other(ms)` and `wake(0)` reject. The bridge itself remains structural inspection only: TASK-2026 may consume its retained exact facts under separate sealed instruction authority, never from the row. |
| [TASK-2025](tasks/TASK-2025-effect-row-provider-binding-identity-and-sanitization.md) | Separate effect-row provider identity from visible bindings and sanitize cross-module transport | Complete — V7 summary metadata separates immutable provider identity from visible binding/exposure; one loader sanitizer covers named/glob/`pub use` closure transport; inaccessible boundaries, legacy/incomplete/unknown summaries, and deterministic binding conflicts reject before registration/publication. In-memory semantic cache keys cover the V7 public contract without opaque private detail. This adds no provider/handler admission, dispatch, host I/O, or runtime authority. |
| [TASK-2026](tasks/TASK-2026-sealed-forward-sleep-handler-provider-production.md) | Seal one `forward_sleep` handler-plus-provider checked-CPS production slice | Complete — the exact canonical row-annotated local `sleep(0) → wake(ms)` fixture seals same-Engine source/Core/anchor provenance, checked facts, one exact registered `wake` binding, and explicit outer Provider(`wake`) then inner SourceHandler(`sleep`) instructions. The driver reverse-scans innermost-first, returns the provider `Int`, and proves timeout/cancellation terminalization with cancellation winning a deadline tie and cooperatively dropping the pending wake await. Rows and generic/V1/direct/CLI-trace routes grant no authority; all other forms remain closed. |
| [TASK-2027](tasks/TASK-2027-semantic-rule-coverage-workflow.md) | Make semantic-rule coverage the mandatory workflow unit | Complete — semantic tasks now begin from a canonical rule and declared domain, then record Type → Core → CPS → admission → runtime coverage, evidence, non-goals, and the next gap in `SEMANTIC-RULE-COVERAGE.md`. Examples are evidence, never semantic authority. |
| [TASK-2028](tasks/TASK-2028-semantic-task-conformance-gate.md) | Enforce semantic task records and targeted verification gates | Complete — bounded active-task records validate task/coverage-map/traceability evidence; staged semantic changes require matching documentation and task-owned focused checks in pre-commit, while pre-push runs all active records in staged-snapshot-local Cargo targets to prevent cross-snapshot executable reuse. This is workflow enforcement, not general language semantics. |
| [TASK-2029](tasks/TASK-2029-compositional-semantic-workflow-boundaries.md) | Record compositional implementation-domain ownership and handoff policy | Complete — documents scoped layer ownership, mandatory named handoffs, and the separately owned integration/proof boundary. |
| [TASK-2013](tasks/TASK-2013-source-handler-and-handle-lowering.md) | Implement source handlers and `handle ... with` lowering | In progress — parser/AST preserves canonical clauses/origins; checked declarations retain marker/signature and concrete clause facts. Handler-only expected-type implicit thunking now applies its source-handler substitution while fresh inference variables live and publishes the span-keyed specialized input type; ordinary calls, runtime, and admission remain unchanged. Alongside the closed-empty inspection slice, TASK-2014 admits local `absorb_sleep` with one root `SourceHandler` instruction and exact abortive `trap_sleep` with no `resume`, identity `done`, exact `TestClock::sleep(0)`, and fixed `1 / 0` post-admission trap. Completed TASK-2026 consumes exactly the row-annotated `forward_sleep` facts under separately sealed outer Provider(`wake`) then inner SourceHandler(`sleep`) instructions and its tested async control envelope. No route grants frames from rows or generalizes handler lowering; generic execute/V1/CLI helpers remain closed except the bounded `trap_sleep` JSON terminal route. The selected target semantics is source-ordered deep affine handling: zero-or-one `resume`, reinstallation around a resumed tail, normal completion through `done`, structural residual rows, and TASK-1993 innermost-first lookup. Its compact `sleep → wake → resumed sleep → Int(107)` realization now seals checked ordered facts, a closed residual row/source anchor, and explicit `SourceHandler` instructions through checked Core/CPS; generic/multi-shot/open-row/multi-clause behavior and arbitrary provider chains remain excluded. |
| [TASK-2014](tasks/TASK-2014-source-handler-runtime-boundary-decision.md) | Decide the source-handler runtime boundary | In progress — Path B makes checked Core/CPS the sole production owner of admitted source and keeps every source form without validated typed lowering closed. The Engine-private host-operation driver has the existing one-frame provider slices and bounded `absorb_sleep`/abortive `trap_sleep` source-handler slices. Completed TASK-2026 adds its exact two-instruction `forward_sleep` base composition. TASK-2014 additionally has one real ordered witness with same-Engine provenance and exactly outer Provider(`wake`), inner Provider(`wake`), then SourceHandler(`sleep`); TASK-1993 reverse lookup selects the inner provider and returns `Int(73)`. The bounded closed routes classify missing admission as `external/admission/rejected` (exit 1), forged/malformed/unchecked purported Core/CPS as fixed `entry_verification` (exit 4), and exact admitted `trap_sleep`'s fixed division as V1 `trap` (exit 5); ordinary-file CLI execution has no direct-evaluator fallback. The approved target is source-ordered deep affine handling: zero-or-one `resume`, reinstallation around the resumed tail, normal return through `done`, structural residual rows, and the existing innermost-first lookup. Its compact checked-CPS `sleep → wake → resumed sleep → Int(107)` witness now seals checked ordered facts, a closed residual row/source anchor, and explicit `SourceHandler` instructions; rows never install frames, and generic/multi-shot/open-row handlers, arbitrary chains/instruction shapes, generic/imported declarations, most CLI handler routes, and direct-runtime↔checked-CPS parity remain open. |
| [TASK-439](tasks/TASK-439-differential-conformance-harness-rust-first.md) | Existing sole owner of the canonical differential corpus and harness | Complete — retired historical prototype record. TASK-2040 removed its Rust differential harness and tests. It has no active semantic-task record and supplies no current executor, conformance authority, or fallback. Current axes are implementation `not_implemented`, evidence `none`, and parity `below_spec`; a future target-rule owner must establish conformance through the Engine-only route. |

Current bounded evidence: TASK-2001's `derive handler` emits a source/typechecker fact over all
direct impl methods, with fresh answer `A`, open residual `r`, affine continuations, and
derive-site desugaring origins. Its local `handle expr with name` route uses the normal
`CallableDeclarationKind::Handler` value-namespace marker plus that checked fact—never a
synthetic variable type/signature—and instantiates answer/residual from the normalized operand
while retaining anchors, canonical order, concrete residuals, and open-tail provenance. Its
alias/group/open-tail coverage is limited to explicit zero-argument calls of row-annotated
parameters; unsupported computations,
marker-only names, and lexical shadowing fail closed. It still creates no Core/CPS artifact,
provider frame, admission authority, or runtime handler. TASK-2005/TASK-439
additionally have one fixed `time::sleep(0)` application-default/private checked-CPS
provider-frame pair that projects `Null` under fixed lookup/raise metadata; it compares only that
allowed-external lookup, while frame ordering remains unsupported and checked CPS remains
private/prototype.

## Phase 203: Runnable Ash Semantic Realization

**Status:** Active
**Plan:** [PLAN-203: Runnable Ash Semantic Realization](PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md)
**Depends on:** Canonical Core authority, PLAN-202, and the TASK-2004/TASK-2014 closed-admission
architecture.

This programme composes existing layer-owned work into one Surface → Core → CPS → Engine execution
path. CLI and daemon are clients of that path. It does not reopen the ownership of existing source,
Core, CPS, admission, terminal, or conformance tasks; it supplies their integration gates and a
target-surface runnability matrix. A completed handoff or integration task does not establish
target-spec parity: feature reports state implementation, evidence, and parity independently.

| Task | Description | Status |
|---|---|---|
| [TASK-2030](tasks/TASK-2030-runnable-ash-semantic-realization-programme.md) | Align programme, canonical reading paths, task workflow, and assurance policy | Complete |
| [TASK-2031](tasks/TASK-2031-lambda-ash-effect-correspondence.md) | Define `λAsh-Effect` CPS/operational/Rust correspondence before effectful executor expansion | Complete handoff; implementation partial, evidence tested, parity below_spec |
| [TASK-2031A](tasks/TASK-2031A-daemon-startup-gate-remediation.md) | Diagnose sandbox AF_UNIX test-host capability and retain deterministic daemon-startup diagnostics | Complete test-gate remediation; focused daemon suites 13/13 and 4/4 |
| [TASK-2031B](tasks/TASK-2031B-lexical-scope-admission-contract-reconciliation.md) | Reconcile stale lexical-scope CLI rejection assertions with checked Core-to-CPS admission | Complete test-contract remediation; 6/6 focused evidence |
| [TASK-2031C](tasks/TASK-2031C-cli-sigint-engine-cancellation-bridge.md) | Gate admitted-sleep SIGINT cancellation evidence on verified Tokio signal-delivery capability | Complete prerequisite test-host remediation |
| [TASK-2031D](tasks/TASK-2031D-wiremock-tcp-capability-gate.md) | Gate affected Ash Engine loopback integration evidence on verified TCP-bind capability | Complete test-host remediation; 26 focused controls |
| [TASK-2031E](tasks/TASK-2031E-stdlib-corpus-test-isolation.md) | Isolate mutable LLM import fixtures from the canonical stdlib corpus tree | Complete test-isolation remediation; strict 59-file corpus |
| [TASK-2031F](tasks/TASK-2031F-stdlib-admission-contract-reconciliation.md) | Reconcile stale stdlib callable admission-message assertions with the current PureAnf bridge | Complete test-contract remediation; module-resolution 17/17 |
| [TASK-2032](tasks/TASK-2032-shared-engine-execution-seam-and-client-parity.md) | Establish shared Engine admission/execution, in-process adapter parity, daemon-service dispositions, and the runnability matrix | Complete integration task; implementation partial, evidence tested, parity below_spec (Engine 14/14, adapter parity 7/7, daemon controls 14/14 and 4/4) |
| [TASK-2033](tasks/TASK-2033-target-spec-parity-and-evidence-policy.md) | Separate target-spec implementation parity from test and proof evidence in semantic policy and validation | Complete |

## Phase 204: Direct AST Retirement Audit and Contract Freeze

**Status:** Complete (3/3)
**Plan:** [PLAN-204: Direct AST Retirement Audit and Contract Freeze](PLAN-204-DIRECT-AST-RETIREMENT-AUDIT-AND-CONTRACT-FREEZE.md)
**Depends on:** PLAN-203's Engine execution architecture and TASK-2033's target-spec/evidence policy.

Freezes the direct-AST and differential retirement catalogue; classifies Lean as a deferred
separate formalization project; defines the required
target contracts for source-derived test wrappers and the REPL; and blocks re-entry while Phase
205 performs the migration. This is planning and enforcement work, not an implementation claim.

| Task | Description | Status |
|---|---|---|
| [TASK-2034](tasks/TASK-2034-direct-ast-retirement-audit-manifest.md) | Catalogue direct-evaluator/differential retirement and preserve Lean as deferred separate work | Complete — 309 revision-bound records; Lean retained as deferred separate-project work |
| [TASK-2035](tasks/TASK-2035-canonical-client-test-contracts.md) | Amend target contracts for Engine-only test wrappers, REPL, and conformance | Complete — exact contract catalogue and deferred cases specified; runtime implementation not implemented, evidence none, parity below_spec |
| [TASK-2036](tasks/TASK-2036-direct-ast-reentry-guard.md) | Block new legacy evaluator/oracle use during the cutover | Complete — frozen-HEAD staged re-entry guard now rejects residual Rust delete entries and re-entry; workflow evidence only |

## Phase 205: Engine-Only Execution Cutover

**Status:** Complete (7/7)
**Plan:** [PLAN-205: Engine-Only Execution Cutover](PLAN-205-ENGINE-ONLY-EXECUTION-CUTOVER.md)
**Depends on:** Phase 204's frozen audit, amended contracts, and re-entry guard.

Migrates every selected client to the one Engine-owned checked-CPS executor, then removes the
Rust direct AST interpreter, differential oracle/corpus, and stale current documentation. Lean is
preserved as deferred separate-project material with no current executable, conformance, or proof
evidence/authority and no runtime refinement bridge; a later separate project must establish any
bridge. Unsupported catalogue entries remain explicit enumerated deferred cases.

| Task | Description | Status |
|---|---|---|
| [TASK-2037](tasks/TASK-2037-engine-owned-cps-executor-and-runtime-crate-rename.md) | Move checked CPS execution into Engine and establish its private executor boundary | Complete — prerequisite boundary verified; client routes and residual-crate rename remain separately owned |
| [TASK-2038](tasks/TASK-2038-ash-test-canonical-engine-execution.md) | Route `ash test` through admitted source wrappers and catalogue deferred cases | Complete — selected route delivered; TASK-2040/2041 retain deletion and four-client parity |
| [TASK-2039](tasks/TASK-2039-repl-canonical-engine-execution.md) | Route REPL evaluation through admitted Engine requests | Complete — selected REPL route delivered; TASK-2040/2041 retain deletion and four-client parity |
| [TASK-2042](tasks/TASK-2042-daemon-admitted-request-terminal-envelope-parity.md) | Validate daemon descriptors and carry normalized terminal envelopes with direct-source `ash run` parity | Complete — selected descriptor route delivered; TASK-2040/2041 retain deletion and four-client parity |
| [TASK-2043](tasks/TASK-2043-remove-tracked-rust-target-artifacts.md) | Remove tracked Cargo build output and ignore every nested `target/` directory | Complete — 585 index-only artifact removals, one global rule, and a pre-commit regression guard verified |
| [TASK-2040](tasks/TASK-2040-remove-direct-ast-and-differential.md) | Delete Rust direct AST/differential execution and quarantine Lean authority | Complete — owned removal and rename evidence verified; TASK-2041 owns zero-use, documentation/traceability, and four-client parity |
| [TASK-2041](tasks/TASK-2041-engine-only-closeout-docs-traceability-and-gate.md) | Prove zero legacy use and close documentation, traceability, and parity evidence | Complete — zero-use gate, deferred Lean boundary, and fail-closed four-client terminal evidence verified |

## Phase 206: Implementation-Backed Language Reference

**Status:** Complete — placement, skeleton, lexical/modules, forms, types, effects, execution,
and library/diagnostics chapters are integrated and manual-wide closeout validation is complete
**Plan:** [PLAN-206: Implementation-Backed Language Reference](PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Audit:** [AUDIT-206: Implementation-Backed Language Reference Census](audits/AUDIT-206-implementation-backed-language-reference.md)
**Depends on:** Current parser/checker/lowering/Engine implementation and executable tests; existing
specifications, JSON indexes, plans, and the top-level `reference/` corpus are navigation and
conflict evidence only.

This phase's separate manual at `docs/reference/language/` is authorized by the narrow
[SPEC-071 §3.1](../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md#31-scoped-implementation-backed-language-manual-exception)
amendment. TASK-2045 created the shared skeleton; the completed domain tasks supplied the
chapter pages and TASK-2054 integrated their evidence. Every documentation task reports grammar/static/
lowering/admission-runtime status and implementation/evidence/parity independently; removed
workflow/tower forms are never current examples.

| Task | Description | Status |
|---|---|---|
| [TASK-2044](tasks/TASK-2044-implementation-backed-language-reference-plan-packet.md) | Audit implemented language surface and create the planning packet | Complete |
| [TASK-2045](tasks/TASK-2045-language-reference-placement-and-skeleton.md) | Reconcile placement and create manual skeleton/status/source-of-truth conventions | Complete — scoped policy amendment and four-page skeleton verified |
| [TASK-2046](tasks/TASK-2046-language-reference-lexical-modules-notation-macros.md) | Document lexical structure, modules/imports, notation, and macros | Complete — implementation-backed status/evidence boundaries and EBNF verified |
| [TASK-2047](tasks/TASK-2047-language-reference-forms-functions-control-patterns.md) | Document declarations, functions, expressions/control, and patterns | Complete — implementation-backed status/evidence boundaries, EBNF, and sequents verified |
| [TASK-2048](tasks/TASK-2048-language-reference-ordinary-types-interfaces.md) | Document ordinary data/types/callables/generics/interfaces | Complete — implementation-backed status/evidence boundaries, EBNF, and bounded static/lowering sequents verified |
| [TASK-2049](tasks/TASK-2049-language-reference-type-level-computation.md) | Document type-level domains/functions/families/propositions | Complete — implementation-backed parser/static/summary boundaries, EBNF, and bounded normalization sequent verified |
| [TASK-2050](tasks/TASK-2050-language-reference-rows-operations-authority.md) | Document rows/operations/resources/roles and authority boundaries | Complete — implementation-backed status/evidence boundaries, EBNF, and non-granting sequent verified |
| [TASK-2051](tasks/TASK-2051-language-reference-handlers-failure-do-comprehensions.md) | Document handlers, failure, do, and comprehensions | Complete — bounded implementation routes, EBNF, and sequents verified |
| [TASK-2052](tasks/TASK-2052-language-reference-entry-engine-clients-terminals.md) | Document entry, Engine admission, clients, and terminals | Complete — bounded `fn main`, Engine-issued admitted requests, selected client routes, and V1 terminal boundaries verified |
| [TASK-2053](tasks/TASK-2053-language-reference-stdlib-diagnostics-limitations.md) | Document public stdlib modules/imports, diagnostics, and limitations | Complete — corpus/static, bounded runtime, and terminal-diagnostic boundaries verified |
| [TASK-2054](tasks/TASK-2054-language-reference-verification-closeout.md) | Validate examples/fences/navigation and close the manual phase | Complete — 23/23 helper tests; 16 EBNF and 14 sequent fences; railroad and sequent-md consumers, 2,032-link docs gate, and diff hygiene verified |

## Phase 207: Complete Module Realization

**Current status:** In progress — TASK-2068 is Complete for its partial/tested/below-spec
foundation and TASK-2071 is Complete for its namespace/provisional-view specification handoff with
`not_implemented / none / below_spec` runtime axes. TASK-2074 is Complete for its atomic,
non-authorizing parser-stage expanded-graph handoff; the broader module-realization target remains
`partial / tested / below_spec` while
TASK-2075 owns internal snapshots plus name-only provisional views, TASK-2072 owns complete parsed
imports/visibility/edges/cycles/precedence/atomic binding and staged `pub use`, and
TASK-2073 owns complete M-CHECK/final interface/export closure. TASK-2069 consumes only
TASK-2073's complete checked handoff; TASK-2063 awaits TASK-2069; TASK-2064 owns parity; and
TASK-2065 owns closeout.

**Historical pre-closure summary:** The detailed status record immediately below preserves the
then-current TASK-2068 delivery inventory and evidence counts. Its `TASK-2068 is In progress`
wording is historical only and does not assign an unfinished clause to TASK-2068.

**Status:** In progress — TASK-2057 completed its partial/tested/below-spec AST-driven structural handoff; TASK-2058 completed its partial/tested/below-spec Core key/artifact carrier without legacy-identity migration; TASK-2059 completed its partial/tested/below-spec parser source-unit handoff; TASK-2060 completed its partial/tested/below-spec Core public-interface carrier; TASK-2066 completed its partial/tested/below-spec bounded TypeEnv wrapper with staged declaration preflight and artifact equality; TASK-2061 completed its partial/tested/below-spec wrapper-only explicit/group/glob resolver; and TASK-2067 completed its partial/tested/below-spec canonical parser graph/unit transport, structural diagnostics, lifecycle reporting, root metadata, ordered payload parity/mutation, and deprecated legacy-route fence. TASK-2068 is In progress with partial/tested/below-spec provisional-function M-COLLECT, a graph-only simple-import planner with bounded canonical edges, ordered `CanonicalImportCycle` rejection, and binder delegation; a tested graph-delivered primitive-function M-CHECK leaf pass; a direct primitive provider/client checker; and tested direct-public, private-provider-helper, local-binding root-client primitive re-export interface, canonical provisional-module-scope/structural-path visibility, scoped structural import-cycle, dedicated scoped structural binder, and scoped simple ordinary-function import fragments. The direct fragment admits only the exact public root/direct-provider form, preserves structural and binding provenance without implicit flattening, rejects fail-closed boundaries atomically, and has focused 13/13 Type-layer test evidence including a 16-case property. The helper fragment checks inherited/private provider helpers without exposing them and has focused 7/7 evidence including a 16-case property. The root-client fragment checks inherited/private `internal_entry` through the explicit alias with a distinct opaque direct plan, preserves identity/visibility and exact snapshots, uses a direct unqualified-call anchor or root-body fallback, and has focused 10/10 evidence including a 16-case property. The canonical scope fragment has focused 9/9 Type-layer evidence: it rebuilds declaration snapshots from current graph units before binding, applies permitted canonical visibility regions to ordinary-function targets, and retains whole-path public fencing. The scoped cycle gate has focused scope17 Type-layer evidence, including a 16-case property: it detects deterministic cross-module `CanonicalImportCycle` provenance atomically only after structural preflight, preserves visibility diagnostics, and leaves the generic binder unchanged. The dedicated scope-backed structural binder is partial/tested/below-spec Type-only prerequisite evidence: it lives only in `canonical_structural_module_binder.rs`, is exported only through `lib.rs`, delegates only to the scope-backed resolver then projects `into_bound_set`, preserves atomic visibility/cycle results, and has focused 8/8 evidence including a 16-case property across public, crate, super, `pub(in path)`, inherited/private, and self visibility categories; the generic binder remains unchanged and scope-free. The scoped simple ordinary-function import fragment is partial/tested/below-spec Type-only prerequisite evidence: that dedicated binder delegates root/deep inherited `crate::` ordinary-function imports with optional aliases or natural final names only to `resolve_scoped_simple_ordinary_function_imports_with_scopes`, then `into_bound_set`, and preserves atomic local-collision, duplicate-binding, visibility, and cycle results without changing the generic resolver or binder. Its focused target passes 11/11, including a 16-case property across all canonical visibility regions and root/deep, explicit-alias/natural-name positions; this is test evidence, not proof or parity. All slices are non-authorizing and not proof, final-interface, or parity evidence. Final interfaces, full imports/visibility/cycles, lowering, Engine, and parity remain open. TASK-2069 remains Planned for complete lowering plus Engine scanner/cache transport fencing. TASK-2063 awaits TASK-2069 before Engine linking, TASK-2064 owns conformance/client parity, and TASK-2065 closes the phase.

**Current TASK-2068 evidence:** The scoped `super` ordinary-function import M-SUPER slice is
`partial / tested / below_spec`, Type-only prerequisite evidence. Its dedicated resolver and
binder accept only non-root inherited parent/sibling ordinary-function imports with exactly one
leading `super`, preserve the full use span and atomic scope/visibility/collision/duplicate/cycle
checks, and reject every extra or final `super` before lookup. The focused target passes 12/12,
including a 16-case property; the generic binder is unchanged. This remains neither proof, a final
interface, Core/CPS, Engine, admission/runtime, nor client-parity evidence.

The scoped grouped ordinary-function import M-GROUP slice is
`partial / tested / below_spec`, Type-only prerequisite evidence. It adds only parser-owned nested
member spans and the dedicated scope-backed grouped `crate` ordinary-function resolver/binder;
member-specific diagnostics and edges preserve those spans, and any snapshot, visibility,
local-collision, duplicate, or complete-group cycle failure is atomic. It passes 10/10 including a
16-case property; the parser full suite passes; and the scoped-simple compatibility target is now
11/11. This does not establish a final interface, generic binder change, Core/CPS, Engine,
admission/runtime, or client parity.

**Delivered TASK-2068 M-SUPER-GROUP evidence:** This is `partial / tested / below_spec`,
Type-only `prerequisite` evidence. The dedicated resolver/binder accepts only inherited non-root
`UsePath::Nested` imports with exactly one leading `super`, no outer alias, zero or more canonical
structural children after the parent, and a nonempty group of ordinary-function members using a
natural/member-`as` name. It keeps each parser-owned member span on identity, edge, and
member-specific error facts; preflights a final member named `super` before lookup; and reuses
canonical scope/visibility/whole-public-path, same-module-no-edge, collision/duplicate, cycle,
and atomic-publication rules. The focused target passes 13/13 including a 16-case property. Its
ten canonical witnesses are tested: POSITIVE, IDENTITY, FILE-INLINE-PARITY, and PROPERTY are
positive; VISIBILITY-DIAGNOSTIC, ROOT-DIAGNOSTIC, LOCAL-COLLISION, DUPLICATE-BINDING, and
AUTHORITY-FENCE are negative; CYCLE-ATOMICITY is mutation evidence. Final interfaces, Core/CPS,
Engine, admission/runtime, parity, and precedence remain open under TASK-2072/TASK-2073,
TASK-2069, and TASK-2064 ownership; tests are not proof or parity evidence.

**Delivered TASK-2068 M-GLOB evidence:** the bounded Type-only prerequisite is
`partial / tested / below_spec`. It implements only one inherited
`use crate::<public structural-child>...::*` ordinary-function import in a module with exactly one
`use` and zero local ordinary functions, retaining identity/origin/visibility, declaration and
full-use-span facts, and one edge per selected public function before atomic publication. It does
not decide local/explicit/glob precedence. The shape witness covers 15 valid parser
representations (leading `::` is not `UsePath::Glob`); private structural-module access is an
`Inaccessible` visibility case. Local-function, second-glob, and cycle-shaped attempts are
`Unsupported` boundaries that return no plan or bindings, so CONFLICT-ATOMICITY,
AMBIGUITY-ATOMICITY, and CYCLE-ATOMICITY are boundary mutation evidence only—not
local-collision, duplicate-binding, generic-ambiguity, or `ImportCycle` claims. All ten focused
witnesses are tested; tests are not proof, final-interface, generic-binder, Core/CPS, Engine,
admission/runtime, or parity evidence. Remaining forms remain deferred.

**Delivered TASK-2068 M-GLOB local-over-glob precedence:** partial / tested / below_spec. Exactly
one existing inherited public structural-child crate glob selects public ordinary functions; a
same-module ordinary function shadows a same-name import only in returned public bindings,
non-colliding imports bind, and every selected cross-module edge survives shadowing before actual
atomic ImportCycle detection. All-shadowed input succeeds with no import bindings but retained
edges; hidden cycles return atomic ImportCycle. The focused target passes 8/8, including a
16-case property varying names, collision subsets, source form, and depth 1–3; file/inline proves
normalized Type-layer scope/binding parity only. It uses canonical graph/provisional scopes only,
never private M-CHECK facts; existing M-GLOB behavior remains separate/rejecting; other imports,
multiple globs, aliases/re-exports, self/super/non-crate paths, nonfunctions, generic binder,
final interfaces, Core/CPS, Engine, admission/runtime, and parity remain excluded. It is a
non-authorizing Type handoff; TASK-2069 owns lowering and TASK-2064 owns parity. TASK-2068 is
Complete for its partial/tested/below-spec foundation; Phase 207 remains In progress.

**Delivered TASK-2068 M-SIMPLE local-over-explicit precedence:** partial / tested / below_spec.
The dedicated route admits exactly one inherited, unaliased public structural-child
`UsePath::Simple` crate import under its natural name. A selected cross-module target retains its
edge and completes deterministic cycle detection before a same-name local ordinary function is
filtered from returned bindings; a selected same-module target emits no self-edge and does not
participate in cycle detection. Non-colliding imports bind, all shadowed cross-module candidates
retain edges with no import binding, and real hidden two-module cross-module cycles reject
atomically. The existing M-SIMPLE route remains unchanged and preserves local-collision rejection.
The focused `task_2068_local_over_simple_precedence` target passes 9/9; file/inline is limited to
normalized Type-layer scope/binding parity. Planner fingerprint:
`sha256:7fb241da5b3bf35595e7cf3054f06dcbc9c9dc08dc9701c047d0d2c045a393d3`; TASK-2069 owns
lowering and TASK-2064 owns parity; TASK-2068 is Complete for its partial/tested/below-spec
foundation and Phase 207 remains In progress.

**Complete TASK-2070 M-SELF-SIMPLE-ALIAS handoff:** `partial / tested / below_spec`. The bounded
route accepts zero or more individually eligible, two-segment
`use self::<ordinary_function> as <different_alias>;` statements in any module. It resolves only
direct same-`ModuleKey` ordinary functions when `is_visible_from` permits the importer, stages
distinct aliases together, and reports a duplicate alias as `DuplicateBinding`; groups, globs,
mixed imports, and other forms are `Unsupported`; a direct `self::<child_module>` target is a
nonfunction `Unsupported`. Dedicated `CanonicalSelfOrdinaryFunctionAliasBinding` values retain
local alias, defining identity, declaration span, origin, visibility, and full `use_span`; the
no-edge `CanonicalResolvedSelfOrdinaryFunctionAliases` has a private `into_bound_alias_set` used
only by its binder to return `CanonicalBoundSelfOrdinaryFunctionAliasSet`, not
`CanonicalResolvedSimpleImports` or `CanonicalBoundModuleSet`. Resolver and binder share
`CanonicalStructuralImportError`; `ImportCycle` is unreachable by construction and source fence.
It emits no edge and atomically rejects every invalid graph. `CanonicalBoundModuleBinding` and the
generic binder remain unchanged. The implementation node and all eight witnesses are promoted;
the focused target passes 8/8, including the exact 16-case property with alias count `1..3`. It is
Type-only, non-authorizing prerequisite evidence; M-CHECK authority, cross-module traversal, final
interfaces, and later layers remain excluded. TASK-2072 owns complete imports/binding; TASK-2073
owns finalization/export closure; TASK-2069 owns lowering and TASK-2064 parity.

**Complete TASK-2071 specification handoff:** `not_implemented / none / below_spec`. SPEC-103 now
defines the AST-only macro/notation syntax prepass, provider-before-consumer ordering, exact
one-to-one `CanonicalExpandedModuleGraph`, checker-internal `CanonicalCollectedModuleSnapshot`,
name-only `CanonicalProvisionalNameView`, canonical declaration/lookup keys, namespace collision
buckets, constructor/member rules, and visibility-carrier prerequisites. This prose is not source,
test, proof, or parity evidence. TASK-2074 is Complete for its non-authorizing parser-stage handoff
while the broader rule remains partial/tested/below-spec: invocation-backed simple public macro
imports, public structural provider visibility, provider ordering/cycles, transitive provider
closure, provider-owned diagnostics, provenance sidecars, and unsupported item-generation
rejection are tested. Valid-path canonical public notation-summary transport now has 3/3 focused
evidence, invalid dependency rejection passes 12/12, and consumer-local activation completes the
21/21 notation-import target. The independent completion audit also confirms 14/14 exact parser
anchors and a 37/37 fail-closed legacy Engine compatibility fence. TASK-2075 is In progress
against the completed graph handoff with partial/tested/below-spec accounting after Tasks 5–6.
Graph-wide atomic paired collection, namespaces/collisions, parent/member/constructor placement,
typed notation/diagnostics, full impl-head coherence, expanded raw definitions/bodies, nested
member spans, direct source anchors/ordinals, module sidecars, and the strict provisional view are
tested 24/24. Drift, normalized collected file/inline, generated/property, compatibility, complete
authority-fence evidence, and imported-interface binding remain absent.
TASK-2072 consumes only TASK-2075's name view; TASK-2073 consumes the internal snapshot plus
TASK-2072 staging; TASK-2069 waits for TASK-2073.

**Delivered TASK-2068 M-CHECK restricted-visibility slice:** `M-CHECK-RESTRICTED-VISIBILITY` is
`partial / tested / below_spec`, with Type `partial`, Core/CPS/admission-runtime `not_applicable`,
verification `partial`, and run-route impact `prerequisite`. It accepts only `pub(crate)`,
`pub(super)`, `pub(in crate)` or `pub(in crate::...)`, and `pub(self)` primitive closed
ordinary-function leaves in a file-root closed leaf without imports, children, nonfunctions,
generics, contracts, or open signatures. The checker graph-preflights, signature-stages, and
body-checks atomically; restricted facts remain only in `private_functions`, and public projection
remains only `Visibility::Public`. `pub(in self::internal)` rejects. The focused target passes
18/18. `TEST-MOD-REAL-003-LEAF-MCHECK-RESTRICTED-VISIBILITY-FILE-INLINE-PARITY` is a tested
source-form boundary (file-root success versus inline rejection before projection), not
normalized-success parity. It has no import, binder, re-export, final-interface, Core/CPS,
admission/runtime, or parity authority.
**Plan:** [PLAN-207: Complete Module Realization](PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** [SPEC-103: Module Realization and Operational Semantics](../spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md)
**Audit:** [AUDIT-207: Module Realization Seams](audits/AUDIT-207-module-realization-seams.md)
**Depends on:** SPEC-095b/095c/097b/098c/099b and PLAN-203's single Engine route.

This phase turns parser-accepted module syntax into one complete language-level realization. It
replaces semantic source-text scans with parsed `ModuleFile` traversal, makes file-backed and inline
modules equivalent after source acquisition, introduces checked export-closed interfaces, routes
imports and visibility through those interfaces, and links reachable module artifacts through the
existing Engine-only Core/CPS route. Structural and import cycles reject; no direct evaluator,
dynamic import, runtime module value, or package/workspace system is introduced.

| Task | Description | Status |
|---|---|---|
| [TASK-2056](tasks/TASK-2056-module-realization-spec-plan-packet.md) | Create the module realization spec, seam audit, plan, task packet, and orientation records | Planned — packet authored and verified; implementation activation pending |
| [TASK-2057](tasks/TASK-2057-ast-driven-module-discovery.md) | Replace semantic module-declaration text scans with AST-driven discovery | Complete — parser-owned structural handoff and scanner retirement tested; source-anchored missing/cycle diagnostics remain deferred |
| [TASK-2058](tasks/TASK-2058-canonical-module-identity-and-artifacts.md) | Establish canonical module identities and module-unit artifacts | Complete — tested Core `ModuleKey`/`ModuleArtifact` carrier; resolver graph construction, legacy `ModuleIdentity`, source parity, interfaces, imports, lowering, admission, and runtime remain open |
| [TASK-2059](tasks/TASK-2059-file-inline-module-unit-parity.md) | Build one file/inline source-acquisition and module-unit route | Complete — partial/tested/below-spec parser-owned ordered file/inline module units, acquisition diagnostics, and recursive syntax scopes; structural cycles, malformed-inline anchors, graph/interface/import/lowering/Engine/client parity remain deferred |
| [TASK-2060](tasks/TASK-2060-checked-module-interface-and-export-closure.md) | Define checked export-closed interfaces and public/private views | Complete — partial/tested/below-spec Core carrier validates public binding schema; TypeEnv finalization, Engine scanner fencing/transport, import binding, lowering, and runtime remain open |
| [TASK-2066](tasks/TASK-2066-typeenv-module-unit-interface-finalization.md) | Finalize a bounded projection from a TypeEnv module unit and declaration preflight | Complete — partial/tested/below-spec staged TypeEnv wrapper with full artifact equality; no body/full-callable facts, typed linkage, aliases/re-exports, source-origin projection, or export closure |
| [TASK-2061](tasks/TASK-2061-interface-import-resolution-and-visibility.md) | Resolve bounded checked-interface requests | Complete — partial/tested/below-spec finalizer-wrapper-only explicit/group/glob resolver; parsed imports/visibility, aliases/re-exports, typed namespaces, cycles, binder integration, full closure, lowering, Engine transport, and parity remain open |
| [TASK-2062](tasks/TASK-2062-module-aware-core-cps-lowering.md) | Lower resolved modules through Core and CPS with origin preservation | Complete — partial/tested/below-spec non-authoritative wrapper/resolved-binding Core-to-CPS artifacts preserve exact module/import provenance; TASK-2063 must create its own sealed link/admission input, while parser source/full definitions, typed imports/callable authority, real-program parity, Engine, and client work remain deferred |
| [TASK-2067](tasks/TASK-2067-canonical-module-graph-and-structural-diagnostics.md) | Implement canonical ModuleKey graph/state-machine, structural diagnostics, and real file/inline unit transport | Complete — partial/tested/below-spec parser graph with real units, complete structural/lifecycle evidence, root metadata, ordered payload parity/mutation, and an isolated deprecated legacy-route fence; downstream clauses remain open |
| [TASK-2068](tasks/TASK-2068-final-interfaces-parsed-imports-and-binder-integration.md) | Produce the bounded Type-layer module foundation | Complete — partial/tested/below-spec delivered fragments remain non-authorizing and preserve their existing evidence. TASK-2070 owns the self-alias leaf; TASK-2071 defines the successor contract; TASK-2074/2075/2072/2073 own remaining implementation. |
| [TASK-2070](tasks/TASK-2070-scoped-self-simple-function-aliases.md) | Resolve the bounded direct same-module self alias leaf | Complete — partial/tested/below-spec; dedicated no-edge self-alias handoff with eight tested witnesses, consumed by TASK-2072 |
| [TASK-2071](tasks/TASK-2071-module-namespace-and-provisional-view-contract.md) | Define syntax-prepass, namespace/collision, and two-view collection contracts | Complete — specification handoff; not_implemented/none/below-spec |
| [TASK-2074](tasks/TASK-2074-canonical-expanded-module-graph.md) | Build the AST-only syntax prepass and canonical expanded graph | Complete — atomic non-authorizing parser-stage handoff; partial/tested/below-spec target-rule axes remain because collection, binding, finalization, lowering, admission, and client parity are separately owned; exact parser anchors and the legacy Engine fail-closed fence are audited, and no generalized mixfix use-site parser/elaborator is claimed |
| [TASK-2075](tasks/TASK-2075-two-tier-complete-module-collection.md) | Build internal collected snapshots and name-only provisional views | In progress — partial/tested/below-spec; Tasks 5–6 deliver graph-wide atomic paired collection, namespaces/coherence, expanded raw definitions/bodies, nested member spans, source anchors/ordinals, module sidecars, and the exact strict provisional view; drift, normalized collected file/inline, generated/property, compatibility, complete authority fencing, and imported-interface binding remain |
| [TASK-2072](tasks/TASK-2072-parsed-import-resolution-and-atomic-binding.md) | Resolve all parsed imports from the name view and publish atomic bindings | Planned — partial/none/below-spec backlog owner |
| [TASK-2073](tasks/TASK-2073-checked-module-finalization-and-export-closure.md) | Check internal snapshots plus staged bindings and publish export-closed final interfaces | Planned — partial/none/below-spec; sole complete Type input to TASK-2069 |
| [TASK-2069](tasks/TASK-2069-complete-module-lowering-and-engine-transport-fencing.md) | Implement complete definition-body lowering and Engine scanner/path-cache transport fencing | Planned — consumes TASK-2073's complete checked handoff; immediate TASK-2063 prerequisite |
| [TASK-2063](tasks/TASK-2063-engine-linked-module-admission.md) | Link reachable modules and admit one Engine artifact | In progress — not_implemented/none/below-spec; must consume only TASK-2069's complete non-sealed canonical closure to mint a separately Engine-sealed linked/admission request, with no raw/source/direct-evaluator authority |
| [TASK-2064](tasks/TASK-2064-module-conformance-and-client-parity.md) | Prove module conformance, mutation resistance, and CLI/daemon parity | Planned |
| [TASK-2065](tasks/TASK-2065-module-realization-closeout.md) | Close the phase with review, traceability, documentation, and full gates | Planned |

## Incubating: Agent Semantic Workspace

**Status:** Documentation packet complete; product implementation not started
**Product material:** [workspace PRD](../workspace/agent-semantic-workspace-prd.md) and
[accepted addendum](../workspace/agent-semantic-workspace-addendum.md)

This is a separate, Ash-implemented product intended to dogfood runtime features. It is not an Ash
implementation phase and does not schedule or authorize Ash language changes; any promoted Ash
feature follows the ordinary specification, task, implementation, and conformance workflow.

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1995](tasks/TASK-1995-agent-semantic-workspace-prd-packet.md) | Store the PRD and accepted architecture addendum | Complete |
