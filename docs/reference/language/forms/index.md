# Forms: Declarations and Expressions

[Language reference](../index.md) · [Status and coverage](../status.md) ·
[Source of truth](../source-of-truth.md)

## Page status

**Reviewed revision:** `423f603c` (refresh AUDIT-206 rows LANG-004 through LANG-007, LANG-015,
LANG-019, and LANG-023 before changing a current-language claim).

**Implementation:** partial. The parser and checker cover many declaration and expression forms,
but admitted execution is a deliberately small `fn main` subset.
**Evidence:** tested. See the source and focused test links on the child pages.
**Parity:** below_spec. The descriptions here follow checked implementation routes, not older
workflow-era guides.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| Module functions | accepted | partial | bounded-only | fixture-bounded | partial | tested | below_spec |
| Source function contracts | accepted | partial | bounded-only | closed | partial | tested | below_spec |
| Builtin function declarations | accepted | partial | bounded-only | closed | partial | tested | below_spec |
| Anonymous and local functions, bindings, blocks, and calls | accepted | partial | bounded-only | closed | partial | tested | below_spec |
| Conditional and pattern forms | accepted | partial | bounded-only | fixture-bounded | partial | tested | below_spec |
| Law and proof declarations | accepted | partial | not-applicable | not-applicable | partial | tested | below_spec |
| Obligation-check expression | accepted | rejected-after-parse | lowered | closed | partial | tested | below_spec |

An admission/runtime cell based on a named exact Engine test is not a general evaluator contract.
A cell indicating no admitted route means that this task found no execution path for that feature
family. The status table is intentionally separate from the pages' prose examples.

## In this chapter

- [Declarations and functions](declarations-and-functions.md) — active top-level declaration
  inventory, ordinary and builtin functions, contracts, and the authoring-only law/proof boundary.
- [Values, bindings, blocks, and calls](values-bindings-blocks-and-calls.md) — function values,
  `let`, scoped blocks, direct calls, and applications.
- [Control flow and patterns](control-flow-and-patterns.md) — `if`, `if let`, `match`, patterns,
  diagnostics, and the rejected `check` static route.

## Scope boundary

The active module parser branches are in
`crates/ash-parser/src/parse_module.rs::module_file`; function declarations are parsed by
`parse_module/fn_defs.rs`, and expression forms by `parse_expr.rs`. The retained
`surface::Definition` variants for capability and policy are not evidence of accepted top-level
source syntax, so they do not appear here as declarations.

This chapter does not document handlers as executable control flow; TASK-2051 owns that
admission/runtime route. It likewise does not make `do` a generic execution claim. The exact
`do { return 42; }` spelling appears below only because the Engine's `fn main` fixture executes it;
its general semantics remain owned by the handlers/effects chapter.

Removed workflow/tower syntax, legacy callable arrows, source `raise`, and `dtype` are excluded
from current examples. They are not fallback spellings.

## Related evidence

- [AUDIT-206 implementation census](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
- [TASK-2047](../../../plan/tasks/TASK-2047-language-reference-forms-functions-control-patterns.md)
- [TASK-2048](../../../plan/tasks/TASK-2048-language-reference-ordinary-types-interfaces.md)
  — callable types and ordinary type declarations
- [TASK-2052](../../../plan/tasks/TASK-2052-language-reference-entry-engine-clients-terminals.md)
  — the separately owned Engine boundary
- [TASK-2053](../../../plan/tasks/TASK-2053-language-reference-stdlib-diagnostics-limitations.md)
  — cross-cutting diagnostic and limitation inventory
