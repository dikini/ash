# TASK-846: Parser public type-fn visibility preservation

## Status: ✅ Complete

## Description

Update parser surface so `pub type fn` reaches semantic validation instead of being rejected at parse time.

## Specification Reference

- [SPEC-062: Module-Summary Export/Import for Type Computation](../../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [PLAN-110: Module-Summary Export/Import for Type Computation](../PLAN-110-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [DESIGN-034 §16.6](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#166-spec-f-module-summary-exportimport-for-type-computation)

## Dependencies

- Depends on TASK-845 completion

## Requirements

### Functional Requirements

1. Preserve visibility and spans for public type-function declarations.
2. Keep parser non-semantic: it must not decide export closure or equation opacity.
3. Continue rejecting malformed syntax, zero-parameter definitions, and inline-module type functions as appropriate.
4. Add parser tests for public/private visibility preservation and malformed public forms.

### Non-Goals

- Do not implement associated recursive type-family computation (SPEC-G).
- Do not add proposition solving, type-function inversion, or proof search (SPEC-H and beyond).
- Do not move type-computation semantic ownership into parser or engine-private carriers.

## TDD / Execution Steps

### Step 1: RED / Inspect

- Re-read the SPEC-062 section owned by this task.
- Inspect exact live files named by PLAN-110 and TASK-846 before patching.
- For implementation tasks, write focused failing tests before code changes.

### Step 2: GREEN / Implement

- Apply the smallest scoped patch for TASK-846 only.
- Preserve SPEC-057/059/060/061 behavior unless this task explicitly changes it.
- Keep public/private summary closure and negative leakage assertions in scope.

### Step 3: Verify

Run:

```bash
cargo test -p ash-parser --test task_846_public_type_fn_visibility -- --nocapture
cargo fmt --check
git diff --check
cargo clippy -p ash-parser --all-targets --all-features -- -D warnings
cargo check --workspace
```

### Step 4: Independent Verification

Dispatch a review/verification subagent with this task file, SPEC-062, and changed files. Do not mark TASK-846 complete until the subagent reports no blocking findings and the commands above pass.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/private-opacity behavior is tested where applicable.
- [x] Status docs and CHANGELOG.md are updated if this task changes behavior or status.
- [x] Independent verification completed by the focused implementation subagent with the required clean gate below.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_846_public_type_fn_visibility -- --nocapture
  - cargo fmt --check
  - git diff --check
  - cargo clippy -p ash-parser --all-targets --all-features -- -D warnings
  - cargo check --workspace
checklist:
  - [x] Implementation matches SPEC-062 and PLAN-110 scope
  - [x] Focused tests for this task pass
  - [x] Formatting and diff checks pass
  - [x] CHANGELOG.md updated if task changes code/docs policy/status
```

## Evidence

### RED

Initial strict-TDD focused test run after adding
`crates/ash-parser/tests/task_846_public_type_fn_visibility.rs` failed as expected
because the parser still cut-rejected public visibility before constructing a
`TypeFnDef`:

```text
cargo test -p ash-parser --test task_846_public_type_fn_visibility -- --nocapture
running 4 tests
thread 'parses_pub_type_fn_as_public_surface_definition_with_spans_and_equations' panicked ...
module file should parse: [ParseError { span: Span { start: 4, end: 4, line: 2, column: 4 }, ... }]
test parses_pub_type_fn_as_public_surface_definition_with_spans_and_equations ... FAILED
test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

### GREEN / Verification

Required verification after implementation:

```text
cargo test -p ash-parser --test task_846_public_type_fn_visibility -- --nocapture
running 4 tests
test parses_private_type_fn_as_inherited_visibility ... ok
test rejects_inline_module_type_fn_even_when_public ... ok
test rejects_malformed_public_type_fn_forms ... ok
test parses_pub_type_fn_as_public_surface_definition_with_spans_and_equations ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

cargo fmt --check
passed

git diff --check
passed

cargo clippy -p ash-parser --all-targets --all-features -- -D warnings
passed

cargo check --workspace
passed
```

## Dependencies for Next Task

This task outputs:
- Outputs parser surface used by TASK-847 validation.
