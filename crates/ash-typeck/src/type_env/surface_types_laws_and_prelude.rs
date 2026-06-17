use super::*;

impl TypeEnv {
    pub(super) fn lower_surface_type_term(
        &self,
        ty: &SurfaceType,
    ) -> Result<TypePropositionTerm, TypeError> {
        match ty {
            SurfaceType::Name(name) => {
                if let Some((domain, constructor)) = self.find_any_domain_constructor(name.as_ref())
                {
                    if !constructor.fields.is_empty() {
                        return Err(TypeError::ConstructorNameMismatch {
                            expected: format!(
                                "{} type arguments for sealed-domain constructor {}",
                                constructor.fields.len(),
                                constructor.exported_name
                            ),
                            found: "0".to_string(),
                            span: Span::default(),
                        });
                    }
                    return Ok(TypePropositionTerm::DomainConstructorApp {
                        constructor: constructor.id.clone(),
                        domain: domain.id.clone(),
                        args: Vec::new(),
                        kind: Kind::Type,
                    });
                }
                self.lower_surface_type_to_canonical(ty)
                    .map(proposition_term_from_canonical)
            }
            SurfaceType::Constructor { name, args } => {
                if let Some((domain, constructor)) = self.find_any_domain_constructor(name.as_ref())
                {
                    let domain = domain.clone();
                    let constructor = constructor.clone();
                    if constructor.fields.len() != args.len() {
                        return Err(TypeError::ConstructorNameMismatch {
                            expected: format!(
                                "{} type arguments for sealed-domain constructor {}",
                                constructor.fields.len(),
                                constructor.exported_name
                            ),
                            found: args.len().to_string(),
                            span: Span::default(),
                        });
                    }
                    let args = args
                        .iter()
                        .map(|arg| self.lower_surface_type_term(arg))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(TypePropositionTerm::DomainConstructorApp {
                        constructor: constructor.id,
                        domain: domain.id,
                        args,
                        kind: Kind::Type,
                    })
                } else if let Some(head) = self.local_type_function_heads.get(name.as_ref()) {
                    let args = args
                        .iter()
                        .map(|arg| self.lower_surface_type_to_canonical(arg))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(TypePropositionTerm::Canonical(
                        CanonicalTypeExpr::ComputationHeadApp {
                            head: head.clone(),
                            args,
                            kind: Kind::Type,
                        },
                    ))
                } else {
                    self.lower_surface_type_to_canonical(ty)
                        .map(proposition_term_from_canonical)
                }
            }
            _ => self
                .lower_surface_type_to_canonical(ty)
                .map(proposition_term_from_canonical),
        }
    }

    pub(super) fn interface_clause_name_and_args<'a>(
        &self,
        interface: &'a SurfaceType,
    ) -> Result<(String, &'a [SurfaceType]), TypeError> {
        match interface {
            SurfaceType::Name(name) => Ok((name.to_string(), &[])),
            SurfaceType::Constructor { name, args } => Ok((name.to_string(), args.as_slice())),
            other => Err(TypeError::ConstructorNameMismatch {
                expected: "interface name or interface type application".to_string(),
                found: surface_projection_base_spelling(other),
                span: Span::default(),
            }),
        }
    }

    pub(super) fn push_proposition_fact(
        &mut self,
        role: PropositionFactRole,
        proposition: TypeProposition,
        source_anchor: SourceAnchor,
        owner_site: PropositionCheckingSite,
        outcome: Option<PropositionOutcome>,
    ) {
        let record = PropositionFactRecord {
            proposition,
            source_anchor,
            owner_site,
            role,
            outcome,
        };
        let facts = match role {
            PropositionFactRole::Requirement => &mut self.proposition_obligations,
            PropositionFactRole::Assumption | PropositionFactRole::Evidence => {
                &mut self.proposition_assumptions
            }
        };
        if !facts.iter().any(|existing| existing == &record) {
            facts.push(record);
        }
    }

    pub(super) fn record_type_var_interface_bound_assumption(
        &mut self,
        var: TypeVar,
        interface: &str,
        source_anchor: SourceAnchor,
        owner_site: PropositionCheckingSite,
    ) {
        let Some(interface_id) = self.interface_identity_for_name(interface).cloned() else {
            return;
        };
        let proposition = TypeProposition::InterfaceBound(InterfaceBoundProposition {
            subject: type_var_proposition_term(var),
            interface: interface_id,
            interface_args: Vec::new(),
        });
        self.push_proposition_fact(
            PropositionFactRole::Assumption,
            proposition,
            source_anchor,
            owner_site,
            None,
        );
    }

    pub(super) fn record_concrete_impl_interface_assumption(
        &mut self,
        interface: &str,
        lowered_type_args: &[Type],
        source_anchor: SourceAnchor,
    ) {
        let Some(interface_id) = self.interface_identity_for_name(interface).cloned() else {
            return;
        };
        let Some((subject, interface_args)) = lowered_type_args.split_first() else {
            return;
        };
        let Some(subject) = self
            .lower_type_to_canonical_for_equality(subject)
            .map(proposition_term_from_canonical)
        else {
            return;
        };
        let Some(interface_args) = interface_args
            .iter()
            .map(|arg| {
                self.lower_type_to_canonical_for_equality(arg)
                    .map(proposition_term_from_canonical)
            })
            .collect::<Option<Vec<_>>>()
        else {
            return;
        };
        let proposition = TypeProposition::InterfaceBound(InterfaceBoundProposition {
            subject,
            interface: interface_id,
            interface_args,
        });
        self.push_proposition_fact(
            PropositionFactRole::Assumption,
            proposition,
            source_anchor,
            PropositionCheckingSite::new(
                0x8753_0000u64 + self.impls.len() as u64,
                PropositionCheckingSiteKind::ConcreteImpl,
                Some(format!("concrete impl for interface {interface}")),
            ),
            None,
        );
    }

    /// Lower a surface `Type` into the Phase 110 canonical type-expression substrate.
    pub fn lower_surface_type_to_canonical(
        &self,
        ty: &SurfaceType,
    ) -> Result<CanonicalTypeExpr, TypeError> {
        match ty {
            SurfaceType::Hole { span } => Err(TypeError::ConstructorNameMismatch {
                expected: "nominal or associated type expression supported by TASK-798 lowering"
                    .to_string(),
                found: "type hole _".to_string(),
                span: *span,
            }),
            SurfaceType::Name(name) => match name.as_ref() {
                "Int" | "String" | "Bool" | "Float" | "Null" | "Time" | "Ref" => {
                    Ok(CanonicalTypeExpr::Primitive(name.to_string()))
                }
                _ => {
                    if let Some(kind) = self.type_parameter_kind(name.as_ref()) {
                        if kind.is_type() {
                            Ok(CanonicalTypeExpr::Var(name.to_string()))
                        } else {
                            Err(TypeError::from(TypeEnvError::InvalidDefinition(
                                format!(
                                    "constructor variable '{}' has kind {}; expected a fully applied proper type",
                                    name, kind
                                ),
                                Span::default(),
                            )))
                        }
                    } else {
                        match self.resolve_type(name.as_ref()) {
                            Ok((qualified, _)) => {
                                self.check_type_constructor_arity(&qualified, 0)?;
                                Ok(CanonicalTypeExpr::NominalApp {
                                    origin: self
                                        .canonical_type_identity_for_visible_name(name.as_ref())?,
                                    visible_name: name.to_string(),
                                    args: vec![],
                                    kind: Kind::Type,
                                })
                            }
                            Err(TypeError::UnboundVariable(_, _)) => {
                                Ok(CanonicalTypeExpr::Var(name.to_string()))
                            }
                            Err(err) => Err(err),
                        }
                    }
                }
            },
            SurfaceType::Constructor { name, args } => {
                if let Some(kind) = self.type_parameter_kind(name.as_ref()) {
                    if kind.is_type() {
                        return Err(TypeError::from(TypeEnvError::InvalidDefinition(
                            format!(
                                "proper type variable '{}' of kind * cannot be applied as a constructor",
                                name
                            ),
                            Span::default(),
                        )));
                    }
                    let expected_arity = kind.arity();
                    if args.len() != expected_arity {
                        return Err(TypeError::from(TypeEnvError::InvalidDefinition(
                            format!(
                                "wrong arity for constructor variable '{}': expected {}, found {}",
                                name,
                                expected_arity,
                                args.len()
                            ),
                            Span::default(),
                        )));
                    }
                    let lowered_args = args
                        .iter()
                        .map(|arg| self.lower_surface_type_to_canonical(arg))
                        .collect::<Result<Vec<_>, _>>()?;
                    return Ok(CanonicalTypeExpr::ConstructorVariableApp(Box::new(
                        ConstructorVariableApp::new(
                            ConstructorVariableRef::new(name.to_string(), kind.clone(), None),
                            lowered_args,
                            Kind::Type,
                            None,
                        ),
                    )));
                }
                let (qualified, _) =
                    self.resolve_type(name.as_ref()).map_err(|err| match err {
                        TypeError::UnboundVariable(_, span) => {
                            TypeError::from(TypeEnvError::InvalidDefinition(
                                format!(
                                    "constructor-variable application '{}<...>' cannot be lowered until TASK-907 tracks constructor variables",
                                    name
                                ),
                                span,
                            ))
                        }
                        err => err,
                    })?;
                self.check_type_constructor_arity(&qualified, args.len())?;
                Ok(CanonicalTypeExpr::NominalApp {
                    origin: self.canonical_type_identity_for_visible_name(name.as_ref())?,
                    visible_name: name.to_string(),
                    args: args
                        .iter()
                        .map(|arg| self.lower_surface_type_to_canonical(arg))
                        .collect::<Result<Vec<_>, _>>()?,
                    kind: Kind::Type,
                })
            }
            SurfaceType::Associated { base, name } => {
                if let SurfaceType::Constructor {
                    name: interface,
                    args,
                } = base.as_ref()
                    && self
                        .lookup_associated_family_declaration(interface, name)
                        .is_some()
                {
                    return self.lower_explicit_associated_family_projection_to_canonical(
                        interface,
                        args,
                        name,
                        Span::default(),
                    );
                }
                if matches!(base.as_ref(), SurfaceType::Associated { .. }) {
                    return Err(TypeError::ConstructorNameMismatch {
                        expected: "supported associated projection base (nested projection bases are unsupported)"
                            .to_string(),
                        found: format!("nested projection base {base:?}"),
                        span: Span::default(),
                    });
                }
                if matches!(
                    base.as_ref(),
                    SurfaceType::Hole { .. }
                        | SurfaceType::Tuple(_)
                        | SurfaceType::Record(_)
                        | SurfaceType::List(_)
                        | SurfaceType::Capability(_)
                        | SurfaceType::Fn(_, _)
                ) {
                    let found = match base.as_ref() {
                        SurfaceType::Tuple(items) => {
                            format!("unsupported projection base Tuple({})", items.len())
                        }
                        SurfaceType::Record(fields) => {
                            format!("unsupported projection base Record({})", fields.len())
                        }
                        SurfaceType::List(_) => "unsupported projection base List".to_string(),
                        SurfaceType::Capability(name) => {
                            format!("unsupported projection base Capability({name})")
                        }
                        SurfaceType::Fn(_, _) => "unsupported projection base Fn".to_string(),
                        SurfaceType::Hole { .. } => {
                            "unsupported projection base type hole _".to_string()
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
                let lowered_base = self.lower_surface_type_to_canonical(base)?;
                self.lower_associated_projection_to_canonical(&lowered_base, name)
            }
            SurfaceType::Tuple(items) => Err(TypeError::ConstructorNameMismatch {
                expected: "nominal or associated type expression supported by TASK-798 lowering"
                    .to_string(),
                found: format!("Tuple({})", items.len()),
                span: Span::default(),
            }),
            SurfaceType::Record(fields) => Err(TypeError::ConstructorNameMismatch {
                expected: "nominal or associated type expression supported by TASK-798 lowering"
                    .to_string(),
                found: format!("Record({})", fields.len()),
                span: Span::default(),
            }),
            SurfaceType::List(_) => Err(TypeError::ConstructorNameMismatch {
                expected: "nominal or associated type expression supported by TASK-798 lowering"
                    .to_string(),
                found: "List".to_string(),
                span: Span::default(),
            }),
            SurfaceType::Capability(name) => Err(TypeError::ConstructorNameMismatch {
                expected: "nominal or associated type expression supported by TASK-798 lowering"
                    .to_string(),
                found: format!("Capability({name})"),
                span: Span::default(),
            }),
            SurfaceType::Fn(_, _) => Err(TypeError::ConstructorNameMismatch {
                expected: "nominal or associated type expression supported by TASK-798 lowering"
                    .to_string(),
                found: "Fn".to_string(),
                span: Span::default(),
            }),
            SurfaceType::AssociatedFamilyProjection {
                interface,
                args,
                member,
                span,
            } => self.lower_explicit_associated_family_projection_to_canonical(
                interface, args, member, *span,
            ),
        }
    }

    /// Elaborate an audited explicit do-target type into a core constructor
    /// expression, preserving exactly one source hole as a partial application.
    ///
    /// This is the TASK-901 semantic substrate only: it validates kind/arity and
    /// hole placement for MVP partial target shapes without selecting Monad
    /// evidence or integrating with do-target resolution.
    pub fn elaborate_do_target_constructor_expr(
        &self,
        ty: &SurfaceType,
    ) -> Result<TypeConstructorExpr, PartialConstructorElaborationError> {
        self.elaborate_partial_type_constructor(ty, true)
    }

    /// Elaborate a surface type/constructor expression into the core
    /// `TypeConstructorExpr` carrier used by partial-constructor consumers.
    pub fn elaborate_partial_type_constructor(
        &self,
        ty: &SurfaceType,
        require_partial_target: bool,
    ) -> Result<TypeConstructorExpr, PartialConstructorElaborationError> {
        match ty {
            SurfaceType::Name(name) => {
                let constructor = name.to_string();
                let arity = self
                    .type_constructor_arity_for_visible_name(name.as_ref())
                    .ok_or_else(|| PartialConstructorElaborationError::UnknownConstructor {
                        constructor: constructor.clone(),
                        span: Span::default(),
                    })?;
                if require_partial_target {
                    if arity > 1 {
                        return Err(
                            PartialConstructorElaborationError::BareHigherArityConstructor {
                                constructor: constructor.clone(),
                                arity,
                                hint: bare_constructor_hole_hint(&constructor, arity),
                                span: Span::default(),
                            },
                        );
                    }
                    return Err(PartialConstructorElaborationError::MissingHole {
                        constructor,
                        span: Span::default(),
                    });
                }
                if arity == 0 {
                    return self
                        .lower_surface_type_to_canonical(ty)
                        .map(TypeConstructorExpr::ProperType)
                        .map_err(|err| {
                            PartialConstructorElaborationError::ArgumentLoweringFailed {
                                constructor,
                                reason: err.to_string(),
                                span: Span::default(),
                            }
                        });
                }
                let origin = self
                    .canonical_type_identity_for_visible_name(name.as_ref())
                    .map_err(
                        |err| PartialConstructorElaborationError::ArgumentLoweringFailed {
                            constructor: constructor.clone(),
                            reason: err.to_string(),
                            span: Span::default(),
                        },
                    )?;
                Ok(TypeConstructorExpr::ConstructorHead(
                    TypeConstructorHeadId::nominal(origin, constructor),
                ))
            }
            SurfaceType::Constructor { name, args } => self.elaborate_constructor_application(
                name.as_ref(),
                args,
                require_partial_target,
                Span::default(),
            ),
            SurfaceType::AssociatedFamilyProjection { span, .. } => {
                if surface_type_contains_hole(ty) {
                    return Err(PartialConstructorElaborationError::NoInversionBoundary {
                        context: "associated-family projection output".to_string(),
                        span: *span,
                    });
                }
                if require_partial_target {
                    return Err(PartialConstructorElaborationError::MissingHole {
                        constructor: "associated-family projection".to_string(),
                        span: *span,
                    });
                }
                self.lower_surface_type_to_canonical(ty)
                    .map(TypeConstructorExpr::ProperType)
                    .map_err(
                        |err| PartialConstructorElaborationError::ArgumentLoweringFailed {
                            constructor: "associated-family projection".to_string(),
                            reason: err.to_string(),
                            span: *span,
                        },
                    )
            }
            SurfaceType::Associated { base, name } => {
                if surface_type_contains_hole(base) {
                    return Err(PartialConstructorElaborationError::NoInversionBoundary {
                        context: format!("associated projection `{name}`"),
                        span: Span::default(),
                    });
                }
                if require_partial_target {
                    return Err(PartialConstructorElaborationError::MissingHole {
                        constructor: format!("associated projection `{name}`"),
                        span: Span::default(),
                    });
                }
                self.lower_surface_type_to_canonical(ty)
                    .map(TypeConstructorExpr::ProperType)
                    .map_err(
                        |err| PartialConstructorElaborationError::ArgumentLoweringFailed {
                            constructor: name.to_string(),
                            reason: err.to_string(),
                            span: Span::default(),
                        },
                    )
            }
            SurfaceType::Hole { span } => Err(
                PartialConstructorElaborationError::UnsupportedHolePosition {
                    reason: "bare `_` has no constructor head or expected value slot".to_string(),
                    span: *span,
                },
            ),
            SurfaceType::List(_)
            | SurfaceType::Tuple(_)
            | SurfaceType::Record(_)
            | SurfaceType::Capability(_)
            | SurfaceType::Fn(_, _) => {
                if surface_type_contains_hole(ty) {
                    return Err(
                        PartialConstructorElaborationError::UnsupportedHolePosition {
                            reason:
                                "holes are enabled only in explicit constructor argument spines"
                                    .to_string(),
                            span: Span::default(),
                        },
                    );
                }
                if require_partial_target {
                    return Err(PartialConstructorElaborationError::MissingHole {
                        constructor: "proper type expression".to_string(),
                        span: Span::default(),
                    });
                }
                self.lower_surface_type_to_canonical(ty)
                    .map(TypeConstructorExpr::ProperType)
                    .map_err(
                        |err| PartialConstructorElaborationError::ArgumentLoweringFailed {
                            constructor: "proper type expression".to_string(),
                            reason: err.to_string(),
                            span: Span::default(),
                        },
                    )
            }
        }
    }

    pub(super) fn elaborate_constructor_application(
        &self,
        constructor: &str,
        args: &[SurfaceType],
        require_partial_target: bool,
        span: Span,
    ) -> Result<TypeConstructorExpr, PartialConstructorElaborationError> {
        let Some(expected_arity) = self.type_constructor_arity_for_visible_name(constructor) else {
            return Err(PartialConstructorElaborationError::UnknownConstructor {
                constructor: constructor.to_string(),
                span,
            });
        };
        if args.len() != expected_arity {
            return Err(PartialConstructorElaborationError::WrongArity {
                constructor: constructor.to_string(),
                expected_arity,
                found_arity: args.len(),
                span,
            });
        }

        let hole_count = args.iter().map(surface_type_hole_count).sum::<usize>();
        if require_partial_target && hole_count == 0 {
            return Err(PartialConstructorElaborationError::MissingHole {
                constructor: constructor.to_string(),
                span,
            });
        }
        if hole_count > 1 {
            return Err(PartialConstructorElaborationError::MultipleHoles {
                constructor: constructor.to_string(),
                count: hole_count,
                span,
            });
        }

        let origin = self
            .canonical_type_identity_for_visible_name(constructor)
            .map_err(
                |err| PartialConstructorElaborationError::ArgumentLoweringFailed {
                    constructor: constructor.to_string(),
                    reason: err.to_string(),
                    span,
                },
            )?;
        if hole_count == 0 {
            return self
                .lower_surface_type_to_canonical(&SurfaceType::Constructor {
                    name: constructor.into(),
                    args: args.to_vec(),
                })
                .map(TypeConstructorExpr::ProperType)
                .map_err(
                    |err| PartialConstructorElaborationError::ArgumentLoweringFailed {
                        constructor: constructor.to_string(),
                        reason: err.to_string(),
                        span,
                    },
                );
        }

        let mut partial_args = Vec::with_capacity(args.len());
        let mut hole_metadata = Vec::with_capacity(1);
        for arg in args {
            match arg {
                SurfaceType::Hole { span: hole_span } => {
                    let id = TypeHoleId::new(hole_metadata.len() as u64);
                    partial_args.push(PartialTypeArg::Hole(id));
                    hole_metadata.push(TypeHoleMetadata::new(
                        id,
                        span_anchor(*hole_span, "type hole"),
                        Some(Kind::Type),
                        TypeHoleAmbiguity::ExpectedValueSlot,
                    ));
                }
                SurfaceType::AssociatedFamilyProjection { span, .. } => {
                    if surface_type_contains_hole(arg) {
                        return Err(PartialConstructorElaborationError::NoInversionBoundary {
                            context: "associated-family projection output".to_string(),
                            span: *span,
                        });
                    }
                    partial_args.push(PartialTypeArg::Applied(Box::new(
                        self.lower_surface_type_to_canonical(arg).map_err(|err| {
                            PartialConstructorElaborationError::ArgumentLoweringFailed {
                                constructor: constructor.to_string(),
                                reason: err.to_string(),
                                span: *span,
                            }
                        })?,
                    )));
                }
                SurfaceType::Associated { .. } if surface_type_contains_hole(arg) => {
                    return Err(PartialConstructorElaborationError::NoInversionBoundary {
                        context: "associated projection".to_string(),
                        span,
                    });
                }
                other if surface_type_contains_hole(other) => {
                    return Err(
                        PartialConstructorElaborationError::UnsupportedHolePosition {
                            reason: "nested holes are not enabled for MVP partial targets"
                                .to_string(),
                            span,
                        },
                    );
                }
                other => partial_args.push(PartialTypeArg::Applied(Box::new(
                    self.lower_surface_type_to_canonical(other).map_err(|err| {
                        PartialConstructorElaborationError::ArgumentLoweringFailed {
                            constructor: constructor.to_string(),
                            reason: err.to_string(),
                            span,
                        }
                    })?,
                ))),
            }
        }

        Ok(TypeConstructorExpr::PartialApplication(
            PartialTypeConstructorApp::new_with_hole_metadata(
                TypeConstructorHeadId::nominal(origin, constructor.to_string()),
                partial_args,
                Kind::n_ary(hole_count),
                hole_metadata,
                Some(span_anchor(
                    span,
                    format!("partial application {constructor}"),
                )),
            ),
        ))
    }

    pub(super) fn type_constructor_arity_for_visible_name(&self, name: &str) -> Option<usize> {
        match name {
            "Int" | "String" | "Bool" | "Float" | "Null" | "Unit" | "Time" | "Ref" | "()" => {
                Some(0)
            }
            _ => self
                .type_info
                .get(name)
                .map(TypeInfo::type_arg_count)
                .or_else(|| self.ast_types.get(name).map(|def| def.params.len())),
        }
    }

    #[must_use]
    pub fn type_identity_for_name(&self, name: &str) -> Option<&TypeDeclId> {
        self.type_alias_identities.get(name)
    }

    #[must_use]
    pub fn interface_identity_for_name(&self, name: &str) -> Option<&InterfaceIdentityId> {
        self.interface_identity_aliases.get(name)
    }

    #[must_use]
    pub fn associated_member_identity_for_interface_member(
        &self,
        interface_name: &str,
        member_name: &str,
    ) -> Option<&AssociatedMemberIdentityId> {
        self.associated_member_identity_aliases
            .get(&(interface_name.to_string(), member_name.to_string()))
    }

    #[must_use]
    pub fn interface_identity_known(&self, id: &InterfaceIdentityId) -> bool {
        self.known_interface_identities.contains(id)
    }

    #[must_use]
    pub fn associated_member_identity_known(&self, id: &AssociatedMemberIdentityId) -> bool {
        self.known_associated_member_identities.contains(id)
    }

    #[must_use]
    pub fn canonical_type_name(&self, id: &TypeDeclId) -> Option<&String> {
        self.canonical_type_names.get(id)
    }

    pub(super) fn canonical_constructor_name_for_equality(
        &self,
        name: &QualifiedName,
    ) -> QualifiedName {
        if !name.is_root() {
            return name.clone();
        }

        self.type_alias_identities
            .get(name.name.as_str())
            .and_then(|id| self.canonical_type_names.get(id))
            .map(|canonical| QualifiedName::root(canonical.clone()))
            .unwrap_or_else(|| name.clone())
    }

    pub(super) fn associated_member_identity_for_visible_interface_member(
        &self,
        interface_name: &str,
        member_name: &str,
    ) -> Option<&AssociatedMemberIdentityId> {
        if let Some(member) =
            self.associated_member_identity_for_interface_member(interface_name, member_name)
        {
            return Some(member);
        }

        let interface_id = self.interface_identity_for_name(interface_name)?;
        self.associated_member_identity_aliases
            .iter()
            .find_map(|((_, visible_member), member)| {
                (visible_member == member_name && &member.interface == interface_id)
                    .then_some(member)
            })
    }

    pub(super) fn canonical_associated_projection_for_equality(
        &self,
        interface_name: &str,
        member_name: &str,
    ) -> Option<(String, String)> {
        let interface_id = self.interface_identity_for_name(interface_name)?;
        let member_id = self
            .associated_member_identity_for_visible_interface_member(interface_name, member_name)?;

        if &member_id.interface != interface_id {
            return None;
        }

        let canonical_interface = self
            .canonical_interface_names
            .get(interface_id)
            .cloned()
            .unwrap_or_else(|| interface_name.to_string());

        Some((canonical_interface, member_id.name.clone()))
    }

    /// Returns the canonical target of a transparent nominal alias application
    /// when all alias arguments are representable in the current type API.
    ///
    /// This helper is intentionally narrow for the Phase 112 normalizer: it only
    /// peels already-registered transparent aliases at normalizer inputs and does
    /// not force associated projections or install new equality forcing points.
    #[must_use]
    pub fn transparent_alias_canonical_target(
        &self,
        origin: &TypeDeclId,
        visible_name: &str,
        args: &[CanonicalTypeExpr],
    ) -> Option<CanonicalTypeExpr> {
        let registered_origin = self
            .type_identity_for_name(visible_name)
            .cloned()
            .unwrap_or_else(|| fallback_canonical_type_decl_id(visible_name));
        if registered_origin != *origin {
            return None;
        }
        let mut bridge = AliasCanonicalVarBridge::default();
        let type_args: Vec<_> = args
            .iter()
            .map(|arg| bridge.placeholder_for_arg(arg))
            .collect();
        let target =
            self.transparent_alias_target(&QualifiedName::root(visible_name), &type_args)?;
        self.type_to_canonical_expr_for_alias(&target, &bridge)
            .map(|target| self.canonical_expr_with_registered_origin(target))
    }

    pub(super) fn canonical_expr_with_registered_origin(
        &self,
        expr: CanonicalTypeExpr,
    ) -> CanonicalTypeExpr {
        match expr {
            CanonicalTypeExpr::NominalApp {
                visible_name,
                args,
                kind,
                origin,
            } => CanonicalTypeExpr::NominalApp {
                origin: self
                    .type_identity_for_name(&visible_name)
                    .cloned()
                    .unwrap_or(origin),
                visible_name,
                args,
                kind,
            },
            other => other,
        }
    }

    pub(super) fn type_to_canonical_expr_for_alias(
        &self,
        ty: &Type,
        bridge: &AliasCanonicalVarBridge,
    ) -> Option<CanonicalTypeExpr> {
        match ty {
            Type::Int => Some(CanonicalTypeExpr::Primitive("Int".to_string())),
            Type::String => Some(CanonicalTypeExpr::Primitive("String".to_string())),
            Type::Bool => Some(CanonicalTypeExpr::Primitive("Bool".to_string())),
            Type::Float => Some(CanonicalTypeExpr::Primitive("Float".to_string())),
            Type::Null => Some(CanonicalTypeExpr::Primitive("Null".to_string())),
            Type::Time => Some(CanonicalTypeExpr::Primitive("Time".to_string())),
            Type::Ref => Some(CanonicalTypeExpr::Primitive("Ref".to_string())),
            Type::Var(var) => bridge
                .args
                .get(var)
                .cloned()
                .or_else(|| Some(CanonicalTypeExpr::Var(format!("T{}", var.0)))),
            Type::Constructor { name, args, kind } if name.is_root() => {
                let args = args
                    .iter()
                    .map(|arg| self.type_to_canonical_expr_for_alias(arg, bridge))
                    .collect::<Option<_>>()?;
                Some(CanonicalTypeExpr::NominalApp {
                    origin: self
                        .type_identity_for_name(&name.name)
                        .cloned()
                        .unwrap_or_else(|| fallback_canonical_type_decl_id(&name.name)),
                    visible_name: name.name.clone(),
                    args,
                    kind: kind.clone(),
                })
            }
            Type::Associated {
                interface,
                base,
                name,
            } => {
                let base = self.type_to_canonical_expr_for_alias(base, bridge)?;
                self.lower_associated_projection_to_canonical(&base, name)
                    .ok()
                    .map(|projection| match projection {
                        CanonicalTypeExpr::Projection {
                            interface: projection_interface,
                            member,
                            args,
                            kind,
                            rigidity,
                        } if projection_interface.name == *interface => {
                            CanonicalTypeExpr::Projection {
                                interface: projection_interface,
                                member,
                                args,
                                kind,
                                rigidity,
                            }
                        }
                        other => other,
                    })
            }
            Type::List(_)
            | Type::Record(_)
            | Type::Cap { .. }
            | Type::Fun(_, _, _)
            | Type::Fn(_, _)
            | Type::ConstructorVariableApp { .. }
            | Type::Instance { .. }
            | Type::InstanceAddr { .. }
            | Type::ControlLink { .. }
            | Type::Constructor { .. } => None,
        }
    }

    /// Recursively peel registered transparent aliases inside a type without
    /// changing current equality/unification boundaries. This helper is for
    /// later boundary adoption tasks; callers that want existing nominal
    /// equality behavior should continue using `canonicalize_type_for_equality`.
    #[must_use]
    pub fn canonicalize_transparent_aliases(&self, ty: &Type) -> Type {
        match ty {
            Type::Constructor { name, args, kind } => {
                let canonical_args: Vec<_> = args
                    .iter()
                    .map(|arg| self.canonicalize_transparent_aliases(arg))
                    .collect();

                if let Some(target) = self.transparent_alias_target(name, &canonical_args) {
                    self.canonicalize_transparent_aliases(&target)
                } else {
                    Type::Constructor {
                        name: name.clone(),
                        args: canonical_args,
                        kind: kind.clone(),
                    }
                }
            }
            Type::ConstructorVariableApp {
                constructor,
                args,
                kind,
            } => Type::ConstructorVariableApp {
                constructor: constructor.clone(),
                args: args
                    .iter()
                    .map(|arg| self.canonicalize_transparent_aliases(arg))
                    .collect(),
                kind: kind.clone(),
            },
            Type::List(inner) => Type::List(Box::new(self.canonicalize_transparent_aliases(inner))),
            Type::Record(fields) => Type::Record(
                fields
                    .iter()
                    .map(|(name, ty)| (name.clone(), self.canonicalize_transparent_aliases(ty)))
                    .collect(),
            ),
            Type::Fn(params, ret) => Type::Fn(
                params
                    .iter()
                    .map(|param| self.canonicalize_transparent_aliases(param))
                    .collect(),
                Box::new(self.canonicalize_transparent_aliases(ret)),
            ),
            Type::Fun(params, ret, effect) => Type::Fun(
                params
                    .iter()
                    .map(|param| self.canonicalize_transparent_aliases(param))
                    .collect(),
                Box::new(self.canonicalize_transparent_aliases(ret)),
                *effect,
            ),
            Type::Associated {
                interface,
                base,
                name,
            } => Type::Associated {
                interface: interface.clone(),
                base: Box::new(self.canonicalize_transparent_aliases(base)),
                name: name.clone(),
            },
            other => other.clone(),
        }
    }

    #[must_use]
    pub fn render_type_for_diagnostics(&self, ty: &Type) -> String {
        ty.to_string()
    }

    #[must_use]
    pub fn canonicalize_type_for_equality(&self, ty: &Type) -> Type {
        match ty {
            Type::Constructor { name, args, kind } => {
                let canonical_args: Vec<_> = args
                    .iter()
                    .map(|arg| self.canonicalize_type_for_equality(arg))
                    .collect();

                if let Some(target) = self.transparent_alias_target(name, &canonical_args) {
                    self.canonicalize_type_for_equality(&target)
                } else {
                    Type::Constructor {
                        name: self.canonical_constructor_name_for_equality(name),
                        args: canonical_args,
                        kind: kind.clone(),
                    }
                }
            }
            Type::ConstructorVariableApp {
                constructor,
                args,
                kind,
            } => Type::ConstructorVariableApp {
                constructor: constructor.clone(),
                args: args
                    .iter()
                    .map(|arg| self.canonicalize_type_for_equality(arg))
                    .collect(),
                kind: kind.clone(),
            },
            Type::List(inner) => Type::List(Box::new(self.canonicalize_type_for_equality(inner))),
            Type::Record(fields) => Type::Record(
                fields
                    .iter()
                    .map(|(name, ty)| (name.clone(), self.canonicalize_type_for_equality(ty)))
                    .collect(),
            ),
            Type::Fn(params, ret) => Type::Fn(
                params
                    .iter()
                    .map(|param| self.canonicalize_type_for_equality(param))
                    .collect(),
                Box::new(self.canonicalize_type_for_equality(ret)),
            ),
            Type::Fun(params, ret, effect) => Type::Fun(
                params
                    .iter()
                    .map(|param| self.canonicalize_type_for_equality(param))
                    .collect(),
                Box::new(self.canonicalize_type_for_equality(ret)),
                *effect,
            ),
            Type::Associated {
                interface,
                base,
                name,
            } => {
                let (canonical_interface, canonical_name) = self
                    .canonical_associated_projection_for_equality(interface, name)
                    .unwrap_or_else(|| (interface.clone(), name.clone()));

                Type::Associated {
                    interface: canonical_interface,
                    base: Box::new(self.canonicalize_type_for_equality(base)),
                    name: canonical_name,
                }
            }
            other => other.clone(),
        }
    }

    /// Canonicalize a scrutinee type for pattern typing and exhaustiveness.
    ///
    /// Unlike equality canonicalization, this API only succeeds when the result
    /// is a concrete ordinary enum ADT with a known constructor universe.
    #[must_use]
    pub fn canonicalize_type_for_pattern(&self, ty: &Type) -> PatternCanonicalization {
        let source_type = ty.clone();
        let candidate = match self.pattern_canonical_candidate_type(ty) {
            Ok(candidate) => candidate,
            Err(reason) => {
                return PatternCanonicalization::Blocked {
                    source_type,
                    reason,
                };
            }
        };

        let Type::Constructor { name, args, kind } = candidate else {
            return PatternCanonicalization::Blocked {
                source_type,
                reason: PatternCanonicalizationBlockedReason::NonAdt,
            };
        };

        if !name.is_root() {
            return PatternCanonicalization::Blocked {
                source_type,
                reason: PatternCanonicalizationBlockedReason::UnknownType { name },
            };
        }

        let canonical_name = self.canonical_constructor_name_for_equality(&name);
        let canonical_type = Type::Constructor {
            name: canonical_name.clone(),
            args: args.clone(),
            kind,
        };

        if args.iter().any(Self::pattern_type_contains_unresolved_var) {
            return PatternCanonicalization::Blocked {
                source_type,
                reason: PatternCanonicalizationBlockedReason::NonConcreteTypeArgument,
            };
        }

        match self.pattern_constructors_for_adt(&canonical_name, &args) {
            Ok(constructors) => PatternCanonicalization::Matchable(PatternCanonicalType {
                source_type,
                canonical_type,
                canonical_name,
                canonical_type_args: args,
                constructors,
            }),
            Err(reason) => PatternCanonicalization::Blocked {
                source_type,
                reason,
            },
        }
    }

    pub(super) fn pattern_canonical_candidate_type(
        &self,
        ty: &Type,
    ) -> Result<Type, PatternCanonicalizationBlockedReason> {
        match ty {
            Type::Associated {
                interface, name, ..
            } => self
                .pattern_normalize_associated_projection(ty)
                .map_err(
                    |()| PatternCanonicalizationBlockedReason::RigidAssociatedProjection {
                        interface: interface.clone(),
                        member: name.clone(),
                    },
                ),
            Type::Var(_) => Err(PatternCanonicalizationBlockedReason::TypeVariable),
            Type::ConstructorVariableApp { constructor, .. } => Err(
                PatternCanonicalizationBlockedReason::ConstructorVariableApplication {
                    constructor: constructor.clone(),
                },
            ),
            _ => Ok(self.canonicalize_type_for_equality(ty)),
        }
    }

    pub(super) fn pattern_type_contains_unresolved_var(ty: &Type) -> bool {
        match ty {
            Type::Var(_) => true,
            Type::List(inner) => Self::pattern_type_contains_unresolved_var(inner),
            Type::Record(fields) => fields
                .iter()
                .any(|(_, field_ty)| Self::pattern_type_contains_unresolved_var(field_ty)),
            Type::Fn(params, ret) => {
                params
                    .iter()
                    .any(Self::pattern_type_contains_unresolved_var)
                    || Self::pattern_type_contains_unresolved_var(ret)
            }
            Type::Fun(params, ret, _) => {
                params
                    .iter()
                    .any(Self::pattern_type_contains_unresolved_var)
                    || Self::pattern_type_contains_unresolved_var(ret)
            }
            Type::Constructor { args, .. } | Type::ConstructorVariableApp { args, .. } => {
                args.iter().any(Self::pattern_type_contains_unresolved_var)
            }
            Type::Associated { base, .. } => Self::pattern_type_contains_unresolved_var(base),
            Type::Int
            | Type::String
            | Type::Bool
            | Type::Float
            | Type::Null
            | Type::Time
            | Type::Ref
            | Type::Cap { .. }
            | Type::Instance { .. }
            | Type::InstanceAddr { .. }
            | Type::ControlLink { .. } => false,
        }
    }

    pub(super) fn pattern_normalize_associated_projection(&self, ty: &Type) -> Result<Type, ()> {
        let canonical = self.type_to_canonical_expr_for_equality(ty).ok_or(())?;
        let outcome = Normalizer::new(self)
            .normalize(&canonical)
            .map_err(|_| ())?;
        self.normal_type_to_pattern_type(&outcome.normal).ok_or(())
    }

    pub(super) fn normal_type_to_pattern_type(&self, normal: &NormalTypeExpr) -> Option<Type> {
        match normal {
            NormalTypeExpr::Primitive(name) => primitive_pattern_type(name),
            NormalTypeExpr::NominalApp {
                visible_name,
                args,
                kind,
                ..
            } => {
                let args = args
                    .iter()
                    .map(|arg| self.normal_type_to_pattern_type(arg))
                    .collect::<Option<Vec<_>>>()?;
                let name = self.canonical_constructor_name_for_equality(&QualifiedName::root(
                    visible_name.clone(),
                ));
                Some(Type::Constructor {
                    name,
                    args,
                    kind: kind.clone(),
                })
            }
            NormalTypeExpr::Var(_)
            | NormalTypeExpr::ConstructorVariableApp { .. }
            | NormalTypeExpr::NeutralComputationApp { .. }
            | NormalTypeExpr::Projection { .. }
            | NormalTypeExpr::DomainConstructorApp { .. }
            | NormalTypeExpr::PromotedDataConstructorApp { .. } => None,
        }
    }

    pub(super) fn pattern_constructors_for_adt(
        &self,
        name: &QualifiedName,
        args: &[Type],
    ) -> Result<Vec<PatternCanonicalConstructor>, PatternCanonicalizationBlockedReason> {
        let unfolded = self.unfold_constructor(name, args).map_err(|_| {
            PatternCanonicalizationBlockedReason::UnknownType { name: name.clone() }
        })?;

        let UnfoldedBody::Enum(variants) = unfolded else {
            return Err(PatternCanonicalizationBlockedReason::NonAdt);
        };

        let mut constructors = Vec::with_capacity(variants.len());
        for (variant_index, variant) in variants.into_iter().enumerate() {
            match self.constructors.get(&variant.name) {
                Some((constructor_type, constructor_index))
                    if constructor_type == &name.name && *constructor_index == variant_index => {}
                _ => {
                    return Err(
                        PatternCanonicalizationBlockedReason::UnknownConstructorUniverse {
                            name: name.clone(),
                        },
                    );
                }
            }

            constructors.push(PatternCanonicalConstructor {
                name: variant.name,
                variant_index,
                fields: variant.fields,
                payload_shape: variant.payload_shape,
            });
        }

        Ok(constructors)
    }

    /// Unify types using TypeEnv's canonical imported-summary identity map.
    pub fn unify_types(&self, left: &Type, right: &Type) -> Result<Substitution, UnifyError> {
        if self
            .definitionally_equal_types_when_canonicalizable(left, right)
            .is_some_and(|equal| equal)
        {
            return Ok(Substitution::new());
        }

        unify(
            &self.canonicalize_type_for_equality(left),
            &self.canonicalize_type_for_equality(right),
        )
    }

    #[must_use]
    pub fn types_equivalent_for_equality(&self, left: &Type, right: &Type) -> bool {
        self.definitionally_equal_types_when_canonicalizable(left, right)
            .unwrap_or_else(|| self.unify_types(left, right).is_ok())
    }

    /// TASK-826 guarded TypeEnv forcing-point helper.
    ///
    /// This wrapper consumes the TASK-817 matrix only at the central TypeEnv
    /// equality boundary: if both current `Type` values can be represented in the
    /// Phase 110 canonical IR, compare their normal forms through the SPEC-060
    /// normalizer/definitional-equality API. Unsupported legacy shapes and
    /// inference-meta solving remain owned by the fallback `Type` unifier.
    #[must_use]
    pub(super) fn definitionally_equal_types_when_canonicalizable(
        &self,
        left: &Type,
        right: &Type,
    ) -> Option<bool> {
        let left = self.canonicalize_type_for_equality(left);
        let right = self.canonicalize_type_for_equality(right);
        let left = self.type_to_canonical_expr_for_equality(&left)?;
        let right = self.type_to_canonical_expr_for_equality(&right)?;
        let evidence = Normalizer::new(self)
            .definitional_equality(&left, &right)
            .ok()?;
        Some(matches!(evidence, DefinitionalEqualityResult::Equal))
    }

    #[must_use]
    pub fn lower_type_to_canonical_for_equality(&self, ty: &Type) -> Option<CanonicalTypeExpr> {
        let ty = self.canonicalize_type_for_equality(ty);
        self.type_to_canonical_expr_for_equality(&ty)
    }

    pub(super) fn type_to_canonical_expr_for_equality(
        &self,
        ty: &Type,
    ) -> Option<CanonicalTypeExpr> {
        match ty {
            Type::Int => Some(CanonicalTypeExpr::Primitive("Int".to_string())),
            Type::String => Some(CanonicalTypeExpr::Primitive("String".to_string())),
            Type::Bool => Some(CanonicalTypeExpr::Primitive("Bool".to_string())),
            Type::Float => Some(CanonicalTypeExpr::Primitive("Float".to_string())),
            Type::Null => Some(CanonicalTypeExpr::Primitive("Null".to_string())),
            Type::Time => Some(CanonicalTypeExpr::Primitive("Time".to_string())),
            Type::Ref => Some(CanonicalTypeExpr::Primitive("Ref".to_string())),
            Type::Var(_) => None,
            Type::Constructor { name, args, kind } if name.is_root() => {
                let args = args
                    .iter()
                    .map(|arg| self.type_to_canonical_expr_for_equality(arg))
                    .collect::<Option<_>>()?;
                let canonical_name = self.canonical_constructor_name_for_equality(name);
                Some(CanonicalTypeExpr::NominalApp {
                    origin: self
                        .type_identity_for_name(&canonical_name.name)
                        .cloned()
                        .unwrap_or_else(|| fallback_canonical_type_decl_id(&canonical_name.name)),
                    visible_name: canonical_name.name,
                    args,
                    kind: kind.clone(),
                })
            }
            Type::ConstructorVariableApp {
                constructor,
                args,
                kind,
            } => {
                let args: Vec<CanonicalTypeExpr> = args
                    .iter()
                    .map(|arg| self.type_to_canonical_expr_for_equality(arg))
                    .collect::<Option<_>>()?;
                let constructor_kind = self
                    .type_parameter_kind(constructor)
                    .cloned()
                    .unwrap_or_else(|| Kind::n_ary(args.len()));
                Some(CanonicalTypeExpr::ConstructorVariableApp(Box::new(
                    ConstructorVariableApp::new(
                        ConstructorVariableRef::new(constructor.clone(), constructor_kind, None),
                        args,
                        kind.clone(),
                        None,
                    ),
                )))
            }
            Type::Associated {
                interface,
                base,
                name,
            } => {
                let (canonical_interface, canonical_name) = self
                    .canonical_associated_projection_for_equality(interface, name)
                    .unwrap_or_else(|| (interface.clone(), name.clone()));
                if let Type::Var(var) = base.as_ref() {
                    if canonical_interface.is_empty() {
                        return None;
                    }
                    let interface_id = self
                        .interface_identity_for_name(&canonical_interface)?
                        .clone();
                    let member = self
                        .associated_member_identity_for_interface_member(
                            &canonical_interface,
                            &canonical_name,
                        )?
                        .clone();
                    return Some(CanonicalTypeExpr::Projection {
                        interface: interface_id,
                        member,
                        args: vec![CanonicalTypeExpr::Var(format!("_t{}", var.0))],
                        kind: Kind::Type,
                        rigidity: ProjectionRigidity::Rigid,
                    });
                }
                let base = self.type_to_canonical_expr_for_equality(base)?;
                self.lower_associated_projection_to_canonical(&base, &canonical_name)
                    .ok()
                    .map(|projection| match projection {
                        CanonicalTypeExpr::Projection {
                            interface,
                            member,
                            args,
                            kind,
                            rigidity,
                        } if interface.name == canonical_interface => {
                            let canonical_interface_id = self
                                .interface_identity_for_name(&canonical_interface)
                                .cloned()
                                .unwrap_or(interface);
                            CanonicalTypeExpr::Projection {
                                interface: canonical_interface_id,
                                member,
                                args,
                                kind,
                                rigidity,
                            }
                        }
                        other => other,
                    })
            }
            Type::List(_)
            | Type::Record(_)
            | Type::Cap { .. }
            | Type::Fun(_, _, _)
            | Type::Fn(_, _)
            | Type::Instance { .. }
            | Type::InstanceAddr { .. }
            | Type::ControlLink { .. }
            | Type::Constructor { .. } => None,
        }
    }

    /// Register an interface declaration.
    pub fn register_interface(&mut self, def: &InterfaceDef) -> Result<(), TypeEnvError> {
        let interface_name = def.name.to_string();
        if self.interfaces.contains_key(&interface_name) {
            return Err(TypeEnvError::DuplicateInterface(
                interface_name,
                Span::default(),
            ));
        }
        let has_sealed_family = def
            .associated_types
            .iter()
            .any(|associated| matches!(associated.kind, AssociatedTypeKind::SealedFamily { .. }));
        let owner_module = if has_sealed_family {
            Some(self.current_module_identity.clone().ok_or_else(|| {
                TypeEnvError::AssociatedFamilyModuleOwnerViolation {
                    family: def
                        .associated_types
                        .iter()
                        .find(|associated| {
                            matches!(associated.kind, AssociatedTypeKind::SealedFamily { .. })
                        })
                        .map(|associated| associated.name.to_string())
                        .unwrap_or_else(|| "<unknown>".to_string()),
                    reason: "missing current module identity while registering sealed family declaration"
                        .to_string(),
                    span: def.span,
                }
            })?)
        } else {
            None
        };

        let interface_param_domains = def
            .type_params
            .iter()
            .map(|param| {
                self.optional_param_domain_constraint(param.domain.as_ref(), param.span)
                    .map(|domain_constraint| AssociatedFamilyInterfaceParamInfo {
                        name: param.name.to_string(),
                        domain_constraint,
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut seen_associated_names: HashMap<String, bool> = HashMap::new();
        for associated in &def.associated_types {
            let is_family = matches!(associated.kind, AssociatedTypeKind::SealedFamily { .. });
            if let Some(previous_was_family) =
                seen_associated_names.insert(associated.name.to_string(), is_family)
            {
                if previous_was_family || is_family {
                    return Err(TypeEnvError::DuplicateAssociatedFamilyHead {
                        interface: interface_name.clone(),
                        family: associated.name.to_string(),
                        span: associated.span,
                    });
                }
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "duplicate associated type '{}' in interface '{}'",
                        associated.name, interface_name
                    ),
                    associated.span,
                ));
            }
        }

        if owner_module.is_some() {
            for associated in &def.associated_types {
                let AssociatedTypeKind::SealedFamily {
                    result_domain,
                    decreases,
                    ..
                } = &associated.kind
                else {
                    continue;
                };
                let family_name = associated.name.to_string();
                self.associated_family_result_constraint_from_surface(
                    result_domain,
                    associated.span,
                )
                .map_err(|err| match err {
                    TypeEnvError::WrongAssociatedFamilyResultDomain { reason, span, .. } => {
                        TypeEnvError::WrongAssociatedFamilyResultDomain {
                            family: family_name.clone(),
                            reason,
                            span,
                        }
                    }
                    other => other,
                })?;
                if let Some(decreases) = decreases {
                    let Some(param) = interface_param_domains
                        .iter()
                        .find(|param| param.name == decreases.param.as_ref())
                    else {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "decreases parameter '{}' is not an interface parameter for associated family '{}::{}'",
                                decreases.param, interface_name, family_name
                            ),
                            decreases.span,
                        ));
                    };
                    if param.domain_constraint.is_none() {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "decreases parameter '{}' for associated family '{}::{}' must have a sealed-domain constraint",
                                decreases.param, interface_name, family_name
                            ),
                            decreases.span,
                        ));
                    }
                }
            }
        }

        let param_mapping: HashMap<String, TypeVar> = def
            .type_params
            .iter()
            .map(|param| (param.to_string(), TypeVar::fresh()))
            .collect();

        let ordered_param_names: Vec<String> =
            def.type_params.iter().map(ToString::to_string).collect();
        let type_param_kinds = interface_param_kinds(&def.type_params);
        let interface_type_params = def
            .type_params
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let evidence_constraints = self.validate_interface_evidence_constraints(
            &interface_name,
            &interface_type_params,
            &type_param_kinds,
            &def.evidence_constraints,
        )?;
        let associated_types = def
            .associated_types
            .iter()
            .map(|a| a.name.to_string())
            .collect::<Vec<_>>();
        let law_names = def
            .laws
            .iter()
            .map(|law| law.name.to_string())
            .collect::<Vec<_>>();

        // Make the interface's own arity visible while converting method
        // signatures. Existing interface syntax uses the interface name as the
        // nominal head in method parameters (for example `Pair<A, B>`), which
        // may coexist with a zero-arity ordinary carrier type named `Pair`.
        self.interfaces.insert(
            interface_name.clone(),
            InterfaceInfo {
                name: interface_name.clone(),
                visibility: core_visibility_from_surface(&def.visibility),
                type_params: interface_type_params.clone(),
                type_param_kinds: type_param_kinds.clone(),
                associated_types: associated_types.clone(),
                evidence_constraints: evidence_constraints.clone(),
                law_names: law_names.clone(),
                methods: HashMap::new(),
            },
        );

        let mut method_env = self.clone();
        for (name, kind) in interface_type_params.iter().zip(type_param_kinds.iter()) {
            method_env.register_type_parameter_kind(name, kind.clone())?;
        }

        let methods = match def
            .methods
            .iter()
            .map(|method| {
                method_env.convert_interface_method(
                    method,
                    &param_mapping,
                    &ordered_param_names,
                    &interface_name,
                )
            })
            .collect::<Result<HashMap<_, _>, _>>()
        {
            Ok(methods) => methods,
            Err(error) => {
                self.interfaces.remove(&interface_name);
                return Err(error);
            }
        };

        self.interfaces.insert(
            interface_name.clone(),
            InterfaceInfo {
                name: interface_name.clone(),
                visibility: core_visibility_from_surface(&def.visibility),
                type_params: interface_type_params.clone(),
                type_param_kinds: type_param_kinds.clone(),
                associated_types: associated_types.clone(),
                evidence_constraints,
                law_names,
                methods: methods.clone(),
            },
        );
        if let Some(current_module) = self.current_module_identity.clone() {
            let interface_id =
                self.ensure_local_interface_identity(&interface_name, &current_module);
            self.local_interface_arities
                .insert(interface_id.clone(), def.type_params.len());
            for associated in &def.associated_types {
                self.ensure_local_associated_member_identity(
                    &interface_name,
                    &interface_id,
                    associated.name.as_ref(),
                );
            }
        }
        if let Some(owner_module) = owner_module {
            let interface_id = self.ensure_local_interface_identity(&interface_name, &owner_module);
            self.local_interface_arities
                .insert(interface_id.clone(), def.type_params.len());
            for associated in &def.associated_types {
                let AssociatedTypeKind::SealedFamily {
                    result_domain,
                    decreases,
                    ..
                } = &associated.kind
                else {
                    continue;
                };
                let family_name = associated.name.to_string();
                let result_domain = self
                    .associated_family_result_constraint_from_surface(
                        result_domain,
                        associated.span,
                    )
                    .map_err(|err| match err {
                        TypeEnvError::WrongAssociatedFamilyResultDomain {
                            reason, span, ..
                        } => TypeEnvError::WrongAssociatedFamilyResultDomain {
                            family: family_name.clone(),
                            reason,
                            span,
                        },
                        other => other,
                    })?;
                if let Some(decreases) = decreases {
                    let Some(param) = interface_param_domains
                        .iter()
                        .find(|param| param.name == decreases.param.as_ref())
                    else {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "decreases parameter '{}' is not an interface parameter for associated family '{}::{}'",
                                decreases.param, interface_name, family_name
                            ),
                            decreases.span,
                        ));
                    };
                    if param.domain_constraint.is_none() {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "decreases parameter '{}' for associated family '{}::{}' must have a sealed-domain constraint",
                                decreases.param, interface_name, family_name
                            ),
                            decreases.span,
                        ));
                    }
                }
                let member = self.ensure_local_associated_member_identity(
                    &interface_name,
                    &interface_id,
                    &family_name,
                );
                let head = AssociatedFamilyHeadId {
                    interface: interface_id.clone(),
                    member,
                };
                if self.associated_family_declarations.contains_key(&head) {
                    return Err(TypeEnvError::DuplicateAssociatedFamilyHead {
                        interface: interface_name.clone(),
                        family: family_name,
                        span: associated.span,
                    });
                }
                let declaration = AssociatedFamilyDeclarationInfo {
                    defining_module: owner_module.clone(),
                    result_domain,
                    decreases: decreases
                        .as_ref()
                        .map(|decreases| decreases.param.to_string()),
                    interface_params: interface_param_domains.clone(),
                    head: head.clone(),
                };
                self.associated_family_name_index.insert(
                    (interface_name.clone(), associated.name.to_string()),
                    head.clone(),
                );
                self.associated_family_declarations
                    .insert(head, declaration);
            }
        }
        if let Some(interface_id) = self.interface_identity_for_name(&interface_name).cloned() {
            let imported = self
                .interface_identity_alias_is_imported
                .get(&interface_name)
                .copied()
                .unwrap_or(false);
            if !imported {
                self.local_interface_arities
                    .insert(interface_id, def.type_params.len());
            }
        }
        if interface_name == "Monad" {
            self.register_compiler_prelude_tower_monad_evidence()?;
        }
        Ok(())
    }

    /// Validate law propositions declared inside an interface.
    pub fn register_interface_laws(
        &mut self,
        interface: &InterfaceDef,
    ) -> Result<(), TypeEnvError> {
        let interface_name = interface.name.to_string();
        let interface_info = self
            .interfaces
            .get(&interface_name)
            .ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!("unknown interface '{interface_name}' while checking laws"),
                    interface.span,
                )
            })?
            .clone();

        let param_mapping = interface
            .type_params
            .iter()
            .map(|param| (param.to_string(), TypeVar::fresh()))
            .collect::<HashMap<_, _>>();

        for law in &interface.laws {
            let mut law_env = self.clone();
            for (method_name, method) in &interface_info.methods {
                law_env.bind_variable(
                    method_name,
                    Type::Fn(method.params.clone(), Box::new(method.return_type.clone())),
                );
            }
            law_env.bind_law_params(law, &param_mapping)?;
            law_env.check_law_proposition(law)?;
        }

        Ok(())
    }

    /// Validate laws declared at module scope.
    pub fn register_module_laws(&mut self, definitions: &[Definition]) -> Result<(), TypeEnvError> {
        let param_mapping = HashMap::new();
        for law in definitions
            .iter()
            .filter_map(|definition| match definition {
                Definition::Law(law) => Some(law),
                _ => None,
            })
        {
            let mut law_env = self.clone();
            law_env.bind_law_params(law, &param_mapping)?;
            law_env.check_law_proposition(law)?;
        }

        Ok(())
    }

    /// Validate proofs declared at module scope against module-scope laws.
    pub fn register_module_proofs(
        &mut self,
        definitions: &[Definition],
    ) -> Result<(), TypeEnvError> {
        self.register_module_proofs_with_fuel(definitions, DEFAULT_PROOF_FUEL)
    }

    /// Validate module-scope proofs with explicit proof-checking fuel.
    pub fn register_module_proofs_with_fuel(
        &mut self,
        definitions: &[Definition],
        proof_fuel: usize,
    ) -> Result<(), TypeEnvError> {
        let module_law_names = definitions
            .iter()
            .filter_map(|definition| match definition {
                Definition::Law(law) => Some(law.name.to_string()),
                _ => None,
            })
            .collect::<HashSet<_>>();

        let module_proofs = definitions
            .iter()
            .filter_map(|definition| match definition {
                Definition::Proof(proof) => Some(proof.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        self.check_proof_cycles(&module_proofs)?;

        for proof in &module_proofs {
            self.check_proof_matches_law(proof, &module_law_names, "module")?;
            self.check_proof_totality_with_fuel(proof, proof_fuel)?;
        }

        Ok(())
    }

    /// Validate proofs declared in an impl block against laws of the implemented interface.
    pub fn register_impl_proofs(&self, implementation: &ImplDef) -> Result<(), TypeEnvError> {
        self.register_impl_proofs_with_fuel(implementation, DEFAULT_PROOF_FUEL)
    }

    /// Validate impl-scoped proofs with explicit proof-checking fuel.
    pub fn register_impl_proofs_with_fuel(
        &self,
        implementation: &ImplDef,
        proof_fuel: usize,
    ) -> Result<(), TypeEnvError> {
        if implementation.proofs.is_empty() {
            return Ok(());
        }

        let interface_name = implementation.interface.to_string();
        let interface = self.interfaces.get(&interface_name).ok_or_else(|| {
            TypeEnvError::InvalidDefinition(
                format!(
                    "proof declarations for impl {} cannot be checked because interface '{}' is unknown",
                    implementation.interface, interface_name
                ),
                implementation.span,
            )
        })?;

        let interface_law_names = interface.law_names.iter().cloned().collect::<HashSet<_>>();
        let scope = format!("interface {interface_name}");
        self.check_proof_cycles(&implementation.proofs)?;
        for proof in &implementation.proofs {
            self.check_proof_matches_law(proof, &interface_law_names, &scope)?;
            self.check_proof_totality_with_fuel(proof, proof_fuel)?;
        }

        Ok(())
    }

    /// Stage-3 proof body totality validation hook.
    ///
    /// Phase 136 Stage 3 currently tracks a conservative proof-body traversal
    /// fuel budget, rejects non-exhaustive AST-level proof matches, and rejects
    /// circular proof dependencies in registration. It still leaves theorem
    /// proving and full proof-term typechecking to follow-on tasks.
    pub fn check_proof_totality(
        &self,
        proof: &ProofDef,
    ) -> Result<ProofTotalityResult, TypeEnvError> {
        self.check_proof_totality_with_fuel(proof, DEFAULT_PROOF_FUEL)
    }

    /// Check proof totality with an explicit traversal fuel budget.
    ///
    /// Exhausting fuel returns an `Untested` proof result instead of a type
    /// error, preserving Stage-3's distinction between inconclusive proof
    /// checking and rejected programs.
    pub fn check_proof_totality_with_fuel(
        &self,
        proof: &ProofDef,
        fuel: usize,
    ) -> Result<ProofTotalityResult, TypeEnvError> {
        let mut proof_env = self.clone();
        for param in &proof.params {
            let ty = lower_proof_param_type(&param.ty, &proof_env);
            proof_env.bind_variable(param.name.as_ref(), ty);
        }
        let mut checker = ProofFuelChecker::new(fuel, proof_env);
        match &proof.body {
            ProofBody::ByDefinition
            | ProofBody::ByTest { .. }
            | ProofBody::ByTestProperty { .. }
            | ProofBody::ByTestSmallWorld => {}
            ProofBody::Expr(expr) => checker.visit_expr(expr),
        }
        checker.finish()
    }

    /// Erase a checked proof to its proposition boundary for Stage-3 proof
    /// irrelevance.
    ///
    /// This API deliberately reuses the default proof totality checker before
    /// constructing the erased carrier. The returned value keeps only the
    /// proved proposition, so proof names, bodies, and witness identities are
    /// definitionally irrelevant within this local/static Stage-3 slice.
    pub fn erase_proof_for_proposition(
        &self,
        proposition: &TypeProposition,
        proof: &ProofDef,
    ) -> Result<ErasedProof, TypeEnvError> {
        self.erase_proof_for_proposition_with_fuel(proposition, proof, DEFAULT_PROOF_FUEL)
    }

    /// Erase a checked proof using an explicit proof-totality fuel budget.
    ///
    /// This mirrors `check_proof_totality_with_fuel` for tests and callers that
    /// need deterministic coverage of inconclusive proof checks. Inconclusive
    /// proof totality is not erased, because proof irrelevance only applies to
    /// checked proofs in this Stage-3 slice.
    pub fn erase_proof_for_proposition_with_fuel(
        &self,
        proposition: &TypeProposition,
        proof: &ProofDef,
        fuel: usize,
    ) -> Result<ErasedProof, TypeEnvError> {
        let totality = self.check_proof_totality_with_fuel(proof, fuel)?;
        if !matches!(totality.status, ProofTotalityStatus::Checked) {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "proof {} could not be erased because totality checking was inconclusive",
                    proof.name
                ),
                proof.span,
            ));
        }

        Ok(ErasedProof {
            proposition: proposition.clone(),
        })
    }

    /// Compare two proofs of the same proposition under Stage-3 proof
    /// irrelevance.
    ///
    /// Both proofs are independently checked and erased. Equality then compares
    /// only the retained proposition boundary, ensuring proofs of the same
    /// proposition collapse while proofs of different propositions do not.
    pub fn proofs_definitionally_equal_for_proposition(
        &self,
        proposition: &TypeProposition,
        left: &ProofDef,
        right: &ProofDef,
    ) -> Result<bool, TypeEnvError> {
        let left = self.erase_proof_for_proposition(proposition, left)?;
        let right = self.erase_proof_for_proposition(proposition, right)?;
        Ok(left == right)
    }

    /// Reject circular dependencies among the supplied proof definitions.
    ///
    /// This Stage-3 direct checker builds a local proof call graph from surface
    /// proof expression bodies and only records `Expr::Call` edges whose callee
    /// name is another proof in the supplied slice. Calls to ordinary functions
    /// or proofs outside the checked slice are ignored by this local API.
    pub fn check_proof_cycles(&self, proofs: &[ProofDef]) -> Result<(), TypeEnvError> {
        let mut proof_names = HashSet::<String>::new();
        for proof in proofs {
            let proof_name = proof.name.to_string();
            if !proof_names.insert(proof_name.clone()) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!("duplicate proof declaration: {proof_name}"),
                    proof.span,
                ));
            }
        }

        let mut graph = BTreeMap::<String, Vec<String>>::new();
        let mut spans = HashMap::<String, Span>::new();

        for proof in proofs {
            let proof_name = proof.name.to_string();
            spans.insert(proof_name.clone(), proof.span);
            let mut collector = ProofCallCollector::new(&proof_names);
            if let ProofBody::Expr(expr) = &proof.body {
                collector.visit_expr(expr);
            }
            let mut callees = collector.into_calls();
            callees.sort();
            graph.insert(proof_name, callees);
        }

        let mut visiting = HashSet::<String>::new();
        let mut visited = HashSet::<String>::new();
        let mut stack = Vec::<String>::new();
        for proof_name in graph.keys() {
            if visited.contains(proof_name) {
                continue;
            }
            if let Some(cycle) =
                detect_proof_cycle(proof_name, &graph, &mut visiting, &mut visited, &mut stack)
            {
                let span = cycle
                    .first()
                    .and_then(|name| spans.get(name))
                    .copied()
                    .unwrap_or_default();
                return Err(TypeEnvError::InvalidDefinition(
                    format!("circular proof dependency: {}", cycle.join(" -> ")),
                    span,
                ));
            }
        }

        Ok(())
    }

    pub(super) fn check_proof_matches_law(
        &self,
        proof: &ProofDef,
        law_names: &HashSet<String>,
        scope: &str,
    ) -> Result<(), TypeEnvError> {
        if law_names.contains(proof.name.as_ref()) {
            return Ok(());
        }

        Err(TypeEnvError::InvalidDefinition(
            format!(
                "proof {} does not match any declared law in {scope} scope",
                proof.name
            ),
            proof.span,
        ))
    }

    pub(super) fn bind_law_params(
        &mut self,
        law: &LawDef,
        param_mapping: &HashMap<String, TypeVar>,
    ) -> Result<(), TypeEnvError> {
        for param in &law.params {
            let ty = surface_type_to_type(&param.ty, param_mapping, self).map_err(|error| {
                TypeEnvError::InvalidDefinition(
                    format!(
                        "law {} parameter '{}' type error: {error}",
                        law.name, param.name
                    ),
                    law.span,
                )
            })?;
            self.bind_variable(param.name.as_ref(), ty);
        }

        Ok(())
    }

    pub(super) fn check_law_proposition(&self, law: &LawDef) -> Result<(), TypeEnvError> {
        if let Err(errors) = crate::purity::check_purity(self, &law.proposition, false) {
            let diagnostics = errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ");
            return Err(TypeEnvError::InvalidDefinition(
                format!("law {} proposition is not pure: {diagnostics}", law.name),
                law.span,
            ));
        }

        let result = crate::check_expr::check_expr(self, &law.proposition);
        if result.is_ok() {
            return Ok(());
        }

        let diagnostics = result
            .errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ");
        Err(TypeEnvError::InvalidDefinition(
            format!(
                "law {} proposition failed to typecheck: {diagnostics}",
                law.name
            ),
            law.span,
        ))
    }

    pub(super) fn register_compiler_prelude_tower_monad_evidence(
        &mut self,
    ) -> Result<(), TypeEnvError> {
        let interface =
            self.interfaces.get("Monad").cloned().ok_or_else(|| {
                TypeEnvError::MissingInterface("Monad".to_string(), Span::default())
            })?;
        let expected_methods = ["unit", "bind"];
        if !expected_methods
            .iter()
            .all(|method| interface.methods.contains_key(*method))
        {
            return Ok(());
        }

        self.register_compiler_prelude_tower_evidence("Functor")?;
        self.register_compiler_prelude_tower_evidence("Applicative")?;
        self.register_compiler_prelude_tower_evidence("Monad")
    }

    pub(super) fn register_compiler_prelude_tower_evidence(
        &mut self,
        interface_name: &str,
    ) -> Result<(), TypeEnvError> {
        let Some(interface) = self.interfaces.get(interface_name).cloned() else {
            return Ok(());
        };

        for carrier in ["Act", "Proc", "Workflow"] {
            if !self.has_type(carrier) {
                continue;
            }
            let surface_args = [SurfaceType::Name(carrier.into())];
            let head_args = self.lower_interface_evidence_args(
                interface_name,
                &interface,
                &surface_args,
                &HashMap::new(),
            )?;
            if self
                .validate_concrete_impl_required_evidence(&interface, &head_args, Span::default())
                .is_err()
            {
                continue;
            }
            if self.impls.iter().any(|scheme| {
                scheme.interface == interface_name
                    && interface_evidence_args_match(&scheme.head_args, &head_args, false)
            }) {
                continue;
            }
            let lowered_type_args: Vec<Type> = head_args
                .iter()
                .map(interface_evidence_arg_as_legacy_type)
                .collect();
            self.impls.push(ImplScheme {
                interface: interface_name.to_string(),
                type_params: Vec::new(),
                head: Type::Constructor {
                    name: QualifiedName::root(interface_name),
                    args: lowered_type_args,
                    kind: Kind::Type,
                },
                head_args,
                where_bounds: Vec::new(),
                associated_type_bindings: HashMap::new(),
                methods: Vec::new(),
            });
        }

        Ok(())
    }

    pub(super) fn validate_associated_family_scheme_totality(
        &self,
        family: &str,
        declaration: &AssociatedFamilyDeclarationInfo,
        scheme: &AssociatedFamilyScheme,
        require_coverage: bool,
    ) -> Result<(), TypeEnvError> {
        let scheme_param_names = scheme
            .params
            .iter()
            .map(|param| param.name.as_str())
            .collect::<HashSet<_>>();
        let recursive = scheme.equations.iter().any(|equation| {
            Self::associated_family_result_contains_head_with_scheme_param_arg(
                &equation.result,
                &scheme.head,
                &scheme_param_names,
            )
        });
        if !recursive {
            if require_coverage && declaration.decreases.is_some() {
                self.validate_associated_family_pattern_coverage(family, scheme)?;
            }
            return Ok(());
        }

        if scheme.equations.iter().any(|equation| {
            Self::associated_family_result_contains_other_family(&equation.result, &scheme.head)
        }) {
            return Err(TypeEnvError::InvalidDefinition(
                format!("mutual recursion in associated family '{family}' is unsupported"),
                anchor_span(&scheme.source_anchor),
            ));
        }

        let Some(decreases) = declaration.decreases.as_deref() else {
            return Err(TypeEnvError::InvalidDefinition(
                format!("missing decreases clause for recursive associated family '{family}'"),
                anchor_span(&scheme.source_anchor),
            ));
        };

        let Some(decreasing_index) = scheme
            .params
            .iter()
            .position(|param| param.name == decreases)
        else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "unknown decreases parameter '{decreases}' in associated family '{family}'"
                ),
                anchor_span(&scheme.source_anchor),
            ));
        };
        let Some(decreasing_domain) = scheme.params[decreasing_index].domain_constraint.as_ref()
        else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "invalid decreases parameter '{decreases}' in associated family '{family}': parameter is not a sealed domain"
                ),
                anchor_span(&scheme.source_anchor),
            ));
        };
        if !self.domain_has_structural_subcomponent_metadata(decreasing_domain)? {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "invalid decreases parameter '{decreases}' in associated family '{family}': sealed domain has no structural subcomponent metadata"
                ),
                anchor_span(&scheme.source_anchor),
            ));
        }

        if require_coverage {
            self.validate_associated_family_pattern_coverage(family, scheme)?;
        }

        for equation in &scheme.equations {
            let allowed = equation
                .interface_arg_patterns
                .get(decreasing_index)
                .map(|pattern| self.direct_associated_family_structural_subcomponent_vars(pattern))
                .transpose()?
                .unwrap_or_default();
            self.validate_recursive_associated_family_calls(
                family,
                &scheme.head,
                decreasing_index,
                &allowed,
                &equation.result,
                anchor_span(&equation.source_anchor),
            )?;
        }
        Ok(())
    }
}
