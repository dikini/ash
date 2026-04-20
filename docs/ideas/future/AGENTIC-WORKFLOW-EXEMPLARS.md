---
status: drafting
created: 2026-04-20
last-revised: 2026-04-20
related-plan-tasks: []
tags: [ai, workflows, exemplars, react, rm, rlm, evals, live-document]
---

# FUTURE-003: Agentic Workflow Exemplars

## Status: Drafting / Live Document

This is a live companion document to `FUTURE-002`.

Its purpose is to track concrete, useful agentic workflow families that Ash should eventually support well, while also using them as benchmark targets and gap-discovery workloads for the language, standard library, tooling, and runtime.

## Summary

The core idea is to maintain a small set of real, implementation-shaped exemplars rather than discussing AI-native workflow infrastructure only in the abstract.

These exemplars should help answer two questions at once:

1. What useful systems do we actually want people to build in Ash?
2. What missing substrate do those systems reveal?

ReAct, RM, and RLM-like families are especially valuable because they stress different parts of the stack:

- ReAct stresses tool dispatch, trajectories, and repair loops.
- RM-like patterns stress routing, reflection, ranking, and evaluation.
- RLM-like patterns stress recursive decomposition, generated helper workflows, code construction, and stronger validation boundaries.

## Relationship to FUTURE-002

`FUTURE-002` is the broader substrate document.

This document is narrower and more operational: it tracks candidate exemplar workflow families and what they are good for as engineering targets.

See also: [FUTURE-002: AI-Native Workflows and Generated Ash Programs](AI-NATIVE-WORKFLOWS.md)

## Why Exemplars Matter

Exemplar systems are useful because they:

- force substrate discussions into concrete requirements
- expose missing contracts in stdlib and runtime surfaces
- provide realistic evaluation targets
- give a better "definition of done" than abstract capability lists
- help discover whether Ash is merely parse-capable or actually fit for AI-native orchestration

## Candidate Exemplars

### 1. ReAct-style Tool Agent

**What it is:**
A workflow that alternates between model reasoning, tool invocation, tool-result ingestion, and final answer production.

**Why it matters:**
This is likely the first serious Ash-native agentic pattern because it is already close to Ash's current capabilities.

**Primary substrate pressure:**
- tool schema contracts
- dynamic tool dispatch
- structured message history
- trajectory representation
- typed final output
- semantic authoring support through LSP/MCP

**Questions it should answer:**
- What is the canonical Ash representation of an agent trajectory?
- How much tool dispatch can live in stdlib/workflow space?
- What diagnostics are missing for generated/edited tool-using workflows?

### 2. RM-like Router / Reflector / Ranker

**What it is:**
A family of workflows that route tasks, compare candidate responses, reflect on failures, or rank alternatives before selecting the next action.

**Why it matters:**
These patterns stress structured comparison and decision logic rather than just tool execution.

**Primary substrate pressure:**
- candidate scoring vocabulary
- typed comparison records
- branching/routing ergonomics
- model orchestration combinators
- eval metrics and benchmark harnesses

**Questions it should answer:**
- What reusable routing/reflection combinators belong in Ash stdlib?
- What minimum evaluation substrate is needed to compare strategies honestly?
- How should ranking or reflection traces be represented?

### 3. Constrained RLM-style Recursive Workflow

**What it is:**
A recursive reasoning workflow that decomposes a large task into subqueries or helper steps, but initially using a constrained action algebra or generated helper workflows rather than open-ended arbitrary code execution.

**Why it matters:**
This is the most direct pressure test for Ash's code-construction and generated-workflow ambitions without immediately requiring a full sandboxed execution environment.

**Primary substrate pressure:**
- quotation/splice or structural helper generation
- generated-program check/repair/execute path
- recursive decomposition contracts
- trajectory and subtrajectory nesting
- trust boundary before execution
- eval and regression infrastructure

**Questions it should answer:**
- What is the smallest viable generated-workflow artifact?
- Can Ash support recursive decomposition without becoming stringly or unsafe?
- What ownership split between engine and runtime is required?

### 4. Workflow Repair / Self-Correction Loop

**What it is:**
A workflow family focused on generating Ash, running diagnostics, repairing the code, and rechecking until it satisfies a contract or hits a stop condition.

**Why it matters:**
This is the most direct benchmark for the "LLMs generate semantically correct Ash workflows" goal.

**Primary substrate pressure:**
- in-memory parse/check APIs
- machine-usable diagnostics
- code actions / repair suggestions
- provenance of generated artifacts
- evaluation of repair quality and convergence

**Questions it should answer:**
- Which LSP/MCP surfaces are mandatory for agentic authoring?
- What is the canonical repair loop artifact model?
- What failure classes should be measured?

### 5. Evaluator / Judge Workflow

**What it is:**
A workflow family that scores outputs, validates structured constraints, compares alternatives, or provides adjudication for another workflow.

**Why it matters:**
Agentic systems need evaluation inside the loop as well as outside it.

**Primary substrate pressure:**
- metric interfaces
- scoring/result schemas
- reproducible traces
- dataset/example harnesses
- comparison and regression contracts

**Questions it should answer:**
- What is the minimal Ash-native evaluation interface?
- How should scored traces be represented?
- Which parts belong in stdlib versus external harnesses?

## Suggested Use of Exemplars

Each exemplar should be tracked as both:

- a useful target capability
- a structured gap-finding exercise

For each exemplar, future revisions of this document should record:

- current feasibility status
- hard blockers by crate/subsystem
- minimum viable version
- what it teaches us about missing Ash substrate
- whether it should become a formal spec/plan/task stream

## Initial Working Hypothesis

A sensible build order is probably:

1. ReAct-style tool agent
2. RM-like routing / reflection patterns
3. Workflow repair / self-correction loop
4. Constrained RLM-style recursive workflow
5. richer evaluator/judge families throughout

This ordering reflects likely implementation leverage, not intrinsic importance.

## Tracking Template For Future Updates

For each exemplar, record:

- **Status:** drafting / reviewing / candidate / accepted / deferred
- **Primary value:** user-facing capability, substrate discovery, or both
- **Main blockers:** parser / typeck / engine / interp / stdlib / LSP-MCP / evals
- **Minimum viable form:** what can be built before the full vision exists
- **What it teaches us:** concrete gaps discovered

## Related Explorations

- [FUTURE-002: AI-Native Workflows and Generated Ash Programs](AI-NATIVE-WORKFLOWS.md)
- [FUTURE-001: First-Class Workflows](FIRST-CLASS-WORKFLOWS.md)
- [OTP-002: Ash OTP Design Considerations](../otp/OTP-002-ash-otp-design.md)

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-04-20 | Create a separate exemplar-oriented live note | The substrate discussion benefits from concrete benchmark targets rather than only abstract infrastructure layers. |
| 2026-04-20 | Treat exemplars as both useful systems and gap-discovery workloads | Real target systems are the best way to discover what Ash, its libraries, and its tooling still lack. |

## Next Steps

- [ ] Add a feasibility row for each exemplar in terms of parser / typeck / engine / interp / stdlib / LSP-MCP / evals.
- [ ] Decide which exemplar should become the first serious implementation target.
- [ ] Link future specs/plans/tasks back to these exemplar families when they become concrete workstreams.
