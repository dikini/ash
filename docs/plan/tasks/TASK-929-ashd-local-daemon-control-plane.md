# TASK-929: ashd Local Daemon Control Plane

## Status: ✅ Complete

## Description

Add a local-first alpha daemon/control surface using the same RuntimeKernel semantics, with roots, definition index, instance table, start/list/status/cancel/reload behavior, and same-user local control. TASK-929 selected the final command spelling as `ash daemon ...`.

## Specification Reference

- [SPEC-069](../../spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md)
- [SPEC-070](../../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)
- [PLAN-118](../PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md)

## Dependencies

- TASK-920 completion
- TASK-927 RuntimeKernel carrier decision
- TASK-925/TASK-926 completion for final compiled artifact/version verification, unless this task explicitly records an interim source/check-summary substrate.

## Requirements

### Functional Requirements

1. Use TASK-920 exact file/callsite/test bindings before implementation.
2. Add RED tests or evidence first.
3. Implement only the SPEC-069/SPEC-070 slice assigned to this task.
4. Specify and test the local control protocol: command surface, binary/subcommand choice (`ashd` binary, `ash daemon`, or another documented shape), socket/path rules, same-user authorization, request/response schema, and list/start/status/cancel/reload operations. If the chosen shape is not SPEC-070's `ashd serve ...`, patch SPEC-070 summary and A70-3 acceptance wording before completing this task.
5. Specify and test reload as a transaction: stage new index, swap on compile/check success, preserve old index on failure, and pin running instances to their admitted artifact/version.
6. Add concrete source targets for the chosen daemon/control-plane shape before implementation (for example a new `ashd` binary/crate, `ash-cli` daemon subcommand modules, and shared protocol types).
7. Patch affected specs/plans/status surfaces if behavior or authority changes.
8. Run focused and broad verification specified by TASK-920.

### Property Requirements

Property tests are required for Rust semantic tasks when TASK-920 identifies a stable strategy. Documentation-only or audit tasks must instead provide corpus consistency evidence.

## TDD Steps

### Step 1: Write failing tests or evidence

Use TASK-920-selected files and exact commands; avoid zero-test filters.

### Step 2: Implement or document the slice

Make the smallest change satisfying this task without pulling later tasks forward.

### Step 3: Integrate at public seams

Wire through parser/typeck/engine/runtime/CLI surfaces named by TASK-920.

### Step 4: Verify and record evidence

Run focused commands, broad relevant gate, docs diff/link checks, and update evidence/status surfaces.

## Dispatch

```
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```
strictness: clean
commands:
  - |
    python3 - <<'PY'
    from pathlib import Path
    p = Path("crates/ash-cli/tests/alpha_ashd_local_daemon_control_plane.rs")
    text = p.read_text()
    names = [
        "ashd_serve_indexes_definitions_without_running_workflows",
        "ashd_reload_updates_definition_table_and_preserves_kernel_mode",
        "ashd_rejects_invalid_root",
    ]
    missing = [name for name in names if f"fn {name}" not in text]
    assert not missing, missing
    print("TASK-929 focused test file and names exist")
    PY
  - cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane -- --nocapture
  - git diff --check
checklist:
  - [x] Focused evidence command patched by TASK-920
  - [x] Focused tests pass
  - [x] Broad relevant gate passes
  - [x] Docs/status/changelog updated if public behavior changed
```

## Completion Evidence

- `python3 - <<'PY' ... PY`: focused test-name assertion prints `TASK-929 focused test file and names exist`.
- `cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane -- --nocapture`: 4 focused tests pass, covering `ash daemon serve`, `list`, `start`, `cancel`, `status`, transactional `reload`, invalid-root rejection, and preservation/rejection of pre-existing non-socket control paths.
- `cargo fmt --check`, `git diff --check`, and `cargo check -p ash-cli` are the broad relevant gates for this task.

## Implementation Notes

- TASK-929 selects the final alpha command spelling as `ash daemon ...`, not a separate `ashd` binary.
- The local control protocol is JSON-lines over a Unix-domain socket. The alpha implementation validates the root and local control path ownership before binding, rejects pre-existing non-socket control paths without deleting them, and only unlinks stale Unix-domain sockets. It does not add TCP, remote, multi-user, service-manager, distributed scheduling, or hot-swap semantics.
- `ash daemon start` admits an instance record pinned to the active source/check-summary artifact identity. It does not claim bytecode execution of long-running workflow bodies in the daemon host, and explicit start args/config/admission-profile request fields plus report/log-path observation remain deferred beyond the TASK-929 MVP.

## Dependencies for Next Task

This task outputs:
- Local alpha daemon control surface.

## Notes

- File targets to inspect or modify: `crates/ash-cli/src/main.rs`, `crates/ash-cli/src/commands/`, `crates/ash-engine/src/`, `crates/ash-interp/src/`, `crates/ash-cli/tests/`, plus the new daemon binary/crate/protocol files selected by TASK-920/TASK-929 requirements.
- Keep PLAN-118 decision gates in sync with implementation reality.
- Do not broaden scope beyond SPEC-069/SPEC-070 without a docs patch and review.
