# TASK-1961: Deprecated Functionality Dependency Audit

**Status:** Complete
**Phase:** [PLAN-201: Deprecated Functionality Removal](../PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md)

## Description

Audit every remaining deprecated functionality occurrence and classify the owner, replacement, and
removal risk before implementation removes behavior.

## Requirements

- Create `docs/plan/audits/AUDIT-201-deprecated-functionality-removal.md`.
- Inventory parser/checker accepted deprecated syntax.
- Inventory AST, lowering, Core/CPS, type/effect, runtime, report, trace, formatter, LSP, CLI,
  template, example, fixture, stdlib, and docs occurrences.
- Classify each occurrence as remove, rename, or historical-prose-only.
- Assign TASK-1962 through TASK-1967 owners for each occurrence.

## TDD Steps

1. Add a failing audit inventory gate that detects deprecated Ash forms in code, fixtures,
   templates, examples, snapshots, Rust source string literals, or active paths.
2. Create AUDIT-201 with initial classifications and owner tasks.
3. Run the audit gate and fix classifications until it passes.
4. Record exact focused commands and results in this task file.

## Completion Checklist

- [x] AUDIT-201 exists and is indexed from the phase plan.
- [x] Every deprecated occurrence has an owner, outcome, replacement target, tests, and risk.
- [x] Audit gate fails on deprecated Ash forms in repository code or unclassified active
      deprecated functionality.
- [x] Historical/reference-only material is explicitly labeled prose and contains no deprecated Ash
      code snippets.
- [x] Focused and docs gates pass.

## Evidence

- Added `docs/plan/audits/AUDIT-201-deprecated-functionality-removal.md`.
- Added the initial repository-wide gate:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate`.
- RED evidence: `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate
  -- --nocapture` fails and reports remaining deprecated Ash form hits across old examples,
  stdlib surfaces, parser/checker/tooling fixtures, and Rust source string literals.
- First cleanup slice: Phase 199 productive helper examples and app templates were rewritten from
  `workflow main` to target `fn main() -> ... { do { return ...; } }` entry syntax.
- Focused non-regression passed after that rewrite:
  `cargo test -p ash-cli --test phase199_template_manifest --test phase199_template_instantiation_cli -- --nocapture`
  and
  `cargo test -p ash-cli --test phase200_examples_current_syntax --test phase200_old_syntax_demoted -- --nocapture`.
- Phase 201 cleanup slice removed historical example `.ash` trees, deleted compatibility-only
  `tests/std` and `tests/workflows` Ash fixtures, removed `std/src/act.ash`, `std/src/proc.ash`,
  and `std/src/workflow.ash`, and stripped `pub capability` declarations from target stdlib
  provider surfaces while preserving runtime-backed `pub builtin fn` APIs.
- Later Phase 201 cleanup removed the active `std/src/ooda.ash` compatibility helper module,
  deleted the OODA demotion test that asserted compatibility behavior, and tightened the removal
  gate against reintroducing the stdlib OODA module/export or ash-lint OODA aliases.
- The active stdlib corpus baseline now has no expected-fail rows after stale module-root and LLM
  helper expected failures were promoted to target-checkable files:
  `cargo test -p ash-cli --test stdlib_corpus_check -- --nocapture`
  reports `files=59, pass=59, fail=0, reference_only=0`.
- The Phase 201 Ash-artifact gate now passes for `std`, `examples`, `templates`, and remaining
  Ash fixtures:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`.
- Final closeout verification superseded the earlier open audit scope: parser/checker acceptance,
  Rust embedded fixtures, historical docs, and internal carrier/vocabulary rows were resolved by
  TASK-1962 through TASK-1968, while retained target-justified mechanisms and follow-up refactors
  were classified by TASK-1969/TASK-1970.
