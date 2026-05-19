# TASK-925: TCIR Computation Expression Boundary

## Status: 📝 Planned

## Description

Introduce or designate a typed computation-expression/TCIR carrier that preserves source, target constructor, evidence identity, tower level, closure, failure, and explicit-lift provenance for execution lowering.

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
4. Treat existing `CoreExpr` / workflow typed artifacts as insufficient unless they preserve source statement spans, target constructor identity, evidence identity, tower level, explicit lifts, and failure-boundary provenance.
5. Add negative checks proving user constructors are not collapsed into Act/Proc/Workflow runtime terms merely for execution convenience.
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
    p = Path("crates/ash-core/tests/alpha_tcir_computation_expression.rs")
    text = p.read_text()
    names = [
        "tcir_records_source_do_target_and_selected_evidence",
        "tcir_preserves_tower_level_and_workflow_artifact_provenance",
    ]
    missing = [name for name in names if f"fn {name}" not in text]
    assert not missing, missing
    print("TASK-925 focused test file and names exist")
    PY
  - cargo test -p ash-core --test alpha_tcir_computation_expression -- --nocapture
  - git diff --check
checklist:
  - [ ] Focused evidence command patched by TASK-920
  - [ ] Focused tests pass
  - [ ] Broad relevant gate passes
  - [ ] Docs/status/changelog updated if public behavior changed
```

## Dependencies for Next Task

This task outputs:
- Typed computation-expression/TCIR carrier.

## Notes

- File targets to inspect or modify: `crates/ash-core/src/type_ir.rs`, `crates/ash-core/src/workflow_carrier.rs`, `crates/ash-typeck/src/check_expr.rs`, `crates/ash-typeck/src/do_target.rs`, `crates/ash-engine/src/monomorphize.rs`, plus parser/source-span files identified by TASK-920.
- Keep PLAN-118 decision gates in sync with implementation reality.
- Do not broaden scope beyond SPEC-069/SPEC-070 without a docs patch and review.
