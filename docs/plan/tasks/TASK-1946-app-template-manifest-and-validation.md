# TASK-1946: App Template Manifest And Validation

**Status:** Planned
**Phase:** [PLAN-199: Productive App Libraries And Templates](../PLAN-199-PRODUCTIVE-APP-LIBRARIES-AND-TEMPLATES.md)

## Description

Define app template metadata, schema, validation rules, and conformance gates.

## Requirements

- Define template identity, version, required profiles, provider/resource expectations, files,
  parameters, and generated checks.
- Validate template metadata fail-closed before instantiation.
- Ensure templates do not grant authority beyond explicit profiles and provider bindings.
- Add tests for missing fields, stale versions, bad profile references, and unsupported syntax.

## TDD Steps

1. Add failing template manifest validation tests.
2. Implement minimal manifest/schema validation.
3. Add negative diagnostics tests.
4. Run focused template validation tests and Rust quality gates.

## Completion Checklist

- [ ] Template manifest/schema is documented.
- [ ] Validation rejects malformed or stale templates.
- [ ] Provider/profile expectations remain explicit.
- [ ] Unsupported syntax is caught before template promotion.
