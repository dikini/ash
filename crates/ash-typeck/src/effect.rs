//! Effect inference and checking (TASK-021)
//!
//! Provides effect inference for workflows and validation of effect constraints
//! using the effect lattice operations.

use ash_core::Effect;
use ash_parser::surface::Workflow;

/// Infer the effect of a workflow by computing the join of all sub-effects
///
/// The effect of a workflow is the least upper bound (join) of all effects
/// in its constituent operations. This ensures that the workflow's effect
/// accurately reflects the most powerful operation it performs.
pub fn infer_effect(workflow: &Workflow) -> Effect {
    match workflow {
        Workflow::Observe { continuation, .. } => {
            let cont_effect = continuation
                .as_ref()
                .map(|c| infer_effect(c))
                .unwrap_or(Effect::Epistemic);
            Effect::Epistemic.join(cont_effect)
        }

        Workflow::Orient { continuation, .. } => {
            let cont_effect = continuation
                .as_ref()
                .map(|c| infer_effect(c))
                .unwrap_or(Effect::Deliberative);
            Effect::Deliberative.join(cont_effect)
        }

        Workflow::Propose { continuation, .. } => {
            let cont_effect = continuation
                .as_ref()
                .map(|c| infer_effect(c))
                .unwrap_or(Effect::Deliberative);
            Effect::Deliberative.join(cont_effect)
        }

        Workflow::Decide {
            then_branch,
            else_branch,
            ..
        } => {
            let then_effect = infer_effect(then_branch);
            let else_effect = else_branch
                .as_ref()
                .map(|b| infer_effect(b))
                .unwrap_or(Effect::Epistemic);
            Effect::Evaluative.join(then_effect.join(else_effect))
        }

        Workflow::Check { continuation, .. } => {
            let cont_effect = continuation
                .as_ref()
                .map(|c| infer_effect(c))
                .unwrap_or(Effect::Evaluative);
            Effect::Evaluative.join(cont_effect)
        }

        Workflow::Act { .. } => Effect::Operational,

        Workflow::Let { continuation, .. } => continuation
            .as_ref()
            .map(|c| infer_effect(c))
            .unwrap_or(Effect::Epistemic),

        Workflow::If {
            then_branch,
            else_branch,
            ..
        } => {
            let then_effect = infer_effect(then_branch);
            let else_effect = else_branch
                .as_ref()
                .map(|b| infer_effect(b))
                .unwrap_or(Effect::Epistemic);
            then_effect.join(else_effect)
        }

        Workflow::For { body, .. } => Effect::Operational.join(infer_effect(body)),

        Workflow::Par { branches, .. } => {
            // Parallel composition: join of all branch effects
            branches
                .iter()
                .map(|b| infer_effect(b))
                .fold(Effect::Epistemic, |acc, e| acc.join(e))
        }

        Workflow::With { body, .. } => infer_effect(body),

        Workflow::Maybe {
            primary, fallback, ..
        } => {
            let primary_effect = infer_effect(primary);
            let fallback_effect = infer_effect(fallback);
            primary_effect.join(fallback_effect)
        }

        Workflow::Must { body, .. } => infer_effect(body),

        Workflow::Seq { first, second, .. } => {
            let first_effect = infer_effect(first);
            let second_effect = infer_effect(second);
            first_effect.join(second_effect)
        }

        Workflow::Done { .. } => Effect::Epistemic,
    }
}

/// Check if an effect satisfies a constraint (e <= required)
///
/// Returns true if the actual effect is at most as powerful as the required effect.
/// This is used to validate that a workflow doesn't exceed allowed effect bounds.
pub fn check_effect(actual: Effect, required: Effect) -> bool {
    // e <= required means actual is less than or equal to required
    actual <= required
}

/// Check if an effect meets a minimum requirement (e >= minimum)
///
/// Returns true if the actual effect is at least as powerful as the minimum.
pub fn check_effect_minimum(actual: Effect, minimum: Effect) -> bool {
    actual.at_least(minimum)
}

/// Compute the join of multiple effects
///
/// The join represents the least upper bound - the most powerful effect.
pub fn join_effects(effects: &[Effect]) -> Effect {
    effects
        .iter()
        .fold(Effect::Epistemic, |acc, &e| acc.join(e))
}

/// Compute the meet of multiple effects
///
/// The meet represents the greatest lower bound - the least powerful effect
/// that is still at most as powerful as all inputs.
pub fn meet_effects(effects: &[Effect]) -> Effect {
    if effects.is_empty() {
        Effect::Operational // Top element
    } else {
        effects
            .iter()
            .skip(1)
            .fold(effects[0], |acc, &e| acc.meet(e))
    }
}

/// Effect context for tracking effect constraints during type checking
#[derive(Debug, Clone)]
pub struct EffectContext {
    /// Current effect bound (maximum allowed effect)
    pub bound: Effect,
    /// Current accumulated effect
    pub current: Effect,
}

impl Default for EffectContext {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectContext {
    /// Create a new effect context with no bounds
    pub fn new() -> Self {
        Self {
            bound: Effect::Operational,
            current: Effect::Epistemic,
        }
    }

    /// Create a context with a specific effect bound
    pub fn with_bound(bound: Effect) -> Self {
        Self {
            bound,
            current: Effect::Epistemic,
        }
    }

    /// Record an effect and check if it exceeds the bound
    pub fn record(&mut self, effect: Effect) -> bool {
        self.current = self.current.join(effect);
        self.check()
    }

    /// Check if current effect is within bounds
    pub fn check(&self) -> bool {
        self.current <= self.bound
    }

    /// Get the current accumulated effect
    pub fn current(&self) -> Effect {
        self.current
    }

    /// Get the effect bound
    pub fn bound(&self) -> Effect {
        self.bound
    }

    /// Reset the current effect
    pub fn reset(&mut self) {
        self.current = Effect::Epistemic;
    }

    /// Create a nested context with the same bound
    pub fn nested(&self) -> Self {
        Self::with_bound(self.bound)
    }

    /// Create a restricted context with a lower bound
    pub fn restricted(&self, bound: Effect) -> Self {
        // The new bound is the meet of current bound and requested bound
        // (more restrictive wins)
        Self::with_bound(self.bound.meet(bound))
    }
}

/// Result of effect inference
#[derive(Debug, Clone)]
pub struct EffectInferenceResult {
    /// The inferred effect
    pub effect: Effect,
    /// Whether the effect is within bounds
    pub within_bounds: bool,
    /// Violations if any
    pub violations: Vec<EffectViolation>,
}

impl EffectInferenceResult {
    /// Create a successful result
    pub fn success(effect: Effect) -> Self {
        Self {
            effect,
            within_bounds: true,
            violations: vec![],
        }
    }

    /// Create a result with violations
    pub fn with_violations(effect: Effect, violations: Vec<EffectViolation>) -> Self {
        Self {
            effect,
            within_bounds: violations.is_empty(),
            violations,
        }
    }
}

/// An effect constraint violation
#[derive(Debug, Clone, PartialEq)]
pub struct EffectViolation {
    /// The actual effect
    pub actual: Effect,
    /// The required/bound effect
    pub required: Effect,
    /// Description of where the violation occurred
    pub location: String,
}

/// Infer effect for a workflow with bounds checking
pub fn infer_effect_with_bounds(workflow: &Workflow, bound: Effect) -> EffectInferenceResult {
    let effect = infer_effect(workflow);

    if effect <= bound {
        EffectInferenceResult::success(effect)
    } else {
        EffectInferenceResult::with_violations(
            effect,
            vec![EffectViolation {
                actual: effect,
                required: bound,
                location: "workflow".to_string(),
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::surface::{ActionRef, Expr, Literal, Pattern};
    use ash_parser::token::Span;

    fn test_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    #[test]
    fn test_infer_effect_done() {
        let workflow = Workflow::Done { span: test_span() };
        assert_eq!(infer_effect(&workflow), Effect::Epistemic);
    }

    #[test]
    fn test_infer_effect_observe() {
        let workflow = Workflow::Observe {
            capability: "read".into(),
            binding: None,
            continuation: None,
            span: test_span(),
        };
        assert_eq!(infer_effect(&workflow), Effect::Epistemic);
    }

    #[test]
    fn test_infer_effect_observe_with_cont() {
        let workflow = Workflow::Observe {
            capability: "read".into(),
            binding: None,
            continuation: Some(Box::new(Workflow::Act {
                action: ActionRef {
                    name: "write".into(),
                    args: vec![],
                },
                guard: None,
                span: test_span(),
            })),
            span: test_span(),
        };
        assert_eq!(infer_effect(&workflow), Effect::Operational);
    }

    #[test]
    fn test_infer_effect_act() {
        let workflow = Workflow::Act {
            action: ActionRef {
                name: "write".into(),
                args: vec![],
            },
            guard: None,
            span: test_span(),
        };
        assert_eq!(infer_effect(&workflow), Effect::Operational);
    }

    #[test]
    fn test_infer_effect_let() {
        let workflow = Workflow::Let {
            pattern: Pattern::Variable("x".into()),
            expr: Expr::Literal(Literal::Int(42)),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };
        assert_eq!(infer_effect(&workflow), Effect::Epistemic);
    }

    #[test]
    fn test_infer_effect_if() {
        let workflow = Workflow::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_branch: Box::new(Workflow::Done { span: test_span() }),
            else_branch: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };
        assert_eq!(infer_effect(&workflow), Effect::Epistemic);
    }

    #[test]
    fn test_infer_effect_if_with_operational() {
        let workflow = Workflow::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_branch: Box::new(Workflow::Act {
                action: ActionRef {
                    name: "write".into(),
                    args: vec![],
                },
                guard: None,
                span: test_span(),
            }),
            else_branch: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };
        assert_eq!(infer_effect(&workflow), Effect::Operational);
    }

    #[test]
    fn test_infer_effect_seq() {
        let workflow = Workflow::Seq {
            first: Box::new(Workflow::Observe {
                capability: "read".into(),
                binding: None,
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
        assert_eq!(infer_effect(&workflow), Effect::Operational);
    }

    #[test]
    fn test_infer_effect_par() {
        let workflow = Workflow::Par {
            branches: vec![
                Workflow::Observe {
                    capability: "read".into(),
                    binding: None,
                    continuation: None,
                    span: test_span(),
                },
                Workflow::Act {
                    action: ActionRef {
                        name: "write".into(),
                        args: vec![],
                    },
                    guard: None,
                    span: test_span(),
                },
            ],
            span: test_span(),
        };
        assert_eq!(infer_effect(&workflow), Effect::Operational);
    }

    #[test]
    fn test_check_effect_valid() {
        assert!(check_effect(Effect::Epistemic, Effect::Operational));
        assert!(check_effect(Effect::Operational, Effect::Operational));
        assert!(check_effect(Effect::Epistemic, Effect::Epistemic));
    }

    #[test]
    fn test_check_effect_invalid() {
        assert!(!check_effect(Effect::Operational, Effect::Epistemic));
        assert!(!check_effect(Effect::Operational, Effect::Deliberative));
    }

    #[test]
    fn test_check_effect_minimum() {
        assert!(check_effect_minimum(Effect::Operational, Effect::Epistemic));
        assert!(check_effect_minimum(
            Effect::Operational,
            Effect::Operational
        ));
        assert!(!check_effect_minimum(
            Effect::Epistemic,
            Effect::Operational
        ));
    }

    #[test]
    fn test_join_effects() {
        let effects = vec![Effect::Epistemic, Effect::Deliberative, Effect::Epistemic];
        assert_eq!(join_effects(&effects), Effect::Deliberative);

        let effects = vec![Effect::Epistemic, Effect::Operational];
        assert_eq!(join_effects(&effects), Effect::Operational);
    }

    #[test]
    fn test_join_effects_empty() {
        let effects: Vec<Effect> = vec![];
        assert_eq!(join_effects(&effects), Effect::Epistemic);
    }

    #[test]
    fn test_meet_effects() {
        let effects = vec![
            Effect::Operational,
            Effect::Deliberative,
            Effect::Operational,
        ];
        assert_eq!(meet_effects(&effects), Effect::Deliberative);

        let effects = vec![Effect::Epistemic, Effect::Operational];
        assert_eq!(meet_effects(&effects), Effect::Epistemic);
    }

    #[test]
    fn test_meet_effects_empty() {
        let effects: Vec<Effect> = vec![];
        assert_eq!(meet_effects(&effects), Effect::Operational);
    }

    #[test]
    fn test_effect_context_creation() {
        let ctx = EffectContext::new();
        assert_eq!(ctx.bound(), Effect::Operational);
        assert_eq!(ctx.current(), Effect::Epistemic);
        assert!(ctx.check());
    }

    #[test]
    fn test_effect_context_with_bound() {
        let ctx = EffectContext::with_bound(Effect::Deliberative);
        assert_eq!(ctx.bound(), Effect::Deliberative);
        assert_eq!(ctx.current(), Effect::Epistemic);
    }

    #[test]
    fn test_effect_context_record() {
        let mut ctx = EffectContext::with_bound(Effect::Deliberative);
        assert!(ctx.record(Effect::Epistemic));
        assert!(ctx.record(Effect::Deliberative));
        assert!(!ctx.record(Effect::Evaluative));
        assert!(!ctx.record(Effect::Operational));
    }

    #[test]
    fn test_effect_context_reset() {
        let mut ctx = EffectContext::new();
        ctx.record(Effect::Operational);
        assert_eq!(ctx.current(), Effect::Operational);

        ctx.reset();
        assert_eq!(ctx.current(), Effect::Epistemic);
    }

    #[test]
    fn test_effect_context_nested() {
        let ctx = EffectContext::with_bound(Effect::Deliberative);
        let nested = ctx.nested();
        assert_eq!(nested.bound(), Effect::Deliberative);
    }

    #[test]
    fn test_effect_context_restricted() {
        let ctx = EffectContext::with_bound(Effect::Operational);
        let restricted = ctx.restricted(Effect::Deliberative);
        assert_eq!(restricted.bound(), Effect::Deliberative);

        // More restrictive should win
        let ctx2 = EffectContext::with_bound(Effect::Deliberative);
        let restricted2 = ctx2.restricted(Effect::Operational);
        assert_eq!(restricted2.bound(), Effect::Deliberative);
    }

    #[test]
    fn test_infer_effect_with_bounds_success() {
        let workflow = Workflow::Observe {
            capability: "read".into(),
            binding: None,
            continuation: None,
            span: test_span(),
        };

        let result = infer_effect_with_bounds(&workflow, Effect::Operational);
        assert!(result.within_bounds);
        assert!(result.violations.is_empty());
        assert_eq!(result.effect, Effect::Epistemic);
    }

    #[test]
    fn test_infer_effect_with_bounds_failure() {
        let workflow = Workflow::Act {
            action: ActionRef {
                name: "write".into(),
                args: vec![],
            },
            guard: None,
            span: test_span(),
        };

        let result = infer_effect_with_bounds(&workflow, Effect::Deliberative);
        assert!(!result.within_bounds);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.effect, Effect::Operational);
    }

    #[test]
    fn test_effect_inference_result_success() {
        let result = EffectInferenceResult::success(Effect::Epistemic);
        assert!(result.within_bounds);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_infer_effect_orient() {
        let workflow = Workflow::Orient {
            expr: Expr::Literal(Literal::Int(42)),
            binding: None,
            continuation: None,
            span: test_span(),
        };
        assert_eq!(infer_effect(&workflow), Effect::Deliberative);
    }

    #[test]
    fn test_infer_effect_propose() {
        let workflow = Workflow::Propose {
            action: ActionRef {
                name: "action".into(),
                args: vec![],
            },
            binding: None,
            continuation: None,
            span: test_span(),
        };
        assert_eq!(infer_effect(&workflow), Effect::Deliberative);
    }

    #[test]
    fn test_infer_effect_decide() {
        let workflow = Workflow::Decide {
            expr: Expr::Literal(Literal::Bool(true)),
            policy: None,
            then_branch: Box::new(Workflow::Done { span: test_span() }),
            else_branch: None,
            span: test_span(),
        };
        assert_eq!(infer_effect(&workflow), Effect::Evaluative);
    }

    #[test]
    fn test_infer_effect_check() {
        let workflow = Workflow::Check {
            target: ash_parser::surface::CheckTarget::Obligation(
                ash_parser::surface::ObligationRef {
                    role: "admin".into(),
                    condition: Expr::Literal(Literal::Bool(true)),
                },
            ),
            continuation: None,
            span: test_span(),
        };
        assert_eq!(infer_effect(&workflow), Effect::Evaluative);
    }

    #[test]
    fn test_infer_effect_with() {
        let workflow = Workflow::With {
            capability: "db".into(),
            body: Box::new(Workflow::Act {
                action: ActionRef {
                    name: "query".into(),
                    args: vec![],
                },
                guard: None,
                span: test_span(),
            }),
            span: test_span(),
        };
        assert_eq!(infer_effect(&workflow), Effect::Operational);
    }

    #[test]
    fn test_infer_effect_maybe() {
        let workflow = Workflow::Maybe {
            primary: Box::new(Workflow::Done { span: test_span() }),
            fallback: Box::new(Workflow::Act {
                action: ActionRef {
                    name: "write".into(),
                    args: vec![],
                },
                guard: None,
                span: test_span(),
            }),
            span: test_span(),
        };
        assert_eq!(infer_effect(&workflow), Effect::Operational);
    }

    #[test]
    fn test_infer_effect_must() {
        let workflow = Workflow::Must {
            body: Box::new(Workflow::Act {
                action: ActionRef {
                    name: "write".into(),
                    args: vec![],
                },
                guard: None,
                span: test_span(),
            }),
            span: test_span(),
        };
        assert_eq!(infer_effect(&workflow), Effect::Operational);
    }
}
