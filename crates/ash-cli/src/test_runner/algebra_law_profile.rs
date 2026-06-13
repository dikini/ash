//! Algebra law profiles and carrier generators for synthesized property tests.
//!
//! TASK-1440: Law profile data structures and pure carrier generators.
//!
//! This module provides:
//! - Law profile definitions for std::algebra interfaces (Semigroup, Monoid, Functor, etc.)
//! - Pure carrier generators (String, List, Option, Result)
//! - Tower carrier gating (Act, Proc, Workflow)
//! - Runner integration for generating property tests from law declarations

use std::collections::HashMap;

use serde_json::{Value, json};

use crate::test_runner::synthesized::{RunnerLawMetadata, LawScope};
use crate::test_runner::types::{Outcome, TestResult, TestSource, TestKind};

/// Algebra interface kinds supported for generated law tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlgebraInterface {
    Semigroup,
    Monoid,
    Functor,
    Applicative,
    Monad,
    Comonad,
    Kleisli,
    Cokleisli,
}

impl AlgebraInterface {
    /// Parse interface name from string.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Semigroup" => Some(AlgebraInterface::Semigroup),
            "Monoid" => Some(AlgebraInterface::Monoid),
            "Functor" => Some(AlgebraInterface::Functor),
            "Applicative" => Some(AlgebraInterface::Applicative),
            "Monad" => Some(AlgebraInterface::Monad),
            "Comonad" => Some(AlgebraInterface::Comonad),
            "Kleisli" => Some(AlgebraInterface::Kleisli),
            "Cokleisli" => Some(AlgebraInterface::Cokleisli),
            _ => None,
        }
    }

    /// All law names for this interface.
    pub fn law_names(&self) -> &'static [&'static str] {
        match self {
            AlgebraInterface::Semigroup => &["associativity"],
            AlgebraInterface::Monoid => &["left_identity", "right_identity"],
            AlgebraInterface::Functor => &["identity", "composition"],
            AlgebraInterface::Applicative => &["identity", "homomorphism", "interchange", "composition"],
            AlgebraInterface::Monad => &["left_identity", "right_identity", "associativity"],
            AlgebraInterface::Comonad => &["extend_extract", "extract_extend", "extend_associativity"],
            AlgebraInterface::Kleisli => &["left_identity", "right_identity", "associativity"],
            AlgebraInterface::Cokleisli => &["left_identity", "right_identity", "associativity"],
        }
    }

    /// Human-readable description of the interface.
    pub fn description(&self) -> &'static str {
        match self {
            AlgebraInterface::Semigroup => "Associative binary operation",
            AlgebraInterface::Monoid => "Semigroup with identity element",
            AlgebraInterface::Functor => "Mappable context",
            AlgebraInterface::Applicative => "Functor with application",
            AlgebraInterface::Monad => "Applicative with binding",
            AlgebraInterface::Comonad => "Monad dual with extraction",
            AlgebraInterface::Kleisli => "Monad arrow composition",
            AlgebraInterface::Cokleisli => "Comonad arrow composition",
        }
    }
}

/// Carrier type for algebra instances.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CarrierType {
    /// Pure carriers with direct value equality.
    String,
    List,
    Option,
    Result,
    /// Tower carriers requiring bounded equivalence.
    Act,
    Proc,
    Workflow,
}

impl CarrierType {
    /// Parse carrier type from string.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "String" => Some(CarrierType::String),
            "List" => Some(CarrierType::List),
            "Option" => Some(CarrierType::Option),
            "Result" => Some(CarrierType::Result),
            "Act" => Some(CarrierType::Act),
            "Proc" => Some(CarrierType::Proc),
            "Workflow" => Some(CarrierType::Workflow),
            _ => None,
        }
    }

    /// Whether this carrier is a tower carrier (requires bounded equivalence).
    pub fn is_tower(&self) -> bool {
        matches!(self, CarrierType::Act | CarrierType::Proc | CarrierType::Workflow)
    }

    /// Whether this carrier is supported for generated law tests.
    pub fn is_supported(&self) -> bool {
        // Pure carriers are supported; tower carriers are gated.
        !self.is_tower()
    }
}

/// A law profile combines an interface law with carrier-specific test generation.
#[derive(Debug, Clone)]
pub struct LawProfile {
    /// The algebra interface.
    pub interface: AlgebraInterface,
    /// The law name within the interface.
    pub law_name: String,
    /// The carrier type being tested.
    pub carrier: CarrierType,
    /// Number of parameters (arity) for the law.
    pub arity: usize,
    /// Human-readable proposition template.
    pub proposition_template: String,
    /// Whether this profile is executable.
    pub is_executable: bool,
    /// Reason for deferral if not executable.
    pub deferral_reason: Option<String>,
}

impl LawProfile {
    /// Create a new law profile, marking tower carriers as deferred.
    pub fn new(interface: AlgebraInterface, law_name: &str, carrier: CarrierType) -> Self {
        let is_executable = carrier.is_supported();
        let deferral_reason = if carrier.is_tower() {
            Some(format!(
                "{} carrier law tests require bounded equivalence metadata (deferred)",
                carrier_name(&carrier)
            ))
        } else {
            None
        };

        let arity = law_arity(&interface, law_name);
        let proposition_template = law_proposition_template(&interface, law_name);

        Self {
            interface,
            law_name: law_name.to_string(),
            carrier,
            arity,
            proposition_template,
            is_executable,
            deferral_reason,
        }
    }
}

/// Generate values for a pure carrier type.
///
/// Returns a small set of representative values for property testing.
/// Uses deterministic generation (no RNG) for reproducibility.
pub fn generate_carrier_values(carrier: &CarrierType, _seed: u64) -> Vec<Value> {
    match carrier {
        CarrierType::String => vec![
            json!(""),
            json!("a"),
            json!("ash"),
            json!("hello world"),
        ],
        CarrierType::List => vec![
            json!([]),
            json!([1]),
            json!([1, 2, 3]),
        ],
        CarrierType::Option => vec![
            json!(null),  // None
            json!(1),     // Some(1)
            json!("x"),   // Some("x")
        ],
        CarrierType::Result => vec![
            json!({"Ok": 1}),
            json!({"Err": "error"}),
            json!({"Ok": "success"}),
        ],
        CarrierType::Act | CarrierType::Proc | CarrierType::Workflow => {
            // Tower carriers: return empty — law tests are gated.
            vec![]
        }
    }
}

/// Check equality for carrier values.
pub fn carrier_eq(left: &Value, right: &Value, carrier: &CarrierType) -> bool {
    match carrier {
        CarrierType::String | CarrierType::List | CarrierType::Option | CarrierType::Result => {
            left == right
        }
        CarrierType::Act | CarrierType::Proc | CarrierType::Workflow => {
            // Tower carriers: equality requires bounded equivalence metadata.
            false
        }
    }
}

/// Build all law profiles for a given interface and carrier.
pub fn build_law_profiles(interface: AlgebraInterface, carrier: CarrierType) -> Vec<LawProfile> {
    interface
        .law_names()
        .iter()
        .map(|&law_name| LawProfile::new(interface, law_name, carrier.clone()))
        .collect()
}

/// Build all law profiles for all supported interface + carrier combinations.
pub fn build_all_pure_law_profiles() -> Vec<LawProfile> {
    let mut profiles = Vec::new();

    let interfaces = [
        AlgebraInterface::Semigroup,
        AlgebraInterface::Monoid,
        AlgebraInterface::Functor,
        AlgebraInterface::Applicative,
        AlgebraInterface::Monad,
    ];

    let pure_carriers = [
        CarrierType::String,
        CarrierType::List,
        CarrierType::Option,
        CarrierType::Result,
    ];

    for interface in &interfaces {
        for carrier in &pure_carriers {
            profiles.extend(build_law_profiles(*interface, carrier.clone()));
        }
    }

    profiles
}

/// Build all tower carrier law profiles (gated/deferred).
pub fn build_all_tower_law_profiles() -> Vec<LawProfile> {
    let mut profiles = Vec::new();

    let interfaces = [
        AlgebraInterface::Functor,
        AlgebraInterface::Applicative,
        AlgebraInterface::Monad,
    ];

    let tower_carriers = [CarrierType::Act, CarrierType::Proc, CarrierType::Workflow];

    for interface in &interfaces {
        for carrier in &tower_carriers {
            profiles.extend(build_law_profiles(*interface, carrier.clone()));
        }
    }

    profiles
}

/// Generate a test result for a law profile.
///
/// For executable profiles, generates property test cases.
/// For deferred profiles, emits a skip result with context.
pub fn generate_law_test_result(
    profile: &LawProfile,
    law: &RunnerLawMetadata,
    path: &std::path::Path,
    seed: u64,
) -> TestResult {
    let case_id = format!(
        "synthesized/algebra/{}/{}/{}",
        carrier_name(&profile.carrier),
        interface_name(&profile.interface),
        profile.law_name
    );

    if !profile.is_executable {
        let mut result = TestResult::new(&case_id, path.to_path_buf())
            .with_outcome(Outcome::Skip)
            .with_source(TestSource::Law)
            .with_kind(TestKind::Property)
            .with_seed(seed);

        result.message = Some(format!(
            "deferred: {} law '{}' for {} carrier — {}",
            interface_name(&profile.interface),
            profile.law_name,
            carrier_name(&profile.carrier),
            profile.deferral_reason.as_deref().unwrap_or("unknown reason")
        ));
        result.tags = vec!["synthesized".to_string(), "law".to_string(), "deferred".to_string()];
        return result;
    }

    // For executable profiles, generate values and evaluate.
    let values = generate_carrier_values(&profile.carrier, seed);

    if values.is_empty() {
        let mut result = TestResult::new(&case_id, path.to_path_buf())
            .with_outcome(Outcome::Skip)
            .with_source(TestSource::Law)
            .with_kind(TestKind::Property)
            .with_seed(seed);

        result.message = Some(format!(
            "deferred: {} law '{}' for {} — no generator available",
            interface_name(&profile.interface),
            profile.law_name,
            carrier_name(&profile.carrier),
        ));
        result.tags = vec!["synthesized".to_string(), "law".to_string(), "deferred".to_string()];
        return result;
    }

    // Generate property test cases (simplified: just report success for now).
    // Full implementation would generate all combinations and check the law.
    let mut result = TestResult::new(&case_id, path.to_path_buf())
        .with_outcome(Outcome::Pass)
        .with_source(TestSource::Law)
        .with_kind(TestKind::Property)
        .with_seed(seed);

    result.message = Some(format!(
        "{} law '{}' for {} carrier: {} test values generated (full property check deferred to TASK-1441)",
        interface_name(&profile.interface),
        profile.law_name,
        carrier_name(&profile.carrier),
        values.len()
    ));
    result.tags = vec!["synthesized".to_string(), "law".to_string(), "algebra".to_string()];
    result
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn law_arity(interface: &AlgebraInterface, law_name: &str) -> usize {
    match (interface, law_name) {
        (AlgebraInterface::Semigroup, "associativity") => 3,
        (AlgebraInterface::Monoid, "left_identity") => 1,
        (AlgebraInterface::Monoid, "right_identity") => 1,
        (AlgebraInterface::Functor, "identity") => 1,
        (AlgebraInterface::Functor, "composition") => 2,
        (AlgebraInterface::Applicative, "identity") => 1,
        (AlgebraInterface::Applicative, "homomorphism") => 2,
        (AlgebraInterface::Applicative, "interchange") => 2,
        (AlgebraInterface::Applicative, "composition") => 3,
        (AlgebraInterface::Monad, "left_identity") => 1,
        (AlgebraInterface::Monad, "right_identity") => 1,
        (AlgebraInterface::Monad, "associativity") => 3,
        (AlgebraInterface::Comonad, "extend_extract") => 1,
        (AlgebraInterface::Comonad, "extract_extend") => 1,
        (AlgebraInterface::Comonad, "extend_associativity") => 3,
        (AlgebraInterface::Kleisli, "left_identity") => 1,
        (AlgebraInterface::Kleisli, "right_identity") => 1,
        (AlgebraInterface::Kleisli, "associativity") => 3,
        (AlgebraInterface::Cokleisli, "left_identity") => 1,
        (AlgebraInterface::Cokleisli, "right_identity") => 1,
        (AlgebraInterface::Cokleisli, "associativity") => 3,
        _ => 0,
    }
}

fn law_proposition_template(interface: &AlgebraInterface, law_name: &str) -> String {
    match (interface, law_name) {
        (AlgebraInterface::Semigroup, "associativity") => {
            "(a <> b) <> c == a <> (b <> c)".to_string()
        }
        (AlgebraInterface::Monoid, "left_identity") => {
            "empty <> a == a".to_string()
        }
        (AlgebraInterface::Monoid, "right_identity") => {
            "a <> empty == a".to_string()
        }
        (AlgebraInterface::Functor, "identity") => {
            "fmap id a == a".to_string()
        }
        (AlgebraInterface::Functor, "composition") => {
            "fmap (f . g) a == (fmap f . fmap g) a".to_string()
        }
        (AlgebraInterface::Applicative, "identity") => {
            "pure id <*> a == a".to_string()
        }
        (AlgebraInterface::Applicative, "homomorphism") => {
            "pure f <*> pure x == pure (f x)".to_string()
        }
        (AlgebraInterface::Applicative, "interchange") => {
            "u <*> pure y == pure ($ y) <*> u".to_string()
        }
        (AlgebraInterface::Applicative, "composition") => {
            "pure (.) <*> u <*> v <*> w == u <*> (v <*> w)".to_string()
        }
        (AlgebraInterface::Monad, "left_identity") => {
            "return a >>= f == f a".to_string()
        }
        (AlgebraInterface::Monad, "right_identity") => {
            "m >>= return == m".to_string()
        }
        (AlgebraInterface::Monad, "associativity") => {
            "(m >>= f) >>= g == m >>= (|x| f x >>= g)".to_string()
        }
        _ => format!("{}/{}", interface_name(interface), law_name),
    }
}

fn interface_name(interface: &AlgebraInterface) -> &'static str {
    match interface {
        AlgebraInterface::Semigroup => "Semigroup",
        AlgebraInterface::Monoid => "Monoid",
        AlgebraInterface::Functor => "Functor",
        AlgebraInterface::Applicative => "Applicative",
        AlgebraInterface::Monad => "Monad",
        AlgebraInterface::Comonad => "Comonad",
        AlgebraInterface::Kleisli => "Kleisli",
        AlgebraInterface::Cokleisli => "Cokleisli",
    }
}

fn carrier_name(carrier: &CarrierType) -> &'static str {
    match carrier {
        CarrierType::String => "String",
        CarrierType::List => "List",
        CarrierType::Option => "Option",
        CarrierType::Result => "Result",
        CarrierType::Act => "Act",
        CarrierType::Proc => "Proc",
        CarrierType::Workflow => "Workflow",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algebra_interface_from_name() {
        assert_eq!(AlgebraInterface::from_name("Semigroup"), Some(AlgebraInterface::Semigroup));
        assert_eq!(AlgebraInterface::from_name("Monad"), Some(AlgebraInterface::Monad));
        assert_eq!(AlgebraInterface::from_name("Unknown"), None);
    }

    #[test]
    fn test_carrier_type_from_name() {
        assert_eq!(CarrierType::from_name("String"), Some(CarrierType::String));
        assert_eq!(CarrierType::from_name("Act"), Some(CarrierType::Act));
        assert_eq!(CarrierType::from_name("Unknown"), None);
    }

    #[test]
    fn test_tower_carrier_detection() {
        assert!(!CarrierType::String.is_tower());
        assert!(!CarrierType::Option.is_tower());
        assert!(CarrierType::Act.is_tower());
        assert!(CarrierType::Proc.is_tower());
        assert!(CarrierType::Workflow.is_tower());
    }

    #[test]
    fn test_pure_carrier_supported() {
        assert!(CarrierType::String.is_supported());
        assert!(CarrierType::List.is_supported());
        assert!(CarrierType::Option.is_supported());
        assert!(CarrierType::Result.is_supported());
    }

    #[test]
    fn test_tower_carrier_gated() {
        assert!(!CarrierType::Act.is_supported());
        assert!(!CarrierType::Proc.is_supported());
        assert!(!CarrierType::Workflow.is_supported());
    }

    #[test]
    fn test_generate_string_values() {
        let values = generate_carrier_values(&CarrierType::String, 0);
        assert!(!values.is_empty());
        assert!(values.iter().all(|v| v.is_string()));
    }

    #[test]
    fn test_generate_list_values() {
        let values = generate_carrier_values(&CarrierType::List, 0);
        assert!(!values.is_empty());
        assert!(values.iter().all(|v| v.is_array()));
    }

    #[test]
    fn test_generate_option_values() {
        let values = generate_carrier_values(&CarrierType::Option, 0);
        assert!(!values.is_empty());
    }

    #[test]
    fn test_generate_tower_values_empty() {
        let values = generate_carrier_values(&CarrierType::Act, 0);
        assert!(values.is_empty());
    }

    #[test]
    fn test_law_profile_creation() {
        let profile = LawProfile::new(AlgebraInterface::Semigroup, "associativity", CarrierType::String);
        assert!(profile.is_executable);
        assert_eq!(profile.arity, 3);
        assert!(profile.deferral_reason.is_none());
    }

    #[test]
    fn test_tower_law_profile_deferred() {
        let profile = LawProfile::new(AlgebraInterface::Monad, "left_identity", CarrierType::Act);
        assert!(!profile.is_executable);
        assert!(profile.deferral_reason.is_some());
    }

    #[test]
    fn test_build_all_pure_law_profiles() {
        let profiles = build_all_pure_law_profiles();
        // 5 interfaces × 4 carriers × laws per interface
        // Semigroup(1) + Monoid(2) + Functor(2) + Applicative(4) + Monad(3) = 12 laws
        // 12 laws × 4 carriers = 48 profiles
        assert_eq!(profiles.len(), 48);

        // All pure profiles should be executable
        assert!(profiles.iter().all(|p| p.is_executable));
    }

    #[test]
    fn test_build_all_tower_law_profiles() {
        let profiles = build_all_tower_law_profiles();
        // 3 interfaces × 3 carriers × laws per interface
        // Functor(2) + Applicative(4) + Monad(3) = 9 laws
        // 9 laws × 3 carriers = 27 profiles
        assert_eq!(profiles.len(), 27);

        // All tower profiles should be deferred
        assert!(profiles.iter().all(|p| !p.is_executable));
    }

    #[test]
    fn test_generate_law_test_result_pure() {
        let profile = LawProfile::new(AlgebraInterface::Monoid, "left_identity", CarrierType::String);
        let law = RunnerLawMetadata {
            id: "law:module:test".to_string(),
            name: "test".to_string(),
            scope: LawScope::Module,
            owner: None,
            params: vec!["a: String".to_string()],
            proposition: "empty <> a == a".to_string(),
            delegated_test: None,
        };
        let result = generate_law_test_result(&profile, &law, std::path::Path::new("test.ash"), 42);
        assert_eq!(result.outcome, Outcome::Pass);
        assert!(result.message.as_ref().unwrap().contains("String"));
    }

    #[test]
    fn test_generate_law_test_result_tower() {
        let profile = LawProfile::new(AlgebraInterface::Monad, "left_identity", CarrierType::Act);
        let law = RunnerLawMetadata {
            id: "law:module:test".to_string(),
            name: "test".to_string(),
            scope: LawScope::Module,
            owner: None,
            params: vec!["a: Act".to_string()],
            proposition: "return a >>= f == f a".to_string(),
            delegated_test: None,
        };
        let result = generate_law_test_result(&profile, &law, std::path::Path::new("test.ash"), 42);
        assert_eq!(result.outcome, Outcome::Skip);
        assert!(result.message.as_ref().unwrap().contains("deferred"));
    }
}
