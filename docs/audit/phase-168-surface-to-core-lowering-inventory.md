# Phase 168 surface-to-Core lowering inventory

## Status

Implemented as TASK-1726 evidence for PLAN-168.

## Scope

This inventory maps the live lowering implementation after the Phase 168 parser/surface substrate
changes. It compares current code to `SPEC-098c` and identifies follow-on implementation ownership.
It does not claim full `SPEC-098c` lowering completion.

## Live lowering entry points

| Entry point | File | Purpose | Notes |
|---|---|---|---|
| `lower_expr` | `crates/ash-parser/src/lower.rs` | Main surface `Expr` → `ash_core::Expr` bridge. | Rejects parser-only forms such as generalized `do`, comprehensions, and now operator sections. |
| `lower_binary_op` / `lower_unary_op` | `crates/ash-parser/src/lower.rs` | Built-in operator mapping. | `BinaryOp::Pipe` is rejected because pipe is parse-time sugar. |
| `lower_fn_contract` | `crates/ash-parser/src/lower.rs` | Function contract clauses → core workflow-contract predicates. | Partial contract subset; not the full predicate/evidence/trace-contract system. |
| `lower_workflow` / workflow helpers | `crates/ash-parser/src/lower.rs` | Legacy surface workflow forms → `ash_core::Workflow`. | Handles existing workflow AST, not the full target surface tower. |
| `lower_module_role_definitions` and capability helpers | `crates/ash-parser/src/lower.rs` | Role/capability metadata → core role/capability summaries. | Supports module/import capability context. |
| `legacy_workflow_adapter` | `crates/ash-engine/src/legacy_workflow_adapter.rs` | Engine-facing workflow-form adapter. | Parallel seam for runtime artifact construction, not a replacement for general lowering. |
| `expand_surface_module` | `crates/ash-parser/src/surface.rs` | Parsed-surface → expanded-surface boundary. | Phase 168 boundary only: ordinary modules pass through; unresolved operator sections fail closed. |

## lowering-family matrix

| SPEC-098c family | Live behavior | Status | Follow-on owner |
|---|---|---|---|
| Parsed-surface to expanded-surface handoff | `expand_surface_module` names the stage and rejects unresolved operator sections. | Partial, Phase 168 substrate implemented. | Next expansion/notation phase should add real expansion outputs and diagnostics. |
| Literal/variable/core expression lowering | `lower_expr` maps literals, variables, field/index access, unary/binary expressions, calls, matches, blocks, closures, constructors, lists, failure handlers. | Implemented for current parser AST. | Maintain as baseline; add source-origin sidecars later. |
| Built-in operator lowering | Semantic `BinaryOp`/`UnaryOp` map to core operators; `Pipe` is rejected because parser desugars it before lowering. | Implemented for built-ins. | Notation phase should separate user-defined operator resolution from built-in lowering. |
| Operator sections and notation erasure | `Expr::OperatorSection` preserves section shape and raw operator token; `lower_expr` returns `UnsupportedFeature`. | Phase 168 fail-closed boundary implemented, erasure deferred. | New notation/section elaboration packet. |
| General macro expansion | No macro AST or expander; expansion diagnostics reserve categories only. | Unowned/deferred. | Macro substrate packet after notation boundary exists. |
| Pure callables and closures | `Expr::FnDef`, `Expr::FnApply`, named function defs, local fns, and closure arrows lower to core function carriers where supported. | Partially implemented for current pure callable subset. | Callable-row lowering packet should reconcile with target callable row semantics. |
| Generalized `do:K` notation | Parser has `Expr::DoBlock`; lowerer rejects as parser-only until typed elaboration. | Rejected/fail-closed. | Do-notation elaboration packet with type-directed target resolution. |
| `act { ... }` legacy block sugar | Existing `ActBlock` lowers through operational call wrappers. | Implemented for current legacy act shape. | Ambient monad/tower unification packet should reconcile legacy act path. |
| Comprehensions | Parser has `Expr::Comprehension`; lowerer rejects as parser-only. | Rejected/fail-closed. | Comprehension-to-do elaboration packet. |
| Handlers / failure handling | `fail` and `with error` parse and lower to core carriers. | Implemented for current operational-bottom subset. | Contract/handler packet should verify rows and recovery semantics against target specs. |
| Contracts and evidence | `lower_fn_contract` lowers a subset of requires/ensures predicates; workflow contract surface is classified separately. | Partial. | Contract lowering packet should integrate predicate sidecars, evidence, blame, and trace contracts. |
| Interfaces/impls/laws/proofs | Parser stores interface/impl/law/proof metadata; lowering focuses on selected summaries and current tests. | Partial/uniform lowering unowned. | Impl/law/proof lowering packet. |
| Capability and role metadata | Capability resolution context and role metadata lowering exist. | Implemented for current capability model. | Future packets should preserve surface origins and contract observation evidence. |
| Source-origin propagation | Spans are carried broadly; `SurfaceOrigin` exists but is not threaded through Core. | Partial carrier only. | Origin/provenance sidecar packet after expanded AST semantics settle. |
| Trace contracts / temporal monitors | Runtime/core sidecars exist from earlier phases, but no complete surface lowering from target trace-contract syntax. | Partial/unowned for surface. | Trace-contract surface lowering packet. |

## Follow-on implementation packet recommendation

A next phase should be explicitly scoped as **Surface Expansion and Notation Elaboration** rather than
"finish all lowering". Recommended task order:

1. **Expansion diagnostics and traversal API**: make `expand_surface_module` traverse all expression,
   workflow, contract, law, and proof surfaces with reusable visitors and structured diagnostics.
2. **Notation declaration parser inventory/design**: define the minimal syntax and AST for declaration
   of operator/notation names without implementing type-directed resolution.
3. **Built-in operator token normalization**: preserve raw token spelling for built-in binary
   expressions where source-origin diagnostics need it, without breaking `BinaryOp` consumers.
4. **Operator-section elaboration**: lower `(+), (x +), (+ x)` to explicit callable surface forms only
   after notation resolution can identify the operator function.
5. **Expanded-surface to Core lowering gate**: change high-level lowerers to accept an
   `ExpandedSurfaceModule` or equivalent so unresolved parsed-surface syntax cannot bypass the gate.
6. **Source-origin sidecar threading**: decide which Core nodes receive origin metadata versus parser
   diagnostics only.
7. **General `do`/comprehension lowering packet**: only after type-directed target resolution is
   available.

## Verification hooks for future packets

- Any new surface-only `Expr` variant must have both parser tests and `lower_expr` fail-closed tests.
- Any expansion-owned form must have an `expand_surface_module` positive/negative test.
- Any Core-lowered sugar must prove the parsed-surface form no longer reaches `lower_expr` directly.
- `ash-lint` and other AST traversals must be updated deliberately for every new `Expr` variant.

## Conclusion

Phase 168 establishes enough live substrate to prevent notation/operator sections from being erased or
misrepresented before lowering. Full `SPEC-098c` lowering remains a follow-on program with concrete
owners listed above.
