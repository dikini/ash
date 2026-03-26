# TASK-256: Phase 44 Closeout Verification

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Verify all Phase 44 tasks complete and audit issues resolved.

**Spec Reference:** docs/audit/codex-comprehensive-review.md, PHASE-44-46-ROADMAP.md

**File Locations:**
- All modified files from TASK-240 through TASK-255
- `docs/plan/PLAN-INDEX.md`
- `CHANGELOG.md`

---

## Background

Phase 44 is the critical audit convergence phase. This task ensures:
1. All 16 tasks (TASK-240 to TASK-255) are verified complete
2. Audit findings are addressed
3. Quality gates pass
4. Documentation is current

---

## Step 1: Verify Task Completion

Check each task status:

```bash
for task in 240 241 242 243 244 245 246 247 248 249 250 251 252 253 254 255; do
    echo "=== TASK-$task ==="
    git log --oneline --all | grep "TASK-$task" | head -1
done
```

Verify all have commits.

---

## Step 2: Run Full Quality Gates

### Clippy

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
echo "Exit code: $?"
```

Expected: 0

### Format

```bash
cargo fmt --check
echo "Exit code: $?"
```

Expected: 0

### Documentation

```bash
cargo doc --workspace --no-deps 2>&1 | grep -c warning
echo "Warning count should be 0"
```

Expected: 0

### Tests

```bash
cargo test --workspace
echo "Exit code: $?"
```

Expected: 0 (all tests pass)

---

## Step 3: Verify Audit Items Addressed

Cross-reference with audit report:

| Audit Item | Task | Status |
|------------|------|--------|
| SPEC-022 obligations not executable | 240, 241 | ⬜ |
| Proxy/yield placeholders | 242, 243, 244 | ⬜ |
| SmtContext unsafe Send/Sync | 245 | ⬜ |
| EngineBuilder no-ops | 246 | ⬜ |
| Stub providers | 247 | ⬜ |
| Role obligation discharge | 248 | ⬜ |
| Clippy warnings | 249 | ⬜ |
| Fmt failures | 250 | ⬜ |
| Doc warnings | 251 | ⬜ |
| unexpected_cfgs | 252 | ⬜ |
| Float truncation | 253 | ⬜ |
| Trace flags decorative | 254 | ⬜ |
| Stale docs | 255 | ⬜ |

All must be ✅

---

## Step 4: Codex Phase Audit (REQUIRED)

Spawn comprehensive codex audit:

```
delegate_task to codex:
  goal: "Phase 44 Closeout Audit"
  context: |
    Phase: 44 - Audit Convergence
    Tasks: TASK-240 through TASK-255
    Audit Report: docs/audit/codex-comprehensive-review.md
    
    Comprehensive Verification:
    
    1. Clone repo and checkout phase-44 branch
    2. Verify all tasks have commits:
       git log --oneline | grep -E "TASK-(240|241|242|243|244|245|246|247|248|249|250|251|252|253|254|255)"
    
    3. Run quality gates:
       cargo clippy --workspace --all-targets --all-features -- -D warnings
       cargo fmt --check
       cargo doc --workspace --no-deps
       cargo test --workspace
    
    4. Verify specific audit items:
       - Oblige/CheckObligation execution (grep execute.rs)
       - Yield lowering (grep lower.rs)
       - SmtContext safety (grep smt.rs for unsafe impl)
       - EngineBuilder real methods (grep lib.rs)
       - Providers real (grep providers.rs for Value::Null)
       - Role discharge validation (grep role_context.rs)
    
    5. Check documentation:
       - CHANGELOG.md updated
       - PLAN-INDEX.md Phase 44 marked complete
    
    Report Format:
    STATUS: [PASS / PARTIAL / FAIL]
    
    Tasks Verified:
    - TASK-240: [commit hash] [PASS/FAIL]
    - TASK-241: [commit hash] [PASS/FAIL]
    ...
    
    Quality Gates:
    - Clippy: [PASS/FAIL]
    - Format: [PASS/FAIL]
    - Doc: [PASS/FAIL]
    - Tests: [PASS/FAIL]
    
    Audit Items:
    - SPEC-022 obligations: [PASS/FAIL]
    - Proxy/yield: [PASS/FAIL]
    - SmtContext: [PASS/FAIL]
    - EngineBuilder: [PASS/FAIL]
    - Providers: [PASS/FAIL]
    - Role discharge: [PASS/FAIL]
    - Quality gates: [PASS/FAIL]
    - Float handling: [PASS/FAIL]
    - Trace flags: [PASS/FAIL]
    - Documentation: [PASS/FAIL]
    
    Blockers (if any): [list]
    
    Recommendation: [PROCEED TO PHASE 45 / FIX BLOCKERS]
```

---

## Step 5: Update PLAN-INDEX.md

Once codex audit passes:

```markdown
<!-- docs/plan/PLAN-INDEX.md -->
| Phase | Tasks | Completed | Status |
|-------|-------|-----------|--------|
| ... | ... | ... | ... |
| 44 | 16 | 16 | ✅ Complete |
```

---

## Step 6: Final Commit

```bash
git add docs/plan/PLAN-INDEX.md
git commit -m "docs: mark Phase 44 complete (TASK-256)

- All 16 audit convergence tasks verified complete
- Quality gates passing:
  - clippy -D warnings: clean
  - cargo fmt --check: clean
  - cargo doc: no warnings
  - cargo test --workspace: all pass
- Audit findings addressed:
  - SPEC-022 obligations executable
  - Proxy/yield end-to-end working
  - SmtContext safe threading
  - EngineBuilder real configuration
  - Providers real implementations
  - Role discharge validated
  - Float handling explicit errors
  - Trace flags functional
  - Documentation current

Phase 44 Deliverable: All audit issues resolved."
```

---

## Completion Checklist

- [ ] All 16 tasks verified complete
- [ ] Clippy passes
- [ ] Format clean
- [ ] Doc warnings eliminated
- [ ] All tests pass
- [ ] Audit items cross-referenced
- [ ] **Codex phase audit passed**
- [ ] PLAN-INDEX.md updated
- [ ] CHANGELOG.md updated

---

**Estimated Hours:** 4
**Blocked by:** TASK-240 through TASK-255
**Blocks:** Phase 45 (Syntax Reduction)

**Note:** This task only passes when codex phase audit reports STATUS: PASS
