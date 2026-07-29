---
id: plan.205.engine-only-execution-cutover
title: Engine-Only Execution Cutover
kind: plan
status: complete
authority: planning
owner: language-semantics
last_verified: 2026-07-29
---

# PLAN-205: Engine-Only Execution Cutover

## Purpose

Realize the Phase-204 frozen contracts and remove every Rust direct AST evaluator, independent
CPS evaluator, and differential oracle. Lean remains a deferred separate formalization project;
it has no current executable, conformance, or proof evidence/authority for Ash and no runtime
refinement bridge. A later separate project must establish any such bridge. The only executable
Ash pipeline after this phase is:

```text
Surface Ash → checked Core → checked CPS → Engine executor → terminal envelope
```

The Engine owns CPS validation and evaluation. `ash run`, `ash test`, and REPL each create a local
Engine instance and never interpret AST, CPS, or a client-local test expression. They do not
communicate with the daemon. The daemon accepts submitted descriptors, uses its own local Engine
instance, and manages long-running programs.

Bounded implementation realizes exactly the target-spec domain—no less, no more. Each selected
slice is declared explicitly and is never generated.

## Required ordering

1. TASK-2037 establishes the Engine-owned CPS executor boundary and stages the `ash-interp` to
   `ash-runtime` migration without retaining a public evaluator API.
2. TASK-2038 and TASK-2039 migrate `ash test` and REPL onto the target contracts from TASK-2035.
3. TASK-2042 carries declared descriptors and normalized terminal envelopes through the daemon;
   daemon and direct `ash run` execution independently use local Engine requests for the same
   source contract.
4. TASK-2040 deletes the Rust direct AST/differential implementation and all corresponding
   fixtures, tests, scripts, and workflows, while preserving Lean sources/docs under the deferred
   handoff defined by TASK-2034.
5. TASK-2041 proves the zero-use state, closes traceability and documentation, and promotes no
   target-spec parity claim beyond the rules actually realized.

If a crate rename must wait for deletion of the final test-only direct evaluator, TASK-2037 owns
the migration contract and TASK-2040 performs the atomic final rename/delete handoff. No
transitional package may expose an evaluator or be reachable from a client route.

## Task sequence

| Task | Outcome | Run-route impact |
|---|---|---|
| [TASK-2037](tasks/TASK-2037-engine-owned-cps-executor-and-runtime-crate-rename.md) | Engine-private CPS executor and residual runtime support boundary | prerequisite |
| [TASK-2038](tasks/TASK-2038-ash-test-canonical-engine-execution.md) | `ash test` source-wrapper execution and deferred-case disposition | active |
| [TASK-2039](tasks/TASK-2039-repl-canonical-engine-execution.md) | REPL Engine client route and parity evidence | active |
| [TASK-2042](tasks/TASK-2042-daemon-admitted-request-terminal-envelope-parity.md) | Daemon descriptor transport and normalized-terminal parity | active |
| [TASK-2040](tasks/TASK-2040-remove-direct-ast-and-differential.md) | Rust direct AST and differential removal; Lean quarantine | active |
| [TASK-2041](tasks/TASK-2041-engine-only-closeout-docs-traceability-and-gate.md) | Zero-use gate, current documentation, and evidence closeout | active |

## Completion evidence

The phase completes only when API and repository scans show no Rust AST evaluator, no non-Engine
CPS execution entry point, no Rust differential executor, and no stale current-document claim.
Lean sources and docs remain only under their explicit deferred separate-project handoff. Focused
and end-to-end tests must show identical normalized terminal results for the same selected source
contract through `ash run`, daemon, `ash test`, and REPL. Every unsupported catalogue entry must
have an explicit deferred/rejected result; none may fall back. Retained Lean
material has no current executable, conformance, or proof evidence/authority and no runtime
refinement bridge; its later separate project owns establishing any bridge.
