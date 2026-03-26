# TASK-271: Phase 45 Closeout Verification

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Verify Phase 45 (Syntax Reduction) complete and specifications ready for implementation.

**Spec Reference:** PHASE-44-46-ROADMAP.md, SPEC-024

**File Locations:**
- `docs/spec/SPEC-024-CAPABILITY-ROLE-REDUCED.md`
- `docs/design/DESIGN-014-SYNTAX-REDUCTION.md`
- `docs/spec/SPEC-017-CAPABILITY.md`

---

## Background

Phase 45 produces the canonical reduced syntax specification. This task verifies:
1. SPEC-024 is complete and consistent
2. DESIGN-014 documents decisions
3. SPEC-017 is updated with constraint syntax
4. Specifications are ready for Phase 46 implementation

---

## Step 1: Verify SPEC-024 Completeness

Check SPEC-024 has all required sections:

```bash
grep "^## [0-9]" docs/spec/SPEC-024-CAPABILITY-ROLE-REDUCED.md
```

Required sections:
- [ ] 1. Overview
- [ ] 2. Capability Definitions
- [ ] 3. Role Definitions
- [ ] 4. Workflow Definitions
- [ ] 5. Lowering Semantics
- [ ] 6. Deferred Features
- [ ] 7. References

---

## Step 2: Verify DESIGN-014

Check design decision record:

```bash
cat docs/design/DESIGN-014-SYNTAX-REDUCTION.md
```

Required:
- [ ] Context explained
- [ ] Decision stated
- [ ] Consequences (positive/negative)
- [ ] Alternatives considered

---

## Step 3: Verify SPEC-017 Updated

Check constraint section added:

```bash
grep -n "Constraint" docs/spec/SPEC-017-CAPABILITY.md
```

Required:
- [ ] Section 5: Constraint Refinement exists
- [ ] `@ { constraints }` syntax documented
- [ ] References SPEC-024

---

## Step 4: Cross-Reference Verification

Ensure specifications are consistent:

| SPEC-024 Feature | SPEC-017 Reference | Lowering Doc | Test |
|------------------|-------------------|--------------|------|
| `plays role(R)` | ✓ | ✓ | TASK-259 |
| `capabilities: [...]` | ✓ | ✓ | TASK-260 |
| `@ { constraints }` | ✓ | ✓ | TASK-260 |
| Implicit role | ✓ | ✓ | TASK-261 |

---

## Step 5: Codex Phase Audit (REQUIRED)

```
delegate_task to codex:
  goal: "Phase 45 Closeout Audit"
  context: |
    Phase: 45 - Syntax Reduction Specification
    Tasks: TASK-257, TASK-258
    Deliverable: Canonical reduced syntax specification
    
    Audit Checklist:
    
    1. SPEC-024-CAPABILITY-ROLE-REDUCED.md
       - All 7 sections present
       - Grammar unambiguous
       - Examples valid
       - Lowering semantics clear
       - Deferred features listed
       - References valid
    
    2. DESIGN-014-SYNTAX-REDUCTION.md
       - Context explained
       - Decision stated
       - Consequences documented
       - Alternatives considered
    
    3. SPEC-017-CAPABILITY.md
       - Constraint section added
       - @ { constraints } documented
       - References SPEC-024
       - Consistent with reduced syntax
    
    4. Consistency Check
       - SPEC-024 matches PHASE-44-46-ROADMAP decisions
       - No contradictions between specs
       - Grammar matches parser expectations
    
    5. Implementation Readiness
       - Grammar implementable
       - Lowering rules clear
       - Test cases derivable
    
    Report Format:
    STATUS: [PASS / PARTIAL / FAIL]
    
    SPEC-024: [VERIFIED / ISSUES: ...]
    DESIGN-014: [VERIFIED / ISSUES: ...]
    SPEC-017 Update: [VERIFIED / ISSUES: ...]
    
    Consistency: [VERIFIED / ISSUES: ...]
    Readiness: [READY / NOT READY: ...]
    
    Blockers: [list if any]
    
    Recommendation: [PROCEED TO PHASE 46 / REVISE SPECS]
```

---

## Step 6: Update PLAN-INDEX.md

Once audit passes:

```markdown
| Phase | Tasks | Completed | Status |
|-------|-------|-----------|--------|
| ... | ... | ... | ... |
| 45 | 3 | 3 | ✅ Complete |
```

---

## Step 7: Commit

```bash
git add docs/plan/PLAN-INDEX.md
git commit -m "docs: mark Phase 45 complete (TASK-271)

- SPEC-024: Reduced capability-role syntax canonicalized
- DESIGN-014: Syntax reduction decisions documented
- SPEC-017: Updated with constraint syntax
- All specifications consistent and ready for implementation
- Codex phase audit passed

Phase 45 Deliverable: Canonical reduced syntax specification"
```

---

## Completion Checklist

- [ ] SPEC-024 sections verified
- [ ] DESIGN-014 complete
- [ ] SPEC-017 updated
- [ ] Cross-reference consistent
- [ ] **Codex phase audit passed**
- [ ] PLAN-INDEX.md updated
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 2
**Blocked by:** TASK-257, TASK-258
**Blocks:** Phase 46
