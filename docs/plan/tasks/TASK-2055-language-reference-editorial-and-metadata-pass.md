# TASK-2055: Language Reference Editorial and Metadata Pass

**Status:** Complete
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Depends on:** TASK-2054

**Semantic task classification:** non-semantic-workflow-enforcement

## Description

Improve the implementation-backed manual under `docs/reference/language/` after its initial closeout. This is an editorial and discoverability pass: remove repetitive defensive prose, make the reader's path clearer, and add manual-specific metadata without changing language claims or the top-level `reference/` corpus policy.

## Requirements

- Add YAML frontmatter to every manual page. It must identify the page, title, kind, status, intended audience, reviewed implementation revision, evidence class, and refresh triggers.
- Add the metadata schema and its maintenance rule to `conventions.md`; it must remain distinct from the top-level `reference/` schema.
- Rewrite pages for direct, reader-oriented explanations. State a boundary once near the relevant claim; do not repeat it as boilerplate after each paragraph or example.
- Restructure each feature page so a reader can find the current support level, what the form looks like, its important limitations, and supporting evidence without reading an implementation census.
- Preserve the implementation/evidence/parity distinction and every material limitation. Do not widen a grammar, lowering, admission, runtime, or parity claim.
- Preserve working links, validated EBNF/sequent fences, and existing evidence paths.

## Completion checklist

- [x] Every page under `docs/reference/language/` has valid manual metadata.
- [x] `conventions.md` defines the manual metadata fields and refresh process.
- [x] Rewritten prose leads with the rule, uses shorter active sentences, and removes repeated disclaimer language from the manual index and the highest-density feature introductions.
- [x] Every changed claim remains grounded in the existing source/tests; this pass changes wording and organization, not support status.
- [x] The manual fence tests/validator, documentation gate, orientation-index self-test, and `git diff --check` pass.
- [x] `CHANGELOG.md` and PLAN-206 describe this editorial follow-up.

## Verification

- `python3` metadata check: 27/27 manual pages have all required metadata fields.
- `node --test tools/docs/validate_language_reference_fences.test.mjs`: 23/23 tests passed.
- `node tools/docs/validate_language_reference_fences.mjs --root docs/reference/language ...`:
  validated all 30 fences.
- `python3 tools/docs/validate_orientation_indexes.py --self-test`: passed.
- `bash scripts/check-docs-gate.sh`: passed; 929 changed-Markdown links checked, none missing.
- `git diff --check`: passed.
- Review remediation: corrected the AUDIT-206 census anchor, clarified that terminal envelopes
  cover successful issuing-Engine dispatch only, and attributed exit-code projection to bootstrap
  execution rather than entry verification. Corrected the associated-family projection spelling
  and listed the library chapter in the status page's coverage summary. Completed the declaration
  EBNF visibility syntax to match the parser and module chapter.
- `cargo test -p ash-engine --test entry_verification`: 21/21 passed.
- `cargo test -p ash-engine --test task_2032_shared_engine_execution_seam
  admitted_program_and_request_reject_a_foreign_engine_before_dispatch`: passed.
