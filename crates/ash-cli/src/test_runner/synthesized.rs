//! Synthesized test generation from contracts, policies, obligations, and laws.
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

use crate::test_runner::types::{TestResult, TestSource};

/// Runner-facing synthesized-case schema version.
pub const RUNNER_SYNTHESIS_SCHEMA_VERSION: &str = "ash-synthesized-v1.0";

/// Maximum explicitly materialized small-world product axes.
const SMALLWORLD_MAX_PRODUCT_AXES: usize = 16;

/// Default generated worlds for law-derived small-world checks when no runner cap is supplied.
const LAW_SMALLWORLD_DEFAULT_MAX_WORLDS: usize = 8;

/// Maximum explicitly materialized small-world list length.
const SMALLWORLD_MAX_LIST_LEN: usize = 16;

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

    let policy_targets = policy_targets_from_module(module);
    if policy_targets.is_empty() {
        rows.push(IntrospectionUnsupportedReason {
            source_kind: "policy".to_string(),
            target_name: path_stem(path),
            reason: "live checked snapshot has no lowered executable policy metadata exposed"
                .to_string(),
        });
    } else {
        rows.extend(
            policy_targets
                .into_iter()
                .map(|target_name| IntrospectionUnsupportedReason {
                    source_kind: "policy".to_string(),
                    target_name,
                    reason: "live checked snapshot identified policy-like source metadata, but executable lowered policy metadata is not exposed for TASK-1012".to_string(),
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
        Requirement::HasCapability { .. } | Requirement::HasRole(_) => None,
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

fn policy_targets_from_module(module: &ModuleFile) -> Vec<String> {
    let mut targets = module
        .definitions
        .iter()
        .filter_map(|definition| match definition {
            Definition::Policy(policy) => Some(policy.name.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    targets.sort();
    targets.dedup();
    targets
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

mod execution;
pub use execution::{
    SynthesizedCase, SynthesizedInputs, SynthesizedOracle, execute_synthesized_case,
};

pub(crate) mod eval;

mod repro;
use repro::{
    deferred_result, fallback_repro, repro_artifact, snapshot_source_label, source_from_label,
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
    seed: Option<u64>,
    max_cases: Option<usize>,
    max_worlds: Option<usize>,
) -> Vec<TestResult> {
    let mut results = Vec::new();

    results.extend(generated_property_results(path, snapshot, seed, max_cases));

    for contract in &snapshot.contracts {
        let cases = contract_requires_cases(path, snapshot, contract);
        if cases.is_empty() && !contract.lowered_requires.is_empty() {
            results.push(deferred_result(
                path,
                TestSource::Contract,
                format!(
                    "synthesized/contract/{}/requires-deferred",
                    contract.callable_name
                ),
                "deferred: contract metadata lacks exact bounded representatives for executable requires oracle",
                repro_artifact(
                    path,
                    snapshot.source_artifact_id.clone(),
                    snapshot.check_summary_id.clone(),
                    format!("contract:{}:requires-deferred", contract.id),
                    0,
                    1,
                    None,
                    json!({ "source": "contract", "target": contract.callable_name, "oracle": "requires" }),
                    None,
                ),
            ));
        }

        results.extend(cases.iter().map(execute_synthesized_case));

        let postcondition_cases = contract_postcondition_cases(path, snapshot, contract);
        if postcondition_cases.is_empty() && !contract.lowered_ensures.is_empty() {
            results.push(deferred_contract_postcondition_result(
                path, snapshot, contract,
            ));
        }
        results.extend(postcondition_cases.iter().map(execute_synthesized_case));
    }

    for policy in &snapshot.policies {
        let cases = policy_terminal_cases(path, snapshot, policy);
        if cases.is_empty() {
            let reason = policy_terminal_deferred_reason(policy);
            results.push(deferred_result(
                path,
                TestSource::Policy,
                format!("synthesized/policy/{}/deferred", policy.policy_name),
                format!("deferred: {reason}"),
                repro_artifact(
                    path,
                    snapshot.source_artifact_id.clone(),
                    snapshot.check_summary_id.clone(),
                    format!("policy:{}:deferred", policy.id),
                    0,
                    1,
                    None,
                    json!({
                        "source": "policy",
                        "target": policy.policy_name,
                        "terminals": policy.supported_terminal_outcomes,
                        "oracle_shape": policy.oracle_shape,
                        "reason": reason,
                    }),
                    None,
                ),
            ));
        }
        results.extend(cases.iter().map(execute_synthesized_case));
    }

    for obligation in &snapshot.obligations {
        let cases = obligation_lifecycle_cases(path, snapshot, obligation);
        if cases.is_empty() {
            results.push(deferred_result(
                path,
                TestSource::Obligation,
                format!(
                    "synthesized/obligation/{}/lifecycle-deferred",
                    obligation.obligation_name
                ),
                "deferred: obligation metadata lacks complete finite lifecycle metadata",
                repro_artifact(
                    path,
                    snapshot.source_artifact_id.clone(),
                    snapshot.check_summary_id.clone(),
                    format!("obligation:{}:deferred", obligation.id),
                    0,
                    1,
                    None,
                    json!({
                        "source": "obligation",
                        "target": obligation.obligation_name,
                        "expectations": obligation.terminal_expectations,
                    }),
                    None,
                ),
            ));
        }
        results.extend(cases.iter().map(execute_synthesized_case));
    }

    results.extend(algebra_law_profile_results(path, snapshot, seed, max_cases));
    results.extend(law_property_results(path, snapshot, seed, max_cases));
    results.extend(smallworld_results(path, snapshot, seed, max_worlds));
    results.extend(law_smallworld_results(path, snapshot, seed, max_worlds));

    for unsupported in &snapshot.unsupported {
        results.push(deferred_result(
            path,
            source_from_label(&unsupported.source_kind),
            format!(
                "synthesized/{}/{}/unsupported",
                unsupported.source_kind, unsupported.target_name
            ),
            format!("deferred: {}", unsupported.reason),
            repro_artifact(
                path,
                snapshot.source_artifact_id.clone(),
                snapshot.check_summary_id.clone(),
                format!(
                    "{}:{}:unsupported",
                    unsupported.source_kind, unsupported.target_name
                ),
                0,
                1,
                None,
                json!({
                    "source": unsupported.source_kind,
                    "target": unsupported.target_name,
                    "reason": unsupported.reason,
                    "snapshot_source": snapshot_source_label(snapshot),
                }),
                None,
            ),
        ));
    }

    results
}

mod property;
use property::generated_property_results;

mod law;
pub(crate) use law::authored_law_test_results;
use law::{algebra_law_profile_results, law_property_results, law_smallworld_results};

mod smallworld;
use smallworld::smallworld_results;

pub(crate) mod value_generation;

mod contract;
use contract::{
    contract_postcondition_cases, contract_requires_cases, deferred_contract_postcondition_result,
    expression_parameter,
};

mod policy;
use policy::{policy_terminal_cases, policy_terminal_deferred_reason};

mod obligation;
use obligation::obligation_lifecycle_cases;

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

/// Generate deferred raw-source fallback rows for policy-like syntax.
///
/// These fallback rows are deferred skips. Executable policy synthesized tests
/// require structured runner metadata and bounded oracle inputs.
///
/// These tests are labeled `source: synthesized:policy`.
pub fn synthesize_policy_tests(path: &Path, source: &str) -> Vec<TestResult> {
    let mut tests = Vec::new();

    // Look for policy definitions
    let lines: Vec<&str> = source.lines().collect();

    for line in &lines {
        let trimmed = line.trim();

        // Detect policy declarations
        if trimmed.starts_with("policy ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let policy_name = parts[1].trim_end_matches('{').to_string();

                // Synthesize allow case test
                let allow_name = format!("synthesized/policy/{}/allow-case", policy_name);
                tests.push(deferred_result(
                    path,
                    TestSource::Policy,
                    allow_name.clone(),
                    "deferred: raw-source policy pattern lacks bounded executable allow oracle",
                    fallback_repro(
                        path,
                        TestSource::Policy,
                        allow_name,
                        json!({ "source": "policy", "oracle": "allow", "fallback": "raw_source_pattern" }),
                    ),
                ));

                // Synthesize deny case test
                let deny_name = format!("synthesized/policy/{}/deny-case", policy_name);
                tests.push(deferred_result(
                    path,
                    TestSource::Policy,
                    deny_name.clone(),
                    "deferred: raw-source policy pattern lacks bounded executable deny oracle",
                    fallback_repro(
                        path,
                        TestSource::Policy,
                        deny_name,
                        json!({ "source": "policy", "oracle": "deny", "fallback": "raw_source_pattern" }),
                    ),
                ));
            }
        }
    }

    // If no policies detected, create one placeholder test
    if tests.is_empty() && source.contains("policy ") {
        let test_name = format!(
            "synthesized/policy/{}/policy-scan",
            path.file_stem().unwrap_or_default().to_string_lossy()
        );
        tests.push(deferred_result(
            path,
            TestSource::Policy,
            test_name.clone(),
            "deferred: policy syntax detected without bounded executable metadata",
            fallback_repro(
                path,
                TestSource::Policy,
                test_name,
                json!({ "source": "policy", "oracle": "unknown", "fallback": "raw_source_scan" }),
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
