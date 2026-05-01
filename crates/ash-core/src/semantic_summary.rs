//! Core-owned module semantic summary carriers for ordinary type metadata.
//!
//! SPEC-057 defines this as an ordinary-type summary substrate. It intentionally
//! does not interpret future type-computation namespaces and does not replace the
//! Phase 108 workflow-summary carriers.

use crate::ast::{Name, Span, TypeBody, TypeVar, Visibility};
use crate::module_graph::{CrateId, ModuleId};
use serde::{Deserialize, Serialize};

/// Canonical identity plus diagnostic metadata for a resolved Ash module.
///
/// The numeric `ModuleId`/`CrateId` pair anchors graph identity. `path` and
/// `source` provide stable transport/debug metadata for summary consumers, but
/// they intentionally do not participate in equality or hashing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleIdentity {
    pub crate_id: Option<CrateId>,
    pub module_id: ModuleId,
    pub path: Vec<String>,
    pub source: ModuleSourceOrigin,
}

impl PartialEq for ModuleIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.crate_id == other.crate_id && self.module_id == other.module_id
    }
}

impl Eq for ModuleIdentity {}

impl std::hash::Hash for ModuleIdentity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.crate_id.hash(state);
        self.module_id.hash(state);
    }
}

impl ModuleIdentity {
    #[must_use]
    pub fn new(
        crate_id: Option<CrateId>,
        module_id: ModuleId,
        path: Vec<String>,
        source: ModuleSourceOrigin,
    ) -> Self {
        Self {
            crate_id,
            module_id,
            path,
            source,
        }
    }
}

/// Source origin for module/type summary diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleSourceOrigin {
    File(String),
    Inline { parent: ModuleId, offset: usize },
    Synthetic { reason: String },
}

/// Source origin for a summary fact. This is diagnostic metadata, not identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceOrigin {
    File(String),
    InlineModule {
        module: ModuleId,
        offset: usize,
    },
    ImportedSummary {
        module: Vec<String>,
        public_anchor: String,
    },
    Synthetic {
        reason: String,
    },
}

/// Diagnostic anchor for source/module context. Spans must not be used as ID inputs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceAnchor {
    pub origin: SourceOrigin,
    pub span: Option<Span>,
    pub label: String,
}

impl SourceAnchor {
    #[must_use]
    pub fn new(origin: SourceOrigin, span: Option<Span>, label: impl Into<String>) -> Self {
        Self {
            origin,
            span,
            label: label.into(),
        }
    }
}

/// Ordinary type declaration item kind participating in canonical identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeDeclItemKind {
    OrdinaryType,
}

/// Canonical ordinary type declaration identity.
///
/// Import aliases and re-export paths should point at this identity; they must
/// not construct new origin identities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeDeclId {
    pub module: ModuleIdentity,
    pub name: Name,
    pub item_kind: TypeDeclItemKind,
}

impl TypeDeclId {
    #[must_use]
    pub fn ordinary(module: ModuleIdentity, name: impl Into<Name>) -> Self {
        Self {
            module,
            name: name.into(),
            item_kind: TypeDeclItemKind::OrdinaryType,
        }
    }
}

/// Constructor/variant payload kind participating in constructor identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructorPayloadKind {
    Unit,
    Record,
    Tuple,
    Struct,
}

/// Canonical constructor/variant identity derived from parent type identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConstructorId {
    pub parent: TypeDeclId,
    pub name: Name,
    pub payload_kind: ConstructorPayloadKind,
}

impl ConstructorId {
    #[must_use]
    pub fn variant(
        parent: TypeDeclId,
        name: impl Into<Name>,
        payload_kind: ConstructorPayloadKind,
    ) -> Self {
        Self {
            parent,
            name: name.into(),
            payload_kind,
        }
    }
}

/// Exported name/path pointing at an origin type identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeExportRef {
    pub exported_name: Name,
    pub origin: TypeDeclId,
    pub anchor: Option<SourceAnchor>,
}

impl TypeExportRef {
    #[must_use]
    pub fn new(exported_name: impl Into<Name>, origin: TypeDeclId) -> Self {
        Self {
            exported_name: exported_name.into(),
            origin,
            anchor: None,
        }
    }

    #[must_use]
    pub fn with_anchor(mut self, anchor: SourceAnchor) -> Self {
        self.anchor = Some(anchor);
        self
    }
}

/// Public re-export path preserving the original type identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReExportSummary {
    pub exported_path: Vec<String>,
    pub origin: TypeDeclId,
    pub anchor: Option<SourceAnchor>,
}

impl ReExportSummary {
    #[must_use]
    pub fn new(exported_path: Vec<String>, origin: TypeDeclId) -> Self {
        Self {
            exported_path,
            origin,
            anchor: None,
        }
    }

    #[must_use]
    pub fn with_anchor(mut self, anchor: SourceAnchor) -> Self {
        self.anchor = Some(anchor);
        self
    }
}

/// Representation visibility carried by ordinary type summaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepresentationExposure {
    Exposed,
    Opaque,
}

/// Public/opaque ordinary type representation metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeRepresentationSummary {
    Exposed(TypeBody),
    Opaque { builtin: bool },
}

impl TypeRepresentationSummary {
    #[must_use]
    pub fn exposed(body: TypeBody) -> Self {
        Self::Exposed(body)
    }

    #[must_use]
    pub fn opaque(builtin: bool) -> Self {
        Self::Opaque { builtin }
    }
}

/// Summary for one ordinary type declaration visible in a module summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeDeclSummary {
    pub id: TypeDeclId,
    pub exported_name: Name,
    pub visibility: Visibility,
    pub params: Vec<TypeVar>,
    pub representation_exposure: RepresentationExposure,
    pub representation: TypeRepresentationSummary,
    pub source_anchor: SourceAnchor,
}

impl TypeDeclSummary {
    #[must_use]
    pub fn new(
        id: TypeDeclId,
        exported_name: impl Into<Name>,
        visibility: Visibility,
        representation_exposure: RepresentationExposure,
        representation: TypeRepresentationSummary,
        source_anchor: SourceAnchor,
    ) -> Self {
        Self {
            id,
            exported_name: exported_name.into(),
            visibility,
            params: Vec::new(),
            representation_exposure,
            representation,
            source_anchor,
        }
    }

    #[must_use]
    pub fn with_params(mut self, params: Vec<TypeVar>) -> Self {
        self.params = params;
        self
    }
}

/// Summary for an exposed ordinary ADT constructor/variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstructorSummary {
    pub id: ConstructorId,
    pub parent: TypeDeclId,
    pub exported_name: Name,
    pub payload_kind: ConstructorPayloadKind,
    pub visibility: Visibility,
    pub source_anchor: SourceAnchor,
}

impl ConstructorSummary {
    #[must_use]
    pub fn new(
        id: ConstructorId,
        parent: TypeDeclId,
        exported_name: impl Into<Name>,
        payload_kind: ConstructorPayloadKind,
        visibility: Visibility,
        source_anchor: SourceAnchor,
    ) -> Self {
        Self {
            id,
            parent,
            exported_name: exported_name.into(),
            payload_kind,
            visibility,
            source_anchor,
        }
    }
}

/// Reference to a transported summary for import/cache accounting.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleSummaryRef {
    pub module: ModuleIdentity,
    pub version: SummaryVersion,
}

/// Version tag for semantic summaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SummaryVersion(pub u16);

impl SummaryVersion {
    pub const SPEC057_ORDINARY_TYPE_V1: Self = Self(1);
}

/// Reserved future identity namespaces. SPEC-057 leaves these uninterpreted.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ReservedSemanticIdentitySlots {
    pub future_type_functions: Vec<String>,
    pub sealed_domains: Vec<String>,
    pub generalized_projections: Vec<String>,
    pub associated_families: Vec<String>,
    pub extensions: Vec<(String, String)>,
}

impl ReservedSemanticIdentitySlots {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.future_type_functions.is_empty()
            && self.sealed_domains.is_empty()
            && self.generalized_projections.is_empty()
            && self.associated_families.is_empty()
            && self.extensions.is_empty()
    }
}

/// Core-owned ordinary-type semantic summary for one module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleSemanticSummary {
    pub module: ModuleIdentity,
    pub version: SummaryVersion,
    pub exported_types: Vec<TypeDeclSummary>,
    pub exported_constructors: Vec<ConstructorSummary>,
    pub re_exports: Vec<ReExportSummary>,
    pub imported_summary_refs: Vec<ModuleSummaryRef>,
    pub reserved_identity_slots: ReservedSemanticIdentitySlots,
    pub diagnostic_anchors: Vec<SourceAnchor>,
}

impl ModuleSemanticSummary {
    #[must_use]
    pub fn new(module: ModuleIdentity) -> Self {
        Self {
            module,
            version: SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
            exported_types: Vec::new(),
            exported_constructors: Vec::new(),
            re_exports: Vec::new(),
            imported_summary_refs: Vec::new(),
            reserved_identity_slots: ReservedSemanticIdentitySlots::default(),
            diagnostic_anchors: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_exported_type(mut self, ty: TypeDeclSummary) -> Self {
        self.exported_types.push(ty);
        self
    }

    #[must_use]
    pub fn with_exported_constructor(mut self, constructor: ConstructorSummary) -> Self {
        self.exported_constructors.push(constructor);
        self
    }

    #[must_use]
    pub fn with_re_export(mut self, re_export: ReExportSummary) -> Self {
        self.re_exports.push(re_export);
        self
    }

    #[must_use]
    pub fn with_imported_summary_ref(mut self, summary_ref: ModuleSummaryRef) -> Self {
        self.imported_summary_refs.push(summary_ref);
        self
    }

    #[must_use]
    pub fn with_diagnostic_anchor(mut self, anchor: SourceAnchor) -> Self {
        self.diagnostic_anchors.push(anchor);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Span, TypeBody, TypeExpr, VariantDef, VariantPayload, Visibility};
    use crate::module_graph::{CrateId, ModuleId};

    fn module_identity() -> ModuleIdentity {
        ModuleIdentity::new(
            Some(CrateId(7)),
            ModuleId(42),
            vec!["crate".into(), "domain".into()],
            ModuleSourceOrigin::File("/repo/src/domain.ash".into()),
        )
    }

    #[test]
    fn type_decl_identity_ignores_module_diagnostic_metadata_spans_and_export_aliases() {
        let module = module_identity();
        let same_module_with_different_diagnostics = ModuleIdentity::new(
            Some(CrateId(7)),
            ModuleId(42),
            vec!["renamed".into(), "domain".into()],
            ModuleSourceOrigin::File("/different/path/domain.ash".into()),
        );
        let origin = TypeDeclId::ordinary(module.clone(), "UserId");
        let alias_import = TypeExportRef::new("Id", origin.clone()).with_anchor(SourceAnchor::new(
            SourceOrigin::File("/repo/src/client.ash".into()),
            Some(Span {
                start: 100,
                end: 102,
            }),
            "import alias",
        ));
        let re_export = ReExportSummary::new(
            vec!["crate".into(), "api".into(), "Id".into()],
            origin.clone(),
        )
        .with_anchor(SourceAnchor::new(
            SourceOrigin::File("/repo/src/api.ash".into()),
            Some(Span {
                start: 200,
                end: 220,
            }),
            "pub use",
        ));

        assert_eq!(module, same_module_with_different_diagnostics);
        assert_eq!(origin, alias_import.origin);
        assert_eq!(origin, re_export.origin);
        assert_eq!(
            origin,
            TypeDeclId::ordinary(same_module_with_different_diagnostics, "UserId")
        );
    }

    #[test]
    fn constructor_identity_derives_from_parent_type_name_and_payload_kind() {
        let parent = TypeDeclId::ordinary(module_identity(), "Status");

        let unit_pending =
            ConstructorId::variant(parent.clone(), "Pending", ConstructorPayloadKind::Unit);
        let record_pending =
            ConstructorId::variant(parent.clone(), "Pending", ConstructorPayloadKind::Record);
        let unit_done =
            ConstructorId::variant(parent.clone(), "Done", ConstructorPayloadKind::Unit);

        assert_eq!(unit_pending.parent, parent);
        assert_ne!(unit_pending, record_pending);
        assert_ne!(unit_pending, unit_done);
    }

    #[test]
    fn module_semantic_summary_represents_public_ordinary_type_and_constructor_exposure() {
        let module = module_identity();
        let type_id = TypeDeclId::ordinary(module.clone(), "Status");
        let pending_id =
            ConstructorId::variant(type_id.clone(), "Pending", ConstructorPayloadKind::Unit);
        let done_id =
            ConstructorId::variant(type_id.clone(), "Done", ConstructorPayloadKind::Record);
        let anchor = SourceAnchor::new(
            SourceOrigin::File("/repo/src/domain.ash".into()),
            Some(Span { start: 5, end: 55 }),
            "type Status",
        );

        let ty = TypeDeclSummary::new(
            type_id.clone(),
            "Status",
            Visibility::Public,
            RepresentationExposure::Exposed,
            TypeRepresentationSummary::exposed(TypeBody::Enum(vec![
                VariantDef {
                    name: "Pending".into(),
                    fields: vec![],
                    payload: VariantPayload::Unit,
                },
                VariantDef {
                    name: "Done".into(),
                    fields: vec![("code".into(), TypeExpr::Named("Int".into()))],
                    payload: VariantPayload::Record(vec![(
                        { "code".into() },
                        TypeExpr::Named("Int".into()),
                    )]),
                },
            ])),
            anchor.clone(),
        );
        let pending = ConstructorSummary::new(
            pending_id,
            type_id.clone(),
            "Pending",
            ConstructorPayloadKind::Unit,
            Visibility::Public,
            anchor.clone(),
        );
        let done = ConstructorSummary::new(
            done_id,
            type_id,
            "Done",
            ConstructorPayloadKind::Record,
            Visibility::Public,
            anchor.clone(),
        );

        let summary = ModuleSemanticSummary::new(module)
            .with_exported_type(ty.clone())
            .with_exported_constructor(pending.clone())
            .with_exported_constructor(done.clone())
            .with_diagnostic_anchor(anchor.clone());

        assert_eq!(summary.version, SummaryVersion::SPEC057_ORDINARY_TYPE_V1);
        assert_eq!(summary.exported_types, vec![ty]);
        assert_eq!(summary.exported_constructors, vec![pending, done]);
        assert_eq!(summary.diagnostic_anchors, vec![anchor]);
    }

    #[test]
    fn reserved_extension_namespaces_default_empty_and_remain_uninterpreted() {
        let summary = ModuleSemanticSummary::new(module_identity());

        assert!(summary.reserved_identity_slots.is_empty());
        assert!(
            summary
                .reserved_identity_slots
                .future_type_functions
                .is_empty()
        );
        assert!(summary.reserved_identity_slots.sealed_domains.is_empty());
        assert!(
            summary
                .reserved_identity_slots
                .generalized_projections
                .is_empty()
        );
        assert!(
            summary
                .reserved_identity_slots
                .associated_families
                .is_empty()
        );
        assert!(summary.reserved_identity_slots.extensions.is_empty());
    }

    #[test]
    fn ordinary_type_summary_does_not_replace_workflow_summary_carrier() {
        let summary = ModuleSemanticSummary::new(module_identity());
        let workflow_summary = crate::workflow_carrier::PublicWorkflowSummary::default();

        assert!(summary.exported_types.is_empty());
        assert_eq!(workflow_summary.node_count, 0);
    }
}
