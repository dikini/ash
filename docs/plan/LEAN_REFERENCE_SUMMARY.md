# Lean Reference Implementation - Summary

## Overview

Complete plan and specification for implementing a Lean 4 reference interpreter for Ash ADT operations.

**Scope**: Phase 1 (6 weeks) - Interpreter only  
**Aspirational**: Phase 2 (TBD) - Formal proofs  
**Value**: Differential testing, executable specification, foundation for verification

## Documents Created

### Specification
| Document | Purpose |
|----------|---------|
| `docs/spec/SPEC-021-LEAN-REFERENCE.md` | Technical specification for the interpreter |

### Plans
| Document | Purpose |
|----------|---------|
| `docs/plan/PLAN-021-LEAN-REFERENCE.md` | Implementation plan with phases and dependencies |
| `docs/plan/LEAN_IMPLEMENTATION_EFFORT.md` | Effort estimates and options |
| `docs/design/BISIMULATION_VERIFICATION.md` | Theoretical foundation (bisimulation) |
| `docs/design/TEST_GENERATION_AND_VERIFICATION.md` | Testing strategies |

### Task Files (TDD Format)
| Task | Description | Est. Hours |
|------|-------------|------------|
| TASK-137 | Lean 4 toolchain setup | 4 |
| TASK-138 | AST type definitions | 16 |
| TASK-139 | Environment and effects | 8 |
| TASK-140 | Expression evaluation | 16 |
| TASK-141 | Pattern matching engine | 16 |
| TASK-142 | Match expression | 12 |
| TASK-143 | If-let expression | 6 |
| TASK-144 | JSON serialization | 12 |
| TASK-145 | Differential testing | 16 |
| TASK-146 | Property tests | 8 |
| TASK-147 | CI integration | 8 |
| TASK-148 | Documentation | 8 |
| **Total** | | **130 hours (~6 weeks)** |

### Aspirational (Future)
| Task | Description | Est. Hours |
|------|-------------|------------|
| TASK-149 | Pattern determinism proof | 20 |
| TASK-150 | Constructor purity proof | 12 |
| TASK-151 | Progress theorem | 40 |
| TASK-152 | Preservation theorem | 40 |

## Key Design Decisions

### 1. Interpreter First, Proofs Later

**Rationale**: Get value quickly with differential testing, add proofs incrementally.

```
Week 1-6:   Interpreter + Differential testing ✓ (immediate value)
Month 3+:   Add proofs as time permits (future enhancement)
```

### 2. JSON Bridge for Differential Testing

**Rationale**: Universal serialization, easy debugging, language agnostic.

```rust
// Rust → JSON → Lean → Compare
let rust_result = rust_interp.eval(workflow);
let lean_result = lean_interp.eval(workflow);
assert_eq!(rust_result.to_json(), lean_result.to_json());
```

### 3. Bisimulation as Theoretical Foundation

**Rationale**: Formal connection between testing and verification.

- **Testing**: Check that implementations are observationally equivalent
- **Proofs**: Prove that equivalence is maintained by all operations

## Project Structure

```
ash/
├── lean_reference/              # NEW
│   ├── Ash/
│   │   ├── Core/
│   │   │   ├── AST.lean
│   │   │   ├── Types.lean
│   │   │   ├── Environment.lean
│   │   │   └── Serialize.lean
│   │   ├── Eval/
│   │   │   ├── Expr.lean
│   │   │   ├── Pattern.lean
│   │   │   └── Match.lean
│   │   └── Differential/
│   │       ├── Types.lean
│   │       ├── Parse.lean
│   │       └── Compare.lean
│   ├── lakefile.lean
│   └── Main.lean
├── tests/differential/          # NEW
│   └── *.json (test corpus)
├── scripts/
│   └── differential_test.sh     # NEW
└── docs/
    ├── spec/SPEC-021-LEAN-REFERENCE.md
    ├── plan/PLAN-021-LEAN-REFERENCE.md
    └── design/
        ├── BISIMULATION_VERIFICATION.md
        └── TEST_GENERATION_AND_VERIFICATION.md
```

## Timeline

| Week | Tasks | Deliverable |
|------|-------|-------------|
| 1 | 137, 138, 139 | AST defined, can print values |
| 2-3 | 140, 141 | Expression eval working |
| 4 | 142, 143 | Full ADT evaluation |
| 5 | 144, 145 | Differential testing working |
| 6 | 146, 147, 148 | CI integration, docs complete |

**Start**: After TASK-121 (ADT Core Types) complete  
**Duration**: 6 weeks parallel to Rust ADT implementation

## Integration with Main Project

### Dependencies
- **Blocks on**: TASK-121 (ADT Core Types) - Rust AST stable
- **Runs parallel to**: TASK-122 through TASK-136 (Rust ADT implementation)
- **Enables**: Differential testing for all ADT tasks

### CI Integration
```yaml
# On every PR affecting interpreter
- Build Lean reference
- Build Rust implementation
- Run 1000 differential tests
- Fail if any mismatch
```

## Success Criteria

### Phase 1 (6 weeks)
- [ ] Can evaluate all ADT test cases
- [ ] 1000+ differential tests passing
- [ ] CI integration working
- [ ] Bug found in Rust via differential testing

### Phase 2 (Aspirational)
- [ ] Pattern match determinism proven
- [ ] Progress theorem proven
- [ ] Published as case study

## Value Proposition

| Stakeholder | Value |
|-------------|-------|
| **Developers** | Catch bugs early, clear oracle for semantics questions |
| **Reviewers** | Differential tests prove correctness automatically |
| **Users** | Higher confidence in language stability |
| **Researchers** | Foundation for verified compilation |

## Next Steps

1. **Review**: SPEC-021 and PLAN-021 with team
2. **Schedule**: Start TASK-137 after TASK-121 complete
3. **Assign**: Developer with FP background to lead
4. **Prepare**: Rust JSON serialization (for bridge)

## Questions?

- **Why Lean?**: Dependent types, theorem proving, executable
- **Why not Coq?**: Lean more ergonomic, better tooling
- **Why not Haskell?**: No dependent types, no proofs
- **Can we skip proofs?**: Yes! Interpreter-only is 6 weeks, high value
- **When do proofs?**: After interpreter stable, as team expertise grows
