# TASK-919: DESIGN-040/041 Current-State and Scope Reconciliation

## Status: ✅ Complete

## Description

Promote DESIGN-040 and DESIGN-041 into the SPEC-069/SPEC-070/PLAN-118 implementation packet, register Phase 122 in PLAN-INDEX, update spec indexes/changelog, and preserve an explicit authority/supersession map before Rust implementation.

## Specification Reference

- [SPEC-069](../../spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md)
- [SPEC-070](../../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)
- [PLAN-118](../PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md)

## Dependencies

- ✅ DESIGN-040 and DESIGN-041 exist
- ✅ SPEC-066 and SPEC-067 are implemented prerequisites

## Requirements

### Functional Requirements

1. Create SPEC-069 and SPEC-070.
2. Create PLAN-118 with task split and supersession map.
3. Create TASK-919 through TASK-932 with Dispatch and Verification sections.
4. Update docs/spec/README.md, PLAN-INDEX, design pointers, and CHANGELOG.md.
5. Verify file existence, ID search, task structure, links, and whitespace.

### Property Requirements

Property tests are required for Rust semantic tasks when TASK-920 identifies a stable strategy. Documentation-only or audit tasks must instead provide corpus consistency evidence.

## TDD Steps

### Step 1: Inspect current state

Confirm live docs, IDs, and prerequisite specs.

### Step 2: Write packet docs

Create specs, plan, and task files.

### Step 3: Update indexes

Patch spec index, PLAN-INDEX, design notes, and changelog.

### Step 4: Verify docs packet

Run scoped docs checks and diff whitespace checks.

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
  - test -f docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
  - test -f docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
  - test -f docs/plan/PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md
  - python3 -c 'from pathlib import Path; import re; files=[Path("docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md"), Path("docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md"), Path("docs/plan/PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md"), *sorted(Path("docs/plan/tasks").glob("TASK-9[12][0-9]-*.md")), *sorted(Path("docs/plan/tasks").glob("TASK-93[0-2]-*.md")), Path("docs/spec/README.md"), Path("CHANGELOG.md")]; pat=re.compile(r"\[[^\]]+\]\(([^)]+\.md(?:#[^)]+)?)\)"); bad=[]; [bad.append(f"{p}:{m.group(1)}") for p in files for m in pat.finditer(p.read_text()) if not (p.parent / m.group(1).split("#",1)[0]).resolve().exists()]; assert not bad, bad; print(f"checked {len(files)} files")'
  - git diff --check
checklist:
  - [x] New spec/plan/task files exist
  - [x] Indexes and changelog updated
  - [x] Scoped markdown links checked
  - [x] Diff whitespace checked
```

## Dependencies for Next Task

This task outputs:
- Draft specs and plan.
- Concrete downstream task range.

## Notes

- File targets to inspect or modify: `docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md`, `docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md`, `docs/plan/PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md`.
- Keep PLAN-118 decision gates in sync with implementation reality.
- Do not broaden scope beyond SPEC-069/SPEC-070 without a docs patch and review.
