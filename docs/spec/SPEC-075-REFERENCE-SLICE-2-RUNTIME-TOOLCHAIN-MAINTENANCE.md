# SPEC-075: Reference Slice 2 Runtime, Toolchain, and Maintenance Manual

**Status:** Implemented MVP
**Date:** 2026-06-01
**Promotes:** [DESIGN-043](../design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
**Extends:** [SPEC-071](SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md)
**Related:** [SPEC-069](SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md), [SPEC-070](SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md), [SPEC-073](SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md), [SPEC-074](SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md)
**Plan:** [PLAN-125](../plan/PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
**Implementation Tasks:** [TASK-992](../plan/tasks/TASK-992-reference-slice-2-packet.md) through [TASK-999](../plan/tasks/TASK-999-reference-slice-2-closeout.md)

## 1. Summary

SPEC-075 defines the second `reference/` corpus slice. The slice expands the Phase 124 pilot into a maintainable Alpha manual for runtime concepts, standard-library tower surfaces, `ash`/`ashgrove` toolchain procedures, RuntimeKernel status, and reference maintenance procedures.

The slice does not replace SPEC-071. SPEC-071 remains the metadata and corpus-governance baseline. SPEC-075 tightens how Reference Slice 2 uses that baseline: ordinary pages stay reader-focused, detailed maintenance rules live under `reference/maintenance/`, and `verified_against.git_commit` becomes the primary pre-Alpha freshness anchor for staleness inspection.

## 2. Normative terms

- **Subsystem detail page:** A reference page under `reference/stdlib/`, `reference/tools/`, or `reference/runtime/` that owns precise behavior, examples, authority links, and limitations for one subsystem.
- **Reader-journey page:** A page under `reference/getting-started/` that guides a user through a basic task and links to subsystem pages for detail.
- **Maintenance page:** A page under `reference/maintenance/` that defines metadata semantics, refresh, staleness inspection, stale-doc triage, release/reference closeout, or agent-card maintenance.
- **Verification baseline:** The commit recorded in `verified_against.git_commit` at which a page's claims were checked.
- **Derived inspection state:** A state computed by a human, script, or agent by comparing changes since the verification baseline against evidence paths and refresh triggers. `needs-inspection` is derived, not a frontmatter lifecycle status.
- **Advisory release metadata:** Optional version or release-tag metadata that supplements, but does not replace, the verification commit during pre-Alpha development.

## 3. Scope

### 3.1 In scope

1. A subsystem-structured `reference/` expansion for stdlib tower pages, toolchain/CLI pages, RuntimeKernel pages, status pages, maintenance pages, and agent cards.
2. A small reader journey for current Alpha basics: what Ash is, installing, updating, running a program, running under the local daemon, cleanup, and next steps.
3. A maintenance metadata reference that specifies how to interpret and update SPEC-071 frontmatter for Slice 2 pages.
4. A staleness inspection procedure based on diffing from `verified_against.git_commit` to `HEAD` and comparing changed paths against evidence paths and refresh triggers.
5. Optional release/version metadata conventions that remain advisory until an Alpha release line exists.
6. Validator or checker hardening sufficient to prevent unvalidated Slice 2 metadata, broken links, missing agent-card link-backs, or missing closeout evidence.
7. Updated status, drift, verification, and feature-matrix pages for the expanded slice.

### 3.2 Out of scope

1. Moving or rewriting the historical `docs/` corpus.
2. Creating a complete tutorial book or hosted documentation service.
3. Providing advanced practical deployment advice before enough Alpha usage exists.
4. Replacing specs as normative authority.
5. Treating version strings or release tags as stronger than the verification commit before Alpha releases exist.
6. Building a full semantic staleness detector.
7. Changing Rust code or Ash runtime behavior except for reference validator/checker tooling if a task explicitly owns it.

## 4. Information architecture requirements

### 4.1 Subsystem detail pages

Reference Slice 2 MUST create or expand subsystem pages for:

```text
reference/stdlib/README.md
reference/stdlib/act.md
reference/stdlib/proc.md
reference/stdlib/workflow.md
reference/stdlib/result.md
reference/tools/README.md
reference/tools/cli.md
reference/tools/ashgrove.md
reference/tools/ashgrove/install.md
reference/tools/ashgrove/update.md
reference/tools/ashgrove/list-current-default.md
reference/tools/ashgrove/remove-cleanup.md
reference/tools/ashgrove/project-dependencies.md
reference/tools/ashgrove/vendor-deploy.md
reference/tools/ashgrove/trust-and-signing.md
reference/tools/ashgrove/source-payload.md
reference/runtime/README.md
reference/runtime/kernel.md
reference/runtime/admission.md
reference/runtime/artifacts.md
reference/runtime/daemon.md
reference/runtime/policy-profiles.md
```

The exact task may add small helper pages, but these pages are the minimum expected surface.

### 4.2 Reader-journey pages

Reference Slice 2 MUST create a basic journey under `reference/getting-started/`:

```text
reference/getting-started/README.md
reference/getting-started/what-is-ash.md
reference/getting-started/install.md
reference/getting-started/update.md
reference/getting-started/run-a-program.md
reference/getting-started/run-as-daemon.md
reference/getting-started/cleanup.md
reference/getting-started/next-steps.md
```

Reader-journey pages MUST link to subsystem pages for details. They MUST NOT duplicate full subsystem policy in ways that would create independent stale surfaces.

### 4.3 Maintenance pages

Reference Slice 2 MUST create maintenance pages:

```text
reference/maintenance/README.md
reference/maintenance/metadata-reference.md
reference/maintenance/staleness-inspection.md
reference/maintenance/refresh-procedure.md
reference/maintenance/stale-doc-triage.md
reference/maintenance/release-checklist.md
reference/maintenance/agent-card-procedure.md
```

Maintenance procedures MUST live here, not as repeated page-specific playbooks inside ordinary reference pages.

### 4.4 Status pages

Reference Slice 2 MUST create or expand status pages:

```text
reference/status/runtime-kernel.md
reference/status/ashgrove.md
reference/status/reference-maintenance.md
reference/status/alpha-limitations.md
reference/status/drift-report.md
reference/status/verification-evidence.md
reference/status/feature-matrix.md
```

Status pages MUST separate maturity/evidence from learner-facing explanations.

### 4.5 Agent cards

Reference Slice 2 MUST create or expand agent cards for:

```text
reference/agents/cards/stdlib-act.md
reference/agents/cards/stdlib-proc.md
reference/agents/cards/stdlib-workflow.md
reference/agents/cards/stdlib-result.md
reference/agents/cards/ash-cli.md
reference/agents/cards/ashgrove.md
reference/agents/cards/runtime-kernel.md
```

Each agent card MUST include `canonical_page` and `canonical_page_path` body fields and MUST NOT introduce semantic claims not present in the canonical page or linked status page.

## 5. Metadata and staleness requirements

### 5.1 Verification baseline

Every expanded Slice 2 page MUST include a non-`unknown` `verified_against.git_commit` after closeout. The value means the page claims were checked against that commit.

A maintainer MUST NOT update `last_verified` or `verified_against.git_commit` unless the evidence declared by the page has been rechecked. Editing page prose is not enough.

### 5.2 Optional release metadata

Slice 2 pages MAY add advisory release metadata under `verified_against`:

```yaml
release_tag: null
ash_version: unreleased-alpha
```

Until Alpha tags exist, this metadata is secondary. `git_commit` remains the primary freshness anchor.

### 5.3 Evidence paths

Evidence lists MUST be specific enough to support diff-based inspection.

- `specs` SHOULD include normative specs or canonical-adjacent specs backing the page.
- `tasks` SHOULD include closeout/audit/task files backing current status.
- `code` SHOULD include implementation paths whose changes may affect the page.
- `tests` SHOULD include exact commands or test paths used as evidence.
- `examples` SHOULD include cited examples or fixtures when a page claims example behavior.

### 5.4 Refresh triggers

`refresh_trigger` entries SHOULD name concrete paths, path globs, or precise semantic changes. Generic entries such as `runtime changes` or `docs changes` are insufficient for Slice 2 expanded pages unless paired with concrete triggers.

Examples of acceptable trigger style:

```yaml
refresh_trigger:
  - crates/ashgrove/src/** changes
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - ashgrove install/update/remove/cleanup command behavior changes
```

### 5.5 Derived inspection state

The staleness inspection procedure MUST define derived states at least for:

- `no-relevant-changes`;
- `needs-inspection`;
- `stale`;
- `partial`;
- `superseded`.

`needs-inspection` MUST remain derived from changed surfaces unless a later spec extends the frontmatter lifecycle states. It is not the same as `status: stale`.

## 6. Page-content requirements

### 6.1 Reader focus

Ordinary reference pages SHOULD teach their topic directly. They MUST NOT bury the reader task under maintenance instructions.

Ordinary pages MAY cross-link to maintenance indexes or status pages where useful, but detailed refresh procedures MUST live in `reference/maintenance/`.

### 6.2 Limitations

Pages MUST state user-visible Alpha limitations where those limitations affect correct use. Limitations are part of the subject matter, not maintenance noise.

For example, Ashgrove pages MUST NOT imply:

- hosted registry service;
- global/system install roots;
- OS package-manager integration;
- arbitrary SemVer solving;
- broad user-defined source ignore-glob CLI;
- signed release-index-as-digest behavior beyond current evidence.

RuntimeKernel pages MUST NOT imply:

- remote or multi-user daemon API;
- distributed scheduling;
- production init-system integration;
- hot-swapping artifacts for already-running instances;
- provider existence as authority;
- file presence as execution.

### 6.3 Examples

Examples MUST be classified using the existing reference example-status vocabulary or an explicitly compatible extension. Aspirational examples MUST NOT be presented as runnable examples.

## 7. Acceptance criteria

| ID | Requirement | Evidence owner |
| --- | --- | --- |
| A75-1 | Phase packet, DESIGN-043, SPEC-075, PLAN-125, task files, spec index, PLAN-INDEX, and CHANGELOG are created and internally linked. | TASK-992 |
| A75-2 | Maintenance metadata reference and staleness/refresh/stale-doc/release/agent-card procedures exist under `reference/maintenance/`. | TASK-993 |
| A75-3 | Reader-journey basics exist under `reference/getting-started/` and cross-link into subsystem pages instead of duplicating subsystem authority. | TASK-994 |
| A75-4 | Ashgrove and CLI procedure pages cover install, update, list/current/default, remove/cleanup, project dependencies, vendor/deploy, trust/signing, and source-payload policy with explicit non-goals. | TASK-995 |
| A75-5 | RuntimeKernel concept/status pages cover kernel, admission, artifacts, daemon, and policy profiles with integrity/authority caveats. | TASK-996 |
| A75-6 | Stdlib tower pages cover `Act`, `Proc`, `Workflow`, and `Result` as public library surfaces while cross-linking language concept pages. | TASK-997 |
| A75-7 | Agent cards and context-pack/common-confusion surfaces are updated for stdlib, CLI/Ashgrove, and RuntimeKernel without forking canonical claims. | TASK-998 |
| A75-8 | Validator/checker coverage, drift report, verification evidence, feature matrix, status pages, broad docs checks, and independent review close out the slice. | TASK-999 |

## 8. Implementation plan

Implementation is governed by [PLAN-125](../plan/PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md). The plan must keep TASK-993 before the bulk page-writing tasks so new pages are authored against the maintenance model from the start.

## 9. Changelog

### 2026-06-01

- TASK-999 closeout promoted SPEC-075 to Implemented MVP after mapping A75-1 through A75-8 to evidence, updating Reference Slice 2 status pages, adding `reference/status/alpha-limitations.md`, and adding the path-based `--slice reference-slice-2` staleness audit command. Runtime/parser/typechecker/stdlib semantics remain unchanged.
- Initial draft for Reference Slice 2, extending SPEC-071 with subsystem/page-scope requirements, reader-journey basics, runtime/toolchain/stdlib coverage, maintenance procedures, and diff-based staleness inspection semantics.
