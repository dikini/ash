# Core/CPS Continuation Multiplicity

This page documents the implemented Phase 164 continuation multiplicity behavior for Core Ash and
CPS IR. It is a reference for the current compiler/runtime substrate, not a surface Ash syntax
proposal. This is not a surface Ash syntax proposal.

Normative behavior is specified by [SPEC-102](../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md)
and tracked by [PLAN-164](../plan/PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md). Non-normative
design background lives in [multi-shot-continuations.md](../design/multi-shot-continuations.md) and
[NOTE-012-MUTUAL-RECURSION-CPS-ASPECTS-DESIGN.md](../notes/NOTE-012-MUTUAL-RECURSION-CPS-ASPECTS-DESIGN.md).

## Behavior

Core continuation types carry explicit multiplicity:

```text
(cont A Ans Row affine)
(cont A Ans {} multi-shot-pure)
```

`affine` continuations may be invoked at most once. CPS runtime keeps the existing consumed-flag
behavior and traps a second invocation.

`multi-shot-pure` continuations may be invoked repeatedly, but only when the Core continuation type
explicitly uses `multi-shot-pure` and the row is the closed empty row `{}`. An empty row by itself
does not imply multi-shot behavior; empty row by itself does not imply multi-shot. Non-empty rows
and open rows such as `{tail r}` are rejected for `multi-shot-pure`.

## Core Text

`.core` remains a fixture/debug format, not surface syntax. Phase 164 adds an answer-binding
continuation invocation form:

```text
(let-cont-call answer resume (lit-int 1) answer)
```

This invokes a continuation, binds its answer to `answer`, then evaluates the body expression. It is
used by Core fixtures for handlers that resume more than once and inspect intermediate answers.

## Lowering

Checked Core-to-CPS lowering preserves continuation facts:

- Core handler resume rows lower to CPS `HandlerClause.resume_row = Known(row)`.
- Core handler resume multiplicity lowers to `HandlerClause.resume_multiplicity`.
- Core `let-cont-call` lowers to CPS `Term::LetContCall` with checked row accounting.
- Generated CPS `Term::LetCont` carries the checked continuation row and remains affine unless the
  source Core continuation type explicitly carries `multi-shot-pure`.

Unchecked or legacy serialized CPS inputs remain conservative: omitted multiplicity defaults to
`Affine`, and omitted handler resume rows deserialize as inherit-from-target compatibility metadata.

## Fixtures

The committed Phase 164 Core fixture names are:

- `multishot_resume_text_roundtrip.core`
- `affine_empty_row_remains_affine.core`
- `invalid_multishot_nonempty_row.core`
- `invalid_multishot_open_row.core`
- `let_cont_call_text_roundtrip.core`
- `motivational_choice_all_outcomes.core`
- `motivational_backtracking_find_first.core`
- `motivational_nested_choice.core`
- `motivational_discard_resume.core`
- `motivational_affine_choice_all_outcomes_invalid.core`
- `motivational_effectful_multishot_invalid.core`

These fixtures intentionally use current `.core` syntax only. Surface syntax for user-facing
choice/search handlers is informational and out of scope for Phase 164.
