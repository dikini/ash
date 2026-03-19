# TASK-175: Align ADT Stdlib and Example Surface

## Status: 📝 Planned

## Description

Align the standard library and examples with the canonical Option/Result helper surface.

This task ensures the user-visible ADT helper surface matches the stabilized spec and the
aligned runtime/type layers.

## Specification Reference

- SPEC-020: ADT Types

## Reference Contract

- `docs/reference/runtime-observable-behavior-contract.md`

## Requirements

### Functional Requirements

1. Expose the canonical Option/Result helper surface in the stdlib
2. Align examples and prelude-visible surface with the canonical helper set
3. Add tests proving the canonical stdlib surface

## Files

- Modify: `std/src/option.ash`
- Modify: `std/src/result.ash`
- Modify: `std/src/prelude.ash`
- Modify: `examples/README.md`
- Test: `crates/ash-parser/tests/stdlib_surface.rs`
- Test: `tests/std/`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add tests proving the standard library and examples expose the canonical Option/Result helper surface.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-parser stdlib_surface -- --nocapture
cargo test --test '*'
```

Expected: at least one failure for missing helper surface.

### Step 3: Implement the minimal fix (Green)

Add only the canonical helper surface and example updates required by the spec.

### Step 4: Verify focused GREEN

Run:

```bash
cargo test -p ash-parser stdlib_surface -- --nocapture
```

Expected: pass.

### Step 5: Verify broader GREEN

Run:

```bash
cargo test --workspace
```

Expected: pass.

### Step 6: Commit

```bash
git add std/src/option.ash std/src/result.ash std/src/prelude.ash examples/README.md tests/std CHANGELOG.md
git commit -m "feat: align adt stdlib surface"
```

## Completion Checklist

- [ ] failing stdlib-surface tests added
- [ ] failure verified
- [ ] canonical stdlib helper surface aligned
- [ ] examples aligned
- [ ] focused and broader verification passing
- [ ] `CHANGELOG.md` updated

## Non-goals

- No new ADT families beyond the canonical spec
- No parser/type/runtime redesign beyond exposed helper surface

## Dependencies

- Depends on: TASK-163, TASK-174
- Blocks: TASK-176
