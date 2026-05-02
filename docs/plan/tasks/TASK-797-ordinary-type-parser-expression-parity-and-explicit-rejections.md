# TASK-797: Ordinary Type Parser Expression Parity and Explicit Rejections

## Status: 📝 Planned

## Description

Bring both ordinary type parsing paths to the Phase 110 parity boundary so `parse_type_def.rs`, `parse_module.rs::parse_surface_type`, and `parse_module.rs::convert_type_expr` either accept the current supported associated-projection subset or reject deferred syntax explicitly. This task is the single owner of parser rejection-boundary evidence for Phase 110.

## Specification Reference

- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [PLAN-106](../PLAN-106-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## Dependencies

- ✅ [TASK-794](TASK-794-type-expression-ir-and-kinding-audit-gate.md)

## Objective

Align parser-private ordinary type-expression parsing with the current Phase 110 surface contract across both parser paths without adding speculative syntax.

## Requirements

1. Audit drift between `parse_type_def.rs::TypeExpr`, `parse_module.rs::parse_surface_type`, and `parse_module.rs::convert_type_expr` against the current SPEC-035 and SPEC-058 subset.
2. Add parser support only for the supported subset required by Phase 110, across both ordinary type-definition parsing and surface/module type parsing.
3. Add and own parser rejection tests for deferred holes, partial type-constructor application, and alternative projection spellings if those paths are encountered; later Phase 110 tasks may rerun these tests but must not create a second parser-evidence owner.
4. Do not add unsupported-shape diagnostics for syntactically admitted `base::Assoc` forms such as `(S::Item)::Assoc`; those belong to TASK-800 after canonical lowering.
5. Do not add public `type fn`, kind binder syntax, or sealed-domain syntax.

## Files

- Modify: `crates/ash-parser/src/parse_type_def.rs`
- Modify: `crates/ash-parser/src/parse_module.rs`
- Modify if necessary: `crates/ash-parser/src/surface.rs`
- Add focused parser tests in `ash-parser`

## TDD Steps

1. Write failing parser tests for the supported associated-projection subset inside ordinary type definitions.
2. Write failing parser tests for the explicit rejection boundaries touched by this task, and organize/name the suite so TASK-804 can cite it as the carried-forward parser rejection evidence for Phase 110.
3. Implement the minimal parser changes to satisfy those tests.
4. Re-run focused parser tests.

## Verification Steps

- [ ] `cargo test -p ash-parser` for the new parser tests
- [ ] `cargo fmt --check`
- [ ] `git diff --check`

## Notes

Single-crate task. Keep public syntax expansion strictly out of scope. Parser rejection-boundary evidence for Phase 110 belongs here; TASK-803 and TASK-804 may only rerun or cite it.
