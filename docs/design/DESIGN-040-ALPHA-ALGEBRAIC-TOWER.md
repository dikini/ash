# DESIGN-040: Alpha Algebraic Tower and Library-Visible Computation APIs

**Status:** Draft design note — not yet normative spec or implementation plan
**Date:** 2026-05-19
**Related:** [DESIGN-020](DESIGN-020-PURE-FUNCTIONS-THREE-VERTEX-MODEL.md), [DESIGN-030](DESIGN-030-PROC-LIBRARY-AND-MINIMAL-RUNTIME-SUBSTRATE.md), [DESIGN-031](DESIGN-031-GENERALIZED-DO-NOTATION.md), [SPEC-001](../spec/SPEC-001-IR.md), [SPEC-003](../spec/SPEC-003-TYPE-SYSTEM.md), [SPEC-047](../spec/SPEC-047-ACT-MONAD.md), [SPEC-048](../spec/SPEC-048-PROC-LIBRARY.md), [SPEC-049](../spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md), [SPEC-050](../spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md), [SPEC-051](../spec/SPEC-051-WORKFLOW-SEMANTICS.md), [SPEC-054](../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md), [SPEC-066](../spec/SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md), [SPEC-067](../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)

## 1. Summary

Alpha should demonstrate a bounded but operationally complete Ash system: the computation tower is algebraic and programmable, generalized monadic code lowers through visible evidence, and accepted programs have a coherent path from source semantics through IR to execution.

The alpha target is:

- `Act<A>`, `Proc<A>`, and `Workflow<A>` have Ash-visible public algebra, type evidence, and construction APIs.
- Their operational representation remains opaque and runtime/compiler-owned.
- Runtime special treatment is allowed only as the implementation of visible algebraic operations and opaque carriers.
- Surface syntax such as `act { ... }`, `do:Act { ... }`, `do:Proc { ... }`, `do:Workflow { ... }`, `do:Result<_, E> { ... }`, and workflow construction should lower through the same typed algebraic construction path.
- Generalized monadic `<-` lowering for user/library `Monad<K>` evidence is an alpha requirement, not a beta-only stretch.
- OODA-specific forms should become libraries/templates over Ash language features, not primitive IR categories; by default, no OODA node remains core alpha semantics.
- Generic functions over monadic computations should be statically resolved and monomorphized during compilation/lowering; runtime elaboration should be rare, explicit, and preferably absent.
- Alpha should include a mature minimal TCIR/AMIR/bytecode spine for both pure and effectful subsets, drawing on [FUTURE-005](../ideas/future/COMPILED-EXECUTION-SUBSTRATE.md). JIT compatibility should shape bytecode/IR design, but JIT implementation is not an alpha target.
- Alpha should update/create big-step and small-step semantics and state how those semantics correspond to TCIR, AMIR, and bytecode.

This note records direction and rationale for future spec updates. It is not itself a spec or plan.

## 2. Alpha principle: visible algebra, opaque mechanics

The alpha release should not require self-hosting the runtime representation of `Act`, `Proc`, or `Workflow`. Full self-hosting remains beyond v1.

It should require that their user-visible construction model is written as Ash-visible library/prelude surface.

Illustrative target shape:

```ash
opaque type Act<A>;
opaque type Proc<A>;
opaque type Workflow<A>;
opaque type P<A>;

interface Monad<M : * -> *> {
    return<A>(a: A) -> M<A>;
    bind<A, B>(ma: M<A>, f: Fn(A) -> M<B>) -> M<B>;
}

impl Monad<Act> { ... }       -- may be intrinsic-backed
impl Monad<Proc> { ... }      -- may be intrinsic-backed
impl Monad<Workflow> { ... }  -- may be intrinsic-backed
```

The `...` may be compiler/runtime intrinsic in alpha. The important point is that the intrinsic is attached to a visible API/evidence item, not to unadvertised parser/runtime behavior.

### 2.1 Positive example

Acceptable alpha architecture:

```ash
module proc {
    opaque type Proc<A>;
    opaque type P<A>;

    builtin fn unit<A>(a: A) -> Proc<A>;
    builtin fn bind<A, B>(ma: Proc<A>, f: Fn(A) -> Proc<B>) -> Proc<B>;
    builtin fn from_act<A>(a: Act<A>) -> Proc<A>;
    builtin fn par<A, B>(a: Proc<A>, b: Proc<B>) -> Proc<(P<A>, P<B>)>;
    builtin fn await<A>(p: P<A>) -> Proc<A>;

    impl Monad<Proc> {
        return = unit;
        bind = bind;
    }
}
```

Runtime may treat `proc::par` specially because it creates child `ProcessId`s and scheduler work. That is acceptable because `par` is an explicit algebraic operation with a public type.

### 2.2 Negative example

Unacceptable alpha architecture:

```text
Parser sees a special Proc syntax node.
Lowering emits an internal runtime-only Proc instruction.
The behavior cannot be named, imported, typed, or abstracted over in Ash code.
No Ash-visible operation corresponds to the construction.
```

This creates non-obvious magic and prevents users or third-party libraries from building derived constructions.

### 2.3 Boundary: runtime can specialize, not exceed

Runtime behavior must not break or exceed what the visible algebra expresses.

Acceptable:

```text
proc::par has a public type and documented semantics.
Runtime implements it by allocating child process identities and scheduler entries.
```

Not acceptable:

```text
A `do:Proc` block silently admits workflow roles, opens capabilities, or installs handlers that no visible Proc/Workflow operation requested.
```

If a capability, contract, role, scheduler, or reporting effect occurs, the corresponding construction must be visible in the typed algebra or in an explicit tower-specific operation.

## 3. Tower interpretation

The working semantic tower remains:

```text
Pure < Act < Proc < Workflow
```

Each layer adds a richer environment and more admissible operations:

| Layer | Public role | Runtime-owned opaque details | Examples of visible operations |
| --- | --- | --- | --- |
| `Pure` | deterministic value computation | lexical environment | `fn`, `let`, `match`, constructors |
| `Act<A>` | sequential effectful computation | `ActEnv`, capability/provider dispatch, effect trace | `act::unit`, `act::bind`, `act::invoke`, `act::guard` |
| `Proc<A>` | process-capable computation | `ProcessId`, child identity, scheduler, handles | `proc::unit`, `proc::bind`, `proc::from_act`, `proc::par`, `proc::await` |
| `Workflow<A>` | governed/admitted process computation | `WorkflowId`, run admission, role/capability admission, obligation/report state | `workflow::unit`, `workflow::bind`, `workflow::from_proc`, `workflow::requires`, `workflow::ensures` |

`Monad<K>` should express sequencing shape. It must not be the source of tower authority.

For example, `Monad<Result<_, E>>` gives domain sequencing for `Result`; it does not grant capability access, process identity, or workflow admission.

## 4. Generalized monadic lowering as the central alpha payoff

The value of exposing the algebra is not merely aesthetic. Alpha should prove the mechanism by allowing Ash libraries and user code to define and use monadic computation constructors beyond the built-in tower cases. Generalized `<-` lowering is therefore part of the alpha target: if a target `K : * -> *` has accepted `Monad<K>` evidence, `do:K { ... }` should lower through that evidence rather than through hardcoded tower-specific cases.

Positive examples:

```ash
fn map<M : * -> *, A, B>(ma: M<A>, f: Fn(A) -> B) -> M<B>
where M: Monad
{
    do:M {
        a <- ma;
        return f(a)
    }
}
```

```ash
fn lift2<M : * -> *, A, B, C>(ma: M<A>, mb: M<B>, f: Fn(A, B) -> C) -> M<C>
where M: Monad
{
    do:M {
        a <- ma;
        b <- mb;
        return f(a, b)
    }
}
```

```ash
fn with_timing<A>(body: Act<A>) -> Act<A> {
    do:Act {
        start <- clock::now();
        result <- body;
        end <- clock::now();
        _ <- log::duration(start, end);
        return result
    }
}
```

These functions are not methods of `Monad`, and should not need to be. They are third-party/library code over monadic computations.

### 4.1 Required alpha mechanisms

Full generalized `<-` lowering requires these implementation substrates to be planned and delivered as part of alpha:

- constructor-kinded type parameters such as `M : * -> *`;
- explicit `Monad<K>` evidence lookup for unary computation constructors;
- partial type-constructor application for do targets such as `Result<_, E>`;
- generic closures/lambdas in the lowered representation;
- evidence-selected method calls for `unit`/`return` and `bind`;
- monomorphization/specialization of accepted generic monadic functions;
- lowering of associated operation/method bodies where the evidence is user/library-defined;
- diagnostics for wrong-kind targets, missing evidence, ambiguous evidence, wrong `<-` RHS shape, and target/result mismatch.

This does not require every useful monad to exist in the standard library at alpha. It does require the mechanism by which user/library monads participate.

### 4.2 `Result<_, E>` as an acceptance case

`Result` is the canonical partial-constructor edge case for alpha do-notation:

```ash
do:Result<_, ParseError> {
    x <- parse_int(input);
    y <- parse_int(other);
    return x + y
}
```

The target elaborates to the unary constructor:

```text
K = λA. Result<A, ParseError>
```

Alpha should allow exactly the disciplined hole shape needed to form the do target. The `_` in `Result<_, E>` is a computation-result slot, not an unconstrained interactive typed hole. Broader typed-hole workflows remain separate from this alpha requirement.

### 4.3 Do-target declaration and inference

Alpha should require explicit targets or expected-type-directed targets.

Required:

```ash
do:Act { ... }
do:Option { ... }
do:Result<_, ParseError> { ... }
```

Allowed if the expected type fixes the constructor:

```ash
let r: Result<Int, ParseError> = do {
    x <- parse_int(input);
    return x + 1
}
```

Deferred:

```ash
do {
    x <- maybe_value;
    return x + 1
}
```

Fully free do-target inference without an explicit target or expected type is not required for alpha.

## 5. Static elaboration and monomorphization

The preferred compilation model is static:

```text
source generic function over monadic computation
  -> kind/evidence/type resolution
  -> monomorphization or specialization
  -> typed lowering to concrete operations
  -> runtime executes concrete Act/Proc/Workflow/Result/etc. operations
```

Runtime elaboration should be rare. If it exists at all, it should be explicit and not the default mechanism for resolving polymorphic computation code.

Positive example:

```text
map<Proc, Int, String>(...) lowers to proc::bind/proc::unit-specialized code.
```

Negative example:

```text
Runtime receives an unresolved `Monad<M>` dictionary and dynamically searches for an implementation while executing user code.
```

The latter undermines Ash's static-first design bias and complicates auditability, optimization, and verification.

## 6. OODA should move out of primitive IR

OODA has value as a workflow pattern, but it should not remain a semantic root of the language.

Current specs and implementation still expose OODA-shaped primitives in several places:

- [SPEC-001](../spec/SPEC-001-IR.md) lists workflow IR forms such as `Observe`, `Orient`, `Propose`, and `Decide`.
- [SPEC-003](../spec/SPEC-003-TYPE-SYSTEM.md) assigns effect classification directly to `Observe`, `Orient`, `Propose`, and `Decide` forms.
- [SPEC-004](../spec/SPEC-004-SEMANTICS.md) and [SPEC-025](../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) contain operational rules/traces for OODA-shaped workflow steps.
- [SPEC-041](../spec/SPEC-041-ASH-LINT-LIBRARY.md) includes an `Ooda` lint category and rules over `Workflow::Decide`.
- [SPEC-042](../spec/SPEC-042-ASH-SOURCE-FORMATTER.md) formats `Observe`, `Orient`, `Propose`, and `Decide` as direct workflow forms.

Alpha/future direction:

```text
OODA-specific vocabulary belongs in Ash libraries, templates, or lint/checking libraries.
By default, no OODA-specific form is retained as core alpha semantics.
The core IR should prefer generic computation, capability, contract, policy, and control-flow forms.
```

Compatibility sugar should be opt-in and justified by migration pressure, not assumed. The target user-facing spelling is ordinary library/template use such as `ooda::observe(...)`, `ooda::orient(...)`, and `ooda::decide(...)`, with the semantic force coming from capabilities, policies, contracts, and workflow construction rather than from primitive OODA IR nodes.

### 6.1 Positive example: OODA as library/template

Illustrative library/template form:

```ash
module ooda {
    fn cycle<A, B, C, D>(
        observe: Workflow<A>,
        orient: Fn(A) -> Workflow<B>,
        decide: Fn(B) -> Workflow<C>,
        act: Fn(C) -> Workflow<D>,
    ) -> Workflow<D> {
        do:Workflow {
            o <- observe;
            r <- orient(o);
            d <- decide(r);
            a <- act(d);
            return a
        }
    }
}
```

This keeps the pattern teachable and reusable without forcing `Observe`/`Orient`/`Decide` to be primitive IR nodes.

### 6.2 Negative example: OODA as hardwired semantics

Less desirable future shape:

```text
Core IR permanently distinguishes Observe/Orient/Propose/Decide as special workflow nodes.
Effect typing, provenance, policy, formatting, and lint rules all branch on those nodes.
Equivalent user-defined patterns cannot get the same treatment without compiler patches.
```

This preserves historical OODA framing at the cost of algebraic simplification.

## 7. Mature minimal IR and bytecode as alpha pressure

[FUTURE-005](../ideas/future/COMPILED-EXECUTION-SUBSTRATE.md) should be considered when defining alpha cutoffs. The alpha release does not need a JIT, but it should include an executable, bounded TCIR/AMIR/bytecode path for both pure and effectful subsets. A schema-only compiled substrate is too weak for alpha because it does not force enough operational decisions; the milestone should be small, but runnable.

The key payoff is not only performance. A TCIR/AMIR/bytecode pipeline forces hard answers to questions that can otherwise remain implicit in an interpreter:

- what is the canonical typed semantic representation after parsing/typechecking;
- what is the lowered abstract-machine representation;
- what facts survive lowering;
- where effect, capability, process, and workflow boundaries are explicit;
- how equivalent surface/library constructions become the same execution form;
- how a verifier can reject malformed execution artifacts;
- how debugging and auditing can trace bytecode back through IR to source.

### 7.1 Alpha-oriented interpretation of FUTURE-005

FUTURE-005 proposes this long-term order:

```text
Typed Canonical IR (TCIR)
  -> Ash Machine IR (AMIR)
  -> Ash Bytecode
  -> optional JIT backend
  -> post-v1 Ash-in-Ash/self-hosting
```

For alpha cutoff discussion, reinterpret that as:

1. TCIR/AMIR maturity is directly relevant.
2. A minimal executable bytecode/VM path is alpha-relevant, not merely a future artifact schema.
3. The pure subset should land first, but the alpha milestone includes effectful execution as well.
4. JIT is design pressure only, not an alpha deliverable.
5. Self-hosting remains beyond v1 and only a non-blocking design pressure.

### 7.2 Relation to the algebraic tower

The algebraic tower and compiled substrate reinforce each other.

If `Act`, `Proc`, and `Workflow` construction is truly visible algebra, then TCIR and AMIR can lower all equivalent constructions through a small set of explicit operations instead of preserving many syntax-specific cases.

Good lowering direction:

```text
surface syntax / library call / template expansion
  -> typed algebraic construction in TCIR
  -> explicit abstract-machine blocks in AMIR
  -> sectioned bytecode with verifier facts and optional traceability metadata
```

Bad lowering direction:

```text
surface syntax variant A -> special runtime path A
surface syntax variant B -> special runtime path B
library call C          -> unrelated runtime path C
```

The latter prevents the compiled substrate from becoming a simplification force.

### 7.3 Minimal IR/bytecode alpha candidate

The alpha target is not a full optimizing compiler. It is a narrow, mature execution spine with both pure and effectful coverage:

- a typed/canonical representation for accepted programs after parsing/typechecking;
- a lowered block/register AMIR subset for pure functions, generalized do lowering, and explicit effect/process/workflow boundaries;
- a bytecode logical schema and executable VM/interpreter for that subset;
- a verifier contract for required safety metadata;
- source/TCIR/AMIR/bytecode traceability in debug artifacts;
- equivalence tests between existing interpretation and AMIR/bytecode execution for the accepted subset.

Positive alpha candidate:

```text
fn add(x: Int, y: Int) -> Int { x + y }

source
  -> TCIR with resolved types/names
  -> AMIR block with registers and arithmetic op
  -> bytecode function section + signature/layout/verifier facts
  -> VM executes same result as interpreter
```

Positive tower-boundary candidate:

```ash
do:Act {
    x <- read_file(path);
    return x
}
```

Debug lowering can show:

```text
TCIR: Act bind + capability requirement file.read
AMIR: capability call block + success continuation + failure continuation
Bytecode: call-capability instruction/range + effect/capability table + source/TCIR/AMIR origin metadata
```

This does not require JIT.

#### 7.3.1 Phased alpha execution subsets

Implementation can still be phased from simpler to richer execution, but the alpha milestone should include all four bounded subsets:

1. Pure subset:
   - literals, variables, `let`;
   - arithmetic/comparison;
   - branches from `if`/`match`;
   - pure function calls and builtin pure calls;
   - ADT construction and pattern match where ADTs are alpha-stable;
   - verifier, source maps, and equivalence tests.
2. Act boundary subset:
   - `Act<A>` as executable effectful computation;
   - `do:Act` lowered through the same `Monad<Act>` path;
   - capability-provider runtime call-outs;
   - success/failure continuations and operational failure routing.
3. Effect VM subset for `Act` and `Proc`:
   - sequential effect steps and environment threading for `Act`;
   - process identity, child handles, `par`/spawn, `await`/join/gather, cancellation, and process-failure attribution for `Proc`;
   - explicit environment split/join rules for process concurrency.
4. Workflow obligations subset:
   - minimal executable governance boundary above `Proc`;
   - requirements, obligations, role/capability admission, reporting, and workflow failure reinterpretation;
   - enough behavior to run a governed workflow end-to-end.

This keeps implementation order sane without weakening the alpha release target.

### 7.4 JIT boundary

JIT is useful as a design constraint and possibly a test/spike tool for bytecode/AMIR shape. It should not be an alpha release target.

Alpha should preserve these JIT-friendly choices from FUTURE-005:

- block/register AMIR rather than a tiny stack VM as the semantic center;
- explicit basic blocks and control flow;
- explicit runtime-helper/capability calls;
- explicit layout/runtime categories;
- sectioned bytecode with optional JIT hints/profile data;
- bytecode that remains useful without JIT.

Alpha should not require:

- Cranelift/LLVM integration;
- native code generation;
- JIT region selection;
- direct JIT of workflow orchestration;
- direct JIT of capability/provider dispatch.

### 7.5 Spec-update pressure from compiled execution

The current [SPEC-001](../spec/SPEC-001-IR.md) says the IR is the canonical core IR and explicitly does not assume bytecode or JIT. That remains historically useful, but alpha planning needs a sharper layer split:

- Surface AST: syntax-preserving parser output.
- TCIR: typed canonical semantic authority.
- AMIR: lowered abstract-machine execution authority.
- Bytecode: durable, sectioned, verifiable artifact encoding AMIR.

Future spec work should decide whether SPEC-001 evolves into TCIR authority or remains a legacy/current-core-IR spec superseded by a new TCIR spec. Avoid trying to make one document own surface AST, typed canonical semantics, abstract-machine execution, and bytecode artifact schema at once.

### 7.6 Positive and negative examples

Positive:

```text
`act { ... }`, `do:Act { ... }`, and explicit `act::bind/act::unit` library calls converge in TCIR before AMIR lowering.
```

Negative:

```text
Legacy `act`, generalized `do`, workflow OODA nodes, and proc combinators each lower directly to unrelated bytecode op families.
```

Positive:

```text
Bytecode verifier checks control-flow validity, register initialization, import/export signatures, effect/capability boundary tables, and ABI versions without needing source spans.
```

Negative:

```text
Bytecode verification requires reparsing source or consulting full debug provenance.
```

Positive:

```text
Debug bytecode can explain an instruction range through bytecode -> AMIR -> TCIR -> source.
```

Negative:

```text
Optimized bytecode is an opaque Rust-serialized blob with no stable logical sections or verifier-independent schema.
```

## 8. Big-step and small-step semantics

Alpha should update or create both big-step and small-step semantics and state how they map to IR. Ash now has enough layers that prose-only semantics will not be sufficient for implementation or verification.

### 8.1 Big-step role

Big-step semantics should explain source-level meaning and typed elaboration results:

```text
Γ; Σ; ρ ⊢ e ⇓ v
Γ; Σ; ρ ⊢ c ⇓ K<v>
Γ; Σ; ρ ⊢ workflow ⇓ WorkflowOutcome
```

Use big-step rules for:

- pure expression evaluation;
- generalized `do` meaning as evidence-selected `unit`/`bind`;
- examples for `Option`, `Result<_, E>`, `Act`, `Proc`, and `Workflow`;
- equivalence between surface syntax and explicit algebraic/library calls.

### 8.2 Small-step role

Small-step semantics should own operational runtime behavior and VM correspondence:

```text
⟨term, env, store, runtime⟩ → ⟨term', env', store', runtime'⟩
```

For process/workflow execution, the state should account for process identities, mailboxes/handles, capability providers, obligations, reports, and failure state:

```text
⟨Processes, Handles, Capabilities, Obligations, Reports⟩
  →
⟨Processes', Handles', Capabilities', Obligations', Reports'⟩
```

Use small-step rules for:

- AMIR and bytecode instruction semantics;
- capability calls and operational failure;
- `Proc` scheduling, `par`, `await`, cancellation, and failure attribution;
- Workflow admission, obligation progression, reporting, and boundary outcomes.

### 8.3 Correspondence to IR

The intended correspondence is:

```text
Surface Ash
  -> typed elaboration / evidence resolution
  -> TCIR as typed semantic authority
  -> AMIR as abstract-machine execution authority
  -> bytecode as durable verifiable execution artifact
```

Correctness target:

```text
If source elaborates to TCIR t,
and t lowers to AMIR a,
and a encodes to bytecode b,
then executing b produces the value, failure, trace, and workflow outcome predicted by the TCIR/AMIR semantics for the accepted subset.
```

For effectful computations, equivalence includes more than final values. It must also preserve capability-call sequence, operational failure attribution, process identity/handle behavior, and workflow obligation/report outcomes.

## 9. Current reality vs alpha target

This section intentionally records drift and future spec-update pressure.

| Area | Current reality | Alpha/future target | Spec-update pressure |
| --- | --- | --- | --- |
| `do:K` target resolution | [SPEC-054](../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) Phase 105 used hidden dictionaries for `Act`/`Proc`; later work added explicit `Monad<K>` evidence boundary for some targets. | Full generalized `<-` lowering through accepted `Monad<K>` evidence is alpha scope; `Act`/`Proc`/`Workflow` expose Ash-visible algebra/evidence; hidden dictionaries are bridge only. | Update SPEC-054 to distinguish implemented bridge, current HKT/evidence reality, and alpha full-lowering requirement. |
| HKT / `Monad<K>` | [SPEC-067](../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md) implements constructor-kinded binders and evidence lookup at MVP boundary. General runtime lowering through arbitrary user-defined Monad methods remains deferred today. | Generic monadic library functions typecheck, select evidence, lower, and monomorphize statically for concrete targets. | New spec needed for generic monadic function elaboration, evidence-selected method calls, evidence-passing/monomorphization, and user-library combinators. |
| `Result<_, E>` | [SPEC-066](../spec/SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md) implements explicit hole target shape and distinguishes missing evidence. | `Monad<Result<_, E>>` should be expressible as type/evidence; full do lowering depends on evidence-selected method lowering. | SPEC-066 likely remains shape authority; future spec must own method lowering and monomorphization. |
| `Act` | [SPEC-047](../spec/SPEC-047-ACT-MONAD.md) already states Act opacity and library algebra. | Make Act's public algebra visibly participate in the same `Monad<Act>` path as `Proc`/`Workflow`. | Patch SPEC-047 after alpha design hardening to point to shared algebra/evidence authority. |
| `Proc` | [SPEC-048](../spec/SPEC-048-PROC-LIBRARY.md) defines public `Proc<A>`, `P<A>`, and proc library operations. | `Proc` algebra/evidence is Ash-visible; runtime implements opaque process mechanics only behind visible ops. | SPEC-048 is close in spirit; update with alpha no-magic rule and shared Monad/evidence relation. |
| `Workflow` | [SPEC-051](../spec/SPEC-051-WORKFLOW-SEMANTICS.md) defines workflow as governance above `Proc`, but workflow construction is not yet fully unified as library-visible algebra. | `Workflow<A>` has visible `unit`/`bind`/`from_proc`/`requires`/`ensures` construction API; workflow syntax lowers to it. | Future workflow algebra spec/update needed. |
| OODA | [SPEC-001](../spec/SPEC-001-IR.md), [SPEC-003](../spec/SPEC-003-TYPE-SYSTEM.md), [SPEC-004](../spec/SPEC-004-SEMANTICS.md), [SPEC-025](../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md), [SPEC-041](../spec/SPEC-041-ASH-LINT-LIBRARY.md), and [SPEC-042](../spec/SPEC-042-ASH-SOURCE-FORMATTER.md) still encode OODA forms directly. | By default no OODA primitive remains core alpha semantics; OODA becomes library/template/lint surface over generic computation/capability/policy forms. | Requires an IR simplification and compatibility/migration design before spec edits, with compatibility sugar opt-in rather than assumed. |
| Inference | Current `check_expr` is largely synthesis-first; local inference exists in several paths; bidirectional target inference is deferred. | Alpha requires strong local inference; bidirectional inference is desirable but may be restricted. | Future inference spec should separate alpha-local inference from beta bidirectional/contextual do-target inference. |
| Runtime elaboration | Some current paths still rely on hidden bridges and special runtime/lowering knowledge. | Static resolution and monomorphization are the default. Runtime elaboration is rare/explicit. | Future lowering/engine spec needed for evidence specialization and runtime boundary rules. |
| Execution IR / bytecode | [SPEC-001](../spec/SPEC-001-IR.md) owns current canonical core IR and explicitly does not assume bytecode/JIT. [FUTURE-005](../ideas/future/COMPILED-EXECUTION-SUBSTRATE.md) is exploratory only. | Alpha should include a mature minimal executable TCIR/AMIR/bytecode spine for pure and effectful subsets; JIT is only design pressure, not an alpha target. | New TCIR/AMIR/bytecode/VM design/spec set needed; decide whether SPEC-001 evolves or is superseded for TCIR authority. |

## 10. Spec-update starting points

When this note is promoted toward implementation-grade work, start with these spec updates or new specs.

### 10.1 Update SPEC-054 generalized do-notation

Current mismatch:

- SPEC-054's original MVP text says constructor-kinded parameters were not supported and `do:Result<_, E>` was future.
- Current reality has SPEC-066/SPEC-067 MVPs for holes/HKT/evidence boundaries.
- Alpha target further requires visible `Act`/`Proc`/`Workflow` evidence instead of source-evidence-free bridge dictionaries.

Required update themes:

1. Preserve Phase 105 bridge history.
2. Add post-SPEC-066/SPEC-067 current-state section.
3. Define alpha target: hidden dictionaries are migration scaffolding only.
4. Require full generalized `<-` lowering through accepted `Monad<K>` evidence, including user/library evidence.
5. Keep explicit-target or expected-type-directed target selection; defer fully free do-target inference.
6. Keep no implicit lifts and no domain-failure conflation.

### 10.2 Update SPEC-047 Act Monad

Current alignment:

- SPEC-047 already states Act opacity and library algebra.
- It still has Act-specific history and `invoke` semantics.

Required update themes:

1. Make `Monad<Act>` the shared sequencing evidence path.
2. Keep `ActEnv` opaque and runtime-managed.
3. Ensure Act-specific runtime authority is tied to visible operations such as `act::invoke`, not merely to `Monad`.

### 10.3 Update SPEC-048 Proc Library

Current alignment:

- SPEC-048 already defines public `Proc<A>`, `P<A>`, `proc::unit`, `proc::bind`, `proc::par`, `proc::await`, etc.

Required update themes:

1. Add explicit alpha no-magic rule.
2. State `Monad<Proc>` evidence relation.
3. Clarify static lowering/monomorphization for proc-generic combinators.
4. Preserve runtime ownership of `ProcessId`, scheduler, handle lifecycle, and failure observation.

### 10.4 Update or extend SPEC-051 Workflow Semantics

Current alignment:

- SPEC-051 correctly places Workflow as governance above Proc.

Required update themes:

1. Add `Workflow<A>` algebra/construction API: `unit`, `bind`, `from_proc`, `from_act`, `requires`, `ensures`, possibly `scope`.
2. Require workflow syntax to lower to workflow algebra/construction artifacts.
3. Keep role/capability admission and obligation/reporting as workflow-specific semantics, but make construction visible.

### 10.5 New spec: static evidence specialization and monadic generic functions

Needed for third-party generic monadic libraries.

Likely owns:

- generic functions over constructor-kinded parameters;
- `where M: Monad` or equivalent proposition/evidence syntax;
- evidence lookup and specialization;
- monomorphization strategy;
- lowering of `do:M` in generic functions;
- generic closure/lambda lowering;
- evidence-selected method calls and associated operation/method body lowering;
- diagnostics for ambiguous/missing evidence;
- `Result<_, E>` as a required partial-constructor acceptance case;
- non-goal: runtime dictionary search as normal execution semantics.

### 10.6 New design/spec set: TCIR, AMIR, and bytecode

Needed to turn FUTURE-005 into alpha-relevant architecture without making JIT an alpha target.

Likely split:

1. TCIR boundary/audit spec:
   - map current parser/typechecker/core/runtime carriers;
   - define typed canonical semantic authority;
   - define required facts: resolved names, types, effects, capabilities, contracts, source anchors.
2. AMIR design spec:
   - block/register abstract-machine model;
   - explicit control flow, calls, failure, yield, capability boundaries, and do/bind lowering;
   - textual/debug form and traceability back to TCIR.
3. Bytecode logical schema spec:
   - sectioned artifact format;
   - required execution/safety sections vs optional trace/debug sections;
   - verifier facts, import/export signatures, effect/capability tables, ABI/versioning.
4. Bytecode verifier spec:
   - safety contract independent of debug traceability;
   - load/link/reject behavior;
   - no source reparse requirement.
5. VM/cache implementation plan:
   - narrow pure subset first;
   - Act boundary and capability call-outs next;
   - effect VM for Act/Proc;
   - minimal Workflow requirements/obligations VM;
   - equivalence tests against interpreter/AMIR for values, failures, traces, process behavior, and workflow outcomes.

JIT should remain a later feasibility spike. It may be used experimentally to test AMIR/bytecode shape, but it should not be required for alpha.

### 10.7 New design/spec: semantics and IR correspondence

Needed before compiled execution becomes implementation-grade.

Likely owns:

- big-step semantics for source-level elaboration and user-visible meaning;
- small-step semantics for AMIR/bytecode, capabilities, Proc scheduling, and Workflow obligations;
- source -> TCIR -> AMIR -> bytecode correspondence theorem shape;
- effectful equivalence criteria beyond final values: traces, failure attribution, process identity, and workflow reports.

### 10.8 New design/spec: OODA demotion and IR simplification

Needed before editing older OODA-heavy specs.

Likely owns:

- inventory of OODA primitive surfaces;
- default removal from core alpha semantics;
- opt-in compatibility strategy for source syntax/examples;
- mapping from OODA forms to library/template/computation constructs such as `ooda::observe`, `ooda::orient`, and `ooda::decide`;
- exploration of whether ordinary libraries are enough or a template facility is needed;
- IR replacement target;
- lint/formatter migration;
- deprecation or preservation policy for existing examples.

## 11. Non-goals for alpha

Alpha does not require:

1. Full self-hosted implementation of `ActEnv`, process scheduler, workflow admission, or provenance internals in Ash.
2. General automatic do-target inference.
3. Full Haskell-style constraint solving.
4. Law proving for `Monad`, `Applicative`, or `Functor`.
5. Runtime dynamic typeclass search.
6. JIT, native-code generation, or direct JIT of workflow/capability dispatch.
7. Full self-hosting of the compiler/runtime.
8. Broad typed-hole development workflows beyond disciplined do-target partial constructor holes.
9. Fully free do-target inference without an explicit target or expected type.
10. Law proving for algebraic interfaces.
11. Arbitrary algebraic effect handlers, resumable continuations, effect rows, or user-defined operation handlers. Capabilities, tower-scoped operational failure, and workflow obligations are the alpha effect boundary.
12. Removing every legacy OODA spelling immediately, unless compatibility migration chooses that explicitly.
13. Rewriting every historical semantics document in one pass.

Alpha should require:

1. visible public algebra/evidence/construction APIs for `Act`, `Proc`, and `Workflow`;
2. full generalized `<-` lowering for accepted user/library `Monad<K>` evidence;
3. `Result<_, E>` and similar disciplined partial-constructor do targets;
4. clear typed lowering through visible APIs;
5. strong local inference for ordinary and do-block code, plus expected-type-directed do-target inference where available;
6. static resolution/monomorphization for generic monadic library functions where accepted;
7. executable pure and effectful TCIR/AMIR/bytecode/VM subsets, including bounded Act/Proc and minimal Workflow obligations execution;
8. updated big-step and small-step semantics with explicit IR correspondence;
9. explicit compatibility boundaries where bridge magic still exists;
10. no non-obvious runtime behavior beyond visible algebra.

## 12. Remaining open questions for future spec time

1. What exact syntax should Ash use for `where M: Monad` when `M : * -> *`?
2. Are `Monad` operations named `return`/`bind`, `unit`/`bind`, or both with one canonical spelling?
3. Should `Workflow<A>` be a direct opaque type constructor or a named envelope over synchronized `Proc<A>` plus `WorkflowContract<A>` artifacts?
4. How should typed elaboration preserve workflow construction artifacts across local variables and module boundaries?
5. Which OODA source forms, if any, remain accepted as opt-in compatibility sugar?
6. Are ordinary libraries sufficient for OODA replacement, or does Ash need first-class workflow/template facilities?
7. What exact monomorphization stage owns generic monadic combinator specialization: typeck, engine, or a new lowering pass?
8. What diagnostics are required to expose bridge/magic boundaries during alpha?
9. What are the exact instruction boundaries for the Act/Proc effect VM and the minimal Workflow obligations VM?
10. What exact traceability chain is required for debug artifacts: source -> Surface AST -> TCIR -> AMIR -> bytecode, or can Surface AST IDs be omitted after TCIR?
11. Which metadata is safety-critical and therefore unstrippable in bytecode artifacts?
12. Does SPEC-001 evolve into TCIR authority, or should a new TCIR spec supersede it while preserving SPEC-001 as current/historical core IR?

## 13. Design position to preserve

The alpha target is not: "the runtime is self-hosted in Ash."

The alpha target is:

```text
The algebraic construction model is visible in Ash.
The opaque runtime implements that model.
Surface syntax lowers through that model.
Libraries can derive constructions over that model.
Generalized monadic code lowers through accepted evidence.
Static compilation resolves and specializes accepted polymorphism.
Accepted programs have a runnable pure/effect/process/workflow execution path within alpha bounds.
Special runtime treatment remains an implementation detail, not an extra semantic authority.
```
