//! Ash Type Checker
//!
//! Type system and type inference for the Ash workflow language.
//!
//! This crate provides:
//! - **types**: Core type definitions and unification (TASK-015 to TASK-018)
//! - **constraints**: Constraint generation for expressions (TASK-019)
//! - **solver**: Constraint solving and type error reporting (TASK-020, TASK-025)
//! - **obligations**: Obligation tracking and proof obligations (TASK-023, TASK-024)

pub mod capability_typecheck;
pub mod check_expr;
pub mod check_pattern;
pub mod constraint_checking;
pub mod constraints;
pub mod diagnostic;
pub(crate) mod do_target;
pub mod effective_caps;
pub mod error;
pub mod exhaustiveness;
pub mod instantiate;
pub mod kind;
pub mod name_binding;
pub mod normalizer;
pub mod obligation_checker;
pub mod obligations;
pub mod policy_check;
pub mod purity;
pub mod qualified_name;
pub mod requirements;
pub mod role_checking;
pub mod solver;
pub mod type_env;
pub mod types;
pub mod visibility;

mod surface_type_lowering;

pub(crate) use surface_type_lowering::bind_pattern_variables;

// SMT-based policy conflict detection using Z3
// Provides compile-time verification of policy constraints
pub mod smt;

#[doc(hidden)]
pub use do_target::{SelectedDoEvidence, SelectedDoOperation};

// Re-export smt module under a unified name
pub use smt as policy;

pub use ash_core::ast::{TypeDef, VariantDef};
pub use check_pattern::{
    Bindings, Irrefutability, IrrefutabilityBlockedReason, IrrefutabilityImpossibleReason,
    IrrefutabilityOutcome, IrrefutabilityWitness, check_irrefutable_pattern,
    check_irrefutable_pattern_with_canonical_type, check_irrefutable_pattern_with_canonicalization,
    check_pattern,
};
pub use constraint_checking::*;
pub use constraints::*;
pub use effective_caps::{
    CapabilitySource, CompositionError, EffectiveCapabilitySet, MergedCapability,
};
pub use instantiate::{InstantiateError, InstantiateSubst, instantiate};
pub use kind::Kind;
pub use name_binding::{NameBinder, NameError};
pub use normalizer::*;
pub use obligation_checker::*;
pub use obligations::*;
pub use policy_check::*;
pub use qualified_name::QualifiedName;
pub use requirements::{
    CheckResult, ContractCheckResult, RequirementContext, RequirementError, check_contract,
    check_requirement,
};
pub use solver::{Solver, TypeError};
pub use type_env::{
    AuthorityProvenanceKind, AuthorityProvenanceReport, BindingProvenanceSourceInfo,
    CapabilityBindingInfo, CapabilityBindingProvenanceInfo, ContractIntrinsicKind,
    ContractIntrinsicParameterClass, DEFAULT_PROOF_FUEL, ErasedProof,
    ImplementationAuthoritySourceInfo, PartialConstructorElaborationError,
    PatternCanonicalConstructor, PatternCanonicalType, PatternCanonicalization,
    PatternCanonicalizationBlockedReason, ProofTotalityResult, ProofTotalityStatus,
    ProofTotalityUntestedReason, ProvenanceSourceKind, PublicComputationAlgebra,
    PublicComputationIntrinsicKind, PublicComputationIntrinsicMapping, PublicComputationManifest,
    PublicComputationManifestKind, PublicComputationOperation, PublicComputationOperationAuthority,
    PublicComputationOperationRole, ResourceBindingProvenanceInfo, ResourceTypeInfo,
    StoredFnContract, TypeEnv,
};
pub use types::*;
pub use visibility::{ModulePath, VisibilityChecker, VisibilityError, VisibilityExt};

/// Test-support facade for do-target resolution without exposing the internal
/// hidden dictionary representation.
#[doc(hidden)]
#[allow(clippy::result_large_err)]
pub fn resolve_do_target_for_test(
    env: &TypeEnv,
    target: &ash_parser::surface::DoTarget,
) -> Result<(), error::ConstructorError> {
    do_target::resolve_do_target(env, target).map(|_| ())
}

use surface_type_lowering::{
    bind_surface_type_parameters, synthetic_program_module_identity, workflow_surface_type_to_type,
};

fn variant_field_types(
    env: &TypeEnv,
    expected: &Type,
    variant_name: &str,
) -> Option<Vec<(String, Type)>> {
    #[allow(clippy::collapsible_if)]
    {
        if let Type::Constructor { name, args, .. } = expected {
            if let Ok(crate::type_env::UnfoldedBody::Enum(variants)) =
                env.unfold_constructor(name, args)
            {
                if let Some(variant) = variants
                    .into_iter()
                    .find(|variant| variant.name == variant_name)
                {
                    return Some(variant.fields);
                }
            }
        }
    }

    let (type_name, variant_index) = env.lookup_constructor(variant_name)?;
    match env.lookup_type_info(type_name.as_str())? {
        crate::type_env::TypeInfo::Enum { variants, .. } => variants
            .get(variant_index)
            .map(|variant| variant.fields.clone()),
        crate::type_env::TypeInfo::Struct { .. } => None,
    }
}

/// Type check a program.
pub fn type_check_program(
    program: &ash_parser::surface::Program,
) -> Result<TypeCheckResult, TypeCheckError> {
    let env = TypeEnv::with_builtin_types();
    type_check_program_in_env(&env, program)
}

/// Configuration for program type checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeCheckConfig {
    /// Fuel budget for Stage-3 proof totality traversal.
    pub proof_fuel: usize,
}

impl Default for TypeCheckConfig {
    fn default() -> Self {
        Self {
            proof_fuel: DEFAULT_PROOF_FUEL,
        }
    }
}

/// A zero-parameter function type carrying an inline row annotates the
/// declaration's callable boundary; its result is the declared function
/// result, not a returned closure. Plain closures and builtin signatures keep
/// their surface shape unchanged.
fn declaration_result_type_for_callable(
    return_type: &ash_parser::surface::Type,
) -> &ash_parser::surface::Type {
    match return_type {
        ash_parser::surface::Type::Fn(params, Some(_), result) if params.is_empty() => result,
        other => other,
    }
}

fn fn_signature_from_parts(
    env: &TypeEnv,
    type_params: &[ash_parser::surface::TypeParam],
    params: &[ash_parser::surface::Param],
    return_type: Option<&ash_parser::surface::Type>,
) -> Result<Type, TypeCheckError> {
    let (signature_env, bindings) = bind_surface_type_parameters(env, type_params)?;
    let param_types = params
        .iter()
        .map(|param| workflow_surface_type_to_type(&signature_env, &param.ty, &bindings))
        .collect::<Result<Vec<_>, _>>()?;
    let return_ty = match return_type {
        Some(ty) => workflow_surface_type_to_type(&signature_env, ty, &bindings)?,
        None => Type::Var(TypeVar::fresh()),
    };
    Ok(Type::Fn(param_types, Box::new(return_ty)))
}

fn reject_runtime_prop_return(
    signature: &Type,
    callable_description: &str,
    name: &str,
) -> Result<(), TypeCheckError> {
    let Type::Fn(_, return_ty) = signature else {
        unreachable!("callable signatures are function types");
    };
    if return_ty.contains_prop_kind() {
        return Err(TypeCheckError::TypeError(format!(
            "Prop-typed values cannot escape into runtime {callable_description} return '{name} -> {return_ty}'"
        )));
    }
    Ok(())
}

/// Compute the type signature of an ordinary `fn` definition.
pub fn fn_signature_type(
    env: &TypeEnv,
    function: &ash_parser::surface::FnDef,
) -> Result<Type, TypeCheckError> {
    let signature = fn_signature_from_parts(
        env,
        &function.type_params,
        &function.params,
        function
            .return_type
            .as_ref()
            .map(declaration_result_type_for_callable),
    )?;
    reject_runtime_prop_return(&signature, "function", function.name.as_ref())?;
    Ok(signature)
}

/// Compute the type signature of a builtin `fn` definition.
pub fn builtin_fn_signature_type(
    env: &TypeEnv,
    function: &ash_parser::surface::BuiltinFnDef,
) -> Result<Type, TypeCheckError> {
    let signature = fn_signature_from_parts(
        env,
        &function.type_params,
        &function.params,
        Some(&function.return_type),
    )?;
    reject_runtime_prop_return(&signature, "builtin function", function.name.as_ref())?;
    Ok(signature)
}

fn row_item_span(item: &ash_parser::surface::ComputationRowItem) -> ash_parser::token::Span {
    use ash_parser::surface::ComputationRowItem;
    match item {
        ComputationRowItem::Operation { span, .. }
        | ComputationRowItem::WholeRow { span, .. }
        | ComputationRowItem::Resource { span, .. }
        | ComputationRowItem::Role { span, .. }
        | ComputationRowItem::Policy { span, .. }
        | ComputationRowItem::Channel { span, .. }
        | ComputationRowItem::Process { span, .. }
        | ComputationRowItem::Fail { span, .. }
        | ComputationRowItem::Evidence { span, .. }
        | ComputationRowItem::Group { span, .. }
        | ComputationRowItem::Tail { span, .. } => *span,
    }
}

fn row_item_text(item: &ash_parser::surface::ComputationRowItem) -> String {
    use ash_parser::surface::ComputationRowItem;
    let path_text = |path: &[ash_parser::surface::Name]| {
        path.iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>()
            .join("::")
    };
    match item {
        ComputationRowItem::Operation {
            path, separator, ..
        } => {
            let Some((last, prefix)) = path.split_last() else {
                return String::new();
            };
            if prefix.is_empty() {
                return last.to_string();
            }
            let separator =
                match separator.unwrap_or(ash_parser::surface::RowPathSeparator::DoubleColon) {
                    ash_parser::surface::RowPathSeparator::Dot => ".",
                    ash_parser::surface::RowPathSeparator::DoubleColon => "::",
                };
            format!("{}{separator}{last}", path_text(prefix))
        }
        ComputationRowItem::WholeRow { variable, .. } => variable.to_string(),
        ComputationRowItem::Resource { path, mode, .. } => mode.as_ref().map_or_else(
            || format!("resource {}", path_text(path)),
            |mode| format!("resource {} {mode}", path_text(path)),
        ),
        ComputationRowItem::Role { path, .. } => format!("role {}", path_text(path)),
        ComputationRowItem::Policy { path, .. } => format!("policy {}", path_text(path)),
        ComputationRowItem::Channel { path, mode, .. } => mode.as_ref().map_or_else(
            || format!("channel {}", path_text(path)),
            |mode| format!("channel {mode} {}", path_text(path)),
        ),
        ComputationRowItem::Process {
            keyword, operation, ..
        } => operation.as_ref().map_or_else(
            || keyword.to_string(),
            |operation| format!("{keyword} {operation}"),
        ),
        ComputationRowItem::Fail { path, .. } => path.as_ref().map_or_else(
            || "fail".to_string(),
            |path| format!("fail {}", path_text(path)),
        ),
        ComputationRowItem::Evidence { path, .. } => format!("evidence {}", path_text(path)),
        ComputationRowItem::Group { path, .. } => format!("group {}", path_text(path)),
        ComputationRowItem::Tail { variable, .. } => format!("| {variable}"),
    }
}

fn unsupported_predicate_like_row_family(
    item: &ash_parser::surface::ComputationRowItem,
) -> Option<&'static str> {
    use ash_parser::surface::ComputationRowItem;
    let first = match item {
        ComputationRowItem::Operation { path, .. } => path.first()?,
        ComputationRowItem::WholeRow { variable, .. } => variable,
        _ => return None,
    };
    [
        "requires",
        "ensures",
        "invariant",
        "law",
        "proof",
        "contract",
    ]
    .into_iter()
    .find(|family| first.as_ref() == *family || first.as_ref().starts_with(&format!("{family}_")))
}

fn validate_operation_row_identity(
    env: &TypeEnv,
    item: &ash_parser::surface::ComputationRowItem,
) -> Result<(), TypeCheckError> {
    let ash_parser::surface::ComputationRowItem::Operation {
        path,
        separator,
        span,
    } = item
    else {
        return Ok(());
    };
    if *separator != Some(ash_parser::surface::RowPathSeparator::DoubleColon) {
        return Ok(());
    }
    let [target, method] = path.as_slice() else {
        return Ok(());
    };
    if !target
        .as_ref()
        .chars()
        .next()
        .is_some_and(char::is_uppercase)
    {
        return Ok(());
    }
    match env.resolve_operation_row_identity(target.as_ref(), method.as_ref()) {
        crate::type_env::OperationRowIdentityResolution::ConcreteImpl { .. }
        | crate::type_env::OperationRowIdentityResolution::AbstractImpl { .. } => Ok(()),
        crate::type_env::OperationRowIdentityResolution::InterfaceQualified {
            suggestion, ..
        } => Err(
            crate::error::TypeEnvError::InterfaceQualifiedOperationRowIdentity {
                item: row_item_text(item),
                suggestion,
                span: *span,
            }
            .into(),
        ),
        crate::type_env::OperationRowIdentityResolution::UnknownImplType { impl_type } => {
            Err(crate::error::TypeEnvError::UnknownOperationRowImplType {
                impl_type,
                item: row_item_text(item),
                span: *span,
            }
            .into())
        }
        crate::type_env::OperationRowIdentityResolution::UnknownMethod { candidates, .. } => {
            Err(crate::error::TypeEnvError::UnknownOperationRowMethod {
                item: row_item_text(item),
                candidates: candidates.join(", "),
                span: *span,
            }
            .into())
        }
    }
}

fn validate_computation_row(
    env: &TypeEnv,
    row: &ash_parser::surface::ComputationRow,
) -> Result<(), TypeCheckError> {
    let mut tail_seen = None;
    for (index, item) in row.items.iter().enumerate() {
        if let Some(family) = unsupported_predicate_like_row_family(item) {
            return Err(crate::error::TypeEnvError::UnsupportedRowItemFamily {
                family: family.to_string(),
                item: row_item_text(item),
                span: row_item_span(item),
            }
            .into());
        }
        validate_operation_row_identity(env, item)?;
        if let ash_parser::surface::ComputationRowItem::Tail { variable, span } = item {
            if tail_seen.is_some() {
                return Err(crate::error::TypeEnvError::DuplicateRowTail {
                    tail: variable.to_string(),
                    span: *span,
                }
                .into());
            }
            tail_seen = Some((variable, *span, index));
        }
    }
    if let Some((variable, span, index)) = tail_seen
        && index + 1 != row.items.len()
    {
        return Err(crate::error::TypeEnvError::RowTailNotFinal {
            tail: variable.to_string(),
            span,
        }
        .into());
    }
    Ok(())
}

fn validate_surface_type_rows(
    env: &TypeEnv,
    ty: &ash_parser::surface::Type,
) -> Result<(), TypeCheckError> {
    use ash_parser::surface::Type as SurfaceType;
    match ty {
        SurfaceType::List(item) | SurfaceType::Associated { base: item, .. } => {
            validate_surface_type_rows(env, item)
        }
        SurfaceType::Tuple(items) => items
            .iter()
            .try_for_each(|item| validate_surface_type_rows(env, item)),
        SurfaceType::Record(fields) => fields
            .iter()
            .try_for_each(|(_, item)| validate_surface_type_rows(env, item)),
        SurfaceType::Constructor { args, .. }
        | SurfaceType::AssociatedFamilyProjection { args, .. } => args
            .iter()
            .try_for_each(|item| validate_surface_type_rows(env, item)),
        SurfaceType::Fn(params, row, ret) => {
            params
                .iter()
                .try_for_each(|param| validate_surface_type_rows(env, param))?;
            if let Some(row) = row {
                validate_computation_row(env, row)?;
            }
            validate_surface_type_rows(env, ret)
        }
        SurfaceType::Name(_) | SurfaceType::Hole { .. } | SurfaceType::Capability(_) => Ok(()),
    }
}

fn validate_callable_rows(
    env: &TypeEnv,
    name: &str,
    params: &[ash_parser::surface::Param],
    return_type: Option<&ash_parser::surface::Type>,
    proposition_tail: Option<&ash_parser::surface::PropositionTail>,
) -> Result<(), TypeCheckError> {
    params
        .iter()
        .try_for_each(|param| validate_surface_type_rows(env, &param.ty))?;
    if let Some(return_type) = return_type {
        validate_surface_type_rows(env, return_type)?;
    }
    if let Some(row) = proposition_tail.and_then(|tail| tail.row.as_ref()) {
        if let Some(ash_parser::surface::Type::Fn(params, Some(inline_row), _)) = return_type
            && params.is_empty()
        {
            return Err(crate::error::TypeEnvError::DuplicateCallableRow {
                callable: name.to_string(),
                inline_span: inline_row.span,
                expanded_span: row.span,
                span: row.span,
            }
            .into());
        }
        validate_computation_row(env, &row.row)?;
    }
    Ok(())
}

fn register_function_contract(
    env: &mut TypeEnv,
    function: &ash_parser::surface::FnDef,
) -> Result<(), TypeCheckError> {
    let params = function
        .params
        .iter()
        .map(|param| {
            (
                param.name.to_string(),
                ash_parser::surface_type_to_core_type(&param.ty),
            )
        })
        .collect::<Vec<_>>();
    let result = function
        .return_type
        .as_ref()
        .map(declaration_result_type_for_callable)
        .map(ash_parser::surface_type_to_core_type);
    let context = ash_parser::FnContractLoweringContext {
        name: function.name.as_ref(),
        params: &params,
        result,
    };
    let lowered = ash_parser::lower_fn_contract(function.contract.as_ref(), &context)
        .map_err(|error| TypeCheckError::TypeError(error.to_string()))?;
    env.bind_fn_contract(
        function.name.as_ref(),
        StoredFnContract {
            param_names: function
                .params
                .iter()
                .map(|param| param.name.to_string())
                .collect(),
            contract: lowered.contract,
            runtime_postconditions: lowered.runtime_postconditions,
        },
    );
    Ok(())
}

fn integer_fact(
    facts: &std::collections::HashMap<String, i64>,
    expr: &ash_parser::surface::Expr,
) -> Option<i64> {
    match expr {
        ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Int(value)) => {
            Some(*value)
        }
        ash_parser::surface::Expr::Unary {
            op: ash_parser::surface::UnaryOp::Neg,
            operand,
            ..
        } => integer_fact(facts, operand).map(|value| -value),
        ash_parser::surface::Expr::Variable { name, .. } => facts.get(name.as_ref()).copied(),
        ash_parser::surface::Expr::Binary {
            op, left, right, ..
        } => {
            let left = integer_fact(facts, left)?;
            let right = integer_fact(facts, right)?;
            match op {
                ash_parser::surface::BinaryOp::Add => Some(left + right),
                ash_parser::surface::BinaryOp::Sub => Some(left - right),
                ash_parser::surface::BinaryOp::Mul => Some(left * right),
                ash_parser::surface::BinaryOp::Div => (right != 0).then_some(left / right),
                ash_parser::surface::BinaryOp::Mod => (right != 0).then_some(left % right),
                _ => None,
            }
        }
        _ => None,
    }
}

fn branch_assumption(
    condition: &ash_parser::surface::Expr,
) -> Option<(String, ash_core::contract::ArithConstraint)> {
    use ash_core::contract::ArithConstraint;
    use ash_parser::surface::{BinaryOp, Expr, Literal};
    let Expr::Binary {
        op, left, right, ..
    } = condition
    else {
        return None;
    };
    let (Expr::Variable { name, .. }, Expr::Literal(Literal::Int(value))) = (&**left, &**right)
    else {
        return None;
    };
    let constraint = match op {
        BinaryOp::Gt => ArithConstraint::Gt(*value),
        BinaryOp::Geq => ArithConstraint::Gte(*value),
        BinaryOp::Lt => ArithConstraint::Lt(*value),
        BinaryOp::Leq => ArithConstraint::Lte(*value),
        BinaryOp::Eq => ArithConstraint::Eq(*value),
        BinaryOp::Neq => ArithConstraint::NotEq(*value),
        _ => return None,
    };
    Some((name.to_string(), constraint))
}

fn validate_function_preconditions(
    env: &TypeEnv,
    expr: &ash_parser::surface::Expr,
    facts: &mut std::collections::HashMap<String, i64>,
    assumptions: &mut std::collections::HashMap<String, Vec<ash_core::contract::ArithConstraint>>,
) -> Result<(), TypeCheckError> {
    use ash_parser::surface::Expr;
    match expr {
        Expr::Call {
            func, module, args, ..
        } => {
            for arg in args {
                validate_function_preconditions(env, arg, facts, assumptions)?;
            }
            let name = module
                .as_ref()
                .map(|module| format!("{module}::{func}"))
                .unwrap_or_else(|| func.to_string());
            if let Some(boundary) = env.lookup_fn_contract(&name) {
                let mut context = RequirementContext::new();
                for (parameter, arg) in boundary.param_names.iter().zip(args) {
                    let value = integer_fact(facts, arg);
                    if let Some(value) = value {
                        context = context.with_fact(parameter.clone(), value);
                    }
                    if let Expr::Variable { name, .. } = arg {
                        for assumption in assumptions.get(name.as_ref()).into_iter().flatten() {
                            context = context
                                .with_arithmetic_assumption(parameter.clone(), assumption.clone());
                        }
                    }
                }
                let result = check_contract(&boundary.contract, &context);
                if !result.is_success() {
                    let details = result
                        .errors()
                        .into_iter()
                        .map(|error| error.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(TypeCheckError::TypeError(format!(
                        "fn precondition may not hold for call '{name}': {details}"
                    )));
                }
            }
            Ok(())
        }
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            let mut scoped_facts = facts.clone();
            for statement in statements {
                if let ash_parser::surface::BlockStmt::Let { pattern, expr, .. } = statement {
                    validate_function_preconditions(env, expr, &mut scoped_facts, assumptions)?;
                    if let (ash_parser::surface::Pattern::Variable { name, .. }, Some(value)) =
                        (pattern, integer_fact(&scoped_facts, expr))
                    {
                        scoped_facts.insert(name.to_string(), value);
                    }
                }
            }
            if let Some(tail) = tail_expr {
                validate_function_preconditions(env, tail, &mut scoped_facts, assumptions)?;
            }
            Ok(())
        }
        Expr::DoBlock { stmts, .. } => {
            let mut scoped_facts = facts.clone();
            for statement in stmts {
                match statement {
                    ash_parser::surface::DoStmt::Let { name, value, .. }
                    | ash_parser::surface::DoStmt::Bind { name, value, .. } => {
                        validate_function_preconditions(
                            env,
                            value,
                            &mut scoped_facts,
                            assumptions,
                        )?;
                        if let Some(value) = integer_fact(&scoped_facts, value) {
                            scoped_facts.insert(name.to_string(), value);
                        }
                    }
                    ash_parser::surface::DoStmt::Expr { value, .. }
                    | ash_parser::surface::DoStmt::Return { value, .. } => {
                        validate_function_preconditions(
                            env,
                            value,
                            &mut scoped_facts,
                            assumptions,
                        )?;
                    }
                }
            }
            Ok(())
        }
        Expr::Binary { left, right, .. } => {
            validate_function_preconditions(env, left, facts, assumptions)?;
            validate_function_preconditions(env, right, facts, assumptions)
        }
        Expr::Unary { operand, .. } | Expr::FieldAccess { base: operand, .. } => {
            validate_function_preconditions(env, operand, facts, assumptions)
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            validate_function_preconditions(env, condition, facts, assumptions)?;
            let mut then_assumptions = assumptions.clone();
            if let Some((name, constraint)) = branch_assumption(condition) {
                then_assumptions.entry(name).or_default().push(constraint);
            }
            validate_function_preconditions(
                env,
                then_branch,
                &mut facts.clone(),
                &mut then_assumptions,
            )?;
            if let Some(else_branch) = else_branch {
                validate_function_preconditions(
                    env,
                    else_branch,
                    &mut facts.clone(),
                    &mut assumptions.clone(),
                )?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn register_public_function_proposition_tail(
    env: &mut TypeEnv,
    tail: &ash_parser::surface::PropositionTail,
    item_name: &str,
    item_kind: &str,
    site_id: u64,
) -> Result<(), TypeCheckError> {
    let obligation_start = env.proposition_obligations().len();
    env.add_proposition_obligations_from_tail(
        tail,
        ash_core::semantic_summary::SourceOrigin::Synthetic {
            reason: format!("{item_kind} proposition checking point {item_name}"),
        },
        crate::type_env::PropositionCheckingSite::new(
            site_id,
            crate::type_env::PropositionCheckingSiteKind::ExplicitRequirement,
            Some(format!("{item_kind} {item_name} proposition tail")),
        ),
    )
    .map_err(|error| {
        TypeCheckError::TypeError(format!("proposition tail lowering failed: {error}"))
    })?;
    env.discharge_required_proposition_obligations_since(obligation_start)
        .map(|_| ())
        .map_err(TypeCheckError::from)
}

fn register_function_signatures(
    env: &mut TypeEnv,
    definitions: &[ash_parser::surface::Definition],
) -> Result<(), TypeCheckError> {
    let mut staged = env.clone();
    for (index, definition) in definitions.iter().enumerate() {
        match definition {
            ash_parser::surface::Definition::Function(function) => {
                validate_callable_rows(
                    &staged,
                    function.name.as_ref(),
                    &function.params,
                    function.return_type.as_ref(),
                    function.proposition_tail.as_ref(),
                )?;
                let signature = fn_signature_type(&staged, function)?;
                staged.bind_variable(function.name.as_ref(), signature);
                if matches!(function.visibility, ash_parser::surface::Visibility::Public)
                    && let Some(tail) = &function.proposition_tail
                {
                    register_public_function_proposition_tail(
                        &mut staged,
                        tail,
                        function.name.as_ref(),
                        "function",
                        0x8801_0000u64 + index as u64,
                    )?;
                }
            }
            ash_parser::surface::Definition::BuiltinFn(function) => {
                validate_callable_rows(
                    &staged,
                    function.name.as_ref(),
                    &function.params,
                    Some(&function.return_type),
                    function.proposition_tail.as_ref(),
                )?;
                let signature = builtin_fn_signature_type(&staged, function)?;
                staged.bind_variable(function.name.as_ref(), signature);
                if matches!(function.visibility, ash_parser::surface::Visibility::Public)
                    && let Some(tail) = &function.proposition_tail
                {
                    register_public_function_proposition_tail(
                        &mut staged,
                        tail,
                        function.name.as_ref(),
                        "builtin function",
                        0x8802_0000u64 + index as u64,
                    )?;
                }
            }
            ash_parser::surface::Definition::Capability(capability) => {
                staged.register_capability_symbol(capability.name.as_ref());
            }
            _ => {}
        }
    }
    for definition in definitions {
        if let ash_parser::surface::Definition::Function(function) = definition {
            register_function_contract(&mut staged, function)?;
        }
    }
    *env = staged;
    Ok(())
}

fn check_function_body_in_env(
    env: &TypeEnv,
    function: &ash_parser::surface::FnDef,
) -> Result<Type, TypeCheckError> {
    let (mut fn_env, bindings) = bind_surface_type_parameters(env, &function.type_params)?;
    let mut param_types = Vec::with_capacity(function.params.len());
    for param in &function.params {
        let param_ty = workflow_surface_type_to_type(&fn_env, &param.ty, &bindings)?;
        fn_env.bind_variable(param.name.as_ref(), param_ty.clone());
        param_types.push(param_ty);
    }

    validate_function_preconditions(
        &fn_env,
        &function.body,
        &mut std::collections::HashMap::new(),
        &mut std::collections::HashMap::new(),
    )?;

    let result = crate::check_expr::check_expr(&fn_env, &function.body);
    if !result.is_ok() {
        let reason = result
            .errors
            .into_iter()
            .next()
            .map(|error| error.to_string())
            .unwrap_or_else(|| format!("failed to typecheck fn '{}'", function.name));
        return Err(TypeCheckError::TypeError(reason));
    }

    let body_ty = result.substitution.apply(&result.ty);
    if let Some(return_type) = &function.return_type {
        let expected = workflow_surface_type_to_type(
            &fn_env,
            declaration_result_type_for_callable(return_type),
            &bindings,
        )?;
        crate::types::unify(&expected, &body_ty).map_err(|_| {
            TypeCheckError::TypeError(format!(
                "fn '{}' declared return type {} but body returns {}",
                function.name, expected, body_ty
            ))
        })?;
    } else if crate::types::type_contains_fun(&body_ty) && param_types.is_empty() {
        return Err(TypeCheckError::TypeError(format!(
            "fn '{}' omitted return type could not be inferred; add an explicit return type",
            function.name
        )));
    }

    Ok(body_ty)
}

fn refine_function_signatures(
    env: &mut TypeEnv,
    definitions: &[ash_parser::surface::Definition],
) -> Result<(), TypeCheckError> {
    for definition in definitions {
        let ash_parser::surface::Definition::Function(function) = definition else {
            continue;
        };
        let body_ty = check_function_body_in_env(env, function)?;
        if function.return_type.is_none() {
            let (signature_env, bindings) =
                bind_surface_type_parameters(env, &function.type_params)?;
            let param_types = function
                .params
                .iter()
                .map(|param| workflow_surface_type_to_type(&signature_env, &param.ty, &bindings))
                .collect::<Result<Vec<_>, _>>()?;
            env.bind_variable(
                function.name.as_ref(),
                Type::Fn(param_types, Box::new(body_ty)),
            );
        }
    }
    Ok(())
}

fn type_check_program_entry(
    env: &TypeEnv,
    program: &ash_parser::surface::Program,
) -> Result<TypeCheckResult, TypeCheckError> {
    let entry_name = program.entry.function.as_ref();
    let entry_function = program
        .definitions
        .iter()
        .find_map(|definition| match definition {
            ash_parser::surface::Definition::Function(function)
                if function.name.as_ref() == entry_name =>
            {
                Some(function)
            }
            _ => None,
        })
        .ok_or_else(|| {
            TypeCheckError::ResolutionError(format!(
                "program entry function '{}' is not defined",
                entry_name
            ))
        })?;

    let entry_type = check_function_body_in_env(env, entry_function)?;
    let mut inferred_types = std::collections::HashMap::new();
    inferred_types.insert(entry_name.to_string(), entry_type);

    Ok(TypeCheckResult {
        substitution: Substitution::new(),
        errors: Vec::new(),
        inferred_types,
        effect: ash_core::Effect::Epistemic,
        obligation_status: crate::obligations::ObligationCheckResult::Success,
        function_contracts: env.function_contracts(),
        authority_provenance: AuthorityProvenanceReport::default(),
    })
}

/// Type check a program with explicit type-checking configuration.
pub fn type_check_program_with_config(
    program: &ash_parser::surface::Program,
    config: &TypeCheckConfig,
) -> Result<TypeCheckResult, TypeCheckError> {
    let env = TypeEnv::with_builtin_types();
    type_check_program_in_env_with_config(&env, program, config)
}

/// Type check a program with a pre-populated type environment.
/// Used when imported callable signatures need to be available during checking.
pub fn type_check_program_in_env(
    initial_env: &TypeEnv,
    program: &ash_parser::surface::Program,
) -> Result<TypeCheckResult, TypeCheckError> {
    type_check_program_in_env_with_config(initial_env, program, &TypeCheckConfig::default())
}

/// Type check a program with a pre-populated type environment and explicit config.
pub fn type_check_program_in_env_with_config(
    initial_env: &TypeEnv,
    program: &ash_parser::surface::Program,
    config: &TypeCheckConfig,
) -> Result<TypeCheckResult, TypeCheckError> {
    type_check_program_in_env_for_module_with_config(
        initial_env,
        program,
        synthetic_program_module_identity(),
        config,
    )
}

/// Type check a program with an explicit current-module identity for local declarations.
///
/// Module-aware callers should use this entry point so sealed associated-family
/// declarations and impl-family schemes record the real defining module instead
/// of the standalone synthetic program identity.
pub fn type_check_program_in_env_for_module(
    initial_env: &TypeEnv,
    program: &ash_parser::surface::Program,
    module_identity: ash_core::semantic_summary::ModuleIdentity,
) -> Result<TypeCheckResult, TypeCheckError> {
    type_check_program_in_env_for_module_with_config(
        initial_env,
        program,
        module_identity,
        &TypeCheckConfig::default(),
    )
}

/// Type check a program with an explicit current-module identity and config.
pub fn type_check_program_in_env_for_module_with_config(
    initial_env: &TypeEnv,
    program: &ash_parser::surface::Program,
    module_identity: ash_core::semantic_summary::ModuleIdentity,
    config: &TypeCheckConfig,
) -> Result<TypeCheckResult, TypeCheckError> {
    let mut env = initial_env.clone();
    env.set_current_module_identity(module_identity);

    for definition in &program.definitions {
        if let ash_parser::surface::Definition::Interface(interface) = definition {
            env.register_interface(interface)
                .map_err(TypeCheckError::from)?;
        }
    }

    for definition in &program.definitions {
        if let ash_parser::surface::Definition::ResourceType(resource_type) = definition {
            env.register_resource_type(resource_type)
                .map_err(TypeCheckError::from)?;
        }
    }

    for definition in &program.definitions {
        if let ash_parser::surface::Definition::Impl(implementation) = definition {
            env.register_impl(implementation)
                .map_err(TypeCheckError::from)?;
        }
    }

    register_function_signatures(&mut env, &program.definitions)?;
    refine_function_signatures(&mut env, &program.definitions)?;

    for definition in &program.definitions {
        if let ash_parser::surface::Definition::Interface(interface) = definition {
            env.register_interface_laws(interface)
                .map_err(TypeCheckError::from)?;
        }
    }
    env.register_module_laws(&program.definitions)
        .map_err(TypeCheckError::from)?;
    for definition in &program.definitions {
        if let ash_parser::surface::Definition::Impl(implementation) = definition {
            env.register_impl_proofs_with_fuel(implementation, config.proof_fuel)
                .map_err(TypeCheckError::from)?;
        }
    }
    env.register_module_proofs_with_fuel(&program.definitions, config.proof_fuel)
        .map_err(TypeCheckError::from)?;

    type_check_program_entry(&env, program)
}

/// Error during type checking
#[derive(Debug, Clone, thiserror::Error)]
pub enum TypeCheckError {
    /// Name resolution failed
    #[error("Name resolution error: {0}")]
    ResolutionError(String),
    /// Type error
    #[error("Type error: {0}")]
    TypeError(String),
    /// Effect constraint violation
    #[error("Effect error: {0}")]
    EffectError(String),
    /// Obligation not satisfied
    #[error("Obligation error: {0}")]
    ObligationError(String),
    /// Type-environment registration error.
    #[error("Type environment error: {0}")]
    TypeEnv(Box<crate::error::TypeEnvError>),
}

impl From<crate::error::TypeEnvError> for TypeCheckError {
    fn from(err: crate::error::TypeEnvError) -> Self {
        Self::TypeEnv(Box::new(err))
    }
}

impl From<Box<crate::error::TypeEnvError>> for TypeCheckError {
    fn from(err: Box<crate::error::TypeEnvError>) -> Self {
        Self::TypeEnv(err)
    }
}

/// Extended type check result with effect and obligation info
#[derive(Debug, Clone)]
pub struct TypeCheckResult {
    /// Final substitution
    pub substitution: Substitution,
    /// Any errors encountered
    pub errors: Vec<TypeError>,
    /// Inferred types for expressions
    pub inferred_types: std::collections::HashMap<String, Type>,
    /// Inferred effect of the workflow
    pub effect: ash_core::Effect,
    /// Obligation check status
    pub obligation_status: ObligationCheckResult,
    /// Lowered pure-function contract boundaries available to runtime consumers.
    pub function_contracts: std::collections::HashMap<String, StoredFnContract>,
    /// Static authority provenance metadata available to runtime admission consumers.
    pub authority_provenance: AuthorityProvenanceReport,
}

impl TypeCheckResult {
    /// Check if type checking succeeded
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty() && self.obligation_status.is_success()
    }

    /// Get the final type after applying substitution
    pub fn final_type(&self, ty: &Type) -> Type {
        self.substitution.apply(ty)
    }
}

impl std::fmt::Display for TypeCheckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_ok() {
            write!(f, "Type check succeeded with effect {:?}", self.effect)
        } else {
            writeln!(f, "Type check failed:")?;
            if !self.errors.is_empty() {
                writeln!(f, "  Type errors: {}", self.errors.len())?;
            }
            if !self.obligation_status.is_success() {
                writeln!(f, "  Obligation status: {:?}", self.obligation_status)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_type_check_error_display() {
        let err = TypeCheckError::ResolutionError("test".to_string());
        assert!(format!("{err}").contains("test"));

        let err = TypeCheckError::TypeError("type mismatch".to_string());
        assert!(format!("{err}").contains("type mismatch"));

        let err = TypeCheckError::EffectError("effect violation".to_string());
        assert!(format!("{err}").contains("effect violation"));

        let err = TypeCheckError::ObligationError("obligation failed".to_string());
        assert!(format!("{err}").contains("obligation failed"));
    }

    #[test]
    fn test_module_exports() {
        // Test that all modules are accessible via crate root
        let _ = ConstraintContext::new();
        let _ = TypeEnv::with_builtin_types();
        let _ = Type::Int;
    }

    /// SPEC-072 / TASK-959: pure closure syntax remains `Fn`, so a target
    /// function may return a closure when its declared return type is a matching
    /// pure callable type.
    #[test]
    fn task959_fn_return_pure_closure_is_accepted() {
        use ash_parser::surface::{
            Definition, Expr, FnDef, Program, ProgramEntry, Type as SurfaceType, Visibility,
        };
        use ash_parser::token::Span;

        fn test_span() -> Span {
            Span::new(0, 0, 1, 1)
        }

        // The declared return type is `(Int) -> Int` (a pure function type).
        // TASK-959 keeps pure closure syntax at the Pure stratum.
        let program = Program {
            definitions: vec![Definition::Function(FnDef {
                visibility: Visibility::Inherited,
                name: "main".into(),
                type_params: vec![],
                params: vec![],
                return_type: Some(SurfaceType::Fn(
                    vec![SurfaceType::Name("Int".into())],
                    None,
                    Box::new(SurfaceType::Name("Int".into())),
                )),
                proposition_tail: None,
                contract: None,
                body: Expr::FnDef {
                    params: vec![("x".into(), Some("Int".into()))],
                    return_type: None,
                    body: Box::new(ash_parser::surface::Expr::Variable {
                        name: "x".into(),
                        span: ash_parser::token::Span::default(),
                    }),
                    span: test_span(),
                },
                span: test_span(),
            })],
            entry: ProgramEntry {
                function: "main".into(),
                span: test_span(),
            },
        };

        let result = type_check_program(&program);
        assert!(
            result.is_ok(),
            "fn returning a matching pure closure should typecheck, got {result:?}"
        );
    }
}
