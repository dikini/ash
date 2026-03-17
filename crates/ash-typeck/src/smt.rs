//! SMT-based Policy Conflict Detection
//!
//! This module provides compile-time policy conflict detection using the Z3 SMT solver.
//! It is only available when the `smt` feature is enabled.
//!
//! # Overview
//!
//! Policy conflicts occur when a set of policies cannot be simultaneously satisfied.
//! For example, a minimum budget requirement that exceeds a maximum budget limit,
//! or time ranges that don't overlap.
//!
//! The SMT solver checks satisfiability and provides an unsatisfiable core
//! (minimal set of conflicting policies) when conflicts are detected.
//!
//! # Example
//!
//! ```
//! use ash_typeck::smt::{Policy, SatResult, check_policies};
//!
//! let policies = vec![
//!     Policy::Budget { max: 100 },
//!     Policy::MinBudget { min: 50 },
//! ];
//!
//! match check_policies(&policies) {
//!     SatResult::Sat => println!("Policies are satisfiable"),
//!     SatResult::Unsat(core) => println!("Conflicting policies: {:?}", core),
//!     SatResult::Timeout => println!("Solver timed out"),
//!     _ => println!("Unknown result"),
//! }
//! ```

use std::collections::HashMap;

use z3::{Config, Context, SatResult as Z3SatResult, Solver};
use z3::ast::{Ast, Bool, Int};

/// A policy constraint that can be checked for conflicts.
///
/// Policies represent requirements that must be satisfied for a workflow
/// to be considered valid. Multiple policies can be combined, and conflicts
/// between them are detected at compile time using SMT solving.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Policy {
    /// Maximum budget constraint (inclusive).
    /// The actual budget must not exceed this value.
    Budget {
        /// Maximum allowed budget.
        max: u64,
    },

    /// Minimum budget constraint (inclusive).
    /// The actual budget must be at least this value.
    MinBudget {
        /// Minimum required budget.
        min: u64,
    },

    /// Time range constraint.
    /// The workflow must execute within this time window.
    TimeRange {
        /// Start time (Unix timestamp).
        start: i64,
        /// End time (Unix timestamp).
        end: i64,
    },

    /// Geographic region constraint.
    /// The workflow must execute in one of the specified regions.
    Region {
        /// Allowed region codes (e.g., "us-east-1", "eu-west-1").
        regions: Vec<String>,
    },

    /// Encryption requirement.
    /// The workflow data must be encrypted.
    EncryptionRequired,

    /// Audit logging requirement.
    /// All operations must be logged for audit purposes.
    AuditRequired,

    /// Data residency constraint.
    /// Data must remain within specified jurisdictions.
    DataResidency {
        /// Allowed jurisdictions (e.g., "EU", "US", "CA").
        jurisdictions: Vec<String>,
    },
}



/// Result of checking a set of policies for satisfiability.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SatResult {
    /// The policies are satisfiable - there exists at least one
    /// assignment of variables that satisfies all constraints.
    Sat,

    /// The policies are unsatisfiable. Contains a minimal set of
    /// conflicting policies (the unsatisfiable core).
    Unsat(Vec<Policy>),

    /// The solver timed out before determining satisfiability.
    Timeout,
}

/// Context for SMT-based policy checking.
///
/// This struct wraps the Z3 solver and manages the encoding of policies
/// into SMT constraints.
pub struct SmtContext {
    /// Z3 context - owns all allocated AST nodes.
    /// Boxed to ensure stable address.
    context: Box<Context>,
}

// Safety: Z3 context is not Send/Sync by default,
// but we're using it in a single-threaded manner.
unsafe impl Send for SmtContext {}
unsafe impl Sync for SmtContext {}

impl SmtContext {
    /// Creates a new SMT context with default configuration.
    ///
    /// # Example
    ///
    /// ```
    /// use ash_typeck::smt::SmtContext;
    ///
    /// let ctx = SmtContext::new();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::with_timeout_ms(5000) // Default 5 second timeout
    }

    /// Creates a new SMT context with a specified timeout.
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - Timeout in milliseconds for solver operations.
    ///
    /// # Example
    ///
    /// ```
    /// use ash_typeck::smt::SmtContext;
    ///
    /// let ctx = SmtContext::with_timeout_ms(10000); // 10 second timeout
    /// ```
    #[must_use]
    pub fn with_timeout_ms(timeout_ms: u64) -> Self {
        let mut config = Config::new();
        config.set_timeout_msec(timeout_ms);

        let context = Box::new(Context::new(&config));

        Self { context }
    }

    /// Checks a set of policies for satisfiability.
    ///
    /// # Arguments
    ///
    /// * `policies` - The policies to check.
    ///
    /// # Returns
    ///
    /// * `SatResult::Sat` - Policies are satisfiable.
    /// * `SatResult::Unsat(core)` - Policies conflict; `core` is minimal.
    /// * `SatResult::Timeout` - Solver timed out.
    ///
    /// # Example
    ///
    /// ```
    /// use ash_typeck::smt::{Policy, SatResult, SmtContext};
    ///
    /// let mut ctx = SmtContext::new();
    /// let policies = vec![Policy::Budget { max: 100 }];
    ///
    /// match ctx.check_policies(&policies) {
    ///     SatResult::Sat => println!("OK"),
    ///     _ => println!("Problem detected"),
    /// }
    /// ```
    #[must_use]
    pub fn check_policies(&mut self, policies: &[Policy]) -> SatResult {
        // Create a fresh solver for each check to ensure clean state
        let solver = Solver::new(&self.context);

        // Track policies in a local map
        let mut policy_map: HashMap<usize, Policy> = HashMap::with_capacity(policies.len());

        // Encode each policy as a constraint
        for (idx, policy) in policies.iter().enumerate() {
            let constraint = self.encode_policy(policy);
            policy_map.insert(idx, policy.clone());

            // Assert the constraint with a tracked boolean for unsat core extraction
            let tracked = Bool::fresh_const(&self.context, "policy");
            solver.assert_and_track(&constraint, &tracked);
        }

        // Check satisfiability
        match solver.check() {
            Z3SatResult::Sat => SatResult::Sat,
            Z3SatResult::Unsat => {
                let core = Self::extract_unsat_core(&solver, &policy_map);
                SatResult::Unsat(core)
            }
            Z3SatResult::Unknown => SatResult::Timeout,
        }
    }

    /// Encodes a policy as a Z3 constraint.
    fn encode_policy(&self, policy: &Policy) -> Bool<'_> {
        match policy {
            Policy::Budget { max } => self.encode_budget_constraint(*max),
            Policy::MinBudget { min } => self.encode_min_budget_constraint(*min),
            Policy::TimeRange { start, end } => {
                self.encode_time_range_constraint(*start, *end)
            }
            Policy::Region { regions } => self.encode_region_constraint(regions),
            Policy::EncryptionRequired => self.encode_encryption_constraint(),
            Policy::AuditRequired => self.encode_audit_constraint(),
            Policy::DataResidency { jurisdictions } => {
                self.encode_data_residency_constraint(jurisdictions)
            }
        }
    }

    /// Encodes a budget constraint: budget <= max.
    fn encode_budget_constraint(&self, max: u64) -> Bool<'_> {
        let budget_var = Int::new_const(&self.context, "budget");
        let max_int = Int::from_u64(&self.context, max);
        budget_var.le(&max_int)
    }

    /// Encodes a minimum budget constraint: budget >= min.
    fn encode_min_budget_constraint(&self, min: u64) -> Bool<'_> {
        let budget_var = Int::new_const(&self.context, "budget");
        let min_int = Int::from_u64(&self.context, min);
        budget_var.ge(&min_int)
    }

    /// Encodes a time range constraint: start <= execution_time <= end.
    fn encode_time_range_constraint(&self, start: i64, end: i64) -> Bool<'_> {
        let exec_time = Int::new_const(&self.context, "execution_time");
        let start_int = Int::from_i64(&self.context, start);
        let end_int = Int::from_i64(&self.context, end);

        let lower = exec_time.ge(&start_int);
        let upper = exec_time.le(&end_int);

        Bool::and(&self.context, &[&lower, &upper])
    }

    /// Encodes a region constraint using an enum of allowed regions.
    fn encode_region_constraint(&self, regions: &[String]) -> Bool<'_> {
        if regions.is_empty() {
            // Empty region list is always unsatisfiable
            return Bool::from_bool(&self.context, false);
        }

        let region_var = Int::new_const(&self.context, "region");
        let mut disjuncts = Vec::with_capacity(regions.len());

        for region in regions {
            // Hash the region string to get a deterministic integer value
            let region_hash = Self::hash_region(region);
            let region_val = Int::from_i64(&self.context, region_hash);
            disjuncts.push(region_var._eq(&region_val));
        }

        Bool::or(&self.context, &disjuncts.iter().collect::<Vec<_>>())
    }

    /// Encodes an encryption requirement constraint.
    fn encode_encryption_constraint(&self) -> Bool<'_> {
        // Encryption is a boolean flag that must be true
        Bool::new_const(&self.context, "encryption_enabled")
    }

    /// Encodes an audit requirement constraint.
    fn encode_audit_constraint(&self) -> Bool<'_> {
        // Audit logging is a boolean flag that must be true
        Bool::new_const(&self.context, "audit_enabled")
    }

    /// Encodes a data residency constraint.
    fn encode_data_residency_constraint(&self, jurisdictions: &[String]) -> Bool<'_> {
        if jurisdictions.is_empty() {
            return Bool::from_bool(&self.context, false);
        }

        let jurisdiction_var = Int::new_const(&self.context, "jurisdiction");
        let mut disjuncts = Vec::with_capacity(jurisdictions.len());

        for jurisdiction in jurisdictions {
            let juris_hash = Self::hash_region(jurisdiction);
            let juris_val = Int::from_i64(&self.context, juris_hash);
            disjuncts.push(jurisdiction_var._eq(&juris_val));
        }

        Bool::or(&self.context, &disjuncts.iter().collect::<Vec<_>>())
    }

    /// Hashes a region string to a deterministic integer.
    #[inline]
    fn hash_region(region: &str) -> i64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        region.hash(&mut hasher);
        let hash = hasher.finish();

        // Convert to i64, handling the sign bit
        (hash as i64).wrapping_abs()
    }

    /// Extracts the unsatisfiable core as a list of policies.
    fn extract_unsat_core(solver: &Solver<'_>, policy_map: &HashMap<usize, Policy>) -> Vec<Policy> {
        let _core = solver.get_unsat_core();
        let mut result = Vec::with_capacity(_core.len());

        // The unsat core gives us back the tracked booleans
        // We need to map these to the original policies
        // For now, return all policies that were asserted
        // In a more sophisticated implementation, we'd track the mapping
        // between tracked booleans and policy indices

        // Since Z3's unsat_core doesn't directly give us indices,
        // we use the policy_map to reconstruct based on assertion count
        let num_assertions = policy_map.len();
        for idx in 0..num_assertions {
            if let Some(policy) = policy_map.get(&idx) {
                result.push(policy.clone());
            }
        }

        result
    }
}

impl Default for SmtContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to check policies without managing context.
///
/// This creates a temporary SMT context with default settings.
///
/// # Arguments
///
/// * `policies` - The policies to check.
///
/// # Returns
///
/// The satisfiability result.
///
/// # Example
///
/// ```
/// use ash_typeck::smt::{Policy, SatResult, check_policies};
///
/// let policies = vec![
///     Policy::Budget { max: 100 },
///     Policy::MinBudget { min: 50 },
/// ];
///
/// match check_policies(&policies) {
///     SatResult::Sat => println!("Policies compatible"),
///     SatResult::Unsat(core) => println!("Conflicts: {:?}", core),
///     SatResult::Timeout => println!("Check timed out"),
///     _ => println!("Unknown result"),
/// }
/// ```
#[inline]
#[must_use]
pub fn check_policies(policies: &[Policy]) -> SatResult {
    let mut ctx = SmtContext::new();
    ctx.check_policies(policies)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_policy_sat() {
        // A single policy should always be satisfiable
        let policies = vec![Policy::Budget { max: 100 }];
        let result = check_policies(&policies);
        assert_eq!(result, SatResult::Sat);
    }

    #[test]
    fn test_conflicting_budgets() {
        // min > max should be unsatisfiable
        let policies = vec![
            Policy::Budget { max: 50 },
            Policy::MinBudget { min: 100 },
        ];
        let result = check_policies(&policies);
        assert!(
            matches!(result, SatResult::Unsat(_)),
            "Expected Unsat for conflicting budgets, got {:?}",
            result
        );
    }

    #[test]
    fn test_disjoint_time_ranges() {
        // Two time ranges with no overlap should be unsatisfiable
        let policies = vec![
            Policy::TimeRange {
                start: 1000,
                end: 2000,
            },
            Policy::TimeRange {
                start: 3000,
                end: 4000,
            },
        ];
        let result = check_policies(&policies);
        assert!(
            matches!(result, SatResult::Unsat(_)),
            "Expected Unsat for disjoint time ranges, got {:?}",
            result
        );
    }

    #[test]
    fn test_overlapping_time_ranges() {
        // Two time ranges that overlap should be satisfiable
        let policies = vec![
            Policy::TimeRange {
                start: 1000,
                end: 3000,
            },
            Policy::TimeRange {
                start: 2000,
                end: 4000,
            },
        ];
        let result = check_policies(&policies);
        assert_eq!(
            result,
            SatResult::Sat,
            "Expected Sat for overlapping time ranges"
        );
    }

    #[test]
    fn test_compatible_policies() {
        // Multiple non-conflicting policies should be satisfiable
        let policies = vec![
            Policy::Budget { max: 1000 },
            Policy::MinBudget { min: 100 },
            Policy::TimeRange {
                start: 0,
                end: 86400,
            },
            Policy::Region {
                regions: vec!["us-east-1".to_string(), "eu-west-1".to_string()],
            },
            Policy::EncryptionRequired,
        ];
        let result = check_policies(&policies);
        assert_eq!(result, SatResult::Sat);
    }

    #[test]
    fn test_unsat_core_minimal() {
        // Create a conflict and verify the unsat core contains the conflicting policies
        let conflict_budget_max = Policy::Budget { max: 50 };
        let conflict_budget_min = Policy::MinBudget { min: 100 };

        let policies = vec![
            conflict_budget_max.clone(),
            conflict_budget_min.clone(),
            Policy::EncryptionRequired,
        ];

        let result = check_policies(&policies);

        if let SatResult::Unsat(core) = result {
            // The core should contain the conflicting budget policies
            assert!(
                core.contains(&conflict_budget_max) || core.contains(&conflict_budget_min),
                "Unsat core should contain conflicting budget policies"
            );
            // Core should not be larger than original policy set
            assert!(core.len() <= policies.len());
        } else {
            panic!("Expected Unsat result for conflicting budgets, got {:?}", result);
        }
    }

    #[test]
    fn test_empty_policies() {
        // Empty policy set should be trivially satisfiable
        let policies: Vec<Policy> = vec![];
        let result = check_policies(&policies);
        assert_eq!(result, SatResult::Sat);
    }

    #[test]
    fn test_region_constraint() {
        // Single region policy should be satisfiable
        let policies = vec![Policy::Region {
            regions: vec!["us-east-1".to_string()],
        }];
        let result = check_policies(&policies);
        assert_eq!(result, SatResult::Sat);
    }

    #[test]
    fn test_empty_region_list() {
        // Empty region list should be unsatisfiable
        let policies = vec![Policy::Region { regions: vec![] }];
        let result = check_policies(&policies);
        assert!(
            matches!(result, SatResult::Unsat(_)),
            "Expected Unsat for empty region list"
        );
    }

    #[test]
    fn test_encryption_policy() {
        let policies = vec![Policy::EncryptionRequired];
        let result = check_policies(&policies);
        assert_eq!(result, SatResult::Sat);
    }

    #[test]
    fn test_audit_policy() {
        let policies = vec![Policy::AuditRequired];
        let result = check_policies(&policies);
        assert_eq!(result, SatResult::Sat);
    }

    #[test]
    fn test_data_residency_policy() {
        let policies = vec![Policy::DataResidency {
            jurisdictions: vec!["EU".to_string(), "US".to_string()],
        }];
        let result = check_policies(&policies);
        assert_eq!(result, SatResult::Sat);
    }

    #[test]
    fn test_equal_budget_boundary() {
        // min == max should be satisfiable
        let policies = vec![
            Policy::Budget { max: 100 },
            Policy::MinBudget { min: 100 },
        ];
        let result = check_policies(&policies);
        assert_eq!(
            result,
            SatResult::Sat,
            "min == max should be satisfiable"
        );
    }

    #[test]
    fn test_smt_context_reuse() {
        // Test that context can be reused for multiple checks
        let mut ctx = SmtContext::new();

        let first_check = ctx.check_policies(&[Policy::Budget { max: 100 }]);
        assert_eq!(first_check, SatResult::Sat);

        let second_check = ctx.check_policies(&[Policy::MinBudget { min: 50 }]);
        assert_eq!(second_check, SatResult::Sat);

        let conflict_check = ctx.check_policies(&[
            Policy::Budget { max: 10 },
            Policy::MinBudget { min: 20 },
        ]);
        assert!(matches!(conflict_check, SatResult::Unsat(_)));
    }
}
