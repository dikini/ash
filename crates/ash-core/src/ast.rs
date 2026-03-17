//! Abstract Syntax Tree for Ash workflows

use crate::{Effect, Provenance, Value};

/// A workflow name
pub type Name = String;

/// Core workflow AST
#[derive(Debug, Clone, PartialEq)]
pub enum Workflow {
    /// OBSERVE capability as pattern in continuation
    Observe {
        capability: Capability,
        pattern: Pattern,
        continuation: Box<Workflow>,
    },
    /// ORIENT expression then continue
    Orient {
        expr: Expr,
        continuation: Box<Workflow>,
    },
    /// PROPOSE action (advisory)
    Propose {
        action: Action,
        continuation: Box<Workflow>,
    },
    /// DECIDE expression under policy then continue
    Decide {
        expr: Expr,
        policy: Name,
        continuation: Box<Workflow>,
    },
    /// CHECK obligation then continue
    Check {
        obligation: Obligation,
        continuation: Box<Workflow>,
    },
    /// ACT action where guard with provenance
    Act {
        action: Action,
        guard: Guard,
        provenance: Provenance,
    },
    /// OBLIG role to workflow
    Oblig {
        role: Role,
        workflow: Box<Workflow>,
    },
    /// LET pattern = expr in continuation
    Let {
        pattern: Pattern,
        expr: Expr,
        continuation: Box<Workflow>,
    },
    /// IF expr then else
    If {
        condition: Expr,
        then_branch: Box<Workflow>,
        else_branch: Box<Workflow>,
    },
    /// Sequential composition
    Seq {
        first: Box<Workflow>,
        second: Box<Workflow>,
    },
    /// Parallel composition
    Par {
        workflows: Vec<Workflow>,
    },
    /// FOREACH pattern in expr do workflow
    ForEach {
        pattern: Pattern,
        collection: Expr,
        body: Box<Workflow>,
    },
    /// RET expression
    Ret { expr: Expr },
    /// WITH capability DO workflow
    With {
        capability: Capability,
        workflow: Box<Workflow>,
    },
    /// MAYBE workflow else workflow
    Maybe {
        primary: Box<Workflow>,
        fallback: Box<Workflow>,
    },
    /// MUST workflow
    Must { workflow: Box<Workflow> },
    /// Terminal
    Done,
}

/// A capability reference
#[derive(Debug, Clone, PartialEq)]
pub struct Capability {
    pub name: Name,
    pub effect: Effect,
    pub constraints: Vec<Constraint>,
}

/// An action to execute
#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    pub name: Name,
    pub arguments: Vec<Expr>,
}

/// Pattern for destructuring
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Variable(Name),
    Tuple(Vec<Pattern>),
    Record(Vec<(Name, Pattern)>),
    Wildcard,
    Literal(Value),
}

/// Guard condition
#[derive(Debug, Clone, PartialEq)]
pub enum Guard {
    Pred(Predicate),
    And(Box<Guard>, Box<Guard>),
    Or(Box<Guard>, Box<Guard>),
    Not(Box<Guard>),
    Always,
    Never,
}

/// A predicate
#[derive(Debug, Clone, PartialEq)]
pub struct Predicate {
    pub name: Name,
    pub arguments: Vec<Expr>,
}

/// Expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Value),
    Variable(Name),
    FieldAccess { expr: Box<Expr>, field: Name },
    IndexAccess { expr: Box<Expr>, index: Box<Expr> },
    Unary { op: UnaryOp, expr: Box<Expr> },
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Call { func: Name, arguments: Vec<Expr> },
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    In,
}

/// Constraint on capabilities
#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    pub predicate: Predicate,
}

/// Deontic obligation
#[derive(Debug, Clone, PartialEq)]
pub enum Obligation {
    Obliged { role: Role, condition: Expr },
    Permitted { role: Role, action: Action },
    Prohibited { role: Role, action: Action },
}

/// Role definition
#[derive(Debug, Clone, PartialEq)]
pub struct Role {
    pub name: Name,
    pub authority: Vec<Capability>,
    pub obligations: Vec<Obligation>,
    pub supervises: Vec<Role>,
}
