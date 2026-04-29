# TASK-769: Workflow Carrier and Coverage Evidence

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [DESIGN-033](../../design/DESIGN-033-WORKFLOW-CONTRACT-OPERATOR-LIFTING.md)
- [SPEC-051](../../spec/SPEC-051-WORKFLOW-SEMANTICS.md)
- [SPEC-053](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md)

## Objective

Add internal semantic carriers for first-class workflows: contract, admission envelope, contract plan, coverage evidence, and reconciliation.

## Requirements

1. Add internal Rust carriers for `WorkflowContract`, `AdmissionEnvelope`, `ContractPlan`, `CoverageEvidence`, and `CoverageError` in the appropriate core/typecheck/runtime boundary crates.
2. Represent `HeaderContract`, `BodyContract`, `TotalContract`, and `Reconcile(C_header, C_body, E_coverage)` explicitly enough for downstream tasks.
3. Implement empty/default evidence for `workflow::unit` and sequential composition evidence for `workflow::bind` scaffolding.
4. Preserve authority/resource/failure/provenance evidence from existing substrate when available.
5. Add component-specific coverage errors for at least authority, resources, failure, provenance, and opaque summaries.
6. Do not expose contract parameters in the public `Workflow<A>` type.

## TDD Steps

1. Write failing unit tests for empty coverage evidence and rejected authority/resource coverage gaps.
2. Write failing tests proving `Reconcile` preserves header/body/evidence identities.
3. Implement minimal carriers and constructors.
4. Run focused tests and affected-crate `cargo check`.
5. Request independent verification before marking complete.

## Verification

- [ ] Coverage carriers compile.
- [ ] Empty evidence works for pure `Workflow` values.
- [ ] Coverage failures produce structured errors.
- [ ] Reconciliation preserves evidence identities.
- [ ] No public `Workflow<C, A>` surface is introduced.
- [ ] CHANGELOG.md updated.
