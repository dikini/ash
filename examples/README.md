# Ash Workflow Examples

This directory contains example workflows demonstrating the Ash workflow language features, from basic concepts to real-world applications.

## Directory Structure

```
examples/
├── 01-basics/          # Basic language features
├── 02-control-flow/    # Control flow patterns
├── 03-policies/        # Policy and governance
└── 04-real-world/      # Real-world applications
```

## Quick Start

Each example can be run using the Ash CLI:

```bash
# Type check an example
ash check examples/01-basics/01-hello-world.ash

# Run an example
ash run examples/01-basics/01-hello-world.ash

# Generate visualization
ash dot examples/01-basics/01-hello-world.ash > workflow.dot
dot -Tpng workflow.dot -o workflow.png
```

## Examples Overview

### 01 - Basics
- **01-hello-world.ash**: Simplest possible workflow
- **02-variables.ash**: Variable binding and patterns
- **03-expressions.ash**: Arithmetic and logical expressions
- **04-observe.ash**: Using the OODA observe pattern

### 02 - Control Flow
- **01-conditionals.ash**: If/then/else branching
- **02-foreach.ash**: Looping over collections
- **03-parallel.ash**: Parallel workflow execution
- **04-sequential.ash**: Sequential composition

### 03 - Policies
- **01-role-based.ash**: Role-based access control
- **02-time-based.ash**: Time-based policy enforcement

### 04 - Real World
- **customer-support.ash**: Support ticket workflow
- **code-review.ash**: Pull request review workflow

## Canonical ADT Helper Surface

The examples and the automatically imported prelude use the canonical Option/Result helper surface:

### Option helper surface

- `is_some`, `is_none`, `unwrap`, `unwrap_or`, `map`, `and`, `or`, `ok_or`

### Result helper surface

- `is_ok`, `is_err`, `unwrap_res`, `unwrap_err`, `unwrap_or_res`, `map_res`, `map_err`, `and_then`, `ok`, `err_opt`

## Learning Path

1. Start with `01-basics/` to understand the core concepts
2. Move to `02-control-flow/` for flow control patterns
3. Explore `03-policies/` for governance features
4. Study `04-real-world/` for practical applications

## Additional Resources

- [Tutorial](../docs/TUTORIAL.md): Step-by-step tutorial
- [API Documentation](../docs/API.md): API reference
- [Language Specification](../docs/spec/): Detailed language specification
