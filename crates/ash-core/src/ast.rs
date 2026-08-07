//! Core syntax tree types for Ash.

use crate::{Effect, Value};
use serde::{Deserialize, Serialize};

/// A workflow name
pub type Name = String;

// Re-export instance types from value module for use in AST
pub use crate::value::{ControlLink, Instance, InstanceAddr};

/// Type variable for generic types
pub type TypeVar = String;

/// Source span for AST nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Correlation ID for linking yield/resume pairs
///
/// Used to match responses from proxies to the original yield requests.
/// Each yield generates a unique correlation ID that must be included
/// in the corresponding resume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(pub u64);

impl CorrelationId {
    /// Create a new correlation ID from a numeric value
    #[must_use]
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Generate a new unique correlation ID
    #[must_use]
    pub fn generate() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// Parameter for workflow definitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: Name,
    pub ty: TypeExpr,
    pub span: Span,
}

/// Generic type parameter with canonical interface bounds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeParam {
    pub name: Name,
    pub bounds: Vec<InterfaceBound>,
    pub span: Span,
}

/// Canonical interface bound `T: Interface` preserved in the AST substrate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceBound {
    pub interface: Name,
    pub span: Span,
}

/// A capability reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capability {
    pub name: Name,
    pub effect: Effect,
    pub constraints: Vec<Constraint>,
}

/// Pattern for destructuring
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Variable {
        name: Name,
        span: Span,
    },
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
            Pattern::Variable { name, .. } => {
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
            Pattern::Variable { .. } | Pattern::Wildcard => false,
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
    Variable {
        name: Name,
        span: Span,
    },
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
        module: Option<Name>,
        arguments: Vec<Expr>,
    },

    /// Constructor expression: Some { value: 42 }
    Constructor {
        name: Name,
        fields: Vec<(Name, Expr)>,
    },

    /// Structural record expression.
    Record {
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

    /// Spawn expression: spawn entry_type with { init: args }
    /// Returns an Instance value containing addr and control link
    Spawn {
        entry_type: Name,
        init: Box<Expr>,
    },

    /// Split expression: split instance_expr
    /// Returns a tuple (InstanceAddr, `Option<ControlLink>`)
    Split(Box<Expr>),

    /// Check obligation expression: `check obligation_name`
    ///
    /// Linearly consumes an obligation that was previously created with `oblige`.
    /// Returns a boolean indicating whether the obligation was found and discharged.
    ///
    /// # Linear Semantics
    /// - First `check` returns `true` and removes the obligation
    /// - Subsequent `check` calls return `false` (obligation already consumed)
    /// - If obligation was never created, returns `false`
    ///
    /// # Example
    /// ```text
    /// // workflow example {
    /// //     oblige audit_trail;      // Creates obligation
    /// //     let ok = check audit_trail;  // Returns true, discharges
    /// //     let ok2 = check audit_trail; // Returns false, already discharged
    /// // }
    /// ```
    CheckObligation {
        /// Name of the obligation to check/discharge
        obligation: Name,
        /// Source span for error reporting
        span: Span,
    },

    /// Operational bottom: evaluate the payload and abort the current dynamic
    /// expression context with an operational failure.
    Fail {
        payload: Box<Expr>,
    },

    /// Scoped dynamic handler for operational failure raised by `Fail`.
    WithError {
        body: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    /// Anonymous function definition (closure creation). SPEC-031 §5.1
    FnDef {
        params: Vec<(String, Option<String>)>, // (name, optional type annotation)
        return_type: Option<String>,
        body: Box<Expr>,
    },

    /// Pure scope extension: evaluate `expr`, bind via `pattern`, evaluate `body`.
    ///
    /// This is the expression-level let-binding.
    /// which is the imperative/monadic form with continuation semantics.
    /// `Expr::Let` composes two pure computations by scope extension:
    /// the bound expression is evaluated, the pattern is matched (irrefutably
    /// for well-typed programs), and the body is evaluated in the extended
    /// environment.
    ///
    /// Surface syntax: `let <pattern> = <expr>; <body>` inside fn bodies.
    /// This is semantically `let <pattern> = <expr> in <body>` — expression
    /// composition, not imperative sequencing.
    Let {
        pattern: Pattern,
        expr: Box<Expr>,
        body: Box<Expr>,
        /// Source span for error reporting (pattern match failure diagnostics).
        span: Span,
    },

    /// Function application. SPEC-031 §5.4
    FnApply {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
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
    Mod,
    And,
    Or,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    /// `in` - Membership test
    In,
    /// `|>` - Pipe operator
    Pipe,
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

/// Top-level module item
///
/// A module can contain capabilities, types, interfaces, and executable declarations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModuleItem {
    /// Capability definition
    Capability(Capability),
    /// Type definition
    Type(TypeDef),
    /// Interface definition
    Interface(InterfaceDef),
    /// Interface impl definition
    Impl(ImplDef),
    /// Builtin function definition (runtime-provided, no Ash-level body)
    BuiltinFn(BuiltinFnDef),
}

/// Builtin function definition (runtime-provided, no Ash-level body).
///
/// Represents a callable registration with a type signature but no body
/// expression -- dispatch happens at runtime via the host environment.
/// Lowered from `surface::BuiltinFnDef`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuiltinFnDef {
    /// Function name
    pub name: Name,
    /// Generic type parameters
    pub type_params: Vec<Name>,
    /// Function parameters with name and type
    pub params: Vec<(Name, TypeExpr)>,
    /// Return type (required for builtins)
    pub return_type: TypeExpr,
    /// Visibility
    pub visibility: Visibility,
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
    /// Whether the type is declared as a runtime/engine managed builtin substrate.
    pub builtin: bool,
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
    /// Explicit payload shape for the variant
    pub payload: VariantPayload,
}

/// Explicit payload shape for enum variants in source/core metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VariantPayload {
    /// Unit variant with no payload
    Unit,
    /// Record variant with named fields
    Record(Vec<(Name, TypeExpr)>),
    /// Tuple variant with positional items
    Tuple(Vec<TypeExpr>),
}

/// Visibility modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// Type constructor application (e.g., `Option<Int>`)
    Constructor { name: Name, args: Vec<TypeExpr> },
    /// Tuple type (e.g., (Int, String))
    Tuple(Vec<TypeExpr>),
    /// Record type (e.g., { x: Int, y: String })
    Record(Vec<(Name, TypeExpr)>),
    /// Associated type projection (e.g., `S::Ok`, `Map<K,V>::Entry`)
    Associated { base: Box<TypeExpr>, name: Name },
}

/// Associated type declaration in an interface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssociatedType {
    pub name: Name,
}

/// Associated type binding in an impl block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssociatedTypeBinding {
    pub name: Name,
    pub ty: TypeExpr,
}

/// Canonical core where bound `T: Interface`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhereBound {
    pub param: Name,
    pub bound: Name,
}

/// Interface-owned evidence constraint preserved from an interface `where` tail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceEvidenceConstraint {
    pub subject: TypeExpr,
    pub required_evidence: TypeExpr,
}

/// Interface definition in source/core metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceDef {
    pub name: Name,
    pub type_params: Vec<TypeVar>,
    pub evidence_constraints: Vec<InterfaceEvidenceConstraint>,
    pub associated_types: Vec<AssociatedType>,
    pub methods: Vec<InterfaceMethodSig>,
    pub visibility: Visibility,
}

/// Interface method signature preserved in the AST substrate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceMethodSig {
    pub name: Name,
    pub params: Vec<TypeExpr>,
    pub return_type: TypeExpr,
}

/// Explicit interface implementation preserved for later coherence/resolution work.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImplDef {
    pub visibility: Visibility,
    pub interface: Name,
    pub type_params: Vec<TypeVar>,
    pub type_args: Vec<TypeExpr>,
    pub where_bounds: Vec<WhereBound>,
    pub associated_type_bindings: Vec<AssociatedTypeBinding>,
    pub methods: Vec<ImplMethodDef>,
}

/// Method body preserved inside an impl block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImplMethodDef {
    pub name: Name,
    pub params: Vec<Name>,
    pub body: Expr,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_bindings() {
        // Variable binds one name
        let p = Pattern::Variable {
            name: "x".to_string(),
            span: crate::ast::Span::default(),
        };
        assert_eq!(p.bindings(), vec!["x"]);

        // Wildcard binds nothing
        let p = Pattern::Wildcard;
        assert!(p.bindings().is_empty());

        // Literal binds nothing
        let p = Pattern::Literal(Value::Int(42));
        assert!(p.bindings().is_empty());

        // Tuple binds all nested patterns
        let p = Pattern::Tuple(vec![
            Pattern::Variable {
                name: "a".to_string(),
                span: crate::ast::Span::default(),
            },
            Pattern::Wildcard,
            Pattern::Variable {
                name: "b".to_string(),
                span: crate::ast::Span::default(),
            },
        ]);
        let mut bindings = p.bindings();
        bindings.sort();
        assert_eq!(bindings, vec!["a", "b"]);

        // Record binds nested patterns (not field names)
        let p = Pattern::Record(vec![
            (
                "field1".to_string(),
                Pattern::Variable {
                    name: "x".to_string(),
                    span: crate::ast::Span::default(),
                },
            ),
            ("field2".to_string(), Pattern::Wildcard),
            (
                "field3".to_string(),
                Pattern::Variable {
                    name: "y".to_string(),
                    span: crate::ast::Span::default(),
                },
            ),
        ]);
        let mut bindings = p.bindings();
        bindings.sort();
        assert_eq!(bindings, vec!["x", "y"]);

        // List with prefix patterns and rest binding
        let p = Pattern::List(
            vec![
                Pattern::Variable {
                    name: "first".to_string(),
                    span: crate::ast::Span::default(),
                },
                Pattern::Variable {
                    name: "second".to_string(),
                    span: crate::ast::Span::default(),
                },
            ],
            Some("rest".to_string()),
        );
        let mut bindings = p.bindings();
        bindings.sort();
        assert_eq!(bindings, vec!["first", "rest", "second"]);

        // List without rest binding
        let p = Pattern::List(
            vec![Pattern::Variable {
                name: "head".to_string(),
                span: crate::ast::Span::default(),
            }],
            None,
        );
        assert_eq!(p.bindings(), vec!["head"]);

        // Nested patterns
        let p = Pattern::Tuple(vec![
            Pattern::List(
                vec![Pattern::Variable {
                    name: "a".to_string(),
                    span: crate::ast::Span::default(),
                }],
                Some("rest".to_string()),
            ),
            Pattern::Variable {
                name: "b".to_string(),
                span: crate::ast::Span::default(),
            },
        ]);
        let mut bindings = p.bindings();
        bindings.sort();
        assert_eq!(bindings, vec!["a", "b", "rest"]);
    }

    #[test]
    fn test_pattern_is_refutable() {
        // Variable is irrefutable (matches anything)
        assert!(
            !Pattern::Variable {
                name: "x".to_string(),
                span: crate::ast::Span::default()
            }
            .is_refutable()
        );

        // Wildcard is irrefutable (matches anything)
        assert!(!Pattern::Wildcard.is_refutable());

        // Literal is refutable (only matches that specific value)
        assert!(Pattern::Literal(Value::Int(42)).is_refutable());

        // Tuple is refutable (needs matching structure)
        assert!(
            Pattern::Tuple(vec![Pattern::Variable {
                name: "x".to_string(),
                span: crate::ast::Span::default()
            }])
            .is_refutable()
        );

        // Record is refutable (needs matching fields)
        assert!(
            Pattern::Record(vec![(
                "a".to_string(),
                Pattern::Variable {
                    name: "x".to_string(),
                    span: crate::ast::Span::default()
                }
            )])
            .is_refutable()
        );

        // List is refutable (needs matching prefix)
        assert!(Pattern::List(vec![], None).is_refutable());
        assert!(
            Pattern::List(
                vec![Pattern::Variable {
                    name: "x".to_string(),
                    span: crate::ast::Span::default()
                }],
                Some("rest".to_string())
            )
            .is_refutable()
        );
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
                (
                    "x".to_string(),
                    Pattern::Variable {
                        name: "x".to_string(),
                        span: crate::ast::Span::default(),
                    },
                ),
                (
                    "y".to_string(),
                    Pattern::Variable {
                        name: "y".to_string(),
                        span: crate::ast::Span::default(),
                    },
                ),
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
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: crate::ast::Span::default(),
            },
            body: Expr::Literal(Value::Int(42)),
        };

        let _arm_with_variant = MatchArm {
            pattern: Pattern::Variant {
                name: "Some".to_string(),
                fields: Some(vec![(
                    "value".to_string(),
                    Pattern::Variable {
                        name: "v".to_string(),
                        span: crate::ast::Span::default(),
                    },
                )]),
            },
            body: Expr::Variable {
                name: "v".to_string(),
                span: crate::ast::Span::default(),
            },
        };
    }

    #[test]
    fn test_expr_match_exists() {
        // Expr::Match should exist with scrutinee and arms
        let _match_expr = Expr::Match {
            scrutinee: Box::new(Expr::Variable {
                name: "opt".to_string(),
                span: crate::ast::Span::default(),
            }),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Variant {
                        name: "Some".to_string(),
                        fields: Some(vec![(
                            "value".to_string(),
                            Pattern::Variable {
                                name: "v".to_string(),
                                span: crate::ast::Span::default(),
                            },
                        )]),
                    },
                    body: Expr::Variable {
                        name: "v".to_string(),
                        span: crate::ast::Span::default(),
                    },
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
                    Pattern::Variable {
                        name: "v".to_string(),
                        span: crate::ast::Span::default(),
                    },
                )]),
            },
            expr: Box::new(Expr::Variable {
                name: "opt".to_string(),
                span: crate::ast::Span::default(),
            }),
            then_branch: Box::new(Expr::Variable {
                name: "v".to_string(),
                span: crate::ast::Span::default(),
            }),
            else_branch: Box::new(Expr::Literal(Value::Int(0))),
        };

        let _if_let_simple = Expr::IfLet {
            pattern: Pattern::Tuple(vec![
                Pattern::Variable {
                    name: "a".to_string(),
                    span: crate::ast::Span::default(),
                },
                Pattern::Variable {
                    name: "b".to_string(),
                    span: crate::ast::Span::default(),
                },
            ]),
            expr: Box::new(Expr::Variable {
                name: "pair".to_string(),
                span: crate::ast::Span::default(),
            }),
            then_branch: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable {
                    name: "a".to_string(),
                    span: crate::ast::Span::default(),
                }),
                right: Box::new(Expr::Variable {
                    name: "b".to_string(),
                    span: crate::ast::Span::default(),
                }),
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
                (
                    "x".to_string(),
                    Pattern::Variable {
                        name: "x_coord".to_string(),
                        span: crate::ast::Span::default(),
                    },
                ),
                (
                    "y".to_string(),
                    Pattern::Variable {
                        name: "y_coord".to_string(),
                        span: crate::ast::Span::default(),
                    },
                ),
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
                Pattern::Variable {
                    name: "v".to_string(),
                    span: crate::ast::Span::default(),
                },
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
                        Pattern::Variable {
                            name: "a".to_string(),
                            span: crate::ast::Span::default(),
                        },
                        Pattern::Variable {
                            name: "b".to_string(),
                            span: crate::ast::Span::default(),
                        },
                    ]),
                )]),
            },
            body: Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable {
                    name: "a".to_string(),
                    span: crate::ast::Span::default(),
                }),
                right: Box::new(Expr::Variable {
                    name: "b".to_string(),
                    span: crate::ast::Span::default(),
                }),
            },
        };
    }

    #[test]
    fn test_expr_match_serde_roundtrip() {
        // Expr::Match should be serializable and deserializable
        let match_expr = Expr::Match {
            scrutinee: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: crate::ast::Span::default(),
            }),
            arms: vec![MatchArm {
                pattern: Pattern::Variable {
                    name: "y".to_string(),
                    span: crate::ast::Span::default(),
                },
                body: Expr::Variable {
                    name: "y".to_string(),
                    span: crate::ast::Span::default(),
                },
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
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: crate::ast::Span::default(),
            },
            expr: Box::new(Expr::Literal(Value::Int(42))),
            then_branch: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: crate::ast::Span::default(),
            }),
            else_branch: Box::new(Expr::Literal(Value::Int(0))),
        };

        let serialized = serde_json::to_string(&if_let).expect("serialization should succeed");
        let deserialized: Expr =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(if_let, deserialized);
    }
}
