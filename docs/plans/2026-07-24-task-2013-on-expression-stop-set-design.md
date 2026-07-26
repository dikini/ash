# TASK-2013 `on expr` Stop-Set Design

**Status:** User-approved bounded parser expansion
**Authority:** [SPEC-095b §4.3](../spec/SPEC-095b-TARGET-GRAMMAR.md#43-handler-expressions)
**Task:** [TASK-2013](../plan/tasks/TASK-2013-source-handler-and-handle-lowering.md), a TASK-1988 follow-up

## Decision

Replace the temporary name-only computation operand of `on` with the existing full expression
grammar.  The parser must parse:

```ash
on expr {
    ImplType::operation(pattern, resume) => body,
    done(value) => done_body,
}
```

without requiring parentheses for call, binary, or record-valued computations.  The following
top-level clause-opening `{` is a parser-local delimiter, not an inline-record or named-record
constructor suffix of the computation.

This is deliberately a parsing decision only.  It preserves the existing structural `Expr::On`
carrier and existing cardinality checks.  It does not admit source handlers to production
execution or alter their type, row, continuation, Core, CPS, provider, frame, or runtime meaning.

## The ambiguity

The normal expression parser correctly treats a brace after a name as an inline record constructor:

```ash
Result { value: run(req) }
```

That normal rule conflicts with the handler delimiter in:

```ash
on computation {
    PosixFs::read(path, resume) => body,
    done(value) => value,
}
```

If `parse_on_expr` simply calls ordinary `expr`, `primary_expr` can consume the handler block as
the record suffix of `computation`.  Conversely, treating every brace following an `on`
computation as a delimiter would make valid record-valued computations impossible.  The decision
therefore is not an unconditional brace stop: it is a context-local, clause-shaped stop-set.

## Parser model

`parse_on_expr` enters a dedicated **on-computation expression mode**.  That mode uses the same
precedence/primary/postfix expression grammar as ordinary `expr`; its only additional behavior is
at a possible inline named-record suffix after a fully parsed top-level operand:

1. At a `{`, non-consumingly inspect the first non-comment token inside the brace.
2. Treat the brace as the handler delimiter only if it starts a canonical handler clause:
   - `done` followed by `(`; or
   - an identifier followed by `::`, an identifier, and `(`.
3. Otherwise leave the brace to the ordinary expression grammar.  In particular, an identifier
   followed by `:` begins a named record constructor field and remains part of the computation.
4. Once the computation parser returns at the recognized delimiter, `parse_on_expr` consumes that
   brace and uses the existing canonical clause parser/cardinality validation unchanged.

The lookahead must inspect only enough punctuation to classify the opening form.  It must not
consume input, resolve operation names, check types, infer rows, or decide whether a clause is
ultimately valid.  A clause-shaped but malformed body remains a committed handler-clause parser
error; a non-clause-shaped record remains an ordinary computation expression.

The parse mode is lexically scoped to the top-level computation operand.  Nested expressions in
call arguments, parentheses, record fields, blocks, lists, closures, and nested handler
expressions retain their normal delimiters unless they themselves begin a nested `on`.  This
prevents an outer `on` from changing how record syntax inside its computation is parsed.

## Examples required by the decision

All of these must parse without compatibility parentheses and must preserve the full computation
AST and spans:

```ash
on run(req) { ImplType::operation(x, k) => x, done(v) => v }
on retries + 1 { ImplType::operation(x, k) => x, done(v) => v }
on { request: run(req) } { ImplType::operation(x, k) => x, done(v) => v }
on Result { value: run(req) } { ImplType::operation(x, k) => x, done(v) => v }
```

The last example is the discriminator: its first brace is an ordinary named-record constructor
and its second brace begins clauses.  The historical string-based `invoke` form remains removed;
clause identities remain symbolic `ImplType::operation` pairs.

## Invariants

1. `on` accepts every expression form already accepted by the current expression grammar, subject
   only to existing parser validity rules.
2. The handler delimiter is recognized only at the computation boundary and only by canonical
   clause shape; record syntax remains available in and as a computation.
3. `Expr::On.computation` retains the selected expression unchanged, with its normal source span;
   the enclosing `on` span and clause spans retain their current meanings.
4. Existing requirements of one or more operation clauses and exactly one `done` continue to be
   enforced unchanged at parse and constructed-AST checked-handler boundaries.
5. No parser branch introduces `invoke`, string dispatch, a compatibility syntax, or a fallback
   parse that changes the meaning of an already valid ordinary expression.
6. This remains fail closed after parsing: the existing ordinary `Expr::On` lowering/typechecking
   boundaries continue to reject unimplemented general handler semantics.

## Explicit non-goals

- Duplicate concrete-operation policy, operation resolution, handler marker admission, answer
  typing, residual-row subtraction, continuation typing/multiplicity, and `resume` semantics.
- Changes to `handle expr with identifier`, including its separate `with` delimiter handling.
- Core `Handle`/`Raise` generalization, CPS production lowering, source-handler registration,
  provider installation, handler frames, engine execution, timeout/cancellation, or CLI behavior.
- Any compatibility form, including deleted inline `handle effect_item with { ... }` or stringy
  `invoke`.

## Acceptance evidence

The parser-focused TASK-2013 suite must first demonstrate each new expression form fails under
the name-only operand, then pass after the parser-mode change.  Assertions must inspect
`Expr::On.computation` and spans rather than merely accepting source text.  Negative controls
must prove that a named record constructor is not mistaken for a handler delimiter and that the
current cardinality diagnostics remain stable.  Focused parser checks, formatting, warnings-denied
Clippy, and the existing handler declaration/lowering controls must pass without adding runtime
coverage claims.
