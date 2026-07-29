# TASK-2046: Language Reference for Lexical Structure, Modules, Notation, and Macros

**Status:** Planned
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Depends on:** TASK-2045
**Owned feature IDs:** LANG-001, LANG-002, LANG-003, LANG-024.

## Description

Document the implementation-backed source-file/module/import/visibility surface and the distinct
notation/macro syntax-phase surfaces.

## Requirements

- Create `docs/reference/language/lexical-and-modules/index.md`,
  `source-files-names-and-literals.md`, `modules-imports-and-visibility.md`, and
  `notation-and-expression-macros.md`.
- Derive grammar from `parse_module::module_file`, `parse_expr`, lexer, and parser tests; state
  module-summary/import routes independently from execution.
- Document notation and macros as syntax/summary mechanisms, including hygiene and lowering
  boundaries. Do not infer runtime authority/callability from a macro summary.
- Document parser-recognized operator sections and their mandatory elaboration-before-lowering
  diagnostic boundary; cross-link forms pages without transferring ownership.
- Give every supported form EBNF and every formal claim a code/test source. Use sequents only for
  actual implemented checking/transition rules.

## Handoffs and dependencies

- **Consumes:** TASK-2045 conventions; `crates/ash-parser/src/{parse_module.rs,parse_expr.rs,surface.rs}`;
  `crates/ash-engine/src/module_loader/**`.
- **Evidence:** `cargo test -p ash-parser --test task_1732_local_notation_table_resolution`,
  `--test task_1754_macro_declaration_parse`, `--test task_1758_macro_lowering_boundaries`, and
  `--test task_1724_operator_section_boundary`, `--test task_1733_operator_section_elaboration`,
  and `cargo test -p ash-engine --test module_import_resolution_tests`.
- **Produces:** links used by forms/types/effects/stdlib pages.
- **Non-goals:** old workflow module syntax, capability/policy source declarations, or a claim that
  successful import proves general Engine execution.

## TDD and verification steps

1. Build a form-to-parser/test table before prose; leave a row failing/incomplete if no route is
   found.
2. Write syntax examples and verify parse/check classification before marking them executable.
3. Validate representative EBNF fences with the railroad project and run named tests.

## Completion checklist

- [ ] All four pages are indexed and every example is classified.
- [ ] Import/visibility, macro expansion, and runtime boundaries are explicit.
- [ ] Removed forms never appear as current examples.
- [ ] Links, changelog, and PLAN-INDEX updates are complete.
