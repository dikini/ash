# NOTE-004: Function vs Capability vs Workflow vs Effect Taxonomy

**Date:** 2026-04-21
**Status:** Open — needs resolution
**Priority:** High — architectural foundation

## Problem

Ash currently has multiple overlapping constructs for "doing work," and the boundaries between them are not crisply defined:

1. **`fn` (pure functions)** — `pub fn` with Ash body, no effects. Dispatched by module resolver.
2. **`builtin fn` (Rust-runtime functions)** — declared in `.ash`, body in Rust via `eval.rs` dispatch table. Currently includes both pure ops (string, list, json) and things that should arguably be capabilities.
3. **Capabilities** — `capability X with act/observe ... end`, backed by `CapabilityProvider` trait. Effectful. Registered with engine.
4. **Workflows** — `workflow main() ...` — top-level orchestration units with explicit effect tracking.
5. **Effects** — `Effect` enum (Epistemic, Deliberative, Evaluative, Operational) — tracks computational power but is only loosely coupled to the above constructs.

## Tensions

- `builtin fn` is a grab bag: `string::to_upper` (pure) and `process::run` (was effectful, now converted) shared the same dispatch path. The conversion of `process::run` to a capability was correct, but the remaining builtins (json, regex) may also need scrutiny.
- The four-way classification from Phase 96 Track B (`pub fn`, `builtin fn`, `capability+act`, `extern fn`) was a start but doesn't resolve the semantic question: what is the *user-facing* mental model?
- `Effect` levels are reported by providers but don't gate anything at the language level yet.
- Workflows and capabilities both carry effects but through different mechanisms (workflow-level effect annotations vs provider-level effect methods).

## Open Questions

1. Should `builtin fn` be restricted to *only* pure operations? If so, what about json_parse — is it pure or effectful?
2. Should the `Effect` enum gate which constructs can use which operations (e.g., only `Operational` workflows can call `act execute`)?
3. Is the distinction between `observe` (epistemic, lazy arg eval) and `execute` (operational, eager arg eval) sufficient, or do we need more granular effect tracking?
4. How do `extern fn` (future FFI) and `builtin fn` relate — should they converge?
5. Should workflows have explicit effect annotations that are checked at typeck time?

## Action

Needs a dedicated spec (or amendment to SPEC-004/SPEC-044) that defines the taxonomy crisply and specifies what can appear where.
