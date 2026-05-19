use ash_typeck::{
    PublicTowerIntrinsicKind, PublicTowerManifestKind, PublicTowerOperationAuthority,
    PublicTowerOperationRole, TypeEnv,
};

#[test]
fn public_tower_manifest_exposes_act_proc_workflow_result_algebra() {
    let env = TypeEnv::with_builtin_types();
    let manifest = env.public_tower_manifest();

    for name in ["Act", "Proc", "Workflow", "Result<_, E>"] {
        let entry = manifest
            .algebra(name)
            .unwrap_or_else(|| panic!("{name} algebra entry is missing"));
        assert_eq!(entry.kind, PublicTowerManifestKind::Monad);
        assert!(
            entry.nameable,
            "{name} must be public/nameable construction algebra"
        );
        assert!(
            entry.typeable,
            "{name} must be public/typeable construction algebra"
        );
    }

    let p = manifest
        .algebra("P")
        .expect("P process handle entry is missing");
    assert_eq!(p.kind, PublicTowerManifestKind::ProcessHandle);
    assert!(p.nameable);
    assert!(p.typeable);
    assert!(
        !p.user_constructible,
        "P<A> is nameable/typeable but not a user construction API"
    );

    for op in [
        "act::unit",
        "act::bind",
        "proc::unit",
        "proc::bind",
        "proc::from_act",
        "proc::par",
        "proc::await",
        "workflow::unit",
        "workflow::bind",
        "workflow::from_proc",
        "workflow::requires",
        "workflow::ensures",
        "Ok",
        "result::and_then",
    ] {
        let operation = manifest
            .operation(op)
            .unwrap_or_else(|| panic!("{op} manifest operation is missing"));
        assert_eq!(
            operation.authority,
            PublicTowerOperationAuthority::VisibleAlgebra,
            "{op} must be requested by visible public algebra"
        );
        assert!(operation.nameable, "{op} must be nameable");
        assert!(operation.typeable, "{op} must be typeable");
    }

    for op in [
        "act::unit",
        "act::bind",
        "proc::unit",
        "proc::bind",
        "proc::from_act",
        "workflow::unit",
        "workflow::bind",
        "workflow::from_proc",
    ] {
        assert!(
            env.lookup_variable(op).is_some(),
            "{op} must be TypeEnv-visible"
        );
    }
    assert!(
        env.lookup_workflow_intrinsic("workflow::requires")
            .is_some()
    );
    assert!(env.lookup_workflow_intrinsic("workflow::ensures").is_some());
}

#[test]
fn visible_intrinsic_mapping_has_no_hidden_unrelated_do_magic() {
    let manifest = TypeEnv::with_builtin_types().public_tower_manifest();

    let act_bind = manifest
        .operation("act::bind")
        .expect("act::bind manifest operation is missing");
    assert_eq!(act_bind.role, PublicTowerOperationRole::Bind);
    assert_eq!(
        act_bind.intrinsic.kind,
        PublicTowerIntrinsicKind::CompilerPreludeEvidence
    );
    assert_eq!(act_bind.intrinsic.visible_operation, "act::bind");
    assert_eq!(
        act_bind.authority,
        PublicTowerOperationAuthority::VisibleAlgebra
    );

    for operation in manifest.operations() {
        assert!(
            operation.intrinsic.visible_operation == operation.name,
            "{} maps through an unrelated hidden intrinsic root {}",
            operation.name,
            operation.intrinsic.visible_operation
        );
        assert_ne!(
            operation.authority,
            PublicTowerOperationAuthority::HiddenSemanticRoot,
            "{} must not introduce hidden semantic root authority",
            operation.name
        );
    }

    let lift_operations: Vec<_> = manifest
        .operations()
        .iter()
        .filter(|operation| operation.role == PublicTowerOperationRole::ExplicitLift)
        .map(|operation| operation.name)
        .collect();
    assert_eq!(
        lift_operations,
        vec![
            "proc::from_act",
            "workflow::from_proc",
            "workflow::from_act"
        ],
        "D5 requires explicit visible tower lifts only"
    );
}

#[test]
fn public_tower_builtin_signatures_use_fresh_type_variables() {
    let env = TypeEnv::with_builtin_types();
    let names = [
        "act::unit",
        "act::bind",
        "workflow::unit",
        "workflow::bind",
        "result::and_then",
    ];

    let mut vars = Vec::new();
    for name in names {
        let ty = env
            .lookup_variable(name)
            .unwrap_or_else(|| panic!("{name} signature is missing"));
        collect_type_vars(&ty, &mut vars);
    }

    assert!(
        vars.iter().all(|var| var.0 > 2),
        "new TASK-921 builtin signatures must allocate fresh type variables, not fixed low IDs: {vars:?}"
    );
}

fn collect_type_vars(ty: &ash_typeck::Type, vars: &mut Vec<ash_typeck::TypeVar>) {
    match ty {
        ash_typeck::Type::Var(var) => vars.push(*var),
        ash_typeck::Type::Fn(params, ret) => {
            for param in params {
                collect_type_vars(param, vars);
            }
            collect_type_vars(ret, vars);
        }
        ash_typeck::Type::Fun(params, ret, _) => {
            for param in params {
                collect_type_vars(param, vars);
            }
            collect_type_vars(ret, vars);
        }
        ash_typeck::Type::List(inner) => collect_type_vars(inner, vars),
        ash_typeck::Type::Record(fields) => {
            for (_, field) in fields {
                collect_type_vars(field, vars);
            }
        }
        ash_typeck::Type::Constructor { args, .. }
        | ash_typeck::Type::ConstructorVariableApp { args, .. } => {
            for arg in args {
                collect_type_vars(arg, vars);
            }
        }
        ash_typeck::Type::Associated { base, .. } => collect_type_vars(base, vars),
        ash_typeck::Type::Int
        | ash_typeck::Type::String
        | ash_typeck::Type::Bool
        | ash_typeck::Type::Float
        | ash_typeck::Type::Null
        | ash_typeck::Type::Time
        | ash_typeck::Type::Ref
        | ash_typeck::Type::Cap { .. }
        | ash_typeck::Type::Instance { .. }
        | ash_typeck::Type::InstanceAddr { .. }
        | ash_typeck::Type::ControlLink { .. } => {}
    }
}
