# Role Convergence Audit

## Scope

Focused follow-up audit for TASK-220 after the role-contract simplification work in TASK-216
through TASK-219.

## Checks Performed

Reviewed user-facing examples and residual docs for:

- canonical `supervises` role syntax,
- wording that presents role hierarchy as part of the live role contract,
- old approval syntax that omitted the explicit `role:` label.

## Updated Materials

- example role definitions in `examples/03-policies/01-role-based.ash`
- example role definitions in `examples/code_review.ash`
- example role definitions in `examples/multi_agent_research.ash`
- example role definitions in `examples/workflows/40_tdd_workflow.ash`
- real-world examples in `examples/04-real-world/customer-support.ash` and
  `examples/04-real-world/code-review.ash`
- explanatory docs in `examples/03-policies/README.md`, `docs/TUTORIAL.md`,
  `docs/SHARO_CORE_LANGUAGE.md`, `docs/book/SUMMARY.md`, and `docs/book/appendix-a.md`

## Result

User-facing examples and residual docs in this audit scope no longer present role supervision or
role hierarchy as part of the canonical role contract.

Approval-role examples in the touched materials now use explicit named-role syntax:

- `require_approval(role: reviewer)`
- `require_approval(role: manager)`

## Intentional Remaining References

Some repository references to `supervises` or supervision remain intentionally outside the live
role contract:

1. planning/task/design documents that describe the migration from the old role model,
2. runtime/process supervision materials about `ControlLink` and workflow lifecycle authority,
3. canonical specs that explicitly say role hierarchy is *not* part of the contract.

These references are historical, explanatory, or process-supervision-specific rather than live role
syntax.
