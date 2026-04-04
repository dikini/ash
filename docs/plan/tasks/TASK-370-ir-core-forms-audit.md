# TASK-370: IR Core Forms Audit (MCE-002)

## Status: ✅ Done

## Description

Inventory all current IR forms in `ash-core` and identify candidates for elimination or consolidation. Produce a formal audit report with specific recommendations for minimizing the IR surface while preserving semantics.

**Reference:** [MCE-002 Exploration](../../ideas/minimal-core/MCE-002-IR-AUDIT.md)

## Current Inventory (from codebase analysis)

### Workflow IR Forms (30 total in `ast.rs`)

| # | Form | Description | Current Usage |
|---|------|-------------|---------------|
| 1 | `Observe` | OBSERVE capability as pattern in continuation | Active |
| 2 | `Receive` | RECEIVE from workflow mailboxes using ordered arm matching | Active |
| 3 | `Orient` | ORIENT expression then continue | Active |
| 4 | `Propose` | PROPOSE action (advisory) | Active |
| 5 | `Decide` | DECIDE expression under policy then continue | Active |
| 6 | `Check` | CHECK obligation then continue | Active |
| 7 | `Act` | ACT action where guard with provenance | Active |
| 8 | `Oblig` | OBLIG role to workflow | Active |
| 9 | `Let` | LET pattern = expr in continuation | Active |
| 10 | `If` | IF expr then else | Active |
| 11 | `Seq` | Sequential composition | Active (primitive sequencing) |
| 12 | `Par` | Parallel composition | Active |
| 13 | `ForEach` | FOREACH pattern in expr do workflow | Active |
| 14 | `Ret` | RET expression | Active |
| 15 | `With` | WITH capability DO workflow | Active |
| 16 | `Maybe` | MAYBE workflow else workflow | Active |
| 17 | `Must` | MUST workflow | Active |
| 18 | `Set` | SET capability:channel = value | **Review needed** |
| 19 | `Send` | SEND capability:channel value | **Review needed** |
| 20 | `Spawn` | Spawn workflow with init args | Active |
| 21 | `Split` | Split instance as pattern in continuation | Active |
| 22 | `Kill` | Kill a workflow instance | Active |
| 23 | `Pause` | Pause a workflow instance | Active |
| 24 | `Resume` | Resume a workflow instance | Active |
| 25 | `CheckHealth` | Check health of a workflow instance | Active |
| 26 | `Oblige` | OBLIGE obligation_name (linear) | Active |
| 27 | `CheckObligation` | CHECK obligation_name as a workflow form with `name`/`span` payload | **Overlap with Expr::CheckObligation / Check?** |
| 28 | `Yield` | YIELD to role with request, suspend | Active |
| 29 | `ProxyResume` | RESUME from yield with response | Active |
| 30 | `Done` | Terminal workflow | Active |

### Expression IR Forms (13 total in `Expr` enum)

| # | Form | Description |
|---|------|-------------|
| 1 | `Literal` | Literal values |
| 2 | `Variable` | Variable references |
| 3 | `FieldAccess` | Record field access |
| 4 | `IndexAccess` | Collection index access |
| 5 | `Unary` | Unary operations (Not, Neg) |
| 6 | `Binary` | Binary operations (+, -, &&, etc.) |
| 7 | `Call` | Function/method calls |
| 8 | `Constructor` | ADT constructor expressions |
| 9 | `Match` | Pattern matching | **Candidate for review** |
| 10 | `IfLet` | If-let sugar | **Sugar for Match** |
| 11 | `Spawn` | Spawn as expression | **Duplicate of Workflow::Spawn?** |
| 12 | `Split` | Split as expression | **Duplicate of Workflow::Split?** |
| 13 | `CheckObligation` | Check obligation as expression | **Overlap with Workflow form?** |

### Additional IR-Related Types

- `Pattern` - Patterns for binding (Variable, Constructor, Literal, Wildcard)
- `Action` - Actions to execute with name and arguments
- `Capability` - Capability references with effect levels
- `ReceiveArm` - Arms for Receive pattern matching
- `Guard` - Guards for Act statements

## Requirements

### Functional Requirements

1. **Complete Inventory**: Document every canonical core IR form in `ast.rs`, plus any overlapping secondary carriers that materially affect consolidation decisions, with:
   - Source location (file, line)
   - Purpose and semantics
   - Current usage in codebase
   - Dependencies on other forms

2. **Expressibility Analysis**: For each form, determine:
   - Is this form primitive (cannot be expressed in terms of others)?
   - Can this form be expressed as sugar over other forms?
   - What would elimination cost (complexity, performance, ergonomics)?

3. **Elimination Candidates Analysis**:

   | Form | Candidate | Expressible As | Confidence | Analysis Required |
   |------|-----------|----------------|------------|-------------------|
   | `Seq` | **Keep** | Primitive sequencing (no valid rewrite to Let) | High | Document why elimination was rejected |
   | `Expr::Match` | Review | `If` + destructuring | Medium | Measure verbosity impact |
   | `IfLet` | Sugar | `Match` | High | Already documented |
   | `Set` | Review | `Act` with capability? | Low | Check effect semantics |
   | `Send` | Review | `Act` with capability? | Low | Check effect semantics |
   | `CheckObligation` | Review | Consolidate with workflow/expression check carriers only if semantics can be preserved | Medium | Resolve semantic overlap without assuming direct unification with `Check` |
   | `Expr::Spawn` | Review | Keep duplicated with `Workflow::Spawn` until statement-vs-expression lowering is clearer | High | Expression vs statement context |

4. **Minimal Core Proposal**: Define essential forms for:
   - Values and bindings
   - Control flow
   - Concurrency
   - Effects
   - Communication

### Deliverables

1. **Audit Report** (`docs/ideas/minimal-core/MCE-002-IR-AUDIT-REPORT.md`):
   - Complete inventory with cross-references
   - Expressibility matrix
   - Elimination recommendations with trade-offs
   - Minimal core proposal

2. **Prototype Rewriting** (optional but recommended):
   - ~~Show `Seq` → `Let` transformation~~ (Invalid: type mismatch between `Workflow` and `Expr`)
   - Investigate alternative elimination approaches for `Seq` if any exist
   - Demonstrate impact of other eliminations on example programs

3. **Updated Exploration Document**:
   - Update MCE-002 with findings
   - ~~Change status to `candidate` if ready for implementation~~ (Completed: MCE-002 promoted to `accepted` and archived)

## Analysis Tasks

### Task 1: Document Current Forms

For each form in `ast.rs`:
```rust
// Example documentation format:
/// Form: Seq
/// Location: ast.rs:72-75
/// Purpose: Sequential composition of two workflows
/// Dependencies: None (primitive sequencing)
/// Usage count: [grep for usage]
/// Candidates: Under investigation - no valid rewrite identified (Workflow vs Expr type mismatch)
```

### Task 2: Seq Elimination Analysis - REJECTED

**Status: REJECTED - Seq is a primitive form.**

The initial hypothesis (`Seq` → `Let` rewrite) was **rejected** due to type mismatch:

```
-- REJECTED: Invalid rewrite due to type mismatch
-- Workflow::Seq takes two Workflows; Workflow::Let expects an Expr
Seq(a, b) => Let { pattern: "_", expr: a, continuation: b }  -- INVALID: 'a' is Workflow, not Expr
```

`Workflow::Seq` composes two `Workflow`s (`first: Box<Workflow>, second: Box<Workflow>`), 
while `Workflow::Let` binds an `Expr` to a pattern (`expr: Expr`). A `Workflow` cannot 
be substituted where an `Expr` is expected.

**Conclusion:** `Seq` is a primitive sequencing construct and cannot be eliminated.
Document this finding in the audit report.

Questions answered:
- ✅ Is `Seq` a primitive form? **YES** - No valid elimination path exists.
- Does eliminating `Seq` affect effect ordering? **N/A** - Elimination not possible.
- What would desugaring look like without `Seq`? **N/A** - Seq is required.

### Task 3: Check/CheckObligation Unification

Compare:
- `Workflow::Check { obligation, continuation }`
- `Workflow::CheckObligation { name, span }`
- `Expr::CheckObligation { obligation, span }`

Determine:
- Are these semantically equivalent?
- Can they be unified?
- What's the migration path?

### Task 4: Expression/Workflow Form Duplication

Analyze duplicates:
- `Expr::Spawn` vs `Workflow::Spawn`
- `Expr::Split` vs `Workflow::Split`
- `Expr::CheckObligation` vs `Workflow::CheckObligation`

Determine:
- Are these needed in both contexts?
- Can expression forms be lowered to workflow forms via proper binding mechanisms (e.g., `Let` with pattern binding)?
  (Note: `Orient` alone is insufficient as it does not bind values—any lowering must
  account for how expression results flow into workflow continuations.)

### Task 5: Minimal Core Definition

Propose minimal essential forms:

**Values:**
- `Literal`, `Variable` (expressions)

**Binding:**
- `Let` (workflow)

**Control:**
- `If` (workflow)
- `Match` (expression - decide on keep/eliminate)

**Effects:**
- `Act` (workflow)
- `Observe` (workflow)

**Concurrency:**
- `Par` (workflow)
- `Spawn` (workflow)

**Communication:**
- `Send`/`Receive` (workflow)
- `Yield`/`ProxyResume` (workflow)

**Control Flow:**
- `Ret` (workflow)
- `Done` (workflow)

## Completion Checklist

- [x] Complete inventory of all 30 canonical `Workflow` forms in `ast.rs`
- [x] Complete inventory of all 13 canonical `Expr` forms in `ast.rs`
- [x] Expressibility analysis for each candidate form
- [x] Documented rewriting rules for eliminations
- [x] Minimal core proposal with rationale
- [x] Impact assessment on existing tests/examples and on major overlapping carriers outside `ast.rs`
- [x] Updated MCE-002 exploration status
- [x] CHANGELOG.md entry

## Resolution Summary

1. ✅ **Seq elimination**: **RESOLVED** - `Seq` is primitive; no valid elimination path exists.
2. ✅ **Check-form overlap**: **PARTIALLY RESOLVED** - `Check`, `Workflow::CheckObligation`, and `Expr::CheckObligation` overlap in name/intention but are not currently shape-equivalent; do not assume direct unification of `Check` with `CheckObligation`.
3. ✅ **Expression/workflow duplication**: **PARTIALLY RESOLVED** - `Expr::Spawn`/`Expr::Split` duplicate workflow-level concepts but should remain until the language chooses a clearer statement-vs-expression lowering discipline.
4. ✅ **Expr::Match elimination**: **RESOLVED FOR NOW** - keep `Expr::Match`; elimination is not justified until pattern tests/extractors are made more explicit.
5. ✅ **Set/Send vs Act**: **RESOLVED FOR NOW** - keep `Set` and `Send` as distinct forms pending clearer capability-operation semantics.

## Related Work

- **MCE-003**: Functions vs Capabilities (affects Call form)
- **MCE-004**: Big-step semantics alignment (forms must support transition rules)
- **MCE-007**: Full layer alignment (depends on this audit)

## Estimated Effort

8-12 hours for comprehensive analysis and documentation

## Dependencies

None - this is an analysis/documentation task

## Blocked By

Nothing

## Blocks

- MCE-004: Needs IR audit to determine what forms need semantics
- MCE-007: Depends on stable IR definition
- Any IR form elimination work
