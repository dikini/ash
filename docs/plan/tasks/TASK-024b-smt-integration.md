# TASK-024b: Z3 SMT Integration for Conflict Detection

## Objective

Integrate Z3 SMT solver for detecting policy conflicts at compile-time. This enables Ash to prove that policy constraints are satisfiable before execution, preventing runtime failures due to contradictory obligations.

## Test Strategy (TDD)

### Property Tests (proptest)

```rust
// Property: Single policy is always satisfiable
proptest! {
    #[test]
    fn single_policy_always_sat(policy in arb_policy()) {
        let result = smt_check(&[policy]);
        prop_assert!(matches!(result, SatResult::Sat));
    }
}

// Property: Conflicting resource thresholds are detected
proptest! {
    #[test]
    fn conflicting_budgets_are_unsat(
        budget in 100u64..10000,
        conflict in 1u64..100
    ) {
        let p1 = Policy::budget_lte(budget);
        let p2 = Policy::budget_lte(budget - conflict);
        let result = smt_check(&[p1, p2]);
        // Actually this might be SAT if both satisfied by lower budget
        // Better: conflicting minimums
    }
}

// Property: Unsat core explains the conflict
proptest! {
    #[test]
    fn unsat_core_is_minimal(conflicting in arb_conflicting_policies()) {
        let result = smt_check_with_core(&conflicting);
        if let SatResult::Unsat(core) = result {
            // Removing any element from core makes it SAT
            for i in 0..core.len() {
                let reduced: Vec<_> = core.iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, p)| p.clone())
                    .collect();
                prop_assert!(matches!(smt_check(&reduced), SatResult::Sat));
            }
        }
    }
}

// Property: Temporal constraint conflicts detected
#[test]
fn temporal_conflicts_detected() {
    // Must run between 9-5 AND must run between 6-8
    let business_hours = Policy::time_range(9, 0, 17, 0);
    let maintenance_window = Policy::time_range(6, 0, 8, 0);
    
    let result = smt_check(&[business_hours, maintenance_window]);
    assert!(matches!(result, SatResult::Unsat(_)));
}
```

### Example Tests

```rust
// Real-world: HIPAA + budget + time window
#[test]
fn hipaa_compliance_scenario() {
    let policies = vec![
        Policy::encryption_required(),
        Policy::region_is("us-east-1"),
        Policy::budget_lte(1000),
        Policy::time_range(9, 0, 17, 0),
    ];
    
    assert!(matches!(smt_check(&policies), SatResult::Sat));
}

// Conflict: GDPR right to erasure vs audit retention
#[test]
fn gdpr_audit_conflict() {
    let policies = vec![
        Policy::gdpr_erasure_on_request(),
        Policy::audit_retention_years(7),
    ];
    
    let result = smt_check(&policies);
    assert!(matches!(result, SatResult::Unsat(_)));
}
```

## Implementation Notes

### Z3 Integration Strategy

```rust
// Feature-gated SMT support
#[cfg(feature = "smt")]
pub mod smt {
    use z3::{Config, Context, Solver, SatResult};
    
    pub struct SmtContext {
        ctx: Context,
        solver: Solver<'static>,
    }
    
    impl SmtContext {
        pub fn new() -> Self {
            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let solver = Solver::new(&ctx);
            SmtContext { ctx, solver }
        }
        
        pub fn check_policies(&self, policies: &[Policy]) -> PolicyCheckResult {
            // Encode policies as SMT constraints
            for policy in policies {
                let constraint = self.encode_policy(policy);
                self.solver.assert(&constraint);
            }
            
            match self.solver.check() {
                SatResult::Sat => PolicyCheckResult::Satisfiable,
                SatResult::Unsat => {
                    let core = self.solver.get_unsat_core();
                    let conflicting = self.decode_core(core);
                    PolicyCheckResult::Conflicting(conflicting)
                }
                SatResult::Unknown => PolicyCheckResult::Timeout,
            }
        }
        
        fn encode_policy(&self, policy: &Policy) -> Bool<'static> {
            match policy {
                Policy::Budget { max } => {
                    let budget = Real::new_const(&self.ctx, "budget");
                    budget.le(&Real::from_real(&self.ctx, *max as i64, 1))
                }
                Policy::TimeRange { start, end } => {
                    let hour = Int::new_const(&self.ctx, "hour");
                    let start_constraint = hour.ge(&Int::from_i64(&self.ctx, *start as i64));
                    let end_constraint = hour.le(&Int::from_i64(&self.ctx, *end as i64));
                    start_constraint.and(&[&end_constraint])
                }
                Policy::Region { allowed } => {
                    // Enum encoding
                    let region = Datatype::new_const(&self.ctx, "region", &region_sort);
                    allowed.iter()
                        .map(|r| region._eq(&region_const(r)))
                        .reduce(|a, b| a.or(&[&b]))
                        .unwrap_or(Bool::from_bool(&self.ctx, false))
                }
                // ... more encodings
            }
        }
    }
}

// Fallback: Structural analysis without SMT
#[cfg(not(feature = "smt"))]
pub mod smt {
    use std::collections::HashSet;
    
    pub struct SmtContext;
    
    impl SmtContext {
        pub fn new() -> Self { SmtContext }
        
        pub fn check_policies(&self, policies: &[Policy]) -> PolicyCheckResult {
            // Simple structural conflict detection
            // - Exact contradictions (budget > 100 AND budget < 50)
            // - Disjoint time ranges
            // - Mutually exclusive regions
            
            let mut budget_mins: Vec<u64> = Vec::new();
            let mut budget_maxes: Vec<u64> = Vec::new();
            
            for policy in policies {
                match policy {
                    Policy::Budget { max } => {
                        for min in &budget_mins {
                            if max < min {
                                return PolicyCheckResult::Conflicting(vec![policy.clone()]);
                            }
                        }
                        budget_maxes.push(*max);
                    }
                    Policy::MinBudget { min } => {
                        for max in &budget_maxes {
                            if min > max {
                                return PolicyCheckResult::Conflicting(vec![policy.clone()]);
                            }
                        }
                        budget_mins.push(*min);
                    }
                    // ... more simple checks
                    _ => {}
                }
            }
            
            PolicyCheckResult::Satisfiable
        }
    }
}
```

### Cargo.toml Feature Flag

```toml
[features]
default = []
smt = ["z3"]

[dependencies]
z3 = { version = "0.12", optional = true }
```

### Policy Constraint Types

```rust
#[derive(Debug, Clone)]
pub enum Policy {
    // Resource constraints
    Budget { max: u64 },
    MinBudget { min: u64 },
    RateLimit { requests: u64, window_secs: u64 },
    
    // Temporal constraints
    TimeRange { start_hour: u8, end_hour: u8 },
    DayOfWeek { days: HashSet<Weekday> },
    MaxDuration { secs: u64 },
    
    // Location/data residency
    Region { allowed: Vec<String> },
    DataResidency { region: String },
    
    // Security/privacy
    EncryptionRequired,
    AuditLevel { level: AuditLevel },
    RetentionDays { days: u32 },
    
    // Deontic (OBLIG-related)
    MustNotify { party: String },
    MustApprove { by: String },
    MaxDelegationDepth { depth: u8 },
}
```

## Completion Criteria

- [ ] Property tests pass: All generated policy sets correctly classified
- [ ] Example tests pass: Real-world scenarios work
- [ ] Unsat cores are minimal and meaningful
- [ ] Timeout handling for complex constraints
- [ ] Feature flag works: Builds without `smt` using structural analysis
- [ ] Error messages explain conflicts in user terms
- [ ] Documentation: How to interpret conflict reports

## Dependencies

- TASK-023: Obligation tracking (to know what policies to check)
- External: z3-sys (system Z3 or bundled)

## Estimation

8 hours (includes learning Z3 API, encoding strategies, testing)

## References

- Z3 Guide: https://microsoft.github.io/z3guide/
- Z3 Rust bindings: https://docs.rs/z3/latest/z3/
- SMT-LIB standard: http://smtlib.cs.uiowa.edu/
