use super::*;

impl TypeEnv {
    /// Resolve a Phase 177 operation-row identity of the form `Target::operation`.
    ///
    /// This query is read-only: it proves whether the row item names an already
    /// registered interface method or concrete impl method, and does not search
    /// handlers, providers, or runtime authority.
    #[must_use]
    pub fn resolve_operation_row_identity(
        &self,
        target: &str,
        method: &str,
    ) -> OperationRowIdentityResolution {
        if let Some(interface) = self.interfaces.get(target)
            && interface.methods.contains_key(method)
        {
            return OperationRowIdentityResolution::InterfaceQualified {
                interface: target.to_string(),
                method: method.to_string(),
                suggestion: self
                    .impls
                    .iter()
                    .filter(|scheme| scheme.interface == target)
                    .find_map(|scheme| {
                        let has_method = scheme
                            .methods
                            .iter()
                            .any(|impl_method| impl_method.name == method);
                        has_method
                            .then(|| {
                                scheme_concrete_target_name(scheme)
                                    .map(|impl_type| format!("{impl_type}::{method}"))
                            })
                            .flatten()
                    })
                    .unwrap_or_else(|| format!("<impl>::{method}")),
            };
        }

        let matching_target_schemes = self
            .impls
            .iter()
            .filter(|scheme| scheme_concrete_target_name(scheme).as_deref() == Some(target))
            .collect::<Vec<_>>();

        if matching_target_schemes.is_empty() {
            return OperationRowIdentityResolution::UnknownImplType {
                impl_type: target.to_string(),
            };
        }

        if let Some(scheme) = matching_target_schemes.iter().find(|scheme| {
            scheme
                .methods
                .iter()
                .any(|impl_method| impl_method.name == method)
        }) {
            return OperationRowIdentityResolution::ConcreteImpl {
                impl_type: target.to_string(),
                interface: scheme.interface.clone(),
                method: method.to_string(),
            };
        }

        OperationRowIdentityResolution::UnknownMethod {
            impl_type: target.to_string(),
            method: method.to_string(),
            candidates: matching_target_schemes
                .into_iter()
                .map(render_impl_scheme_head)
                .collect(),
        }
    }

    /// Create a new child environment with this as parent
    ///
    /// Used for block scoping - variables bound in the child
    /// are not visible in the parent. The workflow effect context is inherited
    /// so that closures nested inside a workflow body still get `Type::Fun`.
    #[must_use]
    pub fn extend(&self) -> Self {
        Self {
            ast_types: self.ast_types.clone(),
            type_info: self.type_info.clone(),
            constructors: self.constructors.clone(),
            transparent_aliases: self.transparent_aliases.clone(),
            type_declaration_states: self.type_declaration_states.clone(),
            type_alias_identities: self.type_alias_identities.clone(),
            canonical_type_names: self.canonical_type_names.clone(),
            interface_identity_aliases: self.interface_identity_aliases.clone(),
            interface_identity_alias_is_imported: self.interface_identity_alias_is_imported.clone(),
            canonical_interface_names: self.canonical_interface_names.clone(),
            local_interface_arities: self.local_interface_arities.clone(),
            known_interface_identities: self.known_interface_identities.clone(),
            associated_member_identity_aliases: self.associated_member_identity_aliases.clone(),
            associated_member_identity_alias_is_imported: self
                .associated_member_identity_alias_is_imported
                .clone(),
            known_associated_member_identities: self.known_associated_member_identities.clone(),
            interfaces: self.interfaces.clone(),
            capability_interfaces: self.capability_interfaces.clone(),
            resource_types: self.resource_types.clone(),
            capability_implementations: self.capability_implementations.clone(),
            capability_bindings: self.capability_bindings.clone(),
            impls: self.impls.clone(),
            proposition_assumptions: self.proposition_assumptions.clone(),
            proposition_obligations: self.proposition_obligations.clone(),
            proposition_predicate_aliases: self.proposition_predicate_aliases.clone(),
            proposition_predicates: self.proposition_predicates.clone(),
            type_var_interface_bounds: self.type_var_interface_bounds.clone(),
            type_parameter_kinds: self.type_parameter_kinds.clone(),
            variables: HashMap::with_capacity(10),
            contract_intrinsics: self.contract_intrinsics.clone(),
            public_workflow_summaries: self.public_workflow_summaries.clone(),
            fn_contracts: self.fn_contracts.clone(),
            capability_symbols: self.capability_symbols.clone(),
            parent: Some(Box::new(self.clone())),
            providers: self.providers.clone(),
            sealed_domain_identities: self.sealed_domain_identities.clone(),
            sealed_domain_aliases: self.sealed_domain_aliases.clone(),
            sealed_domain_summaries: self.sealed_domain_summaries.clone(),
            promoted_data_kind_identities: self.promoted_data_kind_identities.clone(),
            promoted_data_kind_aliases: self.promoted_data_kind_aliases.clone(),
            promoted_data_kind_summaries: self.promoted_data_kind_summaries.clone(),
            promoted_constructor_summaries: self.promoted_constructor_summaries.clone(),
            promoted_constructor_kinds: self.promoted_constructor_kinds.clone(),
            local_type_function_heads: self.local_type_function_heads.clone(),
            local_type_functions: self.local_type_functions.clone(),
            current_module_identity: self.current_module_identity.clone(),
            associated_family_declarations: self.associated_family_declarations.clone(),
            associated_family_name_index: self.associated_family_name_index.clone(),
            associated_family_schemes: self.associated_family_schemes.clone(),
            ambient_effect: self.ambient_effect,
        }
    }

    /// Check if an interface is registered.
    pub fn has_interface(&self, name: &str) -> bool {
        self.interfaces.contains_key(name)
    }

    /// Look up a registered interface.
    pub fn lookup_interface(&self, name: &str) -> Option<&InterfaceInfo> {
        self.interfaces.get(name)
    }

    /// Resolve explicit interface evidence by matching the registered impl head spine.
    pub fn resolve_interface_evidence(
        &self,
        interface: &str,
        args: &[SurfaceType],
    ) -> Result<&ImplScheme, TypeEnvError> {
        let interface_info = self.interfaces.get(interface).ok_or_else(|| {
            TypeEnvError::MissingInterface(interface.to_string(), Span::default())
        })?;
        if interface_info.type_params.len() != args.len() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "interface '{}' expects {} type parameters, but evidence lookup provides {}",
                    interface,
                    interface_info.type_params.len(),
                    args.len()
                ),
                Span::default(),
            ));
        }

        let evidence_args =
            self.lower_interface_evidence_args(interface, interface_info, args, &HashMap::new())?;
        let mut matches = self.impls.iter().filter(|scheme| {
            scheme.interface == interface
                && interface_evidence_args_match(
                    &scheme.head_args,
                    &evidence_args,
                    !scheme.type_params.is_empty(),
                )
        });
        let first = matches.next().ok_or_else(|| TypeEnvError::MissingImpl {
            interface: interface.to_string(),
            ty: render_interface_evidence_key(interface, &evidence_args),
            span: Span::default(),
        })?;
        if matches.next().is_some() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "ambiguous evidence for {}",
                    render_interface_evidence_key(interface, &evidence_args)
                ),
                Span::default(),
            ));
        }
        Ok(first)
    }

    /// Check if a capability interface is registered.
    pub fn has_capability_interface(&self, name: &str) -> bool {
        self.capability_interfaces.contains_key(name)
    }

    /// Look up a registered capability interface.
    pub fn lookup_capability_interface(&self, name: &str) -> Option<&CapabilityInterfaceInfo> {
        self.capability_interfaces.get(name)
    }

    /// Look up a registered capability operation signature.
    pub fn lookup_capability_operation(
        &self,
        interface: &str,
        operation: &str,
    ) -> Option<&CapabilityOperationInfo> {
        self.capability_interfaces
            .get(interface)
            .and_then(|info| info.operations.get(operation))
    }

    /// Check if a capability implementation is registered.
    pub fn has_capability_implementation(&self, name: &str) -> bool {
        self.capability_implementations.contains_key(name)
    }

    /// Look up a registered capability implementation.
    pub fn lookup_capability_implementation(
        &self,
        name: &str,
    ) -> Option<&CapabilityImplementationInfo> {
        self.capability_implementations.get(name)
    }

    /// Register a workflow-admitted capability binding for operation-call resolution.
    pub fn register_capability_binding(&mut self, binding: CapabilityBindingInfo) {
        self.capability_bindings
            .insert(binding.name.clone(), binding);
    }

    /// Look up a workflow-admitted capability binding by local binding name.
    pub fn lookup_capability_binding(&self, name: &str) -> Option<&CapabilityBindingInfo> {
        self.capability_bindings
            .get(name)
            .or_else(|| self.parent.as_ref()?.lookup_capability_binding(name))
    }

    /// Check whether a workflow-admitted capability binding exists.
    pub fn has_capability_binding(&self, name: &str) -> bool {
        self.lookup_capability_binding(name).is_some()
    }

    /// Return local workflow-admitted capability binding names.
    pub fn capability_binding_names(&self) -> Vec<String> {
        self.capability_bindings.keys().cloned().collect()
    }

    /// Return all registered impl schemes.
    pub fn impl_schemes(&self) -> &[ImplScheme] {
        &self.impls
    }

    pub(super) fn type_var_has_interface_bound(&self, var: TypeVar, interface: &str) -> bool {
        self.type_var_interface_bounds
            .get(&var)
            .is_some_and(|bounds| {
                bounds
                    .iter()
                    .any(|bound| self.interface_entails_interface(bound, interface))
            })
            || self
                .parent
                .as_ref()
                .is_some_and(|parent| parent.type_var_has_interface_bound(var, interface))
    }

    pub fn normalize_associated_types(
        &self,
        ty: &Type,
        scheme: &ImplScheme,
        subst: &Substitution,
    ) -> Result<Type, TypeEnvError> {
        match ty {
            Type::Associated {
                interface,
                base: _,
                name,
            } => {
                if scheme.interface != *interface {
                    return Err(TypeEnvError::MismatchedProjectionInterface {
                        expected: scheme.interface.clone(),
                        found: interface.clone(),
                        span: Span::default(),
                    });
                }
                let binding = scheme.associated_type_bindings.get(name).ok_or_else(|| {
                    TypeEnvError::MissingAssociatedType {
                        interface: interface.clone(),
                        name: name.clone(),
                        span: Span::default(),
                    }
                })?;
                let normalized = subst.apply(binding);
                self.normalize_associated_types(&normalized, scheme, subst)
            }
            Type::Constructor { name, args, kind } => {
                let norm_args = args
                    .iter()
                    .map(|a| self.normalize_associated_types(a, scheme, subst))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Type::Constructor {
                    name: name.clone(),
                    args: norm_args,
                    kind: kind.clone(),
                })
            }
            Type::Fun(params, ret, effect) => {
                let norm_params = params
                    .iter()
                    .map(|p| self.normalize_associated_types(p, scheme, subst))
                    .collect::<Result<Vec<_>, _>>()?;
                let norm_ret = self.normalize_associated_types(ret, scheme, subst)?;
                Ok(Type::Fun(norm_params, Box::new(norm_ret), *effect))
            }
            Type::Fn(params, ret) => {
                let norm_params = params
                    .iter()
                    .map(|p| self.normalize_associated_types(p, scheme, subst))
                    .collect::<Result<Vec<_>, _>>()?;
                let norm_ret = self.normalize_associated_types(ret, scheme, subst)?;
                Ok(Type::Fn(norm_params, Box::new(norm_ret)))
            }
            Type::List(inner) => Ok(Type::List(Box::new(
                self.normalize_associated_types(inner, scheme, subst)?,
            ))),
            Type::Record(fields) => {
                let norm_fields = fields
                    .iter()
                    .map(|(n, t)| {
                        Ok((
                            n.clone(),
                            self.normalize_associated_types(t, scheme, subst)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Type::Record(norm_fields))
            }
            other => Ok(other.clone()),
        }
    }

    /// Resolve a canonical `Interface::method(value)` call.
    pub fn resolve_interface_method_call(
        &self,
        interface: &str,
        method: &str,
        arg_types: &[Type],
    ) -> Result<Type, TypeEnvError> {
        let (selected, scheme) = match self.select_impl_scheme(interface, method, arg_types) {
            Ok(selected) => selected,
            Err(err) => {
                if let Some(return_ty) = self
                    .resolve_interface_method_call_from_generic_bound(interface, method, arg_types)
                {
                    return Ok(return_ty);
                }
                return Err(err);
            }
        };
        let method_info = scheme
            .methods
            .iter()
            .find(|m| m.name == method)
            .ok_or_else(|| TypeEnvError::MissingInterfaceMethod {
                interface: interface.to_string(),
                method: method.to_string(),
                span: Span::default(),
            })?;
        let mut call_substitution = selected.substitution.clone();
        for (expected, actual) in method_info.params.iter().zip(arg_types.iter()) {
            let sub = self
                .unify_types(&call_substitution.apply(expected), actual)
                .map_err(|e| TypeEnvError::InvalidDefinition(format!("{e}"), Span::default()))?;
            call_substitution = call_substitution.compose(&sub);
        }
        let raw_return = call_substitution.apply(&method_info.return_type);
        self.normalize_associated_types(&raw_return, scheme, &call_substitution)
    }

    pub(super) fn resolve_interface_method_call_from_generic_bound(
        &self,
        interface: &str,
        method: &str,
        arg_types: &[Type],
    ) -> Option<Type> {
        let interface_info = self.interfaces.get(interface)?;
        let method_info = interface_info.methods.get(method)?;
        if method_info.params.len() != arg_types.len() {
            return None;
        }
        if method_info
            .params
            .iter()
            .any(type_contains_constructor_variable_app)
        {
            return None;
        }

        let mut subst = Substitution::new();
        for (expected, actual) in method_info.params.iter().zip(arg_types.iter()) {
            let sub = self.unify_types(&subst.apply(expected), actual).ok()?;
            subst = subst.compose(&sub);
        }

        if method_info.type_params.is_empty() {
            return None;
        }

        let interface_id = self.interface_identity_for_name(interface).cloned()?;
        let all_generic_args_have_evidence = method_info.type_params.iter().all(|tp| {
            let Type::Var(var) = subst.apply(&Type::Var(*tp)) else {
                return false;
            };
            let proposition = TypeProposition::InterfaceBound(InterfaceBoundProposition {
                subject: type_var_proposition_term(var),
                interface: interface_id.clone(),
                interface_args: Vec::new(),
            });
            matches!(
                self.solve_proposition(&proposition, None),
                Ok(PropositionOutcome::Satisfied(_))
            )
        });
        if !all_generic_args_have_evidence {
            return None;
        }

        Some(subst.apply(&method_info.return_type))
    }

    pub fn select_impl_scheme(
        &self,
        interface: &str,
        method: &str,
        arg_types: &[Type],
    ) -> Result<(SelectedScheme, &ImplScheme), TypeEnvError> {
        let interface_info = self.interfaces.get(interface).ok_or_else(|| {
            TypeEnvError::MissingInterface(interface.to_string(), Span::default())
        })?;

        let method_info = interface_info.methods.get(method).ok_or_else(|| {
            TypeEnvError::MissingInterfaceMethod {
                interface: interface.to_string(),
                method: method.to_string(),
                span: Span::default(),
            }
        })?;

        if method_info.params.len() != arg_types.len() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "interface method '{}::{}' expects {} arguments, found {}",
                    interface,
                    method,
                    method_info.params.len(),
                    arg_types.len()
                ),
                Span::default(),
            ));
        }

        let method_type_params: HashSet<TypeVar> =
            method_info.method_type_params.iter().copied().collect();
        if method_info.params.iter().any(|param| {
            type_contains_constructor_variable_app(param)
                && !constructor_variable_apps_are_payload_anchored(param, &method_type_params)
        }) {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "interface '{}' type parameters could not be fully determined from arguments; evidence lookup does not invert constructor-variable applications",
                    interface
                ),
                Span::default(),
            ));
        }

        let mut subst = Substitution::new();
        let mut constructor_bindings = HashMap::new();
        for (expected, actual) in method_info.params.iter().zip(arg_types.iter()) {
            match_interface_method_call_pattern(
                self,
                expected,
                actual,
                &mut subst,
                &mut constructor_bindings,
            )?;
        }

        let head_args: Vec<Type> = interface_info
            .type_params
            .iter()
            .zip(method_info.type_params.iter())
            .zip(interface_info.type_param_kinds.iter())
            .map(|((param_name, tp), kind)| {
                if kind.is_type() {
                    subst.apply(&Type::Var(*tp))
                } else {
                    constructor_bindings
                        .get(param_name)
                        .cloned()
                        .unwrap_or_else(|| subst.apply(&Type::Var(*tp)))
                }
            })
            .collect();

        if head_args.iter().any(|t| {
            if let Type::Var(var) = t {
                !self.type_var_has_interface_bound(*var, interface)
            } else {
                false
            }
        }) {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "interface '{}' type parameters could not be fully determined from arguments",
                    interface
                ),
                Span::default(),
            ));
        }

        let target_head = Type::Constructor {
            name: QualifiedName::root(interface.to_string()),
            args: head_args,
            kind: Kind::Type,
        };

        let (selected, scheme) = self.find_matching_impl_scheme(interface, &target_head, 0)?;

        if !scheme.methods.iter().any(|m| m.name == method) {
            return Err(TypeEnvError::MissingInterfaceMethod {
                interface: interface.to_string(),
                method: method.to_string(),
                span: Span::default(),
            });
        }

        Ok((selected, scheme))
    }

    pub(super) fn find_matching_impl_scheme(
        &self,
        interface: &str,
        target_head: &Type,
        depth: usize,
    ) -> Result<(SelectedScheme, &ImplScheme), TypeEnvError> {
        if depth > 32 {
            return Err(TypeEnvError::RecursiveBound {
                message: "depth limit".into(),
                span: Span::default(),
            });
        }
        for scheme in self.impls.iter().filter(|s| s.interface == interface) {
            if let Ok(scheme_subst) = self.unify_types(&scheme.head, target_head) {
                let mut bounds_ok = true;
                for bound in &scheme.where_bounds {
                    let bounded_ty = scheme_subst.apply(&Type::Var(bound.type_var));
                    let bound_head = Type::Constructor {
                        name: QualifiedName::root(bound.interface.clone()),
                        args: vec![bounded_ty],
                        kind: Kind::Type,
                    };
                    match self.find_matching_impl_scheme(&bound.interface, &bound_head, depth + 1) {
                        Ok(_) => {}
                        Err(TypeEnvError::RecursiveBound { .. }) => {
                            return Err(TypeEnvError::RecursiveBound {
                                message: "depth limit".into(),
                                span: Span::default(),
                            });
                        }
                        Err(_) => {
                            bounds_ok = false;
                            break;
                        }
                    }
                }
                if bounds_ok {
                    return Ok((
                        SelectedScheme {
                            substitution: scheme_subst,
                        },
                        scheme,
                    ));
                }
            }
        }
        Err(TypeEnvError::MissingImpl {
            interface: interface.to_string(),
            ty: target_head.to_string(),
            span: Span::default(),
        })
    }

    /// Resolve a type name to its qualified form and info
    pub fn resolve_type(
        &self,
        name: &str,
    ) -> Result<(QualifiedName, Option<&TypeInfo>), TypeError> {
        // Try as primitive first
        match name {
            "Int" | "String" | "Bool" | "Float" | "Null" | "Unit" | "Time" | "Ref" | "()" => {
                return Ok((
                    QualifiedName::root(if name == "Unit" { "Null" } else { name }),
                    None,
                ));
            }
            _ => {}
        }

        // Try local types. Identity-only summaries deliberately resolve as
        // names with known arity but without unfoldable representation.
        if self.type_info.contains_key(name) {
            if self.is_identity_only_name(name) {
                return Ok((QualifiedName::root(name), None));
            }
            return Ok((QualifiedName::root(name), self.type_info.get(name)));
        }

        // Try AST types for types not yet converted
        if self.ast_types.contains_key(name) {
            return Ok((QualifiedName::root(name), None));
        }

        Err(TypeError::UnboundVariable(
            name.to_string(),
            Span::default(),
        ))
    }

    /// Check the number of type arguments supplied to a known builtin process type constructor.
    pub fn check_type_constructor_arity(
        &self,
        name: &QualifiedName,
        found_arity: usize,
    ) -> Result<(), TypeError> {
        if !name.is_root() {
            return Ok(());
        }

        match self.interfaces.get(&name.name) {
            Some(interface) if found_arity > 0 => {
                let expected_arity = interface.type_params.len();
                if expected_arity != found_arity {
                    return Err(TypeError::ConstructorArityMismatch {
                        name: name.display(),
                        expected_arity,
                        found_arity,
                        span: Span::default(),
                    });
                }
                return Ok(());
            }
            _ => {}
        }

        let Some(type_def) = self.ast_types.get(&name.name) else {
            return Ok(());
        };

        if self.is_placeholder_name(&name.name) {
            return Ok(());
        }

        let expected_arity = self
            .type_info
            .get(&name.name)
            .map(TypeInfo::type_arg_count)
            .unwrap_or_else(|| type_def.params.len());

        if expected_arity != found_arity {
            return Err(TypeError::ConstructorArityMismatch {
                name: name.display(),
                expected_arity,
                found_arity,
                span: Span::default(),
            });
        }

        Ok(())
    }

    /// Unfold a constructor to its definition with type arguments substituted
    pub fn unfold_constructor(
        &self,
        name: &QualifiedName,
        args: &[Type],
    ) -> Result<UnfoldedBody, TypeError> {
        let (_, type_info) = self.resolve_type(&name.name)?;

        let type_info =
            type_info.ok_or_else(|| TypeError::NotAConstructor(name.display(), Span::default()))?;

        match type_info {
            TypeInfo::Enum {
                params, variants, ..
            } => {
                if params.len() != args.len() {
                    return Err(TypeError::ConstructorArityMismatch {
                        name: name.display(),
                        expected_arity: params.len(),
                        found_arity: args.len(),
                        span: Span::default(),
                    });
                }

                // Create substitution from param vars to args
                let subst = params.iter().copied().zip(args.iter().cloned()).fold(
                    Substitution::new(),
                    |mut acc, (var, ty)| {
                        acc.insert(var, ty);
                        acc
                    },
                );

                // Apply substitution to variants
                let unfolded_variants: Vec<_> = variants
                    .iter()
                    .map(|v| VariantInfo {
                        name: v.name.clone(),
                        fields: v
                            .fields
                            .iter()
                            .map(|(n, t)| (n.clone(), subst.apply(t)))
                            .collect(),
                        payload_shape: v.payload_shape.clone(),
                    })
                    .collect();

                Ok(UnfoldedBody::Enum(unfolded_variants))
            }
            TypeInfo::Struct { params, fields, .. } => {
                if params.len() != args.len() {
                    return Err(TypeError::ConstructorArityMismatch {
                        name: name.display(),
                        expected_arity: params.len(),
                        found_arity: args.len(),
                        span: Span::default(),
                    });
                }

                // Create substitution from param vars to args
                let subst = params.iter().copied().zip(args.iter().cloned()).fold(
                    Substitution::new(),
                    |mut acc, (var, ty)| {
                        acc.insert(var, ty);
                        acc
                    },
                );

                // Apply substitution to fields
                let unfolded_fields: Vec<_> = fields
                    .iter()
                    .map(|(n, t)| (n.clone(), subst.apply(t)))
                    .collect();

                Ok(UnfoldedBody::Struct(unfolded_fields))
            }
        }
    }

    // ============================================================
    // Capability Provider Methods
    // ============================================================

    /// Register a capability provider.
    ///
    /// # Arguments
    /// * `name` - The provider name (e.g., "io", "http", "db")
    pub fn register_provider(&mut self, name: impl Into<String>) {
        self.providers.insert(name.into());
    }

    /// Check if a provider is registered.
    ///
    /// # Arguments
    /// * `name` - The provider name to check
    ///
    /// # Returns
    /// * `true` - If the provider is registered.
    /// * `false` - If the provider is not registered.
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains(name)
    }

    /// Get all registered providers.
    pub fn providers(&self) -> &HashSet<String> {
        &self.providers
    }
}

fn scheme_concrete_target_name(scheme: &ImplScheme) -> Option<String> {
    match scheme.head_args.first()? {
        InterfaceEvidenceArg::Proper(Type::Constructor { name, args, .. }) if args.is_empty() => {
            name.is_root().then(|| name.name.clone())
        }
        _ => None,
    }
}

fn render_impl_scheme_head(scheme: &ImplScheme) -> String {
    let args = scheme
        .head_args
        .iter()
        .map(render_interface_evidence_arg_for_row)
        .collect::<Vec<_>>()
        .join(", ");
    format!("impl {}<{}>", scheme.interface, args)
}

fn render_interface_evidence_arg_for_row(arg: &InterfaceEvidenceArg) -> String {
    match arg {
        InterfaceEvidenceArg::Proper(ty) => ty.to_string(),
        InterfaceEvidenceArg::Constructor(expr) => format!("{expr:?}"),
    }
}
