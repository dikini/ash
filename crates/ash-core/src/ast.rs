//! Abstract Syntax Tree for Ash workflows

use crate::{Effect, Provenance, Value};
use serde::{Deserialize, Serialize};

/// A workflow name
pub type Name = String;

// Re-export instance types from value module for use in AST
pub use crate::value::{ControlLink, Instance, InstanceAddr};

/// Type variable for generic types
pub type TypeVar = String;

/// Core workflow AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    Oblig { role: Role, workflow: Box<Workflow> },
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
    Par { workflows: Vec<Workflow> },
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
    /// SET capability:channel = value
    Set {
        capability: Name,
        channel: Name,
        value: Expr,
    },
    /// SEND capability:channel value
    Send {
        capability: Name,
        channel: Name,
        value: Expr,
    },
    /// Terminal
    Done,

    /// Spawn workflow: spawn MyClass with { init: args } as pattern in continuation
    Spawn {
        workflow_type: Name,
        init: Expr,
        pattern: Pattern,
        continuation: Box<Workflow>,
    },

    /// Split instance: split instance_expr as pattern in continuation
    Split {
        expr: Expr,
        pattern: Pattern,
        continuation: Box<Workflow>,
    },
}

/// A capability reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capability {
    pub name: Name,
    pub effect: Effect,
    pub constraints: Vec<Constraint>,
}

/// An action to execute
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub name: Name,
    pub arguments: Vec<Expr>,
}

/// Pattern for destructuring
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Variable(Name),
    Tuple(Vec<Pattern>),
    Record(Vec<(Name, Pattern)>),
    List(Vec<Pattern>, Option<Name>), // [a, b, ..rest] - prefix patterns with optional rest
    Wildcard,
    Literal(Value),

    /// Variant pattern: Some { value: x } or just Some (unit variant)
    Variant {
        name: Name,
        fields: Option<Vec<(Name, Pattern)>>,
    },
}

impl Pattern {
    /// Returns all variable names bound by this pattern
    pub fn bindings(&self) -> Vec<Name> {
        let mut result = Vec::new();
        self.collect_bindings(&mut result);
        result
    }

    fn collect_bindings(&self, result: &mut Vec<Name>) {
        match self {
            Pattern::Variable(name) => {
                // Skip underscore bindings (wildcard pattern)
                if name.as_str() != "_" {
                    result.push(name.clone());
                }
            }
            Pattern::Tuple(patterns) => {
                for p in patterns {
                    p.collect_bindings(result);
                }
            }
            Pattern::Record(fields) => {
                for (_, p) in fields {
                    p.collect_bindings(result);
                }
            }
            Pattern::List(patterns, rest) => {
                for p in patterns {
                    p.collect_bindings(result);
                }
                if let Some(name) = rest {
                    // Skip underscore bindings (wildcard pattern)
                    if name.as_str() != "_" {
                        result.push(name.clone());
                    }
                }
            }
            Pattern::Wildcard | Pattern::Literal(_) => {
                // No bindings
            }
            Pattern::Variant { fields, .. } => {
                if let Some(fields) = fields {
                    for (_, p) in fields {
                        p.collect_bindings(result);
                    }
                }
            }
        }
    }

    /// Returns true if pattern can fail to match some value
    pub fn is_refutable(&self) -> bool {
        match self {
            // Variable and Wildcard are irrefutable - they match any value
            Pattern::Variable(_) | Pattern::Wildcard => false,
            // Everything else is refutable - can fail to match
            Pattern::Tuple(_) | Pattern::Record(_) | Pattern::List(_, _) | Pattern::Literal(_) => {
                true
            }
            Pattern::Variant { .. } => true,
        }
    }
}

/// Guard condition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Guard {
    Pred(Predicate),
    And(Box<Guard>, Box<Guard>),
    Or(Box<Guard>, Box<Guard>),
    Not(Box<Guard>),
    Always,
    Never,
}

/// A predicate
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Predicate {
    pub name: Name,
    pub arguments: Vec<Expr>,
}

/// Expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Literal(Value),
    Variable(Name),
    FieldAccess {
        expr: Box<Expr>,
        field: Name,
    },
    IndexAccess {
        expr: Box<Expr>,
        index: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        func: Name,
        arguments: Vec<Expr>,
    },

    /// Constructor expression: Some { value: 42 }
    Constructor {
        name: Name,
        fields: Vec<(Name, Expr)>,
    },

    /// Match expression
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    /// If-let expression (sugar for match)
    IfLet {
        pattern: Pattern,
        expr: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },

    /// Spawn expression: spawn workflow_type with { init: args }
    /// Returns an Instance value containing addr and control link
    Spawn {
        workflow_type: Name,
        init: Box<Expr>,
    },

    /// Split expression: split instance_expr
    /// Returns a tuple (InstanceAddr, Option<ControlLink>)
    Split(Box<Expr>),
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
    Neg,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Match arm: pattern => expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
}

/// Constraint on capabilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constraint {
    pub predicate: Predicate,
}

/// Observe expression for sampling behaviour providers
///
/// Represents an `observe capability:channel [where constraints] as pattern` construct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observe {
    /// Capability name (e.g., "sensor" in "sensor:temp")
    pub capability: Name,
    /// Channel name (e.g., "temp" in "sensor:temp")
    pub channel: Name,
    /// Optional constraints for filtering
    pub constraints: Vec<Constraint>,
    /// Pattern to bind the result to
    pub pattern: Pattern,
}

/// Changed expression for change detection
///
/// Represents a `changed capability:channel [where constraints]` construct
/// for detecting changes in observed values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Changed {
    /// Capability name
    pub capability: Name,
    /// Channel name
    pub channel: Name,
    /// Optional constraints for filtering
    pub constraints: Vec<Constraint>,
}

/// Deontic obligation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Obligation {
    Obliged { role: Role, condition: Expr },
    Permitted { role: Role, action: Action },
    Prohibited { role: Role, action: Action },
}

/// Role definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Role {
    pub name: Name,
    pub authority: Vec<Capability>,
    pub obligations: Vec<Obligation>,
    pub supervises: Vec<Role>,
}

/// Type definition in source code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeDef {
    /// Name of the type being defined
    pub name: Name,
    /// Type parameters for generic types
    pub params: Vec<TypeVar>,
    /// Body of the type definition
    pub body: TypeBody,
    /// Visibility of the type
    pub visibility: Visibility,
}

/// Body of a type definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeBody {
    /// type Point = { x: Int, y: Int }
    Struct(Vec<(Name, TypeExpr)>),
    /// type Status = Pending | Processing { ... }
    Enum(Vec<VariantDef>),
    /// type Name = String
    Alias(TypeExpr),
}

/// Variant definition for enums
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantDef {
    /// Name of the variant
    pub name: Name,
    /// Fields of the variant (name, type pairs)
    pub fields: Vec<(Name, TypeExpr)>,
}

/// Visibility modifier
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    /// Public visibility (accessible from anywhere)
    Public,
    /// Crate visibility (accessible within the crate)
    Crate,
    /// Private visibility (accessible only within the module)
    Private,
}

/// Surface syntax type expression (to be resolved)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeExpr {
    /// Named type (e.g., Int, String, MyType)
    Named(Name),
    /// Type constructor application (e.g., Option<Int>)
    Constructor { name: Name, args: Vec<TypeExpr> },
    /// Tuple type (e.g., (Int, String))
    Tuple(Vec<TypeExpr>),
    /// Record type (e.g., { x: Int, y: String })
    Record(Vec<(Name, TypeExpr)>),
}

/// Top-level definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Definition {
    /// Workflow definition
    Workflow(Workflow),
    /// Type definition
    TypeDef(TypeDef),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_construction() {
        // Test Observe
        let _observe = Workflow::Observe {
            capability: Capability {
                name: "read".to_string(),
                effect: Effect::Epistemic,
                constraints: vec![],
            },
            pattern: Pattern::Variable("x".to_string()),
            continuation: Box::new(Workflow::Done),
        };

        // Test Orient
        let _orient = Workflow::Orient {
            expr: Expr::Literal(Value::Int(42)),
            continuation: Box::new(Workflow::Done),
        };

        // Test Propose
        let _propose = Workflow::Propose {
            action: Action {
                name: "do_something".to_string(),
                arguments: vec![],
            },
            continuation: Box::new(Workflow::Done),
        };

        // Test Decide
        let _decide = Workflow::Decide {
            expr: Expr::Literal(Value::Bool(true)),
            policy: "default".to_string(),
            continuation: Box::new(Workflow::Done),
        };

        // Test Let
        let _let_wf = Workflow::Let {
            pattern: Pattern::Tuple(vec![
                Pattern::Variable("a".to_string()),
                Pattern::Variable("b".to_string()),
            ]),
            expr: Expr::Literal(Value::List(vec![Value::Int(1), Value::Int(2)])),
            continuation: Box::new(Workflow::Done),
        };

        // Test If
        let _if_wf = Workflow::If {
            condition: Expr::Literal(Value::Bool(true)),
            then_branch: Box::new(Workflow::Done),
            else_branch: Box::new(Workflow::Done),
        };

        // Test Seq
        let _seq = Workflow::Seq {
            first: Box::new(Workflow::Done),
            second: Box::new(Workflow::Done),
        };

        // Test Par
        let _par = Workflow::Par {
            workflows: vec![Workflow::Done, Workflow::Done],
        };

        // Test ForEach with List pattern
        let _foreach = Workflow::ForEach {
            pattern: Pattern::List(
                vec![Pattern::Variable("head".to_string())],
                Some("tail".to_string()),
            ),
            collection: Expr::Literal(Value::List(vec![])),
            body: Box::new(Workflow::Done),
        };

        // Test Ret
        let _ret = Workflow::Ret {
            expr: Expr::Literal(Value::Int(42)),
        };

        // Test With
        let _with = Workflow::With {
            capability: Capability {
                name: "cap".to_string(),
                effect: Effect::Operational,
                constraints: vec![],
            },
            workflow: Box::new(Workflow::Done),
        };

        // Test Maybe
        let _maybe = Workflow::Maybe {
            primary: Box::new(Workflow::Done),
            fallback: Box::new(Workflow::Done),
        };

        // Test Must
        let _must = Workflow::Must {
            workflow: Box::new(Workflow::Done),
        };

        // Test Done
        let _done = Workflow::Done;
    }

    #[test]
    fn test_pattern_bindings() {
        // Variable binds one name
        let p = Pattern::Variable("x".to_string());
        assert_eq!(p.bindings(), vec!["x"]);

        // Wildcard binds nothing
        let p = Pattern::Wildcard;
        assert!(p.bindings().is_empty());

        // Literal binds nothing
        let p = Pattern::Literal(Value::Int(42));
        assert!(p.bindings().is_empty());

        // Tuple binds all nested patterns
        let p = Pattern::Tuple(vec![
            Pattern::Variable("a".to_string()),
            Pattern::Wildcard,
            Pattern::Variable("b".to_string()),
        ]);
        let mut bindings = p.bindings();
        bindings.sort();
        assert_eq!(bindings, vec!["a", "b"]);

        // Record binds nested patterns (not field names)
        let p = Pattern::Record(vec![
            ("field1".to_string(), Pattern::Variable("x".to_string())),
            ("field2".to_string(), Pattern::Wildcard),
            ("field3".to_string(), Pattern::Variable("y".to_string())),
        ]);
        let mut bindings = p.bindings();
        bindings.sort();
        assert_eq!(bindings, vec!["x", "y"]);

        // List with prefix patterns and rest binding
        let p = Pattern::List(
            vec![
                Pattern::Variable("first".to_string()),
                Pattern::Variable("second".to_string()),
            ],
            Some("rest".to_string()),
        );
        let mut bindings = p.bindings();
        bindings.sort();
        assert_eq!(bindings, vec!["first", "rest", "second"]);

        // List without rest binding
        let p = Pattern::List(vec![Pattern::Variable("head".to_string())], None);
        assert_eq!(p.bindings(), vec!["head"]);

        // Nested patterns
        let p = Pattern::Tuple(vec![
            Pattern::List(
                vec![Pattern::Variable("a".to_string())],
                Some("rest".to_string()),
            ),
            Pattern::Variable("b".to_string()),
        ]);
        let mut bindings = p.bindings();
        bindings.sort();
        assert_eq!(bindings, vec!["a", "b", "rest"]);
    }

    #[test]
    fn test_pattern_is_refutable() {
        // Variable is irrefutable (matches anything)
        assert!(!Pattern::Variable("x".to_string()).is_refutable());

        // Wildcard is irrefutable (matches anything)
        assert!(!Pattern::Wildcard.is_refutable());

        // Literal is refutable (only matches that specific value)
        assert!(Pattern::Literal(Value::Int(42)).is_refutable());

        // Tuple is refutable (needs matching structure)
        assert!(Pattern::Tuple(vec![Pattern::Variable("x".to_string())]).is_refutable());

        // Record is refutable (needs matching fields)
        assert!(
            Pattern::Record(vec![("a".to_string(), Pattern::Variable("x".to_string()))])
                .is_refutable()
        );

        // List is refutable (needs matching prefix)
        assert!(Pattern::List(vec![], None).is_refutable());
        assert!(
            Pattern::List(
                vec![Pattern::Variable("x".to_string())],
                Some("rest".to_string())
            )
            .is_refutable()
        );
    }

    #[test]
    fn test_serde_roundtrip() {
        // Create a complex workflow for testing
        let workflow = Workflow::Let {
            pattern: Pattern::Tuple(vec![
                Pattern::Variable("x".to_string()),
                Pattern::List(
                    vec![Pattern::Variable("head".to_string())],
                    Some("tail".to_string()),
                ),
            ]),
            expr: Expr::Call {
                func: "get_data".to_string(),
                arguments: vec![
                    Expr::Literal(Value::String("test".to_string())),
                    Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Literal(Value::Int(1))),
                        right: Box::new(Expr::Literal(Value::Int(2))),
                    },
                ],
            },
            continuation: Box::new(Workflow::If {
                condition: Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(Expr::Variable("x".to_string())),
                },
                then_branch: Box::new(Workflow::Ret {
                    expr: Expr::Literal(Value::Null),
                }),
                else_branch: Box::new(Workflow::Seq {
                    first: Box::new(Workflow::Done),
                    second: Box::new(Workflow::Par {
                        workflows: vec![Workflow::Done, Workflow::Done],
                    }),
                }),
            }),
        };

        // Serialize to JSON
        let serialized = serde_json::to_string(&workflow).expect("serialization should succeed");

        // Deserialize back
        let deserialized: Workflow =
            serde_json::from_str(&serialized).expect("deserialization should succeed");

        // Should be equal
        assert_eq!(workflow, deserialized);
    }

    #[test]
    fn test_serde_roundtrip_simple_workflows() {
        // Test Done
        let done = Workflow::Done;
        let json = serde_json::to_string(&done).unwrap();
        let recovered: Workflow = serde_json::from_str(&json).unwrap();
        assert_eq!(done, recovered);

        // Test Ret
        let ret = Workflow::Ret {
            expr: Expr::Literal(Value::Int(42)),
        };
        let json = serde_json::to_string(&ret).unwrap();
        let recovered: Workflow = serde_json::from_str(&json).unwrap();
        assert_eq!(ret, recovered);

        // Test Observe
        let observe = Workflow::Observe {
            capability: Capability {
                name: "test".to_string(),
                effect: Effect::Epistemic,
                constraints: vec![],
            },
            pattern: Pattern::Wildcard,
            continuation: Box::new(Workflow::Done),
        };
        let json = serde_json::to_string(&observe).unwrap();
        let recovered: Workflow = serde_json::from_str(&json).unwrap();
        assert_eq!(observe, recovered);
    }

    // ============================================================
    // TASK-120: AST Extensions for ADTs - Compilation Tests
    // These tests verify that the new ADT-related types exist
    // and can be constructed. They will fail to compile until
    // the types are implemented.
    // ============================================================

    #[test]
    fn test_pattern_variant_exists() {
        // Pattern::Variant should exist with name and optional fields
        let _variant_without_fields = Pattern::Variant {
            name: "Some".to_string(),
            fields: None,
        };

        let _variant_with_fields = Pattern::Variant {
            name: "Point".to_string(),
            fields: Some(vec![
                ("x".to_string(), Pattern::Variable("x".to_string())),
                ("y".to_string(), Pattern::Variable("y".to_string())),
            ]),
        };
    }

    #[test]
    fn test_expr_constructor_exists() {
        // Expr::Constructor should exist with name and fields
        let _constructor_without_fields = Expr::Constructor {
            name: "None".to_string(),
            fields: vec![],
        };

        let _constructor_with_fields = Expr::Constructor {
            name: "Some".to_string(),
            fields: vec![("value".to_string(), Expr::Literal(Value::Int(42)))],
        };

        let _constructor_multiple_fields = Expr::Constructor {
            name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Expr::Literal(Value::Int(0))),
                ("y".to_string(), Expr::Literal(Value::Int(0))),
            ],
        };
    }

    #[test]
    fn test_match_arm_struct_exists() {
        // MatchArm struct should exist with pattern and body
        let _arm = MatchArm {
            pattern: Pattern::Variable("x".to_string()),
            body: Expr::Literal(Value::Int(42)),
        };

        let _arm_with_variant = MatchArm {
            pattern: Pattern::Variant {
                name: "Some".to_string(),
                fields: Some(vec![(
                    "value".to_string(),
                    Pattern::Variable("v".to_string()),
                )]),
            },
            body: Expr::Variable("v".to_string()),
        };
    }

    #[test]
    fn test_expr_match_exists() {
        // Expr::Match should exist with scrutinee and arms
        let _match_expr = Expr::Match {
            scrutinee: Box::new(Expr::Variable("opt".to_string())),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Variant {
                        name: "Some".to_string(),
                        fields: Some(vec![(
                            "value".to_string(),
                            Pattern::Variable("v".to_string()),
                        )]),
                    },
                    body: Expr::Variable("v".to_string()),
                },
                MatchArm {
                    pattern: Pattern::Variant {
                        name: "None".to_string(),
                        fields: None,
                    },
                    body: Expr::Literal(Value::Int(0)),
                },
            ],
        };
    }

    #[test]
    fn test_expr_if_let_exists() {
        // Expr::IfLet should exist with pattern, expr, then_branch, and else_branch
        let _if_let = Expr::IfLet {
            pattern: Pattern::Variant {
                name: "Some".to_string(),
                fields: Some(vec![(
                    "value".to_string(),
                    Pattern::Variable("v".to_string()),
                )]),
            },
            expr: Box::new(Expr::Variable("opt".to_string())),
            then_branch: Box::new(Expr::Variable("v".to_string())),
            else_branch: Box::new(Expr::Literal(Value::Int(0))),
        };

        let _if_let_simple = Expr::IfLet {
            pattern: Pattern::Tuple(vec![
                Pattern::Variable("a".to_string()),
                Pattern::Variable("b".to_string()),
            ]),
            expr: Box::new(Expr::Variable("pair".to_string())),
            then_branch: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable("a".to_string())),
                right: Box::new(Expr::Variable("b".to_string())),
            }),
            else_branch: Box::new(Expr::Literal(Value::Int(0))),
        };
    }

    #[test]
    fn test_pattern_variant_bindings() {
        // Pattern::Variant should contribute bindings from its fields
        let pattern = Pattern::Variant {
            name: "Point".to_string(),
            fields: Some(vec![
                ("x".to_string(), Pattern::Variable("x_coord".to_string())),
                ("y".to_string(), Pattern::Variable("y_coord".to_string())),
            ]),
        };
        let mut bindings = pattern.bindings();
        bindings.sort();
        assert_eq!(bindings, vec!["x_coord", "y_coord"]);
    }

    #[test]
    fn test_pattern_variant_is_refutable() {
        // Pattern::Variant should be refutable
        let pattern = Pattern::Variant {
            name: "Some".to_string(),
            fields: Some(vec![(
                "value".to_string(),
                Pattern::Variable("v".to_string()),
            )]),
        };
        assert!(pattern.is_refutable());

        // Even without fields, it's refutable
        let pattern_no_fields = Pattern::Variant {
            name: "None".to_string(),
            fields: None,
        };
        assert!(pattern_no_fields.is_refutable());
    }

    #[test]
    fn test_match_arm_with_nested_patterns() {
        // MatchArm should work with complex nested patterns
        let _arm = MatchArm {
            pattern: Pattern::Variant {
                name: "Ok".to_string(),
                fields: Some(vec![(
                    "value".to_string(),
                    Pattern::Tuple(vec![
                        Pattern::Variable("a".to_string()),
                        Pattern::Variable("b".to_string()),
                    ]),
                )]),
            },
            body: Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable("a".to_string())),
                right: Box::new(Expr::Variable("b".to_string())),
            },
        };
    }

    #[test]
    fn test_expr_match_serde_roundtrip() {
        // Expr::Match should be serializable and deserializable
        let match_expr = Expr::Match {
            scrutinee: Box::new(Expr::Variable("x".to_string())),
            arms: vec![MatchArm {
                pattern: Pattern::Variable("y".to_string()),
                body: Expr::Variable("y".to_string()),
            }],
        };

        let serialized = serde_json::to_string(&match_expr).expect("serialization should succeed");
        let deserialized: Expr =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(match_expr, deserialized);
    }

    #[test]
    fn test_expr_if_let_serde_roundtrip() {
        // Expr::IfLet should be serializable and deserializable
        let if_let = Expr::IfLet {
            pattern: Pattern::Variable("x".to_string()),
            expr: Box::new(Expr::Literal(Value::Int(42))),
            then_branch: Box::new(Expr::Variable("x".to_string())),
            else_branch: Box::new(Expr::Literal(Value::Int(0))),
        };

        let serialized = serde_json::to_string(&if_let).expect("serialization should succeed");
        let deserialized: Expr =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(if_let, deserialized);
    }
}
