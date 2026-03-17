//! Testing utilities for Ash Core
//!
//! This module provides helper functions and builders for constructing
//! test fixtures and common test scenarios.

use crate::ast::{Capability, Expr, Pattern, Workflow};
use crate::effect::Effect;
use crate::value::Value;

/// Builder for constructing Workflows in tests
pub struct WorkflowBuilder {
    workflow: Workflow,
}

impl WorkflowBuilder {
    /// Start with a terminal workflow
    pub fn done() -> Self {
        Self {
            workflow: Workflow::Done,
        }
    }

    /// Start with a return workflow
    pub fn ret(expr: Expr) -> Self {
        Self {
            workflow: Workflow::Ret { expr },
        }
    }

    /// Start with a literal return
    pub fn ret_val(value: Value) -> Self {
        Self {
            workflow: Workflow::Ret {
                expr: Expr::Literal(value),
            },
        }
    }

    /// Wrap current workflow in a Let binding
    pub fn bind(self, pattern: Pattern, expr: Expr) -> Self {
        Self {
            workflow: Workflow::Let {
                pattern,
                expr,
                continuation: Box::new(self.workflow),
            },
        }
    }

    /// Wrap current workflow in a Let binding with a literal value
    pub fn bind_val(self, name: &str, value: Value) -> Self {
        Self {
            workflow: Workflow::Let {
                pattern: Pattern::Variable(name.into()),
                expr: Expr::Literal(value),
                continuation: Box::new(self.workflow),
            },
        }
    }

    /// Sequence another workflow after the current one
    pub fn then(self, next: Workflow) -> Self {
        Self {
            workflow: Workflow::Seq {
                first: Box::new(self.workflow),
                second: Box::new(next),
            },
        }
    }

    /// Wrap current workflow in a conditional
    pub fn when(self, condition: Expr) -> WorkflowConditionBuilder {
        WorkflowConditionBuilder {
            condition,
            then_branch: self.workflow,
        }
    }

    /// Build the final workflow
    pub fn build(self) -> Workflow {
        self.workflow
    }
}

/// Builder for conditional workflows
pub struct WorkflowConditionBuilder {
    condition: Expr,
    then_branch: Workflow,
}

impl WorkflowConditionBuilder {
    /// Specify the else branch
    pub fn otherwise(self, else_branch: Workflow) -> WorkflowBuilder {
        WorkflowBuilder {
            workflow: Workflow::If {
                condition: self.condition,
                then_branch: Box::new(self.then_branch),
                else_branch: Box::new(else_branch),
            },
        }
    }
}

/// Create a simple capability for testing
pub fn test_capability(name: &str, effect: Effect) -> Capability {
    Capability {
        name: name.into(),
        effect,
        constraints: vec![],
    }
}

/// Create a variable pattern
pub fn var(name: &str) -> Pattern {
    Pattern::Variable(name.into())
}

/// Create a literal expression
pub fn lit(value: Value) -> Expr {
    Expr::Literal(value)
}

/// Create a variable expression
pub fn var_expr(name: &str) -> Expr {
    Expr::Variable(name.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_builder_done() {
        let wf = WorkflowBuilder::done().build();
        assert!(matches!(wf, Workflow::Done));
    }

    #[test]
    fn test_workflow_builder_ret() {
        let wf = WorkflowBuilder::ret_val(Value::Int(42)).build();
        assert!(matches!(wf, Workflow::Ret { .. }));
    }

    #[test]
    fn test_workflow_builder_bind() {
        let wf = WorkflowBuilder::done().bind_val("x", Value::Int(1)).build();
        assert!(matches!(wf, Workflow::Let { .. }));
    }

    #[test]
    fn test_workflow_builder_sequence() {
        let wf = WorkflowBuilder::ret_val(Value::Int(1))
            .then(Workflow::Done)
            .build();
        assert!(matches!(wf, Workflow::Seq { .. }));
    }

    #[test]
    fn test_workflow_builder_conditional() {
        let wf = WorkflowBuilder::done()
            .when(lit(Value::Bool(true)))
            .otherwise(Workflow::Done)
            .build();
        assert!(matches!(wf, Workflow::If { .. }));
    }

    #[test]
    fn test_test_capability() {
        let cap = test_capability("sensor", Effect::Epistemic);
        assert_eq!(cap.name, "sensor");
        assert_eq!(cap.effect, Effect::Epistemic);
        assert!(cap.constraints.is_empty());
    }

    #[test]
    fn test_var_pattern() {
        let pat = var("x");
        assert!(matches!(pat, Pattern::Variable(name) if name == "x"));
    }

    #[test]
    fn test_lit_expr() {
        let expr = lit(Value::Int(42));
        assert!(matches!(expr, Expr::Literal(Value::Int(42))));
    }

    #[test]
    fn test_var_expr() {
        let expr = var_expr("foo");
        assert!(matches!(expr, Expr::Variable(name) if name == "foo"));
    }
}
