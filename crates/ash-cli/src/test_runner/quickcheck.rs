//! QuickCheck-style strategy metadata, deterministic context splitting, and bounded materialization.
//!
//! Phase 151 keeps the Phase 150 metadata bridge as a compatibility input, but
//! makes the runner-facing model explicit: strategy descriptors name ordinary
//! Ash `Strategy<A>` values, generation is anchored by a versioned
//! `GenContext`, and all generated evidence records the effective RNG contract.

use serde::Serialize;
use serde_json::{Value, json};

use crate::test_runner::synthesized::value_generation::{
    GeneratedValueDomain, generated_domain_for_param,
};
use crate::test_runner::synthesized::{TypeGeneratorDescriptor, TypeGeneratorSource};

/// Versioned deterministic RNG/split contract for QuickCheck v1.
pub const QUICKCHECK_RNG_ALGORITHM_V1: &str = "ash-quickcheck-rng-v1";

/// Seed origin recorded in run/replay artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QuickCheckSeedSource {
    /// Runner generated a fresh seed because no external/source seed was supplied.
    Random,
    /// User supplied `--seed` or an equivalent external replay seed.
    Cli,
    /// Source metadata supplied a seed. Allowed, but discouraged/linted.
    Source,
}

impl QuickCheckSeedSource {
    /// Stable lowercase label used by existing JSON snippets.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Random => "random",
            Self::Cli => "cli",
            Self::Source => "source",
        }
    }
}

/// Effective QuickCheck seed policy for a property run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct QuickCheckSeedPolicy {
    /// Effective seed consumed by the runner.
    pub seed: u64,
    /// Where the seed came from.
    pub seed_source: QuickCheckSeedSource,
    /// Source-pinned seed, when present, even if an external seed overrides it.
    pub source_seed: Option<u64>,
    /// RNG/split contract version.
    pub rng_algorithm: &'static str,
}

impl QuickCheckSeedPolicy {
    /// Build effective policy from external CLI/replay seed and source metadata.
    #[must_use]
    pub fn resolve(cli_seed: Option<u64>, source_seed: Option<u64>) -> Self {
        if let Some(seed) = cli_seed {
            return Self {
                seed,
                seed_source: QuickCheckSeedSource::Cli,
                source_seed,
                rng_algorithm: QUICKCHECK_RNG_ALGORITHM_V1,
            };
        }
        if let Some(seed) = source_seed {
            return Self {
                seed,
                seed_source: QuickCheckSeedSource::Source,
                source_seed,
                rng_algorithm: QUICKCHECK_RNG_ALGORITHM_V1,
            };
        }
        Self {
            seed: random_quickcheck_seed(),
            seed_source: QuickCheckSeedSource::Random,
            source_seed: None,
            rng_algorithm: QUICKCHECK_RNG_ALGORITHM_V1,
        }
    }
}

/// Generator-visible configuration for one candidate value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GenContext {
    /// Deterministic RNG state for this context.
    pub seed: u64,
    /// Strategy-specific size parameter.
    pub size: u32,
    /// Human-readable split path for traces/replay.
    pub path: Vec<String>,
}

impl GenContext {
    /// Create a root context.
    #[must_use]
    pub fn root(seed: u64, size: u32) -> Self {
        Self {
            seed,
            size,
            path: Vec::new(),
        }
    }

    /// Deterministically split by ordinal index.
    #[must_use]
    pub fn split(&self, index: u64) -> Self {
        let mixed = splitmix64(self.seed ^ index.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let mut path = self.path.clone();
        path.push(format!("split:{index}"));
        Self {
            seed: mixed,
            size: self.size,
            path,
        }
    }

    /// Deterministically split by a named variant label.
    #[must_use]
    pub fn variant(&self, label: &str) -> Self {
        let label_hash = stable_label_hash(label);
        let mixed = splitmix64(self.seed ^ label_hash);
        let mut path = self.path.clone();
        path.push(format!("variant:{label}"));
        Self {
            seed: mixed,
            size: self.size,
            path,
        }
    }

    /// Deterministically split by a label and element index.
    #[must_use]
    pub fn indexed(&self, label: &str, index: u64) -> Self {
        self.variant(label).split(index)
    }

    /// Return the same RNG path with a different size.
    #[must_use]
    pub fn resize(&self, size: u32) -> Self {
        let mut next = self.clone();
        next.size = size;
        next.path.push(format!("resize:{size}"));
        next
    }

    /// Deterministically choose a boolean from this context.
    #[must_use]
    pub fn choose_bool(&self) -> bool {
        splitmix64(self.seed) & 1 == 1
    }

    /// Deterministically choose an integer in the inclusive range.
    #[must_use]
    pub fn choose_int(&self, min: i64, max: i64) -> i64 {
        if min >= max {
            return min;
        }
        let width = (max as i128 - min as i128 + 1) as u64;
        min + (splitmix64(self.seed) % width) as i64
    }
}

/// Metadata bridge for `-- @test strategy <binding>: <strategy-path>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickCheckStrategyOverride {
    /// Property/law parameter binding to populate.
    pub binding: String,
    /// Ash strategy function/path, for example `test::quickcheck::int::positive`.
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
    /// Versioned RNG/split algorithm used by this descriptor.
    pub rng_algorithm: String,
}

/// Resolve a parameter declaration using an optional explicit strategy override.
pub fn domain_for_param_with_strategy(
    param: &str,
    strategy_path: Option<&str>,
) -> Option<GeneratedValueDomain> {
    resolve_domain_for_param_with_strategy(param, strategy_path, true).ok()
}

/// Resolve a parameter declaration with explicit in-scope `Arbitrary<A>` evidence.
pub fn resolve_domain_for_param_with_strategy(
    param: &str,
    strategy_path: Option<&str>,
    arbitrary_evidence_in_scope: bool,
) -> Result<GeneratedValueDomain, String> {
    let (binding, type_name) = parse_param(param).ok_or_else(|| {
        format!("invalid generated property test: malformed @test params declaration `{param}`")
    })?;
    let Some(strategy_path) = strategy_path else {
        if !arbitrary_evidence_in_scope {
            return Err(format!(
                "invalid generated property test: missing in-scope Arbitrary<{type_name}> evidence; import test::quickcheck::{{Arbitrary}} or test::quickcheck::prelude before using default QuickCheck generation"
            ));
        }
        let mut domain = generated_domain_for_param(param).ok_or_else(|| {
            format!(
                "invalid generated property test: unsupported @test params type domain for Arbitrary<{type_name}>"
            )
        })?;
        let descriptor = strategy_descriptor(binding, type_name, None).ok_or_else(|| {
            format!("invalid generated property test: unsupported Arbitrary<{type_name}> evidence")
        })?;
        domain.descriptor.id = format!("strategy:{binding}:{}", descriptor.strategy_id);
        domain.descriptor.seed_policy = Some(format!(
            "{}:{}",
            QUICKCHECK_RNG_ALGORITHM_V1, "arbitrary_default"
        ));
        return Ok(domain);
    };

    let canonical_path = canonical_strategy_path(strategy_path);
    let values = strategy_values(canonical_path, type_name).ok_or_else(|| {
        format!(
            "invalid generated property test: unsupported @test params type domain or quickcheck strategy `{strategy_path}` for `{binding}: {type_name}`"
        )
    })?;
    strategy_descriptor(binding, type_name, Some(canonical_path)).ok_or_else(|| {
        format!(
            "invalid generated property test: unsupported quickcheck strategy `{strategy_path}` for `{binding}: {type_name}`"
        )
    })?;
    Ok(GeneratedValueDomain {
        binding: binding.to_string(),
        type_name: type_name.to_string(),
        descriptor: TypeGeneratorDescriptor {
            id: format!("strategy:{binding}:{canonical_path}"),
            target_type: type_name.to_string(),
            source: TypeGeneratorSource::FiniteDomain,
            exact_values: values.clone(),
            seed_policy: Some(format!(
                "{}:strategy_override:{canonical_path}",
                QUICKCHECK_RNG_ALGORITHM_V1
            )),
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
        Some(path) => {
            let canonical = canonical_strategy_path(path);
            strategy_values(canonical, type_name)?;
            (
                canonical.to_string(),
                "explicit_strategy_override".to_string(),
            )
        }
        None => (
            format!("test::quickcheck::arbitrary::arbitrary<{type_name}>"),
            "arbitrary_default".to_string(),
        ),
    };
    Some(QuickCheckStrategyDescriptor {
        strategy_id,
        binding: binding.to_string(),
        target_type: type_name.to_string(),
        domain_role,
        law_coherence: "ordinary Strategy<A> gen/shrink selected from in-scope evidence"
            .to_string(),
        rng_algorithm: QUICKCHECK_RNG_ALGORITHM_V1.to_string(),
    })
}

/// Source-seed lint message used by CLI/tests.
#[must_use]
pub fn source_seed_warning(
    source_seed: Option<u64>,
    effective: QuickCheckSeedPolicy,
) -> Option<String> {
    source_seed.map(|seed| {
        let suffix = if effective.seed_source == QuickCheckSeedSource::Cli {
            format!("; overridden by external seed {}", effective.seed)
        } else {
            String::new()
        };
        format!(
            "source-pinned QuickCheck seed {seed} reduces exploration; prefer CLI replay seed or failure artifact replay{suffix}"
        )
    })
}

fn parse_param(param: &str) -> Option<(&str, &str)> {
    let (binding, type_name) = param.split_once(':')?;
    Some((binding.trim(), type_name.trim()))
}

fn canonical_strategy_path(path: &str) -> &str {
    match path {
        "test::quickcheck::ints" | "test::quickcheck::small_ints" => "test::quickcheck::int::any",
        "test::quickcheck::positive_ints" => "test::quickcheck::int::positive",
        "test::quickcheck::nonzero_ints" => "test::quickcheck::int::nonzero",
        "test::quickcheck::bools" => "test::quickcheck::bool::any",
        "test::quickcheck::strings" => "test::quickcheck::string::any",
        "test::quickcheck::identifiers" => "test::quickcheck::string::identifier",
        "test::quickcheck::sorted_int_lists" => "test::quickcheck::list::sorted_ints",
        "test::quickcheck::nonempty_int_lists" => "test::quickcheck::list::nonempty_ints",
        other => other,
    }
}

fn strategy_values(strategy_path: &str, type_name: &str) -> Option<Vec<Value>> {
    match (strategy_path, type_name) {
        ("test::quickcheck::int::any" | "test::quickcheck::int::small", "Int") => {
            Some(vec![json!(-1), json!(0), json!(1), json!(2)])
        }
        ("test::quickcheck::int::positive", "Int") => Some(vec![json!(1), json!(2), json!(3)]),
        ("test::quickcheck::int::nonzero", "Int") => Some(vec![json!(-1), json!(1), json!(2)]),
        ("test::quickcheck::bool::any", "Bool") => Some(vec![json!(false), json!(true)]),
        ("test::quickcheck::string::any" | "test::quickcheck::string::identifier", "String") => {
            Some(vec![json!(""), json!("ash"), json!("counterexample")])
        }
        ("test::quickcheck::list::sorted_ints", "List<Int>") => Some(vec![
            json!([]),
            json!([-1]),
            json!([-1, 0]),
            json!([0, 1, 2]),
        ]),
        ("test::quickcheck::list::nonempty_ints", "List<Int>") => {
            Some(vec![json!([0]), json!([-1, 0]), json!([1, 2])])
        }
        _ => None,
    }
}

fn random_quickcheck_seed() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0xA5A5_A5A5_5A5A_5A5A);
    splitmix64(nanos ^ std::process::id() as u64)
}

fn stable_label_hash(label: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in label.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    hash
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_context_split_helpers_are_stable() {
        let root = GenContext::root(123, 20);
        let left = root.variant("left").indexed("elem", 2).resize(7);
        let again = GenContext::root(123, 20)
            .variant("left")
            .indexed("elem", 2)
            .resize(7);
        assert_eq!(left, again);
        assert_eq!(left.size, 7);
        assert_eq!(
            left.path,
            vec!["variant:left", "variant:elem", "split:2", "resize:7"]
        );
        assert_ne!(root.split(0).seed, root.split(1).seed);
    }

    #[test]
    fn source_seed_policy_is_overridden_by_cli_seed() {
        let source = QuickCheckSeedPolicy::resolve(None, Some(1));
        assert_eq!(source.seed, 1);
        assert_eq!(source.seed_source, QuickCheckSeedSource::Source);

        let cli = QuickCheckSeedPolicy::resolve(Some(99), Some(1));
        assert_eq!(cli.seed, 99);
        assert_eq!(cli.source_seed, Some(1));
        assert_eq!(cli.seed_source, QuickCheckSeedSource::Cli);
        assert!(
            source_seed_warning(Some(1), cli)
                .unwrap()
                .contains("overridden by external seed 99")
        );
    }

    #[test]
    fn canonical_strategy_paths_accept_alpha_aliases() {
        let old = domain_for_param_with_strategy("x: Int", Some("test::quickcheck::positive_ints"))
            .expect("alpha alias should canonicalize");
        let new = domain_for_param_with_strategy("x: Int", Some("test::quickcheck::int::positive"))
            .expect("canonical strategy should resolve");
        assert_eq!(old.values, new.values);
        assert_eq!(
            old.descriptor.id,
            "strategy:x:test::quickcheck::int::positive"
        );
    }
}
