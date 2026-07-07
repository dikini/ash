# TASK-1943: Current-Syntax Library/Template Audit Remediation

**Status:** Planned
**Phase:** [PLAN-199: Productive App Libraries And Templates](../PLAN-199-PRODUCTIVE-APP-LIBRARIES-AND-TEMPLATES.md)

## Description

Review and revise productive stdlib modules, examples, and template-like files to current target
syntax before building new app templates.

## Requirements

- Audit `std/src`, `examples`, `tests/std`, and template-like workflow/example assets.
- Classify files as current executable, current reference, historical/reference-only, or removed
  from productive paths.
- Revise productive libraries and examples to current syntax where required.
- Add parse/check/run or artifact assertions for files promoted to productive paths.

## TDD Steps

1. Add inventory checks or focused CLI/engine tests for productive library/example candidates.
2. Confirm stale syntax is detected.
3. Revise selected files to current syntax.
4. Re-run checks and record classification evidence.

## Completion Checklist

- [ ] Productive library/example/template candidates are inventoried.
- [ ] Historical/reference-only files are explicitly excluded from productive paths.
- [ ] Required libraries are revised to current syntax.
- [ ] Promoted productive files have executable or artifact gates.
