# TASK-180: Formalize Policy Evaluation and Verification Semantics

## Status: ✅ Complete

## Description

Tighten the semantics of named policy bindings, normalized policy lowering, workflow `decide`,
capability verification outcomes, and policy rejection ownership.

## Specification Reference

- SPEC-003: Type System
- SPEC-004: Operational Semantics
- SPEC-006: Policy Definitions
- SPEC-007: Policy Combinators
- SPEC-008: Dynamic Policies
- SPEC-017: Capability Integration
- SPEC-018: Capability Matrix

## Requirements

### Functional Requirements

1. Define one end-to-end policy story from named binding to runtime decision
2. Separate workflow `decide` outcomes from capability-verification outcomes
3. Define which policy failures are lowering/type/runtime failures
4. Make the policy contract proof-shaped and implementation-shaped

## TDD Evidence

### Red

Before this change, the policy docs still left these boundaries open to interpretation:

- workflow `decide` and capability verification shared an ambiguous outcome taxonomy,
- `Warn` appeared in the canonical policy taxonomy without a clear role in the policy story,
- named binding, closure, lowering identity, and runtime verification boundaries were not split
  explicitly enough for mechanical implementation.

### Green

The canonical policy story is now explicit:

- policy definitions lower through closed named bindings into canonical `CorePolicy` identities,
- workflow `decide` consumes only `Permit` / `Deny`,
- capability verification may consume `{Permit, Deny, RequireApproval, Transform}`,
- `Warn` is verification metadata, not a policy decision,
- parser, lowering, type, and runtime/verification failures are owned by distinct phases.

## Files

- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-006-POLICY-DEFINITIONS.md`
- Modify: `docs/spec/SPEC-007-POLICY-COMBINATORS.md`
- Modify: `docs/spec/SPEC-008-DYNAMIC-POLICIES.md`
- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Modify: `docs/spec/SPEC-018-CAPABILITY-MATRIX.md`
- Modify: `docs/reference/type-to-runtime-contract.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- ambiguous outcome domains,
- ambiguous failure ownership,
- ambiguous named-binding semantics.

### Step 2: Verify RED

Expected failure conditions:
- at least one workflow-vs-capability policy boundary still requires interpretation.

### Step 3: Implement the minimal spec fix (Green)

Tighten only policy evaluation and verification semantics.

### Step 4: Verify GREEN

Expected pass conditions:
- one explicit policy story exists from source binding to runtime outcome.

### Step 5: Commit

```bash
git add docs/spec/SPEC-003-TYPE-SYSTEM.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-006-POLICY-DEFINITIONS.md docs/spec/SPEC-007-POLICY-COMBINATORS.md docs/spec/SPEC-008-DYNAMIC-POLICIES.md docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md docs/spec/SPEC-018-CAPABILITY-MATRIX.md docs/reference/type-to-runtime-contract.md CHANGELOG.md
git commit -m "docs: formalize policy evaluation and verification semantics"
```

## Completion Checklist

- [x] policy outcome domains documented
- [x] policy failure ownership documented
- [x] named policy semantics documented
- [x] `CHANGELOG.md` updated

## Non-goals

- No SMT implementation changes
- No runtime code changes

## Dependencies

- Depends on: TASK-177, TASK-178
- Blocks: TASK-165, TASK-166, TASK-168, TASK-169, TASK-171, TASK-184
