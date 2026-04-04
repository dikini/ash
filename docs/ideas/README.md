# Ideas & Explorations Index

This directory tracks **pre-specification explorations** — design questions, concept investigations, and architectural options that are not yet ready for the formal PLAN-INDEX workflow.

## Purpose

- **Capture ideas before they're ready** — rough notes, design sketches, alternatives
- **Support iteration without overhead** — no task files, no formal tracking until promoted
- **Build living documents** — explorations evolve from stream-of-consciousness to candidate specs
- **Archive abandoned paths** — record why an approach was rejected

## Status Lifecycle

| Status | Meaning | Next Action |
|--------|---------|-------------|
| `drafting` | Initial thoughts, stream of consciousness | Iterate, add structure |
| `reviewing` | Ready for discussion and refinement | Review with collaborator |
| `candidate` | Mature enough to become formal work | Promote to PLAN-INDEX |
| `accepted` | Content moved to `docs/spec/` or implemented | Archive with reference |
| `rejected` | Approach abandoned | Move to `archived/` with rationale |
| `merged` | Content absorbed into another exploration | Archive, link to successor |
| `deferred` | Valid idea, postponed to future work | Keep in `future/`, revisit later |

## Current Explorations

### Minimal Core Execution Environment

| ID | Title | Status | Last Revised | Notes |
|----|-------|--------|--------------|-------|
| MCE-001 | [Entry Point](minimal-core/MCE-001-ENTRY-POINT.md) | `candidate` | 2026-03-30 | How Ash programs start → [Phase 57](../plan/PLAN-INDEX.md#phase-57-entry-point-and-program-execution) |
| MCE-002 | [IR Core Forms Audit](minimal-core/MCE-002-IR-AUDIT.md) | `accepted` | 2026-04-03 | Promoted to [TASK-370](../plan/tasks/TASK-370-ir-core-forms-audit.md) |
| MCE-003 | [Functions vs Capabilities](minimal-core/MCE-003-FUNCTIONS-VS-CAPS.md) | `drafting` | 2026-03-30 | Do we need functions or are capabilities enough? |
| MCE-004 | [Big-Step Semantics Alignment](minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md) | `drafting` | 2026-03-30 | Surface syntax ↔ IR ↔ big-step cleanup |
| MCE-005 | [Small-Step Semantics](minimal-core/MCE-005-SMALL-STEP.md) | `drafting` | 2026-03-30 | Develop small-step semantics |
| MCE-006 | [Small-Step ↔ IR Execution](minimal-core/MCE-006-SMALL-STEP-IR.md) | `drafting` | 2026-03-30 | Align small-step with IR execution |
| MCE-007 | [Full Layer Alignment](minimal-core/MCE-007-FULL-ALIGNMENT.md) | `drafting` | 2026-03-30 | Surface, IR, both semantics, interpreter |
| MCE-008 | [Runtime Cleanup](minimal-core/MCE-008-RUNTIME-CLEANUP.md) | `drafting` | 2026-03-30 | Libraries and capabilities runtime support |
| MCE-009 | [Test & Example Workflows](minimal-core/MCE-009-TEST-WORKFLOWS.md) | `drafting` | 2026-03-30 | Develop and run test/example workflows |

### Type System

| ID | Title | Status | Last Revised | Notes |
|----|-------|--------|--------------|-------|
| TYPES-001 | [Tuple Variant Syntax](type-system/TYPES-001-tuple-variants.md) | `drafting` | 2026-03-30 | Tuple vs record syntax for ADT variants |
| TYPES-002 | [Ad-Hoc Polymorphism](type-system/TYPES-002-ad-hoc-polymorphism.md) | `drafting` | 2026-04-01 | Preserved reasoning trace with review note; intentionally meandering |
| TYPES-002 V2 | [Ad-Hoc Polymorphism V2](type-system/TYPES-002-ad-hoc-polymorphism-v2.md) | `reviewing` | 2026-04-01 | Cleaner open exploration with Ash-native examples and decision-driving workloads |
| TYPES-003 | [Capability and Effect Vocabulary](type-system/TYPES-003-capabilities-effects-vocabulary.md) | `drafting` | 2026-04-01 | Judgment-oriented bridge note to disambiguate capability facets and their relation to effects |
| TYPES-004 | [Effect Typing Foundations](type-system/TYPES-004-effect-typing-foundations.md) | `drafting` | 2026-04-01 | Grade-first exploration of effect-producing forms, composition, and the provider metadata boundary |

### OTP / Actor Model Explorations

Research and design explorations for OTP-like supervision, fault tolerance, and process management in Ash.

| ID | Title | Status | Last Revised | Notes |
|----|-------|--------|--------------|-------|
| OTP-001 | [Erlang/OTP Architecture Analysis](otp/OTP-001-erlang-otp-analysis.md) | `drafting` | 2026-03-31 | Comprehensive analysis of gen_server and supervisor behaviors |
| OTP-002 | [Ash OTP Design Considerations](otp/OTP-002-ash-otp-design.md) | `drafting` | 2026-03-31 | Design options for OTP-like functionality in Ash |

### Future / Deferred

| ID | Title | Status | Last Revised | Notes |
|----|-------|--------|--------------|-------|
| FUTURE-001 | [First-Class Workflows](future/FIRST-CLASS-WORKFLOWS.md) | `deferred` | 2026-03-30 | Post-minimal-core: workflows as values |

## Adding a New Exploration

1. Use the [template](templates/exploration-template.md)
2. Place in appropriate subdirectory (or create new topic directory)
3. Add entry to table above
4. Set initial status to `drafting`

## Promoting to PLAN-INDEX

When an exploration reaches `candidate` status:

1. Create task file in `docs/plan/tasks/`
2. Reference the exploration document in task description
3. Set exploration status to `accepted`
4. Archive the exploration with link to new task

## Maintenance

- Review stale items weekly (last revised > 2 weeks)
- Update status as ideas mature or are abandoned
- Ensure archived items explain the "why" of rejection
