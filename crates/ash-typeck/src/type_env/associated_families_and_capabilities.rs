use super::*;

impl TypeEnv {
    pub(super) fn validate_associated_family_pattern_coverage(
        &self,
        family: &str,
        scheme: &AssociatedFamilyScheme,
    ) -> Result<(), TypeEnvError> {
        let params = scheme
            .params
            .iter()
            .map(|param| TypeFunctionParam {
                name: param.name.clone(),
                ty: param.ty.clone(),
                kind: param.kind.clone(),
                domain_constraint: param.domain_constraint.clone(),
                source_anchor: param.source_anchor.clone(),
            })
            .collect::<Vec<_>>();
        let pseudo_head = TypeComputationHeadId::new(scheme.head.interface.module.clone(), family);
        let equations = scheme
            .equations
            .iter()
            .map(|equation| {
                let patterns = equation
                    .interface_arg_patterns
                    .iter()
                    .map(Self::associated_family_pattern_to_type_function_pattern)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeFunctionEquation {
                    head: pseudo_head.clone(),
                    ordinal: equation.ordinal,
                    patterns,
                    result: TypeFunctionResultExpr::Var {
                        name: "__task865_result".to_string(),
                        kind: Kind::Type,
                        constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
                        source_anchor: equation.source_anchor.clone(),
                    },
                    source_anchor: equation.source_anchor.clone(),
                    case_head_anchor: equation.case_head_anchor.clone(),
                })
            })
            .collect::<Result<Vec<_>, TypeEnvError>>()?;
        self.validate_type_function_pattern_coverage(
            family,
            &params,
            &equations,
            anchor_span(&scheme.source_anchor),
        )
    }

    pub(super) fn associated_family_pattern_to_type_function_pattern(
        pattern: &AssociatedFamilyPattern,
    ) -> Result<TypeFunctionPattern, TypeEnvError> {
        match pattern {
            AssociatedFamilyPattern::DomainConstructor {
                constructor,
                domain,
                fields,
                constraint,
                source_anchor,
            } => {
                let fields = fields
                    .iter()
                    .map(Self::associated_family_pattern_to_type_function_pattern)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeFunctionPattern::DomainConstructor {
                    constructor: constructor.clone(),
                    domain: domain.clone(),
                    fields,
                    constraint: associated_family_constraint_to_type_function_pattern(constraint),
                    source_anchor: source_anchor.clone(),
                })
            }
            AssociatedFamilyPattern::Var {
                name,
                constraint,
                source_anchor,
            } => Ok(TypeFunctionPattern::Var {
                name: name.clone(),
                constraint: associated_family_constraint_to_type_function_pattern(constraint),
                source_anchor: source_anchor.clone(),
            }),
            AssociatedFamilyPattern::Wildcard {
                constraint,
                source_anchor,
            } => Ok(TypeFunctionPattern::Wildcard {
                constraint: associated_family_constraint_to_type_function_pattern(constraint),
                source_anchor: source_anchor.clone(),
            }),
            AssociatedFamilyPattern::NominalApp {
                visible_name,
                source_anchor,
                ..
            }
            | AssociatedFamilyPattern::Primitive {
                name: visible_name,
                source_anchor,
                ..
            } => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated family coverage pattern '{visible_name}' is not a sealed-domain pattern"
                ),
                anchor_span(source_anchor),
            )),
        }
    }

    pub(super) fn direct_associated_family_structural_subcomponent_vars(
        &self,
        pattern: &AssociatedFamilyPattern,
    ) -> Result<HashSet<String>, TypeEnvError> {
        let AssociatedFamilyPattern::DomainConstructor {
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
                    "unknown sealed domain '{}' in associated-family recursion matrix",
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
            if let AssociatedFamilyPattern::Var { name, .. } = field_pattern {
                vars.insert(name.clone());
            }
        }
        Ok(vars)
    }

    pub(super) fn validate_recursive_associated_family_calls(
        &self,
        family: &str,
        self_head: &AssociatedFamilyHeadId,
        decreasing_index: usize,
        allowed_subcomponents: &HashSet<String>,
        expr: &AssociatedFamilyResultExpr,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match expr {
            AssociatedFamilyResultExpr::Primitive { .. }
            | AssociatedFamilyResultExpr::Var { .. } => Ok(()),
            AssociatedFamilyResultExpr::NominalApp { args, .. }
            | AssociatedFamilyResultExpr::DomainConstructorApp { args, .. }
            | AssociatedFamilyResultExpr::Projection { args, .. }
            | AssociatedFamilyResultExpr::ComputationHeadApp { args, .. } => {
                for arg in args {
                    self.validate_recursive_associated_family_calls(
                        family,
                        self_head,
                        decreasing_index,
                        allowed_subcomponents,
                        arg,
                        span,
                    )?;
                }
                Ok(())
            }
            AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                head,
                interface_args,
                ..
            } => {
                for arg in interface_args {
                    self.validate_recursive_associated_family_calls(
                        family,
                        self_head,
                        decreasing_index,
                        allowed_subcomponents,
                        arg,
                        span,
                    )?;
                }
                if head == self_head {
                    let Some(decreasing_arg) = interface_args.get(decreasing_index) else {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "non-decreasing recursive call in associated family '{family}': missing decreasing argument"
                            ),
                            span,
                        ));
                    };
                    match decreasing_arg {
                        AssociatedFamilyResultExpr::Var { name, .. }
                            if allowed_subcomponents.contains(name) =>
                        {
                            Ok(())
                        }
                        _ => Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "non-decreasing recursive call in associated family '{family}': decreasing argument must be a direct structural subcomponent"
                            ),
                            span,
                        )),
                    }
                } else {
                    Err(TypeEnvError::InvalidDefinition(
                        format!("mutual recursion in associated family '{family}' is unsupported"),
                        span,
                    ))
                }
            }
        }
    }

    pub(super) fn associated_family_result_contains_head_with_scheme_param_arg(
        expr: &AssociatedFamilyResultExpr,
        needle: &AssociatedFamilyHeadId,
        scheme_param_names: &HashSet<&str>,
    ) -> bool {
        match expr {
            AssociatedFamilyResultExpr::Primitive { .. } | AssociatedFamilyResultExpr::Var { .. } => false,
            AssociatedFamilyResultExpr::NominalApp { args, .. }
            | AssociatedFamilyResultExpr::DomainConstructorApp { args, .. }
            | AssociatedFamilyResultExpr::Projection { args, .. }
            | AssociatedFamilyResultExpr::ComputationHeadApp { args, .. } => args.iter().any(|arg| {
                Self::associated_family_result_contains_head_with_scheme_param_arg(
                    arg,
                    needle,
                    scheme_param_names,
                )
            }),
            AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                head,
                interface_args,
                ..
            } => {
                (head == needle
                    && interface_args.iter().any(|arg| {
                        matches!(arg, AssociatedFamilyResultExpr::Var { name, .. } if scheme_param_names.contains(name.as_str()))
                    }))
                    || interface_args.iter().any(|arg| {
                        Self::associated_family_result_contains_head_with_scheme_param_arg(
                            arg,
                            needle,
                            scheme_param_names,
                        )
                    })
            }
        }
    }

    pub(super) fn associated_family_result_contains_other_family(
        expr: &AssociatedFamilyResultExpr,
        self_head: &AssociatedFamilyHeadId,
    ) -> bool {
        match expr {
            AssociatedFamilyResultExpr::Primitive { .. }
            | AssociatedFamilyResultExpr::Var { .. } => false,
            AssociatedFamilyResultExpr::NominalApp { args, .. }
            | AssociatedFamilyResultExpr::DomainConstructorApp { args, .. }
            | AssociatedFamilyResultExpr::Projection { args, .. }
            | AssociatedFamilyResultExpr::ComputationHeadApp { args, .. } => args
                .iter()
                .any(|arg| Self::associated_family_result_contains_other_family(arg, self_head)),
            AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                head,
                interface_args,
                ..
            } => {
                head != self_head
                    || interface_args.iter().any(|arg| {
                        Self::associated_family_result_contains_other_family(arg, self_head)
                    })
            }
        }
    }

    /// Register a coherence-checked associated-family scheme for a sealed family head.
    pub fn register_associated_family_scheme(
        &mut self,
        scheme: AssociatedFamilyScheme,
        defining_module: ModuleIdentity,
    ) -> Result<(), TypeEnvError> {
        self.register_associated_family_scheme_with_totality(scheme, defining_module, true)
    }

    pub(super) fn register_associated_family_scheme_with_totality(
        &mut self,
        scheme: AssociatedFamilyScheme,
        defining_module: ModuleIdentity,
        require_totality: bool,
    ) -> Result<(), TypeEnvError> {
        let declaration = self
            .associated_family_declarations
            .get(&scheme.head)
            .cloned()
            .ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    "associated family scheme references an undeclared sealed family head"
                        .to_string(),
                    Span::default(),
                )
            })?;
        let family = declaration.head.member.name.to_string();

        if declaration.defining_module != defining_module {
            return Err(TypeEnvError::UnauthorizedAssociatedFamilyExtension {
                family,
                owner_module: declaration.defining_module,
                attempted_module: defining_module,
                span: anchor_span(&scheme.source_anchor),
            });
        }

        if scheme.result_kind != Kind::Type {
            return Err(TypeEnvError::WrongAssociatedFamilyResultKind {
                family,
                expected: format!("{:?}", Kind::Type),
                found: format!("{:?}", scheme.result_kind),
                span: anchor_span(&scheme.source_anchor),
            });
        }

        if scheme.equations.is_empty() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated family scheme for '{family}' must contain at least one equation"
                ),
                anchor_span(&scheme.source_anchor),
            ));
        }

        if !matches_associated_family_result_constraint(
            &scheme.result_domain,
            &declaration.result_domain,
        ) {
            return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: family.clone(),
                reason: "scheme result-domain annotation does not match the associated family declaration"
                    .to_string(),
                span: anchor_span(&scheme.source_anchor),
            });
        }

        for equation in &scheme.equations {
            if equation.head != scheme.head {
                return Err(TypeEnvError::InvalidDefinition(
                    "associated family scheme equation head does not match scheme head".to_string(),
                    anchor_span(&equation.source_anchor),
                ));
            }
            if equation.interface_arg_patterns.len() != declaration.interface_params.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated family scheme for '{family}' expects {} interface argument patterns, found {}",
                        declaration.interface_params.len(),
                        equation.interface_arg_patterns.len()
                    ),
                    anchor_span(&equation.source_anchor),
                ));
            }
        }

        self.validate_associated_family_scheme_totality(
            &family,
            &declaration,
            &scheme,
            require_totality,
        )?;

        for equation in &scheme.equations {
            if !Self::associated_family_expr_conforms_to_constraint(
                &equation.result,
                &declaration.result_domain,
            ) {
                return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                    family: family.clone(),
                    reason: format!(
                        "RHS does not conform to associated family result constraint {}",
                        associated_family_result_constraint_label(&declaration.result_domain)
                    ),
                    span: anchor_span(&equation.source_anchor),
                });
            }
        }

        if let Some(existing_schemes) = self.associated_family_schemes.get(&scheme.head) {
            for existing in existing_schemes {
                for existing_equation in &existing.scheme.equations {
                    for new_equation in &scheme.equations {
                        if Self::associated_family_pattern_spines_overlap(
                            &existing_equation.interface_arg_patterns,
                            &new_equation.interface_arg_patterns,
                        ) {
                            return Err(TypeEnvError::OverlappingAssociatedFamilyScheme {
                                family: family.clone(),
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
                defining_module,
                scheme,
            });
        Ok(())
    }

    pub(super) fn convert_capability_operation(
        &self,
        operation: &CapabilityOperationSig,
    ) -> Result<(String, CapabilityOperationInfo), TypeEnvError> {
        let param_names = operation
            .params
            .iter()
            .map(|param| param.name.to_string())
            .collect();
        let param_mapping = HashMap::new();
        let params = operation
            .params
            .iter()
            .map(|param| surface_type_to_type(&param.ty, &param_mapping, self))
            .collect::<Result<Vec<_>, _>>()?;
        let return_type = surface_type_to_type(&operation.return_type, &param_mapping, self)?;

        Ok((
            operation.name.to_string(),
            CapabilityOperationInfo {
                mode: operation.mode,
                param_names,
                params,
                return_type,
            },
        ))
    }

    /// Register a resource type declaration.
    pub fn register_resource_type(&mut self, def: &ResourceTypeDef) -> Result<(), TypeEnvError> {
        let resource_name = def.name.to_string();
        if self.resource_types.contains_key(&resource_name) {
            return Err(TypeEnvError::InvalidDefinition(
                format!("resource type '{resource_name}' is already defined"),
                def.span,
            ));
        }

        let mut field_names = HashSet::with_capacity(def.fields.len());
        for field in &def.fields {
            let field_name = field.name.to_string();
            if !field_names.insert(field_name.clone()) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "resource type '{resource_name}' defines duplicate field '{field_name}'"
                    ),
                    field.span,
                ));
            }
        }

        let param_mapping = HashMap::new();
        let fields = def
            .fields
            .iter()
            .map(|field| {
                surface_type_to_type(&field.ty, &param_mapping, self)
                    .map(|ty| (field.name.to_string(), ty))
                    .map_err(|error| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "resource type '{resource_name}' field '{}' has invalid ordinary type: {error}",
                                field.name
                            ),
                            field.span,
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        self.resource_types.insert(
            resource_name.clone(),
            ResourceTypeInfo {
                name: resource_name,
                fields,
            },
        );
        Ok(())
    }

    /// Check if a resource type is registered.
    pub fn has_resource_type(&self, name: &str) -> bool {
        self.resource_types.contains_key(name)
    }

    /// Look up a registered resource type.
    pub fn lookup_resource_type(&self, name: &str) -> Option<&ResourceTypeInfo> {
        self.resource_types.get(name)
    }

    /// Register a capability interface declaration.
    pub fn register_capability_interface(
        &mut self,
        def: &CapabilityInterfaceDef,
    ) -> Result<(), TypeEnvError> {
        let interface_name = def.name.to_string();
        if self.capability_interfaces.contains_key(&interface_name) {
            return Err(TypeEnvError::InvalidDefinition(
                format!("capability interface '{interface_name}' is already defined"),
                def.span,
            ));
        }

        let mut operations = HashMap::with_capacity(def.operations.len());
        let mut operation_names = HashSet::with_capacity(def.operations.len());
        for operation in &def.operations {
            let operation_name = operation.name.to_string();
            if !operation_names.insert(operation_name.clone()) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "capability interface '{interface_name}' defines duplicate operation '{operation_name}'"
                    ),
                    operation.span,
                ));
            }
        }

        for operation in &def.operations {
            let (operation_name, operation_info) = self.convert_capability_operation(operation)?;
            operations.insert(operation_name, operation_info);
        }

        self.capability_interfaces.insert(
            interface_name.clone(),
            CapabilityInterfaceInfo {
                name: interface_name,
                operations,
            },
        );

        Ok(())
    }

    /// True if this environment is currently type-checking a capability implementation body.
    #[must_use]
    pub fn is_capability_implementation_body(&self) -> bool {
        self.capability_implementation_body
    }

    /// Register a capability implementation recipe and validate conformance to its interface.
    pub fn register_capability_implementation(
        &mut self,
        def: &CapabilityImplementationDef,
    ) -> Result<(), TypeEnvError> {
        let implementation_name = def.name.to_string();
        if self
            .capability_implementations
            .contains_key(&implementation_name)
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!("capability implementation '{implementation_name}' is already defined"),
                def.span,
            ));
        }

        let interface_name = def.interface.to_string();
        let interface = self
            .capability_interfaces
            .get(&interface_name)
            .cloned()
            .ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation '{implementation_name}' targets unknown capability interface '{interface_name}'"
                    ),
                    def.span,
                )
            })?;

        let mut operation_names = HashSet::with_capacity(def.operations.len());
        for operation in &def.operations {
            let operation_name = operation.name.to_string();
            if !operation_names.insert(operation_name.clone()) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation '{implementation_name}' defines duplicate operation '{operation_name}'"
                    ),
                    operation.span,
                ));
            }
        }

        for operation_name in interface.operations.keys() {
            if !operation_names.contains(operation_name) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation '{implementation_name}' is missing required operation '{operation_name}' for interface '{interface_name}'"
                    ),
                    def.span,
                ));
            }
        }

        for operation_name in &operation_names {
            if !interface.operations.contains_key(operation_name) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation '{implementation_name}' defines extra operation '{operation_name}' not present in interface '{interface_name}'"
                    ),
                    def.span,
                ));
            }
        }

        let dependencies = def
            .dependencies
            .iter()
            .map(|dependency| self.convert_capability_implementation_dependency(dependency))
            .collect::<Result<Vec<_>, _>>()?;
        let mut dependency_names = HashSet::with_capacity(dependencies.len());
        for dependency in &dependencies {
            if !dependency_names.insert(dependency.name.clone()) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation '{implementation_name}' defines duplicate dependency '{}'",
                        dependency.name
                    ),
                    def.span,
                ));
            }
        }

        let mut operations = HashMap::with_capacity(def.operations.len());
        for operation in &def.operations {
            let operation_name = operation.name.to_string();
            let expected = interface.operations.get(&operation_name).ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation '{implementation_name}' defines extra operation '{operation_name}' not present in interface '{interface_name}'"
                    ),
                    operation.span,
                )
            })?;
            let operation_info = self.convert_capability_implementation_operation(operation)?;

            if operation_info.mode != expected.mode {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation operation '{implementation_name}::{operation_name}' mode mismatch: expected {:?}, found {:?}",
                        expected.mode, operation_info.mode
                    ),
                    operation.span,
                ));
            }

            if operation_info.params.len() != expected.params.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation operation '{implementation_name}::{operation_name}' arity mismatch: expected {} parameters, found {}",
                        expected.params.len(),
                        operation_info.params.len()
                    ),
                    operation.span,
                ));
            }

            for (index, (expected_param, actual_param)) in expected
                .params
                .iter()
                .zip(operation_info.params.iter())
                .enumerate()
            {
                if !self.types_equivalent_for_equality(expected_param, actual_param) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "capability implementation operation '{implementation_name}::{operation_name}' parameter {index} type mismatch: expected {expected_param}, found {actual_param}"
                        ),
                        operation.span,
                    ));
                }
            }

            if !self
                .types_equivalent_for_equality(&operation_info.return_type, &expected.return_type)
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation operation '{implementation_name}::{operation_name}' return type mismatch: expected {}, found {}",
                        expected.return_type, operation_info.return_type
                    ),
                    operation.span,
                ));
            }

            for param_name in &operation_info.param_names {
                if dependency_names.contains(param_name) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "capability implementation operation '{implementation_name}::{operation_name}' parameter '{param_name}' collides with a declared dependency name"
                        ),
                        operation.span,
                    ));
                }
            }

            self.validate_capability_implementation_operation_body(
                &implementation_name,
                operation,
                &operation_info,
                &dependencies,
            )?;

            operations.insert(operation_name, operation_info);
        }

        let authority_provenance = classify_authority_provenance(&dependencies);
        let authority_sources = implementation_authority_sources(&dependencies);

        self.capability_implementations.insert(
            implementation_name.clone(),
            CapabilityImplementationInfo {
                name: implementation_name,
                interface: interface_name,
                dependencies,
                operations,
                authority_provenance,
                authority_sources,
            },
        );

        Ok(())
    }

    pub(super) fn convert_capability_implementation_dependency(
        &self,
        dependency: &CapabilityImplementationDependency,
    ) -> Result<CapabilityImplementationDependencyInfo, TypeEnvError> {
        let name = dependency.name.to_string();
        let target_name = surface_type_name(&dependency.ty).ok_or_else(|| {
            TypeEnvError::InvalidDefinition(
                format!(
                    "{:?} dependency '{name}' must name a single target type or interface",
                    dependency.kind
                ),
                dependency.span,
            )
        })?;

        match dependency.kind {
            CapabilityImplementationDependencyKind::Resource => {
                if !self.has_resource_type(&target_name) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "resource dependency '{name}' references unknown resource type '{target_name}'"
                        ),
                        dependency.span,
                    ));
                }
                Ok(CapabilityImplementationDependencyInfo {
                    kind: dependency.kind,
                    name,
                    ty: Type::Constructor {
                        name: QualifiedName::root(target_name.clone()),
                        args: vec![],
                        kind: Kind::Type,
                    },
                    target_name: Some(target_name),
                })
            }
            CapabilityImplementationDependencyKind::Capability => {
                if !self.has_capability_interface(&target_name) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "capability dependency '{name}' references unknown capability interface '{target_name}'"
                        ),
                        dependency.span,
                    ));
                }
                Ok(CapabilityImplementationDependencyInfo {
                    kind: dependency.kind,
                    name,
                    ty: Type::Cap {
                        name: Box::from(target_name.as_str()),
                        effect: ash_core::Effect::Operational,
                    },
                    target_name: Some(target_name),
                })
            }
            CapabilityImplementationDependencyKind::Config => {
                let param_mapping = HashMap::new();
                let ty = surface_type_to_type(&dependency.ty, &param_mapping, self)?;
                Ok(CapabilityImplementationDependencyInfo {
                    kind: dependency.kind,
                    name,
                    ty,
                    target_name: None,
                })
            }
        }
    }

    pub(super) fn convert_capability_implementation_operation(
        &self,
        operation: &CapabilityImplementationOperation,
    ) -> Result<CapabilityImplementationOperationInfo, TypeEnvError> {
        let param_mapping = HashMap::new();
        let param_names = operation
            .params
            .iter()
            .map(|param| param.name.to_string())
            .collect();
        let params = operation
            .params
            .iter()
            .map(|param| surface_type_to_type(&param.ty, &param_mapping, self))
            .collect::<Result<Vec<_>, _>>()?;
        let return_type = surface_type_to_type(&operation.return_type, &param_mapping, self)?;
        Ok(CapabilityImplementationOperationInfo {
            mode: operation.mode,
            param_names,
            params,
            return_type,
        })
    }

    pub(super) fn validate_capability_implementation_operation_body(
        &self,
        implementation_name: &str,
        operation: &CapabilityImplementationOperation,
        operation_info: &CapabilityImplementationOperationInfo,
        dependencies: &[CapabilityImplementationDependencyInfo],
    ) -> Result<(), TypeEnvError> {
        let mut body_env = self.capability_implementation_body_env(operation_info.mode);
        for dependency in dependencies {
            if !matches!(
                dependency.kind,
                CapabilityImplementationDependencyKind::Config
            ) {
                continue;
            }
            body_env.bind_variable(&dependency.name, dependency.ty.clone());
        }
        for (param_name, param_type) in operation_info
            .param_names
            .iter()
            .zip(operation_info.params.iter())
        {
            body_env.bind_variable(param_name, param_type.clone());
        }

        let body_result = crate::check_expr::check_expr(&body_env, &operation.body);
        if !body_result.is_ok() {
            let reason = body_result
                .errors
                .into_iter()
                .next()
                .map(|error| error.to_string())
                .unwrap_or_else(|| {
                    format!(
                        "failed to typecheck body for capability implementation operation '{}::{}'",
                        implementation_name, operation.name
                    )
                });
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "invalid capability implementation operation body for '{}::{}': {}",
                    implementation_name, operation.name, reason
                ),
                operation.span,
            ));
        }

        let actual_return_ty = body_result.substitution.apply(&body_result.ty);
        self.unify_types(&operation_info.return_type, &actual_return_ty)
            .map(|_| ())
            .map_err(|_| {
                TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation operation body '{}::{}' must return {}, found {}",
                        implementation_name,
                        operation.name,
                        operation_info.return_type,
                        actual_return_ty
                    ),
                    operation.span,
                )
            })
    }

    pub(super) fn capability_implementation_body_env(&self, mode: CapabilityOperationMode) -> Self {
        let mut body_env = Self {
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
            capability_bindings: HashMap::new(),
            impls: self.impls.clone(),
            proposition_assumptions: self.proposition_assumptions.clone(),
            proposition_obligations: self.proposition_obligations.clone(),
            proposition_predicate_aliases: self.proposition_predicate_aliases.clone(),
            proposition_predicates: self.proposition_predicates.clone(),
            type_var_interface_bounds: self.type_var_interface_bounds.clone(),
            type_parameter_kinds: self.type_parameter_kinds.clone(),
            variables: HashMap::with_capacity(10),
            workflow_intrinsics: self.workflow_intrinsics.clone(),
            public_workflow_summaries: HashMap::new(),
            fn_contracts: HashMap::new(),
            capability_symbols: HashSet::new(),
            parent: None,
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
            workflow_effect: None,
            capability_implementation_body: true,
        };
        let effect = match mode {
            CapabilityOperationMode::Observe => ash_core::Effect::Epistemic,
            CapabilityOperationMode::Execute => ash_core::Effect::Operational,
        };
        body_env.set_workflow_effect(effect);
        body_env
    }

    pub(super) fn type_constructor_expr_kind(&self, expr: &TypeConstructorExpr) -> Option<Kind> {
        match expr {
            TypeConstructorExpr::ProperType(_) => Some(Kind::Type),
            TypeConstructorExpr::ConstructorHead(head) => match head {
                TypeConstructorHeadId::Nominal { visible_name, .. } => self
                    .type_constructor_arity_for_visible_name(visible_name)
                    .map(Kind::n_ary),
                TypeConstructorHeadId::Computation(_) => None,
                _ => None,
            },
            TypeConstructorExpr::PartialApplication(app) => Some(app.result_kind.clone()),
            _ => None,
        }
    }

    pub(super) fn lower_interface_evidence_args(
        &self,
        interface_name: &str,
        interface: &InterfaceInfo,
        args: &[SurfaceType],
        param_mapping: &HashMap<String, TypeVar>,
    ) -> Result<Vec<InterfaceEvidenceArg>, TypeEnvError> {
        interface
            .type_param_kinds
            .iter()
            .zip(args.iter())
            .map(|(expected_kind, arg)| {
                if expected_kind.is_type() {
                    return surface_type_to_type(arg, param_mapping, self)
                        .map(InterfaceEvidenceArg::Proper);
                }

                let expr = match arg {
                    SurfaceType::Name(name) => {
                        let constructor = name.to_string();
                        let arity = self
                            .type_constructor_arity_for_visible_name(name.as_ref())
                            .ok_or_else(|| {
                                TypeEnvError::InvalidDefinition(
                                    format!(
                                        "unknown constructor evidence argument '{constructor}' for interface '{interface_name}'"
                                    ),
                                    Span::default(),
                                )
                            })?;
                        if arity == 0 {
                            self.lower_surface_type_to_canonical(arg)
                                .map(TypeConstructorExpr::ProperType)
                                .map_err(|err| {
                                    TypeEnvError::InvalidDefinition(
                                        format!(
                                            "invalid constructor evidence argument for interface '{interface_name}': {err}"
                                        ),
                                        Span::default(),
                                    )
                                })?
                        } else {
                            let origin = self
                                .type_identity_for_name(name.as_ref())
                                .cloned()
                                .unwrap_or_else(|| fallback_canonical_type_decl_id(name.as_ref()));
                            TypeConstructorExpr::ConstructorHead(TypeConstructorHeadId::nominal(
                                origin,
                                constructor,
                            ))
                        }
                    }
                    _ => self.elaborate_partial_type_constructor(arg, false).map_err(|err| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "invalid constructor evidence argument for interface '{interface_name}': {err}"
                            ),
                            Span::default(),
                        )
                    })?,
                };
                let found_kind = self.type_constructor_expr_kind(&expr).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "unsupported constructor evidence argument '{}' for interface '{interface_name}'",
                            render_type_constructor_expr(&expr)
                        ),
                        Span::default(),
                    )
                })?;
                if &found_kind != expected_kind {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "interface '{interface_name}' evidence argument '{}' has kind {found_kind}, expected {expected_kind}",
                            render_type_constructor_expr(&expr)
                        ),
                        Span::default(),
                    ));
                }

                Ok(InterfaceEvidenceArg::Constructor(Box::new(expr)))
            })
            .collect()
    }

    /// Register a closed-world interface impl.
    pub fn register_impl(&mut self, def: &ImplDef) -> Result<(), TypeEnvError> {
        let interface_name = def.interface.to_string();
        let interface = self
            .interfaces
            .get(&interface_name)
            .cloned()
            .ok_or_else(|| {
                TypeEnvError::MissingInterface(interface_name.clone(), Span::default())
            })?;

        if interface.type_params.len() != def.type_args.len() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "interface '{}' expects {} type parameters, but impl provides {}",
                    interface_name,
                    interface.type_params.len(),
                    def.type_args.len()
                ),
                Span::default(),
            ));
        }
        reject_constructor_kinded_interface_params(&def.type_params, "impl parameter", "TASK-908")?;

        let param_mapping: HashMap<String, TypeVar> = def
            .type_params
            .iter()
            .map(|param| (param.to_string(), TypeVar::fresh()))
            .collect();

        let head_args = self.lower_interface_evidence_args(
            &interface_name,
            &interface,
            &def.type_args,
            &param_mapping,
        )?;

        let lowered_type_args: Vec<Type> = head_args
            .iter()
            .map(|arg| interface_evidence_arg_as_legacy_type_with_params(arg, &param_mapping))
            .collect();

        if def.type_params.is_empty()
            && !lowered_type_args
                .iter()
                .all(is_closed_world_nominal_impl_target)
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!("impl for interface '{interface_name}' must target concrete nominal types"),
                Span::default(),
            ));
        }

        let impl_head = Type::Constructor {
            name: QualifiedName::root(interface_name.clone()),
            args: lowered_type_args.clone(),
            kind: Kind::Type,
        };

        // Overlap check
        for scheme in self.impls.iter().filter(|s| s.interface == interface_name) {
            if self.unify_types(&scheme.head, &impl_head).is_ok() {
                if scheme.type_params.is_empty() && def.type_params.is_empty() {
                    if head_args
                        .iter()
                        .any(|arg| matches!(arg, InterfaceEvidenceArg::Constructor(_)))
                    {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "duplicate overlapping impl for evidence {}",
                                render_interface_evidence_key(&interface_name, &head_args)
                            ),
                            Span::default(),
                        ));
                    }
                    return Err(TypeEnvError::DuplicateImpl {
                        interface: interface_name,
                        ty: impl_head.to_string(),
                        span: Span::default(),
                    });
                }
                return Err(TypeEnvError::OverlappingImpls {
                    interface: interface_name,
                    span: Span::default(),
                });
            }
        }

        let where_bounds: Vec<WhereBound> = def
            .where_bounds
            .iter()
            .map(|wb| {
                let type_var = param_mapping
                    .get(wb.param.as_ref())
                    .copied()
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!("unknown type parameter '{}' in where bound", wb.param),
                            Span::default(),
                        )
                    })?;
                let bound_interface = wb.bound.to_string();
                if !self.has_interface(&bound_interface) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!("unknown interface '{}' in where bound", bound_interface),
                        Span::default(),
                    ));
                }
                Ok(WhereBound {
                    type_var,
                    interface: bound_interface,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut impl_binding_env = self.clone();
        for bound in &where_bounds {
            impl_binding_env
                .type_var_interface_bounds
                .entry(bound.type_var)
                .or_default()
                .insert(bound.interface.clone());
        }

        let family_declarations = self
            .associated_family_declarations_for_interface(&interface_name)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        let family_names = family_declarations
            .iter()
            .map(|decl| decl.head.member.name.to_string())
            .collect::<HashSet<_>>();
        let ordinary_associated_names = interface
            .associated_types
            .iter()
            .filter(|name| !family_names.contains(name.as_str()))
            .cloned()
            .collect::<HashSet<_>>();
        let mut family_var_constraints = HashMap::new();
        for param in &def.type_params {
            if let Some(domain) =
                self.optional_param_domain_constraint(param.domain.as_ref(), param.span)?
            {
                family_var_constraints.insert(
                    param.name.to_string(),
                    AssociatedFamilyResultConstraint::Domain(domain),
                );
            }
        }
        for family in &family_declarations {
            for (arg, param) in def.type_args.iter().zip(family.interface_params.iter()) {
                if let (SurfaceType::Name(name), Some(domain)) =
                    (arg, param.domain_constraint.as_ref())
                {
                    family_var_constraints
                        .entry(name.to_string())
                        .or_insert_with(|| {
                            AssociatedFamilyResultConstraint::Domain(domain.clone())
                        });
                }
            }
        }
        let impl_family_module = if family_declarations.is_empty() {
            None
        } else {
            Some(self.current_module_identity.clone().ok_or_else(|| {
                TypeEnvError::AssociatedFamilyModuleOwnerViolation {
                    family: family_declarations
                        .first()
                        .map(|family| family.head.member.name.to_string())
                        .unwrap_or_else(|| "<unknown>".to_string()),
                    reason: "missing current module identity while registering sealed family impl"
                        .to_string(),
                    span: def.span,
                }
            })?)
        };

        for binding in &def.associated_type_bindings {
            let binding_name = binding.name.to_string();
            if !interface.associated_types.contains(&binding_name) {
                return Err(
                    if family_declarations.is_empty() || !ordinary_associated_names.is_empty() {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "extraneous associated type binding '{binding_name}' in impl for interface '{interface_name}'"
                            ),
                            binding.span,
                        )
                    } else {
                        TypeEnvError::ExtraAssociatedFamilyBinding {
                            interface: interface_name.clone(),
                            family: binding_name,
                            span: binding.span,
                        }
                    },
                );
            }
        }

        let mut staged_family_schemes = Vec::new();
        for family in &family_declarations {
            let family_name = family.head.member.name.to_string();
            let Some(binding) = def
                .associated_type_bindings
                .iter()
                .find(|binding| binding.name.as_ref() == family_name)
            else {
                return Err(TypeEnvError::MissingAssociatedFamilyBinding {
                    interface: interface_name.clone(),
                    family: family_name,
                    span: def.span,
                });
            };
            let result = self
                .lower_associated_family_result_expr(
                    &binding.ty,
                    &family.result_domain,
                    &family_var_constraints,
                    binding.span,
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
            let params = family
                .interface_params
                .iter()
                .map(|param| AssociatedFamilySchemeParam {
                    name: param.name.clone(),
                    ty: CanonicalTypeExpr::Var(param.name.clone()),
                    kind: Kind::Type,
                    domain_constraint: param.domain_constraint.clone(),
                    source_anchor: span_anchor(
                        binding.span,
                        format!("associated family param {}", param.name),
                    ),
                })
                .collect::<Vec<_>>();
            let interface_arg_patterns = def
                .type_args
                .iter()
                .zip(family.interface_params.iter())
                .map(|(arg, param)| {
                    self.lower_associated_family_pattern(
                        arg,
                        param.domain_constraint.as_ref(),
                        &family_var_constraints,
                        binding.span,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let scheme = AssociatedFamilyScheme {
                head: family.head.clone(),
                params,
                result_domain: canonical_expr_for_associated_family_constraint(
                    &family.result_domain,
                ),
                result_kind: Kind::Type,
                equations: vec![AssociatedFamilyEquation {
                    head: family.head.clone(),
                    ordinal: 0,
                    interface_arg_patterns,
                    result,
                    decreases: None,
                    source_anchor: span_anchor(binding.span, "associated family equation"),
                    case_head_anchor: span_anchor(binding.span, "associated family case head"),
                }],
                source_anchor: span_anchor(binding.span, "associated family scheme"),
            };
            let defining_module = impl_family_module
                .clone()
                .expect("family declarations require module context");
            staged_family_schemes.push((scheme, defining_module));
        }

        let associated_type_bindings: HashMap<String, Type> = def
            .associated_type_bindings
            .iter()
            .filter(|binding| !family_names.contains(binding.name.as_ref()))
            .map(|binding| {
                let ty = surface_type_to_type(&binding.ty, &param_mapping, &impl_binding_env)?;
                if let Some(name) = unresolved_associated_projection_name(&ty) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "unresolved associated type '{name}' in impl associated type binding '{}' for interface '{interface_name}'",
                            binding.name
                        ),
                        Span::default(),
                    ));
                }
                Ok((binding.name.to_string(), ty))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        for assoc_name in &interface.associated_types {
            if family_names.contains(assoc_name) {
                continue;
            }
            if !associated_type_bindings.contains_key(assoc_name) {
                return Err(TypeEnvError::MissingAssociatedType {
                    interface: interface_name.clone(),
                    name: assoc_name.clone(),
                    span: Span::default(),
                });
            }
        }
        for bound_name in associated_type_bindings.keys() {
            if !interface.associated_types.contains(bound_name) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "extraneous associated type binding '{bound_name}' in impl for interface '{interface_name}'"
                    ),
                    Span::default(),
                ));
            }
        }

        self.validate_impl_required_evidence(&interface, &head_args, &where_bounds, def.span)?;

        let temp_scheme = ImplScheme {
            interface: interface.name.clone(),
            type_params: param_mapping.values().copied().collect(),
            head: impl_head.clone(),
            head_args: head_args.clone(),
            where_bounds: where_bounds.clone(),
            associated_type_bindings: associated_type_bindings.clone(),
            methods: vec![],
        };
        let constructor_arg_mapping = interface
            .type_params
            .iter()
            .cloned()
            .zip(head_args.iter().cloned())
            .collect::<HashMap<_, _>>();

        let mut method_names = HashSet::new();
        let mut method_infos = Vec::new();
        for method in &def.methods {
            let method_name = method.name.to_string();
            let Some(method_info) = interface.methods.get(&method_name) else {
                return Err(TypeEnvError::MissingInterfaceMethod {
                    interface: interface.name.clone(),
                    method: method_name,
                    span: Span::default(),
                });
            };

            if !method_names.insert(method_name.clone()) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "duplicate method '{method_name}' in impl for interface '{}'",
                        interface.name
                    ),
                    Span::default(),
                ));
            }

            if method_info.params.len() != method.params.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "impl method '{}::{}' signature expects {} parameters, found {}",
                        interface.name,
                        method_name,
                        method_info.params.len(),
                        method.params.len()
                    ),
                    Span::default(),
                ));
            }

            let mut subst = Substitution::new();
            for (tv, concrete_arg) in method_info.type_params.iter().zip(lowered_type_args.iter()) {
                subst.insert(*tv, concrete_arg.clone());
            }

            let mut method_env = self.clone();
            for (param_name, param_type) in method.params.iter().zip(method_info.params.iter()) {
                let param_ty = substitute_constructor_variable_apps(
                    &subst.apply(param_type),
                    &constructor_arg_mapping,
                    &param_mapping,
                );
                method_env.bind_variable(param_name.as_ref(), param_ty);
            }

            let expected_return_ty = substitute_constructor_variable_apps(
                &subst.apply(&method_info.return_type),
                &constructor_arg_mapping,
                &param_mapping,
            );
            let expected_return_ty =
                self.normalize_associated_types(&expected_return_ty, &temp_scheme, &subst)?;

            let body_result = crate::check_expr::check_expr(&method_env, &method.body);
            if !body_result.is_ok() {
                let reason = body_result
                    .errors
                    .into_iter()
                    .next()
                    .map(|error| error.to_string())
                    .unwrap_or_else(|| {
                        format!(
                            "failed to typecheck body for impl method '{}::{}'",
                            interface.name, method_name
                        )
                    });

                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "invalid impl method body for '{}::{}': {}",
                        interface.name, method_name, reason
                    ),
                    Span::default(),
                ));
            }

            let actual_return_ty = body_result.substitution.apply(&body_result.ty);
            let return_substitution = self
                .unify_types(&expected_return_ty, &actual_return_ty)
                .map_err(|_| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "impl method '{}::{}' must return {}, found {}",
                            interface.name, method_name, expected_return_ty, actual_return_ty
                        ),
                        Span::default(),
                    )
                })?;
            for method_var in &method_info.method_type_params {
                let inferred = return_substitution
                    .apply(&body_result.substitution.apply(&Type::Var(*method_var)));
                if inferred != Type::Var(*method_var) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "impl method '{}::{}' must keep method payload type variable {} independent, but body constrains it to {}",
                            interface.name, method_name, method_var.0, inferred
                        ),
                        Span::default(),
                    ));
                }
            }

            let core_body = ash_parser::lower_expr(&method.body).map_err(|e| {
                TypeEnvError::InvalidDefinition(format!("lowering error: {e}"), Span::default())
            })?;

            method_infos.push(ImplMethodInfo {
                name: method_name,
                param_names: method
                    .params
                    .iter()
                    .map(|param| param.to_string())
                    .collect(),
                type_params: method_info.type_params.clone(),
                method_type_params: method_info.method_type_params.clone(),
                params: method_info
                    .params
                    .iter()
                    .map(|t| {
                        substitute_constructor_variable_apps(
                            &subst.apply(t),
                            &constructor_arg_mapping,
                            &param_mapping,
                        )
                    })
                    .collect(),
                return_type: expected_return_ty,
                body: core_body,
            });
        }

        for required_method in interface.methods.keys() {
            if !method_names.contains(required_method) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "impl for interface '{}' is missing method '{required_method}'",
                        interface.name
                    ),
                    Span::default(),
                ));
            }
        }

        let previous_family_schemes = self.associated_family_schemes.clone();
        for (scheme, defining_module) in staged_family_schemes {
            if let Err(error) =
                self.register_associated_family_scheme_with_totality(scheme, defining_module, false)
            {
                self.associated_family_schemes = previous_family_schemes;
                return Err(error);
            }
        }

        for (bound, source_bound) in where_bounds.iter().zip(def.where_bounds.iter()) {
            self.record_type_var_interface_bound_assumption(
                bound.type_var,
                &bound.interface,
                proposition_source_anchor(
                    SourceOrigin::Synthetic {
                        reason: "impl where-bound proposition assumption".to_string(),
                    },
                    source_bound.span,
                    format!(
                        "impl where-bound type variable {} satisfies interface {}",
                        bound.type_var.0, bound.interface
                    ),
                ),
                PropositionCheckingSite::new(
                    0x8752_0000u64 + u64::from(bound.type_var.0),
                    PropositionCheckingSiteKind::ImplWhereBound,
                    Some(format!(
                        "impl where type_var_{}: {}",
                        bound.type_var.0, bound.interface
                    )),
                ),
            );
        }

        if def.type_params.is_empty() {
            self.record_concrete_impl_interface_assumption(
                &interface.name,
                &lowered_type_args,
                proposition_source_anchor(
                    SourceOrigin::Synthetic {
                        reason: "concrete impl proposition assumption".to_string(),
                    },
                    def.span,
                    format!("concrete impl evidence for interface {}", interface.name),
                ),
            );
        }

        self.impls.push(ImplScheme {
            interface: interface.name,
            type_params: param_mapping.values().copied().collect(),
            head: impl_head,
            head_args,
            where_bounds,
            associated_type_bindings,
            methods: method_infos,
        });

        Ok(())
    }

    /// Look up a constructor by name
    ///
    /// Returns `Some((type_name, variant_index))` if found, `None` otherwise
    pub fn lookup_constructor(&self, name: &str) -> Option<(TypeName, VariantIndex)> {
        self.constructors.get(name).cloned()
    }

    /// Look up a type definition by name (AST version)
    pub fn lookup_type(&self, name: &str) -> Option<&TypeDef> {
        self.ast_types.get(name)
    }

    /// Iterate over AST type definitions visible in this environment.
    pub fn ast_type_defs(&self) -> impl Iterator<Item = (&TypeName, &TypeDef)> {
        self.ast_types.iter()
    }

    /// Look up internal type info by name
    pub fn lookup_type_info(&self, name: &str) -> Option<&TypeInfo> {
        self.type_info.get(name)
    }

    #[cfg(test)]
    pub(crate) fn remove_type_info_for_test(&mut self, name: &str) {
        self.type_info.remove(name);
    }

    /// Get the variant definition for a constructor
    pub fn get_variant(
        &self,
        constructor_name: &str,
    ) -> Option<(&TypeInfo, VariantIndex, &VariantInfo)> {
        let (type_name, variant_index) = self.lookup_constructor(constructor_name)?;
        let type_info = self.type_info.get(&type_name)?;

        if let TypeInfo::Enum { variants, .. } = type_info {
            variants
                .get(variant_index)
                .map(|v| (type_info, variant_index, v))
        } else {
            None
        }
    }

    /// Add builtin types (Option, Result, and List)
    pub fn add_builtin_types(&mut self) {
        self.add_option_type();
        self.add_result_type();
        self.add_list_type();
        self.add_record_type();
        self.add_act_type();
        self.add_proc_type();
        self.add_workflow_type();
        self.add_process_handle_type();
        self.add_act_builtin_values();
        self.add_proc_builtin_values();
        self.add_workflow_builtin_values();
        self.add_result_builtin_values();
        self.add_filesystem_builtin_values();
        self.add_http_builtin_values();
        self.add_builtin_capability_symbols();
    }

    pub(super) fn add_builtin_capability_symbols(&mut self) {
        for capability in ["Args", "Dir", "Fs", "Meta", "Stdio"] {
            self.register_capability_symbol(capability);
        }
    }

    /// Add filesystem stdlib wrapper signatures used by provider-backed builtin dispatch.
    pub(super) fn add_filesystem_builtin_values(&mut self) {
        let path = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("PathBuf"),
            args: vec![],
            kind: crate::Kind::Type,
        };

        for module in ["fs", "io::fs"] {
            let exists = format!("{module}::exists");
            self.bind_variable(
                &exists,
                crate::types::Type::Fn(vec![path.clone()], Box::new(crate::types::Type::Bool)),
            );
            let read_to_string = format!("{module}::read_to_string");
            self.bind_variable(
                &read_to_string,
                crate::types::Type::Fn(vec![path.clone()], Box::new(crate::types::Type::String)),
            );
            let append = format!("{module}::append");
            self.bind_variable(
                &append,
                crate::types::Type::Fn(
                    vec![path.clone(), crate::types::Type::String],
                    Box::new(crate::types::Type::Null),
                ),
            );
            let write_string = format!("{module}::write_string");
            self.bind_variable(
                &write_string,
                crate::types::Type::Fn(
                    vec![path.clone(), crate::types::Type::String],
                    Box::new(crate::types::Type::Null),
                ),
            );
        }

        for module in ["dir", "io::dir"] {
            let read_dir = format!("{module}::read_dir");
            self.bind_variable(
                &read_dir,
                crate::types::Type::Fn(
                    vec![path.clone()],
                    Box::new(crate::types::Type::List(Box::new(
                        crate::types::Type::String,
                    ))),
                ),
            );
        }

        let metadata_ty = crate::types::Type::Record(vec![
            ("is_file".into(), crate::types::Type::Bool),
            ("is_dir".into(), crate::types::Type::Bool),
            ("len".into(), crate::types::Type::Int),
            ("readonly".into(), crate::types::Type::Bool),
        ]);
        for module in ["meta", "io::meta"] {
            let metadata = format!("{module}::metadata");
            self.bind_variable(
                &metadata,
                crate::types::Type::Fn(vec![path.clone()], Box::new(metadata_ty.clone())),
            );
        }
    }

    /// Add HTTP stdlib wrapper signatures used by provider-backed builtin dispatch.
    pub(super) fn add_http_builtin_values(&mut self) {
        let response_ty = crate::types::Type::Record(vec![
            ("status".into(), crate::types::Type::Int),
            ("headers".into(), crate::types::Type::Record(vec![])),
            ("body".into(), crate::types::Type::String),
        ]);

        self.bind_variable(
            "http::get",
            crate::types::Type::Fn(
                vec![crate::types::Type::String],
                Box::new(response_ty.clone()),
            ),
        );
        self.bind_variable(
            "http::post",
            crate::types::Type::Fn(
                vec![crate::types::Type::String, crate::types::Type::String],
                Box::new(response_ty.clone()),
            ),
        );
        self.bind_variable(
            "http::put",
            crate::types::Type::Fn(
                vec![crate::types::Type::String, crate::types::Type::String],
                Box::new(response_ty.clone()),
            ),
        );
        self.bind_variable(
            "http::delete",
            crate::types::Type::Fn(vec![crate::types::Type::String], Box::new(response_ty)),
        );
    }

    /// Add the Option<T> type
    pub(super) fn add_option_type(&mut self) {
        // Option<T> = Some { value: T } | None
        let option_type = TypeDef {
            name: "Option".to_string(),
            params: vec!["T".to_string()],
            body: TypeBody::Enum(vec![
                VariantDef {
                    name: "Some".to_string(),
                    fields: vec![("value".to_string(), TypeExpr::Named("T".to_string()))],
                    payload: VariantPayload::Record(vec![(
                        "value".to_string(),
                        TypeExpr::Named("T".to_string()),
                    )]),
                },
                VariantDef {
                    name: "None".to_string(),
                    fields: vec![],
                    payload: VariantPayload::Unit,
                },
            ]),
            visibility: ash_core::ast::Visibility::Public,
            builtin: false,
        };

        self.register_type_identity(&option_type)
            .expect("Failed to register Option type");
        self.expose_type_representation("Option")
            .expect("Failed to expose Option constructors");
    }

    /// Add the Result<T, E> type
    pub(super) fn add_result_type(&mut self) {
        // Result<T, E> = Ok { value: T } | Err { error: E }
        let result_type = TypeDef {
            name: "Result".to_string(),
            params: vec!["T".to_string(), "E".to_string()],
            body: TypeBody::Enum(vec![
                VariantDef {
                    name: "Ok".to_string(),
                    fields: vec![("value".to_string(), TypeExpr::Named("T".to_string()))],
                    payload: VariantPayload::Record(vec![(
                        "value".to_string(),
                        TypeExpr::Named("T".to_string()),
                    )]),
                },
                VariantDef {
                    name: "Err".to_string(),
                    fields: vec![("error".to_string(), TypeExpr::Named("E".to_string()))],
                    payload: VariantPayload::Record(vec![(
                        "error".to_string(),
                        TypeExpr::Named("E".to_string()),
                    )]),
                },
            ]),
            visibility: ash_core::ast::Visibility::Public,
            builtin: false,
        };

        self.register_type(&result_type)
            .expect("Failed to register Result type");
    }

    /// Add the List<T> type
    pub(super) fn add_list_type(&mut self) {
        // List<T> is a generic builtin type represented as a struct with a type parameter
        let list_type = TypeDef {
            name: "List".to_string(),
            params: vec!["T".to_string()],
            body: TypeBody::Struct(vec![]), // opaque builtin; no fields needed for type checking
            visibility: ash_core::ast::Visibility::Public,
            builtin: true,
        };

        self.register_type(&list_type)
            .expect("Failed to register List type");
    }

    /// Add the Record type
    pub(super) fn add_record_type(&mut self) {
        let record_type = TypeDef {
            name: "Record".to_string(),
            params: vec![],
            body: TypeBody::Struct(vec![]),
            visibility: ash_core::ast::Visibility::Public,
            builtin: true,
        };

        self.register_type_identity(&record_type)
            .expect("Failed to register Record type");
        self.expose_type_representation("Record")
            .expect("Failed to expose Record representation");
    }

    /// Add the Act<T> type
    pub(super) fn add_act_type(&mut self) {
        let act_type = TypeDef {
            name: "Act".to_string(),
            params: vec!["T".to_string()],
            body: TypeBody::Struct(vec![]),
            visibility: ash_core::ast::Visibility::Public,
            builtin: true,
        };

        self.register_type(&act_type)
            .expect("Failed to register Act type");
    }

    /// Add the Proc<T> type.
    pub(super) fn add_proc_type(&mut self) {
        let proc_type = TypeDef {
            name: "Proc".to_string(),
            params: vec!["T".to_string()],
            body: TypeBody::Struct(vec![]),
            visibility: ash_core::ast::Visibility::Public,
            builtin: true,
        };

        self.register_type(&proc_type)
            .expect("Failed to register Proc type");
    }

    /// Add the public Workflow<T> type.
    pub(super) fn add_workflow_type(&mut self) {
        let workflow_type = TypeDef {
            name: "Workflow".to_string(),
            params: vec!["T".to_string()],
            body: TypeBody::Struct(vec![]),
            visibility: ash_core::ast::Visibility::Public,
            builtin: true,
        };

        self.register_type(&workflow_type)
            .expect("Failed to register Workflow type");
    }

    /// Add the opaque P<T> process handle type.
    pub(super) fn add_process_handle_type(&mut self) {
        let process_handle_type = TypeDef {
            name: "P".to_string(),
            params: vec!["T".to_string()],
            body: TypeBody::Struct(vec![]),
            visibility: ash_core::ast::Visibility::Public,
            builtin: true,
        };

        self.register_type(&process_handle_type)
            .expect("Failed to register P type");
    }

    /// Add the qualified act module builtin value signatures.
    pub(super) fn add_act_builtin_values(&mut self) {
        let a = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let b = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let act_a = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Act"),
            args: vec![a.clone()],
            kind: crate::Kind::Type,
        };
        let act_b = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Act"),
            args: vec![b.clone()],
            kind: crate::Kind::Type,
        };

        self.bind_variable(
            "act::unit",
            crate::types::Type::Fn(vec![a.clone()], Box::new(act_a.clone())),
        );
        self.bind_variable(
            "act::bind",
            crate::types::Type::Fn(
                vec![
                    act_a.clone(),
                    crate::types::Type::Fn(vec![a], Box::new(act_b.clone())),
                ],
                Box::new(act_b.clone()),
            ),
        );
        self.bind_variable(
            "act::then",
            crate::types::Type::Fn(vec![act_a.clone(), act_b.clone()], Box::new(act_b)),
        );
        self.bind_variable(
            "act::guard",
            crate::types::Type::Fn(
                vec![crate::types::Type::String, act_a.clone()],
                Box::new(act_a),
            ),
        );
        self.bind_variable(
            "act::policy_check",
            crate::types::Type::Fn(
                vec![crate::types::Type::String],
                Box::new(crate::types::Type::Bool),
            ),
        );
    }

    /// Add the qualified proc module builtin value signatures.
    pub(super) fn add_proc_builtin_values(&mut self) {
        let a = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let b = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let act_a = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Act"),
            args: vec![a.clone()],
            kind: crate::Kind::Type,
        };
        let proc_a = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Proc"),
            args: vec![a.clone()],
            kind: crate::Kind::Type,
        };
        let proc_b = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Proc"),
            args: vec![b.clone()],
            kind: crate::Kind::Type,
        };
        let handle_a = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("P"),
            args: vec![a.clone()],
            kind: crate::Kind::Type,
        };
        let handle_b = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("P"),
            args: vec![b.clone()],
            kind: crate::Kind::Type,
        };
        let proc_null = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Proc"),
            args: vec![crate::types::Type::Null],
            kind: crate::Kind::Type,
        };
        let proc_pair_handles = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Proc"),
            args: vec![crate::types::Type::Record(vec![
                ("_0".into(), handle_a.clone()),
                ("_1".into(), handle_b.clone()),
            ])],
            kind: crate::Kind::Type,
        };
        let proc_pair_ab = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Proc"),
            args: vec![crate::types::Type::Record(vec![
                ("_0".into(), a.clone()),
                ("_1".into(), b.clone()),
            ])],
            kind: crate::Kind::Type,
        };
        let list_a = crate::types::Type::List(Box::new(a.clone()));
        let list_handle_a = crate::types::Type::List(Box::new(handle_a.clone()));
        let proc_list_a = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Proc"),
            args: vec![list_a.clone()],
            kind: crate::Kind::Type,
        };
        let list_handle_b = crate::types::Type::List(Box::new(handle_b.clone()));
        let proc_list_handle_b = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Proc"),
            args: vec![list_handle_b],
            kind: crate::Kind::Type,
        };

        self.bind_variable(
            "proc::unit",
            crate::types::Type::Fn(vec![a.clone()], Box::new(proc_a.clone())),
        );
        self.bind_variable(
            "proc::from_act",
            crate::types::Type::Fn(vec![act_a], Box::new(proc_a.clone())),
        );
        self.bind_variable(
            "proc::bind",
            crate::types::Type::Fn(
                vec![
                    proc_a.clone(),
                    crate::types::Type::Fn(vec![a.clone()], Box::new(proc_b.clone())),
                ],
                Box::new(proc_b.clone()),
            ),
        );
        self.bind_variable(
            "proc::then",
            crate::types::Type::Fn(
                vec![proc_a.clone(), proc_b.clone()],
                Box::new(proc_b.clone()),
            ),
        );
        self.bind_variable(
            "proc::await",
            crate::types::Type::Fn(vec![handle_a.clone()], Box::new(proc_a.clone())),
        );
        self.bind_variable(
            "proc::yield",
            crate::types::Type::Fn(vec![], Box::new(proc_null)),
        );
        self.bind_variable(
            "proc::par",
            crate::types::Type::Fn(
                vec![proc_a.clone(), proc_b.clone()],
                Box::new(proc_pair_handles),
            ),
        );
        self.bind_variable(
            "proc::scatter",
            crate::types::Type::Fn(
                vec![list_a, crate::types::Type::Fn(vec![a], Box::new(proc_b))],
                Box::new(proc_list_handle_b),
            ),
        );
        self.bind_variable(
            "proc::join",
            crate::types::Type::Fn(vec![handle_a, handle_b], Box::new(proc_pair_ab)),
        );
        self.bind_variable(
            "proc::gather",
            crate::types::Type::Fn(vec![list_handle_a], Box::new(proc_list_a)),
        );
    }

    /// Add the qualified workflow module builtin value signatures.
    pub(super) fn add_workflow_builtin_values(&mut self) {
        let a = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let b = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let workflow_a = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Workflow"),
            args: vec![a.clone()],
            kind: crate::Kind::Type,
        };
        let workflow_b = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Workflow"),
            args: vec![b.clone()],
            kind: crate::Kind::Type,
        };
        let proc_a = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Proc"),
            args: vec![a.clone()],
            kind: crate::Kind::Type,
        };
        let act_a = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Act"),
            args: vec![a.clone()],
            kind: crate::Kind::Type,
        };
        self.bind_variable(
            "workflow::unit",
            crate::types::Type::Fn(vec![a.clone()], Box::new(workflow_a.clone())),
        );
        self.bind_variable(
            "workflow::bind",
            crate::types::Type::Fn(
                vec![
                    workflow_a.clone(),
                    crate::types::Type::Fn(vec![a], Box::new(workflow_b.clone())),
                ],
                Box::new(workflow_b.clone()),
            ),
        );
        self.bind_variable(
            "workflow::then",
            crate::types::Type::Fn(
                vec![workflow_a.clone(), workflow_b.clone()],
                Box::new(workflow_b),
            ),
        );
        self.bind_variable(
            "workflow::from_proc",
            crate::types::Type::Fn(vec![proc_a], Box::new(workflow_a.clone())),
        );
        self.bind_variable(
            "workflow::from_act",
            crate::types::Type::Fn(vec![act_a], Box::new(workflow_a)),
        );
        let workflow_unit = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Workflow"),
            args: vec![crate::types::Type::Null],
            kind: crate::Kind::Type,
        };
        self.workflow_intrinsics.insert(
            "workflow::requires".to_string(),
            WorkflowIntrinsic::requires(workflow_unit.clone()),
        );
        self.workflow_intrinsics.insert(
            "workflow::ensures".to_string(),
            WorkflowIntrinsic::ensures(workflow_unit),
        );
    }

    /// Add the qualified result module helper signatures used by the public tower manifest.
    pub(super) fn add_result_builtin_values(&mut self) {
        let t = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let e = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let u = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let result_t_e = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Result"),
            args: vec![t.clone(), e.clone()],
            kind: crate::Kind::Type,
        };
        let result_u_e = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Result"),
            args: vec![u.clone(), e],
            kind: crate::Kind::Type,
        };

        self.bind_variable(
            "result::and_then",
            crate::types::Type::Fn(
                vec![
                    result_t_e,
                    crate::types::Type::Fn(vec![t], Box::new(result_u_e.clone())),
                ],
                Box::new(result_u_e),
            ),
        );
    }

    /// Check if a type is registered
    pub fn has_type(&self, name: &str) -> bool {
        self.ast_types.contains_key(name)
    }

    /// Check if a type is registered with a full (non-placeholder) definition.
    /// Returns `false` for unregistered names and for placeholder entries.
    pub fn has_full_type(&self, name: &str) -> bool {
        match self.ast_types.get(name) {
            None => false,
            Some(_) => matches!(
                self.type_declaration_states.get(name),
                Some(TypeDeclarationState::Full)
            ),
        }
    }

    /// Check if a constructor is registered
    pub fn has_constructor(&self, name: &str) -> bool {
        self.constructors.contains_key(name)
    }

    /// Bind a variable to a type in this environment
    pub fn bind_variable(&mut self, name: &str, ty: crate::types::Type) {
        self.variables.insert(name.to_string(), ty);
    }

    /// Look up a compiler-known workflow intrinsic.
    pub fn lookup_workflow_intrinsic(&self, name: &str) -> Option<WorkflowIntrinsic> {
        self.workflow_intrinsics.get(name).cloned().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.lookup_workflow_intrinsic(name))
        })
    }

    /// Bind a public Workflow summary imported from module metadata.
    pub fn bind_public_workflow_summary(
        &mut self,
        name: &str,
        summary: ash_core::workflow_carrier::PublicWorkflowSummary,
    ) {
        self.public_workflow_summaries
            .insert(name.to_string(), summary);
    }

    /// Look up a public Workflow summary by local or qualified binding name.
    pub fn lookup_public_workflow_summary(
        &self,
        name: &str,
    ) -> Option<ash_core::workflow_carrier::PublicWorkflowSummary> {
        self.public_workflow_summaries
            .get(name)
            .cloned()
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|parent| parent.lookup_public_workflow_summary(name))
            })
    }

    /// Return the names of all registered unit constructors.
    pub fn unit_constructor_names(&self) -> impl Iterator<Item = String> + '_ {
        self.constructors.iter().filter_map(|(name, _)| {
            self.get_variant(name).and_then(|(_, _, variant)| {
                (variant.payload_shape == VariantPayloadShape::Unit).then(|| name.clone())
            })
        })
    }

    /// Return the names of all bound variables (used for name resolution of imported callables).
    pub fn variable_names(&self) -> impl Iterator<Item = String> + '_ {
        self.variables.keys().cloned()
    }

    /// Store the lowered contract boundary for a pure function.
    pub fn bind_fn_contract(&mut self, name: &str, contract: StoredFnContract) {
        self.fn_contracts.insert(name.to_string(), contract);
    }

    /// Record that a workflow type variable satisfies an interface bound.
    pub fn bind_type_var_interface_bound(&mut self, var: TypeVar, interface: &str) {
        let inserted = self
            .type_var_interface_bounds
            .entry(var)
            .or_default()
            .insert(interface.to_string());
        if inserted {
            self.record_type_var_interface_bound_assumption(
                var,
                interface,
                synthetic_proposition_source_anchor(format!(
                    "type variable {} satisfies interface {interface}",
                    var.0
                )),
                PropositionCheckingSite::new(
                    0x8751_0000u64 + u64::from(var.0),
                    PropositionCheckingSiteKind::TypeVariableInterfaceBound,
                    Some(format!("type_var_{}: {interface}", var.0)),
                ),
            );
        }
    }

    /// Register the kind of a source-visible type parameter in this TypeEnv.
    pub fn register_type_parameter_kind(
        &mut self,
        name: impl Into<String>,
        kind: Kind,
    ) -> Result<(), TypeEnvError> {
        let name = name.into();
        if let Some(existing) = self.type_parameter_kinds.get(&name)
            && existing != &kind
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "type parameter '{name}' already has kind {existing}, cannot also register kind {kind}"
                ),
                Span::default(),
            ));
        }
        self.type_parameter_kinds.insert(name, kind);
        Ok(())
    }

    /// Look up the kind of a source-visible type parameter.
    #[must_use]
    pub fn type_parameter_kind(&self, name: &str) -> Option<&Kind> {
        self.type_parameter_kinds.get(name).or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.type_parameter_kind(name))
        })
    }

    /// Look up a variable's type in this environment
    ///
    /// Searches current scope first, then parent scopes
    pub fn lookup_variable(&self, name: &str) -> Option<crate::types::Type> {
        if let Some(ty) = self.variables.get(name) {
            return Some(ty.clone());
        }
        if let Some(ref parent) = self.parent {
            return parent.lookup_variable(name);
        }
        None
    }

    /// Look up a lowered pure-function contract boundary.
    pub fn lookup_fn_contract(&self, name: &str) -> Option<StoredFnContract> {
        if let Some(contract) = self.fn_contracts.get(name) {
            return Some(contract.clone());
        }
        if let Some(ref parent) = self.parent {
            return parent.lookup_fn_contract(name);
        }
        None
    }

    /// Snapshot all lowered pure-function contract boundaries in scope.
    pub fn function_contracts(&self) -> HashMap<String, StoredFnContract> {
        let mut contracts = self
            .parent
            .as_ref()
            .map_or_else(HashMap::new, |parent| parent.function_contracts());
        contracts.extend(self.fn_contracts.clone());
        contracts
    }

    /// Resolve a function call target.
    ///
    /// Qualified calls must resolve to the exact qualified binding; they must not silently
    /// fall back to an unrelated unqualified function with the same base name.
    pub fn lookup_call_target(
        &self,
        module: Option<&str>,
        name: &str,
    ) -> Option<crate::types::Type> {
        match module {
            Some(module) => self.lookup_variable(&format!("{module}::{name}")),
            None => self.lookup_variable(name),
        }
    }

    pub fn register_capability_symbol(&mut self, name: impl Into<String>) {
        self.capability_symbols.insert(name.into());
    }

    pub fn has_capability_symbol(&self, name: &str) -> bool {
        self.capability_symbols.contains(name)
            || self
                .parent
                .as_ref()
                .is_some_and(|parent| parent.has_capability_symbol(name))
    }
}
