use super::*;

impl TypeEnv {
    /// Second pass: validate source-ADT, source-constructor, field-domain, and kinding metadata.
    pub(super) fn validate_and_register_promoted_data_kind(
        &mut self,
        data_kind: &PromotedDataKindSummary,
    ) -> Result<(), TypeEnvError> {
        let source_visible_name = self
            .canonical_type_names
            .get(&data_kind.source_type)
            .cloned()
            .ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!(
                        "promoted data-kind '{}' references unknown source ADT '{}'",
                        data_kind.exported_name, data_kind.source_type.name
                    ),
                    anchor_span(&data_kind.source_anchor),
                )
            })?;
        let source_variants = match self.type_info.get(&source_visible_name).cloned() {
            Some(TypeInfo::Enum { variants, .. }) => variants,
            _ => {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "promoted data-kind '{}' source ADT '{}' is not an exposed enum",
                        data_kind.exported_name, data_kind.source_type.name
                    ),
                    anchor_span(&data_kind.source_anchor),
                ));
            }
        };

        if data_kind.constructors.len() != source_variants.len() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data-kind '{}' has {} constructor(s) but source ADT '{}' has {}",
                    data_kind.exported_name,
                    data_kind.constructors.len(),
                    data_kind.source_type.name,
                    source_variants.len()
                ),
                anchor_span(&data_kind.source_anchor),
            ));
        }

        for (index, constructor) in data_kind.constructors.iter().enumerate() {
            if constructor.source_constructor.parent != data_kind.source_type {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "source constructor for promoted constructor '{}' does not belong to source ADT '{}'",
                        constructor.exported_name, data_kind.source_type.name
                    ),
                    anchor_span(&constructor.source_anchor),
                ));
            }

            let source_variant = &source_variants[index];
            if constructor.source_constructor.name != source_variant.name {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "promoted constructor '{}' at index {} does not match source constructor '{}'",
                        constructor.exported_name, index, source_variant.name
                    ),
                    anchor_span(&constructor.source_anchor),
                ));
            }
            let actual_payload_kind = match &source_variant.payload_shape {
                VariantPayloadShape::Unit => ConstructorPayloadKind::Unit,
                VariantPayloadShape::Record => ConstructorPayloadKind::Record,
                VariantPayloadShape::Tuple => ConstructorPayloadKind::Tuple,
            };
            if actual_payload_kind != constructor.source_constructor.payload_kind {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "promoted constructor '{}' source payload kind {:?} conflicts with exposed source ADT payload kind {:?}",
                        constructor.exported_name,
                        constructor.source_constructor.payload_kind,
                        actual_payload_kind
                    ),
                    anchor_span(&constructor.source_anchor),
                ));
            }
            if constructor.fields.len() != source_variant.fields.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "promoted constructor '{}' has {} promoted field(s) but source constructor '{}' has {} field(s)",
                        constructor.exported_name,
                        constructor.fields.len(),
                        constructor.source_constructor.name,
                        source_variant.fields.len()
                    ),
                    anchor_span(&constructor.source_anchor),
                ));
            }

            let mut field_constraints = Vec::with_capacity(constructor.fields.len());
            for (index, field) in constructor.fields.iter().enumerate() {
                let (source_field_name, source_field_ty) = &source_variant.fields[index];
                if source_variant.payload_shape == VariantPayloadShape::Record
                    && field.name.as_str() != source_field_name.as_str()
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "field '{}' in promoted constructor '{}' does not match source field '{}'",
                            field.name, constructor.exported_name, source_field_name
                        ),
                        anchor_span(&field.source_anchor),
                    ));
                }
                if field.kind != Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "field '{}' in promoted constructor '{}' has non-Type kind",
                            field.name, constructor.exported_name
                        ),
                        anchor_span(&field.source_anchor),
                    ));
                }
                let Some(field_data_kind) = field.data_kind_constraint.clone() else {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "field '{}' in promoted constructor '{}' lacks promoted data-kind constraint",
                            field.name, constructor.exported_name
                        ),
                        anchor_span(&field.source_anchor),
                    ));
                };
                if !self
                    .promoted_data_kind_identities
                    .contains(&field_data_kind)
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "field '{}' in promoted constructor '{}' references unknown promoted data kind '{}'",
                            field.name, constructor.exported_name, field_data_kind.name
                        ),
                        anchor_span(&field.source_anchor),
                    ));
                }
                let expected_source_name = self
                    .canonical_type_names
                    .get(&field_data_kind.source_type)
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "field '{}' in promoted constructor '{}' references promoted data kind '{}' with unknown source ADT '{}'",
                                field.name,
                                constructor.exported_name,
                                field_data_kind.name,
                                field_data_kind.source_type.name
                            ),
                            anchor_span(&field.source_anchor),
                        )
                    })?;
                let source_field_matches_promoted_kind = matches!(
                    source_field_ty,
                    Type::Constructor { name, args, kind }
                        if args.is_empty() && kind.is_type() && name.name == *expected_source_name
                );
                if !source_field_matches_promoted_kind {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "field '{}' in promoted constructor '{}' expects source field type for promoted data kind '{}'",
                            field.name, constructor.exported_name, field_data_kind.name
                        ),
                        anchor_span(&field.source_anchor),
                    ));
                }
                field_constraints.push(Some(field_data_kind));
            }

            self.promoted_constructor_kinds.insert(
                constructor.id.clone(),
                PromotedConstructorKindInfo {
                    kind: Kind::n_ary(constructor.fields.len()),
                    result_data_kind: data_kind.id.clone(),
                    field_data_kind_constraints: field_constraints,
                },
            );
            self.promoted_constructor_summaries
                .insert(constructor.id.clone(), constructor.clone());
        }

        let should_store_data_kind = self
            .promoted_data_kind_summaries
            .get(&data_kind.id)
            .is_none_or(|existing| {
                is_dependency_metadata_name(&existing.exported_name)
                    || !is_dependency_metadata_name(&data_kind.exported_name)
            });
        if should_store_data_kind {
            self.promoted_data_kind_summaries
                .insert(data_kind.id.clone(), data_kind.clone());
        }
        Ok(())
    }

    /// Look up a promoted data kind by its visible exported name.
    #[must_use]
    pub fn lookup_promoted_data_kind(&self, name: &str) -> Option<&PromotedDataKindSummary> {
        let id = self.promoted_data_kind_aliases.get(name)?;
        self.promoted_data_kind_summaries.get(id)
    }

    /// Look up a promoted data kind by canonical identity.
    #[must_use]
    pub fn lookup_promoted_data_kind_by_id(
        &self,
        id: &PromotedDataKindId,
    ) -> Option<&PromotedDataKindSummary> {
        self.promoted_data_kind_summaries.get(id)
    }

    /// Look up a promoted data constructor by canonical identity.
    #[must_use]
    pub fn lookup_promoted_constructor_by_id(
        &self,
        id: &PromotedConstructorId,
    ) -> Option<&PromotedConstructorSummary> {
        self.promoted_constructor_summaries.get(id)
    }

    /// Return checked kind/domain metadata for a promoted data constructor.
    #[must_use]
    pub fn promoted_constructor_kind(
        &self,
        id: &PromotedConstructorId,
    ) -> Option<&PromotedConstructorKindInfo> {
        self.promoted_constructor_kinds.get(id)
    }

    /// Register a source-ordered batch of module-local type functions.
    ///
    /// TASK-834 deliberately performs only minimal honest lowering/registration:
    /// the current head is provisional during its own lowering, earlier published
    /// heads are visible, later same-module heads are rejected, and the checked
    /// carrier is published only after lowering succeeds. Deeper SPEC-E validation
    /// (coverage, overlap, and recursion proof obligations) remains owned by
    /// TASK-836/837.
    pub fn register_local_type_functions(
        &mut self,
        module: &ModuleIdentity,
        defs: &[SurfaceTypeFnDef],
    ) -> Result<(), TypeEnvError> {
        let mut staged = self.clone();
        staged.register_local_type_functions_inner(module, defs)?;
        *self = staged;
        Ok(())
    }

    /// Register a local sealed-domain summary for source declarations in the current module.
    ///
    /// Unlike `register_module_semantic_summary`, this does not require public visibility because
    /// it models same-module domains before export filtering. Public export validation rejects any
    /// `pub type fn` whose checked equations depend on private domains or marker constructors.
    pub fn register_local_sealed_domain_summary(
        &mut self,
        domain: &SealedDomainSummary,
    ) -> Result<(), TypeEnvError> {
        let mut staged = self.clone();
        staged.declare_sealed_domain_identity(domain)?;
        staged.validate_and_register_sealed_domain(domain)?;
        *self = staged;
        Ok(())
    }

    /// Look up a source-visible type function by local or imported name.
    #[must_use]
    pub fn lookup_local_type_function(&self, name: &str) -> Option<&TypeFunctionDef> {
        let head = self.local_type_function_heads.get(name)?;
        self.local_type_functions.get(head)
    }

    /// Make an imported public type-function summary source-visible under `name`.
    ///
    /// Import loaders call this only for explicitly selected or glob-imported
    /// public heads. Dependency-closure helper heads remain normalizer-available
    /// by canonical identity but are not inserted here.
    pub fn expose_imported_type_function_name(
        &mut self,
        name: impl Into<String>,
        head: TypeComputationHeadId,
    ) -> Result<(), TypeEnvError> {
        let name = name.into();
        if !self.local_type_functions.contains_key(&head) {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "cannot expose imported type function '{}' before registering summary head '{}::{}'",
                    name,
                    head.module.path.join("::"),
                    head.name
                ),
                Span::default(),
            ));
        }
        if let Some(existing) = self.local_type_function_heads.get(&name) {
            if existing == &head {
                return Ok(());
            }
            return Err(TypeEnvError::ImportOrderConflict {
                family: "type-function visible name".to_string(),
                name,
                span: Span::default(),
            });
        }
        self.local_type_function_heads.insert(name, head);
        Ok(())
    }

    /// Look up a published computation head by canonical identity.
    ///
    /// This unified normalizer lookup covers checked local declarations,
    /// atomically imported public summaries, and any future TypeEnv-owned
    /// computation-head sources. Imported heads are deliberately not inserted into
    /// `local_type_function_heads`, so they remain unavailable to local-name
    /// source lookup unless a later import/re-export path makes them visible.
    #[must_use]
    pub(crate) fn lookup_type_function_by_head(
        &self,
        head: &TypeComputationHeadId,
    ) -> Option<&TypeFunctionDef> {
        self.local_type_functions.get(head)
    }

    /// Iterate source-visible local and imported type-function names.
    ///
    /// Imported dependency-closure helper heads are intentionally omitted unless
    /// the import loader explicitly exposes them through selected/glob syntax.
    pub fn local_type_function_names(&self) -> impl Iterator<Item = &str> {
        self.local_type_function_heads.keys().map(String::as_str)
    }

    /// Lower checked, transparent, export-closed public local type functions into
    /// SPEC-062 public computation summaries.
    ///
    /// This only exports already-validated public source definitions for the
    /// requested defining module. It deliberately does not register imported
    /// normalizer facts or expose private/local-only type functions.
    pub fn export_public_type_function_summaries(
        &self,
        module: &ModuleIdentity,
    ) -> Result<Vec<TypeFunctionSummary>, TypeEnvError> {
        let mut defs = self
            .local_type_functions
            .values()
            .filter(|def| {
                def.visibility == ash_core::ast::Visibility::Public && def.head.module == *module
            })
            .collect::<Vec<_>>();
        defs.sort_by(|left, right| {
            let left_start = left
                .source_anchors
                .definition
                .span
                .map_or(usize::MAX, |s| s.start);
            let right_start = right
                .source_anchors
                .definition
                .span
                .map_or(usize::MAX, |s| s.start);
            left_start
                .cmp(&right_start)
                .then_with(|| left.name.cmp(&right.name))
        });

        defs.into_iter()
            .map(|def| self.lower_public_type_function_summary(def))
            .collect()
    }

    /// Lower checked public associated-family schemes for the requested module into
    /// SPEC-063 public associated-family summaries.
    pub fn export_public_associated_family_summaries(
        &self,
        module: &ModuleIdentity,
    ) -> Result<Vec<AssociatedFamilySummary>, TypeEnvError> {
        let mut declarations = self
            .associated_family_declarations
            .values()
            .filter(|declaration| declaration.defining_module == *module)
            .collect::<Vec<_>>();
        declarations.sort_by(|left, right| {
            left.head
                .interface
                .name
                .cmp(&right.head.interface.name)
                .then_with(|| left.head.member.name.cmp(&right.head.member.name))
        });

        let mut exportable_heads = HashMap::new();
        for declaration in &declarations {
            let interface_name = declaration.head.interface.name.to_string();
            let is_public_interface = self
                .interfaces
                .get(&interface_name)
                .is_some_and(|info| matches!(info.visibility, ash_core::ast::Visibility::Public));
            if !is_public_interface {
                continue;
            }
            let schemes = self
                .associated_family_schemes
                .get(&declaration.head)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|registered| registered.defining_module == *module)
                .map(|registered| registered.scheme)
                .collect::<Vec<_>>();
            if schemes.is_empty() {
                continue;
            }
            exportable_heads.insert(declaration.head.clone(), true);

            let mut closure = PublicAssociatedFamilyClosure::default();
            self.collect_public_associated_family_constraint_closure(
                &declaration.result_domain,
                &mut closure,
            );
            for scheme in &schemes {
                self.collect_public_associated_family_scheme_closure(scheme, &mut closure)?;
            }
            for dependency in closure.associated_families {
                if dependency == declaration.head {
                    continue;
                }
                if self
                    .associated_family_declarations
                    .get(&dependency)
                    .is_some_and(|dependency_declaration| {
                        dependency_declaration.defining_module == *module
                            && !self
                                .interfaces
                                .get(dependency.interface.name.as_str())
                                .is_some_and(|info| {
                                    matches!(info.visibility, ash_core::ast::Visibility::Public)
                                })
                    })
                {
                    exportable_heads.entry(dependency).or_insert(false);
                }
            }
        }

        let mut summaries = Vec::new();
        for declaration in declarations {
            let Some(source_visible) = exportable_heads.get(&declaration.head).copied() else {
                continue;
            };
            let schemes = self
                .associated_family_schemes
                .get(&declaration.head)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|registered| registered.defining_module == *module)
                .map(|registered| registered.scheme)
                .collect::<Vec<_>>();
            if schemes.is_empty() {
                continue;
            }
            let mut closure = PublicAssociatedFamilyClosure::default();
            self.collect_public_associated_family_constraint_closure(
                &declaration.result_domain,
                &mut closure,
            );
            for scheme in &schemes {
                self.collect_public_associated_family_scheme_closure(scheme, &mut closure)?;
            }
            self.validate_public_associated_family_export_closure(&closure, &declaration.head)?;
            let associated_family_refs =
                closure.associated_family_summary_refs(&declaration.head, module, self);
            let helper_family_count = associated_family_refs
                .iter()
                .filter(|reference| !reference.source_visible)
                .count();
            let public_associated_family_count = associated_family_refs.len() + 1;
            let decreases = declaration
                .decreases
                .as_ref()
                .and_then(|param| {
                    declaration
                        .interface_params
                        .iter()
                        .position(|candidate| candidate.name == *param)
                        .map(|index| (param, index))
                })
                .and_then(|(param, index)| {
                    declaration
                        .interface_params
                        .get(index)
                        .and_then(|param_info| param_info.domain_constraint.clone())
                        .map(|domain| ValidatedDecreasesSummary {
                            parameter: param.clone(),
                            parameter_index: index,
                            domain,
                            structural_recursion_checked: true,
                            source_anchor: SourceAnchor::new(
                                SourceOrigin::Synthetic {
                                    reason: "associated family decreases export".to_string(),
                                },
                                None,
                                format!("associated family decreases {param}"),
                            ),
                        })
                })
                .into_iter()
                .collect::<Vec<_>>();
            summaries.push(AssociatedFamilySummary {
                head: declaration.head.clone(),
                interface_identity: declaration.head.interface.clone(),
                member_identity: declaration.head.member.clone(),
                visible_name: if source_visible {
                    declaration.head.member.name.to_string()
                } else {
                    dependency_metadata_name(&declaration.head.member.name)
                },
                result_domain: canonical_expr_for_associated_family_constraint(
                    &declaration.result_domain,
                ),
                result_kind: Kind::Type,
                export_mode: AssociatedFamilyExportMode::TransparentEquations,
                schemes,
                dependency_closure: ash_core::semantic_summary::AssociatedFamilyDependencyClosure {
                    ordinary_types: closure.ordinary_types.iter().cloned().collect(),
                    sealed_domains: closure.sealed_domains.iter().cloned().collect(),
                    domain_constructors: closure.domain_constructors.iter().cloned().collect(),
                    type_functions: closure.type_functions.iter().cloned().collect(),
                    associated_projections: closure.projections.iter().cloned().collect(),
                    associated_families: associated_family_refs,
                    type_function_summaries: Vec::new(),
                    closure_metadata: AssociatedFamilyClosureMetadata {
                        public_closure_checked: true,
                        public_ordinary_type_count: closure.ordinary_types.len(),
                        public_sealed_domain_count: closure.sealed_domains.len(),
                        public_domain_constructor_count: closure.domain_constructors.len(),
                        public_type_function_count: closure.type_functions.len(),
                        public_associated_family_count,
                        public_projection_count: closure.projections.len(),
                        helper_family_count,
                    },
                },
                source_anchor: SourceAnchor::new(
                    SourceOrigin::Synthetic {
                        reason: "associated family summary export".to_string(),
                    },
                    None,
                    format!("associated family summary {}", declaration.head.member.name),
                ),
                revalidation_metadata: AssociatedFamilyRevalidationMetadata {
                    spec_version: SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4,
                    kind_and_domain_checked: true,
                    coverage_and_overlap_checked: true,
                    coherence_checked: true,
                    recursion_checked: true,
                    decreases,
                },
            });
        }
        Ok(summaries)
    }

    pub(super) fn validate_public_associated_family_export_closure(
        &self,
        closure: &PublicAssociatedFamilyClosure,
        head: &AssociatedFamilyHeadId,
    ) -> Result<(), TypeEnvError> {
        for ty in &closure.ordinary_types {
            if ty.module == head.interface.module
                && self
                    .ast_types
                    .get(ty.name.as_str())
                    .is_some_and(|def| !matches!(def.visibility, ash_core::ast::Visibility::Public))
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "public associated family '{}::{}' references private ordinary type '{}'",
                        head.interface.name, head.member.name, ty.name
                    ),
                    Span::default(),
                ));
            }
        }
        for domain in &closure.sealed_domains {
            if domain.module == head.interface.module
                && self
                    .sealed_domain_summaries
                    .get(domain)
                    .is_some_and(|summary| {
                        !matches!(summary.visibility, ash_core::ast::Visibility::Public)
                    })
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "public associated family '{}::{}' references private sealed domain '{}'",
                        head.interface.name, head.member.name, domain.name
                    ),
                    Span::default(),
                ));
            }
        }
        Ok(())
    }

    pub(super) fn collect_public_associated_family_constraint_closure(
        &self,
        constraint: &AssociatedFamilyResultConstraint,
        closure: &mut PublicAssociatedFamilyClosure,
    ) {
        match constraint {
            AssociatedFamilyResultConstraint::Kind(_) => {}
            AssociatedFamilyResultConstraint::Domain(domain) => {
                closure.sealed_domains.insert(domain.clone());
            }
        }
    }

    pub(super) fn collect_public_canonical_type_closure_for_associated_family(
        &self,
        ty: &CanonicalTypeExpr,
        closure: &mut PublicAssociatedFamilyClosure,
    ) {
        match ty {
            CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::Var(_) => {}
            CanonicalTypeExpr::NominalApp { origin, args, .. } => {
                closure.ordinary_types.insert(origin.clone());
                for arg in args {
                    self.collect_public_canonical_type_closure_for_associated_family(arg, closure);
                }
            }
            CanonicalTypeExpr::Projection {
                interface,
                member,
                args,
                kind,
                rigidity,
            } => {
                closure.projections.insert(AssociatedFamilyProjection {
                    head: AssociatedFamilyHeadId {
                        interface: interface.clone(),
                        member: member.clone(),
                    },
                    interface_args: args.clone(),
                    kind: kind.clone(),
                    rigidity: *rigidity,
                    mode: AssociatedFamilyProjectionMode::NeutralBlockedOrUnavailable,
                });
                for arg in args {
                    self.collect_public_canonical_type_closure_for_associated_family(arg, closure);
                }
            }
            CanonicalTypeExpr::ComputationHeadApp { head, args, .. } => {
                closure.type_functions.insert(head.clone());
                for arg in args {
                    self.collect_public_canonical_type_closure_for_associated_family(arg, closure);
                }
            }
            CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
                for arg in &app.args {
                    self.collect_public_canonical_type_closure_for_associated_family(arg, closure);
                }
            }
            CanonicalTypeExpr::ConstructorVariableApp(app) => {
                for arg in &app.args {
                    self.collect_public_canonical_type_closure_for_associated_family(arg, closure);
                }
            }
        }
    }

    pub(super) fn collect_public_associated_family_scheme_closure(
        &self,
        scheme: &AssociatedFamilyScheme,
        closure: &mut PublicAssociatedFamilyClosure,
    ) -> Result<(), TypeEnvError> {
        self.collect_public_canonical_type_closure_for_associated_family(
            &scheme.result_domain,
            closure,
        );
        for param in &scheme.params {
            self.collect_public_canonical_type_closure_for_associated_family(&param.ty, closure);
            if let Some(domain) = &param.domain_constraint {
                closure.sealed_domains.insert(domain.clone());
            }
        }
        for equation in &scheme.equations {
            for pattern in &equation.interface_arg_patterns {
                self.collect_public_associated_family_pattern_closure(pattern, closure);
            }
            self.collect_public_associated_family_result_closure(&equation.result, closure)?;
        }
        Ok(())
    }

    pub(super) fn collect_public_associated_family_pattern_closure(
        &self,
        pattern: &AssociatedFamilyPattern,
        closure: &mut PublicAssociatedFamilyClosure,
    ) {
        match pattern {
            AssociatedFamilyPattern::Var { constraint, .. }
            | AssociatedFamilyPattern::Wildcard { constraint, .. } => {
                self.collect_public_associated_family_constraint_closure(constraint, closure);
            }
            AssociatedFamilyPattern::Primitive { constraint, .. } => {
                self.collect_public_associated_family_constraint_closure(constraint, closure);
            }
            AssociatedFamilyPattern::NominalApp {
                origin,
                args,
                constraint,
                ..
            } => {
                closure.ordinary_types.insert(origin.clone());
                self.collect_public_associated_family_constraint_closure(constraint, closure);
                for arg in args {
                    self.collect_public_associated_family_pattern_closure(arg, closure);
                }
            }
            AssociatedFamilyPattern::DomainConstructor {
                domain,
                constructor,
                fields,
                constraint,
                ..
            } => {
                closure.sealed_domains.insert((**domain).clone());
                closure.domain_constructors.insert((**constructor).clone());
                self.collect_public_associated_family_constraint_closure(constraint, closure);
                for field in fields {
                    self.collect_public_associated_family_pattern_closure(field, closure);
                }
            }
        }
    }

    pub(super) fn collect_public_associated_family_head_closure(
        &self,
        head: &AssociatedFamilyHeadId,
        closure: &mut PublicAssociatedFamilyClosure,
    ) -> Result<(), TypeEnvError> {
        if !closure.associated_families.insert(head.clone()) {
            return Ok(());
        }
        let schemes = self
            .associated_family_schemes
            .get(head)
            .cloned()
            .unwrap_or_default();
        for registered in schemes {
            self.collect_public_associated_family_scheme_closure(&registered.scheme, closure)?;
        }
        Ok(())
    }

    pub(super) fn collect_public_associated_family_result_closure(
        &self,
        expr: &AssociatedFamilyResultExpr,
        closure: &mut PublicAssociatedFamilyClosure,
    ) -> Result<(), TypeEnvError> {
        match expr {
            AssociatedFamilyResultExpr::Primitive { constraint, .. }
            | AssociatedFamilyResultExpr::Var { constraint, .. } => {
                self.collect_public_associated_family_constraint_closure(constraint, closure);
            }
            AssociatedFamilyResultExpr::NominalApp {
                origin,
                args,
                constraint,
                ..
            } => {
                closure.ordinary_types.insert(origin.clone());
                self.collect_public_associated_family_constraint_closure(constraint, closure);
                for arg in args {
                    self.collect_public_associated_family_result_closure(arg, closure)?;
                }
            }
            AssociatedFamilyResultExpr::DomainConstructorApp {
                domain,
                constructor,
                args,
                constraint,
                ..
            } => {
                closure.sealed_domains.insert(domain.clone());
                closure.domain_constructors.insert(constructor.clone());
                self.collect_public_associated_family_constraint_closure(constraint, closure);
                for arg in args {
                    self.collect_public_associated_family_result_closure(arg, closure)?;
                }
            }
            AssociatedFamilyResultExpr::Projection {
                args, constraint, ..
            }
            | AssociatedFamilyResultExpr::ComputationHeadApp {
                args, constraint, ..
            } => {
                self.collect_public_associated_family_constraint_closure(constraint, closure);
                if let AssociatedFamilyResultExpr::ComputationHeadApp { head, .. } = expr {
                    closure.type_functions.insert(head.clone());
                }
                for arg in args {
                    self.collect_public_associated_family_result_closure(arg, closure)?;
                }
            }
            AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                head,
                interface_args,
                kind,
                rigidity,
                constraint,
                ..
            } => {
                self.collect_public_associated_family_head_closure(head, closure)?;
                self.collect_public_associated_family_constraint_closure(constraint, closure);
                let canonical_args = interface_args
                    .iter()
                    .map(associated_family_result_expr_to_canonical)
                    .collect::<Result<Vec<_>, _>>()?;
                closure.projections.insert(AssociatedFamilyProjection {
                    head: head.clone(),
                    interface_args: canonical_args,
                    kind: kind.clone(),
                    rigidity: *rigidity,
                    mode: AssociatedFamilyProjectionMode::ReducibleSealedFamilyHead,
                });
                for arg in interface_args {
                    self.collect_public_associated_family_result_closure(arg, closure)?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn lower_public_type_function_summary(
        &self,
        def: &TypeFunctionDef,
    ) -> Result<TypeFunctionSummary, TypeEnvError> {
        self.validate_public_type_function_export_closure(def, Span::default())?;

        let mut closure = PublicTypeFunctionClosure::default();
        self.collect_public_type_function_def_closure(def, &mut closure);

        Ok(TypeFunctionSummary {
            exported_name: def.name.clone(),
            head: def.head.clone(),
            visibility: def.visibility,
            params: def
                .params
                .iter()
                .map(|param| TypeFunctionParamSummary {
                    name: param.name.clone(),
                    ty: param.ty.clone(),
                    kind: param.kind.clone(),
                    domain_constraint: param.domain_constraint.clone(),
                    source_anchor: param.source_anchor.clone(),
                })
                .collect(),
            return_type: def.return_type.clone(),
            return_kind: def.return_kind.clone(),
            result_constraint: def.result_constraint.clone(),
            export_mode: TypeFunctionExportMode::TransparentEquations,
            source_anchors: def.source_anchors.clone(),
            equations: def.equations.clone(),
            dependency_summary_refs: closure.dependency_summary_refs(),
            closure_metadata: TypeFunctionClosureMetadata {
                public_closure_checked: true,
                public_ordinary_type_count: closure.ordinary_types.len(),
                public_sealed_domain_count: closure.sealed_domains.len(),
                public_type_function_count: closure.type_functions.len(),
                public_projection_count: closure.projections.len(),
            },
            revalidation_metadata: TypeFunctionRevalidationMetadata {
                spec_version: SummaryVersion::SPEC062_TYPE_COMPUTATION_V3,
                structural_recursion_checked: true,
                kind_and_domain_checked: true,
                coverage_and_overlap_checked: true,
                decreases_param: def.decreases.clone(),
            },
        })
    }

    pub(super) fn collect_public_type_function_def_closure(
        &self,
        def: &TypeFunctionDef,
        closure: &mut PublicTypeFunctionClosure,
    ) {
        if !closure.type_functions.insert(def.head.clone()) {
            return;
        }
        for param in &def.params {
            if let Some(domain) = &param.domain_constraint {
                closure.sealed_domains.insert(domain.clone());
            }
            self.collect_public_canonical_type_closure(&param.ty, closure);
        }
        if let TypeFunctionResultConstraint::Domain(domain) = &def.result_constraint {
            closure.sealed_domains.insert(domain.clone());
        }
        self.collect_public_canonical_type_closure(&def.return_type, closure);
        for equation in &def.equations {
            for pattern in &equation.patterns {
                self.collect_public_pattern_closure(pattern, closure);
            }
            self.collect_public_result_closure(&equation.result, closure);
        }
    }

    pub(super) fn collect_public_type_function_head_closure(
        &self,
        head: &TypeComputationHeadId,
        closure: &mut PublicTypeFunctionClosure,
    ) {
        match self.local_type_functions.get(head) {
            Some(def) if def.visibility == ash_core::ast::Visibility::Public => {
                self.collect_public_type_function_def_closure(def, closure);
            }
            _ => {
                closure.type_functions.insert(head.clone());
            }
        }
    }

    pub(super) fn collect_public_canonical_type_closure(
        &self,
        ty: &CanonicalTypeExpr,
        closure: &mut PublicTypeFunctionClosure,
    ) {
        match ty {
            CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::Var(_) => {}
            CanonicalTypeExpr::NominalApp { origin, args, .. } => {
                closure.ordinary_types.insert(origin.clone());
                for arg in args {
                    self.collect_public_canonical_type_closure(arg, closure);
                }
            }
            CanonicalTypeExpr::Projection {
                interface,
                member,
                args,
                ..
            } => {
                closure
                    .projections
                    .insert((interface.clone(), member.clone()));
                for arg in args {
                    self.collect_public_canonical_type_closure(arg, closure);
                }
            }
            CanonicalTypeExpr::ComputationHeadApp { head, args, .. } => {
                self.collect_public_type_function_head_closure(head, closure);
                for arg in args {
                    self.collect_public_canonical_type_closure(arg, closure);
                }
            }
            CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
                closure.promoted_data_kinds.insert(app.data_kind.clone());
                closure
                    .promoted_constructors
                    .insert(app.constructor.clone());
                closure
                    .ordinary_types
                    .insert(app.data_kind.source_type.clone());
                for arg in &app.args {
                    self.collect_public_canonical_type_closure(arg, closure);
                }
            }
            CanonicalTypeExpr::ConstructorVariableApp(app) => {
                for arg in &app.args {
                    self.collect_public_canonical_type_closure(arg, closure);
                }
            }
        }
    }

    pub(super) fn collect_public_pattern_closure(
        &self,
        pattern: &TypeFunctionPattern,
        closure: &mut PublicTypeFunctionClosure,
    ) {
        match pattern {
            TypeFunctionPattern::DomainConstructor { domain, fields, .. } => {
                closure.sealed_domains.insert((**domain).clone());
                for field in fields {
                    self.collect_public_pattern_closure(field, closure);
                }
            }
            TypeFunctionPattern::Var { constraint, .. }
            | TypeFunctionPattern::Wildcard { constraint, .. } => {
                if let TypeFunctionPatternConstraint::Domain(domain) = constraint {
                    closure.sealed_domains.insert(domain.clone());
                }
            }
        }
    }

    pub(super) fn collect_public_result_closure(
        &self,
        expr: &TypeFunctionResultExpr,
        closure: &mut PublicTypeFunctionClosure,
    ) {
        match expr {
            TypeFunctionResultExpr::Primitive { constraint, .. }
            | TypeFunctionResultExpr::Var { constraint, .. } => {
                Self::collect_result_constraint_closure(constraint, closure);
            }
            TypeFunctionResultExpr::NominalApp {
                origin,
                args,
                constraint,
                ..
            } => {
                closure.ordinary_types.insert(origin.clone());
                Self::collect_result_constraint_closure(constraint, closure);
                for arg in args {
                    self.collect_public_result_closure(arg, closure);
                }
            }
            TypeFunctionResultExpr::DomainConstructorApp {
                domain,
                args,
                constraint,
                ..
            } => {
                closure.sealed_domains.insert(domain.clone());
                Self::collect_result_constraint_closure(constraint, closure);
                for arg in args {
                    self.collect_public_result_closure(arg, closure);
                }
            }
            TypeFunctionResultExpr::PromotedDataConstructorApp {
                constructor,
                data_kind,
                args,
                constraint,
                ..
            } => {
                closure.promoted_data_kinds.insert((**data_kind).clone());
                closure
                    .promoted_constructors
                    .insert((**constructor).clone());
                closure.ordinary_types.insert(data_kind.source_type.clone());
                Self::collect_result_constraint_closure(constraint, closure);
                for arg in args {
                    self.collect_public_result_closure(arg, closure);
                }
            }
            TypeFunctionResultExpr::Projection {
                interface,
                member,
                args,
                constraint,
                ..
            } => {
                closure
                    .projections
                    .insert((interface.clone(), member.clone()));
                Self::collect_result_constraint_closure(constraint, closure);
                for arg in args {
                    self.collect_public_result_closure(arg, closure);
                }
            }
            TypeFunctionResultExpr::ComputationHeadApp {
                head,
                args,
                constraint,
                ..
            } => {
                self.collect_public_type_function_head_closure(head, closure);
                Self::collect_result_constraint_closure(constraint, closure);
                for arg in args {
                    self.collect_public_result_closure(arg, closure);
                }
            }
        }
    }

    pub(super) fn collect_result_constraint_closure(
        constraint: &TypeFunctionResultConstraint,
        closure: &mut PublicTypeFunctionClosure,
    ) {
        if let TypeFunctionResultConstraint::Domain(domain) = constraint {
            closure.sealed_domains.insert(domain.clone());
        }
    }

    pub(super) fn register_local_type_functions_inner(
        &mut self,
        module: &ModuleIdentity,
        defs: &[SurfaceTypeFnDef],
    ) -> Result<(), TypeEnvError> {
        let mut seen_in_batch = HashSet::new();
        for def in defs {
            let name = def.name.to_string();
            if self.local_type_function_heads.contains_key(&name)
                || !seen_in_batch.insert(name.clone())
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!("duplicate type function '{name}'"),
                    def.span,
                ));
            }
        }

        for (index, def) in defs.iter().enumerate() {
            let later_names: HashSet<String> = defs
                .iter()
                .skip(index + 1)
                .map(|later| later.name.to_string())
                .collect();
            let lowered = self.lower_local_type_function(module, def, &later_names)?;
            let obligation_start = self.proposition_obligations.len();
            self.local_type_function_heads
                .insert(lowered.name.clone(), lowered.head.clone());
            self.local_type_functions
                .insert(lowered.head.clone(), lowered);
            if let Some(tail) = &def.proposition_tail {
                self.add_proposition_obligations_from_tail(
                    tail,
                    SourceOrigin::Synthetic {
                        reason: format!(
                            "type function proposition checking point {}::{}",
                            module.path.join("::"),
                            def.name
                        ),
                    },
                    PropositionCheckingSite::new(
                        0x8800_0000u64 + index as u64,
                        PropositionCheckingSiteKind::ExplicitRequirement,
                        Some(format!("type fn {} proposition tail", def.name)),
                    ),
                )
                .map_err(proposition_revalidation_error)?;
                self.discharge_required_proposition_obligations_from(obligation_start)?;
            }
        }
        Ok(())
    }

    pub(super) fn lower_local_type_function(
        &self,
        module: &ModuleIdentity,
        def: &SurfaceTypeFnDef,
        later_names: &HashSet<String>,
    ) -> Result<TypeFunctionDef, TypeEnvError> {
        let head = TypeComputationHeadId::new(module.clone(), def.name.to_string());
        let params = def
            .params
            .iter()
            .map(|param| {
                let (ty, constraint) = self.lower_type_fn_signature_type(&param.ty)?;
                Ok(TypeFunctionParam {
                    name: param.name.to_string(),
                    ty,
                    kind: Kind::Type,
                    domain_constraint: constraint,
                    source_anchor: span_anchor(param.span, format!("type fn param {}", param.name)),
                })
            })
            .collect::<Result<Vec<_>, TypeEnvError>>()?;
        if !params.iter().any(|param| param.domain_constraint.is_some()) {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "type function '{}' has no sealed-domain scrutinee in its parameter list",
                    def.name
                ),
                def.header_span,
            ));
        }
        let (return_type, result_domain) = self.lower_type_fn_signature_type(&def.return_type)?;
        let result_constraint = match result_domain.clone() {
            Some(domain) => TypeFunctionResultConstraint::Domain(domain),
            None => TypeFunctionResultConstraint::Kind(Kind::Type),
        };

        let mut equations = Vec::with_capacity(def.equations.len());
        for (ordinal, equation) in def.equations.iter().enumerate() {
            if equation.head.as_ref() != def.name.as_ref() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "case head '{}' does not match type function '{}'",
                        equation.head, def.name
                    ),
                    equation.head_span,
                ));
            }
            if equation.patterns.len() != params.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "type function '{}' equation arity mismatch: expected {}, found {}",
                        def.name,
                        params.len(),
                        equation.patterns.len()
                    ),
                    equation.span,
                ));
            }
            let mut pattern_vars = HashMap::new();
            let patterns = equation
                .patterns
                .iter()
                .zip(&params)
                .map(|(pattern, param)| {
                    let constraint = constraint_for_param(param);
                    self.lower_type_function_pattern(pattern, &constraint, &mut pattern_vars)
                })
                .collect::<Result<Vec<_>, TypeEnvError>>()?;
            let result_context = TypeFunctionResultLoweringContext {
                pattern_vars: &pattern_vars,
                current_head: Some((&def.name, &head, &params, &result_constraint)),
                later_names,
            };
            let result = self.lower_type_function_result_expr(
                &equation.result,
                result_domain.as_ref(),
                &result_context,
                equation.result_span,
            )?;
            self.validate_type_function_result_constraint(
                &result,
                &result_constraint,
                equation.result_span,
            )?;
            equations.push(TypeFunctionEquation {
                head: head.clone(),
                ordinal,
                patterns,
                result,
                source_anchor: span_anchor(equation.span, format!("type fn equation {ordinal}")),
                case_head_anchor: span_anchor(
                    equation.head_span,
                    format!("case head {}", equation.head),
                ),
            });
        }

        self.validate_type_function_pattern_coverage(
            def.name.as_ref(),
            &params,
            &equations,
            def.header_span,
        )?;

        self.validate_type_function_structural_recursion(
            def.name.as_ref(),
            &head,
            &params,
            def.decreases
                .as_ref()
                .map(|decreases| decreases.param.as_ref()),
            &equations,
            def.header_span,
        )?;

        let lowered = TypeFunctionDef {
            visibility: core_visibility_from_surface(&def.visibility),
            head,
            name: def.name.to_string(),
            params,
            return_type,
            return_kind: Kind::Type,
            result_constraint,
            decreases: def
                .decreases
                .as_ref()
                .map(|decreases| decreases.param.to_string()),
            source_anchors: TypeFunctionSourceAnchors {
                definition: span_anchor(def.header_span, format!("type fn {}", def.name)),
                decreases: def.decreases.as_ref().map(|decreases| {
                    span_anchor(decreases.span, format!("decreases {}", decreases.param))
                }),
            },
            equations,
        };
        if lowered.visibility == ash_core::ast::Visibility::Public {
            self.validate_public_type_function_export_closure(&lowered, def.span)?;
        }
        Ok(lowered)
    }

    pub(super) fn validate_public_type_function_export_closure(
        &self,
        def: &TypeFunctionDef,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        for equation in &def.equations {
            for pattern in &equation.patterns {
                self.validate_public_type_function_pattern_export_closure(def, pattern, span)?;
            }
            self.validate_public_type_function_result_export_closure(def, &equation.result, span)?;
        }
        for param in &def.params {
            if let Some(domain) = &param.domain_constraint {
                self.ensure_public_type_function_domain_dependency(def, domain, span)?;
            }
            self.validate_public_canonical_type_dependency(def, &param.ty, span)?;
        }
        if let TypeFunctionResultConstraint::Domain(domain) = &def.result_constraint {
            self.ensure_public_type_function_domain_dependency(def, domain, span)?;
        }
        self.validate_public_canonical_type_dependency(def, &def.return_type, span)
    }

    pub(super) fn validate_public_type_function_pattern_export_closure(
        &self,
        def: &TypeFunctionDef,
        pattern: &TypeFunctionPattern,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match pattern {
            TypeFunctionPattern::DomainConstructor {
                constructor,
                domain,
                fields,
                ..
            } => {
                self.ensure_public_type_function_constructor_dependency(def, constructor, span)?;
                self.ensure_public_type_function_domain_dependency(def, domain, span)?;
                for field in fields {
                    self.validate_public_type_function_pattern_export_closure(def, field, span)?;
                }
                Ok(())
            }
            TypeFunctionPattern::Var { constraint, .. }
            | TypeFunctionPattern::Wildcard { constraint, .. } => {
                if let TypeFunctionPatternConstraint::Domain(domain) = constraint {
                    self.ensure_public_type_function_domain_dependency(def, domain, span)?;
                }
                Ok(())
            }
        }
    }

    pub(super) fn validate_public_type_function_result_export_closure(
        &self,
        def: &TypeFunctionDef,
        expr: &TypeFunctionResultExpr,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match expr {
            TypeFunctionResultExpr::Primitive { .. } => Ok(()),
            TypeFunctionResultExpr::Var { constraint, .. } => {
                if let TypeFunctionResultConstraint::Domain(domain) = constraint {
                    self.ensure_public_type_function_domain_dependency(def, domain, span)?;
                }
                Ok(())
            }
            TypeFunctionResultExpr::NominalApp {
                visible_name, args, ..
            } => {
                self.ensure_public_type_function_ordinary_type_dependency(def, visible_name, span)?;
                for arg in args {
                    self.validate_public_type_function_result_export_closure(def, arg, span)?;
                }
                Ok(())
            }
            TypeFunctionResultExpr::DomainConstructorApp {
                constructor,
                domain,
                args,
                ..
            } => {
                self.ensure_public_type_function_constructor_dependency(def, constructor, span)?;
                self.ensure_public_type_function_domain_dependency(def, domain, span)?;
                for arg in args {
                    self.validate_public_type_function_result_export_closure(def, arg, span)?;
                }
                Ok(())
            }
            TypeFunctionResultExpr::PromotedDataConstructorApp {
                constructor,
                data_kind,
                args,
                kind,
                ..
            } => {
                self.validate_registered_promoted_constructor_app(
                    constructor,
                    data_kind,
                    args.len(),
                    kind,
                    span,
                )?;
                self.ensure_public_type_function_promoted_constructor_dependency(
                    def,
                    constructor,
                    data_kind,
                    span,
                )?;
                for arg in args {
                    self.validate_public_type_function_result_export_closure(def, arg, span)?;
                }
                Ok(())
            }
            TypeFunctionResultExpr::Projection {
                interface,
                member,
                args,
                ..
            } => {
                self.ensure_public_type_function_projection_dependency(
                    def, interface, member, span,
                )?;
                for arg in args {
                    self.validate_public_type_function_result_export_closure(def, arg, span)?;
                }
                Ok(())
            }
            TypeFunctionResultExpr::ComputationHeadApp { head, args, .. } => {
                if head != &def.head {
                    let Some(callee) = self.local_type_functions.get(head) else {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "public type function '{}' export closure cannot resolve type function dependency '{}'",
                                def.name, head.name
                            ),
                            span,
                        ));
                    };
                    if callee.visibility != ash_core::ast::Visibility::Public {
                        return Err(TypeEnvError::PrivateDependencyExportFailure {
                            public_item: def.name.clone(),
                            dependency: callee.name.clone(),
                            dependency_kind: "type function".to_string(),
                            span,
                        });
                    }
                }
                for arg in args {
                    self.validate_public_type_function_result_export_closure(def, arg, span)?;
                }
                Ok(())
            }
        }
    }

    pub(super) fn validate_public_canonical_type_dependency(
        &self,
        def: &TypeFunctionDef,
        ty: &CanonicalTypeExpr,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match ty {
            CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::Var(_) => Ok(()),
            CanonicalTypeExpr::NominalApp {
                visible_name, args, ..
            } => {
                self.ensure_public_type_function_ordinary_type_dependency(def, visible_name, span)?;
                for arg in args {
                    self.validate_public_canonical_type_dependency(def, arg, span)?;
                }
                Ok(())
            }
            CanonicalTypeExpr::Projection {
                interface,
                member,
                args,
                ..
            } => {
                self.ensure_public_type_function_projection_dependency(
                    def, interface, member, span,
                )?;
                for arg in args {
                    self.validate_public_canonical_type_dependency(def, arg, span)?;
                }
                Ok(())
            }
            CanonicalTypeExpr::ComputationHeadApp { args, .. } => {
                for arg in args {
                    self.validate_public_canonical_type_dependency(def, arg, span)?;
                }
                Ok(())
            }
            CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
                self.validate_registered_promoted_constructor_app(
                    &app.constructor,
                    &app.data_kind,
                    app.args.len(),
                    &app.kind,
                    span,
                )?;
                self.ensure_public_type_function_promoted_constructor_dependency(
                    def,
                    &app.constructor,
                    &app.data_kind,
                    span,
                )?;
                for arg in &app.args {
                    self.validate_public_canonical_type_dependency(def, arg, span)?;
                }
                Ok(())
            }
            CanonicalTypeExpr::ConstructorVariableApp(app) => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' cannot export constructor-variable application '{}' until TASK-907 tracks constructor variables",
                    def.name, app.constructor.name
                ),
                span,
            )),
        }
    }

    pub(super) fn ensure_public_type_function_domain_dependency(
        &self,
        def: &TypeFunctionDef,
        domain: &SealedDomainId,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        let Some(summary) = self.lookup_sealed_domain_by_id(domain) else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' export closure cannot resolve sealed domain '{}'",
                    def.name, domain.name
                ),
                span,
            ));
        };
        if summary.visibility != ash_core::ast::Visibility::Public {
            return Err(TypeEnvError::PrivateDependencyExportFailure {
                public_item: def.name.clone(),
                dependency: summary.exported_name.clone(),
                dependency_kind: "sealed domain".to_string(),
                span: if anchor_span(&summary.anchor) == Span::default() {
                    span
                } else {
                    anchor_span(&summary.anchor)
                },
            });
        }
        Ok(())
    }

    pub(super) fn ensure_public_type_function_constructor_dependency(
        &self,
        def: &TypeFunctionDef,
        constructor: &DomainConstructorId,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        let Some(domain) = self.lookup_sealed_domain_by_id(&constructor.domain) else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' export closure cannot resolve marker constructor '{}'",
                    def.name, constructor.name
                ),
                span,
            ));
        };
        if domain.visibility != ash_core::ast::Visibility::Public {
            return Err(TypeEnvError::PrivateDependencyExportFailure {
                public_item: def.name.clone(),
                dependency: constructor.name.clone(),
                dependency_kind: "marker constructor".to_string(),
                span,
            });
        }
        Ok(())
    }

    pub(super) fn ensure_public_type_function_promoted_data_kind_dependency(
        &self,
        def: &TypeFunctionDef,
        data_kind: &PromotedDataKindId,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        let Some(summary) = self.lookup_promoted_data_kind_by_id(data_kind) else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' export closure cannot resolve promoted data kind '{}'",
                    def.name, data_kind.name
                ),
                span,
            ));
        };
        if summary.visibility != ash_core::ast::Visibility::Public {
            return Err(TypeEnvError::PrivateDependencyExportFailure {
                public_item: def.name.clone(),
                dependency: summary.exported_name.clone(),
                dependency_kind: "promoted data kind".to_string(),
                span: if anchor_span(&summary.source_anchor) == Span::default() {
                    span
                } else {
                    anchor_span(&summary.source_anchor)
                },
            });
        }
        self.ensure_public_type_function_ordinary_type_dependency(
            def,
            &data_kind.source_type.name,
            span,
        )?;
        Ok(())
    }

    pub(super) fn ensure_public_type_function_promoted_constructor_dependency(
        &self,
        def: &TypeFunctionDef,
        constructor: &PromotedConstructorId,
        data_kind: &PromotedDataKindId,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        self.ensure_public_type_function_promoted_data_kind_dependency(def, data_kind, span)?;
        let Some(summary) = self.promoted_constructor_summaries.get(constructor) else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' export closure cannot resolve promoted data constructor '{}'",
                    def.name, constructor.name
                ),
                span,
            ));
        };
        if constructor.kind != *data_kind {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' export closure references promoted constructor '{}' from promoted data kind '{}', not '{}'",
                    def.name, constructor.name, constructor.kind.name, data_kind.name
                ),
                span,
            ));
        }
        if summary.visibility != ash_core::ast::Visibility::Public {
            return Err(TypeEnvError::PrivateDependencyExportFailure {
                public_item: def.name.clone(),
                dependency: summary.exported_name.clone(),
                dependency_kind: "promoted data constructor".to_string(),
                span: if anchor_span(&summary.source_anchor) == Span::default() {
                    span
                } else {
                    anchor_span(&summary.source_anchor)
                },
            });
        }
        Ok(())
    }

    pub(super) fn ensure_public_type_function_projection_dependency(
        &self,
        def: &TypeFunctionDef,
        interface: &InterfaceIdentityId,
        member: &AssociatedMemberIdentityId,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        if !self.known_interface_identities.contains(interface) {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' export closure cannot resolve projection interface '{}'",
                    def.name, interface.name
                ),
                span,
            ));
        }
        if !self.known_associated_member_identities.contains(member)
            || member.interface != *interface
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' export closure cannot resolve projection member '{}::{}'",
                    def.name, interface.name, member.name
                ),
                span,
            ));
        }
        if let Some(info) = self.interfaces.get(interface.name.as_str())
            && info.visibility != ash_core::ast::Visibility::Public
        {
            return Err(TypeEnvError::PrivateDependencyExportFailure {
                public_item: def.name.clone(),
                dependency: format!("{}::{}", interface.name, member.name),
                dependency_kind: "projection".to_string(),
                span,
            });
        }
        Ok(())
    }

    pub(super) fn ensure_public_type_function_ordinary_type_dependency(
        &self,
        def: &TypeFunctionDef,
        visible_name: &str,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        if let Some(type_def) = self.ast_types.get(visible_name)
            && type_def.visibility != ash_core::ast::Visibility::Public
        {
            return Err(TypeEnvError::PrivateDependencyExportFailure {
                public_item: def.name.clone(),
                dependency: visible_name.to_string(),
                dependency_kind: "ordinary type".to_string(),
                span,
            });
        }
        Ok(())
    }

    pub(super) fn lower_type_fn_signature_type(
        &self,
        ty: &SurfaceType,
    ) -> Result<(CanonicalTypeExpr, Option<SealedDomainId>), TypeEnvError> {
        if let SurfaceType::Name(name) = ty {
            if name.as_ref() == "Type" {
                return Ok((CanonicalTypeExpr::Var("Type".to_string()), None));
            }
            if let Some(domain) = self.lookup_sealed_domain(name.as_ref()) {
                return Ok((
                    CanonicalTypeExpr::Var(domain.exported_name.clone()),
                    Some(domain.id.clone()),
                ));
            }
        }
        let canonical = self.lower_surface_type_to_canonical(ty).map_err(|err| {
            let spelling =
                surface_type_name(ty).unwrap_or_else(|| surface_projection_base_spelling(ty));
            TypeEnvError::InvalidDefinition(
                format!("unresolved type in type-function signature '{spelling}': {err}"),
                Span::default(),
            )
        })?;
        if matches!(canonical, CanonicalTypeExpr::Var(_)) {
            let spelling =
                surface_type_name(ty).unwrap_or_else(|| surface_projection_base_spelling(ty));
            return Err(TypeEnvError::InvalidDefinition(
                format!("unresolved type in type-function signature '{spelling}'"),
                Span::default(),
            ));
        }
        Ok((canonical, None))
    }

    pub(super) fn validate_type_function_pattern_coverage(
        &self,
        name: &str,
        params: &[TypeFunctionParam],
        equations: &[TypeFunctionEquation],
        span: Span,
    ) -> Result<(), TypeEnvError> {
        let sealed_positions = params
            .iter()
            .enumerate()
            .filter_map(|(index, param)| {
                param
                    .domain_constraint
                    .clone()
                    .map(|domain| (index, domain))
            })
            .collect::<Vec<_>>();
        if sealed_positions.is_empty() {
            return Ok(());
        }

        let spaces = sealed_positions
            .iter()
            .map(|(param_index, domain)| {
                self.coverage_space_for_domain(
                    domain,
                    equations
                        .iter()
                        .filter_map(|equation| equation.patterns.get(*param_index)),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let universe = Self::coverage_tuple_universe(&spaces);
        let mut covered = HashSet::new();
        let mut covered_by_default = HashSet::new();

        for equation in equations {
            let row_patterns = sealed_positions
                .iter()
                .map(|(index, _)| &equation.patterns[*index])
                .collect::<Vec<_>>();
            let row_space = universe
                .iter()
                .filter(|tuple| {
                    tuple.iter().zip(&row_patterns).all(|(value, pattern)| {
                        Self::coverage_value_matches_pattern(value, pattern)
                    })
                })
                .cloned()
                .collect::<HashSet<_>>();
            let residual = row_space
                .difference(&covered)
                .cloned()
                .collect::<HashSet<_>>();
            let has_default = row_patterns
                .iter()
                .any(|pattern| Self::pattern_has_domain_default(pattern));
            let is_all_default = row_patterns
                .iter()
                .all(|pattern| Self::pattern_is_all_domain_default(pattern));
            if residual.is_empty() {
                let message = if has_default && is_all_default {
                    format!(
                        "empty residual default in type function '{name}' equation {}",
                        equation.ordinal
                    )
                } else if row_space
                    .iter()
                    .any(|value| covered_by_default.contains(value))
                {
                    format!(
                        "unreachable type function equation {} in '{name}' after earlier default",
                        equation.ordinal
                    )
                } else {
                    format!(
                        "overlapping type function equation {} in '{name}'",
                        equation.ordinal
                    )
                };
                return Err(TypeEnvError::InvalidDefinition(message, span));
            }
            if has_default {
                covered_by_default.extend(residual.iter().cloned());
            }
            covered.extend(residual);
        }

        if covered.len() != universe.len() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "non-exhaustive type function '{name}': uncovered closed constructor tuple(s)"
                ),
                span,
            ));
        }
        Ok(())
    }

    pub(super) fn validate_type_function_structural_recursion(
        &self,
        name: &str,
        head: &TypeComputationHeadId,
        params: &[TypeFunctionParam],
        decreases: Option<&str>,
        equations: &[TypeFunctionEquation],
        span: Span,
    ) -> Result<(), TypeEnvError> {
        let recursive = equations
            .iter()
            .any(|equation| Self::result_contains_computation_head(&equation.result, head));

        let Some(decreases) = decreases else {
            if recursive {
                return Err(TypeEnvError::InvalidDefinition(
                    format!("missing decreases clause for recursive type function '{name}'"),
                    span,
                ));
            }
            return Ok(());
        };

        let Some(decreasing_index) = params.iter().position(|param| param.name == decreases) else {
            return Err(TypeEnvError::InvalidDefinition(
                format!("unknown decreases parameter '{decreases}' in type function '{name}'"),
                span,
            ));
        };

        let Some(decreasing_domain) = params[decreasing_index].domain_constraint.as_ref() else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "invalid decreases parameter '{decreases}' in type function '{name}': parameter is not a sealed domain"
                ),
                span,
            ));
        };

        if !self.domain_has_structural_subcomponent_metadata(decreasing_domain)? {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "invalid decreases parameter '{decreases}' in type function '{name}': sealed domain has no structural subcomponent metadata"
                ),
                span,
            ));
        }

        for equation in equations {
            let allowed_subcomponents = equation
                .patterns
                .get(decreasing_index)
                .map(|pattern| self.direct_structural_subcomponent_vars(pattern))
                .transpose()?
                .unwrap_or_default();
            self.validate_recursive_calls_in_result(
                name,
                head,
                decreasing_index,
                &allowed_subcomponents,
                &equation.result,
                span,
            )?;
        }

        Ok(())
    }

    pub(super) fn domain_has_structural_subcomponent_metadata(
        &self,
        domain: &SealedDomainId,
    ) -> Result<bool, TypeEnvError> {
        let summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
            TypeEnvError::InvalidDefinition(
                format!(
                    "unknown sealed domain '{}' in decreases clause",
                    domain.name
                ),
                Span::default(),
            )
        })?;
        Ok(summary.constructors.iter().any(|constructor| {
            constructor
                .fields
                .iter()
                .any(|field| field.structural_status == StructuralFieldStatus::StructuralSelfDomain)
        }))
    }

    pub(super) fn direct_structural_subcomponent_vars(
        &self,
        pattern: &TypeFunctionPattern,
    ) -> Result<HashSet<String>, TypeEnvError> {
        let TypeFunctionPattern::DomainConstructor {
            constructor,
            domain,
            fields,
            ..
        } = pattern
        else {
            return Ok(HashSet::new());
        };
        let summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
            TypeEnvError::InvalidDefinition(
                format!(
                    "unknown sealed domain '{}' in recursion matrix",
                    domain.name
                ),
                Span::default(),
            )
        })?;
        let Some(constructor_summary) = summary
            .constructors
            .iter()
            .find(|candidate| candidate.id == **constructor)
        else {
            return Ok(HashSet::new());
        };

        let mut vars = HashSet::new();
        for (field_pattern, field) in fields.iter().zip(&constructor_summary.fields) {
            if field.structural_status != StructuralFieldStatus::StructuralSelfDomain {
                continue;
            }
            if let TypeFunctionPattern::Var { name, .. } = field_pattern {
                vars.insert(name.clone());
            }
        }
        Ok(vars)
    }

    pub(super) fn validate_recursive_calls_in_result(
        &self,
        function_name: &str,
        self_head: &TypeComputationHeadId,
        decreasing_index: usize,
        allowed_subcomponents: &HashSet<String>,
        expr: &TypeFunctionResultExpr,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match expr {
            TypeFunctionResultExpr::Primitive { .. } | TypeFunctionResultExpr::Var { .. } => Ok(()),
            TypeFunctionResultExpr::NominalApp { args, .. }
            | TypeFunctionResultExpr::DomainConstructorApp { args, .. }
            | TypeFunctionResultExpr::PromotedDataConstructorApp { args, .. }
            | TypeFunctionResultExpr::Projection { args, .. } => {
                for arg in args {
                    self.validate_recursive_calls_in_result(
                        function_name,
                        self_head,
                        decreasing_index,
                        allowed_subcomponents,
                        arg,
                        span,
                    )?;
                }
                Ok(())
            }
            TypeFunctionResultExpr::ComputationHeadApp { head, args, .. } => {
                for arg in args {
                    self.validate_recursive_calls_in_result(
                        function_name,
                        self_head,
                        decreasing_index,
                        allowed_subcomponents,
                        arg,
                        span,
                    )?;
                }
                if head == self_head {
                    let Some(decreasing_arg) = args.get(decreasing_index) else {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "non-decreasing recursive call in type function '{function_name}': missing decreasing argument"
                            ),
                            span,
                        ));
                    };
                    match decreasing_arg {
                        TypeFunctionResultExpr::Var { name, .. }
                            if allowed_subcomponents.contains(name) =>
                        {
                            Ok(())
                        }
                        _ => Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "non-decreasing recursive call in type function '{function_name}': decreasing argument must be a direct structural subcomponent"
                            ),
                            span,
                        )),
                    }
                } else {
                    Ok(())
                }
            }
        }
    }

    pub(super) fn result_contains_computation_head(
        expr: &TypeFunctionResultExpr,
        needle: &TypeComputationHeadId,
    ) -> bool {
        match expr {
            TypeFunctionResultExpr::Primitive { .. } | TypeFunctionResultExpr::Var { .. } => false,
            TypeFunctionResultExpr::NominalApp { args, .. }
            | TypeFunctionResultExpr::DomainConstructorApp { args, .. }
            | TypeFunctionResultExpr::PromotedDataConstructorApp { args, .. }
            | TypeFunctionResultExpr::Projection { args, .. } => args
                .iter()
                .any(|arg| Self::result_contains_computation_head(arg, needle)),
            TypeFunctionResultExpr::ComputationHeadApp { head, args, .. } => {
                head == needle
                    || args
                        .iter()
                        .any(|arg| Self::result_contains_computation_head(arg, needle))
            }
        }
    }

    pub(super) fn coverage_space_for_domain<'a>(
        &self,
        domain: &SealedDomainId,
        patterns: impl Iterator<Item = &'a TypeFunctionPattern>,
    ) -> Result<TypeFunctionCoverageSpace, TypeEnvError> {
        let summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
            TypeEnvError::InvalidDefinition(
                format!("unknown sealed domain '{}' in coverage matrix", domain.name),
                Span::default(),
            )
        })?;
        let mut inspected: HashMap<(DomainConstructorId, usize), Vec<&TypeFunctionPattern>> =
            HashMap::new();
        for pattern in patterns {
            self.collect_coverage_inspections(pattern, &mut inspected)?;
        }
        let mut alts = Vec::with_capacity(summary.constructors.len());
        for constructor in &summary.constructors {
            let mut fields = Vec::with_capacity(constructor.fields.len());
            for (field_index, field) in constructor.fields.iter().enumerate() {
                if let Some(nested_patterns) = inspected.get(&(constructor.id.clone(), field_index))
                {
                    let nested_domain = field.domain_constraint.clone().ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "nested constructor pattern under '{}' field '{}' requires a sealed-domain field",
                                constructor.exported_name, field.name
                            ),
                            Span::default(),
                        )
                    })?;
                    fields.push(Some(self.coverage_space_for_domain(
                        &nested_domain,
                        nested_patterns.iter().copied(),
                    )?));
                } else {
                    fields.push(None);
                }
            }
            alts.push(TypeFunctionCoverageAlt {
                constructor: constructor.id.clone(),
                fields,
            });
        }
        Ok(TypeFunctionCoverageSpace {
            domain: domain.clone(),
            alts,
        })
    }

    pub(super) fn collect_coverage_inspections<'a>(
        &self,
        pattern: &'a TypeFunctionPattern,
        inspected: &mut HashMap<(DomainConstructorId, usize), Vec<&'a TypeFunctionPattern>>,
    ) -> Result<(), TypeEnvError> {
        let TypeFunctionPattern::DomainConstructor {
            constructor,
            domain,
            fields,
            ..
        } = pattern
        else {
            return Ok(());
        };
        let summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
            TypeEnvError::InvalidDefinition(
                format!("unknown sealed domain '{}' in coverage matrix", domain.name),
                Span::default(),
            )
        })?;
        let Some(constructor_summary) = summary
            .constructors
            .iter()
            .find(|candidate| candidate.id == **constructor)
        else {
            return Ok(());
        };
        for (field_index, field_pattern) in fields.iter().enumerate() {
            if matches!(field_pattern, TypeFunctionPattern::DomainConstructor { .. }) {
                inspected
                    .entry(((**constructor).clone(), field_index))
                    .or_default()
                    .push(field_pattern);
                let Some(field) = constructor_summary.fields.get(field_index) else {
                    continue;
                };
                if field.domain_constraint.is_none() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "nested constructor pattern under '{}' field '{}' requires a sealed-domain field",
                            constructor_summary.exported_name, field.name
                        ),
                        Span::default(),
                    ));
                }
                self.collect_coverage_inspections(field_pattern, inspected)?;
            }
        }
        Ok(())
    }

    pub(super) fn coverage_tuple_universe(
        spaces: &[TypeFunctionCoverageSpace],
    ) -> HashSet<Vec<TypeFunctionCoverageValue>> {
        let mut tuples = vec![Vec::new()];
        for values in spaces.iter().map(Self::coverage_values_for_space) {
            let mut next = Vec::new();
            for prefix in &tuples {
                for value in &values {
                    let mut tuple = prefix.clone();
                    tuple.push(value.clone());
                    next.push(tuple);
                }
            }
            tuples = next;
        }
        tuples.into_iter().collect()
    }

    pub(super) fn coverage_values_for_space(
        space: &TypeFunctionCoverageSpace,
    ) -> Vec<TypeFunctionCoverageValue> {
        let _ = &space.domain;
        let mut values = Vec::new();
        for alt in &space.alts {
            let mut field_values = vec![Vec::new()];
            for field_space in &alt.fields {
                if let Some(field_space) = field_space {
                    let nested_values = Self::coverage_values_for_space(field_space);
                    let mut next = Vec::new();
                    for prefix in &field_values {
                        for nested in &nested_values {
                            let mut fields = prefix.clone();
                            fields.push(Some(nested.clone()));
                            next.push(fields);
                        }
                    }
                    field_values = next;
                } else {
                    for prefix in &mut field_values {
                        prefix.push(None);
                    }
                }
            }
            values.extend(
                field_values
                    .into_iter()
                    .map(|fields| TypeFunctionCoverageValue {
                        constructor: alt.constructor.clone(),
                        fields,
                    }),
            );
        }
        values
    }

    pub(super) fn coverage_value_matches_pattern(
        value: &TypeFunctionCoverageValue,
        pattern: &TypeFunctionPattern,
    ) -> bool {
        match pattern {
            TypeFunctionPattern::Wildcard { .. } | TypeFunctionPattern::Var { .. } => true,
            TypeFunctionPattern::DomainConstructor {
                constructor,
                fields,
                ..
            } => {
                constructor.as_ref() == &value.constructor
                    && fields.iter().enumerate().all(|(index, field_pattern)| {
                        match value.fields.get(index).and_then(Option::as_ref) {
                            Some(nested) => {
                                Self::coverage_value_matches_pattern(nested, field_pattern)
                            }
                            None => !matches!(
                                field_pattern,
                                TypeFunctionPattern::DomainConstructor { .. }
                            ),
                        }
                    })
            }
        }
    }

    pub(super) fn pattern_has_domain_default(pattern: &TypeFunctionPattern) -> bool {
        match pattern {
            TypeFunctionPattern::Wildcard { constraint, .. }
            | TypeFunctionPattern::Var { constraint, .. } => {
                matches!(constraint, TypeFunctionPatternConstraint::Domain(_))
            }
            TypeFunctionPattern::DomainConstructor { fields, .. } => {
                fields.iter().any(Self::pattern_has_domain_default)
            }
        }
    }

    pub(super) fn pattern_is_all_domain_default(pattern: &TypeFunctionPattern) -> bool {
        matches!(
            pattern,
            TypeFunctionPattern::Wildcard {
                constraint: TypeFunctionPatternConstraint::Domain(_),
                ..
            } | TypeFunctionPattern::Var {
                constraint: TypeFunctionPatternConstraint::Domain(_),
                ..
            }
        )
    }
}
