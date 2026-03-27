//! Surface AST to Core IR lowering.
//!
//! This module converts the surface syntax AST into the core IR representation
//! used by the ash-core crate.

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
    ActionRef, BinaryOp, CapabilityDef, CheckTarget, EffectType, Expr, Guard, Literal,
    ObligationRef, Pattern, PolicyExpr, Predicate, StreamPattern, Type, UnaryOp,
    Workflow as SurfaceWorkflow, WorkflowDef, YieldArm,
};

#[cfg(test)]
use crate::surface::{Definition, RoleDef};

/// Error returned when lowering surface AST to core IR fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweringError {
    /// Float literals are not supported in the core IR.
    FloatNotSupported,
}

impl fmt::Display for LoweringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoweringError::FloatNotSupported => {
                write!(f, "float literals are not supported")
            }
        }
    }
}

impl std::error::Error for LoweringError {}

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
pub fn lower_workflow(def: &WorkflowDef) -> Result<CoreWorkflow, LoweringError> {
    // Create a provenance for the workflow
    let provenance = Provenance::new();

    lower_workflow_body(&def.body, &provenance)
}

/// Result of lowering a workflow definition with optional implicit role.
#[derive(Debug, Clone)]
pub struct LoweredWorkflow {
    /// The lowered workflow body
    pub workflow: CoreWorkflow,
    /// The implicit role generated from capabilities, if any
    pub implicit_role: Option<CoreRole>,
    /// The updated plays_roles (includes implicit role if generated)
    pub plays_roles: Vec<String>,
}

/// Lower a workflow definition with implicit role generation.
///
/// Per SPEC-024 Section 5.1: `capabilities: [...]` desugars to implicit role.
/// The implicit role name is `{workflow_name}_default`.
///
/// ```ash
/// -- Surface:
/// workflow X capabilities: [C1, C2] { ... }
///
/// -- Lowered:
/// role X_default { capabilities: [C1, C2] }
/// workflow X plays role(X_default) { ... }
/// ```
pub fn lower_workflow_def(def: &WorkflowDef) -> Result<LoweredWorkflow, LoweringError> {
    // Start with explicit plays_roles
    let mut plays_roles: Vec<String> = def.plays_roles.iter().map(|r| r.name.to_string()).collect();

    // Generate implicit role if capabilities are declared
    let implicit_role = if !def.capabilities.is_empty() {
        let role_name = generate_implicit_role_name(def.name.as_ref());

        let role = CoreRole {
            name: role_name.clone(),
            authority: def
                .capabilities
                .iter()
                .map(lower_capability_decl)
                .collect::<Result<Vec<_>, _>>()?,
            obligations: vec![],
        };

        // Add implicit role to workflow's plays_roles
        plays_roles.push(role_name);

        Some(role)
    } else {
        None
    };

    // Lower the workflow body
    let workflow = lower_workflow(def)?;

    Ok(LoweredWorkflow {
        workflow,
        implicit_role,
        plays_roles,
    })
}

/// Generate implicit role name for a workflow.
///
/// The implicit role name is `{workflow_name}_default`.
fn generate_implicit_role_name(workflow_name: &str) -> String {
    format!("{}_default", workflow_name)
}

/// Lower a capability declaration to core Capability.
fn lower_capability_decl(
    decl: &crate::surface::CapabilityDecl,
) -> Result<Capability, LoweringError> {
    Ok(Capability {
        name: decl.capability.to_string(),
        effect: Effect::Epistemic, // Default effect for workflow capabilities
        constraints: lower_capability_constraints(decl.constraints.as_ref())?,
    })
}

/// Lower capability constraints from surface to core.
fn lower_capability_constraints(
    constraints: Option<&crate::surface::ConstraintBlock>,
) -> Result<Vec<ash_core::Constraint>, LoweringError> {
    let Some(block) = constraints else {
        return Ok(vec![]);
    };

    block
        .fields
        .iter()
        .map(lower_constraint_field)
        .collect::<Result<Vec<_>, _>>()
}

/// Lower a constraint field to core Constraint.
fn lower_constraint_field(
    field: &crate::surface::ConstraintField,
) -> Result<ash_core::Constraint, LoweringError> {
    // Convert constraint value to predicate arguments
    let args = vec![lower_constraint_value(&field.value)?];

    Ok(ash_core::Constraint {
        predicate: ash_core::Predicate {
            name: field.name.to_string(),
            arguments: args,
        },
    })
}

/// Lower a constraint value to core expression.
fn lower_constraint_value(
    value: &crate::surface::ConstraintValue,
) -> Result<CoreExpr, LoweringError> {
    match value {
        crate::surface::ConstraintValue::Bool(b) => {
            Ok(CoreExpr::Literal(ash_core::Value::Bool(*b)))
        }
        crate::surface::ConstraintValue::Int(n) => Ok(CoreExpr::Literal(ash_core::Value::Int(*n))),
        crate::surface::ConstraintValue::String(s) => {
            Ok(CoreExpr::Literal(ash_core::Value::String(s.clone())))
        }
        crate::surface::ConstraintValue::Array(arr) => {
            let elements = arr
                .iter()
                .map(lower_constraint_value)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CoreExpr::Literal(ash_core::Value::List(Box::new(
                elements
                    .into_iter()
                    .map(|e| match e {
                        CoreExpr::Literal(v) => v,
                        _ => ash_core::Value::Null,
                    })
                    .collect(),
            ))))
        }
        crate::surface::ConstraintValue::Object(obj) => {
            // Objects are lowered as record literals (HashMap)
            use std::collections::HashMap;
            let mut fields = HashMap::new();
            for (k, v) in obj {
                let value = lower_constraint_value(v).map(|e| match e {
                    CoreExpr::Literal(v) => v,
                    _ => ash_core::Value::Null,
                })?;
                fields.insert(k.clone(), value);
            }
            Ok(CoreExpr::Literal(ash_core::Value::Record(Box::new(fields))))
        }
    }
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
                lower_capability_def(capability).ok()
            }
            _ => None,
        })
        .ok_or_else(|| RoleLoweringError {
            role: role_name.to_string(),
            authority: authority_name.to_string(),
        })
}

#[allow(dead_code)]
fn lower_capability_def(def: &CapabilityDef) -> Result<Capability, LoweringError> {
    Ok(Capability {
        name: def.name.to_string(),
        effect: lower_effect_type(def.effect),
        constraints: def
            .constraints
            .iter()
            .map(lower_constraint)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

#[allow(dead_code)]
fn lower_constraint(
    constraint: &crate::surface::Constraint,
) -> Result<ash_core::Constraint, LoweringError> {
    Ok(ash_core::Constraint {
        predicate: lower_predicate(&constraint.predicate)?,
    })
}

/// Lower a workflow body to core IR.
fn lower_workflow_body(
    workflow: &SurfaceWorkflow,
    provenance: &Provenance,
) -> Result<CoreWorkflow, LoweringError> {
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
                .transpose()?
                .unwrap_or(CorePattern::Wildcard);

            let cont = continuation
                .as_ref()
                .map(|c| lower_workflow_body(c, provenance))
                .transpose()?
                .unwrap_or(CoreWorkflow::Done);

            Ok(CoreWorkflow::Observe {
                capability: Capability {
                    name: capability.to_string(),
                    effect: Effect::Epistemic,
                    constraints: vec![],
                },
                pattern,
                continuation: Box::new(cont),
            })
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
                .transpose()?
                .unwrap_or(CoreWorkflow::Done);

            Ok(CoreWorkflow::Orient {
                expr: lower_expr(expr)?,
                continuation: Box::new(cont),
            })
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
                .transpose()?
                .unwrap_or(CoreWorkflow::Done);

            Ok(CoreWorkflow::Propose {
                action: lower_action(action)?,
                continuation: Box::new(cont),
            })
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

            Ok(CoreWorkflow::Decide {
                expr: lower_expr(expr)?,
                policy: policy
                    .as_ref()
                    .expect("canonical decide lowering requires an explicit named policy")
                    .to_string(),
                continuation: Box::new(lower_workflow_body(then_branch, provenance)?),
            })
        }

        SurfaceWorkflow::Check {
            target,
            continuation,
            ..
        } => {
            let cont = continuation
                .as_ref()
                .map(|c| lower_workflow_body(c, provenance))
                .transpose()?
                .unwrap_or(CoreWorkflow::Done);

            Ok(CoreWorkflow::Check {
                obligation: lower_check_target(target)?,
                continuation: Box::new(cont),
            })
        }

        SurfaceWorkflow::Oblige { obligation, .. } => Ok(CoreWorkflow::Oblige {
            name: obligation.to_string(),
            span: Default::default(),
        }),

        SurfaceWorkflow::Act { action, guard, .. } => Ok(CoreWorkflow::Act {
            action: lower_action(action)?,
            guard: guard
                .as_ref()
                .map(lower_guard)
                .transpose()?
                .unwrap_or(CoreGuard::Always),
            provenance: provenance.clone(),
        }),

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
                .transpose()?
                .unwrap_or(CoreWorkflow::Done);

            Ok(CoreWorkflow::Seq {
                first: Box::new(CoreWorkflow::Set {
                    capability: capability.to_string(),
                    channel: channel.to_string(),
                    value: lower_expr(value)?,
                }),
                second: Box::new(cont),
            })
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
                .transpose()?
                .unwrap_or(CoreWorkflow::Done);

            Ok(CoreWorkflow::Seq {
                first: Box::new(CoreWorkflow::Send {
                    capability: capability.to_string(),
                    channel: channel.to_string(),
                    value: lower_expr(value)?,
                }),
                second: Box::new(cont),
            })
        }

        SurfaceWorkflow::Receive {
            mode,
            arms,
            is_control,
            ..
        } => Ok(CoreWorkflow::Receive {
            mode: lower_receive_mode(mode),
            arms: arms
                .iter()
                .map(|arm| lower_receive_arm(arm, provenance))
                .collect::<Result<Vec<_>, _>>()?,
            control: *is_control,
        }),

        SurfaceWorkflow::Let {
            pattern,
            expr,
            continuation,
            ..
        } => {
            let cont = continuation
                .as_ref()
                .map(|c| lower_workflow_body(c, provenance))
                .transpose()?
                .unwrap_or(CoreWorkflow::Done);

            Ok(CoreWorkflow::Let {
                pattern: lower_pattern(pattern)?,
                expr: lower_expr(expr)?,
                continuation: Box::new(cont),
            })
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
                .transpose()?
                .unwrap_or(CoreWorkflow::Done);
            let then_wf = lower_workflow_body(then_branch, provenance)?;

            Ok(CoreWorkflow::If {
                condition: lower_expr(condition)?,
                then_branch: Box::new(then_wf),
                else_branch: Box::new(else_wf),
            })
        }

        SurfaceWorkflow::For {
            pattern,
            collection,
            body,
            ..
        } => Ok(CoreWorkflow::ForEach {
            pattern: lower_pattern(pattern)?,
            collection: lower_expr(collection)?,
            body: Box::new(lower_workflow_body(body, provenance)?),
        }),

        SurfaceWorkflow::Par { branches, .. } => {
            let workflows: Vec<_> = branches
                .iter()
                .map(|b| lower_workflow_body(b, provenance))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(CoreWorkflow::Par { workflows })
        }

        SurfaceWorkflow::With {
            capability, body, ..
        } => Ok(CoreWorkflow::With {
            capability: Capability {
                name: capability.to_string(),
                effect: Effect::Epistemic,
                constraints: vec![],
            },
            workflow: Box::new(lower_workflow_body(body, provenance)?),
        }),

        SurfaceWorkflow::Maybe {
            primary, fallback, ..
        } => Ok(CoreWorkflow::Maybe {
            primary: Box::new(lower_workflow_body(primary, provenance)?),
            fallback: Box::new(lower_workflow_body(fallback, provenance)?),
        }),

        SurfaceWorkflow::Must { body, .. } => Ok(CoreWorkflow::Must {
            workflow: Box::new(lower_workflow_body(body, provenance)?),
        }),

        SurfaceWorkflow::Seq { first, second, .. } => Ok(CoreWorkflow::Seq {
            first: Box::new(lower_workflow_body(first, provenance)?),
            second: Box::new(lower_workflow_body(second, provenance)?),
        }),

        SurfaceWorkflow::Done { .. } => Ok(CoreWorkflow::Done),

        SurfaceWorkflow::Ret { expr, .. } => Ok(CoreWorkflow::Ret {
            expr: lower_expr(expr)?,
        }),

        // Proxy workflow constructs
        SurfaceWorkflow::Yield {
            role,
            expr,
            resume_var,
            resume_type,
            arms,
            span,
        } => {
            // Lower the request expression
            let request = Box::new(lower_expr(expr)?);

            // Convert surface Type to core TypeExpr
            let expected_response_type = lower_type_to_type_expr(resume_type);

            // Lower the yield arms into a continuation workflow
            let continuation = Box::new(lower_yield_arms(resume_var, arms, provenance)?);

            // Convert surface span to core span
            let core_span = ash_core::Span {
                start: span.start,
                end: span.end,
            };

            Ok(CoreWorkflow::Yield {
                role: role.to_string(),
                request,
                expected_response_type,
                continuation,
                span: core_span,
                resume_var: resume_var.to_string(),
            })
        }

        SurfaceWorkflow::Resume { expr, .. } => Ok(CoreWorkflow::Ret {
            expr: lower_expr(expr)?,
        }),
    }
}

/// Convert a surface Type to a core TypeExpr.
fn lower_type_to_type_expr(ty: &Type) -> ash_core::workflow_contract::TypeExpr {
    use ash_core::workflow_contract::TypeExpr;
    match ty {
        Type::Name(name) => TypeExpr::Named(name.to_string()),
        Type::List(inner) => TypeExpr::Constructor {
            name: "List".to_string(),
            args: vec![lower_type_to_type_expr(inner)],
        },
        Type::Record(fields) => TypeExpr::Constructor {
            name: "Record".to_string(),
            args: fields
                .iter()
                .map(|(_, t)| lower_type_to_type_expr(t))
                .collect(),
        },
        Type::Capability(name) => TypeExpr::Constructor {
            name: "Capability".to_string(),
            args: vec![TypeExpr::Named(name.to_string())],
        },
        Type::Constructor { name, args } => TypeExpr::Constructor {
            name: name.to_string(),
            args: args.iter().map(lower_type_to_type_expr).collect(),
        },
    }
}

/// Lower yield arms into a continuation workflow.
///
/// The resume_var is bound to the response value, and then the arms
/// are processed as pattern matches.
fn lower_yield_arms(
    resume_var: &str,
    arms: &[YieldArm],
    provenance: &Provenance,
) -> Result<CoreWorkflow, LoweringError> {
    if arms.is_empty() {
        return Ok(CoreWorkflow::Done);
    }

    // Convert the arms into a match expression
    // For now, we create a Let binding for the resume variable
    // followed by the body of the first arm (single arm case)
    // or a series of If expressions for multiple arms

    if arms.len() == 1 {
        // Single arm: bind the pattern and execute the body
        let arm = &arms[0];
        Ok(CoreWorkflow::Let {
            pattern: lower_pattern(&arm.pattern)?,
            expr: CoreExpr::Variable(resume_var.to_string()),
            continuation: Box::new(lower_workflow_body(&arm.body, provenance)?),
        })
    } else {
        // Multiple arms: create a cascade of If expressions
        // For now, use the first arm's pattern as the main match
        // and subsequent arms as fallbacks
        let first_arm = &arms[0];
        let _rest_continuation = if arms.len() > 1 {
            lower_yield_arms(resume_var, &arms[1..], provenance)?
        } else {
            CoreWorkflow::Done
        };

        Ok(CoreWorkflow::Let {
            pattern: lower_pattern(&first_arm.pattern)?,
            expr: CoreExpr::Variable(resume_var.to_string()),
            continuation: Box::new(lower_workflow_body(&first_arm.body, provenance)?),
        })
    }
}

/// Lower a surface expression to core IR.
pub fn lower_expr(expr: &Expr) -> Result<CoreExpr, LoweringError> {
    match expr {
        Expr::Literal(lit) => Ok(CoreExpr::Literal(lower_literal(lit)?)),

        Expr::Variable(name) => Ok(CoreExpr::Variable(name.to_string())),

        Expr::FieldAccess { base, field, .. } => Ok(CoreExpr::FieldAccess {
            expr: Box::new(lower_expr(base)?),
            field: field.to_string(),
        }),

        Expr::IndexAccess { base, index, .. } => Ok(CoreExpr::IndexAccess {
            expr: Box::new(lower_expr(base)?),
            index: Box::new(lower_expr(index)?),
        }),

        Expr::Unary { op, operand, .. } => Ok(CoreExpr::Unary {
            op: lower_unary_op(*op),
            expr: Box::new(lower_expr(operand)?),
        }),

        Expr::Binary {
            op, left, right, ..
        } => Ok(CoreExpr::Binary {
            op: lower_binary_op(*op),
            left: Box::new(lower_expr(left)?),
            right: Box::new(lower_expr(right)?),
        }),

        Expr::Call { func, args, .. } => Ok(CoreExpr::Call {
            func: func.to_string(),
            arguments: args.iter().map(lower_expr).collect::<Result<Vec<_>, _>>()?,
        }),

        Expr::Match {
            scrutinee, arms, ..
        } => Ok(CoreExpr::Match {
            scrutinee: Box::new(lower_expr(scrutinee)?),
            arms: arms
                .iter()
                .map(|arm| {
                    Ok(CoreMatchArm {
                        pattern: lower_pattern(&arm.pattern)?,
                        body: lower_expr(&arm.body)?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        }),

        Expr::Policy(policy_expr) => Ok(lower_policy_expr(policy_expr)),

        Expr::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
            ..
        } => Ok(CoreExpr::IfLet {
            pattern: lower_pattern(pattern)?,
            expr: Box::new(lower_expr(expr)?),
            then_branch: Box::new(lower_expr(then_branch)?),
            else_branch: Box::new(lower_expr(else_branch)?),
        }),

        Expr::CheckObligation { obligation, span } => Ok(CoreExpr::CheckObligation {
            obligation: obligation.to_string(),
            span: ash_core::Span {
                start: span.start,
                end: span.end,
            },
        }),

        Expr::Constructor { name, fields, .. } => Ok(CoreExpr::Constructor {
            name: name.to_string(),
            fields: fields
                .iter()
                .map(|(n, e)| Ok((n.to_string(), lower_expr(e)?)))
                .collect::<Result<_, _>>()?,
        }),
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

fn lower_receive_arm(
    arm: &crate::surface::ReceiveArm,
    provenance: &Provenance,
) -> Result<CoreReceiveArm, LoweringError> {
    Ok(CoreReceiveArm {
        pattern: lower_receive_pattern(&arm.pattern)?,
        guard: arm.guard.as_ref().map(lower_expr).transpose()?,
        body: lower_workflow_body(&arm.body, provenance)?,
    })
}

fn lower_receive_pattern(pattern: &StreamPattern) -> Result<CoreReceivePattern, LoweringError> {
    match pattern {
        StreamPattern::Binding {
            capability,
            channel,
            pattern,
        } => Ok(CoreReceivePattern::Stream {
            capability: capability.to_string(),
            channel: channel.to_string(),
            pattern: lower_pattern(pattern)?,
        }),
        StreamPattern::Literal(value) => Ok(CoreReceivePattern::Literal(ash_core::Value::String(
            value.to_string(),
        ))),
        StreamPattern::Wildcard => Ok(CoreReceivePattern::Wildcard),
    }
}

/// Lower a check target to core IR.
fn lower_check_target(target: &CheckTarget) -> Result<CoreObligation, LoweringError> {
    match target {
        CheckTarget::Obligation(obl) => lower_obligation(obl),
        CheckTarget::Policy(_) => panic!("policy instances are not valid canonical check targets"),
    }
}

/// Lower a literal value.
fn lower_literal(lit: &Literal) -> Result<ash_core::Value, LoweringError> {
    use ash_core::Value;

    match lit {
        Literal::Int(n) => Ok(Value::Int(*n)),
        Literal::Float(_) => Err(LoweringError::FloatNotSupported),
        Literal::String(s) => Ok(Value::String(s.to_string())),
        Literal::Bool(b) => Ok(Value::Bool(*b)),
        Literal::Null => Ok(Value::Null),
        Literal::List(elements) => {
            let lowered: Vec<_> = elements
                .iter()
                .map(lower_literal)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Value::List(Box::new(lowered)))
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
pub fn lower_pattern(pattern: &Pattern) -> Result<CorePattern, LoweringError> {
    match pattern {
        Pattern::Variable(name) => Ok(CorePattern::Variable(name.to_string())),

        Pattern::Wildcard => Ok(CorePattern::Wildcard),

        Pattern::Tuple(patterns) => {
            let lowered: Result<Vec<_>, _> = patterns.iter().map(lower_pattern).collect();
            Ok(CorePattern::Tuple(lowered?))
        }

        Pattern::Record(fields) => {
            let lowered: Result<Vec<_>, _> = fields
                .iter()
                .map(|(name, pat)| Ok((name.to_string(), lower_pattern(pat)?)))
                .collect();
            Ok(CorePattern::Record(lowered?))
        }

        Pattern::List { elements, rest } => {
            let lowered: Result<Vec<_>, _> = elements.iter().map(lower_pattern).collect();
            Ok(CorePattern::List(
                lowered?,
                rest.as_ref().map(|r| r.to_string()),
            ))
        }

        Pattern::Variant { name, fields } => Ok(CorePattern::Variant {
            name: name.to_string(),
            fields: fields
                .as_ref()
                .map(|fs| {
                    fs.iter()
                        .map(|(n, p)| Ok((n.to_string(), lower_pattern(p)?)))
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?,
        }),

        Pattern::Literal(lit) => Ok(CorePattern::Literal(lower_literal(lit)?)),
    }
}

/// Lower an action reference to core Action.
fn lower_action(action: &ActionRef) -> Result<CoreAction, LoweringError> {
    Ok(CoreAction {
        name: action.name.to_string(),
        arguments: action
            .args
            .iter()
            .map(lower_expr)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

/// Lower an obligation reference to core Obligation.
fn lower_obligation(obligation: &ObligationRef) -> Result<CoreObligation, LoweringError> {
    Ok(CoreObligation::Obliged {
        role: CoreRole {
            name: obligation.role.to_string(),
            authority: vec![],
            obligations: vec![],
        },
        condition: lower_expr(&obligation.condition)?,
    })
}

#[cfg(test)]
fn lower_role_obligation_name(name: &str) -> CoreRoleObligationRef {
    CoreRoleObligationRef {
        name: name.to_string(),
    }
}

/// Lower a guard to core IR.
fn lower_guard(guard: &Guard) -> Result<CoreGuard, LoweringError> {
    match guard {
        Guard::Always => Ok(CoreGuard::Always),
        Guard::Never => Ok(CoreGuard::Never),
        Guard::Pred(pred) => Ok(CoreGuard::Pred(lower_predicate(pred)?)),
        Guard::And(left, right) => Ok(CoreGuard::And(
            Box::new(lower_guard(left)?),
            Box::new(lower_guard(right)?),
        )),
        Guard::Or(left, right) => Ok(CoreGuard::Or(
            Box::new(lower_guard(left)?),
            Box::new(lower_guard(right)?),
        )),
        Guard::Not(inner) => Ok(CoreGuard::Not(Box::new(lower_guard(inner)?))),
    }
}

/// Lower a predicate to core IR.
fn lower_predicate(pred: &Predicate) -> Result<CorePredicate, LoweringError> {
    Ok(CorePredicate {
        name: pred.name.to_string(),
        arguments: pred
            .args
            .iter()
            .map(lower_expr)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

/// Lower an effect type to core Effect.
#[allow(dead_code)]
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
        let core = lower_workflow_body(&surface, &Provenance::new()).unwrap();
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
        let core = lower_workflow_body(&surface, &Provenance::new()).unwrap();
        assert!(matches!(core, CoreWorkflow::Let { .. }));
    }

    #[test]
    fn test_lower_expr_literal() {
        let surface = SurfaceExpr::Literal(SurfaceLiteral::Int(42));
        let core = lower_expr(&surface).unwrap();
        assert!(matches!(core, CoreExpr::Literal(ash_core::Value::Int(42))));
    }

    #[test]
    fn test_lower_expr_variable() {
        let surface = SurfaceExpr::Variable("my_var".into());
        let core = lower_expr(&surface).unwrap();
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
        let core = lower_expr(&surface).unwrap();
        assert!(matches!(
            core,
            CoreExpr::Binary {
                op: ash_core::BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_lower_expr_float_literal_error() {
        let surface = SurfaceExpr::Literal(SurfaceLiteral::Float(3.14));
        let result = lower_expr(&surface);
        assert!(matches!(result, Err(LoweringError::FloatNotSupported)));
    }

    #[test]
    fn test_lower_pattern_variable() {
        let surface = Pattern::Variable("x".into());
        let core = lower_pattern(&surface).unwrap();
        assert!(matches!(core, CorePattern::Variable(name) if name == "x"));
    }

    #[test]
    fn test_lower_pattern_wildcard() {
        let surface = Pattern::Wildcard;
        let core = lower_pattern(&surface).unwrap();
        assert!(matches!(core, CorePattern::Wildcard));
    }

    #[test]
    fn test_lower_pattern_tuple() {
        let surface = Pattern::Tuple(vec![
            Pattern::Variable("a".into()),
            Pattern::Variable("b".into()),
        ]);
        let core = lower_pattern(&surface).unwrap();
        assert!(matches!(core, CorePattern::Tuple(pats) if pats.len() == 2));
    }

    #[test]
    fn test_lower_literal_int() {
        let surface = SurfaceLiteral::Int(42);
        let core = lower_literal(&surface).unwrap();
        assert!(matches!(core, ash_core::Value::Int(42)));
    }

    #[test]
    fn test_lower_literal_string() {
        let surface = SurfaceLiteral::String("hello".into());
        let core = lower_literal(&surface).unwrap();
        assert!(matches!(core, ash_core::Value::String(s) if s == "hello"));
    }

    #[test]
    fn test_lower_obligation_uses_simplified_role_shape() {
        let surface = ObligationRef {
            role: "manager".into(),
            condition: SurfaceExpr::Variable("approved".into()),
        };

        let core = lower_obligation(&surface).unwrap();

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
        let core = lower_workflow_body(&surface, &Provenance::new()).unwrap();
        assert!(matches!(core, CoreWorkflow::If { .. }));
    }

    #[test]
    fn test_lower_seq() {
        let surface = SurfaceWorkflow::Seq {
            first: Box::new(SurfaceWorkflow::Done { span: dummy_span() }),
            second: Box::new(SurfaceWorkflow::Done { span: dummy_span() }),
            span: dummy_span(),
        };
        let core = lower_workflow_body(&surface, &Provenance::new()).unwrap();
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
        let core = lower_workflow_body(&surface, &Provenance::new()).unwrap();
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
        let core = lower_workflow_body(&surface, &Provenance::new()).unwrap();
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
        let core = lower_workflow_body(&surface, &Provenance::new()).unwrap();
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
