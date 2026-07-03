# TASK-1873: CLI Entry Spec Reconciliation

**Status:** Complete
**Plan:** [PLAN-186](../PLAN-186-SURFACE-FUNCTION-CLI-ENTRY.md)

## Description

Reconcile target specs and orientation indexes after CLI dry-run accepts function-first entry files.

## Requirements

- Update target grammar/lowering notes only if they need CLI-facing clarification.
- Update `SPEC-INDEX.md` and `NOTE-INDEX.md` read paths so surface-function CLI entry work is discoverable.
- Avoid describing workflow syntax as the target core entry language.

## TDD Steps

Documentation task; validation uses docs gates.

## Completion Checklist

- [x] Relevant specs/indexes updated.
- [x] Docs gate passes.
- [x] Task evidence records validation.

## Evidence

- Updated `SPEC-095b` and `SPEC-098c` current implementation notes for CLI dry-run support of function-first entry files.
- Updated `SPEC-INDEX.md` and `NOTE-INDEX.md` read paths to include PLAN-186.
- Verification: `python3 tools/docs/validate_orientation_indexes.py --self-test` and `bash scripts/check-docs-gate.sh` passed.
