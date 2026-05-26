# TASK-957: Pure callable type parser

## Status: 📝 Planned

## Description

Implement preferred pure callable type parsing for `(A, B) -> C` while preserving legacy `Fn(A, B) -> C` compatibility.

## Specification Reference

- SPEC-072 §5
- SPEC-072 C72-1 through C72-3

## Dependencies

- ✅ TASK-955: Tower callable syntax packet
- ✅ TASK-956: Callable syntax audit gate

## Requirements

### Functional Requirements

1. Add RED parser tests for `(Int, Int) -> Int`, `Int -> Int`, legacy `Fn(Int, Int) -> Int`, and tuple-argument disambiguation.
2. Implement parser support with maximal-munch-safe arrow handling.
3. Preserve spans and source fidelity for diagnostics.

### Parser-Specific Requirements

1. Implement callable-domain parsing as a separate syntactic path from tuple-type parsing. `(A, B) -> C` must lower directly to a two-argument callable domain, not to a unary tuple argument.
2. Cover both parser paths: `parse_module.rs` source type annotations and `parse_type_def.rs` type-expression/alias parsing.
3. Audit whether `parse_type_def::TypeExpr` needs a dedicated function/callable carrier instead of synthetic `Constructor { name: "Fn" }` to preserve domain arity.
4. Add focused tests for `(Int, String) -> Bool`, `Int -> Bool`, legacy `Fn(Int, String) -> Bool`, and the chosen unary tuple-argument spelling or diagnostic.

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
  - |
    python3 - <<'PY'
    from pathlib import Path
    path = Path('crates/ash-parser/tests/task_957_callable_type_parser.rs')
    text = path.read_text()
    required = [
        'module_annotation_parses_parenthesized_n_ary_callable_domain',
        'type_alias_parses_parenthesized_n_ary_callable_domain',
        'legacy_fn_syntax_remains_compatible',
        'tuple_domain_is_not_silently_lowered_as_unary_argument',
        'unary_tuple_argument_spelling_is_explicit_or_diagnostic',
    ]
    missing = [name for name in required if name not in text]
    assert not missing, f'{path} missing tests: {missing}'
    PY
  - cargo test -p ash-parser --test task_957_callable_type_parser -- --nocapture
checklist:
  - [ ] Focused tests pass.
  - [ ] Formatting clean.
  - [ ] Clippy clean if Rust touched.
  - [ ] CHANGELOG.md updated for implementation or docs changes.
```

## Dependencies for Next Task

This task contributes to PLAN-121 and SPEC-072 completion.

## Notes

Area: parser. Keep the callable-stratum axis separate from return type. Pure smart constructors returning `Act<T>`, `Proc<T>`, or `Workflow<T>` remain pure callables.
