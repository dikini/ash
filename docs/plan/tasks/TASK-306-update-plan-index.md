# TASK-306: PLAN-INDEX.md Finalization

## Status: 📝 Planned

## Description

Final update to PLAN-INDEX.md to reflect completed Phase 48 tasks and add Phase 49 with proper structure.

## Phase 48 Status Updates Required

All 10 Phase 48 tasks should be marked ✅ Complete:

| Task | Current | Target |
|------|---------|--------|
| TASK-284 | ✅ Complete | Keep ✅ |
| TASK-285 | ✅ Complete | Keep ✅ |
| TASK-286 | ✅ Complete | Keep ✅ |
| TASK-287 | ✅ Complete | Keep ✅ |
| TASK-288 | ✅ Complete | Keep ✅ |
| TASK-289 | 📝 Planned (in 48.1) | ✅ Complete |
| TASK-290 | 📝 Planned (in 48.1) | ✅ Complete |
| TASK-291 | 📝 Planned (in 48.1) | ✅ Complete |
| TASK-292 | 📝 Planned (in 48.2) | 🟡 Partial (tests ready, needs TASK-299) |
| TASK-293 | 📝 Planned (in 48.2) | ✅ Complete |
| TASK-294 | 📝 Planned (in 48.2) | ✅ Complete |
| TASK-295 | 📝 Planned (in 48.4) | ✅ Complete |
| TASK-296 | 📝 Planned (in 48.4) | ✅ Complete |
| TASK-297 | 📝 Planned (in 48.5) | ✅ Complete |
| TASK-298 | 📝 Planned (in 48.5) | ✅ Complete |

## Phase 49 Addition

Add new section:

```markdown
## Phase 49: Phase 48 Integration & Hardening

**Goal:** Complete integration of partially-finished Phase 48 tasks, harden edge cases, and achieve full SPEC compliance for all Phase 48 deliverables.

**Status:** In Progress
**Priority:** High
**Estimated Total:** ~48 hours

### 49.1: CLI Input Integration
| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-299](tasks/TASK-299-type-checker-workflow-parameters.md) | Type checker: bind workflow parameters from input | SPEC-005 | 8 | 📝 Planned |
| [TASK-300](tasks/TASK-300-cli-input-integration-tests.md) | Unignore and verify CLI --input integration tests | SPEC-005 | 4 | 📝 Planned |

### 49.2: Type System Hardening
| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-301](tasks/TASK-301-obligation-branch-semantics.md) | Verify obligation branch/merge semantics are correct | SPEC-022 | 6 | 📝 Planned |
| [TASK-302](tasks/TASK-302-expression-typing-edge-cases.md) | Add edge case tests for expression typing fixes | SPEC-003 | 4 | 📝 Planned |

### 49.3: Integration Test Coverage
| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-303](tasks/TASK-303-engine-provider-e2e-tests.md) | End-to-end tests for engine capability providers | SPEC-010 | 6 | 📝 Planned |
| [TASK-304](tasks/TASK-304-role-semantics-integration-tests.md) | Integration tests for role runtime semantics | SPEC-019 | 6 | 📝 Planned |

### 49.4: Documentation & Changelog Consolidation
| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-305](tasks/TASK-305-changelog-consolidation.md) | Consolidate CHANGELOG.md entries from Phase 48 worktrees | N/A | 2 | 📝 Planned |
| [TASK-306](tasks/TASK-306-update-plan-index.md) | Finalize PLAN-INDEX.md with all completed Phase 48/49 tasks | N/A | 2 | 📝 Planned |
```

## Files to Modify

- `docs/plan/PLAN-INDEX.md`

## Completion Checklist

- [ ] All Phase 48 tasks marked ✅ Complete (except TASK-292 partial)
- [ ] Phase 49 section added with all 8 new tasks
- [ ] Task links point to correct task files
- [ ] Phase 48 summary updated
- [ ] Estimated hours calculated
