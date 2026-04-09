# Capability Call Dispatch Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Split operational capability execution into explicit provider/action dispatch and add act-less operational call sugar for symbolic capability names and explicit `provider:action(...)`.

**Architecture:** Symbolic capability calls resolve to `(provider, action)` pairs in the resolver, while explicit `provider:action(...)` forms provide that pair directly. Parser, lowering, interpreter, engine, and provider traits all converge on one canonical runtime mechanism: `lookup(provider) -> execute(action_name, args)`.

**Tech Stack:** Rust, winnow parser, Ash core/interpreter/engine crates, active spec corpus

---

Canonical planning documents:

- [DESIGN-016](../design/DESIGN-016-CAPABILITY-CALL-DISPATCH.md)
- [PLAN-016](../plan/PLAN-016-CAPABILITY-CALL-DISPATCH.md)

Execution tasks:

1. [TASK-463](../plan/tasks/TASK-463-spec-capability-call-dispatch-contract.md)
2. [TASK-464](../plan/tasks/TASK-464-surface-operational-call-sugar.md)
3. [TASK-465](../plan/tasks/TASK-465-core-act-provider-action-shape.md)
4. [TASK-466](../plan/tasks/TASK-466-resolver-capability-target-pairs.md)
5. [TASK-467](../plan/tasks/TASK-467-provider-local-execute-dispatch.md)
6. [TASK-468](../plan/tasks/TASK-468-engine-provider-split-dispatch.md)
7. [TASK-469](../plan/tasks/TASK-469-capability-call-docs-and-examples.md)
8. [TASK-470](../plan/tasks/TASK-470-capability-call-dispatch-verification.md)

Recommended execution mode:

- Freeze the spec contract first.
- Land surface/core shape before runtime trait migration.
- Migrate engine providers only after runtime dispatch is explicit.
- Close with docs/examples and full verification.
