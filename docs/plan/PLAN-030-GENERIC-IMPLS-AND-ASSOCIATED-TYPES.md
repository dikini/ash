# PLAN-030: Multi-Parameter Interfaces, Generic Implementations, and Associated Types

Remove the single type-parameter restriction on interfaces, enable generic `impl` blocks with `where` bounds, and add associated types on interfaces. This is the final extension needed for ergonomic generic libraries (serde-style serialization, besedarium-style query building).

**Specs:** SPEC-033, SPEC-034, SPEC-035
**Phase:** 83
**Priority:** High
**Status:** 📝 Planned

## Overview

This plan builds on Phase 82 (multi-parameter methods) to complete the interface system:

1. **Multi-parameter interfaces (SPEC-033):** Interfaces may declare any number of type parameters (e.g., `Map<K, V>`). The impl registry is redesigned to use full interface applications (`Map<String, Int>`) as keys.
2. **Generic impls (SPEC-034):** `impl<T> Interface<Head> where T: Interface` syntax. The registry migrates from `HashMap` to a scheme-based `Vec<ImplScheme>` with ordered search, overlap checking, and recursive bound resolution.
3. **Associated types (SPEC-035):** `type Name` inside interfaces, `type Name = TypeExpr` inside impls, and projection syntax (`S::Ok`) with normalization during resolution.
4. **Engine monomorphization:** A new post-typecheck lowering pass instantiates generic impl bodies and substitutes associated types at call sites.

## Prerequisites

- Phase 82 (TASK-561, TASK-562) must be complete.

## Tasks

| Task | Description | Spec | Est. Hours | Status | Dependencies |
|------|-------------|------|------------|--------|--------------|
| [TASK-563](tasks/TASK-563-typeck-multi-param-interfaces.md) | Type checker: multi-parameter interfaces and impl registry redesign | SPEC-033 §5 | 4 | ✅ Complete | Phase 82 |
| [TASK-564](tasks/TASK-564-parser-generic-impls-and-associated-types.md) | Parser/AST: generic impl syntax, `where` bounds, associated types | SPEC-034 §4, SPEC-035 §4 | 5 | 📝 Planned | TASK-563 |
| [TASK-565](tasks/TASK-565-typeck-generic-impl-schemes.md) | Type checker: impl schemes, overlap checking, recursive resolution | SPEC-034 §5 | 6 | 📝 Planned | TASK-564 |
| [TASK-566](tasks/TASK-566-engine-monomorphization.md) | Engine: post-typecheck monomorphization pass for generic impls | SPEC-034 §6 | 6 | 📝 Planned | TASK-565 |
| [TASK-567](tasks/TASK-567-typeck-associated-types.md) | Type checker: `Type::Associated`, normalization, rigid projections | SPEC-035 §5 | 6 | 📝 Planned | TASK-565 |
| [TASK-568](tasks/TASK-568-engine-associated-type-substitution.md) | Engine: associated type substitution in monomorphized bodies | SPEC-035 §6 | 3 | 📝 Planned | TASK-566, TASK-567 |

**Total Estimate:** 30 hours (25 hours remaining after TASK-563)

## Execution Order

Phase 83 tasks must execute **sequentially**, not concurrently:

1. **TASK-563** ✅ Complete → establishes multi-parameter interface support and the full-interface-application registry key model.
2. **TASK-564** → blocked until TASK-563 is complete. Adds parser/AST support for generic impl syntax and associated types.
3. **TASK-565** → blocked until TASK-564 is complete. Rewrites the impl registry from `HashMap` to `Vec<ImplScheme>`; this is the highest-risk structural change.
4. **TASK-566 + TASK-567** → blocked until TASK-565 is complete. Can proceed in parallel once the scheme registry is stable:
   - TASK-566: engine monomorphization
   - TASK-567: associated type normalization
5. **TASK-568** → blocked until both TASK-566 and TASK-567 are complete. Final integration of associated-type substitution into monomorphized bodies.

## Deliverable

- Interfaces accept any number of type parameters.
- Generic impls with `where` bounds compile and resolve recursively.
- Overlapping impl schemes are rejected at registration time.
- Interface calls return fully monomorphized, concrete method bodies.
- Associated types (`S::Ok`) normalize to concrete types after impl selection.
- Rigid projections (`fn<T: Serializer>(s: T) -> T::Ok`) type-check in generic code.
- `Type::Associated` never appears at runtime.
