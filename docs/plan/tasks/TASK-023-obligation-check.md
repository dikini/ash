# TASK-023: Obligation Tracking

## Status: 🟢 Complete

## Description

Implement obligation tracking that monitors deontic constraints (obligations, permissions, prohibitions) throughout workflow execution paths.

## Specification Reference

- SPEC-003: Type System - Section 4.6 Deontic Constraints
- SHARO_CORE_LANGUAGE.md - Section 6. Deontic Logic Layer

## Requirements

### Obligation Types

```rust
/// Deontic obligations in the type system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Obligation {
    /// Role is obliged to ensure condition holds
    Obliged { role: Box<str>, condition: Condition },
    
    /// Role is permitted to perform action
    Permitted { role: Box<str>, action: ActionRef },
    
    /// Role is prohibited from performing action
    Prohibited { role: Box<str>, action: ActionRef },
    
    /// Permission delegated from one role to another
    Delegated { from: Box<str>, to: Box<str>, action: ActionRef },
}

/// Conditions that can be obligations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Condition {
    /// Boolean expression condition
    Expr(Expr),
    
    /// Check predicate
    Check { predicate: Box<str>, args: Vec<Expr> },
    
    /// Verify condition
    Verify { condition: Box<str> },
    
    /// Ensure condition
    Ensure { condition: Box<str> },
    
    /// Compound: all conditions must hold
    All(Vec<Condition>),
    
    /// Compound: any condition may hold
    Any(Vec<Condition>),
}

/// Obligation status tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObligationStatus {
    /// Obligation is pending (not yet checked)
    Pending,
    /// Obligation has been satisfied
    Satisfied,
    /// Obligation has been violated
    Violated,
    /// Obligation was waived
    Waived,
}
```

### Obligation Context

```rust
/// Context for tracking obligations during type checking
#[derive(Debug, Clone, Default)]
pub struct ObligationContext {
    /// Active obligations that must be satisfied
    pub active: Vec<Obligation>,
    
    /// Discharged obligations (satisfied)
    pub discharged: Vec<(Obligation, Span)>,
    
    /// Violated obligations
    pub violated: Vec<(Obligation, Span)>,
    
    /// Role definitions
    pub roles: HashMap<Box<str>, RoleDef>,
}

impl ObligationContext {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add an obligation to the active set
    pub fn incur(&mut self, obligation: Obligation) {
        self.active.push(obligation);
    }
    
    /// Discharge an obligation
    pub fn discharge(&mut self, obligation: &Obligation, span: Span) -> Result<(), ObligationError> {
        if let Some(idx) = self.active.iter().position(|o| o == obligation) {
            let obl = self.active.remove(idx);
            self.discharged.push((obl, span));
            Ok(())
        } else {
            Err(ObligationError::NoSuchObligation {
                obligation: format!("{:?}", obligation),
            })
        }
    }
    
    /// Check if an obligation is active
    pub fn has_obligation(&self, obligation: &Obligation) -> bool {
        self.active.contains(obligation)
    }
    
    /// Get obligations for a specific role
    pub fn obligations_for(&self, role: &str) -> Vec<&Obligation> {
        self.active.iter()
            .filter(|o| match o {
                Obligation::Obliged { role: r, .. } => r == role,
                Obligation::Permitted { role: r, .. } => r == role,
                Obligation::Prohibited { role: r, .. } => r == role,
                _ => false,
            })
            .collect()
    }
    
    /// Check for prohibition violations
    pub fn check_prohibition(&mut self, role: &str, action: &ActionRef) -> Result<(), ObligationError> {
        let prohibition = Obligation::Prohibited {
            role: role.into(),
            action: action.clone(),
        };
        
        if self.active.contains(&prohibition) {
            Err(ObligationError::ProhibitionViolated {
                role: role.to_string(),
                action: action.name.to_string(),
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ObligationError {
    #[error("Obligation not found: {obligation}")]
    NoSuchObligation { obligation: String },
    
    #[error("Prohibition violated: {role} performed prohibited action {action}")]
    ProhibitionViolated { role: String, action: String },
    
    #[error("Unfulfilled obligations remain: {count}")]
    UnfulfilledObligations { count: usize },
}
```

### Workflow Obligation Analysis

```rust
/// Analyze obligations in a workflow
pub fn analyze_obligations(
    ctx: &mut ObligationContext,
    workflow: &Workflow,
) -> Result<Effect, Vec<ObligationError>> {
    match workflow {
        Workflow::Check { obligation, continuation } => {
            // Check discharge
            if let Some(obl) = obligation_from_ref(obligation) {
                ctx.discharge(&obl, Span::default())?;
            }
            
            // Continue with rest
            if let Some(cont) = continuation {
                analyze_obligations(ctx, cont)
            } else {
                Ok(Effect::Epistemic)
            }
        }
        
        Workflow::Oblig { role, workflow } => {
            // Incurring an obligation adds evaluative effect
            let obl = Obligation::Obliged {
                role: role.name.clone(),
                condition: Condition::All(vec![]), // Simplified
            };
            ctx.incur(obl);
            
            let eff = analyze_obligations(ctx, workflow)?;
            Ok(Effect::Evaluative.join(eff))
        }
        
        Workflow::Act { action, .. } => {
            // Check if any active role is prohibited from this action
            for obl in &ctx.active {
                if let Obligation::Prohibited { role, action: prohibited } = obl {
                    if prohibited.name == action.name {
                        ctx.violated.push((obl.clone(), Span::default()));
                        return Err(vec![ObligationError::ProhibitionViolated {
                            role: role.to_string(),
                            action: action.name.to_string(),
                        }]);
                    }
                }
            }
            
            Ok(Effect::Operational)
        }
        
        Workflow::Seq { first, second } => {
            let eff1 = analyze_obligations(ctx, first)?;
            let eff2 = analyze_obligations(ctx, second)?;
            Ok(eff1.join(eff2))
        }
        
        Workflow::Par { workflows } => {
            // For parallel branches, obligations must be satisfied in all branches
            let effects: Vec<_> = workflows.iter()
                .map(|w| analyze_obligations(ctx, w))
                .collect::<Result<Vec<_>, _>>()?;
            
            Ok(effects.into_iter().fold(Effect::Epistemic, Effect::join))
        }
        
        Workflow::If { then_branch, else_branch, .. } => {
            // Check obligations in both branches
            let eff1 = analyze_obligations(ctx, then_branch)?;
            let eff2 = else_branch.as_ref()
                .map(|e| analyze_obligations(ctx, e))
                .transpose()?
                .unwrap_or(Effect::Epistemic);
            
            Ok(eff1.join(eff2))
        }
        
        Workflow::With { .. } => {
            // Capability scoping may add permissions
            Ok(Effect::Epistemic)
        }
        
        _ => Ok(Effect::Epistemic),
    }
}

fn obligation_from_ref(obl_ref: &ObligationRef) -> Option<Obligation> {
    // Convert obligation reference to obligation
    // This would look up in role definitions
    None // Simplified
}
```

### Role Hierarchy

```rust
/// Check role hierarchy for supervision
pub fn is_supervisor(ctx: &ObligationContext, supervisor: &str, subordinate: &str) -> bool {
    if supervisor == subordinate {
        return true;
    }
    
    if let Some(role) = ctx.roles.get(subordinate) {
        role.supervises.iter().any(|s| is_supervisor(ctx, supervisor, s))
    } else {
        false
    }
}

/// Verify that a supervisor can verify a subordinate's obligation
pub fn can_verify_obligation(
    ctx: &ObligationContext,
    verifier: &str,
    obligation: &Obligation,
) -> bool {
    match obligation {
        Obligation::Obliged { role, .. } => is_supervisor(ctx, verifier, role),
        _ => true, // Other obligations don't require supervision
    }
}
```

## TDD Steps

### Step 1: Define Obligation Types

Create `crates/ash-typeck/src/obligations.rs` with Obligation enum.

### Step 2: Implement ObligationContext

Add context for tracking obligations.

### Step 3: Implement Workflow Analysis

Add analyze_obligations for all workflow types.

### Step 4: Implement Role Hierarchy

Add supervision checking.

### Step 5: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incur_obligation() {
        let mut ctx = ObligationContext::new();
        let obl = Obligation::Obliged {
            role: "admin".into(),
            condition: Condition::All(vec![]),
        };
        
        ctx.incur(obl.clone());
        assert!(ctx.has_obligation(&obl));
    }

    #[test]
    fn test_discharge_obligation() {
        let mut ctx = ObligationContext::new();
        let obl = Obligation::Obliged {
            role: "admin".into(),
            condition: Condition::All(vec![]),
        };
        
        ctx.incur(obl.clone());
        ctx.discharge(&obl, Span::default()).unwrap();
        
        assert!(!ctx.has_obligation(&obl));
        assert_eq!(ctx.discharged.len(), 1);
    }

    #[test]
    fn test_prohibition_violation() {
        let mut ctx = ObligationContext::new();
        let obl = Obligation::Prohibited {
            role: "user".into(),
            action: ActionRef { name: "delete".into(), args: vec![] },
        };
        
        ctx.incur(obl);
        
        let result = ctx.check_prohibition("user", &ActionRef { 
            name: "delete".into(), 
            args: vec![] 
        });
        
        assert!(result.is_err());
    }

    #[test]
    fn test_role_hierarchy() {
        let mut ctx = ObligationContext::new();
        ctx.roles.insert("manager".into(), RoleDef {
            name: "manager".into(),
            supervises: vec!["employee".into()],
            ..Default::default()
        });
        ctx.roles.insert("employee".into(), RoleDef {
            name: "employee".into(),
            supervises: vec![],
            ..Default::default()
        });
        
        assert!(is_supervisor(&ctx, "manager", "employee"));
        assert!(!is_supervisor(&ctx, "employee", "manager"));
    }
}
```

## Completion Checklist

- [ ] Obligation enum with all variants
- [ ] Condition types for obligation expressions
- [ ] ObligationContext for tracking
- [ ] Incur/discharge operations
- [ ] Prohibition checking
- [ ] Workflow obligation analysis
- [ ] Role hierarchy support
- [ ] Verification authority checking
- [ ] Unit tests for obligation lifecycle
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Completeness**: Are all deontic operators tracked?
2. **Hierarchy**: Is supervision transitive?
3. **Error detection**: Are violations caught early?

## Estimated Effort

6 hours

## Dependencies

- ash-core: Role, Workflow types
- TASK-001: Effect lattice (uses Effect)

## Blocked By

- ash-core: Core types

## Blocks

- TASK-024: Proof obligations (uses obligation tracking)
- TASK-025: Type errors (obligation errors)
