//! Requirement checking at call sites for contracts (TASK-228)
//!
//! Provides compile-time verification that call sites satisfy the requirements
//! of invoked workflows. Supports capability, role, and arithmetic constraints
//! with SMT integration for arithmetic verification.

use ash_core::workflow_contract::{ArithConstraint, Contract, Effect, Requirement};
use std::collections::HashMap;
use thiserror::Error;

/// Error type for requirement checking failures
#[derive(Debug, Clone, PartialEq, Error)]
pub enum RequirementError {
    /// Missing required capability
    #[error("missing capability '{cap}': requires at least {required:?} effect, found {found:?}")]
    MissingCapability {
        /// Capability name
        cap: String,
        /// Required effect level
        required: Effect,
        /// Found effect level (if any)
        found: Option<Effect>,
    },

    /// Missing required role
    #[error("missing role '{role}'")]
    MissingRole { role: String },

    /// Arithmetic constraint violation
    #[error("arithmetic constraint failed for '{var}': {constraint:?}")]
    ArithConstraintViolated {
        /// Variable name
        var: String,
        /// Constraint that failed
        constraint: ArithConstraint,
        /// Actual value (if known)
        actual: Option<i64>,
    },

    /// Unknown variable in constraint
    #[error("unknown variable '{var}' in constraint")]
    UnknownVariable { var: String },

    /// SMT solver error
    #[error("SMT solver error: {0}")]
    SmtError(String),
}

/// Context for checking requirements at a call site
#[derive(Debug, Clone, Default)]
pub struct RequirementContext {
    /// Available capabilities: cap_name -> effect level
    capabilities: HashMap<String, Effect>,
    /// Available roles
    roles: HashSet<String>,
    /// Known facts about variables: var_name -> value
    facts: HashMap<String, i64>,
}

use std::collections::HashSet;

impl RequirementContext {
    /// Create a new empty requirement context
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a capability to the context
    pub fn with_capability(mut self, cap: impl Into<String>, effect: Effect) -> Self {
        self.capabilities.insert(cap.into(), effect);
        self
    }

    /// Add a role to the context
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.insert(role.into());
        self
    }

    /// Add a fact about a variable
    pub fn with_fact(mut self, var: impl Into<String>, value: i64) -> Self {
        self.facts.insert(var.into(), value);
        self
    }

    /// Check if the context has a capability with at least the required effect
    pub fn has_capability(&self, cap: &str, min_effect: Effect) -> bool {
        match self.capabilities.get(cap) {
            Some(&effect) => effect_level(effect) >= effect_level(min_effect),
            None => false,
        }
    }

    /// Check if the context has a role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(role)
    }

    /// Get a fact value for a variable
    pub fn get_fact(&self, var: &str) -> Option<i64> {
        self.facts.get(var).copied()
    }

    /// Get all capabilities
    pub fn capabilities(&self) -> &HashMap<String, Effect> {
        &self.capabilities
    }

    /// Get all roles
    pub fn roles(&self) -> &HashSet<String> {
        &self.roles
    }

    /// Get all facts
    pub fn facts(&self) -> &HashMap<String, i64> {
        &self.facts
    }
}

/// Get the numeric level of an effect for comparison
fn effect_level(effect: Effect) -> u8 {
    match effect {
        Effect::Epistemic => 0,
        Effect::Deliberative => 1,
        Effect::Evaluative => 2,
        Effect::Operational => 3,
    }
}

/// Result of checking a single requirement
#[derive(Debug, Clone, PartialEq)]
pub enum CheckResult {
    /// Requirement satisfied
    Satisfied,
    /// Requirement failed with error
    Failed(RequirementError),
}

impl CheckResult {
    /// Check if the result is satisfied
    pub fn is_satisfied(&self) -> bool {
        matches!(self, CheckResult::Satisfied)
    }

    /// Check if the result failed
    pub fn is_failed(&self) -> bool {
        matches!(self, CheckResult::Failed(_))
    }
}

/// Check a single requirement against a context
///
/// # Arguments
///
/// * `req` - The requirement to check
/// * `ctx` - The requirement context at the call site
///
/// # Returns
///
/// * `CheckResult::Satisfied` - The requirement is met
/// * `CheckResult::Failed` - The requirement is not met
///
/// # Examples
///
/// ```
/// use ash_typeck::requirements::{RequirementContext, check_requirement, CheckResult};
/// use ash_core::workflow_contract::{Requirement, Effect};
///
/// let ctx = RequirementContext::new()
///     .with_capability("file_io", Effect::Operational)
///     .with_role("admin");
///
/// // Check capability requirement
/// let req = Requirement::HasCapability {
///     cap: "file_io".into(),
///     min_effect: Effect::Epistemic,
/// };
/// let result = check_requirement(&req, &ctx);
/// assert!(result.is_satisfied());
///
/// // Check role requirement
/// let req = Requirement::HasRole("admin".into());
/// let result = check_requirement(&req, &ctx);
/// assert!(result.is_satisfied());
/// ```
pub fn check_requirement(req: &Requirement, ctx: &RequirementContext) -> CheckResult {
    match req {
        Requirement::HasCapability { cap, min_effect } => {
            if ctx.has_capability(cap, *min_effect) {
                CheckResult::Satisfied
            } else {
                let found = ctx.capabilities.get(cap).copied();
                CheckResult::Failed(RequirementError::MissingCapability {
                    cap: cap.clone(),
                    required: *min_effect,
                    found,
                })
            }
        }
        Requirement::HasRole(role) => {
            if ctx.has_role(role) {
                CheckResult::Satisfied
            } else {
                CheckResult::Failed(RequirementError::MissingRole { role: role.clone() })
            }
        }
        Requirement::Arithmetic { var, constraint } => {
            check_arithmetic_constraint(var, constraint, ctx)
        }
    }
}

/// Check an arithmetic constraint against the context
fn check_arithmetic_constraint(
    var: &str,
    constraint: &ArithConstraint,
    ctx: &RequirementContext,
) -> CheckResult {
    let value = match ctx.get_fact(var) {
        Some(v) => v,
        None => {
            return CheckResult::Failed(RequirementError::UnknownVariable { var: var.into() });
        }
    };

    let satisfied = match constraint {
        ArithConstraint::Gt(min) => value > *min,
        ArithConstraint::Lt(max) => value < *max,
        ArithConstraint::Gte(min) => value >= *min,
        ArithConstraint::Lte(max) => value <= *max,
        ArithConstraint::Eq(expected) => value == *expected,
        ArithConstraint::Range { min, max } => value >= *min && value <= *max,
    };

    if satisfied {
        CheckResult::Satisfied
    } else {
        CheckResult::Failed(RequirementError::ArithConstraintViolated {
            var: var.into(),
            constraint: constraint.clone(),
            actual: Some(value),
        })
    }
}

/// Result of checking a contract
#[derive(Debug, Clone, PartialEq)]
pub struct ContractCheckResult {
    /// Results for each requirement (in order)
    pub requirement_results: Vec<CheckResult>,
    /// Overall success
    pub success: bool,
    /// Failed requirements (indices)
    pub failed_indices: Vec<usize>,
}

impl ContractCheckResult {
    /// Create a result from individual requirement results
    fn from_results(results: Vec<CheckResult>) -> Self {
        let failed_indices: Vec<usize> = results
            .iter()
            .enumerate()
            .filter(|(_, r)| r.is_failed())
            .map(|(i, _)| i)
            .collect();

        Self {
            success: failed_indices.is_empty(),
            requirement_results: results,
            failed_indices,
        }
    }

    /// Check if all requirements are satisfied
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get all errors from failed requirements
    pub fn errors(&self) -> Vec<&RequirementError> {
        self.requirement_results
            .iter()
            .filter_map(|r| match r {
                CheckResult::Failed(e) => Some(e),
                _ => None,
            })
            .collect()
    }
}

/// Check all requirements of a contract against a context
///
/// # Arguments
///
/// * `contract` - The contract to check
/// * `ctx` - The requirement context at the call site
///
/// # Returns
///
/// A `ContractCheckResult` containing results for all requirements
///
/// # Examples
///
/// ```
/// use ash_typeck::requirements::{RequirementContext, check_contract};
/// use ash_core::workflow_contract::{Contract, Requirement, Effect, ArithConstraint};
///
/// let contract = Contract::new()
///     .with_requirement(Requirement::HasRole("admin".into()))
///     .with_requirement(Requirement::Arithmetic {
///         var: "amount".into(),
///         constraint: ArithConstraint::Gt(0),
///     });
///
/// let ctx = RequirementContext::new()
///     .with_role("admin")
///     .with_fact("amount", 100);
///
/// let result = check_contract(&contract, &ctx);
/// assert!(result.is_success());
/// ```
pub fn check_contract(contract: &Contract, ctx: &RequirementContext) -> ContractCheckResult {
    let results: Vec<CheckResult> = contract
        .requires
        .iter()
        .map(|req| check_requirement(req, ctx))
        .collect();

    ContractCheckResult::from_results(results)
}

/// SMT-based arithmetic constraint checker using Z3
///
/// This module provides SMT-based verification for arithmetic constraints
/// when values are not known at compile time (symbolic checking).
#[cfg(feature = "smt")]
pub mod smt_checker {
    use super::*;
    use z3::ast::{Ast, Int};
    use z3::{Config, Context, SatResult as Z3SatResult, Solver};

    /// SMT context for symbolic arithmetic checking
    pub struct SmtChecker {
        context: Box<Context>,
    }

    impl SmtChecker {
        /// Create a new SMT checker with default timeout
        pub fn new() -> Self {
            Self::with_timeout_ms(5000)
        }

        /// Create a new SMT checker with specified timeout
        pub fn with_timeout_ms(timeout_ms: u64) -> Self {
            let mut config = Config::new();
            config.set_timeout_msec(timeout_ms);
            let context = Box::new(Context::new(&config));
            Self { context }
        }

        /// Check if arithmetic constraints are satisfiable
        ///
        /// Returns true if there exists a valuation that satisfies all constraints
        pub fn check_constraints(&self, constraints: &[(String, ArithConstraint)]) -> bool {
            let solver = Solver::new(&self.context);

            for (var, constraint) in constraints {
                let var_ast = Int::new_const(&self.context, var.as_str());
                let constraint_ast = self.encode_constraint(&var_ast, constraint);
                solver.assert(&constraint_ast);
            }

            matches!(solver.check(), Z3SatResult::Sat)
        }

        fn encode_constraint<'a>(
            &self,
            var: &Int<'a>,
            constraint: &ArithConstraint,
        ) -> z3::ast::Bool<'a> {
            match constraint {
                ArithConstraint::Gt(val) => {
                    let val_ast = Int::from_i64(&self.context, *val);
                    var.gt(&val_ast)
                }
                ArithConstraint::Lt(val) => {
                    let val_ast = Int::from_i64(&self.context, *val);
                    var.lt(&val_ast)
                }
                ArithConstraint::Gte(val) => {
                    let val_ast = Int::from_i64(&self.context, *val);
                    var.ge(&val_ast)
                }
                ArithConstraint::Lte(val) => {
                    let val_ast = Int::from_i64(&self.context, *val);
                    var.le(&val_ast)
                }
                ArithConstraint::Eq(val) => {
                    let val_ast = Int::from_i64(&self.context, *val);
                    var._eq(&val_ast)
                }
                ArithConstraint::Range { min, max } => {
                    let min_ast = Int::from_i64(&self.context, *min);
                    let max_ast = Int::from_i64(&self.context, *max);
                    let lower = var.ge(&min_ast);
                    let upper = var.le(&max_ast);
                    z3::ast::Bool::and(&self.context, &[&lower, &upper])
                }
            }
        }
    }

    impl Default for SmtChecker {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::workflow_contract::{ArithConstraint, Contract, Effect, Requirement};

    // =========================================================================
    // RequirementContext tests
    // =========================================================================

    #[test]
    fn test_context_creation() {
        let ctx = RequirementContext::new();
        assert!(ctx.capabilities().is_empty());
        assert!(ctx.roles().is_empty());
        assert!(ctx.facts().is_empty());
    }

    #[test]
    fn test_context_with_capability() {
        let ctx = RequirementContext::new().with_capability("file_io", Effect::Operational);
        // Effect levels: Epistemic(0) < Deliberative(1) < Evaluative(2) < Operational(3)
        assert!(ctx.has_capability("file_io", Effect::Epistemic)); // 3 >= 0, satisfies
        assert!(ctx.has_capability("file_io", Effect::Operational)); // 3 >= 3, exact match
        assert!(ctx.has_capability("file_io", Effect::Deliberative)); // 3 >= 1, satisfies
        assert!(!ctx.has_capability("network", Effect::Epistemic)); // Missing capability
    }

    #[test]
    fn test_context_with_role() {
        let ctx = RequirementContext::new().with_role("admin");
        assert!(ctx.has_role("admin"));
        assert!(!ctx.has_role("user"));
    }

    #[test]
    fn test_context_with_fact() {
        let ctx = RequirementContext::new().with_fact("amount", 100);
        assert_eq!(ctx.get_fact("amount"), Some(100));
        assert_eq!(ctx.get_fact("unknown"), None);
    }

    #[test]
    fn test_context_builder_pattern() {
        let ctx = RequirementContext::new()
            .with_capability("file_io", Effect::Operational)
            .with_capability("network", Effect::Epistemic)
            .with_role("admin")
            .with_role("user")
            .with_fact("x", 10)
            .with_fact("y", 20);

        assert_eq!(ctx.capabilities().len(), 2);
        assert_eq!(ctx.roles().len(), 2);
        assert_eq!(ctx.facts().len(), 2);
    }

    // =========================================================================
    // check_requirement tests - HasCapability
    // =========================================================================

    #[test]
    fn test_check_requirement_capability_satisfied() {
        let ctx = RequirementContext::new().with_capability("file_io", Effect::Operational);

        let req = Requirement::HasCapability {
            cap: "file_io".into(),
            min_effect: Effect::Epistemic,
        };

        let result = check_requirement(&req, &ctx);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_check_requirement_capability_exact_match() {
        let ctx = RequirementContext::new().with_capability("file_io", Effect::Operational);

        let req = Requirement::HasCapability {
            cap: "file_io".into(),
            min_effect: Effect::Operational,
        };

        let result = check_requirement(&req, &ctx);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_check_requirement_capability_insufficient_effect() {
        let ctx = RequirementContext::new().with_capability("file_io", Effect::Epistemic);

        let req = Requirement::HasCapability {
            cap: "file_io".into(),
            min_effect: Effect::Operational,
        };

        let result = check_requirement(&req, &ctx);
        assert!(result.is_failed());

        match result {
            CheckResult::Failed(RequirementError::MissingCapability { cap, required, found }) => {
                assert_eq!(cap, "file_io");
                assert_eq!(required, Effect::Operational);
                assert_eq!(found, Some(Effect::Epistemic));
            }
            _ => panic!("Expected MissingCapability error"),
        }
    }

    #[test]
    fn test_check_requirement_capability_missing() {
        let ctx = RequirementContext::new();

        let req = Requirement::HasCapability {
            cap: "file_io".into(),
            min_effect: Effect::Epistemic,
        };

        let result = check_requirement(&req, &ctx);
        assert!(result.is_failed());

        match result {
            CheckResult::Failed(RequirementError::MissingCapability { cap, required, found }) => {
                assert_eq!(cap, "file_io");
                assert_eq!(required, Effect::Epistemic);
                assert_eq!(found, None);
            }
            _ => panic!("Expected MissingCapability error"),
        }
    }

    // =========================================================================
    // check_requirement tests - HasRole
    // =========================================================================

    #[test]
    fn test_check_requirement_role_satisfied() {
        let ctx = RequirementContext::new().with_role("admin");

        let req = Requirement::HasRole("admin".into());

        let result = check_requirement(&req, &ctx);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_check_requirement_role_missing() {
        let ctx = RequirementContext::new().with_role("user");

        let req = Requirement::HasRole("admin".into());

        let result = check_requirement(&req, &ctx);
        assert!(result.is_failed());

        match result {
            CheckResult::Failed(RequirementError::MissingRole { role }) => {
                assert_eq!(role, "admin");
            }
            _ => panic!("Expected MissingRole error"),
        }
    }

    // =========================================================================
    // check_requirement tests - Arithmetic
    // =========================================================================

    #[test]
    fn test_check_requirement_arithmetic_gt_satisfied() {
        let ctx = RequirementContext::new().with_fact("amount", 100);

        let req = Requirement::Arithmetic {
            var: "amount".into(),
            constraint: ArithConstraint::Gt(0),
        };

        let result = check_requirement(&req, &ctx);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_check_requirement_arithmetic_gt_failed() {
        let ctx = RequirementContext::new().with_fact("amount", 0);

        let req = Requirement::Arithmetic {
            var: "amount".into(),
            constraint: ArithConstraint::Gt(0),
        };

        let result = check_requirement(&req, &ctx);
        assert!(result.is_failed());
    }

    #[test]
    fn test_check_requirement_arithmetic_lt_satisfied() {
        let ctx = RequirementContext::new().with_fact("amount", 50);

        let req = Requirement::Arithmetic {
            var: "amount".into(),
            constraint: ArithConstraint::Lt(100),
        };

        let result = check_requirement(&req, &ctx);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_check_requirement_arithmetic_gte_satisfied() {
        let ctx = RequirementContext::new().with_fact("amount", 0);

        let req = Requirement::Arithmetic {
            var: "amount".into(),
            constraint: ArithConstraint::Gte(0),
        };

        let result = check_requirement(&req, &ctx);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_check_requirement_arithmetic_lte_satisfied() {
        let ctx = RequirementContext::new().with_fact("amount", 100);

        let req = Requirement::Arithmetic {
            var: "amount".into(),
            constraint: ArithConstraint::Lte(100),
        };

        let result = check_requirement(&req, &ctx);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_check_requirement_arithmetic_eq_satisfied() {
        let ctx = RequirementContext::new().with_fact("status", 42);

        let req = Requirement::Arithmetic {
            var: "status".into(),
            constraint: ArithConstraint::Eq(42),
        };

        let result = check_requirement(&req, &ctx);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_check_requirement_arithmetic_range_satisfied() {
        let ctx = RequirementContext::new().with_fact("value", 50);

        let req = Requirement::Arithmetic {
            var: "value".into(),
            constraint: ArithConstraint::Range { min: 0, max: 100 },
        };

        let result = check_requirement(&req, &ctx);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_check_requirement_arithmetic_range_boundary() {
        let ctx = RequirementContext::new().with_fact("value", 0);

        let req = Requirement::Arithmetic {
            var: "value".into(),
            constraint: ArithConstraint::Range { min: 0, max: 100 },
        };

        let result = check_requirement(&req, &ctx);
        assert!(result.is_satisfied());

        let ctx = RequirementContext::new().with_fact("value", 100);
        let result = check_requirement(&req, &ctx);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_check_requirement_arithmetic_unknown_variable() {
        let ctx = RequirementContext::new();

        let req = Requirement::Arithmetic {
            var: "unknown".into(),
            constraint: ArithConstraint::Gt(0),
        };

        let result = check_requirement(&req, &ctx);
        assert!(result.is_failed());

        match result {
            CheckResult::Failed(RequirementError::UnknownVariable { var }) => {
                assert_eq!(var, "unknown");
            }
            _ => panic!("Expected UnknownVariable error"),
        }
    }

    // =========================================================================
    // check_contract tests
    // =========================================================================

    #[test]
    fn test_check_contract_all_satisfied() {
        let contract = Contract::new()
            .with_requirement(Requirement::HasRole("admin".into()))
            .with_requirement(Requirement::Arithmetic {
                var: "amount".into(),
                constraint: ArithConstraint::Gt(0),
            });

        let ctx = RequirementContext::new()
            .with_role("admin")
            .with_fact("amount", 100);

        let result = check_contract(&contract, &ctx);
        assert!(result.is_success());
        assert!(result.failed_indices.is_empty());
    }

    #[test]
    fn test_check_contract_some_failed() {
        let contract = Contract::new()
            .with_requirement(Requirement::HasRole("admin".into()))
            .with_requirement(Requirement::HasRole("user".into()));

        let ctx = RequirementContext::new().with_role("admin");

        let result = check_contract(&contract, &ctx);
        assert!(!result.is_success());
        assert_eq!(result.failed_indices.len(), 1);
        assert_eq!(result.failed_indices[0], 1);
    }

    #[test]
    fn test_check_contract_all_failed() {
        let contract = Contract::new()
            .with_requirement(Requirement::HasRole("admin".into()))
            .with_requirement(Requirement::HasCapability {
                cap: "file_io".into(),
                min_effect: Effect::Operational,
            });

        let ctx = RequirementContext::new();

        let result = check_contract(&contract, &ctx);
        assert!(!result.is_success());
        assert_eq!(result.failed_indices.len(), 2);
    }

    #[test]
    fn test_check_contract_empty() {
        let contract = Contract::new();
        let ctx = RequirementContext::new();

        let result = check_contract(&contract, &ctx);
        assert!(result.is_success());
        assert!(result.requirement_results.is_empty());
    }

    #[test]
    fn test_contract_check_result_errors() {
        let contract = Contract::new()
            .with_requirement(Requirement::HasRole("admin".into()))
            .with_requirement(Requirement::HasRole("user".into()));

        let ctx = RequirementContext::new().with_role("admin");

        let result = check_contract(&contract, &ctx);
        let errors = result.errors();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], RequirementError::MissingRole { .. }));
    }

    // =========================================================================
    // Effect ordering tests
    // =========================================================================

    #[test]
    fn test_effect_ordering() {
        // Epistemic (0) < Deliberative (1) < Evaluative (2) < Operational (3)
        assert!(effect_level(Effect::Epistemic) < effect_level(Effect::Deliberative));
        assert!(effect_level(Effect::Deliberative) < effect_level(Effect::Evaluative));
        assert!(effect_level(Effect::Evaluative) < effect_level(Effect::Operational));
    }

    #[test]
    fn test_capability_effect_levels() {
        let ctx = RequirementContext::new().with_capability("res", Effect::Evaluative);

        // Should satisfy lower effect requirements
        assert!(ctx.has_capability("res", Effect::Epistemic));
        assert!(ctx.has_capability("res", Effect::Deliberative));
        assert!(ctx.has_capability("res", Effect::Evaluative));

        // Should not satisfy higher effect requirements
        assert!(!ctx.has_capability("res", Effect::Operational));
    }

    // =========================================================================
    // Integration tests
    // =========================================================================

    #[test]
    fn test_complex_contract() {
        // A realistic contract with multiple requirement types
        let contract = Contract::new()
            .with_requirement(Requirement::HasCapability {
                cap: "database".into(),
                min_effect: Effect::Operational,
            })
            .with_requirement(Requirement::HasRole("db_admin".into()))
            .with_requirement(Requirement::Arithmetic {
                var: "timeout_ms".into(),
                constraint: ArithConstraint::Range { min: 100, max: 5000 },
            })
            .with_requirement(Requirement::Arithmetic {
                var: "max_retries".into(),
                constraint: ArithConstraint::Gte(0),
            });

        // Satisfy all requirements
        let ctx = RequirementContext::new()
            .with_capability("database", Effect::Operational)
            .with_role("db_admin")
            .with_fact("timeout_ms", 1000)
            .with_fact("max_retries", 3);

        let result = check_contract(&contract, &ctx);
        assert!(result.is_success());

        // Fail one requirement
        let ctx = RequirementContext::new()
            .with_capability("database", Effect::Operational)
            .with_role("db_admin")
            .with_fact("timeout_ms", 10000) // Out of range
            .with_fact("max_retries", 3);

        let result = check_contract(&contract, &ctx);
        assert!(!result.is_success());
        assert_eq!(result.failed_indices.len(), 1);
    }

    #[test]
    fn test_error_display() {
        let err = RequirementError::MissingCapability {
            cap: "file_io".into(),
            required: Effect::Operational,
            found: Some(Effect::Epistemic),
        };
        let msg = format!("{err}");
        assert!(msg.contains("file_io"));
        assert!(msg.contains("Operational"));

        let err = RequirementError::MissingRole { role: "admin".into() };
        let msg = format!("{err}");
        assert!(msg.contains("admin"));

        let err = RequirementError::ArithConstraintViolated {
            var: "amount".into(),
            constraint: ArithConstraint::Gt(0),
            actual: Some(-5),
        };
        let msg = format!("{err}");
        assert!(msg.contains("amount"));
        assert!(msg.contains("Gt"));

        let err = RequirementError::UnknownVariable { var: "x".into() };
        let msg = format!("{err}");
        assert!(msg.contains("x"));
    }
}

#[cfg(all(test, feature = "proptest"))]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn capability_satisfies_same_or_lower_effect(
            effect in prop::sample::select(vec![
                Effect::Epistemic,
                Effect::Deliberative,
                Effect::Evaluative,
                Effect::Operational,
            ])
        ) {
            let ctx = RequirementContext::new().with_capability("test", effect);

            // Should satisfy same effect level
            let req = Requirement::HasCapability {
                cap: "test".into(),
                min_effect: effect,
            };
            assert!(check_requirement(&req, &ctx).is_satisfied());
        }

        #[test]
        fn arithmetic_gt_satisfied_for_positive_values(
            value in 1i64..1000,
            min in -1000i64..0
        ) {
            let ctx = RequirementContext::new().with_fact("x", value);
            let req = Requirement::Arithmetic {
                var: "x".into(),
                constraint: ArithConstraint::Gt(min),
            };
            assert!(check_requirement(&req, &ctx).is_satisfied());
        }

        #[test]
        fn arithmetic_range_satisfied_for_values_in_range(
            value in 0i64..100,
            min in -50i64..0,
            max in 100i64..200
        ) {
            let ctx = RequirementContext::new().with_fact("x", value);
            let req = Requirement::Arithmetic {
                var: "x".into(),
                constraint: ArithConstraint::Range { min, max },
            };
            assert!(check_requirement(&req, &ctx).is_satisfied());
        }

        #[test]
        fn role_satisfied_when_present(role in "[a-z_][a-z0-9_]*") {
            let ctx = RequirementContext::new().with_role(&role);
            let req = Requirement::HasRole(role);
            assert!(check_requirement(&req, &ctx).is_satisfied());
        }
    }
}
