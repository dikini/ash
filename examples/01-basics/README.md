# Basic Examples

This directory contains basic examples introducing Ash workflow concepts.

## Files

### 01-hello-world.ash
The simplest possible workflow - just returns a greeting.

```bash
ash run 01-hello-world.ash
```

### 02-variables.ash
Demonstrates variable binding and pattern matching:
- Simple variable binding
- Tuple destructuring
- Record destructuring
- List patterns with rest binding
- Wildcard patterns

### 03-expressions.ash
Shows various expression types:
- Arithmetic operations (+, -, *, /)
- Comparison operators (==, !=, <, >, <=, >=)
- Logical operators (&&, ||, !)
- Boolean literals and null

### 04-observe.ash
Introduces the OODA (Observe-Orient-Decide-Act) pattern:
- `observe` - Read data from capabilities
- `orient` - Analyze and transform data
- `decide` - Apply policies
- `act` - Execute actions

## Key Concepts

1. **Workflows** are the main unit of computation
2. **Bindings** connect patterns to expressions
3. **Patterns** destructure values
4. **Expressions** compute values
5. **OODA Pattern** structures workflow phases
