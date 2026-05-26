# TASK-960: Reserved tower callable arrows

## Status: 📝 Planned

## Description

Reserve Act/Proc/Workflow callable and closure arrows with targeted diagnostics rather than silently accepting or misclassifying them.

## Specification Reference

- SPEC-072 §5.5-§5.7
- SPEC-072 §6.3
- SPEC-072 C72-5 through C72-7

## Dependencies

- ✅ TASK-955: Tower callable syntax packet
- 📝 TASK-956: Callable syntax audit gate
- 📝 TASK-957: Pure callable type parser
- 📝 TASK-959: Pure closure arrow syntax

## Requirements

### Functional Requirements

1. Add RED diagnostics tests for `(A) -*> B`, `(A) => B`, `(A) =*> B` in type contexts.
2. Add RED diagnostics tests for `|x| -*> { ... }`, `|x| => { ... }`, and `|x| =*> { ... }` closure contexts.
3. Implement fail-closed reservation diagnostics and smart-constructor distinction docs/tests.

### Reservation Requirements

1. Recognize reserved arrows with maximal munch before generic parse failure in callable-type and closure-literal contexts.
2. Add reserved diagnostics tests for type alias, function parameter, function return, interface/builtin signature if applicable, and closure contexts.
3. Add negative leakage tests proving match-arm `=>` remains legal where currently supported while closure `=>` is reserved for Proc closures.
4. Ensure reserved arrows do not lower to `Type::Fn`, `Type::Fun`, or pure callables returning `Act`/`Proc`/`Workflow`.

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

Area: parser/typechecker diagnostics. Keep the callable-stratum axis separate from return type. Pure smart constructors returning `Act<T>`, `Proc<T>`, or `Workflow<T>` remain pure callables.
