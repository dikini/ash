# TASK-923: Generalized Do Full Bind Lowering

## Status: 📝 Planned

## Description

Implement full `do:K` `<-` lowering through selected `Monad<K>` evidence for Act/Proc/Workflow, `Result<_, E>`, and user/library monads accepted by SPEC-067 evidence.

## Specification Reference

- [SPEC-069](../../spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md)
- [SPEC-070](../../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)
- [PLAN-118](../PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md)

## Dependencies

- TASK-920 completion
- TASK-922 completion for selected `Monad<K>` operation body/intrinsic-shim carriers before full `<-` lowering closeout.

## Requirements

### Functional Requirements

1. Use TASK-920 exact file/callsite/test bindings before implementation.
2. Add RED tests or evidence first.
3. Implement only the SPEC-069/SPEC-070 slice assigned to this task.
4. Add canonical `Monad<Result<_, E>>` evidence or an explicitly equivalent compiler-prelude shim if A69-4 remains alpha acceptance scope.
5. Prove generalized `<-` lowering survives through engine specialization/monomorphization and interpreter dispatch, not only typecheck elaboration.
6. Patch affected specs/plans/status surfaces if behavior or authority changes.
7. Run focused and broad verification specified by TASK-920.

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
    p = Path("crates/ash-typeck/tests/alpha_generalized_do_full_bind_lowering.rs")
    text = p.read_text()
    names = [
        "do_result_bind_lowers_through_monad_bind_evidence",
        "user_option_do_bind_uses_selected_monad_evidence",
        "generic_monad_do_specializes_before_execution",
    ]
    missing = [name for name in names if f"fn {name}" not in text]
    assert not missing, missing
    print("TASK-923 focused test file and names exist")
    PY
  - cargo test -p ash-typeck --test alpha_generalized_do_full_bind_lowering -- --nocapture
  - git diff --check
checklist:
  - [ ] Focused evidence command patched by TASK-920
  - [ ] Focused tests pass
  - [ ] Broad relevant gate passes
  - [ ] Docs/status/changelog updated if public behavior changed
```

## Dependencies for Next Task

This task outputs:
- Full generalized `do:K` bind lowering.

## Notes

- File targets to inspect or modify: `crates/ash-typeck/src/check_expr.rs`, `crates/ash-typeck/src/do_target.rs`, `crates/ash-engine/src/monomorphize.rs`, `crates/ash-interp/src/eval.rs`, `crates/ash-typeck/tests/`.
- Keep PLAN-118 decision gates in sync with implementation reality.
- Do not broaden scope beyond SPEC-069/SPEC-070 without a docs patch and review.
