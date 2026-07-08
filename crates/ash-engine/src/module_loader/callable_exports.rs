//! Public callable, workflow, and capability export helpers.

use ash_core::workflow_carrier::lower_workflow_form;

use super::{
    CallableKind, CallableSignature, CoreTypeBody, CoreTypeDef, CoreVisibility, Definition,
    EngineError, Expr, HashMap, HashSet, InlineCallable, OpenPostcondition, Parser, Path,
    ProcContractSummary, ProcFailureSummary, ProcLowerSummary, ProcProvenanceSummary,
    ProcResourceAuthoritySummary, PublicWorkflowSummary, SourceOrigin, Type, WorkflowBinder,
    WorkflowForm, WorkflowNodeId, WorkflowScope, callable_row_requirement_from_builtin,
    callable_row_requirement_from_fn_def, extract_semicolon_snippets, new_input,
    parse_builtin_fn_definition, parse_fn_definition,
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
    let row_requirement = callable_row_requirement_from_fn_def(&function);

    ImportedCallableExport {
        callable: InlineCallable {
            exported_name: name,
            params,
            effectful_names: HashSet::new(),
            kind: CallableKind::User { body },
            row_requirement,
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
            row_requirement: callable_row_requirement_from_builtin(&builtin),
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
