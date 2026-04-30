# TASK-773: Workflow Algebra and Contract Intrinsic Call Elaboration

## Status: 🚧 Expanded first slice implemented

Implemented first slice covers qualified ordinary-call preservation for `workflow::unit`, `workflow::bind`, `workflow::then`, `workflow::from_proc`, `workflow::from_act`, `workflow::requires`, and `workflow::ensures` in `do:Workflow` construction contexts. Focused coverage now includes explicit Proc/Act lift artifact preservation, direct `any_role([...])` contract classifier semantics, stored/partial/prebuilt contract intrinsic misuse rejection, and standalone open `workflow::ensures(result ...)` rejection. Full opaque Proc/Act summary validation, imported workflow summary recovery, broader standalone closed-postcondition policy, and broader composition/import/export cases remain follow-up work.

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [TASK-770](TASK-770-workflow-contract-surface-classifier-and-header-events.md)
- [TASK-771](TASK-771-workflow-type-stdlib-and-intrinsic-parameters.md)
- [TASK-772](TASK-772-workflow-form-preserving-do-target.md)

## Objective

Implement WorkflowForm-aware ordinary expression elaboration for compiler-known calls to the first-slice workflow algebra operations. In Workflow construction contexts, `workflow::unit`, `workflow::bind`, `workflow::then`, `workflow::from_proc`, `workflow::from_act`, `workflow::requires`, and `workflow::ensures` must produce or preserve `WorkflowForm` artifacts instead of lowering only to CoreExpr dictionary calls. Contract injection calls must continue to avoid exposing first-class contract values.

## Dependencies

- 📝 TASK-770: classifier and contract surface.
- 📝 TASK-771: qualified workflow builtins, shared workflow carriers, and non-denotable intrinsic parameter classes.
- 📝 TASK-772: WorkflowForm-preserving typed-do artifact.

## Requirements

1. Special-case only ordinary calls whose callee resolves exactly to a compiler-known qualified workflow builtin: `workflow::unit`, `workflow::bind`, `workflow::then`, `workflow::from_proc`, `workflow::from_act`, `workflow::requires`, or `workflow::ensures`.
2. Restrict WorkflowForm-aware call handling to Workflow construction contexts: `do:Workflow`, `[...]: Workflow` after SPEC-055 normalization, compiler-known workflow algebra composition, checked initialization/composition of Workflow values, and internal legacy declaration translation.
3. Elaborate `workflow::unit(e)` to `Unit(e)`.
4. Elaborate `workflow::bind(w, f)` to `Bind(form(w), binder, form(f binder))`, checking the continuation under binder scope. Reject if the continuation cannot be checked as producing a `WorkflowForm` / `Workflow<B>` artifact.
5. Elaborate `workflow::then(w1, w2)` to `Bind(form(w1), _, form(w2))`, preserving source order and projection events.
6. Elaborate `workflow::from_proc(p)` to `FromProc(p)` and lower Proc summary obligations; reject opaque Proc values whose required summaries are unavailable.
7. Elaborate `workflow::from_act(a)` to `FromAct(a)` or an equivalent `FromProc(proc::from_act(a))` representation, with the same lower Act/Proc summary obligations; reject opaque Act values whose required summaries are unavailable.
8. For `workflow::requires(expr)` and `workflow::ensures(expr)`, capture the raw argument expression before ordinary argument typechecking/name resolution of a `Requirement` / `OpenPostcondition` parameter.
9. Classify contract arguments with the same classifier used by statement forms and legacy header events.
10. Produce the same `Requires` / `Ensures` WorkflowForm nodes and projection events as statement forms, modulo source-origin metadata.
11. Named/local/imported `Workflow<A>` values used by these operations must carry or reference a live `WorkflowTypedArtifact` or public `WorkflowContractSummary` / workflow summary. Use the `TypeEnv` sidecar, artifact registry, or equivalent typed binding metadata introduced for `do:Workflow` so named/local values can find their artifact; reject bind/then/use of opaque `Workflow<A>` values lacking a form or public summary.
12. Reject higher-order use, partial application, storing intrinsic names as values, passing prebuilt `Requirement` variables, or exporting/importing contract argument values.
13. Standalone open `workflow::ensures(Q)` without a suffix workflow result target must reject at WorkflowForm finalization unless `Q` is explicitly closed and SPEC-056 allows that narrow case.
14. Qualified `workflow::...` names must resolve in the same compiler-known namespace style as `proc::...` names. Unqualified `unit`, `bind`, `then`, `from_proc`, `from_act`, `requires`, and `ensures` must not be made available merely because the surrounding target is `Workflow`.

## TDD Steps

1. Write WorkflowForm-shape tests for ordinary `workflow::unit(x)` calls producing `Unit(x)`.
2. Write binder-scoped continuation tests for `workflow::bind(w, fn x -> ...)` or the implementation's typed continuation representation, proving the suffix is checked under the binder and rejects non-Workflow continuations.
3. Write sequencing tests for `workflow::then(w1, w2)` producing binder-ignored `Bind(form(w1), _, form(w2))` shape.
4. Write explicit-lift tests for `workflow::from_proc(p)` and `workflow::from_act(a)` preserving `FromProc` / `FromAct` (or equivalent) forms and lower summary obligations.
5. Write equivalence tests for `requires: role(admin);` and `_ <- workflow::requires(role(admin));`.
6. Write equivalence tests for `ensures: result > 0;` and `_ <- workflow::ensures(result > 0);`.
7. Write `any_role([...])` intrinsic-call tests proving OR-role semantics match statement form semantics.
8. Write negative tests for higher-order/stored/partial intrinsic use and ordinary first-class `Requirement` / `OpenPostcondition` values.
9. Write negative tests for standalone unresolved open `ensures`.
10. Write namespace tests proving qualified `workflow::...` names resolve like qualified `proc::...` names and unqualified workflow operations are not implicitly imported by `do:Workflow`.
11. Write opaque-summary tests for named/local/imported `Workflow`, `Proc`, or `Act` values that lack required artifacts/summaries, plus positive named/local `Workflow<A>` tests that recover a registered live artifact.
12. Implement WorkflowForm-aware call recognition and event/artifact construction.
13. Run focused typechecker/elaboration tests.

## Verification

- [x] `workflow::unit`, `workflow::bind`, `workflow::then`, `workflow::from_proc`, and `workflow::from_act` ordinary calls elaborate to preserved WorkflowForm artifacts in Workflow construction contexts.
- [x] `workflow::bind` continuations are checked under binder scope and reject continuations that cannot yield a Workflow form.
- [x] Direct contract intrinsic calls elaborate to the same WorkflowForm events as statement forms.
- [x] Contract arguments are classified before ordinary value typing as contract values.
- [x] `Requirement` / `OpenPostcondition` remain non-denotable.
- [ ] Named/local/imported opaque Workflow values without a live artifact or public summary are rejected for bind/then/use, while named/local values with registered artifacts are accepted.
- [x] Qualified `workflow::...` builtins resolve; unqualified operation names are not implicitly imported by `do:Workflow`.
- [x] Standalone unresolved `ensures` rejects with a targeted diagnostic.
- [x] Existing ordinary function-call behavior does not regress.
- [x] CHANGELOG.md updated.
