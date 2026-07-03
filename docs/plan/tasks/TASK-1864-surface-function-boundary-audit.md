# TASK-1864: Surface Function Boundary Audit

**Status:** Complete
**Plan:** [PLAN-185](../PLAN-185-SURFACE-FUNCTION-LANGUAGE.md)

## Description

Audit the current implementation boundary for surface `fn` declarations, explicit rows, target `do { ... }`, ordinary expression bodies, and entry/runtime workflow compatibility.

## Requirements

- Identify which requested surface-language features are already implemented.
- Identify the implementation gap for `fn main` as a target entry source without privileged workflow syntax.
- Record evidence in this task file before closeout.
- Avoid treating workflow compatibility as the target semantic model.

## TDD Steps

1. Inspect parser/typechecker/engine paths and existing tests.
2. Write failing coverage for the selected gap in TASK-1865.

## Completion Checklist

- [x] Existing support recorded.
- [x] Selected implementation gap recorded.
- [x] Evidence references added.

## Evidence

- Existing support: parser/typechecker already accepted explicit callable rows, `where row`, ambient `do { ... }`, match expressions, named record constructors, ADT constructors, and function body calls.
- Gap selected for TASK-1865: engine parsing required at least one `workflow` definition even when source contained ordinary top-level functions with `fn main`.
- Additional gap selected for TASK-1866: local type definitions from function-entry modules were not registered with full local representation for checking signatures, ADT patterns, and nominal record field access.
