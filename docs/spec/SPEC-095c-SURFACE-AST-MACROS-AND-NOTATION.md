---
id: spec.ash.surface-ast-macros-notation
title: Ash Surface AST, Macro Expansion, and Notation
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-29
verified_against:
  specs:
    - docs/spec/SPEC-095b-TARGET-GRAMMAR.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md
  audits:
    - docs/audit/2026-06-29-target-spec-notes-gap-audit.md
---

# SPEC-095c: Surface AST, Macro Expansion, and Notation

**Status:** Draft — target surface AST and expansion substrate.
**Scope:** This document specifies the source-preserving surface tree, macro-expansion boundary,
user-defined notation, and operator sections that sit between target grammar and Core lowering.
**Depends on:** SPEC-095b, SPEC-097b, SPEC-098c.

## 1. Purpose

Ash target grammar must guide an eventual `syn`-like library: a parser-facing API suitable for
formatters, linters, code generation, and Rust-style macros. The parser must therefore produce a
source-preserving syntax tree before semantic elaboration. Surface sugar, macro output,
user-defined notation, and operator sections are resolved before Core lowering while preserving
source origins for diagnostics and tooling.

## 2. Non-goals

- This spec does not implement parser, macro expander, or lowering code.
- This spec does not define typed macros.
- This spec does not make binder-introducing mixfix notation part of the initial notation model.
- This spec does not add new Core operators. Notation expands to ordinary callable syntax.

## 3. Pipeline

```text
tokens
  -> concrete syntax tree / parsed surface AST
  -> macro expansion
  -> notation and operator-section resolution
  -> expanded surface AST
  -> name resolution and type elaboration
  -> Core AST + sidecars
  -> Core type checking
  -> CPS lowering
```

The parsed surface AST is source-preserving. The expanded surface AST is still source-level Ash but
has macro invocations expanded and notation resolved. Core receives neither macros nor notation.

## 4. Syntax tree layers

### 4.1 Token tree / concrete syntax

The token/concrete layer preserves spans, delimiters, doc comments, attributes, grouping,
operator-like tokens, keyword spelling, and macro token trees. Tools may use this layer for
formatting and code generation without committing to semantic resolution.

### 4.2 Parsed surface AST

The parsed surface AST groups tokens into Ash syntax categories while preserving syntactic shape:

```text
File      ::= { Item }
Item      ::= Fn | Handler | Interface | Impl | Type | Newtype | Fact | Evidence | Notation | MacroCall | ...
Expr      ::= Path | Call | Block | Do | Handle | On | Prefix | InfixChain | Suffix | MixfixUse
            | OperatorSection | MacroCall | Paren | ...
Type      ::= Path | FnType | RowType | ModeType | ParenType | ...
Row       ::= RowItems | RowVar | RowTail
WhereItem ::= RowItem | FactDecl | EvidenceDecl | ProofDecl | Constraint
```

This shape is intentionally syntax-first. It must not collapse `a + b`, `add(a, b)`, and `(+)(a,b)`
into the same node before notation resolution.

### 4.3 Expanded surface AST

The expanded surface AST has macro invocations expanded and notation resolved into callable forms,
but it still contains Ash surface concepts such as functions, handlers, `do`, facts/evidence, rows,
and contracts. It is the input to the surface-to-Core lowering spec.

### 4.4 Elaborated Core boundary

At the Core boundary:

- macros are gone;
- notation and operator sections are gone;
- source origins are preserved;
- facts/evidence have stable identities;
- contract predicates lower to predicate sidecars;
- operation identities are resolved to abstract or concrete impl-type operations.

## 5. Source preservation and origin metadata

Every syntax node carries at least a primary span. Nodes that arise from expansion or desugaring also
carry origin metadata:

```text
Origin ::= Source(span)
         | MacroExpansion { call_span, expansion_id }
         | NotationExpansion { notation_span, target }
         | OperatorSection { section_span, operator_span }
         | Desugaring { source_span, rule }
```

The exact Rust carrier shape is not fixed here, but an implementation should be compatible with an
`ash_syntax`/`ash_syn` style library that exposes token streams, surface nodes, spans, attributes,
and generated-origin metadata.

## 6. Macro expansion boundary

Macros are syntax-to-syntax transformations over token trees or parsed surface AST nodes. The base
macro model is hygiene-ready: generated identifiers carry call-site/definition-site metadata and an
expansion id even if full hygiene is implemented later.

Macro expansion occurs before notation resolution unless a macro explicitly quotes raw tokens. Macro
output must be parsed or re-associated using the active notation table before type checking.

## 7. Notation declarations

Notation declarations bind syntactic sugar to callable targets. They are items in the surface AST:

```text
NotationDecl ::= visibility? FixityDecl NotationPattern "=" callable_path
FixityDecl   ::= "prefix" precedence?
               | "infixl" precedence
               | "infixr" precedence
               | "infix" precedence
               | "suffix" precedence?
               | "mixfix"
```

The declaration creates no Core primitive. It registers a notation pattern for the current module or
exported surface and expands uses to calls of the target callable.

In short: notation is source-level sugar and is gone before Core.

### 7.1 Prefix notation

```ash
prefix ! = not
!ready
```

expands to:

```ash
not(ready)
```

### 7.2 Infix notation

```ash
infixl 6 <+> = combine
left <+> right
```

expands to:

```ash
combine(left, right)
```

Infix chains may be parsed as flat `InfixChain` nodes and re-associated after imports provide the
active notation table. Precedence conflicts are diagnostics, not type-inference problems.

### 7.3 Suffix notation

```ash
suffix ? = is_present
value?
```

expands to:

```ash
is_present(value)
```

### 7.4 Mixfix notation

Mixfix notation uses explicit holes. Initial mixfix is expression sugar only and does not introduce
binders.

```ash
mixfix _ between _ and _ = between
x between lo and hi
```

expands to:

```ash
between(x, lo, hi)
```

Binder-introducing forms such as `for _ in _ yield _` are future macro territory unless a later spec
adds hygiene and binder rules for notation.

## 8. Operator sections

Operator sections are source-level callable sugar for binary infix notation. They preserve shape in
the parsed surface AST:

```text
Expr::OperatorSection {
  operator: OperatorToken,
  kind: Bare | Left | Right,
  left: Option<Expr>,
  right: Option<Expr>,
  span: Span,
}
```

Given:

```ash
infixl 6 <+> = combine
```

full infix use expands as usual:

```ash
a <+> b       => combine(a, b)
```

A left section:

```ash
(a <+>)       => fn (b) -> combine(a, b)
```

A right section:

```ash
(<+> b)       => fn (a) -> combine(a, b)
```

A bare operator value:

```ash
(<+>)         => combine
```

or to an eta-expanded equivalent if required by representation.

Initial operator sections are limited to binary infix operators. Partial application of arbitrary
mixfix notation is deferred.

## 9. Typing and authority invariants

Notation and sections are type checked after expansion as ordinary callable syntax. If the target
callable has type:

```text
op : (A, B) -> {r} C
```

then:

```text
(a op) : B -> {r} C
(op b) : A -> {r} C
(op)   : (A, B) -> {r} C
```

The latent row `{r}` is preserved. Notation cannot hide authority, failure, evidence, or contract
requirements. In predicate/contract position, expansion happens before predicate well-formedness;
the resolved callable must still be an admitted pure predicate function.

## 10. Import/export and active notation tables

A module's active notation table is derived from local notation declarations and imported/exported
notation. Ambiguous or conflicting notation declarations are rejected before type inference. The
implementation may parse expression operator chains before the full table is known, then run a
fixity-resolution pass once imports are resolved.

## 11. Desugaring invariants

- Macro expansion is complete before Core lowering.
- Notation and operator sections are erased before Core lowering.
- Expanded nodes retain source-origin metadata.
- Contract predicate checking runs after macro/notation expansion and before predicate lowering.
- Core sees ordinary calls, closures, handlers, rows, facts/evidence, and sidecars; it does not see
  custom operator syntax.

## 12. See also

- [SPEC-095b: Target Grammar](SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-097b: Target Type System](SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098c: Surface-to-Core Lowering](SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [Audit: Target spec gaps against notes](../audit/2026-06-29-target-spec-notes-gap-audit.md)

## 13. Changelog

- 2026-06-29: Created as the target surface AST, macro expansion, notation, and operator-section substrate.
