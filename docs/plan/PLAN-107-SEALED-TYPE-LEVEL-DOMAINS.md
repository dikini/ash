# PLAN-107: Sealed Type-Level Domains

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Phase 111 is DESIGN-034 SPEC-C. Do not implement normalization, public `type fn`, promoted data kinds, runtime marker inhabitants, type-level pattern matching, constructor-only import/export surfaces, generic sealed domains, or mutual recursive domain SCC support under this plan.

**Goal:** Implement [SPEC-059](../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md) by adding sealed `type domain` declarations, core-owned sealed-domain identities and constructor metadata, public semantic-summary transport for visible domain constructor sets, and typechecker validation for domain/field/visibility invariants on top of the completed Phase 110 substrate.

**Architecture:** Phase 111 is a metadata and validation packet. `ash-parser` owns the narrow `sealed type domain` surface and explicit rejection boundaries. `ash-core` owns canonical sealed-domain and marker-constructor identities plus public semantic-summary carriers. `ash-parser::lower` / lowering owns translation from surface declarations into core metadata. `ash-engine` transports public sealed-domain summaries across module boundaries without fabricating runtime constructors. `ash-typeck` registers imported/local domain metadata, validates field domain references and structural status, and enforces visibility and anti-leak rules. Coverage engines, normalization, and structural `type fn` consumers remain future packets.

**Tech Stack:** Rust 2024, `ash-core`, `ash-parser`, `ash-engine`, `ash-typeck`, existing Phase 109/110 semantic-summary infrastructure, canonical `Kind`, focused Rust tests, Markdown docs.

---

## Phase 111: Sealed Type-Level Domains

**Status:** 📝 Planned
**Spec:** [SPEC-059](../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)
**Design:** [DESIGN-034](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
**Depends on:** [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)

### Task table

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-806](tasks/TASK-806-spec-c-spec-plan-packet.md) | Promote DESIGN-034 SPEC-C into SPEC-059/PLAN-107 and register Phase 111 | Docs/Planning | 4 | ✅ Complete |
| [TASK-807](tasks/TASK-807-sealed-domain-audit-gate.md) | Audit the live parser/core/engine/typeck sealed-domain boundary and freeze the implementation gate | Docs/Substrate | 4 | 📝 Planned |
| [TASK-808](tasks/TASK-808-parser-surface-for-sealed-type-domains.md) | Add the narrow `sealed type domain` parser surface and explicit rejection boundaries | Parser/Substrate | 5 | 📝 Planned |
| [TASK-809](tasks/TASK-809-core-domain-kind-ids-and-summary-carriers.md) | Add core sealed-domain identities, constructor field metadata, and public summary carriers | Core/Substrate | 6 | 📝 Planned |
| [TASK-810](tasks/TASK-810-domain-lowering-and-summary-versioning.md) | Lower sealed-domain declarations into canonical core metadata and advance summary versioning | Parser/Core | 6 | 📝 Planned |
| [TASK-811](tasks/TASK-811-engine-domain-summary-export-import.md) | Transport public sealed-domain summaries through engine export/import paths | Engine/Substrate | 6 | 📝 Planned |
| [TASK-812](tasks/TASK-812-typeenv-domain-registration-and-validation.md) | Register imported/local domain summaries and validate visibility / anti-leak rules | Type/Substrate | 7 | 📝 Planned |
| [TASK-813](tasks/TASK-813-sealed-domain-diagnostics-and-non-interference.md) | Add diagnostics, negative tests, and non-interference coverage for Phase 111 | Diagnostics/Tests | 6 | 📝 Planned |
| [TASK-814](tasks/TASK-814-spec-c-closeout-docs-and-verification.md) | Reconcile docs/status/changelog and record closeout verification evidence | Docs/Planning | 4 | 📝 Planned |
| [TASK-815](tasks/TASK-815-phase111-review-remediation.md) | Remediate post-closeout review findings for Phase 111 | Review/Hardening | 6 | 📝 Planned |

Estimated total: 54 hours.

## Tracks

### Track A: Spec Gate and Audit

- TASK-806 creates the normative SPEC-C packet.
- TASK-807 audits the live parser/core/engine/typeck substrate before implementation begins and records what must remain deferred.

### Track B: Core Identity and Summary Substrate

- TASK-809 adds the core-owned sealed-domain identities, domain constructor field metadata, public summary carriers, and summary-version evolution contract needed for transport.

### Track C: Parser and Lowering Boundary

- TASK-808 adds the narrow `sealed type domain` surface and owns explicit parser rejection-boundary evidence.
- TASK-810 lowers accepted sealed-domain declarations into core metadata and source anchors, widening the module metadata lowering path without reinterpreting domains as ordinary types.

### Track D: Engine and TypeEnv Registration

- TASK-811 transports visible sealed-domain summaries through module loading and import/export paths.
- TASK-812 consumes those summaries in `TypeEnv`, registering local/imported domain metadata and enforcing visibility and anti-leak rules.

### Track E: Diagnostics and Closeout

- TASK-813 adds negative diagnostics and non-interference coverage.
- TASK-814 reconciles status surfaces and verification evidence.
- TASK-815 reserves the post-review hardening slice.

## Execution Order

Phase 111 is mostly sequential with a narrow parser/core split:

1. TASK-806 first.
2. TASK-807 second; no Rust implementation begins before the audit gate lands.
3. TASK-809 must land before any lowering or import/export work because the rest of the packet depends on core identities, field metadata, and summary carriers.
4. TASK-808 depends on TASK-807 and may proceed in parallel with final review/fix work for TASK-809 once the parser-side syntax contract is frozen.
5. TASK-810 depends on TASK-808 and TASK-809.
6. TASK-811 depends on TASK-809 and TASK-810.
7. TASK-812 depends on TASK-809 and TASK-811.
8. TASK-813 depends on TASK-808 through TASK-812 and cites the parser-boundary evidence owned by TASK-808 rather than replacing it.
9. TASK-814 depends on TASK-813.
10. TASK-815 depends on independent review after TASK-814.

## Implementation Constraints

1. Sealed-domain and marker-constructor identities are owned by `ash-core`, not by parser-private or typechecker-private data structures.
2. Marker constructors must not be modeled as ordinary runtime `TypeDeclId` / `ConstructorId` in a way that claims promoted data kinds already exist.
3. `ash-parser` remains authoritative for the accepted `sealed type domain` surface and explicit rejections; do not invent alternate surface spellings in later tasks.
4. `ash-engine` transport must remain metadata-only; no runtime constructor or pattern semantics are added in this phase.
5. `ash-typeck` owns domain registration, visibility checks, field/domain validation, and structural-status computation.
6. This phase must preserve Phase 109/110 ordinary-type summary/import/export behavior.
7. Constructor visibility inherits domain visibility in this first slice; per-constructor visibility remains deferred.
8. Inline-module sealed-domain declarations remain out of scope and must stay explicitly rejected.
9. No task may implement normalization, public `type fn`, promoted data kinds, mutual recursive domain SCC support, or type-level pattern matching.
10. TASK-808 is the single owner of parser rejection-boundary evidence for Phase 111; later tasks may cite or rerun that suite but must not create a second parser-evidence owner.

## Verification Strategy

Every implementation task must include focused tests for the changed crate and explicit non-regression coverage. The phase-level closeout must verify:

1. sealed `type domain` file-module declarations parse on the accepted subset and explicit rejections remain explicit;
2. `ash-core` exposes canonical sealed-domain and marker-constructor identities plus field metadata;
3. module summaries carrying sealed-domain metadata use the Phase 111 summary-version contract and preserve Phase 109/110 ordinary metadata;
4. `ash-engine` transports public sealed-domain summaries without leaking private domains or fabricating runtime constructors;
5. `TypeEnv` registers imported/local domain metadata, rejects visibility leaks, and validates constructor/domain identity consistency;
6. malformed field domains/kinds, duplicate constructor names, and unsupported recursive structures fail with domain-aware diagnostics;
7. `TypeList` exposes exactly `Nil` and `Cons` to future coverage/equality consumers keyed by domain identity, while unrelated nominal constructors do not match;
8. inline-module sealed-domain declarations remain explicitly unsupported;
9. Phase 109/110 ordinary type, interface, workflow, capability, resource, `do`, and comprehension behavior remains non-regressed;
10. docs/spec index, PLAN-INDEX, task statuses, and CHANGELOG are reconciled honestly and TASK-815 records exact focused/broad verification commands plus any residual-failure classification.

## Decision Gates

- D1: Phase 111 is sealed-domain metadata and validation only; it is not a normalization or public `type fn` packet.
- D2: Marker constructors are type-level only in this phase and do not imply runtime value constructors or promoted data kinds.
- D3: Domain and constructor equality/matching keys are canonical identities, not visible names alone.
- D4: Constructor visibility inherits domain visibility for the first slice; per-constructor visibility remains deferred.
- D5: Structural recursion support is only a validation substrate here; user-facing type-level recursion or pattern matching remains deferred.
- D6: Inline-module sealed-domain declarations remain unsupported until a later packet explicitly widens the lowering/engine boundary.
- D7: Before TypeEnv registration work begins, Phase 111 must already have (a) core-owned sealed-domain identities and summary carriers and (b) parser-owned explicit surface/rejection evidence.

## Completion Checklist

- [ ] SPEC-059 is registered in `docs/spec/README.md`.
- [ ] PLAN-107 and TASK-806 through TASK-815 are registered in `docs/plan/PLAN-INDEX.md`.
- [ ] `ash-core` exposes sealed-domain identities, constructor field metadata, and public summary carriers.
- [ ] A Phase 111 summary version exists and preserves prior ordinary metadata semantics.
- [ ] `ash-parser` accepts the narrow `sealed type domain` subset and rejects deferred shapes explicitly.
- [ ] Module metadata lowering produces canonical sealed-domain metadata with source anchors.
- [ ] `ash-engine` transports visible sealed-domain summaries without private-constructor leakage.
- [ ] `TypeEnv` registers domain metadata and validates visibility / constructor / field-domain consistency.
- [ ] Conservative structural self-domain restrictions are enforced.
- [ ] Existing Phase 109/110 behavior remains non-regressed.
- [ ] Docs/status/changelog are reconciled, TASK-814 records exact verification evidence, and review findings are closed via TASK-815 before the phase is marked fully complete.
