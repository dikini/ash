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

use surface_type_lowering::{synthetic_program_module_identity, workflow_surface_type_to_type};

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

fn lower_surface_type_for_typecheck(
    env: &TypeEnv,
    ty: &ash_parser::surface::Type,
) -> Result<Type, TypeCheckError> {
    workflow_surface_type_to_type(env, ty, &std::collections::HashMap::new())
        .map_err(|error| TypeCheckError::TypeError(error.to_string()))
}

fn fn_signature_from_parts(
    env: &TypeEnv,
    params: &[ash_parser::surface::Param],
    return_type: Option<&ash_parser::surface::Type>,
) -> Result<Type, TypeCheckError> {
    let mut param_types = Vec::with_capacity(params.len());
    for param in params {
        param_types.push(lower_surface_type_for_typecheck(env, &param.ty)?);
    }
    let return_ty = match return_type {
        Some(ty) => lower_surface_type_for_typecheck(env, ty)?,
        None => Type::Var(TypeVar::fresh()),
    };
    Ok(Type::Fn(param_types, Box::new(return_ty)))
}

/// Compute the type signature of an ordinary `fn` definition.
pub fn fn_signature_type(
    env: &TypeEnv,
    function: &ash_parser::surface::FnDef,
) -> Result<Type, TypeCheckError> {
    fn_signature_from_parts(env, &function.params, function.return_type.as_ref())
}

/// Compute the type signature of a builtin `fn` definition.
pub fn builtin_fn_signature_type(
    env: &TypeEnv,
    function: &ash_parser::surface::BuiltinFnDef,
) -> Result<Type, TypeCheckError> {
    fn_signature_from_parts(env, &function.params, Some(&function.return_type))
}

fn register_function_signatures(
    env: &mut TypeEnv,
    definitions: &[ash_parser::surface::Definition],
) -> Result<(), TypeCheckError> {
    for definition in definitions {
        match definition {
            ash_parser::surface::Definition::Function(function) => {
                let signature =
                    fn_signature_from_parts(env, &function.params, function.return_type.as_ref())?;
                env.bind_variable(function.name.as_ref(), signature);
            }
            ash_parser::surface::Definition::BuiltinFn(function) => {
                let signature =
                    fn_signature_from_parts(env, &function.params, Some(&function.return_type))?;
                env.bind_variable(function.name.as_ref(), signature);
            }
            ash_parser::surface::Definition::Capability(capability) => {
                env.register_capability_symbol(capability.name.as_ref());
            }
            _ => {}
        }
    }
    Ok(())
}

fn check_function_body_in_env(
    env: &TypeEnv,
    function: &ash_parser::surface::FnDef,
) -> Result<Type, TypeCheckError> {
    let mut fn_env = env.clone();
    let mut param_types = Vec::with_capacity(function.params.len());
    for param in &function.params {
        let param_ty = lower_surface_type_for_typecheck(env, &param.ty)?;
        fn_env.bind_variable(param.name.as_ref(), param_ty.clone());
        param_types.push(param_ty);
    }

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
        let expected = lower_surface_type_for_typecheck(env, return_type)?;
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
            let param_types = function
                .params
                .iter()
                .map(|param| lower_surface_type_for_typecheck(env, &param.ty))
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
