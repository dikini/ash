# NOTE-021: Row, Callable, Where, and Fact Syntax

**Date:** 2026-06-27
**Status:** Living document -- syntax exploration for follow-up specs
**Purpose:** Record the current target direction for computation-row spelling, row-bearing
callable types, expanded `where` syntax for heavy signatures, and common declaration shape
for named predicate/proof facts. This note is intentionally pre-spec: it captures usability
and consistency decisions before SPEC-095b, SPEC-096b, and SPEC-097b are revised.

Companion to NOTE-013 through NOTE-020, especially NOTE-020's computation-row taxonomy and
NOTE-014's contract/evidence boundary.

## Pre-Spec Delta

This note intentionally differs from the current target specs in several places. When the
project moves from notes to spec updates, reconcile at least these deltas:

- **Row kind spelling:** NOTE-021 uses `Row` as the source kind. SPEC-095b/SPEC-097b still
  use `EffectRow`.
- **Prose terminology:** NOTE-020/NOTE-021 use "computation row" for the type-level row
  concept. SPEC-095b/SPEC-096b/SPEC-097b still mostly say "effect row."
- **Expanded row layout:** NOTE-021 adds `where row { ... }` as an alternate layout for the
  callable type row. SPEC-095b currently describes only inline row syntax in function types.
- **Duplicate row spelling:** NOTE-021 rejects using both inline row syntax and
  `where row { ... }` on the same callable. The specs do not yet state this rule.
- **Where ordering/defaulting:** NOTE-021 treats `where` items as unordered and optional;
  missing rows are inferred, defaulted to empty where permitted, or checked from the expected
  callable type. The specs do not yet define this.
- **Evidence rows:** NOTE-021 says source rows contain evidence requirements, where evidence
  denotes a fact plus proof/check/record. SPEC-096b/SPEC-097b still allow direct contract/law
  row items in examples.
- **Fact/proof grammar:** NOTE-021 sketches a shared `requires`/`ensures`/`invariant`/`law`/
  `proof` declaration shape, but exact grammar and semantics are deferred to a separate
  facts/evidence/obligations track.

## 0. Motivation

Target Ash needs row-bearing callable types, but rows can become large. A small row is easy
to read inline:

```ash
fn read(path: Path) -> {fs.read} String
```

A large row that mixes operations, failure, policy, evidence, and lifecycle facts is still
semantically just a list, but it is not a pleasant inline list. Ash therefore needs two
equivalent authoring forms:

1. a compact inline row for simple callable types;
2. an expanded `where` form for heavy callable signatures.

The syntax must remain familiar to humans and agents. The design target is not novelty. It is
ordinary lexical scoping, ordinary module qualification, and a small number of repeated
shapes that agents can classify cold.

## 1. Settled Direction

The current direction is:

- Inline row syntax remains `A -> {row} B` and `fn f(...) -> {row} B`.
- Heavy rows may move to a post-signature `where` section with a `row { ... }` block.
  This is an alternate layout for the callable type row, not a second row.
- Predicate-like artifacts are named declarations, even when local to one callable.
- `requires`, `ensures`, `invariant`, `law`, and `proof` use one common declaration shape.
- Rows contain evidence requirements, not raw predicate/law bodies.
- Lexical scoping defines shadowing. Module paths define nonlocal qualification.
- No special function-scope escape syntax such as `outer::local_fact` is introduced.
- Row tails are explicit only for row-polymorphic callables, using `| r` as the final row
  entry.

## 2. Inline Callable Rows

The compact form remains the ordinary notation for small rows:

```ash
fn parse(input: String) -> {fail ParseError} Ast

fn read(path: Path) -> {fs.read} String

fn map<A, B, r: Row>(
    xs: List<A>,
    f: A -> {r} B,
) -> {r} List<B>
```

Read:

```text
A -> {rho} B
```

as:

```text
A -> Ash<rho, B>
```

The row sits between input and output because it describes the computation needed to get
from the input to the output. This is the most compact user surface and maps directly to the
semantic boundary.

The user-facing row kind is `Row`, not `EffectRow`, because computation rows include more
than ordinary effect operations. In prose, "computation row" names the concept; in source,
`Row` is the brief kind name.

## 3. Expanded Where Rows

Rows belong to callable types. For heavy signatures, the return type should stay visible and
the row may move into `where` as an expanded layout of that same callable type row:

```ash
fn process(req: Request) -> Response
where
    row {
        http.get,
        llm.complete,
        policy request_policy,
        fail ProcessError,
        evidence response_contract_check,
    }
{
    ...
}
```

The compact and expanded forms are equivalent:

```ash
fn read(path: Path) -> {fs.read} String
```

is equivalent to:

```ash
fn read(path: Path) -> String
where
    row {
        fs.read,
    }
```

Suggested rules:

- A callable may have at most one explicit `row { ... }` block.
- A callable may specify its row in exactly one place: inline in the callable type or expanded
  as `where row { ... }`.
- Specifying both an inline row and an expanded `where row { ... }` block is an error, even if
  the rows are textually or semantically equivalent.
- `row { ... }` is an unordered item inside the callable's `where` section. Its order relative
  to other `where` items does not affect meaning.
- Every supported `where` item is optional. If neither inline row syntax nor `where row { ... }`
  appears, the row is inferred, defaulted to empty where inference permits, or checked from the
  expected callable type.
- `| r` is permitted only as the final row entry.
- Inline row syntax is preferred for short rows and higher-order callable parameters.
- Expanded `where row { ... }` is preferred for large rows or rows with evidence/lifecycle
  facts.

Invalid duplicate row spelling:

```ash
fn read(path: Path) -> {fs.read} String
where
    row {
        fs.read,
    }
{
    ...
}
```

Suggested diagnostic:

```text
row specified twice for callable `read`
  inline row appears in return type
  expanded row appears in where clause
choose one spelling
```

Other `where` items do not count as row spellings:

```ash
fn read(path: Path) -> {fs.read} String
where
    requires non_empty_path {
        path != ""
    }
{
    ...
}
```

In this example, the callable type supplies the row. The `where` section supplies local
facts, proofs, evidence declarations, or constraints.

## 4. Row Tails

Rows are closed unless they explicitly name a tail variable.

Inline:

```ash
fn map<A, B, r: Row>(xs: List<A>, f: A -> {r} B) -> {r} List<B>

fn lift_log<A, r: Row>(x: A) -> {log.write | r} A
```

Expanded:

```ash
fn process<A, r: Row>(input: A) -> Output
where
    row {
        fs.read,
        log.write,
        fail ProcessError,
        evidence process_trace,
        | r,
    }
{
    ...
}
```

The tail is just row polymorphism. It is not a handler stack, not an ordering device, and not
an authority grant.

For complex higher-order callable parameters, prefer inline rows or named callable aliases
over separating the parameter from its row in a distant `where` clause:

```ash
type Step<A, B, r: Row> = A -> {r} B

fn traverse<A, B, r: Row>(xs: List<A>, f: Step<A, B, r>) -> {r} List<B>
```

This keeps the effect of the callable parameter visible at the parameter's type site.

## 5. Named Fact Declarations

Anonymous predicate-like facts make rows and diagnostics hard to track. Target Ash should
name predicate-like artifacts even when the name is local:

```ash
fn transfer(req: TransferRequest) -> Approval
where
    requires positive_amount {
        req.amount > 0
    }

    ensures audit_on_approval {
        result.approved implies audit_recorded(req)
    }
{
    ...
}
```

The name gives the compiler and runtime a stable handle for discharge, evidence, reports,
and diagnostics:

```text
failed to discharge `requires positive_amount`
missing evidence for `ensures audit_on_approval`
```

Rows store resolved fact identities internally, not source strings.

## 6. Common Fact Grammar Shape

`requires`, `ensures`, `invariant`, `law`, and `proof` should use one declaration family
wherever they appear: module scope, callable `where` clauses, future interfaces, or future
theory declarations.

Illustrative shape:

```ash
fact_kind name(params?)
    relation?
{
    body
}
```

where `fact_kind` is one of:

```text
requires | ensures | invariant | law | proof
```

Examples:

```ash
requires non_empty_path(path: Path) {
    path != ""
}

ensures sorted_output(xs: List<Int>, result: List<Int>) {
    sorted(result)
}

law associative<T>(op: Op<T>) {
    op(op(a, b), c) == op(a, op(b, c))
}

proof sorted_output_check(xs: List<Int>)
    proves sorted_output(xs, result)
{
    by runtime_check
}
```

Inside a callable, parameters may be omitted when the fact uses the callable's signature
binders:

```ash
fn sort(xs: List<Int>) -> List<Int>
where
    ensures sorted_output {
        sorted(result)
    }
{
    ...
}
```

This is still a named fact declaration. Its full identity includes its lexical scope and
declared fact kind.

## 7. Facts, Laws, and Evidence

Rows contain evidence requirements, not facts themselves. A fact is a named claim such as a
precondition, postcondition, invariant, or law. Evidence is the named discharge artifact that
denotes both:

1. the fact being discharged;
2. the proof, check, assumption, or record that discharges it.

The resolution chain is:

```text
row item -> evidence id -> fact id -> predicate/law body
```

Typical local shape:

```ash
fn normalize(xs: List<Int>) -> List<Int>
where
    row {
        fail NormalizeError,
        evidence sorted_output_proof,
    }

    ensures sorted_output {
        sorted(result)
    }

    proof sorted_output_proof
        proves sorted_output
    {
        by runtime_check
    }
{
    ...
}
```

Typical module-evidence shape:

```ash
fn stable_sort(xs: List<Int>) -> List<Int>
where
    row {
        evidence crate::proofs::stable_sort_evidence,
    }
{
    ...
}
```

Direct row entries such as `law crate::stable_sort` are not forbidden, but they should be
understood as convenient repetition or a future hook, not the ordinary source form. In the
ordinary case, evidence is the row-level dependency because the evidence object says what
fact it proves and how that fact is discharged.

Future parameterized law evidence fits the same pattern:

```ash
fn combine_all<T>(xs: List<T>, op: Op<T>) -> T
where
    row {
        evidence laws::associative(op),
    }
{
    ...
}
```

If the language later needs to distinguish assuming a law from requiring proof of a law, that
distinction should be explicit in the row syntax. It should not be smuggled into a bare
`law` row item.

If Ash later needs to export an undischarged fact that callers must satisfy, it should use a
distinct spelling such as `obligation sorted_output`. It should not overload `ensures
sorted_output` or `law associative(op)` as evidence.

## 8. Lexical Scoping And Name Resolution

Fact names obey ordinary lexical scoping. Module-qualified names obey the normal module
system. There is no special syntax for escaping to an enclosing function's local facts.

```ash
law sorted_output {
    ...
}

fn normalize(xs: List<Int>) -> List<Int>
where
    row {
        evidence sorted_output_proof,
        evidence crate::proofs::stable_sort_evidence,
    }

    ensures sorted_output {
        sorted(result)
    }

    proof sorted_output_proof
        proves sorted_output
    {
        by runtime_check
    }
{
    ...
}
```

In this example:

- unqualified `sorted_output` inside the local proof resolves to the local `ensures`;
- `crate::proofs::stable_sort_evidence` resolves through the module system;
- the module-level `law sorted_output` is shadowed for unqualified lookup inside the
  callable's `where` scope.

Nested functions follow the same rules:

```ash
fn outer(xs: List<Int>) -> List<Int>
where
    law stable {
        ...
    }
{
    fn inner(ys: List<Int>) -> List<Int>
    where
        row {
            evidence inner_stability,
        }

        law stable {
            ...
        }

        proof inner_stability
            proves stable
        {
            ...
        }
    {
        ...
    }

    inner(xs)
}
```

Inside `inner`, unqualified `stable` resolves to `inner`'s local law. There is no special
source spelling for "outer function's local `stable`." If the outer fact must be referenced
from inner scopes, use normal remedies: rename it, move it to module scope, import a
module-level declaration with an alias, or restructure the declarations.

Recommended collision rule:

```text
Within one lexical scope, predicate-like fact names share one namespace.
```

So a scope should reject or require renaming for:

```ash
where
    requires sorted {
        sorted(xs)
    }

    law sorted {
        ...
    }
```

This avoids confusing rows and evidence references such as `evidence sorted_proof` or
`proves sorted`.

## 9. Lowering Intuition

The source:

```ash
fn process(req: Request) -> Response
where
    row {
        http.get,
        fail ProcessError,
        evidence response_contract_check,
    }

    requires valid_request {
        valid(req)
    }

    ensures valid_response {
        valid(result)
    }

    proof response_contract_check
        proves valid_response
    {
        by runtime_check
    }
{
    ...
}
```

has an internal summary along these lines:

```text
callable row:
  http.get
  fail ProcessError
  evidence response_contract_check

evidence:
  response_contract_check proves process::where::valid_response

facts:
  process::where::valid_request = requires valid(req)
  process::where::valid_response = ensures valid(result)
```

The source row remains readable because predicate bodies live in named declarations. The
compiler summary can still carry every row, fact, discharge, and evidence record needed by
Core, CPS, diagnostics, and runtime reporting.

## 10. Open Questions

1. What is the exact grammar for proof bodies (`by smt`, `by runtime_check`, tactic blocks,
   evidence constructors, or ordinary Ash expressions)?
2. Are direct row entries such as `requires valid_input` and `law associative(op)` retained
   as public convenience syntax, explicit obligation syntax, or only emitted in compiler
   summaries?
3. How should facts whose predicates mention captured binders appear in exported summaries
   for nested functions or escaping closures?
4. What lifecycle metadata must evidence carry for static proof, runtime check, trusted
   assumption, recorded audit evidence, invalidation, and replay?

## 11. Working Principle

The design rule for future specs:

```text
Rows contain evidence requirements.
Evidence denotes a fact plus its proof, check, assumption, or record.
Where clauses define the local facts and evidence those rows may reference.
Lexical scope resolves names; module paths qualify public declarations.
```
