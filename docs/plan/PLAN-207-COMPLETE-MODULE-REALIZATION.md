---
id: plan.207.complete-module-realization
title: Complete Module Realization
kind: plan
status: planned
authority: planning
owner: language-semantics
last_verified: 2026-08-02
---

# PLAN-207: Complete Module Realization

## Purpose

Implement [SPEC-103](../spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md): one complete, language-level module route for both `mod name;` and `mod name { ... }`.

```text
Surface ModuleFile
  -> AST-driven graph and source acquisition
  -> expanded module units
  -> checked export-closed interfaces
  -> resolved imports and visibility
  -> Core modules
  -> CPS modules
  -> admitted Engine artifact
  -> CLI/daemon terminal parity
```

A file-backed child and an inline child differ only before source acquisition. After that point they must have equal module semantics for equal declarations.

## Baseline and authority

- [SPEC-103](../spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md) owns the target module rule.
- [AUDIT-207](audits/AUDIT-207-module-realization-seams.md) records the current split parser/resolver/summary/import/Engine seams.
- [PLAN-203](PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md) owns integration of the one Surface → Core → CPS → Engine route and client parity. This phase supplies module artifacts to that route; it does not create another evaluator.
- [SPEC-095b](../spec/SPEC-095b-TARGET-GRAMMAR.md), [SPEC-095c](../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md), [SPEC-097b](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md), [SPEC-098b](../spec/SPEC-098b-TARGET-IR.md), [SPEC-098c](../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md), [SPEC-099b](../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md), and [SPEC-099c](../spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md) remain the grammar, syntax-phase, type, IR, lowering, and Engine operational owners.
- [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md) and [SPEC-062](../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md) remain the implemented bounded summary substrate. This phase extends their transport compatibly; it does not recreate their type identities, closure rules, versioning, or import-order semantics.
- [PLAN-206](PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md) and its audit provide current-state limitation evidence only. They do not authorize target semantics.

## Completion contract

The phase is complete only when:

1. every structural module edge originates in a parsed `ModuleFile` declaration;
2. no normal semantic path scans raw text for a module declaration, ordinary declaration, import, export, or visibility fact after parsing;
3. file-backed and inline modules with equivalent declarations have equivalent normalized interfaces and Core/CPS artifacts;
4. imports and visibility use stable declaration identities and checked interfaces;
5. incomplete modules cannot publish partial public interfaces;
6. reachable entry modules lower and execute only through the Engine-owned checked Core/CPS route; and
7. one multi-module admitted program has CLI/daemon normalized-terminal parity.

## Scope

### In scope

- Stable crate-qualified module paths, graph identities, source origins, structural graph construction, and source-acquisition diagnostics.
- AST-driven replacement or strict quarantine of text scanning at all semantic module seams.
- Common file-backed/inline module-unit construction.
- Versioned export-closed interfaces, public/private views, and identity-preserving re-exports.
- Interface-based imports, qualified resolution, and all parsed visibility forms.
- Module-aware source-to-Core, Core-to-CPS, Engine linking, admission, and selected entry execution.
- Positive, negative, mutation, parity, and diagnostics evidence.

### Non-goals

- New lexical module/import spelling, package, registry, dynamic-import, hot-reload, or
  runtime-module syntax. The existing `use` and nested `mod` item forms become permitted in an
  inline module so it has the same item domain as a file module.
- Dynamic runtime module values or automatic module initialization.
- Import-cycle initialization or cross-module recursive initialization. Structural and import cycles reject in this phase.
- A full incremental workspace/LSP database.
- Macro runtime callability, new macro hygiene semantics, or imported-notation activation beyond existing syntax-phase contracts.
- Any direct evaluator, Engine bypass, or authority inferred from a module interface.

## Decision gates

No user decision remains open for the initial realization. The phase fixes these conservative rules:

| Gate | Decision | Impact |
|---|---|---|
| D1 | `mod` structural cycles reject; import cycles also reject in the initial implementation | Prevents unspecified initialization and partial interface publication |
| D2 | `ModuleFile` AST is authoritative; text scans are removed or fenced as non-authorizing migration checks | Eliminates parser/resolver/Engine disagreement |
| D3 | File-backed and inline modules share one module-unit and interface pipeline after source acquisition | Makes parity an executable invariant, not prose |
| D4 | A module is a compile/link namespace, not a runtime value; entry execution remains Engine-owned | Preserves PLAN-203's one-executor contract |

D1-D4 are target-contract decisions in SPEC-103, not implementation discretion. Any change requires a SPEC-103 amendment before code changes.

## Workstreams and task order

```text
Track A — semantic substrate
  TASK-2057 AST-driven discovery ─┐
  TASK-2058 stable graph/artifacts ├─> TASK-2059 common source acquisition
                                   │
Track B — interface semantics      ├─> TASK-2060 checked interfaces
                                   └─> TASK-2061 imports and visibility

Track C — realization
  TASK-2060 + TASK-2061 -> TASK-2062 module lowering -> TASK-2063 Engine linking

Track D — evidence
  TASK-2059 + TASK-2061 + TASK-2063 -> TASK-2064 conformance and parity -> TASK-2065 closeout
```

TASK-2057 and TASK-2058 may proceed in parallel after the packet. TASK-2059 consumes both.
TASK-2060 finalizes the interface shape before TASK-2061 starts; TASK-2061 then consumes that
shape to bind imports and visibility. TASK-2062, TASK-2063, TASK-2064, and TASK-2065 are ordered.

## Semantic-rule ownership

| Rule | Type | Core | CPS | Admission/runtime | Integration owner |
|---|---|---|---|---|---|
| MOD-REAL-001 AST graph identity | TASK-2057/2058 | non-authorizing | not applicable | not applicable | TASK-2064 |
| MOD-REAL-002 file/inline parity | TASK-2059 | TASK-2062 | TASK-2062 | TASK-2063 | TASK-2064 |
| MOD-REAL-003 checked interfaces | TASK-2060 | interface metadata | non-authorizing | non-authorizing | TASK-2064 |
| MOD-REAL-004 import/visibility | TASK-2061 | resolved identity metadata | non-authorizing | non-authorizing | TASK-2064 |
| MOD-REAL-005 module lowering | consumes checked facts | TASK-2062 | TASK-2062 | prerequisite | TASK-2063 |
| MOD-REAL-006 linked execution | consumes checked facts | consumes Core/CPS | consumes Core/CPS | TASK-2063 | TASK-2064 |

## Tasks

| Task | Title | Status | Run-route impact |
|---|---|---|---|
| [TASK-2056](tasks/TASK-2056-module-realization-spec-plan-packet.md) | Create the module realization spec, seam audit, plan, and task packet | Planned — packet authored and verified; implementation activation remains pending | none |
| [TASK-2057](tasks/TASK-2057-ast-driven-module-discovery.md) | Replace semantic module-declaration text scans with AST-driven discovery | Planned | prerequisite |
| [TASK-2058](tasks/TASK-2058-canonical-module-identity-and-artifacts.md) | Establish canonical module identities and module-unit artifacts | Planned | prerequisite |
| [TASK-2059](tasks/TASK-2059-file-inline-module-unit-parity.md) | Build one file/inline source-acquisition and module-unit route | Planned | prerequisite |
| [TASK-2060](tasks/TASK-2060-checked-module-interface-and-export-closure.md) | Define checked export-closed interfaces and public/private views | Planned | prerequisite |
| [TASK-2061](tasks/TASK-2061-interface-import-resolution-and-visibility.md) | Resolve imports and visibility from checked interfaces | Planned | prerequisite |
| [TASK-2062](tasks/TASK-2062-module-aware-core-cps-lowering.md) | Lower resolved modules through Core and CPS with origin preservation | Planned | prerequisite |
| [TASK-2063](tasks/TASK-2063-engine-linked-module-admission.md) | Link reachable modules and admit one Engine artifact | Planned | active |
| [TASK-2064](tasks/TASK-2064-module-conformance-and-client-parity.md) | Prove module conformance, mutation resistance, and CLI/daemon parity | Planned | active |
| [TASK-2065](tasks/TASK-2065-module-realization-closeout.md) | Close the phase with review, traceability, documentation, and full gates | Planned | none |

## Phase evidence policy

Each implementation task starts only after it is promoted to **In progress**, linked to the `MOD-REAL-*` coverage row it owns, and given an active semantic-task record with focused commands. A completed handoff does not establish complete-feature parity. The phase reports `implemented` only after every SPEC-103 clause has implementation and evidence; until then every incomplete rule remains `partial` and `below_spec`.

TASK-2064 owns cross-layer conformance. It compares the same admitted source tree, inputs, module identities, and run-control envelope through CLI and daemon, then compares normalized terminal results. It must reject any direct-evaluator fallback.

## Global verification

```text
cargo fmt --check
cargo test -p ash-parser
cargo test -p ash-core
cargo test -p ash-typeck
cargo test -p ash-engine
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```

The closeout task adds exact focused module conformance commands once TASK-2057 through TASK-2064 create their test targets.