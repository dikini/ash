use ash_typeck::{
    PublicComputationManifestKind, PublicComputationOperationAuthority,
    PublicComputationOperationRole, TypeEnv,
};

#[test]
fn public_computation_manifest_exposes_current_computation_algebras() {
    let env = TypeEnv::with_builtin_types();
    let manifest = env.public_computation_manifest();

    let name = "Result<_, E>";
    let entry = manifest
        .algebra(name)
        .unwrap_or_else(|| panic!("{name} algebra entry is missing"));
    assert_eq!(entry.kind, PublicComputationManifestKind::Monad);
    assert!(
        entry.nameable,
        "{name} must be public/nameable construction algebra"
    );
    assert!(
        entry.typeable,
        "{name} must be public/typeable construction algebra"
    );
    assert!(
        manifest.algebra("Workflow").is_none(),
        "Workflow must not remain a public computation algebra"
    );
    for retired in ["Act", "Proc"] {
        assert!(
            manifest.algebra(retired).is_none(),
            "{retired} must not remain a public computation algebra"
        );
    }

    let p = manifest
        .algebra("P")
        .expect("P process handle entry is missing");
    assert_eq!(p.kind, PublicComputationManifestKind::ProcessHandle);
    assert!(p.nameable);
    assert!(p.typeable);
    assert!(
        !p.user_constructible,
        "P<A> is nameable/typeable but not a user construction API"
    );

    for op in [
        "contract::requires",
        "contract::ensures",
        "Ok",
        "result::and_then",
    ] {
        let operation = manifest
            .operation(op)
            .unwrap_or_else(|| panic!("{op} manifest operation is missing"));
        assert!(operation.nameable, "{op} must be nameable");
        assert!(operation.typeable, "{op} must be typeable");
    }
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
        "workflow::from_act",
        "application::unit",
        "application::bind",
    ] {
        assert!(
            manifest.operation(op).is_none(),
            "{op} must not remain in the public computation manifest"
        );
    }
    for op in ["Ok", "result::and_then"] {
        let operation = manifest
            .operation(op)
            .unwrap_or_else(|| panic!("{op} manifest operation is missing"));
        assert_eq!(
            operation.authority,
            PublicComputationOperationAuthority::VisibleAlgebra,
            "{op} must be requested by visible public algebra"
        );
    }

    for op in [
        "act::unit",
        "act::bind",
        "proc::unit",
        "proc::bind",
        "proc::from_act",
        "proc::yield",
    ] {
        assert!(
            env.lookup_variable(op).is_none(),
            "{op} must not remain TypeEnv-visible"
        );
    }
    for op in [
        "workflow::unit",
        "workflow::bind",
        "workflow::from_proc",
        "workflow::from_act",
        "application::unit",
        "application::bind",
    ] {
        assert!(
            env.lookup_variable(op).is_none(),
            "{op} must not remain TypeEnv-visible"
        );
    }
    assert!(
        env.lookup_contract_intrinsic("contract::requires")
            .is_some()
    );
    assert!(env.lookup_contract_intrinsic("contract::ensures").is_some());
}

#[test]
fn visible_intrinsic_mapping_has_no_hidden_unrelated_do_magic() {
    let manifest = TypeEnv::with_builtin_types().public_computation_manifest();

    for operation in manifest.operations() {
        assert!(
            operation.intrinsic.visible_operation == operation.name,
            "{} maps through an unrelated hidden intrinsic root {}",
            operation.name,
            operation.intrinsic.visible_operation
        );
        if operation.name.starts_with("contract::") {
            assert_eq!(
                operation.authority,
                PublicComputationOperationAuthority::HiddenSemanticRoot,
                "{} must be compiler-owned contract evidence, not visible algebra",
                operation.name
            );
        } else {
            assert_ne!(
                operation.authority,
                PublicComputationOperationAuthority::HiddenSemanticRoot,
                "{} must not introduce hidden semantic root authority",
                operation.name
            );
        }
    }

    let lift_operations: Vec<_> = manifest
        .operations()
        .iter()
        .filter(|operation| operation.role == PublicComputationOperationRole::ExplicitLift)
        .map(|operation| operation.name)
        .collect();
    assert!(lift_operations.is_empty());
}

#[test]
fn public_computation_builtin_signatures_use_fresh_type_variables() {
    let env = TypeEnv::with_builtin_types();
    let names = ["result::and_then"];

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
