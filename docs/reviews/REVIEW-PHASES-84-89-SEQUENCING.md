# Review: Phase Sequencing for Phases 84-89

**Review Date:** 2026-04-15  
**Scope:** Phases 84-89 (PLAN-031 through PLAN-035, SPEC-039 through SPEC-043)  
**Reviewer:** CLI Agent

---

## 1. Executive Summary

The phase sequencing for the tooling infrastructure block (84-89) contains **one critical dependency error**, **one missing integration task**, and **several scheduling inefficiencies**. The overall calendar time is dominated by Phase 87 (LSP/MCP, 180 h), so optimizing the pre-87 pipeline and parallelizing non-critical work is the highest-leverage improvement.

**Critical path (unchanged in substance):**  
`84 → 85 → 86 → 87 → {88b, 89}`

**Minimum wall-clock time:** ~6.2 weeks (1 FTE) or ~5.5 weeks with a second developer picking up the formatter core in parallel.

---

## 2. Parallelization Claims

### 2.1 Claim: "Phase 86 can run in parallel with Phases 84-85"
**Verdict: INVALID / HIGH RISK**

Phase 84 (TASK-570) introduces a **breaking AST change**: `Expr::Variable(Name)` becomes `Expr::Variable(Name, Span)` (and the same for `Pattern::Variable`). Phase 86 (TASK-574) requires rewriting `ash-lint` lints as AST visitors. Any visitor that pattern-matches on `Expr::Variable` will fail to compile the moment TASK-570 lands.

*Evidence:*
- TASK-570 explicitly lists `ash-typeck/src/check_expr.rs`, `ash-typeck/src/check_pattern.rs`, etc. as files that must be updated.
- TASK-574 says lint rules must become "AST visitors, not string searches."
- Rust pattern matches are exhaustive; a visitor arm `Expr::Variable(name)` would become a compilation error after TASK-570.

**Recommendation:** Sequence TASK-570 *before* TASK-574. The remainder of Phase 84 (TASK-571, comment trivia) is independent and can still run in parallel with Phase 85 and most of 86.

### 2.2 Other Parallelization Opportunities

| Parallel Track | Valid? | Notes |
|----------------|--------|-------|
| TASK-571 (comment trivia) with TASK-572+573 (error spans + trait) | **Valid** | No cross-dependencies. Both only require TASK-570 to complete first (for the mechanical refactor of match sites). |
| Phase 88a (core formatter crate) with Phase 87 (LSP) | **Valid** | Core formatter only needs CommentTable from TASK-571. The LSP `textDocument/formatting` handler (88b) must wait for 87. |
| Phase 89 (Salsa) with Phase 87 | **Partially valid** | Salsa replaces `ash-lsp-core` cache, so it needs the core crate to exist. It can start once 87’s VFS and diagnostic pipeline are stable (~week 4 of 87), overlapping with the tail end of 87. |

---

## 3. Dependency Edges

### 3.1 Correctly Represented
- Phase 87 is correctly blocked by Phases 84, 85, and 86 (TASK-569 "Hard Prerequisites").
- Phase 88 is correctly blocked by Phase 84 (CommentTable) and Phase 87 (LSP skeleton).
- Phase 89 is correctly blocked by Phase 87 (`ash-lsp-core` must exist).

### 3.2 Missing / Incorrect Dependencies

**A. Missing `AshLspError` bridge for `LintDiagnostic`**
- TASK-573 lists `AshLspError` implementations for `ParseError`, `ConstructorError`, `TypeEnvError`, `ExhaustivenessError`, and `NameError`.
- It **omits** `LintDiagnostic` exported by `ash-lint` in Phase 86.
- Yet `ash-lsp-core` is supposed to "Aggregate diagnostics from `ash-parser`, `ash-typeck`, and `ash-lint`" (TASK-569 §2).
- **Gap:** There is no explicit task to either (a) implement `AshLspError` for `LintDiagnostic`, or (b) define a conversion from `LintDiagnostic` to a type that does.

**Recommendation:** Add a micro-task (~2-4 h) inside Phase 85 or 86: "Implement `AshLspError` for `ash-lint::LintDiagnostic`" or "Define `LintDiagnostic::to_lsp_diagnostic()`". This prevents a last-minute integration surprise in Phase 87.

**B. PLAN-INDEX.md references wrong plan for Phase 87**
- Phase 87 references **PLAN-029** ("Multi-Parameter Interface Methods", Phase 82).
- There is no dedicated PLAN file for SPEC-038 (LSP & MCP).
- **Recommendation:** Create `PLAN-036-LSP-MCP-INTERFACE.md` (or reuse `PLAN-030` if unused) and update the index. At minimum, change the reference to point to `TASK-569`.

**C. Implicit dependency: TASK-572 on TASK-570**
- The plans do not explicitly note that TASK-572 touches `check_expr.rs` and `check_pattern.rs`, which are also heavily modified by TASK-570.
- **Recommendation:** Add an explicit edge: `TASK-570 → TASK-572`.

---

## 4. Estimate Consistency and Realism

| Phase | Tasks | Estimate | Timeline Claim | Assessment |
|-------|-------|----------|----------------|------------|
| 84 | 570, 571 | 16 h | 1 week | **Realistic.** Mechanical refactor (6 h) + lexer change (10 h). |
| 85 | 572, 573 | 18 h | 1 week | **Realistic.** Threading spans through error sites is tedious but bounded. |
| 86 | 574 | 12 h | 1 week | **Realistic**, but only if it follows TASK-570. Rewriting 4 simple AST visitors into a lib is ~1-2 days. |
| 87 | 569 | 180 h | 5 weeks | **Realistic for an LSP MVP**, possibly slightly optimistic for "production-quality" plus MCP bridge plus VSCode skeleton. |
| 88 | 575 | 40 h | 2 weeks | **Reasonable.** Comment-preserving formatters are finicky; 40 h is tight but doable for a MVP. |
| 89 | 576 | 32 h | 1.5-2 weeks | **Optimistic.** Salsa integration often balloons due to trait-bound refactoring and invalidation debugging. Budget 40-48 h. |

**Total:** 298 h planned. With the recommended split of Phase 88, core formatter work is ~32 h + LSP hookup ~8 h. Salsa should be padded to ~40 h. **Revised total: ~310-318 h** (~8 weeks single-dev, ~5.5 weeks with 2 devs).

---

## 5. Scheduling Optimizations

### 5.1 Optimize the Pre-LSP Pipeline
Current pre-87 chain: 16 + 18 + 12 = **46 h** (all sequential).  
Optimized pre-87 chain:

```
TASK-570 (6 h)
    ├──→ TASK-571 (10 h)  ──┐
    └──→ TASK-572 (12 h)    │
         TASK-573 (6 h)     │
                            ├──→ TASK-574 (12 h)
```

**Critical path:** 6 + max(10, 18) + 12 = **36 h** (saves ~10 h).

### 5.2 Split Phase 88 to Run in Parallel with 87
- **88a — Core formatter (`ash-formatter` crate):** 32 h, starts after TASK-571 completes. Can run in parallel with the bulk of Phase 87.
- **88b — LSP formatting handler + CLI hookup:** 8 h, blocked by Phase 87 skeleton.

This removes 32 h from the post-87 sequential backlog.

### 5.3 Overlap Phase 89 with the Tail of Phase 87
Salsa integration does not need every LSP handler to be finished. It needs:
1. `ash-lsp-core` crate structure (week 1 of 87)
2. VFS + change application (week 1-2 of 87)
3. Diagnostic pipeline (week 2-3 of 87)

Once those three milestones are stable, the Salsa migration can begin.  
**Suggestion:** Start TASK-576 no later than the beginning of "Phase 4" of TASK-569 (MCP bridge, week 4). This creates ~1 week of overlap, potentially saving 8-16 h of wall-clock time.

### 5.4 Recommended Timeline (2 Developers)

| Week | Dev A | Dev B |
|------|-------|-------|
| 1 | TASK-570 → TASK-572+573 | TASK-571 (starts after 570) |
| 2 | TASK-574 | 88a core formatter |
| 3-6 | Phase 87 (LSP/MCP core) | Continue 88a + assist on 87 tests |
| 7 | Finish 87 + 88b LSP hookup | Start Phase 89 (Salsa) |
| 7.5 | — | Finish Phase 89 |

**Wall-clock: ~7 weeks** (down from ~8 weeks single-dev).

---

## 6. Specific Resequencing Recommendations

1. **Enforce `TASK-570 → TASK-574`.** Do not claim Phase 86 runs in parallel with 84.
2. **Add explicit integration task:** "Implement `AshLspError` for `LintDiagnostic`" (~2-4 h) between 86 and 87.
3. **Fix `PLAN-INDEX.md`:** Change Phase 87 plan reference from `PLAN-029` to a new `PLAN-036` or to `TASK-569`.
4. **Split Phase 88** into 88a (core, parallel with 87) and 88b (LSP hookup, post-87).
5. **Pad Phase 89 estimate** from 32 h to 40-48 h to account for Salsa invalidation debugging.
6. **Start Phase 89 during week 4 of Phase 87** rather than strictly after 87 completes.

---

## 7. Risk Summary

| Risk | Impact | Mitigation |
|------|--------|------------|
| Merge conflicts between 84 and 86 | High | Sequence 84 before 86. |
| Lint diagnostics do not map to LSP | Medium | Add explicit bridge task. |
| Salsa invalidation bugs blow schedule | Medium | Pad estimate; start early to surface bugs. |
| Formatter comment edge cases | Low | Keep 88a in parallel with 87; extra slack. |
