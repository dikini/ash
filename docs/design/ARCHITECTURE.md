# Ash Architecture

## Overview

Ash is a reference implementation of the Sharo Core Language (SHC) - a workflow language for governed AI systems. The architecture follows a layered design with clear separation between syntax, semantics, and execution.

## Design Principles

1. **Correctness First**: Use Rust's type system to prevent errors at compile time
2. **Zero-Cost Abstractions**: Runtime performance equals hand-written code
3. **Composability**: Small, focused components that compose cleanly
4. **Observability**: Every decision is traceable and auditable
5. **Testability**: Property-based testing throughout (TDD)

## Architecture Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLI / API                               │
│                    (ash-cli, ash-server)                        │
├─────────────────────────────────────────────────────────────────┤
│                      Runtime (ash-interp)                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │  Eval    │  │  Policy  │  │  Effect  │  │   GC     │        │
│  │  Engine  │  │  Engine  │  │  Tracker │  │ (futures)│        │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
├─────────────────────────────────────────────────────────────────┤
│                    Provenance (ash-provenance)                  │
│            Audit trails, trace recording, lineage               │
├─────────────────────────────────────────────────────────────────┤
│                    Type System (ash-typeck)                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │  Effect  │  │   Type   │  │ Oblig-   │  │  Proof   │        │
│  │  Lattice │  │ Checker  │  │ ation    │  │ Engine   │        │
│  │          │  │          │  │ Checker  │  │          │        │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
├─────────────────────────────────────────────────────────────────┤
│                    Parser (ash-parser)                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                      │
│  │  Lexer   │  │  Parser  │  │  AST     │                      │
│  │          │  │ (winnow) │  │  Lower   │                      │
│  └──────────┘  └──────────┘  └──────────┘                      │
├─────────────────────────────────────────────────────────────────┤
│                    Core (ash-core)                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │    IR    │  │  Effect  │  │  Value   │  │ Proven-  │        │
│  │   (AST)  │  │  System  │  │  System  │  │ ance     │        │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

## Component Specifications

### 1. ash-core

**Purpose**: Foundation types used by all other crates.

**Key Types**:
- `Workflow`: The core AST node type
- `Effect`: Lattice for tracking computational power
- `Value`: Runtime values with serde support
- `Provenance`: Audit trail information
- `TraceEvent`: Observable execution events

**Design Decisions**:
- Immutable AST nodes (cheap cloning via Arc where needed)
- `Effect` implements `PartialOrd` to form a lattice
- `Value` uses `Box<[T]>` for fixed-size collections

**Invariants**:
- Effects are never coerced "up" without explicit join
- Values are always valid (parse don't validate at this level)

### 2. ash-parser

**Purpose**: Transform surface syntax into IR.

**Pipeline**:
```
Source Text → Lexer → Token Stream → Parser → Surface AST → Lower → Core AST
```

**Key Modules**:
- `lexer`: Tokenization with helpful error messages
- `parser`: Recursive descent using winnow
- `surface_ast`: Intermediate representation
- `lower`: Translation to core IR with desugaring

**Error Strategy**:
- Rich error types with spans
- Multiple errors collected when possible
- Suggestions for common mistakes

### 3. ash-typeck

**Purpose**: Static analysis and verification.

**Passes**:
1. **Name Resolution**: Bind identifiers to definitions
2. **Effect Inference**: Compute minimal effects for each workflow
3. **Type Checking**: Verify type compatibility
4. **Obligation Analysis**: Track deontic constraints
5. **Proof Obligation Generation**: Emit conditions to verify
6. **Constraint Solving**: Detect policy conflicts via SMT

**Key Algorithms**:
- Hindley-Milner style effect inference with join/meet
- Dataflow analysis for obligation tracking
- SMT solving via Z3 for policy conflict detection
- Optimization for constraint satisfaction problems

**SMT Integration**:
```
┌──────────────────────────────────────────────┐
│           Policy Constraint Encoder          │
│  (Convert policy constraints to SMT-LIB)     │
└─────────────────────┬────────────────────────┘
                      ▼
┌──────────────────────────────────────────────┐
│              Z3 Solver Context               │
│  - Resource thresholds (budget, rate limits) │
│  - Temporal constraints (time windows)       │
│  - Cardinality constraints (retry limits)    │
│  - Cross-variable constraints                │
│  - Optimization objectives                   │
└─────────────────────┬────────────────────────┘
                      ▼
┌──────────────────────────────────────────────┐
│         Conflict Detection Results           │
│  - SAT: Policies compatible                  │
│  - UNSAT: Conflict with unsat core           │
│  - UNKNOWN: Solver timeout                   │
└──────────────────────────────────────────────┘
```

**Feature Flag**: `smt = ["z3"]` - enables full constraint solving
**Fallback**: Without `smt` flag, uses structural analysis for simple conflicts

### 4. ash-interp

**Purpose**: Execute workflows.

**Architecture**:
```
┌──────────────────────────────────────────────┐
│              Interpreter Loop                │
├──────────────────────────────────────────────┤
│  1. Match on workflow node                   │
│  2. Evaluate in current context              │
│  3. Handle effects via capability providers  │
│  4. Record trace events                      │
│  5. Return result with updated state         │
└──────────────────────────────────────────────┘
```

**Capability System**:
- Trait-based capability providers
- Async execution throughout
- Cancellation support

**State Management**:
- Immutable contexts with structural sharing
- Persistent data structures for large collections

### 5. ash-provenance

**Purpose**: Complete audit trail and lineage tracking.

**Features**:
- Merkle-tree-like trace integrity
- Queryable trace store
- Export to standard formats (W3C PROV, etc.)

**Storage**:
- In-memory for short workflows
- Append-only log for production
- Optional cryptographic signing

### 6. ash-cli

**Purpose**: User-facing interface.

**Commands**:
- `check`: Type check and lint
- `run`: Execute with various output formats
- `trace`: Run with full provenance capture
- `repl`: Interactive development
- `serve`: HTTP API for integration

## Data Flow

### Compile-Time Flow

```
.ash file → Parse → Surface AST → Lower → Core AST → Type Check → Verified IR
                ↓                      ↓              ↓
              Errors               Warnings      Proof Obligations
```

### Run-Time Flow

```
Verified IR → Prepare → Execute → Record → Result + Trace
                  ↓         ↓
           Capabilities  Effects
                  ↓         ↓
            External     Audit
              World       Log
```

## Effect System Design

The effect lattice forms the foundation of safety:

```
                    Operational
                        |
                   Evaluative
                        |
                  Deliberative
                        |
                    Epistemic
```

**Join (⊔)**: Least upper bound - used for sequential composition
**Meet (⊓)**: Greatest lower bound - used for capability intersection

**Properties**:
- Associative: (a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)
- Commutative: a ⊔ b = b ⊔ a
- Idempotent: a ⊔ a = a
- Has bottom (Epistemic) and top (Operational)

## Testing Strategy

### Unit Tests
- Every function has property-based tests
- Invariants checked via `proptest`
- Example-based tests for regression prevention

### Integration Tests
- Full workflow execution
- Round-trip: parse → check → run → trace
- Property: Traces are always reproducible

### Verification
- Type safety: Well-typed programs don't get stuck
- Effect safety: No operational action without decision
- Audit completeness: Every action is traceable
- Policy consistency: No contradictory obligations via SMT

### SMT Testing
- Property: Policy conflicts are detected before execution
- Property: Unsat cores explain *why* policies conflict
- Property: Optimization objectives find optimal valid configurations
- Fuzzing: Random policy constraints should not crash solver
- Regression: Real-world conflict patterns from deployment

## Performance Considerations

### Zero-Cost Abstractions
- Effects are compile-time tracked where possible
- Runtime effect checks are branch-predicted (likely/unlikely)
- No allocation in hot paths

### Memory Layout
- Small: `Workflow` nodes fit in cache lines
- Cache-friendly: Sequential trace recording
- Zero-copy: Slices over owned data where possible

### Async Design
- Work-stealing runtime (Tokio)
- Cooperative cancellation
- Bounded channels for backpressure

## Security Model

### Threat Model
1. Malicious workflow code
2. Compromised capability providers
3. Audit tampering
4. Policy bypass attempts

### Mitigations
- Sandboxed capability execution
- Cryptographic trace integrity
- Separation of policy evaluation from execution
- Immutable audit logs

## Extension Points

### Adding New Effects
1. Add variant to `Effect` enum
2. Update lattice operations
3. Add capability provider trait
4. Update parser and type checker

### Adding New Workflow Constructs
1. Add variant to `Workflow` AST
2. Implement lowering from surface syntax
3. Add type checking rules
4. Implement interpreter handler
5. Add trace event variant

## Migration Path

From prototype to production:

1. **Phase 1**: Core IR + Parser (basic execution)
2. **Phase 2**: Type system + Effects (safety)
3. **Phase 3**: Provenance + Audit (observability)
4. **Phase 4**: Optimizations + Distribution (scale)

Each phase maintains backward compatibility with previous.

## References

- [Sharo Core Language Specification](../SHARO_CORE_LANGUAGE.md)
- Rust API Guidelines
- W3C PROV Standard
- Capability-Based Security (Miller et al.)
