# Track D Tasks: Integration and Verification

## TASK-669: End-to-End Multi-File Workflow Execution Tests

**Track:** D
**Depends on:** Track A complete, Track B complete
**Est. Hours:** 4-6

Write comprehensive end-to-end tests for multi-file workflow execution:

1. Two-file project: `main.ash` imports `utils.ash`
2. Three-file project with nested modules: `main.ash` → `lib/mod.ash` + `lib/helpers.ash`
3. Stdlib integration: `main.ash` uses `std::json`, `std::list`, `std::io::stdio`
4. Mixed: `main.ash` imports local file AND stdlib module
5. Entry workflow with imports: canonical entry that uses `std::runtime`
6. Error scenarios: missing import, circular import, type mismatch across files
7. CLI integration: `ash run`, `ash check`, `ash trace` all work with multi-file projects

**Files:**
- Create: `crates/ash-engine/tests/multi_file_execution.rs`
- Create: `crates/ash-cli/tests/multi_file.rs`
- Create test fixtures under `tests/fixtures/multi_file_project/`

**Verification:** All tests pass. Multi-file workflows execute correctly.

---

## TASK-670: Capability Boundary Audit

**Track:** D
**Depends on:** Track B complete, Track C complete
**Est. Hours:** 3-4

Audit the complete capability surface after all new builtins and providers:

1. Every `builtin fn` declaration has a corresponding Rust handler
2. Every provider has constraint enforcement
3. Effect levels are correctly classified (IO = Operational, JSON = Epistemic, etc.)
4. Capability boundary document updated
5. Spec processor's `capability_boundary.rs` checks are updated for new providers

**Files:**
- Modify: `docs/notes/NOTE-004-STDLIB-BUILTIN-GAP.md` (mark gaps as closed)
- Modify: `apps/spec_processor/src/capability_boundary.rs` (new provider checks)
- Create: `docs/reference/CAPABILITY-SURFACE.md` (authoritative list)

**Verification:** Spec processor audit passes with zero unexpected gaps.

---

## TASK-671: Performance Baseline and Regression Suite

**Track:** D
**Depends on:** Track A complete
**Est. Hours:** 3-4

Establish performance baselines for the module resolution path:

1. Benchmark: single-file workflow execution (baseline, should not regress)
2. Benchmark: multi-file workflow with 5 imported modules
3. Benchmark: stdlib resolution with 10 stdlib imports
4. Benchmark: large module graph (20+ files)
5. Cold-start vs warm-start resolver performance
6. Memory usage profiling for module graph

**Files:**
- Create: `crates/ash-bench/benches/module_resolution.rs`
- Modify: `crates/ash-bench/` as needed

**Verification:** Baseline numbers recorded. No regression >10% on single-file execution.
