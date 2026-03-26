# Ash API Documentation

Complete API reference for all Ash crates.

## Table of Contents

- [ash-core](#ash-core) - Core types and IR
- [ash-parser](#ash-parser) - Lexer and parser
- [ash-typeck](#ash-typeck) - Type checker
- [ash-interp](#ash-interp) - Interpreter
- [ash-provenance](#ash-provenance) - Audit trails
- [ash-cli](#ash-cli) - Command line interface

---

## ash-core

Core abstract syntax tree, effects, and values.

### Effect System

The effect lattice tracks computational power:

```rust
use ash_core::effect::Effect;

// Effect levels (from least to most powerful)
let e1 = Effect::Epistemic;      // Read-only
let e2 = Effect::Deliberative;   // Analysis
let e3 = Effect::Evaluative;     // Decisions
let e4 = Effect::Operational;    // Side effects

// Lattice operations
let join = e1.join(e2);          // Least upper bound
let meet = e3.meet(e4);          // Greatest lower bound
let can = e4.at_least(e2);       // Check if effect is sufficient
```

### Values

Runtime values in Ash:

```rust
use ash_core::value::Value;

let v = Value::Record([
    ("name".to_string(), Value::String("test".to_string())),
    ("count".to_string(), Value::Int(42)),
].into_iter().collect());

// Accessors
if let Some(n) = v.as_int() {
    println!("Integer: {}", n);
}
```

### AST Types

Workflow AST representation:

```rust
use ash_core::ast::{Workflow, Expr, Pattern, BinaryOp};

// Create a simple workflow
let wf = Workflow::Let {
    pattern: Pattern::Variable("x".to_string()),
    expr: Expr::Literal(Value::Int(10)),
    continuation: Box::new(Workflow::Ret {
        expr: Expr::Variable("x".to_string()),
    }),
};
```

### Re-exports

```rust
pub use ast::*;        // Workflow, Expr, Pattern, etc.
pub use effect::*;     // Effect, EffectContext
pub use provenance::*; // Provenance, AuditTrail
pub use value::*;      // Value
pub use visualize::*;  // Visualization helpers
```

---

## ash-parser

Lexer and parser for Ash source code.

### Surface AST

Parsed representation with span information:

```rust
use ash_parser::surface::{Program, Workflow, Expr, Pattern};
use ash_parser::token::Span;

// Parse source code
let source = r#"
workflow main {
    let x = 42
    ret x
}
"#;
```

### Error Handling

Parse errors with recovery:

```rust
use ash_parser::error::ParseError;
use ash_parser::error_recovery::ErrorRecovery;

// Errors include source location
let error = ParseError::new(span, "unexpected token");
```

### Main Types

```rust
pub use surface::*;           // Surface AST types
pub use token::*;             // Tokens and spans
pub use error::*;             // Parse errors
pub use error_recovery::*;    // Error recovery
pub use lexer::*;             // Lexer
pub use parse_workflow::*;    // Workflow parser
pub use parse_expr::*;        // Expression parser
```

---

## ash-typeck

Type checker and effect inference.

### Type Checking

Main entry point:

```rust
use ash_typeck::type_check_workflow;
use ash_parser::surface::Workflow;

let workflow = Workflow::Done { span: Span::default() };
let result = type_check_workflow(&workflow)?;

if result.is_ok() {
    println!("Type check passed with effect {:?}", result.effect);
}
```

### Types

Core type system:

```rust
use ash_typeck::types::{Type, Substitution};

// Type constructors
let int = Type::Int;
let bool = Type::Bool;
let list = Type::List(Box::new(Type::Int));
let record = Type::Record(vec![
    ("name".to_string(), Type::String),
    ("age".to_string(), Type::Int),
]);

// Type variables
let var = Type::Var(0);

// Unification
let mut subst = Substitution::new();
subst.unify(&int, &Type::Int)?;
```

### Constraints

Constraint generation:

```rust
use ash_typeck::constraints::ConstraintContext;

let mut ctx = ConstraintContext::new();
// Generate constraints from AST
let constraints = ctx.constraints();
```

### Solver

Constraint solving:

```rust
use ash_typeck::solver::Solver;

let mut solver = Solver::new();
let substitution = solver.solve(&constraints)?;
```

### Effects

Effect inference:

```rust
use ash_typeck::effect::{EffectContext, infer_effect};

let ctx = EffectContext::new();
let effect = infer_effect(&workflow);
```

### Re-exports

```rust
pub use types::*;        // Type, Substitution, TypeVar
pub use constraints::*;  // ConstraintContext, Constraint
pub use solver::*;       // Solver, TypeError
pub use effect::*;       // EffectContext, infer_effect
pub use names::*;        // NameResolver
pub use obligations::*;  // ObligationTracker
```

---

## ash-interp

Runtime interpreter for Ash workflows.

### Simple Execution

Quick execution with defaults:

```rust
use ash_interp::interpret;
use ash_core::{Workflow, Expr, Value};

let workflow = Workflow::Ret {
    expr: Expr::Literal(Value::Int(42)),
};

let result = interpret(&workflow).await?;
assert_eq!(result, Value::Int(42));
```

### Context-Based Execution

Execution with custom context:

```rust
use ash_interp::{Context, execute_workflow};
use ash_interp::capability::{CapabilityRegistry, MockProvider};

// Create context
let mut ctx = Context::new();

// Register capabilities
let mut registry = CapabilityRegistry::new();
registry.register("read_file", MockProvider::new(Value::String("content".to_string())));

// Execute
let result = execute_workflow(&workflow, &ctx).await?;
```

### Capabilities

Capability providers:

```rust
use ash_interp::capability::{CapabilityProvider, CapabilityContext};
use async_trait::async_trait;

#[async_trait]
impl CapabilityProvider for MyProvider {
    async fn invoke(&self, ctx: &CapabilityContext, args: Vec<Value>) -> EvalResult<Value> {
        // Implementation
        Ok(Value::Null)
    }
}
```

### Evaluation

Expression evaluation:

```rust
use ash_interp::eval_expr;

let expr = Expr::Binary {
    op: BinaryOp::Add,
    left: Box::new(Expr::Literal(Value::Int(10))),
    right: Box::new(Expr::Literal(Value::Int(20))),
};

let value = eval_expr(&expr, &ctx).await?;
```

### Pattern Matching

```rust
use ash_interp::match_pattern;

let pattern = Pattern::Tuple(vec![
    Pattern::Variable("x".to_string()),
    Pattern::Variable("y".to_string()),
]);

let value = Value::List(vec![Value::Int(1), Value::Int(2)]);
let bindings = match_pattern(&pattern, &value)?;
```

### Re-exports

```rust
pub use context::Context;
pub use error::{EvalError, EvalResult, ExecError, ExecResult};
pub use eval::eval_expr;
pub use execute::{execute_workflow, execute_simple};
pub use pattern::match_pattern;
pub use guard::eval_guard;
pub use capability::{CapabilityProvider, CapabilityRegistry, MockProvider};
pub use policy::{Policy, PolicyEvaluator, PolicyResult};
```

---

## ash-provenance

Audit trails and provenance tracking.

### Audit Trail

Recording actions:

```rust
use ash_provenance::{AuditTrail, AuditEvent};

let mut trail = AuditTrail::new();

trail.record(AuditEvent::Action {
    action: "send_email".to_string(),
    actor: "user123".to_string(),
    timestamp: Utc::now(),
    details: json!({"to": "test@example.com"}),
});
```

### Provenance Graph

Tracking data lineage:

```rust
use ash_provenance::ProvenanceGraph;

let mut graph = ProvenanceGraph::new();
graph.add_node("source", Provenance::External);
graph.add_node("transform", Provenance::Derived(vec!["source".to_string()]));
```

### Verification

Verifying audit trails:

```rust
use ash_provenance::verify;

let result = verify::verify_trail(&trail)?;
assert!(result.is_valid());
```

---

## ash-cli

Command-line interface.

### Commands

```bash
# Type check a workflow
ash check <file.ash>

# Execute a workflow
ash run <file.ash>

# Run with provenance tracing
ash trace <file.ash>

# Interactive REPL
ash repl

# Generate DOT visualization
ash dot <file.ash>
```

### Options

```bash
# Enable verbose output
ash -v run workflow.ash

# Disable colors
ash --no-color check workflow.ash

# Show version
ash --version

# Show help
ash --help
```

### REPL Commands

When in REPL mode:

```
> let x = 42
> x + 10
52
> :type x
Int
> :help
Available commands:
  :help     Show this help
  :type     Show type of expression
  :effects  Show inferred effects
  :quit     Exit REPL
```

---

## Type Reference

### Core Types

| Type | Description | Example |
|------|-------------|---------|
| `Int` | 64-bit integer | `42` |
| `Float` | 64-bit float | `3.14` |
| `String` | UTF-8 string | `"hello"` |
| `Bool` | Boolean | `true`, `false` |
| `Null` | Null value | `null` |
| `Time` | Timestamp | - |
| `Ref` | External reference | - |
| `List<T>` | Homogeneous list | `[1, 2, 3]` |
| `Record` | Struct/map | `{x: 1, y: 2}` |
| `Cap` | Capability reference | - |

### Workflow Types

| Constructor | Effect | Description |
|-------------|--------|-------------|
| `Observe` | Epistemic | Read external data |
| `Orient` | Deliberative | Analyze/transform |
| `Propose` | Evaluative | Suggest action |
| `Decide` | Evaluative | Policy decision |
| `Check` | Evaluative | Verify obligation |
| `Act` | Operational | Execute action |
| `Let` | Varies | Variable binding |
| `If` | Varies | Conditional |
| `For` | Varies | Iteration |
| `Par` | Varies | Parallel composition |
| `Seq` | Varies | Sequential composition |

---

## Feature Flags

### ash-core

- `proptest-helpers` - Enable property testing helpers
- `serde` - Serialization support (enabled by default)

### ash-parser

- `error-recovery` - Enable error recovery (enabled by default)
- `location-info` - Full span information

### ash-typeck

- `smt` - Enable Z3 SMT solver for advanced constraint solving
- `detailed-errors` - Enhanced error messages

### ash-interp

- `async` - Async runtime support (enabled by default)
- `tracing` - Detailed execution tracing

---

## Version Compatibility

| Crate | Version | MSRV |
|-------|---------|------|
| ash-core | 0.1.0 | 1.94.0 |
| ash-parser | 0.1.0 | 1.94.0 |
| ash-typeck | 0.1.0 | 1.94.0 |
| ash-interp | 0.1.0 | 1.94.0 |
| ash-cli | 0.1.0 | 1.94.0 |

---

For more information, see the [tutorial](TUTORIAL.md) and [language specification](spec/).
