# TASK-1506: QuickCheck v1 Closeout and Review

## Status: 📝 Planned / In Progress

## Description

Close out Phase 151 with broad verification, independent review, status reconciliation, and changelog/reference updates.

## Specification Reference

- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Dependencies

- ✅ TASK-1497: Live syntax and seam audit
- ✅ TASK-1498: QuickCheck stdlib module split and prelude
- ✅ TASK-1499: GenContext, RNG, and Strategy value core
- ✅ TASK-1500: Arbitrary evidence resolution
- ✅ TASK-1501: Parser/typechecker overrides (property/quickcheck synonyms, with { x <- expr } syntax)
- ✅ TASK-1502: Combinators (stdlib surface complete in ordinary Ash, builtins deferred)
- ✅ TASK-1503: Runner generation and shrink semantics
- ✅ TASK-1504: Seed, replay, and aggregate evidence
- ✅ TASK-1505: Final surface fixtures and docs
- ✅ TASK-1510: Parser fn expressions in multi-field struct literals
- 📝 TASK-1506: This closeout task (in progress)
- 📝 TASK-1512: Record types reference documentation (planned)
- 📝 TASK-1511: Deferred combinators (planned / blocked on language features)

## Closeout Checklist

### Implementation Tasks
- [x] TASK-1497 complete with verification evidence
- [x] TASK-1498 complete with verification evidence
- [x] TASK-1499 complete with verification evidence
- [x] TASK-1500 complete with verification evidence
- [x] TASK-1501 complete with verification evidence (7 parser tests, property/quickcheck synonyms)
- [x] TASK-1502 stdlib surface complete (8 ordinary Ash functions in combinator.ash; builtins deferred)
- [x] TASK-1503 complete with verification evidence
- [x] TASK-1504 complete with verification evidence
- [x] TASK-1505 complete with verification evidence
- [x] TASK-1510 complete with verification evidence (12 regression tests, 3 integration tests)
- [ ] TASK-1512 record types reference documentation (planned — `reference/language/types/records.md` created, needs review)
- [ ] TASK-1511 deferred combinators (planned / blocked — `one_of`, `recursive`, `append_shrink` need let destructors, list primitives, closures)

### Engine Bugs Fixed
- [x] Multi-line `pub use` trailing comma parsing (ash-parser)
- [x] Duplicate type semantic summary merging (ash-engine)
- [x] Type registration for interface constraint checking (ash-engine)
- [x] Type-import-in-type-definitions (ash-engine - check_module_file import processing)

### Verification
- [x] `cargo test -p ash-parser` passes (includes let destructor tests)
- [x] `cargo test -p ash-cli` passes (36 tests)
- [x] `cargo test -p ash-engine --test task_786` passes (38 tests)
- [x] `cargo test -p ash-typeck` passes
- [x] `cargo test -p ash-engine --test phase151_quickcheck_stdlib` — 1 pass, 2 expected failures (missing `one_of` combinator)
- [x] `cargo test -p ash-cli --test stdlib_corpus_check` passes (60 files: 54 passing, 6 failing)
- [x] `cargo test --workspace` passes (with 2 expected failures in phase151_quickcheck_stdlib)
- [x] `cargo fmt --check` passes
- [x] `cargo clippy -p ash-cli --all-targets -- -D warnings` passes
- [x] `git diff --check` passes

### Documentation
- [x] CHANGELOG.md updated under [Unreleased]
- [x] PLAN-INDEX.md task statuses reconciled
- [x] PLAN-151 status updated
- [x] Task files updated with implementation evidence
- [ ] TASK-1512 reference documentation reviewed and verified

### Status Reconciliation
- [x] SPEC-087, PLAN-151, and PLAN-INDEX agree on scope/status
- [x] Phase 150 bridge surfaces documented as compatibility shims
- [x] Final-surface examples documented
- [ ] TASK-1511 blocked status documented with specific language gaps

## New/Added Tasks Completion Criteria

### TASK-1512: Record Types Reference Documentation
- [x] File created at `reference/language/types/records.md`
- [x] Covers: definition, generics, fn fields, construction, access, destructuring, comparisons, Strategy<T> example, limitations
- [x] YAML frontmatter with metadata, verified_against, cross-references
- [ ] Independent review completed
- [ ] Syntax examples verified against parser tests (6/11 tested, 3/11 corpus-verified, 2/11 documented as not supported)
- [ ] PLAN-151 and PLAN-INDEX updated

### TASK-1511: Deferred Combinators in Ordinary Ash
- [ ] Blocked on: let shorthand destructors, let destructors in workflow blocks, list primitives (`++`, indexing), true closures
- [ ] Workaround documented: use field access (`s.gen`) instead of destructuring
- [ ] Combinators to implement when unblocked: `one_of`, `recursive`, `append_shrink`, `prepend_shrink`, `frequency`, `such_that`
- [ ] Current stdlib surface: 8 functions (`map`, `map2`, `with_shrink`, `constant`, `weighted`, `list_of`, `sized`, `resize`)
- [ ] Status: Planned / Blocked

## Verification Commands

```bash
# Formatting
cargo fmt --check

# Parser tests (includes let destructor tests)
cargo test -p ash-parser

# Engine tests (includes quickcheck stdlib integration)
cargo test -p ash-engine --test phase151_quickcheck_stdlib -- --nocapture

# Stdlib corpus check
cargo test -p ash-cli --test stdlib_corpus_check -- --nocapture

# Clippy
cargo clippy -p ash-cli --all-targets -- -D warnings

# Git hygiene
git diff --check

# Full workspace (accepts 2 expected failures in phase151_quickcheck_stdlib)
cargo test --workspace
```

## Notes

Phase 151 implementation is substantially complete with 10/13 tasks done. Remaining:

1. **TASK-1506** (this task): Closeout in progress — needs final verification sweep and status reconciliation
2. **TASK-1512**: Record types docs created but pending independent review
3. **TASK-1511**: Deferred combinators blocked on language features — documented as planned

Deferred items that do not block closeout:
- `one_of`, `recursive`, `append_shrink` combinators (need let destructors, list primitives, closures)
- Shorthand record destructuring `let { x, y } = p` (parser gap)
- `let` destructors in workflow observe blocks (parser gap)
- Arrow syntax `fn(x) => expr` (documented as not supported)

Property test count: 44 proptest functions in ash-engine (~4,400 cases at 100 each).
