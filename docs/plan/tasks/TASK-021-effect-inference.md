# TASK-021: Effect Inference

## Status: 🟢 Complete

## Description

Implement effect inference that computes the minimal effect for each workflow expression using the effect lattice.

## Specification Reference

- SPEC-003: Type System - Section 5. Effect Inference
- SPEC-001: IR - Section 2.1 Effect Lattice

## Requirements

### Effect Inference Function

```rust
/// Infer the effect of a workflow
pub fn infer_effect(workflow: &Workflow) -> Effect {
    match workflow {
        Workflow::Observe { continuation, .. } => {
            // Effect is join of epistemic and continuation effect
            Effect::Epistemic.join(infer_effect(continuation))
        }
        
        Workflow::Orient { continuation, .. } => {
            Effect::Deliberative.join(infer_effect(continuation))
        }
        
        Workflow::Propose { continuation, .. } => {
            Effect::Deliberative.join(infer_effect(continuation))
        }
        
        Workflow::Decide { continuation, .. } => {
            Effect::Evaluative.join(infer_effect(continuation))
        }
        
        Workflow::Check { continuation, .. } => {
            Effect::Evaluative.join(infer_effect(continuation))
        }
        
        Workflow::Act { .. } => {
            // Act is always operational
            Effect::Operational
        }
        
        Workflow::Oblig { workflow, .. } => {
            // Obligation adds evaluative effect
            Effect::Evaluative.join(infer_effect(workflow))
        }
        
        Workflow::Let { continuation, .. } => {
            infer_effect(continuation)
        }
        
        Workflow::If { then_branch, else_branch, .. } => {
            let then_eff = infer_effect(then_branch);
            let else_eff = else_branch
                .as_ref()
                .map(|e| infer_effect(e))
                .unwrap_or(Effect::Epistemic);
            then_eff.join(else_eff)
        }
        
        Workflow::Seq { first, second } => {
            let eff1 = infer_effect(first);
            let eff2 = infer_effect(second);
            eff1.join(eff2)
        }
        
        Workflow::Par { workflows } => {
            workflows.iter()
                .map(|w| infer_effect(w))
                .fold(Effect::Epistemic, Effect::join)
        }
        
        Workflow::ForEach { body, .. } => {
            infer_effect(body)
        }
        
        Workflow::With { workflow, .. } => {
            infer_effect(workflow)
        }
        
        Workflow::Maybe { primary, fallback } => {
            let eff1 = infer_effect(primary);
            let eff2 = infer_effect(fallback);
            eff1.join(eff2)
        }
        
        Workflow::Must { workflow } => {
            infer_effect(workflow)
        }
        
        Workflow::Attempt { try_body, catch_body } => {
            let eff1 = infer_effect(try_body);
            let eff2 = infer_effect(catch_body);
            eff1.join(eff2)
        }
        
        Workflow::Ret { .. } => Effect::Epistemic,
        Workflow::Done => Effect::Epistemic,
    }
}
```

### Effect Checking

```rust
/// Check if a workflow's effect is compatible with a bound
pub fn check_effect_bound(workflow: &Workflow, bound: Effect) -> Result<(), EffectError> {
    let inferred = infer_effect(workflow);
    
    if inferred <= bound {
        Ok(())
    } else {
        Err(EffectError::EffectExceeded {
            inferred,
            bound,
            workflow: format!("{:?}", workflow),
        })
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum EffectError {
    #[error("Effect exceeded: workflow has effect {inferred}, but bound is {bound}")]
    EffectExceeded { inferred: Effect, bound: Effect, workflow: String },
    
    #[error("Operational action without decision")]
    UnauthorizedOperational,
}
```

### Effect Annotation

```rust
/// Annotate a workflow with its inferred effect
#[derive(Debug, Clone)]
pub struct EffectAnnotated<T> {
    pub value: T,
    pub effect: Effect,
}

/// Annotate all sub-workflows with their effects
pub fn annotate_effects(workflow: Workflow) -> EffectAnnotated<Workflow> {
    let effect = infer_effect(&workflow);
    
    // Recursively annotate children
    let annotated = match workflow {
        Workflow::Seq { first, second } => {
            Workflow::Seq {
                first: Box::new(annotate_effects(*first).value),
                second: Box::new(annotate_effects(*second).value),
            }
        }
        Workflow::Par { workflows } => {
            Workflow::Par {
                workflows: workflows.into_iter()
                    .map(|w| annotate_effects(w).value)
                    .collect(),
            }
        }
        // ... other recursive cases
        other => other,
    };
    
    EffectAnnotated {
        value: annotated,
        effect,
    }
}
```

### Effect Polymorphism

```rust
/// Effect polymorphism: track effect variables
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectVar(pub u32);

/// Effect with possible polymorphism
#[derive(Debug, Clone, PartialEq)]
pub enum PolyEffect {
    Concrete(Effect),
    Var(EffectVar),
    Join(Box<PolyEffect>, Box<PolyEffect>),
}

impl PolyEffect {
    pub fn resolve(&self, subst: &EffectSubst) -> Effect {
        match self {
            PolyEffect::Concrete(e) => *e,
            PolyEffect::Var(v) => subst.get(*v).cloned().unwrap_or(Effect::Epistemic),
            PolyEffect::Join(e1, e2) => {
                let r1 = e1.resolve(subst);
                let r2 = e2.resolve(subst);
                r1.join(r2)
            }
        }
    }
}

/// Effect substitution
pub struct EffectSubst {
    mappings: HashMap<EffectVar, Effect>,
}
```

### Effect Safety Checks

```rust
/// Check that every operational action is preceded by a decision
pub fn check_effect_safety(workflow: &Workflow) -> Result<(), EffectSafetyError> {
    check_effect_safety_inner(workflow, false)
}

fn check_effect_safety_inner(
    workflow: &Workflow,
    has_decision: bool,
) -> Result<(), EffectSafetyError> {
    match workflow {
        Workflow::Act { .. } if !has_decision => {
            Err(EffectSafetyError::UnauthorizedAction)
        }
        
        Workflow::Decide { continuation, .. } => {
            // After a decide, subsequent actions are authorized
            check_effect_safety_inner(continuation, true)
        }
        
        Workflow::Seq { first, second } => {
            // Check first, then propagate decision to second
            let decision_after_first = has_decision || has_decide(first);
            check_effect_safety_inner(first, has_decision)?;
            check_effect_safety_inner(second, decision_after_first)
        }
        
        Workflow::If { then_branch, else_branch, .. } => {
            check_effect_safety_inner(then_branch, has_decision)?;
            if let Some(else_) = else_branch {
                check_effect_safety_inner(else_, has_decision)?;
            }
            Ok(())
        }
        
        Workflow::Par { workflows } => {
            for wf in workflows {
                check_effect_safety_inner(wf, has_decision)?;
            }
            Ok(())
        }
        
        _ => Ok(()),
    }
}

fn has_decide(workflow: &Workflow) -> bool {
    match workflow {
        Workflow::Decide { .. } => true,
        Workflow::Seq { first, second } => has_decide(first) || has_decide(second),
        Workflow::If { then_branch, else_branch, .. } => {
            has_decide(then_branch) || else_branch.as_ref().map_or(false, has_decide)
        }
        Workflow::Par { workflows } => workflows.iter().any(has_decide),
        _ => false,
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum EffectSafetyError {
    #[error("Operational action without preceding decision")]
    UnauthorizedAction,
}
```

## TDD Steps

### Step 1: Implement infer_effect

Create `crates/ash-typeck/src/effect.rs` with effect inference.

### Step 2: Implement Effect Checking

Add check_effect_bound and EffectError.

### Step 3: Implement Effect Annotation

Add EffectAnnotated for tracking effects.

### Step 4: Implement Effect Safety

Add check_effect_safety for policy enforcement.

### Step 5: Write Property Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_observe_effect() {
        let wf = Workflow::Observe {
            capability: Capability { name: "read".into() },
            pattern: Pattern::Wildcard,
            continuation: Box::new(Workflow::Done),
        };
        
        let effect = infer_effect(&wf);
        assert_eq!(effect, Effect::Epistemic);
    }

    #[test]
    fn test_act_effect() {
        let wf = Workflow::Act {
            action: Action { name: "write".into(), args: vec![] },
            guard: None,
            provenance: Provenance::default(),
        };
        
        let effect = infer_effect(&wf);
        assert_eq!(effect, Effect::Operational);
    }

    #[test]
    fn test_seq_effect_joins() {
        let wf = Workflow::Seq {
            first: Box::new(Workflow::Observe {
                capability: Capability { name: "read".into() },
                pattern: Pattern::Wildcard,
                continuation: Box::new(Workflow::Done),
            }),
            second: Box::new(Workflow::Act {
                action: Action { name: "write".into(), args: vec![] },
                guard: None,
                provenance: Provenance::default(),
            }),
        };
        
        let effect = infer_effect(&wf);
        assert_eq!(effect, Effect::Operational);
    }

    #[test]
    fn test_effect_safety_passes_with_decide() {
        let wf = Workflow::Seq {
            first: Box::new(Workflow::Decide {
                expr: Expr::Literal(Value::Bool(true)),
                policy: "policy".into(),
                continuation: Box::new(Workflow::Act {
                    action: Action { name: "write".into(), args: vec![] },
                    guard: None,
                    provenance: Provenance::default(),
                }),
            }),
            second: Box::new(Workflow::Done),
        };
        
        assert!(check_effect_safety(&wf).is_ok());
    }

    #[test]
    fn test_effect_safety_fails_without_decide() {
        let wf = Workflow::Act {
            action: Action { name: "write".into(), args: vec![] },
            guard: None,
            provenance: Provenance::default(),
        };
        
        assert!(check_effect_safety(&wf).is_err());
    }

    proptest! {
        #[test]
        fn prop_effect_monotonicity(wf in arb_workflow()) {
            // Sub-workflows should have effect <= overall effect
            let overall = infer_effect(&wf);
            
            for sub in wf.sub_workflows() {
                let sub_eff = infer_effect(&sub);
                prop_assert!(sub_eff <= overall);
            }
        }
    }
}
```

## Completion Checklist

- [ ] infer_effect for all workflow types
- [ ] Effect bound checking
- [ ] Effect annotation
- [ ] Effect polymorphism support
- [ ] Effect safety checks
- [ ] Property tests for effect lattice properties
- [ ] Unit tests for each workflow construct
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Completeness**: Is effect inferred for all workflow types?
2. **Safety**: Are unauthorized operations detected?
3. **Properties**: Does effect monotonicity hold?

## Estimated Effort

6 hours

## Dependencies

- TASK-001: Effect lattice (uses Effect, join)
- TASK-003: Workflow AST (operates on these types)

## Blocked By

- TASK-001: Effect lattice
- TASK-003: Workflow AST

## Blocks

- TASK-024: Proof obligations (effect safety)
- TASK-025: Type errors (effect errors)
