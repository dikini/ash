# TASK-1955: LSP Current-Syntax Migration Polish

**Status:** Planned
**Phase:** [PLAN-200: Tooling And Migration Polish](../PLAN-200-TOOLING-AND-MIGRATION-POLISH.md)

## Description

Polish LSP behavior for current syntax and surface migration diagnostics for retained old forms.

## Requirements

- Verify LSP diagnostics match CLI diagnostics for selected current and deprecated syntax cases.
- Improve hover, document symbols, workspace symbols, semantic tokens, goto definition, and
  references for current syntax used by Phase 199 examples/templates.
- Ensure old forms are reported as migration diagnostics where retained.
- Prevent compatibility syntax from polluting current-syntax LSP paths.

## TDD Steps

1. Add failing LSP integration tests for current-syntax examples and deprecated-form diagnostics.
2. Verify the LSP misses current symbols/navigation or emits generic old-form diagnostics.
3. Implement LSP polish through parser/typechecker-backed data.
4. Re-run LSP and CLI diagnostic parity tests.

## Completion Checklist

- [ ] LSP diagnostics align with CLI diagnostics for selected cases.
- [ ] Current-syntax hover/symbol/navigation cases pass.
- [ ] Deprecated forms surface migration diagnostics.
- [ ] Compatibility paths are isolated from current-syntax happy paths.
