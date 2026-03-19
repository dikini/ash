# Lean Reference Interpreter Implementation Plan

## Overview

Implementation plan for a Lean 4 reference interpreter for Ash ADT operations. This is **Phase 1** focusing on the interpreter only; formal proofs are aspirational for Phase 2.

**Goal**: Enable differential testing between Lean (specification) and Rust (implementation) to catch bugs systematically.

**Architecture**: Lean interpreter as executable semantics, JSON bridge for differential testing.

**Tech Stack**: Lean 4, Lake build system, JSON serialization

---

## Task Dependencies

```
TASK-137 (Setup)
    │
    ├──→ TASK-138 (AST Types)
    │       │
    │       ├──→ TASK-139 (Environment)
    │       │       │
    │       │       ├──→ TASK-140 (Expression Eval)
    │       │       │       │
    │       │       │       ├──→ TASK-141 (Pattern Match)
    │       │       │       │       │
    │       │       │       │       ├──→ TASK-142 (Match Expr)
    │       │       │       │       │
    │       │       │       ├──→ TASK-143 (If-Let)
    │       │       │
    │       │       └──→ TASK-144 (JSON Serialization)
    │               │
    │               └──→ TASK-145 (Differential Testing)
    │                       │
    │                       ├──→ TASK-146 (Property Tests)
    │                       │
    │                       └──→ TASK-147 (CI Integration)
    │
    └──→ TASK-148 (Documentation) [parallel]
```

---

## Phase 1: Core Interpreter (6 weeks)

### Week 1: Foundation

| Task | Description | Est. Hours | Status |
|------|-------------|------------|--------|
| [TASK-137](tasks/TASK-137-lean-setup.md) | Lean 4 toolchain and project setup | 4 | 🟡 Ready |
| [TASK-138](tasks/TASK-138-lean-ast-types.md) | AST type definitions | 16 | 🟡 Ready |
| [TASK-139](tasks/TASK-139-lean-environment.md) | Environment and effect tracking | 8 | 🟡 Ready |

**Week 1 Deliverable**: Can define and print Ash AST in Lean

### Week 2-3: Core Evaluation

| Task | Description | Est. Hours | Status |
|------|-------------|------------|--------|
| [TASK-140](tasks/TASK-140-lean-expression-eval.md) | Expression evaluation | 16 | 🟡 Ready |
| [TASK-141](tasks/TASK-141-lean-pattern-match.md) | Pattern matching engine | 16 | 🟡 Ready |

**Week 2-3 Deliverable**: Can evaluate basic expressions

### Week 4: Control Flow

| Task | Description | Est. Hours | Status |
|------|-------------|------------|--------|
| [TASK-142](tasks/TASK-142-lean-match-expr.md) | Match expression evaluation | 12 | 🟡 Ready |
| [TASK-143](tasks/TASK-143-lean-if-let.md) | If-let expression | 6 | 🟡 Ready |

**Week 4 Deliverable**: Full ADT evaluation (constructors + pattern matching)

### Week 5-6: Differential Testing

| Task | Description | Est. Hours | Status |
|------|-------------|------------|--------|
| [TASK-144](tasks/TASK-144-lean-json-serialization.md) | JSON bridge | 12 | 🟡 Ready |
| [TASK-145](tasks/TASK-145-lean-differential-testing.md) | Differential testing harness | 16 | 🟡 Ready |
| [TASK-146](tasks/TASK-146-lean-property-tests.md) | Property-based tests | 8 | 🟡 Ready |
| [TASK-147](tasks/TASK-147-lean-ci-integration.md) | CI integration | 8 | 🟡 Ready |
| [TASK-148](tasks/TASK-148-lean-documentation.md) | Documentation | 8 | 🟡 Ready |

**Week 5-6 Deliverable**: Automated differential testing in CI

---

## Phase 2: Formal Proofs (Aspirational - Not Planned)

Future work after Phase 1 is complete and stable:

| Task | Description | Est. Hours | Status |
|------|-------------|------------|--------|
| TASK-149 | Pattern match determinism proof | 20 | 🔴 Future |
| TASK-150 | Constructor purity proof | 12 | 🔴 Future |
| TASK-151 | Progress theorem | 40 | 🔴 Future |
| TASK-152 | Preservation theorem | 40 | 🔴 Future |

**Note**: Phase 2 is aspirational. Prioritize based on value and feasibility.

---

## Success Criteria

### Phase 1 Success

- [ ] Lean interpreter can evaluate all ADT test cases
- [ ] Differential tests pass for 1000+ generated programs
- [ ] CI runs differential tests on every PR
- [ ] Bug found in Rust implementation through differential testing

### Phase 2 Success (Aspirational)

- [ ] Key theorems formally proven
- [ ] Proofs published/documented
- [ ] Extracted code passes all tests

---

## Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Lean learning curve | +2 weeks | High | Start with interpreter only; defer proofs |
| AST drift between Rust/Lean | Ongoing | Medium | Shared test corpus; update together |
| Performance too slow | +1 week | Low | Optimize later; correctness over speed |
| JSON serialization bugs | +3 days | Medium | Roundtrip tests; property tests |

---

## Integration with Main Project

### Location

```
ash/
├── crates/                  # Rust implementation
│   ├── ash-core/
│   ├── ash-parser/
│   └── ...
├── lean_reference/          # NEW: Lean reference
│   ├── Ash/
│   │   ├── Core/
│   │   ├── Eval/
│   │   └── Differential/
│   ├── lakefile.lean
│   └── README.md
├── tests/differential/      # NEW: Differential test corpus
├── scripts/
│   └── differential_test.sh # NEW: Test runner
└── .github/workflows/
    └── lean-reference.yml   # NEW: CI workflow
```

### Workflow

1. **Rust changes** → Differential tests run automatically
2. **Mismatch detected** → Investigate (usually Rust bug, occasionally spec clarification)
3. **Lean changes** → Update spec documentation
4. **Both pass** → High confidence in correctness

---

## Resource Requirements

### Development

- **1 developer** with:
  - Rust experience (for differential bridge)
  - Functional programming background
  - Willingness to learn Lean (steep but rewarding curve)

### Compute

- CI: +5 minutes per build (Lean compilation)
- Differential tests: ~1 minute for 1000 test cases
- Caching: Lake build artifacts

---

## Timeline Summary

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| Phase 1 | 6 weeks | Working differential testing |
| Phase 2 | TBD (aspirational) | Formal proofs |

**Start Date**: After ADT core types (TASK-121) complete
**Dependencies**: Rust AST stable enough for serialization

---

## Related Documents

- SPEC-021: Lean Reference Specification
- SPEC-004: Operational Semantics (source of truth)
- TASK-137 through TASK-148: Individual task files
- docs/design/BISIMULATION_VERIFICATION.md: Theoretical foundation
