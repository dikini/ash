//! Value-domain metadata for the independent QuickCheck runner.

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
}
