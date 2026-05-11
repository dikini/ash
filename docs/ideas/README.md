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
| `candidate` | Mature enough to become formal work or to anchor a formal promotion task | Promote to PLAN-INDEX or derive a task/reference artifact |
| `accepted` | Content moved into `docs/spec/`, a reference artifact, or otherwise materially realized | Archive with reference |
| `closeout-published` | Closeout/reporting artifact published while residual follow-on work may still remain | Keep indexed as a published closeout artifact |
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
| MCE-004 | [Big-Step Semantics Alignment](minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md) | `accepted` | 2026-04-05 | Resolved surface syntax ↔ canonical IR ↔ big-step alignment → [TASK-393](../plan/tasks/TASK-393-big-step-semantics-alignment.md) |
| MCE-005 | [Small-Step Semantics](minimal-core/MCE-005-SMALL-STEP.md) | `accepted` | 2026-04-05 | Accepted Phase 61 small-step backbone and rule inventory; [SPEC-025](../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) is now the docs/spec home for that accepted contract → [TASK-394](../plan/tasks/TASK-394-small-step-semantics-scope-and-configuration-contract.md), [TASK-395](../plan/tasks/TASK-395-canonical-workflow-small-step-rule-set-and-concurrency-semantics.md), [TASK-396](../plan/tasks/TASK-396-small-step-big-step-correspondence-and-mce-006-handoff.md) |
| MCE-006 | [Small-Step ↔ IR Execution](minimal-core/MCE-006-SMALL-STEP-IR.md) | `accepted` | 2026-04-05 | Phase 63 / TASK-401 through TASK-404 now freeze the runtime carrier mapping, control/blocking/completion story, `Par` correspondence, observable-preservation checklist, divergence taxonomy, and concise MCE-007 handoff; [SPEC-025](../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) is the aligned docs/spec home, while verdict remains partial observable realization of the accepted backbone |
| MCE-007 | [Full Layer Alignment](minimal-core/MCE-007-FULL-ALIGNMENT.md) | `closeout-published` | 2026-04-07 | TASK-398 ingests the frozen MCE-006 Phase 63 packet, TASK-399 classifies the remaining rows into packaging-only work, accepted partiality, and true residual drift with owners, TASK-400 publishes the final closeout/signoff/checklist artifact, TASK-405 adds a conservative authoritative blocked/terminal/invalid outcome classification in `ash-interp`, TASK-406 narrows to a sealed/write-once retained completion carrier, TASK-407 adds the real spawned-child execution/sealing path, TASK-408 preserves direct child terminal success/error payloads in the retained record, TASK-409 adds a conservative retained effect-summary slice (`effects.terminal_upper_bound` plus conservative `effects.reached_upper_bound`), TASK-410 adds one honest retained obligations slice based on terminal-visible local/role obligation state, and TASK-428 now freezes [SPEC-026](../spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md) as the canonical cross-implementation conformance anchor while exact obligations/provenance/exact-effect parity remains open |
| MCE-008 | [Runtime Cleanup](minimal-core/MCE-008-RUNTIME-CLEANUP.md) | `drafting` | 2026-04-05 | Runtime cleanup remains broad. TASK-405 is complete as the first concrete runtime-side follow-on for blocked/terminal/invalid classification, TASK-406 is now complete for its scoped retained-completion carrier goal, TASK-407 supplies the runtime-owned spawned-child execution substrate plus honest automatic completion sealing, TASK-408 adds one honest retained `CompletionPayload.result`-like slice (`result` plus `terminal_result()`), TASK-409 adds one conservative retained `CompletionPayload.effects`-like slice (`effects.terminal_upper_bound` plus conservative `effects.reached_upper_bound`), TASK-410 adds one honest retained `CompletionPayload.obligations`-like slice based on terminal-visible local/role obligation state, TASK-411 adds one conservative retained provenance slice based on runtime-owned child identity/spawn lineage, and TASK-412 is the next planned follow-on for a dedicated completion-wait carrier/API while exact provenance/exact-effect/full-Ω parity and broader cumulative-carrier follow-ons remain open |
| MCE-009 | [Test & Example Workflows](minimal-core/MCE-009-TEST-WORKFLOWS.md) | `drafting` | 2026-03-30 | Develop and run test/example workflows |

### Type System

| ID | Title | Status | Last Revised | Notes |
|----|-------|--------|--------------|-------|
| TYPES-001 | [Canonical Tuple Variant Syntax](type-system/TYPES-001-tuple-variants.md) | `candidate` | 2026-04-06 | Canonicalizes parenthesized tuple-variant syntax; TASK-413 completed the initial spec-promotion pass and follow-on implementation/reconciliation work remains |
| TYPES-002 | [Ad-Hoc Polymorphism](type-system/TYPES-002-ad-hoc-polymorphism.md) | `drafting` | 2026-04-06 | Preserved reasoning trace only; intentionally non-normative and not the planning target by itself |
| TYPES-002 V2 | [Ad-Hoc Polymorphism V2](type-system/TYPES-002-ad-hoc-polymorphism-v2.md) | `reviewing` | 2026-04-06 | Main polished exploration and broader serious discussion surface; pair with the narrowed [MVP cut](type-system/TYPES-002-v2-mvp-cut.md) for active planning |
| TYPES-003 | [Capability and Effect Vocabulary](type-system/TYPES-003-capabilities-effects-vocabulary.md) | `candidate` | 2026-04-06 | Reasoning record behind the promoted [type-system vocabulary guidance](../reference/type-system-vocabulary-guidance.md); docs/spec convergence completed by [TASK-414](../plan/tasks/TASK-414-effect-typing-contract-promotion.md) |
| TYPES-004 | [Effect Typing Foundations](type-system/TYPES-004-effect-typing-foundations.md) | `candidate` | 2026-04-06 | Reasoning record behind the promoted coarse effect-typing contract and workflow-form grading tables; follow-on `Pure` staging remains explicit after [TASK-414](../plan/tasks/TASK-414-effect-typing-contract-promotion.md) |
| TYPES-002 MVP | [Closed-World Interfaces MVP Cut](type-system/TYPES-002-v2-mvp-cut.md) | `candidate` | 2026-04-06 | Narrowed follow-on target frozen by [TASK-415](../plan/tasks/TASK-415-closed-world-interfaces-mvp-spec-cut.md): canonical `T: Interface` bounds, canonical `Interface::method(value)` calls, strong coherence, and explicit deferrals |

### OTP / Actor Model Explorations

Research and design explorations for OTP-like supervision, fault tolerance, and process management in Ash.

| ID | Title | Status | Last Revised | Notes |
|----|-------|--------|--------------|-------|
| OTP-001 | [Erlang/OTP Architecture Analysis](otp/OTP-001-erlang-otp-analysis.md) | `drafting` | 2026-03-31 | Comprehensive analysis of gen_server and supervisor behaviors |
| OTP-002 | [Ash OTP Design Considerations](otp/OTP-002-ash-otp-design.md) | `drafting` | 2026-03-31 | Design options for OTP-like functionality in Ash |
| OTP-003 | [GenServer-like Design Patterns for Ash](otp/OTP-003-genserver-design-patterns.md) | `drafting` | 2026-05-11 | Compares reducer, callback dictionary, capability-backed, Proc, Workflow, resource, protocol, codegen, and supervisor-first GenServer-like patterns with examples and gap inventory |
| OTP-004 | [Harnessed Worker / Bisimulation-like Control Patterns for Ash](otp/OTP-004-harnessed-worker-bisimulation-patterns.md) | `drafting` | 2026-05-11 | Compares product-state reference, lockstep controller/worker, shadow-model, evidence-carrying, capability-membrane, event-log, workflow-governed, typed-protocol, supervisor, and N-version harness patterns for pure-Ash LLM workflow control examples |

### Future / Deferred

| ID | Title | Status | Last Revised | Notes |
|----|-------|--------|--------------|-------|
| FUTURE-001 | [First-Class Workflows](future/FIRST-CLASS-WORKFLOWS.md) | `deferred` | 2026-03-30 | Post-minimal-core: workflows as values |
| FUTURE-002 | [AI-Native Workflows and Generated Ash Programs](future/AI-NATIVE-WORKFLOWS.md) | `drafting` | 2026-04-20 | Live exploration of LSP/MCP + quotation/splice + evals as complementary substrate; treats ReAct/RM/RLM-like systems as both useful targets and gap-discovery workloads for Ash infrastructure |
| FUTURE-003 | [Agentic Workflow Exemplars](future/AGENTIC-WORKFLOW-EXEMPLARS.md) | `drafting` | 2026-04-20 | Companion live note tracking ReAct/RM/RLM-like families as concrete benchmark targets, implementation targets, and substrate gap-finders |
| FUTURE-004 | [Ash Wiki as Human/AI Shared Knowledge Substrate](future/ASH-WIKI-HUMAN-AI-KNOWLEDGE-SUBSTRATE.md) | `drafting` | 2026-04-20 | Explores a static-first wiki layer as shared human/AI project memory, audit substrate, onboarding surface, and queryable knowledge service over the Ash corpus |
| FUTURE-005 | [Compiled Execution Substrate](future/COMPILED-EXECUTION-SUBSTRATE.md) | `drafting` | 2026-05-12 | Captures the future TCIR → AMIR → bytecode → JIT direction, traceable sectioned bytecode artifacts, semi-stable AMIR text, and non-blocking Ash-in-Ash design pressure |

## Adding a New Exploration

1. Use the [template](templates/exploration-template.md)
2. Place in appropriate subdirectory (or create new topic directory)
3. Add entry to table above
4. Set initial status to `drafting`

## Promoting to PLAN-INDEX

When an exploration reaches `candidate` status:

1. Create a task file in `docs/plan/tasks/` or derive a reusable reference/spec artifact
2. Reference the exploration document in the promoted artifact
3. Keep the exploration as `candidate` while the promotion work is underway, or mark it `accepted` once the promoted corpus materially realizes it
4. Archive the exploration only when it has truly been superseded by the promoted artifact

## Maintenance

- Review stale items weekly (last revised > 2 weeks)
- Update status as ideas mature or are abandoned
- Ensure archived items explain the "why" of rejection
