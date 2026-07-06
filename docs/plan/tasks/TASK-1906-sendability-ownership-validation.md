# TASK-1906: Sendability Ownership Validation

**Status:** ✅ Complete
**Phase:** [PLAN-195: Process And Concurrency Model](../PLAN-195-PROCESS-AND-CONCURRENCY-MODEL.md)

## Description

Validate sendability, ownership transfer, affine movement, and borrowed-resource rejection across
process boundaries.

## Requirements

- Accept owned sendable values.
- Reject non-sendable closures, borrowed resources, unstable observers, and live handler frames.
- Emit structured diagnostics for invalid transfers.

## TDD Steps

1. Write failing positive and negative transfer tests.
2. Implement sendability and ownership validation.
3. Add diagnostics tests for each rejection family.

## Completion Checklist

- [x] Sendable owned values cross process boundaries.
- [x] Non-sendable and borrowed values fail closed.
- [x] Diagnostics identify the rejected transfer reason.
