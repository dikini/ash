# Ash Specification Index

This directory contains the canonical specifications for the Ash workflow language.

## Active Specifications

| Spec | Title | Status | Description |
|------|-------|--------|-------------|
| SPEC-001 | Intermediate Representation | Active | Core AST types, serialization, and IR semantics |
| SPEC-002 | Surface Language | Active | Surface syntax and parsing |
| SPEC-003 | Type System | Active | Type checking, inference, and constraint solving |
| SPEC-004 | Operational Semantics | Active | Operational semantics and evaluation rules |
| SPEC-005 | Ash CLI Specification | Active | Command-line interface and commands |
| SPEC-006 | Policy Definition Syntax | Active | Policy definition and structure |
| SPEC-007 | Policy Combinators | Active | Policy combination operators |
| SPEC-008 | Dynamic Policies | Active | Runtime policy modification |
| SPEC-009 | Module System | Active | Namespaces, imports, and module resolution |
| SPEC-010 | Embedding | Active | Embedding Ash in host applications |
| SPEC-011 | REPL | Active | Interactive REPL semantics |
| SPEC-012 | Imports | Active | Import system and resolution |
| SPEC-013 | Streams | Active | Stream processing and semantics |
| SPEC-014 | Behaviours | Active | Behaviour definitions and contracts |
| SPEC-015 | Typed Providers | Active | Capability providers with types |
| SPEC-016 | Output | Active | Output formatting and destinations |
| SPEC-017 | Capability Integration | Active | Capability integration with system features |
| SPEC-018 | Capability Matrix | Active | Capability permission matrix |
| SPEC-019 | Role Runtime Semantics | Active | Role-based execution semantics |
| SPEC-020 | Algebraic Data Types | Active | Sum types, product types, and pattern matching |
| SPEC-021 | Lean Reference | Active | Reference Lean formalization |
| SPEC-021 | Runtime Observable Behavior | Active | Runtime behavior observation |
| SPEC-022 | Workflow Typing with Constraints | Active | Contracts, obligations, and linear resource tracking |
| SPEC-023 | Proxy Workflows | Active | Proxy workflow patterns and semantics |

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
