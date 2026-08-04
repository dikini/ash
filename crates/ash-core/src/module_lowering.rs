//! Non-executable module-aware Core and CPS artifact carriers.
//!
//! These values retain the exact finalized module artifact and immutable
//! checked-import snapshots alongside the already checked Core and CPS
//! results. They are public transport/data values, not sealed authority
//! tokens: a caller can construct or clone them, so they cannot be accepted as
//! evidence for admission, execution, or callable authority.
//!
//! A later, separately sealed typechecker/Engine-owned handoff must establish
//! and consume any admission authority. No type in this module can stand in
//! for that handoff, even when its artifact and import metadata look valid.

use crate::core_ash_typecheck::TypedCoreProgram;
use crate::cps::Term;
use crate::module_graph::ModuleArtifact;
use crate::module_interface::ModuleInterfaceBinding;

/// An immutable resolved-import snapshot retained by module lowering.
///
/// The local name is the importing module's alias. The binding retains the
/// defining identity and source origin established by checked-interface
/// resolution; it is metadata, not a callable environment entry or admission
/// credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedModuleImport {
    local_name: String,
    binding: ModuleInterfaceBinding,
}

impl ResolvedModuleImport {
    /// Creates one resolved-import metadata snapshot.
    #[must_use]
    pub fn new(local_name: impl Into<String>, binding: ModuleInterfaceBinding) -> Self {
        Self {
            local_name: local_name.into(),
            binding,
        }
    }

    /// Returns the local alias requested by the importing module.
    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }

    /// Returns the resolved checked-interface binding snapshot.
    #[must_use]
    pub fn binding(&self) -> &ModuleInterfaceBinding {
        &self.binding
    }
}

/// Checked Core content paired with module and import provenance.
///
/// This carrier is intentionally non-executable and non-authorizing. Because
/// it is public data, neither this value nor its checked Core content can be
/// used as admission or execution evidence. Its imports are retained in
/// canonical local-name/defining-identity order for deterministic diagnostics
/// and downstream inspection only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleCoreArtifact {
    module_artifact: ModuleArtifact,
    imports: Vec<ResolvedModuleImport>,
    checked_core_program: TypedCoreProgram,
}

impl ModuleCoreArtifact {
    /// Creates a Core artifact from a finalized module artifact and checked content.
    #[must_use]
    pub fn new(
        module_artifact: ModuleArtifact,
        mut imports: Vec<ResolvedModuleImport>,
        checked_core_program: TypedCoreProgram,
    ) -> Self {
        canonicalize_imports(&mut imports);
        Self {
            module_artifact,
            imports,
            checked_core_program,
        }
    }

    /// Returns the exact finalized module artifact that owns this result.
    #[must_use]
    pub const fn module_artifact(&self) -> &ModuleArtifact {
        &self.module_artifact
    }

    /// Returns immutable resolved-import snapshots in deterministic order.
    #[must_use]
    pub fn imports(&self) -> &[ResolvedModuleImport] {
        &self.imports
    }

    /// Returns the validated and type-checked Core program.
    #[must_use]
    pub const fn checked_core_program(&self) -> &TypedCoreProgram {
        &self.checked_core_program
    }
}

/// Checked CPS content paired with module and import provenance.
///
/// This carrier is intentionally non-executable and non-authorizing. Runtime
/// admission and Engine execution remain separate downstream responsibilities;
/// this public data cannot prove that either is permitted.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleCpsArtifact {
    module_artifact: ModuleArtifact,
    imports: Vec<ResolvedModuleImport>,
    cps_program: Term,
}

impl ModuleCpsArtifact {
    /// Derives CPS provenance metadata from one already-issued Core artifact.
    ///
    /// This copies the Core artifact's exact module artifact and resolved
    /// import snapshots after a checked Core-to-CPS bridge succeeds. It does
    /// not admit or execute the CPS term, and the result is still public data
    /// rather than an admission credential.
    #[must_use]
    pub fn from_core_artifact(core_artifact: &ModuleCoreArtifact, cps_program: Term) -> Self {
        Self {
            module_artifact: core_artifact.module_artifact.clone(),
            imports: core_artifact.imports.clone(),
            cps_program,
        }
    }

    /// Creates a CPS artifact from a finalized module artifact and checked content.
    ///
    /// This standalone transport constructor does not produce admission or
    /// execution evidence. The checked module-lowering path instead uses
    /// [`Self::from_core_artifact`] to retain Core-derived provenance.
    #[must_use]
    pub fn new(
        module_artifact: ModuleArtifact,
        mut imports: Vec<ResolvedModuleImport>,
        cps_program: Term,
    ) -> Self {
        canonicalize_imports(&mut imports);
        Self {
            module_artifact,
            imports,
            cps_program,
        }
    }

    /// Returns the exact finalized module artifact carried by this data value.
    #[must_use]
    pub const fn module_artifact(&self) -> &ModuleArtifact {
        &self.module_artifact
    }

    /// Returns immutable resolved-import snapshots in deterministic order.
    #[must_use]
    pub fn imports(&self) -> &[ResolvedModuleImport] {
        &self.imports
    }

    /// Returns the CPS term emitted by the checked Core-to-CPS bridge.
    #[must_use]
    pub const fn cps_program(&self) -> &Term {
        &self.cps_program
    }
}

fn canonicalize_imports(imports: &mut [ResolvedModuleImport]) {
    imports.sort_by(|left, right| {
        left.local_name().cmp(right.local_name()).then_with(|| {
            left.binding()
                .defining_identity()
                .cmp(right.binding().defining_identity())
        })
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Visibility;
    use crate::core_ash::{CoreAtom, CoreExpr, CoreRow};
    use crate::core_ash_lower::CoreLoweringContext;
    use crate::core_ash_typecheck::{CoreTypeCheckEnv, type_check_and_lower_core_program};
    use crate::core_ash_validate::{RawCoreProgram, validate_core_program};
    use crate::cps::ContRef;
    use crate::module_graph::{ModuleArtifactOrigin, ModuleKey};
    use crate::module_interface::ModuleInterfaceBindingKind;

    #[test]
    fn cps_artifact_derives_metadata_from_its_core_artifact() {
        let module_key = ModuleKey::root("client").expect("fixture key is canonical");
        let origin = ModuleArtifactOrigin::File("src/client.ash".to_owned());
        let artifact = ModuleArtifact::new(module_key.clone(), origin.clone(), None, Vec::new())
            .expect("fixture artifact is structurally valid");
        let import = ResolvedModuleImport::new(
            "remote",
            ModuleInterfaceBinding::declaration(
                "serve",
                ModuleKey::root("garden").expect("fixture key is canonical"),
                "serve",
                ModuleInterfaceBindingKind::Callable,
                Visibility::Public,
                ModuleArtifactOrigin::File("src/garden.ash".to_owned()),
            ),
        );
        let lowered = type_check_and_lower_core_program(
            validate_core_program(RawCoreProgram::new(CoreExpr::Atom(CoreAtom::LitInt(7))))
                .expect("literal Core fixture validates"),
            &CoreTypeCheckEnv::default(),
            CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default()),
        )
        .expect("literal Core fixture type-checks and lowers");
        let (checked_core_program, cps_program) = lowered.into_parts();
        let core_artifact = ModuleCoreArtifact::new(artifact, vec![import], checked_core_program);

        let cps_artifact = ModuleCpsArtifact::from_core_artifact(&core_artifact, cps_program);

        assert_eq!(
            cps_artifact.module_artifact(),
            core_artifact.module_artifact()
        );
        assert_eq!(cps_artifact.imports(), core_artifact.imports());
        assert_eq!(cps_artifact.module_artifact().key(), &module_key);
        assert_eq!(cps_artifact.module_artifact().origin(), &origin);
    }
}
