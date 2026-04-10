# TASK-498: Bootstrap Stdlib IO Modules Through Runtime and Module Wiring

## Status: Done

## Description

Make the new `io` stdlib modules load, resolve, and execute through the same authoritative
module/capability/provider paths used by the rest of the system. This task closes the gap between
stdlib source files and actual engine execution.

## Specification Reference

- [PLAN-022: Stdlib IO V1](../PLAN-022-STDLIB-IO-V1.md)
- [Stdlib `io` V1 Design](../../plans/2026-04-10-stdlib-io-v1-design.md)
- [SPEC-009: Modules](../../spec/SPEC-009-MODULES.md)
- [SPEC-012: Imports](../../spec/SPEC-012-IMPORTS.md)
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)
- [TASK-477](TASK-477-stdlib-capability-bootstrap-and-bridge-removal.md)
- [TASK-479](TASK-479-module-owned-capability-resolution-verification.md)

## Dependencies

- [TASK-495](TASK-495-stdlib-io-stdio-surface-and-provider-alignment.md)
- [TASK-496](TASK-496-stdlib-io-files-dir-meta-surface.md)
- [TASK-497](TASK-497-stdlib-io-buffered-helpers-and-ambient-sugar.md)

## Requirements

1. Ensure the new `io` stdlib files are loaded from `std/src/` like the existing stdlib modules.
2. Ensure capability exports from the new `io` modules participate in the shared module-owned
   capability-resolution pipeline.
3. Audit engine/module-loading/runtime registration paths for any stdlib assumptions that still
   hard-code older runtime-only module names.
4. Keep provider registration in the engine explicit and unsurprising:
   - `with_stdio_capabilities()`
   - `with_fs_capabilities()`
5. Add focused tests proving that stdlib `io` imports and runtime provider setup meet in the same
   execution path.

## Guidance

### Ash target example

```ash
use io::fs;
use io::stdio;

workflow main() {
    let text = fs::read_to_string("input.txt");
    act stdio::println(text);
    done;
}
```

### Rust guidance

The current engine/provider bootstrap anchors are:

- [lib.rs](../../../crates/ash-engine/src/lib.rs)
- [module_loader.rs](../../../crates/ash-engine/src/module_loader.rs)
- [entry.rs](../../../crates/ash-engine/src/entry.rs)

Reference builder style:

```rust
let engine = Engine::new()
    .with_stdio_capabilities()
    .with_fs_capabilities()
    .build()?;
```

Prefer extending these paths over introducing parallel `io`-specific bootstrappers.

## Likely Files

- Modify: `crates/ash-engine/src/lib.rs`
- Modify: `crates/ash-engine/src/module_loader.rs`
- Optional Modify: `crates/ash-engine/src/entry.rs`
- Optional Modify: `crates/ash-parser/src/capability_export.rs`
- Optional Modify: `crates/ash-parser/src/capability_pipeline.rs`
- Modify: `crates/ash-engine/tests/provider_wiring_test.rs`
- Modify: `crates/ash-engine/tests/e2e_capability_provider_tests.rs`

## TDD Steps

### Red

- Add focused engine tests showing stdlib `io` imports do not yet round-trip through module loading
  and provider execution.

### Green

- Make the new stdlib modules participate in the existing authoritative loading and resolution path.

## Completion Checklist

- [ ] new `io` stdlib modules load through the normal stdlib root
- [ ] `io` capability exports participate in shared resolution
- [ ] engine provider setup remains explicit and documented
- [ ] focused engine/runtime tests pass
- [ ] no parallel bootstrap path introduced
- [ ] CHANGELOG.md updated
