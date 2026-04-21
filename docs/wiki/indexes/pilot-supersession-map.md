---
id: "pilot-supersession-map"
title: "Pilot Supersession Map — LSP/Tooling Cluster"
type: index
authority: advisory
status: current
health: aligned
updated: 2026-04-21
summary: "Supersession relationships within the LSP/tooling pilot slice of the Ash wiki."
carrier: frontmatter
---

# Pilot Supersession Map — LSP/Tooling Cluster

This map records supersession relationships among artifacts in the LSP/tooling cluster. The scope matches the pilot authority map: SPEC-038 through SPEC-043, related designs, plans, and tasks.

## Full Supersessions

### SPEC-038-RESEARCH → SPEC-038

| Field | Value |
|-------|-------|
| Predecessor | SPEC-038-RUST-LSP-MCP-RESEARCH-2025 |
| Successor | SPEC-038-LANGUAGE-SERVER.md |
| Scope | Full |
| Date | 2026-04-15 (research) → 2026-04-16 (spec) |
| Residual Authority | None |

The research document surveyed Rust LSP frameworks, MCP libraries, and caching strategies. Its findings were directly incorporated into SPEC-038's technology choices (tower-lsp-server, rmcp, salsa). The research doc retains `authority: historical` and `status: done` for archaeology but carries no normative weight.

**Presentation note:** The research file should carry a visible supersession banner in any rendered output.

## Partial Supersessions

None identified within this pilot slice.

### Adjacent Partial Supersessions (outside scope, noted for completeness)

- SPEC-021 (Lean Reference) and SPEC-021 (Runtime Observable Behavior) share a numbering slot. This is a naming collision, not a supersession. It should be resolved separately but is noted here because both are referenced from the formalization boundary docs that the LSP tooling cluster depends on.

## Implicit Supersession Chains

The LSP/tooling cluster has a natural dependency chain where later specs implicitly supersede the "placeholder" or "not yet designed" statements in earlier ones:

| Earlier Artifact | Later Artifact | Superseded Surface | Residual Authority |
|------------------|----------------|--------------------|--------------------|
| SPEC-038 §Deferred: formatting | SPEC-042 | Source formatting scope and contract | SPEC-038 retains authority for LSP integration hooks |
| SPEC-038 §Deferred: incremental analysis | SPEC-043 | Salsa integration scope and contract | SPEC-038 retains authority for LSP integration hooks |
| SPEC-038 §Deferred: lint library | SPEC-041 | Lint library extraction scope | SPEC-038 retains authority for LSP integration hooks |
| SPEC-039 binding spans | TASK-570 | Implementation of span fields | SPEC-039 retains normative authority |
| SPEC-040 error spans | TASK-572 | Implementation of span fields on error types | SPEC-040 retains normative authority |

These are **spec-to-implementation** relationships, not true supersessions. The spec remains normative; the task implements the spec's requirements. No `superseded_by` field is needed because the spec retains its authority. However, future wiki tooling should model this as a `implemented_by` or `realized_in` relationship type.

## No-Supersession Artifacts

The following artifact families within the pilot slice have no supersession relationships:

| Family | Reason |
|--------|--------|
| All completed plans (PLAN-031 through PLAN-036) | Plans are advisory records of execution intent; they are done, not superseded |
| All completed tasks (TASK-569 through TASK-576) | Tasks are done work records; they are not superseded |
| Visual programming designs (DESIGN-VP-001, VP-README) | No successor exists yet; exploratory status maintained |
| Reference documents | Normative reference docs are not superseded within this slice |

## Friction Points Discovered

### FP-1: Spec numbering collision (SPEC-021)

SPEC-021 exists in two unrelated files:
- `SPEC-021-LEAN-REFERENCE.md` (Lean 4 reference interpreter)
- `SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md` (runtime observable behavior)

This is not a supersession but a naming collision. The wiki schema's uniqueness constraint on `id` would flag both as `SPEC-021`. Resolution: one must be renumbered. This is a concrete friction point the pilot uncovered.

**Severity:** Medium — schema validation would reject the registry until resolved.
**Recommended action:** Renumber one SPEC-021 to SPEC-046 or SPEC-047.

### FP-2: Implicit dependency chains are not first-class

The spec → plan → task → implementation chain is currently modeled only inside PLAN-INDEX.md task tables. The wiki metadata schema has `related_specs`, `related_plans`, `related_tasks` fields but no explicit `implemented_by` / `realizes` relationship type. For a completed subsystem like the tooling cluster, this is manageable. For active development, the lack of a directed relationship type will make drift detection harder.

**Severity:** Low for the pilot, but expected to matter for active subsystems.
**Recommended action:** Consider adding `realized_by` / `implements` as a follow-on schema extension.

### FP-3: "Draft" status on completed, normative specs

Every spec in this cluster declares `status: draft` despite being fully implemented (all phases done, all tasks complete). The Ash convention is that specs are "implementation-grade drafts" that never get promoted to `current`. This creates a semantic tension with SPEC-045's status vocabulary where `draft` implies pre-acceptance and `current` implies accepted-and-active.

**Severity:** Medium — the wiki schema's `draft` vs `current` distinction loses meaning if all normative specs remain `draft` forever.
**Recommended action:** Either promote completed specs to `status: current` or add a `status: accepted` variant for specs that have been implemented and are actively governing code.

### FP-4: Plan files lack machine-readable IDs

Plans in `docs/plans/` use date-prefixed filenames (e.g., `2026-04-08-manual-ci-workflows-plan.md`) or `PLAN-XXX-` prefixes. The date-prefixed ones have no stable ID for the wiki registry to key on. The PLAN-XXX ones do.

**Severity:** Low — can use filename as ID during registry generation.
**Recommended action:** Establish a convention that all plans get a PLAN-XXX ID.

### FP-5: Design notes are unnumbered

Design notes (`DESIGN-NOTE-CORPUS-ANALYSIS-SUBSTRATE.md`, etc.) use descriptive names rather than numbered IDs. This is fine for human navigation but makes registry keying ambiguous.

**Severity:** Low.
**Recommended action:** Assign DESIGN-NOTE-NNN IDs or accept descriptive IDs in the registry.
