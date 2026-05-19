# TASK-920: Alpha Visible Tower Audit Gate

## Status: ✅ Complete

## Description

Audit live parser, typechecker, evidence, Act/Proc/Workflow, IR/execution, and CLI/runtime seams before implementation. Patch TASK-921 through TASK-931 verification blocks with exact focused commands that exercise non-empty evidence and file/callsite ownership.

## Specification Reference

- [SPEC-069](../../spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md)
- [SPEC-070](../../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)

## Dependencies

- ✅ TASK-919 docs packet complete

## Requirements

### Functional Requirements

1. Map live `DoBlock`, `do_target`, `Monad`, workflow algebra, ActEnv, Proc, WorkflowFailure, CLI, and runtime surfaces.
2. Identify exact files and current/target callsites for downstream tasks.
3. Write audit artifact under `docs/plan/audits/`.
4. Replace downstream fail-closed guards with exact focused commands.

### Property Requirements

Property tests are required for Rust semantic tasks when TASK-920 identifies a stable strategy. Documentation-only or audit tasks must instead provide corpus consistency evidence.

## TDD Steps

### Step 1: Run live code survey

Search/read exact current callsites.

### Step 2: Create audit artifact

Record current state, target seams, owner crate, exact tests.

### Step 3: Patch downstream guards

Replace `false # TASK-920` in TASK-921 through TASK-931.

### Step 4: Verify audit

Run task-structure scan, docs links, and diff check.

## Dispatch

```
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Verification

These are TASK-920 completion gates. They passed after the audit artifact was created and TASK-921 through TASK-931 verification commands were patched.

```
strictness: clean
commands:
  - test -f docs/plan/audits/TASK-920-alpha-visible-tower-audit-gate.md
  - |
    python3 - <<'PY'
    from pathlib import Path
    files = [next(Path("docs/plan/tasks").glob(f"TASK-{n}-*.md")) for n in range(921, 932)]
    missing = [str(p) for p in files if "false # TASK-920" in p.read_text()]
    forbidden = ["cargo test ... || true", "echo \"placeholder\"", "echo placeholder", "placeholder command"]
    placeholders = [f"{p}:{sentinel}" for p in files for sentinel in forbidden if sentinel in p.read_text()]
    assert not missing, missing
    assert not placeholders, placeholders
    print("downstream guards and placeholder sentinels patched")
    PY
  - git diff --check
checklist:
  - [x] Audit artifact exists
  - [x] Downstream focused commands patched
  - [x] No `false # TASK-920` or obvious zero-test/self-satisfying placeholders remain
  - [x] Scoped docs checks pass
```

## Completion Notes

- Added `docs/plan/audits/TASK-920-alpha-visible-tower-audit-gate.md`.
- Patched TASK-921 through TASK-931 verification blocks with Python file/name assertions and exact focused cargo test commands.
- Verified `test -f docs/plan/audits/TASK-920-alpha-visible-tower-audit-gate.md`, the downstream guard scan, and `git diff --check`.

## Dependencies for Next Task

This task outputs:
- Audit artifact with live call graph.
- Downstream tasks made ready/blocked honestly.

## Notes

- File targets to inspect or modify: `crates/ash-parser/src/surface.rs`, `crates/ash-typeck/src/do_target.rs`, `crates/ash-typeck/src/check_expr.rs`, `crates/ash-typeck/src/type_env.rs`, `crates/ash-core/src/runtime.rs`, `crates/ash-interp/src/eval.rs`, `crates/ash-cli/src/main.rs`.
- Keep PLAN-118 decision gates in sync with implementation reality.
- Do not broaden scope beyond SPEC-069/SPEC-070 without a docs patch and review.
