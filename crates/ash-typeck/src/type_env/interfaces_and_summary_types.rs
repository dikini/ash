use super::*;

impl TypeEnv {
    pub(super) fn validate_interface_evidence_constraints(
        &self,
        interface_name: &str,
        interface_type_params: &[String],
        type_param_kinds: &[Kind],
        constraints: &[ash_parser::surface::InterfaceEvidenceConstraint],
    ) -> Result<Vec<InterfaceEvidenceConstraintInfo>, TypeEnvError> {
        let param_positions = interface_type_params
            .iter()
            .enumerate()
            .map(|(index, name)| (name.as_str(), index))
            .collect::<HashMap<_, _>>();
        let mut lowered = Vec::with_capacity(constraints.len());

        for constraint in constraints {
            let Some(subject_param) = interface_constraint_subject_name(&constraint.subject) else {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "interface evidence constraint on '{interface_name}' must use an interface parameter as its subject"
                    ),
                    constraint.span,
                ));
            };
            let Some(&subject_param_index) = param_positions.get(subject_param) else {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "interface evidence constraint subject '{subject_param}' is not an interface parameter of '{interface_name}'"
                    ),
                    constraint.span,
                ));
            };
            let Some(required_interface) =
                interface_constraint_required_name(&constraint.interface)
            else {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "interface evidence constraint '{subject_param}: ...' on '{interface_name}' must use the MVP required evidence shape T: Interface"
                    ),
                    constraint.span,
                ));
            };
            if required_interface == interface_name {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "interface evidence constraint cycle: '{interface_name}' requires itself"
                    ),
                    constraint.span,
                ));
            }
            let required_info = self.interfaces.get(required_interface).ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!(
                        "unknown required evidence interface '{required_interface}' in evidence constraint on '{interface_name}'"
                    ),
                    constraint.span,
                )
            })?;
            if required_info.type_params.len() != 1 {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "required evidence interface '{required_interface}' in evidence constraint on '{interface_name}' must have arity 1 for the MVP T: Interface shape, found {}",
                        required_info.type_params.len()
                    ),
                    constraint.span,
                ));
            }
            let subject_kind = &type_param_kinds[subject_param_index];
            let required_kind = &required_info.type_param_kinds[0];
            if subject_kind != required_kind {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "required evidence interface '{required_interface}' expects subject kind {required_kind}, but interface parameter '{subject_param}' on '{interface_name}' has kind {subject_kind}"
                    ),
                    constraint.span,
                ));
            }
            if self.interface_constraint_graph_reaches(required_interface, interface_name) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "interface evidence constraint cycle: '{interface_name}' requires '{required_interface}' which requires '{interface_name}'"
                    ),
                    constraint.span,
                ));
            }

            lowered.push(InterfaceEvidenceConstraintInfo {
                subject_param: subject_param.to_string(),
                subject_param_index,
                required_interface: required_interface.to_string(),
            });
        }

        Ok(lowered)
    }

    pub(super) fn interface_constraint_graph_reaches(&self, start: &str, target: &str) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![start];
        while let Some(interface_name) = stack.pop() {
            if !visited.insert(interface_name) {
                continue;
            }
            let Some(info) = self.interfaces.get(interface_name) else {
                continue;
            };
            for constraint in &info.evidence_constraints {
                if constraint.required_interface == target {
                    return true;
                }
                stack.push(constraint.required_interface.as_str());
            }
        }
        false
    }

    pub(super) fn interface_entails_interface(&self, available: &str, required: &str) -> bool {
        if available == required {
            return true;
        }

        let mut visited = HashSet::new();
        let mut stack = vec![available];
        while let Some(interface_name) = stack.pop() {
            if !visited.insert(interface_name) {
                continue;
            }
            let Some(info) = self.interfaces.get(interface_name) else {
                continue;
            };
            for constraint in &info.evidence_constraints {
                if constraint.subject_param_index != 0 {
                    continue;
                }
                if constraint.required_interface == required {
                    return true;
                }
                stack.push(constraint.required_interface.as_str());
            }
        }
        false
    }

    pub(super) fn has_concrete_interface_evidence(
        &self,
        interface: &str,
        args: &[InterfaceEvidenceArg],
    ) -> bool {
        self.impls.iter().any(|scheme| {
            scheme.interface == interface
                && scheme.where_bounds.is_empty()
                && if scheme.type_params.is_empty() {
                    interface_evidence_args_match(&scheme.head_args, args, false)
                } else {
                    interface_evidence_args_match(&scheme.head_args, args, true)
                }
        })
    }

    pub(super) fn validate_concrete_impl_required_evidence(
        &self,
        interface: &InterfaceInfo,
        head_args: &[InterfaceEvidenceArg],
        error_span: Span,
    ) -> Result<(), TypeEnvError> {
        self.validate_impl_required_evidence(interface, head_args, &[], error_span)
    }

    pub(super) fn validate_impl_required_evidence(
        &self,
        interface: &InterfaceInfo,
        head_args: &[InterfaceEvidenceArg],
        where_bounds: &[WhereBound],
        error_span: Span,
    ) -> Result<(), TypeEnvError> {
        for constraint in &interface.evidence_constraints {
            let required_args = [head_args[constraint.subject_param_index].clone()];
            if self.has_concrete_interface_evidence(&constraint.required_interface, &required_args)
            {
                continue;
            }
            if let InterfaceEvidenceArg::Proper(Type::Var(var)) = &required_args[0]
                && where_bounds.iter().any(|bound| {
                    bound.type_var == *var
                        && self.interface_entails_interface(
                            &bound.interface,
                            &constraint.required_interface,
                        )
                })
            {
                continue;
            }
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "impl evidence {} requires missing required evidence {}",
                    render_interface_evidence_key(&interface.name, head_args),
                    render_interface_evidence_key(&constraint.required_interface, &required_args)
                ),
                error_span,
            ));
        }
        Ok(())
    }

    pub(super) fn convert_interface_method(
        &self,
        method: &InterfaceMethodSig,
        param_mapping: &HashMap<String, TypeVar>,
        ordered_param_names: &[String],
        interface_name: &str,
    ) -> Result<(String, InterfaceMethodInfo), TypeEnvError> {
        // Allow multi-parameter interface methods for associated-type support (TASK-567)
        let mut method_env = self.clone();
        for name in ordered_param_names {
            method_env
                .type_var_interface_bounds
                .entry(param_mapping[name])
                .or_default()
                .insert(interface_name.to_string());
        }

        let mut method_param_mapping = param_mapping.clone();
        let mut implicit_method_type_params = BTreeMap::new();
        for ty in method
            .params
            .iter()
            .chain(std::iter::once(&method.return_type))
        {
            collect_implicit_interface_method_type_params(
                ty,
                param_mapping,
                &method_env,
                &mut implicit_method_type_params,
            );
        }
        for (name, var) in &implicit_method_type_params {
            method_param_mapping.insert(name.clone(), *var);
            method_env.register_type_parameter_kind(name, Kind::Type)?;
        }

        let params: Vec<Type> = method
            .params
            .iter()
            .map(|ty| surface_type_to_type(ty, &method_param_mapping, &method_env))
            .collect::<Result<Vec<_>, _>>()?;

        let return_type =
            surface_type_to_type(&method.return_type, &method_param_mapping, &method_env)?;

        let type_params: Vec<TypeVar> = ordered_param_names
            .iter()
            .map(|name| param_mapping[name])
            .collect();
        let method_type_params = implicit_method_type_params.values().copied().collect();

        Ok((
            method.name.to_string(),
            InterfaceMethodInfo {
                type_params,
                method_type_params,
                params,
                return_type,
            },
        ))
    }

    /// Create a new empty type environment
    #[must_use]
    pub fn new() -> Self {
        Self {
            ast_types: HashMap::with_capacity(10),
            type_info: HashMap::with_capacity(10),
            constructors: HashMap::with_capacity(10),
            transparent_aliases: HashSet::with_capacity(4),
            type_declaration_states: HashMap::with_capacity(10),
            type_alias_identities: HashMap::with_capacity(10),
            canonical_type_names: HashMap::with_capacity(10),
            interface_identity_aliases: HashMap::with_capacity(4),
            interface_identity_alias_is_imported: HashMap::with_capacity(4),
            canonical_interface_names: HashMap::with_capacity(4),
            local_interface_arities: HashMap::with_capacity(4),
            known_interface_identities: HashSet::with_capacity(4),
            associated_member_identity_aliases: HashMap::with_capacity(4),
            associated_member_identity_alias_is_imported: HashMap::with_capacity(4),
            known_associated_member_identities: HashSet::with_capacity(4),
            interfaces: HashMap::with_capacity(4),
            capability_interfaces: HashMap::with_capacity(4),
            resource_types: HashMap::with_capacity(4),
            capability_implementations: HashMap::with_capacity(4),
            capability_bindings: HashMap::with_capacity(4),
            impls: Vec::new(),
            proposition_assumptions: Vec::new(),
            proposition_obligations: Vec::new(),
            proposition_predicate_aliases: HashMap::with_capacity(4),
            proposition_predicates: HashMap::with_capacity(4),
            type_var_interface_bounds: HashMap::with_capacity(4),
            type_parameter_kinds: HashMap::with_capacity(4),
            variables: HashMap::with_capacity(10),
            workflow_intrinsics: HashMap::with_capacity(2),
            public_workflow_summaries: HashMap::with_capacity(2),
            fn_contracts: HashMap::with_capacity(10),
            capability_symbols: HashSet::with_capacity(8),
            parent: None,
            providers: HashSet::new(),
            sealed_domain_identities: HashSet::new(),
            sealed_domain_aliases: HashMap::new(),
            sealed_domain_summaries: HashMap::new(),
            promoted_data_kind_identities: HashSet::new(),
            promoted_data_kind_aliases: HashMap::new(),
            promoted_data_kind_summaries: HashMap::new(),
            promoted_constructor_summaries: HashMap::new(),
            promoted_constructor_kinds: HashMap::new(),
            local_type_function_heads: HashMap::new(),
            local_type_functions: HashMap::new(),
            current_module_identity: None,
            associated_family_declarations: HashMap::new(),
            associated_family_name_index: HashMap::new(),
            associated_family_schemes: HashMap::new(),
            workflow_effect: None,
            capability_implementation_body: false,
        }
    }

    /// Return the workflow effect level currently in scope, if any.
    ///
    /// `Some(effect)` ⟹ we are inside a workflow body; closures get `Type::Fun`.
    /// `None`         ⟹ pure-fn or module-level context; closures get `Type::Fn`.
    #[must_use]
    pub fn workflow_effect(&self) -> Option<ash_core::Effect> {
        self.workflow_effect
    }

    /// Return the public computation-tower manifest for alpha tower algebra.
    #[must_use]
    pub fn public_tower_manifest(&self) -> &'static PublicTowerManifest {
        &PUBLIC_TOWER_MANIFEST
    }

    /// Set the module identity used for source-local semantic declarations.
    pub fn set_current_module_identity(&mut self, module: ModuleIdentity) {
        self.current_module_identity = Some(module);
    }

    /// Return the module identity used for source-local semantic declarations.
    #[must_use]
    pub fn current_module_identity(&self) -> Option<&ModuleIdentity> {
        self.current_module_identity.as_ref()
    }

    /// Set the source-local module identity only when the environment does not already have one.
    pub fn ensure_current_module_identity(&mut self, module: ModuleIdentity) {
        self.current_module_identity.get_or_insert(module);
    }

    pub(super) fn ensure_local_interface_identity(
        &mut self,
        interface_name: &str,
        module: &ModuleIdentity,
    ) -> InterfaceIdentityId {
        if let Some(existing) = self.interface_identity_aliases.get(interface_name) {
            return existing.clone();
        }

        let identity = InterfaceIdentityId::new(module.clone(), interface_name.to_string());
        self.known_interface_identities.insert(identity.clone());
        self.canonical_interface_names
            .insert(identity.clone(), interface_name.to_string());
        self.interface_identity_aliases
            .insert(interface_name.to_string(), identity.clone());
        self.interface_identity_alias_is_imported
            .insert(interface_name.to_string(), false);
        identity
    }

    pub(super) fn ensure_local_associated_member_identity(
        &mut self,
        interface_name: &str,
        interface: &InterfaceIdentityId,
        member_name: &str,
    ) -> AssociatedMemberIdentityId {
        let alias_key = (interface_name.to_string(), member_name.to_string());
        if let Some(existing) = self.associated_member_identity_aliases.get(&alias_key) {
            return existing.clone();
        }

        let identity = AssociatedMemberIdentityId::associated_type(
            interface.clone(),
            member_name.to_string(),
            vec![interface_name.to_string(), member_name.to_string()],
        );
        self.known_associated_member_identities
            .insert(identity.clone());
        self.associated_member_identity_aliases
            .insert(alias_key.clone(), identity.clone());
        self.associated_member_identity_alias_is_imported
            .insert(alias_key, false);
        identity
    }

    pub(super) fn sealed_domain_constraint_from_surface(
        &self,
        ty: &SurfaceType,
        span: Span,
    ) -> Result<SealedDomainId, TypeEnvError> {
        let SurfaceType::Name(name) = ty else {
            return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: "<declaration>".to_string(),
                reason: format!(
                    "expected sealed domain name, found {}",
                    surface_projection_base_spelling(ty)
                ),
                span,
            });
        };
        self.lookup_sealed_domain(name.as_ref())
            .map(|domain| domain.id.clone())
            .ok_or_else(|| TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: "<declaration>".to_string(),
                reason: format!("unknown sealed result domain '{name}'"),
                span,
            })
    }

    pub(super) fn associated_family_result_constraint_from_surface(
        &self,
        ty: &SurfaceType,
        span: Span,
    ) -> Result<AssociatedFamilyResultConstraint, TypeEnvError> {
        if matches!(ty, SurfaceType::Name(name) if name.as_ref() == "Type") {
            return Ok(AssociatedFamilyResultConstraint::Kind(Kind::Type));
        }
        self.sealed_domain_constraint_from_surface(ty, span)
            .map(AssociatedFamilyResultConstraint::Domain)
    }

    pub(super) fn optional_param_domain_constraint(
        &self,
        ty: Option<&SurfaceType>,
        span: Span,
    ) -> Result<Option<SealedDomainId>, TypeEnvError> {
        ty.map(|ty| self.sealed_domain_constraint_from_surface(ty, span))
            .transpose()
    }

    pub(super) fn associated_family_declarations_for_interface(
        &self,
        interface_name: &str,
    ) -> Vec<&AssociatedFamilyDeclarationInfo> {
        self.associated_family_name_index
            .iter()
            .filter_map(|((candidate_interface, _), head)| {
                (candidate_interface == interface_name)
                    .then(|| self.associated_family_declarations.get(head))
                    .flatten()
            })
            .collect()
    }

    /// Look up sealed associated-family declaration metadata by visible interface/member names.
    #[must_use]
    pub fn lookup_associated_family_declaration(
        &self,
        interface_name: &str,
        family_name: &str,
    ) -> Option<&AssociatedFamilyDeclarationInfo> {
        let head = self
            .associated_family_name_index
            .get(&(interface_name.to_string(), family_name.to_string()))?;
        self.associated_family_declarations.get(head)
    }

    /// Return coherence-checked associated-family schemes for a canonical head.
    #[must_use]
    pub fn associated_family_schemes(
        &self,
        head: &AssociatedFamilyHeadId,
    ) -> Option<&Vec<RegisteredAssociatedFamilyScheme>> {
        self.associated_family_schemes.get(head)
    }

    /// Look up sealed associated-family declaration metadata by canonical head.
    #[must_use]
    pub fn lookup_associated_family_declaration_by_head(
        &self,
        head: &AssociatedFamilyHeadId,
    ) -> Option<&AssociatedFamilyDeclarationInfo> {
        self.associated_family_declarations.get(head)
    }

    /// Reduce one local associated-family projection from already-normalized
    /// normalizer arguments.
    ///
    /// This TASK-866/TASK-867 API consults validated local or imported family
    /// declarations and schemes that are normalizer-available in this `TypeEnv`.
    #[must_use]
    pub fn reduce_local_associated_family_projection_from_normal_args(
        &self,
        head: &AssociatedFamilyHeadId,
        interface_args: &[NormalTypeExpr],
    ) -> LocalAssociatedFamilyProjectionLookup<'_> {
        let Some(_declaration) = self.lookup_associated_family_declaration_by_head(head) else {
            let reason = if self.associated_member_identity_known(&head.member) {
                NormalFormBlockReason::AssociatedFamilyNotSealed
            } else {
                NormalFormBlockReason::MissingAssociatedEvidence
            };
            return LocalAssociatedFamilyProjectionLookup::Blocked {
                family: Box::new(head.clone()),
                reason,
            };
        };

        let Some(schemes) = self.associated_family_schemes.get(head) else {
            return LocalAssociatedFamilyProjectionLookup::Blocked {
                family: Box::new(head.clone()),
                reason: NormalFormBlockReason::AssociatedFamilyLocalUnavailable,
            };
        };

        let mut selected = Vec::new();
        let mut blocker = None;
        for registered in schemes {
            for equation in &registered.scheme.equations {
                if equation.interface_arg_patterns.len() != interface_args.len() {
                    continue;
                }
                let mut bindings = BTreeMap::new();
                match Self::match_associated_family_normal_pattern_spine(
                    &equation.interface_arg_patterns,
                    interface_args,
                    &mut bindings,
                ) {
                    Ok(()) => selected.push(SelectedNormalizedAssociatedFamilyScheme {
                        family_head: head.clone(),
                        registered,
                        equation,
                        scheme_param_bindings: bindings,
                    }),
                    Err(AssociatedFamilyMatchFailure::Blocked(reason)) => blocker = Some(reason),
                    Err(AssociatedFamilyMatchFailure::NoMatch) => {}
                }
            }
        }

        match selected.len() {
            1 => {
                let selected = selected.remove(0);
                let result = Self::substitute_associated_family_result_expr_from_normal_bindings(
                    &selected.equation.result,
                    &selected.scheme_param_bindings,
                );
                LocalAssociatedFamilyProjectionLookup::Reduced(Box::new(
                    LocalAssociatedFamilyReduction { selected, result },
                ))
            }
            n if n > 1 => LocalAssociatedFamilyProjectionLookup::Blocked {
                family: Box::new(head.clone()),
                reason: NormalFormBlockReason::AmbiguousAssociatedFamilySelection,
            },
            _ => LocalAssociatedFamilyProjectionLookup::Blocked {
                family: Box::new(head.clone()),
                reason: blocker.map_or(
                    NormalFormBlockReason::MissingAssociatedEvidence,
                    associated_family_selection_blocker_to_normal_reason,
                ),
            },
        }
    }

    /// Select a unique associated-family scheme by one-way structural matching.
    #[must_use]
    pub fn select_associated_family_scheme(
        &self,
        head: &AssociatedFamilyHeadId,
        interface_args: &[CanonicalTypeExpr],
    ) -> AssociatedFamilySelection<'_> {
        let Some(schemes) = self.associated_family_schemes.get(head) else {
            return AssociatedFamilySelection::NoMatch {
                family: head.clone(),
            };
        };
        let mut selected = Vec::new();
        let mut blocker = None;
        for registered in schemes {
            for equation in &registered.scheme.equations {
                if equation.interface_arg_patterns.len() != interface_args.len() {
                    continue;
                }
                let mut bindings = BTreeMap::new();
                match Self::match_associated_family_pattern_spine(
                    &equation.interface_arg_patterns,
                    interface_args,
                    &mut bindings,
                ) {
                    Ok(()) => selected.push(SelectedAssociatedFamilyScheme {
                        family_head: head.clone(),
                        registered,
                        equation,
                        scheme_param_bindings: bindings,
                    }),
                    Err(AssociatedFamilyMatchFailure::Blocked(reason)) => blocker = Some(reason),
                    Err(AssociatedFamilyMatchFailure::NoMatch) => {}
                }
            }
        }
        match selected.len() {
            1 => AssociatedFamilySelection::Selected(selected.remove(0)),
            n if n > 1 => AssociatedFamilySelection::Ambiguous {
                family: head.clone(),
                candidate_count: n,
            },
            _ => blocker.map_or_else(
                || AssociatedFamilySelection::NoMatch {
                    family: head.clone(),
                },
                |reason| AssociatedFamilySelection::Blocked {
                    family: head.clone(),
                    reason,
                },
            ),
        }
    }

    /// Reduce a projection once when a unique associated-family scheme applies.
    pub fn reduce_associated_family_projection_once(
        &self,
        head: &AssociatedFamilyHeadId,
        interface_args: &[CanonicalTypeExpr],
    ) -> Result<AssociatedFamilyReduction<'_>, TypeEnvError> {
        match self.select_associated_family_scheme(head, interface_args) {
            AssociatedFamilySelection::Selected(selected) => {
                let result = Self::substitute_associated_family_result_expr(
                    &selected.equation.result,
                    &selected.scheme_param_bindings,
                );
                Ok(AssociatedFamilyReduction { selected, result })
            }
            AssociatedFamilySelection::Ambiguous {
                candidate_count, ..
            } => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "ambiguous associated-family selection for '{}::{}' with {candidate_count} candidates",
                    head.interface.name, head.member.name
                ),
                Span::default(),
            )),
            AssociatedFamilySelection::Blocked { reason, .. } => {
                Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family selection for '{}::{}' is blocked by {reason:?}",
                        head.interface.name, head.member.name
                    ),
                    Span::default(),
                ))
            }
            AssociatedFamilySelection::NoMatch { .. } => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "no associated-family scheme matches '{}::{}'",
                    head.interface.name, head.member.name
                ),
                Span::default(),
            )),
        }
    }

    pub(super) fn match_associated_family_pattern_spine(
        patterns: &[AssociatedFamilyPattern],
        args: &[CanonicalTypeExpr],
        bindings: &mut BTreeMap<String, CanonicalTypeExpr>,
    ) -> Result<(), AssociatedFamilyMatchFailure> {
        for (pattern, arg) in patterns.iter().zip(args.iter()) {
            Self::match_associated_family_pattern(pattern, arg, bindings)?;
        }
        Ok(())
    }

    pub(super) fn match_associated_family_pattern(
        pattern: &AssociatedFamilyPattern,
        arg: &CanonicalTypeExpr,
        bindings: &mut BTreeMap<String, CanonicalTypeExpr>,
    ) -> Result<(), AssociatedFamilyMatchFailure> {
        match pattern {
            AssociatedFamilyPattern::Var { name, .. } => {
                Self::ensure_associated_family_arg_is_capturable(arg)?;
                match bindings.get(name) {
                    Some(existing) if existing == arg => Ok(()),
                    Some(_) => Err(AssociatedFamilyMatchFailure::NoMatch),
                    None => {
                        bindings.insert(name.clone(), arg.clone());
                        Ok(())
                    }
                }
            }
            AssociatedFamilyPattern::Wildcard { .. } => {
                Self::ensure_associated_family_arg_is_capturable(arg)
            }
            AssociatedFamilyPattern::Primitive { name, .. } => match arg {
                CanonicalTypeExpr::Primitive(arg_name) if name == arg_name => Ok(()),
                CanonicalTypeExpr::Var(_) => Err(AssociatedFamilyMatchFailure::Blocked(
                    AssociatedFamilySelectionBlocker::AbstractScrutinee,
                )),
                CanonicalTypeExpr::ComputationHeadApp { .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(
                        AssociatedFamilySelectionBlocker::NeutralScrutinee,
                    ))
                }
                CanonicalTypeExpr::Projection { rigidity, .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(match rigidity {
                        ProjectionRigidity::Rigid => {
                            AssociatedFamilySelectionBlocker::RigidProjection
                        }
                        ProjectionRigidity::Neutral => {
                            AssociatedFamilySelectionBlocker::NeutralScrutinee
                        }
                    }))
                }
                _ => Err(AssociatedFamilyMatchFailure::NoMatch),
            },
            AssociatedFamilyPattern::NominalApp {
                origin,
                visible_name,
                args: pattern_args,
                ..
            } => match arg {
                CanonicalTypeExpr::NominalApp {
                    origin: arg_origin,
                    visible_name: arg_name,
                    args: arg_args,
                    ..
                } if origin == arg_origin
                    && visible_name == arg_name
                    && pattern_args.len() == arg_args.len() =>
                {
                    Self::match_associated_family_pattern_spine(pattern_args, arg_args, bindings)
                }
                CanonicalTypeExpr::Var(_) => Err(AssociatedFamilyMatchFailure::Blocked(
                    AssociatedFamilySelectionBlocker::AbstractScrutinee,
                )),
                CanonicalTypeExpr::ComputationHeadApp { .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(
                        AssociatedFamilySelectionBlocker::NeutralScrutinee,
                    ))
                }
                CanonicalTypeExpr::Projection { rigidity, .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(match rigidity {
                        ProjectionRigidity::Rigid => {
                            AssociatedFamilySelectionBlocker::RigidProjection
                        }
                        ProjectionRigidity::Neutral => {
                            AssociatedFamilySelectionBlocker::NeutralScrutinee
                        }
                    }))
                }
                _ => Err(AssociatedFamilyMatchFailure::NoMatch),
            },
            AssociatedFamilyPattern::DomainConstructor { .. } => match arg {
                CanonicalTypeExpr::Var(_) => Err(AssociatedFamilyMatchFailure::Blocked(
                    AssociatedFamilySelectionBlocker::AbstractScrutinee,
                )),
                CanonicalTypeExpr::ComputationHeadApp { .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(
                        AssociatedFamilySelectionBlocker::NeutralScrutinee,
                    ))
                }
                CanonicalTypeExpr::Projection { rigidity, .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(match rigidity {
                        ProjectionRigidity::Rigid => {
                            AssociatedFamilySelectionBlocker::RigidProjection
                        }
                        ProjectionRigidity::Neutral => {
                            AssociatedFamilySelectionBlocker::NeutralScrutinee
                        }
                    }))
                }
                _ => Err(AssociatedFamilyMatchFailure::NoMatch),
            },
        }
    }

    pub(super) fn ensure_associated_family_arg_is_capturable(
        arg: &CanonicalTypeExpr,
    ) -> Result<(), AssociatedFamilyMatchFailure> {
        match arg {
            CanonicalTypeExpr::ComputationHeadApp { .. } => {
                Err(AssociatedFamilyMatchFailure::Blocked(
                    AssociatedFamilySelectionBlocker::NeutralScrutinee,
                ))
            }
            CanonicalTypeExpr::Projection { rigidity, .. } => {
                Err(AssociatedFamilyMatchFailure::Blocked(match rigidity {
                    ProjectionRigidity::Rigid => AssociatedFamilySelectionBlocker::RigidProjection,
                    ProjectionRigidity::Neutral => {
                        AssociatedFamilySelectionBlocker::NeutralScrutinee
                    }
                }))
            }
            CanonicalTypeExpr::PromotedDataConstructorApp { .. } => Err(
                AssociatedFamilyMatchFailure::Blocked(AssociatedFamilySelectionBlocker::Ambiguous),
            ),
            _ => Ok(()),
        }
    }

    pub(super) fn match_associated_family_normal_pattern_spine(
        patterns: &[AssociatedFamilyPattern],
        args: &[NormalTypeExpr],
        bindings: &mut BTreeMap<String, NormalTypeExpr>,
    ) -> Result<(), AssociatedFamilyMatchFailure> {
        for (pattern, arg) in patterns.iter().zip(args.iter()) {
            Self::match_associated_family_normal_pattern(pattern, arg, bindings)?;
        }
        Ok(())
    }

    pub(super) fn match_associated_family_normal_pattern(
        pattern: &AssociatedFamilyPattern,
        arg: &NormalTypeExpr,
        bindings: &mut BTreeMap<String, NormalTypeExpr>,
    ) -> Result<(), AssociatedFamilyMatchFailure> {
        match pattern {
            AssociatedFamilyPattern::Var { name, .. } => {
                Self::ensure_associated_family_normal_arg_is_capturable(arg)?;
                match bindings.get(name) {
                    Some(existing) if existing == arg => Ok(()),
                    Some(_) => Err(AssociatedFamilyMatchFailure::NoMatch),
                    None => {
                        bindings.insert(name.clone(), arg.clone());
                        Ok(())
                    }
                }
            }
            AssociatedFamilyPattern::Wildcard { .. } => {
                Self::ensure_associated_family_normal_arg_is_capturable(arg)
            }
            AssociatedFamilyPattern::Primitive { name, .. } => match arg {
                NormalTypeExpr::Primitive(arg_name) if name == arg_name => Ok(()),
                NormalTypeExpr::Var(_) => Err(AssociatedFamilyMatchFailure::Blocked(
                    AssociatedFamilySelectionBlocker::AbstractScrutinee,
                )),
                NormalTypeExpr::NeutralComputationApp { .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(
                        AssociatedFamilySelectionBlocker::NeutralScrutinee,
                    ))
                }
                NormalTypeExpr::Projection { rigidity, .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(match rigidity {
                        ProjectionRigidity::Rigid => {
                            AssociatedFamilySelectionBlocker::RigidProjection
                        }
                        ProjectionRigidity::Neutral => {
                            AssociatedFamilySelectionBlocker::NeutralScrutinee
                        }
                    }))
                }
                _ => Err(AssociatedFamilyMatchFailure::NoMatch),
            },
            AssociatedFamilyPattern::NominalApp {
                origin,
                visible_name,
                args: pattern_args,
                ..
            } => match arg {
                NormalTypeExpr::NominalApp {
                    origin: arg_origin,
                    visible_name: arg_name,
                    args: arg_args,
                    ..
                } if origin == arg_origin
                    && visible_name == arg_name
                    && pattern_args.len() == arg_args.len() =>
                {
                    Self::match_associated_family_normal_pattern_spine(
                        pattern_args,
                        arg_args,
                        bindings,
                    )
                }
                NormalTypeExpr::Var(_) => Err(AssociatedFamilyMatchFailure::Blocked(
                    AssociatedFamilySelectionBlocker::AbstractScrutinee,
                )),
                NormalTypeExpr::NeutralComputationApp { .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(
                        AssociatedFamilySelectionBlocker::NeutralScrutinee,
                    ))
                }
                NormalTypeExpr::Projection { rigidity, .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(match rigidity {
                        ProjectionRigidity::Rigid => {
                            AssociatedFamilySelectionBlocker::RigidProjection
                        }
                        ProjectionRigidity::Neutral => {
                            AssociatedFamilySelectionBlocker::NeutralScrutinee
                        }
                    }))
                }
                _ => Err(AssociatedFamilyMatchFailure::NoMatch),
            },
            AssociatedFamilyPattern::DomainConstructor {
                constructor,
                domain,
                fields,
                ..
            } => match arg {
                NormalTypeExpr::DomainConstructorApp {
                    constructor: arg_constructor,
                    domain: arg_domain,
                    args: arg_args,
                    ..
                } if constructor.as_ref() == arg_constructor
                    && domain.as_ref() == arg_domain
                    && fields.len() == arg_args.len() =>
                {
                    Self::match_associated_family_normal_pattern_spine(fields, arg_args, bindings)
                }
                NormalTypeExpr::Var(_) => Err(AssociatedFamilyMatchFailure::Blocked(
                    AssociatedFamilySelectionBlocker::AbstractScrutinee,
                )),
                NormalTypeExpr::NeutralComputationApp { .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(
                        AssociatedFamilySelectionBlocker::NeutralScrutinee,
                    ))
                }
                NormalTypeExpr::Projection { rigidity, .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(match rigidity {
                        ProjectionRigidity::Rigid => {
                            AssociatedFamilySelectionBlocker::RigidProjection
                        }
                        ProjectionRigidity::Neutral => {
                            AssociatedFamilySelectionBlocker::NeutralScrutinee
                        }
                    }))
                }
                _ => Err(AssociatedFamilyMatchFailure::NoMatch),
            },
        }
    }

    pub(super) fn ensure_associated_family_normal_arg_is_capturable(
        arg: &NormalTypeExpr,
    ) -> Result<(), AssociatedFamilyMatchFailure> {
        match arg {
            NormalTypeExpr::NeutralComputationApp { .. } => {
                Err(AssociatedFamilyMatchFailure::Blocked(
                    AssociatedFamilySelectionBlocker::NeutralScrutinee,
                ))
            }
            NormalTypeExpr::Projection { rigidity, .. } => {
                Err(AssociatedFamilyMatchFailure::Blocked(match rigidity {
                    ProjectionRigidity::Rigid => AssociatedFamilySelectionBlocker::RigidProjection,
                    ProjectionRigidity::Neutral => {
                        AssociatedFamilySelectionBlocker::NeutralScrutinee
                    }
                }))
            }
            NormalTypeExpr::PromotedDataConstructorApp { .. } => Err(
                AssociatedFamilyMatchFailure::Blocked(AssociatedFamilySelectionBlocker::Ambiguous),
            ),
            _ => Ok(()),
        }
    }

    pub(super) fn substitute_associated_family_result_expr_from_normal_bindings(
        result: &AssociatedFamilyResultExpr,
        bindings: &BTreeMap<String, NormalTypeExpr>,
    ) -> AssociatedFamilyResultExpr {
        match result {
            AssociatedFamilyResultExpr::Var {
                name,
                source_anchor,
                ..
            } => bindings
                .get(name)
                .cloned()
                .and_then(|normal| {
                    associated_family_result_from_normal(normal, source_anchor.clone()).ok()
                })
                .unwrap_or_else(|| result.clone()),
            AssociatedFamilyResultExpr::Primitive { .. } => result.clone(),
            AssociatedFamilyResultExpr::NominalApp {
                origin,
                visible_name,
                args,
                kind,
                constraint,
                source_anchor,
            } => AssociatedFamilyResultExpr::NominalApp {
                origin: origin.clone(),
                visible_name: visible_name.clone(),
                args: args
                    .iter()
                    .map(|arg| {
                        Self::substitute_associated_family_result_expr_from_normal_bindings(
                            arg, bindings,
                        )
                    })
                    .collect(),
                kind: kind.clone(),
                constraint: constraint.clone(),
                source_anchor: source_anchor.clone(),
            },
            AssociatedFamilyResultExpr::DomainConstructorApp {
                constructor,
                domain,
                args,
                kind,
                constraint,
                source_anchor,
            } => AssociatedFamilyResultExpr::DomainConstructorApp {
                constructor: constructor.clone(),
                domain: domain.clone(),
                args: args
                    .iter()
                    .map(|arg| {
                        Self::substitute_associated_family_result_expr_from_normal_bindings(
                            arg, bindings,
                        )
                    })
                    .collect(),
                kind: kind.clone(),
                constraint: constraint.clone(),
                source_anchor: source_anchor.clone(),
            },
            AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                head,
                interface_args,
                kind,
                constraint,
                source_anchor,
                ..
            } => {
                let interface_args = interface_args
                    .iter()
                    .map(|arg| {
                        Self::substitute_associated_family_result_expr_from_normal_bindings(
                            arg, bindings,
                        )
                    })
                    .collect::<Vec<_>>();
                let rigidity = projection_rigidity_for_associated_family_args(&interface_args);
                AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                    head: head.clone(),
                    interface_args,
                    kind: kind.clone(),
                    constraint: constraint.clone(),
                    rigidity,
                    source_anchor: source_anchor.clone(),
                }
            }
            AssociatedFamilyResultExpr::Projection {
                interface,
                member,
                args,
                kind,
                constraint,
                source_anchor,
                ..
            } => {
                let args = args
                    .iter()
                    .map(|arg| {
                        Self::substitute_associated_family_result_expr_from_normal_bindings(
                            arg, bindings,
                        )
                    })
                    .collect::<Vec<_>>();
                let rigidity = projection_rigidity_for_associated_family_args(&args);
                AssociatedFamilyResultExpr::Projection {
                    interface: interface.clone(),
                    member: member.clone(),
                    args,
                    kind: kind.clone(),
                    constraint: constraint.clone(),
                    rigidity,
                    source_anchor: source_anchor.clone(),
                }
            }
            AssociatedFamilyResultExpr::ComputationHeadApp {
                head,
                args,
                kind,
                constraint,
                source_anchor,
            } => AssociatedFamilyResultExpr::ComputationHeadApp {
                head: head.clone(),
                args: args
                    .iter()
                    .map(|arg| {
                        Self::substitute_associated_family_result_expr_from_normal_bindings(
                            arg, bindings,
                        )
                    })
                    .collect(),
                kind: kind.clone(),
                constraint: constraint.clone(),
                source_anchor: source_anchor.clone(),
            },
        }
    }

    pub(super) fn substitute_associated_family_result_expr(
        result: &AssociatedFamilyResultExpr,
        bindings: &BTreeMap<String, CanonicalTypeExpr>,
    ) -> AssociatedFamilyResultExpr {
        match result {
            AssociatedFamilyResultExpr::Var {
                name,
                source_anchor,
                ..
            } => bindings
                .get(name)
                .cloned()
                .and_then(|canonical| {
                    associated_family_result_from_canonical(canonical, Span::default()).ok()
                })
                .unwrap_or_else(|| AssociatedFamilyResultExpr::Var {
                    name: name.clone(),
                    kind: Kind::Type,
                    constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
                    source_anchor: source_anchor.clone(),
                }),
            AssociatedFamilyResultExpr::Primitive { .. } => result.clone(),
            AssociatedFamilyResultExpr::NominalApp {
                origin,
                visible_name,
                args,
                kind,
                constraint,
                source_anchor,
            } => AssociatedFamilyResultExpr::NominalApp {
                origin: origin.clone(),
                visible_name: visible_name.clone(),
                args: args
                    .iter()
                    .map(|arg| Self::substitute_associated_family_result_expr(arg, bindings))
                    .collect(),
                kind: kind.clone(),
                constraint: constraint.clone(),
                source_anchor: source_anchor.clone(),
            },
            AssociatedFamilyResultExpr::DomainConstructorApp {
                constructor,
                domain,
                args,
                kind,
                constraint,
                source_anchor,
            } => AssociatedFamilyResultExpr::DomainConstructorApp {
                constructor: constructor.clone(),
                domain: domain.clone(),
                args: args
                    .iter()
                    .map(|arg| Self::substitute_associated_family_result_expr(arg, bindings))
                    .collect(),
                kind: kind.clone(),
                constraint: constraint.clone(),
                source_anchor: source_anchor.clone(),
            },
            AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                head,
                interface_args,
                kind,
                constraint,
                rigidity: _,
                source_anchor,
            } => {
                let interface_args = interface_args
                    .iter()
                    .map(|arg| Self::substitute_associated_family_result_expr(arg, bindings))
                    .collect::<Vec<_>>();
                let rigidity = projection_rigidity_for_associated_family_args(&interface_args);
                AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                    head: head.clone(),
                    interface_args,
                    kind: kind.clone(),
                    constraint: constraint.clone(),
                    rigidity,
                    source_anchor: source_anchor.clone(),
                }
            }
            AssociatedFamilyResultExpr::Projection {
                interface,
                member,
                args,
                kind,
                constraint,
                rigidity: _,
                source_anchor,
            } => {
                let args = args
                    .iter()
                    .map(|arg| Self::substitute_associated_family_result_expr(arg, bindings))
                    .collect::<Vec<_>>();
                let rigidity = projection_rigidity_for_associated_family_args(&args);
                AssociatedFamilyResultExpr::Projection {
                    interface: interface.clone(),
                    member: member.clone(),
                    args,
                    kind: kind.clone(),
                    constraint: constraint.clone(),
                    rigidity,
                    source_anchor: source_anchor.clone(),
                }
            }
            AssociatedFamilyResultExpr::ComputationHeadApp {
                head,
                args,
                kind,
                constraint,
                source_anchor,
            } => AssociatedFamilyResultExpr::ComputationHeadApp {
                head: head.clone(),
                args: args
                    .iter()
                    .map(|arg| Self::substitute_associated_family_result_expr(arg, bindings))
                    .collect(),
                kind: kind.clone(),
                constraint: constraint.clone(),
                source_anchor: source_anchor.clone(),
            },
        }
    }

    pub(super) fn associated_family_result_expr_constraint(
        expr: &AssociatedFamilyResultExpr,
    ) -> &AssociatedFamilyResultConstraint {
        match expr {
            AssociatedFamilyResultExpr::Primitive { constraint, .. }
            | AssociatedFamilyResultExpr::Var { constraint, .. }
            | AssociatedFamilyResultExpr::NominalApp { constraint, .. }
            | AssociatedFamilyResultExpr::DomainConstructorApp { constraint, .. }
            | AssociatedFamilyResultExpr::AssociatedFamilyProjection { constraint, .. }
            | AssociatedFamilyResultExpr::Projection { constraint, .. }
            | AssociatedFamilyResultExpr::ComputationHeadApp { constraint, .. } => constraint,
        }
    }

    pub(super) fn associated_family_expr_conforms_to_constraint(
        expr: &AssociatedFamilyResultExpr,
        expected: &AssociatedFamilyResultConstraint,
    ) -> bool {
        match expected {
            AssociatedFamilyResultConstraint::Kind(expected_kind) => {
                matches!(
                    Self::associated_family_result_expr_constraint(expr),
                    AssociatedFamilyResultConstraint::Kind(actual_kind) if actual_kind == expected_kind
                ) || matches!(
                    Self::associated_family_result_expr_constraint(expr),
                    AssociatedFamilyResultConstraint::Domain(_) if expected_kind == &Kind::Type
                )
            }
            AssociatedFamilyResultConstraint::Domain(expected_domain) => match expr {
                AssociatedFamilyResultExpr::DomainConstructorApp {
                    domain, constraint, ..
                } => {
                    domain == expected_domain
                        && matches!(constraint, AssociatedFamilyResultConstraint::Domain(actual) if actual == expected_domain)
                }
                other => matches!(
                    Self::associated_family_result_expr_constraint(other),
                    AssociatedFamilyResultConstraint::Domain(actual) if actual == expected_domain
                ),
            },
        }
    }

    pub(super) fn lower_associated_family_result_expr(
        &self,
        ty: &SurfaceType,
        expected_constraint: &AssociatedFamilyResultConstraint,
        var_constraints: &HashMap<String, AssociatedFamilyResultConstraint>,
        span: Span,
    ) -> Result<AssociatedFamilyResultExpr, TypeEnvError> {
        let Some(expected_domain) = (match expected_constraint {
            AssociatedFamilyResultConstraint::Domain(domain) => Some(domain),
            AssociatedFamilyResultConstraint::Kind(_) => None,
        }) else {
            return self.lower_associated_family_unconstrained_result_expr(
                ty,
                var_constraints,
                span,
            );
        };
        match ty {
            SurfaceType::AssociatedFamilyProjection {
                interface,
                args,
                member,
                span: projection_span,
            } => self.lower_associated_family_projection_result_expr(
                interface,
                args,
                member,
                expected_constraint,
                var_constraints,
                *projection_span,
            ),
            SurfaceType::Name(name) => {
                if let Some((domain, constructor)) =
                    self.find_domain_constructor_cloned(expected_domain, name.as_ref())
                {
                    return self.lower_associated_family_domain_constructor_result(
                        &domain,
                        &constructor,
                        &[],
                        var_constraints,
                        span,
                    );
                }
                if let Some((domain, _)) = self.find_any_domain_constructor(name.as_ref()) {
                    return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                        family: "<impl binding>".to_string(),
                        reason: format!(
                            "marker constructor '{}' belongs to sealed domain '{}', not '{}'",
                            name, domain.exported_name, expected_domain.name
                        ),
                        span,
                    });
                }
                Ok(AssociatedFamilyResultExpr::Var {
                    name: name.to_string(),
                    kind: Kind::Type,
                    constraint: var_constraints
                        .get(name.as_ref())
                        .cloned()
                        .unwrap_or(AssociatedFamilyResultConstraint::Kind(Kind::Type)),
                    source_anchor: span_anchor(span, format!("associated family result {name}")),
                })
            }
            SurfaceType::Constructor { name, args } => {
                if let Some((domain, constructor)) =
                    self.find_domain_constructor_cloned(expected_domain, name.as_ref())
                {
                    return self.lower_associated_family_domain_constructor_result(
                        &domain,
                        &constructor,
                        args,
                        var_constraints,
                        span,
                    );
                }
                if let Some((domain, _)) = self.find_any_domain_constructor(name.as_ref()) {
                    return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                        family: "<impl binding>".to_string(),
                        reason: format!(
                            "marker constructor '{}' belongs to sealed domain '{}', not '{}'",
                            name, domain.exported_name, expected_domain.name
                        ),
                        span,
                    });
                }
                self.lower_surface_type_to_canonical(ty)
                    .map_err(|err| TypeEnvError::InvalidDefinition(format!("{err}"), span))
                    .and_then(|canonical| associated_family_result_from_canonical(canonical, span))
            }
            _ => self
                .lower_surface_type_to_canonical(ty)
                .map_err(|err| TypeEnvError::InvalidDefinition(format!("{err}"), span))
                .and_then(|canonical| associated_family_result_from_canonical(canonical, span)),
        }
    }

    pub(super) fn find_domain_constructor_cloned(
        &self,
        domain_id: &SealedDomainId,
        constructor_name: &str,
    ) -> Option<(SealedDomainSummary, DomainConstructorSummary)> {
        self.find_domain_constructor(domain_id, constructor_name)
            .map(|(domain, constructor)| (domain.clone(), constructor.clone()))
    }

    pub(super) fn lower_associated_family_domain_constructor_result(
        &self,
        domain: &SealedDomainSummary,
        constructor: &DomainConstructorSummary,
        args: &[SurfaceType],
        var_constraints: &HashMap<String, AssociatedFamilyResultConstraint>,
        span: Span,
    ) -> Result<AssociatedFamilyResultExpr, TypeEnvError> {
        if constructor.fields.len() != args.len() {
            return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: "<impl binding>".to_string(),
                reason: format!(
                    "marker constructor '{}' expects {} type arguments, found {}",
                    constructor.exported_name,
                    constructor.fields.len(),
                    args.len()
                ),
                span,
            });
        }
        let args = constructor
            .fields
            .iter()
            .zip(args.iter())
            .map(|(field, arg)| {
                if let Some(field_domain) = &field.domain_constraint {
                    self.lower_associated_family_result_expr(
                        arg,
                        &AssociatedFamilyResultConstraint::Domain(field_domain.clone()),
                        var_constraints,
                        span,
                    )
                } else {
                    self.lower_associated_family_unconstrained_result_expr(
                        arg,
                        var_constraints,
                        span,
                    )
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AssociatedFamilyResultExpr::DomainConstructorApp {
            constructor: constructor.id.clone(),
            domain: domain.id.clone(),
            args,
            kind: Kind::Type,
            constraint: AssociatedFamilyResultConstraint::Domain(domain.id.clone()),
            source_anchor: span_anchor(
                span,
                format!("associated family result {}", constructor.exported_name),
            ),
        })
    }

    pub(super) fn lower_associated_family_unconstrained_result_expr(
        &self,
        ty: &SurfaceType,
        var_constraints: &HashMap<String, AssociatedFamilyResultConstraint>,
        span: Span,
    ) -> Result<AssociatedFamilyResultExpr, TypeEnvError> {
        match ty {
            SurfaceType::AssociatedFamilyProjection {
                interface,
                args,
                member,
                span: projection_span,
            } => self.lower_associated_family_projection_result_expr(
                interface,
                args,
                member,
                &AssociatedFamilyResultConstraint::Kind(Kind::Type),
                var_constraints,
                *projection_span,
            ),
            SurfaceType::Name(name) => {
                if let Some(constraint) = var_constraints.get(name.as_ref()) {
                    Ok(AssociatedFamilyResultExpr::Var {
                        name: name.to_string(),
                        kind: Kind::Type,
                        constraint: constraint.clone(),
                        source_anchor: span_anchor(
                            span,
                            format!("associated family result {name}"),
                        ),
                    })
                } else {
                    self.lower_surface_type_to_canonical(ty)
                        .map_err(|err| TypeEnvError::InvalidDefinition(format!("{err}"), span))
                        .and_then(|canonical| {
                            associated_family_result_from_canonical(canonical, span)
                        })
                }
            }
            _ => self
                .lower_surface_type_to_canonical(ty)
                .map_err(|err| TypeEnvError::InvalidDefinition(format!("{err}"), span))
                .and_then(|canonical| associated_family_result_from_canonical(canonical, span)),
        }
    }

    pub(super) fn associated_family_constraint_for_domain(
        domain: Option<&SealedDomainId>,
    ) -> AssociatedFamilyResultConstraint {
        domain.map_or(
            AssociatedFamilyResultConstraint::Kind(Kind::Type),
            |domain| AssociatedFamilyResultConstraint::Domain(domain.clone()),
        )
    }

    pub(super) fn lower_associated_family_pattern(
        &self,
        ty: &SurfaceType,
        expected_domain: Option<&SealedDomainId>,
        var_constraints: &HashMap<String, AssociatedFamilyResultConstraint>,
        span: Span,
    ) -> Result<AssociatedFamilyPattern, TypeEnvError> {
        let constraint = Self::associated_family_constraint_for_domain(expected_domain);
        match ty {
            SurfaceType::Name(name) => {
                if let Some(domain_id) = expected_domain {
                    if let Some((domain, constructor)) =
                        self.find_domain_constructor_cloned(domain_id, name.as_ref())
                    {
                        return self.lower_associated_family_domain_constructor_pattern(
                            &domain,
                            &constructor,
                            &[],
                            var_constraints,
                            span,
                        );
                    }
                    if let Some((domain, _)) = self.find_any_domain_constructor(name.as_ref()) {
                        return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                            family: "<impl head>".to_string(),
                            reason: format!(
                                "marker constructor '{}' belongs to sealed domain '{}', not '{}'",
                                name, domain.exported_name, domain_id.name
                            ),
                            span,
                        });
                    }
                }
                if let Some(var_constraint) = var_constraints.get(name.as_ref()) {
                    return Ok(AssociatedFamilyPattern::Var {
                        name: name.to_string(),
                        constraint: var_constraint.clone(),
                        source_anchor: span_anchor(
                            span,
                            format!("associated family pattern {name}"),
                        ),
                    });
                }
                let canonical = self
                    .lower_surface_type_to_canonical(ty)
                    .map_err(|err| TypeEnvError::InvalidDefinition(format!("{err}"), span))?;
                Self::associated_family_pattern_from_canonical(
                    canonical,
                    &constraint,
                    var_constraints,
                    span,
                )
            }
            SurfaceType::Constructor { name, args } => {
                if let Some(domain_id) = expected_domain {
                    if let Some((domain, constructor)) =
                        self.find_domain_constructor_cloned(domain_id, name.as_ref())
                    {
                        return self.lower_associated_family_domain_constructor_pattern(
                            &domain,
                            &constructor,
                            args,
                            var_constraints,
                            span,
                        );
                    }
                    return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                        family: "<impl head>".to_string(),
                        reason: format!(
                            "unknown marker constructor '{}' for sealed domain '{}'",
                            name, domain_id.name
                        ),
                        span,
                    });
                }
                let canonical = self
                    .lower_surface_type_to_canonical(ty)
                    .map_err(|err| TypeEnvError::InvalidDefinition(format!("{err}"), span))?;
                Self::associated_family_pattern_from_canonical(
                    canonical,
                    &constraint,
                    var_constraints,
                    span,
                )
            }
            SurfaceType::List(item) => {
                if expected_domain.is_some() {
                    return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                        family: "<impl head>".to_string(),
                        reason: "list pattern requires an unconstrained Type interface parameter"
                            .to_string(),
                        span,
                    });
                }
                let list_ty = SurfaceType::Constructor {
                    name: "List".into(),
                    args: vec![item.as_ref().clone()],
                };
                let canonical = self
                    .lower_surface_type_to_canonical(&list_ty)
                    .map_err(|err| TypeEnvError::InvalidDefinition(format!("{err}"), span))?;
                Self::associated_family_pattern_from_canonical(
                    canonical,
                    &constraint,
                    var_constraints,
                    span,
                )
            }
            _ => Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: "<impl head>".to_string(),
                reason: format!(
                    "unsupported associated-family impl-head pattern '{}'",
                    surface_projection_base_spelling(ty)
                ),
                span,
            }),
        }
    }

    pub(super) fn associated_family_pattern_from_canonical(
        canonical: CanonicalTypeExpr,
        constraint: &AssociatedFamilyResultConstraint,
        var_constraints: &HashMap<String, AssociatedFamilyResultConstraint>,
        span: Span,
    ) -> Result<AssociatedFamilyPattern, TypeEnvError> {
        match canonical {
            CanonicalTypeExpr::Primitive(name) => Ok(AssociatedFamilyPattern::Primitive {
                name: name.clone(),
                constraint: constraint.clone(),
                source_anchor: span_anchor(
                    span,
                    format!("associated family primitive pattern {name}"),
                ),
            }),
            CanonicalTypeExpr::Var(name) => Ok(AssociatedFamilyPattern::Var {
                name: name.clone(),
                constraint: var_constraints
                    .get(name.as_str())
                    .cloned()
                    .unwrap_or_else(|| constraint.clone()),
                source_anchor: span_anchor(span, format!("associated family pattern {name}")),
            }),
            CanonicalTypeExpr::NominalApp {
                origin,
                visible_name,
                args,
                kind: _,
            } => Ok(AssociatedFamilyPattern::NominalApp {
                origin,
                visible_name: visible_name.clone(),
                args: args
                    .into_iter()
                    .map(|arg| {
                        Self::associated_family_pattern_from_canonical(
                            arg,
                            constraint,
                            var_constraints,
                            span,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                constraint: constraint.clone(),
                source_anchor: span_anchor(
                    span,
                    format!("associated family pattern {visible_name}"),
                ),
            }),
            CanonicalTypeExpr::Projection { .. }
            | CanonicalTypeExpr::ComputationHeadApp { .. }
            | CanonicalTypeExpr::PromotedDataConstructorApp(_) => {
                Ok(AssociatedFamilyPattern::Wildcard {
                    constraint: constraint.clone(),
                    source_anchor: span_anchor(span, "associated family unsupported pattern"),
                })
            }
            CanonicalTypeExpr::ConstructorVariableApp(app) => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "constructor-variable application '{}' cannot be lowered to an associated-family pattern until TASK-907 tracks constructor variables",
                    app.constructor.name
                ),
                span,
            )),
        }
    }

    pub(super) fn lower_associated_family_domain_constructor_pattern(
        &self,
        domain: &SealedDomainSummary,
        constructor: &DomainConstructorSummary,
        args: &[SurfaceType],
        var_constraints: &HashMap<String, AssociatedFamilyResultConstraint>,
        span: Span,
    ) -> Result<AssociatedFamilyPattern, TypeEnvError> {
        if constructor.fields.len() != args.len() {
            return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: "<impl head>".to_string(),
                reason: format!(
                    "marker constructor '{}' expects {} type arguments, found {}",
                    constructor.exported_name,
                    constructor.fields.len(),
                    args.len()
                ),
                span,
            });
        }
        let fields = constructor
            .fields
            .iter()
            .zip(args.iter())
            .map(|(field, arg)| {
                self.lower_associated_family_pattern(
                    arg,
                    field.domain_constraint.as_ref(),
                    var_constraints,
                    span,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AssociatedFamilyPattern::DomainConstructor {
            constructor: Box::new(constructor.id.clone()),
            domain: Box::new(domain.id.clone()),
            fields,
            constraint: AssociatedFamilyResultConstraint::Domain(domain.id.clone()),
            source_anchor: span_anchor(
                span,
                format!("associated family pattern {}", constructor.exported_name),
            ),
        })
    }

    pub(super) fn associated_family_pattern_spines_overlap(
        left: &[AssociatedFamilyPattern],
        right: &[AssociatedFamilyPattern],
    ) -> bool {
        left.len() == right.len()
            && left
                .iter()
                .zip(right.iter())
                .all(|(left, right)| Self::associated_family_patterns_overlap(left, right))
    }

    pub(super) fn associated_family_patterns_overlap(
        left: &AssociatedFamilyPattern,
        right: &AssociatedFamilyPattern,
    ) -> bool {
        match (left, right) {
            (
                AssociatedFamilyPattern::DomainConstructor {
                    constructor: left_constructor,
                    domain: left_domain,
                    fields: left_fields,
                    ..
                },
                AssociatedFamilyPattern::DomainConstructor {
                    constructor: right_constructor,
                    domain: right_domain,
                    fields: right_fields,
                    ..
                },
            ) => {
                left_constructor == right_constructor
                    && left_domain == right_domain
                    && Self::associated_family_pattern_spines_overlap(left_fields, right_fields)
            }
            (
                AssociatedFamilyPattern::NominalApp {
                    origin: left_origin,
                    visible_name: left_name,
                    args: left_args,
                    ..
                },
                AssociatedFamilyPattern::NominalApp {
                    origin: right_origin,
                    visible_name: right_name,
                    args: right_args,
                    ..
                },
            ) => {
                left_origin == right_origin
                    && left_name == right_name
                    && Self::associated_family_pattern_spines_overlap(left_args, right_args)
            }
            (
                AssociatedFamilyPattern::Primitive {
                    name: left_name, ..
                },
                AssociatedFamilyPattern::Primitive {
                    name: right_name, ..
                },
            ) => left_name == right_name,
            (AssociatedFamilyPattern::Primitive { .. }, AssociatedFamilyPattern::Var { .. })
            | (AssociatedFamilyPattern::Var { .. }, AssociatedFamilyPattern::Primitive { .. })
            | (
                AssociatedFamilyPattern::Primitive { .. },
                AssociatedFamilyPattern::Wildcard { .. },
            )
            | (
                AssociatedFamilyPattern::Wildcard { .. },
                AssociatedFamilyPattern::Primitive { .. },
            ) => true,
            (AssociatedFamilyPattern::Primitive { .. }, _)
            | (_, AssociatedFamilyPattern::Primitive { .. }) => false,
            (
                AssociatedFamilyPattern::DomainConstructor { .. },
                AssociatedFamilyPattern::NominalApp { .. },
            )
            | (
                AssociatedFamilyPattern::NominalApp { .. },
                AssociatedFamilyPattern::DomainConstructor { .. },
            ) => false,
            (
                AssociatedFamilyPattern::DomainConstructor { .. },
                AssociatedFamilyPattern::Var { .. },
            )
            | (
                AssociatedFamilyPattern::Var { .. },
                AssociatedFamilyPattern::DomainConstructor { .. },
            )
            | (
                AssociatedFamilyPattern::DomainConstructor { .. },
                AssociatedFamilyPattern::Wildcard { .. },
            )
            | (
                AssociatedFamilyPattern::Wildcard { .. },
                AssociatedFamilyPattern::DomainConstructor { .. },
            )
            | (AssociatedFamilyPattern::Var { .. }, AssociatedFamilyPattern::Var { .. })
            | (AssociatedFamilyPattern::Var { .. }, AssociatedFamilyPattern::Wildcard { .. })
            | (AssociatedFamilyPattern::Wildcard { .. }, AssociatedFamilyPattern::Var { .. })
            | (AssociatedFamilyPattern::NominalApp { .. }, AssociatedFamilyPattern::Var { .. })
            | (AssociatedFamilyPattern::Var { .. }, AssociatedFamilyPattern::NominalApp { .. })
            | (
                AssociatedFamilyPattern::NominalApp { .. },
                AssociatedFamilyPattern::Wildcard { .. },
            )
            | (
                AssociatedFamilyPattern::Wildcard { .. },
                AssociatedFamilyPattern::NominalApp { .. },
            )
            | (
                AssociatedFamilyPattern::Wildcard { .. },
                AssociatedFamilyPattern::Wildcard { .. },
            ) => true,
        }
    }

    /// Enter a workflow context at the given effect level.
    ///
    /// All `Expr::FnDef` nodes type-checked in this environment (or any child
    /// derived from it via `extend()`) will be assigned `Type::Fun(…, effect)`
    /// instead of the pure `Type::Fn(…)`.
    pub fn set_workflow_effect(&mut self, effect: ash_core::Effect) {
        self.workflow_effect = Some(effect);
    }

    /// Create a new type environment with builtin types registered
    #[must_use]
    pub fn with_builtin_types() -> Self {
        let mut env = Self::new();
        env.add_builtin_types();
        env
    }

    /// Pre-declare a type name by inserting a placeholder into `ast_types`.
    /// This allows `resolve_type` to find the name during sibling type registration.
    /// The placeholder will be upgraded by a subsequent `register_type` call.
    pub fn declare_type_name(&mut self, name: &str) {
        let placeholder = TypeDef {
            name: name.to_owned(),
            params: vec![],
            body: TypeBody::Struct(vec![]), // minimal placeholder: empty struct
            visibility: ash_core::ast::Visibility::Public,
            builtin: false,
        };
        self.ast_types.entry(name.to_owned()).or_insert(placeholder);
        self.type_declaration_states
            .entry(name.to_owned())
            .or_insert(TypeDeclarationState::Placeholder);
    }

    pub(super) fn is_placeholder_name(&self, name: &str) -> bool {
        matches!(
            self.type_declaration_states.get(name),
            Some(TypeDeclarationState::Placeholder)
        )
    }

    pub(super) fn is_identity_only_name(&self, name: &str) -> bool {
        matches!(
            self.type_declaration_states.get(name),
            Some(TypeDeclarationState::IdentityOnly)
        )
    }

    /// Register a type definition without exposing its constructors or
    /// representation symbols.
    pub fn register_type_identity(&mut self, def: &TypeDef) -> Result<(), TypeEnvError> {
        let type_name = def.name.clone();

        if self.ast_types.contains_key(&type_name) {
            // Allow upgrading an explicit placeholder, or replacing an
            // identity-only summary declaration with the same imported fallback
            // definition.
            if !self.is_placeholder_name(&type_name) && !self.is_identity_only_name(&type_name) {
                return Err(TypeEnvError::DuplicateType(type_name, Span::default()));
            }
            // Placeholder/identity-only entry will be replaced below.
        }

        // Convert to internal TypeInfo for type checking
        let type_info = convert_type_def(def, self).map_err(|e| {
            TypeEnvError::InvalidDefinition(format!("type '{}': {e}", def.name), Span::default())
        })?;

        self.ast_types.insert(type_name.clone(), def.clone());
        self.type_info.insert(type_name, type_info);
        self.type_declaration_states
            .insert(def.name.clone(), TypeDeclarationState::Full);
        Ok(())
    }

    /// Expose constructors/representation for a previously-registered type.
    pub fn expose_type_representation(&mut self, name: &str) -> Result<(), TypeEnvError> {
        let Some(type_info) = self.type_info.get(name).cloned() else {
            return Err(TypeEnvError::TypeNotFound(
                name.to_string(),
                Span::default(),
            ));
        };

        match type_info {
            TypeInfo::Enum { variants, .. } => {
                for (index, variant) in variants.iter().enumerate() {
                    self.constructors
                        .insert(variant.name.clone(), (name.to_string(), index));
                }
            }
            TypeInfo::Struct { fields, .. } if matches!(fields.as_slice(), [(field_name, _)] if field_name == "__alias_target") =>
            {
                self.transparent_aliases.insert(name.to_string());
            }
            TypeInfo::Struct { .. } => {}
        }

        Ok(())
    }

    #[must_use]
    pub fn transparent_alias_target(&self, name: &QualifiedName, args: &[Type]) -> Option<Type> {
        if !self.transparent_aliases.contains(name.name.as_str()) {
            return None;
        }

        match self.unfold_constructor(name, args).ok()? {
            UnfoldedBody::Struct(fields) => match fields.as_slice() {
                [(field_name, target)] if field_name == "__alias_target" => Some(target.clone()),
                _ => None,
            },
            UnfoldedBody::Enum(_) => None,
        }
    }

    /// Register a type definition and its constructors from AST TypeDef
    pub fn register_type(&mut self, def: &TypeDef) -> Result<(), TypeEnvError> {
        self.register_type_identity(def)?;
        self.type_alias_identities
            .entry(def.name.clone())
            .or_insert_with(|| fallback_canonical_type_decl_id(&def.name));
        if let Some(identity) = self.type_alias_identities.get(&def.name).cloned() {
            self.canonical_type_names
                .entry(identity)
                .or_insert_with(|| def.name.clone());
        }
        self.expose_type_representation(&def.name)
    }

    pub(super) fn existing_summary_contract_conflicts(
        &self,
        visible_name: &str,
        existing: &TypeDef,
        summary: &TypeDeclSummary,
    ) -> bool {
        if existing.params != summary.params || existing.visibility != summary.visibility {
            return true;
        }

        match self.type_declaration_states.get(visible_name) {
            Some(TypeDeclarationState::Full) => match &summary.representation {
                TypeRepresentationSummary::Exposed(body) => existing.body != *body,
                TypeRepresentationSummary::Opaque { builtin: true } => !existing.builtin,
                TypeRepresentationSummary::Opaque { builtin: false } => true,
            },
            Some(TypeDeclarationState::IdentityOnly) => false,
            Some(TypeDeclarationState::Placeholder) | None => false,
        }
    }

    pub(super) fn declare_summary_type_identity(
        &mut self,
        summary: &TypeDeclSummary,
    ) -> Result<(), TypeEnvError> {
        let visible_name = summary.exported_name.clone();
        let conflicting_existing_summary = self
            .canonical_type_names
            .get(&summary.id)
            .cloned()
            .is_some_and(|existing_visible_name| {
                existing_visible_name != visible_name
                    && self
                        .ast_types
                        .get(&existing_visible_name)
                        .is_some_and(|existing| {
                            self.existing_summary_contract_conflicts(
                                &existing_visible_name,
                                existing,
                                summary,
                            )
                        })
            });
        if conflicting_existing_summary {
            return Err(TypeEnvError::InvalidDefinition(
                conflicting_summary_contract_diagnostic(&visible_name),
                Span::default(),
            ));
        }
        let fallback_compatible_builtin_identity = self
            .type_alias_identities
            .get(&visible_name)
            .is_some_and(|existing| existing == &fallback_canonical_type_decl_id(&visible_name))
            && self.ast_types.get(&visible_name).is_some_and(|existing| {
                (is_builtin_prelude_ordinary_type_compatibility_name(&visible_name)
                    && !self.existing_summary_contract_conflicts(&visible_name, existing, summary))
                    || (existing.builtin
                        && matches!(
                            summary.representation,
                            TypeRepresentationSummary::Opaque { .. }
                        ))
            });
        match self.type_alias_identities.get(&visible_name) {
            Some(existing) if existing != &summary.id && !fallback_compatible_builtin_identity => {
                return Err(TypeEnvError::InvalidDefinition(
                    duplicate_summary_identity_diagnostic(&visible_name, existing, summary),
                    Span::default(),
                ));
            }
            _ => {}
        }
        if let Some(existing) = self.ast_types.get(&visible_name) {
            let existing_identity = self.type_alias_identities.get(&visible_name);
            if !self.is_placeholder_name(&visible_name) && existing_identity != Some(&summary.id) {
                if fallback_compatible_builtin_identity {
                    self.type_alias_identities
                        .insert(visible_name.clone(), summary.id.clone());
                    self.canonical_type_names
                        .entry(summary.id.clone())
                        .or_insert(visible_name);
                    return Ok(());
                }
                if matches!(
                    (&summary.representation, existing.builtin),
                    (TypeRepresentationSummary::Opaque { builtin: true }, true)
                ) {
                    self.type_alias_identities
                        .insert(visible_name.clone(), summary.id.clone());
                    self.canonical_type_names
                        .entry(summary.id.clone())
                        .or_insert(visible_name);
                    return Ok(());
                }
                if (existing_identity.is_none()
                    || existing_identity == Some(&fallback_canonical_type_decl_id(&visible_name)))
                    && is_builtin_prelude_ordinary_type_compatibility_name(&visible_name)
                    && !self.existing_summary_contract_conflicts(&visible_name, existing, summary)
                {
                    self.type_alias_identities
                        .insert(visible_name.clone(), summary.id.clone());
                    self.canonical_type_names
                        .entry(summary.id.clone())
                        .or_insert(visible_name);
                    return Ok(());
                }
                if let Some(existing_identity) = existing_identity {
                    return Err(TypeEnvError::InvalidDefinition(
                        duplicate_summary_identity_diagnostic(
                            &visible_name,
                            existing_identity,
                            summary,
                        ),
                        Span::default(),
                    ));
                }
                return Err(TypeEnvError::DuplicateType(visible_name, Span::default()));
            }
            if existing_identity == Some(&summary.id)
                && self.existing_summary_contract_conflicts(&visible_name, existing, summary)
            {
                return Err(TypeEnvError::InvalidDefinition(
                    conflicting_summary_contract_diagnostic(&visible_name),
                    Span::default(),
                ));
            }
        }

        let identity_def = TypeDef {
            name: visible_name.clone(),
            params: summary.params.clone(),
            body: TypeBody::Struct(vec![]),
            visibility: summary.visibility,
            builtin: matches!(
                summary.representation,
                TypeRepresentationSummary::Opaque { builtin: true }
            ),
        };
        self.ast_types.insert(visible_name.clone(), identity_def);
        let type_info = TypeInfo::Struct {
            name: visible_name.clone(),
            params: summary.params.iter().map(|_| TypeVar::fresh()).collect(),
            fields: vec![],
        };
        self.type_info.insert(visible_name.clone(), type_info);
        self.type_declaration_states
            .insert(visible_name.clone(), TypeDeclarationState::IdentityOnly);
        self.type_alias_identities
            .insert(visible_name.clone(), summary.id.clone());
        self.canonical_type_names
            .entry(summary.id.clone())
            .or_insert(visible_name);
        Ok(())
    }

    pub(super) fn expose_summary_type_representation(
        &mut self,
        ty: &TypeDeclSummary,
        constructors: &[ConstructorSummary],
    ) -> Result<(), TypeEnvError> {
        let visible_name = ty.exported_name.as_str();
        let Some(type_info) = self.type_info.get(visible_name).cloned() else {
            return Err(TypeEnvError::TypeNotFound(
                visible_name.to_string(),
                Span::default(),
            ));
        };

        match type_info {
            TypeInfo::Enum { variants, .. } => {
                let matching_constructors = constructors
                    .iter()
                    .filter(|constructor| constructor.parent == ty.id)
                    .collect::<Vec<_>>();
                if !matching_constructors.is_empty() {
                    for constructor in &matching_constructors {
                        let Some((index, _)) = variants
                            .iter()
                            .enumerate()
                            .find(|(_, variant)| variant.name == constructor.id.name)
                        else {
                            return Err(TypeEnvError::InvalidDefinition(
                                format!(
                                    "constructor summary '{}' does not match any exposed variant on type '{}'",
                                    constructor.exported_name, visible_name
                                ),
                                Span::default(),
                            ));
                        };
                        match self.constructors.get(&constructor.exported_name) {
                            Some((existing_type, existing_index))
                                if existing_type != visible_name || *existing_index != index =>
                            {
                                return Err(TypeEnvError::InvalidDefinition(
                                    format!(
                                        "duplicate exported constructor summary '{}' conflicts with an existing constructor binding",
                                        constructor.exported_name
                                    ),
                                    Span::default(),
                                ));
                            }
                            _ => {}
                        }
                    }
                }
                for constructor in matching_constructors {
                    let Some((index, _)) = variants
                        .iter()
                        .enumerate()
                        .find(|(_, variant)| variant.name == constructor.id.name)
                    else {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "constructor summary '{}' does not match any exposed variant on type '{}'",
                                constructor.exported_name, visible_name
                            ),
                            Span::default(),
                        ));
                    };
                    match self.constructors.get(&constructor.exported_name) {
                        Some((existing_type, existing_index))
                            if existing_type != visible_name || *existing_index != index =>
                        {
                            return Err(TypeEnvError::InvalidDefinition(
                                format!(
                                    "duplicate exported constructor summary '{}' conflicts with an existing constructor binding",
                                    constructor.exported_name
                                ),
                                Span::default(),
                            ));
                        }
                        _ => {}
                    }
                    self.constructors.insert(
                        constructor.exported_name.clone(),
                        (visible_name.to_string(), index),
                    );
                }
            }
            TypeInfo::Struct { fields, .. } if matches!(fields.as_slice(), [(field_name, _)] if field_name == "__alias_target") =>
            {
                self.transparent_aliases.insert(visible_name.to_string());
            }
            TypeInfo::Struct { .. } => {
                if constructors
                    .iter()
                    .any(|constructor| constructor.parent == ty.id)
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "constructor summaries for '{}' require an exposed enum body",
                            visible_name
                        ),
                        Span::default(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Register all visible ordinary type identities from a module semantic summary first,
    /// then validate/expose public representations in a second pass.
    pub fn register_module_semantic_summary(
        &mut self,
        summary: &ModuleSemanticSummary,
    ) -> Result<(), TypeEnvError> {
        self.register_module_semantic_summaries(std::slice::from_ref(summary))
    }
}
