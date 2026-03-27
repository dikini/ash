# TASK-322-COORD: Coordination Plan for SPEC-024 Capabilities Syntax Implementation

## Status: 🔴 Blocking - Coordination Task

## Overview

This is the **coordination task** for implementing SPEC-024 `capabilities:` syntax. It orchestrates 6 sub-tasks (322A-322F) plus a final comprehensive review. Each sub-task follows the **Main Agent / Sub-Agent** pattern with dedicated code review.

## Sub-Process Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           TASK-322-COORD                                    │
│                     (Main Coordination Agent)                               │
│  - Sequences sub-tasks in dependency order                                  │
│  - Spawns sub-agents for each implementation task                           │
│  - Spawns review sub-agents after each implementation                       │
│  - Orchestrates final comprehensive review                                  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
        ┌───────────────────────────┼───────────────────────────┐
        ▼                           ▼                           ▼
┌──────────────┐          ┌──────────────┐          ┌──────────────┐
│  Sub-Agent   │          │  Sub-Agent   │          │  Sub-Agent   │
│  (TASK-322A) │          │  (TASK-322B) │          │  (TASK-322C) │
│  AST Change  │          │  Parser      │          │  Type Checker│
└──────────────┘          └──────────────┘          └──────────────┘
        │                           │                           │
        ▼                           ▼                           ▼
┌──────────────┐          ┌──────────────┐          ┌──────────────┐
│  Review Sub  │          │  Review Sub  │          │  Review Sub  │
│  -Agent      │          │  -Agent      │          │  -Agent      │
└──────────────┘          └──────────────┘          └──────────────┘
        │                           │                           │
        └───────────────────────────┼───────────────────────────┘
                                    ▼
                           ┌──────────────┐
                           │  Sub-Agent   │
                           │  (TASK-322D) │
                           │  Runtime     │
                           └──────────────┘
                                   │
                                   ▼
                           ┌──────────────┐
                           │  Review Sub  │
                           │  -Agent      │
                           └──────────────┘
                                   │
                                   ▼
                           ┌──────────────┐
                           │  Sub-Agent   │
                           │  (TASK-322E) │
                           │  Lowering    │
                           └──────────────┘
                                   │
                                   ▼
                           ┌──────────────┐
                           │  Review Sub  │
                           │  -Agent      │
                           └──────────────┘
                                   │
                                   ▼
                           ┌──────────────┐
                           │  Sub-Agent   │
                           │  (TASK-322F) │
                           │  Tests/Integ │
                           └──────────────┘
                                   │
                                   ▼
                           ┌──────────────┐
                           │  Review Sub  │
                           │  -Agent      │
                           └──────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                     FINAL COMPREHENSIVE REVIEW                              │
│  - Full codebase review                                                     │
│  - SPEC-024 compliance verification                                         │
│  - Workspace test verification                                              │
│  - Clippy/format check                                                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Execution Flow

### Phase 1: AST Update (TASK-322A)
```
1. Spawn Implementation Sub-Agent for TASK-322A
   Input: TASK-322A task file
   Output: Modified surface.rs, placeholder updates

2. Verify: cargo test --package ash-parser

3. Spawn Review Sub-Agent
   Input: Git diff of changes
   Output: Review report (Critical/Suggestions/Approval)

4. If changes requested:
   - Fix issues
   - Re-run review
   
5. Mark TASK-322A complete
```

### Phase 2: Parser Update (TASK-322B)
```
1. Spawn Implementation Sub-Agent for TASK-322B
   Input: TASK-322B task file (depends on 322A)
   Output: Modified parse_module.rs

2. Verify: cargo test --package ash-parser

3. Spawn Review Sub-Agent
   Input: Git diff of changes
   Output: Review report

4. Fix issues if any

5. Mark TASK-322B complete
```

### Phase 3: Type Checker (TASK-322C)
```
1. Spawn Implementation Sub-Agent for TASK-322C
   Input: TASK-322C task file
   Output: Modified role_checking.rs

2. Verify: cargo test --package ash-typeck

3. Spawn Review Sub-Agent

4. Fix issues if any

5. Mark TASK-322C complete
```

### Phase 4: Runtime (TASK-322D)
```
1. Spawn Implementation Sub-Agent for TASK-322D
   Input: TASK-322D task file
   Output: Modified role_runtime.rs

2. Verify: cargo test --package ash-interp

3. Spawn Review Sub-Agent

4. Fix issues if any

5. Mark TASK-322D complete
```

### Phase 5: Lowering (TASK-322E)
```
1. Spawn Implementation Sub-Agent for TASK-322E
   Input: TASK-322E task file
   Output: Modified lowering code

2. Verify: cargo test --package ash-lower

3. Spawn Review Sub-Agent

4. Fix issues if any

5. Mark TASK-322E complete
```

### Phase 6: Tests & Integration (TASK-322F)
```
1. Spawn Implementation Sub-Agent for TASK-322F
   Input: TASK-322F task file
   Output: Updated tests, new integration tests

2. Verify: cargo test --workspace

3. Spawn Review Sub-Agent

4. Fix issues if any

5. Mark TASK-322F complete
```

### Phase 7: Final Comprehensive Review
```
1. Spawn Comprehensive Review Sub-Agent
   Input: Full set of changes across all sub-tasks
   
   Review Checklist:
   - [ ] SPEC-024 compliance verified
   - [ ] All 6 sub-task reviews completed
   - [ ] cargo test --workspace passes
   - [ ] cargo clippy --workspace --all-targets clean
   - [ ] cargo fmt --check passes
   - [ ] No new warnings introduced
   - [ ] Documentation updated (if needed)
   - [ ] CHANGELOG.md updated
   - [ ] Integration tests pass
   - [ ] Property tests pass (if applicable)

2. If issues found:
   - Spawn fix sub-agent for specific issues
   - Re-run comprehensive review

3. Mark TASK-322 complete
```

## Sub-Agent Specifications

### Implementation Sub-Agent

**Toolset:** terminal, file, web (as needed)
**Context:** Task file (TASK-322X)
**Instructions:**
```
You are an Implementation Sub-Agent for TASK-322X.

Follow TDD:
1. Read the task file completely
2. Write tests first (as specified in task)
3. Run tests to verify they fail
4. Implement the changes
5. Run tests to verify they pass
6. Run cargo check/clippy
7. Report completion with:
   - Files modified
   - Tests added/passed
   - Any issues encountered
```

### Review Sub-Agent

**Toolset:** terminal, file
**Context:** Git diff of changes
**Instructions:**
```
You are a Review Sub-Agent for TASK-322X.

Review the code changes for:
1. Correctness - does it implement the spec?
2. Safety - any unsafe code, panics, or edge cases?
3. Performance - unnecessary allocations or clones?
4. Idiomatic Rust - follows rust-skills guidelines
5. Test coverage - are edge cases tested?

Run:
- cargo fmt --check
- cargo clippy --package <package>
- cargo test --package <package>

Output format:
## Summary
[1-2 sentence assessment]

## Critical Issues (Must Fix)
- [Issue description + suggested fix]

## Suggestions (Nice to Have)
- [Suggestion description]

## Verification
- [x] cargo fmt --check passes
- [x] cargo clippy clean
- [x] All tests pass
- [Status: APPROVED or REQUEST CHANGES]
```

### Comprehensive Review Sub-Agent

**Toolset:** terminal, file
**Context:** Full workspace changes
**Instructions:**
```
You are a Comprehensive Review Sub-Agent for TASK-322.

Perform final verification:
1. Run cargo test --workspace
2. Run cargo clippy --workspace --all-targets --all-features
3. Run cargo fmt --check
4. Verify SPEC-024 compliance
5. Check integration tests
6. Verify no regressions

Output comprehensive report with:
- Overall assessment
- Any remaining issues
- Final approval or blockers
```

## Coordination Commands

### Spawn Implementation
```bash
# Example: Spawn TASK-322A implementation
# Delegate to sub-agent with task file context
```

### Spawn Review
```bash
# After implementation completes:
# 1. Get git diff
# 2. Delegate to review sub-agent
```

### Track Progress

| Sub-task | Status | Review Status | Blocked By |
|----------|--------|---------------|------------|
| 322A | 🔲 Not Started | - | None |
| 322B | 🔲 Not Started | - | 322A |
| 322C | 🔲 Not Started | - | 322B |
| 322D | 🔲 Not Started | - | 322C |
| 322E | 🔲 Not Started | - | 322D |
| 322F | 🔲 Not Started | - | 322E |
| Final Review | 🔲 Not Started | - | 322F |

## Completion Criteria

All of the following must be true:
- [ ] All 6 sub-tasks marked complete
- [ ] All 6 sub-task reviews approved
- [ ] Comprehensive review approved
- [ ] cargo test --workspace passes
- [ ] cargo clippy --workspace --all-targets --all-features clean
- [ ] cargo fmt --check passes
- [ ] SPEC-024 compliance verified
- [ ] CHANGELOG.md updated

## Estimated Timeline

| Phase | Task | Est. Hours | Cumulative |
|-------|------|------------|------------|
| 1 | 322A + Review | 1-2 | 1-2 |
| 2 | 322B + Review | 2-3 | 3-5 |
| 3 | 322C + Review | 2-3 | 5-8 |
| 4 | 322D + Review | 3-4 | 8-12 |
| 5 | 322E + Review | 2-3 | 10-15 |
| 6 | 322F + Review | 2-3 | 12-18 |
| 7 | Final Review | 1 | 13-19 |

**Total:** 13-19 hours

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Review finds major issues | Loop back to implementation, re-review |
| Integration tests fail | Debug in isolation, fix, re-run |
| Clippy warnings | Fix immediately, don't accumulate |
| SPEC-024 compliance gap | Catch in review, don't proceed until fixed |

## Related

- Parent: TASK-322
- Sub-tasks: TASK-322A through TASK-322F
- Spec: SPEC-024-CAPABILITY-ROLE-REDUCED.md
