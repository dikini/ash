# TASK-1745: Add expansion identity and origin-chain carriers for generated surface nodes

## Status: ✅ Complete

## Description

Make expansion-origin preservation enforceable by adding or tightening a narrow surface-side expansion identity and origin-chain carrier for generated surface nodes. This must support notation/operator-section products now and macro products later, without changing Core provenance APIs.

## Specification Reference

- PLAN-171: `docs/plan/PLAN-171-MACRO-NOTATION-HYGIENE-AND-EXPANSION-BOUNDARIES.md`
- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- Phase 170 closeout: `docs/plan/PLAN-170-EXPANDED-SURFACE-INTEGRATION-AND-NOTATION-SCOPING.md`

## Dependencies

- ✅ TASK-1743: Phase 171 plan packet (complete)
- ✅ TASK-1744: Hygiene, origin, and scope audit

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Core-visible origin provenance | TASK-1741 / PLAN-170 | Core API changes too broad | No | Keep surface-side carrier only | Parser/engine tests inspect surface origin, not Core provenance |
| Full macro expansion IDs | SPEC-095c §6 | Macro execution absent | Partially | Add generic expansion identity usable by future macros | No macro execution tests in this task |

## Requirements

1. Start from live `SurfaceOrigin`/expanded-surface carrier shape found by TASK-1744.
2. Add a stable `ExpansionId` or equivalent only if current carriers cannot distinguish multiple generated nodes from the same source span.
3. Represent origin chains for generated nodes, including at least:
   - source span;
   - notation expansion target or operator token;
   - generated-node expansion id;
   - parent origin when an expansion product is expanded again.
4. Thread origin metadata through existing notation/operator-section elaboration paths.
5. Add focused tests proving origin metadata survives expansion of local notation and binary operator sections.
6. Do not change Core provenance or runtime trace schemas.

## TDD Steps

### Step 1: Write RED origin tests

**Expected file:** `crates/ash-parser/tests/task_1745_expansion_origin_chain.rs` or the nearest existing parser integration-test location.

Test cases:
1. Local notation expansion produces generated nodes with a non-source origin.
2. Operator-section eta expansion carries section span and operator span.
3. Nested expansion preserves an origin chain rather than overwriting the earlier origin.

### Step 2: Implement narrow carrier

**Likely files:**
- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/src/lower.rs`
- expansion/notation helper modules identified by TASK-1744

### Step 3: Preserve compatibility

Existing APIs may keep convenience constructors, but high-level expansion paths must initialize origin metadata explicitly.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1745_expansion_origin_chain -- --nocapture
  - cargo test -p ash-parser
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] RED tests existed before implementation.
  - [x] Origin chains survive notation and operator-section expansion.
  - [x] No Core provenance/runtime schema changes were made.
```

## Completion Evidence

Added `ExpansionId` plus parent-origin sidecars to `ExpandedSurfaceOrigin` in `crates/ash-parser/src/surface.rs`, with expansion traversal assigning stable IDs in origin order and preserving parent origin context for nested expansion products. Added `crates/ash-parser/tests/task_1745_expansion_origin_chain.rs` covering distinct expansion IDs, local notation origin targets, nested notation-to-operator-section origin chains, and parent-origin preservation through non-call recursive shapes such as binary expressions inside generated expansion products. No Core provenance or runtime trace schemas were changed.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 30
toolsets: [terminal, file]
```

## Dependencies for Next Task

Provides generated-node identity data needed by TASK-1746 hygiene fences and TASK-1749 boundary validation.
