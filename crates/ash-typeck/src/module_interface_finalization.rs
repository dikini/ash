//! Type checker-owned finalization of bounded module-interface projections.
//!
//! A [`PublicModuleInterface`] remains Core data until a collection issued from
//! a parser-owned [`ModuleUnit`] finalizes it here. This module deliberately
//! performs only artifact continuity and a bounded parser declaration
//! name/kind check. It neither resolves imports nor claims complete typed
//! namespace linkage or export closure.

use std::collections::BTreeSet;

use ash_core::module_graph::{ModuleArtifact, ModuleKey};
use ash_core::module_interface::{
    ModuleInterfaceBinding, ModuleInterfaceBindingKind, ModuleInterfaceDefiningIdentity,
    PublicModuleInterface,
};
use ash_parser::module::ModuleUnit;
use ash_parser::surface::{Definition, Visibility as SurfaceVisibility};
use thiserror::Error;

use crate::{CallableDeclarationKind, TypeEnv};

/// Typechecker provenance collected from one parser-owned module unit.
///
/// Collection validates public functions and handlers against declaration
/// markers registered under one canonical module-key context in [`TypeEnv`].
/// It retains only validated facts, plus parser-visible child modules and
/// syntax macros. Builtins, typed namespaces, and re-exports are deliberately
/// outside this bounded slice.
#[derive(Debug)]
pub struct TypeEnvModuleInterfaceCollection {
    artifact: ModuleArtifact,
    collected_public_bindings: CollectedPublicBindings,
}

/// A TypeEnv-issued wrapper around a checked Core public projection.
///
/// Its constructor is intentionally private: callers must first collect a
/// coherent [`ModuleUnit`] through [`TypeEnvModuleInterfaceCollection`]. This
/// wrapper is not import, Engine, admission, or runtime authority.
#[derive(Debug, Clone, PartialEq)]
pub struct FinalizedModuleInterface {
    module_key: ModuleKey,
    interface: PublicModuleInterface,
}

/// A failure while collecting or finalizing a bounded module interface.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ModuleInterfaceFinalizationError {
    /// The parser module unit did not retain a coherent structural artifact.
    #[error("collected module artifact {module} is structurally inconsistent")]
    InvalidCollectedArtifact {
        /// Canonical key of the structurally inconsistent artifact.
        module: ModuleKey,
    },
    /// The raw Core carrier belongs to a different canonical module.
    #[error("raw interface module {actual} does not match collected module {expected}")]
    ArtifactKeyMismatch {
        /// Canonical key retained by the TypeEnv collection.
        expected: ModuleKey,
        /// Canonical key retained by the raw Core carrier.
        actual: ModuleKey,
    },
    /// The raw Core carrier changed artifact facts after TypeEnv collection.
    #[error("raw interface artifact for module {module} differs from the collected artifact")]
    ArtifactMismatch {
        /// Canonical module identity whose artifact facts differ.
        module: ModuleKey,
    },
    /// The TypeEnv finalization context is already bound to another module key.
    #[error(
        "TypeEnv finalization context is bound to module {expected}, not collected module {actual}"
    )]
    TypeEnvModuleKeyMismatch {
        /// Canonical module key already bound to the TypeEnv context.
        expected: ModuleKey,
        /// Canonical key requested by the newly collected module unit.
        actual: ModuleKey,
    },
    /// TypeEnv declaration preflight rejected the collected module unit.
    #[error("TypeEnv declaration registration failed for module {module}: {reason}")]
    TypeEnvDeclarationRegistration {
        /// Canonical key of the module whose declaration preflight failed.
        module: ModuleKey,
        /// TypeEnv's declaration-preflight diagnostic.
        reason: String,
    },
    /// A public parser callable conflicts with its TypeEnv declaration marker.
    #[error(
        "public callable {name:?} requires {expected_kind:?} TypeEnv registration, found {actual_kind:?}"
    )]
    UnregisteredPublicCallable {
        /// Public callable spelling whose TypeEnv marker is incompatible.
        name: String,
        /// Marker implied by the parser declaration.
        expected_kind: CallableDeclarationKind,
        /// Marker currently registered in the TypeEnv, if any.
        actual_kind: Option<CallableDeclarationKind>,
    },
    /// A public Core binding has no matching parser/typechecker collection fact.
    #[error(
        "public {kind:?} binding {name:?} in module {module} lacks collected TypeEnv provenance"
    )]
    UncollectedPublicBinding {
        /// Visible public spelling rejected by the bounded projection check.
        name: String,
        /// Namespace of the rejected public binding.
        kind: ModuleInterfaceBindingKind,
        /// Module whose collected facts did not justify the binding.
        module: ModuleKey,
    },
}

impl TypeEnvModuleInterfaceCollection {
    /// Collects bounded parser declaration facts under one [`TypeEnv`].
    ///
    /// # Errors
    ///
    /// Returns [`ModuleInterfaceFinalizationError::InvalidCollectedArtifact`]
    /// if the supplied module unit does not retain coherent canonical
    /// structural facts, or
    /// [`ModuleInterfaceFinalizationError::TypeEnvModuleKeyMismatch`] when the
    /// TypeEnv is already bound to another canonical module key, or
    /// [`ModuleInterfaceFinalizationError::TypeEnvDeclarationRegistration`]
    /// when TypeEnv declaration preflight rejects the supplied module unit, or
    /// [`ModuleInterfaceFinalizationError::UnregisteredPublicCallable`] when a
    /// public function or handler has an incompatible existing marker.
    pub fn collect(
        type_env: &mut TypeEnv,
        module_unit: &ModuleUnit,
    ) -> Result<Self, ModuleInterfaceFinalizationError> {
        let artifact = module_unit.artifact();
        if !artifact_is_structurally_coherent(artifact) {
            return Err(ModuleInterfaceFinalizationError::InvalidCollectedArtifact {
                module: artifact.key().clone(),
            });
        }

        let collected_public_bindings = collect_public_bindings(module_unit);
        let mut staged_type_env = type_env.clone();
        staged_type_env
            .bind_module_interface_finalization_key(artifact.key())
            .map_err(
                |expected| ModuleInterfaceFinalizationError::TypeEnvModuleKeyMismatch {
                    expected,
                    actual: artifact.key().clone(),
                },
            )?;
        precheck_public_callable_marker_conflicts(&staged_type_env, &collected_public_bindings)?;
        staged_type_env
            .register_surface_declarations(module_unit.body().definitions())
            .map_err(
                |error| ModuleInterfaceFinalizationError::TypeEnvDeclarationRegistration {
                    module: artifact.key().clone(),
                    reason: error.to_string(),
                },
            )?;
        verify_public_callable_markers(&staged_type_env, &collected_public_bindings)?;
        *type_env = staged_type_env;

        Ok(Self {
            artifact: artifact.clone(),
            collected_public_bindings,
        })
    }

    /// Finalizes a raw Core projection only when it matches collected facts.
    ///
    /// # Errors
    ///
    /// Returns an error when the raw interface artifact differs from the
    /// collected module artifact or publishes a binding absent from the
    /// bounded parser/typechecker collection.
    pub fn finalize(
        self,
        interface: PublicModuleInterface,
    ) -> Result<FinalizedModuleInterface, ModuleInterfaceFinalizationError> {
        let actual_artifact = interface.artifact();
        if actual_artifact.key() != self.artifact.key() {
            return Err(ModuleInterfaceFinalizationError::ArtifactKeyMismatch {
                expected: self.artifact.key().clone(),
                actual: actual_artifact.key().clone(),
            });
        }
        if actual_artifact != &self.artifact {
            return Err(ModuleInterfaceFinalizationError::ArtifactMismatch {
                module: self.artifact.key().clone(),
            });
        }

        for binding in interface.bindings() {
            if !self.binding_is_collected(binding) {
                return Err(ModuleInterfaceFinalizationError::UncollectedPublicBinding {
                    name: binding.visible_name().to_owned(),
                    kind: binding.kind(),
                    module: self.artifact.key().clone(),
                });
            }
        }

        Ok(FinalizedModuleInterface {
            module_key: self.artifact.key().clone(),
            interface,
        })
    }

    fn binding_is_collected(&self, binding: &ModuleInterfaceBinding) -> bool {
        let fact = CollectedPublicBinding {
            name: binding.visible_name().to_owned(),
            kind: binding.kind(),
        };
        if !self.collected_public_bindings.bindings.contains(&fact) {
            return false;
        }

        match binding.defining_identity() {
            ModuleInterfaceDefiningIdentity::ChildModule(child) => {
                self.artifact
                    .key()
                    .child(binding.visible_name())
                    .ok()
                    .as_ref()
                    == Some(child)
            }
            ModuleInterfaceDefiningIdentity::Declaration(identity) => {
                identity.module == *self.artifact.key()
                    && identity.name == binding.visible_name()
                    && identity.kind == binding.kind()
            }
        }
    }
}

impl FinalizedModuleInterface {
    /// Returns the canonical key that the TypeEnv collection finalized.
    #[must_use]
    pub fn module_key(&self) -> &ModuleKey {
        &self.module_key
    }

    /// Returns the exact finalized module artifact without exposing the raw
    /// Core public-interface carrier to downstream module stages.
    #[must_use]
    pub const fn module_artifact(&self) -> &ModuleArtifact {
        self.interface.artifact()
    }

    /// Returns the immutable checked Core projection.
    #[must_use]
    pub fn interface(&self) -> &PublicModuleInterface {
        &self.interface
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CollectedPublicBinding {
    name: String,
    kind: ModuleInterfaceBindingKind,
}

#[derive(Debug, Default)]
struct CollectedPublicBindings {
    bindings: BTreeSet<CollectedPublicBinding>,
    public_callables: Vec<CollectedPublicCallable>,
}

#[derive(Debug)]
struct CollectedPublicCallable {
    name: String,
    declaration_kind: CallableDeclarationKind,
}

fn artifact_is_structurally_coherent(artifact: &ModuleArtifact) -> bool {
    artifact.structural_parent() == artifact.key().parent().as_ref()
        && artifact
            .child_keys()
            .iter()
            .all(|child| child.parent().as_ref() == Some(artifact.key()))
}

fn collect_public_bindings(module_unit: &ModuleUnit) -> CollectedPublicBindings {
    let mut collected = CollectedPublicBindings::default();

    for declaration in module_unit.body().module_decls() {
        if declaration.visibility == SurfaceVisibility::Public {
            collected.bindings.insert(CollectedPublicBinding {
                name: declaration.name.to_string(),
                kind: ModuleInterfaceBindingKind::ChildModule,
            });
        }
    }

    for definition in module_unit.body().definitions() {
        match definition {
            Definition::Function(function) if function.visibility == SurfaceVisibility::Public => {
                collect_public_callable(
                    &mut collected,
                    function.name.as_ref(),
                    CallableDeclarationKind::Function,
                );
            }
            Definition::Handler(handler) if handler.visibility == SurfaceVisibility::Public => {
                collect_public_callable(
                    &mut collected,
                    handler.name.as_ref(),
                    CallableDeclarationKind::Handler,
                );
            }
            Definition::Macro(macro_definition)
                if macro_definition.visibility == SurfaceVisibility::Public =>
            {
                // Macros stay parser/syntax-only: no TypeEnv callable marker
                // is asserted for them in this bounded finalizer.
                collected.bindings.insert(CollectedPublicBinding {
                    name: macro_definition.name.to_string(),
                    kind: ModuleInterfaceBindingKind::SyntaxMacro,
                });
            }
            // Builtins have no Function/Handler TypeEnv declaration marker.
            // Their public interface treatment is intentionally deferred.
            Definition::BuiltinFn(_) => {}
            // Notation has no single declaration name in the current parser
            // carrier, so this bounded V1 collector cannot justify publishing
            // it. Other typed namespaces and re-exports are likewise deferred.
            _ => {}
        }
    }

    collected
}

fn collect_public_callable(
    collected: &mut CollectedPublicBindings,
    name: &str,
    declaration_kind: CallableDeclarationKind,
) {
    let name = name.to_owned();
    collected.bindings.insert(CollectedPublicBinding {
        name: name.clone(),
        kind: ModuleInterfaceBindingKind::Callable,
    });
    collected.public_callables.push(CollectedPublicCallable {
        name,
        declaration_kind,
    });
}

fn precheck_public_callable_marker_conflicts(
    type_env: &TypeEnv,
    collected: &CollectedPublicBindings,
) -> Result<(), ModuleInterfaceFinalizationError> {
    for callable in &collected.public_callables {
        let actual_kind = type_env.callable_declaration_kind(&callable.name);
        if let Some(actual_kind) = actual_kind
            && actual_kind != callable.declaration_kind
        {
            return Err(
                ModuleInterfaceFinalizationError::UnregisteredPublicCallable {
                    name: callable.name.clone(),
                    expected_kind: callable.declaration_kind,
                    actual_kind: Some(actual_kind),
                },
            );
        }
    }

    Ok(())
}

fn verify_public_callable_markers(
    type_env: &TypeEnv,
    collected: &CollectedPublicBindings,
) -> Result<(), ModuleInterfaceFinalizationError> {
    for callable in &collected.public_callables {
        let actual_kind = type_env.callable_declaration_kind(&callable.name);
        if actual_kind != Some(callable.declaration_kind) {
            return Err(
                ModuleInterfaceFinalizationError::UnregisteredPublicCallable {
                    name: callable.name.clone(),
                    expected_kind: callable.declaration_kind,
                    actual_kind,
                },
            );
        }
    }

    Ok(())
}
