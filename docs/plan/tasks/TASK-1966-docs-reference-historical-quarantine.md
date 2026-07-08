# TASK-1966: Docs/Reference Historical Quarantine

**Status:** Complete
**Phase:** [PLAN-201: Deprecated Functionality Removal](../PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md)

## Description

Reconcile docs, references, spec indexes, and notes after deprecated functionality is removed from
current Ash.

## Requirements

- Productive docs, tutorials, templates, and examples must contain no deprecated functionality.
- Historical/reference docs may mention removed forms only with explicit prose labels and without
  Ash code blocks or snippets.
- SPEC-INDEX and NOTE-INDEX must route current work away from removed functionality.
- Migration docs must describe removed behavior without implying support.

## TDD Steps

1. Add or update docs gates that fail on deprecated Ash snippets in productive docs and unlabeled
   historical prose.
2. Update docs/reference/spec/note wording according to AUDIT-201.
3. Run docs gates and orientation index validation.
4. Record exact verification in this task file.

## Completion Checklist

- [x] Productive docs and tutorials are target-only.
- [x] Historical/reference docs are explicitly labeled prose with no deprecated Ash code snippets.
- [x] SPEC-INDEX and NOTE-INDEX route current work to target Ash.
- [x] Migration guidance says removed, not deprecated-but-supported.
- [x] Docs and orientation gates pass.

## Evidence

- Rewrote `docs/TUTORIAL.md` examples from removed workflow declarations to target `fn main`
  entries matching the checked example files.
- Replaced the stale `docs/API.md` API sample page, which constructed removed workflow carriers
  directly, with current crate/API orientation and a removed-form boundary statement.
- Quarantined `docs/book/appendix-b.md` by removing source-shaped provider examples that used
  removed workflow and observe-with forms, leaving historical prose and links to current
  productive examples/templates/plans.
- Refreshed `docs/README.md` to route readers through current productive docs, examples, templates,
  and orientation indexes.
- Updated `docs/book/appendix-c.md` to stop presenting deleted workflow example trees as current
  runnable examples.
- Replaced `docs/book/appendix-a.md` with a current example inventory that lists only checked
  target examples instead of the deleted workflow-era catalog.
- Replaced the stale `docs/book/appendix-c.md` file tree and run commands with current productive
  docs/example roots and target `ash check` commands.
- Retargeted `docs/reference/core-ash-text-format.md` examples from removed Core aliases
  (`cap`/`proc`) to canonical `operation`/`process` spelling.
- Quarantined stale `reference/stdlib` tower pages, derivative language pages, example
  classification, status/common-confusion notes, and stdlib agent cards so they no longer claim
  deleted tower stdlib files or old phase examples as current evidence.
- Removed source-shaped historical tower carrier snippets from the focused reference paths.
- Superseded `AUDIT-199-current-syntax-library-template-inventory.md` because it still classified
  deleted example and removed stdlib tower files as current executable Phase 199 assets.
- Removed deleted evidence paths from Result/runtime/test reference metadata, including
  `tests/std/result.ash`, the removed daemon child-failure test, and command strings in
  `verified_against.tests`.
- Repaired CPS/IR reference metadata and links after the Phase 201 quarantine: added validator
  frontmatter to IR leaf pages, retargeted interpreter evidence from `crates/ash-interp/src/cps.rs`
  to `crates/ash-interp/src/cps/mod.rs`, removed broken current links to tower-era reference
  pages, and corrected spec links.
- Retargeted remaining docs/reference source-shaped examples by converting algebra/test snippets
  from removed workflow/`ret` forms to target `pub fn main`, removing the legacy proof spelling
  from Ash code blocks, and reducing the historical Phase 101 capability/resource parser
  substrate page to prose-only removed-form history.
- Retargeted target-grammar and WorkflowForm-era spec/note routing after Phase 201 removal:
  `SPEC-095b` now describes old workflow/act/tower source families as removed historical forms
  rather than accepted compatibility aliases, its source-shaped workflow declaration example was
  removed, `SPEC-INDEX` and `NOTE-INDEX` route PLAN-196 through the Phase 201 removed-form
  boundary, and SPEC-056/NOTE-010 warning/translation prose now describes historical migration
  behavior rather than current support.
- Retargeted productive book index/appendix labels away from OODA compatibility wording to target
  effects and policy terminology.
- Replaced the stale `docs/book/SUMMARY.md` linked chapter map with a current orientation summary
  because the old map pointed at removed chapter files and workflow-era topics as current book
  structure.
- Extended the Phase 201 removal gate to scan productive docs/book/tutorial paths:
  `docs/API.md`, `docs/README.md`, `docs/TUTORIAL.md`, `docs/book`, and `docs/tutorials`.
- Focused verification:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`;
  focused reference sweep for deleted tower stdlib/example paths and source-shaped carrier
  spellings;
  `python3 tools/reference/check_frontmatter.py`;
  `python3 tools/reference/check_frontmatter.py --pilot`;
  `cargo fmt --all --check`;
  `python3 tools/docs/validate_orientation_indexes.py --self-test`;
  `bash scripts/check-docs-gate.sh`;
  `git diff --check`.
- Focused verification after the target-grammar/spec-index quarantine slice:
  `rg -n --glob '!docs/plan/**' --glob '!reference/**' --glob '!CHANGELOG.md' 'compatibility-only.*legacy `workflow`|legacy workflow declaration|workflow keyword form is accepted|accepted as a compatibility alias|deprecated legacy workflow|deprecated-but-supported|legacy workflow.*accepted|current workflow declaration surface remains accepted|compatibility translation for legacy workflow declarations|compatibility with the legacy workflow declaration' docs/spec docs/notes docs/README.md docs/API.md docs/TUTORIAL.md docs/book`
  is silent;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`;
  `python3 tools/docs/validate_orientation_indexes.py --self-test`;
  `bash scripts/check-docs-gate.sh`.
- Retargeted residual spec/note migration wording so old callable, act/tower, capability, and
  workflow forms are described as removed/historical rather than compatibility syntax across
  SPEC-027, SPEC-031, SPEC-047, SPEC-052, SPEC-054, SPEC-056, SPEC-063, SPEC-072, SPEC-095b,
  SPEC-096b, SPEC-097, SPEC-097b, SPEC-098c, NOTE-010, NOTE-019, NOTE-035, and
  `docs/spec/README.md`.
- Rewrote remaining executable `.ash` fixtures that used `workflow main ... { ret ... }` or
  `workflow main { done }` to target `fn main` entries in the Phase 145-148 fixture corpus and
  the spec-processor mock repo.
- Retargeted the root `README.md` away from deleted workflow-era examples and removed
  source-shaped examples from `docs/SHARO_CORE_LANGUAGE.md`, leaving historical scenario prose
  only. Also removed stale compatibility wording from `crates/ash-lint/README.md` and
  `std/src/test/quickcheck/mod.ash`.
- Focused verification after the residual spec/note and executable `.ash` fixture slice:
  `rg -n --glob '!docs/plan/**' --glob '!reference/**' --glob '!CHANGELOG.md' 'For migration compatibility only|compatibility window|compatibility implementation|Initial compatibility|remains accepted.*compatibility|legacy migration lowering boundary|legacy `Fn|Legacy `Fn|legacy SPEC-047 statement grammar remains accepted|migration compatibility spelling|compatibility spelling|compatibility syntax|Fn\(Int, Int\) -> Int|Fn\(Int, String\) -> Bool|Fn\(<param_types>\)|Fn\(<params>\)' docs/spec docs/notes docs/README.md docs/API.md docs/TUTORIAL.md docs/book`
  is silent;
  `rg -n --glob '!docs/plan/**' --glob '!reference/**' --glob '!CHANGELOG.md' 'deprecated compatibility|deprecated legacy|deprecated development|deprecated forms|accepted temporarily|accepted as .*compatibility|only for compatibility|Preserve legacy compatibility|current legacy capability|must preserve compatibility with existing' docs/spec docs/notes docs/README.md docs/API.md docs/TUTORIAL.md docs/book`
  is silent;
  `rg -n -P -g '*.ash' '^(?!\s*--)(?=.*\b(workflow|act|observe|orient|propose|decide|ret|do:(Act|Proc|Workflow)|pub capability)\b|.*\bFn\s*\().*' .`
  is silent.
- Verification for the same slice:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`;
  `python3 tools/docs/validate_orientation_indexes.py --self-test`;
  `bash scripts/check-docs-gate.sh`;
  `cargo fmt --all --check`;
  `git diff --check`;
  `cargo test -p ash-cli --test phase147_coverage_mutation -- --nocapture`;
  `cargo test -p ash-cli --test phase148_flake_orchestration -- --nocapture`;
  `cargo test -p spec_processor --test collect_tests --test example_check_tests -- --nocapture`.
- Verification after expanding the docs coverage:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`;
  `bash scripts/check-docs-gate.sh`;
  `git diff --check`.
