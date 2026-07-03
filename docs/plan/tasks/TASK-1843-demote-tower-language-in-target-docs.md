# TASK-1843: Demote tower language in target docs

## Description

Review target docs for wording that could make `Act`, `Proc`, or `Workflow` look like target semantic foundations.

## Requirements

- Keep `Act`, `Proc`, and `Workflow` as profiles/library/runtime concepts where useful.
- Ensure target docs say Core/direct-style computation is the semantic path.
- Avoid compatibility language unless explicitly marked historical/current-state.

## Completion criteria

- [x] Target docs use one Core computation model wording.
- [x] Remaining tower references are clearly profiles/library/runtime concepts.

## Evidence

- Updated `SPEC-095b` and `SPEC-098c` to demote explicit `do:Act`, `do:Proc`, and `do:Workflow` to compatibility/profile forms.
- Updated `NOTE-019` to state one checked direct-style computation model and rows as requirement metadata.
- Remaining `Act`, `Proc`, and `Workflow` references in touched target docs are compatibility/profile/library/runtime context, not target semantic foundations.

## Depends on

- TASK-1839.
