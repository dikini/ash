use ash_core::Effect;
use ash_core::workflow_contract::{Effect as ContractEffect, Requirement};
use ash_parser::surface::{ActionRef, Expr, Literal, Workflow};
use ash_parser::token::Span;
use ash_typeck::effect::{infer_effect, join_effects};
use ash_typeck::requirements::{
    CheckResult, RequirementContext, RequirementError, check_requirement,
};
use ash_typeck::runtime_verification::{EffectChecker, VerificationError};

fn test_span() -> Span {
    Span::default()
}

#[test]
fn workflow_form_effect_classification_matches_promoted_contract() {
    let for_workflow = Workflow::For {
        pattern: ash_parser::surface::Pattern::Wildcard,
        collection: Expr::Literal(Literal::Int(1)),
        body: Box::new(Workflow::Done { span: test_span() }),
        span: test_span(),
    };
    assert_eq!(infer_effect(&for_workflow), Effect::Epistemic);

    let ret_workflow = Workflow::Ret {
        expr: Expr::Literal(Literal::Int(42)),
        span: test_span(),
    };
    assert_eq!(infer_effect(&ret_workflow), Effect::Epistemic);

    let oblige_workflow = Workflow::Oblige {
        obligation: "audit".into(),
        span: test_span(),
    };
    assert_eq!(infer_effect(&oblige_workflow), Effect::Epistemic);
}

#[test]
fn join_based_composition_preserves_highest_coarse_grade() {
    let composed = Workflow::Seq {
        first: Box::new(Workflow::Check {
            target: ash_parser::surface::CheckTarget::Obligation(
                ash_parser::surface::ObligationRef {
                    role: "reviewer".into(),
                    condition: Expr::Literal(Literal::Bool(true)),
                },
            ),
            continuation: None,
            span: test_span(),
        }),
        second: Box::new(Workflow::Act {
            action: ActionRef {
                name: "write".into(),
                args: vec![],
            },
            guard: None,
            span: test_span(),
        }),
        span: test_span(),
    };

    assert_eq!(infer_effect(&composed), Effect::Operational);
    assert_eq!(
        join_effects(&[Effect::Epistemic, Effect::Evaluative, Effect::Operational]),
        Effect::Operational
    );
}

#[test]
fn provider_metadata_compatibility_rejects_weaker_metadata() {
    let ctx = RequirementContext::new().with_capability_metadata(
        "sensor",
        ContractEffect::Evaluative,
        ContractEffect::Deliberative,
    );
    let req = Requirement::HasCapability {
        cap: "sensor".into(),
        min_effect: ContractEffect::Epistemic,
    };

    let result = check_requirement(&req, &ctx);
    match result {
        CheckResult::Failed(RequirementError::IncompatibleProviderMetadata {
            cap,
            source_effect,
            provider_effect,
        }) => {
            assert_eq!(cap, "sensor");
            assert_eq!(source_effect, ContractEffect::Evaluative);
            assert_eq!(provider_effect, ContractEffect::Deliberative);
        }
        other => panic!("expected incompatible provider metadata failure, got {other:?}"),
    }
}

#[test]
fn source_level_classification_wins_over_provider_metadata_overreach() {
    let ctx = RequirementContext::new().with_capability_metadata(
        "sensor",
        ContractEffect::Epistemic,
        ContractEffect::Operational,
    );

    let epistemic_req = Requirement::HasCapability {
        cap: "sensor".into(),
        min_effect: ContractEffect::Epistemic,
    };
    assert!(check_requirement(&epistemic_req, &ctx).is_satisfied());

    let operational_req = Requirement::HasCapability {
        cap: "sensor".into(),
        min_effect: ContractEffect::Operational,
    };

    match check_requirement(&operational_req, &ctx) {
        CheckResult::Failed(RequirementError::MissingCapability {
            cap,
            required,
            found,
        }) => {
            assert_eq!(cap, "sensor");
            assert_eq!(required, ContractEffect::Operational);
            assert_eq!(found, Some(ContractEffect::Epistemic));
        }
        other => panic!("expected source-level effect to remain authoritative, got {other:?}"),
    }
}

#[test]
fn runtime_effect_checker_accepts_preclassified_effects() {
    let checker = EffectChecker::new();

    let ok = checker.check_inferred(Effect::Evaluative, Effect::Operational);
    assert!(ok.is_ok());

    let too_high = checker.check_inferred(Effect::Operational, Effect::Evaluative);
    assert!(too_high.errors.iter().any(|error| matches!(
        error,
        VerificationError::EffectTooHigh {
            workflow_effect: Effect::Operational,
            max_allowed: Effect::Evaluative,
        }
    )));
}
