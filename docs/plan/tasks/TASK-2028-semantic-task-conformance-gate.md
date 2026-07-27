# TASK-2028: Semantic Task Conformance Gate

**Status:** Complete — bounded active TASK-1988 follow-up records, cross-document validation,
and targeted pre-commit/pre-push verification enforcement are in place. This enforces delivery
evidence only; it adds no general language execution semantics.
**Semantic task classification:** non-semantic-workflow-enforcement
**Phase:** Follow-up from TASK-2027 and the TASK-1988 implementation follow-ups

## Description

Mechanically enforce the semantic-rule-first delivery chain for semantic implementation work. A
semantic task must declare its canonical rules, bounded or general domain, layer coverage,
positive/negative/mutation/parity evidence, explicit non-goals, next obligation, and targeted
verification commands. Local gates must reject missing or inconsistent evidence and run the
affected task's integration checks.

## Requirements

- Use a checked-in structured manifest, not heuristic parsing of Markdown prose.
- Validate task, coverage-map, traceability, and manifest links.
- Require bounded labelling to agree across task, manifest, coverage map, and traceability.
- For touched semantic implementation paths, require matching task, manifest, coverage-map,
  traceability, and changelog changes.
- Run declared focused verification commands in pre-commit and all active semantic-task commands
  in pre-push.
- Reject unsafe or unrecognized verification commands.
- Repair stale TASK-2004 rejection evidence so it tests an actually unsupported source form.

## Semantic workflow record

- **Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, and the linked target
  rule for each migrated task.
- **Declared domain:** bounded active TASK-1988 semantic follow-ups and their local gate paths.
- **Layers:** planning/evidence enforcement spans Type, Core, CPS, admission/runtime, and
  verification; it does not add language execution semantics.
- **Evidence:** validator unit tests, gate integration tests, task-owned Rust integration tests,
  negative malformed-record tests, and mutation tests for required staged evidence.
- **Non-goals:** general semantic realization, inferring ownership from fixtures, or expanding
  production admission.
- **Next obligation:** migrate future semantic tasks at creation and expand the command allowlist
  only with a test and explicit task-owned evidence.

## TDD Steps

1. Write validator tests for a valid record and each required rejection before creating the
   validator.
2. Write shell gate tests for staged semantic evidence and command selection before implementing
   the runner.
3. Replace TASK-2004's obsolete negative controls with RED tests for a genuinely unsupported
   source form, then prove the checked admission rejection.
4. Run the focused task integration checks and document/traceability gates.

## Completion Checklist

- [x] Structured records exist for each active semantic follow-up and validate against canonical
  traceability.
- [x] Coverage-map and task-file links make each task's rule, domain, layers, evidence,
  non-goals, and next obligation auditable.
- [x] Pre-commit runs the affected semantic task's declared integration checks.
- [x] Pre-push runs all active semantic-task verification commands.
- [x] TASK-2004 has a passing genuinely unsupported-source rejection control on both source and
  file routes.
- [x] Validator/gate mutation tests, docs gates, Rust quality checks, changelog, and code review
  pass.
