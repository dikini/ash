//! Surface AST to Core IR lowering.
//!
//! This module converts the surface syntax AST into the core IR representation
//! used by the ash-core crate.

use std::{cell::RefCell, fmt};

use ash_core::adt::tuple_field_name;
use ash_core::{
    Capability, Effect, Expr as CoreExpr, Guard as CoreGuard, Kind, MatchArm as CoreMatchArm,
    Obligation as CoreObligation, Pattern as CorePattern, Predicate as CorePredicate, Provenance,
    ReceiveArm as CoreReceiveArm, ReceivePattern as CoreReceivePattern, Role as CoreRole,
    Workflow as CoreWorkflow,
};

#[cfg(test)]
use ash_core::RoleObligationRef as CoreRoleObligationRef;

use crate::capability_export::{CapabilityResolutionContext, ModuleId};
use crate::surface::{
    BinaryOp, BlockStmt, CapabilityDef, CheckTarget, DoStmt, EffectType, ExpandedSurfaceModule,
    Expr, Guard, Literal, ModuleFile, ObligationRef, Pattern, PolicyExpr, Predicate, StreamPattern,
    Type, UnaryOp, Workflow as SurfaceWorkflow, WorkflowDef, YieldArm, expand_surface_module,
    visit_exprs_in_module,
};

thread_local! {
    static ACTIVE_EFFECTFUL_NAMES: RefCell<Option<std::collections::HashSet<String>>> =
        const { RefCell::new(None) };
}

fn with_active_effectful_names<T>(
    effectful_names: &std::collections::HashSet<String>,
    f: impl FnOnce() -> T,
) -> T {
    let previous = ACTIVE_EFFECTFUL_NAMES.with(|cell| cell.replace(Some(effectful_names.clone())));
    let result = f();
    ACTIVE_EFFECTFUL_NAMES.with(|cell| {
        let _ = cell.replace(previous);
    });
    result
}

fn active_effectful_names_contains(name: &str) -> bool {
    ACTIVE_EFFECTFUL_NAMES.with(|cell| {
        cell.borrow()
            .as_ref()
            .is_some_and(|names| names.contains(name))
    })
}

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
    reject_kinded_type_params(
        &def.type_params,
        "kinded workflow type parameters are parsed by TASK-906 but lowered by TASK-907",
    )?;

    // Create a provenance for the workflow
    let provenance = Provenance::new();

    let core = with_active_effectful_names(&ctx.effectful_names, || {
        lower_workflow_body(&def.body, &provenance, ctx)
    })?;
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
    reject_kinded_type_params(
        &def.type_params,
        "kinded workflow type parameters are parsed by TASK-906 but lowered by TASK-907",
    )?;

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

fn reject_kinded_type_params(
    params: &[crate::surface::TypeParam],
    message: &'static str,
) -> Result<(), LoweringError> {
    for param in params {
        if param
            .kind
            .as_ref()
            .is_some_and(|annotation| annotation.kind != Kind::Type)
        {
            return Err(LoweringError::UnsupportedFeature(message.to_string()));
        }
    }

    Ok(())
}

fn reject_kinded_interface_type_params(
    params: &[crate::surface::InterfaceTypeParam],
    message: &'static str,
) -> Result<(), LoweringError> {
    for param in params {
        if param
            .kind
            .as_ref()
            .is_some_and(|annotation| annotation.kind != Kind::Type)
        {
            return Err(LoweringError::UnsupportedFeature(message.to_string()));
        }
    }

    Ok(())
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
            Ok(CoreExpr::Literal(ash_core::Value::list_from_vec(
                elements
                    .into_iter()
                    .map(|e| match e {
                        CoreExpr::Literal(v) => v,
                        _ => ash_core::Value::Null,
                    })
                    .collect(),
            )))
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
                    // Phase 158 fix: First check if this is a function call (not a capability)
                    // If the name is not in BUILTIN_FUNCTIONS and not an effectful name,
                    // it's likely a user-defined function call, not a capability.
                    if !BUILTIN_FUNCTIONS.contains(&capability_name.as_ref())
                        && !active_effectful_names_contains(capability_name.as_ref())
                    {
                        // This is a function call, not a capability call
                        // Lower it as an Orient wrapping a FnApply expression
                        let cont = continuation
                            .as_ref()
                            .map(|c| lower_workflow_body(c, provenance, ctx))
                            .transpose()?
                            .unwrap_or(CoreWorkflow::Done);
                        return Ok(CoreWorkflow::Orient {
                            expr: CoreExpr::FnApply {
                                func: Box::new(CoreExpr::Variable {
                                    name: capability_name.to_string(),
                                    span: ash_core::Span::default(),
                                }),
                                args: action
                                    .args
                                    .iter()
                                    .map(lower_expr)
                                    .collect::<Result<Vec<_>, _>>()?,
                            },
                            continuation: Box::new(cont),
                        });
                    }
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
        Type::Hole { .. } => {
            panic!(
                "type holes require SPEC-066 semantic lowering before workflow-contract lowering"
            )
        }
        Type::List(inner) => TypeExpr::Constructor {
            name: "List".to_string(),
            args: vec![lower_type_to_type_expr(inner)],
        },
        Type::Tuple(items) => TypeExpr::Tuple(items.iter().map(lower_type_to_type_expr).collect()),
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
        Type::Fn(_, _, _) => TypeExpr::Constructor {
            name: "Fn".to_string(),
            args: vec![],
        },
        Type::Associated { base, name } => TypeExpr::Constructor {
            name: name.to_string(),
            args: vec![lower_type_to_type_expr(base)],
        },
        Type::AssociatedFamilyProjection { .. } => panic!(
            "associated family projections require Phase 115 semantic lowering before workflow-contract lowering"
        ),
    }
}

/// Convert a surface Type to a core AST TypeExpr.
pub fn lower_surface_type(ty: &Type) -> ash_core::ast::TypeExpr {
    use ash_core::ast::TypeExpr;
    match ty {
        Type::Name(name) => TypeExpr::Named(name.to_string()),
        Type::Hole { .. } => {
            panic!("type holes require SPEC-066 semantic lowering before core AST lowering")
        }
        Type::List(inner) => TypeExpr::Constructor {
            name: "List".to_string(),
            args: vec![lower_surface_type(inner)],
        },
        Type::Tuple(items) => TypeExpr::Tuple(items.iter().map(lower_surface_type).collect()),
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
        Type::AssociatedFamilyProjection {
            interface,
            args,
            member,
            ..
        } => TypeExpr::Associated {
            base: Box::new(TypeExpr::Constructor {
                name: interface.to_string(),
                args: args.iter().map(lower_surface_type).collect(),
            }),
            name: member.to_string(),
        },
        Type::Fn(params, _row, ret) => {
            let mut args: Vec<_> = params.iter().map(lower_surface_type).collect();
            args.push(lower_surface_type(ret));
            TypeExpr::Constructor {
                name: "Fn".to_string(),
                args,
            }
        }
    }
}

/// Core ordinary type declarations plus their core-owned semantic summary.
#[derive(Debug, Clone, PartialEq)]
pub struct LoweredTypeMetadata {
    pub type_defs: Vec<ash_core::ast::TypeDef>,
    pub type_function_defs: Vec<crate::surface::TypeFnDef>,
    pub summary: ash_core::semantic_summary::ModuleSemanticSummary,
}

/// Lower ordinary surface type declarations in a parsed module to core carriers.
///
/// The caller supplies the resolved module identity so canonical type identities
/// are module-anchored instead of string-only.
#[must_use]
pub fn lower_module_type_metadata(
    module: &crate::surface::ModuleFile,
    module_identity: ash_core::semantic_summary::ModuleIdentity,
) -> LoweredTypeMetadata {
    let module_anchor = source_anchor_for_module(module, &module_identity);
    let mut summary =
        ash_core::semantic_summary::ModuleSemanticSummary::new(module_identity.clone())
            .with_diagnostic_anchor(module_anchor);
    let mut type_defs = Vec::new();
    let mut type_function_defs = Vec::new();

    let mut has_sealed_domains = false;

    for definition in &module.definitions {
        match definition {
            crate::surface::Definition::Type(surface_type) => {
                let core_type = lower_surface_type_def(surface_type);
                let type_id = ash_core::semantic_summary::TypeDeclId::ordinary(
                    module_identity.clone(),
                    core_type.name.clone(),
                );
                let anchor = source_anchor_for_type(surface_type, module, &module_identity);
                let representation_exposure = if core_type.builtin {
                    ash_core::semantic_summary::RepresentationExposure::Opaque
                } else {
                    ash_core::semantic_summary::RepresentationExposure::Exposed
                };
                let representation = if core_type.builtin {
                    ash_core::semantic_summary::TypeRepresentationSummary::opaque(true)
                } else {
                    ash_core::semantic_summary::TypeRepresentationSummary::exposed(
                        core_type.body.clone(),
                    )
                };

                let type_summary = ash_core::semantic_summary::TypeDeclSummary::new(
                    type_id.clone(),
                    core_type.name.clone(),
                    core_type.visibility,
                    representation_exposure,
                    representation,
                    anchor.clone(),
                )
                .with_params(core_type.params.clone());
                summary = summary.with_exported_type(type_summary);

                if !core_type.builtin
                    && let ash_core::ast::TypeBody::Enum(variants) = &core_type.body
                {
                    for variant in variants {
                        let payload_kind = constructor_payload_kind(&variant.payload);
                        let constructor_id = ash_core::semantic_summary::ConstructorId::variant(
                            type_id.clone(),
                            variant.name.clone(),
                            payload_kind,
                        );
                        let constructor = ash_core::semantic_summary::ConstructorSummary::new(
                            constructor_id,
                            type_id.clone(),
                            variant.name.clone(),
                            payload_kind,
                            core_type.visibility,
                            anchor.clone(),
                        );
                        summary = summary.with_exported_constructor(constructor);
                    }
                }

                summary = summary.with_diagnostic_anchor(anchor);
                type_defs.push(core_type);
            }
            crate::surface::Definition::SealedDomain(sd) => {
                let domain_summary = lower_sealed_domain(sd, module, &module_identity);
                summary = summary.with_exported_sealed_domain(domain_summary);
                has_sealed_domains = true;
            }
            crate::surface::Definition::TypeFn(type_fn) => {
                type_function_defs.push(type_fn.clone());
            }
            _ => continue,
        }
    }

    if has_sealed_domains {
        summary.version = ash_core::semantic_summary::SummaryVersion::SPEC059_SEALED_DOMAIN_V2;
    }

    LoweredTypeMetadata {
        type_defs,
        type_function_defs,
        summary,
    }
}

#[must_use]
pub fn lower_surface_type_def(type_def: &crate::surface::TypeDef) -> ash_core::ast::TypeDef {
    ash_core::ast::TypeDef {
        name: type_def.name.to_string(),
        params: type_def.params.iter().map(ToString::to_string).collect(),
        body: lower_surface_type_body(&type_def.body),
        visibility: lower_surface_visibility(&type_def.visibility),
        builtin: type_def.builtin,
    }
}

/// Lower a sealed-domain surface declaration into a `SealedDomainSummary`.
///
/// Creates canonical identities for the domain and its marker constructors,
/// maps field slots to unconstrained or domain-constrained summaries, and
/// derives `StructuralSelfDomain` status for self-referencing fields.
fn lower_sealed_domain(
    sd: &crate::surface::SealedDomainDef,
    module: &crate::surface::ModuleFile,
    module_identity: &ash_core::semantic_summary::ModuleIdentity,
) -> ash_core::semantic_summary::SealedDomainSummary {
    use ash_core::semantic_summary::{
        DomainConstructorId, DomainConstructorSummary, DomainFieldSummary, SealedDomainId,
        SealedDomainSummary,
    };

    let domain_id = SealedDomainId::new(module_identity.clone(), sd.name.clone());
    let visibility = lower_surface_visibility(&sd.visibility);

    let domain_anchor = source_anchor_for_sealed_domain(sd, module, module_identity);

    let mut domain_summary = SealedDomainSummary::new(
        domain_id.clone(),
        sd.name.clone(),
        visibility,
        domain_anchor,
    );

    for ctor in &sd.constructors {
        let ctor_id = DomainConstructorId::new(domain_id.clone(), ctor.name.clone());
        let ctor_anchor = source_anchor_for_domain_constructor(ctor, module, module_identity);

        let fields: Vec<DomainFieldSummary> = ctor
            .fields
            .iter()
            .map(|field| match &field.slot {
                crate::surface::DomainSlot::Type => {
                    DomainFieldSummary::unconstrained(field.name.clone())
                }
                crate::surface::DomainSlot::DomainRef(ref_name) => {
                    let target_domain_id = if ref_name.as_ref() == sd.name.as_ref() {
                        domain_id.clone()
                    } else {
                        SealedDomainId::new(module_identity.clone(), ref_name.clone())
                    };
                    DomainFieldSummary::constrained_to(
                        field.name.clone(),
                        &domain_id,
                        target_domain_id,
                    )
                }
            })
            .collect();

        let ctor_summary =
            DomainConstructorSummary::new(ctor_id, ctor.name.clone(), fields, ctor_anchor);
        domain_summary = domain_summary.with_constructor(ctor_summary);
    }

    domain_summary
}

fn source_anchor_for_sealed_domain(
    sd: &crate::surface::SealedDomainDef,
    module: &crate::surface::ModuleFile,
    module_identity: &ash_core::semantic_summary::ModuleIdentity,
) -> ash_core::semantic_summary::SourceAnchor {
    let origin = source_origin_from_module(module, module_identity);
    ash_core::semantic_summary::SourceAnchor::new(
        origin,
        Some(to_core_span(sd.span)),
        format!("sealed type domain {}", sd.name),
    )
}

fn source_anchor_for_domain_constructor(
    ctor: &crate::surface::DomainConstructor,
    module: &crate::surface::ModuleFile,
    module_identity: &ash_core::semantic_summary::ModuleIdentity,
) -> ash_core::semantic_summary::SourceAnchor {
    let origin = source_origin_from_module(module, module_identity);
    ash_core::semantic_summary::SourceAnchor::new(
        origin,
        Some(to_core_span(ctor.span)),
        format!("domain constructor {}", ctor.name),
    )
}

fn lower_surface_type_body(body: &crate::surface::TypeBody) -> ash_core::ast::TypeBody {
    match body {
        crate::surface::TypeBody::Struct(fields) => ash_core::ast::TypeBody::Struct(
            fields
                .iter()
                .map(|field| (field.name.to_string(), lower_surface_type(&field.ty)))
                .collect(),
        ),
        crate::surface::TypeBody::Enum(variants) => {
            ash_core::ast::TypeBody::Enum(variants.iter().map(lower_surface_variant_def).collect())
        }
        crate::surface::TypeBody::Alias(ty) => {
            ash_core::ast::TypeBody::Alias(lower_surface_type(ty))
        }
    }
}

fn lower_surface_variant_def(variant: &crate::surface::VariantDef) -> ash_core::ast::VariantDef {
    ash_core::ast::VariantDef {
        name: variant.name.to_string(),
        fields: variant
            .fields
            .iter()
            .map(|field| (field.name.to_string(), lower_surface_type(&field.ty)))
            .collect(),
        payload: lower_surface_variant_payload(&variant.payload),
    }
}

fn lower_surface_variant_payload(
    payload: &crate::surface::VariantPayload,
) -> ash_core::ast::VariantPayload {
    match payload {
        crate::surface::VariantPayload::Unit => ash_core::ast::VariantPayload::Unit,
        crate::surface::VariantPayload::Record(fields) => ash_core::ast::VariantPayload::Record(
            fields
                .iter()
                .map(|field| (field.name.to_string(), lower_surface_type(&field.ty)))
                .collect(),
        ),
        crate::surface::VariantPayload::Tuple(items) => {
            ash_core::ast::VariantPayload::Tuple(items.iter().map(lower_surface_type).collect())
        }
    }
}

fn lower_surface_visibility(visibility: &crate::surface::Visibility) -> ash_core::ast::Visibility {
    match visibility {
        crate::surface::Visibility::Public => ash_core::ast::Visibility::Public,
        crate::surface::Visibility::Crate => ash_core::ast::Visibility::Crate,
        _ => ash_core::ast::Visibility::Private,
    }
}

fn constructor_payload_kind(
    payload: &ash_core::ast::VariantPayload,
) -> ash_core::semantic_summary::ConstructorPayloadKind {
    match payload {
        ash_core::ast::VariantPayload::Unit => {
            ash_core::semantic_summary::ConstructorPayloadKind::Unit
        }
        ash_core::ast::VariantPayload::Record(_) => {
            ash_core::semantic_summary::ConstructorPayloadKind::Record
        }
        ash_core::ast::VariantPayload::Tuple(_) => {
            ash_core::semantic_summary::ConstructorPayloadKind::Tuple
        }
    }
}

fn source_anchor_for_module(
    module: &crate::surface::ModuleFile,
    module_identity: &ash_core::semantic_summary::ModuleIdentity,
) -> ash_core::semantic_summary::SourceAnchor {
    ash_core::semantic_summary::SourceAnchor::new(
        source_origin_from_module(module, module_identity),
        Some(to_core_span(module.span)),
        "module",
    )
}

fn source_anchor_for_type(
    type_def: &crate::surface::TypeDef,
    module: &crate::surface::ModuleFile,
    module_identity: &ash_core::semantic_summary::ModuleIdentity,
) -> ash_core::semantic_summary::SourceAnchor {
    let origin = type_def
        .source
        .as_ref()
        .map(|source| ash_core::semantic_summary::SourceOrigin::File(source.to_string()))
        .unwrap_or_else(|| source_origin_from_module(module, module_identity));
    ash_core::semantic_summary::SourceAnchor::new(
        origin,
        Some(to_core_span(type_def.span)),
        format!("type {}", type_def.name),
    )
}

fn source_origin_from_module(
    module: &crate::surface::ModuleFile,
    module_identity: &ash_core::semantic_summary::ModuleIdentity,
) -> ash_core::semantic_summary::SourceOrigin {
    if let Some(path) = &module.path {
        return ash_core::semantic_summary::SourceOrigin::File(path.to_string());
    }

    match &module_identity.source {
        ash_core::semantic_summary::ModuleSourceOrigin::File(path) => {
            ash_core::semantic_summary::SourceOrigin::File(path.clone())
        }
        ash_core::semantic_summary::ModuleSourceOrigin::Inline { parent, offset } => {
            ash_core::semantic_summary::SourceOrigin::InlineModule {
                module: *parent,
                offset: *offset,
            }
        }
        ash_core::semantic_summary::ModuleSourceOrigin::Synthetic { reason } => {
            ash_core::semantic_summary::SourceOrigin::Synthetic {
                reason: reason.clone(),
            }
        }
    }
}

fn to_core_span(span: crate::token::Span) -> ash_core::ast::Span {
    ash_core::ast::Span {
        start: span.start,
        end: span.end,
    }
}

/// Lower a surface interface definition to core AST.
pub fn lower_interface_def(
    iface: &crate::surface::InterfaceDef,
) -> Result<ash_core::ast::InterfaceDef, LoweringError> {
    use ash_core::ast::{
        AssociatedType, InterfaceDef, InterfaceEvidenceConstraint, InterfaceMethodSig, Visibility,
    };
    reject_kinded_interface_type_params(
        &iface.type_params,
        "kinded interface parameters are parsed by TASK-906 but lowered by TASK-907",
    )?;
    Ok(InterfaceDef {
        name: iface.name.to_string(),
        type_params: iface.type_params.iter().map(|n| n.to_string()).collect(),
        evidence_constraints: iface
            .evidence_constraints
            .iter()
            .map(|constraint| InterfaceEvidenceConstraint {
                subject: lower_surface_type(&constraint.subject),
                required_evidence: lower_surface_type(&constraint.interface),
            })
            .collect(),
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
    reject_kinded_interface_type_params(
        &impl_def.type_params,
        "kinded impl parameters are parsed by TASK-906 but lowered by TASK-907",
    )?;
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
    "__unit",
    "__bind",
    "__then",
    "__fail",
];

/// Lower a surface expression to core IR.
pub fn lower_expr_with_context(
    expr: &Expr,
    ctx: &LoweringContext,
) -> Result<CoreExpr, LoweringError> {
    with_active_effectful_names(&ctx.effectful_names, || lower_expr(expr))
}

/// Lower a surface expression to core IR.
pub fn lower_expr(expr: &Expr) -> Result<CoreExpr, LoweringError> {
    match expr {
        Expr::OperatorSection { section } => Err(LoweringError::UnsupportedFeature(format!(
            "operator section `{}` must be resolved before Core lowering",
            section.operator.spelling
        ))),
        Expr::MacroInvocation { invocation } => Err(LoweringError::UnsupportedFeature(format!(
            "unexpanded macro invocation carrier `{}!` reached lowering",
            invocation.name
        ))),
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

        Expr::Fail { payload, .. } => Ok(CoreExpr::Fail {
            payload: Box::new(lower_expr(payload)?),
        }),

        Expr::WithError { body, arms, .. } => Ok(CoreExpr::WithError {
            body: Box::new(lower_expr(body)?),
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
        Expr::Record { fields, .. } => Ok(CoreExpr::Record {
            fields: fields
                .iter()
                .map(|(name, expr)| Ok((name.to_string(), lower_expr(expr)?)))
                .collect::<Result<Vec<_>, _>>()?,
        }),

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
        Expr::Block {
            statements,
            tail_expr,
            span: _,
        } => {
            // Desugar { let x = e1; let y = e2; tail } into nested Expr::Let
            let tail = tail_expr
                .as_deref()
                .map_or_else(|| Ok(CoreExpr::Literal(ash_core::Value::Null)), lower_expr)?;

            let mut result = tail;
            for stmt in statements.iter().rev() {
                match stmt {
                    BlockStmt::Let {
                        pattern,
                        expr,
                        span: stmt_span,
                    } => {
                        result = CoreExpr::Let {
                            pattern: lower_pattern(pattern)?,
                            expr: Box::new(lower_expr(expr)?),
                            body: Box::new(result),
                            span: ash_core::Span {
                                start: stmt_span.start,
                                end: stmt_span.end,
                            },
                        };
                    }
                }
            }
            Ok(result)
        }

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

        Expr::ActBlock { stmts, .. } => lower_act_block(stmts),

        Expr::List { items, .. } => {
            // Lower [a, b, c] to Cons { head: a, tail: Cons { head: b, tail: Cons { head: c, tail: Nil } } }
            // by building from right to left
            let mut result = CoreExpr::Constructor {
                name: "Nil".to_string(),
                fields: vec![],
            };
            for item in items.iter().rev() {
                result = CoreExpr::Constructor {
                    name: "Cons".to_string(),
                    fields: vec![
                        ("head".to_string(), lower_expr(item)?),
                        ("tail".to_string(), result),
                    ],
                };
            }
            Ok(result)
        }

        Expr::DoBlock {
            target,
            stmts,
            span,
        } if target.name.as_ref() == "__ambient" && target.args.is_empty() => {
            lower_ambient_do_block(stmts, *span)
        }

        Expr::DoBlock { .. } => Err(LoweringError::ExprNotLowerable {
            kind: "generic do block requires typed do elaboration before lowering",
        }),

        Expr::Comprehension { .. } => Err(LoweringError::ExprNotLowerable {
            kind: "comprehension requires typed do elaboration before lowering",
        }),
    }
}

fn lower_ambient_do_block(
    stmts: &[DoStmt],
    _span: crate::token::Span,
) -> Result<CoreExpr, LoweringError> {
    if stmts.is_empty() {
        return Err(LoweringError::UnsupportedFeature(
            "empty target ambient do block".to_string(),
        ));
    }

    for (index, stmt) in stmts.iter().enumerate() {
        if matches!(stmt, DoStmt::Return { .. }) && index + 1 < stmts.len() {
            return Err(LoweringError::UnsupportedFeature(
                "return must be the last statement in a target ambient do block".to_string(),
            ));
        }
    }

    let Some(DoStmt::Return { value, .. }) = stmts.last() else {
        return Err(LoweringError::UnsupportedFeature(
            "target ambient do block must end with a return statement".to_string(),
        ));
    };

    let mut result = lower_expr(value)?;
    for stmt in stmts[..stmts.len() - 1].iter().rev() {
        match stmt {
            DoStmt::Let {
                name,
                value,
                span: stmt_span,
            }
            | DoStmt::Bind {
                name,
                value,
                span: stmt_span,
            } => {
                result = CoreExpr::Let {
                    pattern: ash_core::Pattern::Variable {
                        name: name.to_string(),
                        span: ash_core::Span {
                            start: stmt_span.start,
                            end: stmt_span.end,
                        },
                    },
                    expr: Box::new(lower_expr(value)?),
                    body: Box::new(result),
                    span: ash_core::Span {
                        start: stmt_span.start,
                        end: stmt_span.end,
                    },
                };
            }
            DoStmt::WorkflowRequires { .. } | DoStmt::WorkflowEnsures { .. } => {
                return Err(LoweringError::UnsupportedFeature(
                    "workflow contract statement requires explicit workflow/profile elaboration"
                        .to_string(),
                ));
            }
            DoStmt::Return { .. } => {
                return Err(LoweringError::UnsupportedFeature(
                    "return must be the last statement in a target ambient do block".to_string(),
                ));
            }
        }
    }

    Ok(result)
}

/// Lower an act block into nested bind/unit calls. SPEC-047 §6.2
///
/// Empty act blocks and invalid statement sequences (e.g., return followed by
/// more statements) are rejected with a lowering error per the spec contract.
fn lower_act_block(stmts: &[crate::surface::ActStmt]) -> Result<CoreExpr, LoweringError> {
    match stmts {
        [] => Err(LoweringError::ExprNotLowerable {
            kind: "empty act block",
        }),
        [crate::surface::ActStmt::Return { value, .. }] => {
            let lowered = lower_expr(value)?;
            Ok(CoreExpr::Call {
                func: "unit".to_string(),
                module: None,
                arguments: vec![lowered],
            })
        }
        [crate::surface::ActStmt::Bind { name, value, .. }, rest @ ..] => {
            let lowered_value = lower_expr(value)?;
            let monadic_value = if is_act_like_surface_expr(value) {
                lowered_value
            } else {
                CoreExpr::Call {
                    func: "unit".to_string(),
                    module: None,
                    arguments: vec![lowered_value],
                }
            };
            let body = lower_act_block(rest)?;
            Ok(CoreExpr::Call {
                func: "bind".to_string(),
                module: None,
                arguments: vec![
                    monadic_value,
                    CoreExpr::FnDef {
                        params: vec![(name.to_string(), None)],
                        return_type: None,
                        body: Box::new(body),
                    },
                ],
            })
        }
        // Catch-all: invalid sequence (e.g., Return followed by more statements)
        _ => Err(LoweringError::ExprNotLowerable {
            kind: "invalid act statement sequence (return must be last)",
        }),
    }
}

fn is_act_like_surface_expr(expr: &Expr) -> bool {
    match expr {
        Expr::ActBlock { .. } => true,
        Expr::DoBlock { .. } => false,
        Expr::Call { func, module, .. } if module.is_none() => {
            matches!(
                func.as_ref(),
                "invoke"
                    | "bind"
                    | "then"
                    | "guard"
                    | "unit"
                    | "__unit"
                    | "__bind"
                    | "__then"
                    | "__fail"
            ) || active_effectful_names_contains(func.as_ref())
        }
        _ => false,
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
/// Low-level helper used by tests and parser/lowering experiments.
///
/// Production module-file boundaries validate the parsed module through surface
/// expansion before accepting exports; they do not route individual expression
/// snippets through this helper because module-local notation context belongs to
/// the whole `ModuleFile`.
pub fn lower_module_expr(expr: &Expr) -> Result<CoreExpr, LoweringError> {
    if matches!(expr, Expr::FnDef { .. }) {
        return Err(LoweringError::FnDefNotAllowedAtModuleScope);
    }
    lower_expr(expr)
}

/// Lower a module only after it has crossed the expanded-surface boundary.
///
/// This gate intentionally lowers expression-bearing surfaces for validation rather than claiming a
/// complete module-to-Core product. It prevents parsed-surface-only notation or operator sections
/// from bypassing expansion at high-level module boundaries.
pub fn lower_expanded_surface_module(module: &ExpandedSurfaceModule) -> Result<(), LoweringError> {
    let mut result = Ok(());
    visit_exprs_in_module(&module.module, &mut |expr| {
        if result.is_ok() {
            result = lower_expr(expr).map(|_| ());
        }
    });
    result
}

/// Expand a parsed module, then validate expression lowering through the expanded-surface gate.
pub fn expand_and_lower_surface_module(module: ModuleFile) -> Result<(), LoweringError> {
    let expanded = expand_surface_module(module)
        .map_err(|err| LoweringError::UnsupportedFeature(err.to_string()))?;
    lower_expanded_surface_module(&expanded)
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

/// Lower a surface `BuiltinFnDef` to core IR.
///
/// Produces a callable registration with no body expression --
/// the builtin marker indicates runtime dispatch by the host environment.
pub fn lower_builtin_fn_def(
    def: &crate::surface::BuiltinFnDef,
) -> Result<ash_core::ast::BuiltinFnDef, LoweringError> {
    use ash_core::ast::Visibility;

    reject_kinded_type_params(
        &def.type_params,
        "kinded builtin function type parameters are parsed by TASK-906 but lowered by TASK-907",
    )?;

    Ok(ash_core::ast::BuiltinFnDef {
        name: def.name.to_string(),
        type_params: def.type_params.iter().map(|n| n.to_string()).collect(),
        params: def
            .params
            .iter()
            .map(|p| (p.name.to_string(), lower_surface_type(&p.ty)))
            .collect(),
        return_type: lower_surface_type(&def.return_type),
        visibility: match def.visibility {
            crate::surface::Visibility::Public => Visibility::Public,
            crate::surface::Visibility::Crate => Visibility::Crate,
            _ => Visibility::Private,
        },
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
            Ok(Value::list_from_vec(lowered))
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
mod tests;
