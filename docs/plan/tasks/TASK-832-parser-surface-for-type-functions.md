# TASK-832: Add parser raw surface syntax and span-preserving AST for module-level type functions

## Status: ✅ Complete

## Description

Add parser raw surface syntax and span-preserving AST for module-level type functions.

## Specification Reference

- [SPEC-061](../../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.5
- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)

## Dependencies

- ✅ Phase 112 / SPEC-060 complete.
- Depends on TASK-831 audit gate completion.

## Dispatch

```
agent: hermes
provider: openai-codex
model: gpt-5.5
profile: default
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Objective

Add parser raw surface syntax and span-preserving AST for module-level type functions.

## Requirements

1. Add surface TypeFn definition carriers with spans.
2. Add `Definition::TypeFn`.
3. Parse type fn headers, rejected visibility prefixes, decreases clauses, case equations, constructor/variable/wildcard patterns, and RHS type expressions.
4. Add a `type fn` dispatch check before ordinary `type` parsing so `starts_with_type_definition` does not consume `type fn` as an ordinary type definition.
5. Reject malformed case heads, missing semicolons, zero-parameter declarations, and visibility-prefixed `type fn` with focused parser diagnostics or parser-stage handoff errors assigned by TASK-831.
6. Preserve accurate spans for header, decreases, case head, patterns, and RHS.
7. Keep parser output raw; no semantic constructor/domain resolution in parser.
8. State and test that SPEC-E `type fn` is file/module top-level only; inline-module parsing must reject or fence it consistently with sealed-domain/type-summary boundaries.
9. Add focused parser acceptance/rejection tests.

## Files

- Modify/create exact files identified by [PLAN-109](../PLAN-109-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md) and the TASK-831 audit gate.
- Update `CHANGELOG.md` for completed implementation/tooling/docs-policy changes.

## TDD Steps

1. Write focused failing tests or docs/audit checks appropriate to task type.
2. Run the focused target and verify the expected failure or missing evidence.
3. Implement the minimal change for this task only.
4. Re-run the focused target and relevant non-regression tests.
5. Update docs/status evidence only after verification.

## Verification

```
strictness: clean
commands:
  - cargo test -p ash-parser --test task_832_type_function_parser -- --nocapture
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Add surface TypeFn definition carriers with spans.
  - [x] Add `Definition::TypeFn`.
  - [x] Parse type fn headers, rejected visibility prefixes, decreases clauses, case equations, constructor/variable/wildcard patterns, and RHS type expressions.
  - [x] Add a `type fn` dispatch check before ordinary `type` parsing so `starts_with_type_definition` does not consume `type fn` as an ordinary type definition.
  - [x] Reject malformed case heads, missing semicolons, zero-parameter declarations, and visibility-prefixed `type fn` with focused parser diagnostics or parser-stage handoff errors assigned by TASK-831.
  - [x] Preserve accurate spans for header, decreases, case head, patterns, and RHS.
  - [x] Keep parser output raw; no semantic constructor/domain resolution in parser.
  - [x] State and test that SPEC-E `type fn` is file/module top-level only; inline-module parsing must reject or fence it consistently with sealed-domain/type-summary boundaries.
  - [x] Add focused parser acceptance/rejection tests.
  - [x] focused tests/evidence recorded in this task file
  - [x] no SPEC-F/G/H scope creep
```

## Verification Evidence

- Added `crates/ash-parser/tests/task_832_type_function_parser.rs` using TDD; initial run failed because `Definition::TypeFn` and `TypePattern` were absent.
- Verified focused parser suite: `cargo test -p ash-parser --test task_832_type_function_parser -- --nocapture` (6 passed).
- Verified sealed-domain parser non-regression: `cargo test -p ash-parser --test task_808_sealed_domain_surface -- --nocapture` (13 passed).
- Verified workspace compile after AST exhaustiveness fixes: `cargo check --workspace`.
- Verified formatting: `cargo fmt --check`.
- Verified whitespace: `git diff --check`.
- Scope note: parser output remains raw `surface::Type`/`TypePattern` syntax only; no lowering, typeck, core carriers, semantic resolution, normalizer integration, or SPEC-F/G/H public/associated/proposition scope was implemented.
```


## Notes

Task type: Parser/Substrate. Estimated effort: 6 hours. Keep the slice within SPEC-061 / Phase 113 boundaries.
