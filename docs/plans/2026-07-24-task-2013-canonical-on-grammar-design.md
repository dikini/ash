# TASK-2013 Canonical `on` Grammar Design

**Status:** Approved bounded implementation slice
**Authority:** [SPEC-095b §4.3](../spec/SPEC-095b-TARGET-GRAMMAR.md#43-handler-expressions)
**Task:** [TASK-2013](../plan/tasks/TASK-2013-source-handler-and-handle-lowering.md) (follow-up identified by TASK-1988)

## Decision

Implement only structural cardinality for the target `on` eliminator. A canonical
`on computation { ... }` has:

1. at least one concrete operation clause; and
2. exactly one `done(value) => body` clause.

Operation clauses retain the source-preserved `ImplType::operation` pair, rather than a stringly
`invoke` name. `PosixFs::read(path, resume) => body` is therefore the normal, symbolically
resolvable clause form. This bounded decision deliberately does **not** impose a duplicate
concrete-operation rule; repeated operation clauses remain outside this slice.

The existing `handler_clause+` grammar is insufficient: it admits a handler with only `done` and
more than one `done`. The approved slice makes those forms invalid before they can be accepted by
a later handler runtime.

## Boundary and ownership

| Concern | Owner in this slice | Required result |
|---|---|---|
| Parse `on`/`done`; preserve existing clause source spans and `ImplType::operation` carriers | `ash-parser` | Valid canonical syntax produces the existing structured surface `Expr::On`; malformed source receives a deterministic parse error. |
| Defend cardinality for handler declarations entering checked-handler facts | `ash-typeck` | A checked handler cannot contain zero operation clauses or other than one `done` clause. |
| Operation identity | Existing parser and declaration checker preserve/resolve the qualifier and name | Identity remains symbolic and resolvable; this slice adds no uniqueness rule. |

The parser is the first rejection boundary for source text. The checked-handler declaration pass
repeats the cardinality checks before it builds declaration facts used by the existing bounded
inspection bridge. The general expression checker's direct `Expr::On` path remains unsupported;
it is not a useful validation boundary for arbitrary handler facts. Parser diagnostics take
priority for ordinary source. Neither boundary evaluates a computation, creates a handler frame,
or changes residual-row, Core, CPS, or runtime behavior.

## Deterministic failures

The implementation must give a stable primary failure classification and span for each malformed
shape. It must not report a later generic expression, lowering, or runtime error instead.

| Source shape | Boundary | Diagnostic class / stable subject |
|---|---|---|
| `on computation { done(v) => v }` | parser, then checked-handler declaration validation | missing concrete operation clause |
| `on computation { PosixFs::read(path, k) => k(path) }` | parser, then checked-handler declaration validation | missing `done` clause |
| two `done` clauses | parser, then checked-handler declaration validation | duplicate `done` clause; point at the second `done` |

Diagnostics must not fall back to deleted string-dispatch or `invoke` terminology. Error
selection is source-order deterministic: the second `done` is blamed, while missing forms report
their enclosing `on` expression deterministically.

## Explicit non-goals

This does **not** broaden the computation expression accepted after `on`; define handler answer
typing; infer or subtract residual rows; enforce continuation multiplicity; lower `Expr::On` to
Core/CPS; create interpreter handler frames; or execute handlers. Those remain TASK-2014 and the
existing Core/CPS and row-semantics tracks. It also does not change `handle ... with`, `with_error`,
provider admission, module summaries, legacy compatibility policy, or duplicate
concrete-operation semantics.
