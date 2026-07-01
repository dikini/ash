//! Public callable, workflow, capability, and snippet export helpers.

use super::{
    CallableKind, CallableSignature, CoreTypeBody, CoreTypeDef, CoreTypeExpr, CoreVisibility,
    CoverageEvidence, Definition, EngineError, Expr, HashMap, HashSet, InlineCallable,
    OpenPostcondition, Parser, Path, ProcContractSummary, ProcFailureSummary, ProcLowerSummary,
    ProcProvenanceSummary, ProcResourceAuthoritySummary, ProjectionEvent, ProjectionEventKind,
    ProjectionKind, PublicWorkflowSummary, SourceOrigin, Type, Workflow, WorkflowBinder,
    WorkflowDef, WorkflowForm, WorkflowNodeId, WorkflowScope, convert_type_def,
    extract_semicolon_snippets, legacy_workflow_def_to_workflow_form, lower_workflow_form,
    new_input, parse_builtin_fn_definition, parse_fn_definition, parse_type_def, workflow_def,
};

pub(super) fn extract_public_capability_names(source: &str) -> Vec<String> {
    extract_semicolon_snippets(source, |trimmed| trimmed.starts_with("pub capability "))
        .into_iter()
        .filter_map(|snippet| {
            snippet
                .trim()
                .strip_prefix("pub capability ")
                .and_then(|rest| rest.split(':').next())
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(ToOwned::to_owned)
        })
        .collect()
}

pub(super) fn capability_type_identity(name: &str) -> CoreTypeDef {
    CoreTypeDef {
        name: name.to_string(),
        params: Vec::new(),
        body: CoreTypeBody::Struct(vec![]),
        visibility: CoreVisibility::Public,
        builtin: true,
    }
}

pub(super) fn parse_type_def_snippet(snippet: &str) -> Result<CoreTypeDef, EngineError> {
    if let Some(type_def) = parse_simple_type_alias_snippet(snippet) {
        return Ok(type_def);
    }

    let mut input = new_input(snippet.trim());
    let parsed = parse_type_def
        .parse_next(&mut input)
        .map_err(|error| EngineError::Parse(format!("{error}")))?;
    convert_type_def(&parsed)
}

fn parse_simple_type_alias_snippet(snippet: &str) -> Option<CoreTypeDef> {
    let trimmed = snippet.trim().strip_suffix(';')?.trim();
    let rest = trimmed.strip_prefix("pub type ")?.trim();
    let (name, target) = rest.split_once('=')?;
    let name = name.trim();
    let target = target.trim();

    let (name, params) = if let Some((base, params_text)) = name.split_once('<') {
        let params_text = params_text.strip_suffix('>')?;
        (
            base.trim(),
            params_text
                .split(',')
                .map(str::trim)
                .filter(|param| !param.is_empty())
                .map(ToOwned::to_owned)
                .collect(),
        )
    } else {
        (name, Vec::new())
    };

    if name.is_empty() || target.is_empty() || target.contains('{') || target.contains('|') {
        return None;
    }

    Some(CoreTypeDef {
        name: name.to_string(),
        params,
        body: CoreTypeBody::Alias(convert_simple_type_expr(target)?),
        visibility: CoreVisibility::Public,
        builtin: false,
    })
}

fn convert_simple_type_expr(text: &str) -> Option<CoreTypeExpr> {
    let text = text.trim();
    if let Some((name, args_text)) = text.split_once('<') {
        let args_text = args_text.strip_suffix('>')?;
        let args = args_text
            .split(',')
            .map(convert_simple_type_expr)
            .collect::<Option<Vec<_>>>()?;
        return Some(CoreTypeExpr::Constructor {
            name: name.trim().to_string(),
            args,
        });
    }

    if text
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == ':')
    {
        Some(CoreTypeExpr::Named(text.to_string()))
    } else {
        None
    }
}

pub(super) fn is_workflow_export_start(trimmed: &str) -> bool {
    starts_with_keyword(trimmed, "workflow") || starts_with_keyword(trimmed, "pub workflow")
}

fn starts_with_keyword(text: &str, keyword: &str) -> bool {
    text.strip_prefix(keyword).is_some_and(|rest| {
        rest.chars()
            .next()
            .is_some_and(|ch| ch.is_whitespace() || ch == '(')
    })
}

pub(super) fn parse_workflow_signature_callable(snippet: &str) -> Option<ImportedCallableExport> {
    let trimmed = snippet.trim();
    let rest = trimmed
        .strip_prefix("pub workflow ")
        .or_else(|| trimmed.strip_prefix("workflow "))?;
    let (name_text, after_name) = rest.split_once('(')?;
    let name = name_text.trim();
    if name.is_empty() {
        return None;
    }
    let (params_text, after_params) = split_balanced_prefix(after_name, '(', ')')?;
    let params = parse_workflow_signature_params(params_text)?;
    let return_type = parse_workflow_signature_return_type(after_params)?;
    let body = workflow_signature_expr_for_body_text(snippet);
    let fn_def = ash_parser::surface::FnDef {
        visibility: ash_parser::surface::Visibility::Public,
        name: name.into(),
        type_params: Vec::new(),
        params,
        return_type: Some(return_type),
        proposition_tail: None,
        contract: None,
        body: body.clone(),
        span: ash_parser::token::Span::default(),
    };

    Some(ImportedCallableExport {
        callable: InlineCallable {
            exported_name: name.to_string(),
            params: fn_def
                .params
                .iter()
                .map(|param| param.name.to_string())
                .collect(),
            effectful_names: HashSet::new(),
            kind: CallableKind::User { body },
            signature: Some(CallableSignature::Function(fn_def)),
            exporting_modules: HashSet::new(),
            workflow_summary: None,
            module_runtime_callables: HashMap::new(),
        },
    })
}

fn split_balanced_prefix(text: &str, open: char, close: char) -> Option<(&str, &str)> {
    let mut depth = 1usize;
    for (index, ch) in text.char_indices() {
        match ch {
            ch if ch == open => depth += 1,
            ch if ch == close => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some((&text[..index], &text[index + ch.len_utf8()..]));
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_workflow_signature_params(text: &str) -> Option<Vec<ash_parser::surface::Param>> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Some(Vec::new());
    }

    trimmed
        .split(',')
        .map(|param| {
            let (name, ty) = param.split_once(':')?;
            Some(ash_parser::surface::Param {
                name: name.trim().into(),
                ty: parse_workflow_signature_type(ty.trim())?,
            })
        })
        .collect()
}

fn parse_workflow_signature_return_type(text: &str) -> Option<ash_parser::surface::Type> {
    let after_arrow = text.trim_start().strip_prefix("->")?;
    let ty_text = after_arrow.split_once('{')?.0.trim();
    parse_workflow_signature_type(ty_text)
}

fn parse_workflow_signature_type(text: &str) -> Option<ash_parser::surface::Type> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if let Some(inner) = text.strip_prefix("cap ") {
        return Some(ash_parser::surface::Type::Capability(inner.trim().into()));
    }
    if let Some((name, args_text)) = text.split_once('<') {
        let args_text = args_text.strip_suffix('>')?;
        let args = split_top_level_commas(args_text)
            .into_iter()
            .map(parse_workflow_signature_type)
            .collect::<Option<Vec<_>>>()?;
        return Some(ash_parser::surface::Type::Constructor {
            name: name.trim().into(),
            args,
        });
    }
    Some(ash_parser::surface::Type::Name(text.into()))
}

fn split_top_level_commas(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, ch) in text.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(text[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(text[start..].trim());
    parts
}

const fn workflow_signature_expr_for_body_text(_snippet: &str) -> Expr {
    Expr::Literal(ash_parser::surface::Literal::Null)
}

pub(super) fn parse_workflow_callable(
    snippet: &str,
) -> Result<Option<ImportedCallableExport>, EngineError> {
    let trimmed = snippet.trim();
    let mut normalized = String::new();
    let workflow_source = trimmed
        .strip_prefix("pub workflow ")
        .map_or(trimmed, |rest| {
            normalized = format!("workflow {rest}");
            normalized.as_str()
        });
    let mut input = new_input(workflow_source);
    let parsed = workflow_def
        .parse_next(&mut input)
        .map_err(|error| EngineError::Parse(format!("{error}")))?;
    extract_callable_from_workflow(parsed)
}

pub(super) fn parse_pub_fn_callable(
    snippet: &str,
) -> Result<Option<ImportedCallableExport>, EngineError> {
    let mut input = new_input(snippet.trim());
    let parsed = parse_fn_definition
        .parse_next(&mut input)
        .map_err(|error| EngineError::Parse(format!("{error}")))?;

    let Definition::Function(function) = parsed else {
        return Err(EngineError::Parse(
            "expected pub fn to parse as a function definition".to_string(),
        ));
    };

    Ok(Some(imported_callable_from_fn_def(function)))
}

pub(super) fn imported_callable_from_fn_def(
    function: ash_parser::surface::FnDef,
) -> ImportedCallableExport {
    let name = function.name.to_string();
    let params = function
        .params
        .iter()
        .map(|param| param.name.to_string())
        .collect::<Vec<_>>();
    let workflow_summary = workflow_returning_pub_fn_summary(&function);
    let body = function.body.clone();

    ImportedCallableExport {
        callable: InlineCallable {
            exported_name: name,
            params,
            effectful_names: HashSet::new(),
            kind: CallableKind::User { body },
            signature: Some(CallableSignature::Function(function)),
            exporting_modules: HashSet::new(),
            workflow_summary,
            module_runtime_callables: HashMap::new(),
        },
    }
}

/// Conservative public-summary adapter for parser-only module export collection.
///
/// This intentionally recognizes only first-class `do:Workflow` expressions
/// whose public contract statements can be classified without typed lowering.
/// Unsupported shapes return `None` rather than inventing public workflow
/// metadata; full typed workflow expression lowering remains owned by typeck.
pub(super) fn workflow_returning_pub_fn_summary(
    function: &ash_parser::surface::FnDef,
) -> Option<PublicWorkflowSummary> {
    let return_type = function.return_type.as_ref()?;
    if !is_workflow_return_type(return_type) {
        return None;
    }

    let workflow_expr = workflow_summary_source_expr(&function.body)?;
    let form = workflow_expr_summary_form(workflow_expr, function.name.as_ref())?;
    let origin = SourceOrigin::ImportedSummary {
        module: String::new(),
        public_anchor: function.name.to_string(),
    };
    let mut lowered = lower_workflow_form(&form, origin.clone());
    for event in &mut lowered.projection_events {
        event.origin = origin.clone();
    }
    Some(PublicWorkflowSummary {
        node_count: lowered.projection_events.len(),
        projection_events: lowered.projection_events,
        coverage: lowered.coverage,
    })
}

fn workflow_summary_source_expr(expr: &Expr) -> Option<&Expr> {
    match expr {
        Expr::DoBlock { .. } => Some(expr),
        Expr::Block {
            statements,
            tail_expr: Some(tail_expr),
            ..
        } if statements.is_empty() => workflow_summary_source_expr(tail_expr),
        _ => None,
    }
}

fn workflow_expr_summary_form(expr: &Expr, anchor: &str) -> Option<WorkflowForm<()>> {
    match expr {
        Expr::DoBlock { target, stmts, .. } if target.name.as_ref() == "Workflow" => {
            let mut next_node = 1;
            let form = workflow_do_stmts_summary_form(stmts, &mut next_node, anchor)?;
            Some(WorkflowForm::Scope {
                node: WorkflowNodeId(next_node),
                scope: WorkflowScope {
                    name: Some(anchor.to_string()),
                    origin: first_class_workflow_source_origin(),
                },
                body: Box::new(form),
            })
        }
        _ => None,
    }
}

fn workflow_do_stmts_summary_form(
    stmts: &[ash_parser::surface::DoStmt],
    next_node: &mut u64,
    anchor: &str,
) -> Option<WorkflowForm<()>> {
    match stmts {
        [ash_parser::surface::DoStmt::Return { .. }] => Some(WorkflowForm::FromProc {
            node: workflow_summary_node(next_node),
            summary: first_class_body_proc_summary(anchor),
        }),
        [
            ash_parser::surface::DoStmt::WorkflowRequires { expr, .. },
            rest @ ..,
        ] => {
            let node = workflow_summary_node(next_node);
            let requirement =
                ash_parser::workflow_contract_classifier::classify_requirement(expr).ok()?;
            let source = WorkflowForm::Requires { node, requirement };
            let next = workflow_do_stmts_summary_form(rest, next_node, anchor)?;
            Some(WorkflowForm::Bind {
                node: workflow_summary_node(next_node),
                source: Box::new(source),
                binder: WorkflowBinder::Ignored,
                next: Box::new(next),
            })
        }
        [
            ash_parser::surface::DoStmt::WorkflowEnsures { expr, .. },
            rest @ ..,
        ] => {
            let node = workflow_summary_node(next_node);
            let postcondition =
                ash_parser::workflow_contract_classifier::classify_postcondition(expr).ok()?;
            let source = WorkflowForm::Ensures {
                node,
                postcondition: OpenPostcondition {
                    predicate: postcondition,
                },
            };
            let next = workflow_do_stmts_summary_form(rest, next_node, anchor)?;
            Some(WorkflowForm::Bind {
                node: workflow_summary_node(next_node),
                source: Box::new(source),
                binder: WorkflowBinder::Ignored,
                next: Box::new(next),
            })
        }
        _ => None,
    }
}

fn first_class_body_proc_summary(anchor: &str) -> ProcLowerSummary {
    ProcLowerSummary {
        coverage_obligation_nodes: Vec::new(),
        contract_summary: Some(ProcContractSummary {
            obligations: Vec::new(),
            public_anchor: Some(format!("first_class_body_as_proc_summary:{anchor}")),
        }),
        failure_summary: Some(ProcFailureSummary {
            routes: Vec::new(),
            conservative: false,
        }),
        resource_authority_summary: Some(ProcResourceAuthoritySummary {
            resources: Vec::new(),
            conservative: false,
        }),
        provenance_summary: Some(ProcProvenanceSummary {
            event_kinds: Vec::new(),
            conservative: false,
        }),
        source_origin: Some(first_class_workflow_source_origin()),
    }
}

fn first_class_workflow_source_origin() -> SourceOrigin {
    SourceOrigin::Synthetic {
        parent_span: None,
        reason: "first-class do:Workflow public summary adapter".to_string(),
    }
}

const fn workflow_summary_node(next_node: &mut u64) -> WorkflowNodeId {
    let node = WorkflowNodeId(*next_node);
    *next_node += 1;
    node
}

fn is_workflow_return_type(return_type: &Type) -> bool {
    match return_type {
        Type::Constructor { name, args } => name.as_ref() == "Workflow" && args.len() == 1,
        Type::Name(name) => name.as_ref() == "Workflow",
        _ => false,
    }
}

/// Diagnostic produced when a `pub fn` snippet fails to parse.
#[derive(Debug, Clone)]
pub struct PubFnDiagnostic {
    /// Function name extracted from the snippet, if possible.
    pub name: Option<String>,
    /// Human-readable reason for the failure.
    pub reason: String,
}

/// Attempt to extract the function name from a `pub fn` snippet.
pub(super) fn extract_fn_name_from_snippet(snippet: &str) -> Option<String> {
    let trimmed = snippet.trim();
    trimmed
        .strip_prefix("pub fn ")
        .and_then(|rest| rest.split(|c: char| c.is_whitespace() || c == '(').next())
        .map(std::string::ToString::to_string)
}

pub(super) fn parse_supported_pub_fn_callable(
    snippet: &str,
) -> Result<Option<ImportedCallableExport>, PubFnDiagnostic> {
    parse_pub_fn_callable(snippet).map_err(|e| PubFnDiagnostic {
        name: extract_fn_name_from_snippet(snippet),
        reason: format!("{e}"),
    })
}

/// Parse a `builtin fn` snippet into an [`ImportedCallableExport`].
///
/// Builtin functions have no Ash-level body, so a null placeholder expression
/// is used.  The important data is the function name and parameter list.
pub(super) fn parse_builtin_fn_callable(
    snippet: &str,
    module: String,
) -> Result<Option<ImportedCallableExport>, EngineError> {
    let mut input = new_input(snippet.trim());
    let parsed = parse_builtin_fn_definition
        .parse_next(&mut input)
        .map_err(|error| EngineError::Parse(format!("{error}")))?;

    let Definition::BuiltinFn(builtin) = parsed else {
        return Err(EngineError::Parse(
            "expected builtin fn to parse as a BuiltinFn definition".to_string(),
        ));
    };

    let name = builtin.name.to_string();
    let params = builtin
        .params
        .iter()
        .map(|param| param.name.to_string())
        .collect::<Vec<_>>();

    Ok(Some(ImportedCallableExport {
        callable: InlineCallable {
            exported_name: name,
            params,
            effectful_names: HashSet::new(),
            kind: CallableKind::Builtin { module },
            signature: Some(CallableSignature::Builtin(builtin)),
            exporting_modules: HashSet::new(),
            workflow_summary: None,
            module_runtime_callables: HashMap::new(),
        },
    }))
}

#[derive(Debug, Clone)]
pub(super) struct ImportedCallableExport {
    pub(super) callable: InlineCallable,
}

#[allow(clippy::unnecessary_wraps)]
pub(super) fn extract_callable_from_workflow(
    workflow: WorkflowDef,
) -> Result<Option<ImportedCallableExport>, EngineError> {
    let workflow_summary = public_workflow_summary_from_workflow(&workflow);
    let WorkflowDef {
        name,
        params,
        declared_return_type,
        body,
        span,
        ..
    } = workflow;

    let fn_def = workflow_signature_from_parts(
        name.clone(),
        params.clone(),
        declared_return_type,
        &body,
        span,
    );
    let signature = Some(CallableSignature::Function(fn_def.clone()));
    let expr = match body {
        Workflow::Ret { expr, .. } => expr,
        _ => workflow_signature_expr(&fn_def),
    };

    Ok(Some(ImportedCallableExport {
        callable: InlineCallable {
            exported_name: name.to_string(),
            params: params
                .into_iter()
                .map(|param| param.name.to_string())
                .collect(),
            effectful_names: HashSet::new(),
            kind: CallableKind::User { body: expr },
            signature,
            exporting_modules: HashSet::new(),
            workflow_summary: Some(workflow_summary),
            module_runtime_callables: HashMap::new(),
        },
    }))
}

pub(super) fn public_workflow_summary_from_workflow(
    workflow: &WorkflowDef,
) -> PublicWorkflowSummary {
    let origin = SourceOrigin::ImportedSummary {
        module: String::new(),
        public_anchor: workflow.name.to_string(),
    };
    let Ok(form) = legacy_workflow_def_to_workflow_form(workflow) else {
        return public_workflow_summary(workflow.name.as_ref());
    };
    let lowered = lower_workflow_form(&form, origin);
    PublicWorkflowSummary {
        node_count: lowered.projection_events.len(),
        projection_events: lowered.projection_events,
        coverage: lowered.coverage,
    }
}

pub(super) fn public_workflow_summary(anchor: &str) -> PublicWorkflowSummary {
    PublicWorkflowSummary {
        node_count: 1,
        projection_events: vec![ProjectionEvent {
            node: WorkflowNodeId(0),
            projection: ProjectionKind::Contract,
            origin: SourceOrigin::ImportedSummary {
                module: String::new(),
                public_anchor: anchor.to_string(),
            },
            kind: ProjectionEventKind::Neutral,
        }],
        coverage: CoverageEvidence::default(),
    }
}

pub(super) fn module_path_text(path: &Path) -> &str {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
}

pub(super) fn stamp_workflow_summary_import_origin(
    summary: &mut PublicWorkflowSummary,
    module: &str,
    public_anchor: &str,
) {
    let origin = SourceOrigin::ImportedSummary {
        module: module.to_string(),
        public_anchor: public_anchor.to_string(),
    };
    for event in &mut summary.projection_events {
        event.origin = origin.clone();
    }
}

fn workflow_signature_from_parts(
    name: ash_parser::surface::Name,
    params: Vec<ash_parser::surface::Parameter>,
    return_type: Option<ash_parser::surface::Type>,
    body: &Workflow,
    span: ash_parser::token::Span,
) -> ash_parser::surface::FnDef {
    ash_parser::surface::FnDef {
        visibility: ash_parser::surface::Visibility::Public,
        name,
        type_params: Vec::new(),
        params: params
            .into_iter()
            .map(|param| ash_parser::surface::Param {
                name: param.name,
                ty: param.ty,
            })
            .collect(),
        return_type,
        proposition_tail: None,
        contract: None,
        body: workflow_signature_expr_for_body(body),
        span,
    }
}

fn workflow_signature_expr(fn_def: &ash_parser::surface::FnDef) -> Expr {
    fn_def.body.clone()
}

fn workflow_signature_expr_for_body(body: &Workflow) -> Expr {
    let span = workflow_span(body);
    Expr::Variable {
        name: "__imported_workflow_body_not_inlined".into(),
        span,
    }
}

const fn workflow_span(workflow: &Workflow) -> ash_parser::token::Span {
    match workflow {
        Workflow::Observe { span, .. }
        | Workflow::Orient { span, .. }
        | Workflow::Propose { span, .. }
        | Workflow::Decide { span, .. }
        | Workflow::Check { span, .. }
        | Workflow::Oblige { span, .. }
        | Workflow::Act { span, .. }
        | Workflow::Let { span, .. }
        | Workflow::If { span, .. }
        | Workflow::For { span, .. }
        | Workflow::With { span, .. }
        | Workflow::Maybe { span, .. }
        | Workflow::Must { span, .. }
        | Workflow::Seq { span, .. }
        | Workflow::Done { span, .. }
        | Workflow::Ret { span, .. }
        | Workflow::Set { span, .. }
        | Workflow::Send { span, .. }
        | Workflow::Receive { span, .. }
        | Workflow::Yield { span, .. }
        | Workflow::Resume { span, .. } => *span,
    }
}
