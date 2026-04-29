# Ash Workflow Examples

This directory contains example workflows demonstrating the Ash workflow language features, from basic concepts to real-world applications.

Phase 107 classifies examples into two explicit categories:

- **Executable conformance examples** are included in the CLI corpus expected-pass set and must pass `ash check`.
- **Reference-only sketches** are preserved as historical or scenario-design material. They carry a visible `REFERENCE-ONLY` marker near the top of the file, are excluded from executable conformance counts, and are not promises of current parser syntax.

## Directory Structure

```
examples/
├── 01-basics/          # Basic language features
├── 02-control-flow/    # Control flow patterns
├── 03-policies/        # Policy and governance
├── 04-real-world/      # Real-world applications
├── 05-phase98/         # Proc/failure/workflow-boundary conformance examples
└── 06-capability-implementations/ # Capability interface/impl/resource examples
```

## Quick Start

For executable conformance examples, use the Ash CLI:

```bash
# Type check a conformance example
ash check examples/01-basics/01-hello-world.ash

# Run an executable conformance example when its runtime providers are available
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
- **03-sequential.ash**: Sequential composition

### 03 - Policies
- **01-role-based.ash**: Role-based access control
- **02-time-based.ash**: Time-based policy enforcement

### 04 - Real World
- **customer-support.ash**: Support ticket workflow
- **code-review.ash**: Pull request review workflow

### 05 - Phase 98 Proc/Failure Conformance
- **01-fail-with-error.ash**: `fail`/`with_error` recovery exercised through `ash check`, `ash run`, and `ash trace`
- **02-proc-par-await-join.ash**: source-level `Proc`/`P`, `par`, `await`, `join`, and `yield` composition; the example returns both live handles and source-built observer procs, CLI run/trace honestly show the returned `Proc` closure, and engine tests wait for retained child results before forcing the observers
- **03-proc-scatter-gather.ash**: `scatter`, `gather`, and `yield` composed as a source-level Proc value that engine tests then force honestly
- **04-workflow-boundary-reporting.ash**: ordinary workflow source used by engine admission/report tests to exercise `WorkflowBoundaryOutcome` / compatibility wrappers

workflow boundary reporting currently requires the engine admission API rather than a standalone CLI/report surface. TASK-717 therefore keeps the reporting assertion in engine admission/report tests and uses `04-workflow-boundary-reporting.ash` as the source-level workflow input for that path.

### 06 - Capability Implementations
- **01-mock-internal-kv.ash**: mock/internal key-value implementation packet for `ash check`
- **02-caching-kv-adapter.ash**: derived logging/cache adapter pattern over another admitted `KeyValue` capability
- **03-recording-replay-sketch.ash**: record/replay capability substitution sketch backed by replay-log authority

These Phase 104 source examples are checkable declaration packets. Runtime execution coverage for the same substitution/adapter/replay patterns is currently provided by `ash-interp` API tests because standalone `ash run` lowering from source-level capability implementation declarations into runtime admissions is not complete yet.

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
