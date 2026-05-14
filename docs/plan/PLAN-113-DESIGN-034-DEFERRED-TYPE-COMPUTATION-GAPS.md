# PLAN-113: DESIGN-034 Deferred Type-Computation Gap Ownership

> **For Hermes:** This is a gap-ownership/backlog packet, not an implementation packet for HKT, holes, promoted data constructors, or pattern/exhaustiveness canonicalization. Do not implement the deferred language features without promoting the corresponding future task into a full SPEC/PLAN packet first.

**Goal:** Close the planning and ownership gaps discovered by the DESIGN-034 §16.9 audit after Phases 109 through 116 implemented SPEC-A through SPEC-H.

**Architecture:** The implemented total type-computation substrate now covers ordinary type summaries, canonical IR, sealed domains, normalization/equality, direct structural `type fn`, public computation summaries, associated families, and conservative propositions. The remaining gaps are future language-feature packets with explicit owners, prerequisites, and non-goals rather than implicit TODOs inside historical DESIGN-034 prose.

**Tech Stack:** Documentation/planning, Ash specs/plans/tasks, focused parser/typechecker regression tests only for existing proposition surface hardening.

---

## Phase 117: DESIGN-034 Gap Ownership and Deferred Packet Backlog

**Status:** 📝 Planned / backlog established
**Design:** [DESIGN-034 §16.9](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#169-cross-packet-implementation-gaps-to-plan-explicitly)

### Task table

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-886](tasks/TASK-886-design034-gap-ownership-and-plan106-reconciliation.md) | Reconcile DESIGN-034 §16.9 gap ownership after SPEC-A through SPEC-H and repair PLAN-106 status drift | Docs/Planning | 4 | ✅ Complete |
| [TASK-887](tasks/TASK-887-promoted-data-constructors-and-named-data-kinds-packet.md) | Promote a future packet for promoted data constructors and named data kinds | Future Spec | TBD | ⏸️ Deferred |
| [TASK-888](tasks/TASK-888-type-holes-and-partial-type-constructor-application-packet.md) | Promote a future packet for type holes/wildcards and partial type-constructor application | Future Spec | TBD | ⏸️ Deferred |
| [TASK-889](tasks/TASK-889-constructor-kinded-parameters-and-hkt-packet.md) | Promote a future packet for constructor-kinded parameters, HKT, and user-defined unary computation abstractions | Future Spec | TBD | ⏸️ Deferred |
| [TASK-890](tasks/TASK-890-pattern-exhaustiveness-alias-canonicalization-packet.md) | Audit and plan alias/projection canonicalization rollout into pattern checking and exhaustiveness | Future Audit/Spec | TBD | ⏸️ Deferred |
| [TASK-891](tasks/TASK-891-multi-arg-interface-bound-proposition-regression.md) | Add focused multi-argument interface-bound proposition regression evidence for existing SPEC-H surface breadth | Tests | 2 | ✅ Complete |

## Gap Ownership Matrix

| DESIGN-034 §16.9 gap | Current state after Phase 116 | Owner / next action | Status |
|---|---|---|---|
| Integrated ordinary `type` declarations in ModuleFile/lowering/export | Implemented by SPEC-057 / PLAN-105 | No follow-up required | ✅ Closed |
| Top-level `type fn` parser/surface/core carriers | Implemented by SPEC-061 / PLAN-109 | No follow-up required | ✅ Closed |
| Sealed type-level domains and exported constructor-set metadata | Implemented by SPEC-059 / PLAN-107 | No follow-up required | ✅ Closed |
| Promoted data constructors or named data kinds | Explicitly deferred by SPEC-057/SPEC-059/SPEC-061; marker constructors are not promoted runtime/ADT constructors | TASK-887 creates the future packet | ⏸️ Deferred |
| Internal `TypeFnApp`, neutral/stuck forms, generalized associated-family projections | Implemented as canonical computation-head apps, neutral computation apps, and associated-family projection carriers by SPEC-058/SPEC-060/SPEC-063 | No follow-up required | ✅ Closed |
| Type holes/wildcards in all type-expression positions | Only type-function pattern wildcards exist; no general source `Type::Hole`/wildcard carrier | TASK-888 creates the future packet | ⏸️ Deferred |
| Constructor-kinded interface params such as `M : * -> *` | Core `Kind` can represent arrows, but parser/interface/impl binders do not support kind binders | TASK-889 creates the future packet | ⏸️ Deferred |
| Complete source-level kind checking for variables, constructors, partial applications, arity | Nominal/projection/computation-head arity is implemented; partial application and public kind binders remain deferred | TASK-888/TASK-889 split the remaining surface | Partial / Deferred |
| Generalized interface-application constraints | SPEC-H supports interface-bound proposition carriers and parser/typeck lowering; TASK-891 adds focused multi-arg regression evidence | TASK-891 | ✅ Closed for SPEC-H MVP |
| Canonical associated projection syntax for multi-arg interface families | Implemented by SPEC-063 / PLAN-111 | No follow-up required | ✅ Closed |
| Replacement/canonicalization of stringly `Type::Associated` into projection identities | Implemented at canonical lowering/equality boundaries by SPEC-058 | Pattern/exhaustiveness rollout remains TASK-890 | Partial / Deferred |
| Recursive associated-family normalizer selecting evidence per projection | Implemented by SPEC-063 / TASK-866 | No follow-up required | ✅ Closed |
| Environment-aware definitional equality forcing points | Implemented by SPEC-060 / TASK-826 | No follow-up required | ✅ Closed |
| Recursive associated type-family termination checking | Implemented by SPEC-063 / TASK-865 | No follow-up required | ✅ Closed |
| Module-summary export/import of type functions, sealed domains, associated families | Implemented by SPEC-059/SPEC-062/SPEC-063 | No follow-up required | ✅ Closed |
| Alias canonicalization policy for normalization and pattern matching | Equality/normalization boundaries implemented; pattern/exhaustiveness explicitly out of Phase 110 | TASK-890 creates the future audit/spec packet | ⏸️ Deferred |
| Diagnostics for neutral/stuck normalization, rigid projections, non-exhaustive type functions, non-decreasing recursion, neutral-blocked equality | Implemented across SPEC-060/SPEC-061/SPEC-063/SPEC-064 | No follow-up required | ✅ Closed |
| Do not encode type-function applications as ordinary `Type::Constructor` | Implemented by separate computation-head and neutral-computation carriers | No follow-up required | ✅ Closed |

## Decision Gates

- D1: Do not reinterpret sealed-domain marker constructors as promoted ADT/runtime constructors. TASK-887 must choose the promotion model explicitly.
- D2: Type holes/partial applications are distinct from HKT binders. TASK-888 owns hole terms and partial-application elaboration; TASK-889 owns constructor-kinded binders/interfaces.
- D3: User-defined `Monad<M>` or `do:Result<_, E>` requires both TASK-888 and TASK-889-class substrate; existing Act/Proc dictionaries remain the bridge until then.
- D4: Pattern/exhaustiveness canonicalization must start with an audit of live pattern/exhaustiveness callsites; do not adopt TypeEnv canonicalization by search-and-replace.
- D5: Future packets must add concrete SPEC/PLAN/TASK ranges before Rust implementation starts.

## Verification Strategy

For this backlog packet:

1. PLAN-106 task rows and checklist must agree with completed task files and PLAN-INDEX.
2. DESIGN-034 must retain its historical §16.9 gap list but add a current ownership/status note.
3. PLAN-INDEX must list the future deferred packet owners so gaps are discoverable without rereading the whole design note.
4. Focused multi-argument interface-bound proposition evidence must run non-zero tests.
5. Scoped docs validation must cover new/edited docs and task links.
