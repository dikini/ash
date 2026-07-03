# PLAN-179: Explicit Row Admission Runtime Wiring

**Status:** ✅ Complete (9/9 tasks complete; planning packet created after Phase 178 closeout)
**Spec:** [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md); [SPEC-098c: Surface-to-Core Lowering](../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md); [SPEC-099b: Target Operational Semantics](../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md); [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md); [SPEC-017: Capability Integration](../spec/SPEC-017-CAPABILITY-INTEGRATION.md); [SPEC-019: Role Runtime Semantics](../spec/SPEC-019-ROLE-RUNTIME-SEMANTICS.md); [SPEC-052: Capability Interfaces and Implementations](../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md); [SPEC-053: Runtime Resources and Authority Provenance](../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md)
**Notes:** [NOTE-009: Capability Interfaces/Implementations/Resources](../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md) *(superseded by the interface/impl/handler model; historical context only)*; [NOTE-020: Computation Row Taxonomy](../notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md); [NOTE-021: Row, Callable, Where, and Fact Syntax](../notes/NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md); [NOTE-022: Effects as Interfaces](../notes/NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md); [NOTE-023: Handler Surface Dispatch](../notes/NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md); [NOTE-025: Effect Identity via Sorts and Impls](../notes/NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)
**Depends on:** [PLAN-178: Source-to-Core Row Lowering Bridge](PLAN-178-SOURCE-TO-CORE-ROW-LOWERING-BRIDGE.md)
**Task range:** TASK-1826 through TASK-1834.

## Goal

Make explicit row requirements preserved by Phase 178 visible to runtime/admission checks without turning row metadata into authority, provider registration, handler installation, or row-polymorphic inference.

## Rationale

Phase 178 proved this bounded path:

```text
source callable row
  -> engine row requirement summary
  -> imported/exported callable metadata
  -> CoreType::Function row metadata
```

It intentionally kept rows authority-neutral. The next useful target-Ash slice is not inference; it is fail-closed runtime/admission interpretation of already explicit row requirements. A workflow or callable may declare that it requires `hostfs.read`, `resource vault write`, `role tenant.admin`, or `policy deployment.approve`; Phase 179 should make admission and runtime checks able to see those requirements and reject missing authority with structured diagnostics, while still requiring real providers, resources, roles, and policies to be registered through existing authority channels. Operation rows are interface/impl-qualified operation identities per NOTE-022/025; the "provider" admission check here refers to an already-registered host/runtime authority, not a deprecated `capability binding`.

## Scope

Phase 179 owns:

- auditing current `WorkflowAdmissionRequest`, provider registry, resource initializer, role, policy, and workflow execution paths against Phase 178 row metadata;
- defining a minimal row requirement admission model for explicit row families already represented in `CallableRowRequirementSummary` and `CoreRow`;
- adding a runtime/admission-facing requirement carrier that can be derived from explicit row metadata without registering authority;
- checking operation rows against existing provider/operation admission data (the provider is an already-registered host/runtime authority, not a deprecated `capability binding`);
- checking resource rows against existing resource initializer/ownership selection data where such data already exists;
- checking role rows against existing role admission data;
- checking policy rows against existing policy/admission paths where such paths already exist, otherwise failing closed with a precise unsupported requirement diagnostic;
- adding tests for local and imported row-bearing callables that prove missing authority rejects and provided authority admits;
- preserving Phase 178 non-authority invariants: parsing/checking row-bearing code must not install providers, resources, runtime modules, roles, policies, handlers, or host hooks.

## Non-goals

- No row-polymorphic inference, solving, defaulting, or implicit row propagation.
- No target handler execution surface or `handle/with` dispatch implementation.
- No new provider/FFI implementation or host adapter surface beyond using existing provider registration APIs.
- No broad stdlib/example migration onto target row syntax.
- No fact/evidence declaration body lowering beyond fail-closed admission checks for already explicit row references.
- No automatic authority grants from rows.

## Decision gates

| Gate | Question | Owner task | Default decision |
|---|---|---|---|
| D1 | Which current runtime/admission paths can consume explicit row requirements without authority leakage? | TASK-1827 | Audit first; do not wire rows directly into registries. |
| D2 | What minimal admission carrier should represent explicit row requirements? | TASK-1828 | Add a derived requirement view beside existing summaries. |
| D3 | How should operation rows map to current provider/operation admission? | TASK-1829 | Require existing provider/operation authority; rows alone do not register providers. |
| D4 | How should resource rows map to current resource ownership/initializer selection? | TASK-1830 | Require existing selected resource authority or fail closed. |
| D5 | How should role/policy rows map to existing role and policy admission? | TASK-1831 | Require explicit admitted role/policy evidence; unsupported paths fail closed. |
| D6 | How should imported row-bearing callable requirements be checked? | TASK-1832 | Imported rows must behave like local rows after metadata transport. |
| D7 | What proves Phase 179 did not regress Phase 178 authority-neutrality? | TASK-1833 | Negative tests around parse/check/import plus missing-authority admission. |
| D8 | What broad verification and review are required before closeout? | TASK-1834 | Full affected runtime/admission gates and independent review. |

## Tasks

| Task | Title | Status |
|---|---|---|
| [TASK-1826](tasks/TASK-1826-phase-179-plan-packet.md) | Create the Phase 179 explicit row admission wiring packet | ✅ Complete |
| [TASK-1827](tasks/TASK-1827-row-admission-runtime-audit.md) | Audit row metadata against admission/runtime authority paths | ✅ Complete |
| [TASK-1828](tasks/TASK-1828-explicit-row-admission-carriers.md) | Add explicit row admission requirement carriers | ✅ Complete |
| [TASK-1829](tasks/TASK-1829-operation-row-provider-admission.md) | Check operation rows against provider/capability admission | ✅ Complete |
| [TASK-1830](tasks/TASK-1830-resource-row-admission.md) | Check resource rows against resource authority | ✅ Complete |
| [TASK-1831](tasks/TASK-1831-role-policy-row-admission.md) | Check role and policy rows against admission authority | ✅ Complete |
| [TASK-1832](tasks/TASK-1832-imported-row-admission.md) | Apply row admission checks across imported callables | ✅ Complete |
| [TASK-1833](tasks/TASK-1833-row-admission-non-authority-regressions.md) | Prove row admission does not install authority | ✅ Complete |
| [TASK-1834](tasks/TASK-1834-phase-179-closeout.md) | Close out Phase 179 with gates and review | ✅ Complete |

## Implementation order

1. TASK-1827 audits current runtime/admission authority seams and records exact owner APIs.
2. TASK-1828 adds a derived admission requirement carrier and tests that it is metadata-only.
3. TASK-1829 wires operation rows to existing provider/capability admission checks.
4. TASK-1830 wires resource rows to existing resource authority checks or explicit fail-closed diagnostics.
5. TASK-1831 wires role/policy rows to existing admission checks or explicit fail-closed diagnostics.
6. TASK-1832 verifies local/imported callable parity.
7. TASK-1833 strengthens non-authority regressions around parse/check/import/execute.
8. TASK-1834 runs broad gates, obtains independent review, fixes findings, and closes the phase.

## Acceptance criteria

- [ ] Current provider, resource, role, policy, workflow admission, and execution seams are audited before implementation.
- [ ] Explicit row metadata can derive admission-facing requirements without registering authority.
- [ ] Missing operation/provider authority rejects with a precise diagnostic.
- [ ] Satisfied operation/provider authority admits through existing provider/capability paths.
- [ ] Resource row requirements are checked against existing resource authority or fail closed as unsupported.
- [ ] Role row requirements are checked against admitted role authority.
- [ ] Policy row requirements are checked against existing policy evidence or fail closed as unsupported.
- [ ] Imported row-bearing callable requirements are checked the same way as local row-bearing callable requirements.
- [ ] Parse/check/import of row-bearing callables remains authority-neutral.
- [ ] Row-polymorphic inference, handler execution, provider registration, and broad corpus migration remain explicitly out of scope.
- [ ] PLAN-INDEX, task files, docs/spec references, and CHANGELOG agree on Phase 179 status.

## Verification baseline

```bash
cargo fmt --check
cargo test -p ash-engine
cargo test -p ash-typeck
cargo test -p ash-core
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

Focused tasks should add narrower commands for each runtime/admission seam. Closeout must run the full baseline unless the task records a user-approved deferral.

## Expected follow-on after Phase 179

If Phase 179 closes cleanly, the next plausible packets are target handler execution, row-polymorphic inference over the now-admissible explicit requirement model, fact/evidence declaration lowering, or stdlib/example corpus migration onto target row syntax.
