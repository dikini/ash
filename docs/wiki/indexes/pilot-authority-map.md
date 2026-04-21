---
id: "pilot-authority-map"
title: "Pilot Authority Map — LSP/Tooling Cluster"
type: index
authority: advisory
status: current
health: aligned
updated: 2026-04-21
summary: "Per-artifact authority, status, and health classification for the LSP/tooling pilot slice of the Ash wiki."
carrier: frontmatter
---

# Pilot Authority Map — LSP/Tooling Cluster

This map classifies every artifact in the LSP/tooling cluster using the Ash wiki metadata model (SPEC-045). The cluster covers language-server, diagnostic, lint, formatter, and incremental-analysis surfaces.

## Scope

**Pilot slice:** LSP/MCP, parser tooling infrastructure, diagnostic infrastructure, lint library, source formatter, and incremental analysis — corresponding to SPEC-038 through SPEC-043, their related designs and plans, and implementation tasks TASK-569 through TASK-576.

This slice was chosen because it:
- spans all artifact types (spec, design, plan, task, idea, reference)
- includes completed, in-progress, and future work
- contains one clear supersession (research doc → spec)
- touches both Rust implementation and documentation-only concerns
- has historical depth (early research, production specs, completed phases)

## Classification

### Specifications

| ID | Title | Authority | Status | Health | Carrier | Notes |
|----|-------|-----------|--------|--------|---------|-------|
| SPEC-038 | Ash LSP & MCP Interface | normative | current | aligned | frontmatter | Implementation-grade draft; Phase 87 complete (TASK-569) |
| SPEC-038-RESEARCH | Rust LSP & MCP Stack Research (2025) | historical | done | aligned | frontmatter | Consumed into SPEC-038 design decisions; retained for archaeology |
| SPEC-039 | Parser Tooling Infrastructure | normative | current | aligned | frontmatter | Phase 84 complete (TASK-570, TASK-571) |
| SPEC-040 | Diagnostic Infrastructure | normative | current | aligned | frontmatter | Phase 85 complete (TASK-572, TASK-573) |
| SPEC-041 | Ash Lint Library Extraction | normative | current | aligned | frontmatter | Phase 86 complete (TASK-574) |
| SPEC-042 | Ash Source Formatter | normative | current | aligned | frontmatter | Phase 88 complete (TASK-575) |
| SPEC-043 | Incremental Analysis Engine | normative | current | aligned | frontmatter | Phase 89 complete (TASK-576) |

### Designs

| ID | Title | Authority | Status | Health | Carrier | Notes |
|----|-------|-----------|--------|--------|---------|-------|
| DESIGN-VP-001 | Modality Ontology for Visual Programming | exploratory | draft | needs-review | registry-fallback | Early-stage exploration; no implementation phase yet |
| DESIGN-VP-README | Visual Programming README | exploratory | draft | needs-review | registry-fallback | Structural overview only |

### Plans

| ID | Title | Authority | Status | Health | Carrier | Notes |
|----|-------|-----------|--------|--------|---------|-------|
| PLAN-031 | Parser Tooling Infrastructure | advisory | done | aligned | registry-fallback | Phase 84 plan; fully executed |
| PLAN-032 | Diagnostic Infrastructure | advisory | done | aligned | registry-fallback | Phase 85 plan; fully executed |
| PLAN-033 | Ash Lint Library | advisory | done | aligned | registry-fallback | Phase 86 plan; fully executed |
| PLAN-034 | Ash Source Formatter | advisory | done | aligned | registry-fallback | Phase 88 plan; fully executed |
| PLAN-035 | Incremental Analysis Engine | advisory | done | aligned | registry-fallback | Phase 89 plan; fully executed |
| PLAN-036 | LSP & MCP Interface | advisory | done | aligned | registry-fallback | Phase 87 plan; fully executed |

### Tasks

| ID | Title | Authority | Status | Health | Carrier | Notes |
|----|-------|-----------|--------|--------|---------|-------|
| TASK-569 | LSP & MCP Implementation | advisory | done | aligned | registry-fallback | Phase 87; 180h estimate, single monolithic task |
| TASK-570 | Parser Binding Spans | advisory | done | aligned | registry-fallback | Phase 84 |
| TASK-571 | Parser Comment Trivia | advisory | done | aligned | registry-fallback | Phase 84 |
| TASK-572 | Typeck Error Spans | advisory | done | aligned | registry-fallback | Phase 85 |
| TASK-573 | AshLspError Trait | advisory | done | aligned | registry-fallback | Phase 85 |
| TASK-574 | Ash Lint Library Extraction | advisory | done | aligned | registry-fallback | Phase 86 |
| TASK-575 | Ash Source Formatter | advisory | done | aligned | registry-fallback | Phase 88 |
| TASK-576 | Ash LSP Salsa Integration | advisory | done | aligned | registry-fallback | Phase 89 |

### Related Design Notes

| ID | Title | Authority | Status | Health | Carrier | Notes |
|----|-------|-----------|--------|--------|---------|-------|
| (design note) | DESIGN-NOTE-CORPUS-ANALYSIS-SUBSTRATE | advisory | current | aligned | registry-fallback | Shared analysis substrate between wiki and spec processor |
| (design note) | DESIGN-NOTE-QUORUM | advisory | current | needs-review | registry-fallback | May relate to LSP consensus semantics |

### Ideas / Future

| ID | Title | Authority | Status | Health | Carrier | Notes |
|----|-------|-----------|--------|--------|---------|-------|
| (future idea) | AGENTIC-WORKFLOWS-EXEMPLARS | exploratory | draft | aligned | registry-fallback | LSP agent integration is a future exemplar |
| (future idea) | FIRST-CLASS-WORKFLOWS | exploratory | draft | aligned | registry-fallback | May affect LSP workflow-aware completion |

### Reference Documents

| ID | Title | Authority | Status | Health | Carrier | Notes |
|----|-------|-----------|--------|--------|---------|-------|
| REF-FORMAL-BOUNDARY | Formalization Boundary | normative | current | aligned | registry-fallback | Defines proof targets; relevant to LSP type-checking surface |
| REF-TYPE-VOCAB | Type System Vocabulary Guidance | normative | current | aligned | registry-fallback | Used by LSP hover/completion |

## Observations

1. **All specs are Draft + normative.** Every LSP/tooling spec declares `status: draft` but `authority: normative`. This is consistent with Ash convention where specs are implementation-grade drafts that become normative upon acceptance. No friction here.

2. **All phases are done.** The entire tooling cluster (Phases 84-89) is complete. This gives a clean, stable baseline for the pilot — no active drift expected.

3. **No stale or known-drift artifacts.** All items classify as `health: aligned`. This is expected for a completed subsystem that was recently converged.

4. **TASK-569 is a 180-hour monolith.** The LSP implementation was done as a single task. For future wiki traceability, this could benefit from decomposition notes, but it's not a classification problem.

5. **Visual programming designs are the only exploratory items.** DESIGN-VP-001 and its README are early-stage with `needs-review` health because they have not been revisited since initial creation.

6. **Research doc (SPEC-038-RESEARCH) is properly historical.** It was consumed into SPEC-038 and is retained for archaeology only.
