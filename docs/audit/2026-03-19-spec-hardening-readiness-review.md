# Audit: Spec Hardening Readiness Review

## Scope

This audit reviews the hardened spec set produced by TASK-177 through TASK-183, plus the canonical
`Result`-based recoverable-failure contract established by TASK-185, as the readiness gate for
resuming Rust convergence work.

Constraints of this audit:
- reporting only
- no specification files changed
- focus on whether the hardened corpus is mechanically usable for Rust convergence and stable for
  Lean formalization

Review date: 2026-03-19

## Summary

The hardened specification set is ready to gate Rust convergence and Lean formalization.

The remaining work in the repository is implementation convergence, not specification ambiguity:
- the canonical semantic corpus is explicit,
- source and handoff contracts are explicit for their layers,
- runtime-observable behavior has a single normative owner,
- execution-model neutrality is stated in the IR and operational semantics,
- recoverable failure is explicit `Result` dataflow rather than canonical `catch`.

## Readiness Findings

### Rust convergence readiness

The specs are now unambiguous enough to drive mechanical Rust convergence because:
- canonical core forms and surface sugar boundaries are frozen,
- parse, lowering, typing, runtime, and observable-behavior ownership are separated,
- `receive`, policy evaluation, ADT dynamics, and runtime-visible output each have one canonical
  contract,
- the readiness boundary explicitly keeps implementation drift in tasks and migration notes rather
  than in the normative specs.

Rust implementation work still remains, but it is now convergence against a fixed contract rather
than interpretation of moving prose.

### Lean formalization readiness

The specs are structured enough for Lean formalization because:
- [docs/reference/formalization-boundary.md](../reference/formalization-boundary.md) identifies the
  canonical semantic corpus,
- authoritative source/handoff contracts are separated from historical and migration-only
  artifacts,
- proof targets and bisimulation targets are listed explicitly,
- the old Lean reference sketch is demoted to legacy context rather than treated as a competing
  specification.

Lean work can now start from the canonical corpus directly and use the source/handoff contracts only
for the layers they own.

### IR execution-model neutrality

The IR contract remains execution-model-neutral:
- it does not assume a tree-walking interpreter,
- it does not assume a JIT,
- it allows future JIT compilation without changing the canonical meaning,
- the operational semantics are expressed as canonical behavior, not backend implementation detail.

### Recoverable failure

No canonical `catch` construct remains in the hardened language definition.
Recoverable failures are represented as explicit `Result` values and handled with ordinary
pattern matching.

## Gate Conclusion

Pass.

Rust convergence may resume under TASK-164 through TASK-176 using the hardened canonical contract.
Lean formalization has a stable starting corpus, and the audit does not require any further
specification hardening before implementation alignment begins.

The audit does not claim the Rust codebase already matches the specs; that remains the purpose of the
implementation tasks.
