# PLAN-105: Unified Type/Module Pipeline and Semantic Summaries

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Phase 109 is SPEC-A from DESIGN-034. Do not implement parser/typechecker/engine work without the corresponding task file. Do not implement `type fn`, sealed type domains, normalization, generalized associated type-family computation, or proposition solving under this plan.

**Goal:** Implement [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md) by routing ordinary `type` metadata through the normal `ModuleFile`, core semantic summary, engine import/export, and TypeEnv registration path.

**Architecture:** Phase 109 is a Tier 0 substrate phase. `ash-parser` preserves ordinary type declarations as surface module items. `ash-core` owns canonical type identities and ordinary-type `ModuleSemanticSummary` carriers. `ash-engine` builds/transports/imports/exports ordinary-type summaries without owning type semantics. `ash-typeck` consumes ordinary-type summaries with two-pass declaration, validation, and representation exposure. TASK-789 quarantined legacy source-snippet ordinary type scanning behind explicit compatibility scopes; snippet scanning is not the normal semantic path. Phase 109 preserves the Phase 108 `PublicWorkflowSummary` transport path; ordinary-type summaries augment workflow summaries, they do not replace them.

**Tech Stack:** Rust 2024, `ash-parser`, `ash-core`, `ash-engine`, `ash-typeck`, existing `ModuleFile`, `TypeDef`, `TypeEnv`, `ModuleGraph`, module loader, ADT/interface metadata, and CLI check surfaces.

---

## Phase 109: Unified Type/Module Pipeline and Semantic Summaries

**Status:** ✅ Complete (TASK-780 through TASK-792 complete)
**Spec:** [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md)
**Design:** [DESIGN-034](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
**Depends on:** [SPEC-003](../spec/SPEC-003-TYPE-SYSTEM.md), [SPEC-009](../spec/SPEC-009-MODULES.md), [SPEC-012](../spec/SPEC-012-IMPORTS.md), [SPEC-020](../spec/SPEC-020-ADT-TYPES.md), [SPEC-030](../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md)

### Task table

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-780](tasks/TASK-780-unified-type-module-pipeline-spec-plan-packet.md) | Promote DESIGN-034 SPEC-A into SPEC-057/PLAN-105 and register Phase 109 | Docs/Planning | 4 | ✅ Complete |
| [TASK-781](tasks/TASK-781-current-type-pipeline-audit-and-semantic-summary-gate.md) | Audit current type pipeline and freeze the semantic-summary implementation gate | Docs/Substrate | 4 | ✅ Complete |
| [TASK-782](tasks/TASK-782-modulefile-ordinary-type-declaration-surface-integration.md) | Parse ordinary `type` declarations as normal ModuleFile items | Parser/Substrate | 6 | ✅ Complete |
| [TASK-783](tasks/TASK-783-core-canonical-type-ids-and-module-semantic-summary-carriers.md) | Add core-owned canonical type IDs and ModuleSemanticSummary carriers | Core/Substrate | 8 | ✅ Complete |
| [TASK-784](tasks/TASK-784-surface-to-core-type-metadata-lowering-and-source-anchors.md) | Lower surface type metadata into core summaries with source anchors | Core/Parser | 6 | ✅ Complete |
| [TASK-785](tasks/TASK-785-engine-summary-builder-and-export-collection-from-modulefile.md) | Build engine export summaries from parsed ModuleFile/core summaries | Engine/Substrate | 8 | ✅ Complete |
| [TASK-786](tasks/TASK-786-import-pub-use-glob-visibility-and-opacity-summary-rules.md) | Implement import/pub-use/glob summary transport with visibility and opacity | Engine/Type | 7 | ✅ Complete |
| [TASK-787](tasks/TASK-787-typeenv-two-pass-registration-from-semantic-summaries.md) | Consume semantic summaries in TypeEnv with two-pass registration | Type/Substrate | 8 | ✅ Complete |
| [TASK-788](tasks/TASK-788-interface-and-associated-member-identity-summary-plumbing.md) | Preserve current interface and associated-member identities in summaries | Type/Substrate | 6 | ✅ Complete |
| [TASK-789](tasks/TASK-789-legacy-type-snippet-scanner-quarantine-removal.md) | Quarantine or remove legacy source-snippet ordinary type scanning | Engine/Compatibility | 5 | ✅ Complete |
| [TASK-790](tasks/TASK-790-diagnostics-negative-tests-and-non-interference-coverage.md) | Add diagnostics, negative tests, and non-interference coverage | Semantic/Tests | 6 | ✅ Complete |
| [TASK-791](tasks/TASK-791-spec-a-closeout-docs-examples-verification.md) | Reconcile docs/examples/status/changelog and run closeout verification | Docs/Planning | 4 | ✅ Complete |
| [TASK-792](tasks/TASK-792-phase109-review-remediation.md) | Remediate post-closeout review findings across docs, TypeEnv, engine summary transport, and stdlib semantics | Review/Hardening | 6 | ✅ Complete |

Estimated total: 78 hours.
Remaining after TASK-792: 0 hours.

## Tracks

### Track A: Spec Gate and Audit

- TASK-780 creates the normative SPEC-A packet.
- TASK-781 audits current parser, core, engine, and typechecker paths before implementation begins, producing `docs/plan/audits/TASK-781-type-pipeline-audit.md`.

### Track B: Parser/Core Semantic Substrate

- TASK-782 makes ordinary type declarations normal ModuleFile items.
- TASK-783 creates core-owned canonical IDs and summary carriers.
- TASK-784 lowers parsed type metadata into core summaries with source anchors.

### Track C: Engine Module Import/Export Path

- TASK-785 builds and exports ordinary-type summaries from ModuleFile/core summaries across `check_module_file`, `collect_module_exports`, `load_ordinary_file`, `parse_file`, `parse_workflow_source_with_imports`, and runtime stdlib type discovery entry points, while preserving existing `InlineCallable.workflow_summary` / `PublicWorkflowSummary` export data.
- TASK-786 applies named import, glob import, `pub use`, visibility, and opacity rules, including non-regression coverage that workflow-returning callables keep their `PublicWorkflowSummary` data through those import/re-export paths.
- TASK-789 quarantines legacy source-snippet type scanning behind explicit compatibility scopes after the normal path is proven.
- TASK-792 hardens engine import/export alias transport and export/check validation after independent review.

### Track D: Typechecker Consumption and Identity Plumbing

- TASK-787 consumes ordinary-type summaries through TypeEnv two-pass declaration/validation/exposure, including canonical identity-aware keys or alias-to-identity bindings and explicit placeholder states. It must register type identities before imported callable signatures, imported `PublicWorkflowSummary` users, and imported `do:Workflow` / `[...]: Workflow` composition checks.
- TASK-788 preserves current interface and associated-member identity metadata in the same summary substrate without adding recursive associated-family computation.

### Track E: Diagnostics and Closeout

- TASK-790 adds negative diagnostics and non-interference coverage.
- TASK-791 reconciles docs/status/changelog and performs final verification.
- TASK-792 reconciles post-closeout review findings, including stale status surfaces, TypeEnv summary authority gaps, engine alias leakage, and stdlib semantic preservation.

## Implementation Constraints

1. Ordinary `type` declarations must be parsed as normal ModuleFile items.
2. Source-snippet scanning is not the normal semantic path.
3. `ash-core` owns canonical IDs and semantic summaries.
4. `ash-engine` transports summaries; it does not own type semantics.
5. `ash-typeck` consumes summaries with two-pass declaration then validation/exposure.
6. Import order must not affect type identity or sibling type resolution.
7. Re-exports preserve canonical identity.
8. Private representations must not leak through public summaries.
9. Existing ADT, module/import, interface, associated type, workflow, capability/resource, do, and comprehension behavior must be preserved.
10. Opaque exported type identities are allowed only for existing explicit builtin/opaque exceptions; this phase does not add general representation-hiding syntax.
11. No task in this phase may implement `type fn`, sealed domains, type-level normalization, generalized associated-family computation, or proposition solving.
12. Ordinary-type `ModuleSemanticSummary` work must preserve Phase 108 workflow-summary transport: `ash_core::workflow_carrier::PublicWorkflowSummary`, `InlineCallable.workflow_summary`, `Workflow.imported_workflow_summaries`, and `TypeEnv` public workflow-summary bindings.

## Verification Strategy

Every implementation task must include focused tests for its changed layer and appropriate non-regression coverage. The phase-level closeout must verify:

1. ordinary type declarations parse as ModuleFile definitions;
2. core summaries carry public ordinary type identities;
3. engine import/export no longer depends on source-snippet type discovery in the normal path;
4. TypeEnv consumes summaries with two-pass registration;
5. public type identities import consistently;
6. private type representations do not leak;
7. constructors import only when representation visibility allows;
8. deferred feature syntax remains rejected or explicitly unsupported;
9. existing ADT/interface/workflow/capability/resource/do/comprehension tests are unaffected;
10. Phase 108 TASK-777 workflow summary import/export regressions still pass, including preservation of imported `PublicWorkflowSummary` data through supported named/glob/`pub use` paths;
11. docs/spec index, PLAN-INDEX, task statuses, and CHANGELOG are reconciled.

## Decision Gates

- D1: Ordinary `type` declarations must be ModuleFile definitions; snippet scanning is not authoritative.
- D2: `ash-core` owns semantic summary carriers and canonical IDs.
- D3: Public/private/crate visibility and opacity are summary invariants.
- D4: TypeEnv registration is two-pass and import-order independent.
- D5: SPEC-A does not implement type computation.
- D6: SPEC-A defines the ordinary type summary roadbed; SPEC-F later extends it with computation-grade summary facts.

## Completion Checklist

- [x] SPEC-057 is registered in docs/spec/README.md.
- [x] PLAN-105 and TASK-780 through TASK-792 are registered in PLAN-INDEX.md.
- [x] Ordinary type declarations are parsed as ModuleFile definitions.
- [x] Core-owned ModuleSemanticSummary or equivalent exists.
- [x] Surface ordinary type declarations lower to core TypeDef values and module-anchored summaries.
- [x] Public ordinary type identities export/import through summaries.
- [x] TypeEnv consumes summaries using two-pass registration.
- [x] Private representations do not leak downstream.
- [x] Constructors are imported/exposed only when representation visibility allows.
- [x] Source-snippet ordinary type scanning is removed or fenced behind documented compatibility tests.
- [x] Existing ADT/interface/workflow/capability/resource/do/comprehension regressions pass for the focused Phase 109 gates, and TASK-792 resolves the prior broad example-corpus parse failure in `examples/06-capability-implementations/01-mock-internal-kv.ash`.
- [x] TASK-787 through TASK-792 docs/changelog/status are reconciled.
- [x] Controller independent-review findings are remediated and covered by regression tests, including alias self-reference rewriting, selected representation dependency transport, TypeEnv summary validation, stdlib corpus repair, and broad verification.
