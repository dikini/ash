//! Bounded value generation and shrinking for synthesized Ash property cases.

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{Value, json};

use super::{TypeGeneratorDescriptor, TypeGeneratorSource};

/// One generated parameter domain with schema metadata.
#[derive(Debug, Clone, Serialize)]
pub struct GeneratedValueDomain {
    /// Binding populated by this domain.
    pub binding: String,
    /// Source-level type name for the binding.
    pub type_name: String,
    /// Deterministic generated representatives.
    pub values: Vec<Value>,
    /// Runner-facing generator descriptor.
    pub descriptor: TypeGeneratorDescriptor,
}

/// Materialized generated property case.
#[derive(Debug, Clone, Serialize)]
pub struct GeneratedCase {
    /// Case index starting at 1.
    pub case_index: usize,
    /// Binding values for this case.
    pub bindings: BTreeMap<String, Value>,
    /// Generator descriptors used to produce this case.
    pub generators: Vec<TypeGeneratorDescriptor>,
}

/// Shrunk counterexample plus the deterministic shrink trace.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ShrunkCounterexample {
    /// Minimal bindings found by the bounded shrink pass.
    pub bindings: BTreeMap<String, Value>,
    /// Candidate snapshots accepted while shrinking.
    pub trace: Vec<Value>,
}

/// Parse a `name: Type` law/test parameter into a generated value domain.
pub fn generated_domain_for_param(param: &str) -> Option<GeneratedValueDomain> {
    let (name, ty) = param.split_once(':')?;
    let binding = name.trim().to_string();
    let type_name = ty.trim().to_string();
    let values = values_for_type(&type_name)?;
    Some(GeneratedValueDomain {
        descriptor: TypeGeneratorDescriptor {
            id: format!("generator:{binding}:{type_name}"),
            target_type: type_name.clone(),
            source: generator_source_for_type(&type_name),
            exact_values: values.clone(),
            seed_policy: Some("deterministic_phase146_bounded_representatives".to_string()),
            max_cases: Some(values.len()),
            unsupported_reason: None,
        },
        binding,
        type_name,
        values,
    })
}

/// Deterministically materialize the bounded cartesian product of generated domains.
pub fn generated_cases(domains: &[GeneratedValueDomain], limit: usize) -> Vec<GeneratedCase> {
    if limit == 0 {
        return Vec::new();
    }
    if domains.is_empty() {
        return vec![GeneratedCase {
            case_index: 1,
            bindings: BTreeMap::new(),
            generators: Vec::new(),
        }];
    }

    let mut cases = Vec::new();
    let mut bindings = BTreeMap::new();
    append_cases(domains, limit, 0, &mut bindings, &mut cases);
    for (index, case) in cases.iter_mut().enumerate() {
        case.case_index = index + 1;
        case.generators = domains
            .iter()
            .map(|domain| domain.descriptor.clone())
            .collect();
    }
    cases
}

fn append_cases(
    domains: &[GeneratedValueDomain],
    limit: usize,
    axis_index: usize,
    bindings: &mut BTreeMap<String, Value>,
    cases: &mut Vec<GeneratedCase>,
) {
    if cases.len() >= limit {
        return;
    }
    if axis_index == domains.len() {
        cases.push(GeneratedCase {
            case_index: 0,
            bindings: bindings.clone(),
            generators: Vec::new(),
        });
        return;
    }
    let domain = &domains[axis_index];
    for value in &domain.values {
        if cases.len() >= limit {
            return;
        }
        bindings.insert(domain.binding.clone(), value.clone());
        append_cases(domains, limit, axis_index + 1, bindings, cases);
        bindings.remove(&domain.binding);
    }
}

/// Greedily shrink a failing binding set while `still_fails` remains true.
pub fn shrink_bindings(
    original: &BTreeMap<String, Value>,
    still_fails: impl Fn(&BTreeMap<String, Value>) -> bool,
) -> ShrunkCounterexample {
    let mut current = original.clone();
    let mut trace = Vec::new();

    for key in original.keys() {
        let Some(value) = current.get(key).cloned() else {
            continue;
        };
        for candidate in shrink_candidates(&value) {
            if candidate == value {
                continue;
            }
            let mut attempt = current.clone();
            attempt.insert(key.clone(), candidate);
            if still_fails(&attempt) {
                current = attempt;
                trace.push(Value::Object(current.clone().into_iter().collect()));
                break;
            }
        }
    }

    ShrunkCounterexample {
        bindings: current,
        trace,
    }
}

fn values_for_type(type_name: &str) -> Option<Vec<Value>> {
    match type_name {
        "Int" => Some(vec![json!(-1), json!(0), json!(1), json!(2)]),
        "Bool" => Some(vec![json!(false), json!(true)]),
        "String" => Some(vec![json!(""), json!("ash"), json!("counterexample")]),
        _ if type_name.starts_with("List<") && type_name.ends_with('>') => {
            let inner = &type_name[5..type_name.len() - 1];
            let inner_values = values_for_type(inner)?;
            let first = inner_values.first().cloned().unwrap_or(Value::Null);
            let second = inner_values
                .get(1)
                .cloned()
                .unwrap_or_else(|| first.clone());
            Some(vec![
                json!([]),
                Value::Array(vec![first.clone()]),
                Value::Array(vec![first, second]),
            ])
        }
        _ if type_name.starts_with("Option<") && type_name.ends_with('>') => {
            let inner = &type_name[7..type_name.len() - 1];
            let inner_values = values_for_type(inner)?;
            Some(vec![
                Value::Null,
                inner_values.first().cloned().unwrap_or(Value::Null),
                inner_values.get(1).cloned().unwrap_or(Value::Null),
            ])
        }
        _ if type_name.starts_with("Result<") && type_name.ends_with('>') => {
            let inner = &type_name[7..type_name.len() - 1];
            let (ok_ty, err_ty) = split_two_type_args(inner)?;
            let ok_values = values_for_type(ok_ty.trim())?;
            let err_values = values_for_type(err_ty.trim())?;
            Some(vec![
                json!({"Ok": ok_values.first().cloned().unwrap_or(Value::Null)}),
                json!({"Ok": ok_values.get(1).cloned().unwrap_or(Value::Null)}),
                json!({"Err": err_values.first().cloned().unwrap_or(Value::Null)}),
            ])
        }
        _ => None,
    }
}

fn generator_source_for_type(type_name: &str) -> TypeGeneratorSource {
    if matches!(type_name, "Int" | "Bool" | "String") {
        TypeGeneratorSource::PrimitiveDomain
    } else {
        TypeGeneratorSource::AdtContainerDomain
    }
}

fn split_two_type_args(input: &str) -> Option<(&str, &str)> {
    let mut depth = 0usize;
    for (index, ch) in input.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return Some((&input[..index], &input[index + 1..])),
            _ => {}
        }
    }
    None
}

fn shrink_candidates(value: &Value) -> Vec<Value> {
    match value {
        Value::Bool(true) => vec![json!(false)],
        Value::Bool(false) => Vec::new(),
        Value::Number(number) => number
            .as_i64()
            .map(|n| {
                let mut candidates = vec![json!(0)];
                if n.abs() > 1 {
                    candidates.push(json!(n / 2));
                }
                if n != 1 {
                    candidates.push(json!(1));
                }
                candidates
            })
            .unwrap_or_default(),
        Value::String(s) if !s.is_empty() => {
            vec![json!(""), json!(s.chars().next().unwrap().to_string())]
        }
        Value::Array(items) if !items.is_empty() => {
            let mut candidates = vec![json!([])];
            if items.len() > 1 {
                candidates.push(Value::Array(vec![items[0].clone()]));
            }
            if let Some(first) = items.first() {
                for shrunk in shrink_candidates(first) {
                    candidates.push(Value::Array(vec![shrunk]));
                }
            }
            candidates
        }
        Value::Object(fields) => {
            if let Some(ok) = fields.get("Ok") {
                let mut candidates = vec![Value::Null];
                candidates.extend(
                    shrink_candidates(ok)
                        .into_iter()
                        .map(|value| json!({"Ok": value})),
                );
                candidates
            } else if let Some(err) = fields.get("Err") {
                let mut candidates = vec![Value::Null];
                candidates.extend(
                    shrink_candidates(err)
                        .into_iter()
                        .map(|value| json!({"Err": value})),
                );
                candidates
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_primitive_and_container_domains() {
        let int_domain = generated_domain_for_param("x: Int").unwrap();
        assert_eq!(int_domain.binding, "x");
        assert_eq!(
            int_domain.values,
            vec![json!(-1), json!(0), json!(1), json!(2)]
        );
        assert_eq!(
            int_domain.descriptor.source,
            TypeGeneratorSource::PrimitiveDomain
        );

        let list_domain = generated_domain_for_param("xs: List<Int>").unwrap();
        assert_eq!(list_domain.binding, "xs");
        assert_eq!(
            list_domain.descriptor.source,
            TypeGeneratorSource::AdtContainerDomain
        );
        assert_eq!(list_domain.values[0], json!([]));
        assert_eq!(list_domain.values[1], json!([-1]));

        let result_domain = generated_domain_for_param("r: Result<Int, String>").unwrap();
        assert_eq!(result_domain.values[0], json!({"Ok": -1}));
        assert_eq!(result_domain.values[2], json!({"Err": ""}));
    }

    #[test]
    fn generated_cases_are_bounded_and_include_generator_schema() {
        let domains = vec![
            generated_domain_for_param("x: Bool").unwrap(),
            generated_domain_for_param("y: Int").unwrap(),
        ];
        let cases = generated_cases(&domains, 3);
        assert_eq!(cases.len(), 3);
        assert_eq!(cases[0].case_index, 1);
        assert_eq!(cases[0].generators.len(), 2);
        assert_eq!(cases[0].bindings["x"], json!(false));
    }

    #[test]
    fn shrink_bindings_keeps_only_candidates_that_still_fail() {
        let original = BTreeMap::from([("x".to_string(), json!(-4))]);
        let shrunk = shrink_bindings(&original, |bindings| bindings["x"].as_i64().unwrap() <= 0);
        assert_eq!(shrunk.bindings["x"], json!(0));
        assert_eq!(shrunk.trace, vec![json!({"x": 0})]);
    }
}
