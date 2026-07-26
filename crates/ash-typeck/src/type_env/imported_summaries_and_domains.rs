use super::*;

/// Compare the public effect-row binding contract, deliberately excluding the
/// enclosing facade's summary identity and diagnostic anchor.  Those fields
/// describe where a facade re-export was published, not the immutable
/// provider or the sanitized row contract a caller receives.
fn effect_row_binding_contract_matches(
    left: &EffectRowExportSummary,
    right: &EffectRowExportSummary,
) -> bool {
    left.exported_name == right.exported_name
        && left.binding.visible_name == right.binding.visible_name
        && left.provider == right.provider
        && left.binding.provider == right.binding.provider
        && left.binding.exposure == right.binding.exposure
        && left.binding.closure_status == right.binding.closure_status
        && left.visibility == right.visibility
        && left.classification == right.classification
        && left.authority == right.authority
        && left.row_items == right.row_items
        && left.closure_metadata == right.closure_metadata
}

impl TypeEnv {
    /// Batch-register imported semantic summaries.
    ///
    /// The batch path declares all imported identities and public computation
    /// heads before equation revalidation so cross-summary public reductions and
    /// dependency-closure helper heads are normalizer-available atomically.
    pub fn register_module_semantic_summaries(
        &mut self,
        summaries: &[ModuleSemanticSummary],
    ) -> Result<(), TypeEnvError> {
        for summary in summaries {
            summary
                .validate_summary_version_contract()
                .map_err(summary_version_contract_error)?;
        }

        let mut staged = self.clone();
        for summary in summaries {
            validate_summary_visibility_and_duplicates(summary)?;
        }
        for summary in summaries {
            for row in &summary.exported_effect_rows {
                staged.register_imported_effect_row_export(row)?;
            }
            for value in &summary.exported_values {
                staged.register_imported_value_export(value)?;
            }
            for ty in &summary.exported_types {
                if let Some(constructor) =
                    imported_nominal_newtype_constructor(ty, &summary.exported_constructors)?
                {
                    staged.declare_imported_nominal_newtype(
                        ty,
                        constructor,
                        ty.visibility == ash_core::ast::Visibility::Public,
                    )?;
                } else {
                    staged.declare_summary_type_identity(ty)?;
                }
            }
            for interface in &summary.interface_identities {
                staged.register_interface_identity_summary_imported(interface)?;
            }
            for member in &summary.associated_member_identities {
                staged.register_associated_member_identity_summary_imported(member)?;
            }
            for domain in &summary.exported_sealed_domains {
                staged.declare_sealed_domain_identity(domain)?;
            }
            for data_kind in &summary.exported_promoted_data_kinds {
                staged.declare_promoted_data_kind_identity(data_kind)?;
            }
        }
        for summary in summaries {
            for type_fn in &summary.exported_type_functions {
                staged.declare_imported_type_function_summary(type_fn)?;
            }
        }
        let hidden_associated_family_heads = hidden_imported_associated_family_heads(summaries);
        for summary in summaries {
            for family in &summary.exported_associated_families {
                staged.declare_imported_associated_family_summary(
                    family,
                    !hidden_associated_family_heads.contains(&family.head),
                )?;
            }
            for predicate in &summary.exported_proposition_predicates {
                staged.register_proposition_predicate_summary(predicate)?;
            }
        }
        for summary in summaries {
            staged.register_module_semantic_summary_representations_and_domains(summary)?;
        }
        for summary in summaries {
            for data_kind in &summary.exported_promoted_data_kinds {
                staged.validate_and_register_promoted_data_kind(data_kind)?;
            }
        }
        for summary in summaries {
            for type_fn in &summary.exported_type_functions {
                staged.validate_imported_type_function_summary(type_fn)?;
            }
        }
        for summary in summaries {
            for family in &summary.exported_associated_families {
                staged.validate_and_register_imported_associated_family_summary(family)?;
            }
            for fact in &summary.exported_proposition_facts {
                staged.validate_and_register_imported_proposition_fact(summary, fact)?;
            }
        }
        *self = staged;
        Ok(())
    }

    fn register_imported_effect_row_export(
        &mut self,
        row: &EffectRowExportSummary,
    ) -> Result<(), TypeEnvError> {
        let name = row.exported_name.to_string();
        match self.imported_effect_rows.get(&name) {
            Some(existing) if effect_row_binding_contract_matches(existing, row) => Ok(()),
            Some(_) => Err(TypeEnvError::ImportOrderConflict {
                family: "effect-row visible binding".to_string(),
                name,
                span: anchor_span(&row.source_anchor),
            }),
            None => {
                self.imported_effect_rows.insert(name, row.clone());
                Ok(())
            }
        }
    }

    fn register_imported_value_export(
        &mut self,
        value: &ValueExportSummary,
    ) -> Result<(), TypeEnvError> {
        let name = value.exported_name.to_string();
        let incoming = match value.kind {
            ValueExportKind::Handler => CallableDeclarationKind::Handler,
        };
        match self.callable_declarations.get(&name) {
            Some(existing) if *existing == incoming => Ok(()),
            Some(_) => Err(TypeEnvError::ImportOrderConflict {
                family: "value export".to_string(),
                name,
                span: anchor_span(&value.source_anchor),
            }),
            None => {
                self.callable_declarations.insert(name, incoming);
                Ok(())
            }
        }
    }

    /// Batch-register imported semantic summaries and atomically discharge all
    /// required proposition facts they introduce.
    pub fn register_module_semantic_summaries_and_discharge_required_propositions(
        &mut self,
        summaries: &[ModuleSemanticSummary],
    ) -> Result<Vec<PropositionOutcome>, TypeEnvError> {
        let mut staged = self.clone();
        staged.register_module_semantic_summaries(summaries)?;
        let outcomes = staged.discharge_required_proposition_obligations()?;
        *self = staged;
        Ok(outcomes)
    }

    pub(super) fn validate_and_register_imported_proposition_fact(
        &mut self,
        summary: &ModuleSemanticSummary,
        fact: &PropositionFactSummary,
    ) -> Result<(), TypeEnvError> {
        let predicate_dependencies = self.validate_public_proposition_dependencies(
            "imported proposition summary fact",
            &fact.proposition,
            anchor_span(&fact.source_anchor),
        )?;
        for dependency in &fact.predicate_dependencies {
            let Some(info) = self.proposition_predicate_by_id(dependency) else {
                return Err(TypeEnvError::UnknownPropositionPredicate {
                    name: dependency.name.to_string(),
                    span: anchor_span(&fact.source_anchor),
                });
            };
            if info.summary.visibility != ash_core::ast::Visibility::Public {
                return Err(private_proposition_dependency_error(
                    "imported proposition summary fact",
                    "proposition predicate",
                    info.summary.exported_name.as_ref(),
                    anchor_span(&fact.source_anchor),
                ));
            }
        }
        for dependency in &predicate_dependencies {
            if !fact.predicate_dependencies.contains(dependency) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "imported proposition summary fact omits predicate dependency '{}' from dependency metadata",
                        dependency.name
                    ),
                    anchor_span(&fact.source_anchor),
                ));
            }
        }

        let outcome = Some(
            self.solve_proposition(&fact.proposition, Some(fact.source_anchor.clone()))
                .map_err(proposition_revalidation_error)?,
        );
        self.push_proposition_fact(
            fact.role,
            fact.proposition.clone(),
            fact.source_anchor.clone(),
            PropositionCheckingSite::new(
                0x8790_0000u64 + self.proposition_obligations.len() as u64,
                PropositionCheckingSiteKind::Synthetic,
                Some(format!(
                    "imported proposition fact from {}",
                    summary.module.path.join("::")
                )),
            ),
            outcome,
        );
        Ok(())
    }

    pub(super) fn register_module_semantic_summary_representations_and_domains(
        &mut self,
        summary: &ModuleSemanticSummary,
    ) -> Result<(), TypeEnvError> {
        for ty in &summary.exported_types {
            if imported_nominal_newtype_constructor(ty, &summary.exported_constructors)?.is_some() {
                continue;
            }
            if ty.representation_exposure != RepresentationExposure::Exposed {
                continue;
            }
            let TypeRepresentationSummary::Exposed(body) = &ty.representation else {
                continue;
            };
            let def = TypeDef {
                name: ty.exported_name.clone(),
                params: ty.params.clone(),
                body: body.clone(),
                visibility: ty.visibility,
                builtin: false,
            };
            let type_info = convert_type_def(&def, self).map_err(|e| {
                TypeEnvError::InvalidDefinition(
                    format!("type '{}': {e}", def.name),
                    Span::default(),
                )
            })?;
            self.ast_types.insert(def.name.clone(), def.clone());
            self.type_info.insert(def.name.clone(), type_info);
            self.type_declaration_states
                .insert(def.name.clone(), TypeDeclarationState::Full);
            self.expose_summary_type_representation(ty, &summary.exported_constructors)?;
        }

        for domain in &summary.exported_sealed_domains {
            self.validate_and_register_sealed_domain(domain)?;
        }
        Ok(())
    }

    pub(super) fn declare_imported_type_function_summary(
        &mut self,
        summary: &TypeFunctionSummary,
    ) -> Result<(), TypeEnvError> {
        if summary.visibility != ash_core::ast::Visibility::Public {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "non-public type-function summary '{}' is not valid public metadata",
                    summary.exported_name
                ),
                Span::default(),
            ));
        }
        if let Some(existing) = self.local_type_functions.get(&summary.head) {
            let incoming = imported_type_function_def(summary);
            if existing != &incoming {
                return Err(TypeEnvError::ImportOrderConflict {
                    family: "type-function summary".to_string(),
                    name: summary.exported_name.clone(),
                    span: summary.equations.first().map_or_else(
                        || anchor_span(&summary.source_anchors.definition),
                        |equation| anchor_span(&equation.source_anchor),
                    ),
                });
            }
            return Ok(());
        }
        self.local_type_functions
            .insert(summary.head.clone(), imported_type_function_def(summary));
        Ok(())
    }

    pub(super) fn validate_imported_type_function_summary(
        &self,
        summary: &TypeFunctionSummary,
    ) -> Result<(), TypeEnvError> {
        if summary.export_mode != TypeFunctionExportMode::TransparentEquations {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "type-function summary '{}' has unsupported export mode",
                    summary.exported_name
                ),
                Span::default(),
            ));
        }
        if summary.revalidation_metadata.spec_version != SummaryVersion::SPEC062_TYPE_COMPUTATION_V3
            || !summary.revalidation_metadata.structural_recursion_checked
            || !summary.revalidation_metadata.kind_and_domain_checked
            || !summary.revalidation_metadata.coverage_and_overlap_checked
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "type-function summary '{}' lacks required SPEC-062 revalidation metadata",
                    summary.exported_name
                ),
                Span::default(),
            ));
        }
        let def = self
            .local_type_functions
            .get(&summary.head)
            .ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!(
                        "type-function summary '{}' head was not declared before validation",
                        summary.exported_name
                    ),
                    Span::default(),
                )
            })?;
        for param in &def.params {
            if param.kind != Kind::Type {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "type-function summary '{}' parameter '{}' has non-Type kind",
                        def.name, param.name
                    ),
                    Span::default(),
                ));
            }
            self.validate_imported_type_function_signature_type(&def.name, &param.ty, "parameter")?;
            if let Some(domain) = &param.domain_constraint
                && self.lookup_sealed_domain_by_id(domain).is_none()
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "type-function summary '{}' parameter '{}' references unknown sealed domain '{}'",
                        def.name, param.name, domain.name
                    ),
                    Span::default(),
                ));
            }
        }
        if def.return_kind != Kind::Type {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "type-function summary '{}' return has non-Type kind",
                    def.name
                ),
                Span::default(),
            ));
        }
        self.validate_imported_type_function_signature_type(&def.name, &def.return_type, "return")?;
        for equation in &def.equations {
            if equation.head != def.head || equation.patterns.len() != def.params.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "type-function summary '{}' equation arity or head mismatch",
                        def.name
                    ),
                    Span::default(),
                ));
            }
            let mut vars = HashMap::new();
            for (pattern, param) in equation.patterns.iter().zip(&def.params) {
                self.validate_imported_type_function_pattern(
                    pattern,
                    &constraint_for_param(param),
                    &mut vars,
                )?;
            }
            let actual = self.validate_imported_type_function_result(&equation.result, &vars)?;
            self.validate_imported_result_constraint_value(
                &actual,
                &def.result_constraint,
                Span::default(),
            )?;
        }
        self.validate_type_function_pattern_coverage(
            &def.name,
            &def.params,
            &def.equations,
            Span::default(),
        )?;
        self.validate_type_function_structural_recursion(
            &def.name,
            &def.head,
            &def.params,
            def.decreases.as_deref(),
            &def.equations,
            Span::default(),
        )?;
        self.validate_public_type_function_export_closure(def, Span::default())
    }

    pub(super) fn associated_family_result_constraint_from_summary(
        &self,
        family: &AssociatedFamilySummary,
    ) -> Result<AssociatedFamilyResultConstraint, TypeEnvError> {
        match &family.result_domain {
            CanonicalTypeExpr::Primitive(name) if name == "Type" => {
                Ok(AssociatedFamilyResultConstraint::Kind(Kind::Type))
            }
            CanonicalTypeExpr::Var(name) => self
                .sealed_domain_summaries
                .values()
                .find(|domain| domain.id.name == *name || domain.exported_name == *name)
                .map(|domain| AssociatedFamilyResultConstraint::Domain(domain.id.clone()))
                .ok_or_else(|| TypeEnvError::WrongAssociatedFamilyResultDomain {
                    family: family.visible_name.clone(),
                    reason: format!("unknown associated-family result domain '{name}'"),
                    span: anchor_span(&family.source_anchor),
                }),
            other => Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: family.visible_name.clone(),
                reason: format!("unsupported associated-family result domain {other:?}"),
                span: anchor_span(&family.source_anchor),
            }),
        }
    }

    pub(super) fn declare_imported_associated_family_summary(
        &mut self,
        family: &AssociatedFamilySummary,
        source_visible: bool,
    ) -> Result<(), TypeEnvError> {
        if family.head.interface != family.interface_identity
            || family.head.member != family.member_identity
            || family.member_identity.interface != family.interface_identity
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' has inconsistent interface/member identities",
                    family.visible_name
                ),
                anchor_span(&family.source_anchor),
            ));
        }
        let first_scheme = family.schemes.first().ok_or_else(|| {
            TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' must contain at least one scheme",
                    family.visible_name
                ),
                anchor_span(&family.source_anchor),
            )
        })?;
        let result_domain = self.associated_family_result_constraint_from_summary(family)?;
        let interface_params = first_scheme
            .params
            .iter()
            .map(|param| AssociatedFamilyInterfaceParamInfo {
                name: param.name.clone(),
                domain_constraint: param.domain_constraint.clone(),
            })
            .collect::<Vec<_>>();

        self.known_interface_identities
            .insert(family.interface_identity.clone());
        self.canonical_interface_names.insert(
            family.interface_identity.clone(),
            family.interface_identity.name.to_string(),
        );
        self.local_interface_arities
            .entry(family.interface_identity.clone())
            .or_insert(interface_params.len());
        self.known_associated_member_identities
            .insert(family.member_identity.clone());

        let declaration = AssociatedFamilyDeclarationInfo {
            defining_module: family.interface_identity.module.clone(),
            result_domain,
            decreases: family
                .revalidation_metadata
                .decreases
                .first()
                .map(|decreases| decreases.parameter.clone()),
            interface_params,
            head: family.head.clone(),
        };
        if let Some(existing) = self.associated_family_declarations.get(&family.head) {
            if existing != &declaration {
                return Err(TypeEnvError::ImportOrderConflict {
                    family: "associated-family summary".to_string(),
                    name: family.visible_name.clone(),
                    span: anchor_span(&family.source_anchor),
                });
            }
        } else {
            self.associated_family_declarations
                .insert(family.head.clone(), declaration);
        }

        if source_visible {
            self.interface_identity_aliases.insert(
                family.interface_identity.name.to_string(),
                family.interface_identity.clone(),
            );
            self.interface_identity_alias_is_imported
                .insert(family.interface_identity.name.to_string(), true);
            self.associated_member_identity_aliases.insert(
                (
                    family.interface_identity.name.to_string(),
                    family.visible_name.clone(),
                ),
                family.member_identity.clone(),
            );
            self.associated_member_identity_alias_is_imported.insert(
                (
                    family.interface_identity.name.to_string(),
                    family.visible_name.clone(),
                ),
                true,
            );
            self.associated_family_name_index.insert(
                (
                    family.interface_identity.name.to_string(),
                    family.visible_name.clone(),
                ),
                family.head.clone(),
            );
        }
        Ok(())
    }

    pub(super) fn validate_and_register_imported_associated_family_summary(
        &mut self,
        family: &AssociatedFamilySummary,
    ) -> Result<(), TypeEnvError> {
        if family.export_mode != AssociatedFamilyExportMode::TransparentEquations
            || family.revalidation_metadata.spec_version
                != SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4
            || !family.revalidation_metadata.kind_and_domain_checked
            || !family.revalidation_metadata.coverage_and_overlap_checked
            || !family.revalidation_metadata.coherence_checked
            || !family.revalidation_metadata.recursion_checked
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' lacks required SPEC-063 revalidation metadata",
                    family.visible_name
                ),
                anchor_span(&family.source_anchor),
            ));
        }
        if family.result_kind != Kind::Type {
            return Err(TypeEnvError::WrongAssociatedFamilyResultKind {
                family: family.visible_name.clone(),
                expected: format!("{:?}", Kind::Type),
                found: format!("{:?}", family.result_kind),
                span: anchor_span(&family.source_anchor),
            });
        }
        if !family
            .dependency_closure
            .closure_metadata
            .public_closure_checked
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' has unchecked dependency closure",
                    family.visible_name
                ),
                anchor_span(&family.source_anchor),
            ));
        }
        let metadata = &family.dependency_closure.closure_metadata;
        if metadata.public_ordinary_type_count != family.dependency_closure.ordinary_types.len()
            || metadata.public_sealed_domain_count != family.dependency_closure.sealed_domains.len()
            || metadata.public_domain_constructor_count
                != family.dependency_closure.domain_constructors.len()
            || metadata.public_type_function_count != family.dependency_closure.type_functions.len()
            || metadata.public_projection_count
                != family.dependency_closure.associated_projections.len()
            || metadata.public_associated_family_count
                != family.dependency_closure.associated_families.len() + 1
            || metadata.helper_family_count
                != family
                    .dependency_closure
                    .associated_families
                    .iter()
                    .filter(|dependency| !dependency.source_visible)
                    .count()
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' has inconsistent dependency closure metadata counts",
                    family.visible_name
                ),
                anchor_span(&family.source_anchor),
            ));
        }
        if !family
            .revalidation_metadata
            .decreases
            .iter()
            .all(|decreases| {
                decreases.structural_recursion_checked
                    && family.schemes.first().is_some_and(|scheme| {
                        scheme
                            .params
                            .get(decreases.parameter_index)
                            .is_some_and(|param| {
                                param.name == decreases.parameter
                                    && param.domain_constraint.as_ref() == Some(&decreases.domain)
                            })
                    })
            })
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' has malformed decreases revalidation metadata",
                    family.visible_name
                ),
                anchor_span(&family.source_anchor),
            ));
        }
        self.validate_imported_associated_family_dependency_closure(family)?;
        self.validate_imported_associated_family_dependency_closure_complete(family)?;
        let declaration = self
            .associated_family_declarations
            .get(&family.head)
            .cloned()
            .ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' was not declared before validation",
                        family.visible_name
                    ),
                    anchor_span(&family.source_anchor),
                )
            })?;
        if !matches_associated_family_result_constraint(
            &family.result_domain,
            &declaration.result_domain,
        ) {
            return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: family.visible_name.clone(),
                reason: "summary result-domain annotation does not match the declaration"
                    .to_string(),
                span: anchor_span(&family.source_anchor),
            });
        }
        if family.schemes.is_empty() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' must contain at least one scheme",
                    family.visible_name
                ),
                anchor_span(&family.source_anchor),
            ));
        }
        for scheme in &family.schemes {
            self.validate_and_insert_imported_associated_family_scheme(
                family,
                &declaration,
                scheme.clone(),
            )?;
        }
        Ok(())
    }

    pub(super) fn validate_imported_associated_family_dependency_closure(
        &self,
        family: &AssociatedFamilySummary,
    ) -> Result<(), TypeEnvError> {
        for ty in &family.dependency_closure.ordinary_types {
            if !self.canonical_type_names.contains_key(ty) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' references unknown ordinary type dependency '{}'",
                        family.visible_name, ty.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }
        for domain in &family.dependency_closure.sealed_domains {
            if self.lookup_sealed_domain_by_id(domain).is_none() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' references unknown sealed-domain dependency '{}'",
                        family.visible_name, domain.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }
        for constructor in &family.dependency_closure.domain_constructors {
            let domain = self.lookup_sealed_domain_by_id(&constructor.domain).ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' references unknown constructor domain '{}'",
                        family.visible_name, constructor.domain.name
                    ),
                    anchor_span(&family.source_anchor),
                )
            })?;
            if !domain
                .constructors
                .iter()
                .any(|candidate| candidate.id == *constructor)
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' references unknown domain constructor '{}'",
                        family.visible_name, constructor.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }
        for head in &family.dependency_closure.type_functions {
            if !self.local_type_functions.contains_key(head) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' references unknown type-function dependency '{}'",
                        family.visible_name, head.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }
        for projection in &family.dependency_closure.associated_projections {
            if !self
                .known_interface_identities
                .contains(&projection.head.interface)
                || !self
                    .known_associated_member_identities
                    .contains(&projection.head.member)
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' references unknown associated projection dependency '{}::{}'",
                        family.visible_name,
                        projection.head.interface.name,
                        projection.head.member.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
            if let Some(declaration) = self.associated_family_declarations.get(&projection.head) {
                let expected = declaration.interface_params.len();
                let found = projection.interface_args.len();
                if found != expected {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' associated projection dependency '{}::{}' has {} interface argument(s), expected {}",
                            family.visible_name,
                            projection.head.interface.name,
                            projection.head.member.name,
                            found,
                            expected
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
            }
        }
        for dependency in &family.dependency_closure.associated_families {
            if !self
                .associated_family_declarations
                .contains_key(&dependency.family)
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' references unknown associated-family dependency '{}::{}'",
                        family.visible_name,
                        dependency.family.interface.name,
                        dependency.family.member.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
            if dependency.normalizer_available
                && !self
                    .associated_family_declarations
                    .contains_key(&dependency.family)
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' lacks normalizer-available dependency '{}::{}'",
                        family.visible_name,
                        dependency.family.interface.name,
                        dependency.family.member.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }
        Ok(())
    }

    pub(super) fn validate_imported_associated_family_dependency_closure_complete(
        &self,
        family: &AssociatedFamilySummary,
    ) -> Result<(), TypeEnvError> {
        let mut required = PublicAssociatedFamilyClosure::default();
        self.collect_public_canonical_type_closure_for_associated_family(
            &family.result_domain,
            &mut required,
        );
        for scheme in &family.schemes {
            self.collect_public_associated_family_scheme_closure(scheme, &mut required)?;
        }
        for projection in &family.dependency_closure.associated_projections {
            for arg in &projection.interface_args {
                self.collect_public_canonical_type_closure_for_associated_family(
                    arg,
                    &mut required,
                );
            }
        }
        required.associated_families.remove(&family.head);

        let ordinary_types = family
            .dependency_closure
            .ordinary_types
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        for ty in required.ordinary_types {
            if !ordinary_types.contains(&ty) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' dependency closure omits ordinary type '{}'",
                        family.visible_name, ty.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }

        let sealed_domains = family
            .dependency_closure
            .sealed_domains
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        for domain in required.sealed_domains {
            if !sealed_domains.contains(&domain) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' dependency closure omits sealed domain '{}'",
                        family.visible_name, domain.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }

        let domain_constructors = family
            .dependency_closure
            .domain_constructors
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        for constructor in required.domain_constructors {
            if !domain_constructors.contains(&constructor) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' dependency closure omits domain constructor '{}'",
                        family.visible_name, constructor.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }

        let type_functions = family
            .dependency_closure
            .type_functions
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        for head in required.type_functions {
            if !type_functions.contains(&head) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' dependency closure omits type function '{}'",
                        family.visible_name, head.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }

        let associated_projections = family
            .dependency_closure
            .associated_projections
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        for projection in required.projections {
            if !associated_projections.contains(&projection) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' dependency closure omits associated projection '{}::{}' with complete argument spine",
                        family.visible_name,
                        projection.head.interface.name,
                        projection.head.member.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }

        let associated_families = family
            .dependency_closure
            .associated_families
            .iter()
            .map(|dependency| dependency.family.clone())
            .collect::<HashSet<_>>();
        for head in required.associated_families {
            if !associated_families.contains(&head) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' dependency closure omits associated family '{}::{}'",
                        family.visible_name, head.interface.name, head.member.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }

        Ok(())
    }

    pub(super) fn validate_and_insert_imported_associated_family_scheme(
        &mut self,
        family: &AssociatedFamilySummary,
        declaration: &AssociatedFamilyDeclarationInfo,
        scheme: AssociatedFamilyScheme,
    ) -> Result<(), TypeEnvError> {
        if scheme.head != family.head {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' scheme head does not match summary head",
                    family.visible_name
                ),
                anchor_span(&scheme.source_anchor),
            ));
        }
        if scheme.result_kind != Kind::Type {
            return Err(TypeEnvError::WrongAssociatedFamilyResultKind {
                family: family.visible_name.clone(),
                expected: format!("{:?}", Kind::Type),
                found: format!("{:?}", scheme.result_kind),
                span: anchor_span(&scheme.source_anchor),
            });
        }
        if !matches_associated_family_result_constraint(
            &scheme.result_domain,
            &declaration.result_domain,
        ) {
            return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: family.visible_name.clone(),
                reason: "scheme result-domain annotation does not match the associated family declaration"
                    .to_string(),
                span: anchor_span(&scheme.source_anchor),
            });
        }
        if scheme.params.len() != declaration.interface_params.len() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' scheme arity mismatch: expected {}, found {}",
                    family.visible_name,
                    declaration.interface_params.len(),
                    scheme.params.len()
                ),
                anchor_span(&scheme.source_anchor),
            ));
        }
        for (param, expected) in scheme.params.iter().zip(&declaration.interface_params) {
            if param.kind != Kind::Type
                || param.name != expected.name
                || param.domain_constraint != expected.domain_constraint
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' scheme parameter '{}' does not match declaration",
                        family.visible_name, param.name
                    ),
                    anchor_span(&param.source_anchor),
                ));
            }
        }
        if scheme.equations.is_empty() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' scheme must contain at least one equation",
                    family.visible_name
                ),
                anchor_span(&scheme.source_anchor),
            ));
        }
        for equation in &scheme.equations {
            if equation.head != scheme.head
                || equation.interface_arg_patterns.len() != declaration.interface_params.len()
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' equation head or arity mismatch",
                        family.visible_name
                    ),
                    anchor_span(&equation.source_anchor),
                ));
            }
            let mut vars = HashMap::new();
            for (pattern, param) in equation
                .interface_arg_patterns
                .iter()
                .zip(&declaration.interface_params)
            {
                self.validate_imported_associated_family_pattern(
                    family,
                    pattern,
                    &param.domain_constraint,
                    &mut vars,
                )?;
            }
            self.validate_imported_associated_family_result_expr(family, &equation.result, &vars)?;
            if !Self::associated_family_expr_conforms_to_constraint(
                &equation.result,
                &declaration.result_domain,
            ) {
                return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                    family: family.visible_name.clone(),
                    reason: format!(
                        "RHS does not conform to associated family result constraint {}",
                        associated_family_result_constraint_label(&declaration.result_domain)
                    ),
                    span: anchor_span(&equation.source_anchor),
                });
            }
        }
        for (index, left) in scheme.equations.iter().enumerate() {
            for right in scheme.equations.iter().skip(index + 1) {
                if Self::associated_family_pattern_spines_overlap(
                    &left.interface_arg_patterns,
                    &right.interface_arg_patterns,
                ) {
                    return Err(TypeEnvError::OverlappingAssociatedFamilyScheme {
                        family: family.visible_name.clone(),
                        span: anchor_span(&right.source_anchor),
                    });
                }
            }
        }
        if let Some(existing_schemes) = self.associated_family_schemes.get(&scheme.head) {
            if existing_schemes
                .iter()
                .any(|existing| existing.scheme == scheme)
            {
                return Ok(());
            }
            for existing in existing_schemes {
                for existing_equation in &existing.scheme.equations {
                    for new_equation in &scheme.equations {
                        if Self::associated_family_pattern_spines_overlap(
                            &existing_equation.interface_arg_patterns,
                            &new_equation.interface_arg_patterns,
                        ) {
                            return Err(TypeEnvError::OverlappingAssociatedFamilyScheme {
                                family: family.visible_name.clone(),
                                span: anchor_span(&new_equation.source_anchor),
                            });
                        }
                    }
                }
            }
        }
        self.associated_family_schemes
            .entry(scheme.head.clone())
            .or_default()
            .push(RegisteredAssociatedFamilyScheme {
                defining_module: declaration.defining_module.clone(),
                scheme,
            });
        Ok(())
    }

    pub(super) fn validate_imported_associated_family_pattern(
        &self,
        family: &AssociatedFamilySummary,
        pattern: &AssociatedFamilyPattern,
        expected_domain: &Option<SealedDomainId>,
        vars: &mut HashMap<String, AssociatedFamilyResultConstraint>,
    ) -> Result<(), TypeEnvError> {
        match pattern {
            AssociatedFamilyPattern::Var {
                name, constraint, ..
            } => {
                let expected = expected_domain.clone().map_or(
                    AssociatedFamilyResultConstraint::Kind(Kind::Type),
                    AssociatedFamilyResultConstraint::Domain,
                );
                if constraint != &expected {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' pattern variable '{}' has invalid constraint",
                            family.visible_name, name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
                if vars.insert(name.clone(), expected).is_some() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' has non-linear pattern variable '{}'",
                            family.visible_name, name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
                Ok(())
            }
            AssociatedFamilyPattern::Wildcard { constraint, .. } => {
                let expected = expected_domain.clone().map_or(
                    AssociatedFamilyResultConstraint::Kind(Kind::Type),
                    AssociatedFamilyResultConstraint::Domain,
                );
                if constraint == &expected {
                    Ok(())
                } else {
                    Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' wildcard pattern has invalid constraint",
                            family.visible_name
                        ),
                        anchor_span(&family.source_anchor),
                    ))
                }
            }
            AssociatedFamilyPattern::Primitive { .. }
            | AssociatedFamilyPattern::NominalApp { .. } => Ok(()),
            AssociatedFamilyPattern::DomainConstructor {
                constructor,
                domain,
                fields,
                ..
            } => {
                let domain_summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' pattern references unknown sealed domain '{}'",
                            family.visible_name, domain.name
                        ),
                        anchor_span(&family.source_anchor),
                    )
                })?;
                if !domain_summary
                    .constructors
                    .iter()
                    .any(|candidate| candidate.id == **constructor)
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' pattern references unknown constructor '{}'",
                            family.visible_name, constructor.name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
                for field in fields {
                    self.validate_imported_associated_family_pattern(family, field, &None, vars)?;
                }
                Ok(())
            }
        }
    }

    pub(super) fn validate_imported_associated_family_result_expr(
        &self,
        family: &AssociatedFamilySummary,
        expr: &AssociatedFamilyResultExpr,
        vars: &HashMap<String, AssociatedFamilyResultConstraint>,
    ) -> Result<(), TypeEnvError> {
        match expr {
            AssociatedFamilyResultExpr::Primitive { kind, .. }
            | AssociatedFamilyResultExpr::Var { kind, .. }
            | AssociatedFamilyResultExpr::NominalApp { kind, .. }
            | AssociatedFamilyResultExpr::DomainConstructorApp { kind, .. }
            | AssociatedFamilyResultExpr::AssociatedFamilyProjection { kind, .. }
            | AssociatedFamilyResultExpr::Projection { kind, .. }
            | AssociatedFamilyResultExpr::ComputationHeadApp { kind, .. } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' result expression has non-Type kind",
                            family.visible_name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
            }
        }
        match expr {
            AssociatedFamilyResultExpr::Var { name, .. } => {
                if vars.contains_key(name) || name == "Type" {
                    Ok(())
                } else {
                    Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' result references unbound variable '{}'",
                            family.visible_name, name
                        ),
                        anchor_span(&family.source_anchor),
                    ))
                }
            }
            AssociatedFamilyResultExpr::Primitive { .. } => Ok(()),
            AssociatedFamilyResultExpr::NominalApp {
                origin,
                visible_name,
                args,
                ..
            } => {
                if !self.canonical_type_names.contains_key(origin) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' result references unknown ordinary type '{}'",
                            family.visible_name, visible_name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
                for arg in args {
                    self.validate_imported_associated_family_result_expr(family, arg, vars)?;
                }
                Ok(())
            }
            AssociatedFamilyResultExpr::DomainConstructorApp {
                constructor,
                domain,
                args,
                ..
            } => {
                let domain_summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' result references unknown sealed domain '{}'",
                            family.visible_name, domain.name
                        ),
                        anchor_span(&family.source_anchor),
                    )
                })?;
                if !domain_summary
                    .constructors
                    .iter()
                    .any(|candidate| candidate.id == *constructor)
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' result references unknown constructor '{}'",
                            family.visible_name, constructor.name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
                for arg in args {
                    self.validate_imported_associated_family_result_expr(family, arg, vars)?;
                }
                Ok(())
            }
            AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                head,
                interface_args,
                ..
            } => {
                if !self.associated_family_declarations.contains_key(head) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' result references unknown associated family '{}::{}'",
                            family.visible_name, head.interface.name, head.member.name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
                for arg in interface_args {
                    self.validate_imported_associated_family_result_expr(family, arg, vars)?;
                }
                Ok(())
            }
            AssociatedFamilyResultExpr::Projection {
                interface,
                member,
                args,
                ..
            } => {
                if !self.known_interface_identities.contains(interface)
                    || !self.known_associated_member_identities.contains(member)
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' result references unknown projection '{}::{}'",
                            family.visible_name, interface.name, member.name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
                for arg in args {
                    self.validate_imported_associated_family_result_expr(family, arg, vars)?;
                }
                Ok(())
            }
            AssociatedFamilyResultExpr::ComputationHeadApp { head, args, .. } => {
                if !self.local_type_functions.contains_key(head) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' result references unknown type function '{}'",
                            family.visible_name, head.name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
                for arg in args {
                    self.validate_imported_associated_family_result_expr(family, arg, vars)?;
                }
                Ok(())
            }
        }
    }

    pub(super) fn validate_imported_type_function_signature_type(
        &self,
        owner: &str,
        ty: &CanonicalTypeExpr,
        position: &str,
    ) -> Result<(), TypeEnvError> {
        match ty {
            CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::Var(_) => Ok(()),
            CanonicalTypeExpr::NominalApp {
                origin,
                visible_name,
                args,
                kind,
            } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} nominal '{visible_name}' has non-Type kind"
                        ),
                        Span::default(),
                    ));
                }
                match self.type_alias_identities.get(visible_name) {
                    Some(registered) if registered == origin => {}
                    Some(registered) => {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "type-function summary '{owner}' {position} nominal '{visible_name}' has identity mismatch: expected {:?}, found {:?}",
                                origin, registered
                            ),
                            Span::default(),
                        ));
                    }
                    None => {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "type-function summary '{owner}' {position} references unknown ordinary type '{visible_name}'"
                            ),
                            Span::default(),
                        ));
                    }
                }
                if !self.canonical_type_names.contains_key(origin) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} references unregistered ordinary type identity {:?}",
                            origin
                        ),
                        Span::default(),
                    ));
                }
                let expected_arity = self
                    .type_info
                    .get(visible_name)
                    .map(TypeInfo::type_arg_count)
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "type-function summary '{owner}' {position} references ordinary type '{visible_name}' without arity metadata"
                            ),
                            Span::default(),
                        )
                    })?;
                if expected_arity != args.len() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} nominal '{visible_name}' arity mismatch: expected {}, found {}",
                            expected_arity,
                            args.len()
                        ),
                        Span::default(),
                    ));
                }
                for arg in args {
                    self.validate_imported_type_function_signature_type(owner, arg, position)?;
                }
                Ok(())
            }
            CanonicalTypeExpr::Projection {
                interface,
                member,
                args,
                kind,
                ..
            } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} projection '{}::{}' has non-Type kind",
                            interface.name, member.name
                        ),
                        Span::default(),
                    ));
                }
                if !self.known_interface_identities.contains(interface) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} references unknown projection interface '{}'",
                            interface.name
                        ),
                        Span::default(),
                    ));
                }
                if !self.known_associated_member_identities.contains(member)
                    || member.interface != *interface
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} references unknown projection member '{}::{}'",
                            interface.name, member.name
                        ),
                        Span::default(),
                    ));
                }
                for arg in args {
                    self.validate_imported_type_function_signature_type(owner, arg, position)?;
                }
                Ok(())
            }
            CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
                self.validate_registered_promoted_constructor_app(
                    &app.constructor,
                    &app.data_kind,
                    app.args.len(),
                    &app.kind,
                    Span::default(),
                )?;
                for (index, arg) in app.args.iter().enumerate() {
                    self.validate_imported_type_function_signature_type(owner, arg, position)?;
                    if let Some(expected_kind) = self
                        .promoted_constructor_kind(&app.constructor)
                        .and_then(|kinding| kinding.field_data_kind_constraints.get(index))
                        .and_then(|constraint| constraint.as_ref())
                    {
                        self.validate_canonical_promoted_data_kind(
                            arg,
                            expected_kind,
                            Span::default(),
                        )?;
                    }
                }
                Ok(())
            }
            CanonicalTypeExpr::ConstructorVariableApp(app) => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "type-function summary '{owner}' {position} contains constructor-variable application '{}', which is unsupported until TASK-907",
                    app.constructor.name
                ),
                Span::default(),
            )),
            CanonicalTypeExpr::ComputationHeadApp { head, args, kind } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} computation head '{}' has non-Type kind",
                            head.name
                        ),
                        Span::default(),
                    ));
                }
                let callee = self.local_type_functions.get(head).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} references unknown type function '{}'",
                            head.name
                        ),
                        Span::default(),
                    )
                })?;
                if callee.params.len() != args.len() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} computation head '{}' arity mismatch: expected {}, found {}",
                            head.name,
                            callee.params.len(),
                            args.len()
                        ),
                        Span::default(),
                    ));
                }
                for arg in args {
                    self.validate_imported_type_function_signature_type(owner, arg, position)?;
                }
                Ok(())
            }
        }
    }

    pub(super) fn validate_imported_type_function_pattern(
        &self,
        pattern: &TypeFunctionPattern,
        expected: &TypeFunctionPatternConstraint,
        vars: &mut HashMap<String, TypeFunctionResultConstraint>,
    ) -> Result<(), TypeEnvError> {
        match pattern {
            TypeFunctionPattern::Var {
                name, constraint, ..
            } => {
                if constraint != expected {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!("type-function pattern variable '{name}' has invalid constraint"),
                        Span::default(),
                    ));
                }
                if vars
                    .insert(name.clone(), result_constraint_from_pattern(constraint))
                    .is_some()
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!("non-linear type-function pattern variable '{name}'"),
                        Span::default(),
                    ));
                }
                Ok(())
            }
            TypeFunctionPattern::Wildcard { constraint, .. } => {
                if constraint != expected {
                    return Err(TypeEnvError::InvalidDefinition(
                        "type-function wildcard pattern has invalid constraint".to_string(),
                        Span::default(),
                    ));
                }
                Ok(())
            }
            TypeFunctionPattern::DomainConstructor {
                constructor,
                domain,
                fields,
                constraint,
                ..
            } => {
                if constraint != expected {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function constructor pattern '{}' has invalid constraint",
                            constructor.name
                        ),
                        Span::default(),
                    ));
                }
                let domain_summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function pattern references unknown sealed domain '{}'",
                            domain.name
                        ),
                        Span::default(),
                    )
                })?;
                let constructor_summary = domain_summary
                    .constructors
                    .iter()
                    .find(|candidate| candidate.id == **constructor)
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "type-function pattern references unknown constructor '{}'",
                                constructor.name
                            ),
                            Span::default(),
                        )
                    })?;
                if constructor_summary.fields.len() != fields.len() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function constructor pattern '{}' field arity mismatch",
                            constructor.name
                        ),
                        Span::default(),
                    ));
                }
                for (field_pattern, field) in fields.iter().zip(&constructor_summary.fields) {
                    let field_constraint = field.domain_constraint.clone().map_or_else(
                        || TypeFunctionPatternConstraint::Kind(field.kind.clone()),
                        TypeFunctionPatternConstraint::Domain,
                    );
                    self.validate_imported_type_function_pattern(
                        field_pattern,
                        &field_constraint,
                        vars,
                    )?;
                }
                Ok(())
            }
        }
    }

    pub(super) fn validate_imported_type_function_result(
        &self,
        expr: &TypeFunctionResultExpr,
        vars: &HashMap<String, TypeFunctionResultConstraint>,
    ) -> Result<TypeFunctionResultConstraint, TypeEnvError> {
        match expr {
            TypeFunctionResultExpr::Primitive { kind, .. } => {
                if kind == &Kind::Type {
                    Ok(TypeFunctionResultConstraint::Kind(Kind::Type))
                } else {
                    Err(TypeEnvError::InvalidDefinition(
                        "type-function result expression has non-Type kind".to_string(),
                        Span::default(),
                    ))
                }
            }
            TypeFunctionResultExpr::Var { name, kind, .. } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!("type-function result variable '{name}' has non-Type kind"),
                        Span::default(),
                    ));
                }
                vars.get(name).cloned().ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!("unbound type-function result variable '{name}'"),
                        Span::default(),
                    )
                })
            }
            TypeFunctionResultExpr::NominalApp {
                origin,
                visible_name,
                args,
                kind,
                ..
            } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        "type-function nominal result expression has non-Type kind".to_string(),
                        Span::default(),
                    ));
                }
                match self.type_alias_identities.get(visible_name) {
                    Some(registered) if registered == origin => {}
                    Some(registered) => {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "type-function result nominal '{}' has identity mismatch: expected {:?}, found {:?}",
                                visible_name, origin, registered
                            ),
                            Span::default(),
                        ));
                    }
                    None => {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "type-function result references unknown ordinary type '{}'",
                                visible_name
                            ),
                            Span::default(),
                        ));
                    }
                }
                if !self.canonical_type_names.contains_key(origin) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function result references unregistered ordinary type identity {:?}",
                            origin
                        ),
                        Span::default(),
                    ));
                }
                let expected_arity = self
                    .type_info
                    .get(visible_name)
                    .map(TypeInfo::type_arg_count)
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "type-function result references ordinary type '{}' without arity metadata",
                                visible_name
                            ),
                            Span::default(),
                        )
                    })?;
                if expected_arity != args.len() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function result nominal '{}' arity mismatch: expected {}, found {}",
                            visible_name,
                            expected_arity,
                            args.len()
                        ),
                        Span::default(),
                    ));
                }
                for arg in args {
                    self.validate_imported_type_function_result(arg, vars)?;
                }
                Ok(TypeFunctionResultConstraint::Kind(Kind::Type))
            }
            TypeFunctionResultExpr::Projection {
                interface,
                member,
                args,
                kind,
                constraint,
                ..
            } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        "type-function projection result expression has non-Type kind".to_string(),
                        Span::default(),
                    ));
                }
                if !self.known_interface_identities.contains(interface) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function projection result references unknown interface identity {:?}",
                            interface
                        ),
                        Span::default(),
                    ));
                }
                if !self.known_associated_member_identities.contains(member) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function projection result references unknown associated member identity {:?}",
                            member
                        ),
                        Span::default(),
                    ));
                }
                if member.interface != *interface {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function projection result member {:?} does not belong to interface {:?}",
                            member, interface
                        ),
                        Span::default(),
                    ));
                }
                if !matches!(constraint, TypeFunctionResultConstraint::Kind(Kind::Type)) {
                    return Err(TypeEnvError::InvalidDefinition(
                        "type-function projection result cannot forge a sealed-domain constraint"
                            .to_string(),
                        Span::default(),
                    ));
                }
                for arg in args {
                    self.validate_imported_type_function_result(arg, vars)?;
                }
                Ok(TypeFunctionResultConstraint::Kind(Kind::Type))
            }
            TypeFunctionResultExpr::DomainConstructorApp {
                constructor,
                domain,
                args,
                kind,
                ..
            } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        "type-function domain-constructor result has non-Type kind".to_string(),
                        Span::default(),
                    ));
                }
                let domain_summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function result references unknown sealed domain '{}'",
                            domain.name
                        ),
                        Span::default(),
                    )
                })?;
                let constructor_summary = domain_summary
                    .constructors
                    .iter()
                    .find(|candidate| candidate.id == *constructor)
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "type-function result references unknown constructor '{}'",
                                constructor.name
                            ),
                            Span::default(),
                        )
                    })?;
                if constructor_summary.fields.len() != args.len() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function result constructor '{}' field arity mismatch",
                            constructor.name
                        ),
                        Span::default(),
                    ));
                }
                for (arg, field) in args.iter().zip(&constructor_summary.fields) {
                    let actual = self.validate_imported_type_function_result(arg, vars)?;
                    let expected = field.domain_constraint.clone().map_or_else(
                        || TypeFunctionResultConstraint::Kind(field.kind.clone()),
                        TypeFunctionResultConstraint::Domain,
                    );
                    self.validate_imported_result_constraint_value(
                        &actual,
                        &expected,
                        Span::default(),
                    )?;
                }
                Ok(TypeFunctionResultConstraint::Domain(domain.clone()))
            }
            TypeFunctionResultExpr::PromotedDataConstructorApp {
                constructor,
                data_kind,
                args,
                kind,
                constraint,
                ..
            } => {
                self.validate_registered_promoted_constructor_app(
                    constructor,
                    data_kind,
                    args.len(),
                    kind,
                    Span::default(),
                )?;
                if !matches!(constraint, TypeFunctionResultConstraint::Kind(Kind::Type)) {
                    return Err(TypeEnvError::InvalidDefinition(
                        "type-function promoted constructor result cannot forge a sealed-domain constraint"
                            .to_string(),
                        Span::default(),
                    ));
                }
                let kinding = self.promoted_constructor_kind(constructor).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function result references unknown promoted data constructor '{}'",
                            constructor.name
                        ),
                        Span::default(),
                    )
                })?;
                for (index, arg) in args.iter().enumerate() {
                    let actual = self.validate_imported_type_function_result(arg, vars)?;
                    self.validate_imported_result_constraint_value(
                        &actual,
                        &TypeFunctionResultConstraint::Kind(Kind::Type),
                        Span::default(),
                    )?;
                    if let Some(expected_kind) = kinding
                        .field_data_kind_constraints
                        .get(index)
                        .and_then(|constraint| constraint.as_ref())
                    {
                        self.validate_promoted_result_arg_data_kind(arg, expected_kind)?;
                    }
                }
                Ok(TypeFunctionResultConstraint::Kind(Kind::Type))
            }
            TypeFunctionResultExpr::ComputationHeadApp {
                head, args, kind, ..
            } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        "type-function computation result has non-Type kind".to_string(),
                        Span::default(),
                    ));
                }
                let callee = self.local_type_functions.get(head).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function result references unknown computation head '{}'",
                            head.name
                        ),
                        Span::default(),
                    )
                })?;
                if callee.params.len() != args.len() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function result computation '{}' arity mismatch",
                            head.name
                        ),
                        Span::default(),
                    ));
                }
                for (arg, param) in args.iter().zip(&callee.params) {
                    let actual = self.validate_imported_type_function_result(arg, vars)?;
                    let expected = param.domain_constraint.clone().map_or_else(
                        || TypeFunctionResultConstraint::Kind(param.kind.clone()),
                        TypeFunctionResultConstraint::Domain,
                    );
                    self.validate_imported_result_constraint_value(
                        &actual,
                        &expected,
                        Span::default(),
                    )?;
                }
                Ok(callee.result_constraint.clone())
            }
        }
    }

    pub(super) fn validate_imported_result_constraint_value(
        &self,
        actual: &TypeFunctionResultConstraint,
        expected: &TypeFunctionResultConstraint,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match (expected, actual) {
            (
                TypeFunctionResultConstraint::Domain(expected_domain),
                TypeFunctionResultConstraint::Domain(actual_domain),
            ) if expected_domain == actual_domain => Ok(()),
            (TypeFunctionResultConstraint::Domain(expected_domain), found) => {
                Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "result domain mismatch: expected sealed domain '{}', found {:?}",
                        expected_domain.name, found
                    ),
                    span,
                ))
            }
            (TypeFunctionResultConstraint::Kind(_), _) => Ok(()),
        }
    }

    pub(super) fn validate_registered_promoted_constructor_app(
        &self,
        constructor: &PromotedConstructorId,
        data_kind: &PromotedDataKindId,
        arg_count: usize,
        kind: &Kind,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        if kind != &Kind::Type {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data constructor '{}' has non-Type kind",
                    constructor.name
                ),
                span,
            ));
        }
        let Some(kind_summary) = self.lookup_promoted_data_kind_by_id(data_kind) else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data constructor '{}' references unknown promoted data kind '{}'",
                    constructor.name, data_kind.name
                ),
                span,
            ));
        };
        if !kind_summary
            .constructors
            .iter()
            .any(|candidate| candidate.id == *constructor)
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data constructor '{}' is not registered in promoted data kind '{}'",
                    constructor.name, data_kind.name
                ),
                span,
            ));
        }
        let Some(kinding) = self.promoted_constructor_kind(constructor) else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data constructor '{}' has no validated kinding metadata",
                    constructor.name
                ),
                span,
            ));
        };
        if &kinding.result_data_kind != data_kind {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data constructor '{}' result data kind mismatch: expected '{}', found '{}'",
                    constructor.name, kinding.result_data_kind.name, data_kind.name
                ),
                span,
            ));
        }
        if kinding.field_data_kind_constraints.len() != arg_count {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data constructor '{}' arity mismatch: expected {}, found {}",
                    constructor.name,
                    kinding.field_data_kind_constraints.len(),
                    arg_count
                ),
                span,
            ));
        }
        Ok(())
    }

    pub(super) fn validate_canonical_promoted_data_kind(
        &self,
        expr: &CanonicalTypeExpr,
        expected_kind: &PromotedDataKindId,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match expr {
            CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
                if &app.data_kind != expected_kind {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "promoted data constructor '{}' has data kind '{}', expected '{}'",
                            app.constructor.name, app.data_kind.name, expected_kind.name
                        ),
                        span,
                    ));
                }
                self.validate_registered_promoted_constructor_app(
                    &app.constructor,
                    &app.data_kind,
                    app.args.len(),
                    &app.kind,
                    span,
                )?;
                let kinding = self
                    .promoted_constructor_kind(&app.constructor)
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "promoted data constructor '{}' has no validated kinding metadata",
                                app.constructor.name
                            ),
                            span,
                        )
                    })?;
                for (index, arg) in app.args.iter().enumerate() {
                    if let Some(field_kind) = kinding
                        .field_data_kind_constraints
                        .get(index)
                        .and_then(|constraint| constraint.as_ref())
                    {
                        self.validate_canonical_promoted_data_kind(arg, field_kind, span)?;
                    }
                }
                Ok(())
            }
            other => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data-kind constrained field expected value of promoted data kind '{}', found {}",
                    expected_kind.name,
                    canonical_type_expr_head_name(other)
                ),
                span,
            )),
        }
    }

    pub(super) fn validate_promoted_result_arg_data_kind(
        &self,
        expr: &TypeFunctionResultExpr,
        expected_kind: &PromotedDataKindId,
    ) -> Result<(), TypeEnvError> {
        match expr {
            TypeFunctionResultExpr::PromotedDataConstructorApp {
                constructor,
                data_kind,
                args,
                kind,
                ..
            } => {
                if data_kind.as_ref() != expected_kind {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "promoted data constructor '{}' has data kind '{}', expected '{}'",
                            constructor.name, data_kind.name, expected_kind.name
                        ),
                        Span::default(),
                    ));
                }
                self.validate_registered_promoted_constructor_app(
                    constructor,
                    data_kind,
                    args.len(),
                    kind,
                    Span::default(),
                )?;
                let kinding = self.promoted_constructor_kind(constructor).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "promoted data constructor '{}' has no validated kinding metadata",
                            constructor.name
                        ),
                        Span::default(),
                    )
                })?;
                for (index, arg) in args.iter().enumerate() {
                    if let Some(field_kind) = kinding
                        .field_data_kind_constraints
                        .get(index)
                        .and_then(|constraint| constraint.as_ref())
                    {
                        self.validate_promoted_result_arg_data_kind(arg, field_kind)?;
                    }
                }
                Ok(())
            }
            other => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data-kind constrained field expected value of promoted data kind '{}', found {}",
                    expected_kind.name,
                    type_function_result_expr_head_name(other)
                ),
                Span::default(),
            )),
        }
    }

    // ------------------------------------------------------------------
    // Sealed-domain registration helpers
    // ------------------------------------------------------------------

    /// First pass: declare a sealed-domain identity and visible alias.
    ///
    /// Checks that the domain identity is not already registered under a
    /// different visible name, and that the visible name does not collide
    /// with ordinary types or other sealed domains.
    pub(super) fn declare_sealed_domain_identity(
        &mut self,
        domain: &SealedDomainSummary,
    ) -> Result<(), TypeEnvError> {
        let visible_name = domain.exported_name.as_str();

        // Check for collision with ordinary types.
        if self.ast_types.contains_key(visible_name)
            || self.type_alias_identities.contains_key(visible_name)
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "sealed domain name '{}' collides with an existing ordinary type",
                    visible_name
                ),
                Span::default(),
            ));
        }

        // Check for collision with other sealed domains (different identity, same name).
        if let Some(existing) = self.sealed_domain_aliases.get(visible_name)
            && existing != &domain.id
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "duplicate sealed domain alias '{}': existing {:?}, new {:?}",
                    visible_name, existing, domain.id
                ),
                Span::default(),
            ));
        }

        // Check that the identity is not already registered under a different name.
        if self.sealed_domain_identities.contains(&domain.id)
            && let Some(alias) = self.sealed_domain_aliases.iter().find_map(|(k, v)| {
                if v == &domain.id {
                    Some(k.as_str())
                } else {
                    None
                }
            })
            && alias != visible_name
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "sealed domain identity already registered under alias '{}'",
                    alias
                ),
                Span::default(),
            ));
        }

        self.sealed_domain_identities.insert(domain.id.clone());
        self.sealed_domain_aliases
            .insert(visible_name.to_string(), domain.id.clone());

        Ok(())
    }

    /// Second pass: validate structural constraints and store the full domain summary.
    ///
    /// Validates:
    /// - Field domain references resolve to known domains
    /// - At most one `StructuralSelfDomain` field per constructor
    /// - Constructor id domain matches enclosing domain
    pub(super) fn validate_and_register_sealed_domain(
        &mut self,
        domain: &SealedDomainSummary,
    ) -> Result<(), TypeEnvError> {
        for constructor in &domain.constructors {
            // Constructor id must reference the enclosing domain.
            if constructor.id.domain != domain.id {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "constructor '{}' in domain '{}' references a different domain",
                        constructor.exported_name, domain.exported_name
                    ),
                    Span::default(),
                ));
            }
            if constructor.id.name != constructor.exported_name {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "constructor '{}' in domain '{}' has id name '{}' that does not match exported name",
                        constructor.exported_name, domain.exported_name, constructor.id.name
                    ),
                    Span::default(),
                ));
            }

            // At most one StructuralSelfDomain field per constructor.
            let structural_count = constructor
                .fields
                .iter()
                .filter(|f| f.structural_status == StructuralFieldStatus::StructuralSelfDomain)
                .count();
            if structural_count > 1 {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "constructor '{}' in domain '{}' has {} structural self-domain fields; at most one is permitted",
                        constructor.exported_name, domain.exported_name, structural_count
                    ),
                    Span::default(),
                ));
            }

            // Validate field kinds, structural status, and domain references.
            for field in &constructor.fields {
                if field.kind != Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "field '{}' in constructor '{}' has non-Type kind",
                            field.name, constructor.exported_name
                        ),
                        Span::default(),
                    ));
                }
                let expected_status = if field.domain_constraint.as_ref() == Some(&domain.id) {
                    StructuralFieldStatus::StructuralSelfDomain
                } else {
                    StructuralFieldStatus::NonStructural
                };
                if field.structural_status != expected_status {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "field '{}' in constructor '{}' has structural status {:?}; expected {:?}",
                            field.name,
                            constructor.exported_name,
                            field.structural_status,
                            expected_status
                        ),
                        Span::default(),
                    ));
                }
                if let Some(ref constraint) = field.domain_constraint {
                    // The constraint must be the enclosing domain (self-reference) or
                    // a domain already declared in this environment.
                    if constraint != &domain.id
                        && !self.sealed_domain_identities.contains(constraint)
                    {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "field '{}' in constructor '{}' references unknown sealed domain",
                                field.name, constructor.exported_name
                            ),
                            Span::default(),
                        ));
                    }
                }
            }
        }

        // Store the full domain summary.
        self.sealed_domain_summaries
            .insert(domain.id.clone(), domain.clone());

        Ok(())
    }

    /// Look up a sealed domain by its visible exported name.
    #[must_use]
    pub fn lookup_sealed_domain(&self, name: &str) -> Option<&SealedDomainSummary> {
        let id = self.sealed_domain_aliases.get(name)?;
        self.sealed_domain_summaries.get(id)
    }

    /// First pass: declare a promoted data-kind identity and visible alias.
    pub(super) fn declare_promoted_data_kind_identity(
        &mut self,
        data_kind: &PromotedDataKindSummary,
    ) -> Result<(), TypeEnvError> {
        let visible_name = data_kind.exported_name.as_str();
        let hidden_dependency_metadata = is_dependency_metadata_name(visible_name);
        if !hidden_dependency_metadata
            && let Some(existing) = self.promoted_data_kind_aliases.get(visible_name)
            && existing != &data_kind.id
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "duplicate promoted data-kind alias '{}': existing {:?}, new {:?}",
                    visible_name, existing, data_kind.id
                ),
                anchor_span(&data_kind.source_anchor),
            ));
        }
        if !hidden_dependency_metadata
            && self.promoted_data_kind_identities.contains(&data_kind.id)
            && let Some(alias) = self
                .promoted_data_kind_aliases
                .iter()
                .find_map(|(alias, id)| {
                    if id == &data_kind.id {
                        Some(alias.as_str())
                    } else {
                        None
                    }
                })
            && alias != visible_name
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data-kind identity already registered under alias '{}'",
                    alias
                ),
                anchor_span(&data_kind.source_anchor),
            ));
        }

        self.promoted_data_kind_identities
            .insert(data_kind.id.clone());
        if !hidden_dependency_metadata {
            self.promoted_data_kind_aliases
                .insert(visible_name.to_string(), data_kind.id.clone());
        }
        Ok(())
    }
}

/// Return the constructor metadata that marks an exported alias body as the
/// representation carrier for the bounded non-generic nominal-newtype form.
///
/// Ordinary aliases never carry value-constructor metadata, so this is an
/// identity-based classification rather than a textual-name convention.
pub(super) fn imported_nominal_newtype_constructor<'a>(
    ty: &TypeDeclSummary,
    constructors: &'a [ConstructorSummary],
) -> Result<Option<&'a ConstructorSummary>, TypeEnvError> {
    if ty.declaration_kind != TypeDeclarationKind::NominalNewtype {
        return Ok(None);
    }
    if !ty.params.is_empty() {
        return Err(TypeEnvError::InvalidDefinition(
            format!(
                "generic imported newtype '{}' is not supported by nominal checking",
                ty.exported_name
            ),
            Span::default(),
        ));
    }
    if !matches!(
        ty.representation,
        TypeRepresentationSummary::Exposed(TypeBody::Alias(_))
    ) {
        return Err(TypeEnvError::InvalidDefinition(
            format!(
                "imported nominal newtype '{}' must carry an exposed alias representation",
                ty.exported_name
            ),
            Span::default(),
        ));
    }
    let mut matching = constructors
        .iter()
        .filter(|constructor| constructor.parent == ty.id);
    let Some(constructor) = matching.next() else {
        return Err(TypeEnvError::InvalidDefinition(
            format!(
                "imported nominal newtype '{}' must export exactly one constructor",
                ty.exported_name
            ),
            Span::default(),
        ));
    };
    if matching.next().is_some()
        || constructor.payload_kind != ConstructorPayloadKind::Tuple
        || constructor.id.name != constructor.exported_name
    {
        return Err(TypeEnvError::InvalidDefinition(
            format!(
                "imported nominal newtype '{}' has invalid constructor metadata",
                ty.exported_name
            ),
            Span::default(),
        ));
    }
    Ok(Some(constructor))
}
