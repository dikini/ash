# TASK-275: Enable Workflow Obligation Checking in Type Checker

## Status: 📝 Planned

## Description

Fix the critical issue where obligation checking is effectively disabled in the main type-check pipeline. The `type_check_workflow()` function creates a fresh empty ObligationTracker and immediately marks success if it has no pending items, without feeding `oblige`/`check` structure into the tracker.

## Specification Reference

- SPEC-022: Workflow Typing Specification
- SPEC-003: Type System Specification

## Dependencies

- ✅ TASK-023: Obligation tracking infrastructure
- ✅ TASK-024: Proof obligation generation
- ✅ TASK-274: Engine provider wiring (related)

## Requirements

### Functional Requirements

1. `type_check_workflow()` must populate ObligationTracker from workflow AST
2. `oblige` statements must register obligations in the tracker
3. `check` expressions must verify obligations are satisfied
4. Unsatisfied obligations at workflow end must be type errors
5. Linear obligation tracking must prevent double-satisfaction

### Current State (Broken)

**File:** `crates/ash-typeck/src/check.rs`

```rust
pub fn type_check_workflow(
    &mut self,
    workflow: &Workflow,
) -> Result<TypeCheckResult, TypeError> {
    // ... type inference ...
    
    // Creates empty tracker - obligations never populated!
    let obligation_tracker = ObligationTracker::new();
    
    // Immediately succeeds because tracker is empty
    if obligation_tracker.has_pending() {
        // This never executes
        return Err(TypeError::UnsatisfiedObligations);
    }
    
    Ok(TypeCheckResult::success())
}
```

### Target State (Fixed)

```rust
pub fn type_check_workflow(
    &mut self,
    workflow: &Workflow,
) -> Result<TypeCheckResult, TypeError> {
    // ... type inference ...
    
    // Create and populate obligation tracker
    let mut obligation_tracker = ObligationTracker::new();
    
    // Walk AST to collect obligations from oblige statements
    self.collect_obligations(&workflow.body, &mut obligation_tracker)?;
    
    // Walk AST to verify check expressions satisfy obligations
    self.verify_checks(&workflow.body, &mut obligation_tracker)?;
    
    // Check for unsatisfied obligations at workflow end
    if let Some(pending) = obligation_tracker.pending_obligations() {
        return Err(TypeError::UnsatisfiedObligations {
            obligations: pending,
        });
    }
    
    Ok(TypeCheckResult::success())
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-typeck/tests/obligation_checking_test.rs`

```rust
//! Tests for workflow obligation checking

use ash_typeck::{TypeChecker, TypeError};
use ash_parser::parse;

#[test]
fn test_obligation_must_be_satisfied() {
    let workflow = parse(r#"
        workflow test {
            oblige logging_enabled;
            act log("hello");
            // ERROR: logging_enabled never checked!
        }
    "#).unwrap();
    
    let mut checker = TypeChecker::new();
    let result = checker.type_check_workflow(&workflow);
    
    assert!(matches!(result, Err(TypeError::UnsatisfiedObligations { .. })));
}

#[test]
fn test_satisfied_obligation_passes() {
    let workflow = parse(r#"
        workflow test {
            oblige logging_enabled;
            check logging_enabled;
            act log("hello");
        }
    "#).unwrap();
    
    let mut checker = TypeChecker::new();
    let result = checker.type_check_workflow(&workflow);
    
    assert!(result.is_ok());
}

#[test]
fn test_check_without_obligation_fails() {
    let workflow = parse(r#"
        workflow test {
            check undefined_obligation;  // ERROR: no such obligation
            act log("hello");
        }
    "#).unwrap();
    
    let mut checker = TypeChecker::new();
    let result = checker.type_check_workflow(&workflow);
    
    assert!(matches!(result, Err(TypeError::UnknownObligation { .. })));
}

#[test]
fn test_double_satisfaction_fails() {
    let workflow = parse(r#"
        workflow test {
            oblige once_only;
            check once_only;
            check once_only;  // ERROR: already satisfied
        }
    "#).unwrap();
    
    let mut checker = TypeChecker::new();
    let result = checker.type_check_workflow(&workflow);
    
    assert!(matches!(result, Err(TypeError::ObligationAlreadySatisfied { .. })));
}

proptest! {
    #[test]
    fn obligation_soundness(obligations in vec(obligations_strategy(), 0..10)) {
        // Property: every checked obligation must have been obliged
        // and every obliged obligation must be checked exactly once
    }
}
```

### Step 2: Implement Obligation Collection

**File:** `crates/ash-typeck/src/obligations.rs`

```rust
use ash_core::{Workflow, Step, Expr};
use crate::ObligationTracker;
use crate::error::TypeError;

pub struct ObligationCollector;

impl ObligationCollector {
    pub fn collect(
        &mut self,
        workflow: &Workflow,
        tracker: &mut ObligationTracker,
    ) -> Result<(), TypeError> {
        self.collect_from_steps(&workflow.body, tracker)
    }
    
    fn collect_from_steps(
        &mut self,
        steps: &[Step],
        tracker: &mut ObligationTracker,
    ) -> Result<(), TypeError> {
        for step in steps {
            match step {
                Step::Oblige { obligation } => {
                    tracker.register_obligation(obligation.clone());
                }
                Step::Check { obligation } => {
                    tracker.satisfy_obligation(obligation)?;
                }
                Step::If { then_branch, else_branch, .. } => {
                    self.collect_from_steps(then_branch, tracker)?;
                    if let Some(else_branch) = else_branch {
                        self.collect_from_steps(else_branch, tracker)?;
                    }
                }
                Step::Match { arms, .. } => {
                    for arm in arms {
                        self.collect_from_steps(&arm.body, tracker)?;
                    }
                }
                // ... other step types
                _ => {}
            }
        }
        Ok(())
    }
}
```

### Step 3: Update Type Checker Integration

**File:** `crates/ash-typeck/src/check.rs`

```rust
impl TypeChecker {
    pub fn type_check_workflow(
        &mut self,
        workflow: &Workflow,
    ) -> Result<TypeCheckResult, TypeError> {
        // Existing type inference
        let mut ctx = TypeInferenceContext::new();
        let body_type = self.infer_workflow_type(workflow, &mut ctx)?;
        
        // NEW: Obligation checking
        let mut tracker = ObligationTracker::new();
        let mut collector = ObligationCollector::new();
        
        collector.collect(workflow, &mut tracker)?;
        
        if tracker.has_pending() {
            return Err(TypeError::UnsatisfiedObligations {
                obligations: tracker.pending_obligations(),
            });
        }
        
        Ok(TypeCheckResult {
            workflow_type: body_type,
            obligations_satisfied: true,
        })
    }
}
```

### Step 4: Add Error Types

**File:** `crates/ash-typeck/src/error.rs`

```rust
#[derive(Debug, Clone, Error, PartialEq)]
pub enum TypeError {
    // ... existing errors ...
    
    #[error("unsatisfied obligations: {obligations:?}")]
    UnsatisfiedObligations {
        obligations: Vec<Obligation>,
    },
    
    #[error("unknown obligation: {name}")]
    UnknownObligation {
        name: String,
        span: Span,
    },
    
    #[error("obligation already satisfied: {name}")]
    ObligationAlreadySatisfied {
        name: String,
        span: Span,
    },
}
```

## Verification Steps

- [ ] `cargo test -p ash-typeck --test obligation_checking_test` passes
- [ ] `cargo test -p ash-typeck --test runtime_verification_contracts` passes
- [ ] `cargo test -p ash-engine` passes (integration)
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Working obligation checking in type checker
- SPEC-022 compliance

Required by:
- TASK-276: Fix unsound expression typing (related type system fixes)

## Notes

**Critical Issue**: SPEC-022's linear obligation tracking is currently non-functional. This breaks a core safety guarantee of the language.

**Design Decision**: Collect obligations in a separate pass before verification to handle forward references and complex control flow.

**Edge Cases**:
- Obligations in unreachable code (if false) - still required to be satisfied
- Branches with different obligation sets - union of all possibilities
- Nested workflows - obligations are scoped per-workflow
