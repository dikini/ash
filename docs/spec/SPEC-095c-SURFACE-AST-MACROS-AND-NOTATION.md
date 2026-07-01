---
id: spec.ash.surface-ast-macros-notation
title: Ash Surface AST, Macro Expansion, and Notation
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-30
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
- This spec does not define arbitrary procedural macros or full macro-by-example matching.
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

Until a real macro expander exists, parsed macro invocations may be preserved as durable surface
carriers for diagnostics, but they are fail-closed: they must not be accepted by the expanded-surface
boundary, type checking, export collection, or Core lowering. Preserving a macro invocation shape is
not macro execution and grants no rows, capabilities, contracts, failures, proof evidence, or runtime
authority.

Macro expansion occurs before notation resolution unless a macro explicitly quotes raw tokens. Macro
output must be parsed or re-associated using the active notation table before type checking.

### 6.1 Phase 172 parser-first expression macro MVP

The first executable macro slice is intentionally smaller than a full macro system. It is a
parser-first expression macro MVP: macro declarations and executable invocations are ordinary parsed
surface syntax, and expansion substitutes parsed expression arguments into a parsed expression
template before notation resolution.

```text
MacroDecl      ::= visibility? "macro" name "(" ParamList? ")" "=>" Expr ";"
MacroInvokeMvp ::= name "!" "(" ExprList? ")"
```

The MVP supports only local expression-position invocations with unqualified names and parenthesized
expression arguments:

```ash
macro inc(x) => add(x, 1);

fn example(n: Int) -> Int {
  inc!(n)
}
```

Expansion is syntax-only and authority-neutral. A macro declaration does not define a callable,
capability, contract, row, failure, proof, or runtime effect. Rows and authority requirements come
only from the ordinary expression produced by expansion and checked later.

The MVP is fail-closed outside its subset:

- bracketed and braced invocations such as `m![...]` and `m!{...}` remain diagnostic carriers unless
  a later token-tree parser task implements them;
- qualified macro-like paths such as `module::m!(x)` are not part of this carrier;
- macro declarations are local to the module or inline-module scope that declares them; imports,
  re-exports, and ordinary callable visibility do not activate macros downstream;
- duplicate, missing, arity-mismatched, recursive, or unsupported-template macros are rejected before
  Core lowering;
- binder-introducing templates are rejected until a later phase specifies binder hygiene.

Supported template bodies are limited to binder-free parsed expressions. Generated expansion nodes
retain stable expansion identity and origin-chain metadata; notation/operator-section products inside
macro output record the macro expansion as their parent origin.

### 6.2 Phase 173 macro summary carriers

Public macro activation is a syntax-phase import/export feature, not callable export. A module may
publish macro summaries only through an explicit macro-summary carrier:

```text
MacroSummary ::= {
  module_path,
  name,
  identity: MacroDeclarationIdentity,
  visibility,
  params,
  input_kind,
  output_kind,
  template_fingerprint,
  hygiene_policy,
  typed_signature?,
  origin_span
}

input_kind  ::= ExprArgs | TokenTree(delimiters)
output_kind ::= Expr | TokenTree | ReparseExpr
```

Macro summaries are explicit syntax-phase metadata. They do not carry rows, contracts, proof
evidence, providers, or runtime effects. They are consumed only by the macro expansion phase. A
macro declaration is importable only when export collection has produced a well-formed
`MacroSummary`; ordinary callable summaries, raw source snippets, and reparsed body strings must not
activate macros.

Phase 175 adds canonical syntax-phase macro identity metadata:

```text
MacroDeclarationIdentity ::= {
  origin: Local | Imported { module_path, exported_name },
  local_name,
  origin_span,
  param_count,
}
```

`MacroDeclarationIdentity` is comparable tooling/expansion metadata only. It is intentionally
separate from ordinary callable identity. Imported aliases change `local_name` while preserving the
origin module/exported name pair. Same-file macro invocations may compare these identities for
navigation and reference grouping, but the identity must not be accepted as a callable binding,
effect source, provider authority, contract evidence, proof evidence, or runtime export.

Imported macro activation is fail-closed:

- missing, duplicate, ambiguous, malformed, or version-incompatible macro summaries reject before
  expansion;
- importing a callable with the same name as a macro does not activate the macro;
- importing a macro does not create a callable binding;
- re-export requires a fresh macro summary, not accidental forwarding through callable export state;
- positive import/export behavior must be paired with negative tests proving no callable leakage and
  no activation from malformed summaries.

### 6.3 Token-tree invocation carriers

Bracketed and braced macro invocations require delimiter-preserving token-tree carriers before they
can execute:

```text
TokenTree     ::= Token(span, kind, spelling)
                | Group(open_span, close_span, delimiter, [TokenTree])
MacroInvokeTt ::= name "!" TokenTreeGroup
delimiter     ::= Paren | Bracket | Brace
```

A token-tree carrier must preserve delimiter kind, open/close spans, nested groups, token spelling,
and enough raw text fallback for diagnostics. A raw substring alone is not an executable token-tree
carrier. Implementations may keep a raw-body field for diagnostics, but executable bracket/brace
macros must operate on structured token trees.

Unsupported token-tree shapes reject before the expanded-surface boundary. Token-tree output may
enter ordinary surface syntax only through a single audited reparse boundary. That boundary must
record macro origin metadata, run the same expanded-surface validation as parsed source, and reject
any residual macro invocation or unresolved notation/operator syntax that the target phase does not
explicitly support.

### 6.4 Binder hygiene metadata

Binder-introducing macros are allowed only when every introduced binding and reference carries
explicit hygiene metadata:

```text
HygieneScope ::= DefinitionSite(expansion_id)
               | CallSite(call_span)
               | Generated(expansion_id, local_id)

HygienicIdent ::= {
  spelling,
  scope,
  origin_span,
  generated: bool
}
```

The expansion validator must enforce both capture directions:

- a generated binder cannot capture caller variables unless the macro summary explicitly marks a
  call-site lookup position;
- a caller/source binder cannot capture generated identifiers;
- definition-site references resolve only through the macro definition environment recorded in the
  macro summary;
- generated identifiers are not source-spellable identifiers and must remain distinguishable in
  diagnostics and LSP-facing surfaces;
- binder-introducing templates that lack complete hygiene metadata reject before expansion output is
  accepted.

Phase 173 supports only expression-local binder-introducing templates. Macro-generated module
declarations, arbitrary item generation, and runtime/provider authority introduction remain out of
scope.

### 6.5 Typed macro signatures and bounded inference

Typed macros attach syntax-phase type obligations to macro inputs and expansion output:

```text
MacroTypeSig ::= "macro" name "(" TypedParamList? ")" "=>" OutputKind ":" Type
TypedParam   ::= name ":" MacroParamType
MacroParamType ::= Type | TokenTreeKind
```

Typed macro checking happens after macro summary resolution and before expansion output is accepted
as expanded surface. A typed macro diagnostic is a macro/type diagnostic, not an arbitrary later Core
failure. The checker must reject:

- missing required annotations on public/imported macros;
- argument/template type mismatches;
- output whose expanded surface cannot satisfy the declared output type or kind;
- typed summaries that conflict across imports/re-exports;
- any attempt to derive authority, rows, contracts, failures, proof evidence, or runtime effects from
  macro metadata itself.

Bounded inference may fill omitted local macro signature parts only when the principal result is
unique from annotated arguments and the template body. Imported/exported summaries must be explicit
or previously inferred at their definition site. Ambiguous inference rejects and asks for an
annotation; it must not default to a convenient type.

A Phase 174 callable-identity proof may use a same-definition-list local callable summary only when
exactly one ordinary `fn` or `builtin fn` with complete parameter and return annotations matches the
call name, arity, and already-inferred argument types. This proof is syntax-phase type evidence only:
it does not export the callable, does not activate imports, and does not turn `MacroSummary` into a
callable summary. Unresolved, duplicate, overloaded/interface, module-qualified, private-cross-module,
or argument-mismatched calls remain annotation-required and fail closed.

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

Implementations that do not carry notation summaries across module boundaries must keep notation
module-local and fail closed for imported notation use. Re-exporting or importing the target callable
does not activate the source notation unless an explicit future carrier transports the notation table
and proves both positive visibility and negative leakage behavior.

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
- Macro summaries are syntax-phase metadata; they do not define callables or public authority.
- Notation and operator sections are erased before Core lowering.
- Expanded nodes retain source-origin metadata, including stable expansion identity for generated
  nodes and parent-origin chains for nested expansion products.
- Generated identifiers are syntax metadata, not source identifiers. Generated names must be
  distinguishable from source-spellable identifiers so expansion cannot accidentally capture or be
  captured by source bindings.
- Contract predicate checking runs after macro/notation expansion and before predicate lowering.
- Core sees ordinary calls, closures, handlers, rows, facts/evidence, and sidecars; it does not see
  custom operator syntax or macro invocations.

## 12. See also

- [SPEC-095b: Target Grammar](SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-097b: Target Type System](SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098c: Surface-to-Core Lowering](SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [Audit: Target spec gaps against notes](../audit/2026-06-29-target-spec-notes-gap-audit.md)

## 13. Changelog

- 2026-06-30: Added Phase 173 contracts for explicit macro summaries, delimiter-preserving token-tree carriers, binder hygiene metadata, typed macro signatures, and bounded macro inference.
- 2026-06-30: Added Phase 172 parser-first expression macro MVP constraints: local `MacroDecl`, parenthesized `name!(...)` execution only, local-only scope, fail-closed unsupported forms, authority-neutral syntax substitution, and origin-chain preservation.
- 2026-06-30: Clarified Phase 171 conservative hygiene invariants: fail-closed macro invocation carriers, local-only notation unless summary carriers exist, expansion identity/origin chains, generated identifier separation, and no Core macro/notation leakage.
- 2026-06-29: Created as the target surface AST, macro expansion, notation, and operator-section substrate.
