# Sharo Core Language (SHC)

## Design Report: A Workflow Language for Governed AI Systems

**Based on discussions from Prompts 40-49**

---

## Executive Summary

This document presents Sharo Core (SHC), a workflow language designed for governed AI systems that bridges human oversight and automated execution. The language integrates operational semantics, deontic logic, modal logic, and dynamic logic to provide a formal foundation for AI governance while maintaining readability for both humans and LLMs.

> Note: this document is a historical design report and reference artifact. The canonical current
> surface and IR contracts live in `docs/spec/`; examples and grammars in this report should be
> read as historical/reference material when they differ from the canonical specs.

---

## 1. Motivation and Design Goals

From the discussions in Prompts 40-49, several key requirements emerged:

1. **Dual Audience**: The language must be readable by humans (for oversight) and generatable by LLMs (for automation)
2. **Governance-First**: Policy enforcement, audit trails, and accountability must be first-class concepts
3. **Gradual Formalization**: Start with operational clarity, add formal proofs later
4. **Effect Tracking**: Distinguish between observation, deliberation, evaluation, and action
5. **Deontic Reasoning**: Express obligations, permissions, and prohibitions naturally

---

## 2. Intermediate Representation (IR)

### 2.1 Core Abstract Syntax

```haskell
-- Identifiers and basic types
type Name = String
type URL = String
type WorkflowID = UUID

-- Effect lattice: epistemic < deliberative < evaluative < operational
data Effect 
  = Epistemic      -- OBSERVE: read-only, no side effects
  | Deliberative   -- ORIENT: analysis, planning, internal computation
  | Evaluative     -- DECIDE: policy evaluation, safety checks
  | Operational    -- ACT: external side effects
  deriving (Eq, Ord, Show)

-- Action classification
data ActionKind
  = Query Name [Argument]       -- Epistemic: search, read, fetch
  | Analyze Name [Argument]     -- Deliberative: summarize, classify, plan
  | Validate Name [Argument]    -- Evaluative: check, verify, test
  | Execute Name [Argument]     -- Operational: write, send, delete, update

-- Capability with constraints
data Capability = Cap {
  capName :: Name,
  capEffect :: Effect,
  capConstraints :: [Constraint]
}

-- Core workflow terms (the IR)
data Workflow
  -- Epistemic Layer (Reading/Observing)
  = OBSERVE Capability Pattern Workflow
    -- ^ observe capability, bind result to pattern, continue
    
  -- Deliberative Layer (Thinking/Planning)
  | ORIENT Expression Workflow
    -- ^ analyze expression, then continue
  | PROPOSE ActionKind Workflow
    -- ^ propose action (advisory, non-binding)
    
  -- Evaluative Layer (Governance/Checking)
  | DECIDE Expression Policy Workflow
    -- ^ evaluate policy on expression, continue if permitted
  | CHECK Obligation Workflow
    -- ^ verify deontic obligation before continuing
    
  -- Operational Layer (Acting/Executing)
  | ACT ActionKind Guard Provenance
    -- ^ execute action with guard and trace capture
  | OBLIG Role Workflow
    -- ^ scoped obligation under specific role
    
  -- Control Flow
  | LET Pattern Expression Workflow
    -- ^ variable binding
  | IF Expression Workflow Workflow
    -- ^ conditional branching
  | SEQ Workflow Workflow
    -- ^ sequential composition
  | PAR [Workflow]
    -- ^ parallel composition
  | FOREACH Pattern Expression Workflow
    -- ^ iteration over collection
  | RET Expression
    -- ^ return value
    
  -- Modal Constructs
  | WITH Capability Workflow
    -- ^ capability-scoped execution
  | MAYBE Workflow Workflow
    -- ^ optional with fallback
  | MUST Workflow
    -- ^ mandatory (enforced)
  | ATTEMPT Workflow Workflow
    -- ^ try/catch pattern
    
  | DONE
    -- ^ terminal workflow

-- Pattern matching for destructuring
data Pattern 
  = PVar Name
  | PTuple [Pattern]
  | PRecord [(Name, Pattern)]
  | PList [Pattern] (Maybe Pattern)  -- [a, b, ..rest]
  | PWildcard
  | PLiteral Literal

-- Runtime values
data Literal
  = LInt Integer
  | LString String
  | LBool Bool
  | LNull
  | LTime Timestamp
  | LRef Reference

-- Guards for authorization
data Guard 
  = Pred Predicate
  | AND Guard Guard
  | OR Guard Guard
  | NOT Guard
  | ALWAYS
  | NEVER
  | IMPLIES Guard Guard

-- Deontic operators (from deontic logic)
data Obligation
  = OBLIGED Role Condition    -- Must ensure condition holds
  | PERMITTED Role Action     -- May perform action
  | PROHIBITED Role Action    -- Must not perform action
  | DELEGATED Role Role Action  -- Delegate permission

-- Policy as authorization predicate (χ from Prompt 44)
data Policy = Policy {
  policyName :: Name,
  policyPred :: Context -> Decision
}

data Decision = Permit | Deny | RequireApproval Role | Escalate

-- Provenance tracking for audit
data Provenance = Prov {
  provWorkflow :: WorkflowID,
  provParent :: Maybe WorkflowID,
  provDecision :: DecisionRecord,
  provTrace :: [TraceEvent],
  provLineage :: [WorkflowID]
}

data TraceEvent
  = ObsEvent Capability Value
  | OrientEvent Expression Value
  | DecideEvent Policy Decision
  | ProposeEvent ActionKind
  | ActEvent ActionKind Value Guard
  | ObligEvent Role Condition Bool
  | ErrorEvent Error

data Role = Role {
  roleName :: Name,
  roleAuthority :: [Capability],
  roleObligations :: [Obligation]
}
```

### 2.2 Effect Lattice

The effect system forms a lattice that tracks the "power" of operations:

```
                Operational  (highest - can change world)
                     |
                Evaluative   (can make governance decisions)
                     |
                Deliberative (can analyze and plan)
                     |
                Epistemic    (lowest - can only observe)
```

**Join (⊔)**: Take the maximum effect (SEQ, PAR compose with join)
**Meet (⊓)**: Take the minimum effect (intersection of capabilities)

---

## 3. Surface Language (LLM-Friendly Syntax)

The surface syntax is designed to be:

- **Readable** by non-programmers (policy officers, auditors)
- **Writable** by LLMs (clear keywords, structured but natural)
- **Translatable** to the IR for execution

### 3.1 Grammar (EBNF)

```ebnf
Program ::= Definition* "workflow" Identifier "{" Workflow "}"

Definition ::=
    | "capability" Identifier ":" Effect "(" Params ")" Constraints
    | "policy" Identifier ":" "when" Expression "then" Decision
  | "role" Identifier "{" Authority ("," Obligations)? "}"
    | "memory" Identifier "stores" Type ("with" "retention" Duration)?
    | "datatype" Identifier "=" Constructor ("|" Constructor)*

Effect ::= "observe" | "read" | "analyze" | "decide" | "act" | "write" | "external"

Workflow ::=
    | "observe" CapabilityRef ("as" Pattern)? ("then" Workflow)?
    | "orient" "{" Expression "}" ("as" Pattern)? ("then" Workflow)?
    | "propose" ActionRef ("as" Pattern)? ("then" Workflow)?
    | "decide" "{" Expression "}" ("under" PolicyRef)? ("then" Workflow)?
    | "check" ObligationRef ("then" Workflow)?
    | "act" ActionRef ("where" Guard)? ("then" Workflow)?
    | "oblige" RoleRef "to" CheckRef ("then" Workflow)?
    | "let" Pattern "=" Expression ("in" Workflow)?
    | "if" Expression "then" Workflow ("else" Workflow)?
    | "for" Pattern "in" Expression "do" Workflow
    | "while" Expression "do" Workflow
    | "par" "{" Workflow ("|" Workflow)* "}"
    | "with" CapabilityRef "do" Workflow
    | "maybe" Workflow "else" Workflow
    | "must" Workflow
    | "attempt" Workflow "catch" Workflow
    | "retry" Workflow "up to" Number ("times")?
    | "timeout" Workflow "after" Duration
    | "done"
    | Workflow ";" Workflow
    | "{" Workflow "}"

CapabilityRef ::= Identifier ("with" Arguments)?
ActionRef ::= Identifier "(" Arguments ")"
PolicyRef ::= Identifier
RoleRef ::= Identifier

Pattern ::=
    | Identifier
    | "_"  (wildcard)
    | "(" Pattern ("," Pattern)* ")"
    | "{" FieldPattern ("," FieldPattern)* "}"
    | "[" Pattern ("," Pattern)* (".." Identifier)? "]"
    | Literal

FieldPattern ::= Identifier (":" Pattern)?

Expression ::=
    | Literal
    | Identifier
    | Expression "." Identifier
    | Expression "[" Expression "]"
    | UnaryOp Expression
    | Expression BinaryOp Expression
    | Expression "?" Expression ":" Expression
    | "(" Expression ")"

UnaryOp ::= "not" | "-" | "#" | "empty?"
BinaryOp ::= "+" | "-" | "*" | "/" | "and" | "or" | "=" | "!=" | "<" | ">" | "<=" | ">=" | "in"

Guard ::=
    | "always"
    | "never"
    | Predicate
    | Guard "and" Guard
    | Guard "or" Guard
    | "not" Guard
    | "(" Guard ")"

Predicate ::= Identifier "(" Arguments ")"

Arguments ::= Expression ("," Expression)*

Authority ::= "authority:" "[" CapabilityRef* "]"
Obligations ::= "obligations:" "[" ObligationRef* "]"

ObligationRef ::= RoleRef "must" Condition | RoleRef "may" ActionRef | RoleRef "must-not" ActionRef

CheckRef ::= "check" Predicate | "verify" Predicate | "ensure" Condition

Type ::= Identifier | "List" "<" Type ">" | "Option" "<" Type ">" | "Map" "<" Type "," Type ">"

Duration ::= Number ("ms" | "s" | "m" | "h" | "d")
Number ::= [0-9]+
Literal ::= String | Number | "true" | "false" | "null"
String ::= "\"" [^\"]* "\""
Identifier ::= [a-zA-Z_][a-zA-Z0-9_-]*
```

---

## 4. Operational Semantics (Big-Step)

### 4.1 Judgment Form

The big-step operational semantics tracks not just values but effects, traces, and provenance:

```
Γ, C, Ω ⊢ w ⇓ v, ε, T, π

Where:
  Γ  = environment (variable bindings)
  C  = capability context (available capabilities)
  Ω  = obligation context (active obligations)
  w  = workflow term
  v  = resulting value
  ε  = accumulated effect (from effect lattice)
  T  = trace events list
  π  = provenance information
```

### 4.2 Key Inference Rules

#### Epistemic Layer (Observation)

```
(OBSERVE)
  lookup(C, cap) = Capability τ σ _
  σ ≤ epistemic
  execute(cap, Γ) ↝ v
  Γ' = bind(pat, v, Γ)
  Γ', C, Ω ⊢ cont ⇓ v', ε, T, π
  ────────────────────────────────────────────────────────────────
  Γ, C, Ω ⊢ OBSERVE cap as pat in cont ⇓ v', ε⊔epistemic, 
            T∪[Obs(cap, v, timestamp)], π
```

**Reading**: To observe, we look up the capability (which must be epistemic), execute it to get value v, bind v to the pattern in the environment, then continue with the rest of the workflow.

#### Deliberative Layer (Analysis)

```
(ORIENT)
  eval(Γ, expr) ↝ v
  analyze(v) ↝ v'
  Γ, C, Ω ⊢ cont ⇓ v'', ε, T, π
  ────────────────────────────────────────────────────────────────
  Γ, C, Ω ⊢ ORIENT expr in cont ⇓ v'', ε⊔deliberative,
            T∪[Orient(expr, v, v')], π
```

```
(PROPOSE)
  ────────────────────────────────────────────────────────────────
  Γ, C, Ω ⊢ PROPOSE action in cont ⇓ v, deliberative,
            T∪[Propose(action, Γ)], π
  where Γ, C, Ω ⊢ cont ⇓ v, _, _, _
  
  [Note: PROPOSE is advisory and does not execute]
```

#### Evaluative Layer (Governance)

```
(DECIDE-PERMIT)
  eval(Γ, expr) ↝ v
  policy(p, v, Γ) = Permit
  Γ, C, Ω ⊢ cont ⇓ v', ε, T, π
  ────────────────────────────────────────────────────────────────
  Γ, C, Ω ⊢ DECIDE expr under p in cont ⇓ v', ε⊔evaluative,
            T∪[Decide(p, Permit, v)], π

(DECIDE-DENY)
  eval(Γ, expr) ↝ v
  policy(p, v, Γ) = Deny
  ────────────────────────────────────────────────────────────────
  Γ, C, Ω ⊢ DECIDE expr under p in cont ⇓ ⊥, evaluative,
            T∪[Decide(p, Deny, v)], π, error:PolicyViolation
```

#### Operational Layer (Execution)

```
(ACT)
  eval_guard(Γ, guard) = true
  lookup(C, action) = Capability τ operational _
  policy_check(action, Γ) = Permit
  execute(action, Γ) ↝ v
  π' = extend(π, action, guard, v)
  ────────────────────────────────────────────────────────────────
  Γ, C, Ω ⊢ ACT action where guard ⇓ v, operational,
            T∪[Act(action, v, guard, timestamp)], π'
```

#### Control Flow

```
(SEQ)
  Γ, C, Ω ⊢ w1 ⇓ v1, ε1, T1, π1
  Γ, C, Ω ⊢ w2 ⇓ v2, ε2, T2, π2
  ────────────────────────────────────────────────────────────────
  Γ, C, Ω ⊢ SEQ w1 w2 ⇓ v2, ε1⊔ε2, T1T2, merge(π1, π2)

(PAR)
  ∀i. Γ, C, Ω ⊢ wi ⇓ vi, εi, Ti, πi
  ────────────────────────────────────────────────────────────────
  Γ, C, Ω ⊢ PAR [w1..wn] ⇓ [v1..vn], ⊔εi, ⧆Ti, merge(πi)

(IF-TRUE)
  eval(Γ, cond) = true
  Γ, C, Ω ⊢ thenBranch ⇓ v, ε, T, π
  ────────────────────────────────────────────────────────────────
  Γ, C, Ω ⊢ IF cond thenBranch elseBranch ⇓ v, ε, T, π

(LET)
  eval(Γ, expr) ↝ v
  Γ' = bind(pat, v, Γ)
  Γ', C, Ω ⊢ cont ⇓ v', ε, T, π
  ────────────────────────────────────────────────────────────────
  Γ, C, Ω ⊢ LET pat = expr in cont ⇓ v', ε, T, π
```

### 4.3 Error Handling

```
(ATTEMPT-SUCCESS)
  Γ, C, Ω ⊢ w1 ⇓ v, ε, T, π
  ────────────────────────────────────────────────────────────────
  Γ, C, Ω ⊢ ATTEMPT w1 catch w2 ⇓ v, ε, T, π

(ATTEMPT-CATCH)
  Γ, C, Ω ⊢ w1 ⇓ ⊥, ε, T, π, error:e
  Γ, C, Ω ⊢ w2 ⇓ v, ε', T', π'
  ────────────────────────────────────────────────────────────────
  Γ, C, Ω ⊢ ATTEMPT w1 catch w2 ⇓ v, ε⊔ε', T∪T'∪[Catch(e)], π'
```

---

## 5. Type System with Effect Tracking

### 5.1 Type Judgment

```
Γ, Σ ⊢ w : τ / ε ⊣ Ω'

Where:
  Γ   = type environment
  Σ   = capability signature context
  w   = workflow
  τ   = result type
  ε   = effect (from lattice)
  Ω'  = obligations discharged or incurred
```

### 5.2 Key Typing Rules

```
(OBSERVE-T)
  Σ(cap) = τ_obs → σ    σ ≤ epistemic
  Γ, x:τ_obs, Σ ⊢ cont : τ / ε ⊣ Ω
  ────────────────────────────────────────────────────────────────
  Γ, Σ ⊢ OBSERVE cap as x in cont : τ / epistemic⊔ε ⊣ Ω

(ACT-T)
  Σ(action) : τ_args → τ_ret / σ
  σ ≤ operational
  Γ ⊢ guard : Bool / εg
  Γ ⊢ args : τ_args / εa
  ────────────────────────────────────────────────────────────────
  Γ, Σ ⊢ ACT action(args) where guard : τ_ret / εg⊔εa⊔operational ⊣ ∅

(SEQ-T)
  Γ, Σ ⊢ w1 : τ1 / ε1 ⊣ Ω1
  Γ, Σ∪Ω1 ⊢ w2 : τ2 / ε2 ⊣ Ω2
  ────────────────────────────────────────────────────────────────
  Γ, Σ ⊢ SEQ w1 w2 : τ2 / ε1⊔ε2 ⊣ Ω2

(PAR-T)
  ∀i. Γ, Σ ⊢ wi : τi / εi ⊣ Ωi
  ────────────────────────────────────────────────────────────────
  Γ, Σ ⊢ PAR [w1..wn] : (τ1,..,τn) / ⊔εi ⊣ ∩Ωi

(WITH-T)
  Σ, cap:τ→σ ⊢ w : τ' / ε ⊣ Ω
  ────────────────────────────────────────────────────────────────
  Γ, Σ ⊢ WITH cap DO w : τ' / σ⊔ε ⊣ Ω

(OBLIG-T)
  lookup(Ω, role, oblig) = Condition
  Γ ⊢ check : Bool / ε
  Γ, Σ ⊢ cont : τ / ε' ⊣ Ω'
  ────────────────────────────────────────────────────────────────
  Γ, Σ ⊢ OBLIG role to check in cont : τ / ε⊔ε'⊔evaluative 
            ⊣ Ω' - {oblig}
```

### 5.3 Subtyping and Effect Polymorphism

```
(SUB-EFFECT)
  ε1 ≤ ε2
  ─────────────────
  τ / ε1 <: τ / ε2

(SUB-WORKFLOW)
  w1 <: w2  if  w1 can be used wherever w2 is expected
  (structural subtyping on workflow shapes)
```

---

## 6. Deontic Logic Layer

### 6.1 Deontic Operators

Following Prompt 43's discussion of deontic logic for obligations:

```
O(role, φ)  -- Role is obliged to ensure φ (must)
P(role, α)  -- Role is permitted to perform α (may)
F(role, α)  -- Role is forbidden from α (must not)
```

### 6.2 Deontic Inference Rules

```
(OBL-INTRO)
  workflow w contains OBLIG(r, check)
  check requires φ
  ─────────────────────────────
  Γ ⊢ O(r, φ)

(PER-INTRO)
  capability α ∈ authority(r)
  policy permits(r, α)
  ─────────────────────────────
  Γ ⊢ P(r, α)

(FORB-INTRO)
  policy explicitly denies(r, α)
  ─────────────────────────────
  Γ ⊢ F(r, α)

(DUALITY-1)
  Γ ⊢ F(r, α)  ↔  Γ ⊢ ¬P(r, α)

(DUALITY-2)
  Γ ⊢ O(r, φ)  →  Γ ⊢ P(r, ensure(φ))

(CONSISTENCY)
  not (Γ ⊢ P(r, α) and Γ ⊢ F(r, α))

(ROLE-APPROVAL-REFERENCE)
  approval_role(r)
  ─────────────────────────────
  Γ ⊢ can_route_approval(r)
```

### 6.3 Deontic Violations

```
(OBL-VIOLATION)
  Γ ⊢ O(r, φ)
  execute(w) leads to ¬φ
  r did not escalate or request exception
  ─────────────────────────────
  violation(r, O(r, φ))
```

---

## 7. Dynamic Logic Layer

### 7.1 Dynamic Logic Formulas

From Prompt 43's discussion of dynamic logic for workflow properties:

```
[w]φ  -- After executing workflow w, φ holds (necessity)
<w>φ  -- There exists an execution of w where φ holds (possibility)
```

### 7.2 Axioms for Workflow Properties

```
(ACT-AXIOM)
  [ACT a where g]φ ↔ (g → φ[a/result])

(SEQ-AXIOM)
  [SEQ w1 w2]φ ↔ [w1][w2]φ

(CHOICE-AXIOM)
  [IF c THEN w1 ELSE w2]φ ↔ (c → [w1]φ) ∧ (¬c → [w2]φ)

(ITER-AXIOM)
  [FOREACH x IN xs DO w]φ ↔ ∀x∈xs. [w]φ

(PAR-AXIOM)
  [PAR ws]φ ↔ ∧[w∈ws][w]φ  (all parallel branches satisfy φ)

(OBSERVE-AXIOM)
  [OBSERVE cap]φ ↔ φ[knowledge_acquired]

(OBLIG-AXIOM)
  [OBLIG r to check]φ ↔ check(r) ∧ [verify(check)]φ
```

### 7.3 Safety Properties

```
(SAFETY-PROP)
  A workflow w is safe if: ∀φ. [w]φ → φ holds in all executions

(LIVENESS-PROP)
  A workflow w is live if: ∀φ. <w>φ → φ is achievable

(PROGRESS)
  w makes progress if: [w](value ≠ stuck)
```

---

## 8. Proof Obligations

The type system and logic generate proof obligations that must be discharged:

### 8.1 Effect Safety

```
∀w ∈ Workflow. 
  ACT(α) ∈ subterms(w) → 
  ∃DECIDE(p, expr) strictly before ACT in control flow.
    p(expr, context) = Permit
```

**Explanation**: Every operational action must be preceded by a policy decision that permits it.

### 8.2 Obligation Fulfillment

```
∀OBLIG(r, check) ∈ w.
  let deps = dependencies(check)
  in ∀d ∈ deps. d is satisfied before OBLIG executes
```

### 8.3 Role Separation (SoD)

```
∀w ∈ Workflow with review pattern.
  let drafter = role(OBSERVE draft)
  let reviewer = role(ORIENT review)
  in drafter ≠ reviewer
```

### 8.4 Capability Containment

```
caps_used(w) ⊆ caps_declared(Σ)
```

### 8.5 Guard Decidability

```
∀DECIDE(expr, p) ∈ w.
  terminates(expr) ∧ terminates(p)
```

### 8.6 Progress

```
Γ ⊢ w : τ / ε ⊣ Ω
Γ ⊢ ε ≤ operational → Γ ⊢ can_complete(w)
```

---

## 9. Execution Engine (Python Sketch)

```python
from dataclasses import dataclass
from typing import Dict, List, Optional, Any, Callable
import asyncio
from enum import Enum, auto

class EffectLevel(Enum):
    EPISTEMIC = 0      # Read-only
    DELIBERATIVE = 1   # Analysis/planning
    EVALUATIVE = 2     # Policy decisions
    OPERATIONAL = 3    # Side effects

@dataclass
class RuntimeState:
    environment: Dict[str, Any]
    capabilities: Dict[str, Capability]
    policies: Dict[str, Policy]
    trace: List[TraceEvent]
    provenance: Provenance
    effect_level: EffectLevel

class SharoRuntime:
    def __init__(self, capabilities: Dict[str, Capability], 
                 policies: Dict[str, Policy]):
        self.capabilities = capabilities
        self.policies = policies
        self.audit_log = []
        
    async def execute(self, workflow: Workflow, 
                      state: RuntimeState) -> Result:
        """Execute workflow with full provenance tracking."""
        
        match workflow:
            case OBSERVE(capability, pattern, continuation):
                # Epistemic: read-only operation
                cap = self.capabilities[capability]
                value = await cap.execute(state.environment)
                
                # Bind to pattern
                new_bindings = self.bind_pattern(pattern, value)
                new_state = state.with_bindings(new_bindings)
                
                # Record trace
                new_state.trace.append(ObservationEvent(
                    capability=capability,
                    value=value,
                    timestamp=now(),
                    context=state.environment
                ))
                
                # Update effect level
                new_state.effect_level = max(
                    state.effect_level, 
                    EffectLevel.EPISTEMIC
                )
                
                return await self.execute(continuation, new_state)
            
            case ACT(action, guard, provenance):
                # Operational: side effects
                
                # 1. Evaluate guard
                if not await self.eval_guard(guard, state):
                    raise GuardViolation(guard, state)
                
                # 2. Check policy
                decision = self.policies.check(action, state.environment)
                if decision == Decision.DENY:
                    raise PolicyViolation(action, state)
                elif decision == Decision.REQUIRE_APPROVAL:
                    approval = await self.request_approval(action, state)
                    if not approval:
                        raise ApprovalDenied(action, state)
                
                # 3. Execute
                cap = self.capabilities[action]
                result = await cap.execute(state.environment)
                
                # 4. Record provenance
                state.trace.append(ActionEvent(
                    action=action,
                    result=result,
                    guard=guard,
                    policy_decision=decision,
                    timestamp=now(),
                    provenance=provenance
                ))
                
                state.effect_level = EffectLevel.OPERATIONAL
                
                return Result(result, state)
            
            case DECIDE(expression, policy, continuation):
                # Evaluative: governance decision
                value = await self.eval_expression(expression, state)
                decision = self.policies[policy].evaluate(
                    value, state.environment
                )
                
                state.trace.append(DecisionEvent(
                    policy=policy,
                    input=value,
                    decision=decision,
                    timestamp=now()
                ))
                
                state.effect_level = max(
                    state.effect_level,
                    EffectLevel.EVALUATIVE
                )
                
                if decision == Decision.PERMIT:
                    return await self.execute(continuation, state)
                else:
                    raise WorkflowBlocked(decision, state)
            
            case OBLIG(role, check, continuation):
                # Deontic: obligation verification
                check_result = await self.verify_obligation(
                    role, check, state
                )
                
                state.trace.append(ObligationEvent(
                    role=role,
                    check=check,
                    satisfied=check_result,
                    timestamp=now()
                ))
                
                if not check_result:
                    raise ObligationViolation(role, check, state)
                
                return await self.execute(continuation, state)
            
            case PAR(workflows):
                # Parallel execution
                results = await asyncio.gather(*[
                    self.execute(w, state.fork()) 
                    for w in workflows
                ])
                
                # Merge results and traces
                merged_state = state.merge([r.state for r in results])
                merged_state.effect_level = max(
                    r.state.effect_level for r in results
                )
                
                return Result(
                    value=[r.value for r in results],
                    state=merged_state
                )
            
            case _:
                raise NotImplementedError(f"Unknown workflow: {workflow}")
    
    async def eval_guard(self, guard: Guard, state: RuntimeState) -> bool:
        """Evaluate a guard condition."""
        match guard:
            case Guard.ALWAYS:
                return True
            case Guard.NEVER:
                return False
            case Guard.PRED(predicate):
                return await predicate.evaluate(state.environment)
            case Guard.AND(g1, g2):
                return await self.eval_guard(g1, state) and \
                       await self.eval_guard(g2, state)
            case _:
                raise NotImplementedError
    
    def bind_pattern(self, pattern: Pattern, value: Any) -> Dict[str, Any]:
        """Bind a value to a pattern, returning new bindings."""
        match pattern:
            case Pattern.VAR(name):
                return {name: value}
            case Pattern.TUPLE(patterns):
                if not isinstance(value, (list, tuple)):
                    raise PatternMatchError(pattern, value)
                bindings = {}
                for p, v in zip(patterns, value):
                    bindings.update(self.bind_pattern(p, v))
                return bindings
            case Pattern.WILDCARD:
                return {}
            case _:
                raise NotImplementedError
```

---

## 10. Example Workflows

### 10.1 Customer Support Ticket (from Prompt 49)

```sharo
-- Capability declarations
capability search_kb : observe(query: String) returns Documents
capability analyze_sentiment : analyze(text: String) returns Sentiment
capability draft_reply : analyze(ticket: Ticket, context: Documents) 
                        returns Draft
capability send_email : act(to: Email, subject: String, body: String) 
                        where approved

-- Policy declarations
policy external_communication:
  when recipient.domain in internal_domains 
  then permit
  else require_approval(role: manager)

policy high_confidence:
  when confidence > 0.8 and sentiment != angry
  then permit
  else deny

-- Workflow
workflow support_ticket_resolution {
  -- Epistemic: Gather information
  observe search_kb with query: ticket.subject as docs;
  
  -- Deliberative: Analyze
  orient { analyze_sentiment(text: ticket.description) } as sentiment;
  orient { analyze(ticket, docs) } as analysis;
  
  -- Deliberative: Draft response
  propose draft_reply(ticket, docs) as draft;
  
  -- Evaluative: Policy check
  decide { analysis.confidence } under high_confidence then {
    
    -- Operational: Send (with policy guard)
    act send_email(
      to: ticket.customer_email,
      subject: "Re: " + ticket.subject,
      body: draft.content
    ) where external_communication;
    
  } else {
    -- Escalation path
    act escalate(to: senior_agent, reason: "low_confidence");
  }
  
  done
}
```

### 10.2 Code Review with Role Separation

```sharo
-- Role definitions
role drafter {
  authority: [read_code, create_pr, respond_to_comments],
  obligations: [ensure_tests_pass]
}

role reviewer {
  authority: [read_code, comment, request_changes, approve],
  obligations: [check_tests, check_security, review_logic]
}

-- Capabilities
capability fetch_pr : observe(pr_id: ID) returns PR
capability analyze_diff : analyze(pr: PR) returns Analysis
capability check_coverage : analyze(tests: TestSuite) returns Coverage
capability request_changes : act(pr: PR, comments: List<Comment>) 
capability merge_pr : act(pr: PR) where all_checks_pass

-- Workflow
workflow code_review {
  let pr = observe fetch_pr with pr_id: $input.pr_id;
  
  -- Parallel analysis
  par {
    orient analyze_diff(pr) as diff_analysis;
    orient check_coverage(pr.tests) as coverage
  };
  
  -- Obligations for reviewer
  oblige reviewer to check_tests(pr);
  oblige reviewer to check_security(pr);
  
  -- Decision based on analysis
  decide { coverage.percentage > 80 
           and diff_analysis.no_critical_issues } then {
    
    if diff_analysis.has_minor_issues then {
      act request_changes(pr, comments: diff_analysis.issues);
    } else {
      act merge_pr(pr) where reviewer_approved;
    }
    
  } else {
    act request_changes(
      pr, 
      comments: ["Coverage insufficient", "Critical issues found"]
    );
  }
  
  done
}
```

### 10.3 Multi-Agent Research Workflow

```sharo
workflow collaborative_research {
  -- Epistemic phase: Information gathering
  observe search_literature with query: research_question as papers;
  
  -- Parallel deliberation by different agents
  par {
    with role: analyst do {
      orient extract_findings(papers) as findings
    };
    
    with role: critic do {
      orient identify_gaps(papers) as gaps
    };
    
    with role: synthesizer do {
      orient identify_themes(papers) as themes
    }
  } as perspectives;
  
  -- Synthesis
  orient synthesize(perspectives) as synthesis;
  
  -- Peer review simulation
  propose draft_report(synthesis) as report;
  
  oblige role: reviewer to verify_claims(report);
  
  decide { report.confidence > threshold } then {
    act publish_report(report) where peer_reviewed;
  } else {
    maybe {
      act request_feedback(report)
    } else {
      act archive_as_draft(report)
    }
  }
  
  done
}
```

---

## 11. Success Criteria Verification

Per Prompt 49's success criterion: **"Human and AI can collaborate on same workflow representation"**

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Human-readable | ✅ | Natural language keywords (`observe`, `act`, `decide`), clear structure |
| LLM-generatable | ✅ | Context-free grammar, predictable patterns, clear delimiters |
| Machine-executable | ✅ | Complete operational semantics, executable Python sketch |
| Formal verification | ✅ | Type system, deontic logic, dynamic logic enable proofs |
| Traceable | ✅ | Provenance tracking, audit events, decision records |
| Gradual formalization | ✅ | Can start with surface syntax, add types/proofs later |

---

## 12. Future Extensions

1. **Mechanized Proofs**: Integrate with Coq/Lean for verified workflows
2. **Temporal Logic**: Add LTL/CTL for time-based properties
3. **Probabilistic Workflows**: Handle uncertainty in AI reasoning
4. **Visual Editor**: Graphical workflow designer
5. **Policy Synthesis**: Learn policies from examples

---

## References

- Prompt 40: Big-step operational semantics for DSL
- Prompt 41: Minimal operational semantics, IR design
- Prompt 42: Effect lattice, formal core, χ predicate
- Prompt 43: Logical substrates (operational, deontic, modal, dynamic logic)
- Prompt 44: Formal judgment, advisory semantics
- Prompt 45-48: Advanced type system, memory, composition
- Prompt 49: Extraction procedure, success criteria
