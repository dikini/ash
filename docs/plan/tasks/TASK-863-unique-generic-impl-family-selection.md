# TASK-863: Unique generic impl-family selection

## Status: 🟡 Ready

## Description

Implement unique associated-family scheme selection and reduction over concrete and abstract argument spines without inversion.

## Specification Reference

- [SPEC-063: Associated Type-Family Computation](../../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [PLAN-111: Associated Type-Family Computation](../PLAN-111-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
- [DESIGN-034 §16.7](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)

## Dependencies

- Depends on TASK-861 completion
- Depends on TASK-862 completion

## Files / Ownership

- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/normalizer.rs` only for selection helper integration points needed by local normalizer lookup
- Create/modify tests: `crates/ash-typeck/tests/task_863_associated_family_selection.rs`

## Requirements

### Functional Requirements

1. Reduce `<Iterator<List<A>>>::Item` to `A` through a unique generic impl scheme.
2. Reduce `<Iterator<List<X>>>::Item` to `X` when `X` is abstract by binding only scheme-owned variables.
3. Reject or block ambiguous selection with structured diagnostics.
4. Implement family selection as one-way matching from scheme head to queried projection spine; never bind caller/environment variables, inference metas, neutral heads, rigid projections, or projected outputs.
5. Add explicit non-inversion tests proving expected output types are not consulted for selection.

### Non-Goals

- Do not implement SPEC-H proposition solving, type-function inversion, proof search, or HKT/hole support.
- Do not move semantic ownership into `ash-parser` or `ash-engine`.
- Preserve existing SPEC-035 simple associated type behavior unless this task explicitly assigns a compatibility bridge.

## TDD / Execution Steps

### Step 1: Write RED tests

- Positive tests for concrete and abstract `List<T>` family reduction.
- Negative tests for ambiguous schemes, neutral-head non-inversion, open queried variables staying opaque, and output-shape comparisons such as `<Append<Xs, Ys>>::Out == Cons<A, Nil>` not binding `Xs` or `Ys`.

### Step 2: Implement selection

- Match canonical interface argument spines structurally.
- Return substitution and selected scheme evidence only on unique match.

### Step 3: Verify non-inversion

- Re-run TASK-825/TASK-829 style non-inversion tests plus focused family selection tests.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-063, PLAN-111, and the changed files. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [ ] Requirements above are satisfied.
- [ ] Focused tests/evidence exist and pass, or docs-only verification is recorded.
- [ ] Negative leakage/non-interference behavior is covered for this task's surface.
- [ ] Status docs and CHANGELOG.md are updated if this task changes release-facing docs.
- [ ] Independent verification completed or scheduled by the closeout task.

## Completion Evidence

- Completion evidence must be recorded by the implementing agent before marking this task complete.

## Dispatch

```yaml
agent: hermes
reasoning: medium
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
    cargo test -p ash-typeck --test task_863_associated_family_selection -- --list | tee /tmp/task_863_associated_family_selection-list.txt
    grep -Eq 'associated_family|task_863' /tmp/task_863_associated_family_selection-list.txt
  - cargo test -p ash-typeck --test task_863_associated_family_selection -- --nocapture
checklist:
  - "[ ] Implementation matches SPEC-063 and PLAN-111 scope"
  - "[ ] Focused tests/evidence for this task pass with non-zero test counts"
  - "[ ] No SPEC-H/proof-search/type-function-inversion behavior added"
```

## Dependencies for Next Task

This task outputs:
- Selected-scheme reduction primitive used by recursive and normalizer integration tasks.
