# SPEC-001: Intermediate Representation (IR)

## Status: Draft

## 1. Overview

This document specifies the canonical core Intermediate Representation (IR) for the Ash workflow
language. The IR is the authoritative contract used throughout lowering, type checking, execution,
and future compilation backends.

The core contract is execution-neutral:

- it does not assume a tree-walking interpreter,
- it does not assume a bytecode or stack-machine runtime,
- it does not assume a JIT, but it allows one later,
- it defines meaning by observable evaluation results, effects, traces, and provenance rather
  than by a particular machine strategy.

Surface syntax may elaborate into this IR, but the IR contract itself is the canonical truth.

## 2. Core Types

### 2.0 Canonical Core Language

The canonical core language is the contractually meaningful subset of Ash syntax and behavior.
These forms are the ones downstream phases must preserve:

- core workflows: `Observe`, `Receive`, `Orient`, `Propose`, `Decide`, `Check`, `Act`, `Oblig`,
  `Let`, `If`, `Seq`, `Par`, `ForEach`, `Ret`, `With`, `Maybe`, `Must`, `Done`
- core expressions: `Literal`, `Variable`, `FieldAccess`, `IndexAccess`, `Unary`, `Binary`,
  `Call`, `Match`, `Constructor`
- core patterns: `Variable`, `Tuple`, `Record`, `List`, `Wildcard`, `Literal`, `Variant`

Anything outside that set is either surface syntax or a lowering convenience. In particular:

- `if let` is a surface convenience that lowers to canonical matching behavior
- parser-only scaffolding is not a core-language contract
- implementation-private representation choices are not part of the IR contract

### 2.1 Effect Lattice

```rust
/// Effect levels form a complete lattice
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Effect {
    Epistemic = 0,      // Input acquisition and read-only observation
    Deliberative = 1,   // Analysis, planning, and proposal formation
    Evaluative = 2,     // Policy and obligation evaluation
    Operational = 3,    // External side effects and irreversible outputs
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
    Receive {
        mode: ReceiveMode,
        arms: Vec<ReceiveArm>,
        control: bool,
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

**Canonical workflow-form contracts**:
- `Check` is obligation-only in the IR. It discharges or rejects an `Obligation`; policies are evaluated by `Decide`, not `Check`.
- `Decide` always carries an explicit named `policy`. There is no policy-free core `Decide` form.
- `Receive` is the canonical IR form for mailbox input. It preserves the receive mode and the ordered arm list from the surface language.

**Execution-neutral IR invariants**:
- The IR meaning is defined by the evaluation relation and observable results, not by evaluator
  control strategy.
- Lowering may normalize or preserve structure, but it must not require a specific backend
  architecture.
- Any future JIT must implement the same core meaning as interpretation, not a different contract.
- Core nodes are stable contract boundaries even when current implementations use convenience
  representations internally.

```rust
pub enum ReceiveMode {
    NonBlocking,
    Blocking(Option<Duration>),
}

pub struct ReceiveArm {
    pattern: ReceivePattern,
    guard: Option<Expr>,
    body: Workflow,
}

pub enum ReceivePattern {
    Stream {
        capability: Capability,
        channel: Name,
        pattern: Pattern,
    },
    Literal(Value),
    Wildcard,
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
    Receive,
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
