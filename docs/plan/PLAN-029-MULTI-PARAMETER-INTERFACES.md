# PLAN-029: Multi-Parameter Interface Methods

Remove the single-parameter restriction on interface method signatures and their call sites. Interface methods may declare any number of parameters, and call sites may pass any number of arguments. This phase covers SPEC-032 only; interface declarations remain limited to a single type parameter.

**Spec:** SPEC-032
**Phase:** 82
**Priority:** High
**Status:** 📝 Planned

## Overview

The closed-world interface MVP hardcodes interface methods to exactly one parameter. This plan removes that restriction and routes all interface calls through the standard `Expr::Call` AST node, eliminating the deprecated `Expr::InterfaceMethodCall` special case.

> **Scope note:** `InterfaceMethodCall` removal is the highest-risk step. It touches 9+ files across parser, type checker (`lib.rs`, `purity.rs`, `names.rs`, `capability_check.rs`), and interpreter (`eval.rs`).

## Tasks

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-561](tasks/TASK-561-parser-multi-param-methods.md) | Parser/AST: multi-parameter method signatures and impl definitions | SPEC-032 §4 | 4 | 📝 Planned |
| [TASK-562](tasks/TASK-562-typeck-multi-param-calls.md) | Type checker/Interpreter: multi-parameter interface call resolution | SPEC-032 §5-6 | 5 | 📝 Planned |

**Total Estimate:** 9 hours

## Deliverable

- Interface methods accept any number of parameters.
- `InterfaceMethodCall` AST node is removed; all interface calls route through `Expr::Call`.
- All existing single-parameter interface tests continue to pass.
- Interface declarations still limited to one type parameter (relaxed in Phase 83).
