# Role Convergence Closeout Audit

## Scope

Fresh closeout audit for Phase 36 after TASK-221 through TASK-225.

## Original Blocker Classes Revisited

1. Placeholder role-obligation lowering in the parser/core path
2. Helper-only or dishonest inline-module role lowering
3. Touched docs/examples overstating canonical role convergence
4. Local inconsistencies in touched role examples

## Verification Evidence

The following commands were re-run in the Phase 36 / TASK-225 worktree:

- `cargo fmt --check`
- `cargo test -p ash-core`
- `cargo test -p ash-parser`
- `cargo clippy -p ash-core -p ash-parser --all-targets --all-features -- -D warnings`

Focused repository audits were also re-run over the touched specs/docs/examples and planning
materials to distinguish live canonical references from intentional historical/process-supervision
references.

## Findings

### 1. Core role metadata is now honest

- `ash-core` role metadata preserves named obligations with `RoleObligationRef`.
- The canonical IR no longer carries role-supervision structure.

### 2. Inline-module role lowering is now honest and non-placeholder

- Parser regression coverage exercises the crate-internal module-level role-definition helper path instead of relying only on a placeholder lowering story.
- Same-module capability definitions preserve authority metadata during role lowering.
- Unknown authority names fail explicitly instead of fabricating placeholder capability metadata.
- After recovery from earlier unknown inline items, later unsupported canonical inline-module items such as `workflow`, `policy`, `datatype`, and visibility-qualified entries still fail explicitly instead of being skipped silently.

This audited scope covers the maintained test-only helper path exercised by parser regression tests; it does not claim a separate non-test parser-facing lowering API, and the helper surface is now narrowed to match that reality.

### 3. Touched docs/examples are in honest states

- Canonical-facing materials now describe the flat role contract (`authority` + `obligations`) and
  explicit named approval roles.
- Non-canonical scenario examples are explicitly framed as reference-oriented or historical.
- `examples/multi_agent_research.ash` no longer refers to an undefined `reviewer` role.

## Intentional Residual References

Residual `supervises` / role-hierarchy references remain only in non-live canonical contexts:

1. historical design/plan/task documents that describe the migration away from the old role model,
2. prior audit notes that record what was changed and why,
3. canonical specs that explicitly state role hierarchy is not part of the contract,
4. runtime/process-supervision materials that discuss workflow lifecycle control rather than role
   semantics.

These remaining references are non-blocking and do not reintroduce role hierarchy into the live
role contract.

## Outcome

No blocker-class role-convergence issues remain in the audited Phase 36 scope after the inline-module honesty follow-up.

Phase 36 remains ready for final branch-level review/closeout, subject only to normal integration steps outside this task.
