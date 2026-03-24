# SPEC-022: Workflow Typing with Constraints

## Status: Active

## 1. Overview

Enable **Hoare-style contracts** (`requires`/`ensures`) and **obligation tracking** in the Ash type system. Obligations are local logical markers used for gating decisions within a workflow, with audit trail support.

**Target:** Release 0.5.0  
**Effort Estimate:** 2-3 months

---

## 2. Design Principles

1. **Obligations are local**: Created and checked within a workflow; no automatic propagation to parent/child
2. **Check is an expression**: Returns success/failure; workflow decides next steps
3. **No prescribed patterns**: Retry, escalate, compensate - workflow chooses
4. **Audit at check points**: Every check creates an audit entry for compliance
5. **Type system ensures discharge**: All obligations must be checked before workflow completes

---

## 3. Syntax

### 3.1 Workflow Contracts

```ash
workflow name(params)
    requires: predicate₁, predicate₂, ...  -- pre-conditions (optional)
    ensures: predicate₁, predicate₂, ...   -- post-conditions (optional)
{
    -- body
}
```

### 3.2 Obligation Constructs

```ash
oblige obligation_name;        -- Introduce an obligation
let status = check obligation_name;  -- Check obligation, returns Bool
```

### 3.3 Example Usage Patterns

```ash
-- Pattern: Check and continue
workflow simple {
    oblige audit_trail;
    act process;
    let _ = check audit_trail;  -- Must check before end
}

-- Pattern: Check with retry
workflow with_retry {
    oblige complete_within_deadline;
    
    loop {
        let done = attempt_work();
        if done {
            let met = check complete_within_deadline;
            if met {
                ret Success;
            } else {
                -- Deadline passed, escalate
                act escalate;
                ret Failed;
            }
        }
        -- Continue looping
    }
}

-- Pattern: Check and compensate
workflow with_compensation {
    oblige transaction_atomic;
    
    act reserve_funds;
    let reserved = check transaction_atomic;
    
    if !reserved {
        act release_funds;  -- Compensate
        ret Failed;
    }
    
    act transfer;
}
```

---

## 4. Type Checking Rules

### 4.1 Obligation Scoping

Obligations are **linear resources** - must be checked exactly once:

```
(OBLIGE)
  ─────────────────────────────────
  Γ ⊢ oblige o : () ▷ Γ, o:Obligation

(CHECK)
  o ∈ Γ
  ─────────────────────────────────
  Γ ⊢ check o : Bool ▷ Γ - {o}
```

### 4.2 Workflow End Check

At workflow completion, obligation set must be empty:

```
(Workflow End)
  Γ ⊢ body : τ
  obligations(Γ) = ∅
  ─────────────────────────────────
  Γ ⊢ workflow : τ
```

If obligations remain: type error (undischarged obligations).

### 4.3 Branching

Obligations must be discharged on **all paths**:

```ash
workflow branches(x: Bool) {
    oblige o;
    if x {
        let _ = check o;  -- discharged here
    }
    -- ERROR: obligation not discharged in else branch
}
```

### 4.4 Sequential Composition

```
Γ ⊢ w₁ : τ₁ ▷ Γ₁
Γ₁ ⊢ w₂ : τ₂ ▷ Γ₂
─────────────────────────────────
Γ ⊢ w₁; w₂ : τ₂ ▷ Γ₂
```

### 4.5 Parallel Composition

Parallel branches each get a **copy** of the obligation set:

```
Γ ⊢ w₁ : τ₁ ▷ Γ₁
Γ ⊢ w₂ : τ₂ ▷ Γ₂
─────────────────────────────────
Γ ⊢ par { w₁; w₂ } : (τ₁, τ₂) ▷ Γ₁ ∩ Γ₂
```

If both branches discharge the same obligation, intersection is empty (good). If one branch discharges and other doesn't, intersection still has obligation (error).

### 4.6 Spawn (Fire-and-Forget)

Spawn creates **isolated** obligation scope:

```
Γ ⊢ w : τ ▷ Γ'
─────────────────────────────────
Γ ⊢ spawn w : Handle<τ> ▷ Γ
```

Child's obligations do not propagate to parent. Parent's obligations do not propagate to child. They are completely separate.

---

## 5. Requirement Checking

Requirements are checked at **call sites**:

```
Γ ⊢ args : types
Γ ⊢ callee_requires : provable
─────────────────────────────────
Γ ⊢ callee(args) : τ
```

Requirements currently supported:

```rust
pub enum Requirement {
    HasCapability(Capability, Effect),
    HasRole(Role),
    Arithmetic { var: String, constraint: ArithConstraint },
}
```

---

## 6. Audit Trail

Every `check` creates an audit entry:

```rust
pub struct AuditEvent {
    pub obligation: String,
    pub workflow_id: WorkflowId,
    pub timestamp: Instant,
    pub result: CheckResult,
    pub context: Value,  -- Workflow state snapshot
}

pub enum CheckResult {
    Satisfied,
    Violated { reason: String },
}
```

---

## 7. Error Messages

### 7.1 Undischarged Obligation

```
error[E0421]: obligation not discharged
  --> example.ash:23:1
   |
10 |     oblige respond_by_deadline;
   |     -------------------------- obligation created here
...
23 | }
   | ^ obligation "respond_by_deadline" not checked
   |
   = help: add `check respond_by_deadline;` or ensure it's checked on all paths
```

### 7.2 Double Check

```
error[E0422]: obligation already discharged
  --> example.ash:15:5
   |
12 |     let _ = check respond_by_deadline;
   |             ------------------------- first check here
15 |     let _ = check respond_by_deadline;
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^ obligation already consumed
```

### 7.3 Unsatisfied Requirement

```
error[E0420]: requirement not satisfied
  --> example.ash:15:5
   |
15 |     withdraw(account, 100);
   |     ^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: requires: account.balance >= amount
   = note: available facts: account.balance = 50
```

---

## 8. Test Strategy

### 8.1 Property Tests

```rust
proptest! {
    #[test]
    fn oblige_adds_to_context(o in obligation_strategy()) {
        let ctx = TypeContext::new();
        let result = check_oblige(&ctx, &o);
        assert!(result.obligations.contains(&o));
    }
    
    #[test]
    fn check_removes_from_context(o in obligation_strategy()) {
        let ctx = TypeContext::with_obligation(o.clone());
        let result = check_check(&ctx, &o);
        assert!(!result.obligations.contains(&o));
        assert_eq!(result.ty, Type::Bool);
    }
    
    #[test]
    fn all_paths_must_discharge(o in obligation_strategy()) {
        // if-then-else where only one branch checks
        let workflow = Workflow::If {
            cond: Expr::bool(true),
            then_branch: Box::new(Workflow::Check(o.clone())),
            else_branch: Box::new(Workflow::Done),
        };
        let result = type_check(&workflow);
        assert!(result.has_error(ErrorKind::UndischargedObligation));
    }
    
    #[test]
    fn spawn_isolates_obligations(o in obligation_strategy()) {
        let inner = Workflow::Oblige(o.clone());
        let spawn = Workflow::Spawn(Box::new(inner));
        let result = type_check(&spawn);
        // Parent should have no obligations after spawn
        assert!(result.obligations.is_empty());
    }
}
```

### 8.2 Example Programs

```ash
-- Test 1: Simple obligation lifecycle
workflow simple_obligation {
    oblige audit_required;
    act process;
    let _ = check audit_required;
}

-- Test 2: Check result used for control flow
workflow check_with_decision {
    oblige timely_response;
    
    let met = check timely_response;
    if met {
        ret Success;
    } else {
        act escalate;
        ret Failed;
    }
}

-- Test 3: Parallel - both branches must discharge
workflow parallel_obligations {
    oblige o1;
    oblige o2;
    
    par {
        { act work1; let _ = check o1; }
        { act work2; let _ = check o2; }
    }
}

-- Test 4: Error - obligation not discharged
workflow bad {
    oblige forgot_this;
    act work;
    -- ERROR: forgot_this not checked
}

-- Test 5: Error - double check
workflow also_bad {
    oblige double;
    let _ = check double;
    let _ = check double;  -- ERROR: already consumed
}
```

---

## 9. Cross-References

| Spec | Relationship | Description |
|------|--------------|-------------|
| [SPEC-001](SPEC-001-IR.md) | Uses | Core IR forms: `Check`, `Oblig` workflow nodes |
| [SPEC-003](SPEC-003-TYPE-SYSTEM.md) | Extends | Type judgment with obligation tracking (Ω ⊢) |
| [SPEC-004](SPEC-004-SEMANTICS.md) | Defines | Operational semantics for `check` evaluation |
| [SPEC-020](SPEC-020-ADT-TYPES.md) | Uses | `Result` type for check returns |

---

## 10. Dependencies

- **SPEC-020**: ADT Types - For `Type` representation
- **TASK-129**: Generic Instantiation - For contract polymorphism
- **Existing SMT**: For arithmetic requirement checking

---

## 11. Implementation Tasks

| Task | Description | Status |
|------|-------------|--------|
| TASK-226 | Workflow Contracts AST Extensions | Complete |
| TASK-227 | Type Check Obligations | Complete |
| TASK-228 | Requirement Checking | Complete |
| TASK-229 | Audit Trail Integration | Complete |
| TASK-230 | Parser Updates for Contracts | Complete |
| TASK-231 | Integration Tests | Complete |
| TASK-232 | Canonicalize SPEC-022 | Complete |

---

## 12. Migration Guide

### 12.1 Existing Code

```ash
-- Before: No contracts, no obligations
workflow process(x: Int) {
    act log;
}
```

### 12.2 With Obligations

```ash
-- After: Optional obligation tracking
workflow process(x: Int) {
    oblige audit_trail;
    act log;
    let _ = check audit_trail;
}
```

### 12.3 Gradual Adoption

1. **Phase 1:** Add obligations to critical workflows (audit points)
2. **Phase 2:** Add requirements to public APIs
3. **Phase 3:** Enable strict mode (all public workflows need contracts)

### 12.4 Audit Log Format

File-based JSON Lines format:

```jsonl
{"obligation":"audit_required","workflow_id":"wf_abc123","timestamp":"2024-01-15T10:30:00Z","result":"Satisfied","context":{"user":"alice","amount":100}}
{"obligation":"respond_by_deadline","workflow_id":"wf_def456","timestamp":"2024-01-15T10:31:00Z","result":"Violated","context":{"reason":"timeout"}}
```

---

## 13. Implementation Decisions

### 13.1 Panic Paths

Warn (don't error) when obligations might not be discharged on panic paths:

```
warning: obligation may not be discharged on panic path
  --> example.ash:12:5
   |
10 |     oblige audit_required;
   |     ---------------------- obligation created here
12 |     act risky_operation;  -- might panic
   |     ^^^^^^^^^^^^^^^^^^^^
   = note: if panic occurs, obligation will be lost
   = help: consider checking before risky operation, or handling in supervisor
```

### 13.2 Audit Backend

File-based audit log for now:

```rust
pub struct FileAuditBackend {
    path: PathBuf,
    file: std::fs::File,
}

impl AuditBackend for FileAuditBackend {
    fn record(&mut self, event: AuditEvent) {
        let line = serde_json::to_string(&event).unwrap();
        writeln!(self.file, "{}", line).unwrap();
    }
}
```

Pluggable backend trait for future database/external service integration.

### 13.3 Phase 1 Requirements

- `HasCapability(Capability, Effect)` - Static check
- `HasRole(Role)` - Static check  
- Arithmetic (`x > 0`, `x < 100`) - SMT check

Time bounds and state predicates deferred.

---

## 14. Future Extensions (Out of Scope)

- **Obligation inheritance across spawn/join** - Needs join semantics first
- **Time-bound obligations** - Needs runtime deadline tracking
- **Crypto proofs** - Distributed verification use case
- **Pattern sugar** - Let patterns emerge before standardizing
- **Database audit backend** - File-based sufficient for now

---

## 15. Changelog

| Date | Version | Change |
|------|---------|--------|
| 2024-01-15 | 0.1.0 | Initial draft from todo-examples |
| 2024-01-15 | 1.0.0 | Canonicalized as SPEC-022; added cross-references, implementation tasks, and changelog |

---

*End of SPEC-022: Workflow Typing with Constraints*
SPECEOF; __hermes_rc=$?; printf '__HERMES_FENCE_a9f7b3__'; exit $__hermes_rc
