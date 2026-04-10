# TASK-499: Add Stdlib IO Integration Tests and Examples

## Status: Done

## Description

Add end-to-end tests and repository examples that demonstrate the intended Ash style for `io` v1.
This task is the main guard against a stdlib surface that looks plausible in docs but does not work
coherently across parser, typechecker, interpreter, and engine.

## Specification Reference

- [PLAN-022: Stdlib IO V1](../PLAN-022-STDLIB-IO-V1.md)
- [Stdlib `io` V1 Design](../../plans/2026-04-10-stdlib-io-v1-design.md)

## Dependencies

- [TASK-498](TASK-498-stdlib-io-bootstrap-and-runtime-wiring.md)

## Requirements

1. Add parser-facing stdlib tests for the new `io` module family.
2. Add engine/runtime tests covering representative `io` usage with provider setup.
3. Add `.ash` examples under `examples/` and/or `tests/std/` showing the intended user-facing style.
4. Cover at least:
   - pure path building
   - stdio echo or print
   - file read/write happy path
   - directory/metadata happy path where practical
5. Keep examples aligned with the approved style: top-level `io` imports, module-level functions,
   and explicit provider setup on the Rust side.

## Guidance

### Ash examples to include

```ash
use io::path;

let report = path::join(path::from_string("reports"), "daily.txt");
```

```ash
use io::fs;
use io::stdio;

workflow main() {
    let text = fs::read_to_string("message.txt");
    act stdio::println(text);
    done;
}
```

### Rust test harness style

Match the existing engine builder/test style:

```rust
let engine = Engine::new()
    .with_stdio_capabilities()
    .with_fs_capabilities()
    .build()
    .expect("engine builds");
```

Use the existing stdlib test files as a style reference instead of inventing a new harness:

- [stdlib_surface.rs](../../../crates/ash-parser/tests/stdlib_surface.rs)
- [stdlib_parsing.rs](../../../crates/ash-parser/tests/stdlib_parsing.rs)

## Likely Files

- Modify: `crates/ash-parser/tests/stdlib_surface.rs`
- Modify: `crates/ash-parser/tests/stdlib_parsing.rs`
- Modify: `crates/ash-engine/tests/e2e_capability_provider_tests.rs`
- Modify: `crates/ash-engine/tests/provider_wiring_test.rs`
- Create: `tests/std/io-*.ash`
- Create or Modify: `examples/README.md`
- Create: `examples/03-io/...` or similar

## TDD Steps

### Red

- Add failing parser/engine/example tests that encode the desired user-facing `io` workflows.

### Green

- Land the minimum implementation needed for all representative `io` scenarios to work together.

## Completion Checklist

- [ ] parser stdlib tests cover new `io` modules
- [ ] engine/runtime tests cover representative `io` workflows
- [ ] `.ash` examples added
- [ ] examples follow the approved import/style conventions
- [ ] focused relevant tests pass
- [ ] CHANGELOG.md updated
