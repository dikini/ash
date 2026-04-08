# TASK-442: General Module Resolution And Stdlib-Backed Execution

## Status: ✅ Complete

## Description

Define the implementation slice that makes general stdlib-backed file workflows executable through `ash run`, with real module resolution for user-defined multi-file modules and library roots. This task is the planning record for the resolver-backed file execution work that will later be implemented in `ash-engine` and `ash-cli`.

## Scope

This task covers:

- general stdlib-backed file workflows executable
- module resolution for user-defined multi-file modules
- imports loaded during ordinary file execution
- stdlib functions and types executable from arbitrary workflow files
- arbitrary stdlib imports such as `option`, `prelude`, and `std/lib`
- `ASH_LIBRARY_PATH` root search with precedence `local tree > ASH_LIBRARY_PATH order > built-in stdlib`
- version-qualified library imports like `math@1::vector`
- single concrete version per library name across one loaded graph

Version-qualified library imports are in scope only as a bootstrap-time resolver feature for development and verification. Packaging, installation, dependency manifests, and version solving across an installed package set remain future work.

This task now lands the resolver-backed ordinary-file execution slice in `ash-engine`. Packaging,
installation manifests, and full package-management dependency solving remain future work.

## Specification Reference

- [SPEC-005: CLI Specification](../../spec/SPEC-005-CLI.md)
- [SPEC-009: Modules](../../spec/SPEC-009-MODULES.md)
- [SPEC-010: Embedding](../../spec/SPEC-010-EMBEDDING.md)
- [SPEC-021: Runtime Observable Behavior](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
- [TASK-363a: Runtime Stdlib Loading Integration](TASK-363a-runtime-stdlib-loading.md)
- [TASK-438: Canonical IR Semantics Corpus and Result Format](TASK-438-canonical-ir-semantics-corpus-and-result-format.md)

## Dependencies

- ✅ [TASK-363a: Runtime Stdlib Loading Integration](TASK-363a-runtime-stdlib-loading.md)
- ✅ [TASK-438: Canonical IR Semantics Corpus and Result Format](TASK-438-canonical-ir-semantics-corpus-and-result-format.md)

## Requirements

### Functional Requirements

1. Make `ash run <file>` capable of executing file-backed workflows that import stdlib modules beyond the narrow entry bootstrap set.
2. Make module resolution work for user-defined multi-file modules rooted at the workflow file's directory tree.
3. Make imports load during ordinary file execution, not only during canonical entry bootstrap.
4. Make stdlib functions and types available from arbitrary workflow files.
5. Support `ASH_LIBRARY_PATH` as a PATH-like search list for additional library roots.
6. Allow version-qualified library imports like `math@1::vector` while enforcing one concrete version per library name across a loaded graph.
7. Reject unqualified external-library imports as ambiguous unless future packaging work later defines a manifest-driven way to disambiguate them.
8. Preserve current single-file and canonical entry workflow behavior while extending import support.

### Non-Functional Requirements

1. Keep the resolver semantics explicit and deterministic.
2. Prefer a single loader in `ash-engine` over CLI-only import handling.
3. Keep the task planning-level only; implementation belongs in later code tasks.
4. Use concise repo-relative references.

## TDD Steps

### Red

Before this task, the repository lacks a dedicated planning record for the resolver-backed file-execution slice:

- ordinary file workflows cannot generally import stdlib modules such as `option`, `prelude`, or `std/lib`;
- user-defined multi-file modules are not resolved from the workflow tree;
- import loading is split between narrow entry handling and ad hoc source classification;
- there is no `ASH_LIBRARY_PATH`-based library root search;
- version-qualified library imports are not yet defined as a repository task.

### Green

This task is complete once:

- the module-resolution work is captured as a concrete task slice;
- the resolver scope, library root precedence, and version rule are recorded;
- downstream implementation tasks can follow one plan without re-deriving the intended behavior.

## Files

- Create: `docs/plans/2026-04-08-module-resolution-stdlib-design.md`
- Create: `docs/plans/2026-04-08-module-resolution-stdlib-plan.md`
- Modify: `docs/plan/PLAN-INDEX.md`

## Completion Checklist

- [x] Ordinary file workflows resolve imports through the workflow tree, `ASH_LIBRARY_PATH`, and the built-in stdlib
- [x] Imported stdlib/user `pub type` definitions load during ordinary file execution
- [x] Imported local helper workflows and supported stdlib `pub fn` helpers execute through engine-owned call inlining
- [x] `pub use` re-exports such as `prelude::{is_some}` resolve during ordinary file execution
- [x] Version-qualified library imports remain supported for bootstrap-time development without package management

## Notes

The current callable support remains intentionally narrow and engine-owned: it covers imported local
helper workflows with single `ret` bodies plus the supported stdlib-style `pub fn` / `pub use`
subset needed for ordinary file execution. Packaging and dependency installation remain future work.
