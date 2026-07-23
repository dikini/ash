# TASK-1980: Reference Tower Routing

**Status:** Complete
**Phase:** [PLAN-201 Semantic Cleanup Follow-up](../PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md)
**Source audit:** [AUDIT-201 Semantic Removal Vs Rename](../audits/AUDIT-201-semantic-removal-vs-rename.md)

## Description

Archive or relabel workflow/tower stdlib and language guidance so current agent-facing references
cannot be read as instructions to write removed tower-era Ash. Historical pages may remain for old
links, but productive read paths must route readers to target effects, provider profiles,
process/channel helpers, contracts/evidence, and application runtime reports.

## Requirements

- Retarget current function reference pages away from public tower guidance.
- Keep historical Act/Proc/Workflow/tower pages explicitly historical and outside productive
  source guidance.
- Add Phase 201 gate coverage for stale tower-routing claims in current reference pages.
- Update Phase 201 audit/task evidence and changelog.

## TDD Steps

1. Add failing Phase 201 gate rows for current-reference tower guidance.
2. Rewrite current function reference/card wording to target effect/process/application guidance.
3. Run the Phase 201 removal gate, docs orientation self-test, docs gate, and whitespace check.

## Completion Checklist

- [x] Current function reference pages do not direct readers to public tower APIs.
- [x] Agent function card does not promote Act/Proc/Workflow as current effect targets.
- [x] Phase 201 gate blocks stale tower-routing claims in current reference pages.
- [x] `CHANGELOG.md` and Phase 201 audit/task evidence are updated.

## Evidence

RED:

```bash
cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture
```

Failed after adding current-reference gate rows for stale phrases such as:

- `runtime-managed effect tower`;
- `live above pure code in the tower`;
- `explicit tower API`;
- `higher tower contexts`;
- `Act/Proc/Workflow closures`;
- `implicitly lift into Act/Proc/Workflow`;
- `reserved tower callable arrows`.

GREEN:

```bash
cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture
rg -n "runtime-managed effect tower|live above pure code in the tower|explicit tower API|higher tower contexts|Act/Proc/Workflow closures|implicitly lift into Act/Proc/Workflow|reserved tower callable arrows" \
  reference/language/functions.md reference/language/functions reference/agents/cards/functions.md || true
```

The final scan produced no matches in current function reference/card paths.

Final active-reader sweep:

```bash
cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate \
  productive_reference_docs_do_not_teach_removed_workflow_tower_model -- --exact
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```

The gate is now also backed by productive getting-started, runtime, CLI, and stdlib reader paths:
they describe checked function artifacts, effect rows, process helpers, and admitted application
instances rather than the removed public tower model.

The companion residual-read-path gate now covers the root README, current function pages and
cards, Result pages and cards, agent metadata, and CLI/test pages. It passed together with the
productive-reference gate, orientation-index self-test, docs gate, and `git diff --check` after
those paths were retargeted.

The final language-guide and metadata gates cover function bodies/calls, record destructuring,
and current reader metadata. They passed after source-shaped legacy forms were removed and current
pages were routed to target function/runtime authority instead of live tower specifications.

The final status/card/maintenance gate covers RuntimeKernel status, current cards, methodology,
and limitation/drift metadata. It passed after those paths adopted checked-function artifact,
application, target-authority, and non-bounded-law terminology.
