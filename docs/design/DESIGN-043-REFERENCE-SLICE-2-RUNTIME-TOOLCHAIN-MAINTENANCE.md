# DESIGN-043: Reference Slice 2 Runtime, Toolchain, and Maintenance Manual

**Status:** Draft design note — promoted to implementation-grade draft by [SPEC-075](../spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
**Date:** 2026-06-01
**Related:** [SPEC-071](../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md), [SPEC-073](../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md), [SPEC-074](../spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md), [DESIGN-042](DESIGN-042-REFERENCE-CORPUS-AND-DOCUMENTATION-GOVERNANCE.md)
**Plan:** [PLAN-125](../plan/PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)

## 1. Summary

Reference Slice 1 proved the `reference/` corpus shape: frontmatter, authority links, pilot pages, agent cards, examples, drift reports, and a repo-local validator. It did not yet make `reference/` a practical Alpha manual. The current corpus is still mostly a language pilot plus status scaffolding.

Reference Slice 2 should make the corpus useful for the next Alpha development boundary without pretending it is a complete book. The slice should add a subsystem-structured runtime, stdlib, and toolchain reference; add a small reader journey for the basic lifecycle; and, most importantly, define the maintenance procedures that keep the reference correct as Ash changes quickly before Alpha release.

## 2. Design principles

### 2.1 Subsystem detail is the durable structure

Subsystem pages are the canonical-adjacent detail surfaces for implementation-backed claims:

- `reference/stdlib/` owns public library and tower operation reference.
- `reference/tools/` owns command and procedure reference for `ash` and `ashgrove`.
- `reference/runtime/` owns RuntimeKernel, admission, artifact, daemon, and policy-profile explanation.
- `reference/status/` owns maturity, verification evidence, known limitations, and drift.
- `reference/maintenance/` owns metadata semantics and refresh procedures.

This keeps implementation details, authority links, tests, and evidence close to the subsystem they describe.

### 2.2 Reader journeys are entry paths, not authority copies

Reader-journey pages under `reference/getting-started/` should answer first questions:

- What is Ash?
- How do I install Ash?
- How do I update Ash?
- How do I run an Ash program?
- How do I run an Ash program under the local daemon?
- How do I remove or clean up Ash toolchains and caches?
- Where do I go next?

These pages should cross-link to subsystem pages for exact policy, command, and evidence details. They should not duplicate full install/update/runtime semantics. Cross-linking is the design mechanism, not a documentation smell.

### 2.3 Maintenance belongs in metadata and procedures, not ordinary page bodies

Normal pages should teach the user task or subsystem. They should not contain page-specific maintenance playbooks.

Maintenance detail belongs in:

- required frontmatter metadata;
- `reference/maintenance/metadata-reference.md`;
- `reference/maintenance/staleness-inspection.md`;
- `reference/maintenance/refresh-procedure.md`;
- `reference/maintenance/stale-doc-triage.md`;
- `reference/maintenance/release-checklist.md`;
- `reference/maintenance/agent-card-procedure.md`.

Ordinary pages may include small cross-links to maintenance docs from an index or status page, but the page body should remain reader-focused.

### 2.4 Git commit is the pre-Alpha freshness anchor

Ash is still moving toward Alpha. Version strings and release tags are weak until there is an Alpha release line. The strongest freshness anchor is the commit recorded in `verified_against.git_commit`.

A page's verification commit means: the page's claims were checked against that repository state. A tool or agent can diff from that commit to `HEAD` and compare changed paths against the page's evidence paths and refresh triggers.

The reference should distinguish:

1. declared status: stored in page frontmatter, maintained by humans;
2. verification baseline: commit, date, evidence paths, commands;
3. derived inspection state: computed by a script or agent, for example `needs-inspection` after relevant changed files are found.

`needs-inspection` should not be stored as the ordinary lifecycle status unless inspection proves the page is actually stale. It is a derived state.

## 3. Information architecture

### 3.1 Reader journey

Planned pages:

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

The journey should be intentionally conservative. It should document supported Alpha basics and point to subsystem detail. Practical advice, risk playbooks, deployment patterns, and troubleshooting recipes can expand later after enough real use accumulates.

### 3.2 Stdlib and tower reference

Planned pages:

```text
reference/stdlib/README.md
reference/stdlib/act.md
reference/stdlib/proc.md
reference/stdlib/workflow.md
reference/stdlib/result.md
```

These pages complement existing `reference/language/` concept pages. The language pages should explain concepts and syntax. The stdlib pages should explain current public operations, examples, evidence, limitations, and common stale claims.

The slice should preserve this distinction:

- `reference/language/effects-act.md` explains what `Act` means.
- `reference/stdlib/act.md` explains the current `act` library surface and use.

The same split applies to `Proc`, `Workflow`, and generalized do.

### 3.3 Toolchain and CLI reference

Planned pages:

```text
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
```

`ash` and `ashgrove` should be separated. `ash` runs, checks, tests, and controls daemon behavior for programs. `ashgrove` installs, updates, removes, cleans up, vendors, and manages toolchains/dependencies.

### 3.4 RuntimeKernel reference

Planned pages:

```text
reference/runtime/README.md
reference/runtime/kernel.md
reference/runtime/admission.md
reference/runtime/artifacts.md
reference/runtime/daemon.md
reference/runtime/policy-profiles.md
```

These pages should teach the execution model without requiring readers to inspect SPEC-069 or SPEC-070 first. They should make integrity and authority boundaries explicit: file presence does not execute code, provider existence is not authority, admission happens before user body execution, and running daemon instances keep admitted artifact identity across reload.

### 3.5 Status and evidence pages

Planned additions or expansions:

```text
reference/status/runtime-kernel.md
reference/status/ashgrove.md
reference/status/reference-maintenance.md
reference/status/alpha-limitations.md
reference/status/drift-report.md
reference/status/verification-evidence.md
reference/status/feature-matrix.md
```

Status pages keep maturity, caveats, and evidence centralized. They prevent ordinary task pages from carrying full verification matrices.

### 3.6 Maintenance pages

Planned pages:

```text
reference/maintenance/README.md
reference/maintenance/metadata-reference.md
reference/maintenance/staleness-inspection.md
reference/maintenance/refresh-procedure.md
reference/maintenance/stale-doc-triage.md
reference/maintenance/release-checklist.md
reference/maintenance/agent-card-procedure.md
```

Maintenance pages are part of the deliverable. The reference cannot scale unless maintainers and agents know how to update metadata, detect possible staleness, and avoid false freshness claims.

### 3.7 Agent derivatives

Planned or expanded cards:

```text
reference/agents/cards/stdlib-act.md
reference/agents/cards/stdlib-proc.md
reference/agents/cards/stdlib-workflow.md
reference/agents/cards/stdlib-result.md
reference/agents/cards/ash-cli.md
reference/agents/cards/ashgrove.md
reference/agents/cards/runtime-kernel.md
```

Agent cards are derivative. They must link back to canonical pages and must not fork semantics. They should carry operational warnings, forbidden stale claims, retrieval tags, and must-check surfaces.

## 4. Metadata model

SPEC-071's existing fields remain the baseline. Slice 2 should tighten their meaning rather than invent a separate system.

### 4.1 Verification baseline

`verified_against.git_commit` means the page was checked against that commit. It is not merely the creation commit.

Rules:

1. Do not update `last_verified` or `verified_against.git_commit` unless the declared evidence was rechecked.
2. Editing prose does not imply freshness.
3. A page can remain declared `current` even if `HEAD` has moved; the derived inspection state determines whether changed surfaces require inspection.
4. Optional release metadata may be added, but commit remains the strong anchor before Alpha release.

Recommended optional fields under `verified_against`:

```yaml
release_tag: null
ash_version: unreleased-alpha
```

These fields are advisory until Alpha tags exist.

### 4.2 Evidence paths

Evidence paths should be repo-relative and concrete enough for staleness inspection.

- `specs`: normative or canonical-adjacent specs backing the page.
- `tasks`: closeout/audit/task files backing current status.
- `code`: implementation paths that, if changed, may affect the page.
- `tests`: exact commands or test paths used as evidence.
- `examples`: reference examples or fixtures cited as executable/illustrative evidence.

### 4.3 Refresh triggers

Refresh triggers should be specific. Generic phrases such as "runtime changes" are too weak for a maintainable corpus.

Preferred triggers name paths, path globs, or precise semantic changes, for example:

```yaml
refresh_trigger:
  - crates/ashgrove/src/** changes
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - ashgrove install/update/remove/cleanup command behavior changes
```

The current validator accepts simple list entries. Slice 2 may extend tooling later, but the written convention should already support future automation.

## 5. Staleness inspection model

A staleness inspector, human or scripted, should:

1. read `verified_against.git_commit`;
2. run `git diff --name-only <commit>..HEAD`;
3. compare changed files against evidence paths and refresh triggers;
4. classify a derived state;
5. update content and metadata only after rechecking authority sources.

Derived states:

| State | Meaning | Stored in frontmatter? |
| --- | --- | --- |
| `no-relevant-changes` | No declared evidence or trigger surface changed. | no |
| `needs-inspection` | A relevant surface changed; content may still be correct. | no |
| `stale` | Inspection found a contradiction. | yes, as `status: stale` |
| `partial` | Inspection found current but incomplete or caveated behavior. | yes, as `status: partial` |
| `superseded` | A newer page replaces this one. | yes, as `status: superseded` |

This avoids turning every commit after verification into a false stale status.

## 6. Non-goals

Reference Slice 2 does not:

1. rewrite or move the historical `docs/` corpus;
2. create a complete Ash book;
3. add a hosted documentation service or wiki;
4. claim full practical deployment advice before real Alpha usage exists;
5. replace specs as normative contracts;
6. make version tags the primary freshness anchor before Alpha releases exist;
7. implement a full semantic staleness detector;
8. change Ash language/runtime behavior.

## 7. Design decisions

- D1: Use subsystem detail pages as durable authority surfaces.
- D2: Use reader-journey pages as thin entry paths with cross-links into subsystem detail.
- D3: Keep maintenance procedures centralized under `reference/maintenance/`; do not add page-specific playbooks to ordinary pages.
- D4: Treat `verified_against.git_commit` as the strongest pre-Alpha freshness anchor.
- D5: Treat `needs-inspection` as derived state from diff inspection, not as a normal page lifecycle status.
- D6: Keep optional version/release metadata advisory until Alpha tags exist.
- D7: Separate language concept pages from stdlib/API pages for `Act`, `Proc`, `Workflow`, and `Result`.
- D8: Require agent cards to remain derivative and cross-linked to canonical pages.

## 8. Changelog

### 2026-06-01

- Initial design note for Reference Slice 2, defining subsystem detail pages, reader-journey entry pages, runtime/toolchain/stdlib scope, and diff-based maintenance metadata strategy.
