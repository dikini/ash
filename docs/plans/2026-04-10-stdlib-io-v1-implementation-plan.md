# Stdlib IO V1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the first Ash `io` standard-library family with pure path utilities, capability-backed stdio/filesystem modules, and end-to-end engine/test coverage.

**Post-Implementation Note:** The implementation produced by this plan is **exploratory/reference-quality**. The stdlib .ash files use aspirational syntax for capabilities and provider calls that demonstrate the intended design but do not yet match the fully-implemented spec. The module structure, provider expansions, and test harness are functional and serve as a foundation for future spec alignment.

**Architecture:** The public surface is capability-oriented and imported as top-level `io::...` modules. `io::path` stays pure, while `io::stdio`, `io::fs`, `io::dir`, and `io::meta` lower to provider-backed operations that build on the existing `StdioProvider` and `FsProvider` infrastructure. Ambient helper functions are thin sugar over the explicit capability/handle model approved in the v1 design.

**Tech Stack:** Rust 2024 workspace crates (`ash-parser`, `ash-typeck`, `ash-interp`, `ash-engine`), Ash stdlib sources under `std/src/`, unified capability providers, Common Changelog.

---

### Task 1: Freeze the `io` contract and examples

**Task File:** `docs/plan/tasks/TASK-493-freeze-stdlib-io-contract.md`

Document the canonical `io` namespace, the v1 module tree, and the capability boundary in specs and active planning docs before implementation spreads assumptions across parser/typecheck/runtime layers.

### Task 2: Add `io` root and pure path surface

**Task File:** `docs/plan/tasks/TASK-494-stdlib-io-root-and-path-surface.md`

Create the root `io` module and pure `io::path` layer first so later host-touching work has a stable namespace and typed path vocabulary to target.

### Task 3: Land stdio surface and provider alignment

**Task File:** `docs/plan/tasks/TASK-495-stdlib-io-stdio-surface-and-provider-alignment.md`

Expose `io::stdio` in Ash and align it with the existing Rust `StdioProvider` so `print`, `println`, and `read_line` have one canonical stdlib story.

### Task 4: Land file, directory, and metadata surface

**Task File:** `docs/plan/tasks/TASK-496-stdlib-io-files-dir-meta-surface.md`

Expand the stdlib and provider surface to cover the practical file-oriented v1 core: file reads/writes, directory listing/mutation, and metadata inspection.

### Task 5: Add buffered helpers and ambient sugar

**Task File:** `docs/plan/tasks/TASK-497-stdlib-io-buffered-helpers-and-ambient-sugar.md`

Add `io::buf` and the first sugar layer that makes the capability-backed surface ergonomic without introducing a second execution model.

### Task 6: Bootstrap new stdlib modules and runtime wiring

**Task File:** `docs/plan/tasks/TASK-498-stdlib-io-bootstrap-and-runtime-wiring.md`

Make the engine/module pipeline load, export, and execute the new `io` modules through the same authoritative stdlib and capability-resolution paths used elsewhere.

### Task 7: Add integration tests and repository examples

**Task File:** `docs/plan/tasks/TASK-499-stdlib-io-integration-tests-and-examples.md`

Prove the intended user experience with parser/typecheck/runtime/engine tests and `.ash` examples that show both the Ash surface and the intended capability setup.

### Task 8: Final docs and verification

**Task File:** `docs/plan/tasks/TASK-500-stdlib-io-docs-and-verification.md`

Close out the phase by updating active docs, changelog, and phase tracking, then run the quality gate.
