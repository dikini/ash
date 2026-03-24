# Ash Specification Index

This directory contains the canonical specifications for the Ash workflow language.

## Active Specifications

| Spec | Title | Status | Description |
|------|-------|--------|-------------|
| SPEC-001 | Intermediate Representation | Active | Core AST types, serialization, and IR semantics |
| SPEC-002 | Capability System | Active | Capability definition, effects, and authority model |
| SPEC-003 | Type System | Active | Type checking, inference, and constraint solving |
| SPEC-004 | Policy Framework | Active | Policy definition, evaluation, and decision semantics |
| SPEC-005 | Provenance and Audit | Active | Audit trails, provenance tracking, and lineage |
| SPEC-006 | Workflow Execution | Active | Runtime semantics and execution model |
| SPEC-007 | Module System | Active | Namespaces, imports, and module resolution |
| SPEC-020 | Generics and Type Parameters | Active | Generic types, constraints, and polymorphism |
| SPEC-021 | Algebraic Data Types | Active | Sum types, product types, and pattern matching |
| SPEC-022 | Workflow Typing with Constraints | Active | Contracts, obligations, and linear resource tracking |

## Deprecated Specifications

None currently.

## Specification Template

New specifications should follow this structure:

```markdown
# SPEC-XXX: Title

**Status:** Active | Draft | Deprecated  
**Supersedes:** SPEC-YYY (if applicable)  
**Related:** SPEC-ZZZ, SPEC-WWW

## Summary

Brief description of what this specification defines.

## Motivation

Why this specification exists and what problems it solves.

## Specification

### Section 1

Detailed technical content.

## Implementation Tasks

- TASK-###: Description

## Changelog

### YYYY-MM-DD

- Initial version
```

## Cross-Reference Guide

When writing specifications, use these cross-reference formats:

- Type system rules: `See [SPEC-003-TYPE-SYSTEM](SPEC-003-TYPE-SYSTEM.md)`
- IR constructs: `See [SPEC-001-IR](SPEC-001-IR.md)`
- Workflow contracts: `See [SPEC-022-WORKFLOW-TYPING](SPEC-022-WORKFLOW-TYPING.md)`

## Review Process

1. Specifications start as drafts in `todo-examples/definitions/`
2. Once implemented and tested, moved to `docs/spec/`
3. Marked as Active when complete
4. Updated via patch commits with changelog entries
