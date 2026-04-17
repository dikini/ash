# DESIGN-027: Small-Step IR Compression and Abstract Machine

## Status: Draft

## Overview

This design specifies a compression of the canonical Ash `Workflow` IR and a transition of the interpreter from a big-step recursive evaluator to a small-step abstract machine. The primary goals are:

1. **Reduce IR complexity** by eliminating continuation-embedding variants from the core `Workflow` AST.
2. **Simplify the interpreter** into an explicit state-machine loop with a control stack.
3. **Align the runtime with a future JIT target** by making the execution model opcode-like and stack-based.

This design does not change user-visible surface syntax. It is a core-IR and runtime transformation.

## Problem Statement

The current `Workflow` enum in `crates/ash-core/src/ast.rs` contains approximately 30 variants. Around 18 of them embed a `continuation: Box<Workflow>` or a sub-workflow field (`Observe`, `Orient`, `Propose`, `Decide`, `Check`, `Act`, `Let`, `If`, `ForEach`, `With`, `Maybe`, `Must`, `Spawn`, `Split`, `Kill`, `Pause`, `Resume`, `CheckHealth`, `Yield`).

This means the AST is **also the control stack**. The interpreter (`execute_workflow_inner` in `crates/ash-interp/src/execute.rs`) is a ~2,600-line recursive async evaluator that traverses this tree, threading runtime context through every recursive call. The Rust async call stack *is* the workflow control stack.

Consequences:
- **IR bloat:** Every sequencing construct requires a dedicated AST node with continuation fields.
- **Interpreter entanglement:** The evaluator fuses AST traversal, effect dispatch, blocking logic, and error propagation into a single recursive function.
- **JIT impedance mismatch:** A JIT backend would need to reconstruct control flow from a tree-shaped IR rather than operating on a flat opcode sequence and an explicit stack.

## Goals

1. Split the current `Workflow` IR into two orthogonal concepts:
   - `Stmt`: an immediate operation (leaf action, no continuation).
   - `Frame`: a pending continuation recorded on an explicit control stack.
2. Reduce the canonical `Stmt` enum to ~12 variants.
3. Define a small-step abstract machine (`Config` + `step` function) that replaces the recursive evaluator.
4. Preserve all existing semantic contracts: effects, provenance, trace, obligations, blocking, and `ControlLink` lifecycle.
5. Make blocking states (`Receive`, `Yield`) explicit in the configuration rather than implicit in parked futures.

## Non-Goals

1. Changing user-visible surface syntax.
2. Changing the big-step semantics contract (`SPEC-004`) or the small-step semantic backbone (`MCE-005`).
3. Adding expression-level micro-stepping in v1.
4. Implementing a JIT backend now.
5. Changing `Expr` evaluation; pure expressions remain atomic.

## Design

### 3.1 Canonical IR Split

#### `Stmt` — Immediate Operation

A `Stmt` represents the *current* action to execute. It never contains a continuation.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    /// Terminal null
    Done,
    /// Terminal value
    Ret { expr: Expr },
    /// Pure variable binding
    Let { pattern: Pattern, expr: Expr },
    /// Effectful action invocation
    Act {
        provider_name: Name,
        action_name: Name,
        arguments: Vec<Expr>,
        guard: Guard,
        provenance: Provenance,
        result_name: Option<Name>,
    },
    /// Capability observation
    Observe { capability: Capability, pattern: Pattern },
    /// Receive from mailbox
    Receive { mode: ReceiveMode, arms: Vec<ReceiveArm>, control: bool },
    /// Conditional dispatch (the condition is pure; branches are workflows)
    If { condition: Expr, then_branch: Workflow, else_branch: Workflow },
    /// Iterate over a pure collection
    ForEach { pattern: Pattern, collection: Expr, body: Workflow },
    /// Proposal (advisory)
    Propose { action_name: Name, action_arguments: Vec<Expr> },
    /// Decision under policy
    Decide { expr: Expr, policy: Name },
    /// Obligation check
    Check { obligation: Obligation },
    /// Capability scoping
    With { capability: Capability, workflow: Workflow },
    /// Obligation scoping
    Oblig { role: Role, workflow: Workflow },
    /// Modal fallback
    Maybe { primary: Workflow, fallback: Workflow },
    /// Mandatory success wrapper
    Must { workflow: Workflow },
    /// Spawn child workflow
    Spawn { workflow_type: Name, init: Expr, pattern: Pattern },
    /// Instance splitting
    Split { expr: Expr, pattern: Pattern },
    /// Control operations
    Kill { target: Name },
    Pause { target: Name },
    Resume { target: Name },
    CheckHealth { target: Name },
    /// Yield to proxy
    Yield {
        role: Name,
        request: Expr,
        expected_response_type: TypeExpr,
        resume_var: Name,
        span: Span,
    },
    /// Set capability channel
    Set { capability: Name, channel: Name, value: Expr },
    /// Send to capability channel
    Send { capability: Name, channel: Name, value: Expr },
    /// Workflow-to-workflow call
    Call { target: Name, arguments: Vec<Expr> },
    /// Introduce linear obligation
    Oblige { name: String, span: Span },
    /// Check obligation by name
    CheckObligation { name: String, span: Span },
}
```

**Note:** `If`, `ForEach`, `With`, `Oblig`, `Maybe`, and `Must` still contain sub-workflows, but these are **not** continuations in the linear-execution sense; they represent control-flow forks that the machine must dispatch into. Once dispatched, the *return path* is handled by `Frame`.

Wait — to keep the compression clean, we can go further. `If`, `ForEach`, `With`, `Oblig`, `Maybe`, and `Must` can be eliminated from `Stmt` entirely by desugaring them at lowering time into simpler primitives plus `Frame` pushes. However, to preserve the **canonical IR contract** (`SPEC-001`) minimally, we keep `Stmt` as the direct counterpart of today's `Workflow` but **without the `continuation` fields**. The lowering layer removes `continuation` fields; the sub-workflows in `If`/`ForEach` become independent `Workflow` values that are pushed onto the `Frame` stack at runtime.

Actually, the cleanest compressed form is:

```rust
pub enum Stmt {
    Done,
    Ret(Expr),
    Let(Pattern, Expr),
    Act { ... },          // no continuation, no result_name
    Observe(Capability, Pattern),
    Receive(ReceiveMode, Vec<ReceiveArm>, bool),
    Propose(Name, Vec<Expr>),
    Decide(Expr, Name),
    Check(Obligation),
    Spawn(Name, Expr, Pattern),
    Split(Expr, Pattern),
    Kill(Name),
    Pause(Name),
    Resume(Name),
    CheckHealth(Name),
    Yield { role, request, expected_response_type, resume_var, span },
    Set(Name, Name, Expr),
    Send(Name, Name, Expr),
    Call(Name, Vec<Expr>),
    Oblige(String, Span),
    CheckObligation(String, Span),
}
```

`If`, `Seq`, `ForEach`, `With`, `Oblig`, `Maybe`, and `Must` are **removed** from `Stmt`. They are realized through `Frame` pushes or through statement lifting / lowering transformations.

#### `Frame` — Continuation Stack Entry

A `Frame` records what to do with the value produced by the current `Stmt`.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Frame {
    /// Run this workflow next (used for Seq and bare block sequencing)
    Seq(Workflow),
    /// Bind the returned value to a pattern, then run the workflow
    BindPattern(Pattern, Workflow),
    /// Bind the returned value to a name, then run the workflow
    BindName(Name, Workflow),
    /// Iterate body over remaining values: (pattern, values, current_index, body)
    ForEachIter(Pattern, Vec<Value>, usize, Workflow),
    /// Resume from a capability scope
    ExitWith(Capability),
    /// Resume from an obligation scope
    ExitOblig(Role),
    /// Catch fallback for Maybe
    Catch(Workflow),
    /// Mandatory-success guard
    MustGuard,
    /// Resume from yield with the response bound to resume_var
    ResumeYield { resume_var: Name, continuation: Workflow },
}
```

#### `Workflow` — Residual Program

In the compressed IR, `Workflow` is reduced to a **sequential block** of statements:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workflow {
    pub statements: Vec<Stmt>,
}
```

**Interpretation:** A `Workflow` is a list of statements to execute in order. At runtime, the machine loads the first statement into `cfg.stmt` and, upon its completion, either pops a `Frame` or loads the next statement from the current workflow block.

However, to preserve call targets (`Spawn`, `Call`, `FnDef` bodies), we still need a standalone `Workflow` type. It is simply a `Vec<Stmt>`.

### 3.2 Runtime Configuration

The small-step configuration (`κ` in `MCE-005`) becomes a concrete Rust struct:

```rust
pub struct Config {
    /// Γ — runtime environment
    pub env: Env,
    /// Ω — obligation state
    pub obligations: Obligations,
    /// π — provenance state
    pub provenance: ProvenanceState,
    /// T — cumulative trace prefix
    pub trace: Trace,
    /// ε̂ — cumulative effect summary
    pub effects: EffectSummary,
    /// Current statement being reduced
    pub stmt: Stmt,
    /// Control stack (pending continuations)
    pub stack: Vec<Frame>,
    /// Current workflow block (for loading subsequent statements)
    pub block: Workflow,
    /// Index of next statement in `block.statements` (if any)
    pub next_index: usize,
}
```

**Note:** `block` and `next_index` represent the *current* sequential context. When entering a sub-workflow (e.g., the `then_branch` of an `If`), the machine pushes the return path as a `Frame` and replaces `block`/`next_index` with the new branch.

### 3.3 The Stepper Loop

The interpreter becomes an async loop over `step`:

```rust
pub async fn execute_config(
    mut cfg: Config,
    runtime: &Runtime,
) -> Result<Value, ExecError> {
    loop {
        match step(&mut cfg, runtime).await? {
            StepOutcome::Terminal(value) => return Ok(value),
            StepOutcome::Blocked(reason) => {
                // Scheduler parks this config; resumes when reason is satisfied
                runtime.park(cfg, reason).await;
            }
            StepOutcome::Progress(label) => {
                // Optional: emit trace/effect deltas to observers
                runtime.emit(label).await;
            }
        }
    }
}
```

The `step` function is the heart of the machine:

```rust
pub async fn step(
    cfg: &mut Config,
    runtime: &Runtime,
) -> Result<StepOutcome, ExecError> {
    match &cfg.stmt {
        Stmt::Done => {
            let outcome = pop_or_terminal(cfg, Value::Null)?;
            Ok(StepOutcome::Progress(StepLabel::silent(outcome)))
        }

        Stmt::Ret(expr) => {
            let value = eval_expr(expr, &cfg.env).map_err(ExecError::Eval)?;
            let outcome = pop_or_terminal(cfg, value)?;
            Ok(StepOutcome::Progress(StepLabel::silent(outcome)))
        }

        Stmt::Let(pattern, expr) => {
            let value = eval_expr(expr, &cfg.env).map_err(ExecError::Eval)?;
            let bindings = match_pattern(pattern, &value)
                .map_err(|_| ExecError::PatternMatchFailed { ... })?;
            cfg.env.extend_with(bindings);
            load_next_stmt(cfg);
            Ok(StepOutcome::Progress(StepLabel::silent()))
        }

        Stmt::Act { provider_name, action_name, arguments, guard, provenance, result_name } => {
            let guard_result = eval_guard(guard, &cfg.env).map_err(...)?;
            if !guard_result {
                return Err(ExecError::GuardFailed { ... });
            }
            let evaluated_args = arguments
                .iter()
                .map(|e| eval_expr(e, &cfg.env).map_err(ExecError::Eval))
                .collect::<Result<Vec<_>, _>>()?;

            let value = runtime
                .execute_act(provider_name, action_name, &evaluated_args)
                .await?;

            if let Some(name) = result_name {
                cfg.env.set(name.clone(), value.clone());
            }

            load_next_stmt(cfg);
            Ok(StepOutcome::Progress(StepLabel::emit(
                TraceFragment::act(action_name),
                EffectDelta::operational(),
            )))
        }

        Stmt::Observe(capability, pattern) => {
            let value = runtime.observe(capability).await?;
            let bindings = match_pattern(pattern, &value).map_err(...)?;
            cfg.env.extend_with(bindings);
            load_next_stmt(cfg);
            Ok(StepOutcome::Progress(StepLabel::emit(
                TraceFragment::observe(&capability.name),
                EffectDelta::from(capability.effect),
            )))
        }

        Stmt::Receive(mode, arms, control) => {
            match runtime.select_receive(cfg, mode, arms, *control).await? {
                ReceiveSelection::Selected(bindings, next_workflow) => {
                    cfg.env.extend_with(bindings);
                    enter_workflow(cfg, next_workflow);
                    Ok(StepOutcome::Progress(StepLabel::silent()))
                }
                ReceiveSelection::Blocked => {
                    Ok(StepOutcome::Blocked(BlockReason::Receive(*mode)))
                }
            }
        }

        Stmt::If { condition, then_branch, else_branch } => {
            let cond_val = eval_expr(condition, &cfg.env).map_err(ExecError::Eval)?;
            let chosen = match cond_val {
                Value::Bool(true) => then_branch.clone(),
                Value::Bool(false) => else_branch.clone(),
                _ => return Err(ExecError::Eval(EvalError::TypeMismatch { ... })),
            };
            // Push return-to-current-block frame
            cfg.stack.push(Frame::ResumeBlock(cfg.block.clone(), cfg.next_index));
            enter_workflow(cfg, chosen);
            Ok(StepOutcome::Progress(StepLabel::silent()))
        }

        Stmt::ForEach { pattern, collection, body } => {
            let coll_val = eval_expr(collection, &cfg.env).map_err(ExecError::Eval)?;
            let values = match coll_val {
                Value::List(vs) => vs,
                _ => return Err(ExecError::Eval(EvalError::TypeMismatch { ... })),
            };
            if values.is_empty() {
                load_next_stmt(cfg);
                Ok(StepOutcome::Progress(StepLabel::silent()))
            } else {
                cfg.stack.push(Frame::ForEachIter(
                    pattern.clone(), values, 0, body.clone(),
                ));
                enter_workflow(cfg, body.clone());
                Ok(StepOutcome::Progress(StepLabel::silent()))
            }
        }

        Stmt::With { capability, workflow } => {
            cfg.stack.push(Frame::ExitWith(capability.clone()));
            enter_workflow(cfg, workflow.clone());
            Ok(StepOutcome::Progress(StepLabel::silent()))
        }

        Stmt::Oblig { role, workflow } => {
            cfg.obligations.enter(role.clone());
            cfg.stack.push(Frame::ExitOblig(role.clone()));
            enter_workflow(cfg, workflow.clone());
            Ok(StepOutcome::Progress(StepLabel::silent()))
        }

        Stmt::Maybe { primary, fallback } => {
            cfg.stack.push(Frame::Catch(fallback.clone()));
            enter_workflow(cfg, primary.clone());
            Ok(StepOutcome::Progress(StepLabel::silent()))
        }

        Stmt::Must { workflow } => {
            cfg.stack.push(Frame::MustGuard);
            enter_workflow(cfg, workflow.clone());
            Ok(StepOutcome::Progress(StepLabel::silent()))
        }

        Stmt::Spawn { workflow_type, init, pattern } => {
            let init_val = eval_expr(init, &cfg.env).map_err(ExecError::Eval)?;
            let (link, child_workflow) = runtime.spawn(workflow_type, init_val).await?;
            let bindings = match_pattern(pattern, &Value::ControlLink(link))
                .map_err(...)?;
            cfg.env.extend_with(bindings);
            load_next_stmt(cfg);
            Ok(StepOutcome::Progress(StepLabel::silent()))
        }

        Stmt::Yield { role, request, expected_response_type, resume_var, span } => {
            let req_val = eval_expr(request, &cfg.env).map_err(ExecError::Eval)?;
            runtime.yield_to_proxy(cfg, role, req_val, expected_response_type, resume_var, span).await?;
            Ok(StepOutcome::Blocked(BlockReason::Yield))
        }

        // ... remaining variants (Set, Send, Call, etc.) follow the same pattern:
        // evaluate arguments atomically, perform effect, then load_next_stmt(cfg)
    }
}
```

**Helper: `pop_or_terminal`**

```rust
fn pop_or_terminal(cfg: &mut Config, value: Value) -> Result<(), ExecError> {
    while let Some(frame) = cfg.stack.pop() {
        match frame {
            Frame::Seq(next) => {
                enter_workflow(cfg, next);
                return Ok(());
            }
            Frame::BindPattern(pattern, next) => {
                let bindings = match_pattern(&pattern, &value).map_err(...)?;
                cfg.env.extend_with(bindings);
                enter_workflow(cfg, next);
                return Ok(());
            }
            Frame::BindName(name, next) => {
                cfg.env.set(name, value.clone());
                enter_workflow(cfg, next);
                return Ok(());
            }
            Frame::ForEachIter(pattern, values, idx, body) => {
                if idx + 1 < values.len() {
                    cfg.stack.push(Frame::ForEachIter(
                        pattern.clone(), values.clone(), idx + 1, body.clone(),
                    ));
                    let bindings = match_pattern(&pattern, &values[idx + 1]).map_err(...)?;
                    cfg.env.extend_with(bindings);
                    enter_workflow(cfg, body);
                } else {
                    // iteration complete; value is discarded (or last body value)
                }
                return Ok(());
            }
            Frame::ExitWith(_) => { /* pop and continue unwinding */ }
            Frame::ExitOblig(_) => { cfg.obligations.exit(); }
            Frame::Catch(_) => { /* fallback not needed; pop and continue */ }
            Frame::MustGuard => {
                if value.is_rejection() {
                    return Err(ExecError::MustFailure);
                }
            }
            Frame::ResumeYield { resume_var, continuation } => {
                cfg.env.set(resume_var, value.clone());
                enter_workflow(cfg, continuation);
                return Ok(());
            }
            Frame::ResumeBlock(block, next_index) => {
                cfg.block = block;
                cfg.next_index = next_index;
                load_next_stmt(cfg);
                return Ok(());
            }
        }
    }
    // Stack empty → terminal
    Err(ExecError::Terminal(value)) // or a dedicated terminal signal
}
```

*Note: The exact terminal signaling can be a dedicated `StepOutcome::Terminal(value)` rather than an error.*

### 3.4 Lowering from Surface to Compressed IR

The surface parser and current lowering layer can remain largely unchanged. The lowering pass (`crates/ash-parser/src/lower.rs`) transforms the current `Workflow` tree into the compressed form by **stripping continuation fields** and producing statement sequences + frame pushes.

| Current `Workflow` variant | Lowered to |
|---|---|
| `Done` | `Stmt::Done` |
| `Ret { expr }` | `Stmt::Ret(expr)` |
| `Let { pattern, expr, continuation }` | `Stmt::Let(pattern, expr)` followed by lowering of `continuation` appended to the current block |
| `Seq { first, second }` | statements of `first` appended with statements of `second` |
| `Act { provider, action, args, guard, provenance, result_name, continuation }` | `Stmt::Act { ... }` followed by continuation block |
| `If { condition, then_branch, else_branch }` | `Stmt::If { condition, then_branch: lowered(then), else_branch: lowered(else) }` |
| `ForEach { pattern, collection, body }` | `Stmt::ForEach { pattern, collection, body: lowered(body) }` |
| `Observe { capability, pattern, continuation }` | `Stmt::Observe(capability, pattern)` followed by continuation block |
| `With { capability, workflow }` | `Stmt::With { capability, workflow: lowered(workflow) }` |
| `Oblig { role, workflow }` | `Stmt::Oblig { role, workflow: lowered(workflow) }` |
| `Maybe { primary, fallback }` | `Stmt::Maybe { primary: lowered(primary), fallback: lowered(fallback) }` |
| `Must { workflow }` | `Stmt::Must { workflow: lowered(workflow) }` |
| `Spawn { ... continuation }` | `Stmt::Spawn { ... }` followed by continuation block |
| `Yield { ..., continuation }` | `Stmt::Yield { ... }` followed by continuation block |
| `Call { target, args, continuation }` | `Stmt::Call(target, args)` followed by continuation block |

**Key insight:** `Stmt` variants that contain sub-workflows (`If`, `ForEach`, `With`, etc.) do so only for **entry points** into nested blocks. Once entered, the machine uses `Frame` to return to the caller block. This is analogous to how a bytecode VM uses `JMP` and the call stack.

### 3.5 Blocking and Resumption

In the big-step interpreter, blocking `Receive` and `Yield` are realized by parking the entire Rust future in a registry. In the small-step machine:

- `Receive` returns `StepOutcome::Blocked(BlockReason::Receive)`.
- `Yield` returns `StepOutcome::Blocked(BlockReason::Yield { correlation_id })`.
- The runtime scheduler stores the `Config` (or a lightweight handle) in `SuspendedYields` / `Mailbox` registry.
- On resumption, the scheduler restores `cfg.stmt` to the next statement and continues the `step` loop.

This makes blocking **explicit and inspectable**, which is required for the `MCE-005` blocked-vs-stuck taxonomy.

### 3.6 Compatibility with Existing Runtime Infrastructure

The current registries (`ControlLinkRegistry`, `ProxyRegistry`, `SuspendedYields`, `Mailbox`) do not need to be rebuilt. They only need to store `Config` handles instead of boxed futures. The `RuntimeState` remains the owner of shared runtime state.

## Risks and Open Questions

1. **Error propagation for `Maybe`/`Must`:** Stack unwinding on `Err(ExecError)` must walk the `Frame` stack to find `Catch` or `MustGuard` frames. We must decide whether this is a dedicated `unwind_stack(cfg, error)` helper or a modification to `StepOutcome::Rejected`.

2. **`ProxyResume` handling:** A `ProxyResume` currently injects a value back into a suspended workflow. In the small-step model, this becomes: look up the suspended `Config` by correlation ID, push `Frame::ResumeYield { resume_var, continuation }`, set `cfg.stmt = Stmt::Done`, and run one step.

3. **Performance of `Frame` cloning:** `Frame` contains `Workflow` blocks. If workflows are large, cloning a `Frame` on every `push` could be expensive. Mitigation: use `Arc<Workflow>` or intern workflow blocks.

4. **Migration path:** We cannot atomically replace the interpreter. A staged migration is:
   - Stage 1: Add `Stmt`, `Frame`, and `Config` alongside existing `Workflow`.
   - Stage 2: Implement the stepper and run it behind a feature flag.
   - Stage 3: Migrate all tests to the stepper.
   - Stage 4: Remove `execute_workflow_inner` and the old `Workflow` continuation fields.

## Acceptance Criteria

1. A written spec amendment (`SPEC-001` addendum or new `SPEC-0XX`) that defines the compressed IR.
2. A prototype `step` function in `crates/ash-interp/src/small_step.rs` that passes at least the `ash-interp` unit test suite.
3. `cargo check`, `cargo clippy`, and `cargo test` pass with no warnings.
4. A compatibility proof (document or test matrix) showing that the small-step machine produces identical observable results to the big-step interpreter for all existing test cases.
