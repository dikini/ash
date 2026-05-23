# TASK-947 reference corpus inventory and metadata pilot

Status: Complete for Phase 124 pilot.

Recommendation: keep `reference/` as the top-level curated corpus name. The pilot found no path or authority blocker. The name is short, distinct from `docs/`, and matches DESIGN-042/SPEC-071.

## Inventory slice

| ID | Artifact | Corpus area | Kind | Authority | Lifecycle | Health | Owner | Source of truth / notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| inv-01 | docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md | docs/spec | spec | normative metadata contract | Draft -> Implemented MVP candidate | fit | reference-corpus | Defines required metadata and R71 acceptance. |
| inv-02 | docs/design/DESIGN-042-REFERENCE-CORPUS-AND-DOCUMENTATION-GOVERNANCE.md | docs/design | design | design rationale | Draft design | fit | reference-corpus | Explains two-corpus split. |
| inv-03 | docs/plan/PLAN-120-REFERENCE-CORPUS-ROLLOUT.md | docs/plan | plan | execution plan | Phase plan | fit | reference-corpus | Owns TASK-946 through TASK-953. |
| inv-04 | docs/plan/tasks/TASK-947-reference-corpus-inventory-and-metadata-pilot.md | docs/plan/tasks | task | implementation evidence | Complete after this artifact | fit | reference-corpus | Inventory task. |
| inv-05 | docs/plan/tasks/TASK-948-reference-skeleton-authority-methodology-style.md | docs/plan/tasks | task | implementation evidence | Complete after skeleton | fit | reference-corpus | Skeleton task. |
| inv-06 | docs/plan/tasks/TASK-949-pure-act-proc-workflow-reference-pilot.md | docs/plan/tasks | task | implementation evidence | Complete after pilot pages | fit | language | Language pilot task. |
| inv-07 | docs/plan/tasks/TASK-950-agent-concept-cards-and-context-pack-index.md | docs/plan/tasks | task | derivative evidence | Complete after cards | fit | agents | Agent derivative task. |
| inv-08 | docs/plan/tasks/TASK-951-reference-static-validator-mvp.md | docs/plan/tasks | task | tooling evidence | Complete after validator | fit | tooling | Static validation task. |
| inv-09 | docs/plan/tasks/TASK-952-reference-examples-and-status-classification.md | docs/plan/tasks | task | status evidence | Complete after classifications | fit | examples/status | Example labels task. |
| inv-10 | docs/plan/tasks/TASK-953-reference-corpus-closeout-and-drift-report.md | docs/plan/tasks | task | closeout evidence | Complete after drift report | fit | reference-corpus | R71 evidence task. |
| inv-11 | docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md | docs/spec | spec | current tower semantics | Implemented MVP | fit | language/runtime | Main authority for Pure/Act/Proc/Workflow and generalized do. |
| inv-12 | docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md | docs/spec | spec | runtime authority | Implemented MVP | fit | runtime | RuntimeKernel and capability-provider authority. |
| inv-13 | docs/spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md | docs/spec | spec | historical/current subset | Implemented, older than SPEC-069 | friction | language | Useful do background, but SPEC-069 supersedes current alpha boundaries. |
| inv-14 | docs/spec/SPEC-047-ACT-MONAD.md | docs/spec | spec | historical/current subset | Implemented, older than SPEC-069 | friction | language | Act details need current opaque-runtime wording. |
| inv-15 | docs/spec/SPEC-048-PROC-LIBRARY.md | docs/spec | spec | historical/current subset | Implemented, older than SPEC-069 | friction | language | Proc library details predate RuntimeKernel framing. |
| inv-16 | docs/spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md | docs/spec | spec | process semantics | Implemented, older than SPEC-070 | friction | runtime | Process semantics are relevant but not the whole alpha runtime. |
| inv-17 | docs/spec/SPEC-051-WORKFLOW-SEMANTICS.md | docs/spec | spec | workflow semantics | Implemented, older than SPEC-069 | friction | language | Workflow contract must be narrowed by explicit lifts/no implicit tower collapse. |
| inv-18 | std/src/act.ash | std/src | stdlib module | live stdlib surface | current alpha | fit | stdlib | Public Act declarations. |
| inv-19 | std/src/proc.ash | std/src | stdlib module | live stdlib surface | current alpha | fit | stdlib | Public Proc declarations. |
| inv-20 | std/src/workflow.ash | std/src | stdlib module | live stdlib surface | current alpha | fit | stdlib | Public Workflow declarations. |
| inv-21 | std/src/result.ash | std/src | stdlib module | live stdlib surface | current alpha | fit | stdlib | Result is separate from Act. |
| inv-22 | examples/07-phase105/01-do-act.ash | examples | example | executable/example corpus | historical-pass candidate | friction | examples | Needs label before citation as normative. |
| inv-23 | examples/07-phase105/03-do-proc-from-act.ash | examples | example | executable/example corpus | historical-pass candidate | friction | examples | Demonstrates explicit lift but phase age matters. |
| inv-24 | examples/09-phase108/04-workflow-explicit-lifts.reference.ash | examples | reference example | reference-only | fit | examples | Reference-only example, not claimed executable. |
| inv-25 | examples/09-phase108/06-legacy-workflow-migration-warning.ash | examples | example | expected diagnostic | fit | examples | Useful expected-fail/historical warning. |
| inv-26 | crates/ash-typeck/tests/alpha_visible_tower_acceptance_matrix.rs | code/tests | test | live regression evidence | current | fit | typechecker | Acceptance evidence for tower boundaries. |
| inv-27 | crates/ash-engine/tests/workflow_contracts_integration.rs | code/tests | test | live regression evidence | current | fit | engine | Runtime workflow contract checks. |
| inv-28 | crates/ash-cli/tests/example_corpus_check.rs | code/tests | test | example-corpus evidence | current | fit | cli/examples | Classifies example executability indirectly. |
| inv-29 | crates/ash-cli/src/commands/run.rs | code | CLI runtime code | current implementation | current | fit | cli/runtime | Runtime invocation evidence path. |
| inv-30 | crates/ash-core/src/runtime_kernel.rs | code | runtime carriers | current implementation | current | fit | runtime | RuntimeKernel identity/admission carrier evidence. |

## Metadata friction

- Historical specs remain useful but can overstate current authority unless `verified_against` and `related.historical_rationale` are separate. No schema change is needed.
- Some examples are reference-only or historical. The pilot records labels in `reference/examples/README.md` instead of treating every cited `.ash` file as a passing test.
- Agent cards need fields such as `canonical_page`, `retrieval_tags`, and `must_check_before_editing` beyond SPEC-071 required frontmatter. The validator accepts these as card body metadata for the pilot.
- Markdown links to repo paths are relative to the reference page location; authors must avoid bare repo-relative Markdown links inside nested pages.
- `canonical` authority is too strong for hand-written pilot pages. The pilot uses `canonical-adjacent` for human pages and `derivative` for agent pages.

## Lifecycle conclusion

The lifecycle model fits the pilot: hand-written reference pages are `current` or `partial`, historical rationale remains in `docs/`, and agent derivatives are marked `derivative`. The SPEC-071 field set fits the pilot without schema edits. Deferred issues are broader generated manifests, full CLI/stdlib declaration matching, and deciding when a page can become `stable`.
