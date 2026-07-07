# TASK-1947: Template Instantiation CLI

**Status:** Planned
**Phase:** [PLAN-199: Productive App Libraries And Templates](../PLAN-199-PRODUCTIVE-APP-LIBRARIES-AND-TEMPLATES.md)

## Description

Add a CLI/template instantiation path with structured fail-closed diagnostics.

## Requirements

- Instantiate validated templates into target project or fixture layouts.
- Support parameter substitution only through typed, validated template parameters.
- Refuse to overwrite files unless an explicit safe overwrite mode is specified.
- Run template conformance checks after instantiation.

## TDD Steps

1. Add failing CLI/template instantiation tests.
2. Implement minimal instantiation and diagnostics.
3. Add overwrite, parameter, and conformance failure tests.
4. Run focused CLI/template tests and Rust quality gates.

## Completion Checklist

- [ ] Template CLI path instantiates validated templates.
- [ ] Parameters are typed and validated.
- [ ] Existing files are protected by default.
- [ ] Post-instantiation checks run and report structured diagnostics.
