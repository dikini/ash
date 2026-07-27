# Runnable Ash Semantic Realization Design

## Goal

Make target Ash executable through one production path while keeping the operational semantics,
the λAsh calculi, conformance evidence, and optional Verus pilots aligned.

## Architecture

```text
Surface Ash → checked Core → checked CPS → Engine CPS executor → terminal envelope
                                              ▲
CLI request ──────────────────────────────────┤
daemon request ───────────────────────────────┘
```

The λAsh calculi are mathematical descriptions of CPS terms, configurations, and transitions.
`λAsh-CPS₀` describes the control kernel; `λAsh-Effect` extends that same transition system with
effect operations, frames, handlers, providers, and resume behavior. Neither calculus is a runtime
route or an intermediate representation emitted after CPS.

## Delivery model

Feature tasks remain compositional: Surface, Core, CPS, admission/runtime, terminal projection,
and conformance each have explicit owners and handoffs. PLAN-203 separately owns the composition
gate: a feature becomes runnable only when its outputs reach the shared Engine path and the
appropriate CLI/daemon integration case.

Verus is a non-blocking experimental assurance track. Selected high-value pilots may prove a Rust
view/refinement property. All other proof candidates remain explicit traceability dispositions
until deliberately selected; they never prevent an executable semantic feature from shipping.
