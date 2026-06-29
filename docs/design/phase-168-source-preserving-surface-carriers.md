# Phase 168 source-preserving surface carrier design

## Status

Implemented as TASK-1722 evidence for PLAN-168.

## Purpose

This design defines the first source-preserving carrier slice for Ash surface syntax. It is narrow on
purpose: it supports Phase 168 notation/operator-section and expanded-surface boundary work without
committing to a full macro library, full CST, or hygiene implementation.

## Inputs

- `docs/audit/phase-168-surface-ast-lowering-inventory.md`
- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md` §3-§6 and §11
- Live parser surfaces in `crates/ash-parser/src/surface.rs` and `crates/ash-parser/src/parse_expr.rs`

## Current → Target carrier table

| Current live Rust type/seam | Current limitation | Target Phase 168 carrier | Implementation location |
|---|---|---|---|
| `token::Span` | Preserves source extent only; no origin class. | Reuse `Span` in all new carriers; add `SurfaceOrigin` for copied/desugared/expanded nodes. | `crates/ash-parser/src/surface.rs` |
| `Expr::Binary { op: BinaryOp, ... }` | Operator spelling is collapsed to semantic built-in op. | Keep existing built-in binary shape stable; add raw-token carrier only where Phase 168 parses operator sections. | `surface.rs`, `parse_expr.rs` |
| No raw operator payload | Notation token spelling/span cannot be named. | `RawOperatorToken { spelling, span }`. | `surface.rs` |
| No operator-section AST | `(+), (x +), (+ x)` cannot be represented honestly. | `OperatorSection { kind, operator, left, right, span }` and `Expr::OperatorSection`. | `surface.rs`, `parse_expr.rs` |
| No expanded-surface stage | Lowering consumes parsed surface AST directly. | `ParsedSurfaceModule`, `ExpandedSurfaceModule`, `ExpansionDiagnostic`, `ExpansionError`, `expand_surface_module`. | `surface.rs` |
| Comment table in parser input | Available but not tied to every AST node. | Keep as module/input metadata for now; do not retrofit per-node comments in Phase 168. | Deferred |
| Attributes/metadata on selected defs | No unified attribute carrier for macros. | Do not introduce universal attribute carrier yet; note as macro follow-on. | Deferred |
| Grouping delimiters | Parentheses usually vanish after parsing. | Preserve grouping only where it changes pre-expansion semantics: operator sections. | `Expr::OperatorSection` |

## API sketch

```rust
pub struct RawOperatorToken {
    pub spelling: Box<str>,
    pub span: Span,
}

pub enum OperatorSectionKind {
    Bare,
    Left,
    Right,
}

pub struct OperatorSection {
    pub kind: OperatorSectionKind,
    pub operator: RawOperatorToken,
    pub left: Option<Box<Expr>>,
    pub right: Option<Box<Expr>>,
    pub span: Span,
}

pub enum SurfaceOrigin {
    Source { span: Span },
    MacroExpansion { call_span: Span, expansion_id: Box<str> },
    NotationExpansion { notation_span: Span, target: Box<str> },
    OperatorSection { section_span: Span, operator_span: Span },
    Desugaring { source_span: Span, rule: Box<str> },
}

pub struct ParsedSurfaceModule {
    pub module: ModuleFile,
    pub origin: SurfaceOrigin,
}

pub struct ExpandedSurfaceModule {
    pub module: ModuleFile,
    pub diagnostics: Vec<ExpansionDiagnostic>,
}
```

## Boundary decisions

1. **Extend `surface.rs` first.** Phase 168 does not introduce a new `ash_syntax` crate or parser
   submodule because existing downstream crates already depend on `ash_parser::surface` and the first
   slice is small.
2. **Preserve existing binary expressions.** Existing accepted syntax such as `a + b` remains
   `Expr::Binary`; Phase 168 does not redesign built-in operator parsing into general notation.
3. **Represent only binary infix sections.** `(+), (x +), (+ x)` get an AST carrier. Generalized
   mixfix holes such as `(_ + _)` fail closed because the full mixfix partial-application model is
   outside this phase.
4. **Name the expanded boundary now.** `expand_surface_module` is a no-op for ordinary modules but
   rejects unresolved operator sections. This prevents unresolved surface-only syntax from leaking
   into Core lowering.
5. **Use explicit deferral diagnostics.** Deferred macro/notation features should become explicit
   `ExpansionDiagnosticKind`/`ExpansionError` cases rather than silent fallthrough.

## Migration constraints

- Existing parser tests that match `Expr::Binary`, `Expr::Call`, `Expr::FnApply`, or `Expr::Block`
  must continue to pass unchanged.
- Downstream exhaustive matches over `Expr` must decide whether `Expr::OperatorSection` is traversed,
  rejected, or ignored. Wildcard masking is discouraged for semantic consumers.
- `lower_expr` must reject `Expr::OperatorSection` until notation resolution or section desugaring is
  implemented.
- Lint and analysis traversals that inspect child expressions should treat operator sections as
  syntax-only and not infer semantic policy/effect facts from them.

## Required tests

- Parser test: `(+), (x +), (+ x)` parse to `Expr::OperatorSection` and preserve raw operator token
  spelling and spans.
- Regression test: `(x + y)` remains an ordinary `Expr::Binary`.
- Fail-closed test: `(_ + _)` is not accepted as a generalized mixfix section.
- Expansion-boundary test: ordinary modules pass through `expand_surface_module`, while modules with
  operator sections return `ExpansionError::UnresolvedOperatorSection`.
- Lowering test: direct `lower_expr` of an operator section returns `UnsupportedFeature` until a later
  phase owns notation/section erasure.

## Deferred carriers

- Full concrete syntax tree or token tree with exact whitespace/comment attachment.
- Macro invocation/definition ASTs and hygiene marks.
- Universal attributes carrier across every declaration/expression node.
- General user-defined operator declarations and precedence tables.
- Generalized mixfix sections and placeholder elaboration.
- Type-inference integration for notation resolution.

## non-goals

- Do not implement full macro expansion.
- Do not lower operator sections to Core in Phase 168.
- Do not make all operators user-definable.
- Do not change accepted semantics of existing built-in binary expressions.
- Do not replace the existing parser AST with a new CST in this phase.
