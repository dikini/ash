use ash_typeck::types::{Substitution, Type, TypeVar, unify};

#[test]
fn task788_simple_associated_type_substitution_updates_only_base_type() {
    let base = TypeVar(788);
    let associated = Type::Associated {
        interface: "Serializer".into(),
        base: Box::new(Type::Var(base)),
        name: "Ok".into(),
    };
    let mut substitution = Substitution::new();
    substitution.insert(base, Type::String);

    assert_eq!(
        substitution.apply(&associated),
        Type::Associated {
            interface: "Serializer".into(),
            base: Box::new(Type::String),
            name: "Ok".into(),
        }
    );
}

#[test]
fn task788_associated_projection_does_not_normalize_or_unify_with_concrete_type() {
    let associated = Type::Associated {
        interface: "Serializer".into(),
        base: Box::new(Type::String),
        name: "Ok".into(),
    };

    assert!(
        unify(&associated, &Type::String).is_err(),
        "associated identity metadata must not introduce projection normalization or definitional equality"
    );
}
