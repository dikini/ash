# TASK-1707: Agent Usability Evaluation

**Status:** ✅ Complete
**Phase:** [PLAN-166](../PLAN-166-DOCS-ORIENTATION-INDEXES.md)
**Owner:** Phase 166

## Description

Run independent subagent before/after usability evaluations and record metrics.

## Specification Reference

- [PLAN-166](../PLAN-166-DOCS-ORIENTATION-INDEXES.md)
- [NOTE-INDEX](../../notes/NOTE-INDEX.md)
- [SPEC-INDEX](../../spec/SPEC-INDEX.md)

## Dependencies

- ✅ TASK-1693: Contract implementation handoff packet

## Requirements

1. Preserve the distinction between structured topic ontology and unstructured retrieval tags.
2. Keep indexes navigational rather than normative.
3. Keep links valid and status/role metadata explicit.
4. Update CHANGELOG and PLAN-INDEX for docs-policy/tooling changes.

## Verification

```text
strictness: clean
commands:
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - git diff --check
checklist:
  - [x] Task deliverable exists.
  - [x] Orientation index validator passes.
  - [x] Docs gate passes.
```

## Completion Notes

Completed in Phase 166 closeout.

## Independent Evaluation Evidence

The controller ran independent read-only Codex subagents with no current-chat context. The
baseline prompt explicitly forbade use of `docs/notes/NOTE-INDEX.md` and
`docs/spec/SPEC-INDEX.md`; the post-index prompts allowed index use.

### Baseline: no orientation indexes

Command:

```bash
codex exec -C /home/dikini/Projects/ash -s read-only --ephemeral \
  -o /tmp/ash_baseline_eval.md "$(cat /tmp/ash_baseline_eval_prompt.txt)"
```

Summary reported by the subagent:

- Commands used: 13 repo commands.
- Searches used: 1 file inventory, 5 broad/targeted corpus searches, and 5 per-file focused scans.
- Files read or sampled: 21.
- Approximate content read: 95k-115k characters, roughly 24k-29k tokens, excluding noisy search output.
- Full selected authority corpus if read end-to-end: 430,836 characters, roughly 105k tokens.
- Finding: discovery was workable but brittle. Exact phrases or task numbers found good paths;
  generic terms such as `contract`, `predicate`, and `sidecar` produced noisy results.

Important baseline confusion:

- `SPEC-099` naming is overloaded across Core language, current operational semantics, target CPS
  operational semantics, and expanded CPS operational semantics.
- Handler/effect syntax spans target grammar, effect rows, handler dispatch, effect identity, and
  TASK-1692; there was no single obvious route.

### First post-index evaluation

Command:

```bash
codex exec -C /home/dikini/Projects/ash -s read-only --ephemeral \
  -o /tmp/ash_postindex_eval.md "$(cat /tmp/ash_postindex_eval_prompt.txt)"
```

Summary reported by the subagent:

- Commands used: 14 shell invocations.
- Searches used: 3 `rg` searches.
- Files opened or directly inspected: approximately 25.
- Approximate direct text read: 120k characters, roughly 30k tokens.
- Finding: indexes substantially reduced discovery cost for contract and current-vs-target planning
  work, but target handler/effect syntax still lacked a direct read path.

Follow-up applied from this evaluation:

- Added `Change target handler/effect/operation syntax` read paths to both `NOTE-INDEX.md` and
  `SPEC-INDEX.md`.

### Second post-index evaluation after read-path repair

Command:

```bash
codex exec -C /home/dikini/Projects/ash -s read-only --ephemeral \
  -o /tmp/ash_postindex_eval2.md "$(cat /tmp/ash_postindex_eval2_prompt.txt)"
```

Summary reported by the subagent:

- Commands/searches: 7 shell commands total, 3 `rg` searches.
- Project docs opened: `docs/notes/NOTE-INDEX.md` and `docs/spec/SPEC-INDEX.md`.
- Approximate project index text read: 35,113 characters, about 8.8k tokens.
- Confidence: high for Core contract predicate artifacts, target handler/effect/operation syntax,
  and temporal/trace monitor sidecars; medium-high for current-vs-target Core/type/effect planning.
- Index coverage: all four evaluation tasks covered after the handler/effect syntax read path was
  added. The subagent requested one more convenience path for current-vs-target planning, which was
  added to `SPEC-INDEX.md`.

Net result:

- Baseline required broader corpus search and 95k-115k characters of direct reading.
- Final indexed workflow found the needed doc sets from two index files, about 35k characters, with
  only one noisy broad search. That is roughly a 63%-69% reduction in direct repository text read
  for orientation before opening the selected authority docs.
