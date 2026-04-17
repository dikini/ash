# NOTE-002: DESIGN-027 Execution Review

## Status: Review Complete

## Summary

This note records a line-by-line review of `DESIGN-027-SMALL-STEP-IR-COMPRESSION.md` against the current `Workflow` AST (`crates/ash-core/src/ast.rs`) and the big-step interpreter (`crates/ash-interp/src/execute.rs`). The review validates whether every current `Workflow` variant maps cleanly to the proposed `Stmt` + `Frame` decomposition.

## Method

1. Enumerate all `Workflow` variants in `ast.rs` (lines 17–203).
2. Inspect the corresponding match arm in `execute_workflow_inner_observed` (`execute.rs`).
3. Determine the `Stmt` mapping and any required `Frame` behavior.
4. Flag anomalies, missing variants, or implementation risks.

## Review Matrix

| # | Workflow Variant | Has `continuation` | Stmt Mapping | Frame Requirement | Mapping Quality | Notes |
|---|------------------|-------------------|--------------|-------------------|-----------------|-------|
| 1 | `Done` | No | `Stmt::Done` | None | ✅ Clean | Terminal null |
| 2 | `Ret { expr }` | No | `Stmt::Ret(expr)` | None | ✅ Clean | Terminal value |
| 3 | `Let { pattern, expr, continuation }` | Yes | `Stmt::Let(pattern, expr)` | `Frame::Seq(continuation)` | ✅ Clean | Pattern bind + continue |
| 4 | `Seq { first, second }` | N/A (two sub-workflows) | Lowered into stmts | `Frame::Seq(second)` | ✅ Clean | Prototype verified |
| 5 | `Observe { capability, pattern, continuation }` | Yes | `Stmt::Observe(cap, pattern)` | `Frame::Seq(continuation)` | ✅ Clean | Same shape as `Let` |
| 6 | `Orient { expr, continuation }` | Yes | `Stmt::Orient(expr)` | `Frame::Seq(continuation)` | ✅ Clean | No-op eval + continue |
| 7 | `Propose { action_name, action_arguments, continuation }` | Yes | `Stmt::Propose(...)` | `Frame::Seq(continuation)` | ✅ Clean | Advisory no-op |
| 8 | `Decide { expr, policy, continuation }` | Yes | `Stmt::Decide(expr, policy)` | `Frame::Seq(continuation)` | ✅ Clean | Policy branch or reject |
| 9 | `Check { obligation, continuation }` | Yes | `Stmt::Check(obligation)` | `Frame::Seq(continuation)` | ✅ Clean | Check then continue |
| 10 | `Act { provider_name, action_name, arguments, guard, provenance, result_name, continuation }` | Yes | `Stmt::Act(...)` | `Frame::Seq(continuation)` + result_name bind | ✅ Clean | Prototype verified; bare-act (continuation=Done) handled by `pop_or_terminal` |
| 11 | `Oblig { role, workflow }` | Sub-workflow | `Stmt::Oblig(role, workflow)` | `Frame::ExitOblig(role)` | ✅ Clean | Enters role scope |
| 12 | `If { condition, then_branch, else_branch }` | Two branches | `Stmt::If(condition, then, else)` | `Frame::Seq(tail)` if in middle of block | ✅ Clean | Prototype verified |
| 13 | `ForEach { pattern, collection, body }` | Sub-workflow | `Stmt::ForEach(pattern, collection, body)` | `Frame::ForEachIter(pattern, values, idx, body)` | ⚠️ Moderate | Big-step uses a Rust `for` loop; small-step needs explicit iterator frame |
| 14 | `With { capability, workflow }` | Sub-workflow | `Stmt::With(capability, workflow)` | `Frame::ExitWith(capability)` | ✅ Clean | Scope entry/exit |
| 15 | `Maybe { primary, fallback }` | Two sub-workflows | `Stmt::Maybe(primary, fallback)` | `Frame::Catch(fallback)` | ⚠️ Moderate | Requires stack unwinding on `Err` to find `Catch` frame |
| 16 | `Must { workflow }` | Sub-workflow | `Stmt::Must(workflow)` | `Frame::MustGuard` | ⚠️ Moderate | Requires stack inspection on `Err` |
| 17 | `Set { capability, channel, value }` | No | `Stmt::Set(...)` | None | ✅ Clean | Terminal null |
| 18 | `Send { capability, channel, value }` | No | `Stmt::Send(...)` | None | ✅ Clean | Terminal null |
| 19 | `Spawn { workflow_type, init, pattern, continuation }` | Yes | `Stmt::Spawn(...)` | `Frame::Seq(continuation)` + pattern bind | ✅ Clean | Same shape as `Observe` |
| 20 | `Split { expr, pattern, continuation }` | Yes | `Stmt::Split(...)` | `Frame::Seq(continuation)` + pattern bind | ✅ Clean | Same shape as `Observe` |
| 21 | `Kill { target, continuation }` | Yes | `Stmt::Kill(target)` | `Frame::Seq(continuation)` | ✅ Clean | Effect + continue |
| 22 | `Pause { target, continuation }` | Yes | `Stmt::Pause(target)` | `Frame::Seq(continuation)` | ✅ Clean | Effect + continue |
| 23 | `Resume { target, continuation }` | Yes | `Stmt::Resume(target)` | `Frame::Seq(continuation)` | ✅ Clean | Effect + continue |
| 24 | `CheckHealth { target, continuation }` | Yes | `Stmt::CheckHealth(target)` | `Frame::Seq(continuation)` | ✅ Clean | Effect + continue |
| 25 | `Oblige { name, span }` | No | `Stmt::Oblige(name, span)` | None | ✅ Clean | Terminal null; mutates obligations |
| 26 | `CheckObligation { name, span }` | No | `Stmt::CheckObligation(name, span)` | None | ✅ Clean | Terminal bool; mutates obligations |
| 27 | `Yield { role, request, expected_response_type, continuation, span, resume_var }` | Yes | `Stmt::Yield(...)` | `Frame::ResumeYield { resume_var, continuation }` | ⚠️ Moderate | Big-step returns `ExecError::YieldSuspended`; small-step returns `StepOutcome::Blocked` |
| 28 | `ProxyResume { value, value_type, correlation_id, span }` | No | N/A (runtime-only) | N/A | ⚠️ Special | This is **not** a normal workflow step. It is an external runtime injection that restores a suspended config. It should be handled by the scheduler, not the stepper. |
| 29 | `Receive { mode, arms, control }` | Arms contain workflows | `Stmt::Receive(mode, arms, control)` | Arms become `StmtList` | ⚠️ Moderate | Big-step delegates to `execute_core_receive`; small-step needs explicit blocked/selected branching |
| 30 | `Workflow::Call` | N/A | N/A | N/A | ❌ Missing | `DESIGN-027` references `Workflow::Call`, but this variant does **not** exist in current `ast.rs` (as of commit `0f52fc2`). It was added in TASK-590 work but is not in `main`. |

## Key Findings

### 1. `Workflow::Call` gap

`DESIGN-027` §3.4 lists `Call { target, arguments, continuation }` as a lowering target, but the current `Workflow` enum does **not** contain this variant. This indicates either:
- The design doc is forward-looking to TASK-590 substrate work, or
- `Workflow::Call` lives on a feature branch that has not merged to `main` yet.

**Impact:** The small-step prototype cannot lower `Workflow::Call` because the variant is absent. Once TASK-590 merges, `Call` will map exactly like `Act` (evaluate arguments, dispatch, bind result, continue).

### 2. `ProxyResume` is not a stepper statement

In the big-step interpreter, `ProxyResume` evaluates a value expression, looks up a suspended yield by correlation ID, binds the value to `resume_var`, and then **recursively executes the stored continuation workflow**.

In the small-step model, this should **not** be a `Stmt` at all. Instead:
1. The scheduler holds the suspended `Config`.
2. On proxy resume, the scheduler pushes `Frame::ResumeYield { resume_var, continuation }` onto the config's stack.
3. The scheduler sets `cfg.stmt = Stmt::Done`.
4. The next `step` pops the `ResumeYield` frame, binds the value, and loads the continuation.

This keeps the stepper pure and pushes the cross-workflow messaging concern to the runtime scheduler.

### 3. `ForEach` requires a new `Frame::ForEachIter`

The current big-step interpreter evaluates the collection and then runs a Rust `for` loop over the items, recursively calling `execute_workflow_inner_observed` for each iteration. In the small-step machine, this loop must be externalized:

- `Stmt::ForEach` evaluates the collection and pushes `Frame::ForEachIter(pattern, values, 0, body)`.
- The frame handler (in `pop_or_terminal` or equivalent) binds the first item and loads the body.
- When the body returns, the frame handler checks if more items remain. If so, it re-pushes itself with `idx + 1` and loads the body again.
- If the list is empty, `Stmt::ForEach` immediately returns `Value::Null`.

This is straightforward but was not included in the prototype.

### 4. `Maybe` and `Must` need error-stack unwinding

Currently, `Maybe` catches **any** `Err(_)` from the primary branch and falls back. `Must` passes the error through unchanged.

In the small-step machine, when `step` produces an `ExecError`, the machine must:
1. Walk the `Frame` stack from top to bottom.
2. If a `Frame::Catch(fallback)` is found, replace the config with the fallback workflow and continue.
3. If a `Frame::MustGuard` is found after no `Catch`, promote the error to `ExecError::MustFailure`.
4. If neither is found, propagate the error as terminal rejection.

This requires a dedicated `unwind_stack(cfg, error)` helper that the prototype did not implement.

### 5. `Yield` blocking semantics are already aligned

The big-step interpreter returns `Err(ExecError::YieldSuspended { ... })`. In the small-step model:
- `Stmt::Yield` evaluates the request.
- Pushes `Frame::ResumeYield { resume_var, continuation }`.
- Returns `StepOutcome::Blocked(BlockReason::Yield { correlation_id })`.
- The scheduler stores the config and resumes it later.

This maps cleanly and was explicitly designed in `DESIGN-027`.

### 6. `Receive` blocking semantics need a small-step wrapper

The big-step interpreter delegates to `execute_core_receive`, which handles mailbox polling, arm selection, timeout, and blocking. For the small-step machine:
- `Stmt::Receive` should call a helper `select_receive_outcome(...)`.
- If an arm is selected, bind the pattern and load the arm's body.
- If blocked, return `StepOutcome::Blocked(BlockReason::Receive)`.
- The scheduler resumes when a message arrives.

The existing `execute_core_receive` can be reused internally, but its interface may need to return `ReceiveSelection` instead of directly executing the body.

## Verdict

**Mapping feasibility: ✅ Viable, with 4 follow-up items.**

1. **Resolve `Workflow::Call` presence.** Either wait for TASK-590 to merge or add the variant to the compressed IR spec once it lands.
2. **Implement `Frame::ForEachIter`, `Frame::Catch`, `Frame::MustGuard`, and `Frame::ResumeYield`.** These are the only frames missing from the prototype.
3. **Add `unwind_stack` for `Maybe`/`Must` error handling.** This is the most delicate part of the transition.
4. **Handle `ProxyResume` at the scheduler level.** Remove it from the `Stmt` enum entirely.

The remaining variants (`Orient`, `Propose`, `Decide`, `Check`, `Oblig`, `With`, `Spawn`, `Split`, `Kill`, `Pause`, `Resume`, `CheckHealth`, `Oblige`, `CheckObligation`, `Set`, `Send`) all follow the exact same "evaluate atomically, then continue" pattern already proven by the prototype's `Act` implementation.

## Prototype Coverage Gap

The small-step prototype (`TASK-604`) only implemented:
- `Done`, `Ret`, `Let`, `Seq`, `If`, `Act`

This covers the **core sequencing and effect primitives** but leaves ~24 variants as `unimplemented!()` in the lowering function. The next iteration should prioritize:
1. `ForEach`, `Maybe`, `Must` (control-flow complexity)
2. `Yield`, `Receive` (blocking state)
3. `Spawn`, `Split` (instance lifecycle)
4. Everything else (mechanical)
