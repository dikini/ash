use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_parser::surface::{BuiltinFnDef, Expr, FnDef, Literal, Type as SurfaceType};
use ash_parser::token::Span;

fn prop_type_expr() -> TypeExpr {
    TypeExpr::Named("Prop".to_string())
}

fn prop_surface_type() -> SurfaceType {
    SurfaceType::Name("Prop".into())
}

fn err_string<E: std::fmt::Display>(result: Result<impl std::fmt::Debug, E>) -> String {
    result
        .expect_err("expected Prop runtime escape rejection")
        .to_string()
}

fn proof_alias_def() -> TypeDef {
    TypeDef {
        name: "Proof".to_string(),
        params: vec![],
        body: TypeBody::Alias(prop_type_expr()),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn identity_alias_def() -> TypeDef {
    TypeDef {
        name: "Id".to_string(),
        params: vec!["T".to_string()],
        body: TypeBody::Alias(TypeExpr::Named("T".to_string())),
        visibility: Visibility::Public,
        builtin: false,
    }
}

#[test]
fn rejects_function_returning_prop() {
    let env = ash_typeck::TypeEnv::with_builtin_types();
    let function = FnDef {
        visibility: ash_parser::surface::Visibility::Public,
        name: "foo".into(),
        type_params: vec![],
        params: vec![],
        return_type: Some(prop_surface_type()),
        proposition_tail: None,
        contract: None,
        body: Expr::Literal(Literal::Bool(true)),
        span: Span::default(),
    };

    let err = err_string(ash_typeck::fn_signature_type(&env, &function));

    assert!(
        err.contains("Prop") && err.contains("runtime") && err.contains("return"),
        "expected runtime Prop return diagnostic, got: {err}"
    );
}

#[test]
fn rejects_function_returning_prop_alias() {
    let mut env = ash_typeck::TypeEnv::with_builtin_types();
    env.register_type(&proof_alias_def())
        .expect("alias-to-Prop type declaration should register as transparent alias");
    let function = FnDef {
        visibility: ash_parser::surface::Visibility::Public,
        name: "foo_alias".into(),
        type_params: vec![],
        params: vec![],
        return_type: Some(SurfaceType::Name("Proof".into())),
        proposition_tail: None,
        contract: None,
        body: Expr::Literal(Literal::Bool(true)),
        span: Span::default(),
    };

    let err = err_string(ash_typeck::fn_signature_type(&env, &function));

    assert!(
        err.contains("Prop") && err.contains("runtime") && err.contains("return"),
        "expected runtime Prop return diagnostic through alias, got: {err}"
    );
}

#[test]
fn rejects_builtin_function_returning_prop() {
    let env = ash_typeck::TypeEnv::with_builtin_types();
    let builtin_fn = BuiltinFnDef {
        visibility: ash_parser::surface::Visibility::Public,
        name: "builtin_proof".into(),
        type_params: vec![],
        params: vec![],
        return_type: prop_surface_type(),
        proposition_tail: None,
        span: Span::default(),
    };

    let err = err_string(ash_typeck::builtin_fn_signature_type(&env, &builtin_fn));

    assert!(
        err.contains("Prop") && err.contains("runtime") && err.contains("builtin function return"),
        "expected runtime Prop builtin-function-return diagnostic, got: {err}"
    );
}

#[test]
fn rejects_builtin_function_returning_prop_alias() {
    let mut env = ash_typeck::TypeEnv::with_builtin_types();
    env.register_type(&proof_alias_def())
        .expect("alias-to-Prop type declaration should register as transparent alias");
    let builtin_fn = BuiltinFnDef {
        visibility: ash_parser::surface::Visibility::Public,
        name: "builtin_proof_alias".into(),
        type_params: vec![],
        params: vec![],
        return_type: SurfaceType::Name("Proof".into()),
        proposition_tail: None,
        span: Span::default(),
    };

    let err = err_string(ash_typeck::builtin_fn_signature_type(&env, &builtin_fn));

    assert!(
        err.contains("Prop") && err.contains("runtime") && err.contains("builtin function return"),
        "expected runtime Prop builtin-function-return diagnostic through alias, got: {err}"
    );
}

#[test]
fn rejects_prop_in_struct_field() {
    let mut env = ash_typeck::TypeEnv::with_builtin_types();
    let def = TypeDef {
        name: "BadStruct".to_string(),
        params: vec![],
        body: TypeBody::Struct(vec![("proof".to_string(), prop_type_expr())]),
        visibility: Visibility::Public,
        builtin: false,
    };

    let err = err_string(env.register_type(&def));

    assert!(
        err.contains("Prop") && err.contains("runtime") && err.contains("struct field"),
        "expected runtime Prop struct-field diagnostic, got: {err}"
    );
}

#[test]
fn rejects_prop_alias_in_struct_field() {
    let mut env = ash_typeck::TypeEnv::with_builtin_types();
    env.register_type(&proof_alias_def())
        .expect("alias-to-Prop type declaration should register as transparent alias");
    let def = TypeDef {
        name: "BadAliasStruct".to_string(),
        params: vec![],
        body: TypeBody::Struct(vec![(
            "proof".to_string(),
            TypeExpr::Named("Proof".to_string()),
        )]),
        visibility: Visibility::Public,
        builtin: false,
    };

    let err = err_string(env.register_type(&def));

    assert!(
        err.contains("Prop") && err.contains("runtime") && err.contains("struct field"),
        "expected runtime Prop struct-field diagnostic through alias, got: {err}"
    );
}

#[test]
fn rejects_prop_generic_alias_in_struct_field() {
    let mut env = ash_typeck::TypeEnv::with_builtin_types();
    env.register_type(&identity_alias_def())
        .expect("generic transparent identity alias should register");
    let def = TypeDef {
        name: "BadGenericAliasStruct".to_string(),
        params: vec![],
        body: TypeBody::Struct(vec![(
            "proof".to_string(),
            TypeExpr::Constructor {
                name: "Id".to_string(),
                args: vec![prop_type_expr()],
            },
        )]),
        visibility: Visibility::Public,
        builtin: false,
    };

    let err = err_string(env.register_type(&def));

    assert!(
        err.contains("Prop") && err.contains("runtime") && err.contains("struct field"),
        "expected runtime Prop struct-field diagnostic through generic alias, got: {err}"
    );
}

#[test]
fn rejects_prop_in_enum_variant() {
    let mut env = ash_typeck::TypeEnv::with_builtin_types();
    let def = TypeDef {
        name: "BadEnum".to_string(),
        params: vec![],
        body: TypeBody::Enum(vec![VariantDef {
            name: "HasProof".to_string(),
            fields: vec![("proof".to_string(), prop_type_expr())],
            payload: VariantPayload::Record(vec![("proof".to_string(), prop_type_expr())]),
        }]),
        visibility: Visibility::Public,
        builtin: false,
    };

    let err = err_string(env.register_type(&def));

    assert!(
        err.contains("Prop") && err.contains("runtime") && err.contains("enum variant"),
        "expected runtime Prop enum-variant diagnostic, got: {err}"
    );
}

#[test]
fn rejects_prop_alias_in_enum_variant() {
    let mut env = ash_typeck::TypeEnv::with_builtin_types();
    env.register_type(&proof_alias_def())
        .expect("alias-to-Prop type declaration should register as transparent alias");
    let def = TypeDef {
        name: "BadAliasEnum".to_string(),
        params: vec![],
        body: TypeBody::Enum(vec![VariantDef {
            name: "HasProof".to_string(),
            fields: vec![("proof".to_string(), TypeExpr::Named("Proof".to_string()))],
            payload: VariantPayload::Record(vec![(
                "proof".to_string(),
                TypeExpr::Named("Proof".to_string()),
            )]),
        }]),
        visibility: Visibility::Public,
        builtin: false,
    };

    let err = err_string(env.register_type(&def));

    assert!(
        err.contains("Prop") && err.contains("runtime") && err.contains("enum variant"),
        "expected runtime Prop enum-variant diagnostic through alias, got: {err}"
    );
}
