# PLAN-100: Capability Interfaces, Implementations, Resources, and Authority Provenance

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Promote NOTE-009 into normative specs and a sequenced implementation program for capability interfaces, Ash-defined capability implementations, runtime resources, and authority provenance.

**Architecture:** This plan splits NOTE-009 into two normative specs: SPEC-052 owns stateless capability interfaces, implementation recipes, and capability bindings; SPEC-053 owns runtime resources, internal/host/derived authority, split/join policy, and resource lifecycle. Implementation proceeds substrate-first: docs/spec hardening, parser/module metadata, typechecking, runtime binding/resources, then Ash-defined implementations and pilot DX.

**Tech Stack:** Ash parser/typechecker/engine/interpreter/std, Rust 2024, proptest, existing `ash_core::CapabilityProvider`, module-owned capability resolution, Act/Proc/Workflow semantic tower.

---

## Phase 100: Spec Hardening and Ownership Split

**Status:** ✅ Complete in this planning packet.

This phase promotes NOTE-009 into SPEC-052 and SPEC-053, then reconciles spec ownership without changing runtime code.

Tasks:

- TASK-720: Write SPEC-052 capability interface/implementation contract.
- TASK-721: Write SPEC-053 runtime resource and authority provenance contract.
- TASK-722: Reconcile adjacent specs and docs with the SPEC-052/SPEC-053 ownership split.
- TASK-723: Phase 100 closeout audit.

## Phase 101: Parser, Surface AST, and Module Metadata

**Status:** 📝 Planned.

Add syntax carriers for `capability interface`, `capability impl`, `resource type`, resource/binding clauses, and module export/import metadata. This phase does not execute capability implementation bodies.

Tasks:

- TASK-724: Add capability interface AST/parser substrate.
- TASK-725: Add capability implementation AST/parser substrate.
- TASK-726: Add resource type and binding clause AST/parser substrate.
- TASK-727: Export/import interface, implementation, and resource metadata through modules.
- TASK-728: Parser/module conformance tests and docs.

## Phase 102: Static Semantics and Binding-Time Type Contracts

**Status:** 📝 Planned.

Type-check interface operation shapes, implementation conformance, resource requirements, provenance source declarations, and module-owned capability binding resolution.

Tasks:

- TASK-729: Add capability interface operation signature environments.
- TASK-730: Add capability implementation conformance checking.
- TASK-731: Add resource type and binding type checking.
- TASK-732: Add authority provenance static validation.
- TASK-733: Add module-owned capability binding resolution.
- TASK-734: Typechecker integration and negative tests.

## Phase 103: Runtime Resource and Binding Substrate

**Status:** 📝 Planned.

Introduce runtime resource instance carriers, capability binding admission, internal authority allocation, derived-authority non-widening checks, and Proc resource split/join policy enforcement.

Tasks:

- TASK-735: Add runtime resource instance carriers.
- TASK-736: Add capability binding admission API.
- TASK-737: Add internal authority allocation and resource admission.
- TASK-738: Add derived-authority non-widening runtime checks.
- TASK-739: Add Proc split/join resource policy enforcement.
- TASK-740: Runtime integration tests for resources and bindings.

## Phase 104: Ash-Defined Capability Implementations and Pilot DX

**Status:** 📝 Planned.

Execute Ash-defined capability implementation bodies and prove the model with mock/replay/internal-resource pilots.

Tasks:

- TASK-741: Execute Ash-defined capability implementation bodies.
- TASK-742: Add adapter, mock, and replay implementation examples.
- TASK-743: Add CLI/engine binding configuration surface.
- TASK-744: Add standard internal KV and test-clock pilot resources.
- TASK-745: Final docs, examples, and verification closeout.

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
