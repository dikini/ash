# Test Generation from Operational Semantics & Correctness Proofs

## Overview

This document describes how to generate test programs from operational semantics and approaches to proving implementation correctness.

## Part 1: Generating Test Programs from Semantics

### 1.1 Property-Based Test Generation

Given operational semantics rules, we can derive generators for:

```rust
/// Strategy: For each inference rule, generate valid premises → conclusion
///
/// Rule: (MATCH-VARIANT)
///   eval(Γ, scrutinee) ↝ Variant(t, V, fields)
///   lookup(arms, V) = ARM(pat, body)
///   ...
///   ─────────────────────────────────────────────────
///   Γ ⊢ MATCH scrutinee { V(pat) => body } ⇓ v
///
/// Generator derived from rule:
proptest! {
    #[test]
    fn prop_match_variant_semantics(
        // Generate valid premise: scrutinee is a Variant
        scrutinee in variant_generator(),
        // Generate matching arm
        arm in matching_arm_generator(&scrutinee),
    ) {
        // When we run the interpreter...
        let result = interpret_match(scrutinee, vec![arm.clone()]);
        
        // ...the conclusion should hold
        assert!(result.is_ok());
        
        // Bindings from pattern should match fields
        let bindings = result.bindings;
        verify_bindings_match_pattern(&bindings, &arm.pattern, &scrutinee);
    }
}
```

### 1.2 Rule Coverage-Based Generation

```rust
/// Ensure every inference rule is exercised
struct RuleCoverage {
    /// Track which rules have been tested
    covered_rules: HashSet<String>,
}

impl RuleCoverage {
    fn generate_for_rule(&mut self, rule_name: &str) -> impl Strategy<Value = Workflow> {
        match rule_name {
            "CONSTRUCTOR-ENUM" => Self::gen_constructor_enum(),
            "CONSTRUCTOR-STRUCT" => Self::gen_constructor_struct(),
            "MATCH-VARIANT" => Self::gen_match_variant(),
            "MATCH-WILDCARD" => Self::gen_match_wildcard(),
            "IF-LET-SUCCESS" => Self::gen_if_let_success(),
            "IF-LET-FAIL" => Self::gen_if_let_fail(),
            "SPAWN-WORKFLOW" => Self::gen_spawn_workflow(),
            "SEND-CONTROL-SOME" => Self::gen_send_control_some(),
            "SEND-CONTROL-NONE" => Self::gen_send_control_none(),
            _ => panic!("Unknown rule: {}", rule_name),
        }
    }
    
    /// Generate a constructor for an enum variant
    fn gen_constructor_enum() -> impl Strategy<Value = Workflow> {
        (arbitrary_type_name(), arbitrary_variant_name(), arbitrary_fields())
            .prop_map(|(typ, variant, fields)| {
                Workflow::Expr(Expr::Constructor {
                    name: format!("{}::{}", typ, variant),
                    fields,
                })
            })
    }
    
    /// Generate a match expression with a variant arm
    fn gen_match_variant() -> impl Strategy<Value = Workflow> {
        (arbitrary_variant(), arbitrary_pattern(), arbitrary_expr())
            .prop_map(|(variant, pattern, body)| {
                let scrutinee = Expr::Constructor {
                    name: variant.name.clone(),
                    fields: variant.fields.clone(),
                };
                Workflow::Expr(Expr::Match {
                    scrutinee: Box::new(scrutinee),
                    arms: vec![MatchArm { pattern, body }],
                })
            })
    }
}

/// Ensure all rules are covered over time
proptest! {
    #![proptest_config(ProptestConfig {
        // Run for a long time to find edge cases
        cases: 10000,
        ..Default::default()
    })]
    
    #[test]
    fn prop_rule_coverage((rule, workflow) in arbitrary_rule_and_workflow()) {
        let result = interpret(workflow);
        
        // Record that this rule was tested
        COVERAGE.record(&rule);
        
        // The specific property depends on the rule
        verify_rule_property(&rule, &workflow, &result);
    }
}
```

### 1.3 Oracle-Based Testing

```rust
/// Use the semantics specification as an oracle
struct SemanticOracle;

impl SemanticOracle {
    /// Compute expected result from semantics rules
    fn expected_result(&self, workflow: &Workflow, context: &Context) -> ExpectedResult {
        // Trace through inference rules
        self.apply_rules(workflow, context)
    }
    
    /// Apply inference rules to derive result
    fn apply_rules(&self, w: &Workflow, ctx: &Context) -> ExpectedResult {
        match w {
            Workflow::Expr(Expr::Match { scrutinee, arms }) => {
                // Apply (MATCH-VARIANT) or (MATCH-WILDCARD)
                let scrut_val = self.eval(ctx, scrutinee);
                match &scrut_val {
                    Value::Variant { variant, fields, .. } => {
                        // Find matching arm
                        for arm in arms {
                            if pattern_matches_variant(&arm.pattern, variant) {
                                let new_ctx = bind_pattern(ctx, &arm.pattern, fields);
                                return self.apply_rules(&Workflow::Expr(arm.body.clone()), &new_ctx);
                            }
                        }
                        ExpectedResult::Error(Error::NonExhaustiveMatch)
                    }
                    _ => ExpectedResult::Error(Error::TypeMismatch),
                }
            }
            // ... other cases
        }
    }
}

proptest! {
    #[test]
    fn prop_semantics_oracle(workflow in arbitrary_well_typed_workflow()) {
        let oracle = SemanticOracle;
        let expected = oracle.expected_result(&workflow, &Context::default());
        
        let actual = interpret(workflow);
        
        assert_eq!(expected, actual.into(), 
            "Implementation diverged from semantics oracle");
    }
}
```

### 1.4 Mutation Testing

```rust
/// Generate variants of valid programs to catch edge cases
struct Mutator;

impl Mutator {
    /// Mutate a pattern to create partial coverage
    fn mutate_pattern(pat: &Pattern) -> Vec<Pattern> {
        match pat {
            Pattern::Variant { name, fields } => {
                let mut mutations = vec![];
                
                // Remove a field binding
                if let Some(fields) = fields {
                    for i in 0..fields.len() {
                        let mut mutated = fields.clone();
                        mutated.remove(i);
                        mutations.push(Pattern::Variant {
                            name: name.clone(),
                            fields: Some(mutated),
                        });
                    }
                }
                
                // Change to wildcard
                mutations.push(Pattern::Wildcard);
                
                mutations
            }
            _ => vec![],
        }
    }
    
    /// Generate a non-exhaustive match (should fail type check)
    fn make_non_exhaustive(match_expr: &Expr) -> Expr {
        if let Expr::Match { scrutinee, arms } = match_expr {
            // Remove last arm
            let mut mutated_arms = arms.clone();
            mutated_arms.pop();
            Expr::Match {
                scrutinee: scrutinee.clone(),
                arms: mutated_arms,
            }
        } else {
            match_expr.clone()
        }
    }
}

proptest! {
    #[test]
    fn prop_mutation_testing(
        (original, mutated) in arbitrary_workflow_pair()
            .prop_filter("different", |(o, m)| o != m)
    ) {
        let original_result = interpret(original);
        let mutated_result = interpret(mutated);
        
        // Mutated version should either:
        // 1. Fail type check (if we broke exhaustiveness)
        // 2. Produce different result (if semantics changed)
        // 3. Fail at runtime (if we created invalid pattern)
        
        if type_check(&mutated).is_ok() {
            // If it type checks, results should differ meaningfully
            assert_ne!(original_result.value, mutated_result.value,
                "Mutation had no effect - possible redundancy");
        }
    }
}
```

### 1.5 Long-Running Fuzzing Campaigns

```rust
/// Continuous fuzzing setup for catching bugs over time
#[cfg(fuzzing)]
mod fuzz_campaign {
    use libfuzzer_sys::fuzz_target;
    
    /// Fuzz target that runs indefinitely
    fuzz_target!(|data: &[u8]| {
        if let Some(workflow) = deserialize_workflow(data) {
            // Run with timeout to catch infinite loops
            let result = std::panic::catch_unwind(|| {
                run_with_timeout(|| interpret(&workflow), Duration::from_secs(5))
            });
            
            match result {
                Ok(Ok(value)) => {
                    // Verify basic invariants
                    assert!(verify_value_invariants(&value));
                }
                Ok(Err(e)) => {
                    // Errors are ok, but shouldn't panic
                    log_error(&workflow, &e);
                }
                Err(_) => {
                    panic!("Interpreter panicked on: {:?}", workflow);
                }
            }
        }
    });
}

/// Regression testing: save crashes and re-run
struct RegressionSuite {
    crashes: Vec<(Workflow, Error)>,
}

impl RegressionSuite {
    fn run(&self) {
        for (workflow, expected_error) in &self.crashes {
            let result = std::panic::catch_unwind(|| interpret(workflow));
            
            // Should either:
            // 1. Produce the expected error (bug not fixed yet)
            // 2. Succeed (bug was fixed)
            // 3. Produce a different error (bug partially fixed)
            
            match result {
                Ok(Ok(_)) => println!("Bug fixed for: {:?}", workflow),
                Ok(Err(e)) if e == *expected_error => {
                    println!("Bug still present: {:?}", workflow)
                }
                Ok(Err(e)) => println!("Bug changed: {:?} -> {:?}", expected_error, e),
                Err(_) => panic!("Regression: interpreter panics on known case"),
            }
        }
    }
}
```

## Part 2: Proving Correctness Beyond Testing

### 2.1 Formal Verification Approaches

```
┌─────────────────────────────────────────────────────────────────┐
│                    CORRECTNESS STRATEGIES                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Level 1: Testing (Dynamic)                                      │
│  ├── Unit tests for individual rules                             │
│  ├── Property tests for invariants                               │
│  └── Fuzzing for edge cases                                      │
│                                                                  │
│  Level 2: Model Checking (Semi-Formal)                           │
│  ├── State space exploration                                     │
│  ├── Temporal property verification                              │
│  └── Bounded model checking                                      │
│                                                                  │
│  Level 3: Type System (Static)                                   │
│  ├── Type safety guarantees                                      │
│  ├── Effect tracking                                             │
│  └── Exhaustiveness checking                                     │
│                                                                  │
│  Level 4: Formal Proof (Mathematical)                            │
│  ├── Soundness proof (well-typed → can't go wrong)               │
│  ├── Completeness proof (all valid programs work)                │
│  └── Behavioral equivalence (impl ≡ spec)                        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Type Safety as Proof

The type system itself provides static guarantees:

```rust
/// Theorem: Type Safety (Preservation + Progress)
/// 
/// Preservation: If Γ ⊢ e : τ and e → e', then Γ ⊢ e' : τ
/// Progress: If Γ ⊢ e : τ, then either e is a value or e → e'
/// 
/// Corollary: Well-typed programs don't get stuck

/// Proof sketch for ADT pattern matching:
/// 
/// Given: Γ ⊢ match scrutinee { arms } : τ
///       Γ ⊢ scrutinee : Option<T>
///       arms are exhaustive (checked by type checker)
/// 
/// To show: Evaluation makes progress
/// 
/// Case analysis on scrutinee's value:
/// 1. scrutinee = Some(v): By exhaustiveness, ∃ arm matching Some
///    → (MATCH-VARIANT) applies → body evaluates
/// 2. scrutinee = None: By exhaustiveness, ∃ arm matching None  
///    → (MATCH-NONE) applies → body evaluates
/// 
/// Therefore, match always makes progress (QED)

/// Rust implementation enforces this via types:
pub fn interpret_match(
    scrutinee: Value,
    arms: Vec<MatchArm>,  // Type-checked to be exhaustive
) -> Result<Value, Error> {
    // By type system guarantee, this always succeeds
    for arm in arms {
        if let Some(bindings) = match_pattern(&arm.pattern, &scrutinee) {
            return eval_with_bindings(arm.body, bindings);
        }
    }
    
    // Type system guarantees we never reach here
    unreachable!("Non-exhaustive match - type checker bug!")
}
```

### 2.3 Mechanized Proofs (Coq/Lean)

```coq
(* Formal proof of pattern matching soundness in Coq *)

Inductive value : Type :=
  | VInt : Z -> value
  | VVariant : string -> string -> list (string * value) -> value
  | VNone : value
  | VSome : value -> value.

Inductive pattern : Type :=
  | PVar : string -> pattern
  | PVariant : string -> list (string * pattern) -> pattern
  | PWildcard : pattern.

Inductive match_pattern : pattern -> value -> option (string -> value) -> Prop :=
  | MP_Var : forall x v,
      match_pattern (PVar x) v (Some (fun y => if y =? x then v else ...))
  | MP_Variant : forall v_name fields p_name p_fields bindings,
      v_name = p_name ->
      match_fields p_fields fields bindings ->
      match_pattern (PVariant p_name p_fields) 
                    (VVariant _ v_name fields) 
                    bindings
  | MP_Wildcard : forall v,
      match_pattern PWildcard v (Some empty_binding).

(* Theorem: Exhaustive patterns always match *)
Theorem exhaustive_match_succeeds :
  forall (scrutinee : value) (arms : list (pattern * expr)),
    type_check scrutinee arms = Ok _ ->
    exists bindings body,
      In (body_pattern, body) arms /\
      match_pattern body_pattern scrutinee bindings.
Proof.
  intros scrutinee arms H_typed.
  (* Proof uses type checker's exhaustiveness proof *)
  destruct scrutinee eqn:E;
  (* Case analysis on scrutinee type *)
  - (* Variant case *)
    apply exhaustiveness_implies_match;
    exact H_typed.
  - (* Other cases... *)
Admitted.
```

### 2.4 Refinement Types

```rust
/// Use refinement types to encode semantics in types
/// (requires liquid Haskell or similar, adapted for Rust)

/// Precondition: Pattern is exhaustive for the scrutinee type
/// Postcondition: Result is the evaluation of the matching arm
#[requires(exhaustive(arms, typeof(scrutinee)))]
#[ensures(result == eval(find_matching_arm(arms, scrutinee)))]
pub fn interpret_match(
    scrutinee: Value,
    arms: Vec<MatchArm>,
) -> Value {
    // Implementation
}

/// Precondition: Control link has not been transferred
/// Postcondition: Control link is now None in context
#[requires(is_some(control_link))]
#[ensures(context.control_link == None)]
pub fn transfer_control(
    target: InstanceAddr,
    control_link: ControlLink,
    context: &mut Context,
) {
    // Transfer logic
}
```

### 2.5 Model Checking

```rust
/// Verify temporal properties of workflow execution
use model_checking::*;

/// Property: Control links are never double-transferred
#[test]
fn prop_no_double_transfer() {
    let property = always(
        implies(
            trace_event(is_control_transfer),
            next(always(not(trace_event(is_control_transfer_same_id))))
        )
    );
    
    let model = WorkflowModel::new(all_possible_spawn_workflows());
    
    assert!(model_check(&model, &property).is_success());
}

/// Property: Pattern matching is exhaustive (runtime check agrees with type check)
#[test]
fn prop_exhaustiveness_sound() {
    let property = always(
        implies(
            type_checker_says_exhaustive(match_expr),
            runtime_match_succeeds(match_expr)
        )
    );
    
    let model = WorkflowModel::new(all_well_typed_matches());
    
    assert!(model_check(&model, &property).is_success());
}
```

### 2.6 Differential Testing (Multiple Implementations)

```rust
/// Compare multiple implementations for equivalence

/// Reference interpreter (directly from semantics)
struct ReferenceInterpreter;

/// Optimized production interpreter
struct ProductionInterpreter;

/// Verified core (small, proven correct)
struct VerifiedCore;

proptest! {
    #[test]
    fn prop_interpreter_equivalence(workflow in arbitrary_well_typed_workflow()) {
        let ref_result = ReferenceInterpreter::interpret(&workflow);
        let prod_result = ProductionInterpreter::interpret(&workflow);
        let verified_result = VerifiedCore::interpret(&workflow);
        
        // All should agree
        assert_eq!(ref_result, prod_result, 
            "Production diverged from reference");
        assert_eq!(ref_result, verified_result,
            "Verified core diverged from reference");
    }
}
```

### 2.7 Proof Carrying Code

```rust
/// Attach proofs to code for external verification

/// Proof that this function satisfies the semantics
#[proof(
    "forall scrutinee, arms.",
    "  exhaustive(arms, typeof(scrutinee)) =>",
    "  interpret_match(scrutinee, arms) = eval(find_match(arms, scrutinee))"
)]
pub fn interpret_match(
    scrutinee: Value,
    arms: Vec<MatchArm>,
) -> Value {
    // Implementation with proof attached
}

/// External verifier checks the proof
#[test]
fn verify_all_proofs() {
    for func in all_annotated_functions() {
        assert!(verify_proof(&func).is_valid(),
            "Proof failed for {}", func.name);
    }
}
```

## Part 3: Continuous Verification Strategy

### 3.1 CI/CD Integration

```yaml
# .github/workflows/verification.yml
name: Continuous Verification

on: [push, pull_request, schedule]

jobs:
  # Fast: Unit tests
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: cargo test --lib

  # Medium: Property tests (10000 cases)
  property-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: cargo test --release prop_ -- --cases 10000

  # Slow: Fuzzing (30 min)
  fuzzing:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule' # Nightly
    steps:
      - uses: actions/checkout@v3
      - run: cargo fuzz run adt_interpreter -- -max_total_time=1800

  # Formal verification (on release candidates)
  formal-verification:
    runs-on: ubuntu-latest
    if: contains(github.ref, 'release')
    steps:
      - uses: actions/checkout@v3
      - run: ./scripts/verify-core.sh
```

### 3.2 Coverage Tracking

```rust
/// Track which semantic rules have been tested
static RULE_COVERAGE: RwLock<HashSet<String>> = RwLock::new(HashSet::new());

pub fn record_rule_test(rule_name: &str) {
    RULE_COVERAGE.write().unwrap().insert(rule_name.to_string());
}

pub fn coverage_report() -> CoverageReport {
    let all_rules = vec![
        "CONSTRUCTOR-ENUM",
        "CONSTRUCTOR-STRUCT",
        "MATCH-VARIANT",
        "MATCH-WILDCARD",
        "IF-LET-SUCCESS",
        "IF-LET-FAIL",
        "SPAWN-WORKFLOW",
        "SEND-CONTROL-SOME",
        "SEND-CONTROL-NONE",
    ];
    
    let covered = RULE_COVERAGE.read().unwrap();
    
    CoverageReport {
        total: all_rules.len(),
        covered: covered.len(),
        uncovered: all_rules.iter()
            .filter(|r| !covered.contains(**r))
            .cloned()
            .collect(),
    }
}
```

## Summary

| Approach | Cost | Confidence | Best For |
|----------|------|------------|----------|
| Unit Tests | Low | Low-Medium | Regression prevention |
| Property Tests | Medium | Medium | Finding edge cases |
| Fuzzing | Medium | Medium-High | Long-term bug finding |
| Type System | Low | Medium-High | Static guarantees |
| Model Checking | High | High | Protocol verification |
| Formal Proof | Very High | Very High | Core algorithms |
| Differential Testing | Medium | High | Implementation validation |

## Recommendation

For Ash ADT implementation:

1. **Immediate**: Property tests for all semantic rules (catch bugs now)
2. **Short-term**: Exhaustive pattern coverage tracking (ensure completeness)
3. **Medium-term**: Formal proof for pattern matching core (foundational correctness)
4. **Long-term**: Differential testing with reference interpreter (ongoing validation)
5. **Continuous**: Fuzzing campaign running nightly (catch regressions)
