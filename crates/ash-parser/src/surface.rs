//! Surface AST types for the Ash parser.
//!
//! This module defines the parsed surface syntax before lowering to Core IR.
//! Surface AST is more flexible and syntactic than Core IR, preserving
//! all source-level constructs with full span information.

use crate::token::Span;

/// A name/identifier in the source code.
pub type Name = Box<str>;

/// A program consists of definitions and a main workflow.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// Top-level definitions (capabilities, policies, roles)
    pub definitions: Vec<Definition>,
    /// The main workflow definition
    pub workflow: WorkflowDef,
}

/// A top-level definition.
#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    /// Capability definition
    Capability(CapabilityDef),
    /// Policy definition
    Policy(PolicyDef),
    /// Role definition
    Role(RoleDef),
}

/// A capability definition.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityDef {
    /// Name of the capability
    pub name: Name,
    /// Effect type of the capability
    pub effect: EffectType,
    /// Parameters to the capability
    pub params: Vec<Param>,
    /// Return type (optional)
    pub return_type: Option<Type>,
    /// Constraints on the capability
    pub constraints: Vec<Constraint>,
    /// Source span
    pub span: Span,
}

/// A policy definition.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyDef {
    /// Name of the policy
    pub name: Name,
    /// Condition expression
    pub condition: Expr,
    /// Decision made when condition is met
    pub decision: Decision,
    /// Source span
    pub span: Span,
}

/// A role definition.
#[derive(Debug, Clone, PartialEq)]
pub struct RoleDef {
    /// Name of the role
    pub name: Name,
    /// Authorities granted to this role
    pub authority: Vec<Name>,
    /// Obligations of this role
    pub obligations: Vec<ObligationRef>,
    /// Roles supervised by this role
    pub supervises: Vec<Name>,
    /// Source span
    pub span: Span,
}

/// A workflow definition.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowDef {
    /// Name of the workflow
    pub name: Name,
    /// The workflow body
    pub body: Workflow,
    /// Source span
    pub span: Span,
}

/// Surface workflow syntax - more flexible than core IR.
#[derive(Debug, Clone, PartialEq)]
pub enum Workflow {
    /// Observe phase: invoke a capability to observe
    Observe {
        /// Capability to invoke
        capability: Name,
        /// Optional binding for result
        binding: Option<Pattern>,
        /// Optional continuation
        continuation: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// Orient phase: evaluate an expression
    Orient {
        /// Expression to evaluate
        expr: Expr,
        /// Optional binding for result
        binding: Option<Pattern>,
        /// Optional continuation
        continuation: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// Propose phase: propose an action
    Propose {
        /// Action to propose
        action: ActionRef,
        /// Optional binding for result
        binding: Option<Pattern>,
        /// Optional continuation
        continuation: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// Decide phase: apply a policy decision
    Decide {
        /// Condition expression
        expr: Expr,
        /// Optional policy name
        policy: Option<Name>,
        /// Then branch
        then_branch: Box<Workflow>,
        /// Optional else branch
        else_branch: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// Check phase: verify an obligation
    Check {
        /// Obligation to check
        obligation: ObligationRef,
        /// Optional continuation
        continuation: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// Act phase: execute an action
    Act {
        /// Action to execute
        action: ActionRef,
        /// Optional guard
        guard: Option<Guard>,
        /// Source span
        span: Span,
    },
    /// Let binding: bind a pattern to an expression
    Let {
        /// Pattern to bind
        pattern: Pattern,
        /// Expression to evaluate
        expr: Expr,
        /// Optional continuation
        continuation: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// Conditional workflow
    If {
        /// Condition expression
        condition: Expr,
        /// Then branch
        then_branch: Box<Workflow>,
        /// Optional else branch
        else_branch: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// For loop: iterate over a collection
    For {
        /// Pattern for each element
        pattern: Pattern,
        /// Collection to iterate over
        collection: Expr,
        /// Body of the loop
        body: Box<Workflow>,
        /// Source span
        span: Span,
    },
    /// Parallel composition: execute branches in parallel
    Par {
        /// Branches to execute in parallel
        branches: Vec<Workflow>,
        /// Source span
        span: Span,
    },
    /// With clause: scoped capability
    With {
        /// Capability to use
        capability: Name,
        /// Body to execute with the capability
        body: Box<Workflow>,
        /// Source span
        span: Span,
    },
    /// Maybe: try primary, fallback on failure
    Maybe {
        /// Primary workflow
        primary: Box<Workflow>,
        /// Fallback workflow
        fallback: Box<Workflow>,
        /// Source span
        span: Span,
    },
    /// Must: ensure workflow succeeds
    Must {
        /// Body that must succeed
        body: Box<Workflow>,
        /// Source span
        span: Span,
    },
    /// Sequential composition
    Seq {
        /// First workflow
        first: Box<Workflow>,
        /// Second workflow
        second: Box<Workflow>,
        /// Source span
        span: Span,
    },
    /// Done: successful completion
    Done {
        /// Source span
        span: Span,
    },
}

/// Expression types.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Literal value
    Literal(Literal),
    /// Variable reference
    Variable(Name),
    /// Field access: base.field
    FieldAccess {
        /// Base expression
        base: Box<Expr>,
        /// Field name
        field: Name,
        /// Source span
        span: Span,
    },
    /// Index access: base[index]
    IndexAccess {
        /// Base expression
        base: Box<Expr>,
        /// Index expression
        index: Box<Expr>,
        /// Source span
        span: Span,
    },
    /// Unary operation
    Unary {
        /// Unary operator
        op: UnaryOp,
        /// Operand
        operand: Box<Expr>,
        /// Source span
        span: Span,
    },
    /// Binary operation
    Binary {
        /// Binary operator
        op: BinaryOp,
        /// Left operand
        left: Box<Expr>,
        /// Right operand
        right: Box<Expr>,
        /// Source span
        span: Span,
    },
    /// Function call
    Call {
        /// Function name
        func: Name,
        /// Arguments
        args: Vec<Expr>,
        /// Source span
        span: Span,
    },
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    /// Logical negation: !
    Not,
    /// Arithmetic negation: -
    Neg,
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    /// Addition: +
    Add,
    /// Subtraction: -
    Sub,
    /// Multiplication: *
    Mul,
    /// Division: /
    Div,
    /// Logical AND: &&
    And,
    /// Logical OR: ||
    Or,
    /// Equality: ==
    Eq,
    /// Inequality: !=
    Neq,
    /// Less than: <
    Lt,
    /// Greater than: >
    Gt,
    /// Less than or equal: <=
    Leq,
    /// Greater than or equal: >=
    Geq,
    /// Membership test: in
    In,
}

/// Pattern types for destructuring.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Variable binding
    Variable(Name),
    /// Wildcard pattern: _
    Wildcard,
    /// Tuple pattern: (a, b, c)
    Tuple(Vec<Pattern>),
    /// Record pattern: { field: pat, ... }
    Record(Vec<(Name, Pattern)>),
    /// List pattern: [a, b, ..rest]
    List {
        /// Element patterns
        elements: Vec<Pattern>,
        /// Optional rest binding
        rest: Option<Name>,
    },
    /// Literal pattern
    Literal(Literal),
}

/// Literal values.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Integer literal
    Int(i64),
    /// Floating-point literal
    Float(f64),
    /// String literal
    String(Box<str>),
    /// Boolean literal
    Bool(bool),
    /// Null literal
    Null,
}

/// Effect type levels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectType {
    /// Read-only observation
    Observe,
    /// Reading data
    Read,
    /// Analyzing data
    Analyze,
    /// Making decisions
    Decide,
    /// Taking actions
    Act,
    /// Writing/modifying data
    Write,
    /// External effects
    External,
}

/// Policy decisions.
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    /// Permit the action
    Permit,
    /// Deny the action
    Deny,
    /// Require approval from a role
    RequireApproval {
        /// Role required for approval
        role: Name,
    },
    /// Escalate to supervisor
    Escalate,
}

/// Reference to an action invocation.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionRef {
    /// Name of the action
    pub name: Name,
    /// Arguments to the action
    pub args: Vec<Expr>,
}

/// Reference to an obligation.
#[derive(Debug, Clone, PartialEq)]
pub struct ObligationRef {
    /// Role with the obligation
    pub role: Name,
    /// Condition that must hold
    pub condition: Expr,
}

/// Parameter with name and type.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    /// Parameter name
    pub name: Name,
    /// Parameter type
    pub ty: Type,
}

/// Type annotations.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Named type
    Name(Name),
    /// List type: [T]
    List(Box<Type>),
    /// Record type: { field: T, ... }
    Record(Vec<(Name, Type)>),
    /// Capability type
    Capability(Name),
}

/// Guard expressions for actions.
#[derive(Debug, Clone, PartialEq)]
pub enum Guard {
    /// Always allow
    Always,
    /// Never allow
    Never,
    /// Predicate guard
    Pred(Predicate),
    /// Conjunction: left AND right
    And(Box<Guard>, Box<Guard>),
    /// Disjunction: left OR right
    Or(Box<Guard>, Box<Guard>),
    /// Negation: NOT guard
    Not(Box<Guard>),
}

/// A predicate expression.
#[derive(Debug, Clone, PartialEq)]
pub struct Predicate {
    /// Predicate name
    pub name: Name,
    /// Predicate arguments
    pub args: Vec<Expr>,
}

/// A constraint on a capability.
#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    /// The constraint predicate
    pub predicate: Predicate,
}

/// Trait for types that have a source span.
pub trait Spanned {
    /// Returns the source span of this node.
    fn span(&self) -> Span;
}

impl Spanned for Workflow {
    fn span(&self) -> Span {
        match self {
            Workflow::Observe { span, .. } => *span,
            Workflow::Orient { span, .. } => *span,
            Workflow::Propose { span, .. } => *span,
            Workflow::Decide { span, .. } => *span,
            Workflow::Check { span, .. } => *span,
            Workflow::Act { span, .. } => *span,
            Workflow::Let { span, .. } => *span,
            Workflow::If { span, .. } => *span,
            Workflow::For { span, .. } => *span,
            Workflow::Par { span, .. } => *span,
            Workflow::With { span, .. } => *span,
            Workflow::Maybe { span, .. } => *span,
            Workflow::Must { span, .. } => *span,
            Workflow::Seq { span, .. } => *span,
            Workflow::Done { span, .. } => *span,
        }
    }
}

impl Spanned for Expr {
    fn span(&self) -> Span {
        match self {
            Expr::Literal(_) => Span::default(),
            Expr::Variable(_) => Span::default(),
            Expr::FieldAccess { span, .. } => *span,
            Expr::IndexAccess { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Call { span, .. } => *span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Construction Tests
    // =========================================================================

    #[test]
    fn test_program_construction() {
        let program = Program {
            definitions: vec![],
            workflow: WorkflowDef {
                name: "main".into(),
                body: Workflow::Done {
                    span: Span::new(0, 4, 1, 1),
                },
                span: Span::new(0, 10, 1, 1),
            },
        };

        assert!(program.definitions.is_empty());
        assert_eq!(program.workflow.name, "main".into());
    }

    #[test]
    fn test_definition_variants() {
        let cap_def = CapabilityDef {
            name: "read_file".into(),
            effect: EffectType::Read,
            params: vec![],
            return_type: None,
            constraints: vec![],
            span: Span::new(0, 20, 1, 1),
        };
        let _def = Definition::Capability(cap_def);

        let policy_def = PolicyDef {
            name: "allow_admin".into(),
            condition: Expr::Literal(Literal::Bool(true)),
            decision: Decision::Permit,
            span: Span::new(0, 15, 1, 1),
        };
        let _def = Definition::Policy(policy_def);

        let role_def = RoleDef {
            name: "admin".into(),
            authority: vec!["read".into(), "write".into()],
            obligations: vec![],
            supervises: vec![],
            span: Span::new(0, 10, 1, 1),
        };
        let _def = Definition::Role(role_def);
    }

    #[test]
    fn test_capability_def_construction() {
        let cap = CapabilityDef {
            name: "write_file".into(),
            effect: EffectType::Write,
            params: vec![
                Param {
                    name: "path".into(),
                    ty: Type::Name("String".into()),
                },
                Param {
                    name: "content".into(),
                    ty: Type::Name("String".into()),
                },
            ],
            return_type: Some(Type::Name("Bool".into())),
            constraints: vec![],
            span: Span::new(0, 50, 1, 1),
        };

        assert_eq!(cap.name, "write_file".into());
        assert_eq!(cap.effect, EffectType::Write);
        assert_eq!(cap.params.len(), 2);
        assert!(cap.return_type.is_some());
    }

    #[test]
    fn test_policy_def_construction() {
        let policy = PolicyDef {
            name: "check_age".into(),
            condition: Expr::Binary {
                op: BinaryOp::Geq,
                left: Box::new(Expr::Variable("age".into())),
                right: Box::new(Expr::Literal(Literal::Int(18))),
                span: Span::new(0, 10, 1, 1),
            },
            decision: Decision::Permit,
            span: Span::new(0, 30, 1, 1),
        };

        assert_eq!(policy.name, "check_age".into());
        assert_eq!(policy.decision, Decision::Permit);
    }

    #[test]
    fn test_role_def_construction() {
        let role = RoleDef {
            name: "manager".into(),
            authority: vec!["approve".into(), "review".into()],
            obligations: vec![ObligationRef {
                role: "manager".into(),
                condition: Expr::Literal(Literal::Bool(true)),
            }],
            supervises: vec!["employee".into()],
            span: Span::new(0, 100, 1, 1),
        };

        assert_eq!(role.name, "manager".into());
        assert_eq!(role.authority.len(), 2);
        assert_eq!(role.obligations.len(), 1);
        assert_eq!(role.supervises.len(), 1);
    }

    #[test]
    fn test_workflow_def_construction() {
        let workflow_def = WorkflowDef {
            name: "process_order".into(),
            body: Workflow::Done {
                span: Span::new(0, 4, 1, 1),
            },
            span: Span::new(0, 20, 1, 1),
        };

        assert_eq!(workflow_def.name, "process_order".into());
    }

    // =========================================================================
    // Workflow Variant Tests
    // =========================================================================

    #[test]
    fn test_workflow_observe() {
        let wf = Workflow::Observe {
            capability: "read_db".into(),
            binding: Some(Pattern::Variable("data".into())),
            continuation: None,
            span: Span::new(0, 20, 1, 1),
        };

        match wf {
            Workflow::Observe { capability, .. } => {
                assert_eq!(capability, "read_db".into());
            }
            _ => panic!("Expected Observe workflow"),
        }
    }

    #[test]
    fn test_workflow_orient() {
        let wf = Workflow::Orient {
            expr: Expr::Variable("data".into()),
            binding: Some(Pattern::Variable("result".into())),
            continuation: None,
            span: Span::new(0, 15, 1, 1),
        };

        match wf {
            Workflow::Orient { expr, .. } => {
                assert!(matches!(expr, Expr::Variable(_)));
            }
            _ => panic!("Expected Orient workflow"),
        }
    }

    #[test]
    fn test_workflow_propose() {
        let wf = Workflow::Propose {
            action: ActionRef {
                name: "send_email".into(),
                args: vec![],
            },
            binding: None,
            continuation: None,
            span: Span::new(0, 15, 1, 1),
        };

        match wf {
            Workflow::Propose { action, .. } => {
                assert_eq!(action.name, "send_email".into());
            }
            _ => panic!("Expected Propose workflow"),
        }
    }

    #[test]
    fn test_workflow_decide() {
        let wf = Workflow::Decide {
            expr: Expr::Literal(Literal::Bool(true)),
            policy: Some("policy1".into()),
            then_branch: Box::new(Workflow::Done {
                span: Span::new(0, 4, 1, 1),
            }),
            else_branch: Some(Box::new(Workflow::Done {
                span: Span::new(5, 9, 1, 1),
            })),
            span: Span::new(0, 30, 1, 1),
        };

        match wf {
            Workflow::Decide {
                policy,
                else_branch,
                ..
            } => {
                assert_eq!(policy, Some("policy1".into()));
                assert!(else_branch.is_some());
            }
            _ => panic!("Expected Decide workflow"),
        }
    }

    #[test]
    fn test_workflow_check() {
        let wf = Workflow::Check {
            obligation: ObligationRef {
                role: "admin".into(),
                condition: Expr::Literal(Literal::Bool(true)),
            },
            continuation: Some(Box::new(Workflow::Done {
                span: Span::new(0, 4, 1, 1),
            })),
            span: Span::new(0, 20, 1, 1),
        };

        match wf {
            Workflow::Check {
                obligation,
                continuation,
                ..
            } => {
                assert_eq!(obligation.role, "admin".into());
                assert!(continuation.is_some());
            }
            _ => panic!("Expected Check workflow"),
        }
    }

    #[test]
    fn test_workflow_act() {
        let wf = Workflow::Act {
            action: ActionRef {
                name: "log_action".into(),
                args: vec![Expr::Literal(Literal::String("test".into()))],
            },
            guard: Some(Guard::Always),
            span: Span::new(0, 20, 1, 1),
        };

        match wf {
            Workflow::Act { action, guard, .. } => {
                assert_eq!(action.name, "log_action".into());
                assert!(matches!(guard, Some(Guard::Always)));
            }
            _ => panic!("Expected Act workflow"),
        }
    }

    #[test]
    fn test_workflow_let() {
        let wf = Workflow::Let {
            pattern: Pattern::Variable("x".into()),
            expr: Expr::Literal(Literal::Int(42)),
            continuation: Some(Box::new(Workflow::Done {
                span: Span::new(0, 4, 1, 1),
            })),
            span: Span::new(0, 15, 1, 1),
        };

        match wf {
            Workflow::Let { pattern, .. } => {
                assert!(matches!(pattern, Pattern::Variable(_)));
            }
            _ => panic!("Expected Let workflow"),
        }
    }

    #[test]
    fn test_workflow_if() {
        let wf = Workflow::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_branch: Box::new(Workflow::Done {
                span: Span::new(0, 4, 1, 1),
            }),
            else_branch: Some(Box::new(Workflow::Done {
                span: Span::new(5, 9, 1, 1),
            })),
            span: Span::new(0, 25, 1, 1),
        };

        match wf {
            Workflow::If { condition, .. } => {
                assert!(matches!(condition, Expr::Literal(Literal::Bool(true))));
            }
            _ => panic!("Expected If workflow"),
        }
    }

    #[test]
    fn test_workflow_for() {
        let wf = Workflow::For {
            pattern: Pattern::Variable("item".into()),
            collection: Expr::Variable("items".into()),
            body: Box::new(Workflow::Done {
                span: Span::new(0, 4, 1, 1),
            }),
            span: Span::new(0, 30, 1, 1),
        };

        match wf {
            Workflow::For {
                pattern,
                collection,
                ..
            } => {
                assert!(matches!(pattern, Pattern::Variable(_)));
                assert!(matches!(collection, Expr::Variable(_)));
            }
            _ => panic!("Expected For workflow"),
        }
    }

    #[test]
    fn test_workflow_par() {
        let wf = Workflow::Par {
            branches: vec![
                Workflow::Done {
                    span: Span::new(0, 4, 1, 1),
                },
                Workflow::Done {
                    span: Span::new(5, 9, 1, 1),
                },
            ],
            span: Span::new(0, 10, 1, 1),
        };

        match wf {
            Workflow::Par { branches, .. } => {
                assert_eq!(branches.len(), 2);
            }
            _ => panic!("Expected Par workflow"),
        }
    }

    #[test]
    fn test_workflow_with() {
        let wf = Workflow::With {
            capability: "db_connection".into(),
            body: Box::new(Workflow::Done {
                span: Span::new(0, 4, 1, 1),
            }),
            span: Span::new(0, 25, 1, 1),
        };

        match wf {
            Workflow::With { capability, .. } => {
                assert_eq!(capability, "db_connection".into());
            }
            _ => panic!("Expected With workflow"),
        }
    }

    #[test]
    fn test_workflow_maybe() {
        let wf = Workflow::Maybe {
            primary: Box::new(Workflow::Done {
                span: Span::new(0, 4, 1, 1),
            }),
            fallback: Box::new(Workflow::Done {
                span: Span::new(5, 9, 1, 1),
            }),
            span: Span::new(0, 15, 1, 1),
        };

        match wf {
            Workflow::Maybe {
                primary, fallback, ..
            } => {
                assert!(matches!(primary.as_ref(), Workflow::Done { .. }));
                assert!(matches!(fallback.as_ref(), Workflow::Done { .. }));
            }
            _ => panic!("Expected Maybe workflow"),
        }
    }

    #[test]
    fn test_workflow_must() {
        let wf = Workflow::Must {
            body: Box::new(Workflow::Done {
                span: Span::new(0, 4, 1, 1),
            }),
            span: Span::new(0, 10, 1, 1),
        };

        match wf {
            Workflow::Must { body, .. } => {
                assert!(matches!(body.as_ref(), Workflow::Done { .. }));
            }
            _ => panic!("Expected Must workflow"),
        }
    }

    #[test]
    fn test_workflow_seq() {
        let wf = Workflow::Seq {
            first: Box::new(Workflow::Done {
                span: Span::new(0, 4, 1, 1),
            }),
            second: Box::new(Workflow::Done {
                span: Span::new(5, 9, 1, 1),
            }),
            span: Span::new(0, 10, 1, 1),
        };

        match wf {
            Workflow::Seq { first, second, .. } => {
                assert!(matches!(first.as_ref(), Workflow::Done { .. }));
                assert!(matches!(second.as_ref(), Workflow::Done { .. }));
            }
            _ => panic!("Expected Seq workflow"),
        }
    }

    #[test]
    fn test_workflow_done() {
        let wf = Workflow::Done {
            span: Span::new(0, 4, 1, 1),
        };

        assert!(matches!(wf, Workflow::Done { .. }));
    }

    // =========================================================================
    // Expression Tests
    // =========================================================================

    #[test]
    fn test_expr_literal() {
        let expr = Expr::Literal(Literal::Int(42));
        assert!(matches!(expr, Expr::Literal(Literal::Int(42))));
    }

    #[test]
    fn test_expr_variable() {
        let expr = Expr::Variable("my_var".into());
        assert!(matches!(expr, Expr::Variable(_)));
        if let Expr::Variable(name) = expr {
            assert_eq!(name, "my_var".into());
        }
    }

    #[test]
    fn test_expr_field_access() {
        let expr = Expr::FieldAccess {
            base: Box::new(Expr::Variable("obj".into())),
            field: "field".into(),
            span: Span::new(0, 10, 1, 1),
        };

        match expr {
            Expr::FieldAccess { base, field, .. } => {
                assert!(matches!(base.as_ref(), Expr::Variable(_)));
                assert_eq!(field, "field".into());
            }
            _ => panic!("Expected FieldAccess"),
        }
    }

    #[test]
    fn test_expr_index_access() {
        let expr = Expr::IndexAccess {
            base: Box::new(Expr::Variable("arr".into())),
            index: Box::new(Expr::Literal(Literal::Int(0))),
            span: Span::new(0, 8, 1, 1),
        };

        match expr {
            Expr::IndexAccess { base, index, .. } => {
                assert!(matches!(base.as_ref(), Expr::Variable(_)));
                assert!(matches!(index.as_ref(), Expr::Literal(Literal::Int(0))));
            }
            _ => panic!("Expected IndexAccess"),
        }
    }

    #[test]
    fn test_expr_unary() {
        let expr = Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expr::Literal(Literal::Bool(false))),
            span: Span::new(0, 5, 1, 1),
        };

        match expr {
            Expr::Unary { op, .. } => {
                assert_eq!(op, UnaryOp::Not);
            }
            _ => panic!("Expected Unary"),
        }
    }

    #[test]
    fn test_expr_binary() {
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Literal(Literal::Int(1))),
            right: Box::new(Expr::Literal(Literal::Int(2))),
            span: Span::new(0, 5, 1, 1),
        };

        match expr {
            Expr::Binary {
                op, left, right, ..
            } => {
                assert_eq!(op, BinaryOp::Add);
                assert!(matches!(left.as_ref(), Expr::Literal(Literal::Int(1))));
                assert!(matches!(right.as_ref(), Expr::Literal(Literal::Int(2))));
            }
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_expr_call() {
        let expr = Expr::Call {
            func: "foo".into(),
            args: vec![
                Expr::Literal(Literal::Int(1)),
                Expr::Literal(Literal::Int(2)),
            ],
            span: Span::new(0, 10, 1, 1),
        };

        match expr {
            Expr::Call { func, args, .. } => {
                assert_eq!(func, "foo".into());
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Call"),
        }
    }

    // =========================================================================
    // Operator Tests
    // =========================================================================

    #[test]
    fn test_unary_ops() {
        assert_eq!(UnaryOp::Not, UnaryOp::Not);
        assert_eq!(UnaryOp::Neg, UnaryOp::Neg);
        assert_ne!(UnaryOp::Not, UnaryOp::Neg);
    }

    #[test]
    fn test_binary_ops() {
        let ops = vec![
            BinaryOp::Add,
            BinaryOp::Sub,
            BinaryOp::Mul,
            BinaryOp::Div,
            BinaryOp::And,
            BinaryOp::Or,
            BinaryOp::Eq,
            BinaryOp::Neq,
            BinaryOp::Lt,
            BinaryOp::Gt,
            BinaryOp::Leq,
            BinaryOp::Geq,
            BinaryOp::In,
        ];

        // Ensure all ops are distinct
        for (i, op1) in ops.iter().enumerate() {
            for (j, op2) in ops.iter().enumerate() {
                if i != j {
                    assert_ne!(op1, op2);
                }
            }
        }
    }

    // =========================================================================
    // Pattern Tests
    // =========================================================================

    #[test]
    fn test_pattern_variable() {
        let pat = Pattern::Variable("x".into());
        assert!(matches!(pat, Pattern::Variable(_)));
        if let Pattern::Variable(name) = pat {
            assert_eq!(name, "x".into());
        }
    }

    #[test]
    fn test_pattern_wildcard() {
        let pat = Pattern::Wildcard;
        assert!(matches!(pat, Pattern::Wildcard));
    }

    #[test]
    fn test_pattern_tuple() {
        let pat = Pattern::Tuple(vec![
            Pattern::Variable("a".into()),
            Pattern::Variable("b".into()),
        ]);

        match pat {
            Pattern::Tuple(patterns) => {
                assert_eq!(patterns.len(), 2);
            }
            _ => panic!("Expected Tuple pattern"),
        }
    }

    #[test]
    fn test_pattern_record() {
        let pat = Pattern::Record(vec![
            ("x".into(), Pattern::Variable("a".into())),
            ("y".into(), Pattern::Variable("b".into())),
        ]);

        match pat {
            Pattern::Record(fields) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "x".into());
                assert_eq!(fields[1].0, "y".into());
            }
            _ => panic!("Expected Record pattern"),
        }
    }

    #[test]
    fn test_pattern_list() {
        let pat = Pattern::List {
            elements: vec![Pattern::Variable("head".into())],
            rest: Some("tail".into()),
        };

        match pat {
            Pattern::List { elements, rest } => {
                assert_eq!(elements.len(), 1);
                assert_eq!(rest, Some("tail".into()));
            }
            _ => panic!("Expected List pattern"),
        }
    }

    #[test]
    fn test_pattern_literal() {
        let pat = Pattern::Literal(Literal::Int(42));
        assert!(matches!(pat, Pattern::Literal(Literal::Int(42))));
    }

    // =========================================================================
    // Literal Tests
    // =========================================================================

    #[test]
    fn test_literal_variants() {
        let int_lit = Literal::Int(42);
        let float_lit = Literal::Float(1.5);
        let string_lit = Literal::String("hello".into());
        let bool_lit = Literal::Bool(true);
        let null_lit = Literal::Null;

        assert_eq!(int_lit, Literal::Int(42));
        assert_eq!(float_lit, Literal::Float(1.5));
        assert_eq!(string_lit, Literal::String("hello".into()));
        assert_eq!(bool_lit, Literal::Bool(true));
        assert_eq!(null_lit, Literal::Null);
    }

    // =========================================================================
    // Effect Type Tests
    // =========================================================================

    #[test]
    fn test_effect_types() {
        let effects = [
            EffectType::Observe,
            EffectType::Read,
            EffectType::Analyze,
            EffectType::Decide,
            EffectType::Act,
            EffectType::Write,
            EffectType::External,
        ];

        // Ensure all effect types are distinct
        for (i, e1) in effects.iter().enumerate() {
            for (j, e2) in effects.iter().enumerate() {
                if i != j {
                    assert_ne!(e1, e2);
                }
            }
        }
    }

    // =========================================================================
    // Decision Tests
    // =========================================================================

    #[test]
    fn test_decision_variants() {
        let permit = Decision::Permit;
        let deny = Decision::Deny;
        let require = Decision::RequireApproval {
            role: "admin".into(),
        };
        let escalate = Decision::Escalate;

        assert!(matches!(permit, Decision::Permit));
        assert!(matches!(deny, Decision::Deny));
        assert!(matches!(require, Decision::RequireApproval { .. }));
        assert!(matches!(escalate, Decision::Escalate));
    }

    // =========================================================================
    // ActionRef Tests
    // =========================================================================

    #[test]
    fn test_action_ref() {
        let action = ActionRef {
            name: "send_email".into(),
            args: vec![Expr::Literal(Literal::String("test".into()))],
        };

        assert_eq!(action.name, "send_email".into());
        assert_eq!(action.args.len(), 1);
    }

    // =========================================================================
    // ObligationRef Tests
    // =========================================================================

    #[test]
    fn test_obligation_ref() {
        let obligation = ObligationRef {
            role: "admin".into(),
            condition: Expr::Literal(Literal::Bool(true)),
        };

        assert_eq!(obligation.role, "admin".into());
        assert!(matches!(
            obligation.condition,
            Expr::Literal(Literal::Bool(true))
        ));
    }

    // =========================================================================
    // Param Tests
    // =========================================================================

    #[test]
    fn test_param() {
        let param = Param {
            name: "x".into(),
            ty: Type::Name("Int".into()),
        };

        assert_eq!(param.name, "x".into());
        assert!(matches!(param.ty, Type::Name(_)));
    }

    // =========================================================================
    // Type Tests
    // =========================================================================

    #[test]
    fn test_type_variants() {
        let name_ty = Type::Name("Int".into());
        let list_ty = Type::List(Box::new(Type::Name("Int".into())));
        let record_ty = Type::Record(vec![
            ("x".into(), Type::Name("Int".into())),
            ("y".into(), Type::Name("String".into())),
        ]);
        let cap_ty = Type::Capability("Read".into());

        assert!(matches!(name_ty, Type::Name(_)));
        assert!(matches!(list_ty, Type::List(_)));
        assert!(matches!(record_ty, Type::Record(_)));
        assert!(matches!(cap_ty, Type::Capability(_)));
    }

    // =========================================================================
    // Guard Tests
    // =========================================================================

    #[test]
    fn test_guard_variants() {
        let always = Guard::Always;
        let never = Guard::Never;
        let pred = Guard::Pred(Predicate {
            name: "is_valid".into(),
            args: vec![],
        });
        let and = Guard::And(Box::new(Guard::Always), Box::new(Guard::Never));
        let or = Guard::Or(Box::new(Guard::Always), Box::new(Guard::Never));
        let not = Guard::Not(Box::new(Guard::Never));

        assert!(matches!(always, Guard::Always));
        assert!(matches!(never, Guard::Never));
        assert!(matches!(pred, Guard::Pred(_)));
        assert!(matches!(and, Guard::And(_, _)));
        assert!(matches!(or, Guard::Or(_, _)));
        assert!(matches!(not, Guard::Not(_)));
    }

    // =========================================================================
    // Predicate Tests
    // =========================================================================

    #[test]
    fn test_predicate() {
        let pred = Predicate {
            name: "is_admin".into(),
            args: vec![Expr::Variable("user".into())],
        };

        assert_eq!(pred.name, "is_admin".into());
        assert_eq!(pred.args.len(), 1);
    }

    // =========================================================================
    // Constraint Tests
    // =========================================================================

    #[test]
    fn test_constraint() {
        let constraint = Constraint {
            predicate: Predicate {
                name: "is_positive".into(),
                args: vec![],
            },
        };

        assert_eq!(constraint.predicate.name, "is_positive".into());
    }

    // =========================================================================
    // Spanned Trait Tests
    // =========================================================================

    #[test]
    fn test_workflow_spanned() {
        let span = Span::new(10, 20, 2, 5);
        let wf = Workflow::Done { span };

        assert_eq!(wf.span(), span);
    }

    #[test]
    fn test_expr_spanned() {
        let span = Span::new(5, 15, 1, 3);
        let expr = Expr::FieldAccess {
            base: Box::new(Expr::Variable("obj".into())),
            field: "field".into(),
            span,
        };

        assert_eq!(expr.span(), span);

        // Literals and variables return default span
        let lit = Expr::Literal(Literal::Int(42));
        assert_eq!(lit.span(), Span::default());

        let var = Expr::Variable("x".into());
        assert_eq!(var.span(), Span::default());
    }

    #[test]
    fn test_spanned_trait_for_all_workflow_variants() {
        let span = Span::new(0, 10, 1, 1);

        // Test that span() returns the correct span for each variant
        assert_eq!(
            Workflow::Observe {
                capability: "x".into(),
                binding: None,
                continuation: None,
                span
            }
            .span(),
            span
        );
        assert_eq!(
            Workflow::Orient {
                expr: Expr::Literal(Literal::Null),
                binding: None,
                continuation: None,
                span
            }
            .span(),
            span
        );
        assert_eq!(
            Workflow::Propose {
                action: ActionRef {
                    name: "x".into(),
                    args: vec![]
                },
                binding: None,
                continuation: None,
                span
            }
            .span(),
            span
        );
        assert_eq!(
            Workflow::Decide {
                expr: Expr::Literal(Literal::Null),
                policy: None,
                then_branch: Box::new(Workflow::Done { span }),
                else_branch: None,
                span
            }
            .span(),
            span
        );
        assert_eq!(
            Workflow::Check {
                obligation: ObligationRef {
                    role: "x".into(),
                    condition: Expr::Literal(Literal::Null)
                },
                continuation: None,
                span
            }
            .span(),
            span
        );
        assert_eq!(
            Workflow::Act {
                action: ActionRef {
                    name: "x".into(),
                    args: vec![]
                },
                guard: None,
                span
            }
            .span(),
            span
        );
        assert_eq!(
            Workflow::Let {
                pattern: Pattern::Wildcard,
                expr: Expr::Literal(Literal::Null),
                continuation: None,
                span
            }
            .span(),
            span
        );
        assert_eq!(
            Workflow::If {
                condition: Expr::Literal(Literal::Null),
                then_branch: Box::new(Workflow::Done { span }),
                else_branch: None,
                span
            }
            .span(),
            span
        );
        assert_eq!(
            Workflow::For {
                pattern: Pattern::Wildcard,
                collection: Expr::Literal(Literal::Null),
                body: Box::new(Workflow::Done { span }),
                span
            }
            .span(),
            span
        );
        assert_eq!(
            Workflow::Par {
                branches: vec![],
                span
            }
            .span(),
            span
        );
        assert_eq!(
            Workflow::With {
                capability: "x".into(),
                body: Box::new(Workflow::Done { span }),
                span
            }
            .span(),
            span
        );
        assert_eq!(
            Workflow::Maybe {
                primary: Box::new(Workflow::Done { span }),
                fallback: Box::new(Workflow::Done { span }),
                span
            }
            .span(),
            span
        );
        assert_eq!(
            Workflow::Must {
                body: Box::new(Workflow::Done { span }),
                span
            }
            .span(),
            span
        );
        assert_eq!(
            Workflow::Seq {
                first: Box::new(Workflow::Done { span }),
                second: Box::new(Workflow::Done { span }),
                span
            }
            .span(),
            span
        );
        assert_eq!(Workflow::Done { span }.span(), span);
    }
}
