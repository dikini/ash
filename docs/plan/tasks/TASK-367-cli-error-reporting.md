# TASK-367: CLI Error Messages for Entry Point Failures

## Status: ⛔ Blocked

## Description

Implement rich error messages for CLI entry point failures, anchored to SPEC-005/SPEC-021.

**VALIDATION GATE - REQUIRED BEFORE IMPLEMENTATION:**

1. **Verify S57-2 (CLI spec)**: ✅ Complete - confirms error message format
2. **Verify S57-3 (observable)**: ✅ Complete - confirms what's user-visible
3. **Anchor to SPEC**: Error format decisions reference SPEC, not MCE

## Error Cases

| Error | Spec Source | Message Format |
|-------|-------------|----------------|
| File not found | SPEC-005 | `error: file not found: {path}` |
| No main workflow | SPEC-003/022 (S57-6) | `error: entry file has no 'main' workflow` |
| Wrong return type | SPEC-003/022 (S57-6) | `error: 'main' has wrong return type\n  expected: Result<(), RuntimeError>\n  found: {X}` |
| Non-capability param | SPEC-017 (S57-5) | `error: parameter '{name}' must be capability type` |
| Parse error | SPEC-002 | Existing parser error format |

## Implementation

```rust
pub fn report_entry_error(error: EntryError) {
    match error {
        EntryError::FileNotFound(path) => {
            eprintln!("error: file not found: {}", path.display());
        }
        EntryError::NoMainWorkflow { file, available } => {
            eprintln!("error: entry file has no 'main' workflow");
            if let Some(wfs) = available {
                eprintln!("  available workflows: {}", wfs.join(", "));
            }
        }
        EntryError::WrongReturnType { expected, found, span } => {
            eprintln!("error: 'main' has wrong return type");
            if let Some(s) = span {
                eprintln!("  --> {}:{}:{}", s.file, s.line, s.column);
            }
            eprintln!("  expected: {}", expected);
            eprintln!("  found: {}", found);
        }
        EntryError::NonCapabilityParam { name, typ, span } => {
            eprintln!("error: parameter '{}' must be capability type", name);
            if let Some(s) = span {
                eprintln!("  --> {}:{}:{}", s.file, s.line, s.column);
            }
            eprintln!("  '{}' has type '{}'", name, typ);
        }
    }
}
```

## Format Guidelines (per SPEC-005)

- Start with `error:`
- Include source location when available
- Show expected vs found for type errors
- Suggest fixes when obvious

## TDD Steps

### Test 1: File Not Found
```rust
let output = run_ash(["run", "missing.ash"]);
assert!(output.stderr.contains("file not found: missing.ash"));
assert_eq!(output.status.code(), Some(1));
```

### Test 2: Missing Main Shows Available
```rust
let entry = "workflow other() {} workflow another() {}";
let output = run_ash_with_entry(entry);
assert!(output.stderr.contains("no 'main' workflow"));
assert!(output.stderr.contains("other") || output.stderr.contains("another"));
```

### Test 3: Type Error Shows Location
```rust
let entry = "workflow main() -> Int { 42 }";
let output = run_ash_with_entry(entry);
assert!(output.stderr.contains("expected: Result<(), RuntimeError>"));
assert!(output.stderr.contains("found: Int"));
```

## Dependencies

- S57-2: CLI error format
- S57-5, S57-6: Specific error conditions
- Existing error reporting infrastructure

## Spec Citations

| Error Source | Spec |
|--------------|------|
| CLI format | SPEC-005 after S57-2 |
| Entry contract | SPEC-003/022 after S57-6 |
| Capabilities | SPEC-017 after S57-5 |
| Parse errors | SPEC-002 |

## Acceptance Criteria

- [ ] S57-2, S57-5, S57-6 show ✅ Complete (VALIDATION GATE)
- [ ] All error cases have clear messages
- [ ] Source locations shown when available
- [ ] Exit code 1 for all errors
- [ ] Tests verify message content
- [ ] Anchored to SPEC, not MCE

## Est. Hours: 2-3
