---
id: language.reference.effects.comprehensions
title: Bracket Comprehensions
kind: feature-reference
status: partial
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["crates/ash-parser/src/parse_expr.rs", "crates/ash-typeck/src/**", "crates/ash-engine/tests/task_1024_stdlib_do_evidence.rs"]
---

# Bracket Comprehensions

[Effects index](index.md) · [Handlers, scoped failure, and `do`](handlers-failure-and-do.md) ·
[Language reference](../index.md)

## Support

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| Bracket comprehension with bind, discard, and `let` qualifiers | accepted | partial | rejected | closed | partial | tested | below_spec |
| Explicit comprehension target | accepted | partial | rejected | closed | partial | tested | below_spec |
| Unannotated comprehension | accepted | partial | rejected | closed | partial | tested | below_spec |

The parser accepts comprehensions. The type checker can elaborate a selected comprehension when
the caller supplies the target and its method evidence. Ordinary lowering rejects every
comprehension. It never guesses a target or a generic bind operation.

Focused evidence:

- `crates/ash-parser/tests/task_755_comprehension_parser.rs`
- `crates/ash-typeck/tests/task_1024_do_and_comprehension_stdlib_evidence.rs`
- `crates/ash-engine/tests/task_1024_stdlib_do_evidence.rs`

The Engine test registers selected stdlib `Monad` methods for typed `do`. It does not run a source
comprehension.

## What a comprehension is

A bracket comprehension has a result expression before `|`, followed by one or more qualifiers.
A qualifier is one of:

- `name <- expression`, which binds a name;
- `_ <- expression`, which discards the bound value; or
- `let name = expression`, which introduces a pure local name.

The optional `: Target` after `]` records an explicit target. Without it, the parser retains no
target. Parsing distinguishes this form from a list literal and from indexing: the `|` is required
for a comprehension. It rejects an empty qualifier list, a trailing qualifier comma, a bare
Boolean-like qualifier, and a malformed target annotation.

**Parser-tested source example; no runtime claim.**

```ash
[result | raw <- read(path), let parsed = parse(raw), _ <- guard(parsed)]: Result<ParseError>
```

This shows the grammar only. It does not show that the named functions or the expression run.

## Static elaboration boundary

The selected static evidence constructs an explicit `Option` comprehension and checks it in an
environment prepared with stdlib interface and implementation evidence. Its elaboration is equal
to the corresponding target-annotated `do` block and preserves selected `Monad<Option>` method
evidence. This narrow relationship is useful for checking the elaborator; it is not a general
source-to-Engine route.

**Static-elaboration shape; not a runnable program.**

```ash
[x | x <- option::pure(1)]: Option
```

```sequent
TypedComprehensionElaboration :=
  [ elaborate_typed_do(target, qualifiers, result) = elaborated ] [ selected_evidence(target) = evidence ]
  ===>
  elaborate_typed_comprehension([ result | qualifiers ] : target) = elaborated with evidence
```

The rule names the tested elaborator correspondence only. It does not say that any target exists
at runtime, that a row grants a handler/provider, or that a selected interface implementation is
an Engine admission token.

## Syntax

The parser requires at least one qualifier. A type hole is accepted only in the local
target-argument grammar; it is not a general source-type-hole rule. The shared `expression` and
`do_target` domains are described with `do` on
[handlers, scoped failure, and `do`](handlers-failure-and-do.md).
The explicit non-copyable exclusion for legacy target names on that page applies equally to a
comprehension's optional `: do_target` annotation; the EBNF head is a structural approximation.

```ebnf
comprehension_expression = "[" expression "|" comprehension_qualifier { "," comprehension_qualifier } "]" [ ":" do_target ] ;
comprehension_qualifier = bind_qualifier | discard_bind_qualifier | let_qualifier ;
bind_qualifier = identifier "<-" expression ;
discard_bind_qualifier = "_" "<-" expression ;
let_qualifier = "let" identifier "=" expression ;
do_target = identifier [ "<" [ do_target_type { "," do_target_type } ] ">" ] ;
do_target_type = identifier [ "<" [ do_target_type { "," do_target_type } ] ">" ] | "_" ;
```

## What works now

Raw lowering reports “comprehension requires typed do elaboration before lowering.” The existing
typed tests do not add a general lowering or Engine execution path. A targetless comprehension
only proves that the parser accepts the syntax.

When a target and stdlib evidence are deliberately prepared, static elaboration still needs
resolved methods with matching types. Target resolution, type arguments, qualifier typing, and
missing evidence all remain checker responsibilities. Neither a comprehension target nor a row
is authority or a handler frame.

## Related evidence

- [AUDIT-206 LANG-014](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
- [TASK-2051](../../../plan/tasks/TASK-2051-language-reference-handlers-failure-do-comprehensions.md)
- `cargo test -p ash-parser --test task_755_comprehension_parser`
- `cargo test -p ash-typeck --test task_1024_do_and_comprehension_stdlib_evidence`
- `cargo test -p ash-engine --test task_1024_stdlib_do_evidence`
