# TASK-1369: Synthetic tests — generate small-world tests from laws

## Status: 📝 Planned

## Description

For each law without a `proof` block, generate small-world tests using SPEC-077 runner framework.

## Requirements

1. Generate test cases from law parameters
2. Use small-world generators for parameter types
3. Assert law proposition for each generated case
4. Report failures with seed and counterexample

## Acceptance Criteria

- [ ] Tests generate for unproven laws
- [ ] Tests pass for valid laws
- [ ] Tests fail for broken laws with counterexample
- [ ] Runner test passes
- [ ] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1368](TASK-1368-runner-law-extraction.md)
- [SPEC-077](../../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
