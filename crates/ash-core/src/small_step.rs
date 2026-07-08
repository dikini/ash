//! Small-step IR compression types.
//!
//! This module provides a compressed intermediate representation for Ash
//! workflows, decomposing the nested `Workflow` AST into `Stmt` + `Frame`
//! + `Config`. It supports a non-recursive small-step abstract machine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    Capability, Expr, Guard, Name, Obligation, Pattern, Provenance, ReceiveArm, ReceiveMode, Role,
    Value, Workflow,
};

/// A list of statements together with their continuation frames.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StmtList {
    /// Statements to execute in order.
    pub stmts: Vec<Stmt>,
    /// Frames that resume after `stmts` finish.
    pub frames: Vec<Frame>,
}

/// A single workflow statement in the compressed IR.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    /// Terminal null.
    Done,
    /// Return the value of an expression.
    Ret {
        /// Expression to evaluate and return.
        expr: Expr,
    },
    /// Bind a pattern to the value of an expression.
    Let {
        /// Pattern to bind.
        pattern: Pattern,
        /// Expression to evaluate.
        expr: Expr,
    },
    /// Execute an action via a capability provider.
    Act {
        /// Provider name.
        provider_name: Name,
        /// Action name.
        action_name: Name,
        /// Argument expressions.
        arguments: Vec<Expr>,
        /// Guard condition.
        guard: Guard,
        /// Provenance metadata.
        provenance: Provenance,
        /// Optional name to bind the result to.
        result_name: Option<Name>,
    },
    /// Call a workflow by name.
    Call {
        /// Target workflow name.
        target: Name,
        /// Argument expressions.
        arguments: Vec<Expr>,
    },
    /// Branch on a boolean condition.
    If {
        /// Condition expression.
        condition: Expr,
        /// Branch taken when the condition is true.
        then_branch: StmtList,
        /// Branch taken when the condition is false.
        else_branch: StmtList,
    },
    /// Observe a capability and bind the result to a pattern.
    Observe {
        /// Capability to observe.
        capability: Capability,
        /// Pattern to bind the observed value.
        pattern: Pattern,
    },
    /// Evaluate an expression (orient) and continue.
    Orient {
        /// Expression to evaluate.
        expr: Expr,
    },
    /// Propose an action (advisory).
    Propose {
        /// Action name.
        action_name: Name,
        /// Argument expressions.
        action_arguments: Vec<Expr>,
    },
    /// Decide an expression under a policy.
    Decide {
        /// Expression to evaluate.
        expr: Expr,
        /// Policy name.
        policy: Name,
    },
    /// Check an obligation and continue.
    Check {
        /// Obligation to check.
        obligation: Obligation,
    },
    /// Scope a capability for the duration of the current block.
    With {
        /// Capability to scope.
        capability: Capability,
    },
    /// Assign a role for the duration of the current block.
    Oblig {
        /// Role to assign.
        role: Role,
    },
    /// Execute the primary block; on failure continue with the nearest `Catch` frame.
    Maybe {
        /// Primary block to attempt.
        primary: StmtList,
    },
    /// Execute the body under a `MustGuard` frame.
    Must {
        /// Body to execute.
        body: StmtList,
    },
    /// Iterate over a collection.
    ForEach {
        /// Pattern to bind each element.
        pattern: Pattern,
        /// Collection expression.
        collection: Expr,
        /// Body to execute for each element.
        body: StmtList,
    },
    /// Spawn a workflow instance.
    Spawn {
        /// Workflow type to spawn.
        entry_type: Name,
        /// Initialization expression.
        init: Expr,
        /// Pattern to bind the spawned instance.
        pattern: Pattern,
    },
    /// Split an instance into its address and control link.
    Split {
        /// Expression to split.
        expr: Expr,
        /// Pattern to bind the result.
        pattern: Pattern,
    },
    /// Kill a workflow instance.
    Kill {
        /// Target instance variable name.
        target: Name,
    },
    /// Pause a workflow instance.
    Pause {
        /// Target instance variable name.
        target: Name,
    },
    /// Resume a workflow instance.
    Resume {
        /// Target instance variable name.
        target: Name,
    },
    /// Check the health of a workflow instance.
    CheckHealth {
        /// Target instance variable name.
        target: Name,
    },
    /// Yield to a role and suspend awaiting a response.
    Yield {
        /// Target role.
        role: Name,
        /// Request expression.
        request: Expr,
        /// Variable to bind the response to on resume.
        resume_var: Name,
    },
    /// Set a value on a writable channel.
    Set {
        /// Capability name.
        capability: Name,
        /// Channel name.
        channel: Name,
        /// Value expression.
        value: Expr,
    },
    /// Send a value on a channel.
    Send {
        /// Capability name.
        capability: Name,
        /// Channel name.
        channel: Name,
        /// Value expression.
        value: Expr,
    },
    /// Introduce a linear obligation.
    Oblige {
        /// Obligation name.
        name: String,
    },
    /// Check/discharge a linear obligation.
    CheckObligation {
        /// Obligation name.
        name: String,
    },
    /// Receive a message from a mailbox.
    Receive {
        /// Receive mode.
        mode: ReceiveMode,
        /// Receive arms.
        arms: Vec<ReceiveArm>,
        /// Whether this is a control receive.
        control: bool,
    },
}

/// A continuation frame representing what to do after a statement list finishes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Frame {
    /// After the current block finishes, continue with `rest`.
    Seq {
        /// Remaining statements and frames to execute.
        rest: StmtList,
    },
    /// Iterate over the remaining items.
    ForEachIter {
        /// Pattern to bind each element.
        pattern: Pattern,
        /// Remaining items to iterate.
        items: Vec<Value>,
        /// Body to execute for each element.
        body: StmtList,
    },
    /// Catch an error and execute the fallback block.
    Catch {
        /// Fallback block to execute on error.
        fallback: StmtList,
    },
    /// Guard a `Must` block; errors are promoted to `MustFailure`.
    MustGuard,
    /// Resume a yielded workflow when a response is available.
    ResumeYield {
        /// Target role.
        role: Name,
        /// Request expression.
        request: Expr,
        /// Expected response type.
        expected_response_type: crate::workflow_contract::TypeExpr,
        /// Continuation to execute after resume.
        continuation: StmtList,
        /// Variable to bind the response to.
        resume_var: Name,
    },
    /// Restore the environment to a saved snapshot (used after a child call finishes).
    RestoreEnv {
        /// The parent environment to restore.
        saved: HashMap<Name, Value>,
    },
}

/// Execution configuration for the small-step machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Config {
    /// Active execution state.
    Running {
        /// Variable bindings.
        env: HashMap<Name, Value>,
        /// Current statements to execute.
        stmts: Vec<Stmt>,
        /// Continuation frames.
        frames: Vec<Frame>,
    },
    /// Terminal success with a value.
    Returned(Value),
    /// Terminal failure with a message.
    Rejected(String),
}

/// Lower a `Workflow` into an initial `Config`.
///
/// # Panics
/// Panics if the workflow contains unsupported variants.
pub fn lower_workflow(workflow: &Workflow) -> Config {
    let list = lower(workflow);
    Config::Running {
        env: HashMap::new(),
        stmts: list.stmts,
        frames: list.frames,
    }
}

fn lower(workflow: &Workflow) -> StmtList {
    match workflow {
        Workflow::Done => StmtList {
            stmts: vec![Stmt::Done],
            frames: vec![],
        },
        Workflow::Ret { expr } => StmtList {
            stmts: vec![Stmt::Ret { expr: expr.clone() }],
            frames: vec![],
        },
        Workflow::Let {
            pattern,
            expr,
            continuation,
        } => {
            let mut list = lower(continuation);
            let mut stmts = vec![Stmt::Let {
                pattern: pattern.clone(),
                expr: expr.clone(),
            }];
            stmts.append(&mut list.stmts);
            list.stmts = stmts;
            list
        }
        Workflow::Seq { first, second } => {
            let first_list = lower(first);
            let second_list = lower(second);
            let mut frames = vec![Frame::Seq { rest: second_list }];
            frames.extend(first_list.frames);
            StmtList {
                stmts: first_list.stmts,
                frames,
            }
        }
        Workflow::If {
            condition,
            then_branch,
            else_branch,
        } => StmtList {
            stmts: vec![Stmt::If {
                condition: condition.clone(),
                then_branch: lower(then_branch),
                else_branch: lower(else_branch),
            }],
            frames: vec![],
        },
        Workflow::Act {
            provider_name,
            action_name,
            arguments,
            guard,
            provenance,
            result_name,
            continuation,
        } => {
            let mut list = lower(continuation);
            let mut stmts = vec![Stmt::Act {
                provider_name: provider_name.clone(),
                action_name: action_name.clone(),
                arguments: arguments.clone(),
                guard: guard.clone(),
                provenance: provenance.clone(),
                result_name: result_name.clone(),
            }];
            stmts.append(&mut list.stmts);
            list.stmts = stmts;
            list
        }
        Workflow::Call {
            target,
            arguments,
            continuation,
        } => {
            let mut list = lower(continuation);
            let mut stmts = vec![Stmt::Call {
                target: target.clone(),
                arguments: arguments.clone(),
            }];
            stmts.append(&mut list.stmts);
            list.stmts = stmts;
            list
        }
        Workflow::Observe {
            capability,
            pattern,
            continuation,
        } => {
            let mut list = lower(continuation);
            let mut stmts = vec![Stmt::Observe {
                capability: capability.clone(),
                pattern: pattern.clone(),
            }];
            stmts.append(&mut list.stmts);
            list.stmts = stmts;
            list
        }
        Workflow::Orient { expr, continuation } => {
            let mut list = lower(continuation);
            let mut stmts = vec![Stmt::Orient { expr: expr.clone() }];
            stmts.append(&mut list.stmts);
            list.stmts = stmts;
            list
        }
        Workflow::Propose {
            action_name,
            action_arguments,
            continuation,
        } => {
            let mut list = lower(continuation);
            let mut stmts = vec![Stmt::Propose {
                action_name: action_name.clone(),
                action_arguments: action_arguments.clone(),
            }];
            stmts.append(&mut list.stmts);
            list.stmts = stmts;
            list
        }
        Workflow::Decide {
            expr,
            policy,
            continuation,
        } => {
            let mut list = lower(continuation);
            let mut stmts = vec![Stmt::Decide {
                expr: expr.clone(),
                policy: policy.clone(),
            }];
            stmts.append(&mut list.stmts);
            list.stmts = stmts;
            list
        }
        Workflow::Check {
            obligation,
            continuation,
        } => {
            let mut list = lower(continuation);
            let mut stmts = vec![Stmt::Check {
                obligation: obligation.clone(),
            }];
            stmts.append(&mut list.stmts);
            list.stmts = stmts;
            list
        }
        Workflow::With {
            capability,
            workflow,
        } => {
            let mut list = lower(workflow);
            let mut stmts = vec![Stmt::With {
                capability: capability.clone(),
            }];
            stmts.append(&mut list.stmts);
            list.stmts = stmts;
            list
        }
        Workflow::Oblig { role, workflow } => {
            let mut list = lower(workflow);
            let mut stmts = vec![Stmt::Oblig { role: role.clone() }];
            stmts.append(&mut list.stmts);
            list.stmts = stmts;
            list
        }
        Workflow::Maybe { primary, fallback } => {
            let mut list = lower(primary);
            list.frames.push(Frame::Catch {
                fallback: lower(fallback),
            });
            list
        }
        Workflow::Must { workflow: inner } => {
            let mut list = lower(inner);
            list.frames.push(Frame::MustGuard);
            list
        }
        Workflow::ForEach {
            pattern,
            collection,
            body,
        } => StmtList {
            stmts: vec![Stmt::ForEach {
                pattern: pattern.clone(),
                collection: collection.clone(),
                body: lower(body),
            }],
            frames: vec![],
        },
        Workflow::Spawn {
            entry_type,
            init,
            pattern,
            continuation,
        } => {
            let mut list = lower(continuation);
            let mut stmts = vec![Stmt::Spawn {
                entry_type: entry_type.clone(),
                init: init.clone(),
                pattern: pattern.clone(),
            }];
            stmts.append(&mut list.stmts);
            list.stmts = stmts;
            list
        }
        Workflow::Split {
            expr,
            pattern,
            continuation,
        } => {
            let mut list = lower(continuation);
            let mut stmts = vec![Stmt::Split {
                expr: expr.clone(),
                pattern: pattern.clone(),
            }];
            stmts.append(&mut list.stmts);
            list.stmts = stmts;
            list
        }
        Workflow::Kill {
            target,
            continuation,
        } => {
            let mut list = lower(continuation);
            let mut stmts = vec![Stmt::Kill {
                target: target.clone(),
            }];
            stmts.append(&mut list.stmts);
            list.stmts = stmts;
            list
        }
        Workflow::Pause {
            target,
            continuation,
        } => {
            let mut list = lower(continuation);
            let mut stmts = vec![Stmt::Pause {
                target: target.clone(),
            }];
            stmts.append(&mut list.stmts);
            list.stmts = stmts;
            list
        }
        Workflow::Resume {
            target,
            continuation,
        } => {
            let mut list = lower(continuation);
            let mut stmts = vec![Stmt::Resume {
                target: target.clone(),
            }];
            stmts.append(&mut list.stmts);
            list.stmts = stmts;
            list
        }
        Workflow::CheckHealth {
            target,
            continuation,
        } => {
            let mut list = lower(continuation);
            let mut stmts = vec![Stmt::CheckHealth {
                target: target.clone(),
            }];
            stmts.append(&mut list.stmts);
            list.stmts = stmts;
            list
        }
        Workflow::Yield {
            role,
            request,
            expected_response_type,
            continuation,
            span: _,
            resume_var,
        } => {
            let continuation_list = lower(continuation);
            StmtList {
                stmts: vec![Stmt::Yield {
                    role: role.clone(),
                    request: *request.clone(),
                    resume_var: resume_var.clone(),
                }],
                frames: vec![Frame::ResumeYield {
                    role: role.clone(),
                    request: *request.clone(),
                    expected_response_type: expected_response_type.clone(),
                    continuation: continuation_list,
                    resume_var: resume_var.clone(),
                }],
            }
        }
        Workflow::Set {
            capability,
            channel,
            value,
        } => StmtList {
            stmts: vec![Stmt::Set {
                capability: capability.clone(),
                channel: channel.clone(),
                value: value.clone(),
            }],
            frames: vec![],
        },
        Workflow::Send {
            capability,
            channel,
            value,
        } => StmtList {
            stmts: vec![Stmt::Send {
                capability: capability.clone(),
                channel: channel.clone(),
                value: value.clone(),
            }],
            frames: vec![],
        },
        Workflow::Oblige { name, span: _ } => StmtList {
            stmts: vec![Stmt::Oblige { name: name.clone() }],
            frames: vec![],
        },
        Workflow::CheckObligation { name, span: _ } => StmtList {
            stmts: vec![Stmt::CheckObligation { name: name.clone() }],
            frames: vec![],
        },
        Workflow::Receive {
            mode,
            arms,
            control,
        } => StmtList {
            stmts: vec![Stmt::Receive {
                mode: *mode,
                arms: arms.clone(),
                control: *control,
            }],
            frames: vec![],
        },
        Workflow::ProxyResume { .. } => {
            unimplemented!("small-step lowering for ProxyResume is not supported in the prototype")
        }
    }
}
