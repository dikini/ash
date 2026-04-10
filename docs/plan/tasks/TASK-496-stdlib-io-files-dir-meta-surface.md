# TASK-496: Add Stdlib IO File, Directory, and Metadata Surface

## Status: Done

## Description

Extend the initial `io` stdlib family with `io::fs`, `io::dir`, and `io::meta`, and align the
existing filesystem provider layer to that contract. This is the practical v1 core for host-backed
file I/O beyond stdio.

## Specification Reference

- [Stdlib `io` V1 Design](../../plans/2026-04-10-stdlib-io-v1-design.md)
- [PLAN-022: Stdlib IO V1](../PLAN-022-STDLIB-IO-V1.md)
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)
- [SPEC-010: Embedding](../../spec/SPEC-010-EMBEDDING.md)

## Dependencies

- [TASK-494](TASK-494-stdlib-io-root-and-path-surface.md)
- [TASK-495](TASK-495-stdlib-io-stdio-surface-and-provider-alignment.md)

## Requirements

1. Add `std/src/io/fs.ash`, `std/src/io/dir.ash`, and `std/src/io/meta.ash`.
2. Define the v1 file/directory/metadata surface around module-level operations first:
   - `io::fs`: `read`, `read_to_string`, `write`, `write_string`, `append`, `copy`, `rename`, `remove_file`
   - `io::dir`: `create_dir`, `create_dir_all`, `remove_dir`, `remove_dir_all`, `read_dir`
   - `io::meta`: `metadata`, `is_file`, `is_dir`, `len`, `readonly`
3. Reuse typed path values from `io::path` as the canonical input shape where the current language
   surface can support them cleanly.
4. Expand the Rust filesystem provider incrementally from its current `read_file` / `write_file` /
   `exists` baseline instead of replacing it wholesale.
5. Keep advanced features out of scope: symlinks, locks, temp files, watchers, and mmap.

## Guidance

### Ash target examples

```ash
use io::fs;
use io::path;

let notes = path::join(path::from_string("/tmp"), "notes.txt");
let content = fs::read_to_string(notes);
act fs::write_string(notes, content ++ "\nupdated");
```

```ash
use io::dir;
use io::meta;
use io::path;

let root = path::from_string("/tmp/work");
act dir::create_dir_all(root);
let info = meta::metadata(path::join(root, "input.txt"));
```

### Rust guidance

Use the existing provider file as the implementation anchor:

- [providers/mod.rs](../../../crates/ash-engine/src/providers/mod.rs)

Behavior should stay recognizably close to:

```rust
use std::fs;

let text = fs::read_to_string("notes.txt")?;
fs::write("notes.txt", text)?;
let md = fs::metadata("notes.txt")?;
assert!(md.is_file());
```

Add capabilities and action names incrementally; avoid one massive provider rewrite.

## Likely Files

- Create: `std/src/io/fs.ash`
- Create: `std/src/io/dir.ash`
- Create: `std/src/io/meta.ash`
- Modify: `std/src/io/mod.ash`
- Modify: `crates/ash-engine/src/providers/mod.rs`
- Modify: `crates/ash-engine/tests/provider_wiring_test.rs`
- Modify: `crates/ash-engine/tests/e2e_capability_provider_tests.rs`
- Modify: `crates/ash-parser/tests/stdlib_surface.rs`
- Modify: `crates/ash-parser/tests/stdlib_parsing.rs`

## TDD Steps

### Red

- Add failing stdlib/provider tests showing the file/directory/metadata modules and operations are
  missing or inconsistent.

### Green

- Implement the minimal coherent surface and provider support that makes the new tests pass.

## Completion Checklist

- [ ] `io::fs` module exists with canonical v1 surface
- [ ] `io::dir` module exists with canonical v1 surface
- [ ] `io::meta` module exists with canonical v1 surface
- [ ] `FsProvider` expanded deliberately from current baseline
- [ ] focused engine/provider tests pass
- [ ] advanced v1 exclusions remain excluded
- [ ] CHANGELOG.md updated
