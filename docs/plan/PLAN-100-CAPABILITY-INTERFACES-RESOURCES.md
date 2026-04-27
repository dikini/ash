# PLAN-100: Capability Interfaces, Implementations, Resources, and Authority Provenance

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Promote [NOTE-009](../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md) into normative specs and a sequenced implementation program for capability interfaces, Ash-defined capability implementations, runtime resources, and authority provenance.

**Architecture:** This plan splits [NOTE-009](../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md) into two normative specs: [SPEC-052](../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) owns stateless capability interfaces, implementation recipes, and capability bindings; [SPEC-053](../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) owns runtime resources, internal/host/derived authority, split/join policy, and resource lifecycle. Implementation proceeds substrate-first: docs/spec hardening, parser/module metadata, typechecking, runtime binding/resources, then Ash-defined implementations and pilot DX.

**Tech Stack:** Ash parser/typechecker/engine/interpreter/std, Rust 2024, proptest, existing `ash_core::CapabilityProvider`, module-owned capability resolution, Act/Proc/Workflow semantic tower.

---

## Phase 100: Spec Hardening and Ownership Split

**Status:** ✅ Complete in this planning packet.

This phase promotes [NOTE-009](../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md) into [SPEC-052](../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) and [SPEC-053](../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md), then reconciles spec ownership without changing runtime code.

Tasks:

- [TASK-720](tasks/TASK-720-write-spec-052-capability-interface-implementation-contract.md): Write [SPEC-052](../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) capability interface/implementation contract.
- [TASK-721](tasks/TASK-721-write-spec-053-runtime-resources-authority-provenance.md): Write [SPEC-053](../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) runtime resource and authority provenance contract.
- [TASK-722](tasks/TASK-722-reconcile-capability-resource-spec-ownership.md): Reconcile adjacent specs and docs with the [SPEC-052](../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)/[SPEC-053](../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) ownership split.
- [TASK-723](tasks/TASK-723-phase-100-closeout-audit.md): Phase 100 closeout audit.

## Phase 101: Parser, Surface AST, and Module Metadata

**Status:** ✅ Complete.

Add syntax carriers for `capability interface`, `capability impl`, `resource type`, resource/binding clauses, and module export/import metadata. This phase does not execute capability implementation bodies.

Tasks:

- [TASK-724](tasks/TASK-724-capability-interface-ast-parser-substrate.md): Add capability interface AST/parser substrate.
- [TASK-725](tasks/TASK-725-capability-implementation-ast-parser-substrate.md): Add capability implementation AST/parser substrate.
- [TASK-726](tasks/TASK-726-resource-type-and-binding-clause-parser-substrate.md): Add resource type and binding clause AST/parser substrate.
- [TASK-727](tasks/TASK-727-module-metadata-for-capability-resource-definitions.md): Export/import interface, implementation, and resource metadata through modules.
- [TASK-728](tasks/TASK-728-parser-module-conformance-tests-and-docs.md): Parser/module conformance tests and docs.

## Phase 102: Static Semantics and Binding-Time Type Contracts

**Status:** 📝 Planned.

Type-check interface operation shapes, implementation conformance, resource requirements, provenance source declarations, and module-owned capability binding resolution.

Tasks:

- [TASK-729](tasks/TASK-729-capability-interface-operation-signature-environments.md): Add capability interface operation signature environments.
- [TASK-730](tasks/TASK-730-capability-implementation-conformance-checking.md): Add capability implementation conformance checking.
- [TASK-731](tasks/TASK-731-resource-type-and-binding-typechecking.md): Add resource type and binding type checking.
- [TASK-732](tasks/TASK-732-authority-provenance-static-validation.md): Add authority provenance static validation.
- [TASK-733](tasks/TASK-733-module-owned-capability-binding-resolution.md): Add module-owned capability binding resolution.
- [TASK-734](tasks/TASK-734-typechecker-integration-and-negative-tests.md): Typechecker integration and negative tests.

## Phase 103: Runtime Resource and Binding Substrate

**Status:** 📝 Planned.

Introduce runtime resource instance carriers, capability binding admission, internal authority allocation, derived-authority non-widening checks, and Proc resource split/join policy enforcement.

Tasks:

- [TASK-735](tasks/TASK-735-runtime-resource-instance-carriers.md): Add runtime resource instance carriers.
- [TASK-736](tasks/TASK-736-capability-binding-admission-api.md): Add capability binding admission API.
- [TASK-737](tasks/TASK-737-internal-authority-allocation-and-resource-admission.md): Add internal authority allocation and resource admission.
- [TASK-738](tasks/TASK-738-derived-authority-non-widening-runtime-checks.md): Add derived-authority non-widening runtime checks.
- [TASK-739](tasks/TASK-739-proc-resource-split-join-policy-enforcement.md): Add Proc split/join resource policy enforcement.
- [TASK-740](tasks/TASK-740-runtime-resource-binding-integration-tests.md): Runtime integration tests for resources and bindings.

## Phase 104: Ash-Defined Capability Implementations and Pilot DX

**Status:** 📝 Planned.

Execute Ash-defined capability implementation bodies and prove the model with mock/replay/internal-resource pilots.

Tasks:

- [TASK-741](tasks/TASK-741-execute-ash-defined-capability-implementation-bodies.md): Execute Ash-defined capability implementation bodies.
- [TASK-742](tasks/TASK-742-adapter-mock-replay-capability-examples.md): Add adapter, mock, and replay implementation examples.
- [TASK-743](tasks/TASK-743-cli-engine-capability-binding-configuration-surface.md): Add CLI/engine binding configuration surface.
- [TASK-744](tasks/TASK-744-standard-internal-kv-and-test-clock-pilots.md): Add standard internal KV and test-clock pilot resources.
- [TASK-745](tasks/TASK-745-capability-resource-final-docs-examples-verification.md): Final docs, examples, and verification closeout.

## Dependencies

```text
Phase 100 (Specs)
  -> Phase 101 (Parser/module metadata)
      -> Phase 102 (Static semantics)
          -> Phase 103 (Runtime resources/bindings)
              -> Phase 104 (Ash-defined impl execution and DX pilots)
```

## Non-goals

1. No persistence/checkpointing implementation in this plan.
2. No first-class value-level `ResourceRef<T>` in the first slice.
3. No dynamic rebinding after workflow/process admission.
4. No implicit manufacture of external authority.
5. No rewrite of all existing stdlib capabilities before the pilot proves the model.

## Verification Strategy

Each implementation phase must end with:

1. focused parser/type/runtime tests for its new substrate;
2. negative tests for authority widening and ambient leakage;
3. `cargo fmt --check`;
4. `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
5. `cargo test --workspace`;
6. independent subagent verification before changing phase status to complete.
