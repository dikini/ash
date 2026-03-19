# Lean Reference Implementation: Effort Estimate

## Overview

Realistic effort breakdown for implementing a Lean 4 reference interpreter for Ash, assuming a developer familiar with Rust and basic functional programming (but not necessarily dependent types).

## Effort Breakdown

### Phase 1: Setup & Core Types (Week 1-2)

| Task | Hours | Notes |
|------|-------|-------|
| Lean 4 toolchain setup | 4 | Lake, LSP, editor integration |
| Project structure | 4 | Mirroring Rust crate structure |
| AST definitions | 16 | Expr, Pattern, Value, Type enums |
| Serialization (JSON) | 8 | For differential testing bridge |
| Basic testing setup | 4 | `#eval` smoke tests |
| **Subtotal** | **36h (~1 week)** | |

**Deliverable**: Can define and print Ash AST in Lean

### Phase 2: Interpreter Core (Week 2-4)

| Task | Hours | Notes |
|------|-------|-------|
| Environment implementation | 8 | Variable binding, lookup |
| Expression evaluator | 16 | Literals, variables, operators |
| Constructor evaluation | 8 | Enum/struct construction |
| Pattern matching engine | 16 | Core matching algorithm |
| Match expression | 12 | Multi-arm match with bindings |
| If-let expression | 6 | Sugar for match |
| Effect tracking | 8 | Epistemic → Operational lattice |
| Error handling | 8 | Proper error propagation |
| **Subtotal** | **82h (~2 weeks)** | |

**Deliverable**: Can interpret basic ADT programs

### Phase 3: Advanced Features (Week 4-6)

| Task | Hours | Notes |
|------|-------|-------|
| Workflow constructs | 16 | Observe, Act, Decide, etc. |
| Parallel composition | 12 | Join semantics |
| Spawn/split/control | 16 | Instance management |
| Capabilities integration | 12 | Capability provider interface |
| Trace recording | 8 | Provenance tracking |
| **Subtotal** | **64h (~1.5 weeks)** | |

**Deliverable**: Full interpreter matching SPEC-004

### Phase 4: Core Proofs (Week 6-9)

| Task | Hours | Notes |
|------|-------|-------|
| Pattern match totality | 16 | `matchPattern` always returns for valid inputs |
| Pattern match determinism | 12 | Same input → same bindings |
| Constructor evaluation pure | 8 | No side effects |
| Progress theorem | 24 | Well-typed programs make progress |
| Preservation theorem | 32 | Types preserved under evaluation |
| Exhaustiveness soundness | 20 | Exhaustive patterns never fail |
| **Subtotal** | **112h (~3 weeks)** | |

**Deliverable**: Key theorems proven in Lean

### Phase 5: Differential Testing Bridge (Week 9-10)

| Task | Hours | Notes |
|------|-------|-------|
| Rust→Lean serialization | 12 | Parse Rust JSON in Lean |
| Lean→Rust serialization | 8 | Export Lean results |
| Step-tracing in Rust | 16 | Instrument interpreter |
| Bisimulation checker | 12 | Compare step-by-step |
| Test corpus generation | 8 | Generate from Lean |
| CI integration | 8 | GitHub Actions workflow |
| **Subtotal** | **64h (~1.5 weeks)** | |

**Deliverable**: Automated differential testing

### Phase 6: Polish & Documentation (Week 10-11)

| Task | Hours | Notes |
|------|-------|-------|
| Code documentation | 12 | Inline comments, docstrings |
| Proof documentation | 16 | Paper-style proof sketches |
| Example programs | 8 | Showcase common patterns |
| Performance tuning | 8 | Make interpreter reasonably fast |
| Bug fixes from testing | 16 | Inevitable issues |
| **Subtotal** | **60h (~1.5 weeks)** | |

**Deliverable**: Production-ready reference

## Total Effort

| Phase | Weeks | Hours | Cumulative |
|-------|-------|-------|------------|
| 1: Setup | 1 | 36 | 36 |
| 2: Interpreter Core | 2 | 82 | 118 |
| 3: Advanced Features | 1.5 | 64 | 182 |
| 4: Core Proofs | 3 | 112 | 294 |
| 5: Differential Bridge | 1.5 | 64 | 358 |
| 6: Polish | 1.5 | 60 | 418 |
| **Total** | **10.5** | **418** | |

**~11 weeks full-time (420 hours)**

## Mitigation Strategies

### Option A: Interpreter Only (No Proofs) - 6 weeks

Skip Phase 4 proofs entirely:

| Phase | Weeks |
|-------|-------|
| Setup | 1 |
| Interpreter Core | 2 |
| Advanced Features | 1.5 |
| Differential Bridge | 1.5 |
| **Total** | **6 weeks** |

**Use case**: Get differential testing running quickly, add proofs later

### Option B: Minimal Proofs Only - 8 weeks

Do only the most critical proofs:

| Proof | Hours | Priority |
|-------|-------|----------|
| Pattern match determinism | 12 | High |
| Constructor purity | 8 | High |
| Progress (simplified) | 16 | Medium |
| **Subtotal** | **36h (~1 week)** | |

**Total**: 7 weeks + 1 week = **8 weeks**

### Option C: Incremental Approach - Ongoing

| Milestone | Effort | Value |
|-----------|--------|-------|
| MVP interpreter | 4 weeks | Differential testing |
| Pattern proofs | +2 weeks | Confidence in matching |
| Type safety | +3 weeks | Foundation proven |
| Full semantics | +2 weeks | Complete verification |

**Use case**: Add to existing project incrementally

## Risk Factors

| Risk | Impact | Mitigation |
|------|--------|------------|
| Lean learning curve | +2-3 weeks | Pair with Lean expert, use community resources |
| Proof difficulty | +2-4 weeks | Start with interpreter only, add proofs gradually |
| AST drift | Ongoing maintenance | Auto-generate from Rust? Shared IDL? |
| Performance | Tests too slow | Extract to C, use compiled Lean |
| Complex proofs | Stuck on theorem | Post to Lean Zulip, simplify theorem |

## Comparison: Lean vs Other Options

| Approach | Effort | Confidence | Maintenance |
|----------|--------|------------|-------------|
| **Lean + proofs** | 11 weeks | Very High | Medium (keep in sync) |
| **Lean no proofs** | 6 weeks | High | Medium |
| **Coq + proofs** | 12 weeks | Very High | Medium |
| **Haskell (no proofs)** | 4 weeks | Medium | Low |
| **Rust (second impl)** | 3 weeks | Low | High (two codebases) |
| **Property tests only** | 1 week | Medium | Low |

## Recommendation

### For Immediate Value (Next 2 Months)

**Go with Option A (Interpreter Only, 6 weeks)**:

1. Week 1-2: Core interpreter (can evaluate constructors and match)
2. Week 3-4: Add remaining expressions
3. Week 5: Differential testing bridge
4. Week 6: Integration and bug fixes

**Value**: Catches bugs in Rust implementation immediately

### For Long-term Confidence (Next 6 Months)

**Add proofs incrementally**:

1. Month 1-2: Interpreter only, running in CI
2. Month 3: Pattern matching proofs
3. Month 4: Progress theorem
4. Month 5-6: Type safety, preservation

**Value**: Mathematical certainty about core algorithms

## Team Composition

| Scenario | Team | Timeline |
|----------|------|----------|
| Solo dev, Lean novice | 1 person | 11-14 weeks |
| Solo dev, Lean experienced | 1 person | 8-10 weeks |
| Pair (Rust + Lean experts) | 2 people | 6-8 weeks |
| Team (parallel work) | 3 people | 5-6 weeks |

## Getting Started (Week 1 Plan)

```bash
# Day 1-2: Setup
lake new ash_reference
cd ash_reference
# Setup editor (VS Code + Lean extension)

# Day 3-4: Core types
# Ash/Core/AST.lean - Define Expr, Pattern, Value

# Day 5: Serialization
# Ash/Core/JSON.lean - Parse from Rust JSON

# Weekend: Learn Lean basics (if needed)
# https://leanprover.github.io/theorem_proving_in_lean4/
```

## Conclusion

**Minimum viable**: 6 weeks for interpreter + differential testing
**Fully proven**: 11 weeks with core theorems

**The 6-week version provides 80% of the value** (differential testing) for 50% of the effort. Proofs can be added incrementally as the project matures.

If you're already considering the full ADT implementation (12 weeks), adding the Lean reference (6 weeks in parallel) is a **50% overhead** for **significant confidence gains**.
