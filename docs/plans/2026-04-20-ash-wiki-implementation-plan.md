# Ash Wiki Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task once the task files exist.

**Goal:** Establish the Ash wiki as a static-first, human/AI shared knowledge substrate over the existing corpus.

**Architecture:** Stage the work from low-risk corpus semantics toward higher-level services. First define metadata, authority, supersession, and audit contracts. Then add generated registries/views. Only after those static foundations exist should browser conversation, onboarding services, and Ash-native query workflows be layered on top.

**Tech Stack:** Markdown corpus in repo, YAML frontmatter or companion metadata, generated JSON/YAML registries, static HTML/Pandoc rendering, audit/query tooling, future Ash-native workflow services.

---

## Phase 0: Corpus Framing and Scope Freeze

### TASK-645: Ash Wiki Concept Packet

**Status:** Complete

**Task Type:** Docs/Planning

**Objective:** Freeze the initial conceptual contract so later implementation tasks have a stable reference.

**Files:**
- Create: `docs/ideas/future/ASH-WIKI-HUMAN-AI-KNOWLEDGE-SUBSTRATE.md`
- Create: `docs/design/DESIGN-029-ASH-WIKI-KNOWLEDGE-SUBSTRATE.md`
- Create: `docs/spec/SPEC-045-ASH-WIKI.md`
- Modify: `docs/ideas/README.md`
- Modify: `docs/spec/README.md`
- Modify: `CHANGELOG.md`

**Step 1: Review corpus context**

Read the existing future/AI-native workflow notes, documentation indexes, and related design/spec artifacts.

**Step 2: Author the concept packet**

Write the idea, design, and spec documents so they agree on:
- static-first principle
- canonical-adjacent role
- authority/status/health model
- supersession model
- audit/drift role
- onboarding/library-service role

**Step 3: Update indexes and changelog**

Add the new exploration and spec to their indexes. Record the addition in `CHANGELOG.md`.

**Step 4: Verification by Task Type**

Corpus consistency sweep:
- [ ] `docs/ideas/README.md` references the new exploration
- [ ] `docs/spec/README.md` references SPEC-045
- [ ] cross-links inside the new docs are valid
- [ ] terminology is consistent across the packet

---

## Phase 1: Metadata and Corpus Semantics

### TASK-646: Ash Wiki Metadata Carrier Schema

**Status:** Complete

**Task Type:** Docs/Planning

**Objective:** Decide how wiki metadata is stored for canonical and synthesized artifacts.

**Files:**
- Modify: `docs/spec/SPEC-045-ASH-WIKI.md`
- Create: `docs/reference/ash-wiki-metadata-schema.md`
- Create: `docs/plan/tasks/TASK-646-ash-wiki-metadata-carrier-schema.md`

**Step 1: Decide representation**

Choose between:
- YAML frontmatter on all managed docs
- companion metadata files for legacy docs
- hybrid model (frontmatter preferred, registry fallback allowed)

**Step 2: Define required/optional fields**

Include exact field names, enums, and validation rules.

**Step 3: Define migration policy**

Specify how pre-existing docs are incrementally adopted without blocking the whole corpus.

**Step 4: Verification by Task Type**

Corpus consistency sweep:
- [ ] representation decision is explicit
- [ ] validation rules are machine-checkable
- [ ] partial-adoption story is documented

### TASK-647: Ash Wiki Pilot Classification Slice

**Status:** Planned

**Task Type:** Docs/Planning

**Objective:** Apply the metadata model to an initial slice of the corpus to validate practicality.

**Files:**
- Create: `docs/wiki/indexes/pilot-authority-map.md`
- Create: `docs/wiki/indexes/pilot-supersession-map.md`
- Create: `docs/plan/tasks/TASK-647-ash-wiki-pilot-classification-slice.md`

**Step 1: Pick pilot scope**

Use one representative slice, for example:
- AI-native workflow notes
- LSP/MCP/tooling specs
- one implementation-heavy subsystem

**Step 2: Classify artifacts**

Assign type/authority/status/health and identify supersession relations.

**Step 3: Record friction points**

Document where the schema is awkward or underspecified.

**Step 4: Verification by Task Type**

Corpus consistency sweep:
- [ ] no artifact in the pilot slice lacks a classification
- [ ] supersession decisions are explicit
- [ ] stale vs historical is not conflated

---

## Phase 2: Registries and Computed Views

### Task 4: Build the document registry generator

**Task Type:** Substrate

**Objective:** Generate a machine-usable registry of wiki-managed artifacts.

**Files:**
- Create: `tools/ash-wiki-registry/` or equivalent implementation path
- Create: `docs/wiki/registry/documents.json`
- Create: `docs/wiki/registry/documents.md`
- Create: `docs/plan/tasks/TASK-XXX-ash-wiki-document-registry.md`

**Step 1: Write failing registry snapshot test**

Define expected registry entries for a small curated set of documents.

**Step 2: Implement extraction**

Scan managed docs, collect metadata, and emit deterministic outputs.

**Step 3: Verify Real Usage**

Search for non-test consumers of the registry output; if none exist yet, mark this task as substrate-complete but service-follow-on required.

**Step 4: Verification by Task Type**

- [ ] registry emits stable IDs and paths
- [ ] invalid metadata is surfaced clearly
- [ ] markdown view remains human-readable

### Task 5: Generate authority/supersession/drift views

**Task Type:** Substrate

**Objective:** Materialize the first required computed views from the registry.

**Files:**
- Create: `docs/wiki/views/current-normative-surface.md`
- Create: `docs/wiki/views/supersession-map.md`
- Create: `docs/wiki/views/drift-dashboard.md`
- Create: `docs/plan/tasks/TASK-XXX-ash-wiki-computed-views.md`

**Step 1: Define view inputs**

Map each output page to concrete registry sources.

**Step 2: Generate deterministic views**

Ensure the views are stable and suitable for git review.

**Step 3: Verify Real Usage**

Link the views from at least one static entrypoint page or index.

**Step 4: Verification by Task Type**

- [ ] views cite or link to source artifacts
- [ ] supersession view distinguishes full vs partial supersession
- [ ] drift dashboard distinguishes recorded drift from missing evidence

---

## Phase 3: Audit / Lint Substrate

### Task 6: Implement corpus-state lint

**Task Type:** Semantic

**Objective:** Validate metadata completeness, legal state combinations, and explicit supersession rules.

**Files:**
- Create: `docs/spec/SPEC-0XX-ASH-WIKI-AUDIT-RULES.md` or equivalent if split needed
- Create: `tools/ash-wiki-lint/` or equivalent implementation path
- Create: `docs/plan/tasks/TASK-XXX-ash-wiki-corpus-state-lint.md`

**Step 1: Write failing validation tests**

Cases should include:
- missing required fields
- illegal authority/type combinations
- superseded docs missing `superseded_by`
- current docs mis-marked as historical

**Step 2: Implement validator**

Emit machine-readable findings and a human-readable report.

**Step 3: Verify Integration Depth**

Check that the lint is invokable from a normal project workflow, not only unit tests.

**Step 4: Verification by Task Type**

- [ ] failures are specific and evidence-backed
- [ ] legal combinations are documented
- [ ] report is suitable for wiki publication

### Task 7: Implement cross-document consistency and drift registry lint

**Task Type:** Semantic

**Objective:** Validate lineage, supersession references, and recorded drift completeness.

**Files:**
- Create: `docs/wiki/audits/`
- Create: `docs/plan/tasks/TASK-XXX-ash-wiki-drift-audit.md`

**Step 1: Write failing checks**

Cover:
- current docs citing superseded docs without explanation
- plan/task/spec lineage gaps
- drift records lacking evidence or cause classification

**Step 2: Implement registry-aware checks**

Use the document registry and audit records as inputs.

**Step 3: Verify Integration Depth**

Ensure findings can be surfaced both in terminal output and as persisted audit artifacts.

**Step 4: Verification by Task Type**

- [ ] findings identify exact source paths/anchors
- [ ] cause taxonomy is enforced
- [ ] unresolved vs accepted temporary drift is distinguished

---

## Phase 4: Onboarding and Query Services

### Task 8: Produce onboarding/library-service bundles

**Task Type:** Substrate

**Objective:** Export stable onboarding packs for humans and AI agents.

**Files:**
- Create: `docs/wiki/services/onboarding/`
- Create: `docs/wiki/services/glossary/`
- Create: `docs/plan/tasks/TASK-XXX-ash-wiki-onboarding-bundles.md`

**Step 1: Define minimum bundle shape**

Include:
- project overview
- subsystem map
- authority model
- active-work snapshot
- known-drift snapshot
- examples

**Step 2: Generate first bundles**

Emit both human-readable and agent-friendly forms where useful.

**Step 3: Verify Real Usage**

Wire at least one real consumer: agent skill, browser page, or command-line entrypoint.

**Step 4: Verification by Task Type**

- [ ] bundle fields are stable and documented
- [ ] content cites source artifacts
- [ ] consumer interface is explicit

### Task 9: Add structured query workflow layer

**Task Type:** Semantic

**Objective:** Define and implement the first typed query workflows over the corpus.

**Files:**
- Create: `docs/reference/ash-wiki-query-contract.md`
- Create: `docs/plan/tasks/TASK-XXX-ash-wiki-query-workflows.md`

**Step 1: Define query contracts**

At minimum:
- explain topic
- trace history
- show authority surface
- audit drift
- map lineage
- find examples
- onboard agent

**Step 2: Implement initial workflow adapters**

Choose the first host (external tool, Hermes tool wrapper, or Ash-native prototype).

**Step 3: Verify Integration Depth**

Ensure responses distinguish canonical claims, synthesized summaries, computed inferences, and uncertainty.

**Step 4: Verification by Task Type**

- [ ] query contracts are typed and documented
- [ ] at least one real consumer path exists
- [ ] trust/citation rules are enforced

---

## Phase 5: Browser and Static Rendering Integration

### Task 10: Static rendering plus browser conversation hooks

**Task Type:** Substrate

**Objective:** Keep the wiki statically renderable while exposing typed query entrypoints in the browser.

**Files:**
- Create: `docs/wiki/rendering/README.md`
- Create: `docs/plan/tasks/TASK-XXX-ash-wiki-browser-surface.md`

**Step 1: Define static-render contract**

Specify what badges, supersession banners, related links, and query hooks appear in rendered HTML.

**Step 2: Define browser hook contract**

Specify how a rendered page invokes the query layer without becoming dependent on one vendor-specific app.

**Step 3: Verify Real Usage**

Render a pilot slice and confirm it remains useful without interactive services enabled.

**Step 4: Verification by Task Type**

- [ ] static rendering preserves authority/supersession context
- [ ] query hooks degrade gracefully when unavailable
- [ ] output works with Pandoc/static-site workflows

---

## Dependencies and Ordering Notes

- Phase 0 is prerequisite to all later work.
- Phase 1 must be substantially complete before reliable registries or lints are possible.
- Phase 2 is prerequisite to robust service/query consumers.
- Phase 3 and Phase 4 can overlap after registry generation exists.
- Phase 5 should wait until at least one real query workflow and one onboarding bundle exist.

## Risks

1. Over-designing metadata before validating pilot-slice ergonomics.
2. Confusing synthesized pages with canonical truth.
3. Generating opaque registries that are unusable in git review.
4. Treating audit as heuristic guesswork instead of evidence-backed findings.
5. Building browser chat before authority and drift semantics are stable.

## Immediate Recommendation

With TASK-645 and TASK-646 complete, execute TASK-647 next. Do not begin registry or service implementation until the pilot classification slice validates that the metadata and trust model are workable on real corpus material.
