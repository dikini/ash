# Capability Call Dispatch Design

This document records the approved design direction for splitting operational execution into an
explicit `(provider, action)` target while adding act-less workflow sugar for symbolic capability
calls and explicit `provider:action(...)`.

Canonical design document:

- [DESIGN-016: Capability Call Dispatch Split and Operational Call Sugar](../design/DESIGN-016-CAPABILITY-CALL-DISPATCH.md)

Key decisions:

1. Core/runtime ACT execution must preserve separate provider and action fields.
2. Symbolic capability names remain resolver-owned and resolve to `(provider, action)` targets.
3. The language should support both explicit `provider:action(...)` and symbolic capability calls.
4. Act-less workflow sugar should default to an `always` guard and allow `when guard`.
5. `SPEC-025` is affected narrowly and must be updated with the same split helper boundary.
