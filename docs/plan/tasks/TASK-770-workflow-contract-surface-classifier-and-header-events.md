# TASK-770: Workflow Contract Surface, Classifier, and Header Events

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
- [TASK-769](TASK-769-workflow-form-projection-semantics.md)

## Objective

Add the parser/surface substrate for first-class workflow contract syntax and deprecated legacy declaration compatibility: `requires:` / `ensures:` do statements, source-ordered `WorkflowHeaderEvent`s, and a legacy-compatible contract-expression classifier skeleton.

## Dependencies

- 📝 TASK-769: Workflow form, projection, obligation, and adapter semantics.

## Requirements

1. Extend `crates/ash-parser/src/surface.rs` with workflow contract statement carriers for generalized do blocks, preferably explicit variants or an equivalent tagged form:
   - `DoStmt::WorkflowRequires { expr, span }`
   - `DoStmt::WorkflowEnsures { expr, span }`
2. Extend `crates/ash-parser/src/parse_expr.rs::parse_do_stmt` to parse `requires: expr;` and `ensures: expr;` while preserving the existing `let`, `<-`, and final `return` behavior.
3. Contract statement variants must preserve raw `Expr` and spans. They are classified later and must not be ordinary-value lowered by the parser.
4. Typechecking in later tasks rejects these statements outside `do:Workflow`; this parser task must not change `do:Act`, `do:Proc`, or comprehension grammar behavior.
5. Extend `crates/ash-parser/src/surface.rs::WorkflowDef` with a source-ordered `header_events: Vec<WorkflowHeaderEvent>` or equivalent. Existing `plays_roles`, `capabilities`, `owned_resources`, `used_bindings`, and `contract` fields may remain as compatibility/derived views.
6. Extend `crates/ash-parser/src/parse_workflow.rs` so `workflow_def`, `parse_plays_roles`, `parse_workflow_header_clauses`, and `parse_opt_contract` preserve exact source order in `WorkflowHeaderEvent`. This likely requires a unified header-event collection loop rather than only the current phase-separated parse order.
7. Keep `ash-parser` ownership limited to raw surface carriers: `DoStmt` raw `requires:` / `ensures:` expressions, `WorkflowHeaderEvent` raw clauses, spans, source order, and origin. Semantic `WorkflowForm`, `WorkflowContract`, coverage, and executable metadata are owned by later layers/shared `ash-core` carriers, not parser AST.
8. Define/implement a classifier skeleton over raw `Expr` with a mapping table matching SPEC-056:
   - `role(name)` -> `Requirement::HasRole(name)`.
   - `any_role([a, b, ...])` -> implemented OR-role requirement carrier, not two AND requirements.
   - bare identifiers in role lists -> symbolic role refs, not lexical variables.
   - arithmetic/boolean legacy predicates -> current compatible precondition/arithmetic carrier.
   - legacy capability/resource/binding headers -> header events preserving current semantics.
   - `ensures` expressions over `result` -> open postconditions with delayed result binder.
9. Add `AnyRole` / role-policy carrier to the parser/core contract model where needed so accepted `any_role` syntax has real semantics.
10. Update cross-crate visitors that exhaustively match `DoStmt` enough to compile or explicitly fail closed in later tasks; do not silently erase contract statements.
11. Update all `WorkflowDef` constructors/tests and exhaustive matches affected by `header_events`, with focused `cargo check` coverage.

## TDD Steps

1. Write parser tests for `do:Workflow { requires: role(admin); return x }` and `do:Workflow { ensures: result > 0; return x }`.
2. Write parser tests proving existing `do:Act` and `do:Proc` syntax still parses unchanged.
3. Write parser tests for mixed legacy workflow headers proving `WorkflowHeaderEvent` preserves exact source order while legacy aggregate fields remain populated, including interleavings such as `plays role`, `requires:`, `owns`, `ensures:`, and `uses`.
4. Write classifier tests for `role(...)`, `any_role([...])`, arithmetic predicates, and delayed `result` postconditions.
5. Write negative classifier tests for `any_role([])` and unclassified contract expressions.
6. Implement the surface/parser/classifier substrate.
7. Run focused parser tests, cross-crate constructor/match compilation checks, and affected `cargo check`.

## Verification

- [ ] `requires: expr;` and `ensures: expr;` parse inside do blocks with spans preserved.
- [ ] Source-ordered `WorkflowHeaderEvent`s preserve legacy declaration header order.
- [ ] Parser additions remain raw surface carriers and do not own semantic WorkflowForm/coverage/runtime metadata.
- [ ] Existing legacy aggregate fields remain available or are derived without behavior loss.
- [ ] `any_role([...])` has a real OR-role semantic carrier.
- [ ] Contract statement nodes are not silently erased by visitors/lowering.
- [ ] WorkflowDef constructor/exhaustive-match fallout is addressed across crates.
- [ ] Existing Act/Proc do parser tests still pass.
- [ ] Focused affected `cargo check` passes.
- [ ] CHANGELOG.md updated.
