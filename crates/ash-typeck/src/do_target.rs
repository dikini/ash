//! Generalized do-notation target resolution substrate.
//!
//! This module intentionally resolves only the computation target and hidden
//! sequencing dictionary. Statement typing and elaboration are deferred to the
//! typed do-block implementation task.

#![allow(clippy::result_large_err)]

use crate::error::{ConstructorError, TypeEnvError};
use crate::type_env::{ImplScheme, InterfaceEvidenceArg};
use crate::{Kind, PartialConstructorElaborationError, QualifiedName, TypeEnv};
use ash_core::ast::Expr as CoreExpr;
use ash_core::type_ir::{
    CanonicalTypeExpr, PartialTypeArg, TypeConstructorExpr, TypeConstructorHeadId,
};
use ash_parser::surface::{DoTarget, Type as SurfaceType};

/// Selected `Monad<K>` evidence snapshot carried through typed do elaboration.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedDoEvidence {
    pub target: QualifiedName,
    pub value_constructor: QualifiedName,
    pub return_op: SelectedDoOperation,
    pub bind_op: SelectedDoOperation,
}

/// Selected operation identity, method body, or intrinsic shim for do lowering.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectedDoOperation {
    HiddenActReturn,
    HiddenActBind,
    Ordinary(QualifiedName),
    EvidenceMethod {
        evidence_key: String,
        method: String,
        params: Vec<String>,
        body: CoreExpr,
    },
    EvidenceIntrinsic {
        evidence_key: String,
        method: String,
        shim: QualifiedName,
    },
    EvidenceUnavailable {
        evidence_key: String,
        method: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DoTowerLevel {
    Effectful,
    Proc,
    Workflow,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DoDictionaryOp {
    HiddenActReturn,
    HiddenActBind,
    Ordinary(QualifiedName),
    EvidenceMethod {
        evidence: DoEvidenceIdentity,
        method: String,
        params: Vec<String>,
        body: CoreExpr,
    },
    EvidenceIntrinsic {
        evidence: DoEvidenceIdentity,
        method: String,
        shim: QualifiedName,
    },
    EvidenceUnavailable {
        evidence: DoEvidenceIdentity,
        method: String,
        span: ash_parser::token::Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct DoEvidenceIdentity {
    interface: String,
    head_args: Vec<InterfaceEvidenceArg>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DoDictionary {
    pub(crate) target: QualifiedName,
    pub(crate) value_constructor: QualifiedName,
    pub(crate) target_args: Vec<SurfaceType>,
    pub(crate) return_op: DoDictionaryOp,
    pub(crate) bind_op: DoDictionaryOp,
    pub(crate) tower_level: DoTowerLevel,
}

impl DoDictionary {
    pub(crate) fn selected_evidence(&self) -> SelectedDoEvidence {
        SelectedDoEvidence {
            target: self.target.clone(),
            value_constructor: self.value_constructor.clone(),
            return_op: self.return_op.selected_operation(),
            bind_op: self.bind_op.selected_operation(),
        }
    }
}

impl DoDictionaryOp {
    pub(crate) fn selected_operation(&self) -> SelectedDoOperation {
        match self {
            DoDictionaryOp::HiddenActReturn => SelectedDoOperation::HiddenActReturn,
            DoDictionaryOp::HiddenActBind => SelectedDoOperation::HiddenActBind,
            DoDictionaryOp::Ordinary(name) => SelectedDoOperation::Ordinary(name.clone()),
            DoDictionaryOp::EvidenceMethod {
                evidence,
                method,
                params,
                body,
            } => SelectedDoOperation::EvidenceMethod {
                evidence_key: evidence.diagnostic_key(),
                method: method.clone(),
                params: params.clone(),
                body: body.clone(),
            },
            DoDictionaryOp::EvidenceIntrinsic {
                evidence,
                method,
                shim,
            } => SelectedDoOperation::EvidenceIntrinsic {
                evidence_key: evidence.diagnostic_key(),
                method: method.clone(),
                shim: shim.clone(),
            },
            DoDictionaryOp::EvidenceUnavailable {
                evidence, method, ..
            } => SelectedDoOperation::EvidenceUnavailable {
                evidence_key: evidence.diagnostic_key(),
                method: method.clone(),
            },
        }
    }
}

impl DoEvidenceIdentity {
    fn from_impl(evidence: &ImplScheme) -> Self {
        Self {
            interface: evidence.interface.clone(),
            head_args: evidence.head_args.clone(),
        }
    }

    pub(crate) fn diagnostic_key(&self) -> String {
        render_evidence_key(&self.interface, &self.head_args)
    }
}

/// Resolve a surface `do:K` target to a sequencing dictionary.
///
/// Compiler-known `Act`, `Proc`, and `Workflow` targets keep their hidden bridge
/// dictionaries during the migration. Other well-shaped unary targets must have
/// explicit `Monad<K>` evidence in the [`TypeEnv`].
pub(crate) fn resolve_do_target(
    env: &TypeEnv,
    target: &DoTarget,
) -> Result<DoDictionary, ConstructorError> {
    let target_name = target.name.as_ref();
    let surface_target = surface_target_type(target);

    if !target.args.is_empty() {
        env.elaborate_do_target_constructor_expr(&surface_target)
            .map_err(|err| do_target_shape_error(err, target.span))?;

        return resolve_monad_evidence_dictionary(env, target, &surface_target);
    }

    let (qualified, type_info) =
        env.resolve_type(target_name)
            .map_err(|_| ConstructorError::UnsupportedExpression {
                kind: format!(
                    "unknown do target '{target_name}'; use a registered computation constructor such as Act, Proc, or Workflow"
                ),
                span: target.span,
            })?;

    if qualified.name == "Result"
        && let Err(err) = env.elaborate_do_target_constructor_expr(&surface_target)
    {
        return Err(do_target_shape_error(err, target.span));
    }

    let arity = type_info
        .map(crate::type_env::TypeInfo::type_arg_count)
        .or_else(|| {
            env.lookup_type(&qualified.name)
                .map(|type_def| type_def.params.len())
        })
        .unwrap_or(0);
    let kind = Kind::n_ary(arity);
    let expected = Kind::n_ary(1);

    if kind != expected {
        return Err(ConstructorError::UnsupportedExpression {
            kind: format!(
                "do target {} has kind {kind}, expected {expected}; use a computation constructor such as Act, Proc, or Workflow",
                qualified.display()
            ),
            span: target.span,
        });
    }

    match qualified.name.as_str() {
        "Act" => Ok(DoDictionary {
            target: qualified.clone(),
            value_constructor: qualified,
            target_args: target.args.clone(),
            return_op: DoDictionaryOp::HiddenActReturn,
            bind_op: DoDictionaryOp::HiddenActBind,
            tower_level: DoTowerLevel::Effectful,
        }),
        "Proc" => Ok(DoDictionary {
            target: qualified.clone(),
            value_constructor: qualified,
            target_args: target.args.clone(),
            return_op: DoDictionaryOp::Ordinary(QualifiedName::qualified(
                vec!["proc".to_string()],
                "unit",
            )),
            bind_op: DoDictionaryOp::Ordinary(QualifiedName::qualified(
                vec!["proc".to_string()],
                "bind",
            )),
            tower_level: DoTowerLevel::Proc,
        }),
        "Workflow" => Ok(DoDictionary {
            target: qualified.clone(),
            value_constructor: qualified,
            target_args: target.args.clone(),
            return_op: DoDictionaryOp::Ordinary(QualifiedName::qualified(
                vec!["workflow".to_string()],
                "unit",
            )),
            bind_op: DoDictionaryOp::Ordinary(QualifiedName::qualified(
                vec!["workflow".to_string()],
                "bind",
            )),
            tower_level: DoTowerLevel::Workflow,
        }),
        "Result" => unreachable!("Result is rejected before MVP dictionary selection"),
        _ => resolve_monad_evidence_dictionary(env, target, &surface_target),
    }
}

fn surface_target_type(target: &DoTarget) -> SurfaceType {
    if target.args.is_empty() {
        SurfaceType::Name(target.name.to_string().into())
    } else {
        SurfaceType::Constructor {
            name: target.name.to_string().into(),
            args: target.args.clone(),
        }
    }
}

fn resolve_monad_evidence_dictionary(
    env: &TypeEnv,
    target: &DoTarget,
    surface_target: &SurfaceType,
) -> Result<DoDictionary, ConstructorError> {
    let evidence =
        match env.resolve_interface_evidence("Monad", std::slice::from_ref(surface_target)) {
            Ok(evidence) => evidence,
            Err(err) => resolve_partial_result_monad_evidence(env, surface_target)
                .ok_or_else(|| missing_monad_evidence_error(surface_target, err, target.span))?,
        };
    let evidence_identity = DoEvidenceIdentity::from_impl(evidence);

    Ok(DoDictionary {
        target: QualifiedName::root(target.name.to_string()),
        value_constructor: QualifiedName::root(target.name.to_string()),
        target_args: target.args.clone(),
        return_op: selected_monad_op(evidence, &evidence_identity, "return", target.span)?,
        bind_op: selected_monad_op(evidence, &evidence_identity, "bind", target.span)?,
        tower_level: DoTowerLevel::Effectful,
    })
}

fn resolve_partial_result_monad_evidence<'a>(
    env: &'a TypeEnv,
    surface_target: &SurfaceType,
) -> Option<&'a ImplScheme> {
    if !is_result_partial_surface_target(surface_target) {
        return None;
    }

    env.impl_schemes().iter().find(|scheme| {
        scheme.interface == "Monad"
            && scheme
                .head_args
                .iter()
                .any(|arg| result_constructor_evidence_matches_surface_target(arg, surface_target))
    })
}

fn result_constructor_evidence_matches_surface_target(
    arg: &InterfaceEvidenceArg,
    surface_target: &SurfaceType,
) -> bool {
    let SurfaceType::Constructor { name, args } = surface_target else {
        return false;
    };
    if name.as_ref() != "Result" {
        return false;
    }

    match arg {
        InterfaceEvidenceArg::Constructor(expr) => match expr.as_ref() {
            TypeConstructorExpr::PartialApplication(app) => {
                type_constructor_head_name(&app.head).is_some_and(|head| head == "Result")
                    && app.args.len() == args.len()
                    && app
                        .args
                        .iter()
                        .zip(args)
                        .all(|(evidence_arg, surface_arg)| {
                            partial_type_arg_matches_surface_type(evidence_arg, surface_arg)
                        })
            }
            _ => false,
        },
        _ => false,
    }
}

fn partial_type_arg_matches_surface_type(
    evidence_arg: &PartialTypeArg,
    surface_arg: &SurfaceType,
) -> bool {
    match (evidence_arg, surface_arg) {
        (PartialTypeArg::Hole(_), SurfaceType::Hole { .. }) => true,
        (PartialTypeArg::Applied(evidence_ty), surface_ty) => {
            canonical_type_expr_matches_surface_type(evidence_ty, surface_ty)
        }
        _ => false,
    }
}

fn canonical_type_expr_matches_surface_type(
    evidence_ty: &CanonicalTypeExpr,
    surface_ty: &SurfaceType,
) -> bool {
    match (evidence_ty, surface_ty) {
        (CanonicalTypeExpr::Primitive(evidence), SurfaceType::Name(surface))
        | (CanonicalTypeExpr::Var(evidence), SurfaceType::Name(surface)) => {
            evidence == surface.as_ref()
        }
        (
            CanonicalTypeExpr::NominalApp {
                visible_name, args, ..
            },
            SurfaceType::Name(surface),
        ) => args.is_empty() && visible_name == surface.as_ref(),
        (
            CanonicalTypeExpr::NominalApp {
                visible_name,
                args: evidence_args,
                ..
            },
            SurfaceType::Constructor { name, args },
        ) => {
            visible_name == name.as_ref()
                && evidence_args.len() == args.len()
                && evidence_args
                    .iter()
                    .zip(args)
                    .all(|(evidence_arg, surface_arg)| {
                        canonical_type_expr_matches_surface_type(evidence_arg, surface_arg)
                    })
        }
        _ => false,
    }
}

fn is_result_partial_surface_target(surface_target: &SurfaceType) -> bool {
    let SurfaceType::Constructor { name, args } = surface_target else {
        return false;
    };
    name.as_ref() == "Result"
        && args.len() == 2
        && args
            .iter()
            .filter(|arg| matches!(arg, SurfaceType::Hole { .. }))
            .count()
            == 1
}

fn selected_monad_op(
    evidence: &ImplScheme,
    evidence_identity: &DoEvidenceIdentity,
    method: &str,
    span: ash_parser::token::Span,
) -> Result<DoDictionaryOp, ConstructorError> {
    if let Some(method_info) = evidence.methods.iter().find(|info| info.name == method) {
        return Ok(DoDictionaryOp::EvidenceMethod {
            evidence: evidence_identity.clone(),
            method: method.to_string(),
            params: method_info.param_names.clone(),
            body: method_info.body.clone(),
        });
    }

    match intrinsic_monad_shim(evidence, method) {
        Some(shim) => Ok(DoDictionaryOp::EvidenceIntrinsic {
            evidence: evidence_identity.clone(),
            method: method.to_string(),
            shim,
        }),
        None => Ok(DoDictionaryOp::EvidenceUnavailable {
            evidence: evidence_identity.clone(),
            method: method.to_string(),
            span,
        }),
    }
}

fn intrinsic_monad_shim(evidence: &ImplScheme, method: &str) -> Option<QualifiedName> {
    let is_result = evidence
        .head_args
        .iter()
        .any(is_result_constructor_evidence);
    match (is_result, method) {
        (true, "return") => Some(QualifiedName::root("Ok".to_string())),
        (true, "bind") => Some(QualifiedName::qualified(
            vec!["result".to_string()],
            "and_then",
        )),
        _ => None,
    }
}

fn is_result_constructor_evidence(arg: &InterfaceEvidenceArg) -> bool {
    match arg {
        InterfaceEvidenceArg::Constructor(expr) => {
            type_constructor_expr_head_name(expr).is_some_and(|name| name == "Result")
        }
        InterfaceEvidenceArg::Proper(crate::types::Type::Constructor { name, .. }) => {
            name.name == "Result"
        }
        InterfaceEvidenceArg::Proper(_) => false,
    }
}

fn type_constructor_expr_head_name(expr: &TypeConstructorExpr) -> Option<&str> {
    match expr {
        TypeConstructorExpr::ConstructorHead(head) => type_constructor_head_name(head),
        TypeConstructorExpr::PartialApplication(app) => type_constructor_head_name(&app.head),
        TypeConstructorExpr::ProperType(CanonicalTypeExpr::NominalApp { visible_name, .. }) => {
            Some(visible_name.as_str())
        }
        _ => None,
    }
}

fn type_constructor_head_name(head: &TypeConstructorHeadId) -> Option<&str> {
    match head {
        TypeConstructorHeadId::Nominal { visible_name, .. } => Some(visible_name.as_str()),
        TypeConstructorHeadId::Computation(head) => Some(head.name.as_str()),
        _ => None,
    }
}

fn render_evidence_key(interface: &str, head_args: &[InterfaceEvidenceArg]) -> String {
    format!(
        "{}<{}>",
        interface,
        head_args
            .iter()
            .map(render_evidence_arg)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_evidence_arg(arg: &InterfaceEvidenceArg) -> String {
    match arg {
        InterfaceEvidenceArg::Proper(ty) => ty.to_string(),
        InterfaceEvidenceArg::Constructor(expr) => render_type_constructor_expr(expr),
    }
}

fn render_type_constructor_expr(expr: &TypeConstructorExpr) -> String {
    match expr {
        TypeConstructorExpr::ProperType(ty) => render_canonical_type_expr(ty),
        TypeConstructorExpr::ConstructorHead(head) => render_type_constructor_head(head),
        TypeConstructorExpr::PartialApplication(app) => format!(
            "{}<{}>",
            render_type_constructor_head(&app.head),
            app.args
                .iter()
                .map(render_partial_type_arg)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => "<unsupported-type-constructor-expr>".to_string(),
    }
}

fn render_type_constructor_head(head: &TypeConstructorHeadId) -> String {
    match head {
        TypeConstructorHeadId::Nominal { visible_name, .. } => visible_name.clone(),
        TypeConstructorHeadId::Computation(head) => head.name.clone(),
        _ => "<unsupported-type-constructor-head>".to_string(),
    }
}

fn render_partial_type_arg(arg: &PartialTypeArg) -> String {
    match arg {
        PartialTypeArg::Applied(ty) => render_canonical_type_expr(ty),
        PartialTypeArg::Hole(_) => "_".to_string(),
        _ => "<unsupported-partial-type-arg>".to_string(),
    }
}

fn render_canonical_type_expr(ty: &CanonicalTypeExpr) -> String {
    match ty {
        CanonicalTypeExpr::Primitive(name) | CanonicalTypeExpr::Var(name) => name.clone(),
        CanonicalTypeExpr::NominalApp {
            visible_name, args, ..
        } => {
            if args.is_empty() {
                visible_name.clone()
            } else {
                format!(
                    "{}<{}>",
                    visible_name,
                    args.iter()
                        .map(render_canonical_type_expr)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        other => format!("{other:?}"),
    }
}

fn missing_monad_evidence_error(
    surface_target: &SurfaceType,
    err: TypeEnvError,
    span: ash_parser::token::Span,
) -> ConstructorError {
    let target = render_surface_type(surface_target);
    let evidence = format!("Monad<{target}>");
    let detail = match err {
        TypeEnvError::MissingImpl { .. } | TypeEnvError::MissingInterface(_, _) => {
            "explicit evidence was not found".to_string()
        }
        other => other.to_string(),
    };

    ConstructorError::UnsupportedExpression {
        kind: format!(
            "missing Monad evidence for do target {target}; required SPEC-067 Monad<K> evidence {evidence}: {detail}"
        ),
        span,
    }
}

fn do_target_shape_error(
    err: PartialConstructorElaborationError,
    fallback_span: ash_parser::token::Span,
) -> ConstructorError {
    let (kind, span) = match err {
        PartialConstructorElaborationError::BareHigherArityConstructor {
            constructor,
            arity,
            hint,
            span,
        } => (
            format!(
                "wrong target shape for do target {constructor}: bare constructor has arity {arity}; write {hint} with an explicit `_` hole"
            ),
            span,
        ),
        PartialConstructorElaborationError::MultipleHoles {
            constructor,
            count,
            span,
        } => (
            format!(
                "multiple type holes in do target {constructor}: found {count}; the MVP accepts exactly one value-position hole"
            ),
            span,
        ),
        PartialConstructorElaborationError::UnsupportedHolePosition { reason, span } => {
            (format!("unsupported do target shape: {reason}"), span)
        }
        PartialConstructorElaborationError::NoInversionBoundary { context, span } => (
            format!(
                "unsupported non-inverting do target shape: cannot elaborate type hole by inverting {context}"
            ),
            span,
        ),
        PartialConstructorElaborationError::MissingHole { constructor, span } => (
            format!(
                "wrong target shape for do target {constructor}: expected exactly one explicit `_` hole"
            ),
            span,
        ),
        PartialConstructorElaborationError::WrongArity {
            constructor,
            expected_arity,
            found_arity,
            span,
        } => (
            format!(
                "wrong target shape for do target {constructor}: expected {expected_arity} type arguments, found {found_arity}"
            ),
            span,
        ),
        PartialConstructorElaborationError::UnknownConstructor { constructor, span } => {
            (format!("unknown do target '{constructor}'"), span)
        }
        PartialConstructorElaborationError::ArgumentLoweringFailed {
            constructor,
            reason,
            span,
        } => (
            format!("unsupported do target shape for {constructor}: {reason}"),
            span,
        ),
    };

    ConstructorError::UnsupportedExpression {
        kind,
        span: if span == ash_parser::token::Span::default() {
            fallback_span
        } else {
            span
        },
    }
}

fn render_surface_type(ty: &SurfaceType) -> String {
    match ty {
        SurfaceType::Name(name) => name.to_string(),
        SurfaceType::Hole { .. } => "_".to_string(),
        SurfaceType::List(item) => format!("[{}]", render_surface_type(item)),
        SurfaceType::Tuple(items) => format!(
            "({})",
            items
                .iter()
                .map(render_surface_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        SurfaceType::Record(fields) => format!(
            "{{{}}}",
            fields
                .iter()
                .map(|(name, ty)| format!("{name}: {}", render_surface_type(ty)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        SurfaceType::Capability(name) => format!("capability {name}"),
        SurfaceType::Constructor { name, args } => format!(
            "{}<{}>",
            name,
            args.iter()
                .map(render_surface_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        SurfaceType::Associated { base, name } => {
            format!("{}::{name}", render_surface_type(base))
        }
        SurfaceType::AssociatedFamilyProjection {
            interface,
            args,
            member,
            ..
        } => format!(
            "<{}<{}>>::{}",
            interface,
            args.iter()
                .map(render_surface_type)
                .collect::<Vec<_>>()
                .join(", "),
            member
        ),
        SurfaceType::Fn(params, ret) => format!(
            "Fn({}) -> {}",
            params
                .iter()
                .map(render_surface_type)
                .collect::<Vec<_>>()
                .join(", "),
            render_surface_type(ret)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::ast::{Name as CoreName, TypeBody, TypeDef, Visibility};
    use ash_parser::surface::{Name, Type};
    use ash_parser::token::Span;

    fn target(name: &str) -> DoTarget {
        DoTarget {
            name: Name::from(name),
            args: vec![],
            span: Span::default(),
        }
    }

    fn resolve(name: &str) -> Result<DoDictionary, ConstructorError> {
        resolve_do_target(&TypeEnv::with_builtin_types(), &target(name))
    }

    fn error_text(err: ConstructorError) -> String {
        match err {
            ConstructorError::UnsupportedExpression { kind, .. } => kind,
            other => other.to_string(),
        }
    }

    #[test]
    fn do_target_act_resolves_to_hidden_act_dictionary() {
        let dict = resolve("Act").expect("Act target should resolve");

        assert_eq!(dict.target, QualifiedName::root("Act"));
        assert_eq!(dict.value_constructor, QualifiedName::root("Act"));
        assert_eq!(dict.return_op, DoDictionaryOp::HiddenActReturn);
        assert_eq!(dict.bind_op, DoDictionaryOp::HiddenActBind);
        assert_eq!(dict.tower_level, DoTowerLevel::Effectful);
    }

    #[test]
    fn do_target_proc_resolves_to_hidden_proc_dictionary() {
        let dict = resolve("Proc").expect("Proc target should resolve");

        assert_eq!(dict.target, QualifiedName::root("Proc"));
        assert_eq!(dict.value_constructor, QualifiedName::root("Proc"));
        assert_eq!(
            dict.return_op,
            DoDictionaryOp::Ordinary(QualifiedName::qualified(vec!["proc".to_string()], "unit"))
        );
        assert_eq!(
            dict.bind_op,
            DoDictionaryOp::Ordinary(QualifiedName::qualified(vec!["proc".to_string()], "bind"))
        );
        assert_eq!(dict.tower_level, DoTowerLevel::Proc);
    }

    #[test]
    fn do_target_int_reports_wrong_kind_not_computation_constructor() {
        let message = error_text(resolve("Int").expect_err("Int is a proper type"));

        assert!(message.contains("do target Int has kind *"), "{message}");
        assert!(message.contains("expected * -> *"), "{message}");
        assert!(message.contains("Act, Proc, or Workflow"), "{message}");
    }

    #[test]
    fn do_target_missing_reports_unknown_target() {
        let message = error_text(resolve("Missing").expect_err("Missing target is unknown"));

        assert!(message.contains("unknown do target 'Missing'"), "{message}");
        assert!(message.contains("Act, Proc, or Workflow"), "{message}");
    }

    #[test]
    fn do_target_bare_result_reports_wrong_shape_with_hole_hint() {
        let message = error_text(resolve("Result").expect_err("Result is not an MVP dictionary"));

        assert!(message.contains("Result"), "{message}");
        assert!(message.contains("Result<_, E>"), "{message}");
        assert!(message.contains("wrong target shape"), "{message}");
        assert!(!message.contains("missing Monad evidence"), "{message}");
    }

    #[test]
    fn do_target_resolution_does_not_import_dictionary_ops_into_lexical_scope() {
        let env = TypeEnv::with_builtin_types();
        let _dict = resolve_do_target(&env, &target("Proc")).expect("Proc target should resolve");

        assert!(env.lookup_variable("bind").is_none());
        assert!(env.lookup_variable("unit").is_none());
        assert!(env.lookup_variable("proc::bind").is_some());
        assert!(env.lookup_variable("proc::unit").is_some());
    }

    #[test]
    fn do_target_with_partial_explicit_args_reaches_missing_monad_evidence() {
        let mut env = TypeEnv::new();
        for type_def in [
            TypeDef {
                name: CoreName::from("Result"),
                params: vec!["T".into(), "E".into()],
                body: TypeBody::Struct(vec![]),
                visibility: Visibility::Public,
                builtin: false,
            },
            TypeDef {
                name: CoreName::from("E"),
                params: vec![],
                body: TypeBody::Struct(vec![]),
                visibility: Visibility::Public,
                builtin: false,
            },
        ] {
            env.register_type(&type_def)
                .expect("register do-target fixture type");
        }
        let result_target = DoTarget {
            name: Name::from("Result"),
            args: vec![
                Type::Hole {
                    span: Span::default(),
                },
                Type::Name(Name::from("E")),
            ],
            span: Span::default(),
        };

        let message = error_text(
            resolve_do_target(&env, &result_target)
                .expect_err("Result<_, E> has shape but no Monad evidence"),
        );

        assert!(message.contains("Result<_, E>"), "{message}");
        assert!(message.contains("missing Monad evidence"), "{message}");
        assert!(!message.contains("wrong target shape"), "{message}");
    }

    #[test]
    fn do_target_with_wrong_explicit_arg_count_reports_shape_error() {
        let env = TypeEnv::with_builtin_types();
        let result_target = DoTarget {
            name: Name::from("Result"),
            args: vec![Type::Name(Name::from("Int"))],
            span: Span::default(),
        };

        let message = error_text(
            resolve_do_target(&env, &result_target)
                .expect_err("Result<Int> is the wrong do-target shape"),
        );

        assert!(message.contains("wrong target shape"), "{message}");
        assert!(message.contains("expected 2 type arguments"), "{message}");
        assert!(!message.contains("missing Monad evidence"), "{message}");
    }

    #[test]
    fn do_target_uses_ast_type_params_when_type_info_is_absent() {
        let mut env = TypeEnv::new();
        env.register_type_identity(&TypeDef {
            name: CoreName::from("Boxed"),
            params: vec!["A".into()],
            body: TypeBody::Struct(vec![]),
            visibility: Visibility::Public,
            builtin: false,
        })
        .expect("register generic type identity");

        env.remove_type_info_for_test("Boxed");

        let message = error_text(
            resolve_do_target(&env, &target("Boxed"))
                .expect_err("generic AST-only target has kind but no Monad evidence"),
        );
        assert!(message.contains("missing Monad evidence"), "{message}");
        assert!(message.contains("Monad<Boxed>"), "{message}");
        assert!(!message.contains("has kind *"), "{message}");
    }
}
