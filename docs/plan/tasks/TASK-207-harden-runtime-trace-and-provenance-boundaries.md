# TASK-207: Harden Runtime Trace and Provenance Boundaries

## Status: 📝 Planned

## Description

Harden runtime trace and provenance capture so it tracks accepted runtime progression consistently
across execution boundaries and workflow wrapper framing.

## Specification Reference

- SPEC-001: Intermediate Representation
- SPEC-004: Operational Semantics
- SPEC-021: Runtime Observable Behavior

## Reference Contract

- `docs/plan/2026-03-20-runtime-boundary-steering-brief.md`
- `docs/reference/runtime-observable-behavior-contract.md`
- `docs/reference/runtime-reasoner-separation-rules.md`

## Requirements

### Functional Requirements

1. Align trace/provenance capture with accepted runtime progression and boundary outcomes
2. Tighten workflow wrapper entry/exit framing where it affects runtime-owned provenance
3. Add focused tests for trace/provenance consistency around runtime execution boundaries
4. Preserve the rule that runtime observability is not projection

## Files

- Modify: `crates/ash-provenance/src/lib.rs`
- Modify: `crates/ash-provenance/src/trace.rs`
- Modify: `crates/ash-macros/src/lib.rs`
- Test: `crates/ash-provenance/tests/runtime_trace_boundaries.rs`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add focused tests for:
- workflow entry/exit provenance framing,
- trace event consistency around accepted runtime progression,
- absence of drift between boundary outcomes and trace capture.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-provenance runtime_trace_boundaries -- --nocapture
```

Expected: fail because the targeted trace/provenance hardening is not implemented yet.

### Step 3: Implement the minimal fix (Green)

Tighten runtime trace/provenance capture to match the hardened runtime boundary contract.

### Step 4: Verify focused GREEN

Run:

```bash
cargo test -p ash-provenance runtime_trace_boundaries -- --nocapture
```

Expected: pass.

### Step 5: Verify broader GREEN

Run:

```bash
cargo test -p ash-provenance
```

Expected: pass.

### Step 6: Commit

```bash
git add crates/ash-provenance/src/lib.rs crates/ash-provenance/src/trace.rs crates/ash-macros/src/lib.rs crates/ash-provenance/tests/runtime_trace_boundaries.rs CHANGELOG.md
git commit -m "fix: harden runtime trace and provenance boundaries"
```

## Completion Checklist

- [ ] failing runtime trace/provenance tests added
- [ ] failure verified
- [ ] trace/provenance boundary capture hardened
- [ ] focused and broader verification passing
- [ ] `CHANGELOG.md` updated

## Non-goals

- No CLI trace output redesign
- No explanatory stage-guidance overlays
- No reasoner-context projection behavior

## Dependencies

- Depends on: TASK-206
- Blocks: TASK-176
