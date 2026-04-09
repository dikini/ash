# TASK-484: Close Out Phase 71 After Module-Scoped Resolution Lands

## Status: Planned

## Description

Once the module-scoped shared-context contract is implemented, update the docs/status surfaces and
run the verification bar needed to close Phase 71 honestly.

## Specification Reference

- [PLAN-017](../PLAN-017-MODULE-OWNED-CAPABILITY-RESOLUTION.md)
- [PLAN-018](../PLAN-018-MODULE-SCOPED-CAPABILITY-RESOLUTION-CLOSURE.md)
- [SPEC-002](../../spec/SPEC-002-SURFACE.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)
- [SPEC-017](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-481](TASK-481-thread-module-id-through-lowering.md)
- ✅ [TASK-482](TASK-482-thread-module-id-through-typeck.md)
- ✅ [TASK-483](TASK-483-remove-typeck-fallback-resolver.md)

## Requirements

1. Update Phase 71 status only after the architectural gap is actually closed.
2. Update `CHANGELOG.md` and `PLAN-INDEX.md` with honest outcomes.
3. Run verification for:
   - `cargo fmt --check`
   - `cargo check --workspace --all-targets`
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - `cargo test -p ash-parser --lib`
   - `cargo test -p ash-typeck --lib`
4. Report any unrelated residual failures explicitly instead of hiding them under completion claims.

## TDD Steps

### Red

- Identify any remaining doc/status text that still treats the gap as open after the code lands.

### Green

- Docs/status and verification outputs match the actual final state.

## Completion Checklist

- [ ] Phase 71 status updated honestly
- [ ] `CHANGELOG.md` updated
- [ ] verification commands run
- [ ] residual unrelated failures called out explicitly if they remain
