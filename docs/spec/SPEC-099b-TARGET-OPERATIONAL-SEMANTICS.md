---
id: spec.ash.operational-semantics.target
title: Ash Target Operational Semantics
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-07-27
verified_against:
  specs:
    - docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md
    - docs/spec/SPEC-100-CORE-TYPE-CHECKING.md
    - docs/spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md
  audits:
    - docs/audit/2026-06-29-target-spec-notes-gap-audit.md
---

# SPEC-099b: Ash Target Operational Semantics

**Status:** Draft — target Core/CPS operational semantics.
**Scope:** This document defines target operational behavior after surface expansion and
surface-to-Core lowering. It includes Core big-step rules, Core/CPS small-step rules, provider-frame
dispatch, structured traps, dynamic contracts, lazy/memo forcing, trace facts, and temporal
monitors.
**Depends on:** SPEC-098b, SPEC-098c, SPEC-100, SPEC-101.

## Relationship to the λAsh calculus suite

This document states target operational behavior for checked Core/CPS. The λAsh calculus suite is
the corresponding mathematical presentation of its CPS portion: `λAsh-CPS₀` explains control,
and `λAsh-Effect` extends that explanation to effectful CPS execution. The suite is not a second
interpreter or lowering route. Each completed extension must name its CPS encoding, operational
state correspondence, Rust Engine view, and terminal-projection agreement. PLAN-203 owns delivery
of that one executable Surface → Core → CPS → Engine path for CLI and daemon clients.

## 1. Relationship to Phase 159

The Phase 159 CPS interpreter semantics remain useful implementation context for continuation
capture, `LetCont`, `Jump`, `Call`, `LetRec`, and shallow handler execution. They are not the full
target semantics. Target semantics now also covers Core terms before CPS, provider frames as runtime
authority, structured diagnostic traps, contract sidecars, trace facts, and monitor behavior.

## 2. Machine state

A target state is:

```text
Σ ::= ⟨term, η, χ, μ, τ, Ω⟩
```

where:

- `η` is the value environment;
- `χ` is the handler/provider frame stack;
- `μ` is the lazy/memo store;
- `τ` is the trace ledger;
- `Ω` is the active monitor set.

Outcomes are values, structured traps, or stuck states for malformed unchecked input. Checked Core
programs should not reach stuck states except through explicitly unchecked/foreign boundaries.

## 3. Core big-step semantics

Core big-step judgment:

```text
Γ; Σ ⊢ e ⇓ outcome
```

Big-step rules summarize checked Core behavior before CPS expansion:

```text
Γ; Σ ⊢ v ⇓ Value(v)
Γ; Σ ⊢ e1 ⇓ Value(v)    Γ[x↦v]; Σ ⊢ e2 ⇓ o
------------------------------------------------
Γ; Σ ⊢ let x = e1 in e2 ⇓ o

Γ; Σ ⊢ m ⇓ Value(v)    Γ[x↦v]; Σ ⊢ k ⇓ o
------------------------------------------------
Γ; Σ ⊢ bind x <- m; k ⇓ o
```

Rows compose by row union through sequencing, but contract summaries compose through the Hoare
predicate-transformer obligations described in SPEC-097b.

## 4. Core/CPS small-step semantics

Small-step judgment:

```text
Σ -> Σ'
```

Representative rules:

```text
⟨LetVal(x, v, t), η, χ, μ, τ, Ω⟩
  -> ⟨t, η[x↦eval(v,η)], χ, μ, τ, Ω⟩

⟨Jump(k, a, ρ), η, χ, μ, τ, Ω⟩
  -> ⟨body(k), captured_env(k)[param(k)↦eval(a,η)], χ, μ, τ, Ω⟩

⟨Call(f, args, k, ρ), η, χ, μ, τ, Ω⟩
  -> ⟨body(f), call_env(f,args,k,η), χ, μ, τ, Ω⟩
```

Small-step owns control behavior, handler/provider-frame traversal, forced thunks, trace emission,
and monitor advancement. Big-step rules may be derived for pure checked fragments.

## 5. Handler and provider frames

Frame stack entries include:

```text
Frame ::= HandlerFrame { clauses, done, residual_row, resume_policy, origin }
        | ProviderFrame { op, provider, authority, origin }
        | MonitorFrame { monitor_id, origin }
```

Operation dispatch searches innermost to outermost by impl/type-qualified operation identity such
as `PosixFs::read` or a checked abstract identity such as `F::read`. Handler frames provide
program-level handling. Provider frames provide runtime authority for operations admitted at a
boundary. Handler and provider frames are searched in the same pass; an inner provider shadows an
outer handler, and an inner handler shadows an outer provider.

```text
lookup_op(op, HandlerFrame(op, clause) :: χ) = Handler(clause)
lookup_op(op, ProviderFrame(op, provider) :: χ) = Provider(provider)
lookup_op(op, frame :: χ) = lookup_op(op, χ)      if frame does not match op
lookup_op(op, []) = UnhandledEffect(op)
```

Provider frames are not skipped. They are the runtime authority representation for provider-backed
operations. A provider call may emit trace facts and may return success, operational failure, or a
structured trap depending on the boundary contract.

Computation rows do not install handler or provider frames. They describe requirements that must
already be discharged by the frame stack, boundary admission facts, or other kind-specific
discharge evidence before the operation is allowed to run.

### Deep affine handler decision (superseding the Phase 159 shallow rule)

This paragraph is the target rule for source handlers and supersedes the former Phase 159
"remove that shallow handler" wording. A `HandlerFrame` retains its checked clauses in source
order, one `done` clause, and a structurally normalized residual row. For a raised operation, the
innermost matching frame still wins under the lookup rules above; within that handler, the first
source-ordered checked clause whose concrete operation identity matches wins. No row creates,
selects, or installs a frame.

When a handler clause is entered, its own frame is absent while the clause body evaluates. The
clause receives an **affine** `resume`: it may be invoked zero or one time, never more. Invoking
`resume(v)` evaluates the captured continuation under the same handler frame reinstalled at its
original stack position, so operations raised by that resumed tail are again handled by the same
ordered handler. The surrounding stack remains in its prior order; TASK-1993 innermost-first
handler/provider lookup is unchanged. A zero-use clause is abortive and returns its clause result
without resuming the captured tail.

A normal completion of the handled computation, including a normally completed resumed tail,
is routed through the handler's `done` clause exactly once; it is not returned raw. The result of
an abortive operation clause is the handler result for that branch. Residual rows are structural:
handled concrete operation identities are removed only as represented by the checked computation,
the remaining ordered/open-tail structure is retained, and clause-body effects are accounted for
by the checked handler facts. This typing information never authorizes a handler or provider frame.

If a provider frame matches, the evaluator invokes the provider handler with the operation
arguments and resume continuation while preserving the provider frame. If no frame matches, the
outcome is structured missing-discharge failure (`UnhandledEffect(op)` in the current CPS
interpreter).

## 6. Structured traps and bottom

Trap is operational bottom:

```text
⟨Trap(reason), η, χ, μ, τ, Ω⟩ -> Outcome::Trap(reason)
```

Trap has any expected result type but contributes no local row. The reason is structured, not a
string. Relevant reasons include:

- `ContractViolation(ContractDiagnostic)`;
- `ContractPredicateFault(PredicateFaultDiagnostic)`;
- `TemporalContractViolation(TemporalDiagnostic)`;
- `TemporalMonitorFault(TemporalDiagnostic)`;
- ordinary operational bottom reasons from Core/CPS.

Trap propagation preserves the original payload, source origin, blame label, and diagnostic data.

## 7. Dynamic contracts

Dynamic contract checks execute runtime check plans produced by SPEC-098c/SPEC-100.

```text
run_check(plan, η, snapshots) = true
---------------------------------------
⟨CheckContract(plan, body), η, χ, μ, τ, Ω⟩ -> ⟨body, η, χ, μ, τ, Ω⟩

run_check(plan, η, snapshots) = false
---------------------------------------
⟨CheckContract(plan, body), η, χ, μ, τ, Ω⟩ -> Trap(ContractViolation(diag(plan)))

run_check(plan, η, snapshots) = fault
---------------------------------------
⟨CheckContract(plan, body), η, χ, μ, τ, Ω⟩ -> Trap(ContractPredicateFault(diag(fault)))
```

A false predicate and a predicate evaluator fault are distinct. Default dynamic contract failure is
structured bottom. Recoverable behavior requires explicit `fail` in the row and an ordinary handler
for that failure.

## 8. Lazy and memo forcing

Strict values evaluate at binding time. Lazy values evaluate at each force. Memo values evaluate on
the first force and replay the terminal result thereafter.

```text
force(lazy thunk)  -> evaluate thunk each time
force(memo empty)  -> evaluate thunk; store Value or Trap
force(memo stored) -> replay stored Value or Trap
```

Contract checks attached to lazy computations run at each force. Contract checks attached to memo
computations run at first force; replay preserves the stored terminal diagnostic and blame label if
the first force trapped.

## 9. Trace facts and temporal monitors

Trace-producing operations append facts to the trace ledger:

```text
emit(fact, τ) = τ · fact
advance(Ω, fact) = Ω'
```

After each emitted fact, active monitors advance. Monitor outcomes are distinct:

- satisfied/ongoing monitor state continues;
- violated formula traps with `TemporalContractViolation`;
- ill-formed monitor state, missing alphabet data, or monitor execution failure traps with
  `TemporalMonitorFault`.

Trace contracts are not handlers. They observe the ledger generated by lowered trace events and
provider/runtime boundaries.

## 10. Surface anchors

Historical `Pure`, `Act`, `Proc`, and `Workflow` names may appear only as legacy reference
vocabulary. Target operational semantics are defined over one ambient computation model with
row/admission facts, trace obligations, and observation boundaries; these names do not introduce
surface forms, Core terms, IR nodes, public stdlib types, runtime entry paths, or separate
operational languages.

## 11. Phase 159 interpreter context

The Phase 159 interpreter rules are retained as implementation context with these correspondences:

| Phase 159 concept | Target role |
|---|---|
| `LetVal`, `LetPrim`, `LetCont`, `Jump`, `Call`, `If`, `LetRec` | CPS small-step term forms |
| continuation capture | still required for `Jump`/`Call` semantics |
| shallow handler frames | historical Phase 159 behavior, superseded by the deep affine target rule in §5 |
| provider persistence | represented by explicit provider frames |
| `Trap(reason)` | structured operational bottom with typed payloads |

Older Phase 159-only limitations such as isolated interpreter scope, scaffold-only row checking, and
missing contract/monitor behavior are historical implementation boundaries, not target semantics.

## 12. See also

- [SPEC-095c: Surface AST, Macro Expansion, and Notation](SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-098b: Target IR](SPEC-098b-TARGET-IR.md)
- [SPEC-098c: Surface-to-Core Lowering](SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [SPEC-100: Core Type Checking](SPEC-100-CORE-TYPE-CHECKING.md)
- [SPEC-101: Lazy and Memo Computation Modes](SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md)
- [PLAN-159: CPS IR Interpreter](../plan/PLAN-159-CPS-IR-INTERPRETER.md)

## 13. Changelog

- 2026-07-03: Reconciled Phase 184 handler/provider semantics: lookup is one innermost-to-outermost pass across handler and provider frames, raise/handle behavior is explicit, and missing discharge is `UnhandledEffect(op)`.
- 2026-07-03: Reconciled Phase 183 operation authority model: dispatch keys are impl/type-qualified operation identities, and rows never install provider/handler frames.
- 2026-06-29: Recast as target Core/CPS operational semantics. Added Core big-step, Core/CPS small-step, provider-frame dispatch, structured traps, dynamic contracts, lazy/memo forcing, trace facts, temporal monitors, and Phase 159 context boundaries.
- 2026-06-19: Initial Phase 159 CPS interpreter semantics for the isolated prototype.
