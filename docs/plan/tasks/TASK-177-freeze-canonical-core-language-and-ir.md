# TASK-177: Freeze Canonical Core Language and Execution-Neutral IR

## Status: 📝 Planned

## Description

Tighten the canonical core language and IR contract so it is explicit which forms belong to core,
which are surface sugar, and which IR invariants are required regardless of whether Ash is
executed by interpretation now or JIT compilation later.

## Specification Reference

- SPEC-001: IR
- SPEC-002: Surface Language
- SPEC-004: Operational Semantics

## Plan Reference

- `docs/plan/2026-03-19-spec-hardening-design.md`
- `docs/plan/2026-03-19-spec-hardening-plan.md`

## Requirements

### Functional Requirements

1. Define one canonical core-language form set
2. Define which user-facing forms are surface sugar only
3. State IR invariants that are neutral between interpreter and future JIT execution
4. Remove normative ambiguity between core truth and implementation convenience

## Files

- Modify: `docs/spec/SPEC-001-IR.md`
- Modify: `docs/spec/SPEC-002-SURFACE.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- missing core-vs-sugar boundaries,
- interpreter-shaped IR assumptions,
- ambiguous core invariants.

### Step 2: Verify RED

Expected failure conditions:
- at least one form still lacks a clear core-vs-sugar classification,
- the IR contract is not yet explicit about interpreter/JIT neutrality.

### Step 3: Implement the minimal spec fix (Green)

Tighten only the canonical core and IR contract.

### Step 4: Verify GREEN

Expected pass conditions:
- core forms and sugar forms are explicit,
- IR invariants are execution-model-neutral,
- the core spec no longer depends on implementation convenience wording.

### Step 5: Commit

```bash
git add docs/spec/SPEC-001-IR.md docs/spec/SPEC-002-SURFACE.md docs/spec/SPEC-004-SEMANTICS.md CHANGELOG.md
git commit -m "docs: tighten core language and ir contract"
```

## Completion Checklist

- [ ] canonical core forms documented
- [ ] sugar-only forms documented
- [ ] execution-neutral IR invariants documented
- [ ] `CHANGELOG.md` updated

## Non-goals

- No Rust code changes
- No parser/lowering implementation work

## Dependencies

- Depends on: TASK-161, TASK-162, TASK-163
- Blocks: TASK-178, TASK-179, TASK-180, TASK-181, TASK-182, TASK-183, TASK-184
