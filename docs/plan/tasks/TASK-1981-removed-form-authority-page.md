# TASK-1981: Removed Form Authority Page

**Status:** Complete
**Phase:** [PLAN-201 Semantic Cleanup Follow-up](../PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md)
**Source audit:** [AUDIT-201 Semantic Removal Vs Rename](../audits/AUDIT-201-semantic-removal-vs-rename.md)

## Description

Add a Phase 201 authority page that lists removed historical forms and current target replacements
without source-shaped deprecated examples. Current docs and agent routing should have one safe page
to cite when deciding whether a term is historical, removed, or target-current.

## Requirements

- Add a `reference/status/` authority page for removed forms and target replacements.
- Do not include Ash code fences or source-shaped deprecated snippets on the page.
- Link the page from current reference/status and agent routing paths.
- Add Phase 201 gate coverage so the page remains prose-only.
- Update Phase 201 audit/task evidence and changelog.

## TDD Steps

1. Add failing/passing Phase 201 gate coverage for the authority page path and prose-only policy.
2. Add the removed-form authority page and route current indexes to it.
3. Run docs gates, the Phase 201 removal gate, link checks, and whitespace checks.

## Completion Checklist

- [x] Removed-form authority page exists under `reference/status/`.
- [x] The page lists removed forms and target replacements without deprecated source snippets.
- [x] Current status/agent indexes route to the page.
- [x] Phase 201 gate enforces the page's prose-only policy.
- [x] `CHANGELOG.md` and Phase 201 audit/task evidence are updated.

## Evidence

RED:

```bash
cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture
```

Failed after adding the page-existence assertion because `reference/status/removed-forms.md` did
not exist yet.

GREEN:

```bash
cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture
rg -n '^```|```ash|workflow::requires|workflow::ensures|observe .* with|act .* with|plays role|capabilities:|\bowns\b|\buses\b' \
  reference/status/removed-forms.md || true
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

The direct scan produced no code fences or source-shaped removed-form snippets on the authority
page.
