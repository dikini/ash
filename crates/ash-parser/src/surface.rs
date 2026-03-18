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
    /// Type parameters for generic policies
    pub type_params: Vec<Name>,
    /// Fields of the policy
    pub fields: Vec<PolicyField>,
    /// Where clause for invariants
    pub where_clause: Option<Expr>,
    /// Source span
    pub span: Span,
}

/// A field in a policy definition.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyField {
    /// Name of the field
    pub name: Name,
    /// Type of the field
    pub ty: Type,
    /// Default value (optional)
    pub default: Option<Expr>,
    /// Source span
    pub span: Span,
}

/// A policy instance (usage of a policy).
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyInstance {
    /// Name of the policy being instantiated
    pub name: Name,
    /// Field initializations
    pub fields: Vec<(Name, Expr)>,
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

/// Visibility modifiers for definitions (pub, pub(crate), etc.)
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub enum Visibility {
    /// Default visibility (private to module)
    #[default]
    Inherited,
    /// `pub` - visible everywhere
    Public,
    /// `pub(crate)` - visible within the crate/package
    Crate,
    /// `pub(super)` - visible to parent module
    Super,
    /// `pub(self)` - equivalent to private (explicit)
    Self_,
    /// `pub(in path)` - visible in specific module path
    Restricted { path: Box<str> },
}

impl Visibility {
    /// Check if this visibility is public (not inherited/private)
    pub fn is_pub(&self) -> bool {
        !matches!(self, Visibility::Inherited)
    }

    /// Check if an item with this visibility is accessible from the given module
    ///
    /// # Arguments
    /// * `from` - the module path where the access is occurring
    /// * `owner` - the module path where the item is defined
    pub fn is_visible_in_module(&self, from: &str, owner: &str) -> bool {
        match self {
            Visibility::Inherited => from == owner,
            Visibility::Public => true,
            Visibility::Crate => !from.starts_with("external"),
            Visibility::Super => from.starts_with(owner),
            Visibility::Self_ => from == owner,
            Visibility::Restricted { path } => from.starts_with(path.as_ref()),
        }
    }
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
    /// Check phase: verify an obligation or policy instance
    Check {
        /// The check target - either an obligation reference (legacy) or policy instance
        target: CheckTarget,
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
    /// Ret: return an expression
    Ret {
        /// Expression to return
        expr: Expr,
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
    /// Policy expression
    Policy(PolicyExpr),
}

/// Policy expression for combinators.
///
/// Policy expressions allow building complex policies from simple primitives
/// using logical, arithmetic, and higher-order combinators (SPEC-007).
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyExpr {
    /// Variable reference to a policy
    Var(Name),
    /// Conjunction: all policies must hold
    And(Vec<PolicyExpr>),
    /// Disjunction: at least one policy must hold
    Or(Vec<PolicyExpr>),
    /// Negation: policy must not hold
    Not(Box<PolicyExpr>),
    /// Implication: if antecedent then consequent
    Implies(Box<PolicyExpr>, Box<PolicyExpr>),
    /// Sequential composition: policies apply in order
    Sequential(Vec<PolicyExpr>),
    /// Concurrent composition: policies apply simultaneously
    Concurrent(Vec<PolicyExpr>),
    /// Universal quantifier: all items satisfy the policy
    ForAll {
        /// Variable name for each item
        var: Name,
        /// Collection expression
        items: Box<Expr>,
        /// Policy body
        body: Box<PolicyExpr>,
        /// Source span
        span: Span,
    },
    /// Existential quantifier: at least one item satisfies the policy
    Exists {
        /// Variable name for each item
        var: Name,
        /// Collection expression
        items: Box<Expr>,
        /// Policy body
        body: Box<PolicyExpr>,
        /// Source span
        span: Span,
    },
    /// Method call on a policy: receiver.method(args)
    MethodCall {
        /// Receiver policy expression
        receiver: Box<PolicyExpr>,
        /// Method name
        method: Name,
        /// Method arguments
        args: Vec<Expr>,
        /// Source span
        span: Span,
    },
    /// Function call returning a policy
    Call {
        /// Function name
        func: Name,
        /// Function arguments
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
    /// List literal: [1, 2, 3]
    List(Vec<Literal>),
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

/// Target of a check statement - either an obligation or a policy instance.
#[derive(Debug, Clone, PartialEq)]
pub enum CheckTarget {
    /// Legacy obligation reference
    Obligation(ObligationRef),
    /// Policy instance check
    Policy(PolicyInstance),
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
            Workflow::Ret { span, .. } => *span,
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
            Expr::Policy(policy_expr) => policy_expr.span(),
        }
    }
}

impl Spanned for PolicyExpr {
    fn span(&self) -> Span {
        match self {
            PolicyExpr::Var(_) => Span::default(),
            PolicyExpr::And(exprs) => {
                // Return span of first expression, or default if empty
                exprs.first().map(Spanned::span).unwrap_or_default()
            }
            PolicyExpr::Or(exprs) => exprs.first().map(Spanned::span).unwrap_or_default(),
            PolicyExpr::Not(expr) => expr.span(),
            PolicyExpr::Implies(left, _) => left.span(),
            PolicyExpr::Sequential(exprs) => exprs.first().map(Spanned::span).unwrap_or_default(),
            PolicyExpr::Concurrent(exprs) => exprs.first().map(Spanned::span).unwrap_or_default(),
            PolicyExpr::ForAll { span, .. } => *span,
            PolicyExpr::Exists { span, .. } => *span,
            PolicyExpr::MethodCall { span, .. } => *span,
            PolicyExpr::Call { span, .. } => *span,
        }
    }
}

impl Spanned for PolicyInstance {
    fn span(&self) -> Span {
        self.span
    }
}

impl Workflow {
    /// Compute the total effect of this workflow.
    ///
    /// Effects form a lattice: Epistemic < Deliberative < Evaluative < Operational
    /// This method computes the join (⊔) of all effects in the workflow.
    pub fn effect(&self) -> ash_core::Effect {
        use ash_core::Effect;

        match self {
            // Read-only observation - pure reads
            Workflow::Observe { continuation, .. } => {
                if let Some(cont) = continuation {
                    Effect::Epistemic.join(cont.effect())
                } else {
                    Effect::Epistemic
                }
            }

            // Pure expression evaluation
            Workflow::Orient { continuation, .. } => {
                if let Some(cont) = continuation {
                    Effect::Epistemic.join(cont.effect())
                } else {
                    Effect::Epistemic
                }
            }

            // Proposing actions requires deliberation
            Workflow::Propose { continuation, .. } => {
                if let Some(cont) = continuation {
                    Effect::Deliberative.join(cont.effect())
                } else {
                    Effect::Deliberative
                }
            }

            // Decision branches - join of both branches
            Workflow::Decide {
                then_branch,
                else_branch,
                ..
            } => {
                let then_effect = then_branch.effect();
                match else_branch {
                    Some(else_b) => then_effect.join(else_b.effect()),
                    None => then_effect,
                }
            }

            // Checking obligations/policies is evaluative
            Workflow::Check { continuation, .. } => {
                if let Some(cont) = continuation {
                    Effect::Evaluative.join(cont.effect())
                } else {
                    Effect::Evaluative
                }
            }

            // Executing actions has side effects
            Workflow::Act { .. } => Effect::Operational,

            // Let binding - effect of the continuation
            Workflow::Let { continuation, .. } => {
                if let Some(cont) = continuation {
                    cont.effect()
                } else {
                    Effect::Epistemic
                }
            }

            // Conditional - join of branches
            Workflow::If {
                then_branch,
                else_branch,
                ..
            } => {
                let then_effect = then_branch.effect();
                match else_branch {
                    Some(else_b) => then_effect.join(else_b.effect()),
                    None => then_effect,
                }
            }

            // For loop - effect of body
            Workflow::For { body, .. } => body.effect(),

            // Parallel composition - join of all branches
            Workflow::Par { branches, .. } => branches
                .iter()
                .map(|w| w.effect())
                .fold(Effect::Epistemic, |a, b| a.join(b)),

            // With clause - effect of body
            Workflow::With { body, .. } => body.effect(),

            // Maybe - join of primary and fallback
            Workflow::Maybe {
                primary, fallback, ..
            } => primary.effect().join(fallback.effect()),

            // Must - effect of body
            Workflow::Must { body, .. } => body.effect(),

            // Sequential composition - join of both
            Workflow::Seq { first, second, .. } => first.effect().join(second.effect()),

            // Done - no effect
            Workflow::Done { .. } => Effect::Epistemic,

            // Return - no effect
            Workflow::Ret { .. } => Effect::Epistemic,
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
            name: "RateLimit".into(),
            type_params: vec![],
            fields: vec![
                PolicyField {
                    name: "requests".into(),
                    ty: Type::Name("Int".into()),
                    default: None,
                    span: Span::new(0, 10, 1, 1),
                },
                PolicyField {
                    name: "window_secs".into(),
                    ty: Type::Name("Int".into()),
                    default: None,
                    span: Span::new(0, 10, 1, 1),
                },
            ],
            where_clause: None,
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
            name: "BoundedResource".into(),
            type_params: vec![],
            fields: vec![
                PolicyField {
                    name: "min".into(),
                    ty: Type::Name("Int".into()),
                    default: None,
                    span: Span::new(0, 10, 1, 1),
                },
                PolicyField {
                    name: "max".into(),
                    ty: Type::Name("Int".into()),
                    default: None,
                    span: Span::new(0, 10, 1, 1),
                },
            ],
            where_clause: Some(Expr::Binary {
                op: BinaryOp::Leq,
                left: Box::new(Expr::Variable("min".into())),
                right: Box::new(Expr::Variable("max".into())),
                span: Span::new(0, 10, 1, 1),
            }),
            span: Span::new(0, 30, 1, 1),
        };

        assert_eq!(policy.name, "BoundedResource".into());
        assert_eq!(policy.fields.len(), 2);
        assert!(policy.where_clause.is_some());
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
            target: CheckTarget::Obligation(ObligationRef {
                role: "admin".into(),
                condition: Expr::Literal(Literal::Bool(true)),
            }),
            continuation: Some(Box::new(Workflow::Done {
                span: Span::new(0, 4, 1, 1),
            })),
            span: Span::new(0, 20, 1, 1),
        };

        match wf {
            Workflow::Check {
                target,
                continuation,
                ..
            } => {
                match target {
                    CheckTarget::Obligation(obl) => {
                        assert_eq!(obl.role, "admin".into());
                    }
                    _ => panic!("Expected obligation target"),
                }
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
    // PolicyExpr Tests
    // =========================================================================

    #[test]
    fn test_policy_expr_var() {
        let expr = PolicyExpr::Var("my_policy".into());
        assert!(matches!(expr, PolicyExpr::Var(name) if name.as_ref() == "my_policy"));
    }

    #[test]
    fn test_policy_expr_and() {
        let expr = PolicyExpr::And(vec![
            PolicyExpr::Var("p1".into()),
            PolicyExpr::Var("p2".into()),
        ]);
        match expr {
            PolicyExpr::And(exprs) => assert_eq!(exprs.len(), 2),
            _ => panic!("Expected And"),
        }
    }

    #[test]
    fn test_policy_expr_or() {
        let expr = PolicyExpr::Or(vec![
            PolicyExpr::Var("p1".into()),
            PolicyExpr::Var("p2".into()),
        ]);
        match expr {
            PolicyExpr::Or(exprs) => assert_eq!(exprs.len(), 2),
            _ => panic!("Expected Or"),
        }
    }

    #[test]
    fn test_policy_expr_not() {
        let expr = PolicyExpr::Not(Box::new(PolicyExpr::Var("p".into())));
        match expr {
            PolicyExpr::Not(inner) => {
                assert!(matches!(inner.as_ref(), PolicyExpr::Var(_)));
            }
            _ => panic!("Expected Not"),
        }
    }

    #[test]
    fn test_policy_expr_implies() {
        let expr = PolicyExpr::Implies(
            Box::new(PolicyExpr::Var("a".into())),
            Box::new(PolicyExpr::Var("b".into())),
        );
        match expr {
            PolicyExpr::Implies(left, right) => {
                assert!(matches!(left.as_ref(), PolicyExpr::Var(_)));
                assert!(matches!(right.as_ref(), PolicyExpr::Var(_)));
            }
            _ => panic!("Expected Implies"),
        }
    }

    #[test]
    fn test_policy_expr_sequential() {
        let expr = PolicyExpr::Sequential(vec![
            PolicyExpr::Var("p1".into()),
            PolicyExpr::Var("p2".into()),
            PolicyExpr::Var("p3".into()),
        ]);
        match expr {
            PolicyExpr::Sequential(exprs) => assert_eq!(exprs.len(), 3),
            _ => panic!("Expected Sequential"),
        }
    }

    #[test]
    fn test_policy_expr_concurrent() {
        let expr = PolicyExpr::Concurrent(vec![
            PolicyExpr::Var("p1".into()),
            PolicyExpr::Var("p2".into()),
        ]);
        match expr {
            PolicyExpr::Concurrent(exprs) => assert_eq!(exprs.len(), 2),
            _ => panic!("Expected Concurrent"),
        }
    }

    #[test]
    fn test_policy_expr_forall() {
        let expr = PolicyExpr::ForAll {
            var: "x".into(),
            items: Box::new(Expr::Variable("items".into())),
            body: Box::new(PolicyExpr::Var("policy".into())),
            span: Span::new(0, 20, 1, 1),
        };
        match expr {
            PolicyExpr::ForAll { var, body, .. } => {
                assert_eq!(var.as_ref(), "x");
                assert!(matches!(body.as_ref(), PolicyExpr::Var(_)));
            }
            _ => panic!("Expected ForAll"),
        }
    }

    #[test]
    fn test_policy_expr_exists() {
        let expr = PolicyExpr::Exists {
            var: "x".into(),
            items: Box::new(Expr::Variable("items".into())),
            body: Box::new(PolicyExpr::Var("policy".into())),
            span: Span::new(0, 20, 1, 1),
        };
        match expr {
            PolicyExpr::Exists { var, body, .. } => {
                assert_eq!(var.as_ref(), "x");
                assert!(matches!(body.as_ref(), PolicyExpr::Var(_)));
            }
            _ => panic!("Expected Exists"),
        }
    }

    #[test]
    fn test_policy_expr_method_call() {
        let expr = PolicyExpr::MethodCall {
            receiver: Box::new(PolicyExpr::Var("base".into())),
            method: "and".into(),
            args: vec![Expr::Variable("other".into())],
            span: Span::new(0, 15, 1, 1),
        };
        match expr {
            PolicyExpr::MethodCall {
                receiver,
                method,
                args,
                ..
            } => {
                assert!(matches!(receiver.as_ref(), PolicyExpr::Var(_)));
                assert_eq!(method.as_ref(), "and");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected MethodCall"),
        }
    }

    #[test]
    fn test_policy_expr_call() {
        let expr = PolicyExpr::Call {
            func: "rate_limit".into(),
            args: vec![Expr::Literal(Literal::Int(100))],
            span: Span::new(0, 15, 1, 1),
        };
        match expr {
            PolicyExpr::Call { func, args, .. } => {
                assert_eq!(func.as_ref(), "rate_limit");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_policy_expr_span() {
        let span = Span::new(10, 20, 1, 5);
        let expr = PolicyExpr::Call {
            func: "test".into(),
            args: vec![],
            span,
        };
        assert_eq!(expr.span(), span);
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
                target: CheckTarget::Obligation(ObligationRef {
                    role: "x".into(),
                    condition: Expr::Literal(Literal::Null)
                }),
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

#[cfg(test)]
mod visibility_tests {
    use super::*;

    #[test]
    fn test_visibility_private() {
        let vis = Visibility::Inherited;
        assert!(!vis.is_pub());
    }

    #[test]
    fn test_visibility_public() {
        let vis = Visibility::Public;
        assert!(vis.is_pub());
    }

    #[test]
    fn test_visibility_crate() {
        let vis = Visibility::Crate;
        assert!(vis.is_pub());
        assert!(vis.is_visible_in_module("crate::foo", "crate::bar"));
    }

    #[test]
    fn test_visibility_super() {
        let vis = Visibility::Super;
        assert!(vis.is_pub());
        assert!(vis.is_visible_in_module("crate::foo::bar", "crate::foo"));
    }

    #[test]
    fn test_visibility_self() {
        let vis = Visibility::Self_;
        assert!(vis.is_pub());
        assert!(vis.is_visible_in_module("crate::foo", "crate::foo"));
        assert!(!vis.is_visible_in_module("crate::foo", "crate::bar"));
    }

    #[test]
    fn test_visibility_restricted() {
        let vis = Visibility::Restricted {
            path: "crate::internal".into(),
        };
        assert!(vis.is_pub());
        assert!(vis.is_visible_in_module("crate::internal::sub", "crate::other"));
        assert!(!vis.is_visible_in_module("crate::public", "crate::other"));
    }
}

#[cfg(test)]
mod effect_tests {
    use super::*;
    use ash_core::Effect;

    fn dummy_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    // =========================================================================
    // Basic Effect Tests
    // =========================================================================

    #[test]
    fn test_observe_effect() {
        let workflow = Workflow::Observe {
            capability: "sensor".into(),
            binding: Some(Pattern::Variable("data".into())),
            continuation: None,
            span: dummy_span(),
        };
        assert_eq!(workflow.effect(), Effect::Epistemic);
    }

    #[test]
    fn test_observe_with_continuation() {
        let workflow = Workflow::Observe {
            capability: "sensor".into(),
            binding: Some(Pattern::Variable("data".into())),
            continuation: Some(Box::new(Workflow::Act {
                action: ActionRef {
                    name: "process".into(),
                    args: vec![],
                },
                guard: None,
                span: dummy_span(),
            })),
            span: dummy_span(),
        };
        // Epistemic join Operational = Operational
        assert_eq!(workflow.effect(), Effect::Operational);
    }

    #[test]
    fn test_orient_effect() {
        let workflow = Workflow::Orient {
            expr: Expr::Literal(Literal::Int(42)),
            binding: Some(Pattern::Variable("result".into())),
            continuation: None,
            span: dummy_span(),
        };
        assert_eq!(workflow.effect(), Effect::Epistemic);
    }

    #[test]
    fn test_propose_effect() {
        let workflow = Workflow::Propose {
            action: ActionRef {
                name: "send_email".into(),
                args: vec![],
            },
            binding: None,
            continuation: None,
            span: dummy_span(),
        };
        assert_eq!(workflow.effect(), Effect::Deliberative);
    }

    #[test]
    fn test_decide_effect() {
        let workflow = Workflow::Decide {
            expr: Expr::Literal(Literal::Bool(true)),
            policy: None,
            then_branch: Box::new(Workflow::Done { span: dummy_span() }),
            else_branch: None,
            span: dummy_span(),
        };
        assert_eq!(workflow.effect(), Effect::Epistemic);
    }

    #[test]
    fn test_check_effect() {
        let workflow = Workflow::Check {
            target: CheckTarget::Obligation(ObligationRef {
                role: "admin".into(),
                condition: Expr::Literal(Literal::Bool(true)),
            }),
            continuation: None,
            span: dummy_span(),
        };
        assert_eq!(workflow.effect(), Effect::Evaluative);
    }

    #[test]
    fn test_act_effect() {
        let workflow = Workflow::Act {
            action: ActionRef {
                name: "write_file".into(),
                args: vec![],
            },
            guard: None,
            span: dummy_span(),
        };
        assert_eq!(workflow.effect(), Effect::Operational);
    }

    #[test]
    fn test_done_effect() {
        let workflow = Workflow::Done { span: dummy_span() };
        assert_eq!(workflow.effect(), Effect::Epistemic);
    }

    #[test]
    fn test_ret_effect() {
        let workflow = Workflow::Ret {
            expr: Expr::Literal(Literal::Int(42)),
            span: dummy_span(),
        };
        assert_eq!(workflow.effect(), Effect::Epistemic);
    }

    // =========================================================================
    // Composite Workflow Effect Tests
    // =========================================================================

    #[test]
    fn test_seq_effect_epistemic_operational() {
        // observe (Epistemic) ; act (Operational) = Operational
        let observe = Workflow::Observe {
            capability: "sensor".into(),
            binding: None,
            continuation: None,
            span: dummy_span(),
        };
        let act = Workflow::Act {
            action: ActionRef {
                name: "process".into(),
                args: vec![],
            },
            guard: None,
            span: dummy_span(),
        };
        let seq = Workflow::Seq {
            first: Box::new(observe),
            second: Box::new(act),
            span: dummy_span(),
        };
        assert_eq!(seq.effect(), Effect::Operational);
    }

    #[test]
    fn test_seq_effect_epistemic_epistemic() {
        // observe (Epistemic) ; observe (Epistemic) = Epistemic
        let observe1 = Workflow::Observe {
            capability: "sensor1".into(),
            binding: None,
            continuation: None,
            span: dummy_span(),
        };
        let observe2 = Workflow::Observe {
            capability: "sensor2".into(),
            binding: None,
            continuation: None,
            span: dummy_span(),
        };
        let seq = Workflow::Seq {
            first: Box::new(observe1),
            second: Box::new(observe2),
            span: dummy_span(),
        };
        assert_eq!(seq.effect(), Effect::Epistemic);
    }

    #[test]
    fn test_par_effect_mixed() {
        // par [observe (Epistemic), act (Operational)] = Operational
        let observe = Workflow::Observe {
            capability: "sensor".into(),
            binding: None,
            continuation: None,
            span: dummy_span(),
        };
        let act = Workflow::Act {
            action: ActionRef {
                name: "process".into(),
                args: vec![],
            },
            guard: None,
            span: dummy_span(),
        };
        let par = Workflow::Par {
            branches: vec![observe, act],
            span: dummy_span(),
        };
        assert_eq!(par.effect(), Effect::Operational);
    }

    #[test]
    fn test_par_effect_empty() {
        // par [] = Epistemic (identity for join)
        let par = Workflow::Par {
            branches: vec![],
            span: dummy_span(),
        };
        assert_eq!(par.effect(), Effect::Epistemic);
    }

    #[test]
    fn test_if_effect_both_branches() {
        // if cond then observe else act = Operational
        let observe = Workflow::Observe {
            capability: "sensor".into(),
            binding: None,
            continuation: None,
            span: dummy_span(),
        };
        let act = Workflow::Act {
            action: ActionRef {
                name: "process".into(),
                args: vec![],
            },
            guard: None,
            span: dummy_span(),
        };
        let if_workflow = Workflow::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_branch: Box::new(observe),
            else_branch: Some(Box::new(act)),
            span: dummy_span(),
        };
        assert_eq!(if_workflow.effect(), Effect::Operational);
    }

    #[test]
    fn test_if_effect_no_else() {
        // if cond then observe (no else) = Epistemic
        let observe = Workflow::Observe {
            capability: "sensor".into(),
            binding: None,
            continuation: None,
            span: dummy_span(),
        };
        let if_workflow = Workflow::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_branch: Box::new(observe),
            else_branch: None,
            span: dummy_span(),
        };
        assert_eq!(if_workflow.effect(), Effect::Epistemic);
    }

    #[test]
    fn test_for_effect() {
        // for x in items { act } = Operational
        let act = Workflow::Act {
            action: ActionRef {
                name: "process".into(),
                args: vec![],
            },
            guard: None,
            span: dummy_span(),
        };
        let for_workflow = Workflow::For {
            pattern: Pattern::Variable("x".into()),
            collection: Expr::Variable("items".into()),
            body: Box::new(act),
            span: dummy_span(),
        };
        assert_eq!(for_workflow.effect(), Effect::Operational);
    }

    #[test]
    fn test_let_effect() {
        // let x = 42 in act = Operational
        let act = Workflow::Act {
            action: ActionRef {
                name: "process".into(),
                args: vec![],
            },
            guard: None,
            span: dummy_span(),
        };
        let let_workflow = Workflow::Let {
            pattern: Pattern::Variable("x".into()),
            expr: Expr::Literal(Literal::Int(42)),
            continuation: Some(Box::new(act)),
            span: dummy_span(),
        };
        assert_eq!(let_workflow.effect(), Effect::Operational);
    }

    #[test]
    fn test_let_no_continuation() {
        // let x = 42 (no continuation) = Epistemic
        let let_workflow = Workflow::Let {
            pattern: Pattern::Variable("x".into()),
            expr: Expr::Literal(Literal::Int(42)),
            continuation: None,
            span: dummy_span(),
        };
        assert_eq!(let_workflow.effect(), Effect::Epistemic);
    }

    #[test]
    fn test_maybe_effect() {
        // maybe { observe } fallback { act } = Operational
        let observe = Workflow::Observe {
            capability: "sensor".into(),
            binding: None,
            continuation: None,
            span: dummy_span(),
        };
        let act = Workflow::Act {
            action: ActionRef {
                name: "process".into(),
                args: vec![],
            },
            guard: None,
            span: dummy_span(),
        };
        let maybe = Workflow::Maybe {
            primary: Box::new(observe),
            fallback: Box::new(act),
            span: dummy_span(),
        };
        assert_eq!(maybe.effect(), Effect::Operational);
    }

    #[test]
    fn test_must_effect() {
        // must { act } = Operational
        let act = Workflow::Act {
            action: ActionRef {
                name: "process".into(),
                args: vec![],
            },
            guard: None,
            span: dummy_span(),
        };
        let must = Workflow::Must {
            body: Box::new(act),
            span: dummy_span(),
        };
        assert_eq!(must.effect(), Effect::Operational);
    }

    #[test]
    fn test_with_effect() {
        // with db { act } = Operational
        let act = Workflow::Act {
            action: ActionRef {
                name: "query".into(),
                args: vec![],
            },
            guard: None,
            span: dummy_span(),
        };
        let with = Workflow::With {
            capability: "database".into(),
            body: Box::new(act),
            span: dummy_span(),
        };
        assert_eq!(with.effect(), Effect::Operational);
    }

    // =========================================================================
    // Lattice Property Tests
    // =========================================================================

    #[test]
    fn test_effect_lattice_ordering() {
        // Verify the lattice ordering: Epistemic < Deliberative < Evaluative < Operational
        let epistemic = Workflow::Observe {
            capability: "x".into(),
            binding: None,
            continuation: None,
            span: dummy_span(),
        };
        let deliberative = Workflow::Propose {
            action: ActionRef {
                name: "x".into(),
                args: vec![],
            },
            binding: None,
            continuation: None,
            span: dummy_span(),
        };
        let evaluative = Workflow::Check {
            target: CheckTarget::Obligation(ObligationRef {
                role: "admin".into(),
                condition: Expr::Literal(Literal::Bool(true)),
            }),
            continuation: None,
            span: dummy_span(),
        };
        let operational = Workflow::Act {
            action: ActionRef {
                name: "x".into(),
                args: vec![],
            },
            guard: None,
            span: dummy_span(),
        };

        assert_eq!(epistemic.effect(), Effect::Epistemic);
        assert_eq!(deliberative.effect(), Effect::Deliberative);
        assert_eq!(evaluative.effect(), Effect::Evaluative);
        assert_eq!(operational.effect(), Effect::Operational);

        // Verify ordering through joins
        assert_eq!(
            Workflow::Seq {
                first: Box::new(epistemic.clone()),
                second: Box::new(deliberative.clone()),
                span: dummy_span(),
            }
            .effect(),
            Effect::Deliberative
        );
        assert_eq!(
            Workflow::Seq {
                first: Box::new(deliberative.clone()),
                second: Box::new(evaluative.clone()),
                span: dummy_span(),
            }
            .effect(),
            Effect::Evaluative
        );
        assert_eq!(
            Workflow::Seq {
                first: Box::new(evaluative.clone()),
                second: Box::new(operational.clone()),
                span: dummy_span(),
            }
            .effect(),
            Effect::Operational
        );
    }

    #[test]
    fn test_nested_composite_effects() {
        // Complex nested workflow: seq { par { observe, observe }, act }
        let observe1 = Workflow::Observe {
            capability: "sensor1".into(),
            binding: None,
            continuation: None,
            span: dummy_span(),
        };
        let observe2 = Workflow::Observe {
            capability: "sensor2".into(),
            binding: None,
            continuation: None,
            span: dummy_span(),
        };
        let par = Workflow::Par {
            branches: vec![observe1, observe2],
            span: dummy_span(),
        };
        let act = Workflow::Act {
            action: ActionRef {
                name: "process".into(),
                args: vec![],
            },
            guard: None,
            span: dummy_span(),
        };
        let seq = Workflow::Seq {
            first: Box::new(par),
            second: Box::new(act),
            span: dummy_span(),
        };

        // par { observe, observe } = Epistemic
        // seq { Epistemic, act } = Operational
        assert_eq!(seq.effect(), Effect::Operational);
    }

    #[test]
    fn test_decide_with_operational_branches() {
        // decide with operational branches
        let act_then = Workflow::Act {
            action: ActionRef {
                name: "process".into(),
                args: vec![],
            },
            guard: None,
            span: dummy_span(),
        };
        let act_else = Workflow::Act {
            action: ActionRef {
                name: "cleanup".into(),
                args: vec![],
            },
            guard: None,
            span: dummy_span(),
        };
        let decide = Workflow::Decide {
            expr: Expr::Literal(Literal::Bool(true)),
            policy: None,
            then_branch: Box::new(act_then),
            else_branch: Some(Box::new(act_else)),
            span: dummy_span(),
        };

        assert_eq!(decide.effect(), Effect::Operational);
    }
}
