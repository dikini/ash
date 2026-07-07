# TASK-1946: App Template Manifest And Validation

**Status:** Complete
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

- [x] Template manifest/schema is documented.
- [x] Validation rejects malformed or stale templates.
- [x] Provider/profile expectations remain explicit.
- [x] Unsupported syntax is caught before template promotion.

## Evidence

- Added `ash_cli::templates` with `TemplateManifest`, provider/file/parameter/check metadata
  structs, `TEMPLATE_SCHEMA_VERSION`, and fail-closed validation diagnostics.
- Documented the schema in
  [phase-199-app-template-manifest-schema.md](../../reference/phase-199-app-template-manifest-schema.md).
- Validation rejects empty identity, stale schema versions, undeclared provider profile references,
  unsafe paths, unknown generated-check files, and unsupported template syntax before promotion.
- Focused verification:
  `cargo test -p ash-cli --test phase199_template_manifest -- --nocapture`.
