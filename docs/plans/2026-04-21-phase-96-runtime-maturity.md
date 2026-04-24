# Phase 96: Runtime Maturity — Multi-File Execution, Stdlib Loading, Capability Surface

## Goal

Close the four critical gaps between "Ash can execute single-file workflows" and "Ash can execute real programs":

1. **G1 — Multi-file module resolution**: Make `ash run` execute workflows that import from sibling modules, user library trees, and the stdlib through a single graph-backed resolver.
2. **G2 — Stdlib loading**: Make the full stdlib available to every workflow (not just the 4-module entry subset).
3. **G3 — Builtin surface completion**: Implement Rust handlers for declared-but-unimplemented stdlib builtins (IO, runtime, LLM stdlib).
4. **G4 — Capability provider surface**: Catalog, prioritize, and implement the next tier of capability providers (HTTP, time, process improvements).

## Motivation

The previous 95 phases built a complete compiler pipeline (parse → check → execute), a mature type system, a capability/security model, LSP/MCP tooling, and a rich stdlib declaration surface. But the runtime can only execute single-file workflows with ~30 builtins. Every real program needs multi-file imports and stdlib access. This phase closes that gap.

## Relationship to Existing Work

This phase **executes** three existing plans that were written but never implemented:

| Existing Plan | Tracks |
|---|---|
| `2026-04-08-module-resolution-stdlib-plan.md` | Track A (resolver, graph loader, CLI integration) |
| `2026-04-03-task-363a-stdlib-loading-plan.md` | Track B (engine stdlib registry) |
| `2026-04-01-task-363-entry-bootstrap-plan.md` | Track A (entry semantics preservation) |

It adds two new tracks for capability surface work that has no prior plan.

## Architecture

```
Track A: Module Resolution Engine (depends on nothing)
  ├── A1: Graph-backed module resolver in ash-engine
  ├── A2: Stdlib as resolver root (full 39 modules)
  ├── A3: CLI integration (ash run <file> uses graph loader)
  └── A4: Entry bootstrap on same loader

Track B: Stdlib Builtin Completion (depends on nothing)
  ├── B1: Audit declared vs implemented builtins
  ├── B2: Implement std::io builtins (path, fs, dir, stdio)
  ├── B3: Implement std::runtime builtins (error, args, supervisor)
  └── B4: Implement std::llm builtins (types, dispatch, tool_agent)

Track C: Capability Provider Surface (depends on nothing)
  ├── C1: HTTP provider (reqwest-backed)
  ├── C2: Time/timer provider
  └── C3: Process provider hardening (sandboxing, constraints)

Track D: Integration & Verification (depends on A + B)
  ├── D1: End-to-end multi-file workflow execution tests
  ├── D2: Capability boundary audit
  └── D3: Performance baseline and regression suite
```

## Decision Gates

### D1: Module resolver architecture — RESOLVED (Tier 1)

**Resolution:** Keep in ash-engine. 21 source files / 7376 lines is moderate. Resolver adds ~3-4 files. Tightly coupled to engine's parse→check→execute pipeline.

**Blocks:** A1, A2, A3

### D2: Stdlib builtin dispatch strategy — RESOLVED (Tier 1)

**Question:** How should new stdlib builtins be dispatched?

**Resolution:** Extend the **existing** `builtin_dispatch_table()` + `dispatch_builtin()` + `eval_function_call()` pattern in eval.rs.

The infrastructure already exists: a `HashMap<&'static str, BuiltinEntry>` table (lines 32-286), arity checking, `UnimplementedBuiltin` support, and `eval_function_call()` match arms (61 patterns, 3322 lines total). Adding ~30 new builtins means ~300 lines of table entries + match arms. When eval.rs exceeds ~4000 lines, extract handler functions into `crates/ash-interp/src/builtins/` as plain functions (not a trait), called from the match arms.

**Key design issue:** IO builtins need access to CapabilityProviders (StdioProvider, FsProvider). eval.rs currently has no provider access. Resolution: thread an optional provider registry reference through Context, populated during engine execution. This is minimal and aligns with RuntimeState's existing provider management.

See: `docs/plans/2026-04-21-phase-96-decision-gate-resolutions.md`

### D3: Capability constraint model for new providers — RESOLVED (Tier 1)

**Resolution:** Per-provider config structs (follow FsConfig pattern). Each provider has its own XxxConfig struct. Providers also interpret `Constraint` predicate values in `observe()` calls per their domain.

**Blocks:** C1, C2, C3

### D4: Stdlib loading scope — RESOLVED (Tier 1)

**Resolution:** On-demand (resolve-and-load via graph resolver). When a `use std::json` is encountered, the resolver loads `std/src/json.ash`, registers its types and builtins. Stdlib total is only 65KB / 2128 lines — no performance concern. Namespace hygiene: users only see types from modules they imported.

**Blocks:** A2, B1

## Predicted Tracks and Timing

| Track | Tasks | Est. Hours | Dependency | Tier |
|---|---|---|---|---|
| **A: Module Resolution** | 6 | 28-40 | None | T0/T1 |
| **B: Stdlib Builtins** | 5 (+1 optional) | 16-26 | None | T0/T1 |
| **C: Capability Providers** | 3 | 13-18 | None | T0/T1 |
| **D: Integration** | 3 | 10-14 | A + B complete | T1 |

**Total estimate:** 67-98 hours across 17-18 tasks.

All four decision gates (D1-D4) are resolved. Tracks A, B, and C can proceed in parallel immediately. Track D gates on A+B complete.

## Parallel Execution Plan

```
Week 1: All gates resolved. Begin execution immediately.
         ├── Track A: TASK-654 (failing tests) → TASK-655 (resolver core)
         ├── Track B: TASK-660 (audit) → TASK-661 (IO builtins)
         └── Track C: TASK-666 (HTTP provider)

Week 2-3: Continue parallel tracks
         ├── Track A: TASK-656 (stdlib root) → TASK-657 (engine threading) → TASK-658+659 (CLI + entry)
         ├── Track B: TASK-662 (runtime) + TASK-664 (lists) in parallel, then TASK-663 (LLM)
         └── Track C: TASK-667 (time) + TASK-668 (process) in parallel

Week 4: Track D integration
         ├── TASK-669 (multi-file e2e tests)
         ├── TASK-670 (capability audit)
         └── TASK-671 (perf baseline)
```

## Deliverable

After Phase 96:
- `ash run multi_file/main.ash` resolves imports from sibling files and the stdlib
- `ash run entry.ash` uses the same resolver, preserving entry semantics
- ~70+ stdlib builtins have Rust implementations (up from ~30)
- HTTP, time, and process capability providers are available
- Integration test suite covers multi-file execution paths

## Specs Affected

- SPEC-009 (Module System) — no amendment, execution catches up to spec
- SPEC-010 (Embedding API) — minor: Engine gains resolver configuration
- SPEC-012 (Imports) — no amendment, execution catches up to spec
- SPEC-030 (Module Type Resolution) — no amendment, build on existing
- SPEC-005 (CLI) — minor: `ash run` gains import resolution behavior

## Non-Goals

- Package manager / dependency solver
- Remote module fetching
- Bytecode compilation or caching
- Incremental compilation
- Concurrency model within workflows
