# TASK-776: Workflow Contract Syntax and Legacy Translation

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
- [SPEC-055](../../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md)
- [SPEC-051](../../spec/SPEC-051-WORKFLOW-SEMANTICS.md)
- [TASK-769](TASK-769-workflow-form-projection-semantics.md)

## Objective

Add the surface syntax and compatibility bridge needed for first-class workflow contracts: `requires:` / `ensures:` statements in `do:Workflow`, compiler-known intrinsic-call handling for `workflow::requires(...)` / `workflow::ensures(...)`, conservative legacy-compatible contract-expression name resolution, and deprecated legacy workflow declaration translation into the same `WorkflowForm` path.

## Requirements

1. Depend on TASK-769's workflow-form/projection schema.
2. Extend the surface AST with workflow contract statement carriers for generalized do blocks:
   - `requires: expr;`
   - `ensures: expr;`
3. Parser support must preserve the legacy colon spelling and statement semicolon inside `do:Workflow`.
4. The parser must not change `do:Act`, `do:Proc`, or comprehension grammar behavior.
5. Define contract-expression classification over ordinary parsed `Expr`:
   - preserve arithmetic/boolean expressions as legacy-compatible preconditions;
   - recognize legacy helper calls such as `role(name)` and `any_role([name, ...])` in contract-expression context;
   - treat bare identifiers inside legacy role lists as symbolic role names;
   - preserve current legacy/core semantic contract cases underneath the new surface, including role, capability, arithmetic/precondition, and postcondition predicate cases already expressible today;
   - treat `result` as an open binder only in postcondition target resolution.
6. Implement or specify intrinsic elaboration for `workflow::requires(expr)` and `workflow::ensures(expr)` so they construct the same `WorkflowForm` events as statement forms without exposing first-class `Requirement` / `OpenPostcondition` values.
7. Attempts to store, pass, return, or pattern-match `Requirement` / `OpenPostcondition` as ordinary values must be rejected as out of scope.
8. Legacy workflow declarations remain accepted but emit `[NEW] DeprecatedLegacyWorkflowDeclaration` warnings.
9. Legacy `plays role(...)`, capability/resource/admission headers, `requires:`, and `ensures:` clauses must translate into leading `Requires` / `Ensures` workflow-form events using the same classifier as new forms.
10. Legacy syntax-heavy workflow bodies may initially translate through the existing body-to-Proc compatibility adapter and then wrap as `FromProc(legacy_body_as_proc_summary)`, but the resulting declaration must use the same `WorkflowForm` projection/obligation/coverage path as first-class workflow expressions.
11. Add regression tests proving deprecated legacy declarations and equivalent first-class `do:Workflow` forms produce equivalent workflow-form contract events.

## TDD Steps

1. Write parser tests for `do:Workflow { requires: role(admin); return x }` and `do:Workflow { ensures: result > 0; return x }`.
2. Write parser/typechecker regression tests proving `do:Act` and `do:Proc` still reject or ignore workflow-only contract statements as appropriate.
3. Write classifier tests for legacy-compatible `role(...)`, `any_role([...])`, arithmetic/boolean requirements, current legacy/core role/capability/precondition/postcondition cases, and `result` postconditions.
4. Write negative tests for ordinary first-class misuse of `Requirement` / `OpenPostcondition` values.
5. Write legacy declaration translation tests proving warnings are emitted and contract events are equivalent to the new workflow-form spelling.
6. Implement parser/surface/typechecker warning and translation changes.
7. Run focused parser/typechecker tests and Act/Proc do/comprehension regressions.

## Verification

- [ ] `requires: expr;` and `ensures: expr;` parse inside `do:Workflow` with spans preserved.
- [ ] Contract statements do not change existing `do:Act`, `do:Proc`, or comprehension behavior.
- [ ] Legacy-compatible contract-expression classification handles `role(...)`, `any_role([...])`, arithmetic/boolean requirements, current legacy/core role/capability/precondition/postcondition cases, and `result` postconditions.
- [ ] `workflow::requires(expr)` and `workflow::ensures(expr)` elaborate to the same events as statement forms.
- [ ] `Requirement` and `OpenPostcondition` cannot be used as ordinary first-class values.
- [ ] Deprecated legacy workflow declarations emit warnings and translate to the same `WorkflowForm` implementation path.
- [ ] Equivalent legacy and first-class forms produce equivalent contract-event sequences.
- [ ] CHANGELOG.md updated.
