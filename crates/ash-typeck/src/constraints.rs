//! Type constraints for Ash (TASK-019)
//!
//! Defines the constraint types and context for collecting constraints
//! during type checking of workflows and expressions.

use crate::types::{Type, TypeVar};
use ash_core::Effect;
use ash_parser::surface::{Expr, Pattern, Workflow};
use std::collections::HashMap;

/// A type constraint generated during constraint collection
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// Two types must be equal
    TypeEqual(Type, Type),
    /// First type must be a subtype of second type
    TypeSubtype(Type, Type),
    /// First effect must be less than or equal to second effect
    EffectLeq(Effect, Effect),
    /// Expression has a specific type
    HasType(Box<Expr>, Type),
    /// Variable binding: pattern has type
    VarBinding(Pattern, Type),
    /// Pattern must match the type
    PatternMatch(Pattern, Type),
    /// Capability requirement
    HasCapability(Box<str>, Effect),
    /// Obligation that must be satisfied
    SatisfiesObligation(Box<str>, Type),
}

impl std::fmt::Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constraint::TypeEqual(t1, t2) => write!(f, "{t1:?} = {t2:?}"),
            Constraint::TypeSubtype(t1, t2) => write!(f, "{t1:?} <: {t2:?}"),
            Constraint::EffectLeq(e1, e2) => write!(f, "{e1} <= {e2}"),
            Constraint::HasType(_, ty) => write!(f, "_ : {ty:?}"),
            Constraint::VarBinding(pat, ty) => write!(f, "{pat:?} : {ty:?}"),
            Constraint::PatternMatch(pat, ty) => write!(f, "{pat:?} matches {ty:?}"),
            Constraint::HasCapability(name, eff) => write!(f, "cap {name} : {eff}"),
            Constraint::SatisfiesObligation(name, ty) => write!(f, "obligation {name} : {ty:?}"),
        }
    }
}

/// Context for collecting constraints during type checking
#[derive(Debug, Clone, Default)]
pub struct ConstraintContext {
    /// Collected constraints
    pub constraints: Vec<Constraint>,
    /// Variable type mappings
    pub var_types: HashMap<Box<str>, Type>,
    /// Next type variable ID
    next_var: u32,
}

impl ConstraintContext {
    /// Create a new empty constraint context
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            var_types: HashMap::new(),
            next_var: 0,
        }
    }

    /// Generate a fresh type variable
    pub fn fresh_var(&mut self) -> TypeVar {
        let var = TypeVar(self.next_var);
        self.next_var += 1;
        var
    }

    /// Add a constraint to the context
    pub fn add(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Add a type equality constraint
    pub fn add_equal(&mut self, t1: Type, t2: Type) {
        self.add(Constraint::TypeEqual(t1, t2));
    }

    /// Add an effect constraint (e1 <= e2)
    pub fn add_effect_leq(&mut self, e1: Effect, e2: Effect) {
        self.add(Constraint::EffectLeq(e1, e2));
    }

    /// Bind a variable name to a type
    pub fn bind_var(&mut self, name: Box<str>, ty: Type) {
        self.var_types.insert(name.clone(), ty.clone());
        self.add(Constraint::VarBinding(Pattern::Variable(name), ty));
    }

    /// Lookup the type of a variable
    pub fn lookup_var(&self, name: &str) -> Option<&Type> {
        self.var_types.get(name)
    }

    /// Get all collected constraints
    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    /// Clear all constraints
    pub fn clear(&mut self) {
        self.constraints.clear();
    }
}

/// Generate constraints for a workflow
pub fn generate_workflow_constraints(ctx: &mut ConstraintContext, workflow: &Workflow) -> Type {
    match workflow {
        Workflow::Observe {
            capability: _,
            binding,
            continuation,
            span: _,
        } => {
            // Observe produces a value of some type (we use a fresh var)
            let result_ty = Type::Var(ctx.fresh_var());

            if let Some(pat) = binding {
                ctx.add(Constraint::PatternMatch(pat.clone(), result_ty.clone()));
            }

            // Effect constraint: observe is epistemic
            ctx.add_effect_leq(Effect::Epistemic, Effect::Epistemic);

            // Continue with rest of workflow
            if let Some(cont) = continuation {
                let cont_ty = generate_workflow_constraints(ctx, cont);
                // Result type is the continuation's type
                ctx.add_equal(result_ty, cont_ty.clone());
                cont_ty
            } else {
                result_ty
            }
        }

        Workflow::Act {
            action: _,
            guard: _,
            span: _,
        } => {
            // Act has operational effect
            ctx.add_effect_leq(Effect::Operational, Effect::Operational);
            // Returns unit/null
            Type::Null
        }

        Workflow::Let {
            pattern,
            expr,
            continuation,
            span: _,
        } => {
            // Generate constraints for the expression
            let expr_ty = generate_expr_constraints(ctx, expr);

            // Pattern must match expression type
            ctx.add(Constraint::PatternMatch(pattern.clone(), expr_ty));

            // Add pattern bindings to context
            let fresh = Type::Var(ctx.fresh_var());
            add_pattern_bindings(ctx, pattern, &fresh);

            // Continue with rest of workflow
            if let Some(cont) = continuation {
                generate_workflow_constraints(ctx, cont)
            } else {
                Type::Null
            }
        }

        Workflow::If {
            condition,
            then_branch,
            else_branch,
            span: _,
        } => {
            // Condition must be boolean
            let cond_ty = generate_expr_constraints(ctx, condition);
            ctx.add_equal(cond_ty, Type::Bool);

            // Both branches must have compatible types
            let then_ty = generate_workflow_constraints(ctx, then_branch);

            if let Some(else_branch) = else_branch {
                let else_ty = generate_workflow_constraints(ctx, else_branch);
                ctx.add_equal(then_ty.clone(), else_ty);
            }

            then_ty
        }

        Workflow::Done { span: _ } => {
            // Done returns null/unit
            Type::Null
        }

        Workflow::Orient {
            expr,
            binding,
            continuation,
            span: _,
        } => {
            let expr_ty = generate_expr_constraints(ctx, expr);

            if let Some(pat) = binding {
                ctx.add(Constraint::PatternMatch(pat.clone(), expr_ty.clone()));
            }

            if let Some(cont) = continuation {
                generate_workflow_constraints(ctx, cont)
            } else {
                expr_ty
            }
        }

        Workflow::Seq { second, .. } => {
            // Sequential composition: result type comes from second
            generate_workflow_constraints(ctx, second)
        }

        // Simplified: other variants not fully implemented
        _ => Type::Var(ctx.fresh_var()),
    }
}

/// Generate constraints for an expression
pub fn generate_expr_constraints(ctx: &mut ConstraintContext, expr: &Expr) -> Type {
    match expr {
        Expr::Literal(lit) => match lit {
            ash_parser::surface::Literal::Int(_) => Type::Int,
            ash_parser::surface::Literal::Float(_) => Type::Var(ctx.fresh_var()), // Float not in core types
            ash_parser::surface::Literal::String(_) => Type::String,
            ash_parser::surface::Literal::Bool(_) => Type::Bool,
            ash_parser::surface::Literal::Null => Type::Null,
        },

        Expr::Variable(name) => {
            // Look up variable type or create fresh variable
            if let Some(ty) = ctx.lookup_var(name) {
                ty.clone()
            } else {
                let fresh = Type::Var(ctx.fresh_var());
                ctx.bind_var(name.clone(), fresh.clone());
                fresh
            }
        }

        Expr::Binary {
            op, left, right, ..
        } => {
            let left_ty = generate_expr_constraints(ctx, left);
            let right_ty = generate_expr_constraints(ctx, right);

            match op {
                ash_parser::surface::BinaryOp::Add
                | ash_parser::surface::BinaryOp::Sub
                | ash_parser::surface::BinaryOp::Mul
                | ash_parser::surface::BinaryOp::Div => {
                    // Arithmetic: both operands must be numeric (Int for simplicity)
                    ctx.add_equal(left_ty.clone(), Type::Int);
                    ctx.add_equal(right_ty, Type::Int);
                    Type::Int
                }
                ash_parser::surface::BinaryOp::Eq
                | ash_parser::surface::BinaryOp::Neq
                | ash_parser::surface::BinaryOp::Lt
                | ash_parser::surface::BinaryOp::Gt
                | ash_parser::surface::BinaryOp::Leq
                | ash_parser::surface::BinaryOp::Geq => {
                    // Comparison: operands must be equal, result is Bool
                    ctx.add_equal(left_ty, right_ty);
                    Type::Bool
                }
                ash_parser::surface::BinaryOp::And | ash_parser::surface::BinaryOp::Or => {
                    // Logical: both must be Bool
                    ctx.add_equal(left_ty, Type::Bool);
                    ctx.add_equal(right_ty, Type::Bool);
                    Type::Bool
                }
                _ => Type::Var(ctx.fresh_var()),
            }
        }

        Expr::Unary { op, operand, .. } => {
            let operand_ty = generate_expr_constraints(ctx, operand);

            match op {
                ash_parser::surface::UnaryOp::Not => {
                    ctx.add_equal(operand_ty, Type::Bool);
                    Type::Bool
                }
                ash_parser::surface::UnaryOp::Neg => {
                    ctx.add_equal(operand_ty, Type::Int);
                    Type::Int
                }
            }
        }

        Expr::FieldAccess { base, field: _, .. } => {
            let _base_ty = generate_expr_constraints(ctx, base);
            // Field access produces a fresh type variable
            Type::Var(ctx.fresh_var())
        }

        Expr::Call { func: _, args, .. } => {
            // Generate constraints for arguments
            for arg in args {
                let _ = generate_expr_constraints(ctx, arg);
            }
            // Function call produces fresh type
            Type::Var(ctx.fresh_var())
        }

        _ => Type::Var(ctx.fresh_var()),
    }
}

/// Add pattern bindings to the constraint context
fn add_pattern_bindings(ctx: &mut ConstraintContext, pattern: &Pattern, ty: &Type) {
    match pattern {
        Pattern::Variable(name) => {
            ctx.bind_var(name.clone(), ty.clone());
        }
        Pattern::Tuple(patterns) => {
            // For tuple patterns, create fresh types for each element
            for pat in patterns {
                let elem_ty = Type::Var(ctx.fresh_var());
                add_pattern_bindings(ctx, pat, &elem_ty);
            }
        }
        Pattern::Record(fields) => {
            // For record patterns, bind each field
            for (name, pat) in fields {
                let field_ty = Type::Var(ctx.fresh_var());
                ctx.bind_var(name.clone(), field_ty.clone());
                add_pattern_bindings(ctx, pat, &field_ty);
            }
        }
        Pattern::List { elements, rest } => {
            for pat in elements {
                let elem_ty = Type::Var(ctx.fresh_var());
                add_pattern_bindings(ctx, pat, &elem_ty);
            }
            if let Some(rest_name) = rest {
                ctx.bind_var(rest_name.clone(), ty.clone());
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::surface::Literal;
    use ash_parser::token::Span;

    fn test_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    #[test]
    fn test_constraint_context_creation() {
        let ctx = ConstraintContext::new();
        assert!(ctx.constraints().is_empty());
        assert!(ctx.var_types.is_empty());
    }

    #[test]
    fn test_fresh_var_generation() {
        let mut ctx = ConstraintContext::new();
        let v1 = ctx.fresh_var();
        let v2 = ctx.fresh_var();
        let v3 = ctx.fresh_var();

        assert_eq!(v1.0, 0);
        assert_eq!(v2.0, 1);
        assert_eq!(v3.0, 2);
    }

    #[test]
    fn test_add_constraint() {
        let mut ctx = ConstraintContext::new();
        ctx.add(Constraint::TypeEqual(Type::Int, Type::Int));

        assert_eq!(ctx.constraints().len(), 1);
    }

    #[test]
    fn test_add_equal() {
        let mut ctx = ConstraintContext::new();
        ctx.add_equal(Type::Int, Type::Int);

        assert_eq!(ctx.constraints().len(), 1);
        assert!(matches!(&ctx.constraints()[0], Constraint::TypeEqual(_, _)));
    }

    #[test]
    fn test_bind_var() {
        let mut ctx = ConstraintContext::new();
        ctx.bind_var("x".into(), Type::Int);

        assert_eq!(ctx.var_types.len(), 1);
        assert_eq!(ctx.lookup_var("x"), Some(&Type::Int));
    }

    #[test]
    fn test_lookup_var_not_found() {
        let ctx = ConstraintContext::new();
        assert_eq!(ctx.lookup_var("nonexistent"), None);
    }

    #[test]
    fn test_generate_expr_literal_int() {
        let mut ctx = ConstraintContext::new();
        let expr = Expr::Literal(Literal::Int(42));
        let ty = generate_expr_constraints(&mut ctx, &expr);

        assert_eq!(ty, Type::Int);
        assert!(ctx.constraints().is_empty());
    }

    #[test]
    fn test_generate_expr_literal_string() {
        let mut ctx = ConstraintContext::new();
        let expr = Expr::Literal(Literal::String("hello".into()));
        let ty = generate_expr_constraints(&mut ctx, &expr);

        assert_eq!(ty, Type::String);
    }

    #[test]
    fn test_generate_expr_literal_bool() {
        let mut ctx = ConstraintContext::new();
        let expr = Expr::Literal(Literal::Bool(true));
        let ty = generate_expr_constraints(&mut ctx, &expr);

        assert_eq!(ty, Type::Bool);
    }

    #[test]
    fn test_generate_expr_variable_new() {
        let mut ctx = ConstraintContext::new();
        let expr = Expr::Variable("x".into());
        let ty = generate_expr_constraints(&mut ctx, &expr);

        // Should create a fresh type variable
        assert!(matches!(ty, Type::Var(_)));
        assert_eq!(ctx.var_types.len(), 1);
    }

    #[test]
    fn test_generate_expr_variable_existing() {
        let mut ctx = ConstraintContext::new();
        ctx.bind_var("x".into(), Type::Int);

        let expr = Expr::Variable("x".into());
        let ty = generate_expr_constraints(&mut ctx, &expr);

        assert_eq!(ty, Type::Int);
    }

    #[test]
    fn test_generate_expr_binary_arithmetic() {
        let mut ctx = ConstraintContext::new();
        let expr = Expr::Binary {
            op: ash_parser::surface::BinaryOp::Add,
            left: Box::new(Expr::Literal(Literal::Int(1))),
            right: Box::new(Expr::Literal(Literal::Int(2))),
            span: test_span(),
        };

        let ty = generate_expr_constraints(&mut ctx, &expr);
        assert_eq!(ty, Type::Int);
        assert_eq!(ctx.constraints().len(), 2); // Both operands constrained to Int
    }

    #[test]
    fn test_generate_expr_binary_comparison() {
        let mut ctx = ConstraintContext::new();
        let expr = Expr::Binary {
            op: ash_parser::surface::BinaryOp::Eq,
            left: Box::new(Expr::Literal(Literal::Int(1))),
            right: Box::new(Expr::Literal(Literal::Int(2))),
            span: test_span(),
        };

        let ty = generate_expr_constraints(&mut ctx, &expr);
        assert_eq!(ty, Type::Bool);
    }

    #[test]
    fn test_generate_expr_binary_logical() {
        let mut ctx = ConstraintContext::new();
        let expr = Expr::Binary {
            op: ash_parser::surface::BinaryOp::And,
            left: Box::new(Expr::Literal(Literal::Bool(true))),
            right: Box::new(Expr::Literal(Literal::Bool(false))),
            span: test_span(),
        };

        let ty = generate_expr_constraints(&mut ctx, &expr);
        assert_eq!(ty, Type::Bool);
        assert_eq!(ctx.constraints().len(), 2); // Both operands constrained to Bool
    }

    #[test]
    fn test_generate_expr_unary_not() {
        let mut ctx = ConstraintContext::new();
        let expr = Expr::Unary {
            op: ash_parser::surface::UnaryOp::Not,
            operand: Box::new(Expr::Literal(Literal::Bool(true))),
            span: test_span(),
        };

        let ty = generate_expr_constraints(&mut ctx, &expr);
        assert_eq!(ty, Type::Bool);
    }

    #[test]
    fn test_generate_expr_unary_neg() {
        let mut ctx = ConstraintContext::new();
        let expr = Expr::Unary {
            op: ash_parser::surface::UnaryOp::Neg,
            operand: Box::new(Expr::Literal(Literal::Int(42))),
            span: test_span(),
        };

        let ty = generate_expr_constraints(&mut ctx, &expr);
        assert_eq!(ty, Type::Int);
    }

    #[test]
    fn test_generate_workflow_done() {
        let mut ctx = ConstraintContext::new();
        let workflow = Workflow::Done { span: test_span() };
        let ty = generate_workflow_constraints(&mut ctx, &workflow);

        assert_eq!(ty, Type::Null);
    }

    #[test]
    fn test_generate_workflow_observe() {
        let mut ctx = ConstraintContext::new();
        let workflow = Workflow::Observe {
            capability: "read".into(),
            binding: Some(Pattern::Variable("x".into())),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let ty = generate_workflow_constraints(&mut ctx, &workflow);
        assert_eq!(ty, Type::Null);

        // Should have effect constraint and pattern match constraint
        assert!(!ctx.constraints().is_empty());
    }

    #[test]
    fn test_generate_workflow_act() {
        let mut ctx = ConstraintContext::new();
        let workflow = Workflow::Act {
            action: ash_parser::surface::ActionRef {
                name: "write".into(),
                args: vec![],
            },
            guard: None,
            span: test_span(),
        };

        let ty = generate_workflow_constraints(&mut ctx, &workflow);
        assert_eq!(ty, Type::Null);

        // Should have operational effect constraint
        let has_effect_constraint = ctx.constraints().iter().any(|c| {
            matches!(
                c,
                Constraint::EffectLeq(Effect::Operational, Effect::Operational)
            )
        });
        assert!(has_effect_constraint);
    }

    #[test]
    fn test_generate_workflow_let() {
        let mut ctx = ConstraintContext::new();
        let workflow = Workflow::Let {
            pattern: Pattern::Variable("x".into()),
            expr: Expr::Literal(Literal::Int(42)),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let ty = generate_workflow_constraints(&mut ctx, &workflow);
        assert_eq!(ty, Type::Null);
        assert!(!ctx.constraints().is_empty());
    }

    #[test]
    fn test_generate_workflow_if() {
        let mut ctx = ConstraintContext::new();
        let workflow = Workflow::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_branch: Box::new(Workflow::Done { span: test_span() }),
            else_branch: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let ty = generate_workflow_constraints(&mut ctx, &workflow);
        assert_eq!(ty, Type::Null);

        // Should have condition constraint (bool) and branch type equality
        assert!(!ctx.constraints().is_empty());
    }

    #[test]
    fn test_constraint_display() {
        let c1 = Constraint::TypeEqual(Type::Int, Type::Bool);
        assert!(format!("{c1}").contains("Int"));
        assert!(format!("{c1}").contains("Bool"));

        let c2 = Constraint::EffectLeq(Effect::Epistemic, Effect::Operational);
        assert_eq!(format!("{c2}"), "epistemic <= operational");
    }
}
