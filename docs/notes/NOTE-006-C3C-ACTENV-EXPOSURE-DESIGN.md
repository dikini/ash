# NOTE-006: C3c design pass for `std::act::guard` policy/environment exposure

Date: 2026-04-23
Status: Active design note for TASK-689C
Related tasks: TASK-683, TASK-689C, TASK-689
Related spec: SPEC-047 §2.5, §7.3, §8

## 1. Problem statement

After landing C3a/C3b, Ash now supports:
- typed record projection in expressions
- projected callable invocation through `FnApply`

However, the literal SPEC-047 ordinary-library `guard` sketch still depends on:
- `env.policies.check(policy)`

That expression cannot yet be implemented honestly in ordinary Ash library code because the relevant `env` carrier is still runtime-only.

The open C3c question is therefore not about parser/typechecker debt anymore. It is a boundary-design question:

Should Phase 97 expose runtime policy/environment state into Ash values, or should it preserve the existing runtime-only `ActEnv` boundary and provide a narrower bridge for `guard`?

## 2. Current hard constraints

### 2.1 TASK-683 / PLAN-097 decision gate

The current committed Phase-97 boundary is explicit:
- `ActEnv` is runtime-only
- `ActEnv` is not an Ash value

Evidence:
- TASK-683 requirement 2: `Do not expose ActEnv as an Ash value.`
- PLAN-INDEX decision gate D3: `ActEnv is runtime-only (not an Ash value)`
- `crates/ash-interp/src/act_env.rs` documents the same boundary in code.

### 2.2 What is already fixed

C3a/C3b already paid down the broader language debt that was the primary technical cause of the `guard` blocker:
- field access is now a real typed expression feature
- projected callable invocation is now parsed/typechecked/evaluated honestly

So C3c should not reopen or duplicate that work.

### 2.3 User-facing goal remains narrow

TASK-689 is still specifically about replacing placeholder `std::act` helpers with honest ordinary library implementations.

That means C3c should be judged by whether it unlocks an honest ordinary-library `guard` without causing disproportionate surface churn or violating already-resolved Phase-97 gates.

## 3. Design options

### Option A: Expose full `ActEnv` as an Ash value

Example target shape:
- ordinary Ash code receives `env`
- `env.policies` projects an Ash-visible runtime policy evaluator or policy stack
- `env.capability_ctx`, `env.provenance`, `env.effects` also become visible or at least representable

Pros:
- literal SPEC-047 sketch becomes expressible with minimal translation
- opens a broad future surface for environment inspection

Cons:
- directly violates TASK-683 / D3 without explicit re-scoping
- leaks runtime internals into the language surface
- forces decisions on serialization, equality, visibility, mutability, and capability safety for runtime carriers
- increases spec and implementation scope well beyond the immediate `guard` need

Assessment:
- Too large for Phase 97 without a new spec/plan track
- Not recommended as the next implementation step

### Option B: Expose a reduced Ash-visible environment facade

Example target shape:
- keep full `ActEnv` runtime-only
- inject a narrower Ash-visible record/facade into Act closures, such as:
  - `{ policies: PolicyHandle }`
  - or `{ policies: { check: Fn(Policy) -> Decision } }`

Pros:
- closer to literal `env.policies.check(policy)` source shape
- avoids exposing the entire runtime carrier

Cons:
- still weakens the current D3 boundary in practice
- introduces a new Ash-visible runtime facade type that must be specified and maintained
- raises further questions:
  - Is the facade only available inside Act closures?
  - Is it serializable/comparable?
  - Is it a real value or a compiler/runtime pseudo-value?
  - How are decision/result variants surfaced?

Assessment:
- More disciplined than full `ActEnv` exposure, but still a new language/runtime surface
- Could be valid in a later dedicated spec, but still too large for the current unblocking task unless explicitly promoted into a new plan item

### Option C: Preserve runtime-only `ActEnv`; add a narrow policy bridge

Example target shape:
- ordinary-library `guard` does not inspect `env` directly
- instead, library code calls a narrow runtime primitive/bridge such as:
  - `policy_check(policy)`
  - or `guard_check(policy)`
  - or another explicitly named primitive returning a stable Ash-level result

The bridge would be the only Ash-visible surface for this concern in Phase 97.

Pros:
- preserves TASK-683 and D3 honestly
- keeps Phase 97 additive and narrow
- unblocks TASK-689 without exposing runtime carriers wholesale
- leaves room for a later, separately-scoped environment-introspection feature if desired

Cons:
- the literal SPEC-047 `env.policies.check(policy)` sketch remains illustrative rather than directly surface-expressible
- requires documenting the bridge clearly so the spec and library do not drift

Assessment:
- Best fit for Phase 97 constraints
- Recommended path

## 4. Recommended C3c decision

Recommendation: choose Option C for Phase 97.

That means:
1. Preserve `ActEnv` as runtime-only.
2. Do not expose full runtime environment state as Ash values in TASK-689C.
3. Introduce, in a later implementation step, a narrow Ash-visible policy-check bridge sufficient for ordinary-library `guard`.
4. Update TASK-689 / TASK-689C / spec notes honestly to say:
   - C3a/C3b generalized the language surface for projection/member-call debt
   - C3c intentionally preserves the runtime-only environment boundary
   - ordinary-library `guard` should use an explicit policy bridge in Phase 97 rather than direct `env` introspection

## 5. Why this is still aligned with the broader-value goal

The broader technical debt was primarily:
- parser/runtime/typechecker inconsistency around field/member access

That debt is now addressed by C3a/C3b.

C3c is a different class of problem:
- runtime boundary design
- security/governance exposure policy
- language-surface contract size

So preserving the boundary here is not abandoning the broader-value work. It is refusing to smuggle a new runtime exposure model into TASK-689C without explicit design authority.

## 6. Proposed follow-up split

### For TASK-689C (current path)
- Conclude C3c design pass with Option C
- Implement only the narrow policy bridge required by `std::act::guard`
- Keep `ActEnv` runtime-only

### For a future separate feature (only if desired)
Create a new spec/plan/task line for:
- Ash-visible runtime environment/facade introspection for Act closures

That future track would answer deliberately:
- which runtime fields are exposed
- whether exposure is read-only
- whether values are serializable/comparable
- what the exact syntax and typing rules are
- whether the surface is general-purpose or Act-only

## 7. Exit criteria for this design pass

This note is successful if it establishes all of the following before runtime exposure work begins:
- the project does not accidentally violate TASK-683 / D3
- TASK-689C has a clear recommendation for next implementation work
- the literal SPEC-047 sketch is treated as a semantic target, not blindly as a mandatory surface syntax if that conflicts with the resolved boundary
- any larger environment-exposure feature is explicitly spun out rather than silently absorbed into Phase 97

## 8. Final recommendation summary

Use C3c to preserve the runtime-only `ActEnv` boundary and unblock `guard` through a narrow explicit policy bridge.

Do not expose full `ActEnv` during Phase 97.
Do not broaden runtime exposure without a separate spec/plan track.
Do not claim TASK-689 is unblocked until that narrow bridge exists and is tested end-to-end.
