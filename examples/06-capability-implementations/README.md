# Phase 104 capability implementation examples

This directory demonstrates the NOTE-009 / SPEC-052 / SPEC-053 capability model:

- `01-mock-internal-kv.ash` shows a mock/internal key-value implementation backed by workflow-owned authority.
- `02-caching-kv-adapter.ash` shows a derived adapter that depends on another admitted `KeyValue` capability plus its own cache authority.
- `03-recording-replay-sketch.ash` sketches record/replay substitutions using a replay-log resource.

Current execution status:

- These source examples are intended for `ash check` conformance. They exercise parser/typechecker support for `capability interface`, `capability impl`, `resource type`, and declared resource/capability/config dependency shapes. Workflow `owns`/`uses` binding examples remain covered by parser/typechecker tests until mixed top-level declaration + workflow source packets are accepted end-to-end by the CLI module parser.
- TASK-741 added runtime API support for executing registered Ash-defined implementation operation bodies. TASK-742 executable coverage therefore lives in `crates/ash-interp/tests/task_742_capability_examples.rs`.
- Standalone `ash run` lowering from source-level capability implementation declarations into runtime admissions and registered operation bodies is not complete yet. These examples intentionally do not claim end-to-end CLI execution until TASK-743/TASK-744 wiring lands.
