# Stdlib `io` V1 Design (FROZEN)

**Date:** 2026-04-10
**Frozen By:** TASK-493
**Status:** Contract locked for Phase 74 implementation

**Post-Implementation Note:** The Phase 74 implementation (completed) produced an **exploratory/reference implementation** of this design. While the module structure, provider expansions, and test harness are functional, the Ash surface syntax in the .ash files uses aspirational constructs that anticipate future spec alignment. See "Implementation Gaps" in PLAN-022 for details.

## Frozen Contract Notice

This document defines the frozen V1 contract for the Ash standard library `io` namespace. The module tree, capability boundaries, and import patterns specified herein are locked and may not change without a formal revision process. Implementation must adhere to this contract.

## Goal

Define the initial standard-library design for Ash `io` as a top-level stdlib module family rooted at `std/src/io/`, with a capability-oriented public surface, explicit handle types as the semantic foundation, and a clear v1 scope that maps cleanly to host Rust functionality without exposing Rust's structure directly as the Ash contract.

## Context

The current Ash stdlib already establishes three relevant constraints:

- stdlib modules are imported as top-level namespaces such as `result` and `runtime`, not through a required `std::` prefix;
- the stdlib surface should remain file-backed and consistent with the module-resolution rules rooted at `std/src/`;
- the runtime and capability model should remain visible in API design rather than being hidden behind unrestricted ambient host access.

The `io` module family therefore needs to satisfy two goals at once:

1. feel broad enough to serve as the obvious standard-library home for text, file, directory, and path operations;
2. preserve Ash's capability discipline so that host access stays explicit in the language model even when ergonomic helper functions are provided.

## Frozen Contract Summary

The following elements are **locked** as the V1 contract:

| Element | Frozen Value |
|---------|--------------|
| **Top-level namespace** | `io` (not `std::io`) |
| **Import style** | `use io::fs;` (canonical), NOT `use std::io::fs;` |
| **V1 module tree** | `io`, `io::path`, `io::stdio`, `io::fs`, `io::dir`, `io::meta`, `io::buf` |
| **Capability-free module** | `io::path` (pure computation only) |
| **Capability-bearing modules** | `io::stdio`, `io::fs`, `io::dir`, `io::meta`, `io::buf` |
| **Result type** | Reuses `result::Result<T, E>`, NOT a new global Result ADT |
| **Deferred modules** | `io::stream`, `io::watch` (explicitly excluded from V1) |

### Canonical Import Examples (Frozen)

```ash
-- Correct V1 import style
use io::fs;
use io::path::PathBuf;
use io::stdio;

-- Incorrect (will not work)
use std::io::fs;  -- ERROR: no std:: prefix for stdlib
```

## Design Decisions

### Root Namespace

`io` is a top-level standard-library module. User code imports it as:

```ash
use io::fs;
use io::path::PathBuf;
use io::stdio;
```

Ash should not require `std::io::...` for stdlib access. This keeps `io` aligned with the existing `option`, `result`, and `runtime` surfaces and remains consistent with the current stdlib import rules.

### Capability Model

`io` uses a mixed model:

- ambient convenience functions are available at the module surface for common tasks;
- explicit handles remain the core abstraction beneath those helpers;
- capability checks conceptually attach to authority-bearing operations rather than to pure values.

This model keeps the first library surface ergonomic while preserving a path toward a stricter explicit-handle runtime substrate later.

### Rust Interop Posture

The `io` design may wrap native Rust functionality internally, but Ash should not mirror Rust module boundaries mechanically. Public Ash modules should be organized around semantic responsibility and capability boundaries. Rust alignment should happen in the implementation layer, especially for handle behavior, open options, and error mapping.

## Module Structure

The recommended v1 module tree is:

- `io`
- `io::path`
- `io::stdio`
- `io::fs`
- `io::dir`
- `io::meta`
- `io::buf`

Two additional modules are intentionally deferred:

- `io::stream`
- `io::watch`

These should only be introduced once Ash needs stable interface-level stream polymorphism or file-notification semantics.

## Module Responsibilities

### `io`

`io` is the shared vocabulary layer. It should stay intentionally small and contain only cross-cutting items that are reused across multiple `io` submodules.

Recommended contents:

- `Error`
- `ErrorKind`
- optional alias `Result<T> = result::Result<T, io::Error>`
- shared enums such as `SeekFrom` or `FileKind` if they are truly cross-cutting

`io` should not become a grab-bag of concrete operations. File, directory, metadata, and terminal behavior belongs in submodules.

### `io::path`

`io::path` is the pure structural path layer. It contains value types and transformations only, with no host access and no capability requirements.

Recommended contents:

- `Path`
- `PathBuf`
- `join`
- `parent`
- `file_name`
- `extension`
- `components`
- `is_absolute`
- `normalize`

This module exists so Ash can model filesystem locations as first-class values without conflating path manipulation with filesystem authority.

### `io::stdio`

`io::stdio` owns standard input/output/error.

Recommended contents:

- ambient helpers: `read_line`, `print`, `println`, `eprint`, `eprintln`
- handle constructors: `stdin()`, `stdout()`, `stderr()`
- explicit handle types: `Stdin`, `Stdout`, `Stderr`
- handle operations: `read_line`, `write`, `flush`

The ambient helpers are convenience APIs only. They should conceptually lower to explicit-handle operations plus capability checks.

### `io::fs`

`io::fs` owns file operations and file handles.

Recommended contents:

- convenience operations: `read`, `read_to_string`, `write`, `write_string`, `append`, `copy`, `rename`, `remove_file`
- file-handle constructors: `open`, `create`, `create_new`, `open_with`
- explicit handle type: `File`
- configuration type: `OpenOptions`

This is the main host-authority-bearing file module. It should be broad enough to feel like a standard library, but it should avoid prematurely exposing advanced OS-specific behavior.

### `io::dir`

`io::dir` owns directory creation, removal, and entry enumeration.

Recommended contents:

- `create_dir`
- `create_dir_all`
- `remove_dir`
- `remove_dir_all`
- `read_dir`
- `ReadDir`
- `DirEntry`

Recursive traversal helpers should be deferred unless the language has a clear, stable iteration story for them.

### `io::meta`

`io::meta` owns metadata and permission inspection.

Recommended contents:

- `metadata`
- later, optionally `symlink_metadata`
- `Metadata`
- `Permissions`
- queries such as `is_file`, `is_dir`, `len`, `modified`, `readonly`

This module should expose inspection, not mutation-heavy platform-specific knobs.

### `io::buf`

`io::buf` should stay helper-oriented in v1. It exists to provide ergonomic buffered or aggregate read/write operations without forcing a full generic stream abstraction into the initial design.

Recommended contents:

- `read_to_end`
- `read_to_string`
- `write_all`
- `lines`

If buffered wrapper types later become necessary, they can be added on top of this helper surface.

## Type and Error Design

Ash already has the generic ADT `result::Result<T, E>`. `io` should reuse it rather than introducing a second general result type.

The appropriate domain-specific pattern is:

- `io::Error` is owned by the `io` root module;
- `io::Result<T>` may exist as a convenience alias to `result::Result<T, io::Error>`;
- submodules should not define parallel error ADTs unless later work identifies a strong reason to do so.

This keeps the stdlib coherent and matches Rust's shape without copying it literally.

Paths should be first-class values rather than plain strings in the canonical surface. String-based convenience overloads may be added later, but the design center should remain typed path values.

## Capability Boundary

The authority split should be modeled by the runtime/provider layer even when the public API presents convenient module functions.

An initial capability vocabulary should distinguish at least:

- reading from stdin;
- writing to stdout;
- writing to stderr;
- reading files;
- writing files;
- creating files;
- listing directories;
- mutating directories;
- inspecting metadata.

Exact capability names can be finalized during implementation, but the design constraint is fixed: pure path manipulation is capability-free, while host-touching operations belong to authority-bearing submodules.

## Ambient Helpers Versus Explicit Handles

Ambient helpers are allowed in v1 because they improve usability for the common case. They should remain thin sugar over the deeper explicit-handle model rather than defining a second execution model.

Conceptually:

- `io::fs::read_to_string(path)` should correspond to opening a file handle and reading from it with file-read authority;
- `io::stdio::println(text)` should correspond to writing to the stdout handle with stdout-write authority.

This principle matters because later runtime work may want to expose injected providers, testing doubles, or more explicit capability passing without redesigning the public surface.

## V1 Exclusions

The first `io` release should explicitly exclude:

- async I/O;
- symlink and hard-link APIs;
- temporary-file helpers;
- file locking;
- memory-mapped I/O;
- file watching or notification APIs;
- recursive directory walkers with rich filtering;
- advanced OS-specific permission mutation APIs;
- generic stream traits or interfaces that depend on broader language abstraction work.

These exclusions are deliberate. They keep the first design broad enough to be useful while avoiding early commitment to abstractions that Ash has not yet stabilized elsewhere.

## Recommended Import Style

Examples and future docs should consistently use top-level stdlib imports:

```ash
use io::fs;
use io::meta;
use io::path::PathBuf;
use io::stdio;
```

This design should not introduce a separate `std::` namespace layer.

## Follow-On Planning

When implementation planning begins, the next planning artifact should:

1. define the exact file layout under `std/src/io/`;
2. choose the initial `io::Error` structure and error-kind taxonomy;
3. define the concrete v1 function and type surface for each submodule;
4. define how ambient helper functions lower to capability/provider-backed operations;
5. identify parser, lowering, typechecking, runtime, and stdlib-surface tests needed for the first `io` tasks.

## Success Criteria

This design is successful if it gives later implementation tasks a clear answer to the following questions:

- why `io` is a top-level stdlib module rather than `std::io`;
- which submodules belong in v1 and which do not;
- which parts of the surface are pure versus authority-bearing;
- how ambient ergonomics coexist with explicit handles;
- how the Ash `io` surface can wrap Rust functionality without becoming a direct Rust mirror.
