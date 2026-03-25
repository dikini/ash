//! Surface AST to Core IR lowering.
//!
//! This module converts the surface syntax AST into the core IR representation
//! used by the ash-core crate.

#[cfg(test)]
use std::fmt;

use ash_core::{
    Action as CoreAction, Capability, Effect, Expr as CoreExpr, Guard as CoreGuard,
    MatchArm as CoreMatchArm, Obligation as CoreObligation, Pattern as CorePattern,
    Predicate as CorePredicate, Provenance, ReceiveArm as CoreReceiveArm,
    ReceivePattern as CoreReceivePattern, Role as CoreRole, Workflow as CoreWorkflow,
};

#[cfg(test)]
use ash_core::RoleObligationRef as CoreRoleObligationRef;

use crate::surface::{
    ActionRef, BinaryOp, CapabilityDef, CheckTarget, Definition, EffectType, Expr, Guard, Literal,
    ObligationRef, Pattern, PolicyExpr, Predicate, RoleDef, StreamPattern, UnaryOp,
    Workflow as SurfaceWorkflow, WorkflowDef,
};

/// Error returned when parsed role metadata cannot be lowered honestly.
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RoleLoweringError {
    pub(crate) role: String,
    pub(crate) authority: String,
}

#[cfg(test)]
impl fmt::Display for RoleLoweringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cannot lower role '{}' because authority '{}' has no matching capability definition",
            self.role, self.authority
        )
    }
}

#[cfg(test)]
impl std::error::Error for RoleLoweringError {}

/// Lower a workflow definition to core IR.
pub fn lower_workflow(def: &WorkflowDef) -> CoreWorkflow {
    // Create a provenance for the workflow
    let provenance = Provenance::new();

    lower_workflow_body(&def.body, &provenance)
}

#[cfg(test)]
fn lower_role_def_with_definitions(
    def: &RoleDef,
    definitions: &[Definition],
) -> Result<CoreRole, RoleLoweringError> {
    Ok(CoreRole {
        name: def.name.to_string(),
        authority: def
            .authority
            .iter()
            .map(|name| lower_role_authority(def.name.as_ref(), name, definitions))
            .collect::<Result<Vec<_>, _>>()?,
        obligations: def
            .obligations
            .iter()
            .map(|name| lower_role_obligation_name(name))
            .collect(),
    })
}

/// Lower all parsed inline-module role definitions into core role metadata.
#[cfg(test)]
pub(crate) fn lower_module_role_definitions(
    module: &crate::module::ModuleDecl,
) -> Result<Vec<CoreRole>, RoleLoweringError> {
    let Some(definitions) = module.definitions() else {
        return Ok(vec![]);
    };

    definitions
        .iter()
        .filter_map(|definition| match definition {
            Definition::Role(role) => Some(lower_role_def_with_definitions(role, definitions)),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
fn lower_role_authority(
    role_name: &str,
    authority_name: &str,
    definitions: &[Definition],
) -> Result<Capability, RoleLoweringError> {
    definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Capability(capability) if capability.name.as_ref() == authority_name => {
                Some(lower_capability_def(capability))
            }
            _ => None,
        })
        .ok_or_else(|| RoleLoweringError {
            role: role_name.to_string(),
            authority: authority_name.to_string(),
        })
}

fn lower_capability_def(def: &CapabilityDef) -> Capability {
    Capability {
        name: def.name.to_string(),
        effect: lower_effect_type(def.effect),
        constraints: def.constraints.iter().map(lower_constraint).collect(),
    }
}

fn lower_constraint(constraint: &crate::surface::Constraint) -> ash_core::Constraint {
    ash_core::Constraint {
        predicate: lower_predicate(&constraint.predicate),
    }
}

/// Lower a workflow body to core IR.
fn lower_workflow_body(workflow: &SurfaceWorkflow, provenance: &Provenance) -> CoreWorkflow {
    match workflow {
        SurfaceWorkflow::Observe {
            capability,
            binding,
            continuation,
            ..
        } => {
            let pattern = binding
                .as_ref()
                .map(lower_pattern)
                .unwrap_or(CorePattern::Wildcard);

            let cont = continuation
                .as_ref()
                .map(|c| lower_workflow_body(c, provenance))
                .unwrap_or(CoreWorkflow::Done);

            CoreWorkflow::Observe {
                capability: Capability {
                    name: capability.to_string(),
                    effect: Effect::Epistemic,
                    constraints: vec![],
                },
                pattern,
                continuation: Box::new(cont),
            }
        }

        SurfaceWorkflow::Orient {
            expr,
            binding: _,
            continuation,
            ..
        } => {
            let cont = continuation
                .as_ref()
                .map(|c| lower_workflow_body(c, provenance))
                .unwrap_or(CoreWorkflow::Done);

            CoreWorkflow::Orient {
                expr: lower_expr(expr),
                continuation: Box::new(cont),
            }
        }

        SurfaceWorkflow::Propose {
            action,
            binding: _,
            continuation,
            ..
        } => {
            let cont = continuation
                .as_ref()
                .map(|c| lower_workflow_body(c, provenance))
                .unwrap_or(CoreWorkflow::Done);

            CoreWorkflow::Propose {
                action: lower_action(action),
                continuation: Box::new(cont),
            }
        }

        SurfaceWorkflow::Decide {
            expr,
            policy,
            then_branch,
            else_branch,
            ..
        } => {
            assert!(
                else_branch.is_none(),
                "legacy decide else-branches are not part of the canonical lowering contract"
            );

            CoreWorkflow::Decide {
                expr: lower_expr(expr),
                policy: policy
                    .as_ref()
                    .expect("canonical decide lowering requires an explicit named policy")
                    .to_string(),
                continuation: Box::new(lower_workflow_body(then_branch, provenance)),
            }
        }

        SurfaceWorkflow::Check {
            target,
            continuation,
            ..
        } => {
            let cont = continuation
                .as_ref()
                .map(|c| lower_workflow_body(c, provenance))
                .unwrap_or(CoreWorkflow::Done);

            CoreWorkflow::Check {
                obligation: lower_check_target(target),
                continuation: Box::new(cont),
            }
        }

        SurfaceWorkflow::Oblige { obligation, .. } => CoreWorkflow::Oblige {
            name: obligation.to_string(),
            span: Default::default(),
        },

        SurfaceWorkflow::Act { action, guard, .. } => CoreWorkflow::Act {
            action: lower_action(action),
            guard: guard.as_ref().map(lower_guard).unwrap_or(CoreGuard::Always),
            provenance: provenance.clone(),
        },

        SurfaceWorkflow::Set {
            capability,
            channel,
            value,
            continuation,
            ..
        } => {
            let cont = continuation
                .as_ref()
                .map(|c| lower_workflow_body(c, provenance))
                .unwrap_or(CoreWorkflow::Done);

            CoreWorkflow::Seq {
                first: Box::new(CoreWorkflow::Set {
                    capability: capability.to_string(),
                    channel: channel.to_string(),
                    value: lower_expr(value),
                }),
                second: Box::new(cont),
            }
        }

        SurfaceWorkflow::Send {
            capability,
            channel,
            value,
            continuation,
            ..
        } => {
            let cont = continuation
                .as_ref()
                .map(|c| lower_workflow_body(c, provenance))
                .unwrap_or(CoreWorkflow::Done);

            CoreWorkflow::Seq {
                first: Box::new(CoreWorkflow::Send {
                    capability: capability.to_string(),
                    channel: channel.to_string(),
                    value: lower_expr(value),
                }),
                second: Box::new(cont),
            }
        }

        SurfaceWorkflow::Receive {
            mode,
            arms,
            is_control,
            ..
        } => CoreWorkflow::Receive {
            mode: lower_receive_mode(mode),
            arms: arms
                .iter()
                .map(|arm| lower_receive_arm(arm, provenance))
                .collect(),
            control: *is_control,
        },

        SurfaceWorkflow::Let {
            pattern,
            expr,
            continuation,
            ..
        } => {
            let cont = continuation
                .as_ref()
                .map(|c| lower_workflow_body(c, provenance))
                .unwrap_or(CoreWorkflow::Done);

            CoreWorkflow::Let {
                pattern: lower_pattern(pattern),
                expr: lower_expr(expr),
                continuation: Box::new(cont),
            }
        }

        SurfaceWorkflow::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            let else_wf = else_branch
                .as_ref()
                .map(|e| lower_workflow_body(e, provenance))
                .unwrap_or(CoreWorkflow::Done);
            let then_wf = lower_workflow_body(then_branch, provenance);

            CoreWorkflow::If {
                condition: lower_expr(condition),
                then_branch: Box::new(then_wf),
                else_branch: Box::new(else_wf),
            }
        }

        SurfaceWorkflow::For {
            pattern,
            collection,
            body,
            ..
        } => CoreWorkflow::ForEach {
            pattern: lower_pattern(pattern),
            collection: lower_expr(collection),
            body: Box::new(lower_workflow_body(body, provenance)),
        },

        SurfaceWorkflow::Par { branches, .. } => {
            let workflows: Vec<_> = branches
                .iter()
                .map(|b| lower_workflow_body(b, provenance))
                .collect();

            CoreWorkflow::Par { workflows }
        }

        SurfaceWorkflow::With {
            capability, body, ..
        } => CoreWorkflow::With {
            capability: Capability {
                name: capability.to_string(),
                effect: Effect::Epistemic,
                constraints: vec![],
            },
            workflow: Box::new(lower_workflow_body(body, provenance)),
        },

        SurfaceWorkflow::Maybe {
            primary, fallback, ..
        } => CoreWorkflow::Maybe {
            primary: Box::new(lower_workflow_body(primary, provenance)),
            fallback: Box::new(lower_workflow_body(fallback, provenance)),
        },

        SurfaceWorkflow::Must { body, .. } => CoreWorkflow::Must {
            workflow: Box::new(lower_workflow_body(body, provenance)),
        },

        SurfaceWorkflow::Seq { first, second, .. } => CoreWorkflow::Seq {
            first: Box::new(lower_workflow_body(first, provenance)),
            second: Box::new(lower_workflow_body(second, provenance)),
        },

        SurfaceWorkflow::Done { .. } => CoreWorkflow::Done,

        SurfaceWorkflow::Ret { expr, .. } => CoreWorkflow::Ret {
            expr: lower_expr(expr),
        },

        // Proxy workflow constructs - for now, lower to a placeholder
        // These will be expanded in future work for full proxy support
        SurfaceWorkflow::Yield { .. } => {
            // Yield is not yet supported in core IR
            // For now, return Done as a placeholder
            CoreWorkflow::Done
        }

        SurfaceWorkflow::Resume { expr, .. } => CoreWorkflow::Ret {
            expr: lower_expr(expr),
        },
    }
}

/// Lower a surface expression to core IR.
pub fn lower_expr(expr: &Expr) -> CoreExpr {
    match expr {
        Expr::Literal(lit) => CoreExpr::Literal(lower_literal(lit)),

        Expr::Variable(name) => CoreExpr::Variable(name.to_string()),

        Expr::FieldAccess { base, field, .. } => CoreExpr::FieldAccess {
            expr: Box::new(lower_expr(base)),
            field: field.to_string(),
        },

        Expr::IndexAccess { base, index, .. } => CoreExpr::IndexAccess {
            expr: Box::new(lower_expr(base)),
            index: Box::new(lower_expr(index)),
        },

        Expr::Unary { op, operand, .. } => CoreExpr::Unary {
            op: lower_unary_op(*op),
            expr: Box::new(lower_expr(operand)),
        },

        Expr::Binary {
            op, left, right, ..
        } => CoreExpr::Binary {
            op: lower_binary_op(*op),
            left: Box::new(lower_expr(left)),
            right: Box::new(lower_expr(right)),
        },

        Expr::Call { func, args, .. } => CoreExpr::Call {
            func: func.to_string(),
            arguments: args.iter().map(lower_expr).collect(),
        },

        Expr::Match {
            scrutinee, arms, ..
        } => CoreExpr::Match {
            scrutinee: Box::new(lower_expr(scrutinee)),
            arms: arms
                .iter()
                .map(|arm| CoreMatchArm {
                    pattern: lower_pattern(&arm.pattern),
                    body: lower_expr(&arm.body),
                })
                .collect(),
        },

        Expr::Policy(policy_expr) => lower_policy_expr(policy_expr),

        Expr::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
            ..
        } => CoreExpr::IfLet {
            pattern: lower_pattern(pattern),
            expr: Box::new(lower_expr(expr)),
            then_branch: Box::new(lower_expr(then_branch)),
            else_branch: Box::new(lower_expr(else_branch)),
        },

        Expr::CheckObligation { obligation, span } => CoreExpr::CheckObligation {
            obligation: obligation.to_string(),
            span: ash_core::Span {
                start: span.start,
                end: span.end,
            },
        },

        Expr::Constructor { name, fields, .. } => CoreExpr::Constructor {
            name: name.to_string(),
            fields: fields
                .iter()
                .map(|(n, e)| (n.to_string(), lower_expr(e)))
                .collect(),
        },
    }
}

/// Lower a policy expression to core IR.
fn lower_policy_expr(expr: &PolicyExpr) -> CoreExpr {
    // For now, policy expressions are lowered as strings
    // A full implementation would lower to a policy representation in core IR
    CoreExpr::Literal(ash_core::Value::String(format!("{:?}", expr)))
}

fn lower_receive_mode(mode: &crate::surface::ReceiveMode) -> ash_core::ReceiveMode {
    match mode {
        crate::surface::ReceiveMode::NonBlocking => ash_core::ReceiveMode::NonBlocking,
        crate::surface::ReceiveMode::Blocking(timeout) => ash_core::ReceiveMode::Blocking(*timeout),
    }
}

fn lower_receive_arm(arm: &crate::surface::ReceiveArm, provenance: &Provenance) -> CoreReceiveArm {
    CoreReceiveArm {
        pattern: lower_receive_pattern(&arm.pattern),
        guard: arm.guard.as_ref().map(lower_expr),
        body: lower_workflow_body(&arm.body, provenance),
    }
}

fn lower_receive_pattern(pattern: &StreamPattern) -> CoreReceivePattern {
    match pattern {
        StreamPattern::Binding {
            capability,
            channel,
            pattern,
        } => CoreReceivePattern::Stream {
            capability: capability.to_string(),
            channel: channel.to_string(),
            pattern: lower_pattern(pattern),
        },
        StreamPattern::Literal(value) => {
            CoreReceivePattern::Literal(ash_core::Value::String(value.to_string()))
        }
        StreamPattern::Wildcard => CoreReceivePattern::Wildcard,
    }
}

/// Lower a check target to core IR.
fn lower_check_target(target: &CheckTarget) -> CoreObligation {
    match target {
        CheckTarget::Obligation(obl) => lower_obligation(obl),
        CheckTarget::Policy(_) => panic!("policy instances are not valid canonical check targets"),
    }
}

/// Lower a literal value.
fn lower_literal(lit: &Literal) -> ash_core::Value {
    use ash_core::Value;

    match lit {
        Literal::Int(n) => Value::Int(*n),
        Literal::Float(f) => {
            // TODO: Add Value::Float support
            // For now, truncate to Int as a placeholder
            Value::Int(*f as i64)
        }
        Literal::String(s) => Value::String(s.to_string()),
        Literal::Bool(b) => Value::Bool(*b),
        Literal::Null => Value::Null,
        Literal::List(elements) => {
            let lowered: Vec<_> = elements.iter().map(lower_literal).collect();
            Value::List(Box::new(lowered))
        }
    }
}

/// Lower a unary operator.
fn lower_unary_op(op: UnaryOp) -> ash_core::UnaryOp {
    match op {
        UnaryOp::Not => ash_core::UnaryOp::Not,
        UnaryOp::Neg => ash_core::UnaryOp::Neg,
    }
}

/// Lower a binary operator.
fn lower_binary_op(op: BinaryOp) -> ash_core::BinaryOp {
    match op {
        BinaryOp::Add => ash_core::BinaryOp::Add,
        BinaryOp::Sub => ash_core::BinaryOp::Sub,
        BinaryOp::Mul => ash_core::BinaryOp::Mul,
        BinaryOp::Div => ash_core::BinaryOp::Div,
        BinaryOp::And => ash_core::BinaryOp::And,
        BinaryOp::Or => ash_core::BinaryOp::Or,
        BinaryOp::Eq => ash_core::BinaryOp::Eq,
        BinaryOp::Neq => ash_core::BinaryOp::Ne,
        BinaryOp::Lt => ash_core::BinaryOp::Lt,
        BinaryOp::Gt => ash_core::BinaryOp::Gt,
        BinaryOp::Leq => ash_core::BinaryOp::Le,
        BinaryOp::Geq => ash_core::BinaryOp::Ge,
        BinaryOp::In => ash_core::BinaryOp::In,
    }
}

/// Lower a pattern to core IR.
pub fn lower_pattern(pattern: &Pattern) -> CorePattern {
    match pattern {
        Pattern::Variable(name) => CorePattern::Variable(name.to_string()),

        Pattern::Wildcard => CorePattern::Wildcard,

        Pattern::Tuple(patterns) => {
            CorePattern::Tuple(patterns.iter().map(lower_pattern).collect())
        }

        Pattern::Record(fields) => {
            let lowered: Vec<_> = fields
                .iter()
                .map(|(name, pat)| (name.to_string(), lower_pattern(pat)))
                .collect();
            CorePattern::Record(lowered)
        }

        Pattern::List { elements, rest } => CorePattern::List(
            elements.iter().map(lower_pattern).collect(),
            rest.as_ref().map(|r| r.to_string()),
        ),

        Pattern::Variant { name, fields } => CorePattern::Variant {
            name: name.to_string(),
            fields: fields.as_ref().map(|fs| {
                fs.iter()
                    .map(|(n, p)| (n.to_string(), lower_pattern(p)))
                    .collect()
            }),
        },

        Pattern::Literal(lit) => CorePattern::Literal(lower_literal(lit)),
    }
}

/// Lower an action reference to core Action.
fn lower_action(action: &ActionRef) -> CoreAction {
    CoreAction {
        name: action.name.to_string(),
        arguments: action.args.iter().map(lower_expr).collect(),
    }
}

/// Lower an obligation reference to core Obligation.
fn lower_obligation(obligation: &ObligationRef) -> CoreObligation {
    CoreObligation::Obliged {
        role: CoreRole {
            name: obligation.role.to_string(),
            authority: vec![],
            obligations: vec![],
        },
        condition: lower_expr(&obligation.condition),
    }
}

#[cfg(test)]
fn lower_role_obligation_name(name: &str) -> CoreRoleObligationRef {
    CoreRoleObligationRef {
        name: name.to_string(),
    }
}

/// Lower a guard to core IR.
fn lower_guard(guard: &Guard) -> CoreGuard {
    match guard {
        Guard::Always => CoreGuard::Always,
        Guard::Never => CoreGuard::Never,
        Guard::Pred(pred) => CoreGuard::Pred(lower_predicate(pred)),
        Guard::And(left, right) => {
            CoreGuard::And(Box::new(lower_guard(left)), Box::new(lower_guard(right)))
        }
        Guard::Or(left, right) => {
            CoreGuard::Or(Box::new(lower_guard(left)), Box::new(lower_guard(right)))
        }
        Guard::Not(inner) => CoreGuard::Not(Box::new(lower_guard(inner))),
    }
}

/// Lower a predicate to core IR.
fn lower_predicate(pred: &Predicate) -> CorePredicate {
    CorePredicate {
        name: pred.name.to_string(),
        arguments: pred.args.iter().map(lower_expr).collect(),
    }
}

/// Lower an effect type to core Effect.
fn lower_effect_type(effect: EffectType) -> Effect {
    match effect {
        EffectType::Observe | EffectType::Read | EffectType::Epistemic => Effect::Epistemic,
        EffectType::Analyze | EffectType::Deliberative => Effect::Deliberative,
        EffectType::Decide | EffectType::Evaluative => Effect::Evaluative,
        EffectType::Act | EffectType::Write | EffectType::External | EffectType::Operational => {
            Effect::Operational
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::{
        BinaryOp, EffectType, Expr as SurfaceExpr, Literal as SurfaceLiteral, Pattern, RoleDef,
        Workflow as SurfaceWorkflow,
    };
    use crate::token::Span;

    fn dummy_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    #[test]
    fn test_lower_done() {
        let surface = SurfaceWorkflow::Done { span: dummy_span() };
        let core = lower_workflow_body(&surface, &Provenance::new());
        assert!(matches!(core, CoreWorkflow::Done));
    }

    #[test]
    fn test_lower_let() {
        let surface = SurfaceWorkflow::Let {
            pattern: Pattern::Variable("x".into()),
            expr: SurfaceExpr::Literal(SurfaceLiteral::Int(42)),
            continuation: Some(Box::new(SurfaceWorkflow::Done { span: dummy_span() })),
            span: dummy_span(),
        };
        let core = lower_workflow_body(&surface, &Provenance::new());
        assert!(matches!(core, CoreWorkflow::Let { .. }));
    }

    #[test]
    fn test_lower_expr_literal() {
        let surface = SurfaceExpr::Literal(SurfaceLiteral::Int(42));
        let core = lower_expr(&surface);
        assert!(matches!(core, CoreExpr::Literal(ash_core::Value::Int(42))));
    }

    #[test]
    fn test_lower_expr_variable() {
        let surface = SurfaceExpr::Variable("my_var".into());
        let core = lower_expr(&surface);
        assert!(matches!(core, CoreExpr::Variable(name) if name == "my_var"));
    }

    #[test]
    fn test_lower_expr_binary() {
        let surface = SurfaceExpr::Binary {
            op: BinaryOp::Add,
            left: Box::new(SurfaceExpr::Literal(SurfaceLiteral::Int(1))),
            right: Box::new(SurfaceExpr::Literal(SurfaceLiteral::Int(2))),
            span: dummy_span(),
        };
        let core = lower_expr(&surface);
        assert!(matches!(
            core,
            CoreExpr::Binary {
                op: ash_core::BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    fn test_lower_pattern_variable() {
        let surface = Pattern::Variable("x".into());
        let core = lower_pattern(&surface);
        assert!(matches!(core, CorePattern::Variable(name) if name == "x"));
    }

    #[test]
    fn test_lower_pattern_wildcard() {
        let surface = Pattern::Wildcard;
        let core = lower_pattern(&surface);
        assert!(matches!(core, CorePattern::Wildcard));
    }

    #[test]
    fn test_lower_pattern_tuple() {
        let surface = Pattern::Tuple(vec![
            Pattern::Variable("a".into()),
            Pattern::Variable("b".into()),
        ]);
        let core = lower_pattern(&surface);
        assert!(matches!(core, CorePattern::Tuple(pats) if pats.len() == 2));
    }

    #[test]
    fn test_lower_literal_int() {
        let surface = SurfaceLiteral::Int(42);
        let core = lower_literal(&surface);
        assert!(matches!(core, ash_core::Value::Int(42)));
    }

    #[test]
    fn test_lower_literal_string() {
        let surface = SurfaceLiteral::String("hello".into());
        let core = lower_literal(&surface);
        assert!(matches!(core, ash_core::Value::String(s) if s == "hello"));
    }

    #[test]
    fn test_lower_obligation_uses_simplified_role_shape() {
        let surface = ObligationRef {
            role: "manager".into(),
            condition: SurfaceExpr::Variable("approved".into()),
        };

        let core = lower_obligation(&surface);

        assert!(matches!(
            core,
            CoreObligation::Obliged {
                role: CoreRole {
                    name,
                    authority,
                    obligations,
                },
                condition: CoreExpr::Variable(condition),
            } if name == "manager"
                && authority.is_empty()
                && obligations.is_empty()
                && condition == "approved"
        ));
    }

    #[test]
    fn test_lower_role_def_preserves_named_authority_refs_and_obligation_refs() {
        let surface = RoleDef {
            name: "reviewer".into(),
            authority: vec!["approve".into(), "review".into()],
            obligations: vec!["check_tests".into()],
            span: dummy_span(),
        };

        let definitions = vec![
            crate::surface::Definition::Capability(crate::surface::CapabilityDef {
                visibility: crate::surface::Visibility::Inherited,
                name: "approve".into(),
                effect: crate::surface::EffectType::Decide,
                params: vec![],
                return_type: None,
                constraints: vec![],
                span: dummy_span(),
            }),
            crate::surface::Definition::Capability(crate::surface::CapabilityDef {
                visibility: crate::surface::Visibility::Inherited,
                name: "review".into(),
                effect: crate::surface::EffectType::Analyze,
                params: vec![],
                return_type: None,
                constraints: vec![],
                span: dummy_span(),
            }),
        ];

        let core = lower_role_def_with_definitions(&surface, &definitions)
            .expect("matching capability definitions should lower authority metadata");

        assert_eq!(core.name, "reviewer");
        assert_eq!(core.authority.len(), 2);
        assert!(matches!(
            &core.authority[0],
            Capability { name, .. } if name == "approve"
        ));
        assert!(matches!(
            &core.authority[1],
            Capability { name, .. } if name == "review"
        ));
        assert!(matches!(
            &core.obligations[..],
            [ash_core::RoleObligationRef { name }] if name == "check_tests"
        ));
    }

    #[test]
    fn test_lower_module_role_definitions_only_lowers_roles() {
        let module = crate::module::ModuleDecl::inline(
            "governance".into(),
            crate::surface::Visibility::Inherited,
            vec![
                crate::surface::Definition::Capability(crate::surface::CapabilityDef {
                    visibility: crate::surface::Visibility::Inherited,
                    name: "approve".into(),
                    effect: crate::surface::EffectType::Read,
                    params: vec![],
                    return_type: None,
                    constraints: vec![],
                    span: dummy_span(),
                }),
                crate::surface::Definition::Role(RoleDef {
                    name: "reviewer".into(),
                    authority: vec!["approve".into()],
                    obligations: vec!["check_tests".into()],
                    span: dummy_span(),
                }),
            ],
            dummy_span(),
        );

        let roles = lower_module_role_definitions(&module)
            .expect("matching capability definitions should lower authority metadata");

        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "reviewer");
        assert!(matches!(
            &roles[0].obligations[..],
            [ash_core::RoleObligationRef { name }] if name == "check_tests"
        ));
    }

    #[test]
    fn test_lower_module_role_definitions_preserves_authority_metadata_from_module_capabilities() {
        let module = crate::module::ModuleDecl::inline(
            "governance".into(),
            crate::surface::Visibility::Inherited,
            vec![
                crate::surface::Definition::Capability(crate::surface::CapabilityDef {
                    visibility: crate::surface::Visibility::Inherited,
                    name: "approve".into(),
                    effect: crate::surface::EffectType::Decide,
                    params: vec![],
                    return_type: None,
                    constraints: vec![crate::surface::Constraint {
                        predicate: crate::surface::Predicate {
                            name: "requires_mfa".into(),
                            args: vec![],
                        },
                    }],
                    span: dummy_span(),
                }),
                crate::surface::Definition::Role(RoleDef {
                    name: "reviewer".into(),
                    authority: vec!["approve".into()],
                    obligations: vec!["check_tests".into()],
                    span: dummy_span(),
                }),
            ],
            dummy_span(),
        );

        let roles = lower_module_role_definitions(&module)
            .expect("matching capability definitions should lower authority metadata");

        assert_eq!(roles.len(), 1);
        assert!(matches!(
            &roles[0].authority[..],
            [Capability {
                name,
                effect: Effect::Evaluative,
                constraints,
            }] if name == "approve"
                && matches!(
                    &constraints[..],
                    [ash_core::Constraint {
                        predicate: ash_core::Predicate { name: predicate_name, arguments }
                    }] if predicate_name == "requires_mfa" && arguments.is_empty()
                )
        ));
    }

    #[test]
    fn test_lower_unary_op() {
        assert!(matches!(
            lower_unary_op(UnaryOp::Not),
            ash_core::UnaryOp::Not
        ));
        assert!(matches!(
            lower_unary_op(UnaryOp::Neg),
            ash_core::UnaryOp::Neg
        ));
    }

    #[test]
    fn test_lower_binary_op() {
        assert!(matches!(
            lower_binary_op(BinaryOp::Add),
            ash_core::BinaryOp::Add
        ));
        assert!(matches!(
            lower_binary_op(BinaryOp::Sub),
            ash_core::BinaryOp::Sub
        ));
        assert!(matches!(
            lower_binary_op(BinaryOp::Mul),
            ash_core::BinaryOp::Mul
        ));
        assert!(matches!(
            lower_binary_op(BinaryOp::Div),
            ash_core::BinaryOp::Div
        ));
        assert!(matches!(
            lower_binary_op(BinaryOp::Eq),
            ash_core::BinaryOp::Eq
        ));
        assert!(matches!(
            lower_binary_op(BinaryOp::And),
            ash_core::BinaryOp::And
        ));
        assert!(matches!(
            lower_binary_op(BinaryOp::Or),
            ash_core::BinaryOp::Or
        ));
    }

    #[test]
    fn test_lower_if() {
        let surface = SurfaceWorkflow::If {
            condition: SurfaceExpr::Literal(SurfaceLiteral::Bool(true)),
            then_branch: Box::new(SurfaceWorkflow::Done { span: dummy_span() }),
            else_branch: Some(Box::new(SurfaceWorkflow::Done { span: dummy_span() })),
            span: dummy_span(),
        };
        let core = lower_workflow_body(&surface, &Provenance::new());
        assert!(matches!(core, CoreWorkflow::If { .. }));
    }

    #[test]
    fn test_lower_seq() {
        let surface = SurfaceWorkflow::Seq {
            first: Box::new(SurfaceWorkflow::Done { span: dummy_span() }),
            second: Box::new(SurfaceWorkflow::Done { span: dummy_span() }),
            span: dummy_span(),
        };
        let core = lower_workflow_body(&surface, &Provenance::new());
        assert!(matches!(core, CoreWorkflow::Seq { .. }));
    }

    #[test]
    fn test_lower_observe() {
        let surface = SurfaceWorkflow::Observe {
            capability: "read".into(),
            binding: Some(Pattern::Variable("x".into())),
            continuation: None,
            span: dummy_span(),
        };
        let core = lower_workflow_body(&surface, &Provenance::new());
        assert!(matches!(core, CoreWorkflow::Observe { .. }));
    }

    #[test]
    fn test_lower_orient() {
        let surface = SurfaceWorkflow::Orient {
            expr: SurfaceExpr::Literal(SurfaceLiteral::Int(42)),
            binding: None,
            continuation: None,
            span: dummy_span(),
        };
        let core = lower_workflow_body(&surface, &Provenance::new());
        assert!(matches!(core, CoreWorkflow::Orient { .. }));
    }

    #[test]
    fn test_lower_par() {
        let surface = SurfaceWorkflow::Par {
            branches: vec![
                SurfaceWorkflow::Done { span: dummy_span() },
                SurfaceWorkflow::Done { span: dummy_span() },
            ],
            span: dummy_span(),
        };
        let core = lower_workflow_body(&surface, &Provenance::new());
        assert!(matches!(core, CoreWorkflow::Par { workflows } if workflows.len() == 2));
    }

    #[test]
    fn test_lower_effect_type() {
        assert!(matches!(
            lower_effect_type(EffectType::Observe),
            Effect::Epistemic
        ));
        assert!(matches!(
            lower_effect_type(EffectType::Read),
            Effect::Epistemic
        ));
        assert!(matches!(
            lower_effect_type(EffectType::Analyze),
            Effect::Deliberative
        ));
        assert!(matches!(
            lower_effect_type(EffectType::Decide),
            Effect::Evaluative
        ));
        assert!(matches!(
            lower_effect_type(EffectType::Act),
            Effect::Operational
        ));
        assert!(matches!(
            lower_effect_type(EffectType::Write),
            Effect::Operational
        ));
        assert!(matches!(
            lower_effect_type(EffectType::External),
            Effect::Operational
        ));
    }
}
