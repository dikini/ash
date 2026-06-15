//! QuickCheck-style strategy metadata and bounded materialization.
//!
//! Phase 150 keeps the public Ash surface in `test::quickcheck` while the
//! first runner slice materializes deterministic bounded representatives from
//! either the default `Arbitrary<T>` strategy or explicit metadata overrides.

use serde::Serialize;
use serde_json::{Value, json};

use crate::test_runner::synthesized::value_generation::{
    GeneratedValueDomain, generated_domain_for_param,
};
use crate::test_runner::synthesized::{TypeGeneratorDescriptor, TypeGeneratorSource};

/// Metadata bridge for `-- @test strategy <binding>: <strategy-path>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickCheckStrategyOverride {
    /// Property/law parameter binding to populate.
    pub binding: String,
    /// Ash strategy function/path, for example `test::quickcheck::sorted_int_lists`.
    pub strategy_path: String,
}

/// Runner-readable strategy descriptor recorded in generated repro artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuickCheckStrategyDescriptor {
    /// Stable strategy identity.
    pub strategy_id: String,
    /// Binding populated by the strategy.
    pub binding: String,
    /// Target Ash type.
    pub target_type: String,
    /// Whether this is default `Arbitrary<T>` evidence or an explicit override.
    pub domain_role: String,
    /// Coherence contract for `Arbitrary<T>` and `Strategy<T>`.
    pub law_coherence: String,
}

/// Resolve a parameter declaration using an optional explicit strategy override.
pub fn domain_for_param_with_strategy(
    param: &str,
    strategy_path: Option<&str>,
) -> Option<GeneratedValueDomain> {
    let (binding, type_name) = parse_param(param)?;
    let Some(strategy_path) = strategy_path else {
        let mut domain = generated_domain_for_param(param)?;
        let descriptor = strategy_descriptor(binding, type_name, None)?;
        domain.descriptor.id = format!("strategy:{binding}:{}", descriptor.strategy_id);
        domain.descriptor.seed_policy = Some("quickcheck_arbitrary_default_v1".to_string());
        return Some(domain);
    };

    let values = strategy_values(strategy_path, type_name)?;
    strategy_descriptor(binding, type_name, Some(strategy_path))?;
    Some(GeneratedValueDomain {
        binding: binding.to_string(),
        type_name: type_name.to_string(),
        descriptor: TypeGeneratorDescriptor {
            id: format!("strategy:{binding}:{strategy_path}"),
            target_type: type_name.to_string(),
            source: TypeGeneratorSource::FiniteDomain,
            exact_values: values.clone(),
            seed_policy: Some(format!("quickcheck_strategy_override_v1:{strategy_path}")),
            max_cases: Some(values.len()),
            unsupported_reason: None,
        },
        values,
    })
}

/// Describe the selected strategy without materializing a full domain.
pub fn strategy_descriptor(
    binding: &str,
    type_name: &str,
    strategy_path: Option<&str>,
) -> Option<QuickCheckStrategyDescriptor> {
    let (strategy_id, domain_role) = match strategy_path {
        Some(path) if strategy_values(path, type_name).is_some() => {
            (path.to_string(), "explicit_strategy_override".to_string())
        }
        Some(_) => return None,
        None => (
            format!("test::quickcheck::arbitrary<{type_name}>"),
            "arbitrary_default".to_string(),
        ),
    };
    Some(QuickCheckStrategyDescriptor {
        strategy_id,
        binding: binding.to_string(),
        target_type: type_name.to_string(),
        domain_role,
        law_coherence: "gen/shrink project from arbitrary()".to_string(),
    })
}

fn parse_param(param: &str) -> Option<(&str, &str)> {
    let (binding, type_name) = param.split_once(':')?;
    Some((binding.trim(), type_name.trim()))
}

fn strategy_values(strategy_path: &str, type_name: &str) -> Option<Vec<Value>> {
    match (strategy_path, type_name) {
        ("test::quickcheck::ints" | "test::quickcheck::small_ints", "Int") => {
            Some(vec![json!(-1), json!(0), json!(1), json!(2)])
        }
        ("test::quickcheck::positive_ints", "Int") => Some(vec![json!(1), json!(2), json!(3)]),
        ("test::quickcheck::nonzero_ints", "Int") => Some(vec![json!(-1), json!(1), json!(2)]),
        ("test::quickcheck::bools", "Bool") => Some(vec![json!(false), json!(true)]),
        ("test::quickcheck::strings" | "test::quickcheck::identifiers", "String") => {
            Some(vec![json!(""), json!("ash"), json!("counterexample")])
        }
        ("test::quickcheck::sorted_int_lists", "List<Int>") => Some(vec![
            json!([]),
            json!([-1]),
            json!([-1, 0]),
            json!([0, 1, 2]),
        ]),
        ("test::quickcheck::nonempty_int_lists", "List<Int>") => {
            Some(vec![json!([0]), json!([-1, 0]), json!([1, 2])])
        }
        _ => None,
    }
}
