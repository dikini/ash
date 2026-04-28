# TASK-752: Do-Notation Diagnostics

## Status: 📝 Planned

## Description

Implement targeted diagnostics and migration warnings for generalized do-notation. This task turns type/parser failures into concise teaching-oriented messages suitable for humans and AI agents.

## Specification Reference

- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) §13
- [SPEC-040](../../spec/SPEC-040-DIAGNOSTIC-INFRASTRUCTURE.md), if applicable to current diagnostic infrastructure

## Dependencies

- 📝 TASK-749: typed do elaboration.
- 📝 TASK-750: Act migration.
- 📝 TASK-751: Proc tower integration.

## Requirements

Diagnostics must cover:

1. unknown do target;
2. target wrong kind;
3. missing dictionary / unsupported target;
4. pure RHS used with `<-`;
5. wrong computation constructor in `<-`;
6. monadic value bound with `let`;
7. missing final `return`;
8. `return` before block end;
9. trailing semicolon after final `return`;
10. deprecated `ret`;
11. legacy `x = effectful_expr;` in `act {}`;
12. Act-to-Proc mismatch with `proc::from_act` hint.

## TDD Steps

### Step 1: Add golden/substring diagnostic tests

**Files:**

- Modify diagnostic tests in `crates/ash-parser`, `crates/ash-typeck`, and/or `crates/ash-cli` depending on existing patterns.

For each family, assert that diagnostics include:

- expected shape;
- found shape;
- one likely fix.

### Step 2: Implement parser diagnostics

Parser-owned examples:

- malformed `do:` target;
- trailing semicolon after new-form final `return`;
- legacy `ret` in new-form do block if parsed there.

### Step 3: Implement type diagnostics

Typechecker-owned examples:

- wrong kind;
- unsupported target;
- wrong `<-` RHS constructor;
- pure RHS with `<-`;
- `let` monadic warning.

### Step 4: Verify through CLI where possible

Run focused CLI/check tests if the existing CLI exposes parser/type diagnostics for snippets.

## Verification Steps

- [ ] Every SPEC-054 §13 diagnostic family has a test.
- [ ] Hints do not mention unavailable syntax except where explicitly future/deferred.
- [ ] Diagnostics preserve source spans.
- [ ] `cargo test --workspace do_diagnostic -- --nocapture` or equivalent passes.
- [ ] Independent review checks diagnostic wording for accuracy and no overclaiming.

## Dependencies for Next Task

Required by:

- TASK-753 closeout.
