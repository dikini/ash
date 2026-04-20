---
status: drafting
created: 2026-04-20
last-revised: 2026-04-20
related-plan-tasks: []
tags: [ai, workflows, lsp, mcp, quotation, splice, macros, rlm, react, evals, future]
---

# FUTURE-002: AI-Native Workflows and Generated Ash Programs

## Status: Drafting / Live Document

This is a live exploration document.

It is meant to accumulate design notes over time rather than freeze a near-term implementation contract. The goal is to track the long-range architectural direction for making Ash a language in which AI systems can author, inspect, transform, validate, and execute Ash workflows natively.

## Summary

The long-term direction is not "implement RLM first" in isolation.

The direction is to grow a complementary substrate stack:

- LSP + MCP for semantic visibility, repair loops, and agent-facing tooling
- quotation / splice / macro systems for program construction and transformation
- REPL for interactive inspection and debugging
- engine / type checker / runtime for authoritative checking and execution
- evaluation infrastructure for measuring generated workflows and agent behaviors

In this framing, tools like ReAct, RM, and eventually RLM-like systems are not merely downstream applications of the infrastructure. They are also useful real-life implementation targets and pressure tests for the infrastructure itself. They should help expose gaps in Ash, its standard libraries, and its tooling/runtime substrate while also serving as valuable user-facing capabilities.

Accordingly, they can be treated simultaneously as:

- real workloads that Ash should eventually support well
- implementation targets that force hidden substrate assumptions into the open
- discovery tools for missing language/library/runtime/tooling features
- candidate Ash-native libraries or workflow patterns once the underlying substrates are mature enough

A second core goal is to let LLMs generate Ash code that is not merely parseable text, but something that can be iteratively diagnosed, repaired, validated, and then executed through the canonical Ash semantic pipeline.

## Problem Statement

Ash already has pieces of an agentic programming environment, but they are not yet organized into a coherent "AI-native workflow" story.

The central questions are:

1. How do we make Ash a good target language for LLM-authored workflows?
2. How do we let LLMs generate Ash workflows that are semantically correct enough to execute safely?
3. How do we make program construction in Ash structural rather than stringly?
4. How do we define agentic patterns like ReAct, RM, and RLM as Ash-level workflows or libraries rather than embedding them permanently in host-language scaffolding?
5. How do we use those patterns as implementation targets that reveal concrete gaps in Ash's semantics, libraries, diagnostics, tooling, and execution model?

## Scope

- **In scope:**
  - long-range substrate planning for AI-authored and AI-executed Ash workflows
  - relationship between LSP/MCP, quotation/splice, derive/macro work, REPL, type checking, and runtime execution
  - architectural requirements for ReAct/RM/RLM-like systems in Ash
  - generated-code check/repair/execute loops
  - evaluation needs for AI-native workflow systems
- **Out of scope:**
  - immediate implementation commitments
  - final surface syntax for quotation/splice/macros
  - near-term task decomposition in PLAN-INDEX
  - claiming current support for AST-as-value or runtime meta-eval
- **Related but separate:**
  - builtin fn rollout and regex migration
  - first-class workflows as values
  - OTP / supervision exploration
  - formatter and incremental analysis details as standalone efforts

## Current Understanding

### What we know

- LSP/MCP and quotation/splice/macro systems should be treated as complementary infrastructure, not competing ideas.
- A serious Ash-native agentic ecosystem likely needs at least four substrate layers:
  1. semantic authoring substrate
  2. program-construction substrate
  3. agent-execution substrate
  4. validation/eval substrate
- ReAct, RM, and RLM-like systems should be tracked both as useful end-user capabilities and as implementation targets that help discover what Ash still lacks.
- ReAct-like systems are likely much closer than full DSPy-style RLM because Ash already has workflow control flow, LLM capability plumbing, and tool-call vocabulary.
- LLM-generated Ash should ideally flow through a canonical loop:
  - generate
  - parse
  - diagnose
  - repair
  - type/effect/policy check
  - execute
- Long term, the desired end state is: LLMs can generate and execute semantically correct Ash workflows, with Ash itself being the language of orchestration.

### What we are uncertain about

- Whether generated-code execution should be owned primarily by the engine or by the interpreter.
- Whether quotations should become first-class runtime values, engine-only meta-objects, or both in staged form.
- How much of RM / ReAct / RLM should be expressible as plain workflows versus requiring dedicated substrate hooks.
- How much dynamic tool dispatch should live in stdlib/workflow space versus engine/runtime support.
- What the minimal evaluation substrate is for declaring the generated-workflow loop usable.

## Current Repo Read (2026-04-20)

This section records the current rough substrate position from live code inspection, not an aspirational target.

### Rough progress estimate

- REPL: ~20-30% toward useful support for AI-native workflow authoring
- LSP/MCP: ~35-45% toward useful support for AI-native workflow authoring
- Actual AST/code-as-data execution substrate: ~5-10%

### What exists today

- REPL has working `:type` and `:ast` inspection surfaces.
- `ash-lsp`, `ash-lsp-core`, `ash-mcp`, and `ash-diagnostic` crates exist in the workspace.
- LSP/MCP provide real, but still partial, semantic tooling.
- MCP already exposes diagnostics, hover, goto-definition, completion, and document symbols.
- Parser/type infrastructure now carries more spans than before, enabling better tooling.

### What is still missing

- No first-class AST/value substrate in `ash_core::Value`.
- No quotation/splice system available as a stable language feature.
- No structural "generated code object -> canonical check -> canonical execute" substrate.
- No rich positional semantic maps for expression-level inferred type/effect hover and repair.
- No evaluation framework specifically for LM-generated workflows and agentic Ash programs.

## Architectural Layers

### 1. Semantic Authoring Substrate

Purpose: help humans and agents write semantically correct Ash.

Includes:
- LSP diagnostics
- MCP tool access to the same analysis
- positional type/effect information
- completion, goto-definition, references, symbols, code actions
- generated-code repair loops driven by semantic feedback

Why it matters:
- enables iterative LLM authoring instead of one-shot text dumping
- lets agents query the language server while building or repairing workflows

### 2. Program-Construction Substrate

Purpose: let Ash manipulate Ash structurally.

Includes:
- quotation
- splice
- macro / derive systems
- structural templates and expansion
- engine-owned meta-programming hooks where appropriate

Why it matters:
- turns generated code from strings into structured artifacts
- allows reusable generation patterns and safe transformations
- makes higher-level workflow combinators practical

### 3. Agent-Execution Substrate

Purpose: express agentic control loops natively in Ash.

Includes:
- LLM and tool orchestration
- trajectory capture
- tool dispatch
- recursive or staged reasoning loops
- typed output handling
- failure/retry/replan structure

Why it matters:
- this is the substrate from which ReAct-, RM-, and RLM-like systems emerge

### 4. Validation / Evaluation Substrate

Purpose: measure and trust generated or agentic workflows.

Includes:
- datasets or example suites
- metrics and scoring
- trace capture
- regression harnesses
- comparison of workflows/models/configurations

Why it matters:
- generated code without evaluation becomes difficult to improve or trust

## Target Capabilities

### Nearer-term target

Define ReAct-like and RM-like orchestration patterns in Ash using:
- workflows
- tool schemas
- tool dispatch
- trajectories
- typed result extraction
- semantic authoring support via LSP/MCP

These should be treated as early implementation targets specifically because they are likely to reveal concrete missing pieces in:
- stdlib tool-dispatch contracts
- structured trajectory vocabulary
- generated-workflow repair loops
- evaluation and regression infrastructure

### Longer-term target

Allow LLMs to generate Ash workflows that can be:
- parsed
- diagnosed
- structurally transformed
- type/effect/policy checked
- evaluated
- executed

At this stage, RM/ReAct/RLM-like systems remain valuable not only as features but as gap-discovery workloads that stress the whole stack.

### Longest-term target

Make Ash a language in which AI systems can author, inspect, transform, validate, and execute Ash workflows natively.

In that world:
- LSP/MCP supports authoring and repair
- quotation/splice supports program construction
- runtime/engine supports authoritative execution
- evals support iterative improvement
- RLM/ReAct/RM become language-level patterns rather than external glue systems

## Design Dimensions

| Dimension | Option A | Option B | Option C |
|-----------|----------|----------|----------|
| Generated code representation | raw source strings | engine-owned syntax/meta objects | runtime AST values |
| Quotation ownership | parser + engine only | parser + engine + limited runtime reflection | full staged runtime AST values |
| Agent repair loop | external tooling only | LSP/MCP-assisted authoring loop | fully internal Ash-hosted repair workflows |
| ReAct/RM implementation style | host-language wrappers | Ash stdlib + minimal runtime hooks | deep runtime primitives |
| RLM implementation style | constrained action algebra | staged/generated Ash helper workflows | open code-execution substrate |

## Suggested Guiding Principles

1. Prefer structural generation over string concatenation.
2. Keep the canonical semantic authority in parser/type checker/engine/runtime, not in tooling.
3. Use LSP/MCP as the authoring and repair surface, not as a replacement for semantic truth.
4. Make generated workflows pass through the same semantic gates as hand-written workflows.
5. Treat evals as a first-class substrate, not a late add-on.
6. Prefer explicit capability and contract boundaries over magical runtime behaviors.

## Candidate Milestone Sequence

### Milestone A: Better semantic tooling for authored/generated code

- richer LSP diagnostics
- expression-level hover
- positional inferred type/effect maps
- stronger MCP query coverage
- code actions for common repair loops

### Milestone B: Structural program construction

- quotation design
- splice design
- derive/macro staging model
- context-specific expansion rules
- engine integration for generated-program checking

### Milestone C: Canonical generated-program check/execute path

- in-memory parse/check APIs
- structured diagnostics and artifacts
- explicit acceptance boundary before execution
- provenance for generated-program execution

### Milestone D: Agentic Ash workflow library surface

- tool-dispatch contracts
- trajectory records
- ReAct-like loops
- RM-like routing / reflection patterns
- structured retry/replan helpers
- typed extraction/result contracts

These should be exercised as real implementation targets, not just theoretical examples.

### Milestone E: Evaluation substrate

- datasets / example suites
- metrics
- trajectory-aware scoring
- comparison and regression tools

This is also where ReAct/RM/RLM-style implementations become useful benchmark programs for the language and tooling itself.

### Milestone F: Advanced recursive systems

- constrained RLM-like systems first
- generated helper workflows
- recursive query/decomposition patterns
- optional future work on broader execution substrates

## Open Questions

1. Should quotation/splice be primarily engine features, runtime features, or staged across both?
2. What is the smallest viable "generated code artifact" that can be passed through canonical checking without becoming stringly?
3. What semantic information must LSP/MCP expose before agent-authored Ash becomes materially reliable?
4. What is the minimal Ash-native substrate needed to make ReAct idiomatic?
5. What is the minimal additional substrate needed to move from ReAct to RM/RLM-like patterns?
6. How should generated-program provenance and policy/effect checks be surfaced to users?

## Related Explorations

- [FUTURE-001: First-Class Workflows](FIRST-CLASS-WORKFLOWS.md)
- [FUTURE-003: Agentic Workflow Exemplars](AGENTIC-WORKFLOW-EXEMPLARS.md)
- [FUTURE-004: Ash Wiki as Human/AI Shared Knowledge Substrate](ASH-WIKI-HUMAN-AI-KNOWLEDGE-SUBSTRATE.md)
- [OTP-001: Erlang/OTP Architecture Analysis](../otp/OTP-001-erlang-otp-analysis.md)
- [OTP-002: Ash OTP Design Considerations](../otp/OTP-002-ash-otp-design.md)

## Cross-Link Notes

- [FUTURE-001: First-Class Workflows](FIRST-CLASS-WORKFLOWS.md) overlaps with this stream because higher-order or workflow-valued abstractions may become important for agentic composition, generated workflow packaging, and reusable orchestration combinators.
- [FUTURE-003: Agentic Workflow Exemplars](AGENTIC-WORKFLOW-EXEMPLARS.md) tracks concrete benchmark families such as ReAct/RM/RLM-like systems as useful workloads and substrate gap-finders for this broader infrastructure stream.
- [FUTURE-004: Ash Wiki as Human/AI Shared Knowledge Substrate](ASH-WIKI-HUMAN-AI-KNOWLEDGE-SUBSTRATE.md) overlaps with this stream because Ash needs a static-first but AI-queryable knowledge substrate to onboard agents, expose authoritative project memory, and support explanation/audit services over the same corpus that future workflows will rely on.
- [OTP-002: Ash OTP Design Considerations](../otp/OTP-002-ash-otp-design.md) overlaps with this stream because supervision, restart semantics, and structured orchestration may become important substrate pieces for robust long-running agent workflows.

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-04-20 | Record AI-native workflows as a separate idea stream in `docs/ideas/` | This direction spans tooling, language design, code construction, runtime execution, and evaluation; it should not be hidden inside any one near-term feature track. |
| 2026-04-20 | Treat LSP/MCP and quotation/splice/macro work as complementary foundations | One supports semantic authoring and repair; the other supports structural program construction. Both are needed for serious AI-authored Ash workflows. |
| 2026-04-20 | Keep this as a live exploration document | The direction is long-range and will evolve as REPL, LSP/MCP, derive, quotation, and execution substrates mature. |
| 2026-04-20 | Treat ReAct/RM/RLM-like systems as both useful capabilities and substrate-discovery targets | They are practical workloads that can surface missing language, library, tooling, and runtime support while also serving as real end-user features. |

## Next Steps

- [ ] Revisit this document as quotation/splice/derive work advances.
- [ ] Add a sharper gap inventory by crate: parser, type checker, engine, interpreter, REPL, LSP/MCP.
- [ ] Decide whether the first serious Ash-native target is ReAct, RM, or a constrained RLM-style loop.
- [ ] Capture the minimal generated-program check/repair/execute API once the ownership boundary (engine vs runtime) is clearer.
- [ ] Add evaluation-substrate notes once there is a concrete dataset/metric direction.
