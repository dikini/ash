use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{ModuleIdentity, ModuleSourceOrigin, SourceAnchor, SourceOrigin};
use ash_core::type_ir::{
    CanonicalTypeExpr, InterfaceBoundProposition, PropositionDeferredKind, PropositionEvidenceRule,
    PropositionOutcome, TypeProposition, TypePropositionTerm,
};
use ash_parser::surface::{
    Definition, ImplDef, InterfaceDef, PropositionClause, PropositionClauseKind, PropositionTail,
};
use ash_parser::token::Span;
use ash_typeck::type_env::{PropositionCheckingSite, PropositionCheckingSiteKind};
use ash_typeck::{Kind, Type, TypeEnv, TypeVar};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn interface_named(module: &ash_parser::surface::ModuleFile, name: &str) -> InterfaceDef {
    module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) if interface.name.as_ref() == name => {
                Some(interface.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("interface {name} should be present"))
}

fn impl_named(module: &ash_parser::surface::ModuleFile, name: &str) -> ImplDef {
    module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Impl(implementation) if implementation.interface.as_ref() == name => {
                Some(implementation.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("impl {name} should be present"))
}

fn module_identity(id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(1)),
        ModuleId(id),
        vec!["task_1043".to_string(), format!("m{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: "task-1043".to_string(),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-1043".to_string(),
        },
        None,
        label,
    )
}

fn type_var_term(var: TypeVar) -> TypePropositionTerm {
    TypePropositionTerm::Canonical(CanonicalTypeExpr::Var(format!("type_var_{}", var.0)))
}

fn interface_bound(env: &TypeEnv, var: TypeVar, interface: &str) -> TypeProposition {
    TypeProposition::InterfaceBound(InterfaceBoundProposition {
        subject: type_var_term(var),
        interface: env
            .interface_identity_for_name(interface)
            .unwrap_or_else(|| panic!("{interface} identity should be registered"))
            .clone(),
        interface_args: vec![],
    })
}

fn env_with_monad_requires_applicative() -> TypeEnv {
    let module = parse(
        r#"
        interface Applicative<F : * -> *> {}
        interface Monad<M : * -> *> where M: Applicative {}
        "#,
    );
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module_identity(1_043_001));
    env.register_interface(&interface_named(&module, "Applicative"))
        .expect("Applicative interface should register");
    env.register_interface(&interface_named(&module, "Monad"))
        .expect("Monad interface should register");
    env
}

#[test]
fn constrained_generic_bound_entails_required_interface_bound_proposition() {
    let mut env = env_with_monad_requires_applicative();
    let var = TypeVar(1043);
    env.bind_type_var_interface_bound(var, "Monad");

    let proposition = interface_bound(&env, var, "Applicative");
    let outcome = env
        .solve_proposition(
            &proposition,
            Some(anchor("M: Applicative required by M: Monad")),
        )
        .expect("proposition solving should not error");

    match outcome {
        PropositionOutcome::Satisfied(evidence) => {
            assert_eq!(evidence.proposition, proposition);
            assert_eq!(
                evidence.rule,
                PropositionEvidenceRule::InScopeInterfaceBound
            );
        }
        other => panic!("expected entailed required evidence, got {other:?}"),
    }
}

#[test]
fn required_generic_bound_does_not_entail_constrained_interface_bound() {
    let mut env = env_with_monad_requires_applicative();
    let var = TypeVar(1044);
    env.bind_type_var_interface_bound(var, "Applicative");

    let proposition = interface_bound(&env, var, "Monad");
    let outcome = env
        .solve_proposition(
            &proposition,
            Some(anchor("M: Monad is not required by M: Applicative")),
        )
        .expect("proposition solving should not error");

    match outcome {
        PropositionOutcome::Deferred(reason) => {
            assert_eq!(reason.proposition, proposition);
            assert_eq!(
                reason.kind,
                PropositionDeferredKind::MissingInterfaceEvidence
            );
            assert!(reason.no_inversion_boundary);
        }
        other => panic!("reverse entailment must remain rejected, got {other:?}"),
    }
}

#[test]
fn impl_where_bound_entails_required_interface_bound_proposition() {
    let module = parse(
        r#"
        interface Weak<T> {}
        interface Strong<T> where T: Weak {}
        interface Uses<T> {}
        impl <T> Uses<T> where T: Strong {}
        "#,
    );
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module_identity(1_043_004));
    env.register_interface(&interface_named(&module, "Weak"))
        .expect("Weak interface should register");
    env.register_interface(&interface_named(&module, "Strong"))
        .expect("Strong interface should register");
    env.register_interface(&interface_named(&module, "Uses"))
        .expect("Uses interface should register");
    env.register_impl(&impl_named(&module, "Uses"))
        .expect("generic Uses<T> impl should register");

    let where_var = env.impl_schemes()[0].where_bounds[0].type_var;
    let proposition = interface_bound(&env, where_var, "Weak");
    let outcome = env
        .solve_proposition(
            &proposition,
            Some(anchor("impl where T: Strong entails T: Weak")),
        )
        .expect("proposition solving should not error");

    match outcome {
        PropositionOutcome::Satisfied(evidence) => {
            assert_eq!(evidence.proposition, proposition);
            assert_eq!(
                evidence.rule,
                PropositionEvidenceRule::InScopeInterfaceBound
            );
        }
        other => panic!("expected impl where-bound required evidence, got {other:?}"),
    }
}

#[test]
fn constrained_interface_argument_entails_required_argument_evidence() {
    let module = parse(
        r#"
        interface Eq<K> {}
        interface RichMap<M, K, V> where K: Eq {}
        "#,
    );
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module_identity(1_043_007));
    env.register_interface(&interface_named(&module, "Eq"))
        .expect("Eq interface should register");
    env.register_interface(&interface_named(&module, "RichMap"))
        .expect("RichMap interface should register");
    for param in ["M", "K", "V"] {
        env.register_type_parameter_kind(param, Kind::Type)
            .expect("type parameter kind should register");
    }

    let tail = PropositionTail {
        clauses: vec![PropositionClause {
            kind: PropositionClauseKind::InterfaceBound {
                subject: ash_parser::surface::Type::Name("M".into()),
                interface: ash_parser::surface::Type::Constructor {
                    name: "RichMap".into(),
                    args: vec![
                        ash_parser::surface::Type::Name("K".into()),
                        ash_parser::surface::Type::Name("V".into()),
                    ],
                },
                colon_span: Span::default(),
            },
            span: Span::default(),
        }],
        where_span: Span::default(),
        span: Span::default(),
    };
    env.add_proposition_assumptions_from_tail(
        &tail,
        SourceOrigin::Synthetic {
            reason: "task-1043-richmap-assumption".to_string(),
        },
        PropositionCheckingSite::new(
            1_043_007,
            PropositionCheckingSiteKind::ImplWhereBound,
            Some("M: RichMap<K, V> assumption".to_string()),
        ),
    )
    .expect("RichMap proposition assumption should lower");

    let proposition = TypeProposition::InterfaceBound(InterfaceBoundProposition {
        subject: TypePropositionTerm::Canonical(CanonicalTypeExpr::Var("K".to_string())),
        interface: env
            .interface_identity_for_name("Eq")
            .expect("Eq identity should be registered")
            .clone(),
        interface_args: vec![],
    });
    let outcome = env
        .solve_proposition(&proposition, Some(anchor("M: RichMap<K, V> entails K: Eq")))
        .expect("proposition solving should not error");

    match outcome {
        PropositionOutcome::Satisfied(evidence) => {
            assert_eq!(evidence.proposition, proposition);
            assert_eq!(
                evidence.rule,
                PropositionEvidenceRule::InScopeInterfaceBound
            );
        }
        other => panic!("expected substituted argument evidence, got {other:?}"),
    }
}

#[test]
fn generic_bound_fallback_does_not_resolve_zero_parameter_interface_methods() {
    let module = parse(
        r#"
        interface Clock {
            now() -> Time
        }
        "#,
    );
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module_identity(1_043_008));
    env.register_interface(&interface_named(&module, "Clock"))
        .expect("Clock interface should register");

    let err = env
        .resolve_interface_method_call("Clock", "now", &[])
        .expect_err("zero-parameter interface methods require concrete evidence");
    let message = err.to_string();
    assert!(message.contains("Clock"), "{message}");
}

#[test]
fn generic_method_lookup_substitutes_constrained_interface_arguments() {
    let module = parse(
        r#"
        interface Eq<K> {
            eq(K, K) -> Bool
        }
        interface RichMap<M, K, V> where K: Eq {}
        "#,
    );
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module_identity(1_043_009));
    env.register_interface(&interface_named(&module, "Eq"))
        .expect("Eq interface should register");
    env.register_interface(&interface_named(&module, "RichMap"))
        .expect("RichMap interface should register");

    let map_var = TypeVar(3_001);
    let key_var = TypeVar(3_002);
    let value_var = TypeVar(3_003);
    let tail = PropositionTail {
        clauses: vec![PropositionClause {
            kind: PropositionClauseKind::InterfaceBound {
                subject: ash_parser::surface::Type::Name(format!("type_var_{}", map_var.0).into()),
                interface: ash_parser::surface::Type::Constructor {
                    name: "RichMap".into(),
                    args: vec![
                        ash_parser::surface::Type::Name(format!("type_var_{}", key_var.0).into()),
                        ash_parser::surface::Type::Name(format!("type_var_{}", value_var.0).into()),
                    ],
                },
                colon_span: Span::default(),
            },
            span: Span::default(),
        }],
        where_span: Span::default(),
        span: Span::default(),
    };
    env.add_proposition_assumptions_from_tail(
        &tail,
        SourceOrigin::Synthetic {
            reason: "task-1043-richmap-method-assumption".to_string(),
        },
        PropositionCheckingSite::new(
            1_043_009,
            PropositionCheckingSiteKind::ImplWhereBound,
            Some("type_var_3001: RichMap<type_var_3002, type_var_3003>".to_string()),
        ),
    )
    .expect("RichMap proposition assumption should lower");

    let resolved = env
        .resolve_interface_method_call("Eq", "eq", &[Type::Var(key_var), Type::Var(key_var)])
        .expect("M: RichMap<K, V> should make required K: Eq method lookup available");
    assert_eq!(resolved, Type::Bool);

    let err = env
        .resolve_interface_method_call("Eq", "eq", &[Type::Var(map_var), Type::Var(map_var)])
        .expect_err("M: RichMap<K, V> must not make M: Eq available");
    let message = err.to_string();
    assert!(message.contains("Eq"), "{message}");
}

#[test]
fn generic_impl_must_discharge_required_evidence_with_where_bound() {
    let missing = parse(
        r#"
        interface Weak<T> {}
        interface Strong<T> where T: Weak {}
        impl <T> Strong<T> {}
        "#,
    );
    let mut missing_env = TypeEnv::with_builtin_types();
    missing_env.set_current_module_identity(module_identity(1_043_005));
    missing_env
        .register_interface(&interface_named(&missing, "Weak"))
        .expect("Weak interface should register");
    missing_env
        .register_interface(&interface_named(&missing, "Strong"))
        .expect("Strong interface should register");

    let err = missing_env
        .register_impl(&impl_named(&missing, "Strong"))
        .expect_err("generic Strong<T> impl must prove required Weak<T> evidence");
    let message = err.to_string();
    assert!(message.contains("Strong"), "{message}");
    assert!(message.contains("Weak"), "{message}");
    assert!(missing_env.impl_schemes().is_empty());

    let discharged = parse(
        r#"
        interface Weak<T> {}
        interface Strong<T> where T: Weak {}
        impl <T> Strong<T> where T: Weak {}
        "#,
    );
    let mut discharged_env = TypeEnv::with_builtin_types();
    discharged_env.set_current_module_identity(module_identity(1_043_006));
    discharged_env
        .register_interface(&interface_named(&discharged, "Weak"))
        .expect("Weak interface should register");
    discharged_env
        .register_interface(&interface_named(&discharged, "Strong"))
        .expect("Strong interface should register");
    discharged_env
        .register_impl(&impl_named(&discharged, "Strong"))
        .expect("where T: Weak should discharge Strong<T>'s required evidence");
    assert_eq!(discharged_env.impl_schemes().len(), 1);
}

fn env_with_strong_requires_weak_method() -> TypeEnv {
    let module = parse(
        r#"
        interface Weak<T> {
            weak_id(T) -> T
        }

        interface Strong<T> where T: Weak {
            strong_id(T) -> T
        }
        "#,
    );
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module_identity(1_043_002));
    env.register_interface(&interface_named(&module, "Weak"))
        .expect("Weak interface should register");
    env.register_interface(&interface_named(&module, "Strong"))
        .expect("Strong interface should register");
    env
}

#[test]
fn constrained_generic_bound_satisfies_required_interface_method_lookup() {
    let mut env = env_with_strong_requires_weak_method();
    let var = TypeVar(2043);
    env.register_type_parameter_kind("M", Kind::Type)
        .expect("generic type parameter kind should register");
    env.bind_type_var_interface_bound(var, "Strong");

    let resolved = env
        .resolve_interface_method_call("Weak", "weak_id", &[Type::Var(var)])
        .expect(
            "M: Strong should make required M: Weak evidence available in generic method lookup",
        );

    assert_eq!(resolved, Type::Var(var));
}

#[test]
fn required_generic_bound_does_not_satisfy_constrained_interface_method_lookup() {
    let mut env = env_with_strong_requires_weak_method();
    let var = TypeVar(2044);
    env.register_type_parameter_kind("M", Kind::Type)
        .expect("generic type parameter kind should register");
    env.bind_type_var_interface_bound(var, "Weak");

    let err = env
        .resolve_interface_method_call("Strong", "strong_id", &[Type::Var(var)])
        .expect_err("M: Weak must not satisfy M: Strong method lookup");
    let message = err.to_string();

    assert!(message.contains("Strong"), "{message}");
}

#[test]
fn generic_entailment_does_not_create_concrete_reverse_evidence() {
    let module = parse(
        r#"
        interface Applicative<F : * -> *> {}
        interface Monad<M : * -> *> where M: Applicative {}
        impl Applicative<Option> {}
        "#,
    );
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module_identity(1_043_003));
    env.register_interface(&interface_named(&module, "Applicative"))
        .expect("Applicative interface should register");
    env.register_interface(&interface_named(&module, "Monad"))
        .expect("Monad interface should register");

    let implementation = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Impl(implementation) => Some(implementation.clone()),
            _ => None,
        })
        .expect("Applicative<Option> impl should be present");
    env.register_impl(&implementation)
        .expect("Applicative<Option> evidence should register");

    let err = env
        .resolve_interface_evidence("Monad", &[ash_parser::surface::Type::Name("Option".into())])
        .expect_err("Applicative<Option> must not derive concrete Monad<Option> evidence");
    let message = err.to_string();

    assert!(message.contains("Monad"), "{message}");
    assert!(message.contains("Option"), "{message}");
}
