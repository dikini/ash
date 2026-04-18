//! Surface AST to Core IR lowering.
//!
//! This module converts the surface syntax AST into the core IR representation
//! used by the ash-core crate.

use std::fmt;

use ash_core::adt::tuple_field_name;
use ash_core::{
    Capability, Effect, Expr as CoreExpr, Guard as CoreGuard, MatchArm as CoreMatchArm,
    Obligation as CoreObligation, Pattern as CorePattern, Predicate as CorePredicate, Provenance,
    ReceiveArm as CoreReceiveArm, ReceivePattern as CoreReceivePattern, Role as CoreRole,
    Workflow as CoreWorkflow,
};

#[cfg(test)]
use ash_core::RoleObligationRef as CoreRoleObligationRef;

use crate::capability_export::{CapabilityResolutionContext, ModuleId};
use crate::surface::{
    BinaryOp, CapabilityDef, CheckTarget, EffectType, Expr, Guard, Literal, ObligationRef, Pattern,
    PolicyExpr, Predicate, StreamPattern, Type, UnaryOp, Workflow as SurfaceWorkflow, WorkflowDef,
    YieldArm,
};

/// Context for lowering workflows with capability resolution.
///
/// This carries the module-owned capability resolution context that maps
/// symbolic capability names to (provider, action) pairs.
#[derive(Debug, Clone, Default)]
pub struct LoweringContext {
    /// Capability resolution context from module/import pipeline.
    pub capability_context: Option<CapabilityResolutionContext>,
    /// Current module ID for module-scoped capability resolution.
    pub current_module: Option<ModuleId>,
    /// Set of unqualified function names that are known to be effectful,
    /// derived from declared capability definitions.
    pub effectful_names: std::collections::HashSet<String>,
}

impl LoweringContext {
    /// Create a new lowering context without capability resolution.
    pub fn new() -> Self {
        Self {
            capability_context: None,
            current_module: None,
            effectful_names: std::collections::HashSet::new(),
        }
    }

    /// Create a new lowering context with capability resolution.
    pub fn with_capability_context(capability_context: CapabilityResolutionContext) -> Self {
        Self {
            capability_context: Some(capability_context),
            current_module: None,
            effectful_names: std::collections::HashSet::new(),
        }
    }

    /// Create a new lowering context with capability resolution for a specific module.
    pub fn with_capability_context_for_module(
        capability_context: CapabilityResolutionContext,
        module_id: ModuleId,
    ) -> Self {
        Self {
            capability_context: Some(capability_context),
            current_module: Some(module_id),
            effectful_names: std::collections::HashSet::new(),
        }
    }

    /// Create a new lowering context with explicit effectful names set.
    pub fn with_effectful_names(effectful_names: std::collections::HashSet<String>) -> Self {
        Self {
            capability_context: None,
            current_module: None,
            effectful_names,
        }
    }

    /// Check if capability resolution is available.
    pub fn has_capability_context(&self) -> bool {
        self.capability_context.is_some()
    }

    /// Resolve a symbolic capability name to (provider, action).
    ///
    /// Returns None if:
    /// - No capability context is available
    /// - No current module is set
    /// - The name is not found in the resolution context
    pub fn resolve_capability(&self, name: &str) -> Option<(String, String)> {
        let context = self.capability_context.as_ref()?;
        let module_id = self.current_module?;
        context.resolve_unqualified(module_id, name)
    }

    /// Resolve a module-qualified capability name to (provider, action).
    ///
    /// This is used for qualified capability calls like `module::capability()`.
    /// It uses the dedicated qualified resolution API instead of building a
    /// combined string for unqualified lookup.
    ///
    /// Returns None if:
    /// - No capability context is available
    /// - The module name is not registered
    /// - The capability is not found in the target module
    pub fn resolve_qualified(
        &self,
        module_name: &str,
        capability_name: &str,
    ) -> Option<(String, String)> {
        let context = self.capability_context.as_ref()?;
        context.resolve_qualified_to_strings(module_name, capability_name)
    }
}

#[cfg(test)]
use crate::surface::{Definition, RoleDef};

/// Error returned when lowering surface AST to core IR fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweringError {
    /// Float literals are not supported in the core IR.
    FloatNotSupported,
    /// Symbolic capability name could not be resolved to a (provider, action) pair.
    UnresolvedCapability { name: String },
    /// Expression form is not valid at this position.
    ExprNotLowerable { kind: &'static str },
    /// fn expression appeared at module scope where only named `pub fn` is allowed.
    FnDefNotAllowedAtModuleScope,
    /// An invalid target was encountered during lowering.
    InvalidTarget(String),
    /// A feature that should have been handled earlier was encountered during lowering.
    UnsupportedFeature(String),
}

impl fmt::Display for LoweringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoweringError::FloatNotSupported => {
                write!(f, "float literals are not supported")
            }
            LoweringError::UnresolvedCapability { name } => {
                write!(
                    f,
                    "unresolved symbolic capability '{}': no (provider, action) mapping found",
                    name
                )
            }
            LoweringError::ExprNotLowerable { kind } => {
                write!(f, "expression form `{kind}` is not valid at this position")
            }
            LoweringError::FnDefNotAllowedAtModuleScope => {
                write!(
                    f,
                    "fn expressions are not valid at module scope; use `pub fn` instead"
                )
            }
            LoweringError::InvalidTarget(msg) => {
                write!(f, "invalid target: {msg}")
            }
            LoweringError::UnsupportedFeature(msg) => {
                write!(f, "unsupported feature: {msg}")
            }
        }
    }
}

impl std::error::Error for LoweringError {}

/// Error returned when lowering a pure-function contract to the core contract subset fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FnContractLoweringError {
    InvalidRequires { message: String },
    InvalidEnsures { message: String },
}

impl fmt::Display for FnContractLoweringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FnContractLoweringError::InvalidRequires { message } => {
                write!(f, "invalid fn requires clause: {message}")
            }
            FnContractLoweringError::InvalidEnsures { message } => {
                write!(f, "invalid fn ensures clause: {message}")
            }
        }
    }
}

impl std::error::Error for FnContractLoweringError {}

/// Lowered fn contract together with the explicit runtime postcondition boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct LoweredFnContract {
    pub contract: ash_core::workflow_contract::Contract,
    pub runtime_postconditions: ash_core::workflow_contract::RuntimePostconditionContract,
}

/// Lower a parsed fn contract into the core contract subset used by type checking and runtime
/// postcondition evaluation.
pub fn lower_fn_contract(
    contract: Option<&crate::surface::Contract>,
) -> Result<LoweredFnContract, FnContractLoweringError> {
    let Some(contract) = contract else {
        return Ok(LoweredFnContract {
            contract: ash_core::workflow_contract::Contract::default(),
            runtime_postconditions:
                ash_core::workflow_contract::RuntimePostconditionContract::default(),
        });
    };

    let requires = contract
        .requires
        .iter()
        .map(lower_fn_requirement)
        .collect::<Result<Vec<_>, _>>()?;
    let ensures = contract
        .ensures
        .iter()
        .map(lower_fn_ensures_clause)
        .collect::<Result<Vec<_>, _>>()?;

    let runtime_postconditions = ash_core::workflow_contract::RuntimePostconditionContract {
        predicates: ensures.clone(),
    };

    Ok(LoweredFnContract {
        contract: ash_core::workflow_contract::Contract { requires, ensures },
        runtime_postconditions,
    })
}

fn lower_fn_requirement(
    requirement: &crate::surface::Requirement,
) -> Result<ash_core::workflow_contract::Requirement, FnContractLoweringError> {
    match requirement {
        crate::surface::Requirement::Arithmetic { expr } => {
            let (var, constraint) = lower_stage1_arith_predicate(expr)
                .map_err(|message| FnContractLoweringError::InvalidRequires { message })?;
            Ok(ash_core::workflow_contract::Requirement::Arithmetic { var, constraint })
        }
        crate::surface::Requirement::HasCapability { .. } => {
            Err(FnContractLoweringError::InvalidRequires {
                message: "fn contracts cannot reference capabilities".to_string(),
            })
        }
        crate::surface::Requirement::HasRole(_) => Err(FnContractLoweringError::InvalidRequires {
            message: "fn contracts cannot reference roles".to_string(),
        }),
    }
}

fn lower_fn_ensures_clause(
    clause: &crate::surface::EnsuresClause,
) -> Result<ash_core::workflow_contract::PostPredicate, FnContractLoweringError> {
    if let Some(constraint) = lower_result_constraint(&clause.expr) {
        return Ok(ash_core::workflow_contract::PostPredicate::ResultSatisfies(
            constraint,
        ));
    }

    if let Some((left, right)) = lower_result_equality(&clause.expr) {
        return Ok(ash_core::workflow_contract::PostPredicate::Eq(left, right));
    }

    Err(FnContractLoweringError::InvalidEnsures {
        message: "fn ensures clauses must be value-level predicates over `result` or simple equality; state assertions are not allowed".to_string(),
    })
}

fn lower_stage1_arith_predicate(
    expr: &Expr,
) -> Result<(String, ash_core::workflow_contract::ArithConstraint), String> {
    match expr {
        Expr::Binary {
            op, left, right, ..
        } => {
            if let (Some(var), Some(value)) = (variable_name(left), int_literal(right)) {
                let constraint = match op {
                    BinaryOp::Gt => ash_core::workflow_contract::ArithConstraint::Gt(value),
                    BinaryOp::Lt => ash_core::workflow_contract::ArithConstraint::Lt(value),
                    BinaryOp::Geq => ash_core::workflow_contract::ArithConstraint::Gte(value),
                    BinaryOp::Leq => ash_core::workflow_contract::ArithConstraint::Lte(value),
                    BinaryOp::Eq => ash_core::workflow_contract::ArithConstraint::Eq(value),
                    BinaryOp::Neq => ash_core::workflow_contract::ArithConstraint::NotEq(value),
                    _ => {
                        return Err(format!(
                            "unsupported Stage 1 arithmetic predicate: {expr:?}"
                        ));
                    }
                };
                return Ok((var.to_string(), constraint));
            }

            if *op == BinaryOp::Eq {
                if let (Some((var, div)), Some(rem)) = (modulo_operand(left), int_literal(right)) {
                    return Ok((
                        var.to_string(),
                        ash_core::workflow_contract::ArithConstraint::Modulo { div, rem },
                    ));
                }
                if let (Some(rem), Some((var, div))) = (int_literal(left), modulo_operand(right)) {
                    return Ok((
                        var.to_string(),
                        ash_core::workflow_contract::ArithConstraint::Modulo { div, rem },
                    ));
                }
            }

            Err(format!(
                "unsupported Stage 1 arithmetic predicate: {expr:?}"
            ))
        }
        _ => Err(format!(
            "unsupported Stage 1 arithmetic predicate: {expr:?}"
        )),
    }
}

fn lower_result_constraint(expr: &Expr) -> Option<ash_core::workflow_contract::ArithConstraint> {
    let (var, constraint) = lower_stage1_arith_predicate(expr).ok()?;
    (var == "result").then_some(constraint)
}

fn lower_result_equality(expr: &Expr) -> Option<(String, String)> {
    let Expr::Binary {
        op: BinaryOp::Eq,
        left,
        right,
        ..
    } = expr
    else {
        return None;
    };

    if variable_name(left) == Some("result") {
        return Some(("result".to_string(), simple_value_expr(right)?));
    }

    if variable_name(right) == Some("result") {
        return Some(("result".to_string(), simple_value_expr(left)?));
    }

    None
}

fn simple_value_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Variable { name, .. } => Some(name.to_string()),
        Expr::Literal(Literal::Int(value)) => Some(value.to_string()),
        Expr::Literal(Literal::String(value)) => Some(format!("\"{value}\"")),
        Expr::Literal(Literal::Bool(value)) => Some(value.to_string()),
        Expr::Literal(Literal::Null) => Some("null".to_string()),
        _ => None,
    }
}

fn variable_name(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Variable { name, .. } => Some(name.as_ref()),
        _ => None,
    }
}

fn int_literal(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::Literal(Literal::Int(value)) => Some(*value),
        _ => None,
    }
}

fn modulo_operand(expr: &Expr) -> Option<(&str, i64)> {
    let Expr::Binary {
        op: BinaryOp::Mod,
        left,
        right,
        ..
    } = expr
    else {
        return None;
    };

    Some((variable_name(left)?, int_literal(right)?))
}

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

/// Extract effectful names from a slice of surface definitions.
///
/// Capability definitions with operational effects (Act, Write, External, Operational)
/// or with a `target_action` are considered effectful. Both the capability name
/// and the target action name (if different) are registered.
pub fn effectful_names_from_definitions(
    definitions: &[crate::surface::Definition],
) -> std::collections::HashSet<String> {
    let mut names = std::collections::HashSet::new();
    for def in definitions {
        if let crate::surface::Definition::Capability(cap_def) = def {
            if cap_def.target_action.is_some()
                || matches!(
                    cap_def.effect,
                    crate::surface::EffectType::Act
                        | crate::surface::EffectType::Write
                        | crate::surface::EffectType::External
                        | crate::surface::EffectType::Operational
                )
            {
                names.insert(cap_def.name.to_string());
            }
            if let Some(action) = &cap_def.target_action {
                names.insert(action.to_string());
            }
        }
    }
    names
}

/// Lower a workflow definition to core IR.
pub fn lower_workflow(def: &WorkflowDef) -> Result<CoreWorkflow, LoweringError> {
    lower_workflow_with_context(def, &LoweringContext::new())
}

/// Lower a workflow definition to core IR with capability resolution context.
pub fn lower_workflow_with_context(
    def: &WorkflowDef,
    ctx: &LoweringContext,
) -> Result<CoreWorkflow, LoweringError> {
    // Create a provenance for the workflow
    let provenance = Provenance::new();

    let core = lower_workflow_body(&def.body, &provenance, ctx)?;
    Ok(crate::lift::lift_workflow_with_names(
        core,
        &ctx.effectful_names,
    ))
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
            .capabilities
            .iter()
            .map(|cap| lower_role_capability(def.name.as_ref(), cap, definitions))
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
fn lower_role_capability(
    role_name: &str,
    cap_decl: &crate::surface::CapabilityDecl,
    definitions: &[Definition],
) -> Result<Capability, RoleLoweringError> {
    let cap_name = cap_decl.capability.as_ref();
    definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Capability(capability) if capability.name.as_ref() == cap_name => {
                lower_capability_def(capability).ok()
            }
            _ => None,
        })
        .ok_or_else(|| RoleLoweringError {
            role: role_name.to_string(),
            authority: cap_name.to_string(),
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
fn is_self_terminating_surface_workflow(workflow: &SurfaceWorkflow) -> bool {
    match workflow {
        SurfaceWorkflow::Ret { .. } | SurfaceWorkflow::Done { .. } => true,
        SurfaceWorkflow::Let { continuation, .. }
        | SurfaceWorkflow::Observe { continuation, .. }
        | SurfaceWorkflow::Orient { continuation, .. }
        | SurfaceWorkflow::Propose { continuation, .. } => continuation
            .as_deref()
            .is_some_and(is_self_terminating_surface_workflow),
        SurfaceWorkflow::Check { continuation, .. } => continuation
            .as_deref()
            .is_none_or(is_self_terminating_surface_workflow),
        SurfaceWorkflow::Act {
            result_name,
            continuation,
            ..
        } => {
            result_name.is_none()
                || continuation
                    .as_deref()
                    .is_some_and(is_self_terminating_surface_workflow)
        }
        SurfaceWorkflow::Decide {
            then_branch,
            else_branch,
            ..
        }
        | SurfaceWorkflow::If {
            then_branch,
            else_branch,
            ..
        } => {
            is_self_terminating_surface_workflow(then_branch)
                && else_branch
                    .as_deref()
                    .is_none_or(is_self_terminating_surface_workflow)
        }
        SurfaceWorkflow::Receive { arms, .. } => arms
            .iter()
            .all(|arm| is_self_terminating_surface_workflow(&arm.body)),
        _ => false,
    }
}

fn lower_workflow_body(
    workflow: &SurfaceWorkflow,
    provenance: &Provenance,
    ctx: &LoweringContext,
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
                .map(|c| lower_workflow_body(c, provenance, ctx))
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
                .map(|c| lower_workflow_body(c, provenance, ctx))
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
                .map(|c| lower_workflow_body(c, provenance, ctx))
                .transpose()?
                .unwrap_or(CoreWorkflow::Done);

            // Extract action name from the operational target
            let action_name = match &action.target {
                crate::surface::OperationalTarget::Symbolic { capability_name } => {
                    capability_name.to_string()
                }
                crate::surface::OperationalTarget::Qualified {
                    module,
                    capability_name,
                } => {
                    format!("{}::{}", module, capability_name)
                }
                crate::surface::OperationalTarget::Explicit { provider, action } => {
                    format!("{}:{}", provider, action)
                }
            };

            Ok(CoreWorkflow::Propose {
                action_name,
                action_arguments: action
                    .args
                    .iter()
                    .map(lower_expr)
                    .collect::<Result<Vec<_>, _>>()?,
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
            if else_branch.is_some() {
                return Err(LoweringError::InvalidTarget(
                    "legacy decide else-branches are not part of the canonical lowering contract"
                        .to_string(),
                ));
            }

            Ok(CoreWorkflow::Decide {
                expr: lower_expr(expr)?,
                policy: policy
                    .as_ref()
                    .ok_or_else(|| {
                        LoweringError::InvalidTarget(
                            "canonical decide lowering requires an explicit named policy"
                                .to_string(),
                        )
                    })?
                    .to_string(),
                continuation: Box::new(lower_workflow_body(then_branch, provenance, ctx)?),
            })
        }

        SurfaceWorkflow::Check {
            target,
            continuation,
            ..
        } => {
            let cont = continuation
                .as_ref()
                .map(|c| lower_workflow_body(c, provenance, ctx))
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

        SurfaceWorkflow::Act {
            action,
            guard,
            result_name,
            continuation,
            ..
        } => {
            // Resolve symbolic/qualified names to (provider, action) pairs using the
            // module-owned capability resolution context. Explicit provider:action
            // calls bypass resolution and use the specified target directly.
            let (provider_name, action_name) = match &action.target {
                crate::surface::OperationalTarget::Symbolic { capability_name } => {
                    // Symbolic capability call - resolve through module-owned context
                    // Per Phase 71: use passed-in context, not built-in mappings
                    match ctx.resolve_capability(capability_name.as_ref()) {
                        Some((provider, action)) => (provider, action),
                        None => {
                            return Err(LoweringError::UnresolvedCapability {
                                name: capability_name.to_string(),
                            });
                        }
                    }
                }
                crate::surface::OperationalTarget::Qualified {
                    module,
                    capability_name,
                } => {
                    // Module-qualified symbolic call: module::capability
                    // Use the dedicated qualified resolution API (Phase 72 fix)
                    // This properly resolves through the target module's exports
                    // rather than building a string for unqualified lookup.
                    match ctx.resolve_qualified(module.as_ref(), capability_name.as_ref()) {
                        Some((provider, action)) => (provider, action),
                        None => {
                            let qualified_name = format!("{}::{}", module, capability_name);
                            return Err(LoweringError::UnresolvedCapability {
                                name: qualified_name,
                            });
                        }
                    }
                }
                crate::surface::OperationalTarget::Explicit { provider, action } => {
                    // Explicit provider:action call - use as-is, bypass resolution
                    (provider.to_string(), action.to_string())
                }
            };

            let cont = continuation
                .as_ref()
                .map(|c| lower_workflow_body(c, provenance, ctx))
                .transpose()?
                .unwrap_or(CoreWorkflow::Done);

            Ok(CoreWorkflow::Act {
                provider_name,
                action_name,
                arguments: action
                    .args
                    .iter()
                    .map(lower_expr)
                    .collect::<Result<Vec<_>, _>>()?,
                guard: guard
                    .as_ref()
                    .map(lower_guard)
                    .transpose()?
                    .unwrap_or(CoreGuard::Always),
                provenance: provenance.clone(),
                result_name: result_name.as_ref().map(|n| n.to_string()),
                continuation: Box::new(cont),
            })
        }

        SurfaceWorkflow::Set {
            capability,
            channel,
            value,
            continuation,
            ..
        } => {
            let cont = continuation
                .as_ref()
                .map(|c| lower_workflow_body(c, provenance, ctx))
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
                .map(|c| lower_workflow_body(c, provenance, ctx))
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
                .map(|arm| lower_receive_arm(arm, provenance, ctx))
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
                .map(|c| lower_workflow_body(c, provenance, ctx))
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
                .map(|e| lower_workflow_body(e, provenance, ctx))
                .transpose()?
                .unwrap_or(CoreWorkflow::Done);
            let then_wf = lower_workflow_body(then_branch, provenance, ctx)?;

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
            body: Box::new(lower_workflow_body(body, provenance, ctx)?),
        }),

        SurfaceWorkflow::With {
            capability, body, ..
        } => Ok(CoreWorkflow::With {
            capability: Capability {
                name: capability.to_string(),
                effect: Effect::Epistemic,
                constraints: vec![],
            },
            workflow: Box::new(lower_workflow_body(body, provenance, ctx)?),
        }),

        SurfaceWorkflow::Maybe {
            primary, fallback, ..
        } => Ok(CoreWorkflow::Maybe {
            primary: Box::new(lower_workflow_body(primary, provenance, ctx)?),
            fallback: Box::new(lower_workflow_body(fallback, provenance, ctx)?),
        }),

        SurfaceWorkflow::Must { body, .. } => Ok(CoreWorkflow::Must {
            workflow: Box::new(lower_workflow_body(body, provenance, ctx)?),
        }),

        SurfaceWorkflow::Seq { first, second, .. } => {
            if matches!(second.as_ref(), SurfaceWorkflow::Done { .. })
                && is_self_terminating_surface_workflow(first)
            {
                return lower_workflow_body(first, provenance, ctx);
            }

            Ok(CoreWorkflow::Seq {
                first: Box::new(lower_workflow_body(first, provenance, ctx)?),
                second: Box::new(lower_workflow_body(second, provenance, ctx)?),
            })
        }

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
            let continuation = Box::new(lower_yield_arms(resume_var, arms, provenance, ctx)?);

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
        Type::Fn(_params, _ret) => TypeExpr::Constructor {
            name: "Fn".to_string(),
            args: vec![],
        },
        Type::Associated { base, name } => TypeExpr::Constructor {
            name: name.to_string(),
            args: vec![lower_type_to_type_expr(base)],
        },
    }
}

/// Convert a surface Type to a core AST TypeExpr.
pub fn lower_surface_type(ty: &Type) -> ash_core::ast::TypeExpr {
    use ash_core::ast::TypeExpr;
    match ty {
        Type::Name(name) => TypeExpr::Named(name.to_string()),
        Type::List(inner) => TypeExpr::Constructor {
            name: "List".to_string(),
            args: vec![lower_surface_type(inner)],
        },
        Type::Record(fields) => TypeExpr::Record(
            fields
                .iter()
                .map(|(n, t)| (n.to_string(), lower_surface_type(t)))
                .collect(),
        ),
        Type::Capability(name) => TypeExpr::Constructor {
            name: "Capability".to_string(),
            args: vec![TypeExpr::Named(name.to_string())],
        },
        Type::Constructor { name, args } => TypeExpr::Constructor {
            name: name.to_string(),
            args: args.iter().map(lower_surface_type).collect(),
        },
        Type::Associated { base, name } => TypeExpr::Associated {
            base: Box::new(lower_surface_type(base)),
            name: name.to_string(),
        },
        Type::Fn(params, ret) => {
            let mut args: Vec<_> = params.iter().map(lower_surface_type).collect();
            args.push(lower_surface_type(ret));
            TypeExpr::Constructor {
                name: "Fn".to_string(),
                args,
            }
        }
    }
}

/// Lower a surface interface definition to core AST.
pub fn lower_interface_def(
    iface: &crate::surface::InterfaceDef,
) -> Result<ash_core::ast::InterfaceDef, LoweringError> {
    use ash_core::ast::{AssociatedType, InterfaceDef, InterfaceMethodSig, Visibility};
    Ok(InterfaceDef {
        name: iface.name.to_string(),
        type_params: iface.type_params.iter().map(|n| n.to_string()).collect(),
        associated_types: iface
            .associated_types
            .iter()
            .map(|d| AssociatedType {
                name: d.name.to_string(),
            })
            .collect(),
        methods: iface
            .methods
            .iter()
            .map(|m| InterfaceMethodSig {
                name: m.name.to_string(),
                params: m.params.iter().map(lower_surface_type).collect(),
                return_type: lower_surface_type(&m.return_type),
            })
            .collect(),
        visibility: match iface.visibility {
            crate::surface::Visibility::Public => Visibility::Public,
            crate::surface::Visibility::Crate => Visibility::Crate,
            _ => Visibility::Private,
        },
    })
}

/// Lower a surface impl definition to core AST.
pub fn lower_impl_def(
    impl_def: &crate::surface::ImplDef,
) -> Result<ash_core::ast::ImplDef, LoweringError> {
    use ash_core::ast::{AssociatedTypeBinding, ImplDef, Visibility, WhereBound};
    Ok(ImplDef {
        visibility: match impl_def.visibility {
            crate::surface::Visibility::Public => Visibility::Public,
            crate::surface::Visibility::Crate => Visibility::Crate,
            _ => Visibility::Private,
        },
        interface: impl_def.interface.to_string(),
        type_params: impl_def.type_params.iter().map(|n| n.to_string()).collect(),
        type_args: impl_def.type_args.iter().map(lower_surface_type).collect(),
        where_bounds: impl_def
            .where_bounds
            .iter()
            .map(|b| WhereBound {
                param: b.param.to_string(),
                bound: b.bound.to_string(),
            })
            .collect(),
        associated_type_bindings: impl_def
            .associated_type_bindings
            .iter()
            .map(|b| AssociatedTypeBinding {
                name: b.name.to_string(),
                ty: lower_surface_type(&b.ty),
            })
            .collect(),
        methods: impl_def
            .methods
            .iter()
            .map(|m| {
                Ok(ash_core::ast::ImplMethodDef {
                    name: m.name.to_string(),
                    params: m.params.iter().map(|p| p.to_string()).collect(),
                    body: lower_expr(&m.body)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
    })
}

/// Lower yield arms into a continuation workflow.
///
/// The resume_var is bound to the response value, and then the arms
/// are processed as pattern matches.
fn lower_yield_arms(
    resume_var: &str,
    arms: &[YieldArm],
    provenance: &Provenance,
    ctx: &LoweringContext,
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
            expr: CoreExpr::Variable {
                name: resume_var.to_string(),
                span: ash_core::Span::default(),
            },
            continuation: Box::new(lower_workflow_body(&arm.body, provenance, ctx)?),
        })
    } else {
        // Multiple arms: create a cascade of If expressions
        // For now, use the first arm's pattern as the main match
        // and subsequent arms as fallbacks
        let first_arm = &arms[0];
        let _rest_continuation = if arms.len() > 1 {
            lower_yield_arms(resume_var, &arms[1..], provenance, ctx)?
        } else {
            CoreWorkflow::Done
        };

        Ok(CoreWorkflow::Let {
            pattern: lower_pattern(&first_arm.pattern)?,
            expr: CoreExpr::Variable {
                name: resume_var.to_string(),
                span: ash_core::Span::default(),
            },
            continuation: Box::new(lower_workflow_body(&first_arm.body, provenance, ctx)?),
        })
    }
}

/// Built-in function names that the interpreter handles via string dispatch in eval_function_call.
/// Calls to these names emit `Expr::Call`; all other calls emit `Expr::FnApply`.
pub const BUILTIN_FUNCTIONS: &[&str] = &[
    "len",
    "append",
    "concat",
    "head",
    "tail",
    "filter",
    "map",
    "starts_with",
    "ends_with",
    "keys",
    "values",
    "is_int",
    "is_string",
    "is_bool",
    "is_list",
    "is_record",
    "is_null",
    "record",
];

/// Lower a surface expression to core IR.
pub fn lower_expr(expr: &Expr) -> Result<CoreExpr, LoweringError> {
    match expr {
        Expr::Literal(lit) => Ok(CoreExpr::Literal(lower_literal(lit)?)),

        Expr::Variable { name, .. } => Ok(CoreExpr::Variable {
            name: name.to_string(),
            span: ash_core::Span::default(),
        }),

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
            op: lower_binary_op(*op)?,
            left: Box::new(lower_expr(left)?),
            right: Box::new(lower_expr(right)?),
        }),

        Expr::Call {
            func, module, args, ..
        } => {
            let lowered_args = args.iter().map(lower_expr).collect::<Result<Vec<_>, _>>()?;

            if module.is_none() && !BUILTIN_FUNCTIONS.contains(&func.as_ref()) {
                // User-defined function call: emit FnApply
                Ok(CoreExpr::FnApply {
                    func: Box::new(CoreExpr::Variable {
                        name: func.to_string(),
                        span: ash_core::Span::default(),
                    }),
                    args: lowered_args,
                })
            } else {
                // Built-in or module-qualified call: keep existing Call behaviour
                Ok(CoreExpr::Call {
                    func: func.to_string(),
                    module: module.as_ref().map(|m| m.to_string()),
                    arguments: lowered_args,
                })
            }
        }

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

        Expr::Constructor {
            name,
            fields,
            payload,
            ..
        } => {
            let lowered_fields = match payload {
                crate::surface::ConstructorPayload::Unit => fields
                    .iter()
                    .map(|(n, e)| Ok((n.to_string(), lower_expr(e)?)))
                    .collect::<Result<Vec<_>, _>>()?,
                crate::surface::ConstructorPayload::Record(record_fields) => record_fields
                    .iter()
                    .map(|(n, e)| Ok((n.to_string(), lower_expr(e)?)))
                    .collect::<Result<Vec<_>, _>>()?,
                crate::surface::ConstructorPayload::Tuple(items) => items
                    .iter()
                    .enumerate()
                    .map(|(index, expr)| Ok((tuple_field_name(index), lower_expr(expr)?)))
                    .collect::<Result<Vec<_>, _>>()?,
            };

            Ok(CoreExpr::Constructor {
                name: name.to_string(),
                fields: lowered_fields,
            })
        }

        Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => Ok(CoreExpr::Match {
            scrutinee: Box::new(lower_expr(condition)?),
            arms: vec![
                CoreMatchArm {
                    pattern: CorePattern::Literal(ash_core::Value::Bool(true)),
                    body: lower_expr(then_branch)?,
                },
                CoreMatchArm {
                    pattern: CorePattern::Literal(ash_core::Value::Bool(false)),
                    body: match else_branch {
                        Some(else_expr) => lower_expr(else_expr)?,
                        None => CoreExpr::Literal(ash_core::Value::Null),
                    },
                },
            ],
        }),
        Expr::Panic { .. } => Err(LoweringError::ExprNotLowerable { kind: "panic" }),
        Expr::Block { .. } => Err(LoweringError::ExprNotLowerable { kind: "block" }),

        Expr::FnDef {
            params,
            return_type,
            body,
            ..
        } => lower_fn_def(params, return_type, body),

        Expr::FnApply { func, args, .. } => {
            let lowered_args = args.iter().map(lower_expr).collect::<Result<Vec<_>, _>>()?;
            Ok(CoreExpr::FnApply {
                func: Box::new(lower_expr(func)?),
                args: lowered_args,
            })
        }
    }
}

/// Lower a surface expression that appears at module scope (top-level).
///
/// This is the same as [`lower_expr`] except that `Expr::FnDef` is rejected
/// because anonymous fn expressions are not valid at module scope;
/// users should write `pub fn` instead.
///
/// Note: this only rejects FnDef at the *top level* of the module expression.
/// FnDef nested inside blocks, let-bindings, or function arguments are valid
/// and are handled by `lower_expr` (which does not impose this restriction).
///
/// TODO: Currently only called from tests.  The engine still uses `lower_expr`
/// in module-scope contexts.  Wire this into the engine's module-lowering path
/// to activate the guard in production.
pub fn lower_module_expr(expr: &Expr) -> Result<CoreExpr, LoweringError> {
    if matches!(expr, Expr::FnDef { .. }) {
        return Err(LoweringError::FnDefNotAllowedAtModuleScope);
    }
    lower_expr(expr)
}

/// Lower a surface FnDef expression to core FnDef.
///
/// Surface FnDef is introduced by the parser (TASK-556). This placeholder handles
/// the core `Expr::FnDef` variant added in TASK-551 so that when the parser starts
/// producing FnDef nodes the lowering path is already wired up.
///
/// # Parameters
/// - `params`: Parameter list as `(name, optional_type_annotation)` pairs.
/// - `return_type`: Optional return type annotation string.
/// - `body`: The function body expression.
pub(crate) fn lower_fn_def(
    params: &[(Box<str>, Option<Box<str>>)],
    return_type: &Option<Box<str>>,
    body: &Expr,
) -> Result<CoreExpr, LoweringError> {
    Ok(CoreExpr::FnDef {
        params: params
            .iter()
            .map(|(n, t)| (n.to_string(), t.as_deref().map(str::to_string)))
            .collect(),
        return_type: return_type.as_deref().map(str::to_string),
        body: Box::new(lower_expr(body)?),
    })
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
    ctx: &LoweringContext,
) -> Result<CoreReceiveArm, LoweringError> {
    Ok(CoreReceiveArm {
        pattern: lower_receive_pattern(&arm.pattern)?,
        guard: arm.guard.as_ref().map(lower_expr).transpose()?,
        body: lower_workflow_body(&arm.body, provenance, ctx)?,
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
        CheckTarget::Policy(_) => Err(LoweringError::InvalidTarget(
            "policy instances are not valid canonical check targets".to_string(),
        )),
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
fn lower_binary_op(op: BinaryOp) -> Result<ash_core::BinaryOp, LoweringError> {
    match op {
        BinaryOp::Add => Ok(ash_core::BinaryOp::Add),
        BinaryOp::Sub => Ok(ash_core::BinaryOp::Sub),
        BinaryOp::Mul => Ok(ash_core::BinaryOp::Mul),
        BinaryOp::Div => Ok(ash_core::BinaryOp::Div),
        BinaryOp::Mod => Ok(ash_core::BinaryOp::Mod),
        BinaryOp::And => Ok(ash_core::BinaryOp::And),
        BinaryOp::Or => Ok(ash_core::BinaryOp::Or),
        BinaryOp::Eq => Ok(ash_core::BinaryOp::Eq),
        BinaryOp::Neq => Ok(ash_core::BinaryOp::Ne),
        BinaryOp::Lt => Ok(ash_core::BinaryOp::Lt),
        BinaryOp::Gt => Ok(ash_core::BinaryOp::Gt),
        BinaryOp::Leq => Ok(ash_core::BinaryOp::Le),
        BinaryOp::Geq => Ok(ash_core::BinaryOp::Ge),
        BinaryOp::In => Ok(ash_core::BinaryOp::In),
        BinaryOp::Pipe => Err(LoweringError::UnsupportedFeature(
            "Pipe operator should be desugared during parsing".to_string(),
        )),
    }
}

/// Lower a pattern to core IR.
pub fn lower_pattern(pattern: &Pattern) -> Result<CorePattern, LoweringError> {
    match pattern {
        Pattern::Variable { name, .. } => Ok(CorePattern::Variable {
            name: name.to_string(),
            span: ash_core::Span::default(),
        }),

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

        Pattern::Variant {
            name,
            fields,
            payload,
            ..
        } => {
            let lowered_fields = match payload {
                crate::surface::VariantPatternPayload::Unit => fields
                    .as_ref()
                    .map(|fs| {
                        fs.iter()
                            .map(|(n, p)| Ok((n.to_string(), lower_pattern(p)?)))
                            .collect::<Result<Vec<_>, _>>()
                    })
                    .transpose()?,
                crate::surface::VariantPatternPayload::Record(record_fields) => Some(
                    record_fields
                        .iter()
                        .map(|(n, p)| Ok((n.to_string(), lower_pattern(p)?)))
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                crate::surface::VariantPatternPayload::Tuple(items) => Some(
                    items
                        .iter()
                        .enumerate()
                        .map(|(index, pattern)| {
                            Ok((tuple_field_name(index), lower_pattern(pattern)?))
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                ),
            };

            Ok(CorePattern::Variant {
                name: name.to_string(),
                fields: lowered_fields,
            })
        }

        Pattern::Literal(lit) => Ok(CorePattern::Literal(lower_literal(lit)?)),
    }
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
        BinaryOp, Contract as SurfaceContract, EffectType, EnsuresClause, Expr as SurfaceExpr,
        Literal as SurfaceLiteral, Pattern, Requirement as SurfaceRequirement, RoleDef,
        Workflow as SurfaceWorkflow,
    };
    use crate::token::Span;

    fn dummy_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    fn int_expr(value: i64) -> SurfaceExpr {
        SurfaceExpr::Literal(SurfaceLiteral::Int(value))
    }

    fn var_expr(name: &str) -> SurfaceExpr {
        SurfaceExpr::Variable {
            name: name.into(),
            span: crate::token::Span::default(),
        }
    }

    #[test]
    fn test_lower_done() {
        let surface = SurfaceWorkflow::Done { span: dummy_span() };
        let core =
            lower_workflow_body(&surface, &Provenance::new(), &LoweringContext::new()).unwrap();
        assert!(matches!(core, CoreWorkflow::Done));
    }

    #[test]
    fn test_lower_let() {
        let surface = SurfaceWorkflow::Let {
            pattern: Pattern::Variable {
                name: "x".into(),
                span: crate::token::Span::default(),
            },
            expr: SurfaceExpr::Literal(SurfaceLiteral::Int(42)),
            continuation: Some(Box::new(SurfaceWorkflow::Done { span: dummy_span() })),
            span: dummy_span(),
        };
        let core =
            lower_workflow_body(&surface, &Provenance::new(), &LoweringContext::new()).unwrap();
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
        let surface = SurfaceExpr::Variable {
            name: "my_var".into(),
            span: crate::token::Span::default(),
        };
        let core = lower_expr(&surface).unwrap();
        assert!(matches!(core, CoreExpr::Variable { name, .. } if name == "my_var"));
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
    fn test_interface_method_call_lowers_as_call() {
        // After TASK-561, interface method calls use Expr::Call with module qualifier
        let surface = SurfaceExpr::Call {
            func: "explain".into(),
            module: Some("Explain".into()),
            args: vec![SurfaceExpr::Variable {
                name: "value".into(),
                span: crate::token::Span::default(),
            }],
            span: crate::token::Span::new(0, 22, 1, 1),
        };

        let result = lower_expr(&surface);
        assert!(result.is_ok());
        let core = result.unwrap();
        match &core {
            CoreExpr::Call {
                func,
                module,
                arguments,
            } => {
                assert_eq!(func, "explain");
                assert_eq!(module.as_deref(), Some("Explain"));
                assert_eq!(arguments.len(), 1);
            }
            other => panic!("expected CoreExpr::Call, got {other:?}"),
        }
    }

    #[test]
    fn test_lower_pattern_variable() {
        let surface = Pattern::Variable {
            name: "x".into(),
            span: crate::token::Span::default(),
        };
        let core = lower_pattern(&surface).unwrap();
        assert!(matches!(core, CorePattern::Variable { name, .. } if name == "x"));
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
            Pattern::Variable {
                name: "a".into(),
                span: crate::token::Span::default(),
            },
            Pattern::Variable {
                name: "b".into(),
                span: crate::token::Span::default(),
            },
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
            condition: SurfaceExpr::Variable {
                name: "approved".into(),
                span: crate::token::Span::default(),
            },
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
                condition: CoreExpr::Variable { name: condition, .. },
            } if name == "manager"
                && authority.is_empty()
                && obligations.is_empty()
                && condition == "approved"
        ));
    }

    #[test]
    fn test_lower_role_def_preserves_named_capability_refs_and_obligation_refs() {
        let surface = RoleDef {
            name: "reviewer".into(),
            capabilities: vec![
                crate::surface::CapabilityDecl {
                    capability: "approve".into(),
                    constraints: None,
                    span: dummy_span(),
                },
                crate::surface::CapabilityDecl {
                    capability: "review".into(),
                    constraints: None,
                    span: dummy_span(),
                },
            ],
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
                target_provider: None,
                target_action: None,
                span: dummy_span(),
            }),
            crate::surface::Definition::Capability(crate::surface::CapabilityDef {
                visibility: crate::surface::Visibility::Inherited,
                name: "review".into(),
                effect: crate::surface::EffectType::Analyze,
                params: vec![],
                return_type: None,
                constraints: vec![],
                target_provider: None,
                target_action: None,
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
                    target_provider: None,
                    target_action: None,
                    span: dummy_span(),
                }),
                crate::surface::Definition::Role(RoleDef {
                    name: "reviewer".into(),
                    capabilities: vec![crate::surface::CapabilityDecl {
                        capability: "approve".into(),
                        constraints: None,
                        span: dummy_span(),
                    }],
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
                    target_provider: None,
                    target_action: None,
                    span: dummy_span(),
                }),
                crate::surface::Definition::Role(RoleDef {
                    name: "reviewer".into(),
                    capabilities: vec![crate::surface::CapabilityDecl {
                        capability: "approve".into(),
                        constraints: None,
                        span: dummy_span(),
                    }],
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
            lower_binary_op(BinaryOp::Add).unwrap(),
            ash_core::BinaryOp::Add
        ));
        assert!(matches!(
            lower_binary_op(BinaryOp::Sub).unwrap(),
            ash_core::BinaryOp::Sub
        ));
        assert!(matches!(
            lower_binary_op(BinaryOp::Mul).unwrap(),
            ash_core::BinaryOp::Mul
        ));
        assert!(matches!(
            lower_binary_op(BinaryOp::Div).unwrap(),
            ash_core::BinaryOp::Div
        ));
        assert!(matches!(
            lower_binary_op(BinaryOp::Mod).unwrap(),
            ash_core::BinaryOp::Mod
        ));
        assert!(matches!(
            lower_binary_op(BinaryOp::Eq).unwrap(),
            ash_core::BinaryOp::Eq
        ));
        assert!(matches!(
            lower_binary_op(BinaryOp::And).unwrap(),
            ash_core::BinaryOp::And
        ));
        assert!(matches!(
            lower_binary_op(BinaryOp::Or).unwrap(),
            ash_core::BinaryOp::Or
        ));
    }

    #[test]
    fn test_lower_fn_contract_stage1_predicates() {
        let contract = SurfaceContract {
            requires: vec![
                SurfaceRequirement::Arithmetic {
                    expr: SurfaceExpr::Binary {
                        op: BinaryOp::Geq,
                        left: Box::new(var_expr("n")),
                        right: Box::new(int_expr(0)),
                        span: dummy_span(),
                    },
                },
                SurfaceRequirement::Arithmetic {
                    expr: SurfaceExpr::Binary {
                        op: BinaryOp::Neq,
                        left: Box::new(var_expr("d")),
                        right: Box::new(int_expr(0)),
                        span: dummy_span(),
                    },
                },
                SurfaceRequirement::Arithmetic {
                    expr: SurfaceExpr::Binary {
                        op: BinaryOp::Eq,
                        left: Box::new(SurfaceExpr::Binary {
                            op: BinaryOp::Mod,
                            left: Box::new(var_expr("n")),
                            right: Box::new(int_expr(2)),
                            span: dummy_span(),
                        }),
                        right: Box::new(int_expr(1)),
                        span: dummy_span(),
                    },
                },
            ],
            ensures: vec![EnsuresClause {
                expr: SurfaceExpr::Binary {
                    op: BinaryOp::Geq,
                    left: Box::new(var_expr("result")),
                    right: Box::new(int_expr(0)),
                    span: dummy_span(),
                },
                span: dummy_span(),
            }],
        };

        let lowered = lower_fn_contract(Some(&contract)).expect("fn contract should lower");
        assert_eq!(lowered.contract.requires.len(), 3);
        assert_eq!(lowered.runtime_postconditions.predicates.len(), 1);
        assert!(matches!(
            &lowered.contract.requires[0],
            ash_core::workflow_contract::Requirement::Arithmetic { var, constraint }
                if var == "n"
                    && matches!(constraint, ash_core::workflow_contract::ArithConstraint::Gte(0))
        ));
        assert!(matches!(
            &lowered.contract.requires[1],
            ash_core::workflow_contract::Requirement::Arithmetic { var, constraint }
                if var == "d"
                    && matches!(constraint, ash_core::workflow_contract::ArithConstraint::NotEq(0))
        ));
        assert!(matches!(
            &lowered.contract.requires[2],
            ash_core::workflow_contract::Requirement::Arithmetic { var, constraint }
                if var == "n"
                    && matches!(
                        constraint,
                        ash_core::workflow_contract::ArithConstraint::Modulo { div: 2, rem: 1 }
                    )
        ));
        assert!(matches!(
            &lowered.runtime_postconditions.predicates[0],
            ash_core::workflow_contract::PostPredicate::ResultSatisfies(
                ash_core::workflow_contract::ArithConstraint::Gte(0)
            )
        ));
    }

    #[test]
    fn test_lower_fn_contract_rejects_non_value_ensures() {
        let contract = SurfaceContract {
            requires: vec![],
            ensures: vec![EnsuresClause {
                expr: SurfaceExpr::Binary {
                    op: BinaryOp::Geq,
                    left: Box::new(var_expr("state")),
                    right: Box::new(int_expr(0)),
                    span: dummy_span(),
                },
                span: dummy_span(),
            }],
        };

        let error = lower_fn_contract(Some(&contract)).expect_err("invalid ensures should fail");
        assert!(matches!(
            error,
            FnContractLoweringError::InvalidEnsures { .. }
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
        let core =
            lower_workflow_body(&surface, &Provenance::new(), &LoweringContext::new()).unwrap();
        assert!(matches!(core, CoreWorkflow::If { .. }));
    }

    #[test]
    fn test_lower_seq() {
        let surface = SurfaceWorkflow::Seq {
            first: Box::new(SurfaceWorkflow::Observe {
                capability: "read".into(),
                binding: None,
                continuation: None,
                span: dummy_span(),
            }),
            second: Box::new(SurfaceWorkflow::Done { span: dummy_span() }),
            span: dummy_span(),
        };
        let core =
            lower_workflow_body(&surface, &Provenance::new(), &LoweringContext::new()).unwrap();
        assert!(matches!(core, CoreWorkflow::Seq { .. }));
    }

    #[test]
    fn test_lower_observe() {
        let surface = SurfaceWorkflow::Observe {
            capability: "read".into(),
            binding: Some(Pattern::Variable {
                name: "x".into(),
                span: crate::token::Span::default(),
            }),
            continuation: None,
            span: dummy_span(),
        };
        let core =
            lower_workflow_body(&surface, &Provenance::new(), &LoweringContext::new()).unwrap();
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
        let core =
            lower_workflow_body(&surface, &Provenance::new(), &LoweringContext::new()).unwrap();
        assert!(matches!(core, CoreWorkflow::Orient { .. }));
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

    // =========================================================================
    // Module-Owned Capability Resolution Tests (TASK-475)
    // =========================================================================

    #[test]
    fn test_lower_act_with_explicit_target_bypasses_resolution() {
        // Explicit provider:action calls should bypass capability resolution
        let surface = SurfaceWorkflow::Act {
            action: crate::surface::ActionRef {
                target: crate::surface::OperationalTarget::Explicit {
                    provider: "io".into(),
                    action: "fs_read".into(),
                },
                args: vec![],
            },
            guard: None,
            result_name: None,
            continuation: None,
            span: dummy_span(),
        };

        // Should work even without capability context
        let ctx = LoweringContext::new();
        let core = lower_workflow_body(&surface, &Provenance::new(), &ctx).unwrap();

        match core {
            CoreWorkflow::Act {
                provider_name,
                action_name,
                ..
            } => {
                assert_eq!(provider_name, "io");
                assert_eq!(action_name, "fs_read");
            }
            _ => panic!("expected Act workflow, got {:?}", core),
        }
    }

    #[test]
    fn test_lower_act_with_symbolic_target_requires_context() {
        // Symbolic capability calls require resolution context
        let surface = SurfaceWorkflow::Act {
            action: crate::surface::ActionRef {
                target: crate::surface::OperationalTarget::Symbolic {
                    capability_name: "fs_read".into(),
                },
                args: vec![],
            },
            guard: None,
            result_name: None,
            continuation: None,
            span: dummy_span(),
        };

        // Without capability context, should fail to resolve
        let ctx = LoweringContext::new();
        let result = lower_workflow_body(&surface, &Provenance::new(), &ctx);
        assert!(
            matches!(result, Err(LoweringError::UnresolvedCapability { name }) if name == "fs_read")
        );
    }

    #[test]
    fn test_lower_act_with_capability_context_resolves_symbolic() {
        // Symbolic capability calls resolve when context has the mapping
        use crate::capability_export::{
            CapabilityEffect, CapabilityExport, CapabilityResolutionContext,
        };
        use ash_core::module_graph::ModuleId;

        let surface = SurfaceWorkflow::Act {
            action: crate::surface::ActionRef {
                target: crate::surface::OperationalTarget::Symbolic {
                    capability_name: "fs_read".into(),
                },
                args: vec![],
            },
            guard: None,
            result_name: None,
            continuation: None,
            span: dummy_span(),
        };

        // Build a capability resolution context with the mapping
        let mut cap_context = CapabilityResolutionContext::new();
        let export = CapabilityExport {
            visible_name: "fs_read".into(),
            declaring_module: ModuleId(0),
            target_provider: "io".into(),
            target_action: "fs_read".into(),
            visibility: crate::surface::Visibility::Public,
            effect: CapabilityEffect::Act,
        };
        cap_context.register(&export);

        let ctx = LoweringContext::with_capability_context_for_module(cap_context, ModuleId(0));
        let core = lower_workflow_body(&surface, &Provenance::new(), &ctx).unwrap();

        match core {
            CoreWorkflow::Act {
                provider_name,
                action_name,
                ..
            } => {
                assert_eq!(provider_name, "io");
                assert_eq!(action_name, "fs_read");
            }
            _ => panic!("expected Act workflow, got {:?}", core),
        }
    }
}
