# TASK-1949: Tutorial Examples And Template Docs

**Status:** Planned
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

- [ ] Tutorial examples use current target syntax.
- [ ] Docs explain provider/profile and template use without implying ambient authority.
- [ ] Examples are tied to executable or artifact gates.
- [ ] Historical examples remain clearly labeled if retained.
