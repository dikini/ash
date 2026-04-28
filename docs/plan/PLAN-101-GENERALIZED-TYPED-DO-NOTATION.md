# PLAN-101: Generalized Typed Do-Notation

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Phase 104 is active/in-flight; do not start Phase 105 implementation until Phase 104 is complete or the user explicitly authorizes isolated non-interfering work.

**Goal:** Implement [SPEC-054](../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) by replacing Act-specific block sequencing with explicit typed `do:K` notation for `Act` and `Proc`, while preserving Phase 104 capability/resource semantics.

**Architecture:** Phase 105 is substrate-first. It introduces a surface `DoBlock` carrier, target/kind/dictionary resolution, typed elaboration, and migration compatibility for `act { ... }`. The MVP uses compiler-known Act/Proc dictionaries shaped like future `Monad<K>` evidence; full user-defined constructor-kinded `Monad` support is deferred.

**Tech Stack:** Ash parser/typechecker/lowerer/interpreter/std, Rust 2024, proptest, existing `Act` and `Proc` type constructors, `std::act`, `std::proc`, Phase 97/98/99 semantic tower substrate, Phase 104 capability/resource substrate.

---

## Phase 105: Generalized Typed Do-Notation

**Status:** 🟢 In Progress (TASK-747 parser/surface substrate, TASK-748 target/dictionary resolution, and TASK-749 typed elaboration complete; TASK-750 Act compatibility next)
**Spec:** [SPEC-054](../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
**Design:** [DESIGN-031](../design/DESIGN-031-GENERALIZED-DO-NOTATION.md)
**Depends on:** Phase 104 closeout for normal implementation sequencing; Phase 97 `Act`, Phase 98 `Proc`, Phase 99 `proc::from_act`.

### Task table

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-746](tasks/TASK-746-generalized-do-notation-spec-plan-packet.md) | Promote DESIGN-031 into SPEC-054/PLAN-101 and register Phase 105 | Docs/Planning | 3 | ✅ Complete |
| [TASK-747](tasks/TASK-747-do-block-surface-ast-and-parser-substrate.md) | Add `DoBlock` surface AST and parser substrate for `do:K` | Substrate | 6 | ✅ Complete |
| [TASK-748](tasks/TASK-748-do-target-kinding-and-dictionary-resolution.md) | Resolve do targets, target kind, and Act/Proc builtin dictionaries | Substrate | 7 | ✅ Complete |
| [TASK-749](tasks/TASK-749-typed-do-elaboration-and-lowering.md) | Type-check `let`/`<-`/`return` and lower after typed elaboration | Semantic | 8 | ✅ Complete |
| [TASK-750](tasks/TASK-750-act-block-compatibility-and-migration.md) | Route `act {}` through generalized do compatibility and legacy warnings | Semantic | 6 | 📝 Planned |
| [TASK-751](tasks/TASK-751-proc-do-integration-and-tower-behavior.md) | Validate `do:Proc`, explicit `proc::from_act`, and tower/failure behavior | Semantic | 7 | 📝 Planned |
| [TASK-752](tasks/TASK-752-do-notation-diagnostics.md) | Add focused do-notation diagnostics and migration warnings | Semantic | 5 | 📝 Planned |
| [TASK-753](tasks/TASK-753-do-notation-docs-examples-closeout.md) | Update docs/examples and perform Phase 105 closeout audit | Docs/Planning | 4 | 📝 Planned |

Estimated total: 46 hours.

## Tracks

### Track A: Spec and Surface Substrate

- TASK-746 establishes the normative packet and Phase 105 traceability.
- TASK-747 adds parser/surface carriers without prematurely lowering.

### Track B: Type-System Elaboration

- TASK-748 resolves target constructors and dictionaries.
- TASK-749 performs typed statement checking and elaboration.

### Track C: Compatibility and Tower Integration

- TASK-750 migrates `act {}` compatibility.
- TASK-751 validates `do:Proc`, explicit Act-to-Proc lifting, and operational-bottom behavior.
- TASK-752 hardens user-facing diagnostics.

### Track D: Closeout

- TASK-753 updates docs/examples/changelog, runs full verification, and requests independent review.

## Scheduling and Phase 104 Coordination

Phase 104 owns execution of Ash-defined capability implementations, adapter/mock/replay examples, CLI binding configuration, internal KV/test-clock pilots, and capability/resource final docs. Phase 105 must not change those semantics while Phase 104 is in flight.

Normal sequencing:

```text
Phase 104 (TASK-741..TASK-745)
  -> Phase 105 (TASK-747..TASK-753)
```

Allowed before Phase 104 closeout only with explicit user authorization:

- docs/spec review;
- parser-only experiments in a separate worktree;
- no changes to capability implementation execution, runtime authority admission, CLI binding configuration, or resource split/join policy.

## Implementation Constraints

1. Preserve `DoBlock` through parsing; do not lower to `unit`/`bind` before type checking.
2. MVP targets are `Act` and `Proc` only.
3. Shape dictionaries as future `Monad<K>` evidence, even if implemented in Rust.
4. Do not implicitly lift `Act<A>` into `Proc<A>`; require `proc::from_act`.
5. Do not treat operational `fail` as domain failure.
6. Keep target-specific operations ordinary scoped names; `do:Proc` does not import `proc::par`.
7. Keep source spans on targets, binders, statements, and final return for diagnostics.
8. Keep legacy `act { x = ...; ret ...; }` compatibility only until migration warnings and examples have landed.

## Verification Strategy

Every implementation task must include:

1. focused parser/typechecker/lowering tests for its changed layer;
2. negative tests for target kind, wrong bind RHS constructor, missing final return, and no implicit lifts where applicable;
3. regression tests proving existing Phase 97/98/99 Act/Proc examples still pass;
4. Phase 104 non-interference checks when touching interpreter/engine paths;
5. `cargo fmt --check`;
6. `cargo test --workspace`;
7. `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
8. independent subagent verification before marking a task complete.

## Deferred Follow-on Phase Candidates

A later Phase 106+ should own full user-definable monad support:

- constructor-kinded interface parameters (`M : * -> *`);
- application of constructor parameters in type expressions (`M<A>`);
- `impl Monad<Act>` and user-defined constructors;
- `do:Result<_, E>` and pure `do:Option`/`do:List` targets;
- law declarations and generated property tests.
