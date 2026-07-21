# PLAN-201: Semantic Cleanup Follow-up

**Status:** In progress (10/12 tasks complete; TASK-1971 and TASK-1972 are in final verification).
**Source audit:** [AUDIT-201 Semantic Removal Vs Rename](audits/AUDIT-201-semantic-removal-vs-rename.md).
**Phase:** [PLAN-201: Deprecated Functionality Removal](PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md).

## Goal

Finish Phase 201 semantic cleanup so the repository has no stale code, no stale docs, no stale
tests, and no rename-only completion claims. Removed workflow/tower/capability mechanisms must be
deleted, folded into target Ash primitives, or proven to be private implementation details.

## Target Invariants

- Target Ash entries are ordinary checked functions with effect rows and application runtime
  reporting.
- Public Act/Proc/Workflow tower forms, workflow declarations, direct capability/provider
  authority forms, and removed callable syntax are not current Ash.
- Provider profiles, row admission, contract/evidence helpers, process/channel primitives, and
  application reports are the retained target abstractions.
- Public APIs, diagnostics, tests, docs, templates, and examples must not teach a separate
  workflow/tower compatibility model.

## Workstreams

| Workstream | Goal | Primary risk |
|------------|------|--------------|
| Parser/lowering | Remove residual workflow-form carriers and ensure target function contracts lower directly. | Retaining `WorkflowDef`/contract adapter shapes under neutral names. |
| Type/effect/Core/TCIR/AMIR | Align entry artifacts, contract evidence, ambient effects, and runtime provenance with target rows/profiles. | Entry/tower metadata survives under computation names. |
| Runtime/engine/interpreter | Unify callable-entry, child-entry, projection, and reports with function/effect-row execution. | Separate runtime category duplicates target function invocation. |
| Tooling/LSP/formatter/CLI/templates | Remove public stale categories from reports, diagnostics, templates, and generated metadata. | Tooling schemas keep old distinctions despite target syntax. |
| Docs/reference/examples/tests | Quarantine historical pages and rewrite/delete tests that preserve old semantics. | Historical docs remain linked as current guidance. |

## Deletion Tasks

| Task | Stale mechanism | Target replacement | Likely files/modules | Tests/docs affected | Risk and sequencing |
|------|-----------------|--------------------|----------------------|---------------------|---------------------|
| TASK-1971 | Residual workflow-form parser/lowering carriers not needed for current contracts. | Direct target function contract events and contract/evidence helpers. | `crates/ash-parser`, parser lowering tests, engine metadata tests. | Delete or rewrite workflow-form-only tests; add carrier absence tests. | Depends on proving current `requires`/`ensures` paths do not need old declaration adapters. |
| TASK-1972 | TCIR/AMIR entry-artifact carriers that preserve workflow-artifact semantics. | Effect-row computation artifacts over target Core/TCIR/AMIR. | `crates/ash-core`, `crates/ash-typeck`, runtime artifact builder. | Rewrite artifact provenance tests to assert function/effect-row identity. | High blast radius; run core/typeck/engine checks together. |
| TASK-1973 | Entry-projection wrappers descended from workflow projection. | Result/application report projection for ordinary function execution. | `crates/ash-interp`, `crates/ash-engine`, focused projection tests. | Rewrite projection tests around target function results and runtime reports. | Coordinate with runtime report schema task. |
| TASK-1974 | Historical reference pages still routed as current docs. | Archived/historical prose or target effects/contracts/process/application pages. | `reference/`, `docs/spec`, `docs/notes`, `docs/plan` indexes. | Update read paths and stale-claim sweeps. | Docs-only but broad link/index impact. |

## Refactor Tasks

| Task | Mechanism to fold into target primitive | Target primitive | Likely files/modules | Required proof |
|------|-----------------------------------------|------------------|----------------------|----------------|
| TASK-1975 | Callable-entry runtime registry. | Ordinary function metadata/cache with effect-row admission. | `crates/ash-interp`, `crates/ash-engine`, CLI run path. | Tests prove registry-backed invocation and ordinary function invocation share type/effect checking and no separate user-visible category. |
| TASK-1976 | Child-entry registry. | Process/channel spawn, join, and cancellation primitives. | `crates/ash-interp`, runtime state, process/channel tests. | Tests prove child execution is process primitive based and does not require workflow-style registry semantics. |
| TASK-1977 | Application/entry reports and daemon JSON. | Application runtime reports over checked target functions. | `crates/ash-cli`, `crates/ash-core`, `crates/ash-engine`. | Schema tests show no workflow/tower categories and docs describe report fields as application runtime metadata. |
| TASK-1978 | Contract intrinsic storage and source-contract carriers. | Contract/evidence helpers over target function contracts. | `crates/ash-typeck`, `crates/ash-core`, parser lowering. | Tests prove contract intrinsics attach to target function contracts without WorkflowForm dependency. |
| TASK-1979 | Ambient/entry effect context. | Row/profile effect typing. | `crates/ash-typeck`, runtime verification, obligation checks. | Tests prove effect context is row/profile-driven and contains no entry-only tower special case. |

## Documentation Tasks

| Task | Documentation cleanup | Files | Acceptance |
|------|-----------------------|-------|------------|
| TASK-1980 | Archive or relabel workflow/tower stdlib/language cards so agent-facing references cannot be read as current. | `reference/agents`, `reference/language`, `reference/stdlib`. | Productive read paths route to target effects/contracts/process/application docs. |
| TASK-1981 | Add a Phase 201 removed-form authority page that lists historical terms and target replacements without source-shaped examples. | `docs/reference` or `reference/status`. | Docs gate passes; no Ash code block contains removed syntax. |
| TASK-1982 | Rewrite tests whose only retained purpose is old semantic compatibility under target names. | affected `crates/*/tests`. | Tests assert target primitives, or are deleted with audit evidence. |

## Gates

Add or extend gates so semantic cleanup cannot regress:

- Public API/name gate for callable-entry and child-entry registry categories after unification.
- IR artifact gate proving old workflow/tower artifact names and variants cannot re-enter.
- Runtime report schema gate proving daemon/application reports expose target application fields
  only.
- Docs read-path gate preventing productive indexes from linking historical workflow/tower pages
  as current guidance.
- Target-function unification tests proving `fn main` entry execution uses the ordinary function
  type/effect path plus application report projection.

## Closeout Audit

Before Phase 201 closes, TASK-1968 must prove each requirement rather than relying on green token
gates:

- old syntax does not parse/check/lower/run/format/template as valid current Ash;
- active repository Ash code and Rust literals contain no removed source forms;
- retained runtime/parser/typechecker/tooling mechanisms are target-justified or assigned to this
  plan;
- public docs, examples, templates, diagnostics, and schemas do not teach stale distinctions;
- full closeout commands from PLAN-201 pass and stale-claim sweeps are recorded.

## Acceptance Criteria

- Every `Plan required` or `Refactor to target primitive` row from the semantic audit has a task
  above.
- Function/effect-row unification owns callable-entry registry, child-entry registry, projection,
  artifact, and report cleanup.
- Documentation cleanup distinguishes historical prose from current target Ash guidance.
- Gates require behavior-removal evidence, not just vocabulary changes.
