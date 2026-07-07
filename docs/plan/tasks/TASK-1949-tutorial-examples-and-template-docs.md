# TASK-1949: Tutorial Examples And Template Docs

**Status:** Complete
**Phase:** [PLAN-199: Productive App Libraries And Templates](../PLAN-199-PRODUCTIVE-APP-LIBRARIES-AND-TEMPLATES.md)

## Description

Add tutorial-quality examples and docs for productive app libraries and templates, tied to executable
or artifact gates.

## Requirements

- Document how to use standard providers/profiles, testing helpers, process/channel helpers, and
  templates.
- Keep historical syntax out of productive tutorial paths.
- Link tutorial examples to template conformance or CLI/engine gates.
- Update example indexes/readmes where needed.

## TDD Steps

1. Add docs/example gate checks for tutorial examples.
2. Write current-syntax tutorial examples.
3. Run docs/example gates.
4. Record stale-syntax sweep evidence.

## Completion Checklist

- [x] Tutorial examples use current target syntax.
- [x] Docs explain provider/profile and template use without implying ambient authority.
- [x] Examples are tied to executable or artifact gates.
- [x] Historical examples remain clearly labeled if retained.

## Evidence

- Added [phase199-productive-apps.md](../../tutorials/phase199-productive-apps.md), a productive app
  tutorial that links the canonical template index, testing helper example, process/channel helper
  example, manifest schema, and focused gate names.
- Added `phase199_tutorial_docs`, a docs gate that requires links to executable/artifact gates and
  rejects stale productive tutorial patterns.
- Linked the Phase 199 tutorial from `docs/TUTORIAL.md` while preserving older historical/tutorial
  sketches as reference-oriented material.
- Focused verification:
  `cargo test -p ash-cli --test phase199_tutorial_docs -- --nocapture`.
