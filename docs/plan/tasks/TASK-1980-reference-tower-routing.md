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
