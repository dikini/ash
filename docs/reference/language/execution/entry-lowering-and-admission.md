# Entry Candidates, Checked Lowering, and Admission

[Execution index](index.md) · [Forms](../forms/declarations-and-functions.md) ·
[Effects and authority](../effects/index.md) · [Source of truth](../source-of-truth.md)

## Status and evidence

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| `fn main` candidate | accepted | partial | bounded-only | fixture-bounded | partial | tested | below_spec |
| Exact pure `Int` entry control | accepted | checked | lowered | admitted-executed | partial | tested | below_spec |
| Canonical zero-parameter `Result<(), RuntimeError>` entry control | accepted | checked | bounded-only | fixture-bounded | partial | tested | below_spec |
| `capability Args` entry parameter | accepted | checked | bounded-only | closed | partial | tested | below_spec |
| `Engine::run` and `Engine::run_file` | not-applicable | not-applicable | bounded-only | fixture-bounded | partial | tested | below_spec |
| `Engine::execute` and `Engine::execute_with_input` | not-applicable | not-applicable | not-applicable | closed | partial | tested | below_spec |

Primary implementation evidence is `crates/ash-engine/src/lib.rs::{admit_program,
new_admitted_program_request,execute_admitted_program,execute_entry_through_admitted_program,
run,run_file,execute,execute_with_input}` and `crates/ash-engine/src/entry.rs::{verify_entry_definition,
entry_input_bindings}`. The decisive positive and negative source evidence is
`crates/ash-engine/tests/task_1865_surface_fn_main_entry.rs`; canonical signature tests are in
`crates/ash-engine/tests/entry_verification.rs`.

## What an entry is now

`fn main` is an ordinary named-function declaration recognized by the module parser. It is a
*source candidate*, not an instruction to execute arbitrary function syntax. Parsing and checking
can retain callable and row metadata for source that the production boundary will later reject.

The Engine's production boundary is explicitly
`ProductionExecutionBoundary::CheckedCoreCpsClosedAdmission`. A source entry becomes executable
only when the same Engine has retained the canonical parse provenance, checked the source, and
materialized one of its sealed checked Core/CPS admission artifacts. `admit_program` is the
operation that makes that decision. On success, only that Engine can create an
`AdmittedProgramRequest`, and `execute_admitted_program` is the shared executor for the request.

This is an admission boundary, not an inference rule from source syntax. In particular, an
ordinary function definition, a `where row` annotation, a capability type, a resource/role name,
or a handler declaration cannot manufacture the sealed artifact or authorize a frame. Rows remain
requirements; see [the effects authority boundary](../effects/index.md#scope-boundary).

## Examples

### Exact admitted pure fixture

This is the bounded positive source route in
`task_1865_surface_fn_main_entry.rs`: it parses, checks, receives the selected pure checked
Core/CPS lowering, is admitted by its issuing Engine, and returns `Int(42)`.

```ash
fn main() -> Int {
    do {
        return 42;
    }
}
```

This exact evidence does **not** make all `Int`-returning main functions executable. The same test
parses and checks a larger entry with records, ADTs, `match`, helper calls, and `do`, then confirms
that the checked-CPS admission boundary rejects it. No direct evaluator is selected after that
rejection.

### Canonical metadata contract — verification evidence only

The entry verifier separately recognizes a canonical application-entry declaration. The return
must be exactly `Result<(), RuntimeError>` and every parameter must be a source capability type.
The zero-parameter `Ok` shape is bootstrapped to exit `0` by `entry_verification`; that is one
fixture-bounded route, not proof that every verified body has production lowering. The following
parameterized spelling is parser/checker/verifier evidence only.

```ash
use result::Result
use runtime::RuntimeError
use runtime::Args

fn main(args: capability Args) -> Result<(), RuntimeError> {
    Ok { value: {} }
}
```

`Args` is not an ambient authority grant. `entry_input_bindings` turns a verified capability
parameter into an input binding only inside the canonical-entry path; it neither installs a
provider nor bypasses checked-CPS admission. In the current bootstrap implementation, a nonempty
input-binding map reaches the closed `execute_with_input` API, so the parameterized spelling has
verification evidence but no admitted runtime example. Non-capability parameters and a return
type such as `Int` fail entry verification even though the parser accepts the function declaration.

### A checked but unadmitted rich entry

The following kind of source may parse and typecheck as an ordinary function program, but it is
outside the sealed production lowering used by `Engine::run`:

```ash
type Lookup = Found { age: Int } | Missing;

fn score(lookup: Lookup) -> Int {
    match lookup {
        Found { age: age } => age,
        Missing => 0,
    }
}

fn main() -> Int {
    score(Found { age: 42 })
}
```

This is a boundary illustration based on the rich negative fixture, not an executable example.
Its rejection is at checked Core/CPS admission; it must not fall back to an older expression
evaluator. The exact fixture has more declarations and a `do` block, so this shortened form is
not a substitute for the test control.

## Syntax

The parser accepts an ordinary function declaration for a main *candidate*. The stronger
canonical-entry return and parameter conditions are verification conditions, so they do not appear
as grammar restrictions here.

```ebnf
entry_candidate = [ visibility ] "fn" "main" [ type_parameters ] "(" [ parameter { "," parameter } ] ")" [ "->" type ] [ proposition_tail ] { requires_clause } { ensures_clause } function_body ;
parameter = identifier ":" type ;
function_body = "{" block_contents "}" ;
```

`visibility`, `type_parameters`, `type`, contract clauses, and block contents are the ordinary
function grammar documented in [Declarations and Functions](../forms/declarations-and-functions.md#syntax).
The EBNF deliberately says neither that `main` must return `Result` nor that its parameters must
be capabilities: those are imposed by `verify_entry_definition`, not by parser acceptance.

## Route semantics and closed generic APIs

The observable route is a guarded Engine transition rather than a general source-language
reduction rule:

```text
source bytes
  → Engine parse and source provenance
  → check
  → sealed checked Core/CPS materialization
  → admit_program
  → Engine-issued AdmittedProgramRequest
  → execute_admitted_program
  → CanonicalTerminalEnvelopeV1
```

Each arrow may reject. Admission requires an artifact issued by the same Engine and validates the
artifact's canonical source anchor and checked facts. The request's cancellation handle is
non-authorizing; it controls a request already admitted by its issuing Engine.

`Engine::run` and `Engine::run_file` parse source (or a file), then use the private
`execute_entry_through_admitted_program` helper. That helper calls `admit_program`, creates an
Engine-issued request, and calls `execute_admitted_program`. It does not call the public generic
`Engine::execute` method.

Conversely, `Engine::execute(&Entry)` and `Engine::execute_with_input(&Entry, bindings)` are
explicitly closed: both return the checked-Core/CPS closed-admission error. Supplying input
bindings does not turn a parsed `Entry` into an admitted program. `run_file_with_input` performs
the parse/check path and then reaches that same closed `execute_with_input` boundary.

No sequent is given here. The implementation supplies a provenance- and issuer-guarded admission
protocol, not a general source-level entry calculus whose premises and conclusion would be
faithfully represented by a sequent.

## Diagnostics and boundaries

- A missing validated checked Core/CPS artifact is an admission failure, not evidence that source
  execution should use a direct evaluator.
- A forged, foreign-Engine, malformed, or provenance-mismatched checked artifact is rejected by
  the sealed verification boundary; its normalized terminal form is
  `InvalidCheckedArtifact` when the public admitted-program seam can classify it.
- The exact canonical entry contract is `main` with return type `Result<(), RuntimeError>` and
  zero or more `capability Name` parameters. It is narrower than general `fn main` syntax.
- The runtime-entry library registry is deliberately narrow. Its accepted imports and entry
  signature verification do not demonstrate arbitrary library execution; see the library chapter
  when it is added.
- Neither this route nor the canonical entry contract revives removed workflow/tower entry syntax.

## Related evidence

- [TASK-1865 function-first entry fixture](../../../plan/tasks/TASK-1865-fn-main-entry-adapter.md)
- [TASK-2052](../../../plan/tasks/TASK-2052-language-reference-entry-engine-clients-terminals.md)
- `cargo test -p ash-engine --test task_1865_surface_fn_main_entry`
- `cargo test -p ash-engine --test entry_verification`
- `crates/ash-engine/src/lib.rs::ProductionExecutionBoundary`
