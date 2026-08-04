//! TASK-2075 RED contracts for collection-visible declaration carriers.
//!
//! Module-scope roles, laws, and proofs retain the existing visibility grammar and
//! declaration-wide spans. Policy has no active module declaration grammar, so its contract here
//! is construction-only. Interface laws and impl proofs remain inherited, parent-scoped members.

use ash_parser::surface::{
    Definition, Expr, LawDef, Literal, PolicyDef, ProofBody, ProofDef, RoleDef, Visibility,
};
use ash_parser::{Span, parse_surface_file};

#[derive(Clone, Copy, Debug)]
enum ModuleEvidenceKind {
    Role,
    Law,
    Proof,
}

impl ModuleEvidenceKind {
    fn source_body(self) -> &'static str {
        match self {
            Self::Role => "role reviewer{capabilities:[]}",
            Self::Law => "law reflexive(x:Int):x==x",
            Self::Proof => "proof reflexive(x:Int){by_definition}",
        }
    }

    fn select(self, definition: &Definition) -> (&Visibility, Span) {
        match (self, definition) {
            (Self::Role, Definition::Role(role)) => (&role.visibility, role.span),
            (Self::Law, Definition::Law(law)) => (&law.visibility, law.span),
            (Self::Proof, Definition::Proof(proof)) => (&proof.visibility, proof.span),
            _ => panic!("expected {self:?}, got {definition:?}"),
        }
    }
}

fn one_definition(source: &str) -> Definition {
    let module = parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module declaration should parse: {source}\n{errors:?}"));
    assert_eq!(module.definitions.len(), 1, "fixture has one definition");
    module
        .definitions
        .into_iter()
        .next()
        .expect("fixture definition exists")
}

fn assert_module_visibility(
    kind: ModuleEvidenceKind,
    prefix: &str,
    expected_visibility: Visibility,
) {
    let source = format!("{prefix}{}", kind.source_body());
    let definition = one_definition(&source);
    let (actual_visibility, span) = kind.select(&definition);

    assert_eq!(actual_visibility, &expected_visibility);
    assert_eq!(span.start, 0, "span starts at the visibility prefix");
    assert_eq!(span.end, source.len(), "span ends at declaration end");
}

#[test]
fn bare_module_role_law_and_proof_retain_inherited_visibility_and_full_spans() {
    for kind in [
        ModuleEvidenceKind::Role,
        ModuleEvidenceKind::Law,
        ModuleEvidenceKind::Proof,
    ] {
        assert_module_visibility(kind, "", Visibility::Inherited);
    }
}

#[test]
fn module_role_law_and_proof_retain_every_applicable_existing_visibility_form() {
    let visibility_cases = [
        ("pub ", Visibility::Public),
        ("pub(crate) ", Visibility::Crate),
        ("pub(super) ", Visibility::Super { levels: 1 }),
        ("pub(self) ", Visibility::Self_),
        (
            "pub(in crate::governance) ",
            Visibility::Restricted {
                path: "crate::governance".into(),
            },
        ),
    ];

    for kind in [
        ModuleEvidenceKind::Role,
        ModuleEvidenceKind::Law,
        ModuleEvidenceKind::Proof,
    ] {
        for (prefix, visibility) in &visibility_cases {
            assert_module_visibility(kind, prefix, visibility.clone());
        }
    }
}

#[test]
fn policy_role_law_and_proof_have_required_explicit_visibility_construction_fields() {
    let span = Span::new(4, 19, 1, 5);
    let policy = PolicyDef {
        visibility: Visibility::Crate,
        name: "Retention".into(),
        type_params: Vec::new(),
        fields: Vec::new(),
        where_clause: None,
        span,
    };
    let role = RoleDef {
        visibility: Visibility::Public,
        name: "reviewer".into(),
        capabilities: Vec::new(),
        obligations: Vec::new(),
        span,
    };
    let law = LawDef {
        visibility: Visibility::Restricted {
            path: "crate::evidence".into(),
        },
        name: "reflexive".into(),
        params: Vec::new(),
        constraints: Vec::new(),
        proposition: Expr::Literal(Literal::Bool(true)),
        span,
    };
    let proof = ProofDef {
        visibility: Visibility::Super { levels: 1 },
        name: "reflexive".into(),
        params: Vec::new(),
        constraints: Vec::new(),
        body: ProofBody::ByDefinition,
        span,
    };

    assert_eq!(policy.visibility, Visibility::Crate);
    assert_eq!(role.visibility, Visibility::Public);
    assert_eq!(
        law.visibility,
        Visibility::Restricted {
            path: "crate::evidence".into()
        }
    );
    assert_eq!(proof.visibility, Visibility::Super { levels: 1 });
}

#[test]
fn interface_laws_and_impl_proofs_remain_inherited_parent_scoped_members() {
    let interface_source = "interface Eq<A>{law reflexive(x:A):x==x}";
    let interface_module = parse_surface_file(interface_source).expect("interface fixture parses");
    let Definition::Interface(interface) = &interface_module.definitions[0] else {
        panic!("expected interface definition")
    };
    let law = &interface.laws[0];
    let law_start = interface_source.find("law").expect("law source anchor");
    assert_eq!(law.visibility, Visibility::Inherited);
    assert_eq!(law.span.start, law_start);
    assert_eq!(law.span.end, interface_source.len() - 1);

    let impl_source = "impl Eq<Int>{proof reflexive(x:Int){by_definition}}";
    let impl_module = parse_surface_file(impl_source).expect("impl fixture parses");
    let Definition::Impl(implementation) = &impl_module.definitions[0] else {
        panic!("expected impl definition")
    };
    let proof = &implementation.proofs[0];
    let proof_start = impl_source.find("proof").expect("proof source anchor");
    assert_eq!(proof.visibility, Visibility::Inherited);
    assert_eq!(proof.span.start, proof_start);
    assert_eq!(proof.span.end, impl_source.len() - 1);
}

#[test]
fn visibility_qualified_nested_laws_and_proofs_are_not_new_grammar() {
    assert!(
        parse_surface_file("interface Eq<A>{pub law reflexive(x:A):x==x}").is_err(),
        "interface laws remain inherited parent-scoped members"
    );
    assert!(
        parse_surface_file("impl Eq<Int>{pub proof reflexive(x:Int){by_definition}}").is_err(),
        "impl proofs remain inherited parent-scoped members"
    );
}
