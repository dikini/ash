use ash_parser::surface::{
    Definition, Expr, FnDef, Literal, ModuleFile, Program, ProgramEntry, ProofDef, Visibility,
};
use ash_parser::token::Span;

fn parse_module(source: &str) -> ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn program_from_module(module: ModuleFile) -> Program {
    let mut definitions = module.definitions;
    definitions.push(Definition::Function(FnDef {
        visibility: Visibility::Inherited,
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        return_type: None,
        proposition_tail: None,
        contract: None,
        body: Expr::Literal(Literal::Null),
        span: Span::default(),
    }));
    Program {
        definitions,
        entry: ProgramEntry {
            function: "main".into(),
            span: Span::default(),
        },
    }
}

fn first_module_proof(source: &str) -> ProofDef {
    parse_module(source)
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Proof(proof) => Some(proof),
            _ => None,
        })
        .expect("source should contain a module-scoped proof")
}

fn first_impl_proof(source: &str) -> ProofDef {
    parse_module(source)
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Impl(implementation) => implementation.proofs.into_iter().next(),
            _ => None,
        })
        .expect("source should contain an impl-scoped proof")
}

#[test]
fn totality_stub_accepts_module_proof_bodies_directly() {
    let proof = first_module_proof(
        r#"
        law reflexive(x: Int): x == x
        proof reflexive(x: Int) {
            loop_forever(x)
        }
        "#,
    );

    ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality(&proof)
        .expect("TASK-1367 totality stub should accept every module proof body for now");
}

#[test]
fn totality_stub_accepts_impl_proof_bodies_directly() {
    let proof = first_impl_proof(
        r#"
        interface Eq<A> {
            equiv(A, A) -> Bool
            law reflexive(x: A): equiv(x, x)
        }

        impl Eq<Int> {
            equiv(a, b) = a == b
            proof reflexive(x: Int) {
                by test "non_total_external_oracle"
            }
        }
        "#,
    );

    ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality(&proof)
        .expect("TASK-1367 totality stub should accept every impl proof body for now");
}

#[test]
fn typecheck_program_runs_totality_stub_without_rejecting_proof_bodies() {
    let module = parse_module(
        r#"
        law reflexive(x: Int): x == x
        proof reflexive(x: Int) {
            loop_forever(x)
        }
        "#,
    );

    ash_typeck::type_check_program(&program_from_module(module))
        .expect("proof totality stub should not reject proof bodies during typechecking");
}
