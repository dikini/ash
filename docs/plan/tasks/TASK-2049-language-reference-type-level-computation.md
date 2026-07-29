# TASK-2049: Language Reference for Type-Level Computation and Propositions

**Status:** Planned
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Depends on:** TASK-2045
**Owned feature IDs:** LANG-010.

## Description

Document the implemented/partial source surface for sealed domains, type functions, associated
families, promoted data kinds, holes, normalization-facing diagnostics, and propositions.

## Requirements

- Create `docs/reference/language/types/type-level-domains-functions-families-and-propositions.md`.
- Keep source grammar, TypeEnv/summary transport, normalization, and runtime execution status
  separate. Most material is static semantics, not executable program semantics.
- Use sequents for actual checker/normalizer judgments only when an exact implemented rule is
  evidenced; record deferred or non-inverting behaviour as a limitation.
- Explicitly exclude `dtype`; explain only `type`, `newtype`, and `data kind` where accepted.

## Handoffs and dependencies

- **Consumes:** parser type-level branches, `ash-typeck` normalizer/diagnostics, and summary
  transport code.
- **Evidence:** `cargo test -p ash-parser --test task_813_sealed_domain_diagnostics`, `--test
  task_846_public_type_fn_visibility`, `--test task_881_proposition_parse_diagnostics`; `cargo
  test -p ash-typeck --test task_827_normalizer_diagnostics`, `--test
  task_868_associated_family_diagnostics`.
- **Produces:** a type-level terminology boundary for TASK-2053 stdlib documentation.
- **Non-goals:** `dtype`, unrestricted proof search/SMT claims, a runtime evaluator for type
  functions, or semantic rules inferred solely from target specs.

## TDD and verification steps

1. Enumerate each public spelling and required negative diagnostic before prose.
2. Verify the listed parser/typeck tests and mark any untested target clause planned or partial.
3. Render all EBNF and checked-sequent fences with the external tools.

## Completion checklist

- [ ] Every form has parser/static/summary/runtime status and exact evidence.
- [ ] `dtype` and other absent forms are excluded, not invented.
- [ ] Normalization/proposition limits are explicit.
- [ ] Removed forms never appear as current examples; indexes/changelog/PLAN-INDEX are updated.
