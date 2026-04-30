# TASK-786: Import, Pub-Use, Glob, Visibility, and Opacity Summary Rules

## Status: 📝 Planned

## References

- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
- [SPEC-057](../../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md)
- [PLAN-105](../PLAN-105-UNIFIED-TYPE-MODULE-PIPELINE-SEMANTIC-SUMMARIES.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)
- [SPEC-020](../../spec/SPEC-020-ADT-TYPES.md)
- [SPEC-030](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md)

## Dependencies

TASK-785.

## Objective

Make ordinary type summary transport coherent across existing import/re-export forms.

## Requirements

1. Implement named import summary transport.
2. Implement glob import summary transport.
3. Implement `pub use` re-export identity preservation, alias semantics, and diagnostics for missing re-export targets.
4. Handle child module summaries without implicit flattening.
5. Enforce public/private/crate visibility and opacity rules without adding new opaque type syntax; opaque exported identities are limited to existing explicit builtin/opaque exceptions.
6. Add import-order independence tests plus constructor-only import and glob-import constructor exposure tests.

## Verification

- [ ] Public type identities import consistently.
- [ ] `pub use` preserves canonical identity.
- [ ] Private type leaks are rejected.
- [ ] Constructor imports obey representation visibility.

## Implementation Notes

- Follow TDD for any code changes.
- Update CHANGELOG.md when the implementation task is completed.
- Run focused tests for the changed crate and broader regressions requested by PLAN-105.
