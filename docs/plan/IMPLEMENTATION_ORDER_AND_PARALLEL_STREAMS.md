# Implementation Order with Parallel Work Streams

## Executive Summary

**Optimal Approach**: 3 parallel work streams over 10 weeks  
**Critical Path**: ADT Core → Type Check → Eval → Integration  
**Risk Mitigation**: Working interpreter at Week 5, Differential testing at Week 8

```
Timeline (10 weeks)
═══════════════════════════════════════════════════════════════════
Week:  1   2   3   4   5   6   7   8   9   10
       │   │   │   │   │   │   │   │   │   │
Stream 1 (ADT Core): ████████░░░░░░░░░░░░░░░░
  ├─ Core types ████
  ├─ Type check ████████
  ├─ Evaluation ████████████
  └─ Integration    ░░░░░░░░████████

Stream 2 (Parser): ░░████████░░░░░░░░░░░░░░░░
  └─ Parse all constructs ████████

Stream 3 (Lean): ░░░░░░████████████░░░░░░░░░░
  ├─ Setup ░░░░██
  ├─ Interpreter  ░░░░░░████████
  └─ Differential ░░░░░░░░░░░░░░████████

Milestones:        ↑   ↑       ↑       ↑
                  M1  M2      M3      M4

M1: Rust types compile (Week 2)
M2: Can parse and type-check (Week 4)
M3: Can evaluate programs (Week 5)
M4: Differential testing working (Week 8)
```

## Work Streams

### Stream 1: ADT Core Implementation (Primary)
**Owner**: Senior Rust developer  
**Duration**: Weeks 1-10 (with gaps for integration)  
**Dependencies**: None

**Week 1-2: Foundation**
- TASK-121: Core types
- TASK-122: Runtime values  
- TASK-123: Unification

**Week 3-4: Type System**
- TASK-127: Check constructors
- TASK-128: Check patterns
- TASK-129: Generics

**Week 5: Evaluation Engine**
- TASK-131: Constructor evaluation
- TASK-132: Pattern matching
- TASK-133: Match evaluation

**Week 8-10: Integration**
- TASK-134: Spawn with Option
- TASK-135: Control transfer
- TASK-136: Standard library

### Stream 2: Parser (Secondary)
**Owner**: Developer with parser experience  
**Duration**: Weeks 2-4  
**Dependencies**: TASK-121 (core types)

**Parallel work starting Week 2:**
- TASK-124: Parse type definitions
- TASK-125: Parse match expressions
- TASK-126: Parse if-let

### Stream 3: Lean Reference (Validation)
**Owner**: Developer with FP/Lean interest  
**Duration**: Weeks 3-9  
**Dependencies**: TASK-121 (stable AST)

**Week 3-4: Setup**
- TASK-137: Lean setup
- TASK-138: AST types
- TASK-139: Environment

**Week 5-6: Interpreter**
- TASK-140: Expression eval
- TASK-141: Pattern matching
- TASK-142: Match expressions
- TASK-143: If-let

**Week 7-9: Differential Testing**
- TASK-144: JSON serialization
- TASK-145: Differential harness
- TASK-146: Property tests
- TASK-147: CI integration
- TASK-148: Documentation

## Critical Path Analysis

```
Critical Path (must be sequential):
═══════════════════════════════════════

TASK-121 (Core Types) ─────────────────────┐
                                           │
TASK-122 (Runtime Values) ───────┐         │
                                 │         │
TASK-123 (Unification) ───────┐  │         │
                              │  │         │
TASK-124 (Parse Types) ────┐  │  │         │
                           │  │  │         │
TASK-127 (Check Ctors) ────┼──┼──┼─────────┤
                           │  │  │         │
TASK-128 (Check Patterns) ─┼──┼──┼─────────┤
                           │  │  │         │
TASK-131 (Eval Ctors) ─────┼──┼──┘         │
                           │  │            │
TASK-132 (Pattern Match) ──┘  │            │
                              │            │
TASK-133 (Match Eval) ────────┘            │
                                           │
TASK-134 (Spawn Option) ───────────────────┤
                                           │
TASK-135 (Control) ────────────────────────┤
                                           │
TASK-136 (Std Lib) ────────────────────────┘

Total critical path: ~8 weeks
```

## Parallel Opportunities

### High Parallelism (Low Coupling)

1. **Parser tasks (TASK-124-126)**
   - Parallel with type checker development
   - Only needs stable AST from TASK-121

2. **Lean interpreter (TASK-137-143)**
   - Parallel with Rust evaluation
   - Only needs AST definitions

3. **Exhaustiveness (TASK-130)**
   - Can be added after TASK-128
   - Self-contained algorithm

### Medium Parallelism (Some Coupling)

4. **Type checking vs Evaluation**
   - Type check tests need evaluation to verify
   - But can develop in parallel with feature flags

5. **JSON serialization (TASK-144)**
   - Needs stable Value types
   - Can be done in parallel with evaluation engine

### Low Parallelism (High Coupling)

6. **Spawn/Control (TASK-134-135)**
   - Needs Option type working
   - Needs evaluation engine
   - Sequential after core ADT

## Optimal Implementation Order

### Phase 1: Foundation (Weeks 1-2)
**Goal**: Types compile, basic structure in place

```
Week 1:
├── Stream 1: TASK-121 (ADT Core Types)
│   └── 6 hours - Type enum, Variant, Struct
│
└── Stream 1: TASK-122 (Runtime Values)
    └── 5 hours - Value enum with variants

Week 2:
├── Stream 1: TASK-123 (Unification)
│   └── 4 hours - Constructor unification
│
├── Stream 2: TASK-124 (Parse Type Defs)
│   └── 6 hours - Enum/struct parsing
│
└── Milestone M1: Rust types compile ✓
```

### Phase 2: Type System (Weeks 3-4)
**Goal**: Can type-check programs

```
Week 3:
├── Stream 1: TASK-127 (Check Constructors)
│   └── 6 hours - Type check constructor exprs
│
├── Stream 2: TASK-125 (Parse Match)
│   └── 5 hours - Match and pattern parsing
│
└── Stream 2: TASK-126 (Parse If-Let)
    └── 3 hours - If-let sugar

Week 4:
├── Stream 1: TASK-128 (Check Patterns)
│   └── 8 hours - Pattern type checking
│
├── Stream 1: TASK-129 (Generics)
│   └── 6 hours - Type instantiation
│
└── Milestone M2: Can parse and type-check ✓
```

### Phase 3: Evaluation (Week 5)
**Goal**: Can evaluate programs

```
Week 5:
├── Stream 1: TASK-131 (Eval Constructors)
│   └── 4 hours - Constructor evaluation
│
├── Stream 1: TASK-132 (Pattern Match Engine)
│   └── 6 hours - Pattern matching
│
├── Stream 1: TASK-133 (Match Eval)
│   └── 5 hours - Match expressions
│
└── Milestone M3: Can evaluate programs ✓
```

### Phase 4: Lean Reference (Weeks 3-6)
**Goal**: Working Lean interpreter (parallel track)

```
Week 3 (overlaps with Phase 1-2):
├── Stream 3: TASK-137 (Lean Setup)
│   └── 4 hours - Toolchain, project
│
└── Stream 3: TASK-138 (Lean AST)
    └── 16 hours - Mirror Rust AST

Week 4:
└── Stream 3: TASK-139 (Environment)
    └── 8 hours - Env, effects

Week 5 (overlaps with Phase 3):
├── Stream 3: TASK-140 (Expr Eval)
│   └── 16 hours - Expression evaluation
│
└── Stream 3: TASK-141 (Pattern Match)
    └── 16 hours - Pattern matching

Week 6:
├── Stream 3: TASK-142 (Match Expr)
│   └── 12 hours - Match expressions
│
└── Stream 3: TASK-143 (If-Let)
    └── 6 hours - If-let sugar
```

### Phase 5: Integration (Weeks 6-8)
**Goal**: Differential testing working

```
Week 6:
├── Stream 1: TASK-130 (Exhaustiveness)
│   └── 8 hours - Exhaustiveness checking
│
└── Stream 3: TASK-144 (JSON Bridge)
    └── 12 hours - Serialization

Week 7:
├── Stream 1: TASK-134 (Spawn Option)
│   └── 6 hours - Spawn with Option
│
└── Stream 3: TASK-145 (Differential)
    └── 16 hours - Testing harness

Week 8:
├── Stream 1: TASK-135 (Control Transfer)
│   └── 5 hours - Control link semantics
│
├── Stream 3: TASK-146 (Property Tests)
│   └── 8 hours - Property tests
│
├── Stream 3: TASK-147 (CI Integration)
│   └── 8 hours - GitHub Actions
│
└── Milestone M4: Differential testing working ✓
```

### Phase 6: Polish (Weeks 9-10)
**Goal**: Production ready

```
Week 9:
├── Stream 1: TASK-136 (Std Lib)
│   └── 6 hours - Option/Result modules
│
├── Stream 3: TASK-148 (Documentation)
│   └── 8 hours - Lean docs
│
└── All: Bug fixes from differential testing

Week 10:
├── All: Performance tuning
├── All: Additional property tests
└── All: Final integration testing
```

## Risk Mitigation

### Risk 1: Lean Learning Curve
**Mitigation**: Start Lean at Week 3, not Week 1
- Rust AST is stable by then
- Developer has 2 weeks to learn Lean basics
- If Lean falls behind, Rust continues independently

### Risk 2: AST Drift
**Mitigation**: Weekly AST review meetings
- Both Rust and Lean teams review changes
- JSON schema as contract between implementations
- Automated schema validation

### Risk 3: Integration Issues
**Mitigation**: Early integration at Week 5
- Don't wait until end
- M3 milestone ensures evaluation works
- Differential testing starts at Week 8

## Team Structure Recommendations

### Option A: 3 Developers (Optimal)

| Developer | Primary Stream | Secondary |
|-----------|---------------|-----------|
| Senior Rust | Stream 1 (ADT Core) | Stream 2 guidance |
| Mid Rust | Stream 2 (Parser) | Stream 1 integration |
| FP/Lean | Stream 3 (Lean) | Rust AST consultation |

### Option B: 2 Developers (Minimal)

| Developer | Primary | Secondary |
|-----------|---------|-----------|
| Senior Full-Stack | Stream 1 (ADT Core) | Stream 3 (Lean) |
| Mid Rust | Stream 2 (Parser) | Stream 1 |

*Note: With 2 devs, Lean may slip or be deferred*

### Option C: 4+ Developers (Accelerated)

Add:
- **Tester/QA**: Property test generation, edge case finding
- **Technical Writer**: Documentation for both codebases

## Dependency Graph with Parallel Tracks

```
                    Week 1    Week 2    Week 3    Week 4    Week 5
                      │         │         │         │         │
Stream 1 (ADT Core):
  TASK-121 ──────────█████████░
  TASK-122 ───────────────────██████████░
  TASK-123 ──────────────────────────────████
  TASK-127 ───────────────────────────────────███████████░
  TASK-128 ───────────────────────────────────────────────████████████
  TASK-131 ────────────────────────────────────────────────────────────████████
  TASK-132 ────────────────────────────────────────────────────────────────────████████
  TASK-133 ────────────────────────────────────────────────────────────────────────────████████

Stream 2 (Parser):
  TASK-124 ───────────────────██████████░
  TASK-125 ───────────────────────────────█████████░
  TASK-126 ──────────────────────────────────────────███

Stream 3 (Lean):
  TASK-137 ──────────────────────────────██
  TASK-138 ───────────────────────────────────████████████████
  TASK-139 ───────────────────────────────────────────────────────████
  TASK-140 ───────────────────────────────────────────────────────────────████████████████
  TASK-141 ────────────────────────────────────────────────────────────────────────────────████████████████

                    Week 6    Week 7    Week 8    Week 9    Week 10
                      │         │         │         │         │
Stream 1:
  TASK-130 ──────────█████████░
  TASK-134 ────────────────────███████░
  TASK-135 ──────────────────────────────█████
  TASK-136 ────────────────────────────────────────██████

Stream 3:
  TASK-142 ──────────█████████████░
  TASK-143 ─────────────────────────███
  TASK-144 ───────────────────────────────█████████████░
  TASK-145 ────────────────────────────────────────────────█████████████████
  TASK-146 ────────────────────────────────────────────────────────────────█████████
  TASK-147 ────────────────────────────────────────────────────────────────────────█████████
  TASK-148 ────────────────────────────────────────────────────────────────────────────────█████████
```

## Communication Plan

### Daily Standups
- Each stream reports blockers
- Cross-stream dependencies discussed
- AST changes announced immediately

### Weekly Integration
- End-of-week demo of working features
- Review differential test results
- Adjust priorities based on findings

### Milestone Reviews
- **M1 (Week 2)**: Show types compiling
- **M2 (Week 4)**: Show type-checking working
- **M3 (Week 5)**: Show evaluation working
- **M4 (Week 8)**: Show differential testing catching bugs

## Decision Points

### Week 3: Lean Go/No-Go
- **Go**: Lean setup complete, developer comfortable
- **No-Go**: Defer Lean, focus on Rust quality

### Week 5: Differential Testing Prep
- Ensure JSON serialization stable
- Define comparison criteria
- Set up test corpus generation

### Week 8: Scope Review
- If behind: Cut TASK-136 (std lib extras)
- If ahead: Add Phase 19 proof work
- If critical: Delay non-core features

## Success Metrics

| Week | Metric | Target |
|------|--------|--------|
| 2 | Type checker tests pass | 50+ tests |
| 4 | End-to-end parse→type check | 100 programs |
| 5 | Evaluation correctness | 90%+ on test suite |
| 8 | Differential match rate | 100% (by design) |
| 10 | Bugs found by differential | ≥1 real bug |

## Summary

**Best order**: Parallel streams with clear milestones  
**Critical insight**: Start Lean at Week 3, not Week 1  
**Key risk**: AST drift - mitigate with weekly sync  
**Success factor**: Early integration (Week 5), not big-bang at end

**Recommended**: 3 developers, 10 weeks, 3 parallel streams
