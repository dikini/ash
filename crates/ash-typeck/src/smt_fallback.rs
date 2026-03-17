//! Fallback Policy Conflict Detection (No SMT)
//!
//! This module provides a simplified policy conflict detection implementation
//! that is used when the `smt` feature is not enabled. It performs structural
//! conflict detection without requiring the Z3 solver.
//!
//! # Limitations
//!
//! - Does not perform true SAT solving
//! - May miss complex conflicts that require constraint solving
//! - Unsat core may not be minimal
//!
//! For full conflict detection, enable the `smt` feature.

/// A policy constraint that can be checked for conflicts.
///
/// This is a simplified version of the policy enum used when the SMT feature
/// is not available. It supports basic policy types for structural checking.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Policy {
    /// Maximum budget constraint (inclusive).
    Budget {
        /// Maximum allowed budget.
        max: u64,
    },

    /// Minimum budget constraint (inclusive).
    MinBudget {
        /// Minimum required budget.
        min: u64,
    },

    /// Time range constraint.
    TimeRange {
        /// Start time (Unix timestamp).
        start: i64,
        /// End time (Unix timestamp).
        end: i64,
    },

    /// Geographic region constraint.
    Region {
        /// Allowed region codes.
        regions: Vec<String>,
    },

    /// Encryption requirement.
    EncryptionRequired,

    /// Audit logging requirement.
    AuditRequired,

    /// Data residency constraint.
    DataResidency {
        /// Allowed jurisdictions.
        jurisdictions: Vec<String>,
    },
}

/// Result of checking a set of policies for satisfiability.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SatResult {
    /// The policies are satisfiable.
    Sat,

    /// The policies are unsatisfiable. Contains conflicting policies.
    Unsat(Vec<Policy>),

    /// The check timed out (fallback does not time out, but preserves API compatibility).
    Timeout,
}

/// Context for fallback policy checking.
///
/// This struct provides a compatible API with `SmtContext` but performs
/// only structural conflict detection.
pub struct SmtContext {
    /// No internal state needed for fallback.
    _private: (),
}

impl SmtContext {
    /// Creates a new fallback SMT context.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Creates a new fallback context with a specified timeout.
    ///
    /// Note: The fallback implementation does not support timeouts,
    /// but this method is provided for API compatibility.
    #[inline]
    #[must_use]
    pub fn with_timeout_ms(_timeout_ms: u32) -> Self {
        Self::new()
    }

    /// Checks a set of policies for structural conflicts.
    ///
    /// This method performs the following checks:
    /// - Budget min/max conflicts (min > max)
    /// - Time range overlaps (disjoint ranges)
    /// - Region compatibility (empty region lists)
    ///
    /// # Arguments
    ///
    /// * `policies` - The policies to check.
    ///
    /// # Returns
    ///
    /// * `SatResult::Sat` - No structural conflicts detected.
    /// * `SatResult::Unsat(core)` - Conflicts detected; `core` contains conflicting policies.
    #[must_use]
    pub fn check_policies(&mut self, policies: &[Policy]) -> SatResult {
        let mut conflicts = Vec::new();

        // Collect budget constraints
        let mut max_budgets: Vec<u64> = Vec::new();
        let mut min_budgets: Vec<u64> = Vec::new();

        // Collect time ranges
        let mut time_ranges: Vec<(i64, i64)> = Vec::new();

        for policy in policies {
            match policy {
                Policy::Budget { max } => max_budgets.push(*max),
                Policy::MinBudget { min } => min_budgets.push(*min),
                Policy::TimeRange { start, end } => {
                    if start > end {
                        // Invalid time range (start after end)
                        conflicts.push(policy.clone());
                    } else {
                        time_ranges.push((*start, *end));
                    }
                }
                Policy::Region { regions } => {
                    if regions.is_empty() {
                        conflicts.push(policy.clone());
                    }
                }
                Policy::DataResidency { jurisdictions } => {
                    if jurisdictions.is_empty() {
                        conflicts.push(policy.clone());
                    }
                }
                Policy::EncryptionRequired | Policy::AuditRequired => {
                    // These are always satisfiable in fallback mode
                }
            }
        }

        // Check budget conflicts
        if let Some(&max_budget) = max_budgets.iter().min() {
            if let Some(&min_budget) = min_budgets.iter().max() {
                if min_budget > max_budget {
                    // Find the policies that caused the conflict
                    for policy in policies {
                        if let Policy::Budget { max } = policy {
                            if *max == max_budget {
                                conflicts.push(policy.clone());
                                break;
                            }
                        }
                    }
                    for policy in policies {
                        if let Policy::MinBudget { min } = policy {
                            if *min == min_budget {
                                conflicts.push(policy.clone());
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Check time range conflicts
        // Multiple time ranges must overlap (intersection must be non-empty)
        if time_ranges.len() > 1 {
            // Find the intersection of all time ranges
            let mut max_start = time_ranges[0].0;
            let mut min_end = time_ranges[0].1;

            for (start, end) in &time_ranges[1..] {
                max_start = max_start.max(*start);
                min_end = min_end.min(*end);
            }

            if max_start > min_end {
                // No overlap - all time ranges conflict with each other
                for policy in policies {
                    if matches!(policy, Policy::TimeRange { .. }) {
                        conflicts.push(policy.clone());
                    }
                }
            }
        }

        if conflicts.is_empty() {
            SatResult::Sat
        } else {
            SatResult::Unsat(conflicts)
        }
    }
}

impl Default for SmtContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to check policies using fallback implementation.
///
/// # Arguments
///
/// * `policies` - The policies to check.
///
/// # Returns
///
/// The satisfiability result based on structural analysis.
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
        let policies = vec![Policy::Budget { max: 100 }];
        let result = check_policies(&policies);
        assert_eq!(result, SatResult::Sat);
    }

    #[test]
    fn test_conflicting_budgets() {
        let policies = vec![
            Policy::Budget { max: 50 },
            Policy::MinBudget { min: 100 },
        ];
        let result = check_policies(&policies);
        assert!(
            matches!(result, SatResult::Unsat(_)),
            "Expected Unsat for conflicting budgets"
        );
    }

    #[test]
    fn test_disjoint_time_ranges() {
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
            "Expected Unsat for disjoint time ranges"
        );
    }

    #[test]
    fn test_overlapping_time_ranges() {
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
        assert_eq!(result, SatResult::Sat);
    }

    #[test]
    fn test_compatible_policies() {
        let policies = vec![
            Policy::Budget { max: 1000 },
            Policy::MinBudget { min: 100 },
            Policy::TimeRange {
                start: 0,
                end: 86400,
            },
            Policy::Region {
                regions: vec!["us-east-1".to_string()],
            },
            Policy::EncryptionRequired,
        ];
        let result = check_policies(&policies);
        assert_eq!(result, SatResult::Sat);
    }

    #[test]
    fn test_empty_policies() {
        let policies: Vec<Policy> = vec![];
        let result = check_policies(&policies);
        assert_eq!(result, SatResult::Sat);
    }

    #[test]
    fn test_empty_region_list() {
        let policies = vec![Policy::Region { regions: vec![] }];
        let result = check_policies(&policies);
        assert!(
            matches!(result, SatResult::Unsat(_)),
            "Expected Unsat for empty region list"
        );
    }

    #[test]
    fn test_equal_budget_boundary() {
        let policies = vec![
            Policy::Budget { max: 100 },
            Policy::MinBudget { min: 100 },
        ];
        let result = check_policies(&policies);
        assert_eq!(result, SatResult::Sat);
    }

    #[test]
    fn test_invalid_time_range() {
        // start > end is invalid
        let policies = vec![Policy::TimeRange {
            start: 2000,
            end: 1000,
        }];
        let result = check_policies(&policies);
        assert!(
            matches!(result, SatResult::Unsat(_)),
            "Expected Unsat for invalid time range (start > end)"
        );
    }
}
