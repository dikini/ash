# Audit: Target spec gaps against notes

## Scope

This audit preserves the 2026-06-29 review of target Ash specs against the current design notes.
It is a working audit, not a normative spec. Use it to plan follow-up spec and task work without
reloading the full conversation context.

Focus areas:

- surface grammar;
- source-preserving surface AST and future macro substrate;
- user-defined prefix, infix, suffix, and mixfix notation;
- operator sections;
- big-step and small-step operational semantics;
- type inference;
- lowering to Core;
- contracts, evidence, and trace contracts.

Primary documents reviewed:

- `docs/spec/SPEC-095b-TARGET-GRAMMAR.md`
- `docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md`
- `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md`
- `docs/spec/SPEC-098b-TARGET-IR.md`
- `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md`
- `docs/spec/SPEC-100-CORE-TYPE-CHECKING.md`
- `docs/notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md`
- `docs/notes/NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md`
- `docs/notes/NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md`
- `docs/notes/NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md`
- `docs/notes/NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md`
- `docs/notes/NOTE-027-CONTRACT-BLAME-AND-SUBSUMPTION.md`
- `docs/notes/NOTE-028-PURITY-EVALUATION-MODES-AND-CONTRACT-TIMING.md`
- `docs/notes/NOTE-029-STRUCTURED-BOTTOM-AND-CONTRACT-DIAGNOSTICS.md`
- `docs/notes/NOTE-030-MONADIC-HOARE-LOGIC-FOR-ASH-COMPUTATIONS.md`
- `docs/notes/NOTE-031-CONTRACT-PREDICATE-WELL-FORMEDNESS-AND-SNAPSHOTS.md`
- `docs/notes/NOTE-033-SURFACE-TO-CORE-CONTRACT-LOWERING.md`
- `docs/notes/NOTE-035-TEMPORAL-AND-CONCURRENT-CONTRACTS.md`

## Summary

`SPEC-096b`, `SPEC-097b`, `SPEC-098b`, and `SPEC-100` have absorbed most contract-system
content from NOTE-031 through NOTE-035. The remaining gaps are concentrated in three places:

1. `SPEC-099b` is still mostly a Phase 159 CPS-interpreter big-step semantics document, not the
   current target operational semantics.
2. The target specs do not yet define a source-preserving surface AST and macro/notation
   substrate suitable for a future `syn`-like library.
3. The general surface-to-Core lowering story is missing. NOTE-033 covers contract predicate
   lowering, but there is no complete lowering spec for functions, `do`, handlers, impl identity,
   facts/evidence, trace contracts, notation, macros, or operator sections.

The audit recommendation is to add a companion surface spec, tentatively
`SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`, then patch `SPEC-095b` and `SPEC-099b` around
it.

## 1. Operational semantics gaps

`SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md` is the largest remaining gap.

Current evidence:

- The header says the document defines the big-step semantics of the CPS IR interpreter
  implemented in Phase 159.
- The rules are big-step-style `⇓` rules over CPS terms.
- The deferrals section still lists full contract discharge as deferred.
- Provider-frame behavior is inconsistent with newer target IR text: `SPEC-099b` skips provider
  frames in handler lookup, while `SPEC-098b` says runtime authority is represented by matching
  provider frames installed at the runtime boundary.

Missing from operational semantics:

- small-step semantics;
- structured contract trap payloads;
- false predicate versus predicate-fault separation;
- temporal monitor violation versus monitor fault separation;
- lazy and memo contract timing;
- memo replay of terminal diagnostics and blame labels;
- trace event emission and monitor observation;
- provider-frame dispatch as runtime authority;
- ambient semantic anchors (`Pure`, `Act`, `Proc`, `Workflow`) over one computation model.

Recommended split:

1. Preserve the Phase 159 CPS-interpreter big-step semantics as implementation context or an
   appendix.
2. Add target Core big-step semantics for checked Core terms.
3. Add target Core/CPS small-step semantics for control, handlers, provider frames, traps,
   lazy/memo forcing, trace facts, and runtime monitors.

## 2. Surface grammar gaps

`SPEC-095b` has absorbed much of NOTE-021, NOTE-023, NOTE-025, NOTE-026, and NOTE-031. It now
includes `Row`, `where row { ... }`, `on`, `handle expr with`, `handler` marker, `derive
handler`, bodyless nominal `type X;`, `newtype`, and restricted predicate grammar.

Remaining grammar drift:

1. `do_stmt` still includes stale inline handler syntax:

   ```ebnf
   "handle" effect_item "with" "{" { handler_arm } "}" ";"
   ```

   The example handles `requires {b != 0}` as if contract failure were a resumable handler case.
   This conflicts with NOTE-029 and the target contract model: default dynamic contract failure is
   structured bottom, not a handled effect. Recoverable contract behavior must lower to explicit
   `fail`.

2. `workflow_stmt` repeats the same stale inline handler form.

3. `SPEC-096b` defines `trace_contract_effect = "trace" trace_contract_path`, but
   `SPEC-095b`'s `contract_effect` production does not include a trace-contract effect.

4. `SPEC-095b` still says “No new operators.” This should be weakened. The initial grammar may
   avoid new built-in operators, but the surface specs must reserve room for user-defined
   notation and operator sections.

Recommended fixes:

- Remove or quarantine inline `handle effect_item with { ... }` as legacy/obsolete syntax.
- Add `trace_contract_effect` to `SPEC-095b`, or explicitly mark trace surface syntax as
  deferred.
- Replace “No new operators” with a forward-compatible notation reservation.
- Point `SPEC-095b` to a new surface AST/macro/notation spec.

## 3. Source-preserving surface AST and macro substrate gap

The target specs need a source-preserving surface AST design. This is not a parser detail. It is
an architectural requirement for future Rust-style macros and a `syn`-like Ash library.

The surface specs should guide an AST substrate that supports:

- stable source-preserving parsing;
- code-as-data;
- hygienic or at least hygiene-ready macro expansion;
- Rust-`syn`-style library ergonomics;
- user-defined notation as desugaring;
- future operator declarations without rewriting the parser or type checker.

Recommended new spec:

```text
SPEC-095c: Surface AST, Macro Expansion, and Notation
```

Recommended pipeline:

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

Recommended AST layers:

1. Token/concrete syntax layer: preserves spans, delimiters, punctuation, attributes, doc
   comments, grouping, unresolved paths, raw operators, macro invocations, and syntax sugar.
2. Surface/expanded AST layer: still source-level Ash, but after macro and notation expansion.
3. Elaborated Core boundary: rows normalized, names resolved, facts/evidence given stable IDs,
   predicates lowered to `LoweredPredicate`, operation identities resolved to `ImplType::op`,
   and source origins preserved as metadata.

Recommended surface node families:

- `Item`
- `Expr`
- `Type`
- `Pattern`
- `Row`
- `WhereItem`
- `FactDecl`
- `EvidenceDecl`
- `HandlerDecl`
- `NotationDecl`
- `MacroCall`

The parser must not prematurely collapse surface syntax into semantic Core forms. Macros and
formatters need to inspect source-level structure before desugaring.

## 4. User-defined notation and operator sections

Target Ash should support custom notation as source-level sugar over callable values.

Notation categories:

- prefix notation;
- infix notation;
- suffix/postfix notation;
- mixfix notation;
- operator sections.

The key invariant:

```text
Notation is gone before Core.
```

A notation declaration specifies the syntactic shape, precedence or associativity when relevant,
the target function/path, arity, expansion phase, name-resolution/hygiene behavior, and
import/export behavior.

Examples are proposed target syntax, not current implementation syntax:

```ash
prefix ! = not
suffix ? = is_present
infixl 6 <+> = combine
mixfix _ "between" _ "and" _ = between
```

Example expansions:

```ash
!ready        => not(ready)
value?        => is_present(value)
a <+> b       => combine(a, b)
x between lo and hi => between(x, lo, hi)
```

Recommended AST shapes before expansion:

```rust
Expr::Prefix { op, rhs, span }
Expr::Infix { lhs, op, rhs, span }
Expr::Suffix { lhs, op, span }
Expr::Mixfix { notation, holes, span }
Expr::Paren { expr, span }
```

Grouping and parentheses must be preserved, even when semantically redundant, because macros and
formatters care about source shape.

### 4.1 Operator sections

Operator sections should be part of the notation design.

Given proposed syntax:

```ash
infixl 6 <+> = combine
```

Full use:

```ash
a <+> b
```

expands to:

```ash
combine(a, b)
```

Left section:

```ash
(a <+>)
```

expands to a unary callable value equivalent to:

```ash
fn (b) -> combine(a, b)
```

Right section:

```ash
(<+> b)
```

expands to:

```ash
fn (a) -> combine(a, b)
```

Bare operator value:

```ash
(<+>)
```

resolves to the callable value denoted by the operator target, either directly as `combine` or
through an eta-expanded equivalent if the implementation needs that for arity or row inference.

Recommended AST shape:

```rust
Expr::OperatorSection {
    operator: OperatorToken,
    kind: OperatorSectionKind,
    left: Option<Box<Expr>>,
    right: Option<Box<Expr>>,
    span: Span,
}

enum OperatorSectionKind {
    Bare,
    Left,
    Right,
}
```

Typing rule:

```text
combine : (A, B) -> {r} C
--------------------------------
(a <+>) : B -> {r} C
(<+> b) : A -> {r} C
(<+>)   : (A, B) -> {r} C
```

Sections preserve the target callable's row. They do not erase effects or authority
requirements.

Initial scope should limit sections to binary infix operators. Partial application of arbitrary
mixfix forms can be designed later.

## 5. Macro compatibility constraints

A Rust-style macro system should operate on syntax, not typed Core.

Surface specs should reserve:

- macro input as token/syntax trees;
- macro output as syntax trees;
- expansion IDs;
- definition-site and call-site spans;
- hygiene-ready identifier metadata;
- source-origin metadata on generated nodes.

Even if Ash starts with simple lexical hygiene, the AST should not preclude full hygiene later.

Notation and macros should stay distinct:

- notation is restricted sugar for callable aliases;
- macros are general syntax-to-syntax expansion.

Binder-introducing mixfix should probably be macro territory at first. Simple mixfix notation
should not introduce arbitrary binders.

## 6. Contract and predicate interaction

Notation and sections may appear inside contract predicates, but they must expand before
predicate well-formedness.

Recommended predicate pipeline:

```text
predicate source tokens
  -> surface predicate AST
  -> macro expansion, if predicates admit macros
  -> notation and operator-section expansion
  -> scoped predicate expression
  -> predicate well-formedness
  -> LoweredPredicate
```

Sugar must not smuggle authority into predicates. If a notation declaration expands to an
authority-bearing operation, the predicate checker still rejects it unless the target callable is
an admitted pure predicate function.

Examples:

```ash
requires in_range {
    x between lo and hi
}
```

is allowed only when `between` resolves to an admitted pure predicate function.

```ash
requires file_exists {
    path exists?
}
```

is rejected if `exists?` expands to an authority-bearing filesystem operation.

## 7. Type inference gaps

The current specs state row and predicate checking rules, but the surface inference story remains
incomplete.

Missing algorithmic details:

- how missing rows default, infer, or come from expected callable types;
- how inline row syntax and `where row { ... }` are normalized into one callable row;
- how local facts and evidence declarations affect row requirements;
- whether a local proof automatically contributes `evidence p` to a row or only discharges an
  existing requirement;
- how handler marker subtyping is formalized in the type system;
- how abstract operation identities such as `F::read` are inferred and later specialized;
- how notation declarations participate in import/name resolution before type checking;
- how operator sections receive callable types and rows after notation resolution.

The Haskell-like model is a good fit for infix notation:

1. Parse expressions into a flat or weakly associated operator-chain shape.
2. Collect local/imported notation declarations.
3. Resolve precedence and associativity.
4. Expand notation to callable syntax.
5. Run ordinary type checking.

## 8. Lowering-to-Core gaps

NOTE-033 covers contract predicate lowering, but there is no complete general surface-to-Core
lowering spec.

Missing lowering rules:

- callable declarations with inline rows and `where row` layout;
- row inference/defaulting and public row summaries;
- `do` sequencing and row accumulation;
- handler application and `on` lowering;
- `done(value)` lowering;
- affine versus multi-shot continuation use checks;
- `derive handler` synthesis;
- operation identity lowering from generic `F::op` to concrete `ImplType::op`;
- local facts, proofs, evidence declarations, and row evidence references;
- direct contract row sugar to canonical fact/evidence/check artifacts;
- trace contracts to `TraceContract` sidecars and monitor plans;
- macro, notation, and operator-section expansion before Core.

General lowering should consume the expanded/desugared surface AST, not raw parser syntax.

## 9. Contract-system status

Contracts are comparatively well reconciled across target specs.

Already present:

- predicate grammar and classification in `SPEC-096b`, `SPEC-097b`, and `SPEC-100`;
- structured bottom and explicit `fail` recoverability boundary;
- contract subsumption and blame polarity;
- Hoare sequencing composition;
- lowered predicate sidecars in `SPEC-098b`;
- trace contract sidecars and monitor diagnostics.

Remaining contract integration gaps:

1. Fact/evidence declaration shape is still not fully normative. NOTE-021 says rows contain
   evidence requirements, but the grammar and type/lowering rules do not yet fully specify facts,
   proofs, evidence declaration, export, and local scope.
2. Contract operational semantics is missing from `SPEC-099b`.
3. Trace contract surface syntax is absent from `SPEC-095b` or at least inconsistent with
   `SPEC-096b`.
4. Lazy/memo contract timing is in the type spec but not in operational semantics or lowering.

## 10. Recommended work order

1. Patch `SPEC-095b` quick drift:
   - remove or quarantine stale inline `handle effect_item with { ... }`;
   - add or explicitly defer `trace_contract_effect`;
   - replace “No new operators” with a notation-forward reservation.
2. Add `SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`:
   - syntax-tree layers;
   - macro expansion phase;
   - notation declarations;
   - prefix/infix/suffix/mixfix forms;
   - operator sections;
   - hygiene-ready metadata;
   - desugaring invariants.
3. Add a general surface-to-Core lowering spec or expand `SPEC-098b` substantially:
   - consume expanded surface AST;
   - define lowering for functions, handlers, `do`, impls, contracts, facts/evidence, trace
     contracts, notation, macros, and sections.
4. Rewrite or split `SPEC-099b`:
   - target Core big-step semantics;
   - Core/CPS small-step semantics;
   - Phase 159 CPS interpreter semantics as appendix or implementation context.
5. Tighten surface type inference:
   - rows;
   - fact/evidence inference;
   - handler marker subtyping;
   - operation identity specialization;
   - notation and operator-section typing.

## 11. Concrete top-level gap statements

Use these statements directly when creating follow-up tasks.

### GAP A: Operational semantics target drift

`SPEC-099b` still describes the Phase 159 CPS interpreter rather than the current target language
semantics. It needs target Core/CPS big-step and small-step rules for provider frames, structured
traps, contracts, lazy/memo force, trace events, and runtime monitors.

### GAP B: Surface AST and macro substrate missing

The target specs do not define a source-preserving surface AST suitable for a future `syn`-like
library and Rust-style macros. A new spec should define syntax-tree layers, macro expansion,
hygiene-ready metadata, desugaring invariants, and the boundary before Core lowering.

### GAP C: Notation and operator sections missing

The target specs do not define custom prefix, infix, suffix, mixfix notation, or operator
sections. These should be source-level callable sugar that expands before Core while preserving
spans and origin metadata.

### GAP D: General surface-to-Core lowering missing

NOTE-033 covers contract predicate lowering, but there is no complete lowering spec for the
expanded surface AST into Core. This leaves handlers, `do`, impl identity, facts/evidence, trace
contracts, notation, macros, and operator sections without an implementation-grade bridge.

### GAP E: Fact/evidence canonical syntax incomplete

The specs acknowledge evidence rows, but fact/proof/evidence declaration syntax, identity,
export, and discharge interactions remain under-specified.

## 12. Gate conclusion

Fail for implementation-grade completeness.

The contract sidecar/type-checking story is strong enough to keep implementation moving, but the
surface and operational layers are not yet hard enough for the next parser/macro/lowering phase.
The next spec packet should focus on `SPEC-095c`, `SPEC-098b` lowering expansion, and a rewritten
or split `SPEC-099b`.


## 13. Phase 167 closeout

Phase 167 implemented the documentation-only closure path recommended by this audit:

- `SPEC-095b` now removes stale inline contract-handler target syntax, reconciles trace-contract row syntax, and reserves user-defined notation/operator sections.
- `SPEC-095c` now owns source-preserving AST, macro expansion, notation, and operator sections.
- `SPEC-098c` now owns expanded-surface-AST-to-Core lowering.
- `SPEC-097b` now includes surface inference rules for rows, evidence, handler markers, operation identity, notation, and operator sections.
- `SPEC-099b` now owns target Core/CPS operational semantics, including provider frames, structured traps, contracts, lazy/memo forcing, trace facts, and temporal monitors.

Implementation of these specs remains future work; this audit is closed for documentation gap coverage.
