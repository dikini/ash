# TASK-858: Associated family audit gate

## Status: ✅ Complete

## Description

Audit live associated projection, impl registration/selection, normalizer, and semantic-summary seams before any Rust implementation.

## Specification Reference

- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [PLAN-111: Associated Type-Family Computation](../PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [DESIGN-034 §16.7](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)

## Dependencies

- ✅ TASK-857: SPEC-G spec/plan packet (complete)

## Files / Ownership

- Create: `docs/plan/audits/TASK-858-associated-family-computation-audit.md`
- Inspect: `crates/ash-parser/src/surface.rs`, `crates/ash-parser/src/parse_module.rs`, `crates/ash-parser/src/parse_type_def.rs`, `crates/ash-parser/src/lower.rs`
- Inspect: `crates/ash-core/src/type_ir.rs`, `crates/ash-core/src/semantic_summary.rs`, `crates/ash-core/src/ast.rs`
- Inspect: `crates/ash-typeck/src/type_env.rs`, `crates/ash-typeck/src/types.rs`, `crates/ash-typeck/src/normalizer.rs`, `crates/ash-typeck/src/error.rs`
- Inspect: `crates/ash-engine/src/module_loader.rs`
- Update or explicitly confirm exact bindings in downstream task files `TASK-859` through `TASK-868` before any Rust implementation begins.

## Requirements

### Functional Requirements

1. Create `docs/plan/audits/TASK-858-associated-family-computation-audit.md` with exact live call graph and owner mapping.
2. Map current SPEC-035 substitution paths, `Type::Associated` carriers/lowering, canonical projection conversion, normalizer projection handling, summary import/export seams, TypeEnv error/span extraction, core AST/lowering pass-through status, and module-owner context gaps.
3. Produce a forcing/selection matrix assigning each future reduction/diagnostic site to TASK-859 through TASK-868.
4. Record any live-code drift from SPEC-063 before code changes begin.
5. Bind each downstream TASK-859 through TASK-868 to exact source files, exact test targets, exact callsite/audit-row IDs, and zero-test-safe verification commands, either by patching the task files or recording an explicit no-change confirmation in the audit artifact.
6. State that no TASK-859+ Rust implementation starts until requirement 5 is complete.

### Non-Goals

- Do not implement SPEC-H proposition solving, type-function inversion, proof search, or HKT/hole support.
- Do not move semantic ownership into `ash-parser` or `ash-engine`.
- Preserve existing SPEC-035 simple associated type behavior unless this task explicitly assigns a compatibility bridge.

## TDD / Execution Steps

### Step 1: Audit live code

- Inspect `crates/ash-parser/src/surface.rs`, `parse_type*.rs`, `lower.rs`.
- Inspect `crates/ash-core/src/type_ir.rs`, `semantic_summary.rs`, and `ast.rs` to confirm whether core AST remains compatibility-only or needs raw carrier updates.
- Inspect `crates/ash-typeck/src/type_env.rs`, `types.rs`, normalizer modules, and associated-type tests.
- Inspect `crates/ash-engine/src/module_loader.rs` summary transport.

### Step 2: Write audit artifact

- Create the audit file with tables: current carriers, gaps, exact callsites, forcing points, summary seams, parser surfaces, TypeEnv/module-owner context, diagnostics/span extraction seams, and non-interference risks.
- Include one downstream binding table with this exact header: `| Task | Source files | Test targets | Callsite/audit-row IDs | Task-file action |`.
- The downstream binding table must contain one non-empty row for each TASK-859 through TASK-868. `Task-file action` must say either `patched` or `confirmed unchanged` for each row.
- Patch downstream task files immediately if the audit changes their file/test/callsite ownership.

### Step 3: Verify audit

- Run docs link/trailing-whitespace checks and `cargo check --workspace`.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-063, PLAN-111, and the changed files. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [x] Requirements above are satisfied.
- [x] Focused tests/evidence exist and pass, or docs-only verification is recorded.
- [x] Negative leakage/non-interference behavior is covered for this task's surface.
- [x] Status docs and CHANGELOG.md are updated if this task changes release-facing docs.
- [x] Independent verification completed or scheduled by the closeout task.

## Completion Evidence

- Created `docs/plan/audits/TASK-858-associated-family-computation-audit.md` with live parser/core/typeck/normalizer/engine seam inventory, forcing matrix, non-interference risks, and the required downstream binding table for TASK-859 through TASK-868.
- Verification evidence (2026-05-12): `cargo fmt --check`, `git diff --check`, `cargo check --workspace`, scoped Markdown link/trailing-whitespace check from this task file, and downstream binding-table structural check all passed after the audit/status docs were updated.
- The implementation did not modify Rust source code; TASK-859+ Rust work remains gated on this completed audit artifact.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
  - |
    python3 - <<'PY'
    import re, sys
    from pathlib import Path
    files = [
        Path('docs/plan/audits/TASK-858-associated-family-computation-audit.md'),
        Path('docs/plan/tasks/TASK-858-associated-family-audit-gate.md'),
        Path('docs/spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md'),
        Path('docs/plan/PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md'),
    ]
    files += sorted(Path('docs/plan/tasks').glob('TASK-85[9]-*.md'))
    files += sorted(Path('docs/plan/tasks').glob('TASK-86[0-8]-*.md'))
    link = re.compile(r'(?<!\!)\[[^\]]+\]\(([^)]+)\)')
    bad = []
    for path in files:
        if not path.exists():
            bad.append(f'{path}: missing')
            continue
        text = path.read_text()
        in_fence = False
        for line_no, line in enumerate(text.splitlines(), 1):
            if line.rstrip() != line:
                bad.append(f'{path}:{line_no}: trailing whitespace')
            if line.strip().startswith('```'):
                in_fence = not in_fence
                continue
            if in_fence:
                continue
            for match in link.finditer(line):
                target = match.group(1).split('#', 1)[0]
                if not target or re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:', target):
                    continue
                if not (path.parent / target).exists():
                    bad.append(f'{path}:{line_no}: broken link {target}')
    if bad:
        print('\n'.join(bad))
        sys.exit(1)
    PY
  - |
    python3 - <<'PY'
    import re, sys
    from pathlib import Path
    audit = Path('docs/plan/audits/TASK-858-associated-family-computation-audit.md')
    if not audit.exists():
        print(f'{audit}: missing')
        sys.exit(1)
    text = audit.read_text()
    missing = []
    required_header = '| Task | Source files | Test targets | Callsite/audit-row IDs | Task-file action |'
    if required_header not in text:
        missing.append(f'audit artifact missing exact downstream binding header: {required_header}')
    task_rows = {}
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped.startswith('|') or 'TASK-' not in stripped:
            continue
        cells = [cell.strip() for cell in stripped.strip('|').split('|')]
        if cells and re.fullmatch(r'TASK-86[0-8]|TASK-859', cells[0]):
            task_rows[cells[0]] = cells
    for task_id in range(859, 869):
        task = f'TASK-{task_id}'
        cells = task_rows.get(task)
        if cells is None:
            missing.append(f'{task}: missing downstream binding table row')
            continue
        if len(cells) < 5:
            missing.append(f'{task}: binding row must have at least 5 columns')
            continue
        labels = ['source files', 'test targets', 'callsite/audit-row IDs', 'task-file action']
        for offset, label in enumerate(labels, 1):
            if not cells[offset] or cells[offset].lower() in {'tbd', 'todo', 'unknown', 'n/a'}:
                missing.append(f'{task}: empty or placeholder {label}')
        action = cells[4].lower()
        if 'patched' not in action and 'confirmed unchanged' not in action:
            missing.append(f'{task}: task-file action must say patched or confirmed unchanged')
    if missing:
        print('\n'.join(missing))
        sys.exit(1)
    PY
checklist:
  - "[ ] Implementation matches SPEC-063 and PLAN-111 scope"
  - "[ ] Focused tests/evidence for this task pass"
  - "[ ] No SPEC-H/proof-search/type-function-inversion behavior added"
```

## Dependencies for Next Task

This task outputs:
- Audit artifact and exact callsite matrix consumed by every downstream implementation task.
