# Phase 108: First-Class Workflow Carrier Examples

Phase 108 promotes `Workflow<A>` into a first-class computation carrier above `Proc<A>`.

Current executable examples focus on the implemented MVP surface:

- `do:Workflow { ... }` produces first-class `Workflow<A>` values.
- `requires:` and `ensures:` statements inject workflow contract events into the same `WorkflowForm` path.
- Legacy `workflow ... { ... }` declarations are still accepted for compatibility, but `ash check` emits `DeprecatedLegacyWorkflowDeclaration` and new code should prefer first-class `Workflow<A>` definitions.

Reference-only examples document intended algebra/comprehension spellings that are already covered at lower parser/typechecker layers, but are not yet end-to-end source-file examples because `parse_file` still lowers some typed workflow expressions before typed elaboration.

## Files

Executable with `ash check`:

- `01-do-workflow-unit.ash`: first-class `Workflow<Int>` value with a pure final result.
- `02-do-workflow-contract-statements.ash`: `requires:` / `ensures:` contract statements in `do:Workflow`.
- `06-legacy-workflow-migration-warning.ash`: deprecated legacy declaration plus first-class rewrite; `ash check` should succeed with a warning.

Reference-only:

- `03-workflow-algebra-intrinsics.reference.ash`: `workflow::unit` / `bind` / `then` / contract intrinsic call spelling.
- `04-workflow-explicit-lifts.reference.ash`: explicit `workflow::from_proc` / `workflow::from_act` lifts; no implicit lift.
- `05-workflow-comprehension.reference.ash`: `[...]: Workflow` comprehension target spelling.

Deferred behavior intentionally not shown as executable:

- implicit `Act<A>` / `Proc<A>` to `Workflow<A>` lifts;
- dynamic admission;
- workflow handles;
- workflow-level parallel operators.
