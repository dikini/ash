# TASK-922: Monad Evidence Method Body Lowering

## Status: 📝 Planned

## Description

Extend `Monad<K>` evidence from target lookup/return-only boundary to carry selected `return` and `bind` operation identities, method bodies, or intrinsic shims suitable for typed lowering and specialization.

## Specification Reference

- [SPEC-069](../../spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md)
- [SPEC-070](../../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)
- [PLAN-118](../PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md)

## Dependencies

- TASK-920 completion

## Requirements

### Functional Requirements

1. Use TASK-920 exact file/callsite/test bindings before implementation.
2. Add RED tests or evidence first.
3. Implement only the SPEC-069/SPEC-070 slice assigned to this task.
4. Carry selected evidence/method identity or intrinsic shim data through `DoDictionary`; mere `Monad` existence lookup and symbolic `Monad::return` / `Monad::bind` names are insufficient.
5. Bind this task to engine monomorphization/specialization seams so selected method bodies do not disappear before execution.
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
    p = Path("crates/ash-typeck/tests/alpha_monad_evidence_method_body_lowering.rs")
    text = p.read_text()
    names = [
        "monad_evidence_records_return_and_bind_method_bodies",
        "do_option_return_only_lowers_through_selected_evidence_body",
        "ambiguous_monad_evidence_rejected_before_lowering",
    ]
    missing = [name for name in names if f"fn {name}" not in text]
    assert not missing, missing
    print("TASK-922 focused test file and names exist")
    PY
  - cargo test -p ash-typeck --test alpha_monad_evidence_method_body_lowering -- --nocapture
  - git diff --check
checklist:
  - [ ] Focused evidence command patched by TASK-920
  - [ ] Focused tests pass
  - [ ] Broad relevant gate passes
  - [ ] Docs/status/changelog updated if public behavior changed
```

## Dependencies for Next Task

This task outputs:
- Evidence carrier for selected Monad operation identity.

## Notes

- File targets to inspect or modify: `crates/ash-typeck/src/type_env.rs`, `crates/ash-typeck/src/do_target.rs`, `crates/ash-engine/src/monomorphize.rs`, `crates/ash-interp/src/eval.rs`, `crates/ash-typeck/tests/`.
- Keep PLAN-118 decision gates in sync with implementation reality.
- Do not broaden scope beyond SPEC-069/SPEC-070 without a docs patch and review.
