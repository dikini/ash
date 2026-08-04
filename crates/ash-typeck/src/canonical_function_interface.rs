//! Checked closed-function projections from canonical parser units.
//!
//! This narrow Type-layer pass accepts only self-contained leaf units with
//! ordinary functions and explicit primitive signatures. It stages sibling
//! signatures before checking bodies, then publishes its result only when the
//! complete graph succeeds.

use std::collections::BTreeMap;

use ash_core::module_graph::{ModuleArtifact, ModuleArtifactOrigin, ModuleKey};
use ash_parser::surface::{Definition, FnDef, Type as SurfaceType, Visibility};
use ash_parser::{CanonicalModuleGraph, Span, Spanned};

use crate::{Type, TypeCheckError, TypeEnv, check_function_body_in_env, fn_signature_type};

/// A Type-layer identity freshly derived from a checked function declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalCheckedFunctionIdentity {
    module_key: ModuleKey,
    name: Box<str>,
}

impl CanonicalCheckedFunctionIdentity {
    /// Returns the canonical module key that defines the checked function.
    #[must_use]
    pub fn module_key(&self) -> &ModuleKey {
        &self.module_key
    }

    /// Returns the parsed defining name before any later binding operation.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// One checked ordinary function in the closed primitive leaf domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalCheckedFunction {
    defining_identity: CanonicalCheckedFunctionIdentity,
    declaration_span: Span,
    body_span: Span,
    origin: ModuleArtifactOrigin,
    visibility: Visibility,
    signature: Type,
    body_type: Type,
}

impl CanonicalCheckedFunction {
    #[allow(
        clippy::too_many_arguments,
        reason = "this crate-private constructor preserves the checked declaration's eight independent facts"
    )]
    pub(crate) fn from_checked_parts(
        module_key: ModuleKey,
        name: Box<str>,
        declaration_span: Span,
        body_span: Span,
        origin: ModuleArtifactOrigin,
        visibility: Visibility,
        signature: Type,
        body_type: Type,
    ) -> Self {
        Self {
            defining_identity: CanonicalCheckedFunctionIdentity { module_key, name },
            declaration_span,
            body_span,
            origin,
            visibility,
            signature,
            body_type,
        }
    }

    /// Returns the Type-layer defining identity of this function.
    #[must_use]
    pub fn defining_identity(&self) -> &CanonicalCheckedFunctionIdentity {
        &self.defining_identity
    }

    /// Returns the parser anchor of the complete declaration.
    #[must_use]
    pub const fn declaration_span(&self) -> Span {
        self.declaration_span
    }

    /// Returns the parser anchor of the checked function body.
    #[must_use]
    pub const fn body_span(&self) -> Span {
        self.body_span
    }

    /// Returns parser acquisition provenance for the defining unit.
    #[must_use]
    pub fn origin(&self) -> &ModuleArtifactOrigin {
        &self.origin
    }

    /// Returns the declaration visibility retained by the checked result.
    #[must_use]
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }

    /// Returns the staged explicit function signature.
    #[must_use]
    pub fn signature(&self) -> &Type {
        &self.signature
    }

    /// Returns the inferred and declared-compatible body type.
    #[must_use]
    pub fn body_type(&self) -> &Type {
        &self.body_type
    }
}

/// The exported subset of one checked closed-function module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalPublicFunctionInterface {
    module_key: ModuleKey,
    origin: ModuleArtifactOrigin,
    exported_functions: BTreeMap<Box<str>, CanonicalCheckedFunction>,
}

impl CanonicalPublicFunctionInterface {
    /// Returns the canonical key of the module whose functions are projected.
    #[must_use]
    pub fn module_key(&self) -> &ModuleKey {
        &self.module_key
    }

    /// Returns acquisition provenance for the projected module.
    #[must_use]
    pub fn origin(&self) -> &ModuleArtifactOrigin {
        &self.origin
    }

    /// Returns one publicly visible checked function by its defining name.
    #[must_use]
    pub fn exported_function(&self, name: &str) -> Option<&CanonicalCheckedFunction> {
        self.exported_functions.get(name)
    }
}

/// Checked closed-function facts for one canonical leaf module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalCheckedFunctionModule {
    artifact: ModuleArtifact,
    private_functions: BTreeMap<Box<str>, CanonicalCheckedFunction>,
    public_projection: CanonicalPublicFunctionInterface,
}

impl CanonicalCheckedFunctionModule {
    /// Returns the parser artifact retained for this checked module.
    #[must_use]
    pub fn artifact(&self) -> &ModuleArtifact {
        &self.artifact
    }

    /// Returns one checked function from the complete private declaration view.
    #[must_use]
    pub fn private_function(&self, name: &str) -> Option<&CanonicalCheckedFunction> {
        self.private_functions.get(name)
    }

    /// Returns the fresh Type-layer projection of publicly visible functions.
    #[must_use]
    pub fn public_interface(&self) -> &CanonicalPublicFunctionInterface {
        &self.public_projection
    }
}

/// Atomically checked closed-function modules keyed by canonical module key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalCheckedFunctionModuleSet {
    modules: BTreeMap<ModuleKey, CanonicalCheckedFunctionModule>,
}

impl CanonicalCheckedFunctionModuleSet {
    /// Returns checked facts for `module` when the complete graph succeeds.
    #[must_use]
    pub fn module(&self, module: &ModuleKey) -> Option<&CanonicalCheckedFunctionModule> {
        self.modules.get(module)
    }
}

/// A failure while checking a canonical closed-function leaf module.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CanonicalModuleCheckError {
    /// A parsed unit contains imports, children, or a non-function definition.
    #[error("unsupported closed-function module shape for {module}: {reason}")]
    UnsupportedModuleShape {
        /// The canonical key of the rejected module.
        module: ModuleKey,
        /// The parser anchor of the unsupported form.
        span: Span,
        /// A stable explanation of the unsupported shape.
        reason: &'static str,
    },
    /// An ordinary function uses a feature outside the primitive leaf domain.
    #[error("unsupported closed-function feature in {module}::{function}: {reason}")]
    UnsupportedFunctionFeature {
        /// The canonical key of the rejected module.
        module: ModuleKey,
        /// The defining function name.
        function: String,
        /// The parser anchor of the complete declaration.
        declaration_span: Span,
        /// A stable explanation of the unsupported feature.
        reason: &'static str,
    },
    /// A public signature is not closed over the primitive leaf domain.
    #[error("public signature for {module}::{function} is outside the closed primitive slice")]
    PublicSignatureOutsideClosedSlice {
        /// The canonical key of the rejected module.
        module: ModuleKey,
        /// The defining public function name.
        function: String,
        /// The parser anchor of the complete declaration.
        declaration_span: Span,
    },
    /// Two ordinary functions in one unit share the same defining name.
    #[error("duplicate closed-function declaration {function:?} in {module}")]
    DuplicateFunction {
        /// The canonical key of the rejected module.
        module: ModuleKey,
        /// The duplicated defining name.
        function: String,
        /// The parser anchor of the later declaration.
        declaration_span: Span,
    },
    /// Signature staging failed before any checked result was published.
    #[error("signature staging failed for {module}::{function}")]
    Signature {
        /// The canonical key of the rejected module.
        module: ModuleKey,
        /// The defining function name.
        function: String,
        /// The parser anchor of the complete declaration.
        declaration_span: Span,
        /// The underlying type-checking diagnostic.
        #[source]
        source: Box<TypeCheckError>,
    },
    /// Body checking failed after sibling signatures were staged.
    #[error("body checking failed for {module}::{function}")]
    BodyCheck {
        /// The canonical key of the rejected module.
        module: ModuleKey,
        /// The defining function name.
        function: String,
        /// The parser anchor of the complete declaration.
        declaration_span: Span,
        /// The underlying type-checking diagnostic.
        #[source]
        source: Box<TypeCheckError>,
    },
}

/// Checks every graph unit in the closed primitive function leaf domain.
///
/// Each unit is preflighted before its function signatures are staged in a
/// fresh builtin environment. All sibling bodies are then checked against
/// that staged view. The returned set is published only after every unit has
/// completed successfully.
///
/// # Errors
///
/// Returns [`CanonicalModuleCheckError`] when a unit or declaration is outside
/// this leaf domain, signature staging fails, or a body fails type checking.
#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract retains parser anchors directly"
)]
pub fn check_closed_function_modules(
    graph: &CanonicalModuleGraph,
) -> Result<CanonicalCheckedFunctionModuleSet, CanonicalModuleCheckError> {
    for (module_key, unit) in graph.module_units() {
        preflight_unit(graph, module_key, unit)?;
    }

    let mut modules = BTreeMap::new();
    for (module_key, unit) in graph.module_units() {
        let checked_module = check_unit(module_key, unit)?;
        modules.insert(module_key.clone(), checked_module);
    }

    Ok(CanonicalCheckedFunctionModuleSet { modules })
}

fn preflight_unit(
    graph: &CanonicalModuleGraph,
    module_key: &ModuleKey,
    unit: &ash_parser::ModuleUnit,
) -> Result<(), CanonicalModuleCheckError> {
    if let Some(use_declaration) = unit.body().uses().first() {
        return Err(CanonicalModuleCheckError::UnsupportedModuleShape {
            module: module_key.clone(),
            span: use_declaration.span,
            reason: "parsed uses are outside the closed-function leaf domain",
        });
    }
    if let Some(declaration) = unit.body().module_decls().first() {
        return Err(CanonicalModuleCheckError::UnsupportedModuleShape {
            module: module_key.clone(),
            span: declaration.span,
            reason: "nested module declarations are outside the closed-function leaf domain",
        });
    }
    if graph
        .children(module_key)
        .is_some_and(|children| !children.is_empty())
    {
        return Err(CanonicalModuleCheckError::UnsupportedModuleShape {
            module: module_key.clone(),
            span: unit.body().span(),
            reason: "structural children are outside the closed-function leaf domain",
        });
    }
    for definition in unit.body().definitions() {
        let Definition::Function(function) = definition else {
            return Err(CanonicalModuleCheckError::UnsupportedModuleShape {
                module: module_key.clone(),
                span: unit.body().span(),
                reason: "only ordinary functions are accepted in the closed-function leaf domain",
            });
        };
        preflight_function(module_key, function)?;
    }
    Ok(())
}

fn preflight_function(
    module_key: &ModuleKey,
    function: &FnDef,
) -> Result<(), CanonicalModuleCheckError> {
    let name = function.name.clone();
    let unsupported = |reason| CanonicalModuleCheckError::UnsupportedFunctionFeature {
        module: module_key.clone(),
        function: name.to_string(),
        declaration_span: function.span,
        reason,
    };

    if !function.type_params.is_empty() {
        return Err(unsupported(
            "generic functions are outside the closed-function leaf domain",
        ));
    }
    if function.proposition_tail.is_some() || function.contract.is_some() {
        return Err(unsupported(
            "function contracts are outside the closed-function leaf domain",
        ));
    }
    let supported_visibility = match &function.visibility {
        Visibility::Inherited
        | Visibility::Public
        | Visibility::Crate
        | Visibility::Super { .. }
        | Visibility::Self_ => true,
        Visibility::Restricted { path } => path.as_ref() == "crate" || path.starts_with("crate::"),
    };
    if !supported_visibility {
        return Err(unsupported(
            "restricted declaration visibility is outside the closed-function leaf domain",
        ));
    }
    let Some(return_type) = function.return_type.as_ref() else {
        return Err(unsupported(
            "an explicit return type is required in the closed-function leaf domain",
        ));
    };
    let has_primitive_signature = function
        .params
        .iter()
        .all(|parameter| is_primitive_surface_type(&parameter.ty))
        && is_primitive_surface_type(return_type);
    if !has_primitive_signature {
        if matches!(&function.visibility, Visibility::Public) {
            return Err(
                CanonicalModuleCheckError::PublicSignatureOutsideClosedSlice {
                    module: module_key.clone(),
                    function: name.to_string(),
                    declaration_span: function.span,
                },
            );
        }
        return Err(unsupported("only closed primitive signatures are accepted"));
    }
    Ok(())
}

fn is_primitive_surface_type(ty: &SurfaceType) -> bool {
    let SurfaceType::Name(name) = ty else {
        return false;
    };
    matches!(
        name.as_ref(),
        "Int" | "String" | "Bool" | "Float" | "Null" | "Time" | "Ref"
    )
}

#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract retains parser anchors directly"
)]
fn check_unit(
    module_key: &ModuleKey,
    unit: &ash_parser::ModuleUnit,
) -> Result<CanonicalCheckedFunctionModule, CanonicalModuleCheckError> {
    let private_functions = check_primitive_function_unit(module_key, unit, &BTreeMap::new())
        .map_err(|error| match error {
            PrimitiveFunctionUnitCheckError::DuplicateFunction {
                function,
                declaration_span,
            } => CanonicalModuleCheckError::DuplicateFunction {
                module: module_key.clone(),
                function: function.to_string(),
                declaration_span,
            },
            PrimitiveFunctionUnitCheckError::Signature {
                function,
                declaration_span,
                source,
            } => CanonicalModuleCheckError::Signature {
                module: module_key.clone(),
                function: function.to_string(),
                declaration_span,
                source: Box::new(source),
            },
            PrimitiveFunctionUnitCheckError::BodyCheck {
                function,
                declaration_span,
                source,
            } => CanonicalModuleCheckError::BodyCheck {
                module: module_key.clone(),
                function: function.to_string(),
                declaration_span,
                source: Box::new(source),
            },
        })?;
    let origin = unit.artifact().origin().clone();
    let mut exported_functions = BTreeMap::new();
    for (name, checked) in &private_functions {
        if matches!(checked.visibility(), Visibility::Public) {
            exported_functions.insert(name.clone(), checked.clone());
        }
    }

    Ok(CanonicalCheckedFunctionModule {
        artifact: unit.artifact().clone(),
        private_functions,
        public_projection: CanonicalPublicFunctionInterface {
            module_key: module_key.clone(),
            origin,
            exported_functions,
        },
    })
}

/// A signature-staging or body-checking failure from the primitive unit helper.
#[derive(Debug)]
pub(crate) enum PrimitiveFunctionUnitCheckError {
    /// Two ordinary functions share a defining name.
    DuplicateFunction {
        function: Box<str>,
        declaration_span: Span,
    },
    /// An explicit function signature did not type check.
    Signature {
        function: Box<str>,
        declaration_span: Span,
        source: TypeCheckError,
    },
    /// A function body did not type check after signatures were staged.
    BodyCheck {
        function: Box<str>,
        declaration_span: Span,
        source: TypeCheckError,
    },
}

/// Checks a preflighted primitive function unit with already-selected imports.
///
/// The caller owns surface-shape admission. This helper stages imported names,
/// then all local function signatures, and checks bodies only against that
/// completed environment. The returned functions are freshly derived facts.
#[allow(
    clippy::result_large_err,
    reason = "callers need the unboxed diagnostic to retain a stable public error contract"
)]
pub(crate) fn check_primitive_function_unit(
    module_key: &ModuleKey,
    unit: &ash_parser::ModuleUnit,
    imported_signatures: &BTreeMap<Box<str>, Type>,
) -> Result<BTreeMap<Box<str>, CanonicalCheckedFunction>, PrimitiveFunctionUnitCheckError> {
    let mut environment = TypeEnv::with_builtin_types();
    for (name, signature) in imported_signatures {
        environment.bind_variable(name, signature.clone());
    }
    let mut staged = BTreeMap::new();
    for definition in unit.body().definitions() {
        let Definition::Function(function) = definition else {
            unreachable!("primitive-unit preflight accepts only ordinary functions");
        };
        let name = function.name.clone();
        let signature = fn_signature_type(&environment, function).map_err(|source| {
            PrimitiveFunctionUnitCheckError::Signature {
                function: name.clone(),
                declaration_span: function.span,
                source,
            }
        })?;
        if staged
            .insert(name.clone(), (function, signature.clone()))
            .is_some()
        {
            return Err(PrimitiveFunctionUnitCheckError::DuplicateFunction {
                function: name,
                declaration_span: function.span,
            });
        }
        environment.bind_variable(function.name.as_ref(), signature);
    }

    let origin = unit.artifact().origin().clone();
    let mut functions = BTreeMap::new();
    for (name, (function, signature)) in staged {
        let body_type = check_function_body_in_env(&environment, function).map_err(|source| {
            PrimitiveFunctionUnitCheckError::BodyCheck {
                function: name.clone(),
                declaration_span: function.span,
                source,
            }
        })?;
        functions.insert(
            name.clone(),
            CanonicalCheckedFunction::from_checked_parts(
                module_key.clone(),
                name,
                function.span,
                function.body.span(),
                origin.clone(),
                function.visibility.clone(),
                signature,
                body_type,
            ),
        );
    }
    Ok(functions)
}
