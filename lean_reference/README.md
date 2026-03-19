# Ash Reference Interpreter

Lean 4 reference implementation of the Ash workflow language.

## Overview

This interpreter serves as:
- **Executable specification** - Direct implementation of SPEC-004
- **Test oracle** - Differential testing against Rust implementation
- **Foundation for verification** - Future formal proofs of correctness

## Setup

### Prerequisites

1. Install Lean 4 via elan:
   ```bash
   curl https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh -sSf | sh
   source $HOME/.elan/env
   ```

2. Verify installation:
   ```bash
   elan --version
   lake --version
   ```

### Building

```bash
# Clone repository (if not already done)
git clone <repository-url>
cd ash/lean_reference

# Download dependencies and build
lake update
lake build
```

### Running

```bash
# Run the interpreter
lake exe ash_ref

# Expected output:
# Ash Reference Interpreter - Version 0.1
# ...
```

## Structure

```
lean_reference/
├── Ash/
│   ├── Core/           # Core types and serialization
│   │   ├── AST.lean    # Value, Expr, Pattern types
│   │   ├── Environment.lean  # Env, Effect, EvalResult
│   │   └── Serialize.lean    # JSON serialization
│   ├── Eval/           # Expression evaluation
│   │   ├── Expr.lean   # Main evaluator
│   │   ├── Pattern.lean  # Pattern matching
│   │   ├── Match.lean    # Match expressions
│   │   └── IfLet.lean    # If-let expressions
│   ├── Differential/   # Testing infrastructure
│   │   ├── Types.lean  # Comparison types
│   │   ├── Parse.lean  # Rust result parsing
│   │   └── Compare.lean  # Result comparison
│   └── Tests/          # Test suite
│       ├── Properties.lean  # Property-based tests
│       ├── Runner.lean      # Test runner
│       └── CI.lean          # CI integration
├── lakefile.lean       # Lake configuration
├── lean-toolchain      # Lean version specification
└── Main.lean           # Entry point
```

## Editor Setup

### VS Code

1. Install the "Lean 4" extension
2. Open the project folder
3. The Lean language server should start automatically

### Emacs

Install lean4-mode:
```bash
# Using straight.el
(straight-use-package 'lean4-mode)
```

## Development

### Building

```bash
lake build                    # Build library and executable
lake build Ash                # Build just the library
lake exe ash_ref              # Build and run executable
```

### Testing

```bash
lake exe test                 # Run test suite (when implemented)
```

### Clean Build

```bash
lake clean                    # Remove build artifacts
lake build                    # Full rebuild
```

## Dependencies

- **std4**: Standard library extensions
- **plausible**: Property-based testing framework

## AST Subset Relationship

**Important**: The Lean interpreter implements an **intentional subset** of the full Ash AST defined in Rust. This is a deliberate design choice for Phase 18 (Core ADT Operations).

### Supported Expressions

The Lean `Expr` type supports:
- ✅ `literal` - Literal values
- ✅ `variable` - Variable references
- ✅ `constructor` - ADT constructor calls
- ✅ `tuple` - Tuple construction
- ✅ `match` - Pattern matching expressions
- ✅ `if_let` - Conditional pattern binding

### Not Supported (by Design)

These Rust `Expr` variants are **intentionally excluded**:
- ❌ `FieldAccess` - Use pattern matching instead
- ❌ `IndexAccess` - Use pattern matching on lists
- ❌ `Unary` - Explicit constructors preferred
- ❌ `Binary` - Use match/if-let for control flow
- ❌ `Call` - Inline constructor calls instead

### Why a Subset?

1. **Verification Focus**: Lean serves as an executable specification for formal proofs
2. **Core ADT Operations**: Phase 18 focuses on constructor evaluation and pattern matching
3. **Tractability**: Smaller language = easier correctness proofs
4. **Differential Testing**: Sufficient for meaningful testing against Rust

See [docs/AST-Subset.md](docs/AST-Subset.md) for the complete comparison table.

## Related Documents

- [SPEC-004: Operational Semantics](../docs/spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Lean Reference](../docs/spec/SPEC-021-LEAN-REFERENCE.md)
- [TASK-137](../docs/plan/tasks/TASK-137-lean-setup.md) - This task
- [AST Subset Documentation](docs/AST-Subset.md) - Detailed comparison

## License

Same as the main Ash project (see ../LICENSE-MIT and ../LICENSE-APACHE)
