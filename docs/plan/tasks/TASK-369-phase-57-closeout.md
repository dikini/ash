# TASK-369: Phase 57 Closeout and Verification

## Status: ⛔ Blocked

## Description

Final verification and closeout for Phase 57: Entry Point and Program Execution.

## Closeout Checklist

### Implementation Verification

- [ ] All 57A tasks (S57-1 through S57-7) show ✅ Complete
- [ ] All 57B tasks (359-368a) show ✅ Complete
- [ ] Code compiles: `cargo build --all`
- [ ] Tests pass: `cargo test --all`
- [ ] Clippy clean: `cargo clippy --all-targets --all-features`
- [ ] Format clean: `cargo fmt --check`
- [ ] Docs build: `cargo doc --no-deps`

### Feature Verification

- [ ] Can run: `ash run hello.ash` → exits 0
- [ ] Args work: `ash run echo.ash -- hello world` → receives args
- [ ] Errors reported: `ash run bad.ash` → helpful error, exit 1
- [ ] Exit codes: Program returning `Err(RuntimeError 42 _)` → exits 42
- [ ] Stdlib loads: `use runtime::RuntimeError` works

### SPEC Alignment

- [ ] All implementation matches updated SPEC (57A)
- [ ] No MCE citations in implementation (SPEC is ground truth)
- [ ] S57-7 review complete

### Documentation

- [ ] CHANGELOG.md updated with Phase 57 entries
- [ ] Module-level docs for modified crates
- [ ] Function docs for public APIs
- [ ] README updated with `ash run` usage

### Test Coverage

- [ ] Minimum tests (TASK-368a) pass
- [ ] Property tests for invariants
- [ ] Edge cases covered

## Verification Commands

```bash
# Build
cargo build --release

# Test
cargo test --all

# Lint
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check

# Docs
cargo doc --no-deps

# Integration test
ash run examples/hello.ash
echo $?  # Should be 0

# Error test
ash run tests/fixtures/missing_main.ash 2>&1 | grep "no 'main' workflow"
echo $?  # Should be 1
```

## Sign-off

| Component | Status | Notes |
|-----------|--------|-------|
| 57A SPEC updates | ⬜ | All S57 tasks |
| ash-std extensions | ⬜ | TASK-359, 360, 361, 362 |
| Runtime bootstrap | ⬜ | TASK-363a, 363b, 363c |
| Entry verification | ⬜ | TASK-364 |
| Exit handling | ⬜ | TASK-365 |
| CLI semantics | ⬜ | TASK-366, 367 |
| Tests | ⬜ | TASK-368a (minimum) |
| Review | ⬜ | TASK-S57-7 |

## Deliverables Summary

**Modified crates:**
- `ash-std`: Extended with runtime modules
- `ash-cli`: Updated `run` command semantics
- `ash-engine`: Bootstrap integration (if modified)

**Documentation:**
- Entry point specification (grounded in SPEC)
- CLI usage guide
- Stdlib organization

**Tests:**
- Unit tests: per-component
- Integration tests: minimum entry point flow

## Post-Closeout

- Update PLAN-INDEX Phase 57 status to ✅ Complete
- Link from CHANGELOG
- Extended tests (TASK-368b) remain deferred

## Est. Hours: 1

## Note on MCE-001

MCE-001 was **guidance** for this work. After 57A, **SPEC is ground truth**.
Do not mark MCE-001 as "accepted" - exploration documents serve their purpose during design.
Archive MCE-001 to `docs/ideas/archived/` if desired, but do not treat as normative.
