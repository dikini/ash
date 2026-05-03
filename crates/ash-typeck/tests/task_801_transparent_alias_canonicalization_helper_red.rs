use ash_core::ast::{TypeBody, TypeDef, TypeExpr, Visibility};
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv};

fn register_identity_alias(env: &mut TypeEnv) {
    env.register_type(&TypeDef {
        name: "Identity".into(),
        params: vec!["T".into()],
        body: TypeBody::Alias(TypeExpr::Named("T".into())),
        visibility: Visibility::Public,
        builtin: false,
    })
    .expect("test precondition: Identity alias should register");
}

fn register_user_id_alias(env: &mut TypeEnv) {
    env.register_type(&TypeDef {
        name: "UserId".into(),
        params: vec![],
        body: TypeBody::Alias(TypeExpr::Named("String".into())),
        visibility: Visibility::Public,
        builtin: false,
    })
    .expect("test precondition: UserId alias should register");
}

#[test]
fn task801_helper_canonicalizes_transparent_aliases_recursively_without_changing_other_structure() {
    let mut env = TypeEnv::with_builtin_types();
    register_identity_alias(&mut env);
    register_user_id_alias(&mut env);

    let aliased = Type::Record(vec![
        (
            "user".into(),
            Type::Constructor {
                name: QualifiedName::root("UserId"),
                args: vec![],
                kind: Kind::Type,
            },
        ),
        (
            "friends".into(),
            Type::List(Box::new(Type::Constructor {
                name: QualifiedName::root("Identity"),
                args: vec![Type::Constructor {
                    name: QualifiedName::root("UserId"),
                    args: vec![],
                    kind: Kind::Type,
                }],
                kind: Kind::Type,
            })),
        ),
    ]);

    let canonical = env.canonicalize_transparent_aliases(&aliased);

    assert_eq!(
        canonical,
        Type::Record(vec![
            ("user".into(), Type::String),
            ("friends".into(), Type::List(Box::new(Type::String))),
        ]),
        "TASK-801 should provide a helper that recursively peels transparent aliases down to canonical forms without requiring every caller to manually probe alias targets"
    );
}

#[test]
fn task801_helper_renders_alias_name_for_diagnostics_while_canonicalizing_under_the_hood() {
    let mut env = TypeEnv::with_builtin_types();
    register_user_id_alias(&mut env);

    let alias = Type::Constructor {
        name: QualifiedName::root("UserId"),
        args: vec![],
        kind: Kind::Type,
    };

    assert_eq!(
        env.render_type_for_diagnostics(&alias),
        "UserId",
        "TASK-801 should preserve the user-facing alias spelling when callers hand diagnostics the source-visible alias form"
    );

    assert_eq!(env.canonicalize_transparent_aliases(&alias), Type::String);
}
