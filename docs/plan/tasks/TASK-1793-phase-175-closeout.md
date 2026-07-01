# TASK-1793: Close out Phase 175 with broad gates and independent review

## Status: 📝 Planned

## Description

Close Phase 175 by running focused and broad verification, resolving independent review findings, and reconciling all planning/status surfaces. Do not close with accepted blockers unless explicitly deferred by the user.

## Specification Reference

- [PLAN-175: Name-Resolution-Backed Semantic Identity for Macros and Tooling](../PLAN-175-NAME-RESOLUTION-BACKED-SEMANTIC-IDENTITY-FOR-MACROS-AND-TOOLING.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-038: Language Server](../../spec/SPEC-038-LANGUAGE-SERVER.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)

## Dependencies

- 📝 TASK-1785 through TASK-1792 complete

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Token-only LSP references | Phase 174 closeout | No resolved semantic identity substrate | This task contributes to substrate | Replace only when resolved identity is proven | Same-name macro/function negative tests |
| Cross-file macro navigation | Phase 174 non-goal | No imported macro identity contract or workspace graph | Summary identity can be designed; graph still absent | Prepare summary-identity boundary only | Unsupported cases return honest no-result |

## Requirements

### Functional Requirements

1. Run all focused Phase 175 parser/engine/LSP identity tests.
2. Run the broad Phase 175 baseline gates.
3. Obtain independent review of identity overclaims, runtime callability leakage, cache invalidation, and stale docs.
4. Fix or explicitly defer every review finding.
5. Mark Phase 175 complete only after gates and review are clean.

### Property Requirements

- Macro identity must remain syntax-phase metadata.
- Macro identity must not imply runtime callability or callable export authority.
- Any reference/navigation behavior must fail closed when identity is ambiguous or unavailable.

## TDD Steps

### Step 1: Run focused verification

Run task-specific parser, engine, and LSP test targets.

### Step 2: Run broad gates

Run the full PLAN-175 verification baseline.

### Step 3: Independent review

Dispatch or run a reviewer and inspect all findings.

### Step 4: Reconcile and close

Patch docs/status/changelog only after findings are addressed.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-parser
  - cargo test -p ash-engine
  - cargo test -p ash-lsp-core
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [ ] Focused tests pass
  - [ ] Broad gates pass
  - [ ] Independent review complete and findings addressed
  - [ ] Status surfaces and changelog agree
```

## Dependencies for Next Task

This task feeds the following Phase 175 tasks according to the dependency table in PLAN-175.

## Notes

Closeout should recommend the next phase only after reviewing what identity substrate Phase 175 actually delivered.
