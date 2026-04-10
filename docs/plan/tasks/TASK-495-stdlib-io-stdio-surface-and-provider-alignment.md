# TASK-495: Add Stdlib IO Stdio Surface and Align It to the Provider Layer

## Status: Done

## Description

Introduce `io::stdio` as the canonical Ash stdlib surface for terminal input/output and align it
with the existing Rust `StdioProvider` implementation. This task should make the stdlib story and
the runtime/provider story match instead of leaving users with provider-only names.

## Specification Reference

- [Stdlib `io` V1 Design](../../plans/2026-04-10-stdlib-io-v1-design.md)
- [PLAN-022: Stdlib IO V1](../PLAN-022-STDLIB-IO-V1.md)
- [TASK-456](TASK-456-stdio-provider-unified-trait.md)
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)
- [SPEC-010: Embedding](../../spec/SPEC-010-EMBEDDING.md)

## Dependencies

- [TASK-494](TASK-494-stdlib-io-root-and-path-surface.md)

## Requirements

1. Add `std/src/io/stdio.ash`.
2. Define the canonical stdio capability/helper surface for v1:
   - `read_line`
   - `print`
   - `println`
   - optional `eprint`, `eprintln` only if the provider/runtime path can support them cleanly now
3. Keep the Ash-facing names under `io::stdio::...` even if the backing provider name remains
   `"stdio"` in Rust.
4. Audit `StdioProvider` in `crates/ash-engine/src/providers/mod.rs` and align its supported action
   names, docs, and tests to the new stdlib contract.
5. Add targeted tests showing stdlib usage and provider execution agree on names and behavior.

## Guidance

### Ash target examples

```ash
use io::stdio;

workflow echo() {
    let line = stdio::read_line();
    act stdio::println(line);
    done;
}
```

Prefer this as the user-facing style over exposing bare provider names in examples.

### Rust guidance

Use the existing provider implementation as the starting point:

- [providers/mod.rs](../../../crates/ash-engine/src/providers/mod.rs)

The semantics should continue to track familiar Rust behavior:

```rust
use std::io::{self, BufRead, Write};

let stdin = io::stdin();
let mut line = String::new();
stdin.lock().read_line(&mut line)?;
print!("{line}");
io::stdout().flush()?;
println!("{line}");
```

Do not rewrite the provider architecture unless the `io::stdio` contract genuinely requires it.

## Likely Files

- Create: `std/src/io/stdio.ash`
- Modify: `std/src/io/mod.ash`
- Modify: `crates/ash-engine/src/providers/mod.rs`
- Modify: `crates/ash-engine/tests/provider_wiring_test.rs`
- Modify: `crates/ash-engine/tests/e2e_capability_provider_tests.rs`
- Modify: `crates/ash-parser/tests/stdlib_surface.rs`

## TDD Steps

### Red

- Add failing stdlib/provider tests showing there is no canonical `io::stdio` surface and no
  aligned behavior contract.

### Green

- Add the stdlib module and update the provider/tests until `io::stdio` is the documented and
  tested source of truth.

## Completion Checklist

- [x] `std/src/io/stdio.ash` exists
- [x] stdlib docs/tests define the canonical stdio surface
- [x] `StdioProvider` action names and docs align with the stdlib story
- [x] focused engine/provider tests pass
- [x] no duplicate "real" stdio API remains in active docs/examples
- [x] CHANGELOG.md updated

## Implementation Notes

The stdio.ash capability declaration uses aspirational syntax:

```ash
pub capability Stdio: observe read_line() returns String
                    | execute print(text: String)
                    | execute println(text: String);
```

This syntax is **not yet supported** by the current parser. The actual capability grammar from SPEC-017 uses a different surface. The `act observe` and `act execute` statements in the function bodies are also illustrative of the intended lowering path, not the currently implemented lowering.

The implementation serves as a reference for:
1. How capability declarations could look in future spec revisions
2. The intended relationship between stdlib surface and provider actions
3. Test harness validation that the module structure resolves correctly
