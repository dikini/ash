# SPEC-001: Intermediate Representation (IR)

## Status: Draft

## 1. Overview

This document specifies the core Intermediate Representation (IR) for the Ash workflow language. The IR is the canonical representation used throughout the compiler and runtime.

## 2. Core Types

### 2.1 Effect Lattice

```rust
/// Effect levels form a complete lattice
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Effect {
    Epistemic = 0,      // Read-only operations
    Deliberative = 1,   // Analysis and planning
    Evaluative = 2,     // Policy decisions
    Operational = 3,    // Side-effecting actions
}
```

**Lattice Operations**:
- `join(Effect, Effect) -> Effect`: Least upper bound
- `meet(Effect, Effect) -> Effect`: Greatest lower bound
- `leq(Effect, Effect) -> bool`: Partial order check

**Axioms** (property-tested):
1. Associativity: `join(a, join(b, c)) == join(join(a, b), c)`
2. Commutativity: `join(a, b) == join(b, a)`
3. Idempotence: `join(a, a) == a`
4. Identity: `join(Epistemic, a) == a`, `join(Operational, a) == Operational`
5. Absorption: `meet(a, join(a, b)) == a`

### 2.2 Workflow AST

```rust
pub enum Workflow {
    // Epistemic layer
    Observe {
        capability: Capability,
        pattern: Pattern,
        continuation: Box<Workflow>,
    },
    
    // Deliberative layer
    Orient {
        expr: Expr,
        continuation: Box<Workflow>,
    },
    Propose {
        action: Action,
        continuation: Box<Workflow>,
    },
    
    // Evaluative layer
    Decide {
        expr: Expr,
        policy: Name,
        continuation: Box<Workflow>,
    },
    Check {
        obligation: Obligation,
        continuation: Box<Workflow>,
    },
    
    // Operational layer
    Act {
        action: Action,
        guard: Guard,
        provenance: Provenance,
    },
    Oblig {
        role: Role,
        workflow: Box<Workflow>,
    },
    
    // Control flow
    Let {
        pattern: Pattern,
        expr: Expr,
        continuation: Box<Workflow>,
    },
    If {
        condition: Expr,
        then_branch: Box<Workflow>,
        else_branch: Box<Workflow>,
    },
    Seq {
        first: Box<Workflow>,
        second: Box<Workflow>,
    },
    Par {
        workflows: Vec<Workflow>,
    },
    ForEach {
        pattern: Pattern,
        collection: Expr,
        body: Box<Workflow>,
    },
    Ret { expr: Expr },
    
    // Modal
    With {
        capability: Capability,
        workflow: Box<Workflow>,
    },
    Maybe {
        primary: Box<Workflow>,
        fallback: Box<Workflow>,
    },
    Must {
        workflow: Box<Workflow>,
    },
    
    Done,
}
```

### 2.3 Values

```rust
pub enum Value {
    Int(i64),
    String(Box<str>),        // Boxed for smaller enum size
    Bool(bool),
    Null,
    Time(DateTime<Utc>),
    Ref(Box<str>),
    List(Box<[Value]>),
    Record(HashMap<Box<str>, Value>),
    Cap(Box<str>),
}
```

**Invariants**:
- `List` and `Record` are immutable after creation
- `String` is valid UTF-8
- `Ref` is a valid URI

### 2.4 Patterns

```rust
pub enum Pattern {
    Variable(Name),
    Tuple(Box<[Pattern]>),
    Record(Box<[(Name, Pattern)]>),
    List(Box<[Pattern]>, Option<Name>), // [a, b, ..rest]
    Wildcard,
    Literal(Value),
}
```

**Matching Rules**:
- `Variable` binds any value
- `Tuple` matches same-length list
- `Record` matches superset of fields
- `List` matches exact prefix, binds rest if specified
- `Wildcard` matches any, no binding
- `Literal` matches equal value

### 2.5 Guards

```rust
pub enum Guard {
    Pred(Predicate),
    And(Box<Guard>, Box<Guard>),
    Or(Box<Guard>, Box<Guard>),
    Not(Box<Guard>),
    Always,
    Never,
}
```

**Semantics**:
- `Always` ≡ true
- `Never` ≡ false
- `And` short-circuits
- `Or` short-circuits
- `Not` negates

## 3. Provenance

```rust
#[derive(Debug, Clone)]
pub struct Provenance {
    pub workflow_id: WorkflowId,
    pub parent: Option<WorkflowId>,
    pub lineage: Box<[WorkflowId]>,
}

pub struct TraceEvent {
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub workflow_id: WorkflowId,
    pub details: EventDetails,
}

pub enum EventType {
    Observation,
    Orientation,
    Decision,
    Proposal,
    Action,
    ObligationCheck,
    Error,
}
```

## 4. Serialization

All IR types implement:
- `Serialize` / `Deserialize` (bincode for internal, JSON for debugging)
- `Debug` with compact representation
- `Display` for human-readable output

## 5. Properties (for property testing)

```rust
// Effect lattice properties
prop_effect_associativity()?
prop_effect_commutativity()?
prop_effect_idempotence()?
prop_effect_absorption()?

// Pattern matching properties
prop_pattern_exhaustive()?
prop_pattern_no_overlap()?
prop_pattern_binding_unique()?

// Workflow properties
prop_workflow_well_formed()?
prop_workflow_effect_monotonic()?
```

## 6. Versioning

The IR is versioned via:
```rust
pub const IR_VERSION: u32 = 1;
```

Breaking changes increment version. Runtime checks version compatibility.

## 7. Related Documents

- SPEC-002: Surface Language
- SPEC-003: Type System
- SPEC-004: Operational Semantics
