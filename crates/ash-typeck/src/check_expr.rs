//! Expression type checking
//!
//! Provides type checking for expressions, including constructor expressions.

use crate::error::ConstructorError;
use crate::exhaustiveness::{Coverage, check_exhaustive};
use crate::type_env::{TypeEnv, TypeInfo, VariantIndex};
use crate::types::{Substitution, Type, TypeVar, unify};
use ash_core::ast::{Pattern as CorePattern, TypeBody, TypeDef};
use ash_parser::lower_pattern;
use ash_parser::surface::{BinaryOp, Expr, Literal, MatchArm, Pattern as SurfacePattern, UnaryOp};
use std::collections::HashSet;

/// Result of type checking an expression
#[derive(Debug, Clone, PartialEq)]
pub struct CheckResult {
    /// The inferred type of the expression
    pub ty: Type,
    /// Any substitutions generated during checking
    pub substitution: Substitution,
    /// Any errors encountered
    pub errors: Vec<ConstructorError>,
}

impl CheckResult {
    /// Create a successful check result
    pub fn success(ty: Type) -> Self {
        Self {
            ty,
            substitution: Substitution::new(),
            errors: Vec::new(),
        }
    }

    /// Create a check result with an error
    pub fn error(err: ConstructorError) -> Self {
        Self {
            ty: Type::Var(TypeVar::fresh()),
            substitution: Substitution::new(),
            errors: vec![err],
        }
    }

    /// Check if the result is successful (no errors)
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Type check an expression
///
/// This function infers the type of an expression and returns the result
/// along with any substitutions and errors.
pub fn check_expr(env: &TypeEnv, expr: &Expr) -> CheckResult {
    match expr {
        Expr::Literal(lit) => check_literal(lit),
        Expr::Variable(_) => {
            // Variables get a fresh type variable (they should be looked up in context)
            CheckResult::success(Type::Var(TypeVar::fresh()))
        }
        Expr::Unary { op, operand, .. } => check_unary(env, *op, operand),
        Expr::Binary {
            op, left, right, ..
        } => check_binary(env, *op, left, right),
        Expr::Constructor { name, fields, .. } => check_constructor(env, name.as_ref(), fields),
        Expr::IfLet {
            then_branch,
            else_branch,
            ..
        } => {
            let then_result = check_expr(env, then_branch);
            let else_result = check_expr(env, else_branch);
            merge_branch_results(then_result, else_result)
        }
        Expr::Match {
            scrutinee, arms, ..
        } => check_match(env, scrutinee, arms),
        _ => {
            // For other expression types, return a fresh type variable
            // These should be implemented as needed
            CheckResult::success(Type::Var(TypeVar::fresh()))
        }
    }
}

/// Check a literal expression
fn check_literal(lit: &Literal) -> CheckResult {
    let ty = match lit {
        Literal::Int(_) => Type::Int,
        Literal::String(_) => Type::String,
        Literal::Bool(_) => Type::Bool,
        Literal::Null => Type::Null,
        Literal::Float(_) => {
            // Floats not yet supported in core types, use fresh variable
            Type::Var(TypeVar::fresh())
        }
        Literal::List(items) => infer_list_literal_type(items),
    };
    CheckResult::success(ty)
}

fn infer_list_literal_type(items: &[Literal]) -> Type {
    let mut iter = items.iter();
    let Some(first) = iter.next() else {
        return Type::List(Box::new(Type::Var(TypeVar::fresh())));
    };

    let first_ty = check_literal(first).ty;
    if iter.all(|item| check_literal(item).ty == first_ty) {
        Type::List(Box::new(first_ty))
    } else {
        Type::List(Box::new(Type::Var(TypeVar::fresh())))
    }
}

fn check_unary(env: &TypeEnv, op: UnaryOp, operand: &Expr) -> CheckResult {
    let operand_result = check_expr(env, operand);
    if !operand_result.is_ok() {
        return operand_result;
    }

    let ty = match op {
        UnaryOp::Not if operand_result.ty == Type::Bool => Type::Bool,
        UnaryOp::Neg if operand_result.ty == Type::Int => Type::Int,
        _ => Type::Var(TypeVar::fresh()),
    };

    CheckResult {
        ty,
        substitution: operand_result.substitution,
        errors: operand_result.errors,
    }
}

fn check_binary(env: &TypeEnv, op: BinaryOp, left: &Expr, right: &Expr) -> CheckResult {
    let left_result = check_expr(env, left);
    let right_result = check_expr(env, right);

    if !left_result.is_ok() || !right_result.is_ok() {
        return CheckResult {
            ty: Type::Var(TypeVar::fresh()),
            substitution: left_result.substitution.compose(&right_result.substitution),
            errors: left_result
                .errors
                .into_iter()
                .chain(right_result.errors)
                .collect(),
        };
    }

    let ty = match op {
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div
            if left_result.ty == Type::Int && right_result.ty == Type::Int =>
        {
            Type::Int
        }
        BinaryOp::And | BinaryOp::Or
            if left_result.ty == Type::Bool && right_result.ty == Type::Bool =>
        {
            Type::Bool
        }
        BinaryOp::Eq
        | BinaryOp::Neq
        | BinaryOp::Lt
        | BinaryOp::Gt
        | BinaryOp::Leq
        | BinaryOp::Geq
            if left_result.ty == right_result.ty =>
        {
            Type::Bool
        }
        BinaryOp::In => Type::Bool,
        _ => Type::Var(TypeVar::fresh()),
    };

    CheckResult {
        ty,
        substitution: left_result.substitution.compose(&right_result.substitution),
        errors: Vec::new(),
    }
}

fn merge_branch_results(left: CheckResult, right: CheckResult) -> CheckResult {
    let substitution = left.substitution.compose(&right.substitution);
    let errors: Vec<ConstructorError> = left.errors.into_iter().chain(right.errors).collect();

    if !errors.is_empty() {
        return CheckResult {
            ty: Type::Var(TypeVar::fresh()),
            substitution,
            errors,
        };
    }

    let ty = unify(&left.ty, &right.ty)
        .map(|subst| subst.apply(&left.ty))
        .unwrap_or(Type::Var(TypeVar::fresh()));

    CheckResult {
        ty,
        substitution,
        errors: Vec::new(),
    }
}

fn resolve_enum_type_def_for_match<'a>(
    env: &'a TypeEnv,
    scrutinee: &Expr,
    arms: &[MatchArm],
) -> Option<&'a TypeDef> {
    if let Expr::Constructor { name, .. } = scrutinee
        && let Some((type_name, _)) = env.lookup_constructor(name.as_ref())
        && let Some(def) = env.lookup_type(type_name.as_str())
        && matches!(&def.body, TypeBody::Enum(_))
    {
        return Some(def);
    }
    for arm in arms {
        if let SurfacePattern::Variant { name, .. } = &arm.pattern
            && let Some((type_name, _)) = env.lookup_constructor(name.as_ref())
            && let Some(def) = env.lookup_type(type_name.as_str())
            && matches!(&def.body, TypeBody::Enum(_))
        {
            return Some(def);
        }
    }
    None
}

fn format_missing_witnesses(witnesses: &[CorePattern]) -> String {
    witnesses
        .iter()
        .map(|p| match p {
            CorePattern::Variant { name, .. } => name.clone(),
            _ => format!("{p:?}"),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn check_match(env: &TypeEnv, scrutinee: &Expr, arms: &[MatchArm]) -> CheckResult {
    let scrutinee_result = check_expr(env, scrutinee);
    let mut errors: Vec<ConstructorError> = scrutinee_result.errors.clone();

    if let Some(type_def) = resolve_enum_type_def_for_match(env, scrutinee, arms) {
        let patterns: Vec<CorePattern> =
            arms.iter().map(|arm| lower_pattern(&arm.pattern)).collect();
        if let Coverage::Missing(witnesses) = check_exhaustive(&patterns, type_def) {
            errors.push(ConstructorError::NonExhaustiveMatch {
                scrutinee_type: type_def.name.clone(),
                missing: format_missing_witnesses(&witnesses),
            });
        }
    }

    let mut arm_merged: Option<CheckResult> = None;
    for arm in arms {
        let body_result = check_expr(env, &arm.body);
        arm_merged = Some(match arm_merged {
            None => body_result,
            Some(prev) => merge_branch_results(prev, body_result),
        });
    }

    let arm_merged =
        arm_merged.unwrap_or_else(|| CheckResult::success(Type::Var(TypeVar::fresh())));

    let substitution = scrutinee_result
        .substitution
        .compose(&arm_merged.substitution);

    errors.extend(arm_merged.errors);

    CheckResult {
        ty: arm_merged.ty,
        substitution,
        errors,
    }
}

/// Check a constructor expression
///
/// Validates that:
/// 1. The constructor name is known
/// 2. All required fields are present
/// 3. No unknown fields are provided
/// 4. Field types match the expected types
fn check_constructor(
    env: &TypeEnv,
    constructor_name: &str,
    fields: &[(Box<str>, Expr)],
) -> CheckResult {
    // Look up the constructor
    let (type_def, variant_idx, variant_def) = match env.get_variant(constructor_name) {
        Some(result) => result,
        None => {
            return CheckResult::error(ConstructorError::UnknownConstructor(
                constructor_name.to_string(),
            ));
        }
    };

    let mut errors = Vec::new();
    let mut substitution = Substitution::new();

    // Get the expected fields for this variant
    let expected_fields: HashSet<&str> = variant_def
        .fields
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();

    // Get the provided field names
    let provided_fields: HashSet<&str> = fields.iter().map(|(name, _)| name.as_ref()).collect();

    // Check for missing fields
    for expected in &expected_fields {
        if !provided_fields.contains(*expected) {
            errors.push(ConstructorError::MissingField {
                constructor: constructor_name.to_string(),
                field: expected.to_string(),
            });
        }
    }

    // Check for unknown fields
    for provided in &provided_fields {
        if !expected_fields.contains(*provided) {
            errors.push(ConstructorError::UnknownField {
                constructor: constructor_name.to_string(),
                field: provided.to_string(),
            });
        }
    }

    // Create a mapping from field name to expected type
    let expected_types: std::collections::HashMap<&str, &Type> = variant_def
        .fields
        .iter()
        .map(|(name, ty)| (name.as_str(), ty))
        .collect();

    // Check field types
    for (field_name, field_expr) in fields {
        if let Some(expected_ty) = expected_types.get(field_name.as_ref()) {
            let field_result = check_expr(env, field_expr);

            // Collect any errors from the field expression
            errors.extend(field_result.errors);

            // Try to unify the field type with the expected type
            let expected_ty_subst = substitution.apply(expected_ty);
            match unify(&expected_ty_subst, &field_result.ty) {
                Ok(sub) => {
                    substitution = substitution.compose(&sub);
                }
                Err(_) => {
                    errors.push(ConstructorError::FieldTypeMismatch {
                        constructor: constructor_name.to_string(),
                        field: field_name.to_string(),
                        expected: expected_ty.to_string(),
                        actual: field_result.ty.to_string(),
                    });
                }
            }
        }
    }

    // Build the result type
    let result_type = build_constructor_type(type_def, variant_idx);

    CheckResult {
        ty: substitution.apply(&result_type),
        substitution,
        errors,
    }
}

/// Build the type for a constructor expression
///
/// For a variant of a generic type, this returns the type constructor
/// with the appropriate type variables.
fn build_constructor_type(type_info: &TypeInfo, _variant_idx: VariantIndex) -> Type {
    use crate::kind::Kind;
    use crate::qualified_name::QualifiedName;

    match type_info {
        TypeInfo::Enum { name, params, .. } => {
            // Build Option<T>, not just T
            Type::Constructor {
                name: QualifiedName::root(name.clone()),
                args: params.iter().map(|p| Type::Var(*p)).collect(),
                kind: Kind::Type,
            }
        }
        TypeInfo::Struct { name, params, .. } => Type::Constructor {
            name: QualifiedName::root(name.clone()),
            args: params.iter().map(|p| Type::Var(*p)).collect(),
            kind: Kind::Type,
        },
    }
}

/// Type check an expression and return the inferred type
///
/// This is a convenience function that returns just the type,
/// discarding errors and substitutions.
pub fn infer_type(env: &TypeEnv, expr: &Expr) -> Type {
    let result = check_expr(env, expr);
    result.substitution.apply(&result.ty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::surface::Literal;
    use ash_parser::token::Span;

    // ============================================================
    // Literal Tests
    // ============================================================

    #[test]
    fn test_check_literal_int() {
        let env = TypeEnv::with_builtin_types();
        let expr = Expr::Literal(Literal::Int(42));
        let result = check_expr(&env, &expr);

        assert!(result.is_ok());
        assert_eq!(result.ty, Type::Int);
    }

    #[test]
    fn test_check_literal_string() {
        let env = TypeEnv::with_builtin_types();
        let expr = Expr::Literal(Literal::String("hello".into()));
        let result = check_expr(&env, &expr);

        assert!(result.is_ok());
        assert_eq!(result.ty, Type::String);
    }

    #[test]
    fn test_check_literal_bool() {
        let env = TypeEnv::with_builtin_types();
        let expr = Expr::Literal(Literal::Bool(true));
        let result = check_expr(&env, &expr);

        assert!(result.is_ok());
        assert_eq!(result.ty, Type::Bool);
    }

    #[test]
    fn test_check_literal_null() {
        let env = TypeEnv::with_builtin_types();
        let expr = Expr::Literal(Literal::Null);
        let result = check_expr(&env, &expr);

        assert!(result.is_ok());
        assert_eq!(result.ty, Type::Null);
    }

    // ============================================================
    // Constructor Tests - Some { value: 42 }
    // ============================================================

    #[test]
    fn test_check_constructor_some_with_value() {
        let env = TypeEnv::with_builtin_types();

        // Some { value: 42 }
        let expr = Expr::Constructor {
            name: "Some".into(),
            fields: vec![("value".into(), Expr::Literal(Literal::Int(42)))],
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(
            result.is_ok(),
            "Expected success, got errors: {:?}",
            result.errors
        );
        // Constructor returns Option<T>, not just T
        match &result.ty {
            Type::Constructor { name, .. } => {
                assert_eq!(name.to_string(), "Option");
            }
            _ => panic!("Expected constructor type, got {:?}", result.ty),
        }
    }

    #[test]
    fn test_check_constructor_none() {
        let env = TypeEnv::with_builtin_types();

        // None { }
        let expr = Expr::Constructor {
            name: "None".into(),
            fields: vec![],
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(
            result.is_ok(),
            "Expected success, got errors: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_check_constructor_unknown() {
        let env = TypeEnv::with_builtin_types();

        // Unknown { value: 42 }
        let expr = Expr::Constructor {
            name: "Unknown".into(),
            fields: vec![("value".into(), Expr::Literal(Literal::Int(42)))],
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(!result.is_ok());
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            result.errors[0],
            ConstructorError::UnknownConstructor(_)
        ));
    }

    #[test]
    fn test_check_constructor_missing_field() {
        let env = TypeEnv::with_builtin_types();

        // Some { } - missing required 'value' field
        let expr = Expr::Constructor {
            name: "Some".into(),
            fields: vec![],
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(!result.is_ok());
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ConstructorError::MissingField { constructor, field }
            if constructor == "Some" && field == "value"
        )));
    }

    #[test]
    fn test_check_constructor_unknown_field() {
        let env = TypeEnv::with_builtin_types();

        // Some { value: 42, extra: "bad" }
        let expr = Expr::Constructor {
            name: "Some".into(),
            fields: vec![
                ("value".into(), Expr::Literal(Literal::Int(42))),
                ("extra".into(), Expr::Literal(Literal::String("bad".into()))),
            ],
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(!result.is_ok());
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ConstructorError::UnknownField { constructor, field }
            if constructor == "Some" && field == "extra"
        )));
    }

    // ============================================================
    // Constructor Tests - Result
    // ============================================================

    #[test]
    fn test_check_constructor_ok() {
        let env = TypeEnv::with_builtin_types();

        // Ok { value: 42 }
        let expr = Expr::Constructor {
            name: "Ok".into(),
            fields: vec![("value".into(), Expr::Literal(Literal::Int(42)))],
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(
            result.is_ok(),
            "Expected success, got errors: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_check_constructor_err() {
        let env = TypeEnv::with_builtin_types();

        // Err { error: "message" }
        let expr = Expr::Constructor {
            name: "Err".into(),
            fields: vec![(
                "error".into(),
                Expr::Literal(Literal::String("message".into())),
            )],
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(
            result.is_ok(),
            "Expected success, got errors: {:?}",
            result.errors
        );
    }

    // ============================================================
    // Match Exhaustiveness Tests (TASK-130 RED)
    // ============================================================

    #[test]
    fn test_match_non_exhaustive_option_reports_error() {
        let env = TypeEnv::with_builtin_types();

        // match None { Some { value: x } => x }
        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Constructor {
                name: "None".into(),
                fields: vec![],
                span: Span::default(),
            }),
            arms: vec![ash_parser::surface::MatchArm {
                pattern: ash_parser::surface::Pattern::Variant {
                    name: "Some".into(),
                    fields: Some(vec![(
                        "value".into(),
                        ash_parser::surface::Pattern::Variable("x".into()),
                    )]),
                },
                body: Box::new(Expr::Variable("x".into())),
                span: Span::default(),
            }],
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(
            !result.is_ok(),
            "Expected non-exhaustive match to report an error"
        );
        assert!(
            result
                .errors
                .iter()
                .any(|err| err.to_string().contains("non-exhaustive")),
            "Expected a non-exhaustive match error, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_match_exhaustive_option_is_accepted() {
        let env = TypeEnv::with_builtin_types();

        // match None {
        //   Some { value: x } => x,
        //   None => 0
        // }
        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Constructor {
                name: "None".into(),
                fields: vec![],
                span: Span::default(),
            }),
            arms: vec![
                ash_parser::surface::MatchArm {
                    pattern: ash_parser::surface::Pattern::Variant {
                        name: "Some".into(),
                        fields: Some(vec![(
                            "value".into(),
                            ash_parser::surface::Pattern::Variable("x".into()),
                        )]),
                    },
                    body: Box::new(Expr::Variable("x".into())),
                    span: Span::default(),
                },
                ash_parser::surface::MatchArm {
                    pattern: ash_parser::surface::Pattern::Variant {
                        name: "None".into(),
                        fields: None,
                    },
                    body: Box::new(Expr::Literal(Literal::Int(0))),
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(
            result.is_ok(),
            "Expected exhaustive match to type check, got errors: {:?}",
            result.errors
        );
    }

    // ============================================================
    // infer_type Tests
    // ============================================================

    #[test]
    fn test_infer_type_literal() {
        let env = TypeEnv::with_builtin_types();
        let expr = Expr::Literal(Literal::Int(42));

        let ty = infer_type(&env, &expr);
        assert_eq!(ty, Type::Int);
    }

    #[test]
    fn test_infer_type_constructor_some() {
        let env = TypeEnv::with_builtin_types();
        let expr = Expr::Constructor {
            name: "Some".into(),
            fields: vec![("value".into(), Expr::Literal(Literal::Int(42)))],
            span: Span::default(),
        };

        let ty = infer_type(&env, &expr);
        // Constructor returns Option<T>, not just T
        match &ty {
            Type::Constructor { name, .. } => {
                assert_eq!(name.to_string(), "Option");
            }
            _ => panic!("Expected constructor type, got {:?}", ty),
        }
    }

    #[test]
    fn constructor_returns_constructor_type() {
        let env = TypeEnv::with_builtin_types();

        // Some { value: 42 } should have type Option<Int>, not Int
        let expr = Expr::Constructor {
            name: "Some".into(),
            fields: vec![("value".into(), Expr::Literal(Literal::Int(42)))],
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        // Should be Option<Int>
        match result.ty {
            Type::Constructor { name, .. } => {
                assert_eq!(name.to_string(), "Option");
            }
            _ => panic!("Expected constructor type, got {:?}", result.ty),
        }
    }

    // ============================================================
    // CheckResult Tests
    // ============================================================

    #[test]
    fn test_check_result_success() {
        let result = CheckResult::success(Type::Int);
        assert!(result.is_ok());
        assert_eq!(result.ty, Type::Int);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_check_result_error() {
        let err = ConstructorError::UnknownConstructor("Foo".to_string());
        let result = CheckResult::error(err.clone());
        assert!(!result.is_ok());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0], err);
    }
}
