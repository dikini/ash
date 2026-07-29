# TASK-1988: Semantic Implementation and Deprecation Packet

> **TASK-2041 status:** This audit packet preserves historical implementation evidence. It does
> not authorize direct AST execution, a non-Engine CPS executor, a differential route, or a
> client fallback.

**Status:** Complete audit packet
**Baseline:** `c9294828` (Phase 201 closeout); audit run 2026-07-24
**Canonical owners:** [Ash Canonical Core](../../spec/CANONICAL-CORE.md):
`VOCAB-TARGET-OVERVIEW-001`, `GRAM-TARGET-MODULE-001`, `TYPE-TARGET-ROW-001`,
`CORE-CPS-SYNTAX-001`, `LOWER-SURFACE-CORE-001`, `SEM-TARGET-CORE-CPS-001`,
`OBS-TARGET-PROJECTION-001`, and `CONF-IMPLEMENTATION-001`.

## Method and classification

This packet records current implementation evidence; it does not promote Rust code or old
workflow-era documents to semantic authority.  Each disposition is one of `retain-private`,
`fold-into-target`, `delete`, or `needs-decision`.  A future deletion requires behavior-level
absence/parity evidence, not a spelling scan.

The Core/CPS vertical slice used exposed rust-analyzer symbol/reference/diagnostic operations
before bounded source inspection. The surface/lowering and runtime-observable slices used bounded
`rg` and source tracing because language-aware MCP was unavailable for those audits. They reconcile
Phase 201 TASK-1971/TASK-1972: neither task changed the Core/CPS prototype sources, and their
function-first artifact handoff remains preserved.

## Rule-to-realization map

| Canonical subject/rule | Current realization evidence | Classification | Disposition and owner |
|---|---|---|---|
| `VOCAB-TARGET-OVERVIEW-001` | Public typechecker builtin `Act<T>`/`Proc<T>` wrappers and tower machinery remain reachable despite function-first canonical vocabulary. | conflicts | [TASK-2000](../tasks/TASK-2000-residual-act-proc-public-machinery-decision.md) is a behavior-preserving decision gate. |
| `GRAM-TARGET-MODULE-001` | Parser/Core paths preserve a partial function-first row path; target aliases, groups, handlers, and newtypes are not all represented, while proxy claims conflict with removed-form policy. | partially-implements | [TASK-2001](../tasks/TASK-2001-target-grammar-gap-and-spec-conflict-decision.md) decides grammar/spec deltas. |
| `TYPE-TARGET-ROW-001` | Function-first rows are partially carried, but the reachable public tower wrappers conflict with the target computation vocabulary; aliases/groups need an explicit grammar/type disposition. | partially-implements/conflicts | [TASK-2000](../tasks/TASK-2000-residual-act-proc-public-machinery-decision.md) and [TASK-2001](../tasks/TASK-2001-target-grammar-gap-and-spec-conflict-decision.md). |
| `CORE-CPS-SYNTAX-001` | `core_ash::{CoreExpr, CoreType, CoreRow}` plus `cps::{Term, Value, TrapReason}` model most syntax; Core validation/checking/lowering produces CPS terms. | partially-implements | [TASK-2003](../tasks/TASK-2003-return-authority-and-cps-kernel-decision.md) resolves the `Return` conflict; [TASK-2004](../tasks/TASK-2004-core-cps-production-boundary-decision.md) decides the realization boundary. |
| `LOWER-SURFACE-CORE-001` | Ambient `do` lowering remains; generic `do` rejects. Macro/notation, handler, and evidence/trace sidecars are not carried as the target handoff requires. | partially-implements | [TASK-2002](../tasks/TASK-2002-generic-do-and-lowering-sidecar-strategy.md) owns strategy and parity fixtures. |
| `SEM-TARGET-CORE-CPS-001` | CPS evaluator validates and executes terms; `HandlerChain::find_operation_frame` is innermost-first across handler/provider frames; missing match yields `UnhandledEffect`. Focused frame/multiplicity tests pass. | partially-implements | [TASK-2004](../tasks/TASK-2004-core-cps-production-boundary-decision.md), [TASK-2005](../tasks/TASK-2005-direct-runtime-core-cps-semantic-parity.md), and [TASK-2006](../tasks/TASK-2006-cps-public-api-visibility-decision.md). |
| `OBS-TARGET-PROJECTION-001` | Production engine executes `ash_core::Expr` through `ash_interp::eval_expr_async`, not `core_ash::CoreExpr` or checked CPS. Trace/session and artifact reports are telemetry/infrastructure, not the canonical terminal projection. CLI JSON conversion exposes `_variant`, contradicting SPEC-021. | conflicts | [TASK-2005](../tasks/TASK-2005-direct-runtime-core-cps-semantic-parity.md), [TASK-2007](../tasks/TASK-2007-cli-core-terminology-clarification.md), and [TASK-2008](../tasks/TASK-2008-json-variant-observable-projection.md). |
| `CONF-IMPLEMENTATION-001` | No canonical differential corpus/harness currently executes and compares the target relation. | unmapped | Existing [TASK-439](../tasks/TASK-439-differential-conformance-harness-rust-first.md) is augmented as the sole owner. |

## Concrete Core/CPS evidence

- `cps::Term` has `LetVal`, `LetPrim`, `LetCont`, `LetContCall`, `Jump`, `Call`, `If`, `Match`,
  `Return`, `Trap`, and effect extensions. `CoreType::Cont` has input, fixed answer, row, and
  multiplicity; Core checking rejects invalid affine/multi-shot use.
- `core_ash_lower::lower_core_program` and
  `core_ash_typecheck::type_check_and_lower_core_program` are not called by non-test Rust
  consumers. `ash_interp::cps::eval_checked` is likewise referenced only by its module,
  validation code, and tests. They are executable prototypes, not current application execution.
- Production `Engine::execute_expr_with_bindings` evaluates `ash_core::Expr`; this is a third
  materially distinct "Core" dialect from `core_ash::CoreExpr` and `cps::Term`.
- Passing focused evidence: Core-to-CPS lowering (14 tests), Core handler/affine checking (17),
  handler/provider dispatch (5), and CPS multiplicity validation/runtime fail-closed behavior
  (11). These establish prototype behavior only; they do not establish production parity.
- `CANONICAL-CORE` includes `Return` in the kernel while SPEC-098b says CPS has no direct return.
  No implementation task may choose one interpretation implicitly.

## Completion and removal constraints

1. No carrier is approved for deletion by this packet.  Every potential removal is assigned a
   task with a required behavior-level absence or parity proof.
2. Public semantic APIs must have a canonical owner or a documented private-machinery rationale.
3. A passing prototype test cannot be used as proof that the current application runtime realizes
   the same rule.
4. TASK-439 is deliberately reused: a second differential harness would create competing result
   formats and corpus authority.

## Verification evidence

```text
cargo test -p ash-core --test task_1627_core_to_cps_basic             # 7 passed
cargo test -p ash-core --test task_1628_core_to_cps_effects           # 7 passed
cargo test -p ash-core --test task_1647_core_handle_affine_resume     # 17 passed
cargo test -p ash-interp --test task_1858_1859_handler_provider_semantics # 5 passed
cargo test -p ash-interp --test task_1683_cps_multishot_validation    # 11 passed
```

The documentation gates recorded in TASK-1988 completion evidence validate the packet and task
routing, not the future implementation tasks.
