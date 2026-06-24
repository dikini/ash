# NOTE-015: Current-to-Target Language Forms — Primitive Budget and Migration Map

**Date:** 2026-06-24
**Status:** Living document — exploration in progress
**Purpose:** Summarize how current Ash language forms should map into the target Ash
language model: a small core language with effect rows, handlers/providers, contracts,
evidence, and library-level profiles replacing most privileged tower-specific syntax.
Companion to NOTE-013 (ambient monad and handler composition) and NOTE-014 (contract
systems unification).

## 0. Motivation

Ash currently has many language constructs that grew as separate answers to related
questions:

1. **How is computation sequenced?** `act`, `do:Act`, `do:Proc`, `do:Workflow`,
   comprehensions, workflow statements.
2. **What authority is needed?** capabilities, roles, resources, workflow headers.
3. **What must be checked or proven?** `requires`, `ensures`, obligations, laws,
   properties, policy decisions.
4. **What runtime service is being requested?** capability calls, process operations,
   channels, workflow admission, reports, provenance.
5. **How does failure move?** `fail`, `panic`, `with_error`, process failure, workflow
   failure, dynamic contract violation.

The target direction is not to delete the concepts. It is to stop making each concept a
separate privileged language island.

The core target model is:

```text
fn + ordinary data/types + effect rows + contracts/evidence + raise/handle/provider frames
```

`Act`, `Proc`, and `Workflow` may remain as public library names, carriers, profiles, or
modules. From the language point of view, however, they should be expressible over the
shared substrate:

```text
Comp<ρ, A>
```

where `ρ` is an effect row describing requirements, not authority grants.

## 1. The Primitive Budget

The target language should keep the primitive set small. A form earns primitive status only
when the type checker or operational semantics cannot be stated cleanly without it.

### 1.1 Core primitives

These remain language/core constructs:

| Form family | Why primitive |
|---|---|
| `fn`, lambda, call | Function abstraction and application are the core computation surface. |
| `let`, scoped bindings | Name binding and lexical scope are core. |
| literals, variables | Atomic expressions. |
| records, tuples, variants | Data construction and elimination need type-level structure. |
| `match`, `if`, `if let` | Typed elimination, exhaustiveness, and branch environments are core. |
| patterns | Binding and elimination correctness require type-aware checking. |
| type declarations, aliases, ADTs | Nominal and structural type identity. |
| modules, imports, visibility | Compilation and public summary boundaries. |
| interfaces/impls/associated types | Type-level evidence and library algebra, especially `Monad<K>`. |
| row-bearing function types | Required to state computation requirements. |
| Core/CPS `Raise`, `Handle`, continuations | Required to execute operation-like effects and resumptive control. |
| contract discharge metadata | Required to preserve proof/runtime/evidence boundaries. |

### 1.2 Substrate primitives, not user-facing islands

Some constructs are primitive in Core/CPS but should not force large surface syntax:

| Substrate | Surface posture |
|---|---|
| effect rows | Surface annotations and inferred summaries. |
| effect item identity | Canonical compiler identity; surface uses readable paths. |
| handler/provider frames | Surface `handle`/library providers; runtime owns dispatch. |
| continuation multiplicity | Core/CPS typing first; surface declarations later. |
| contract discharge records | Compiler/evidence metadata, not ordinary user data. |
| trusted extern boundary | Effect-owned implementation hook, not unrestricted callable escape. |

### 1.3 Library forms and corpus migration targets

Most current tower/governance forms should move here. They remain useful, but their
semantics should be explainable as library declarations, effect operations, providers,
contracts, or row profiles. Current surface forms do not need a language-level
compatibility layer: the only legacy users are the standard library, documentation examples,
and tests, so migration can be handled as project corpus work.

## 2. The Central Resolved Direction

### 2.1 `Act`, `Proc`, and `Workflow` are library-level profiles/carriers

The old tower remains an explanation of increasing operational power:

```text
Pure < Act < Proc < Workflow
```

The target meaning is row-profile based:

```text
Pure      = Comp<{}, A>
Act       = Comp<{cap/resource/fail/evidence...}, A>
Proc      = Comp<{Act effects + proc/channel...}, A>
Workflow  = Comp<{Proc effects + role/policy/contract/obligation/report...}, A>
```

This resolves the conceptual split:

- the ambient computation monad sequences Ash computations;
- `bind` and `return` are row-polymorphic structural operations of that ambient monad;
- The row says what operations or obligations may be needed.
- The profile says which row shapes are admissible in a context.
- A carrier such as `Act<A>` or `Workflow<A>` may remain as library/API surface, but it is
  not a distinct semantic foundation.

### 2.2 Other monads are interpreted by effects and handlers

Target Ash should not model `Act`, `Proc`, `Workflow`, `State`, `Either`, `Future`, or
similar computation stories as separate nested monad transformers in the language core.

The language has one ambient monad. Other "monads" are represented by:

```text
effect operations + rows + handlers/providers + contracts/evidence
```

Composition of these interpretations is handler composition:

```ash
handle
  handle
    computation
  with H_inner
with H_outer
```

The effect row is intentionally unordered. It records **what** operations or requirements
may appear. It does not record **which handler runs first**. Handler nesting records order.

For some handlers, order does not matter: disjoint operations with compatible deep handlers
may commute. That is a special law that can be proved under conditions, not a general rule.
In general, handler composition is order-sensitive. State outside failure is not equivalent
to failure outside state; a shallow non-resuming handler changes what later handlers can see;
multi-shot resumption is legal only when continuation multiplicity and rows allow it.

This is the replacement for monad-transformer composition:

```text
old transformer story:  StateT s (EitherT e IO) encodes order in nested types
target Ash story:       rows list requirements, nested handlers encode interpretation order
```

### 2.3 Example: exception and nondeterminism

Consider two effect operations:

```ash
effect Exception<E> {
    throw(e: E) -> Never;
}

effect Choice {
    choose<A>(xs: List<A>) -> A;
}
```

A computation that uses both has one ambient sequencing form. The binds do not select an
`ExceptionT` or `ListT` transformer. They accumulate requirements in the row:

```ash
fn pick_inverse(xs: List<Int>) -> {choice.choose, fail DivideByZero} Int {
    do {
        x <- choose(xs);          -- row: {choice.choose}
        if x == 0 {
            throw(DivideByZero);  -- row: {fail DivideByZero}
        };
        return 100 / x            -- row: {}
    }
}
```

Schematic row progression:

```text
return v
  : Comp<{}, Int>

choose(xs)
  : Comp<{choice.choose}, Int>

throw(DivideByZero)
  : Comp<{fail DivideByZero}, Never>

bind(choose(xs), x -> if x == 0 then throw(...) else return (100 / x))
  : Comp<{choice.choose, fail DivideByZero}, Int>
```

Now install a handler for `choice.choose`. At the row level, the handler does not reorder
the row. It matches and peels one operation from the row, binding the rest as the residual
tail:

```text
body row:
  {choice.choose, fail DivideByZero | a}

handler pattern:
  {choice.choose | r}

match:
  r := {fail DivideByZero | a}

after handling choice.choose:
  residual body row = handler.row union r
                    = handler.row union {fail DivideByZero | a}
```

If `AllChoices` is pure apart from interpreting `choice.choose`, its local handler row is
empty:

```text
handler.row = {}

handle choice.choose with AllChoices
  body:     {choice.choose, fail DivideByZero | a}
  residual: {fail DivideByZero | a}
```

The failure effect remains because the choice handler did not interpret it. A later
exception handler can peel that residual row:

```text
body row:
  {fail DivideByZero | a}

handler pattern:
  {fail DivideByZero | r}

match:
  r := {a}

after handling fail DivideByZero:
  residual body row = exception_handler.row union {a}
```

This is the row-level intuition for handler composition. Each handler removes the operation
it interprets and propagates the unmatched tail. The row tail is not an ordering device; it
is the still-unhandled requirement set.

The row says only that the computation may choose and may fail. It does not say whether
failure aborts the whole search or only the current branch. That is decided by handler
nesting.

Exception outside nondeterminism:

```ash
handle fail DivideByZero with ExceptionAsResult {
    handle choice.choose with AllChoices {
        pick_inverse([10, 0, 5])
    }
}
```

Interpretation: the inner `AllChoices` handler explores branches. If a branch throws, the
throw escapes to the outer exception handler. Depending on the precise exception handler,
one thrown branch may abort the whole search and produce `Err(DivideByZero)`.

Nondeterminism outside exception:

```ash
handle choice.choose with AllChoices {
    handle fail DivideByZero with DropFailedBranch {
        pick_inverse([10, 0, 5])
    }
}
```

Interpretation: the inner exception handler catches failure per branch and can discard only
that branch. The outer choice handler can still collect successful branch results, for
example `[10, 20]`.

Both executions start from the same row:

```text
{choice.choose, fail DivideByZero}
```

The difference is not in the row. The difference is in the handler stack:

```text
ExceptionAsResult . AllChoices      -- exception interprets failures outside the search
AllChoices . DropFailedBranch       -- exception is interpreted inside each branch
```

This is the target Ash replacement for choosing between transformer stacks such as:

```text
ExceptT E List A     -- branch-local failures, collect successes
ListT (Either E) A   -- failure can abort the whole list computation
```

The ambient `bind` is the same in both cases. It threads the current continuation and
accumulates the row. The nested handlers decide how each operation is interpreted and
whether composition order matters for that pair of effects.

### 2.4 Rows are requirements, not grants

This rule must remain the central diagnostic and design principle.

```ash
fn read_config(path: String) -> {cap fs.read} String
```

The row above does not grant file authority. It states that running the computation requires
`fs.read` to be discharged by admitted capability, role entailment, provider frame, workflow
admission, or another explicit boundary.

Consequences:

- effect aliases are abbreviations, not authority bundles;
- role entailment is discharge, not row normalization;
- policy effects require named policy evaluators, not arbitrary boolean expressions;
- contracts require discharge records, not silent erasure;
- provider/handler installation is operational authority, not syntax decoration.

## 3. Capabilities as the Example Pattern

Capabilities show the intended move for many current constructs.

Current Ash has `capability` declarations, capability references in workflow headers,
operation modes, provider implementations, and capability calls. Target Ash should factor
that into:

```text
capability declaration
  -> effect operation signature
  -> row item required by callers
  -> contracts on arguments/results
  -> provider/handler implementation
  -> optional trusted extern boundary
```

Illustrative target shape:

```ash
effect Fs {
    read(path: String) -> String
        requires { path != "" }
        extern "host.fs.read";
}
```

The core does not need a privileged "capability subsystem" separate from effects. It needs:

1. canonical operation identity: `cap Fs.read`;
2. row contribution: `{cap Fs.read}`;
3. contract contribution: e.g. `requires {path != ""}`;
4. provider or handler discharge;
5. optional trusted host implementation.

The surface can still expose capability-like authoring forms if they are useful. Their
lowering should be honest: they define typed effect operations and provider/admission
paths.

## 4. Current-to-Target Form Matrix

The following matrix is intentionally descriptive, not yet normative. It records the
intended direction, current implementation posture, resolved decisions, and open work.

### 4.1 Core expression and data forms

| Current form | Implemented/current posture | Intended target | Resolved | To resolve |
|---|---|---|---|---|
| literals, variables | Implemented ordinary expressions | Keep core | Yes | No major design issue. |
| function definitions `fn` | Implemented, currently no full surface effect rows | Keep as primary computation declaration | Yes | Surface row annotation shape and inference/reporting policy. |
| function expressions/closures | Implemented with current limitations | Keep core with row-bearing closure types | Mostly | Effect-safe capture and row summaries must be fully aligned. |
| function call | Implemented | Keep core; call row = callee body row plus continuation row in CPS | Yes in Core/CPS direction | Surface-to-Core lowering remains. |
| `let` | Implemented | Keep core | Yes | Destructuring irrefutability and diagnostics continue to mature. |
| records/tuples/lists | Implemented to varying depth | Keep data forms; list remains library-backed collection | Mostly | List builtin-to-stdlib migration and collection algebra details. |
| ADTs/variants | Implemented | Keep core type/data feature | Yes | Exhaustiveness/canonicalization completeness. |
| field/index access | Implemented | Keep expression sugar over projection | Yes | Index syntax cleanup if tuple/list semantics diverge. |
| `if`, ternary | Implemented | Keep conditionals; ternary can remain sugar | Yes | None central to effect redesign. |
| `match` | Implemented/currently hardening | Keep core eliminator | Yes | Complete refutable/exhaustiveness/canonicalization story. |
| `if let` | Implemented/current grammar records it | Keep explicit refutable eliminator | Yes | Exact surface spelling and branch diagnostics. |
| pipe `|>` | Token/reserved/currently limited | Library/syntax sugar | Not central | Decide whether to keep active surface. |

### 4.2 Type and module forms

| Current form | Implemented/current posture | Intended target | Resolved | To resolve |
|---|---|---|---|---|
| `crate`, `dependency` | Implemented crate root metadata | Keep tooling/module surface | Yes | Dependency semantics outside language core. |
| `mod`, `use`, visibility | Implemented | Keep language/module surface | Yes | Public row summary export/import details. |
| `type` aliases/records/enums | Implemented | Keep core type surface | Yes | No effect-specific issue. |
| generics and kinds | Implemented and expanded over time | Keep; add row kind as first-class | Mostly | Surface syntax for row variables and constraints. |
| type holes `_` | Implemented | Keep inference aid | Yes | Interaction with partial type constructors remains type-system work. |
| associated types/families | Implemented/advanced | Keep type-level feature | Yes | Row-bearing callable types in associated positions. |
| sealed domains, data kinds, type functions | Implemented in later phases | Keep as type-level machinery | Mostly | Which pieces are user-facing vs compiler/library discipline. |
| callable arrows / tower arrows | Reserved or partially designed | Prefer row-bearing function types | Direction yes | Migrate stdlib/docs/tests to the target spelling rather than preserving old arrows as language compatibility. |

### 4.3 Algebra and sequencing forms

| Current form | Implemented/current posture | Intended target | Resolved | To resolve |
|---|---|---|---|---|
| `act { ... }` | Implemented as current Act helper syntax | Remove as language syntax; migrate corpus to target `do`/library surface | Direction yes | Update stdlib/docs/tests and add target-form coverage. |
| `do:Act`, `do:Proc`, `do:Workflow` | Implemented typed-do targets | Remove as language syntax unless profile annotations earn target status | Direction yes | Decide if explicit profile annotations remain useful; otherwise migrate corpus. |
| `do:K` | Implemented for selected targets/evidence | Keep as general sequencing sugar over algebra/evidence | Yes | Arbitrary user monads and row-polymorphic `Comp` integration. |
| comprehensions | Implemented/partially target-aware | Keep as sugar over `map`/`bind`/guards via evidence | Mostly | Guard/filter semantics and target inference boundaries. |
| `return` in do | Implemented | Keep as target/unit in do, not function-level control | Yes | None central. |
| legacy `ret` | Implemented compatibility | Remove from target syntax; migrate corpus to `return` | Yes | Corpus migration only. |
| `Monad<K>` evidence | Implemented for selected targets | Primary sequencing abstraction | Yes | Law evidence and arbitrary user evidence execution. |

### 4.4 Effect, capability, resource, and host boundary forms

| Current form | Implemented/current posture | Intended target | Resolved | To resolve |
|---|---|---|---|---|
| `capability` declaration | Implemented current subsystem | Prefer target `effect` operation declaration plus provider/admission metadata | Direction yes | Decide whether `capability` remains a domain library authoring form; no language compatibility layer required. |
| capability operation modes `read/write/...` | Implemented vocabulary | Operation metadata or contract/resource tags | Partial | Whether modes are semantic row items or diagnostics/docs only. |
| capability impls/providers | Implemented around current model | Handler/provider implementation for effect operations | Direction yes | Provider frame API, authority provenance, extern placement. |
| `builtin fn` | Implemented | Keep narrow compiler-known escape hatch | Partial | Prefer stdlib/effect-owned externs where possible. |
| `external` / `extern fn` | Reserved/not fully active | Trusted effect-owned host hook | Direction yes | Surface syntax and safety boundary. |
| resources / `owns` | Implemented as workflow/header concepts | Resource row items plus ownership/borrow/provenance discharge | Direction yes | Split/join/borrow algebra and diagnostics. |
| effect aliases/groups | Target spec only | Keep as row abbreviation/diagnostic grouping | Yes | Parser/typechecker implementation and private alias export rules. |
| arbitrary user-defined effects | Out of current target scope | Deferred | Yes deferred | Minimal declaration surface if/when admitted. |

### 4.5 Governance forms: roles, policies, obligations, contracts

| Current form | Implemented/current posture | Intended target | Resolved | To resolve |
|---|---|---|---|---|
| `role` declarations | Implemented current governance | Library/governance declarations that entail row items | Direction yes | Role admission, multi-role model, invalidation of entailment evidence. |
| `plays role` | Workflow header syntax | `role` row item/admission requirement | Yes direction | Migrate corpus to target admission/profile spelling. |
| policy expressions | Implemented separate expression DSL | Named policy programs/evaluators discharging policy row items | Direction yes | First-class vs named-only policy boundary. |
| `decide under policy` | Workflow statement | Policy effect/evaluator boundary plus branch/library combinator | Direction yes | Decision-domain typing and evidence reporting. |
| `requires`, `ensures` on fn/workflow | Implemented current contracts | Contract row items/refinements/discharge records | Yes direction | Surface-to-Core predicate structuralization. |
| `invariant` / guards | Partially target/current via related forms | Contract row items attached to loops/data/channel boundaries | Direction yes | Exact attachment sites and temporal behavior. |
| obligations / `oblige` / `check` | Implemented workflow concepts | Contract/evidence/liveness row items | Partial | Distinguish safety obligations from temporal/liveness obligations. |
| `law` | Implemented declaration form | Evidence-producing universal contract, discharged once per impl | Yes direction | Proof modes and interface law integration. |
| `proof` | Implemented declaration form | Evidence declaration/discharge mechanism | Direction yes | Trusted proof artifact model. |
| `property` / `quickcheck` / `small_world` | Implemented/testing-related | Test/falsification metadata, not runtime row item | Yes direction | Evidence reporting and law-test distinction. |
| `prop` | Implemented proposition declaration | Type/evidence proposition layer | Partial | Relationship to row contracts and proof obligations. |

### 4.6 Process, channel, workflow, and interaction forms

| Current form | Implemented/current posture | Intended target | Resolved | To resolve |
|---|---|---|---|---|
| `Proc<A>`, `P<A>` | Implemented public carriers/handles | Library/API over process/channel effects | Direction yes | Public carrier persistence vs type alias/profile story. |
| `proc::par`, `await`, `join`, `yield` | Implemented library/builtin surface | Library operations requiring `proc` row items | Direction yes | Handler/provider representation and failure aggregation. |
| `send`, `receive` workflow statements | Implemented workflow syntax | Channel effect operations with guard contracts | Direction yes | Queue semantics, guard failure behavior, session/protocol typing. |
| `receive wait` | Implemented current syntax | Channel receive plus timeout/failure/process effects | Partial | Timeout as failure effect, process effect, or library combinator. |
| `Workflow<A>` | Implemented first-class carrier | Library/governance carrier/profile over process plus contract plan | Direction yes | How much public carrier remains after row profiles mature. |
| `workflow` keyword | Implemented legacy/core syntax | Remove as primitive language syntax; model as governed `fn`, app entry, or library carrier | Yes direction | Migrate stdlib/docs/tests to target forms. |
| workflow headers | Implemented | Row/admission/contract summaries | Yes direction | Map concepts into target app/function/admission declarations for corpus migration. |
| `observe`, `orient`, `propose` | Implemented workflow statements | Likely library/protocol/evidence operations | Partial | Whether they are core workflow vocabulary or domain libraries. |
| `maybe`, `must` | Implemented workflow statements | Failure/contract/library combinators | Partial | Interaction with `fail`, `with_error`, and row accounting. |
| `done` | Implemented workflow terminator | Remove as target syntax unless it earns library/domain status | Direction likely | Migrate corpus to `return`/unit/runner-specific completion. |
| `with expr do` | Implemented workflow statement | Scoped provider/admission/handler installation | Direction yes | Concrete handler/provider lowering. |
| `yield` declaration/statement | Implemented current workflow/proxy area | Protocol/library operation over continuations or process effects | Partial | Relationship to CPS continuations and interaction protocols. |
| `proxy`, `resume` | Implemented/reserved in workflow interaction area | Protocol library over continuations, channels, or workflow admission | Open | Needs dedicated interaction-protocol review. |

### 4.7 Failure and bottom forms

This is the least resolved cluster and deserves its own follow-up note.

| Current form | Implemented/current posture | Intended target | Resolved | To resolve |
|---|---|---|---|---|
| `fail` | Implemented operational bottom/failure form | Explicit failure effect when recoverable; trap/bottom when unrecoverable | Partial | Concrete failure row taxonomy. |
| `panic` | Implemented in function statements | Debug/host trap, not ordinary domain failure | Direction likely | Whether user-facing `panic` remains. |
| `with_error` | Implemented scoped handling form | Handler/library surface over failure effects or traps where permitted | Partial | Which failures are resumable/recoverable. |
| dynamic contract failure | Implemented in Core direction as trap or explicit `fail` | Structured bottom or failure effect with blame | Direction yes | Blame, observability, handler behavior. |
| authority denial | Current capability/runtime error paths | Admission/discharge failure, not policy denial | Direction yes | Diagnostic taxonomy and runtime evidence. |
| policy denial | Current policy failure path | Named policy decision failure with evidence | Direction yes | Failure row spelling vs boundary rejection. |
| process failure | Implemented process/runtime specs | Process terminal state observed by handles | Mostly | Aggregation and supervision surface. |
| workflow failure | Implemented workflow boundary model | Boundary reinterpretation/reporting of lower failures | Mostly | Full failure cause taxonomy and reports. |

## 5. Implemented Baseline vs Target State

### 5.1 Implemented current language facts

The current parser and specs already implement or recognize a broad surface:

- ordinary functions, closures, calls, records, tuples, lists, variants, `match`, `if let`;
- module declarations, imports, visibility, crate metadata;
- type declarations, generics, kind annotations, associated types/families, type holes;
- `act` blocks, typed `do:K`, selected `Monad<K>` evidence, and comprehensions;
- public `Act`, `Proc`, `Workflow`, and process-handle concepts;
- workflow declarations, workflow headers, and many workflow-specific statements;
- capability, role, policy, law, proof, property, proposition, resource-related surfaces;
- operational failure forms such as `fail`, `panic`, and `with_error`;
- Core/CPS infrastructure for rows, typed Core, CPS lowering, handlers, continuations,
  lazy/memo modes, and continuation multiplicity.

This means the target direction is not greenfield. It is a convergence project.

### 5.2 Target language facts

The target language should make the following true:

1. There is one ordinary computation surface: `fn` with row-bearing types.
2. `Act`, `Proc`, and `Workflow` are library/profile/carrier names over the same substrate.
3. Capability, process, channel, and failure operations lower to operation-like effects.
4. Roles, policies, contracts, resources, and evidence are ambient/boundary discharge items,
   not ordinary raised operations unless a specific recoverable form says otherwise.
5. Contracts have explicit discharge modes: static, evidence, dynamic.
6. Laws can produce evidence; properties cannot discharge hard type obligations.
7. External host calls are effect-owned implementation hooks.
8. Legacy workflow syntax lowers away before semantic analysis.
9. Core Ash is the canonical direct-style checked language.
10. CPS IR is the executable control/effect representation.

## 6. Resolved Design Decisions

### 6.1 Rows are unordered requirement sets

Rows should remain unordered sets of requirements. Handler/provider nesting order determines
which handler sees an operation first. The row answers **what** may be required; the handler
stack answers **how** and **when** requirements are discharged.

This means the row is not a monad-transformer stack. It does not specify composition order.
Composition order is the operational handler stack, and handler composition is not generally
commutative.

### 6.2 Profiles are constraints, not privileges

`Act`, `Proc`, and `Workflow` profiles constrain admissible rows. They do not grant authority.

```text
Act profile accepts capability/resource/failure/evidence rows.
Proc profile accepts Act rows plus process/channel rows.
Workflow profile accepts Proc rows plus governance/report rows.
```

### 6.3 Effect aliases do not grant authority

An alias or group expands to row items or improves diagnostics. It must never behave like an
admission package.

### 6.4 Capabilities are effect operations plus discharge

The current capability concept should be explained in terms of:

```text
operation identity + row item + contracts + provider/handler + optional extern
```

### 6.5 Contracts are layered, not schizophrenic

Hoare contracts, laws, and properties stay distinct at the surface. They unify through
contract/evidence machinery, not by pretending they have the same lifecycle.

### 6.6 Properties do not discharge rows

Properties are falsification/test instruments. They can produce reports and confidence
signals, but they are not proof.

### 6.7 Core/CPS owns the hard semantics

Surface syntax should elaborate into Core. Core type checking and CPS lowering should own
row facts, discharge facts, continuation rows, and handler/provider behavior.

### 6.8 Monad composition is handler composition

The target language has one ambient monad for sequencing. Domain-specific monadic behavior
is supplied by handlers interpreting effect operations. Therefore:

- row extension adds possible operations;
- handler nesting composes interpretations;
- handler order is semantically visible unless proven irrelevant;
- algebraic laws for handler composition are evidence/proof obligations, not assumptions.

## 7. To Be Resolved

### 7.1 Surface spelling for effect declarations

Open choice:

1. keep `capability` as domain-friendly sugar;
2. introduce `effect` declarations as the canonical operation declaration form;
3. allow both, with `capability` lowering to a restricted `effect` declaration.

Recommendation: make `effect` the canonical target vocabulary and retain `capability` only
as a domain-friendly spelling if it earns its keep. Otherwise, migrate the project-owned
corpus to `effect` declarations and remove the language form.

### 7.2 External function boundary

Externs should not be ordinary pure functions. The unresolved details:

- Can externs appear only inside effect/provider declarations?
- Can trusted handlers own externs instead?
- What syntax distinguishes safe Ash calls from raw host ABI hooks?
- How are extern contracts and ABI failures represented?

The semantic requirement is already clear: ordinary Ash code calls typed effect operations,
not raw externs.

### 7.3 Failure taxonomy

Ash needs concrete row/IR spelling for at least:

- recoverable domain-like failure;
- unrecoverable trap/bottom;
- contract violation;
- authority/admission denial;
- policy denial;
- host ABI failure;
- process cancellation/failure;
- workflow boundary failure.

The current `fail`/`panic`/`with_error`/workflow-failure surfaces should be reclassified
against that taxonomy.

### 7.4 Contract blame and subsumption

NOTE-014 identifies the blockers:

- caller vs callee blame for dynamic contracts;
- interface-to-impl precondition/postcondition variance;
- monadic Hoare logic through `bind`;
- temporal contracts for process/workflow behavior;
- interaction with lazy/memo evaluation timing.

### 7.5 Workflow interaction vocabulary

Forms such as `observe`, `orient`, `propose`, `yield`, `proxy`, and `resume` need a separate
review. They may be:

1. domain library constructs;
2. protocol effects over channels/continuations;
3. workflow-governance helpers;
4. removal candidates to retire.

They should not remain primitive merely because they are old workflow statements. Since the
legacy users are project-owned corpus files, the target can choose the cleaner model and
migrate examples/tests instead of accepting old syntax indefinitely.

### 7.6 Builtins vs stdlib

The target language should reduce compiler-known builtins where a library/evidence form is
sufficient. But some primitives remain necessary for Core/CPS, host effects, and bootstrapping.

The open work is an honesty boundary:

```text
compiler primitive vs trusted builtin vs stdlib function vs effect-owned extern
```

## 8. Migration Strategy

### 8.1 Stage A: Inventory and classification

Every current grammar form should receive one classification:

```text
core primitive
substrate primitive
library surface
effect operation declaration
contract/evidence declaration
corpus migration target
removal candidate
```

This note is a first pass at that classification.

### 8.2 Stage B: Target replacements before corpus migration

For each current form used by the standard library, documentation examples, or tests, define
the target replacement and add target-form coverage before migrating that corpus:

- functions and closures;
- `act`/`do` blocks;
- workflow headers and statements;
- capability calls;
- process/channel operations;
- contracts and obligations.

### 8.3 Stage C: Corpus migration

Project-owned legacy uses should be rewritten to target constructs:

```text
workflow header      -> function row + admission/contract metadata
capability call      -> effect operation raise/call with provider discharge
receive guard        -> channel effect + guard contract
oblige/check         -> obligation/contract row item + evidence state
do:Act/do:Proc/...   -> do block checked against a row profile, if profiles remain
```

This is not a promise that the language accepts both forms indefinitely. It is a scheduling
plan for stdlib/docs/tests migration.

### 8.4 Stage D: Documentation and diagnostics

Diagnostics should teach the target model:

- "missing authority for `cap fs.read`" rather than generic unhandled effect;
- "policy `P` denied action" distinct from missing authority;
- "contract `requires P` not discharged" with static/evidence/dynamic mode;
- "process effect used outside Proc-capable profile";
- "effect alias `IO` expands to missing concrete item `cap log.write`."

### 8.5 Stage E: Remove old surface once corpus is migrated

Old syntax can be removed once:

1. the target form exists;
2. stdlib sources are migrated;
3. documentation examples are migrated;
4. tests cover the target form;
5. any required historical examples are either rewritten or marked historical/non-target.

## 9. Working Principle

The design rule for future cleanup:

```text
If a construct names authority, governance, orchestration, runtime service, or evidence,
it should be represented as an effect row item, handler/provider/admission rule, contract
discharge, or library declaration unless the core type system or operational semantics
requires it as a primitive.
```

This preserves Ash's concepts while simplifying the language:

- fewer privileged syntactic islands;
- more reusable library abstractions;
- clearer authority boundaries;
- better diagnostics;
- one Core/CPS semantic path.

## 10. References

Internal references:

- [SPEC-095a: Current Grammar](../spec/SPEC-095a-CURRENT-GRAMMAR.md)
- [SPEC-095b: Target Grammar](../spec/SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-099: Core Ash](../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
- [NOTE-013: Ambient Monad and Handler Composition Algebra](NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md)
- [NOTE-014: Contract Systems Unification](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md)

## 11. Changelog

- 2026-06-24: Initial synthesis note. Captures current-to-target language-form taxonomy,
  resolved direction, implemented baseline, and unresolved cleanup areas for the surface
  language convergence effort.
