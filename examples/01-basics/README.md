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
Introduces the OODA (Observe-Orient-Decide-Act) compatibility pattern as
ordinary library/template calls over the current tower algebra:
- `ooda::observe` - Mark observation-shaped template values
- `ooda::orient` - Mark orientation-shaped template values
- `ooda::decide` - Mark decision-shaped template values
- `ooda::act` - Mark action-shaped template values

Historical examples remain useful teaching material, but alpha execution
semantics come from visible `Act`, `Proc`, and `Workflow` operations rather
than primitive OODA IR roots.

## Key Concepts

1. **Workflows** are the main unit of computation
2. **Bindings** connect patterns to expressions
3. **Patterns** destructure values
4. **Expressions** compute values
5. **OODA Pattern** remains a library/template compatibility convention
