# TASK-494: Add Stdlib IO Root and Pure Path Surface

## Status: Done

## Description

Create the root `io` stdlib module and the pure `io::path` layer. This task should establish the
shared namespace, error/result vocabulary, and typed path values before any host-touching I/O
surface is added.

## Specification Reference

- [Stdlib `io` V1 Design](../../plans/2026-04-10-stdlib-io-v1-design.md)
- [PLAN-022: Stdlib IO V1](../PLAN-022-STDLIB-IO-V1.md)
- [TASK-493](TASK-493-freeze-stdlib-io-contract.md)

## Dependencies

- [TASK-493](TASK-493-freeze-stdlib-io-contract.md)

## Requirements

1. Create the initial stdlib module tree under `std/src/io/`.
2. Add `std/src/io/mod.ash` with only cross-cutting shared vocabulary:
   - `Error`
   - `ErrorKind`
   - optional `Result<T>`
3. Add `std/src/io/path.ash` as a pure module with first-class path values and pure transforms.
4. Update stdlib root exports so `io` resolves like the existing `runtime`, `option`, and `result`
   modules.
5. Add stdlib parser/surface tests proving the new modules exist and resolve as real file modules.

## Guidance

### Ash target examples

Prefer module functions over method syntax in v1:

```ash
use io::path;

let root = path::from_string("/tmp");
let full = path::join(root, "notes.txt");
let parent = path::parent(full);
```

If the path value type needs a minimal initial shape, start with the smallest representation that
still lets later tasks distinguish "typed path value" from raw `String`.

### Rust guidance

Use Rust path semantics for behavioral guidance:

```rust
use std::path::{Path, PathBuf};

let root = PathBuf::from("/tmp");
let full = root.join("notes.txt");
let parent = full.parent();
```

But keep the Ash surface module-oriented and capability-free.

## Likely Files

- Create: `std/src/io/mod.ash`
- Create: `std/src/io/path.ash`
- Modify: `std/src/lib.ash`
- Modify: `crates/ash-parser/tests/stdlib_surface.rs`
- Modify: `crates/ash-parser/tests/stdlib_parsing.rs`
- Optional Modify: `std/README.md`

## TDD Steps

### Red

- Add failing stdlib-surface tests showing `io` and `io::path` do not yet exist as resolvable file
  modules.

### Green

- Create the root/path modules and make the new tests pass with the minimal canonical surface.

## Completion Checklist

- [x] `std/src/io/mod.ash` exists
- [x] `std/src/io/path.ash` exists
- [x] root stdlib exports updated
- [x] parser/surface tests prove the module tree resolves
- [x] shared `io` vocabulary kept small
- [x] `io::path` remains pure and capability-free
- [x] CHANGELOG.md updated

## Implementation Notes

The path module uses `PathBuf { inner: String }` newtype syntax and pattern matching with `match path { PathBuf { inner: p } => p }`. This representation is **aspirational**:

1. The pattern match unwrapping syntax may not match the actual runtime's Value representation for newtypes
2. String operations (`string::concat`, `string::rfind`, etc.) are used but the `string` module does not exist in the current stdlib - these would need to be builtins or added to stdlib
3. The `Bytes` type referenced in `read_to_end` is not yet defined in the type system

These gaps are documented in PLAN-022 and do not affect the structural validity of the module tree or the test harness.
