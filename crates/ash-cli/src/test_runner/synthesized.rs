//! Synthesized test generation from contracts, obligations, and laws.
//!
//! TASK-513: Opt-in synthesized test planning. These are NOT run by default.
//! They must be explicitly requested via `--include-synthesized` or `--only-synthesized`.
//!
//! Synthesized tests complement authored tests but are never a substitute.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ash_parser::surface::{
    BinaryOp, Definition, Expr, LawDef, Literal, ModuleFile, Param, ProofBody, Requirement, Type,
    UnaryOp,
};
use ash_parser::{LoweringContext, effectful_names_from_definitions, lower_expr_with_context};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::test_runner::types::{TestKind, TestResult, TestSource};

/// Runner-facing synthesized-case schema version.
pub const RUNNER_SYNTHESIS_SCHEMA_VERSION: &str = "ash-synthesized-v1.0";

mod schema;
pub use schema::*;

pub fn build_runner_introspection_snapshot(
    path: &Path,
    engine: &ash_engine::Engine,
) -> Result<RunnerIntrospectionSnapshot, String> {
    let source = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read source for live snapshot: {error}"))?;
    let module = parse_synthesized_metadata_module(path, &source)?;
    let check_source = checked_source_kind(path, &source, engine)?;

    Ok(snapshot_from_checked_module(
        path,
        &source,
        &module,
        check_source,
    ))
}

fn parse_synthesized_metadata_module(path: &Path, source: &str) -> Result<ModuleFile, String> {
    // Parser-owned module structure is authoritative whenever the canonical
    // surface parser can represent the source. The stripper remains only for
    // legacy parser-failure compatibility and never authorizes synthesized
    // execution.
    if let Ok(module) = ash_parser::parse_surface_file_with_path(source, Some(path)) {
        return Ok(module);
    }

    let metadata_source = strip_synthesized_metadata_non_definition_lines(source);
    ash_parser::parse_surface_file_with_path(&metadata_source, Some(path)).map_err(|errors| {
        let diagnostics = errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ");
        format!("parse error during live snapshot module production: {diagnostics}")
    })
}

fn strip_synthesized_metadata_non_definition_lines(source: &str) -> String {
    let mut kept = Vec::new();
    let mut skipping_import = false;

    for line in source.lines() {
        let trimmed = line.trim_start();
        if skipping_import {
            if trimmed.ends_with(';') {
                skipping_import = false;
            }
            continue;
        }

        if trimmed.starts_with("use ") || trimmed.starts_with("pub use ") {
            let opens = trimmed.matches('{').count();
            let closes = trimmed.matches('}').count();
            if !trimmed.ends_with(';') && opens != closes {
                skipping_import = true;
            }
            continue;
        }

        if trimmed.starts_with("mod ") || trimmed.starts_with("pub mod ") {
            continue;
        }

        kept.push(line);
    }

    kept.join("\n")
}

fn checked_source_kind(
    path: &Path,
    source: &str,
    engine: &ash_engine::Engine,
) -> Result<&'static str, String> {
    match engine.parse_file_source(path, source) {
        Ok(mut workflow) => {
            engine
                .check(&mut workflow)
                .map_err(|error| format!("type error during live snapshot production: {error}"))?;
            Ok("workflow")
        }
        Err(workflow_error) => {
            let module_check = engine.check_module_file(path).map_err(|module_error| {
                format!(
                    "parse/check error during live snapshot production: workflow parse failed ({workflow_error}); module check failed ({module_error})"
                )
            })?;
            if module_check.errors.is_empty() {
                Ok("module-file")
            } else {
                Err(format!(
                    "module check error during live snapshot production: {}",
                    module_check.errors.join("; ")
                ))
            }
        }
    }
}

fn snapshot_from_checked_module(
    path: &Path,
    source: &str,
    module: &ModuleFile,
    check_source: &str,
) -> RunnerIntrospectionSnapshot {
    let source_hash = stable_sha256(&["source", source]);
    let module_identity = module_identity_for_path(path);
    let source_artifact_id = format!("source-file:{}#{source_hash}", path.display());
    let check_summary_id = stable_sha256(&[
        "checked-runner-introspection",
        RUNNER_SYNTHESIS_SCHEMA_VERSION,
        &module_identity,
        &source_hash,
        check_source,
    ]);

    let contracts = executable_contracts_from_checked_module(module);
    let laws = extract_laws(module);
    let supported_contract_names = contracts
        .iter()
        .map(|contract| contract.callable_name.clone())
        .collect::<Vec<_>>();

    RunnerIntrospectionSnapshot {
        schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
        module_identity,
        source_artifact_id,
        check_summary_id: format!("checked:{check_summary_id}"),
        contracts,
        laws,
        unsupported: unsupported_rows_from_checked_module(path, module, &supported_contract_names),
        ..RunnerIntrospectionSnapshot::default()
    }
}

fn module_identity_for_path(path: &Path) -> String {
    path.file_stem().and_then(|stem| stem.to_str()).map_or_else(
        || path.display().to_string(),
        |stem| format!("module:{stem}"),
    )
}

fn unsupported_rows_from_checked_module(
    path: &Path,
    module: &ModuleFile,
    supported_contract_names: &[String],
) -> Vec<IntrospectionUnsupportedReason> {
    let mut rows = Vec::new();

    let all_contract_targets = contract_targets_from_module(module);
    let contract_targets = all_contract_targets
        .iter()
        .filter(|target| {
            !supported_contract_names
                .iter()
                .any(|supported| supported == *target)
        })
        .cloned()
        .collect::<Vec<_>>();
    if all_contract_targets.is_empty() {
        rows.push(IntrospectionUnsupportedReason {
            source_kind: "contract".to_string(),
            target_name: path_stem(path),
            reason: "live checked snapshot has no lowered executable contract metadata exposed"
                .to_string(),
        });
    } else if !contract_targets.is_empty() {
        rows.extend(
            contract_targets
                .into_iter()
                .map(|target_name| IntrospectionUnsupportedReason {
                    source_kind: "contract".to_string(),
                    target_name,
                    reason: "live checked snapshot identified contract-like source metadata, but executable lowered contract metadata is not exposed for TASK-1012".to_string(),
                }),
        );
    }

    let obligation_targets = obligation_targets_from_module(module);
    if obligation_targets.is_empty() {
        rows.push(IntrospectionUnsupportedReason {
            source_kind: "obligation".to_string(),
            target_name: path_stem(path),
            reason: "live checked snapshot has no lowered executable obligation lifecycle metadata exposed"
                .to_string(),
        });
    } else {
        rows.extend(
            obligation_targets
                .into_iter()
                .map(|target_name| IntrospectionUnsupportedReason {
                    source_kind: "obligation".to_string(),
                    target_name,
                    reason: "live checked snapshot identified obligation-like source metadata, but executable lowered lifecycle metadata is not exposed for TASK-1012".to_string(),
                }),
        );
    }

    rows
}

fn contract_targets_from_module(module: &ModuleFile) -> Vec<String> {
    let mut targets = Vec::new();

    for definition in &module.definitions {
        if let Definition::Function(function) = definition
            && contract_has_rows(&function.contract)
        {
            targets.push(function.name.to_string());
        }
    }

    targets.sort();
    targets.dedup();
    targets
}

/// Extract runner-facing law metadata from a parsed module.
pub fn extract_laws(module: &ModuleFile) -> Vec<RunnerLawMetadata> {
    let proof_scopes = proof_scopes(module);
    let mut laws = Vec::new();

    for definition in &module.definitions {
        match definition {
            Definition::Interface(interface) => {
                let interface_name = interface.name.to_string();
                let hand_proved_interface_law_names = proof_scopes
                    .interface
                    .get(&interface_name)
                    .cloned()
                    .unwrap_or_default();
                let delegated_interface_law_names = proof_scopes
                    .interface_by_test
                    .get(&interface_name)
                    .cloned()
                    .unwrap_or_default();
                for law in &interface.laws {
                    if hand_proved_interface_law_names.contains(&*law.name) {
                        continue;
                    }
                    laws.push(law_metadata(
                        law,
                        LawScope::Interface,
                        Some(interface_name.clone()),
                        delegated_interface_law_names.get(&*law.name).cloned(),
                    ));
                }
            }
            Definition::Law(law) if !proof_scopes.module.contains(&*law.name) => {
                laws.push(law_metadata(
                    law,
                    LawScope::Module,
                    None,
                    proof_scopes.module_by_test.get(&*law.name).cloned(),
                ));
            }
            _ => {}
        }
    }

    laws
}

struct ProofScopes {
    module: BTreeSet<String>,
    module_by_test: BTreeMap<String, LawTestEvidence>,
    interface: BTreeMap<String, BTreeSet<String>>,
    interface_by_test: BTreeMap<String, BTreeMap<String, LawTestEvidence>>,
}

fn proof_scopes(module: &ModuleFile) -> ProofScopes {
    let mut scopes = ProofScopes {
        module: BTreeSet::new(),
        module_by_test: BTreeMap::new(),
        interface: BTreeMap::new(),
        interface_by_test: BTreeMap::new(),
    };
    for definition in &module.definitions {
        match definition {
            Definition::Proof(proof) => match law_test_evidence_from_proof_body(&proof.body) {
                Some(evidence) => {
                    scopes
                        .module_by_test
                        .insert(proof.name.to_string(), evidence);
                }
                _ => {
                    scopes.module.insert(proof.name.to_string());
                }
            },
            Definition::Impl(impl_def) => {
                for proof in &impl_def.proofs {
                    match law_test_evidence_from_proof_body(&proof.body) {
                        Some(evidence) => {
                            scopes
                                .interface_by_test
                                .entry(impl_def.interface.to_string())
                                .or_default()
                                .insert(proof.name.to_string(), evidence);
                        }
                        _ => {
                            scopes
                                .interface
                                .entry(impl_def.interface.to_string())
                                .or_default()
                                .insert(proof.name.to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }
    scopes
}

fn law_metadata(
    law: &LawDef,
    scope: LawScope,
    owner: Option<String>,
    test_evidence: Option<LawTestEvidence>,
) -> RunnerLawMetadata {
    let scope_segment = match scope {
        LawScope::Module => "module".to_string(),
        LawScope::Interface => format!(
            "interface:{}",
            owner
                .as_deref()
                .expect("interface law metadata should include an owner")
        ),
    };

    let delegated_test = match &test_evidence {
        Some(LawTestEvidence::Authored { test_name }) => Some(test_name.clone()),
        _ => None,
    };

    RunnerLawMetadata {
        id: format!("law:{scope_segment}:{}", law.name),
        name: law.name.to_string(),
        scope,
        owner,
        params: law.params.iter().map(format_param).collect(),
        proposition: format_expr(&law.proposition),
        delegated_test,
        test_evidence,
    }
}

fn law_test_evidence_from_proof_body(body: &ProofBody) -> Option<LawTestEvidence> {
    match body {
        ProofBody::ByTest { test_name } => Some(LawTestEvidence::Authored {
            test_name: test_name.clone(),
        }),
        ProofBody::ByTestProperty { strategies } => Some(LawTestEvidence::Property {
            strategies: strategies
                .iter()
                .map(|binding| PropertyStrategyDescriptor {
                    param_name: binding.param_name.clone(),
                    strategy_expr: format_expr(&binding.strategy_expr),
                })
                .collect(),
        }),
        ProofBody::ByTestSmallWorld => Some(LawTestEvidence::SmallWorld),
        _ => None,
    }
}

fn format_param(param: &Param) -> String {
    format!("{}: {}", param.name, format_type(&param.ty))
}

fn format_type(ty: &Type) -> String {
    match ty {
        Type::Name(name) => name.to_string(),
        Type::Hole { .. } => "_".to_string(),
        Type::List(inner) => format!("[{}]", format_type(inner)),
        Type::Tuple(items) => {
            let items = items.iter().map(format_type).collect::<Vec<_>>().join(", ");
            format!("({items})")
        }
        Type::Record(fields) => {
            let fields = fields
                .iter()
                .map(|(name, ty)| format!("{name}: {}", format_type(ty)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{ {fields} }}")
        }
        Type::Capability(name) => format!("Capability<{name}>"),
        Type::Constructor { name, args } => {
            let args = args.iter().map(format_type).collect::<Vec<_>>().join(", ");
            format!("{name}<{args}>")
        }
        Type::Associated { base, name } => format!("{}::{name}", format_type(base)),
        Type::AssociatedFamilyProjection {
            interface,
            args,
            member,
            ..
        } => {
            let args = args.iter().map(format_type).collect::<Vec<_>>().join(", ");
            format!("<{interface}<{args}>>::{member}")
        }
        Type::Fn(params, _row, ret) => {
            let params = params
                .iter()
                .map(format_type)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({params}) -> {}", format_type(ret))
        }
    }
}

fn format_expr(expr: &Expr) -> String {
    match expr {
        Expr::Literal(literal) => format_literal(literal),
        Expr::Variable { name, .. } => name.to_string(),
        Expr::FieldAccess { base, field, .. } => format!("{}.{field}", format_expr(base)),
        Expr::IndexAccess { base, index, .. } => {
            format!("{}[{}]", format_expr(base), format_expr(index))
        }
        Expr::Unary { op, operand, .. } => {
            format!("{}{}", unary_op_symbol(*op), format_expr(operand))
        }
        Expr::Binary {
            op, left, right, ..
        } => format!(
            "{} {} {}",
            format_expr(left),
            binary_op_symbol(*op),
            format_expr(right)
        ),
        Expr::Call {
            func, module, args, ..
        } => {
            let args = args.iter().map(format_expr).collect::<Vec<_>>().join(", ");
            match module {
                Some(module) => format!("{module}::{func}({args})"),
                None => format!("{func}({args})"),
            }
        }
        Expr::FnApply { func, args, .. } => {
            let args = args.iter().map(format_expr).collect::<Vec<_>>().join(", ");
            format!("{}({args})", format_expr(func))
        }
        Expr::List { items, .. } => {
            let items = items.iter().map(format_expr).collect::<Vec<_>>().join(", ");
            format!("[{items}]")
        }
        unsupported => format!("<unsupported law expression: {unsupported:?}>"),
    }
}

fn format_literal(literal: &Literal) -> String {
    match literal {
        Literal::Int(value) => value.to_string(),
        Literal::Float(value) => value.0.to_string(),
        Literal::String(value) => format!("\"{value}\""),
        Literal::Bool(value) => value.to_string(),
        Literal::Null => "null".to_string(),
        Literal::List(items) => {
            let items = items
                .iter()
                .map(format_literal)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{items}]")
        }
    }
}

fn unary_op_symbol(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Not => "!",
        UnaryOp::Neg => "-",
    }
}

fn binary_op_symbol(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
        BinaryOp::Eq => "==",
        BinaryOp::Neq => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Gt => ">",
        BinaryOp::Leq => "<=",
        BinaryOp::Geq => ">=",
        BinaryOp::In => "in",
        BinaryOp::Pipe => "|>",
    }
}

fn executable_contracts_from_checked_module(module: &ModuleFile) -> Vec<RunnerContractMetadata> {
    let mut contracts = Vec::new();
    let lowering_ctx = LoweringContext::with_effectful_names(effectful_names_from_definitions(
        &module.definitions,
    ));

    for definition in &module.definitions {
        let Definition::Function(function) = definition else {
            continue;
        };
        let Some(contract) = &function.contract else {
            continue;
        };
        if contract.ensures.is_empty() || !function.type_params.is_empty() {
            continue;
        }

        let Some(return_type) = function.return_type.as_ref().and_then(type_name) else {
            continue;
        };
        if return_type != "Int" {
            continue;
        }

        let mut param_names = Vec::new();
        let mut param_types = Vec::new();
        let mut supported_params = true;
        for param in &function.params {
            let Some(param_type) = type_name(&param.ty) else {
                supported_params = false;
                break;
            };
            if param_type != "Int" {
                supported_params = false;
                break;
            }
            param_names.push(param.name.to_string());
            param_types.push(param_type);
        }
        if !supported_params || param_names.is_empty() {
            continue;
        }

        let lowered_requires = contract
            .requires
            .iter()
            .filter_map(requirement_expression)
            .collect::<Vec<_>>();
        if lowered_requires.len() != contract.requires.len() {
            continue;
        }
        let lowered_ensures = contract
            .ensures
            .iter()
            .filter_map(|clause| expr_to_simple_string(&clause.expr))
            .collect::<Vec<_>>();
        if lowered_ensures.len() != contract.ensures.len() {
            continue;
        }
        let executable_postconditions = contract
            .ensures
            .iter()
            .zip(&lowered_ensures)
            .map(|(clause, display)| {
                lower_expr_with_context(&clause.expr, &lowering_ctx).map(|expression| {
                    ContractPostconditionOracle {
                        display: display.clone(),
                        expression,
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>();
        let Ok(executable_postconditions) = executable_postconditions else {
            continue;
        };
        let Ok(body_expression) = lower_expr_with_context(&function.body, &lowering_ctx) else {
            continue;
        };

        let generation_hints =
            finite_contract_generation_hints(&param_names, &param_types, &lowered_requires);
        if generation_hints
            .iter()
            .all(|hint| hint.source != TypeGeneratorSource::ContractValid)
        {
            continue;
        }

        let mut executable_case_kinds = vec![SynthesizedOracleKind::PostconditionHolds];
        if !lowered_requires.is_empty() {
            executable_case_kinds.push(SynthesizedOracleKind::PreconditionBoundary);
        }

        contracts.push(RunnerContractMetadata {
            id: format!("contract:{}", function.name),
            callable_name: function.name.to_string(),
            callable_kind: "pure_function".to_string(),
            param_names,
            param_types,
            return_type: Some(return_type),
            lowered_requires,
            lowered_ensures,
            executable_postconditions,
            executable_target: Some(ContractExecutableTarget {
                kind: ContractExecutableTargetKind::PureFunction,
                target_ref: function.name.to_string(),
                setup: ContractExecutionSetup::PureNoSetup,
                body: ContractTargetBody::ReturnExpression {
                    expression: body_expression,
                },
            }),
            generation_hints,
            executable_case_kinds,
            ..RunnerContractMetadata::default()
        });
    }

    contracts
}

fn type_name(ty: &Type) -> Option<String> {
    match ty {
        Type::Name(name) => Some(name.to_string()),
        _ => None,
    }
}

fn requirement_expression(requirement: &Requirement) -> Option<String> {
    match requirement {
        Requirement::Arithmetic { expr } => expr_to_simple_string(expr),
        Requirement::HasCapability { .. } => None,
    }
}

fn expr_to_simple_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Literal(literal) => literal_to_simple_string(literal),
        Expr::Variable { name, .. } => Some(name.to_string()),
        Expr::Binary {
            op, left, right, ..
        } => Some(format!(
            "{} {} {}",
            expr_to_simple_string(left)?,
            binary_op_token(op)?,
            expr_to_simple_string(right)?
        )),
        Expr::Block { tail_expr, .. } => tail_expr.as_deref().and_then(expr_to_simple_string),
        _ => None,
    }
}

fn literal_to_simple_string(literal: &Literal) -> Option<String> {
    match literal {
        Literal::Int(value) => Some(value.to_string()),
        Literal::Bool(value) => Some(value.to_string()),
        Literal::String(value) => Some(format!("{value:?}")),
        Literal::Null => Some("null".to_string()),
        Literal::Float(_) | Literal::List(_) => None,
    }
}

fn binary_op_token(op: &BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add => Some("+"),
        BinaryOp::Sub => Some("-"),
        BinaryOp::Mul => Some("*"),
        BinaryOp::Div => Some("/"),
        BinaryOp::Mod => Some("%"),
        BinaryOp::Eq => Some("=="),
        BinaryOp::Neq => Some("!="),
        BinaryOp::Lt => Some("<"),
        BinaryOp::Gt => Some(">"),
        BinaryOp::Leq => Some("<="),
        BinaryOp::Geq => Some(">="),
        BinaryOp::And | BinaryOp::Or | BinaryOp::In | BinaryOp::Pipe => None,
    }
}

fn finite_contract_generation_hints(
    param_names: &[String],
    param_types: &[String],
    lowered_requires: &[String],
) -> Vec<TypeGeneratorDescriptor> {
    let mut hints = Vec::new();

    for expression in lowered_requires {
        let Some(param) = expression_parameter(expression) else {
            continue;
        };
        let Some(param_index) = param_names.iter().position(|name| name == &param) else {
            continue;
        };
        let Some(param_type) = param_types.get(param_index) else {
            continue;
        };
        let Some((valid, invalid)) = finite_boundary_values_from_expression(expression) else {
            continue;
        };
        hints.push(TypeGeneratorDescriptor {
            id: format!("{param}-valid"),
            target_type: param_type.clone(),
            source: TypeGeneratorSource::ContractValid,
            exact_values: vec![json!(valid)],
            seed_policy: Some("derived_from_checked_contract_boundary".to_string()),
            max_cases: Some(1),
            ..TypeGeneratorDescriptor::default()
        });
        hints.push(TypeGeneratorDescriptor {
            id: format!("{param}-invalid"),
            target_type: param_type.clone(),
            source: TypeGeneratorSource::ContractInvalidNearby,
            exact_values: vec![json!(invalid)],
            seed_policy: Some("derived_from_checked_contract_boundary".to_string()),
            max_cases: Some(1),
            ..TypeGeneratorDescriptor::default()
        });
    }

    hints
}

fn finite_boundary_values_from_expression(expression: &str) -> Option<(i64, i64)> {
    let tokens = expression.split_whitespace().collect::<Vec<_>>();
    if tokens.len() != 3 {
        return None;
    }
    let boundary = tokens[2].parse::<i64>().ok()?;
    match tokens[1] {
        ">" => Some((boundary.checked_add(1)?, boundary)),
        ">=" => Some((boundary, boundary.checked_sub(1)?)),
        "<" => Some((boundary.checked_sub(1)?, boundary)),
        "<=" => Some((boundary, boundary.checked_add(1)?)),
        "==" => Some((boundary, boundary.checked_add(1)?)),
        "!=" => Some((boundary.checked_add(1)?, boundary)),
        _ => None,
    }
}

fn contract_has_rows(contract: &Option<ash_parser::surface::Contract>) -> bool {
    contract
        .as_ref()
        .is_some_and(|contract| !contract.requires.is_empty() || !contract.ensures.is_empty())
}

fn obligation_targets_from_module(_module: &ModuleFile) -> Vec<String> {
    Vec::new()
}

fn path_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn stable_sha256(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.len().to_le_bytes());
        hasher.update(part.as_bytes());
    }
    format!("sha256:{:x}", hasher.finalize())
}

mod repro;
use repro::{
    deferred_result, deferred_result_with_kind, fallback_repro, repro_artifact,
    snapshot_source_label, source_from_label,
};

/// Generate executable synthesized results from structured runner metadata.
pub fn synthesize_from_snapshot(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
) -> Vec<TestResult> {
    synthesize_from_snapshot_with_limits(path, snapshot, None, None, None)
}

/// Generate executable synthesized results with runner generation limits.
pub fn synthesize_from_snapshot_with_limits(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    _seed: Option<u64>,
    _max_cases: Option<usize>,
    _max_worlds: Option<usize>,
) -> Vec<TestResult> {
    deferred_compatibility_results(path, snapshot)
}

/// Present structured metadata through the retained public compatibility API.
///
/// This API accepts no Engine capability and therefore cannot submit an
/// admitted program. It records each metadata identity as deferred instead of
/// treating the metadata as an executable oracle.
fn deferred_compatibility_results(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
) -> Vec<TestResult> {
    let mut results = Vec::new();

    for contract in &snapshot.contracts {
        results.push(compatibility_deferred_result(
            path,
            snapshot,
            CompatibilityDeferred {
                source: TestSource::Contract,
                kind: TestKind::Unit,
                name: format!("synthesized/contract/{}/deferred", contract.callable_name),
                case_id: contract.id.clone(),
                reason: "legacy synthesized metadata API has no Engine submission authority"
                    .to_string(),
                oracle_snapshot: json!({
                    "source": "contract",
                    "target": contract.callable_name,
                    "metadata_id": contract.id,
                }),
            },
        ));
    }

    for obligation in &snapshot.obligations {
        results.push(compatibility_deferred_result(
            path,
            snapshot,
            CompatibilityDeferred {
                source: TestSource::Obligation,
                kind: TestKind::Unit,
                name: format!(
                    "synthesized/obligation/{}/deferred",
                    obligation.obligation_name
                ),
                case_id: obligation.id.clone(),
                reason: "obligation metadata has no Engine submission authority".to_string(),
                oracle_snapshot: json!({
                    "source": "obligation",
                    "target": obligation.obligation_name,
                    "metadata_id": obligation.id,
                }),
            },
        ));
    }

    for law in &snapshot.laws {
        results.push(compatibility_deferred_result(
            path,
            snapshot,
            CompatibilityDeferred {
                source: TestSource::Law,
                kind: TestKind::Unit,
                name: format!("synthesized/law/{}/deferred", law.name),
                case_id: law.id.clone(),
                reason: "law metadata has no Engine submission authority".to_string(),
                oracle_snapshot: json!({
                    "source": "law",
                    "law": law.name,
                    "metadata_id": law.id,
                }),
            },
        ));
    }

    for descriptor in &snapshot.generators {
        results.push(compatibility_deferred_result(
            path,
            snapshot,
            CompatibilityDeferred {
                source: TestSource::Contract,
                kind: TestKind::Property,
                name: format!("synthesized/property/{}/deferred", descriptor.id),
                case_id: format!("property:{}", descriptor.id),
                reason: "generated property metadata has no Engine submission authority"
                    .to_string(),
                oracle_snapshot: json!({
                    "source": "property",
                    "descriptor_id": descriptor.id,
                    "target_type": descriptor.target_type,
                }),
            },
        ));
    }

    for domain in &snapshot.small_world_domains {
        results.push(compatibility_deferred_result(
            path,
            snapshot,
            CompatibilityDeferred {
                source: domain.source,
                kind: TestKind::SmallWorld,
                name: format!("synthesized/smallworld/{}/deferred", domain.id),
                case_id: format!("smallworld:{}", domain.id),
                reason: "small-world metadata has no Engine submission authority".to_string(),
                oracle_snapshot: json!({
                    "source": "smallworld",
                    "domain_id": domain.id,
                }),
            },
        ));
    }

    for unsupported in &snapshot.unsupported {
        results.push(compatibility_deferred_result(
            path,
            snapshot,
            CompatibilityDeferred {
                source: source_from_label(&unsupported.source_kind),
                kind: TestKind::Unit,
                name: format!(
                    "synthesized/{}/{}/unsupported",
                    unsupported.source_kind, unsupported.target_name
                ),
                case_id: format!(
                    "{}:{}:unsupported",
                    unsupported.source_kind, unsupported.target_name
                ),
                reason: normalize_deferred_reason(&unsupported.reason),
                oracle_snapshot: json!({
                    "source": unsupported.source_kind,
                    "target": unsupported.target_name,
                    "reason": unsupported.reason,
                    "snapshot_source": snapshot_source_label(snapshot),
                }),
            },
        ));
    }

    results
}

struct CompatibilityDeferred {
    source: TestSource,
    kind: TestKind,
    name: String,
    case_id: String,
    reason: String,
    oracle_snapshot: serde_json::Value,
}

fn compatibility_deferred_result(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    deferred: CompatibilityDeferred,
) -> TestResult {
    let CompatibilityDeferred {
        source,
        kind,
        name,
        case_id,
        reason,
        mut oracle_snapshot,
    } = deferred;
    let message = normalize_deferred_reason(&reason);
    oracle_snapshot["execution_route"] = json!("deferred_before_execution");
    let mut repro = repro_artifact(
        path,
        snapshot.source_artifact_id.clone(),
        snapshot.check_summary_id.clone(),
        case_id,
        0,
        1,
        None,
        oracle_snapshot,
        None,
    );
    if source == TestSource::Law {
        repro.replay_command = format!("ash test {} --only-synthesized laws", path.display());
    }
    let mut result = deferred_result_with_kind(path, source, kind, name, message, repro);
    result.tags = vec!["synthesized".to_string(), "deferred".to_string()];
    result
}

/// Generate test-client results through the canonical admitted Engine seam.
///
/// The test route recognizes only the two source identities declared by
/// TASK-2035. Every other metadata shape is represented as a deferred result;
/// the client does not interpret metadata as an executable program.
pub fn synthesize_from_snapshot_with_engine_limits(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    engine: &ash_engine::Engine,
    _seed: Option<u64>,
    _max_cases: Option<usize>,
    _max_worlds: Option<usize>,
) -> Vec<TestResult> {
    let timeout = std::time::Duration::from_secs(30);
    let mut results = snapshot
        .contracts
        .iter()
        .map(|contract| catalogue_engine_result(path, snapshot, contract, engine, timeout))
        .collect::<Vec<_>>();

    results.extend(snapshot.unsupported.iter().map(|unsupported| {
        let source = source_from_label(&unsupported.source_kind);
        let case_id = if source == TestSource::Contract {
            unsupported.target_name.clone()
        } else {
            format!(
                "{}:{}:unsupported",
                unsupported.source_kind, unsupported.target_name
            )
        };
        let reason = normalize_deferred_reason(&unsupported.reason);
        let mut result = deferred_result(
            path,
            source,
            format!(
                "synthesized/{}/{}",
                unsupported.source_kind, unsupported.target_name
            ),
            reason.clone(),
            repro_artifact(
                path,
                snapshot.source_artifact_id.clone(),
                snapshot.check_summary_id.clone(),
                case_id,
                0,
                1,
                None,
                json!({
                    "source": unsupported.source_kind,
                    "target": unsupported.target_name,
                    "reason": reason,
                    "execution_route": "deferred_before_execution",
                }),
                None,
            ),
        );
        result.tags = vec!["synthesized".to_string(), unsupported.source_kind.clone()];
        result
    }));

    results.extend(snapshot.obligations.iter().map(|obligation| {
        deferred_metadata_result(
            path,
            snapshot,
            TestSource::Obligation,
            &obligation.id,
            &obligation.obligation_name,
            "obligation metadata has no TASK-2035 source identity",
        )
    }));
    results.extend(snapshot.laws.iter().map(|law| {
        deferred_metadata_result(
            path,
            snapshot,
            TestSource::Law,
            &law.id,
            &law.name,
            "law metadata has no TASK-2035 source identity",
        )
    }));
    results.extend(
        snapshot
            .generators
            .iter()
            .map(|descriptor| deferred_generated_property_result(path, snapshot, descriptor)),
    );
    results.extend(snapshot.small_world_domains.iter().map(|domain| {
        deferred_metadata_result(
            path,
            snapshot,
            domain.source,
            &domain.id,
            &domain.id,
            "small-world metadata has no TASK-2035 source identity",
        )
    }));
    results
}

fn deferred_generated_property_result(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    descriptor: &TypeGeneratorDescriptor,
) -> TestResult {
    let message = "deferred: generated property metadata has no TASK-2035 source identity";
    let repro = repro_artifact(
        path,
        snapshot.source_artifact_id.clone(),
        snapshot.check_summary_id.clone(),
        format!("property:{}", descriptor.id),
        0,
        1,
        None,
        json!({
            "descriptor_id": descriptor.id,
            "descriptor": descriptor,
            "execution_route": "deferred_before_execution",
        }),
        None,
    );
    let mut result = deferred_result_with_kind(
        path,
        TestSource::Contract,
        TestKind::Property,
        format!("synthesized/property/{}/deferred", descriptor.id),
        message,
        repro,
    );
    result.tags = vec![
        "synthesized".to_string(),
        "property".to_string(),
        "deferred".to_string(),
    ];
    result
}

fn deferred_metadata_result(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    source: TestSource,
    id: &str,
    target: &str,
    reason: &str,
) -> TestResult {
    let message = format!("deferred: {reason}");
    let mut oracle_snapshot = json!({
        "target": target,
        "reason": message,
        "execution_route": "deferred_before_execution",
    });
    if source == TestSource::Law {
        oracle_snapshot["law"] = json!(target);
    }
    let mut repro = repro_artifact(
        path,
        snapshot.source_artifact_id.clone(),
        snapshot.check_summary_id.clone(),
        id.to_string(),
        0,
        1,
        None,
        oracle_snapshot,
        None,
    );
    if source == TestSource::Law {
        repro.replay_command = format!("ash test {} --only-synthesized laws", path.display());
    }
    let mut result = deferred_result(
        path,
        source,
        format!("synthesized/{target}/deferred"),
        message.clone(),
        repro,
    );
    result.tags = vec!["synthesized".to_string()];
    result
}

fn normalize_deferred_reason(reason: &str) -> String {
    if reason.starts_with("deferred:") {
        reason.to_string()
    } else {
        format!("deferred: {reason}")
    }
}

mod authored_law;
pub(crate) use authored_law::authored_law_test_results;

mod contract;
use contract::{catalogue_engine_result, expression_parameter};

// Shared QuickCheck generators remain available to the independent quickcheck
// runner. They do not execute synthesized property or law cases.
pub(crate) mod value_generation;

pub fn synthesize_contract_tests(path: &Path, source: &str) -> Vec<TestResult> {
    let mut tests = Vec::new();

    // Simple pattern-based contract detection for V1.
    // Look for target function declarations with requires/ensures clauses.
    let lines: Vec<&str> = source.lines().collect();
    let mut in_function = false;
    let mut function_name = String::new();

    for line in &lines {
        let trimmed = line.trim();

        if trimmed.starts_with("fn ") {
            in_function = true;
            // Extract name (simple heuristic)
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                function_name = parts[1]
                    .trim_end_matches('{')
                    .trim_end_matches('(')
                    .to_string();
            }
        }

        // Detect requires clauses
        if in_function && trimmed.contains("requires") {
            let test_name = format!("synthesized/contract/{}/requires-boundary", function_name);
            tests.push(deferred_result(
                path,
                TestSource::Contract,
                test_name.clone(),
                "deferred: raw-source requires pattern is not lowered executable contract metadata",
                fallback_repro(
                    path,
                    TestSource::Contract,
                    test_name,
                    json!({ "source": "contract", "oracle": "requires", "fallback": "raw_source_pattern" }),
                ),
            ));
        }

        // Detect ensures clauses
        if in_function && trimmed.contains("ensures") {
            let test_name = format!("synthesized/contract/{}/ensures-boundary", function_name);
            tests.push(deferred_result(
                path,
                TestSource::Contract,
                test_name.clone(),
                "deferred: raw-source ensures pattern is not lowered executable contract metadata",
                fallback_repro(
                    path,
                    TestSource::Contract,
                    test_name,
                    json!({ "source": "contract", "oracle": "ensures", "fallback": "raw_source_pattern" }),
                ),
            ));
        }

        // End of function (simple heuristic)
        if trimmed == "}" || trimmed.ends_with("}") {
            in_function = false;
            function_name.clear();
        }
    }

    // If no contracts detected, create one placeholder test to show synthesis is working
    if tests.is_empty() && source.contains("fn ") {
        let test_name = format!(
            "synthesized/contract/{}/contract-scan",
            path.file_stem().unwrap_or_default().to_string_lossy()
        );
        tests.push(deferred_result(
            path,
            TestSource::Contract,
            test_name.clone(),
            "deferred: no lowered executable contract metadata found in file",
            fallback_repro(
                path,
                TestSource::Contract,
                test_name,
                json!({ "source": "contract", "oracle": "none", "fallback": "raw_source_scan" }),
            ),
        ));
    }

    tests
}

/// Generate deferred raw-source fallback rows for obligation-like syntax.
///
/// These fallback rows are deferred skips. Executable obligation lifecycle rows
/// require explicit finite lifecycle world metadata from a structured runner
/// snapshot.
///
/// These tests are labeled `source: synthesized:obligation`.
pub fn synthesize_obligation_tests(path: &Path, source: &str) -> Vec<TestResult> {
    let mut tests = Vec::new();

    // Look for obligation declarations and usage
    let oblige_count = source.matches("oblige").count();
    let check_count = source.matches("check").count();

    // Synthesize lifecycle tests based on obligation patterns found
    if oblige_count > 0 || check_count > 0 || source.contains("Obligation") {
        // Obligation introduced test
        let introduced_name = format!(
            "synthesized/obligation/{}/introduced",
            path.file_stem().unwrap_or_default().to_string_lossy()
        );
        tests.push(deferred_result(
            path,
            TestSource::Obligation,
            introduced_name.clone(),
            format!(
                "deferred: raw-source obligation patterns ({} oblige / {} check) lack executable lifecycle metadata",
                oblige_count, check_count
            ),
            fallback_repro(
                path,
                TestSource::Obligation,
                introduced_name,
                json!({ "source": "obligation", "oracle": "introduced", "fallback": "raw_source_pattern" }),
            ),
        ));

        // Obligation discharged test
        let discharged_name = format!(
            "synthesized/obligation/{}/discharged",
            path.file_stem().unwrap_or_default().to_string_lossy()
        );
        tests.push(deferred_result(
            path,
            TestSource::Obligation,
            discharged_name.clone(),
            "deferred: raw-source obligation pattern lacks executable discharge lifecycle metadata",
            fallback_repro(
                path,
                TestSource::Obligation,
                discharged_name,
                json!({ "source": "obligation", "oracle": "discharged", "fallback": "raw_source_pattern" }),
            ),
        ));

        // Double-discharge detection test
        let double_name = format!(
            "synthesized/obligation/{}/double-discharge-detected",
            path.file_stem().unwrap_or_default().to_string_lossy()
        );
        tests.push(deferred_result(
            path,
            TestSource::Obligation,
            double_name.clone(),
            "deferred: raw-source obligation pattern lacks executable double-discharge lifecycle metadata",
            fallback_repro(
                path,
                TestSource::Obligation,
                double_name,
                json!({ "source": "obligation", "oracle": "double_discharge", "fallback": "raw_source_pattern" }),
            ),
        ));
    } else {
        // No obligations detected - add a skip test to show synthesis ran
        let test_name = format!(
            "synthesized/obligation/{}/obligation-scan",
            path.file_stem().unwrap_or_default().to_string_lossy()
        );
        tests.push(deferred_result(
            path,
            TestSource::Obligation,
            test_name.clone(),
            "deferred: no executable obligation lifecycle metadata found in file",
            fallback_repro(
                path,
                TestSource::Obligation,
                test_name,
                json!({ "source": "obligation", "oracle": "none", "fallback": "raw_source_scan" }),
            ),
        ));
    }

    tests
}

#[cfg(test)]
mod tests;
