# TASK-497: Add Stdlib IO Buffered Helpers and Ambient Sugar

## Status: Done

## Description

Introduce `io::buf` and the first ergonomic helper layer over the capability-backed `io` modules.
This task should make common operations pleasant to use without inventing a separate runtime model
or prematurely committing to generic stream traits.

## Specification Reference

- [Stdlib `io` V1 Design](../../plans/2026-04-10-stdlib-io-v1-design.md)
- [PLAN-022: Stdlib IO V1](../PLAN-022-STDLIB-IO-V1.md)

## Dependencies

- [TASK-495](TASK-495-stdlib-io-stdio-surface-and-provider-alignment.md)
- [TASK-496](TASK-496-stdlib-io-files-dir-meta-surface.md)

## Requirements

1. Add `std/src/io/buf.ash`.
2. Define helper-oriented operations such as:
   - `read_to_end`
   - `read_to_string`
   - `write_all`
   - `lines`
3. Add only the ambient sugar that the current language/runtime can express cleanly.
4. Keep helpers shallow: they should correspond to capability/provider-backed operations rather than
   a second hidden execution path.
5. Defer generic stream traits/interfaces and buffered wrapper types if they force unstable language
   abstractions.

## Guidance

### Ash target examples

```ash
use io::buf;
use io::fs;
use io::path;

let path = path::join(path::from_string("/tmp"), "input.txt");
let text = fs::read_to_string(path);
let lines = buf::lines(text);
```

```ash
use io::stdio;

act stdio::println("ready");
```

The helper surface should feel like a small ergonomic layer, not a second I/O subsystem.

### Rust guidance

Rust reference points:

```rust
use std::io::{BufRead, Read, Write};

let mut s = String::new();
reader.read_to_string(&mut s)?;
writer.write_all(s.as_bytes())?;
```

Take semantic cues from these operations, but keep the Ash API module-oriented and v1-sized.

## Likely Files

- Create: `std/src/io/buf.ash`
- Modify: `std/src/io/mod.ash`
- Modify: `std/src/io/stdio.ash`
- Modify: `std/src/io/fs.ash`
- Modify: `crates/ash-parser/tests/stdlib_surface.rs`
- Optional Modify: `std/README.md`

## TDD Steps

### Red

- Add failing stdlib-surface tests that assert the desired buffered/helper vocabulary is missing.

### Green

- Add the smallest helper layer that makes common `io` workflows ergonomic without widening scope.

## Completion Checklist

- [ ] `io::buf` module exists
- [ ] helper surface is module-oriented and shallow
- [ ] no generic stream trait design is forced into v1
- [ ] stdlib surface tests updated
- [ ] CHANGELOG.md updated
