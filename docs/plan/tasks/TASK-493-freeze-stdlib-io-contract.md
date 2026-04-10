# TASK-493: Freeze Stdlib IO V1 Contract

## Status: Done

## Description

Turn the approved `io` v1 design into the canonical implementation contract before parser,
typecheck, runtime, and stdlib work start landing. This task should freeze the namespace, module
split, capability boundary, and first examples so later tasks share one source of truth.

## Specification Reference

- [Stdlib `io` V1 Design](../../plans/2026-04-10-stdlib-io-v1-design.md)
- [PLAN-022: Stdlib IO V1](../PLAN-022-STDLIB-IO-V1.md)
- [SPEC-009: Modules](../../spec/SPEC-009-MODULES.md)
- [SPEC-012: Imports](../../spec/SPEC-012-IMPORTS.md)
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)
- [SPEC-010: Embedding](../../spec/SPEC-010-EMBEDDING.md)

## Requirements

1. Define `io` as a top-level stdlib namespace imported as `use io::...`, not `use std::io::...`.
2. Freeze the v1 module tree:
   - `io`
   - `io::path`
   - `io::stdio`
   - `io::fs`
   - `io::dir`
   - `io::meta`
   - `io::buf`
3. State explicitly that `io::path` is pure while `io::stdio`, `io::fs`, `io::dir`, and `io::meta`
   are capability-bearing host-touching modules.
4. Reuse `result::Result<T, E>` rather than introducing a second global `Result` ADT.
5. Add Ash-facing examples and Rust-facing implementation guidance for the modules that later tasks
   will build.

## Guidance

### Ash target examples

These examples are the target style the rest of the phase should preserve:

```ash
use io::fs;
use io::path;
use io::stdio;

let config_dir = path::join(path::from_string("config"), "app.toml");
let text = fs::read_to_string(config_dir);
act stdio::println(text);
```

```ash
use io::dir;
use io::meta;
use io::path;

let root = path::from_string("/tmp/work");
let entries = dir::read_dir(root);
let info = meta::metadata(path::join(root, "input.txt"));
```

### Rust guidance

Implementers should use Rust APIs as semantic references, not as a direct namespace template:

- `std::path::{Path, PathBuf}` guides `io::path`
- `std::io::{stdin, stdout, Write}` guides `io::stdio`
- `std::fs::{read_to_string, write, create_dir_all, metadata}` guides `io::fs`, `io::dir`, and `io::meta`

Do not mechanically copy Rust methods into Ash if module-level functions are the better fit for the
current language surface.

## Likely Files

- Modify: `docs/spec/SPEC-009-MODULES.md`
- Modify: `docs/spec/SPEC-012-IMPORTS.md`
- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Optional Modify: `docs/spec/SPEC-010-EMBEDDING.md`
- Modify: `docs/plans/2026-04-10-stdlib-io-v1-design.md`

## TDD Steps

### Red

- Identify every active spec/doc location that currently implies `io` through older provider names
  (`stdio`, `fs`) or lacks a normative stdlib `io` module contract.

### Green

- Update the relevant specs/docs so the `io` namespace, v1 module tree, and capability boundary are
  all defined once and consistently.

## Completion Checklist

- [ ] `io` namespace contract frozen in active docs/specs
- [ ] v1 module tree documented
- [ ] pure-vs-host-touching boundary documented
- [ ] Ash examples added
- [ ] Rust reference examples added
- [ ] CHANGELOG.md updated
