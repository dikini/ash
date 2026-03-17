# TASK-024: Proof Obligation Generation

## Status: 🔴 Not Started

## Description

Implement proof obligation generation that emits conditions that must be verified for workflow correctness, safety, and policy compliance.

## Specification Reference

- SPEC-003: Type System - Section 6. Proof Obligations
- SHARO_CORE_LANGUAGE.md - Section 8. Proof Obligations

## Requirements

### Proof Obligation Types

```rust
/// Proof obligations generated during type checking
#[derive(Debug, Clone, PartialEq)]
pub enum ProofObligation {
    /// Effect Safety: Every ACT must be preceded by DECIDE
    EffectSafety {
        action: ActionRef,
        location: Span,
        required_decision: Option<Box<str>>,
    },
    
    /// Obligation Fulfillment: All obligations must be discharged
    ObligationFulfillment {
        obligation: Obligation,
        location: Span,
        discharge_point: Option<Span>,
    },
    
    /// Role Separation of Duties: Conflicting roles must be separate
    RoleSeparation {
        role1: Box<str>,
        role2: Box<str>,
        reason: SoDReason,
        location: Span,
    },
    
    /// Guard Decidability: Guards must be decidable
    GuardDecidable {
        guard: Guard,
        location: Span,
    },
    
    /// Capability Containment: Used capabilities must be declared
    CapabilityContainment {
        capability: Box<str>,
        location: Span,
    },
    
    /// Policy Consistency: Policies must not contradict
    PolicyConsistency {
        policy1: Box<str>,
        policy2: Box<str>,
        conflict: ConflictType,
        location: Span,
    },
    
    /// Progress: Workflow must be able to complete
    Progress {
        workflow: WorkflowId,
        blocking_point: Span,
    },
    
    /// Resource Bound: Resource usage must be within bounds
    ResourceBound {
        resource: ResourceType,
        bound: Bound,
        usage: Usage,
        location: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SoDReason {
    /// Same person cannot draft and approve
    DraftApprove,
    /// Same person cannot request and authorize
    RequestAuthorize,
    /// Custom separation reason
    Custom(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    ContradictoryConditions,
    OverlappingPermissions,
    UnreachablePolicy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResourceType {
    Time,
    Memory,
    Calls,
    Budget,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Bound {
    Finite(usize),
    Unlimited,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Usage {
    Known(usize),
    Unknown,
    DependsOn(Box<str>), // Depends on input
}
```

### Proof Obligation Generator

```rust
/// Generates proof obligations for a program
#[derive(Debug, Default)]
pub struct ObligationGenerator {
    pub obligations: Vec<ProofObligation>,
    pub warnings: Vec<ObligationWarning>,
}

impl ObligationGenerator {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Generate all proof obligations for a program
    pub fn generate(&mut self, program: &Program) -> Vec<ProofObligation> {
        // Check effect safety
        self.check_effect_safety(&program.workflow);
        
        // Check obligation fulfillment
        self.check_obligation_fulfillment(&program.workflow);
        
        // Check role separation
        for def in &program.definitions {
            if let Definition::Workflow(wf) = def {
                self.check_role_separation(wf);
            }
        }
        
        // Check guard decidability
        self.check_guard_decidability(&program.workflow);
        
        // Check capability containment
        self.check_capability_containment(program);
        
        // Check progress
        self.check_progress(&program.workflow);
        
        self.obligations.clone()
    }
    
    /// Check effect safety: all ACTs must be preceded by DECIDE
    fn check_effect_safety(&mut self, workflow: &Workflow) {
        self.check_effect_safety_inner(workflow, false);
    }
    
    fn check_effect_safety_inner(&mut self, workflow: &Workflow, has_decision: bool) {
        match workflow {
            Workflow::Act { action, .. } => {
                if !has_decision {
                    self.obligations.push(ProofObligation::EffectSafety {
                        action: action.clone(),
                        location: Span::default(),
                        required_decision: None,
                    });
                }
            }
            
            Workflow::Decide { policy, continuation, .. } => {
                self.check_effect_safety_inner(continuation, true);
            }
            
            Workflow::Seq { first, second } => {
                let has_decide_after_first = has_decision || self.has_decide(first);
                self.check_effect_safety_inner(first, has_decision);
                self.check_effect_safety_inner(second, has_decide_after_first);
            }
            
            Workflow::If { then_branch, else_branch, .. } => {
                self.check_effect_safety_inner(then_branch, has_decision);
                if let Some(else_) = else_branch {
                    self.check_effect_safety_inner(else_, has_decision);
                }
            }
            
            Workflow::Par { workflows } => {
                for wf in workflows {
                    self.check_effect_safety_inner(wf, has_decision);
                }
            }
            
            Workflow::With { body, .. } => {
                self.check_effect_safety_inner(body, has_decision);
            }
            
            _ => {}
        }
    }
    
    fn has_decide(&self, workflow: &Workflow) -> bool {
        match workflow {
            Workflow::Decide { .. } => true,
            Workflow::Seq { first, second } => {
                self.has_decide(first) || self.has_decide(second)
            }
            Workflow::If { then_branch, else_branch, .. } => {
                self.has_decide(then_branch) || 
                else_branch.as_ref().map_or(false, |e| self.has_decide(e))
            }
            _ => false,
        }
    }
    
    /// Check that all obligations are fulfilled
    fn check_obligation_fulfillment(&mut self, workflow: &Workflow) {
        // Collect incurred obligations
        let mut incurred: Vec<(Obligation, Span)> = Vec::new();
        self.collect_obligations(workflow, &mut incurred);
        
        // Collect discharge points
        let mut discharged: Vec<Obligation> = Vec::new();
        self.collect_discharges(workflow, &mut discharged);
        
        // Find unfulfilled obligations
        for (obl, span) in incurred {
            if !discharged.contains(&obl) {
                self.obligations.push(ProofObligation::ObligationFulfillment {
                    obligation: obl,
                    location: span,
                    discharge_point: None,
                });
            }
        }
    }
    
    fn collect_obligations(&self, workflow: &Workflow, acc: &mut Vec<(Obligation, Span)>) {
        match workflow {
            Workflow::Oblig { role, workflow } => {
                let obl = Obligation::Obliged {
                    role: role.name.clone(),
                    condition: Condition::All(vec![]),
                };
                acc.push((obl, Span::default()));
                self.collect_obligations(workflow, acc);
            }
            Workflow::Seq { first, second } => {
                self.collect_obligations(first, acc);
                self.collect_obligations(second, acc);
            }
            _ => {}
        }
    }
    
    fn collect_discharges(&self, workflow: &Workflow, acc: &mut Vec<Obligation>) {
        match workflow {
            Workflow::Check { obligation, continuation } => {
                if let Some(obl) = obligation_from_ref(obligation) {
                    acc.push(obl);
                }
                if let Some(cont) = continuation {
                    self.collect_discharges(cont, acc);
                }
            }
            Workflow::Seq { first, second } => {
                self.collect_discharges(first, acc);
                self.collect_discharges(second, acc);
            }
            _ => {}
        }
    }
    
    /// Check role separation of duties
    fn check_role_separation(&mut self, workflow: &Workflow) {
        // Find all role references in the workflow
        let mut roles_by_activity: HashMap<ActivityType, Vec<Box<str>>> = HashMap::new();
        
        self.collect_roles(workflow, &mut roles_by_activity);
        
        // Check for conflicts
        if let Some(drafters) = roles_by_activity.get(&ActivityType::Draft) {
            if let Some(approvers) = roles_by_activity.get(&ActivityType::Approve) {
                for drafter in drafters {
                    if approvers.contains(drafter) {
                        self.obligations.push(ProofObligation::RoleSeparation {
                            role1: drafter.clone(),
                            role2: drafter.clone(),
                            reason: SoDReason::DraftApprove,
                            location: Span::default(),
                        });
                    }
                }
            }
        }
    }
    
    fn collect_roles(&self, workflow: &Workflow, acc: &mut HashMap<ActivityType, Vec<Box<str>>>) {
        // Simplified - would analyze workflow for role usage patterns
    }
    
    /// Check that guards are decidable
    fn check_guard_decidability(&mut self, workflow: &Workflow) {
        match workflow {
            Workflow::Act { guard: Some(guard), .. } => {
                if !is_decidable(guard) {
                    self.obligations.push(ProofObligation::GuardDecidable {
                        guard: guard.clone(),
                        location: Span::default(),
                    });
                }
            }
            Workflow::Decide { expr, .. } => {
                // Check that decision expression terminates
            }
            Workflow::Seq { first, second } => {
                self.check_guard_decidability(first);
                self.check_guard_decidability(second);
            }
            _ => {}
        }
    }
    
    /// Check that all used capabilities are declared
    fn check_capability_containment(&mut self, program: &Program) {
        // Collect declared capabilities
        let declared: HashSet<_> = program.definitions.iter()
            .filter_map(|d| match d {
                Definition::Capability(c) => Some(c.name.clone()),
                _ => None,
            })
            .collect();
        
        // Collect used capabilities
        let mut used = HashSet::new();
        self.collect_capabilities(&program.workflow, &mut used);
        
        // Find undeclared capabilities
        for cap in used {
            if !declared.contains(&cap) {
                self.obligations.push(ProofObligation::CapabilityContainment {
                    capability: cap,
                    location: Span::default(),
                });
            }
        }
    }
    
    fn collect_capabilities(&self, workflow: &Workflow, acc: &mut HashSet<Box<str>>) {
        match workflow {
            Workflow::Observe { capability, continuation } => {
                acc.insert(capability.name.clone());
                self.collect_capabilities(continuation, acc);
            }
            Workflow::Act { action, .. } => {
                acc.insert(action.name.clone());
            }
            Workflow::Seq { first, second } => {
                self.collect_capabilities(first, acc);
                self.collect_capabilities(second, acc);
            }
            _ => {}
        }
    }
    
    /// Check that workflow can make progress
    fn check_progress(&mut self, workflow: &Workflow) {
        // Check for infinite loops without progress conditions
        // Check for deadlocks in parallel composition
        // Check for unreachable code
    }
}

fn is_decidable(guard: &Guard) -> bool {
    // Simplified - would check for termination
    true
}

fn obligation_from_ref(obl_ref: &ObligationRef) -> Option<Obligation> {
    None // Simplified
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ActivityType {
    Draft,
    Approve,
    Review,
    Execute,
}

#[derive(Debug, Clone)]
struct ObligationWarning {
    // Warning information
}
```

### Obligation Verification

```rust
/// Verify that all proof obligations are satisfied
pub fn verify_obligations(obligations: &[ProofObligation]) -> Result<(), Vec<ProofObligation>> {
    let unsatisfied: Vec<_> = obligations.iter()
        .filter(|o| !is_satisfied(o))
        .cloned()
        .collect();
    
    if unsatisfied.is_empty() {
        Ok(())
    } else {
        Err(unsatisfied)
    }
}

fn is_satisfied(obligation: &ProofObligation) -> bool {
    match obligation {
        // Some obligations can be automatically verified
        ProofObligation::CapabilityContainment { capability, .. } => {
            // Check if capability exists in scope
            false // Simplified
        }
        _ => false, // Most obligations require external verification
    }
}
```

## TDD Steps

### Step 1: Define Proof Obligation Types

Create `crates/ash-typeck/src/proofs.rs` with ProofObligation enum.

### Step 2: Implement ObligationGenerator

Add generator for all obligation types.

### Step 3: Implement Effect Safety Check

Add check_effect_safety with proper traversal.

### Step 4: Implement Other Checks

Add obligation fulfillment, role separation, etc.

### Step 5: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_safety_detects_unauthorized_act() {
        let wf = Workflow::Act {
            action: ActionRef { name: "delete".into(), args: vec![] },
            guard: None,
            provenance: Provenance::default(),
        };
        
        let mut gen = ObligationGenerator::new();
        gen.check_effect_safety(&wf);
        
        assert!(gen.obligations.iter().any(|o| matches!(o,
            ProofObligation::EffectSafety { .. }
        )));
    }

    #[test]
    fn test_capability_containment_detects_undeclared() {
        let program = Program {
            definitions: vec![],
            workflow: Workflow::Act {
                action: ActionRef { name: "undeclared".into(), args: vec![] },
                guard: None,
                provenance: Provenance::default(),
            },
        };
        
        let mut gen = ObligationGenerator::new();
        gen.check_capability_containment(&program);
        
        assert!(gen.obligations.iter().any(|o| matches!(o,
            ProofObligation::CapabilityContainment { cap, .. } if cap == "undeclared"
        )));
    }
}
```

## Completion Checklist

- [ ] ProofObligation enum with all variants
- [ ] ObligationGenerator struct
- [ ] Effect safety checking
- [ ] Obligation fulfillment checking
- [ ] Role separation checking
- [ ] Guard decidability checking
- [ ] Capability containment checking
- [ ] Progress checking
- [ ] Obligation verification
- [ ] Unit tests for each obligation type
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Completeness**: Are all proof obligations from SPEC-003 covered?
2. **Accuracy**: Do checks correctly identify violations?
3. **Usability**: Are generated obligations actionable?

## Estimated Effort

6 hours

## Dependencies

- ash-core: Core types
- TASK-021: Effect inference (uses effect analysis)
- TASK-023: Obligation tracking (uses obligation types)

## Blocked By

- ash-core: Core types

## Blocks

- TASK-024b: SMT integration (uses proof obligations)
- TASK-025: Type errors (obligation errors)
