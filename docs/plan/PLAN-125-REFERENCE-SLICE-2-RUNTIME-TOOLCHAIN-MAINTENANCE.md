# PLAN-125: Reference Slice 2 Runtime, Toolchain, and Maintenance Manual

> **For Hermes:** This is a documentation and tooling phase. Use subagent-driven-development for page groups and independent reviews. Do not change Ash runtime semantics. Do not turn reader-journey pages into subsystem policy copies. TASK-993 is the hard maintenance-model gate before bulk page authoring.

**Goal:** Expand the current barebones `reference/` corpus into a maintainable Alpha manual covering basic reader journeys, stdlib tower surfaces, `ash`/`ashgrove` procedures, RuntimeKernel status, and reference maintenance/staleness procedures.

**Architecture:** Keep subsystem pages as durable detail surfaces and reader-journey pages as thin cross-linked entry paths. Put maintenance semantics in frontmatter plus `reference/maintenance/`, not in every ordinary page body. Use `verified_against.git_commit` as the pre-Alpha freshness anchor and derive `needs-inspection` from diffs against evidence paths and refresh triggers.

**Tech Stack:** Markdown reference pages with SPEC-071 frontmatter; repo-local Python reference validator/checker; scoped Markdown link checks; existing specs/tasks/code/tests as evidence; no Rust runtime changes unless validator tooling requires a tiny docs-tool patch.

---

## 1. Status

**Status:** 🚧 In Progress; TASK-992 through TASK-995 complete
**Spec:** [SPEC-075](../spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
**Design:** [DESIGN-043](../design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md)
**Task range:** [TASK-992](tasks/TASK-992-reference-slice-2-packet.md) through [TASK-999](tasks/TASK-999-reference-slice-2-closeout.md)

TASK-992 created the design/spec/plan/task packet and registered Phase 130. TASK-993 defined the metadata/staleness maintenance model. TASK-994 added the thin getting-started reader journey and draft link targets for later subsystem pages. TASK-995 expanded the Ash CLI/Ashgrove command and procedure reference pages.

## 2. Scope

### In scope

- `reference/getting-started/` basics: what Ash is, install, update, run, daemon, cleanup, next steps.
- `reference/stdlib/` pages for `Act`, `Proc`, `Workflow`, and `Result` public library surfaces.
- `reference/tools/` pages for `ash`, `ashgrove`, install/update/list/current/default/remove/cleanup/dependencies/vendor/trust/source payload procedures.
- `reference/runtime/` pages for RuntimeKernel, admission, artifacts, daemon, and policy profiles.
- `reference/status/` pages for runtime, Ashgrove, Alpha limitations, maintenance coverage, drift, feature matrix, and verification evidence.
- `reference/maintenance/` pages for metadata, staleness inspection, refresh, stale-doc triage, release checklist, and agent-card maintenance.
- Agent cards and context-pack updates for the new major pages.
- Validator/checker hardening needed to support Slice 2 metadata and closeout evidence.

### Out of scope

- Moving or rewriting the historical `docs/` corpus.
- Full tutorial book, hosted docs service, or dynamic wiki.
- Advanced deployment/risk advice beyond current evidence.
- Runtime, parser, typechecker, Ashgrove behavior changes.
- Full semantic staleness detection.
- Treating version metadata as stronger than commit metadata before Alpha tags exist.

## 3. Task table

| Task | Description | Est. Hours | Status |
| --- | --- | ---: | --- |
| [TASK-992](tasks/TASK-992-reference-slice-2-packet.md) | Create DESIGN-043/SPEC-075/PLAN-125/TASK packet and register Phase 130 | 5 | ✅ Complete |
| [TASK-993](tasks/TASK-993-reference-maintenance-metadata-and-staleness.md) | Create the maintenance metadata/staleness procedure substrate before bulk pages | 10 | ✅ Complete |
| [TASK-994](tasks/TASK-994-reference-getting-started-journey.md) | Add basic reader-journey pages for what Ash is, install, update, run, daemon, cleanup, and next steps | 10 | ✅ Complete |
| [TASK-995](tasks/TASK-995-reference-ashgrove-and-cli-procedures.md) | Add `ash`/`ashgrove` toolchain and procedure reference pages | 14 | ✅ Complete |
| [TASK-996](tasks/TASK-996-reference-runtime-kernel-pages.md) | Add RuntimeKernel concept and status pages | 12 | 📝 Planned |
| [TASK-997](tasks/TASK-997-reference-stdlib-tower-pages.md) | Add stdlib tower reference pages for Act, Proc, Workflow, and Result | 12 | 📝 Planned |
| [TASK-998](tasks/TASK-998-reference-agent-cards-and-context-pack.md) | Add/update agent cards, context-pack index, and common-confusion warnings | 8 | 📝 Planned |
| [TASK-999](tasks/TASK-999-reference-slice-2-closeout.md) | Run validator/checker hardening, drift/feature/status reconciliation, broad docs checks, and independent review | 8 | 📝 Planned |

Total estimate: 79 hours.

## 4. Execution order

The phase is intentionally ordered:

1. TASK-992 creates and registers the packet.
2. TASK-993 defines maintenance metadata and staleness inspection before page authors write against the model.
3. TASK-994 creates the reader journey with links into planned subsystem pages.
4. TASK-995 through TASK-997 create the subsystem detail pages.
5. TASK-998 creates derivative agent surfaces after canonical pages exist.
6. TASK-999 validates, reconciles, and closes the slice.

TASK-995, TASK-996, and TASK-997 may run in parallel only after TASK-993 completes and after TASK-994 has established stable journey link targets or placeholders.

## 5. Decision gates

- D1: Subsystem pages are the durable detail/reference surfaces; reader journeys are cross-linked entry paths.
- D2: Maintenance procedures live under `reference/maintenance/`; ordinary pages carry metadata and user-facing limitations, not page-specific maintenance playbooks.
- D3: `verified_against.git_commit` is the strong pre-Alpha freshness anchor. Optional version/release fields are advisory until Alpha tags exist.
- D4: `needs-inspection` is a derived state computed from diffs against evidence paths and refresh triggers, not a new frontmatter lifecycle status in this phase.
- D5: The phase must not duplicate full subsystem policy into reader-journey pages.
- D6: `reference/language/` concept pages and `reference/stdlib/` API pages remain distinct and cross-linked.
- D7: Ashgrove pages must preserve SPEC-073/SPEC-074 non-goals and fail-closed trust/source-payload boundaries.
- D8: RuntimeKernel pages must preserve SPEC-070 authority/integrity boundaries: file presence is not execution, provider existence is not authority, and reload affects future starts rather than mutating admitted running instances.
- D9: TASK-993 is a hard gate; bulk page tasks must not start until maintenance metadata/staleness semantics are documented.

## 6. Page inventory

Minimum expected page inventory is the union of these groups:

```text
reference/getting-started/README.md
reference/getting-started/what-is-ash.md
reference/getting-started/install.md
reference/getting-started/update.md
reference/getting-started/run-a-program.md
reference/getting-started/run-as-daemon.md
reference/getting-started/cleanup.md
reference/getting-started/next-steps.md
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
reference/status/runtime-kernel.md
reference/status/ashgrove.md
reference/status/reference-maintenance.md
reference/status/alpha-limitations.md
reference/status/drift-report.md
reference/status/verification-evidence.md
reference/status/feature-matrix.md
reference/maintenance/README.md
reference/maintenance/metadata-reference.md
reference/maintenance/staleness-inspection.md
reference/maintenance/refresh-procedure.md
reference/maintenance/stale-doc-triage.md
reference/maintenance/release-checklist.md
reference/maintenance/agent-card-procedure.md
reference/agents/cards/stdlib-act.md
reference/agents/cards/stdlib-proc.md
reference/agents/cards/stdlib-workflow.md
reference/agents/cards/stdlib-result.md
reference/agents/cards/ash-cli.md
reference/agents/cards/ashgrove.md
reference/agents/cards/runtime-kernel.md
```

The implementation may add helper pages, but TASK-999 must account for every minimum page or record an explicit scope change.

## 7. Verification strategy

Docs packet verification for TASK-992:

```bash
git diff --check
python3 -m py_compile tools/reference/check_frontmatter.py
python3 tools/reference/check_frontmatter.py --pilot
python3 - <<'PY'
from pathlib import Path
files = [
    'docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md',
    'docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md',
    'docs/plan/PLAN-125-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md',
] + [f'docs/plan/tasks/TASK-{n}-{slug}.md' for n, slug in [
    (992, 'reference-slice-2-packet'),
    (993, 'reference-maintenance-metadata-and-staleness'),
    (994, 'reference-getting-started-journey'),
    (995, 'reference-ashgrove-and-cli-procedures'),
    (996, 'reference-runtime-kernel-pages'),
    (997, 'reference-stdlib-tower-pages'),
    (998, 'reference-agent-cards-and-context-pack'),
    (999, 'reference-slice-2-closeout'),
]]
missing = [p for p in files if not Path(p).exists()]
assert not missing, missing
idx = Path('docs/plan/PLAN-INDEX.md').read_text()
assert '## Phase 130: Reference Slice 2 Runtime, Toolchain, and Maintenance Manual' in idx
for n in range(992, 1000):
    assert f'TASK-{n}' in idx
print('phase130 packet structure verified')
PY
```

TASK-999 closeout verification must additionally run:

```bash
python3 tools/reference/check_frontmatter.py
python3 tools/reference/check_frontmatter.py --pilot
python3 tools/reference/check_staleness.py --slice reference-slice-2
python3 -m py_compile tools/reference/check_frontmatter.py tools/reference/check_staleness.py
git diff --check
cargo fmt --all --check
```

If the staleness checker is intentionally not implemented, TASK-999 must replace that command with a documented human/agent staleness-inspection audit over every Slice 2 page and must record why automation remains deferred.

## 8. Completion criteria

The phase is complete only when:

- TASK-993 maintenance metadata/staleness procedures exist and are referenced by the relevant indexes/status pages;
- every minimum Slice 2 page exists or has an explicit accepted scope-change note;
- every expanded page has SPEC-071 frontmatter with a non-`unknown` verification commit after closeout;
- reader-journey pages cross-link to subsystem details rather than duplicating policy;
- Ashgrove pages preserve SPEC-073/SPEC-074 non-goals;
- RuntimeKernel pages preserve authority/integrity caveats;
- agent cards link back to canonical pages and do not fork semantics;
- validator/checker evidence covers the new surfaces;
- drift report, verification evidence, feature matrix, status index, and CHANGELOG are updated;
- independent review finds no blocking overclaim, stale-link, metadata, or maintenance-procedure issues.

## 9. Changelog

### 2026-06-01

- TASK-993 added the Reference Slice 2 maintenance pages, reference-maintenance status surface, cross-links, and stdlib-only path-based staleness checker while keeping later Slice 2 page tasks planned.
- TASK-994 added the getting-started reader journey, surfaced it from the reference root/index, and created draft toolchain/runtime detail targets for later subsystem expansion.
- TASK-995 replaced the TASK-994 Ashgrove/CLI draft placeholders with command-map and procedure pages covering install, update, selectors, remove/cleanup, project dependencies, vendor/deploy, trust/signing, source-payload policy, and Ashgrove status while preserving SPEC-073/SPEC-074 non-goals and fail-closed boundaries.
- TASK-992 created the Phase 130 Reference Slice 2 planning packet. The packet defines subsystem detail pages, reader-journey basics, maintenance metadata/staleness procedures, Ashgrove/RuntimeKernel/stdlib coverage, agent-card updates, and closeout validation requirements.
