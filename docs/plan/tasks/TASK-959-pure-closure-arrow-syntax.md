# TASK-959: Pure closure arrow syntax

## Status: 📝 Planned

## Description

Implement pure closure shorthand `|args| -> body` and stop treating `|args| => body` as preferred pure syntax.

## Specification Reference

- SPEC-072 §6
- SPEC-072 C72-4

## Dependencies

- ✅ TASK-955: Tower callable syntax packet
- 📝 TASK-956: Callable syntax audit gate (must locate live closure parser/lowering seams first)
- 📝 TASK-957: Pure callable type parser, if shared arrow token handling lands there

## Requirements

### Functional Requirements

1. Add RED parser/typechecker/interpreter tests for `|x| -> x + 1` and `|x, y| -> x + y` in supported contexts.
2. Add migration or rejection test for old `|x| => x + 1` pure syntax.
3. Implement closure parsing/lowering updates without changing closure capture semantics beyond the spelling.

### Closure-Specific Requirements

1. Implement `|args| -> body` only in closure-literal context.
2. Do not reuse match-arm `=>` parsing for closure arrows.
3. Ensure `|args| -> body` remains Pure-stratum even in higher tower contexts; captures/body must satisfy pure-closure rules.
4. Add focused migration/rejection coverage for old `|args| => body` as pure syntax.

### Non-goals

- Do not implement Act/Proc/Workflow callable runtime semantics unless this task explicitly says so.
- Do not introduce partial application or currying.
- Do not silently reinterpret higher-stratum arrows as pure functions returning computation values.

## Work Steps

1. Inspect the exact live files named by TASK-956 or this task.
2. Write focused RED tests or docs assertions before changing behavior.
3. Implement or document the minimal target behavior.
4. Run focused verification.
5. Update status surfaces and CHANGELOG.md if files beyond tests are changed.
6. Request independent review before marking complete.

## Dispatch

```yaml
agent: codex
reasoning: medium
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - false # TASK-956 audit must replace this with exact focused non-zero verification commands before implementation starts.
checklist:
  - [ ] Focused tests pass.
  - [ ] Formatting clean.
  - [ ] Clippy clean if Rust touched.
  - [ ] CHANGELOG.md updated for implementation or docs changes.
```

## Dependencies for Next Task

This task contributes to PLAN-121 and SPEC-072 completion.

## Notes

Area: parser/typechecker/runtime. Keep the callable-stratum axis separate from return type. Pure smart constructors returning `Act<T>`, `Proc<T>`, or `Workflow<T>` remain pure callables.
