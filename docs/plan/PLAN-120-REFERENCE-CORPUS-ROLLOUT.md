# PLAN-120: Reference Corpus Rollout

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. This is a documentation-infrastructure phase; do not migrate the whole `docs/` corpus or implement a dynamic wiki in this phase.

**Goal:** Establish a separate curated `reference/` corpus for human and AI-agent audiences while preserving `docs/` as the working/historical Ash corpus.

**Architecture:** DESIGN-042 defines the two-corpus model. SPEC-071 defines the reference metadata, authority, crosslinking, tone, and maintenance contract. This plan creates the first pilot slice, static validation tools, and drift-report closeout without rewriting historical docs.

**Tech Stack:** Markdown, YAML frontmatter, Python static validators under `tools/reference/`, current Rust/Ash workspace for evidence commands, existing docs/spec/plan/task/changelog workflow.

---

## 1. Status

**Status:** 📝 Planned after packet creation
**Spec:** [SPEC-071](../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md)
**Design:** [DESIGN-042](../design/DESIGN-042-REFERENCE-CORPUS-AND-DOCUMENTATION-GOVERNANCE.md)
**Task range:** [TASK-946](tasks/TASK-946-reference-corpus-design-packet.md) through [TASK-953](tasks/TASK-953-reference-corpus-closeout-and-drift-report.md)

TASK-946 creates the design/spec/plan packet and registers Phase 124. TASK-947 through TASK-953 remain planned implementation tasks for the reference skeleton, metadata inventory, pilot pages, agent derivatives, validator, example/status classification, and closeout drift report.

## 2. Scope

### In scope

- Top-level `reference/` skeleton.
- Metadata schema and pilot enforcement.
- Authority, methodology, tone, and style guides.
- Pure/Act/Proc/Workflow pilot reference pages.
- Agent concept cards/context-pack index for the same pilot slice.
- Static validator MVP in report/hard-fail mode as appropriate per task.
- Example/status classification for cited pilot examples.
- Drift report and next-slice recommendation.

### Out of scope

- Moving or rewriting the existing `docs/` tree.
- Full language manual completion.
- Dynamic wiki/search service.
- Stabilizing Ash public APIs.
- Full stdlib reference generation beyond pilot hooks.
- Broad IDE/tooling doc audit beyond references needed for the pilot.

## 3. Task table

| Task | Description | Est. Hours | Status |
| --- | --- | ---: | --- |
| [TASK-946](tasks/TASK-946-reference-corpus-design-packet.md) | Create DESIGN-042/SPEC-071/PLAN-120 packet and register Phase 124 | 4 | ✅ Complete |
| [TASK-947](tasks/TASK-947-reference-corpus-inventory-and-metadata-pilot.md) | Inventory/classify a pilot corpus slice and freeze metadata fit/friction points | 8 | 📝 Planned |
| [TASK-948](tasks/TASK-948-reference-skeleton-authority-methodology-style.md) | Create `reference/` skeleton plus authority, methodology, style, and status guides | 8 | 📝 Planned |
| [TASK-949](tasks/TASK-949-pure-act-proc-workflow-reference-pilot.md) | Write Pure/Act/Proc/Workflow pilot reference pages with typed traceability links | 12 | 📝 Planned |
| [TASK-950](tasks/TASK-950-agent-concept-cards-and-context-pack-index.md) | Add agent concept cards, common-confusion warnings, and context-pack index for pilot slice | 8 | 📝 Planned |
| [TASK-951](tasks/TASK-951-reference-static-validator-mvp.md) | Implement static reference metadata/link/path/ID validator MVP | 14 | 📝 Planned |
| [TASK-952](tasks/TASK-952-reference-examples-and-status-classification.md) | Classify cited examples/status/limitations and wire pilot feature matrix | 10 | 📝 Planned |
| [TASK-953](tasks/TASK-953-reference-corpus-closeout-and-drift-report.md) | Close out SPEC-071 pilot with drift report, verification evidence, and next-slice recommendation | 8 | 📝 Planned |

## 4. Tracks

- **Track A — Packet and inventory:** TASK-946/TASK-947 establish the contract and test it against real corpus friction.
- **Track B — Human reference skeleton:** TASK-948/TASK-949 create the reading surface and pilot concept pages.
- **Track C — Agent derivatives:** TASK-950 adds retrieval/context affordances without semantic forking.
- **Track D — Tooling and status:** TASK-951/TASK-952 add static checks and example/status classification.
- **Track E — Closeout:** TASK-953 reconciles drift, limitations, and next rollout slice.

## 5. Decision gates

- **D1:** `docs/` remains the working/historical corpus. The phase must not move or rewrite historical docs wholesale.
- **D2:** `reference/` is the pilot top-level curated reference root unless TASK-947 discovers a concrete blocker.
- **D3:** Reference pages are canonical-adjacent projections, not replacements for current specs.
- **D4:** Human and agent docs share the same semantic spine; agent cards/packs cannot introduce independent semantics.
- **D5:** Every pilot reference page must carry metadata and typed authority links.
- **D6:** Drift must be reported explicitly rather than silently harmonized.
- **D7:** Static validators start narrow and deterministic; no dynamic service or database is required for this phase.
- **D8:** Phase closeout must recommend the next slice based on pilot friction, not bulk-migrate by default.

## 6. Verification strategy

Docs-only tasks use:

```bash
git diff --check
python3 - <<'PY'
from pathlib import Path
required = [
  'docs/design/DESIGN-042-REFERENCE-CORPUS-AND-DOCUMENTATION-GOVERNANCE.md',
  'docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md',
  'docs/plan/PLAN-120-REFERENCE-CORPUS-ROLLOUT.md',
]
for rel in required:
    assert Path(rel).exists(), rel
PY
```

Implementation/tooling tasks add focused Python validator tests, `cargo fmt --check` if Rust is touched, `cargo check --workspace` if public Rust surfaces change, and the repo's broad gates before closeout.

## 7. Closeout expectations

PLAN-120 is complete only when:

1. SPEC-071 acceptance criteria R71-1 through R71-7 are mapped to concrete evidence.
2. PLAN-INDEX, PLAN-120, task files, spec index, reference status pages, and CHANGELOG agree.
3. Reference validators run and report known limitations honestly.
4. Independent review checks for overclaiming, stale authority links, and agent/human semantic divergence.
5. The phase explicitly lists what remains outside the pilot.
