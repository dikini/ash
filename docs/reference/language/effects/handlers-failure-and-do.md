# Handlers, Scoped Failure, and `do`

[Effects index](index.md) · [Comprehensions](comprehensions.md) ·
[Rows and operations](rows-aliases-groups-and-operations.md) ·
[Language reference](../index.md)

## Status and evidence

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| `handler` declaration and canonical `on` body | accepted | partial | bounded-only | fixture-bounded | partial | tested | below_spec |
| `handle expression with handler_name` | accepted | partial | bounded-only | fixture-bounded | partial | tested | below_spec |
| `fail payload` | accepted | checked | lowered | closed | partial | tested | below_spec |
| `with_error { body } handle { ... }` | accepted | checked | lowered | closed | partial | tested | below_spec |
| Exact ambient `do { return 42; }` entry fixture | accepted | checked | lowered | fixture-bounded | partial | tested | below_spec |
| Ambient `<-` binding and richer sequences | accepted | checked | lowered | closed | partial | tested | below_spec |
| Target-annotated `do:K { ... }` | accepted | partial | rejected | closed | partial | tested | below_spec |

The parser routes are `crates/ash-parser/src/parse_module/fn_defs.rs::parse_handler_declaration`
and `crates/ash-parser/src/parse_expr.rs::{parse_on_expr,parse_handle_with_expr,parse_with_error_expr,parse_do_block_expr}`.
The ordinary surface lowerer is intentionally narrower: raw `on` and `handle` expressions reject
until a typed handler bridge is selected, target-annotated `do` rejects until typed-do elaboration,
and only ambient `do` has a direct local lowering. `fail` and `with_error` lower to legacy Core
failure carriers; that carrier is not an Engine admission or execution route.

Focused evidence:

- `crates/ash-parser/tests/task_2013_handler_surface.rs`
- `crates/ash-typeck/tests/task_2013_handler_core_lowering.rs`
- `crates/ash-engine/tests/task_2014_handler_production_admission.rs`
- `crates/ash-engine/tests/task_2013_deep_affine_handler_semantics.rs`
- `crates/ash-engine/tests/task_2026_forward_sleep_production_admission.rs`
- `crates/ash-parser/tests/task_708_fail_with_error.rs`
- `crates/ash-typeck/tests/task_708_operational_bottom.rs`
- `crates/ash-typeck/tests/task_1006_with_error_total_handlers.rs`
- `crates/ash-typeck/tests/task_1841_ambient_do.rs`
- `crates/ash-typeck/tests/task_1024_do_and_comprehension_stdlib_evidence.rs`
- `crates/ash-engine/tests/task_1865_surface_fn_main_entry.rs`

Rows are requirement metadata only. In particular, a row, a checked handler fact, or a handler
name does not install a handler frame, select a provider, or mint runtime authority. See
[rows, aliases, groups, and operations](rows-aliases-groups-and-operations.md) for that
non-granting boundary.

## Handler declarations, `on`, and `handle … with`

Use `handler` to declare a callable with a function-like signature. The bounded checked/lowering
routes documented here use an `on` expression as their canonical body. `on` first names a
computation expression, then gives concrete operation clauses of the form
`Implementation::operation(pattern, resume) => expression`, together with one
`done(binding) => expression` clause. The parser requires at least one concrete operation clause
and exactly one `done` clause. It preserves all clause order; it does not turn the source row into
a fresh set of frames.

Use `handle expression with handler_name` to apply a named handler. Parsing records the expression
and the handler name without resolving it. The checked route compares the handler's implicit
computation input and normalized row with the handled expression. It rejects a mismatched result
type or operation row before a typed handler fact can reach the narrow lowering bridge.

**Fixture-bounded executable example.** This is the exact `absorb_sleep` shape exercised by
`task_2014_handler_production_admission.rs`. It is evidence for this sealed source fixture only;
it is not a general recipe for admitted handlers.

```ash
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }

handler absorb_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => resume(ms),
        done(value) => value,
    }
}

fn main() -> Int { handle TestClock::sleep(0) with absorb_sleep }
```

The Engine also has separately sealed test routes named `trap_sleep`, `deep_affine_clock`, and
`forward_sleep`. They establish their individual checked Core/CPS admission cases, including an
abortive trap, deep affine reinstatement, and one forward-to-provider case. They do not establish
arbitrary handler names, clause collections, continuation use, residual rows, provider selection,
or client parity. In particular, generic `Engine::execute` and `execute_with_input` remain closed
for a checked handler entry in the `absorb_sleep` test.

### Narrow typed handler bridge

The ordinary `lower_expr` path rejects source `on` and `handle` outright. The separate
`ash_typeck::lower_checked_handler_application_to_core` inspection bridge admits only a much
narrower typed fact: a tail `handle … with` application in the selected entry, exactly one
operation clause, a canonical `on` body, and the bridge's supported closed/explicitly sealed
residual and clause shapes. For its ordinary identity slice the `done` clause must return its
binding unchanged. A multiple-clause handler, an open residual row, an arbitrary done body, or an
unsupported continuation body rejects instead of being simplified into a Core frame.

```sequent
TypedHandlerBridge :=
  [ checked_handler_application(entry, handler) = fact ] [ exactly_one_operation(fact) ] [ supported_handler_shape(fact) ]
  ===>
  lower_checked_handler_application_to_core(entry, fact) = CoreHandle(fact)
```

This is a rule for the private, checked bridge, not a general source-handler operational
semantics. The `CoreHandle` result alone does not execute a handler or grant authority; the
separately checked Engine fixture route still has to issue its opaque admission.

## Scoped failure

`fail payload` is a source expression. The checker gives it a fresh, bottom-compatible result
type, so it can unify with the other branch of a checked expression. It does not make every
payload type or surrounding program an admitted failure computation.

`with_error { body } handle { pattern => expression; ... }` scopes pattern arms around a block
body. Each arm must typecheck against the body result type; pattern errors are reported by the
ordinary pattern route. An arm whose body is itself `fail` is bottom-compatible with the enclosing
result. The parser accepts an empty arm block as well as zero or more arms with optional trailing
semicolons, but a useful scoped failure handler needs arms that the checker can validate. Coverage
is an additional checker boundary: for a known closed failure payload, empty or non-exhaustive arms
produce `NonExhaustiveWithErrorHandler`; when the payload universe is unavailable, blocked, or
unsupported, an empty or constructor-specific (non-wildcard) arm collection can instead produce
`WithErrorHandlerCoverageDeferred`. A wildcard or variable arm is universal and avoids this
coverage gap. These outcomes are checked diagnostics, not runtime recovery behavior.

**Checked and lowered fragment; no Engine runtime claim.** The parser and checker tests cover
this fragment and `lower_expr` produces `CoreExpr::WithError` containing `CoreExpr::Fail`.

```ash
with_error { fail "boom" } handle { _ => 1; }
```

```sequent
FailBottomType :=
  [ GAMMA |- payload : P ]
  ===>
  GAMMA |- fail payload : fresh_alpha
```

`fresh_alpha` denotes the fresh inference variable exercised by the checker; the rule states its
bottom-compatible static role, not a runtime failure transition. The Core `Fail`, `WithError`, and
any CPS `Raise` carrier are internal lowering representations. There is no source `raise` form in
the active parser, so this page does not define one.

## Ambient `do`

Ambient `do { ... }` is local sequencing syntax. It accepts `let` bindings, `<-` bindings,
semicolon-terminated expression statements, and a final `return expression`. In this ambient
route, `<-` is not a Monad bind: the checker gives the bound expression's result type directly to
the local name, and lowering turns both `let` and `<-` into Core `Let` bindings. The lowerer
requires a nonempty block with a final `return`; it rejects a return before later statements.

**Fixture-bounded executable entry.** The Engine runs this exact `fn main` source in
`task_1865_surface_fn_main_entry.rs` and returns `42`. This is the complete runtime claim for
ambient `do`; it does not cover a different return value, a local binding, a helper, or a richer
body.

```ash
fn main() -> Int {
    do {
        return 42;
    }
}
```

**Checked and lowered source fragment; closed at admission.** The `<-` example is covered by the
ambient checker/lowerer tests, but no Engine execution route is evidenced for it.

```ash
do {
    x <- 1;
    return x
}
```

```sequent
AmbientDoBindLower :=
  [ lower(value) = V ] [ lower(rest) = R ]
  ===>
  lower(do { x <- value; rest }) = let x = V in R
```

This rule applies only to the ambient lowering slice and only when the remaining block satisfies
the final-return condition. It neither resolves a type-class operation nor produces an effect
row, handler frame, provider, or admission token.

The distinct `task_2003_local_call_core_cps_lowering.rs` fixture shows a local helper containing
`do { return 7; }` reach checked CPS lowering, not Engine execution. A richer main-body sequence
with `<-`, records, calls, and a final return is checked/lowered by
`task_1865_surface_fn_main_entry.rs` but fails the checked Core/CPS admission boundary. Thus the
runtime status is fixture-bounded for the exact `42` entry and closed for ambient binding or
richer sequencing; neither result makes `<-` monadic.

## Target-annotated `do`

`do:K { ... }` retains an explicit target name and optional simple type arguments. Its typed
elaborator has selected evidence paths for `Option` and `Result<_, String>` using registered
`Monad` interface evidence. That is a static elaboration result. The ordinary surface lowerer
rejects every target-annotated block with “generic do block requires typed do elaboration before
lowering”, and the normal Engine route does not admit it.

**Static-elaboration example; not a runnable source program.** It requires the prepared stdlib
evidence environment from `task_1024_do_and_comprehension_stdlib_evidence.rs`.

```ash
do:Option {
    x <- option::pure(1);
    return x
}
```

The target parser accepts a name with optional nested simple type arguments and locally permits a
type hole in this target position. Target interpretation, interface selection, and all runtime
behavior remain bounded by the named tests.

**Excluded target names; not source examples.** `parse_do_target` rejects `Act`, `Proc`, and
`Workflow` before typechecking. The generic `identifier` head in the EBNF below records the
accepted structural shape, not permission to use those explicitly rejected names.

## Syntax

The following grammar records the accepted surface shapes. `expression`, `pattern`,
`function_body`, `surface_type`, and `computation_row` are shared parser domains documented in
their owning chapters. The `on` cardinality rules and the final-return rule for a `do` block are
parser/lowerer side conditions described above rather than EBNF-only constraints.

```ebnf
handler_declaration = [ visibility ] "handler" callable_name [ type_parameters ] "(" [ parameter { "," parameter } [ "," ] ] ")" "->" surface_type [ proposition_tail ] [ contract ] function_body ;
on_expression = "on" expression "{" handler_clause { [ handler_clause_separator ] handler_clause } [ handler_clause_separator ] "}" ;
handler_clause = operation_clause | done_clause ;
operation_clause = identifier "::" identifier "(" pattern "," identifier ")" "=>" expression ;
done_clause = "done" "(" identifier ")" "=>" expression ;
handler_clause_separator = "," | ";" ;
handle_with_expression = "handle" expression "with" identifier ;
fail_expression = "fail" expression ;
with_error_expression = "with_error" function_body "handle" "{" [ error_arm { [ ";" ] error_arm } [ ";" ] ] "}" ;
error_arm = pattern "=>" expression ;
do_block_expression = "do" [ ":" do_target ] "{" [ do_statement { do_statement } ] "}" ;
do_target = identifier [ "<" [ do_target_type { "," do_target_type } ] ">" ] ;
do_target_type = identifier [ "<" [ do_target_type { "," do_target_type } ] ">" ] | "_" ;
do_statement = "let" identifier "=" expression ";" | identifier "<-" expression ";" | expression ";" | "return" expression [ ";" ] ;
visibility = "pub" | "pub" "(" "crate" ")" ;
```

## Diagnostics and boundaries

- An `on` body without an operation clause, without `done`, or with more than one `done` clause
  is rejected by the parser. A checked handler application with a mismatched result or normalized
  row is rejected by the typechecker.
- Raw source lowering rejects `on` and `handle` rather than synthesizing a general Core handler.
  The typed bridge rejects unsupported facts before it can erase operation, continuation, or
  residual-row information.
- `fail` and `with_error` have parser/typechecker/Core-carrier evidence, but no general admitted
  Engine runtime route. Do not treat their internal CPS carrier as source syntax.
- Ambient `do` rejects empty or non-final-return blocks at lowering. Its sole Engine execution
  evidence is the exact literal-return `fn main` fixture above; ambient `<-`/richer sequences are
  closed at admission and never implement monadic binding. Target-annotated `do` needs the
  separate typed elaborator and rejects ordinary lowering.
- [Comprehensions](comprehensions.md) reuse the target/evidence boundary but have no direct
  ordinary lowering route.

## Related evidence

- [AUDIT-206 LANG-013, LANG-014, and LANG-022](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
- [TASK-2051](../../../plan/tasks/TASK-2051-language-reference-handlers-failure-do-comprehensions.md)
- `cargo test -p ash-parser --test task_2013_handler_surface --test task_708_fail_with_error --test task_755_comprehension_parser`
- `cargo test -p ash-typeck --test task_2013_handler_core_lowering --test task_708_operational_bottom --test task_1006_with_error_total_handlers --test task_1841_ambient_do --test task_1024_do_and_comprehension_stdlib_evidence`
- `cargo test -p ash-engine --test task_2014_handler_production_admission --test task_2013_deep_affine_handler_semantics --test task_2026_forward_sleep_production_admission --test task_1024_stdlib_do_evidence --test task_1865_surface_fn_main_entry`
