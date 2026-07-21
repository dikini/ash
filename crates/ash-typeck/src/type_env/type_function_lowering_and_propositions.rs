use super::*;

impl TypeEnv {
    pub(super) fn lower_type_function_pattern(
        &self,
        pattern: &SurfaceTypePattern,
        constraint: &TypeFunctionPatternConstraint,
        pattern_vars: &mut HashMap<String, TypeFunctionPatternConstraint>,
    ) -> Result<TypeFunctionPattern, TypeEnvError> {
        match pattern {
            SurfaceTypePattern::Wildcard { span } => Ok(TypeFunctionPattern::Wildcard {
                constraint: constraint.clone(),
                source_anchor: span_anchor(*span, "wildcard type pattern"),
            }),
            SurfaceTypePattern::Var { name, span } => {
                if let TypeFunctionPatternConstraint::Domain(domain_id) = constraint
                    && let Some((domain, constructor)) =
                        self.find_domain_constructor(domain_id, name.as_ref())
                {
                    return self.lower_domain_constructor_pattern(
                        constructor,
                        domain,
                        &[],
                        *span,
                        pattern_vars,
                    );
                }
                let name = name.to_string();
                if pattern_vars
                    .insert(name.clone(), constraint.clone())
                    .is_some()
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!("repeated type pattern variable '{name}'"),
                        *span,
                    ));
                }
                Ok(TypeFunctionPattern::Var {
                    name,
                    constraint: constraint.clone(),
                    source_anchor: span_anchor(*span, "type pattern variable"),
                })
            }
            SurfaceTypePattern::Constructor { name, args, span } => {
                let TypeFunctionPatternConstraint::Domain(domain_id) = constraint else {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "constructor pattern '{}' requires a sealed-domain position",
                            name
                        ),
                        *span,
                    ));
                };
                let Some((domain, constructor)) =
                    self.find_domain_constructor(domain_id, name.as_ref())
                else {
                    if let Some((other_domain, _)) = self.find_any_domain_constructor(name.as_ref())
                    {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "marker constructor '{}' belongs to sealed domain '{}', not expected sealed domain '{}'",
                                name, other_domain.exported_name, domain_id.name
                            ),
                            *span,
                        ));
                    }
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "unknown marker constructor '{}' for sealed-domain pattern",
                            name
                        ),
                        *span,
                    ));
                };
                if self.visible_type_head_exists(name.as_ref())
                    || self.local_type_function_heads.contains_key(name.as_ref())
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "ambiguous marker constructor '{}' also resolves as a type-level head",
                            name
                        ),
                        *span,
                    ));
                }
                self.lower_domain_constructor_pattern(
                    constructor,
                    domain,
                    args,
                    *span,
                    pattern_vars,
                )
            }
        }
    }

    pub(super) fn lower_domain_constructor_pattern(
        &self,
        constructor: &DomainConstructorSummary,
        domain: &SealedDomainSummary,
        args: &[SurfaceTypePattern],
        span: Span,
        pattern_vars: &mut HashMap<String, TypeFunctionPatternConstraint>,
    ) -> Result<TypeFunctionPattern, TypeEnvError> {
        if constructor.fields.len() != args.len() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "marker constructor '{}' pattern arity mismatch: expected {}, found {}",
                    constructor.exported_name,
                    constructor.fields.len(),
                    args.len()
                ),
                span,
            ));
        }
        let fields = args
            .iter()
            .zip(&constructor.fields)
            .map(|(arg, field)| {
                let constraint = field
                    .domain_constraint
                    .clone()
                    .map(TypeFunctionPatternConstraint::Domain)
                    .unwrap_or_else(|| TypeFunctionPatternConstraint::Kind(field.kind.clone()));
                self.lower_type_function_pattern(arg, &constraint, pattern_vars)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(TypeFunctionPattern::DomainConstructor {
            constructor: Box::new(constructor.id.clone()),
            domain: Box::new(domain.id.clone()),
            fields,
            constraint: TypeFunctionPatternConstraint::Domain(domain.id.clone()),
            source_anchor: span_anchor(
                span,
                format!("marker constructor pattern {}", constructor.exported_name),
            ),
        })
    }

    pub(super) fn lower_type_function_result_expr(
        &self,
        ty: &SurfaceType,
        expected_domain: Option<&SealedDomainId>,
        context: &TypeFunctionResultLoweringContext<'_>,
        span: Span,
    ) -> Result<TypeFunctionResultExpr, TypeEnvError> {
        match ty {
            SurfaceType::Name(name) => self.lower_type_function_result_head(
                name.as_ref(),
                &[],
                expected_domain,
                context,
                span,
            ),
            SurfaceType::Constructor { name, args } => self.lower_type_function_result_head(
                name.as_ref(),
                args,
                expected_domain,
                context,
                span,
            ),
            other => self
                .lower_surface_type_to_canonical(other)
                .and_then(|canonical| {
                    type_function_result_from_canonical(canonical, span)
                        .map_err(|err| TypeError::TypeEnv(Box::new(err)))
                })
                .map_err(|err| {
                    TypeEnvError::InvalidDefinition(format!("result kind mismatch: {err}"), span)
                }),
        }
    }

    pub(super) fn lower_type_function_result_head(
        &self,
        name: &str,
        args: &[SurfaceType],
        expected_domain: Option<&SealedDomainId>,
        context: &TypeFunctionResultLoweringContext<'_>,
        span: Span,
    ) -> Result<TypeFunctionResultExpr, TypeEnvError> {
        if args.is_empty() && context.pattern_vars.contains_key(name) {
            let constraint = context
                .pattern_vars
                .get(name)
                .expect("checked contains_key");
            return Ok(TypeFunctionResultExpr::Var {
                name: name.to_string(),
                kind: Kind::Type,
                constraint: result_constraint_from_pattern(constraint),
                source_anchor: span_anchor(span, format!("type pattern variable {name}")),
            });
        }
        if let Some(domain_id) = expected_domain {
            if let Some((domain, constructor)) = self.find_domain_constructor(domain_id, name) {
                let current_head_has_same_name = context
                    .current_head
                    .as_ref()
                    .is_some_and(|(self_name, _, _, _)| name == *self_name);
                if self.visible_type_head_exists(name)
                    || self.local_type_function_heads.contains_key(name)
                    || current_head_has_same_name
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "ambiguous marker constructor '{name}' also resolves as a type-level head"
                        ),
                        span,
                    ));
                }
                if constructor.fields.len() != args.len() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "marker constructor '{}' result arity mismatch: expected {}, found {}",
                            constructor.exported_name,
                            constructor.fields.len(),
                            args.len()
                        ),
                        span,
                    ));
                }
                let mut lowered_args = Vec::with_capacity(args.len());
                for (index, (arg, field)) in args.iter().zip(&constructor.fields).enumerate() {
                    let lowered = self.lower_type_function_result_expr(
                        arg,
                        field.domain_constraint.as_ref(),
                        context,
                        span,
                    )?;
                    if let Some(expected_domain) = &field.domain_constraint {
                        match self.result_expr_constraint(&lowered) {
                            TypeFunctionResultConstraint::Domain(actual)
                                if actual == *expected_domain => {}
                            found => {
                                return Err(TypeEnvError::InvalidDefinition(
                                    format!(
                                        "result constructor field {index} domain mismatch: expected sealed domain '{}', found {:?}",
                                        expected_domain.name, found
                                    ),
                                    span,
                                ));
                            }
                        }
                    }
                    lowered_args.push(lowered);
                }
                return Ok(TypeFunctionResultExpr::DomainConstructorApp {
                    constructor: constructor.id.clone(),
                    domain: domain.id.clone(),
                    args: lowered_args,
                    kind: Kind::Type,
                    constraint: TypeFunctionResultConstraint::Domain(domain.id.clone()),
                    source_anchor: span_anchor(span, format!("marker constructor result {name}")),
                });
            }
            if let Some((other_domain, _)) = self.find_any_domain_constructor(name) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "marker constructor '{name}' belongs to sealed domain '{}', not expected sealed domain '{}'",
                        other_domain.exported_name, domain_id.name
                    ),
                    span,
                ));
            }
        }
        if let Some((_, head, params, result_constraint)) = context
            .current_head
            .filter(|(self_name, _, _, _)| name == *self_name)
        {
            if self.visible_type_head_exists(name) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!("ambiguous type-function/type head '{name}'"),
                    span,
                ));
            }
            if params.len() != args.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "type function '{name}' application arity mismatch: expected {}, found {}",
                        params.len(),
                        args.len()
                    ),
                    span,
                ));
            }
            let lowered_args = args
                .iter()
                .zip(params)
                .map(|(arg, param)| {
                    self.lower_type_function_result_expr(
                        arg,
                        param.domain_constraint.as_ref(),
                        context,
                        span,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            self.validate_type_function_application_args(name, &lowered_args, params, span)?;
            return Ok(TypeFunctionResultExpr::ComputationHeadApp {
                head: head.clone(),
                args: lowered_args,
                kind: Kind::Type,
                constraint: result_constraint.clone(),
                source_anchor: span_anchor(span, format!("type function call {name}")),
            });
        }
        if let Some(head) = self.local_type_function_heads.get(name) {
            if self.visible_type_head_exists(name) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!("ambiguous type-function/type head '{name}'"),
                    span,
                ));
            }
            let callee = self.local_type_functions.get(head).ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!("unresolved type function or type head '{name}'"),
                    span,
                )
            })?;
            if callee.params.len() != args.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "type function '{name}' application arity mismatch: expected {}, found {}",
                        callee.params.len(),
                        args.len()
                    ),
                    span,
                ));
            }
            let lowered_args = args
                .iter()
                .zip(&callee.params)
                .map(|(arg, param)| {
                    self.lower_type_function_result_expr(
                        arg,
                        param.domain_constraint.as_ref(),
                        context,
                        span,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            self.validate_type_function_application_args(
                name,
                &lowered_args,
                &callee.params,
                span,
            )?;
            return Ok(TypeFunctionResultExpr::ComputationHeadApp {
                head: head.clone(),
                args: lowered_args,
                kind: Kind::Type,
                constraint: callee.result_constraint.clone(),
                source_anchor: span_anchor(span, format!("type function call {name}")),
            });
        }
        if context.later_names.contains(name) {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "forward reference to later type function '{name}' is unsupported in SPEC-E"
                ),
                span,
            ));
        }
        if args.is_empty()
            && matches!(
                name,
                "Int" | "String" | "Bool" | "Float" | "Null" | "Time" | "Ref"
            )
        {
            return Ok(TypeFunctionResultExpr::Primitive {
                name: name.to_string(),
                kind: Kind::Type,
                constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
                source_anchor: span_anchor(span, format!("primitive type {name}")),
            });
        }
        if args.is_empty() && name.chars().next().is_some_and(char::is_lowercase) {
            return Err(TypeEnvError::InvalidDefinition(
                format!("unknown RHS type variable '{name}'"),
                span,
            ));
        }
        let surface = if args.is_empty() {
            SurfaceType::Name(Box::from(name))
        } else {
            SurfaceType::Constructor {
                name: Box::from(name),
                args: args.to_vec(),
            }
        };
        self.lower_surface_type_to_canonical(&surface)
            .and_then(|canonical| {
                type_function_result_from_canonical(canonical, span)
                    .map_err(|err| TypeError::TypeEnv(Box::new(err)))
            })
            .map_err(|_| {
                let prefix =
                    if name.chars().next().is_some_and(char::is_uppercase) && args.is_empty() {
                        "result kind mismatch: "
                    } else {
                        ""
                    };
                TypeEnvError::InvalidDefinition(
                    format!("{prefix}unresolved type function or type head '{name}'"),
                    span,
                )
            })
    }

    pub(super) fn visible_type_head_exists(&self, name: &str) -> bool {
        self.ast_types.contains_key(name) || self.type_alias_identities.contains_key(name)
    }

    pub(super) fn result_expr_constraint(
        &self,
        expr: &TypeFunctionResultExpr,
    ) -> TypeFunctionResultConstraint {
        match expr {
            TypeFunctionResultExpr::Primitive { constraint, .. }
            | TypeFunctionResultExpr::Var { constraint, .. }
            | TypeFunctionResultExpr::NominalApp { constraint, .. }
            | TypeFunctionResultExpr::DomainConstructorApp { constraint, .. }
            | TypeFunctionResultExpr::PromotedDataConstructorApp { constraint, .. }
            | TypeFunctionResultExpr::Projection { constraint, .. }
            | TypeFunctionResultExpr::ComputationHeadApp { constraint, .. } => constraint.clone(),
        }
    }

    pub(super) fn validate_type_function_result_constraint(
        &self,
        expr: &TypeFunctionResultExpr,
        expected: &TypeFunctionResultConstraint,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        let actual = self.result_expr_constraint(expr);
        match (expected, actual) {
            (
                TypeFunctionResultConstraint::Domain(expected_domain),
                TypeFunctionResultConstraint::Domain(actual_domain),
            ) if expected_domain == &actual_domain => Ok(()),
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

    pub(super) fn validate_type_function_application_args(
        &self,
        name: &str,
        args: &[TypeFunctionResultExpr],
        params: &[TypeFunctionParam],
        span: Span,
    ) -> Result<(), TypeEnvError> {
        for (index, (arg, param)) in args.iter().zip(params).enumerate() {
            if let Some(expected_domain) = &param.domain_constraint {
                match self.result_expr_constraint(arg) {
                    TypeFunctionResultConstraint::Domain(actual) if actual == *expected_domain => {}
                    found => {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "type function '{name}' argument {index} domain mismatch: expected sealed domain '{}', found {:?}",
                                expected_domain.name, found
                            ),
                            span,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub(super) fn find_domain_constructor(
        &self,
        domain_id: &SealedDomainId,
        constructor_name: &str,
    ) -> Option<(&SealedDomainSummary, &DomainConstructorSummary)> {
        let domain = self.lookup_sealed_domain_by_id(domain_id)?;
        let constructor = domain
            .constructors
            .iter()
            .find(|constructor| constructor.exported_name == constructor_name)?;
        Some((domain, constructor))
    }

    pub(super) fn find_any_domain_constructor(
        &self,
        constructor_name: &str,
    ) -> Option<(&SealedDomainSummary, &DomainConstructorSummary)> {
        self.sealed_domain_summaries.values().find_map(|domain| {
            domain
                .constructors
                .iter()
                .find(|constructor| constructor.exported_name == constructor_name)
                .map(|constructor| (domain, constructor))
        })
    }

    /// Look up a sealed domain by its canonical identity.
    #[must_use]
    pub fn lookup_sealed_domain_by_id(&self, id: &SealedDomainId) -> Option<&SealedDomainSummary> {
        self.sealed_domain_summaries.get(id)
    }

    /// Iterate over all visible sealed-domain exported names.
    pub fn sealed_domain_names(&self) -> impl Iterator<Item = &str> {
        self.sealed_domain_aliases.keys().map(String::as_str)
    }

    /// Register an interface identity summary in the canonical Phase 110 registry.
    pub fn register_interface_identity_summary(
        &mut self,
        summary: &InterfaceIdentitySummary,
    ) -> Result<(), TypeEnvError> {
        self.register_interface_identity_summary_with_provenance(summary, false)
    }

    pub(super) fn register_interface_identity_summary_imported(
        &mut self,
        summary: &InterfaceIdentitySummary,
    ) -> Result<(), TypeEnvError> {
        self.register_interface_identity_summary_with_provenance(summary, true)
    }

    pub(super) fn register_interface_identity_summary_with_provenance(
        &mut self,
        summary: &InterfaceIdentitySummary,
        imported: bool,
    ) -> Result<(), TypeEnvError> {
        self.known_interface_identities.insert(summary.id.clone());
        self.canonical_interface_names
            .insert(summary.id.clone(), summary.name.to_string());

        let visible_name = summary.name.as_str();
        if let Some(existing) = self.interface_identity_aliases.get(visible_name)
            && existing != &summary.id
        {
            let existing_is_imported = self
                .interface_identity_alias_is_imported
                .get(visible_name)
                .copied()
                .unwrap_or(false);
            if imported || !existing_is_imported {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "conflicting visible interface alias '{}': {:?} vs {:?}",
                        summary.name, existing, summary.id
                    ),
                    Span::default(),
                ));
            }
        }

        self.interface_identity_aliases
            .insert(summary.name.to_string(), summary.id.clone());
        self.interface_identity_alias_is_imported
            .insert(summary.name.to_string(), imported);
        if !imported {
            let Some(interface) = self.interfaces.get(summary.name.as_str()) else {
                return Ok(());
            };
            self.local_interface_arities
                .insert(summary.id.clone(), interface.type_params.len());
        }
        Ok(())
    }

    /// Register an associated-member identity summary in the canonical Phase 110 registry.
    pub fn register_associated_member_identity_summary(
        &mut self,
        summary: &AssociatedMemberIdentitySummary,
    ) -> Result<(), TypeEnvError> {
        self.register_associated_member_identity_summary_with_provenance(summary, false)
    }

    pub(super) fn register_associated_member_identity_summary_imported(
        &mut self,
        summary: &AssociatedMemberIdentitySummary,
    ) -> Result<(), TypeEnvError> {
        self.register_associated_member_identity_summary_with_provenance(summary, true)
    }

    pub(super) fn register_associated_member_identity_summary_with_provenance(
        &mut self,
        summary: &AssociatedMemberIdentitySummary,
        imported: bool,
    ) -> Result<(), TypeEnvError> {
        self.known_associated_member_identities
            .insert(summary.id.clone());
        let alias_key = (
            summary.id.interface.name.to_string(),
            summary.name.to_string(),
        );
        if let Some(existing) = self.associated_member_identity_aliases.get(&alias_key)
            && existing != &summary.id
        {
            let existing_is_imported = self
                .associated_member_identity_alias_is_imported
                .get(&alias_key)
                .copied()
                .unwrap_or(false);
            if imported || !existing_is_imported {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "conflicting visible associated-member alias '{}::{}': {:?} vs {:?}",
                        alias_key.0, alias_key.1, existing, summary.id
                    ),
                    Span::default(),
                ));
            }
        }
        self.associated_member_identity_aliases
            .insert(alias_key.clone(), summary.id.clone());
        self.associated_member_identity_alias_is_imported
            .insert(alias_key, imported);
        Ok(())
    }

    pub(super) fn lower_associated_projection_to_canonical(
        &self,
        base: &CanonicalTypeExpr,
        member_name: &str,
    ) -> Result<CanonicalTypeExpr, TypeError> {
        let projection_spelling = format!(
            "{}::{}",
            canonical_projection_base_spelling(base),
            member_name
        );
        let (base_name, projection_args, rigidity) = match base {
            CanonicalTypeExpr::Var(name) => (
                name.clone(),
                vec![CanonicalTypeExpr::Var(name.clone())],
                ProjectionRigidity::Neutral,
            ),
            CanonicalTypeExpr::NominalApp {
                visible_name, args, ..
            } => (
                visible_name.clone(),
                args.clone(),
                projection_rigidity_for_canonical_args(args),
            ),
            CanonicalTypeExpr::Projection { .. } => {
                return Err(TypeError::ConstructorNameMismatch {
                    expected: "supported associated projection base (nested projection bases are unsupported)"
                        .to_string(),
                    found: format!("nested projection base {projection_spelling}"),
                    span: Span::default(),
                });
            }
            _ => {
                return Err(TypeError::ConstructorNameMismatch {
                    expected:
                        "supported associated projection base (type variable or nominal application)"
                            .to_string(),
                    found: format!("unsupported projection base {projection_spelling}"),
                    span: Span::default(),
                });
            }
        };

        let interface = self
            .interface_identity_for_name(&base_name)
            .cloned()
            .or_else(|| {
                self.interfaces.iter().find_map(|(iface_name, iface_info)| {
                    iface_info
                        .associated_types
                        .contains(&member_name.to_string())
                        .then(|| self.interface_identity_for_name(iface_name).cloned())
                        .flatten()
                })
            })
            .or_else(|| {
                let mut matches = self
                    .known_associated_member_identities
                    .iter()
                    .filter(|id| id.name == member_name)
                    .map(|id| id.interface.clone());
                let first = matches.next()?;
                matches.all(|candidate| candidate == first).then_some(first)
            })
            .ok_or_else(|| TypeError::ConstructorNameMismatch {
                expected: "registered associated projection".to_string(),
                found: format!("{base_name}::{member_name}"),
                span: Span::default(),
            })?;

        let member = self
            .associated_member_identity_for_interface_member(&interface.name, member_name)
            .cloned()
            .ok_or_else(|| TypeError::ConstructorNameMismatch {
                expected: format!("registered member on interface {}", interface.name),
                found: projection_spelling.clone(),
                span: Span::default(),
            })?;

        let expected_arity = self
            .local_interface_arities
            .get(&interface)
            .copied()
            .unwrap_or(projection_args.len());
        if expected_arity != projection_args.len() {
            return Err(TypeError::ConstructorArityMismatch {
                name: format!("{} for projection {}", interface.name, projection_spelling),
                expected_arity,
                found_arity: projection_args.len(),
                span: Span::default(),
            });
        }

        let rigidity = if self
            .lookup_associated_family_declaration(&interface.name, member_name)
            .is_some()
        {
            rigidity
        } else if matches!(base, CanonicalTypeExpr::NominalApp { .. }) {
            ProjectionRigidity::Rigid
        } else {
            rigidity
        };

        Ok(CanonicalTypeExpr::Projection {
            interface,
            member,
            args: projection_args,
            kind: Kind::Type,
            rigidity,
        })
    }

    pub(super) fn lower_explicit_associated_family_projection_to_canonical(
        &self,
        interface_name: &str,
        args: &[SurfaceType],
        member_name: &str,
        span: Span,
    ) -> Result<CanonicalTypeExpr, TypeError> {
        let declaration = self
            .lookup_associated_family_declaration(interface_name, member_name)
            .ok_or_else(|| TypeError::ConstructorNameMismatch {
                expected: "registered sealed associated-family projection".to_string(),
                found: format!("<{interface_name}<...>>::{member_name}"),
                span,
            })?;

        if declaration.interface_params.len() != args.len() {
            return Err(TypeError::ConstructorArityMismatch {
                name: format!("{}::{}", interface_name, member_name),
                expected_arity: declaration.interface_params.len(),
                found_arity: args.len(),
                span,
            });
        }

        let lowered_args = args
            .iter()
            .map(|arg| self.lower_surface_type_to_canonical(arg))
            .collect::<Result<Vec<_>, _>>()?;
        let rigidity = projection_rigidity_for_canonical_args(&lowered_args);

        Ok(CanonicalTypeExpr::Projection {
            interface: declaration.head.interface.clone(),
            member: declaration.head.member.clone(),
            args: lowered_args,
            kind: Kind::Type,
            rigidity,
        })
    }

    pub(super) fn lower_associated_family_projection_result_expr(
        &self,
        interface_name: &str,
        args: &[SurfaceType],
        member_name: &str,
        expected_constraint: &AssociatedFamilyResultConstraint,
        var_constraints: &HashMap<String, AssociatedFamilyResultConstraint>,
        span: Span,
    ) -> Result<AssociatedFamilyResultExpr, TypeEnvError> {
        let declaration = self
            .lookup_associated_family_declaration(interface_name, member_name)
            .cloned()
            .ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!("unknown sealed associated-family projection '<{interface_name}<...>>::{member_name}'"),
                    span,
                )
            })?;

        if declaration.interface_params.len() != args.len() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family projection '{}::{}' expects {} interface arguments, found {}",
                    interface_name,
                    member_name,
                    declaration.interface_params.len(),
                    args.len()
                ),
                span,
            ));
        }

        let interface_args = args
            .iter()
            .zip(declaration.interface_params.iter())
            .map(|(arg, param)| {
                let constraint =
                    Self::associated_family_constraint_for_domain(param.domain_constraint.as_ref());
                self.lower_associated_family_result_expr(arg, &constraint, var_constraints, span)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let result = AssociatedFamilyResultExpr::AssociatedFamilyProjection {
            head: declaration.head.clone(),
            interface_args,
            kind: Kind::Type,
            constraint: declaration.result_domain.clone(),
            rigidity: projection_rigidity_for_associated_family_args(&[]),
            source_anchor: span_anchor(
                span,
                format!("associated family projection {interface_name}::{member_name}"),
            ),
        };
        let result = match result {
            AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                head,
                interface_args,
                kind,
                constraint,
                source_anchor,
                ..
            } => {
                let rigidity = projection_rigidity_for_associated_family_args(&interface_args);
                AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                    head,
                    interface_args,
                    kind,
                    constraint,
                    rigidity,
                    source_anchor,
                }
            }
            _ => unreachable!("constructed as associated family projection"),
        };
        if !Self::associated_family_expr_conforms_to_constraint(&result, expected_constraint) {
            return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: format!("{}::{}", interface_name, member_name),
                reason: format!(
                    "projection result constraint '{}' does not conform to expected '{}'",
                    associated_family_result_constraint_label(&declaration.result_domain),
                    associated_family_result_constraint_label(expected_constraint)
                ),
                span,
            });
        }
        Ok(result)
    }

    pub(super) fn canonical_type_identity_for_visible_name(
        &self,
        visible_name: &str,
    ) -> Result<TypeDeclId, TypeError> {
        self.type_identity_for_name(visible_name)
            .cloned()
            .ok_or_else(|| TypeError::ConstructorNameMismatch {
                expected: "registered canonical type identity".to_string(),
                found: visible_name.to_string(),
                span: Span::default(),
            })
    }

    /// Lower a core `TypeExpr` into the Phase 110 canonical type-expression substrate.
    pub fn lower_core_type_expr_to_canonical(
        &self,
        expr: &TypeExpr,
    ) -> Result<CanonicalTypeExpr, TypeError> {
        match expr {
            TypeExpr::Named(name) => match name.as_str() {
                "Int" | "String" | "Bool" | "Float" | "Null" | "Unit" | "Time" | "Ref" => {
                    Ok(CanonicalTypeExpr::Primitive(name.clone()))
                }
                _ => {
                    if let Some(kind) = self.type_parameter_kind(name) {
                        if kind.is_type() {
                            Ok(CanonicalTypeExpr::Var(name.clone()))
                        } else {
                            Err(TypeError::from(TypeEnvError::InvalidDefinition(
                                format!(
                                    "constructor variable '{name}' has kind {kind}; expected a fully applied proper type"
                                ),
                                Span::default(),
                            )))
                        }
                    } else {
                        match self.resolve_type(name) {
                            Ok((qualified, _)) => {
                                self.check_type_constructor_arity(&qualified, 0)?;
                                Ok(CanonicalTypeExpr::NominalApp {
                                    origin: self.canonical_type_identity_for_visible_name(name)?,
                                    visible_name: name.clone(),
                                    args: vec![],
                                    kind: Kind::Type,
                                })
                            }
                            Err(TypeError::UnboundVariable(_, _)) => {
                                Ok(CanonicalTypeExpr::Var(name.clone()))
                            }
                            Err(err) => Err(err),
                        }
                    }
                }
            },
            TypeExpr::Constructor { name, args } => {
                if let Some(kind) = self.type_parameter_kind(name) {
                    if kind.is_type() {
                        return Err(TypeError::from(TypeEnvError::InvalidDefinition(
                            format!(
                                "proper type variable '{name}' of kind * cannot be applied as a constructor"
                            ),
                            Span::default(),
                        )));
                    }
                    let expected_arity = kind.arity();
                    if args.len() != expected_arity {
                        return Err(TypeError::from(TypeEnvError::InvalidDefinition(
                            format!(
                                "wrong arity for constructor variable '{name}': expected {expected_arity}, found {}",
                                args.len()
                            ),
                            Span::default(),
                        )));
                    }
                    let lowered_args = args
                        .iter()
                        .map(|arg| self.lower_core_type_expr_to_canonical(arg))
                        .collect::<Result<Vec<_>, _>>()?;
                    return Ok(CanonicalTypeExpr::ConstructorVariableApp(Box::new(
                        ConstructorVariableApp::new(
                            ConstructorVariableRef::new(name.clone(), kind.clone(), None),
                            lowered_args,
                            Kind::Type,
                            None,
                        ),
                    )));
                }
                let (qualified, _) = self.resolve_type(name)?;
                self.check_type_constructor_arity(&qualified, args.len())?;
                Ok(CanonicalTypeExpr::NominalApp {
                    origin: self.canonical_type_identity_for_visible_name(name)?,
                    visible_name: name.clone(),
                    args: args
                        .iter()
                        .map(|arg| self.lower_core_type_expr_to_canonical(arg))
                        .collect::<Result<Vec<_>, _>>()?,
                    kind: Kind::Type,
                })
            }
            TypeExpr::Tuple(items) => Err(TypeError::ConstructorNameMismatch {
                expected: "nominal or associated type expression supported by TASK-798 lowering"
                    .to_string(),
                found: format!("Tuple({})", items.len()),
                span: Span::default(),
            }),
            TypeExpr::Record(fields) => Err(TypeError::ConstructorNameMismatch {
                expected: "nominal or associated type expression supported by TASK-798 lowering"
                    .to_string(),
                found: format!("Record({})", fields.len()),
                span: Span::default(),
            }),
            TypeExpr::Associated { base, name } => {
                if matches!(base.as_ref(), TypeExpr::Associated { .. }) {
                    return Err(TypeError::ConstructorNameMismatch {
                        expected: "supported associated projection base (nested projection bases are unsupported)"
                            .to_string(),
                        found: format!("nested projection base {base:?}"),
                        span: Span::default(),
                    });
                }
                if matches!(base.as_ref(), TypeExpr::Tuple(_) | TypeExpr::Record(_)) {
                    let found = match base.as_ref() {
                        TypeExpr::Tuple(items) => {
                            format!("unsupported projection base Tuple({})", items.len())
                        }
                        TypeExpr::Record(fields) => {
                            format!("unsupported projection base Record({})", fields.len())
                        }
                        _ => unreachable!("guarded by matches!"),
                    };
                    return Err(TypeError::ConstructorNameMismatch {
                        expected: "supported associated projection base (type variable or nominal application)"
                            .to_string(),
                        found,
                        span: Span::default(),
                    });
                }
                let lowered_base = self.lower_core_type_expr_to_canonical(base)?;
                self.lower_associated_projection_to_canonical(&lowered_base, name)
            }
        }
    }

    /// Lower a surface proposition tail into canonical proposition carriers without solving.
    pub fn register_proposition_predicate_decl(
        &mut self,
        decl: &PropositionPredicateDecl,
    ) -> Result<PropositionPredicateId, TypeError> {
        reject_constructor_kinded_proposition_params(
            &decl.params,
            "proposition predicate parameter",
            "TASK-908",
        )
        .map_err(TypeError::from)?;

        let module = self
            .current_module_identity
            .clone()
            .unwrap_or_else(synthetic_proposition_module_identity);
        let origin = proposition_module_source_origin(&module);
        let id = PropositionPredicateId::new(module, decl.name.to_string());
        let params = decl
            .params
            .iter()
            .map(|param| {
                let ty = self.lower_surface_type_to_canonical(&param.domain)?;
                Ok(PropositionPredicateParamSummary {
                    name: param.name.to_string(),
                    ty,
                    kind: Kind::Type,
                    source_anchor: proposition_source_anchor(
                        origin.clone(),
                        param.span,
                        format!("proposition predicate parameter {}", param.name),
                    ),
                })
            })
            .collect::<Result<Vec<_>, TypeError>>()?;
        let summary = PropositionPredicateSummary {
            id: id.clone(),
            exported_name: decl.name.to_string(),
            visibility: core_visibility_from_surface(&decl.visibility),
            params,
            source_anchor: proposition_source_anchor(
                origin,
                decl.span,
                format!("proposition predicate {}", decl.name),
            ),
        };
        self.register_proposition_predicate_summary_with_solver_kind(
            &summary,
            PropositionPredicateSolverKind::DeferredUnsupported,
        )?;
        Ok(id)
    }

    pub fn register_proposition_predicate_summary(
        &mut self,
        summary: &PropositionPredicateSummary,
    ) -> Result<(), TypeEnvError> {
        self.register_proposition_predicate_summary_with_solver_kind(
            summary,
            PropositionPredicateSolverKind::DeferredUnsupported,
        )
    }

    pub fn register_builtin_proposition_predicate_summary(
        &mut self,
        summary: &PropositionPredicateSummary,
    ) -> Result<(), TypeEnvError> {
        self.register_proposition_predicate_summary_with_solver_kind(
            summary,
            PropositionPredicateSolverKind::CompilerBuiltinSatisfied,
        )
    }

    pub(super) fn register_proposition_predicate_summary_with_solver_kind(
        &mut self,
        summary: &PropositionPredicateSummary,
        solver_kind: PropositionPredicateSolverKind,
    ) -> Result<(), TypeEnvError> {
        self.validate_public_proposition_predicate_summary_dependencies(summary)?;
        let visible_name = summary.exported_name.to_string();
        if let Some(existing) = self.proposition_predicate_aliases.get(&visible_name)
            && existing != &summary.id
        {
            return Err(TypeEnvError::ImportOrderConflict {
                family: "proposition predicate visible name".to_string(),
                name: visible_name,
                span: anchor_span(&summary.source_anchor),
            });
        }
        if let Some(existing) = self.proposition_predicates.get(&summary.id) {
            if existing.summary != *summary || existing.solver_kind != solver_kind {
                return Err(TypeEnvError::ImportOrderConflict {
                    family: "proposition predicate summary".to_string(),
                    name: summary.exported_name.to_string(),
                    span: anchor_span(&summary.source_anchor),
                });
            }
            return Ok(());
        }
        self.proposition_predicate_aliases
            .insert(visible_name, summary.id.clone());
        self.proposition_predicates.insert(
            summary.id.clone(),
            PropositionPredicateInfo {
                summary: summary.clone(),
                solver_kind,
            },
        );
        Ok(())
    }

    #[must_use]
    pub fn lookup_proposition_predicate(&self, name: &str) -> Option<&PropositionPredicateInfo> {
        let id = self.proposition_predicate_aliases.get(name)?;
        self.proposition_predicates.get(id)
    }

    #[must_use]
    pub fn proposition_predicate_by_id(
        &self,
        id: &PropositionPredicateId,
    ) -> Option<&PropositionPredicateInfo> {
        self.proposition_predicates.get(id)
    }

    /// Lower a surface proposition tail into canonical proposition carriers without solving.
    pub fn lower_proposition_tail(
        &self,
        tail: &PropositionTail,
        source_origin: SourceOrigin,
    ) -> Result<Vec<LoweredPropositionClause>, TypeError> {
        tail.clauses
            .iter()
            .map(|clause| self.lower_proposition_clause(clause, source_origin.clone()))
            .collect()
    }

    /// Add required proposition obligations generated by a specific checking site.
    pub fn add_proposition_obligations_from_tail(
        &mut self,
        tail: &PropositionTail,
        source_origin: SourceOrigin,
        owner_site: PropositionCheckingSite,
    ) -> Result<(), TypeError> {
        let lowered = self.lower_proposition_tail(tail, source_origin)?;
        for clause in lowered {
            self.push_proposition_fact(
                PropositionFactRole::Requirement,
                clause.proposition,
                clause.source_anchor,
                owner_site.clone(),
                clause.outcome,
            );
        }
        Ok(())
    }

    /// Add assumed proposition facts generated by a specific checking site.
    pub fn add_proposition_assumptions_from_tail(
        &mut self,
        tail: &PropositionTail,
        source_origin: SourceOrigin,
        owner_site: PropositionCheckingSite,
    ) -> Result<(), TypeError> {
        let lowered = self.lower_proposition_tail(tail, source_origin)?;
        for clause in lowered {
            self.push_proposition_fact(
                PropositionFactRole::Assumption,
                clause.proposition,
                clause.source_anchor,
                owner_site.clone(),
                clause.outcome,
            );
        }
        Ok(())
    }

    /// Proposition assumptions available as inputs to later solvers.
    #[must_use]
    pub fn proposition_assumptions(&self) -> &[PropositionFactRecord] {
        &self.proposition_assumptions
    }

    /// Required proposition obligations that later task-owned solvers must discharge.
    #[must_use]
    pub fn proposition_obligations(&self) -> &[PropositionFactRecord] {
        &self.proposition_obligations
    }

    /// Export public proposition requirements through the SPEC-064/V5 summary carrier.
    pub fn export_public_proposition_fact_summaries(
        &self,
        module: &ModuleIdentity,
    ) -> Result<Vec<PropositionFactSummary>, TypeEnvError> {
        let public_item = format!(
            "module '{}' public proposition requirement",
            module.path.join("::")
        );
        let mut facts = Vec::new();
        for record in &self.proposition_obligations {
            let predicate_dependencies = self.validate_public_proposition_dependencies(
                &public_item,
                &record.proposition,
                anchor_span(&record.source_anchor),
            )?;
            let outcome = match &record.outcome {
                Some(outcome) => Some(outcome.clone()),
                None => Some(
                    self.solve_proposition(&record.proposition, Some(record.source_anchor.clone()))
                        .map_err(proposition_revalidation_error)?,
                ),
            };
            facts.push(PropositionFactSummary {
                proposition: record.proposition.clone(),
                role: record.role,
                source_anchor: record.source_anchor.clone(),
                predicate_dependencies,
                dependency_summary_refs: Vec::new(),
                outcome,
            });
        }
        Ok(facts)
    }

    pub(super) fn validate_public_proposition_dependencies(
        &self,
        public_item: &str,
        proposition: &TypeProposition,
        span: Span,
    ) -> Result<Vec<PropositionPredicateId>, TypeEnvError> {
        let mut predicate_dependencies = Vec::new();
        match proposition {
            TypeProposition::Equality(equality) => {
                self.validate_public_proposition_term_dependencies(
                    public_item,
                    &equality.lhs,
                    span,
                )?;
                self.validate_public_proposition_term_dependencies(
                    public_item,
                    &equality.rhs,
                    span,
                )?;
            }
            TypeProposition::Disequality(disequality) => {
                self.validate_public_proposition_term_dependencies(
                    public_item,
                    &disequality.lhs,
                    span,
                )?;
                self.validate_public_proposition_term_dependencies(
                    public_item,
                    &disequality.rhs,
                    span,
                )?;
            }
            TypeProposition::InterfaceBound(bound) => {
                if !self.public_interface_dependency_known(&bound.interface) {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "interface",
                        &bound.interface.name,
                        span,
                    ));
                }
                self.validate_public_proposition_term_dependencies(
                    public_item,
                    &bound.subject,
                    span,
                )?;
                for arg in &bound.interface_args {
                    self.validate_public_proposition_term_dependencies(public_item, arg, span)?;
                }
            }
            TypeProposition::NamedPredicate(named) => {
                let Some(info) = self.proposition_predicate_by_id(&named.predicate) else {
                    return Err(TypeEnvError::UnknownPropositionPredicate {
                        name: named.predicate.name.to_string(),
                        span,
                    });
                };
                if info.summary.visibility != ash_core::ast::Visibility::Public {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "proposition predicate",
                        &info.summary.exported_name,
                        span,
                    ));
                }
                if info.summary.params.len() != named.args.len() {
                    return Err(TypeEnvError::PropositionPredicateArityMismatch {
                        name: info.summary.exported_name.to_string(),
                        expected: info.summary.params.len(),
                        actual: named.args.len(),
                        span,
                    });
                }
                predicate_dependencies.push(named.predicate.clone());
                for arg in &named.args {
                    self.validate_public_proposition_term_dependencies(public_item, arg, span)?;
                }
            }
        }
        predicate_dependencies.sort_by(|left, right| {
            left.module
                .path
                .cmp(&right.module.path)
                .then_with(|| left.name.cmp(&right.name))
        });
        predicate_dependencies.dedup();
        Ok(predicate_dependencies)
    }

    pub(super) fn validate_public_proposition_predicate_summary_dependencies(
        &self,
        summary: &PropositionPredicateSummary,
    ) -> Result<(), TypeEnvError> {
        if summary.visibility != ash_core::ast::Visibility::Public {
            return Ok(());
        }
        let public_item = format!("public proposition predicate '{}'", summary.exported_name);
        for param in &summary.params {
            self.validate_public_canonical_proposition_dependencies(
                &public_item,
                &param.ty,
                anchor_span(&param.source_anchor),
            )?;
        }
        Ok(())
    }

    pub(super) fn validate_public_proposition_term_dependencies(
        &self,
        public_item: &str,
        term: &TypePropositionTerm,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match term {
            TypePropositionTerm::Canonical(expr) => {
                self.validate_public_canonical_proposition_dependencies(public_item, expr, span)
            }
            TypePropositionTerm::DomainConstructorApp {
                constructor,
                domain,
                args,
                ..
            } => {
                let Some(summary) = self.lookup_sealed_domain_by_id(domain) else {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "sealed domain",
                        &domain.name,
                        span,
                    ));
                };
                if summary.visibility != ash_core::ast::Visibility::Public {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "sealed domain",
                        &summary.exported_name,
                        span,
                    ));
                }
                if !summary
                    .constructors
                    .iter()
                    .any(|candidate| candidate.id == *constructor)
                {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "domain constructor",
                        &constructor.name,
                        span,
                    ));
                }
                for arg in args {
                    self.validate_public_proposition_term_dependencies(public_item, arg, span)?;
                }
                Ok(())
            }
        }
    }

    pub(super) fn public_interface_dependency_known(
        &self,
        interface: &InterfaceIdentityId,
    ) -> bool {
        if !self.known_interface_identities.contains(interface) {
            return false;
        }
        self.interfaces
            .get(interface.name.as_str())
            .is_none_or(|info| info.visibility == ash_core::ast::Visibility::Public)
    }

    pub(super) fn public_associated_member_dependency_known(
        &self,
        member: &AssociatedMemberIdentityId,
    ) -> bool {
        self.known_associated_member_identities.contains(member)
            && self.public_interface_dependency_known(&member.interface)
    }

    pub(super) fn validate_public_canonical_proposition_dependencies(
        &self,
        public_item: &str,
        expr: &CanonicalTypeExpr,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match expr {
            CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::Var(_) => Ok(()),
            CanonicalTypeExpr::NominalApp { origin, args, .. } => {
                let Some(visible_name) = self.canonical_type_names.get(origin) else {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "ordinary type",
                        &origin.name,
                        span,
                    ));
                };
                if !self.ast_types.get(visible_name).is_some_and(|ty| {
                    ty.visibility == ash_core::ast::Visibility::Public || ty.builtin
                }) {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "ordinary type",
                        visible_name,
                        span,
                    ));
                }
                for arg in args {
                    self.validate_public_canonical_proposition_dependencies(
                        public_item,
                        arg,
                        span,
                    )?;
                }
                Ok(())
            }
            CanonicalTypeExpr::Projection {
                interface,
                member,
                args,
                ..
            } => {
                if !self.public_interface_dependency_known(interface) {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "interface",
                        &interface.name,
                        span,
                    ));
                }
                if !self.public_associated_member_dependency_known(member) {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "associated member",
                        &member.name,
                        span,
                    ));
                }
                for arg in args {
                    self.validate_public_canonical_proposition_dependencies(
                        public_item,
                        arg,
                        span,
                    )?;
                }
                Ok(())
            }
            CanonicalTypeExpr::ComputationHeadApp { head, args, .. } => {
                let Some(def) = self.local_type_functions.get(head) else {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "type function",
                        &head.name,
                        span,
                    ));
                };
                if def.visibility != ash_core::ast::Visibility::Public {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "type function",
                        &head.name,
                        span,
                    ));
                }
                for arg in args {
                    self.validate_public_canonical_proposition_dependencies(
                        public_item,
                        arg,
                        span,
                    )?;
                }
                Ok(())
            }
            CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
                let Some(kind_summary) = self.lookup_promoted_data_kind_by_id(&app.data_kind)
                else {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "promoted data kind",
                        &app.data_kind.name,
                        span,
                    ));
                };
                if kind_summary.visibility != ash_core::ast::Visibility::Public {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "promoted data kind",
                        &kind_summary.exported_name,
                        span,
                    ));
                }
                let Some(source_visible_name) =
                    self.canonical_type_names.get(&kind_summary.source_type)
                else {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "promoted source ADT",
                        &kind_summary.source_type.name,
                        span,
                    ));
                };
                if !self.ast_types.get(source_visible_name).is_some_and(|ty| {
                    ty.visibility == ash_core::ast::Visibility::Public || ty.builtin
                }) {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "promoted source ADT",
                        source_visible_name,
                        span,
                    ));
                }
                let Some(constructor_summary) =
                    self.lookup_promoted_constructor_by_id(&app.constructor)
                else {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "promoted data constructor",
                        &app.constructor.name,
                        span,
                    ));
                };
                if constructor_summary.visibility != ash_core::ast::Visibility::Public
                    || constructor_summary.id.kind != kind_summary.id
                {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "promoted data constructor",
                        &constructor_summary.exported_name,
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
                    self.validate_public_canonical_proposition_dependencies(
                        public_item,
                        arg,
                        span,
                    )?;
                    if let Some(expected_kind) = kinding
                        .field_data_kind_constraints
                        .get(index)
                        .and_then(|constraint| constraint.as_ref())
                    {
                        self.validate_canonical_promoted_data_kind(arg, expected_kind, span)?;
                    }
                }
                Ok(())
            }
            CanonicalTypeExpr::ConstructorVariableApp(app) => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public proposition '{public_item}' cannot export constructor-variable application '{}' until TASK-908 defines higher-kinded evidence summaries",
                    app.constructor.name
                ),
                span,
            )),
        }
    }

    /// Add one required proposition obligation that has already been lowered to core carriers.
    pub fn add_proposition_obligation(
        &mut self,
        proposition: TypeProposition,
        source_anchor: SourceAnchor,
        owner_site: PropositionCheckingSite,
    ) {
        self.push_proposition_fact(
            PropositionFactRole::Requirement,
            proposition,
            source_anchor,
            owner_site,
            None,
        );
    }

    /// Solve one proposition using the conservative SPEC-064 equality/disequality layer.
    pub fn solve_proposition(
        &self,
        proposition: &TypeProposition,
        source_anchor: Option<SourceAnchor>,
    ) -> Result<PropositionOutcome, TypeError> {
        match proposition {
            TypeProposition::Equality(equality) => {
                self.solve_equality_proposition(proposition, equality, source_anchor)
            }
            TypeProposition::Disequality(disequality) => {
                self.solve_disequality_proposition(proposition, disequality, source_anchor)
            }
            TypeProposition::InterfaceBound(bound) => {
                Ok(self.solve_interface_bound_proposition(proposition, bound, source_anchor))
            }
            TypeProposition::NamedPredicate(named) => {
                self.solve_named_predicate_proposition(proposition, named, source_anchor)
            }
        }
    }

    pub(super) fn solve_interface_bound_proposition(
        &self,
        proposition: &TypeProposition,
        bound: &InterfaceBoundProposition,
        source_anchor: Option<SourceAnchor>,
    ) -> PropositionOutcome {
        let exact_evidence = self.proposition_assumptions.iter().find_map(|record| {
            if !matches!(
                record.role,
                PropositionFactRole::Assumption | PropositionFactRole::Evidence
            ) {
                return None;
            }
            if !matches!(
                &record.proposition,
                TypeProposition::InterfaceBound(assumed) if assumed == bound
            ) {
                return None;
            }
            match record.owner_site.kind {
                PropositionCheckingSiteKind::ConcreteImpl => {
                    Some((record, PropositionEvidenceRule::ConcreteImplEvidence))
                }
                PropositionCheckingSiteKind::TypeVariableInterfaceBound
                | PropositionCheckingSiteKind::ImplWhereBound => {
                    Some((record, PropositionEvidenceRule::InScopeInterfaceBound))
                }
                PropositionCheckingSiteKind::ExplicitRequirement
                | PropositionCheckingSiteKind::Synthetic => None,
            }
        });

        let Some((record, rule)) = exact_evidence else {
            if let Some(record) = self.interface_bound_assumption_entails(bound) {
                return proposition_satisfaction(
                    proposition,
                    None,
                    PropositionEvidenceRule::InScopeInterfaceBound,
                    source_anchor.or_else(|| Some(record.source_anchor.clone())),
                );
            }
            return proposition_deferral(
                proposition,
                PropositionDeferredKind::MissingInterfaceEvidence,
                source_anchor,
                true,
            );
        };

        proposition_satisfaction(
            proposition,
            None,
            rule,
            source_anchor.or_else(|| Some(record.source_anchor.clone())),
        )
    }

    pub(super) fn interface_bound_assumption_entails(
        &self,
        required: &InterfaceBoundProposition,
    ) -> Option<&PropositionFactRecord> {
        self.proposition_assumptions.iter().find(|record| {
            if !matches!(
                record.role,
                PropositionFactRole::Assumption | PropositionFactRole::Evidence
            ) {
                return false;
            }
            if !matches!(
                record.owner_site.kind,
                PropositionCheckingSiteKind::TypeVariableInterfaceBound
                    | PropositionCheckingSiteKind::ImplWhereBound
            ) {
                return false;
            }
            let TypeProposition::InterfaceBound(available) = &record.proposition else {
                return false;
            };
            self.interface_bound_entails_required(available, required)
        })
    }

    pub(super) fn interface_bound_entails_required(
        &self,
        available: &InterfaceBoundProposition,
        required: &InterfaceBoundProposition,
    ) -> bool {
        let Some(available_interface) = self.canonical_interface_names.get(&available.interface)
        else {
            return false;
        };
        let Some(required_interface) = self.canonical_interface_names.get(&required.interface)
        else {
            return false;
        };

        let mut visited = HashSet::new();
        let mut stack = vec![(
            available_interface.clone(),
            available.subject.clone(),
            available.interface_args.clone(),
        )];
        while let Some((interface_name, subject, interface_args)) = stack.pop() {
            if !visited.insert((
                interface_name.clone(),
                subject.clone(),
                interface_args.clone(),
            )) {
                continue;
            }
            if interface_name == *required_interface
                && subject == required.subject
                && interface_args == required.interface_args
            {
                return true;
            }

            let Some(info) = self.interfaces.get(&interface_name) else {
                continue;
            };
            for constraint in &info.evidence_constraints {
                let Some(required_subject) = interface_constraint_subject_term(
                    &subject,
                    &interface_args,
                    constraint.subject_param_index,
                ) else {
                    continue;
                };
                stack.push((
                    constraint.required_interface.clone(),
                    required_subject,
                    Vec::new(),
                ));
            }
        }

        false
    }

    pub(super) fn solve_named_predicate_proposition(
        &self,
        proposition: &TypeProposition,
        named: &NamedPredicateProposition,
        source_anchor: Option<SourceAnchor>,
    ) -> Result<PropositionOutcome, TypeError> {
        let Some(info) = self.proposition_predicate_by_id(&named.predicate) else {
            return Err(TypeEnvError::UnknownPropositionPredicate {
                name: named.predicate.name.to_string(),
                span: source_anchor
                    .as_ref()
                    .map_or_else(Span::default, anchor_span),
            }
            .into());
        };

        if info.summary.params.len() != named.args.len() {
            return Err(TypeEnvError::PropositionPredicateArityMismatch {
                name: info.summary.exported_name.to_string(),
                expected: info.summary.params.len(),
                actual: named.args.len(),
                span: source_anchor
                    .as_ref()
                    .map_or_else(Span::default, anchor_span),
            }
            .into());
        }

        match info.solver_kind {
            PropositionPredicateSolverKind::CompilerBuiltinSatisfied => {
                Ok(proposition_satisfaction(
                    proposition,
                    None,
                    PropositionEvidenceRule::NamedPredicateAssumption,
                    source_anchor,
                ))
            }
            PropositionPredicateSolverKind::DeferredUnsupported => Ok(proposition_deferral(
                proposition,
                PropositionDeferredKind::UnsupportedNamedPredicate,
                source_anchor,
                true,
            )),
        }
    }

    /// Solve all stored proposition obligations, updating each fact record with its outcome.
    pub fn solve_proposition_obligations(&mut self) -> Result<Vec<PropositionOutcome>, TypeError> {
        let pending = self
            .proposition_obligations
            .iter()
            .enumerate()
            .map(|(index, record)| {
                (
                    index,
                    record.proposition.clone(),
                    record.source_anchor.clone(),
                )
            })
            .collect::<Vec<_>>();

        let mut outcomes = Vec::with_capacity(pending.len());
        for (index, proposition, source_anchor) in pending {
            let outcome = self.solve_proposition(&proposition, Some(source_anchor))?;
            if let Some(record) = self.proposition_obligations.get_mut(index) {
                record.outcome = Some(outcome.clone());
            }
            outcomes.push(outcome);
        }
        Ok(outcomes)
    }

    /// Solve and require discharge of every stored proposition obligation.
    ///
    /// Plain solving may conservatively return deferred outcomes. Checking points
    /// that require proofs call this stricter path so refuted or deferred
    /// propositions become ordinary type-environment errors without invoking
    /// inversion or meta-solving.
    pub fn discharge_required_proposition_obligations(
        &mut self,
    ) -> Result<Vec<PropositionOutcome>, TypeEnvError> {
        self.discharge_required_proposition_obligations_from(0)
    }

    #[allow(dead_code)]
    pub(crate) fn discharge_required_proposition_obligations_since(
        &mut self,
        start_index: usize,
    ) -> Result<Vec<PropositionOutcome>, TypeEnvError> {
        self.discharge_required_proposition_obligations_from(start_index)
    }

    pub(super) fn discharge_required_proposition_obligations_from(
        &mut self,
        start_index: usize,
    ) -> Result<Vec<PropositionOutcome>, TypeEnvError> {
        let pending = self
            .proposition_obligations
            .iter()
            .enumerate()
            .skip(start_index)
            .map(|(index, record)| {
                (
                    index,
                    record.proposition.clone(),
                    record.source_anchor.clone(),
                    record.owner_site.clone(),
                )
            })
            .collect::<Vec<_>>();

        let mut checked = Vec::with_capacity(pending.len());
        for (index, proposition, source_anchor, owner_site) in pending {
            let outcome = self
                .solve_proposition(&proposition, Some(source_anchor.clone()))
                .map_err(proposition_revalidation_error)?;
            match &outcome {
                PropositionOutcome::Satisfied(_) => checked.push((index, outcome)),
                PropositionOutcome::Refuted(_) | PropositionOutcome::Deferred(_) => {
                    return Err(required_proposition_discharge_error(
                        &owner_site,
                        &source_anchor,
                        &outcome,
                    ));
                }
            }
        }
        let mut outcomes = Vec::with_capacity(checked.len());
        for (index, outcome) in checked {
            if let Some(record) = self.proposition_obligations.get_mut(index) {
                record.outcome = Some(outcome.clone());
            }
            outcomes.push(outcome);
        }
        Ok(outcomes)
    }

    pub(super) fn solve_equality_proposition(
        &self,
        proposition: &TypeProposition,
        equality: &TypeEqualityProposition,
        source_anchor: Option<SourceAnchor>,
    ) -> Result<PropositionOutcome, TypeError> {
        let span = source_anchor.as_ref().map(anchor_span).unwrap_or_default();
        self.validate_proposition_term_promoted_operands(&equality.lhs, span)?;
        self.validate_proposition_term_promoted_operands(&equality.rhs, span)?;
        let (result, lhs_norm, rhs_norm) =
            self.compare_proposition_terms(&equality.lhs, &equality.rhs)?;
        let normalized_terms = Some(proposition_comparison_terms(
            lhs_norm.clone(),
            rhs_norm.clone(),
        ));

        Ok(match result {
            DefinitionalEqualityResult::Equal => proposition_satisfaction(
                proposition,
                normalized_terms,
                PropositionEvidenceRule::DefinitionalEquality,
                source_anchor,
            ),
            DefinitionalEqualityResult::NotEqual { .. } => proposition_refutation(
                proposition,
                normalized_terms,
                PropositionRefutationReason::DefinitionalEquality,
                source_anchor,
            ),
            DefinitionalEqualityResult::BlockedByNeutrality {
                neutral_subterms, ..
            } => {
                let kind = if neutral_subterms.is_empty() {
                    proposition_deferred_kind_from_blocked_normals(&lhs_norm, &rhs_norm)
                } else {
                    proposition_deferred_kind_from_blockers(&neutral_subterms)
                };
                proposition_deferral(proposition, kind, source_anchor, true)
            }
        })
    }

    pub(super) fn solve_disequality_proposition(
        &self,
        proposition: &TypeProposition,
        disequality: &TypeDisequalityProposition,
        source_anchor: Option<SourceAnchor>,
    ) -> Result<PropositionOutcome, TypeError> {
        let span = source_anchor.as_ref().map(anchor_span).unwrap_or_default();
        self.validate_proposition_term_promoted_operands(&disequality.lhs, span)?;
        self.validate_proposition_term_promoted_operands(&disequality.rhs, span)?;
        let normalizer = Normalizer::new(self);
        let lhs_norm = self.normalize_proposition_term(&normalizer, &disequality.lhs)?;
        let rhs_norm = self.normalize_proposition_term(&normalizer, &disequality.rhs)?;
        let comparison = normalizer.definitional_equality_normal_forms(&lhs_norm, &rhs_norm);
        let normalized_terms = Some(proposition_comparison_terms(
            lhs_norm.clone(),
            rhs_norm.clone(),
        ));

        if matches!(comparison, DefinitionalEqualityResult::Equal) {
            return Ok(proposition_refutation(
                proposition,
                normalized_terms,
                PropositionRefutationReason::DefinitionalEquality,
                source_anchor,
            ));
        }

        if sealed_domain_constructor_heads_are_disjoint(&lhs_norm, &rhs_norm) {
            return Ok(proposition_satisfaction(
                proposition,
                normalized_terms,
                PropositionEvidenceRule::SealedDomainConstructorDisjointness,
                source_anchor,
            ));
        }

        let kind = match comparison {
            DefinitionalEqualityResult::BlockedByNeutrality {
                neutral_subterms, ..
            } if !neutral_subterms.is_empty() => {
                proposition_deferred_kind_from_blockers(&neutral_subterms)
            }
            _ if proposition_normal_form_is_open_or_blocked(&lhs_norm)
                || proposition_normal_form_is_open_or_blocked(&rhs_norm) =>
            {
                proposition_deferred_kind_from_blocked_normals(&lhs_norm, &rhs_norm)
            }
            _ => PropositionDeferredKind::UnsupportedProofSearch,
        };

        Ok(proposition_deferral(proposition, kind, source_anchor, true))
    }

    pub(super) fn validate_proposition_term_promoted_operands(
        &self,
        term: &TypePropositionTerm,
        span: Span,
    ) -> Result<(), TypeError> {
        match term {
            TypePropositionTerm::Canonical(expr) => {
                self.validate_canonical_proposition_promoted_operands(expr, span)
            }
            TypePropositionTerm::DomainConstructorApp { args, .. } => {
                for arg in args {
                    self.validate_proposition_term_promoted_operands(arg, span)?;
                }
                Ok(())
            }
        }
    }

    pub(super) fn validate_canonical_proposition_promoted_operands(
        &self,
        expr: &CanonicalTypeExpr,
        span: Span,
    ) -> Result<(), TypeError> {
        match expr {
            CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::Var(_) => Ok(()),
            CanonicalTypeExpr::NominalApp { args, .. }
            | CanonicalTypeExpr::Projection { args, .. }
            | CanonicalTypeExpr::ComputationHeadApp { args, .. } => {
                for arg in args {
                    self.validate_canonical_proposition_promoted_operands(arg, span)?;
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
                let field_data_kind_constraints = self
                    .promoted_constructor_kind(&app.constructor)
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "promoted data constructor '{}' has no validated kinding metadata",
                                app.constructor.name
                            ),
                            span,
                        )
                    })?
                    .field_data_kind_constraints
                    .clone();
                for (index, arg) in app.args.iter().enumerate() {
                    self.validate_canonical_proposition_promoted_operands(arg, span)?;
                    if let Some(expected_kind) = field_data_kind_constraints
                        .get(index)
                        .and_then(|constraint| constraint.as_ref())
                    {
                        self.validate_canonical_promoted_data_kind(arg, expected_kind, span)?;
                    }
                }
                Ok(())
            }
            CanonicalTypeExpr::ConstructorVariableApp(app) => {
                for arg in &app.args {
                    self.validate_canonical_proposition_promoted_operands(arg, span)?;
                }
                Ok(())
            }
        }
    }

    pub(super) fn compare_proposition_terms(
        &self,
        lhs: &TypePropositionTerm,
        rhs: &TypePropositionTerm,
    ) -> Result<(DefinitionalEqualityResult, NormalTypeExpr, NormalTypeExpr), TypeError> {
        let normalizer = Normalizer::new(self);
        match (lhs, rhs) {
            (TypePropositionTerm::Canonical(lhs), TypePropositionTerm::Canonical(rhs)) => {
                let result = normalizer
                    .definitional_equality(lhs, rhs)
                    .map_err(proposition_normalization_error)?;
                let lhs_norm = match &result {
                    DefinitionalEqualityResult::Equal => {
                        normalizer
                            .normalize(lhs)
                            .map_err(proposition_normalization_error)?
                            .normal
                    }
                    DefinitionalEqualityResult::NotEqual { lhs_norm, .. }
                    | DefinitionalEqualityResult::BlockedByNeutrality { lhs_norm, .. } => {
                        lhs_norm.clone()
                    }
                };
                let rhs_norm = match &result {
                    DefinitionalEqualityResult::Equal => {
                        normalizer
                            .normalize(rhs)
                            .map_err(proposition_normalization_error)?
                            .normal
                    }
                    DefinitionalEqualityResult::NotEqual { rhs_norm, .. }
                    | DefinitionalEqualityResult::BlockedByNeutrality { rhs_norm, .. } => {
                        rhs_norm.clone()
                    }
                };
                Ok((result, lhs_norm, rhs_norm))
            }
            _ => {
                let lhs_norm = self.normalize_proposition_term(&normalizer, lhs)?;
                let rhs_norm = self.normalize_proposition_term(&normalizer, rhs)?;
                let result = normalizer.definitional_equality_normal_forms(&lhs_norm, &rhs_norm);
                Ok((result, lhs_norm, rhs_norm))
            }
        }
    }

    pub(super) fn normalize_proposition_term(
        &self,
        normalizer: &Normalizer<'_>,
        term: &TypePropositionTerm,
    ) -> Result<NormalTypeExpr, TypeError> {
        match term {
            TypePropositionTerm::Canonical(expr) => normalizer
                .normalize(expr)
                .map(|outcome| outcome.normal)
                .map_err(proposition_normalization_error),
            TypePropositionTerm::DomainConstructorApp {
                constructor,
                domain,
                args,
                kind,
            } => {
                let args = args
                    .iter()
                    .map(|arg| self.normalize_proposition_term(normalizer, arg))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(NormalTypeExpr::DomainConstructorApp {
                    constructor: constructor.clone(),
                    domain: domain.clone(),
                    args,
                    kind: kind.clone(),
                })
            }
        }
    }

    pub(super) fn lower_proposition_clause(
        &self,
        clause: &PropositionClause,
        source_origin: SourceOrigin,
    ) -> Result<LoweredPropositionClause, TypeError> {
        let source_anchor =
            proposition_source_anchor(source_origin, clause.span, "source proposition clause");
        let (proposition, outcome) = match &clause.kind {
            PropositionClauseKind::Equality { lhs, rhs, .. } => {
                let lhs = self.lower_surface_type_term(lhs)?;
                let rhs = self.lower_surface_type_term(rhs)?;
                (
                    TypeProposition::Equality(TypeEqualityProposition { lhs, rhs }),
                    None,
                )
            }
            PropositionClauseKind::Disequality { lhs, rhs, .. } => {
                let lhs = self.lower_surface_type_term(lhs)?;
                let rhs = self.lower_surface_type_term(rhs)?;
                (
                    TypeProposition::Disequality(TypeDisequalityProposition { lhs, rhs }),
                    None,
                )
            }
            PropositionClauseKind::InterfaceBound {
                subject, interface, ..
            } => {
                let subject = self.lower_surface_type_term(subject)?;
                let (interface_name, interface_args) =
                    self.interface_clause_name_and_args(interface)?;
                let interface_id = self
                    .interface_identity_for_name(&interface_name)
                    .cloned()
                    .ok_or_else(|| {
                        TypeEnvError::MissingInterface(interface_name.clone(), clause.span)
                    })?;
                let interface_args = interface_args
                    .iter()
                    .map(|arg| self.lower_surface_type_term(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                (
                    TypeProposition::InterfaceBound(InterfaceBoundProposition {
                        subject,
                        interface: interface_id,
                        interface_args,
                    }),
                    None,
                )
            }
            PropositionClauseKind::NamedPredicate {
                name,
                name_span,
                args,
            } => {
                let predicate_info = self
                    .lookup_proposition_predicate(name.as_ref())
                    .ok_or_else(|| {
                        TypeError::from(TypeEnvError::UnknownPropositionPredicate {
                            name: name.to_string(),
                            span: *name_span,
                        })
                    })?;
                if predicate_info.summary.params.len() != args.len() {
                    return Err(TypeEnvError::PropositionPredicateArityMismatch {
                        name: name.to_string(),
                        expected: predicate_info.summary.params.len(),
                        actual: args.len(),
                        span: clause.span,
                    }
                    .into());
                }
                let args = args
                    .iter()
                    .map(|arg| self.lower_surface_type_term(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                let proposition = TypeProposition::NamedPredicate(NamedPredicateProposition {
                    predicate: predicate_info.summary.id.clone(),
                    args,
                });
                let outcome = match predicate_info.solver_kind {
                    PropositionPredicateSolverKind::CompilerBuiltinSatisfied => None,
                    PropositionPredicateSolverKind::DeferredUnsupported => {
                        Some(proposition_deferral(
                            &proposition,
                            PropositionDeferredKind::UnsupportedNamedPredicate,
                            Some(source_anchor.clone()),
                            true,
                        ))
                    }
                };
                (proposition, outcome)
            }
        };
        Ok(LoweredPropositionClause {
            proposition,
            source_anchor,
            outcome,
        })
    }
}
