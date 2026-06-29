# Phase 168 surface AST and lowering inventory

## Status

Implemented as TASK-1721 evidence for PLAN-168.

## Scope

This audit maps the live parser and lowering seams against `SPEC-095c` and `SPEC-098c` before Phase
168 implementation changes. It is a gap table, not a normative spec.

## Read surfaces

- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/src/parse_expr.rs`
- `crates/ash-parser/src/parse_module.rs`
- `crates/ash-parser/src/parse_module/fn_defs.rs`
- `crates/ash-parser/src/lower.rs`
- `crates/ash-engine/src/legacy_workflow_adapter.rs`
- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`

## Current implementation map

| Layer from SPEC-095c | Live seam | Current behavior | Risk | Owning task |
|---|---|---|---|---|
| Token/concrete layer | `lexer.rs`, `token.rs`, `ParseInput` | Tokens carry spans; comments are collected in `CommentTable`; there is no public CST/token-tree layer. | Formatting/macro tooling cannot round-trip all source shape yet. | TASK-1722 |
| Parsed surface AST | `surface.rs` `ModuleFile`, `Definition`, `Expr`, `WorkflowDef` | Parser produces typed surface AST with spans on most nodes; many constructs are already surface-only (`DoBlock`, `Comprehension`, `ActBlock`). | AST already mixes parsed source with some semantic normalization/desugaring assumptions. | TASK-1722/TASK-1723 |
| Operator-like tokens | `parse_expr.rs` precedence parser, `BinaryOp`/`UnaryOp` | Built-in infix operators lower immediately to semantic `BinaryOp`; raw operator token spelling is not preserved for binary expressions. | User-defined notation cannot be represented without a raw-token carrier or explicit fail-closed boundary. | TASK-1723 |
| Operator sections | No pre-existing `Expr` variant before Phase 168 | `(+), (x +), (+ x)` had no honest AST boundary. | Sections could be rejected opaquely or accidentally parsed as ordinary parenthesized expressions. | TASK-1724 |
| Macro calls | No macro-call AST carrier in live parser | Full macro syntax is not implemented. | Future macro work needs a token/concrete tree or explicit deferred diagnostic. | Future macro packet after Phase 168 |
| Expanded surface AST | No named live boundary before Phase 168 | Lowering consumes parser `Expr` directly. | Specs require macros/notation/sections to be gone before Core, but code had no named checkpoint. | TASK-1725 |
| Surface-to-Core lowering | `crates/ash-parser/src/lower.rs` | Direct `Expr` to `ash_core::Expr` lowering; surface-only forms fail closed through `UnsupportedFeature` in several paths. | `SPEC-098c` families are not uniformly owned; lowering can only be audited family-by-family. | TASK-1726 |
| Legacy workflow adapter | `crates/ash-engine/src/legacy_workflow_adapter.rs` | Workflow header/body summaries are adapted into `WorkflowForm` for contracts/projections. | This is a parallel workflow-specific lowering seam, not the general expanded-surface-to-Core bridge. | TASK-1726 |

## Consumer assumptions

| Consumer | Assumption today | Impact |
|---|---|---|
| `ash_parser::lower::lower_expr` | `Expr` has already had parser-only unsupported features rejected or will reject them locally. | New surface-only variants must fail closed here until expansion/lowering owns them. |
| `ash_engine::legacy_workflow_adapter` | Legacy `WorkflowDef` header/body syntax is already classified enough for contract/projection summaries. | General surface expansion must not redefine this workflow-specific adapter accidentally. |
| `ash-lint` policy checks | Exhaustively traverses `Expr` for policy nodes. | New `Expr` variants require traversal/rejection decisions, not wildcard masking. |
| Parser integration tests | Many tests directly pattern-match `Expr` variants. | Source-preserving carriers must avoid needless broad constructor churn. |

## gap table

| SPEC requirement | Live code seam | Current behavior | Downstream risk | Proposed owning task |
|---|---|---|---|---|
| Preserve raw operator tokens before notation resolution | `parse_expr.rs`, `surface.rs` | Built-in operators collapse to `BinaryOp`; section forms had no raw token before Phase 168. | Notation implementation would need to infer source spelling from semantic op. | TASK-1723 |
| Binary infix operator sections exist as callable sugar before Core | `parse_expr.rs`, `surface.rs`, `lower.rs` | No pre-existing AST boundary; Phase 168 adds `Expr::OperatorSection` and rejects lowering until expansion. | Silent erasure to `Call`/`FnApply` would lose section kind and blame span. | TASK-1724 |
| Expanded surface AST is a named stage | `surface.rs`, `lower.rs` | No pre-existing named stage; lowerer consumes parsed surface AST directly. | Macro/notation deferral can be overclaimed as if lowering happened. | TASK-1725 |
| No notation/macros/sections leak into Core | `lower.rs` | Several parser-only forms already return `UnsupportedFeature`; operator sections need the same fail-closed guard. | Core may receive unresolved surface sugar. | TASK-1724/TASK-1725 |
| General surface-to-Core lowering by family | `lower.rs`, `legacy_workflow_adapter.rs` | Lowering exists for many current forms, but not as a `SPEC-098c` family matrix. | Follow-on phases may duplicate or bypass existing lowering. | TASK-1726 |

## TASK-1721 conclusion

The live parser has a substantial surface AST but lacks a CST/token-tree layer, full raw notation
preservation, and a named expanded-surface boundary. Phase 168 should therefore add narrow carriers
and fail-closed guards rather than a full `ash_syntax`/macro implementation.
