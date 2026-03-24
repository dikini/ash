//! Desugaring transformations for Ash surface AST.
//!
//! This module applies syntactic transformations to simplify the surface AST
//! before lowering to core IR. It handles:
//! - Semicolon sequencing transformation
//! - Optional binding expansion
//! - Other syntactic sugar elimination

use crate::surface::{Pattern, Workflow};

/// Desugar a workflow definition, applying all transformation passes.
///
/// This is the main entry point for desugaring.
pub fn desugar_workflow(workflow: &Workflow) -> Workflow {
    let workflow = desugar_sequencing(workflow);
    let workflow = desugar_optional_bindings(&workflow);
    desugar_nested_blocks(&workflow)
}

/// Desugar semicolon sequencing into explicit Seq nodes.
///
/// In surface syntax, `stmt1; stmt2; stmt3` represents sequential composition.
/// This pass ensures all sequencing is represented as explicit `Seq` nodes.
fn desugar_sequencing(workflow: &Workflow) -> Workflow {
    match workflow {
        Workflow::Seq {
            first,
            second,
            span,
        } => {
            let new_first = desugar_sequencing(first);
            let new_second = desugar_sequencing(second);
            Workflow::Seq {
                first: Box::new(new_first),
                second: Box::new(new_second),
                span: *span,
            }
        }

        // For workflows with continuations, desugar the continuation
        Workflow::Observe {
            capability,
            binding,
            continuation,
            span,
        } => {
            let new_continuation = continuation
                .as_ref()
                .map(|c| Box::new(desugar_sequencing(c)));
            Workflow::Observe {
                capability: capability.clone(),
                binding: binding.clone(),
                continuation: new_continuation,
                span: *span,
            }
        }

        Workflow::Orient {
            expr,
            binding,
            continuation,
            span,
        } => {
            let new_continuation = continuation
                .as_ref()
                .map(|c| Box::new(desugar_sequencing(c)));
            Workflow::Orient {
                expr: expr.clone(),
                binding: binding.clone(),
                continuation: new_continuation,
                span: *span,
            }
        }

        Workflow::Propose {
            action,
            binding,
            continuation,
            span,
        } => {
            let new_continuation = continuation
                .as_ref()
                .map(|c| Box::new(desugar_sequencing(c)));
            Workflow::Propose {
                action: action.clone(),
                binding: binding.clone(),
                continuation: new_continuation,
                span: *span,
            }
        }

        Workflow::Check {
            target,
            continuation,
            span,
        } => {
            let new_continuation = continuation
                .as_ref()
                .map(|c| Box::new(desugar_sequencing(c)));
            Workflow::Check {
                target: target.clone(),
                continuation: new_continuation,
                span: *span,
            }
        }

        Workflow::Let {
            pattern,
            expr,
            continuation,
            span,
        } => {
            let new_continuation = continuation
                .as_ref()
                .map(|c| Box::new(desugar_sequencing(c)));
            Workflow::Let {
                pattern: pattern.clone(),
                expr: expr.clone(),
                continuation: new_continuation,
                span: *span,
            }
        }

        // For other constructs, recursively desugar nested workflows
        Workflow::Decide {
            expr,
            policy,
            then_branch,
            else_branch,
            span,
        } => Workflow::Decide {
            expr: expr.clone(),
            policy: policy.clone(),
            then_branch: Box::new(desugar_sequencing(then_branch)),
            else_branch: else_branch
                .as_ref()
                .map(|e| Box::new(desugar_sequencing(e))),
            span: *span,
        },

        Workflow::If {
            condition,
            then_branch,
            else_branch,
            span,
        } => Workflow::If {
            condition: condition.clone(),
            then_branch: Box::new(desugar_sequencing(then_branch)),
            else_branch: else_branch
                .as_ref()
                .map(|e| Box::new(desugar_sequencing(e))),
            span: *span,
        },

        Workflow::For {
            pattern,
            collection,
            body,
            span,
        } => Workflow::For {
            pattern: pattern.clone(),
            collection: collection.clone(),
            body: Box::new(desugar_sequencing(body)),
            span: *span,
        },

        Workflow::Par { branches, span } => Workflow::Par {
            branches: branches.iter().map(desugar_sequencing).collect(),
            span: *span,
        },

        Workflow::With {
            capability,
            body,
            span,
        } => Workflow::With {
            capability: capability.clone(),
            body: Box::new(desugar_sequencing(body)),
            span: *span,
        },

        Workflow::Maybe {
            primary,
            fallback,
            span,
        } => Workflow::Maybe {
            primary: Box::new(desugar_sequencing(primary)),
            fallback: Box::new(desugar_sequencing(fallback)),
            span: *span,
        },

        Workflow::Must { body, span } => Workflow::Must {
            body: Box::new(desugar_sequencing(body)),
            span: *span,
        },

        // Set and Send with continuations
        Workflow::Set {
            capability,
            channel,
            value,
            continuation,
            span,
        } => {
            let new_continuation = continuation
                .as_ref()
                .map(|c| Box::new(desugar_sequencing(c)));
            Workflow::Set {
                capability: capability.clone(),
                channel: channel.clone(),
                value: value.clone(),
                continuation: new_continuation,
                span: *span,
            }
        }

        Workflow::Send {
            capability,
            channel,
            value,
            continuation,
            span,
        } => {
            let new_continuation = continuation
                .as_ref()
                .map(|c| Box::new(desugar_sequencing(c)));
            Workflow::Send {
                capability: capability.clone(),
                channel: channel.clone(),
                value: value.clone(),
                continuation: new_continuation,
                span: *span,
            }
        }

        // Receive - desugar all arm bodies
        Workflow::Receive {
            mode,
            arms,
            is_control,
            span,
        } => {
            let new_arms = arms
                .iter()
                .map(|arm| crate::surface::ReceiveArm {
                    pattern: arm.pattern.clone(),
                    guard: arm.guard.clone(),
                    body: desugar_sequencing(&arm.body),
                    span: arm.span,
                })
                .collect();
            Workflow::Receive {
                mode: mode.clone(),
                arms: new_arms,
                is_control: *is_control,
                span: *span,
            }
        }

        // Leaf nodes
        Workflow::Act { .. } | Workflow::Done { .. } | Workflow::Ret { .. } | Workflow::Oblige { .. } => workflow.clone(),
    }
}

/// Desugar optional bindings into explicit wildcard patterns where needed.
///
/// When a binding is omitted (e.g., `observe cap` without `as pattern`),
/// this pass inserts a wildcard pattern to make the binding explicit.
fn desugar_optional_bindings(workflow: &Workflow) -> Workflow {
    match workflow {
        Workflow::Observe {
            capability,
            binding,
            continuation,
            span,
        } => {
            let new_binding = binding.clone().or(Some(Pattern::Wildcard));
            let new_continuation = continuation
                .as_ref()
                .map(|c| Box::new(desugar_optional_bindings(c)));
            Workflow::Observe {
                capability: capability.clone(),
                binding: new_binding,
                continuation: new_continuation,
                span: *span,
            }
        }

        Workflow::Orient {
            expr,
            binding,
            continuation,
            span,
        } => {
            let new_continuation = continuation
                .as_ref()
                .map(|c| Box::new(desugar_optional_bindings(c)));
            Workflow::Orient {
                expr: expr.clone(),
                binding: binding.clone(),
                continuation: new_continuation,
                span: *span,
            }
        }

        Workflow::Propose {
            action,
            binding,
            continuation,
            span,
        } => {
            let new_continuation = continuation
                .as_ref()
                .map(|c| Box::new(desugar_optional_bindings(c)));
            Workflow::Propose {
                action: action.clone(),
                binding: binding.clone(),
                continuation: new_continuation,
                span: *span,
            }
        }

        // Recursively process other constructs
        Workflow::Seq {
            first,
            second,
            span,
        } => Workflow::Seq {
            first: Box::new(desugar_optional_bindings(first)),
            second: Box::new(desugar_optional_bindings(second)),
            span: *span,
        },

        Workflow::Check {
            target,
            continuation,
            span,
        } => {
            let new_continuation = continuation
                .as_ref()
                .map(|c| Box::new(desugar_optional_bindings(c)));
            Workflow::Check {
                target: target.clone(),
                continuation: new_continuation,
                span: *span,
            }
        }

        Workflow::Let {
            pattern,
            expr,
            continuation,
            span,
        } => {
            let new_continuation = continuation
                .as_ref()
                .map(|c| Box::new(desugar_optional_bindings(c)));
            Workflow::Let {
                pattern: pattern.clone(),
                expr: expr.clone(),
                continuation: new_continuation,
                span: *span,
            }
        }

        Workflow::Decide {
            expr,
            policy,
            then_branch,
            else_branch,
            span,
        } => Workflow::Decide {
            expr: expr.clone(),
            policy: policy.clone(),
            then_branch: Box::new(desugar_optional_bindings(then_branch)),
            else_branch: else_branch
                .as_ref()
                .map(|e| Box::new(desugar_optional_bindings(e))),
            span: *span,
        },

        Workflow::If {
            condition,
            then_branch,
            else_branch,
            span,
        } => Workflow::If {
            condition: condition.clone(),
            then_branch: Box::new(desugar_optional_bindings(then_branch)),
            else_branch: else_branch
                .as_ref()
                .map(|e| Box::new(desugar_optional_bindings(e))),
            span: *span,
        },

        Workflow::For {
            pattern,
            collection,
            body,
            span,
        } => Workflow::For {
            pattern: pattern.clone(),
            collection: collection.clone(),
            body: Box::new(desugar_optional_bindings(body)),
            span: *span,
        },

        Workflow::Par { branches, span } => Workflow::Par {
            branches: branches.iter().map(desugar_optional_bindings).collect(),
            span: *span,
        },

        Workflow::With {
            capability,
            body,
            span,
        } => Workflow::With {
            capability: capability.clone(),
            body: Box::new(desugar_optional_bindings(body)),
            span: *span,
        },

        Workflow::Maybe {
            primary,
            fallback,
            span,
        } => Workflow::Maybe {
            primary: Box::new(desugar_optional_bindings(primary)),
            fallback: Box::new(desugar_optional_bindings(fallback)),
            span: *span,
        },

        Workflow::Must { body, span } => Workflow::Must {
            body: Box::new(desugar_optional_bindings(body)),
            span: *span,
        },

        // Set and Send with continuations
        Workflow::Set {
            capability,
            channel,
            value,
            continuation,
            span,
        } => {
            let new_continuation = continuation
                .as_ref()
                .map(|c| Box::new(desugar_optional_bindings(c)));
            Workflow::Set {
                capability: capability.clone(),
                channel: channel.clone(),
                value: value.clone(),
                continuation: new_continuation,
                span: *span,
            }
        }

        Workflow::Send {
            capability,
            channel,
            value,
            continuation,
            span,
        } => {
            let new_continuation = continuation
                .as_ref()
                .map(|c| Box::new(desugar_optional_bindings(c)));
            Workflow::Send {
                capability: capability.clone(),
                channel: channel.clone(),
                value: value.clone(),
                continuation: new_continuation,
                span: *span,
            }
        }

        // Receive - desugar arm bodies
        Workflow::Receive {
            mode,
            arms,
            is_control,
            span,
        } => {
            let new_arms = arms
                .iter()
                .map(|arm| crate::surface::ReceiveArm {
                    pattern: arm.pattern.clone(),
                    guard: arm.guard.clone(),
                    body: desugar_optional_bindings(&arm.body),
                    span: arm.span,
                })
                .collect();
            Workflow::Receive {
                mode: mode.clone(),
                arms: new_arms,
                is_control: *is_control,
                span: *span,
            }
        }

        Workflow::Act { .. } | Workflow::Done { .. } | Workflow::Ret { .. } | Workflow::Oblige { .. } => workflow.clone(),
    }
}

/// Desugar nested blocks into flatter structures.
///
/// This pass simplifies nested blocks and handles edge cases like
/// single-statement blocks.
fn desugar_nested_blocks(workflow: &Workflow) -> Workflow {
    match workflow {
        // Flatten nested Seq if either side is Done
        Workflow::Seq {
            first,
            second,
            span,
        } => {
            let new_first = desugar_nested_blocks(first);
            let new_second = desugar_nested_blocks(second);

            // If first is Done, just return second
            if matches!(new_first, Workflow::Done { .. }) {
                return new_second;
            }

            // If second is Done, just return first
            if matches!(new_second, Workflow::Done { .. }) {
                return new_first;
            }

            Workflow::Seq {
                first: Box::new(new_first),
                second: Box::new(new_second),
                span: *span,
            }
        }

        // Recursively process other constructs
        Workflow::Observe {
            capability,
            binding,
            continuation,
            span,
        } => Workflow::Observe {
            capability: capability.clone(),
            binding: binding.clone(),
            continuation: continuation
                .as_ref()
                .map(|c| Box::new(desugar_nested_blocks(c))),
            span: *span,
        },

        Workflow::Orient {
            expr,
            binding,
            continuation,
            span,
        } => Workflow::Orient {
            expr: expr.clone(),
            binding: binding.clone(),
            continuation: continuation
                .as_ref()
                .map(|c| Box::new(desugar_nested_blocks(c))),
            span: *span,
        },

        Workflow::Propose {
            action,
            binding,
            continuation,
            span,
        } => Workflow::Propose {
            action: action.clone(),
            binding: binding.clone(),
            continuation: continuation
                .as_ref()
                .map(|c| Box::new(desugar_nested_blocks(c))),
            span: *span,
        },

        Workflow::Check {
            target,
            continuation,
            span,
        } => Workflow::Check {
            target: target.clone(),
            continuation: continuation
                .as_ref()
                .map(|c| Box::new(desugar_nested_blocks(c))),
            span: *span,
        },

        Workflow::Let {
            pattern,
            expr,
            continuation,
            span,
        } => Workflow::Let {
            pattern: pattern.clone(),
            expr: expr.clone(),
            continuation: continuation
                .as_ref()
                .map(|c| Box::new(desugar_nested_blocks(c))),
            span: *span,
        },

        Workflow::Decide {
            expr,
            policy,
            then_branch,
            else_branch,
            span,
        } => Workflow::Decide {
            expr: expr.clone(),
            policy: policy.clone(),
            then_branch: Box::new(desugar_nested_blocks(then_branch)),
            else_branch: else_branch
                .as_ref()
                .map(|e| Box::new(desugar_nested_blocks(e))),
            span: *span,
        },

        Workflow::If {
            condition,
            then_branch,
            else_branch,
            span,
        } => Workflow::If {
            condition: condition.clone(),
            then_branch: Box::new(desugar_nested_blocks(then_branch)),
            else_branch: else_branch
                .as_ref()
                .map(|e| Box::new(desugar_nested_blocks(e))),
            span: *span,
        },

        Workflow::For {
            pattern,
            collection,
            body,
            span,
        } => Workflow::For {
            pattern: pattern.clone(),
            collection: collection.clone(),
            body: Box::new(desugar_nested_blocks(body)),
            span: *span,
        },

        Workflow::Par { branches, span } => Workflow::Par {
            branches: branches.iter().map(desugar_nested_blocks).collect(),
            span: *span,
        },

        Workflow::With {
            capability,
            body,
            span,
        } => Workflow::With {
            capability: capability.clone(),
            body: Box::new(desugar_nested_blocks(body)),
            span: *span,
        },

        Workflow::Maybe {
            primary,
            fallback,
            span,
        } => Workflow::Maybe {
            primary: Box::new(desugar_nested_blocks(primary)),
            fallback: Box::new(desugar_nested_blocks(fallback)),
            span: *span,
        },

        Workflow::Must { body, span } => Workflow::Must {
            body: Box::new(desugar_nested_blocks(body)),
            span: *span,
        },

        // Set and Send with continuations
        Workflow::Set {
            capability,
            channel,
            value,
            continuation,
            span,
        } => Workflow::Set {
            capability: capability.clone(),
            channel: channel.clone(),
            value: value.clone(),
            continuation: continuation
                .as_ref()
                .map(|c| Box::new(desugar_nested_blocks(c))),
            span: *span,
        },

        Workflow::Send {
            capability,
            channel,
            value,
            continuation,
            span,
        } => Workflow::Send {
            capability: capability.clone(),
            channel: channel.clone(),
            value: value.clone(),
            continuation: continuation
                .as_ref()
                .map(|c| Box::new(desugar_nested_blocks(c))),
            span: *span,
        },

        // Receive - desugar arm bodies
        Workflow::Receive {
            mode,
            arms,
            is_control,
            span,
        } => {
            let new_arms = arms
                .iter()
                .map(|arm| crate::surface::ReceiveArm {
                    pattern: arm.pattern.clone(),
                    guard: arm.guard.clone(),
                    body: desugar_nested_blocks(&arm.body),
                    span: arm.span,
                })
                .collect();
            Workflow::Receive {
                mode: mode.clone(),
                arms: new_arms,
                is_control: *is_control,
                span: *span,
            }
        }

        Workflow::Act { .. } | Workflow::Done { .. } | Workflow::Ret { .. } | Workflow::Oblige { .. } => workflow.clone(),
    }
}

/// Simplify a workflow by removing unnecessary constructs.
///
/// This is an additional pass that can be run after desugaring to
/// produce cleaner output.
pub fn simplify(workflow: &Workflow) -> Workflow {
    desugar_nested_blocks(workflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::{ActionRef, Expr, Literal};
    use crate::token::Span;

    fn dummy_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    #[test]
    fn test_desugar_workflow_done() {
        let wf = Workflow::Done { span: dummy_span() };
        let result = desugar_workflow(&wf);
        assert!(matches!(result, Workflow::Done { .. }));
    }

    #[test]
    fn test_desugar_sequencing_flat() {
        // Seq of two dones should stay as Seq
        let wf = Workflow::Seq {
            first: Box::new(Workflow::Done { span: dummy_span() }),
            second: Box::new(Workflow::Done { span: dummy_span() }),
            span: dummy_span(),
        };
        let result = desugar_sequencing(&wf);
        assert!(matches!(result, Workflow::Seq { .. }));
    }

    #[test]
    fn test_desugar_optional_bindings_observe() {
        // Observe without binding gets wildcard
        let wf = Workflow::Observe {
            capability: "test".into(),
            binding: None,
            continuation: None,
            span: dummy_span(),
        };
        let result = desugar_optional_bindings(&wf);
        match result {
            Workflow::Observe { binding, .. } => {
                assert!(matches!(binding, Some(Pattern::Wildcard)));
            }
            _ => panic!("Expected Observe"),
        }
    }

    #[test]
    fn test_desugar_optional_bindings_observe_with_binding() {
        // Observe with existing binding should keep it
        let wf = Workflow::Observe {
            capability: "test".into(),
            binding: Some(Pattern::Variable("x".into())),
            continuation: None,
            span: dummy_span(),
        };
        let result = desugar_optional_bindings(&wf);
        match result {
            Workflow::Observe { binding, .. } => {
                assert!(matches!(binding, Some(Pattern::Variable(name)) if name.as_ref() == "x"));
            }
            _ => panic!("Expected Observe"),
        }
    }

    #[test]
    fn test_desugar_nested_blocks_eliminate_done_left() {
        // done; x should become just x
        let wf = Workflow::Seq {
            first: Box::new(Workflow::Done { span: dummy_span() }),
            second: Box::new(Workflow::Done { span: dummy_span() }),
            span: dummy_span(),
        };
        let result = desugar_nested_blocks(&wf);
        // Since both are Done, one gets eliminated
        assert!(matches!(result, Workflow::Done { .. }));
    }

    #[test]
    fn test_desugar_nested_blocks_eliminate_done_right() {
        // x; done should become just x
        let inner = Workflow::Act {
            action: ActionRef {
                name: "test".into(),
                args: vec![],
            },
            guard: None,
            span: dummy_span(),
        };
        let wf = Workflow::Seq {
            first: Box::new(inner.clone()),
            second: Box::new(Workflow::Done { span: dummy_span() }),
            span: dummy_span(),
        };
        let result = desugar_nested_blocks(&wf);
        // Should return just the Act
        assert!(matches!(result, Workflow::Act { .. }));
    }

    #[test]
    fn test_simplify() {
        let wf = Workflow::Seq {
            first: Box::new(Workflow::Done { span: dummy_span() }),
            second: Box::new(Workflow::Done { span: dummy_span() }),
            span: dummy_span(),
        };
        let result = simplify(&wf);
        assert!(matches!(result, Workflow::Done { .. }));
    }

    #[test]
    fn test_desugar_if() {
        let wf = Workflow::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_branch: Box::new(Workflow::Done { span: dummy_span() }),
            else_branch: Some(Box::new(Workflow::Done { span: dummy_span() })),
            span: dummy_span(),
        };
        let result = desugar_workflow(&wf);
        assert!(matches!(result, Workflow::If { .. }));
    }

    #[test]
    fn test_desugar_par() {
        let wf = Workflow::Par {
            branches: vec![
                Workflow::Done { span: dummy_span() },
                Workflow::Done { span: dummy_span() },
            ],
            span: dummy_span(),
        };
        let result = desugar_workflow(&wf);
        assert!(matches!(result, Workflow::Par { branches, .. } if branches.len() == 2));
    }

    #[test]
    fn test_desugar_maybe() {
        let wf = Workflow::Maybe {
            primary: Box::new(Workflow::Done { span: dummy_span() }),
            fallback: Box::new(Workflow::Done { span: dummy_span() }),
            span: dummy_span(),
        };
        let result = desugar_workflow(&wf);
        assert!(matches!(result, Workflow::Maybe { .. }));
    }

    #[test]
    fn test_desugar_must() {
        let wf = Workflow::Must {
            body: Box::new(Workflow::Done { span: dummy_span() }),
            span: dummy_span(),
        };
        let result = desugar_workflow(&wf);
        assert!(matches!(result, Workflow::Must { .. }));
    }

    #[test]
    fn test_desugar_with() {
        let wf = Workflow::With {
            capability: "db".into(),
            body: Box::new(Workflow::Done { span: dummy_span() }),
            span: dummy_span(),
        };
        let result = desugar_workflow(&wf);
        assert!(matches!(result, Workflow::With { .. }));
    }

    #[test]
    fn test_desugar_let() {
        let wf = Workflow::Let {
            pattern: Pattern::Variable("x".into()),
            expr: Expr::Literal(Literal::Int(42)),
            continuation: None,
            span: dummy_span(),
        };
        let result = desugar_workflow(&wf);
        assert!(matches!(result, Workflow::Let { .. }));
    }

    #[test]
    fn test_desugar_for() {
        let wf = Workflow::For {
            pattern: Pattern::Variable("item".into()),
            collection: Expr::Variable("items".into()),
            body: Box::new(Workflow::Done { span: dummy_span() }),
            span: dummy_span(),
        };
        let result = desugar_workflow(&wf);
        assert!(matches!(result, Workflow::For { .. }));
    }
}
