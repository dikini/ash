//! Surface AST to Core IR lowering.
//!
//! This module converts the surface syntax AST into the core IR representation
//! used by the ash-core crate.
//!
//! The `contract_predicate` submodule provides the TASK-1893/1894 bridge from
//! surface predicate expressions to core [`ContractPredicateExpr`] carriers.

use std::{cell::RefCell, fmt};

use ash_core::adt::tuple_field_name;
use ash_core::{
    Capability, Effect, Expr as CoreExpr, Kind, MatchArm as CoreMatchArm, Pattern as CorePattern,
    Predicate as CorePredicate,
};

#[cfg(test)]
use ash_core::{Role as CoreRole, RoleObligationRef as CoreRoleObligationRef};

use crate::capability_export::{CapabilityResolutionContext, ModuleId};
use crate::surface::{
    BinaryOp, BlockStmt, CapabilityDef, DoStmt, EffectType, ExpandedSurfaceModule, Expr, Literal,
    ModuleFile, Pattern, PolicyExpr, Predicate, Type, UnaryOp, expand_surface_module,
    visit_exprs_in_module,
};

pub mod contract_predicate;

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

/// Lowered fn contract sidecars produced by the TASK-1895 Core predicate lowering boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct LoweredFnContract {
    /// Requires-clause discharge records, one per precondition.
    pub requires_discharges: Vec<ash_core::core_ash_contract::ContractDischargeRecord>,
    /// Ensures-clause discharge records, one per postcondition.
    pub ensures_discharges: Vec<ash_core::core_ash_contract::ContractDischargeRecord>,
    /// Retained classified contract carrier used by current contract/runtime hooks.
    pub contract: ash_core::contract::Contract,
    /// Explicit runtime postcondition boundary for interpreter hooks.
    pub runtime_postconditions: ash_core::contract::RuntimePostconditionContract,
}

impl LoweredFnContract {
    /// Returns all discharge records (requires then ensures).
    #[must_use]
    pub fn discharges(&self) -> Vec<ash_core::core_ash_contract::ContractDischargeRecord> {
        let mut all = self.requires_discharges.clone();
        all.extend(self.ensures_discharges.clone());
        all
    }
}

/// Context needed to build a [`PredicateEnvironment`] for a fn contract.
#[derive(Debug, Clone, PartialEq)]
pub struct FnContractLoweringContext<'a> {
    /// Owning callable name for boundary identity.
    pub name: &'a str,
    /// Parameters with their Core types.
    pub params: &'a [(String, ash_core::core_ash::CoreType)],
    /// Optional return type of the callable.
    pub result: Option<ash_core::core_ash::CoreType>,
}

/// Lower a parsed fn contract into the TASK-1895 Core predicate sidecars.
///
/// Builds a boundary-local [`PredicateEnvironment`] from the supplied parameters
/// and result type, then translates each `requires`/`ensures` clause into a
/// [`ContractPredicateExpr`] and lowers it through [`lower_contract_predicate`].
/// The resulting discharge records are attached to the returned
/// [`LoweredFnContract`].
pub fn lower_fn_contract(
    contract: Option<&crate::surface::Contract>,
    ctx: &FnContractLoweringContext<'_>,
) -> Result<LoweredFnContract, FnContractLoweringError> {
    let Some(contract) = contract else {
        return Ok(LoweredFnContract {
            requires_discharges: Vec::new(),
            ensures_discharges: Vec::new(),
            contract: ash_core::contract::Contract::default(),
            runtime_postconditions: ash_core::contract::RuntimePostconditionContract::default(),
        });
    };

    let requires_env = build_predicate_environment(ctx, BoundaryKind::Requires);
    let ensures_env = build_predicate_environment(ctx, BoundaryKind::Ensures);

    let mut requires_discharges = Vec::new();
    let mut requires = Vec::new();
    for (index, requirement) in contract.requires.iter().enumerate() {
        let (predicate, discharge) = lower_fn_requirement(requirement, ctx, &requires_env, index)?;
        requires_discharges.push(discharge);
        requires.push(predicate);
    }

    let mut ensures_discharges = Vec::new();
    let mut ensures = Vec::new();
    for (index, clause) in contract.ensures.iter().enumerate() {
        let (predicate, discharge) = lower_fn_ensures_clause(clause, ctx, &ensures_env, index)?;
        ensures_discharges.push(discharge);
        ensures.push(predicate);
    }

    let runtime_postconditions = ash_core::contract::RuntimePostconditionContract {
        predicates: ensures.clone(),
    };

    Ok(LoweredFnContract {
        requires_discharges,
        ensures_discharges,
        contract: ash_core::contract::Contract { requires, ensures },
        runtime_postconditions,
    })
}

fn boundary_id(name: &str, kind: BoundaryKind) -> ash_core::core_ash_contract::CoreBoundaryId {
    use ash_core::core_ash_contract::CoreBoundaryId;
    match kind {
        BoundaryKind::Requires => CoreBoundaryId::new(format!("fn:{name}:requires")),
        BoundaryKind::Ensures => CoreBoundaryId::new(format!("fn:{name}:ensures")),
    }
}

fn boundary_kind_to_core(kind: BoundaryKind) -> ash_core::core_ash_contract::BoundaryKind {
    match kind {
        BoundaryKind::Requires => ash_core::core_ash_contract::BoundaryKind::Requires,
        BoundaryKind::Ensures => ash_core::core_ash_contract::BoundaryKind::Ensures,
    }
}

fn build_predicate_environment(
    ctx: &FnContractLoweringContext<'_>,
    kind: BoundaryKind,
) -> ash_core::core_ash_contract::PredicateEnvironment {
    use ash_core::core_ash_contract::{PredicateBinder, PredicateBinderKind, PredicateEnvironment};

    let boundary = boundary_id(ctx.name, kind);
    let mut binders: Vec<PredicateBinder> = ctx
        .params
        .iter()
        .enumerate()
        .map(|(index, (name, ty))| {
            PredicateBinder::new(
                boundary.clone(),
                name.clone(),
                name.clone(),
                PredicateBinderKind::Parameter,
                ty.clone(),
                ash_core::core_ash::CoreSourceSpan {
                    file: None,
                    start: index,
                    end: index.saturating_add(1),
                },
            )
        })
        .collect();

    if let Some(result_ty) = &ctx.result {
        binders.push(PredicateBinder::new(
            boundary.clone(),
            "result".to_string(),
            "result".to_string(),
            PredicateBinderKind::Result,
            result_ty.clone(),
            ash_core::core_ash::CoreSourceSpan {
                file: None,
                start: 0,
                end: 1,
            },
        ));
    }

    PredicateEnvironment::new(boundary, binders, Vec::new(), Vec::new())
}

pub fn surface_type_to_core_type(ty: &crate::surface::Type) -> ash_core::core_ash::CoreType {
    use ash_core::core_ash::CoreType;
    match ty {
        crate::surface::Type::Name(name) => CoreType::Base(name.to_string()),
        crate::surface::Type::Capability(name) => CoreType::Named(name.to_string()),
        crate::surface::Type::Constructor { name, args } => CoreType::App {
            name: name.to_string(),
            args: args.iter().map(surface_type_to_core_type).collect(),
        },
        crate::surface::Type::List(inner) => CoreType::App {
            name: "List".to_string(),
            args: vec![surface_type_to_core_type(inner)],
        },
        crate::surface::Type::Tuple(items) => {
            CoreType::Tuple(items.iter().map(surface_type_to_core_type).collect())
        }
        crate::surface::Type::Record(fields) => CoreType::Record(
            fields
                .iter()
                .map(|(n, t)| (n.to_string(), surface_type_to_core_type(t)))
                .collect(),
        ),
        crate::surface::Type::Fn(params, _row, ret) => CoreType::Function {
            params: params.iter().map(surface_type_to_core_type).collect(),
            result: Box::new(surface_type_to_core_type(ret)),
            row: ash_core::core_ash::CoreRow::default(),
        },
        crate::surface::Type::Associated { base, name } => CoreType::App {
            name: name.to_string(),
            args: vec![surface_type_to_core_type(base)],
        },
        _ => CoreType::Base("?".to_string()),
    }
}

fn lower_fn_requirement(
    requirement: &crate::surface::Requirement,
    ctx: &FnContractLoweringContext<'_>,
    env: &ash_core::core_ash_contract::PredicateEnvironment,
    index: usize,
) -> Result<
    (
        ash_core::contract::Requirement,
        ash_core::core_ash_contract::ContractDischargeRecord,
    ),
    FnContractLoweringError,
> {
    match requirement {
        crate::surface::Requirement::Arithmetic { expr } => {
            let contract_text = format!("{expr:?}");
            let (var, constraint) = lower_stage1_arith_predicate(expr)
                .map_err(|message| FnContractLoweringError::InvalidRequires { message })?;
            let predicate_expr = arith_constraint_to_predicate_expr(
                &var,
                &constraint,
                env,
                ctx,
                BoundaryKind::Requires,
            )?;
            let span = ash_core::core_ash::CoreSourceSpan {
                file: None,
                start: 0,
                end: 1,
            };
            let discharge = lower_predicate_to_discharge(
                predicate_expr,
                env,
                BoundaryKind::Requires,
                index,
                contract_text,
                span,
            )
            .map_err(|message| FnContractLoweringError::InvalidRequires { message })?;
            Ok((
                ash_core::contract::Requirement::Arithmetic { var, constraint },
                discharge,
            ))
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
    ctx: &FnContractLoweringContext<'_>,
    env: &ash_core::core_ash_contract::PredicateEnvironment,
    index: usize,
) -> Result<
    (
        ash_core::contract::PostPredicate,
        ash_core::core_ash_contract::ContractDischargeRecord,
    ),
    FnContractLoweringError,
> {
    let contract_text = format!("{clause:?}");
    if let Some(constraint) = lower_result_constraint(&clause.expr) {
        let predicate_expr = arith_constraint_to_predicate_expr(
            "result",
            &constraint,
            env,
            ctx,
            BoundaryKind::Ensures,
        )?;
        let span = ash_core::core_ash::CoreSourceSpan {
            file: None,
            start: 0,
            end: 1,
        };
        let discharge = lower_predicate_to_discharge(
            predicate_expr,
            env,
            BoundaryKind::Ensures,
            index,
            contract_text,
            span,
        )
        .map_err(|message| FnContractLoweringError::InvalidEnsures { message })?;
        return Ok((
            ash_core::contract::PostPredicate::ResultSatisfies(constraint),
            discharge,
        ));
    }

    if let Some((left, right)) = lower_result_equality(&clause.expr) {
        let predicate_expr = result_equality_to_predicate_expr(&left, &right, env, ctx)?;
        let span = ash_core::core_ash::CoreSourceSpan {
            file: None,
            start: 0,
            end: 1,
        };
        let discharge = lower_predicate_to_discharge(
            predicate_expr,
            env,
            BoundaryKind::Ensures,
            index,
            contract_text,
            span,
        )
        .map_err(|message| FnContractLoweringError::InvalidEnsures { message })?;
        return Ok((
            ash_core::contract::PostPredicate::Eq(left, right),
            discharge,
        ));
    }

    Err(FnContractLoweringError::InvalidEnsures {
        message: "fn ensures clauses must be value-level predicates over `result` or simple equality; state assertions are not allowed".to_string(),
    })
}

fn lower_predicate_to_discharge(
    expr: ash_core::core_ash_contract::ContractPredicateExpr,
    env: &ash_core::core_ash_contract::PredicateEnvironment,
    kind: BoundaryKind,
    index: usize,
    contract_text: String,
    span: ash_core::core_ash::CoreSourceSpan,
) -> Result<ash_core::core_ash_contract::ContractDischargeRecord, String> {
    use ash_core::core_ash_contract::{
        ContractRecoverability, CoreBlameLabel, CoreBlameParty, CoreBlamePolarity,
    };

    let boundary = env.boundary().clone();
    let _boundary_kind = boundary_kind_to_core(kind);
    let blame = match kind {
        BoundaryKind::Requires => CoreBlameLabel::new(
            CoreBlameParty::Caller,
            CoreBlamePolarity::Negative,
            boundary.clone(),
        ),
        BoundaryKind::Ensures => CoreBlameLabel::new(
            CoreBlameParty::Callee,
            CoreBlamePolarity::Positive,
            boundary.clone(),
        ),
    };

    let lowering = ash_core::core_ash_contract::lower_contract_predicate(
        boundary.clone(),
        env.clone(),
        expr,
        ash_core::core_ash::CoreType::Base("Bool".to_string()),
        span.clone(),
        contract_text,
        blame.clone(),
        ContractRecoverability::TrapDefault,
    )
    .map_err(|error| format!("{error:?}"))?;

    let status = if let Some(plan) = lowering.runtime_check {
        ash_core::core_ash_contract::ContractDischargeStatus::Dynamic {
            plan: Box::new(plan),
        }
    } else {
        ash_core::core_ash_contract::ContractDischargeStatus::Deferred {
            reason: "classified-contract-deferred".into(),
        }
    };

    Ok(ash_core::core_ash_contract::ContractDischargeRecord::new(
        format!("fn-contract-{index}"),
        boundary,
        status,
        span,
        Some(blame),
    ))
}

fn arith_constraint_to_predicate_expr(
    var_name: &str,
    constraint: &ash_core::contract::ArithConstraint,
    env: &ash_core::core_ash_contract::PredicateEnvironment,
    ctx: &FnContractLoweringContext<'_>,
    _kind: BoundaryKind,
) -> Result<ash_core::core_ash_contract::ContractPredicateExpr, FnContractLoweringError> {
    use ash_core::core_ash_contract::ContractPredicateExpr;

    let binder =
        find_binder_ref(env, var_name).ok_or_else(|| FnContractLoweringError::InvalidRequires {
            message: format!("unknown contract variable '{var_name}'"),
        })?;
    let binder_expr = if var_name == "result" {
        ContractPredicateExpr::Result(binder)
    } else {
        ContractPredicateExpr::Binder(binder)
    };
    let value_ty = if let Some((_, ty)) = ctx.params.iter().find(|(n, _)| n == var_name) {
        ty.clone()
    } else if var_name == "result" {
        ctx.result
            .clone()
            .unwrap_or(ash_core::core_ash::CoreType::Base("Int".to_string()))
    } else {
        ash_core::core_ash::CoreType::Base("Int".to_string())
    };

    let int_lit = |v: i64| ContractPredicateExpr::IntLit(i128::from(v));
    let cmp_expr = match constraint {
        ash_core::contract::ArithConstraint::Gt(v) => {
            ContractPredicateExpr::Gt(Box::new(binder_expr), Box::new(int_lit(*v)))
        }
        ash_core::contract::ArithConstraint::Lt(v) => {
            ContractPredicateExpr::Lt(Box::new(binder_expr), Box::new(int_lit(*v)))
        }
        ash_core::contract::ArithConstraint::Gte(v) => {
            ContractPredicateExpr::Ge(Box::new(binder_expr), Box::new(int_lit(*v)))
        }
        ash_core::contract::ArithConstraint::Lte(v) => {
            ContractPredicateExpr::Le(Box::new(binder_expr), Box::new(int_lit(*v)))
        }
        ash_core::contract::ArithConstraint::Eq(v) => {
            ContractPredicateExpr::Eq(Box::new(binder_expr), Box::new(int_lit(*v)))
        }
        ash_core::contract::ArithConstraint::NotEq(v) => {
            ContractPredicateExpr::Ne(Box::new(binder_expr), Box::new(int_lit(*v)))
        }
        ash_core::contract::ArithConstraint::Modulo { div, rem } => {
            let rem_expr =
                ContractPredicateExpr::Rem(Box::new(binder_expr.clone()), Box::new(int_lit(*div)));
            ContractPredicateExpr::Eq(Box::new(rem_expr), Box::new(int_lit(*rem)))
        }
        ash_core::contract::ArithConstraint::Range { min, max } => {
            let ge_min =
                ContractPredicateExpr::Ge(Box::new(binder_expr.clone()), Box::new(int_lit(*min)));
            let le_max = ContractPredicateExpr::Le(Box::new(binder_expr), Box::new(int_lit(*max)));
            ContractPredicateExpr::And(Box::new(ge_min), Box::new(le_max))
        }
    };
    Ok(cast_expr_to_type(cmp_expr, &value_ty))
}

fn result_equality_to_predicate_expr(
    left: &str,
    right: &str,
    env: &ash_core::core_ash_contract::PredicateEnvironment,
    ctx: &FnContractLoweringContext<'_>,
) -> Result<ash_core::core_ash_contract::ContractPredicateExpr, FnContractLoweringError> {
    use ash_core::core_ash_contract::ContractPredicateExpr;

    let left_expr = if left == "result" {
        let binder = find_binder_ref(env, "result").ok_or_else(|| {
            FnContractLoweringError::InvalidEnsures {
                message: "result binder not available".to_string(),
            }
        })?;
        ContractPredicateExpr::Result(binder)
    } else {
        contract_predicate_expr_from_simple_value(left, env, ctx).ok_or_else(|| {
            FnContractLoweringError::InvalidEnsures {
                message: format!("unsupported equality operand '{left}'"),
            }
        })?
    };
    let right_expr =
        contract_predicate_expr_from_simple_value(right, env, ctx).ok_or_else(|| {
            FnContractLoweringError::InvalidEnsures {
                message: format!("unsupported equality operand '{right}'"),
            }
        })?;
    Ok(ContractPredicateExpr::Eq(
        Box::new(left_expr),
        Box::new(right_expr),
    ))
}

fn contract_predicate_expr_from_simple_value(
    value: &str,
    env: &ash_core::core_ash_contract::PredicateEnvironment,
    _ctx: &FnContractLoweringContext<'_>,
) -> Option<ash_core::core_ash_contract::ContractPredicateExpr> {
    use ash_core::core_ash_contract::ContractPredicateExpr;

    if value == "result" {
        let binder = find_binder_ref(env, "result")?;
        return Some(ContractPredicateExpr::Result(binder));
    }

    if let Some(binder) = find_binder_ref(env, value) {
        return Some(ContractPredicateExpr::Binder(binder));
    }

    if value.parse::<i64>().is_ok() {
        return Some(ContractPredicateExpr::IntLit(i128::from(
            value.parse::<i64>().ok()?,
        )));
    }

    if value.parse::<bool>().is_ok() {
        return Some(ContractPredicateExpr::BoolLit(value.parse::<bool>().ok()?));
    }

    if value == "null" {
        return Some(ContractPredicateExpr::UnitLit);
    }

    if value.starts_with('"') && value.ends_with('"') {
        return Some(ContractPredicateExpr::StringLit(
            value[1..value.len() - 1].to_string(),
        ));
    }

    if value == "true" {
        return Some(ContractPredicateExpr::BoolLit(true));
    }
    if value == "false" {
        return Some(ContractPredicateExpr::BoolLit(false));
    }

    None
}

fn find_binder_ref(
    env: &ash_core::core_ash_contract::PredicateEnvironment,
    name: &str,
) -> Option<ash_core::core_ash_contract::PredicateBinderRef> {
    env.binders()
        .iter()
        .find(|b| b.id().local() == name)
        .map(|b| b.ref_())
}

fn cast_expr_to_type(
    expr: ash_core::core_ash_contract::ContractPredicateExpr,
    ty: &ash_core::core_ash::CoreType,
) -> ash_core::core_ash_contract::ContractPredicateExpr {
    let _ = ty;
    expr
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoundaryKind {
    Requires,
    Ensures,
}

fn lower_stage1_arith_predicate(
    expr: &Expr,
) -> Result<(String, ash_core::contract::ArithConstraint), String> {
    match expr {
        Expr::Binary {
            op, left, right, ..
        } => {
            if let (Some(var), Some(value)) = (variable_name(left), int_literal(right)) {
                let constraint = match op {
                    BinaryOp::Gt => ash_core::contract::ArithConstraint::Gt(value),
                    BinaryOp::Lt => ash_core::contract::ArithConstraint::Lt(value),
                    BinaryOp::Geq => ash_core::contract::ArithConstraint::Gte(value),
                    BinaryOp::Leq => ash_core::contract::ArithConstraint::Lte(value),
                    BinaryOp::Eq => ash_core::contract::ArithConstraint::Eq(value),
                    BinaryOp::Neq => ash_core::contract::ArithConstraint::NotEq(value),
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
                        ash_core::contract::ArithConstraint::Modulo { div, rem },
                    ));
                }
                if let (Some(rem), Some((var, div))) = (int_literal(left), modulo_operand(right)) {
                    return Ok((
                        var.to_string(),
                        ash_core::contract::ArithConstraint::Modulo { div, rem },
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

fn lower_result_constraint(expr: &Expr) -> Option<ash_core::contract::ArithConstraint> {
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
                    BlockStmt::Expr {
                        expr,
                        span: stmt_span,
                    } => {
                        result = CoreExpr::Let {
                            pattern: CorePattern::Variable {
                                name: "_".to_string(),
                                span: ash_core::Span {
                                    start: stmt_span.start,
                                    end: stmt_span.end,
                                },
                            },
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
            DoStmt::Expr {
                value,
                span: stmt_span,
            } => {
                result = CoreExpr::Let {
                    pattern: ash_core::Pattern::Variable {
                        name: "_".to_string(),
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
            DoStmt::Return { .. } => {
                return Err(LoweringError::UnsupportedFeature(
                    "return must be the last statement in a target ambient do block".to_string(),
                ));
            }
        }
    }

    Ok(result)
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

#[cfg(test)]
fn lower_role_obligation_name(name: &str) -> CoreRoleObligationRef {
    CoreRoleObligationRef {
        name: name.to_string(),
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
