use super::*;

/// Unfolded type body with substituted type arguments
#[derive(Debug, Clone, PartialEq)]
pub enum UnfoldedBody {
    /// Enum with variants
    Enum(Vec<VariantInfo>),
    /// Struct with fields
    Struct(Vec<(FieldName, Type)>),
}

pub(super) fn detect_proof_cycle(
    proof_name: &str,
    graph: &BTreeMap<String, Vec<String>>,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
    stack: &mut Vec<String>,
) -> Option<Vec<String>> {
    if let Some(cycle_start) = stack.iter().position(|name| name == proof_name) {
        let mut cycle = stack[cycle_start..].to_vec();
        cycle.push(proof_name.to_string());
        return Some(cycle);
    }
    if visited.contains(proof_name) {
        return None;
    }

    visiting.insert(proof_name.to_string());
    stack.push(proof_name.to_string());
    if let Some(callees) = graph.get(proof_name) {
        for callee in callees {
            if visiting.contains(callee)
                && let Some(cycle_start) = stack.iter().position(|name| name == callee)
            {
                let mut cycle = stack[cycle_start..].to_vec();
                cycle.push(callee.clone());
                return Some(cycle);
            }
            if let Some(cycle) = detect_proof_cycle(callee, graph, visiting, visited, stack) {
                return Some(cycle);
            }
        }
    }
    stack.pop();
    visiting.remove(proof_name);
    visited.insert(proof_name.to_string());
    None
}

pub(super) struct ProofCallCollector<'a> {
    pub(super) proof_names: &'a HashSet<String>,
    pub(super) calls: HashSet<String>,
}

impl<'a> ProofCallCollector<'a> {
    pub(super) fn new(proof_names: &'a HashSet<String>) -> Self {
        Self {
            proof_names,
            calls: HashSet::new(),
        }
    }

    pub(super) fn into_calls(self) -> Vec<String> {
        self.calls.into_iter().collect()
    }

    pub(super) fn record_call(&mut self, module: Option<&Name>, func: &Name) {
        if module.is_some() {
            return;
        }

        let func = func.to_string();
        if self.proof_names.contains(&func) {
            self.calls.insert(func);
        }
    }

    pub(super) fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::OperatorSection { section } => {
                if let Some(left) = &section.left {
                    self.visit_expr(left);
                }
                if let Some(right) = &section.right {
                    self.visit_expr(right);
                }
            }
            Expr::Literal(_)
            | Expr::Variable { .. }
            | Expr::CheckObligation { .. }
            | Expr::Panic { .. } => {}
            Expr::Policy(policy) => self.visit_policy_expr(policy),
            Expr::FieldAccess { base, .. } | Expr::Unary { operand: base, .. } => {
                self.visit_expr(base);
            }
            Expr::IndexAccess { base, index, .. } => {
                self.visit_expr(base);
                self.visit_expr(index);
            }
            Expr::Binary { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            Expr::Call {
                func, module, args, ..
            } => {
                self.record_call(module.as_ref(), func);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            Expr::List { items, .. } => {
                for item in items {
                    self.visit_expr(item);
                }
            }
            Expr::Match {
                scrutinee, arms, ..
            } => {
                self.visit_expr(scrutinee);
                self.visit_match_arms(arms);
            }
            Expr::IfLet {
                expr,
                then_branch,
                else_branch,
                ..
            } => {
                self.visit_expr(expr);
                self.visit_expr(then_branch);
                self.visit_expr(else_branch);
            }
            Expr::Constructor {
                fields, payload, ..
            } => {
                for (_, field_expr) in fields {
                    self.visit_expr(field_expr);
                }
                match payload {
                    ConstructorPayload::Unit => {}
                    ConstructorPayload::Record(items) => {
                        for (_, item) in items {
                            self.visit_expr(item);
                        }
                    }
                    ConstructorPayload::Tuple(items) => {
                        for item in items {
                            self.visit_expr(item);
                        }
                    }
                }
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.visit_expr(condition);
                self.visit_expr(then_branch);
                if let Some(else_branch) = else_branch {
                    self.visit_expr(else_branch);
                }
            }
            Expr::Fail { payload, .. } => self.visit_expr(payload),
            Expr::WithError { body, arms, .. } => {
                self.visit_expr(body);
                self.visit_match_arms(arms);
            }
            Expr::Block {
                statements,
                tail_expr,
                ..
            } => {
                for statement in statements {
                    match statement {
                        BlockStmt::Let { expr, .. } => self.visit_expr(expr),
                    }
                }
                if let Some(tail_expr) = tail_expr {
                    self.visit_expr(tail_expr);
                }
            }
            Expr::FnDef { body, .. } => self.visit_expr(body),
            Expr::FnApply { func, args, .. } => {
                self.visit_expr(func);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            Expr::ActBlock { stmts, .. } => self.visit_act_stmts(stmts),
            Expr::DoBlock { stmts, .. } => self.visit_do_stmts(stmts),
            Expr::Comprehension {
                result, qualifiers, ..
            } => {
                self.visit_expr(result);
                for qualifier in qualifiers {
                    match qualifier {
                        ComprehensionQualifier::Bind { value, .. }
                        | ComprehensionQualifier::DiscardBind { value, .. }
                        | ComprehensionQualifier::Let { value, .. } => self.visit_expr(value),
                    }
                }
            }
        }
    }

    pub(super) fn visit_match_arms(&mut self, arms: &[MatchArm]) {
        for arm in arms {
            self.visit_expr(&arm.body);
        }
    }

    pub(super) fn visit_policy_expr(&mut self, policy: &ash_parser::surface::PolicyExpr) {
        match policy {
            ash_parser::surface::PolicyExpr::Var { .. } => {}
            ash_parser::surface::PolicyExpr::And(policies)
            | ash_parser::surface::PolicyExpr::Or(policies)
            | ash_parser::surface::PolicyExpr::Sequential(policies)
            | ash_parser::surface::PolicyExpr::Concurrent(policies) => {
                for policy in policies {
                    self.visit_policy_expr(policy);
                }
            }
            ash_parser::surface::PolicyExpr::Not(policy) => self.visit_policy_expr(policy),
            ash_parser::surface::PolicyExpr::Implies(left, right) => {
                self.visit_policy_expr(left);
                self.visit_policy_expr(right);
            }
            ash_parser::surface::PolicyExpr::ForAll { items, body, .. }
            | ash_parser::surface::PolicyExpr::Exists { items, body, .. } => {
                self.visit_expr(items);
                self.visit_policy_expr(body);
            }
            ash_parser::surface::PolicyExpr::MethodCall { receiver, args, .. } => {
                self.visit_policy_expr(receiver);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            ash_parser::surface::PolicyExpr::Call { args, .. } => {
                for arg in args {
                    self.visit_expr(arg);
                }
            }
        }
    }

    pub(super) fn visit_act_stmts(&mut self, stmts: &[ActStmt]) {
        for stmt in stmts {
            match stmt {
                ActStmt::Bind { value, .. } | ActStmt::Return { value, .. } => {
                    self.visit_expr(value);
                }
            }
        }
    }

    pub(super) fn visit_do_stmts(&mut self, stmts: &[DoStmt]) {
        for stmt in stmts {
            match stmt {
                DoStmt::Let { value, .. }
                | DoStmt::Bind { value, .. }
                | DoStmt::Return { value, .. } => self.visit_expr(value),
                DoStmt::WorkflowRequires { expr, .. } | DoStmt::WorkflowEnsures { expr, .. } => {
                    self.visit_expr(expr);
                }
            }
        }
    }
}

pub(super) struct ProofFuelChecker {
    pub(super) limit: usize,
    pub(super) remaining: usize,
    pub(super) exhausted: bool,
    pub(super) env: TypeEnv,
    pub(super) error: Option<TypeEnvError>,
}

impl ProofFuelChecker {
    pub(super) const fn new(limit: usize, env: TypeEnv) -> Self {
        Self {
            limit,
            remaining: limit,
            exhausted: false,
            env,
            error: None,
        }
    }

    pub(super) fn finish(self) -> Result<ProofTotalityResult, TypeEnvError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        Ok(ProofTotalityResult {
            status: if self.exhausted {
                ProofTotalityStatus::Untested(ProofTotalityUntestedReason::FuelExhausted)
            } else {
                ProofTotalityStatus::Checked
            },
            fuel_limit: self.limit,
            fuel_remaining: self.remaining,
        })
    }

    pub(super) fn consume(&mut self) -> bool {
        if self.error.is_some() || self.exhausted {
            return false;
        }
        if self.remaining == 0 {
            self.exhausted = true;
            return false;
        }
        self.remaining -= 1;
        true
    }

    pub(super) fn visit_expr(&mut self, expr: &Expr) {
        if !self.consume() {
            return;
        }
        match expr {
            Expr::OperatorSection { section } => {
                if let Some(left) = &section.left {
                    self.visit_expr(left);
                }
                if let Some(right) = &section.right {
                    self.visit_expr(right);
                }
            }
            Expr::Literal(_)
            | Expr::Variable { .. }
            | Expr::Policy(_)
            | Expr::CheckObligation { .. }
            | Expr::Panic { .. } => {}
            Expr::FieldAccess { base, .. } | Expr::Unary { operand: base, .. } => {
                self.visit_expr(base);
            }
            Expr::IndexAccess { base, index, .. } => {
                self.visit_expr(base);
                self.visit_expr(index);
            }
            Expr::Binary { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            Expr::Call { args, .. } | Expr::List { items: args, .. } => {
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            Expr::Match {
                scrutinee,
                arms,
                span,
            } => self.visit_match(scrutinee, arms, *span),
            Expr::IfLet {
                expr,
                then_branch,
                else_branch,
                ..
            } => {
                self.visit_expr(expr);
                self.visit_expr(then_branch);
                self.visit_expr(else_branch);
            }
            Expr::Constructor {
                fields, payload, ..
            } => {
                for (_, field_expr) in fields {
                    self.visit_expr(field_expr);
                }
                match payload {
                    ConstructorPayload::Unit => {}
                    ConstructorPayload::Record(items) => {
                        for (_, item) in items {
                            self.visit_expr(item);
                        }
                    }
                    ConstructorPayload::Tuple(items) => {
                        for item in items {
                            self.visit_expr(item);
                        }
                    }
                }
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.visit_expr(condition);
                self.visit_expr(then_branch);
                if let Some(else_branch) = else_branch {
                    self.visit_expr(else_branch);
                }
            }
            Expr::Fail { payload, .. } => self.visit_expr(payload),
            Expr::WithError { body, arms, .. } => {
                self.visit_expr(body);
                self.visit_match_arms(arms);
            }
            Expr::Block {
                statements,
                tail_expr,
                ..
            } => self.visit_block(statements, tail_expr.as_deref()),
            Expr::FnDef { params, body, .. } => self.visit_fn_body(params, body),
            Expr::FnApply { func, args, .. } => {
                self.visit_expr(func);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            Expr::ActBlock { stmts, .. } => self.visit_act_stmts(stmts),
            Expr::DoBlock { stmts, .. } => self.visit_do_stmts(stmts),
            Expr::Comprehension {
                result, qualifiers, ..
            } => {
                self.visit_expr(result);
                for qualifier in qualifiers {
                    match qualifier {
                        ComprehensionQualifier::Bind { value, .. }
                        | ComprehensionQualifier::DiscardBind { value, .. }
                        | ComprehensionQualifier::Let { value, .. } => self.visit_expr(value),
                    }
                }
            }
        }
    }

    pub(super) fn visit_block(&mut self, statements: &[BlockStmt], tail_expr: Option<&Expr>) {
        let mut block_env = self.env.clone();
        for statement in statements {
            match statement {
                BlockStmt::Let {
                    pattern,
                    expr,
                    span,
                } => {
                    self.visit_expr_with_env(expr, block_env.clone());
                    if self.error.is_some() || self.exhausted {
                        return;
                    }

                    let checked = crate::check_expr::check_expr(&block_env, expr);
                    if !checked.errors.is_empty() {
                        bind_untyped_pattern_for_totality(&mut block_env, pattern);
                        continue;
                    }
                    let expr_type = checked.substitution.apply(&checked.ty);
                    if let Err(error) = bind_pattern_for_type(&mut block_env, pattern, &expr_type) {
                        self.error = Some(TypeEnvError::InvalidDefinition(
                            format!("proof block let pattern type error: {error}"),
                            *span,
                        ));
                        return;
                    }
                }
            }
        }

        if let Some(tail_expr) = tail_expr {
            self.visit_expr_with_env(tail_expr, block_env);
        }
    }

    pub(super) fn visit_fn_body(&mut self, params: &[(Name, Option<Name>)], body: &Expr) {
        let mut fn_env = self.env.clone();
        let param_mapping = std::collections::HashMap::new();
        for (name, ty_ann) in params {
            let ty = ty_ann
                .as_ref()
                .and_then(|ann| {
                    crate::type_env::surface_type_to_type(
                        &SurfaceType::Name(ann.clone()),
                        &param_mapping,
                        &fn_env,
                    )
                    .ok()
                })
                .unwrap_or_else(|| Type::Var(TypeVar::fresh()));
            fn_env.bind_variable(name.as_ref(), ty);
        }
        self.visit_expr_with_env(body, fn_env);
    }

    pub(super) fn visit_match(&mut self, scrutinee: &Expr, arms: &[MatchArm], span: Span) {
        if proof_match_has_universal_arm(arms) {
            self.visit_expr(scrutinee);
            if self.error.is_some() || self.exhausted {
                return;
            }

            let checked = crate::check_expr::check_expr(&self.env, scrutinee);
            if checked.errors.is_empty() {
                let scrutinee_type = checked.substitution.apply(&checked.ty);
                self.visit_match_arms_with_scrutinee(arms, &scrutinee_type, span);
            } else {
                self.visit_untyped_match_arms(arms);
            }
            return;
        }

        self.visit_expr(scrutinee);
        if self.error.is_some() || self.exhausted {
            return;
        }

        let checked = crate::check_expr::check_expr(&self.env, scrutinee);
        if !checked.errors.is_empty() {
            self.error = Some(TypeEnvError::InvalidDefinition(
                format!(
                    "proof match scrutinee type could not be resolved: {:?}",
                    checked.errors
                ),
                span,
            ));
            return;
        }
        let scrutinee_type = checked.substitution.apply(&checked.ty);
        let patterns = arms
            .iter()
            .map(|arm| lower_pattern(&arm.pattern))
            .collect::<Result<Vec<CorePattern>, _>>();
        let Ok(patterns) = patterns else {
            self.error = Some(TypeEnvError::InvalidDefinition(
                "proof match contains a pattern that cannot be lowered".to_string(),
                span,
            ));
            return;
        };
        match check_match_exhaustive(&self.env, &patterns, &scrutinee_type) {
            MatchCoverage::Covered => {}
            MatchCoverage::Missing(missing) => {
                self.error = Some(TypeEnvError::InvalidDefinition(
                    format!(
                        "non-exhaustive proof match for {scrutinee_type}; missing {}; add a `_` catch-all or cover every constructor",
                        format_core_patterns(&missing)
                    ),
                    span,
                ));
                return;
            }
            MatchCoverage::Blocked {
                source_type,
                reason,
            } => {
                if proof_match_constructor_names_cover_type(&self.env, arms, &source_type) {
                    self.visit_match_arms_with_scrutinee(arms, &scrutinee_type, span);
                    return;
                }
                self.error = Some(TypeEnvError::InvalidDefinition(
                    format!(
                        "proof match exhaustiveness is blocked for {source_type}: {reason:?}; add a `_` catch-all"
                    ),
                    span,
                ));
                return;
            }
            MatchCoverage::Unsupported {
                scrutinee_type,
                reason,
            } => {
                self.error = Some(TypeEnvError::InvalidDefinition(
                    format!(
                        "proof match exhaustiveness unsupported for {scrutinee_type}: {reason}; add a `_` catch-all"
                    ),
                    span,
                ));
                return;
            }
        }

        self.visit_match_arms_with_scrutinee(arms, &scrutinee_type, span);
    }

    pub(super) fn visit_match_arms_with_scrutinee(
        &mut self,
        arms: &[MatchArm],
        scrutinee_type: &Type,
        span: Span,
    ) {
        let pattern_env = crate::check_expr::pattern_type_env_from_type_env(&self.env);
        let canonical_scrutinee = match self.env.canonicalize_type_for_pattern(scrutinee_type) {
            PatternCanonicalization::Matchable(canonical) => Some(canonical),
            PatternCanonicalization::Blocked { .. } => None,
        };

        for arm in arms {
            let bindings = match canonical_scrutinee.as_ref() {
                Some(canonical) => crate::check_pattern::check_pattern_with_canonical_type(
                    &pattern_env,
                    &arm.pattern,
                    canonical,
                ),
                None => {
                    crate::check_pattern::check_pattern(&pattern_env, &arm.pattern, scrutinee_type)
                }
            };

            let mut arm_env = self.env.clone();
            match bindings {
                Ok(bindings) => {
                    for (name, ty) in bindings {
                        arm_env.bind_variable(&name, ty);
                    }
                }
                Err(error) => {
                    self.error = Some(TypeEnvError::InvalidDefinition(
                        format!("proof match arm pattern type error: {error}"),
                        span,
                    ));
                    return;
                }
            }

            self.visit_expr_with_env(&arm.body, arm_env);
            if self.error.is_some() || self.exhausted {
                return;
            }
        }
    }

    pub(super) fn visit_expr_with_env(&mut self, expr: &Expr, env: TypeEnv) {
        let outer_env = std::mem::replace(&mut self.env, env);
        self.visit_expr(expr);
        self.env = outer_env;
    }

    pub(super) fn visit_untyped_match_arms(&mut self, arms: &[MatchArm]) {
        for arm in arms {
            let mut arm_env = self.env.clone();
            if let Pattern::Variable { name, .. } = &arm.pattern
                && name.as_ref() != "_"
            {
                arm_env.bind_variable(name.as_ref(), Type::Var(TypeVar::fresh()));
            }
            self.visit_expr_with_env(&arm.body, arm_env);
            if self.error.is_some() || self.exhausted {
                return;
            }
        }
    }

    pub(super) fn visit_match_arms(&mut self, arms: &[MatchArm]) {
        for arm in arms {
            self.visit_expr(&arm.body);
        }
    }

    pub(super) fn visit_act_stmts(&mut self, stmts: &[ActStmt]) {
        for stmt in stmts {
            match stmt {
                ActStmt::Bind { value, .. } | ActStmt::Return { value, .. } => {
                    self.visit_expr(value);
                }
            }
        }
    }

    pub(super) fn visit_do_stmts(&mut self, stmts: &[DoStmt]) {
        for stmt in stmts {
            match stmt {
                DoStmt::Let { value, .. }
                | DoStmt::Bind { value, .. }
                | DoStmt::Return { value, .. } => self.visit_expr(value),
                DoStmt::WorkflowRequires { expr, .. } | DoStmt::WorkflowEnsures { expr, .. } => {
                    self.visit_expr(expr);
                }
            }
        }
    }
}

pub(super) fn proof_match_constructor_names_cover_type(
    env: &TypeEnv,
    arms: &[MatchArm],
    scrutinee_type: &Type,
) -> bool {
    let Type::Constructor { name, .. } = scrutinee_type else {
        return false;
    };
    let type_name = name.to_string();
    let Some(TypeInfo::Enum { variants, .. }) = env.type_info.get(type_name.as_str()) else {
        return false;
    };

    let mut covered = std::collections::HashSet::new();
    for arm in arms {
        match &arm.pattern {
            Pattern::Wildcard | Pattern::Variable { .. } => return true,
            Pattern::Variant { name, payload, .. }
                if variant_payload_pattern_is_untyped_irrefutable(payload) =>
            {
                covered.insert(name.to_string());
            }
            _ => {}
        }
    }

    variants
        .iter()
        .all(|variant| covered.contains(&variant.name))
}

pub(super) fn variant_payload_pattern_is_untyped_irrefutable(
    payload: &ash_parser::surface::VariantPatternPayload,
) -> bool {
    match payload {
        ash_parser::surface::VariantPatternPayload::Unit => true,
        ash_parser::surface::VariantPatternPayload::Record(fields) => fields
            .iter()
            .all(|(_, pattern)| pattern_is_untyped_irrefutable(pattern)),
        ash_parser::surface::VariantPatternPayload::Tuple(items) => {
            items.iter().all(pattern_is_untyped_irrefutable)
        }
    }
}

pub(super) fn pattern_is_untyped_irrefutable(pattern: &Pattern) -> bool {
    match pattern {
        Pattern::Wildcard | Pattern::Variable { .. } => true,
        Pattern::Tuple(items) => items.iter().all(pattern_is_untyped_irrefutable),
        Pattern::Record(fields) => fields
            .iter()
            .all(|(_, pattern)| pattern_is_untyped_irrefutable(pattern)),
        Pattern::List { .. } | Pattern::Literal(_) | Pattern::Variant { .. } => false,
    }
}

pub(super) fn lower_proof_param_type(surface: &SurfaceType, env: &TypeEnv) -> Type {
    let param_mapping = std::collections::HashMap::new();
    if let Ok(ty) = surface_type_to_type(surface, &param_mapping, env) {
        return ty;
    }

    match surface {
        SurfaceType::Name(_) | SurfaceType::Hole { .. } | SurfaceType::Capability(_) => {
            Type::Var(TypeVar::fresh())
        }
        SurfaceType::List(item) => Type::List(Box::new(lower_proof_param_type(item, env))),
        SurfaceType::Tuple(items) => Type::Constructor {
            name: QualifiedName::root("Tuple"),
            args: items
                .iter()
                .map(|item| lower_proof_param_type(item, env))
                .collect(),
            kind: Kind::Type,
        },
        SurfaceType::Record(fields) => Type::Record(
            fields
                .iter()
                .map(|(name, ty)| {
                    (
                        Box::<str>::from(name.as_ref()),
                        lower_proof_param_type(ty, env),
                    )
                })
                .collect(),
        ),
        SurfaceType::Constructor { name, args } => Type::Constructor {
            name: QualifiedName::root(name.to_string()),
            args: args
                .iter()
                .map(|arg| lower_proof_param_type(arg, env))
                .collect(),
            kind: Kind::Type,
        },
        SurfaceType::Associated { .. } | SurfaceType::AssociatedFamilyProjection { .. } => {
            Type::Var(TypeVar::fresh())
        }
        SurfaceType::Fn(params, ret) => Type::Fn(
            params
                .iter()
                .map(|param| lower_proof_param_type(param, env))
                .collect(),
            Box::new(lower_proof_param_type(ret, env)),
        ),
    }
}

pub(super) fn proof_match_has_universal_arm(arms: &[MatchArm]) -> bool {
    arms.iter()
        .any(|arm| matches!(arm.pattern, Pattern::Wildcard | Pattern::Variable { .. }))
}

pub(super) fn bind_untyped_pattern_for_totality(env: &mut TypeEnv, pattern: &Pattern) {
    match pattern {
        Pattern::Variable { name, .. } if name.as_ref() != "_" => {
            env.bind_variable(name.as_ref(), Type::Var(TypeVar::fresh()));
        }
        Pattern::Tuple(patterns) => {
            for pattern in patterns {
                bind_untyped_pattern_for_totality(env, pattern);
            }
        }
        Pattern::Record(fields)
        | Pattern::Variant {
            fields: Some(fields),
            ..
        } => {
            for (_, pattern) in fields {
                bind_untyped_pattern_for_totality(env, pattern);
            }
        }
        Pattern::List { elements, rest } => {
            for pattern in elements {
                bind_untyped_pattern_for_totality(env, pattern);
            }
            if let Some(rest) = rest {
                env.bind_variable(rest.as_ref(), Type::Var(TypeVar::fresh()));
            }
        }
        Pattern::Literal(_) | Pattern::Wildcard | Pattern::Variant { fields: None, .. } => {}
        Pattern::Variable { .. } => {}
    }
}

pub(super) fn bind_pattern_for_type(
    env: &mut TypeEnv,
    pattern: &Pattern,
    ty: &Type,
) -> Result<(), TypeError> {
    let pattern_env = crate::check_expr::pattern_type_env_from_type_env(env);
    let bindings = match env.canonicalize_type_for_pattern(ty) {
        PatternCanonicalization::Matchable(canonical) => {
            crate::check_pattern::check_pattern_with_canonical_type(
                &pattern_env,
                pattern,
                &canonical,
            )
        }
        PatternCanonicalization::Blocked { .. } => {
            crate::check_pattern::check_pattern(&pattern_env, pattern, ty)
        }
    }?;
    for (name, ty) in bindings {
        env.bind_variable(&name, ty);
    }
    Ok(())
}

pub(super) fn format_core_patterns(patterns: &[CorePattern]) -> String {
    patterns
        .iter()
        .map(|pattern| format!("{pattern:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}
