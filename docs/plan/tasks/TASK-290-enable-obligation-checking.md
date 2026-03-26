# TASK-290: Enable Workflow Obligation Checking in Type Checker

## Status: 📝 Planned

## Description

Fix the critical issue where workflow obligation checking is effectively disabled in the main type-check pipeline. `type_check_workflow()` creates an empty ObligationTracker and immediately checks it. The oblige/check structure never feeds into the tracker, so SPEC-022 discharge rules, branch discipline, and completion-time rejection are not enforced.

## Specification Reference

- SPEC-022: Workflow Typing with Constraints
- SPEC-003: Type System Specification

## Dependencies

- ✅ TASK-226: Workflow contracts AST
- ✅ TASK-227: Type check obligations as linear resources
- ✅ TASK-228: Requirement checking

## Critical File Locations

- `crates/ash-typeck/src/lib.rs:97` - empty ObligationTracker created
- `crates/ash-typeck/src/lib.rs:119` - tracker checked without any obligations added

## Requirements

### Functional Requirements

1. `oblige` statements must add obligations to the tracker
2. `check` expressions must consume obligations from the tracker
3. Branch join points must require matching obligation sets
4. Workflow completion must require all obligations discharged
5. SPEC-022 discharge rules must be enforced

### Current State (Broken)

**File:** `crates/ash-typeck/src/lib.rs:97-119`

```rust
pub fn type_check_workflow(&mut self, workflow: &Workflow) -> Result<Type, TypeError> {
    // Empty tracker - obligations never added!
    let mut obligation_tracker = ObligationTracker::new();
    
    for step in &workflow.body {
        match step {
            Step::Oblige { obligation } => {
                // Type check the obligation but DON'T track it
                self.type_check_obligation(obligation)?;
                // MISSING: obligation_tracker.add(obligation);
            }
            Step::Check { obligation } => {
                // Type check the check but DON'T consume
                self.type_check_check(obligation)?;
                // MISSING: obligation_tracker.consume(obligation)?;
            }
            // ... other steps
        }
    }
    
    // Check empty tracker - always succeeds!
    obligation_tracker.check_complete()?;  // Nothing to check!
    
    Ok(Type::Unit)
}
```

Problems:
1. Obligations are type-checked but not tracked
2. Check expressions don't consume tracked obligations
3. Branch discipline not enforced
4. Completion check always succeeds
5. SPEC-022 rules are not implemented

### Target State (Fixed)

```rust
pub fn type_check_workflow(&mut self, workflow: &Workflow) -> Result<Type, TypeError> {
    let mut obligation_tracker = ObligationTracker::new();
    let mut branch_obligations: Vec<ObligationSet> = vec![];
    
    for step in &workflow.body {
        match step {
            Step::Oblige { obligation } => {
                let obligation_ty = self.type_check_obligation(obligation)?;
                // FIX: Track the obligation
                obligation_tracker.add(obligation.clone(), obligation_ty)?;
            }
            Step::Check { obligation } => {
                let obligation_ty = self.type_check_check(obligation)?;
                // FIX: Consume the obligation
                obligation_tracker.consume(obligation, obligation_ty)?;
            }
            Step::If { condition, then_branch, else_branch } => {
                self.type_check_expr(condition)?;
                
                // Type check each branch with a fork of the tracker
                let then_obligations = self.check_branch(
                    then_branch,
                    obligation_tracker.fork()
                )?;
                
                let else_obligations = if let Some(else_branch) = else_branch {
                    self.check_branch(
                        else_branch,
                        obligation_tracker.fork()
                    )?
                } else {
                    obligation_tracker.current().clone()
                };
                
                // FIX: Branch join requires matching obligation sets
                let joined = ObligationSet::join(then_obligations, else_obligations)?;
                obligation_tracker.replace(joined);
            }
            Step::Match { scrutinee, arms } => {
                let scrutinee_ty = self.type_check_expr(scrutinee)?;
                
                let mut arm_obligations: Vec<ObligationSet> = vec![];
                for arm in arms {
                    let arm_tracker = obligation_tracker.fork();
                    self.check_pattern(&arm.pattern, &scrutinee_ty)?;
                    let arm_result = self.check_branch(&arm.body, arm_tracker)?;
                    arm_obligations.push(arm_result);
                }
                
                // FIX: Match arms must have compatible obligation sets
                let joined = arm_obligations.into_iter()
                    .reduce(ObligationSet::join)
                    .unwrap_or_default();
                obligation_tracker.replace(joined);
            }
            // ... other steps
        }
    }
    
    // FIX: Check all obligations discharged at completion
    obligation_tracker.check_complete()?;  // Now actually checks!
    
    Ok(Type::Unit)
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-typeck/tests/obligation_tracking_test.rs`

```rust
//! Tests for obligation tracking enforcement

use ash_typeck::TypeChecker;
use ash_parser::parse_workflow;

#[test]
fn test_obligate_requires_check() {
    let workflow = r#"
        workflow test {
            oblige CanRead("data");
            // Missing: check CanRead("data");
            act read("data");
        }
    "#;
    
    let parsed = parse_workflow(workflow).unwrap();
    let mut checker = TypeChecker::new();
    
    // Should fail: obligation not discharged
    let result = checker.type_check_workflow(&parsed);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("undischarged obligation"));
}

#[test]
fn test_check_without_obligate_fails() {
    let workflow = r#"
        workflow test {
            // No oblige, but trying to check
            check CanRead("data");
            act read("data");
        }
    "#;
    
    let parsed = parse_workflow(workflow).unwrap();
    let mut checker = TypeChecker::new();
    
    // Should fail: checking obligation that wasn't created
    let result = checker.type_check_workflow(&parsed);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no such obligation"));
}

#[test]
fn test_obligate_check_pair_succeeds() {
    let workflow = r#"
        workflow test {
            oblige CanRead("data");
            check CanRead("data");
            act read("data");
        }
    "#;
    
    let parsed = parse_workflow(workflow).unwrap();
    let mut checker = TypeChecker::new();
    
    // Should succeed: obligation created and discharged
    let result = checker.type_check_workflow(&parsed);
    assert!(result.is_ok());
}

#[test]
fn test_branch_obligation_discipline() {
    let workflow = r#"
        workflow test(cond: Bool) {
            oblige CanRead("data");
            if cond {
                check CanRead("data");
                act read("data");
            } else {
                // Missing check in else branch
                act skip();
            }
        }
    "#;
    
    let parsed = parse_workflow(workflow).unwrap();
    let mut checker = TypeChecker::new();
    
    // Should fail: branches have incompatible obligation sets
    let result = checker.type_check_workflow(&parsed);
    assert!(result.is_err());
}

#[test]
fn test_match_arm_obligation_discipline() {
    let workflow = r#"
        workflow test(x: Option<Int>) {
            oblige CanRead("data");
            match x {
                Some(v) => {
                    check CanRead("data");
                    act read("data");
                }
                None => {
                    // Missing check in None arm
                    act skip();
                }
            }
        }
    "#;
    
    let parsed = parse_workflow(workflow).unwrap();
    let mut checker = TypeChecker::new();
    
    // Should fail: match arms have incompatible obligation sets
    let result = checker.type_check_workflow(&parsed);
    assert!(result.is_err());
}

#[test]
fn test_nested_obligation_tracking() {
    let workflow = r#"
        workflow test {
            oblige CanRead("a");
            oblige CanWrite("b");
            
            check CanRead("a");
            act read("a");
            
            check CanWrite("b");
            act write("b");
        }
    "#;
    
    let parsed = parse_workflow(workflow).unwrap();
    let mut checker = TypeChecker::new();
    
    // Should succeed: all obligations discharged
    let result = checker.type_check_workflow(&parsed);
    assert!(result.is_ok());
}

#[test]
fn test_obligation_linear_usage() {
    let workflow = r#"
        workflow test {
            oblige CanRead("data");
            check CanRead("data");
            check CanRead("data");  // Second check - should fail
            act read("data");
        }
    "#;
    
    let parsed = parse_workflow(workflow).unwrap();
    let mut checker = TypeChecker::new();
    
    // Should fail: obligation already consumed
    let result = checker.type_check_workflow(&parsed);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already consumed"));
}

proptest! {
    #[test]
    fn obligation_tracker_is_linear(obligations in prop::collection::vec(obligation_strategy(), 0..10)) {
        let mut tracker = ObligationTracker::new();
        
        // Add all obligations
        for obl in &obligations {
            tracker.add(obl.clone()).unwrap();
        }
        
        // Each can be consumed exactly once
        for obl in &obligations {
            assert!(tracker.consume(obl.clone()).is_ok());
            assert!(tracker.consume(obl.clone()).is_err());  // Second consume fails
        }
        
        // All discharged
        assert!(tracker.check_complete().is_ok());
    }
}
```

### Step 2: Implement Obligation Tracking in Type Checker

**File:** `crates/ash-typeck/src/lib.rs`

```rust
use crate::obligation_tracker::ObligationTracker;

impl TypeChecker {
    pub fn type_check_workflow(&mut self, workflow: &Workflow) -> Result<Type, TypeError> {
        let mut tracker = ObligationTracker::new();
        let mut ctx = TypeContext::new();
        
        // Add parameters to context
        for param in &workflow.params {
            ctx.bind(param.name.clone(), param.ty.clone());
        }
        
        for step in &workflow.body {
            self.type_check_step(step, &mut ctx, &mut tracker)?;
        }
        
        // Check all obligations discharged
        tracker.check_complete()
            .map_err(|e| TypeError::UndischargedObligations(e))?;
        
        Ok(Type::Unit)
    }
    
    fn type_check_step(
        &mut self,
        step: &Step,
        ctx: &mut TypeContext,
        tracker: &mut ObligationTracker,
    ) -> Result<(), TypeError> {
        match step {
            Step::Let { name, value, ty } => {
                let value_ty = self.type_check_expr(value, ctx)?;
                if let Some(expected_ty) = ty {
                    self.unify(&value_ty, expected_ty)?;
                }
                ctx.bind(name.clone(), value_ty);
            }
            Step::Oblige { obligation } => {
                let obl_ty = self.type_check_obligation(obligation, ctx)?;
                tracker.add(obligation.clone(), obl_ty)?;
            }
            Step::Check { obligation } => {
                let obl_ty = self.type_check_obligation(obligation, ctx)?;
                tracker.consume(obligation, obl_ty)?;
            }
            Step::If { condition, then_branch, else_branch } => {
                let cond_ty = self.type_check_expr(condition, ctx)?;
                self.unify(&cond_ty, &Type::Bool)?;
                
                // Check branches and merge obligation sets
                let then_tracker = tracker.fork();
                let then_obligations = self.check_branch(then_branch, ctx, then_tracker)?;
                
                let else_obligations = if let Some(else_branch) = else_branch {
                    let else_tracker = tracker.fork();
                    self.check_branch(else_branch, ctx, else_tracker)?
                } else {
                    tracker.current().clone()
                };
                
                let joined = ObligationSet::join(then_obligations, else_obligations)
                    .map_err(|e| TypeError::BranchObligationMismatch(e))?;
                tracker.replace(joined);
            }
            Step::Match { scrutinee, arms } => {
                let scrut_ty = self.type_check_expr(scrutinee, ctx)?;
                
                let mut arm_obligations = vec![];
                for arm in arms {
                    let mut arm_ctx = ctx.clone();
                    self.bind_pattern(&arm.pattern, &scrut_ty, &mut arm_ctx)?;
                    let arm_tracker = tracker.fork();
                    let arm_obl = self.check_branch(&arm.body, &mut arm_ctx, arm_tracker)?;
                    arm_obligations.push(arm_obl);
                }
                
                let joined = arm_obligations.into_iter()
                    .reduce(ObligationSet::join)
                    .unwrap_or_default();
                tracker.replace(joined);
            }
            // ... other steps
        }
        
        Ok(())
    }
}
```

### Step 3: Implement ObligationTracker

**File:** `crates/ash-typeck/src/obligation_tracker.rs`

```rust
//! Linear obligation tracking per SPEC-022

use std::collections::HashMap;
use ash_core::{Obligation, Type};

#[derive(Debug, Clone)]
pub struct ObligationTracker {
    active: HashMap<ObligationKey, ObligationState>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ObligationKey {
    name: String,
    target: String,
}

#[derive(Debug, Clone)]
struct ObligationState {
    ty: Type,
    consumed: bool,
}

impl ObligationTracker {
    pub fn new() -> Self {
        Self {
            active: HashMap::new(),
        }
    }
    
    /// Add a new obligation (must not already exist)
    pub fn add(&mut self, obl: Obligation, ty: Type) -> Result<(), TrackerError> {
        let key = ObligationKey::from(&obl);
        if self.active.contains_key(&key) {
            return Err(TrackerError::DuplicateObligation(obl));
        }
        self.active.insert(key, ObligationState { ty, consumed: false });
        Ok(())
    }
    
    /// Consume an obligation (must exist and not be consumed)
    pub fn consume(&mut self, obl: &Obligation, ty: Type) -> Result<(), TrackerError> {
        let key = ObligationKey::from(obl);
        let state = self.active.get_mut(&key)
            .ok_or_else(|| TrackerError::NoSuchObligation(obl.clone()))?;
        
        if state.consumed {
            return Err(TrackerError::AlreadyConsumed(obl.clone()));
        }
        
        // Type must match
        if state.ty != ty {
            return Err(TrackerError::TypeMismatch {
                expected: state.ty.clone(),
                actual: ty,
            });
        }
        
        state.consumed = true;
        Ok(())
    }
    
    /// Check all obligations have been consumed
    pub fn check_complete(&self) -> Result<(), TrackerError> {
        let undischarged: Vec<_> = self.active.iter()
            .filter(|(_, state)| !state.consumed)
            .map(|(key, _)| key)
            .collect();
        
        if !undischarged.is_empty() {
            return Err(TrackerError::UndischargedObligations(
                undischarged.into_iter().map(|k| k.to_obligation()).collect()
            ));
        }
        
        Ok(())
    }
    
    /// Fork for branch checking
    pub fn fork(&self) -> Self {
        self.clone()
    }
    
    /// Replace current state (for branch joining)
    pub fn replace(&mut self, other: ObligationSet) {
        self.active = other.into_map();
    }
    
    /// Get current obligation set
    pub fn current(&self) -> ObligationSet {
        ObligationSet::from_map(self.active.clone())
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-typeck --test obligation_tracking_test` passes
- [ ] Obligations without checks fail type checking
- [ ] Checks without obligations fail type checking
- [ ] Branch obligation discipline enforced
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Working obligation tracking in type checker
- SPEC-022 compliance for oblige/check semantics

Required by:
- Full workflow contract enforcement
- Capability requirement verification

## Notes

**Critical Issue**: This is a SPEC-022 compliance violation. The oblige/check syntax exists but has no semantic enforcement.

**Risk Assessment**: High - core type system feature is non-functional.

**Implementation Strategy**:
1. First: Implement ObligationTracker with linear semantics
2. Second: Wire tracker into type_check_workflow
3. Third: Add branch obligation discipline
4. Fourth: Add comprehensive tests

**Testing Strategy**: Need tests for success cases, missing oblige, missing check, branch mismatch, match arm mismatch, and linear consumption.
