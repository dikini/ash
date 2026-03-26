# TASK-272: Phase 46 Closeout Verification

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Verify Phase 46 (Unified Capability-Role Implementation) complete.

**Spec Reference:** PHASE-44-46-ROADMAP.md, SPEC-024

**File Locations:**
- All Phase 46 implementation files
- `docs/plan/PLAN-INDEX.md`

---

## Background

Phase 46 implements the reduced syntax specification. This task verifies:
1. All parser extensions work (TASK-259, 260, 261)
2. Type system integration complete (TASK-262, 263, 264)
3. Runtime integration functional (TASK-265, 266, 267)
4. Optional agent harness implemented (TASK-268, 269, 270)
5. Full system integration tests pass

---

## Step 1: Verify Task Completion

Check all Phase 46 tasks:

```bash
for task in 259 260 261 262 263 264 265 266 267 268 269 270; do
    echo "=== TASK-$task ==="
    git log --oneline | grep "TASK-$task" | head -1 || echo "NOT FOUND"
done
```

---

## Step 2: Run Full Test Suite

### Parser Tests

```bash
cargo test --package ash-parser plays_role -v
cargo test --package ash-parser capability_constraint -v
cargo test --package ash-parser implicit_role -v
```

### Type System Tests

```bash
cargo test --package ash-typeck role_type -v
cargo test --package ash-typeck constraint -v
cargo test --package ash-typeck effective_caps -v
```

### Runtime Tests

```bash
cargo test --package ash-interp role_runtime -v
cargo test --package ash-interp constraint_enforcement -v
cargo test --package ash-interp yield_routing -v
```

### Harness Tests (Optional)

```bash
cargo test --package ash-core agent_harness -v
cargo test --package ash-engine harness -v
cargo test --package ash-engine mcp -v
```

---

## Step 3: Full Integration Test

Create end-to-end test:

```rust
// tests/integration/capability_role_workflow.rs
#[test]
fn test_full_capability_role_workflow() {
    // Parse workflow with plays role and capabilities
    // Type check
    // Execute with role resolution
    // Verify constraints enforced
}
```

---

## Step 4: Quality Gates

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --check
cargo doc --workspace --no-deps
cargo test --workspace
```

All must pass.

---

## Step 5: Codex Phase Audit (REQUIRED)

```
delegate_task to codex:
  goal: "Phase 46 Closeout Audit"
  context: |
    Phase: 46 - Unified Capability-Role Implementation
    Tasks: TASK-259 through TASK-270
    Deliverable: Unified capability-role-workflow system
    
    Comprehensive Verification:
    
    1. Parser (TASK-259, 260, 261)
       - 'plays role(R)' parses
       - 'capabilities: [...] @ {...}' parses
       - Implicit role generated in lowering
       - All syntax tests pass
    
    2. Type System (TASK-262, 263, 264)
       - Role existence checked
       - Capabilities composed from roles
       - Constraints validated
       - Effective capability sets computed
       - All type tests pass
    
    3. Runtime (TASK-265, 266, 267)
       - Roles resolve to grants
       - Constraints enforced at runtime
       - Yield routes to role handlers
       - All runtime tests pass
    
    4. Harness (TASK-268, 269, 270) - Optional
       - Agent harness capability defined
       - Harness operations work
       - MCP provider functional
       - All harness tests pass
    
    5. Integration
       - End-to-end workflow with roles
       - Capability checks work
       - Constraint enforcement works
       - Yield/resume works
    
    6. Quality Gates
       - clippy -D warnings: clean
       - cargo fmt --check: clean
       - cargo doc: no warnings
       - cargo test --workspace: all pass
    
    Report Format:
    STATUS: [PASS / PARTIAL / FAIL]
    
    Component Status:
    - Parser: [PASS/FAIL]
    - Type System: [PASS/FAIL]
    - Runtime: [PASS/FAIL]
    - Harness (optional): [PASS/FAIL/N/A]
    - Integration: [PASS/FAIL]
    
    Quality Gates:
    - Clippy: [PASS/FAIL]
    - Format: [PASS/FAIL]
    - Doc: [PASS/FAIL]
    - Tests: [PASS/FAIL]
    
    SPEC Compliance:
    - SPEC-024: [PASS/FAIL]
    - SPEC-017: [PASS/FAIL]
    - SPEC-019: [PASS/FAIL]
    
    Blockers: [list if any]
    
    Recommendation: [MARK COMPLETE / FIX BLOCKERS]
```

---

## Step 6: Update PLAN-INDEX.md

```markdown
| Phase | Tasks | Completed | Status |
|-------|-------|-----------|--------|
| ... | ... | ... | ... |
| 46 | 14 | 14 | ✅ Complete |
```

---

## Step 7: Final Commit

```bash
git add docs/plan/PLAN-INDEX.md
git commit -m "docs: mark Phase 46 complete (TASK-272)

- Phase 46.1 Parser: plays role, capabilities with constraints, implicit roles
- Phase 46.2 Type System: role checking, constraint validation, effective caps
- Phase 46.3 Runtime: role resolution, constraint enforcement, yield routing
- Phase 46.4 Harness: agent harness capability, operations, MCP provider
- All quality gates passing
- Codex phase audit passed
- Full system integration verified

Phase 44-46 Deliverable: Unified capability-role-workflow system complete.
All audit issues resolved. Reduced syntax implemented."
```

---

## Completion Checklist

- [ ] All Phase 46 tasks verified
- [ ] Parser tests pass
- [ ] Type system tests pass
- [ ] Runtime tests pass
- [ ] Harness tests pass (if implemented)
- [ ] Integration tests pass
- [ ] Quality gates pass
- [ ] **Codex phase audit passed**
- [ ] PLAN-INDEX.md updated
- [ ] CHANGELOG.md updated

---

**Estimated Hours:** 4
**Blocked by:** TASK-259 through TASK-270
**Blocks:** None (Phase 44-46 complete)
