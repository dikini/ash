# Phase 172 Macro Execution MVP Audit

## Status

TASK-1752 audit artifact for PLAN-172. This audit records the live Phase 171 state and freezes the safe parser-first macro execution subset before implementation.

## Current carriers and parsers

### Parsed macro invocation carrier

Live carrier: `crates/ash-parser/src/surface.rs`:

- `Expr::MacroInvocation { invocation: MacroInvocation }`
- `MacroInvocation { name, delimiter, raw_body, span }`
- `MacroDelimiter::{Paren, Bracket, Brace}`

Live parser: `crates/ash-parser/src/parse_expr.rs`:

- `parse_macro_invocation_after_bang` only activates after an unqualified expression name followed by `!` and a real macro delimiter.
- It preserves a conservative raw body substring and delimiter.
- It is not a token-tree parser: it counts only the matching delimiter pair and does not understand strings, comments, escapes, or mixed delimiter nesting.
- Qualified macro-like paths such as `module::m!(x)` are not represented by the carrier and must remain rejected unless a later phase adds qualified macro path support.

### Expansion boundary

Live expansion entrypoint: `crates/ash-parser/src/surface.rs::expand_surface_module`.

Current order:

1. Build local notation table for each module/inline-module scope.
2. Elaborate notation/operator-section syntax to ordinary surface calls/closures.
3. Reject any remaining `Expr::OperatorSection`.
4. Reject any remaining `Expr::MacroInvocation` as `ExpansionError::DeferredMacroInvocation`.
5. Return `ExpandedSurfaceModule` with origin sidecars.

Phase 172 changes this order by inserting a local macro declaration/registry pass and supported expression macro expansion before notation/operator-section elaboration. Unsupported macros still reject before Core.

### High-level validation paths

High-level engine/module-loader paths already call expanded-surface validation:

- `crates/ash-engine/src/module_loader.rs::check_importable_module_file`
- `crates/ash-engine/src/module_loader.rs::expand_surface_module_file`
- `crates/ash-engine/src/module_loader.rs::load_ordinary_file`

Lowering rejects macro invocations directly. Typechecker/lint/repl visitors contain explicit `Expr::MacroInvocation` handling and must be revisited when adding macro declarations or structured invocation data.

## Parser-first MVP decision

Only parenthesized local expression-position invocations execute in Phase 172:

```ash
macro inc(x) => add(x, 1);
fn f(n: Int) -> Int { inc!(n) }
```

Executable invocation subset:

- unqualified `name!(expr, ...)`;
- `MacroDelimiter::Paren` only;
- arguments parsed as ordinary surface expressions;
- local module macro declarations only;
- exact arity;
- expansion before notation/operator-section resolution and before Core lowering.

Fail-closed invocation subset:

- `name![...]` and `name!{...}` remain diagnostic carriers but do not execute;
- `module::name!(...)` remains rejected;
- missing macro names reject;
- duplicate local macro declarations reject;
- imported/re-exported macro declarations do not activate;
- malformed argument lists reject;
- recursion or expansion-depth overflow rejects.

## Template whitelist

The macro template body is parsed as an ordinary surface `Expr`. The MVP substitutes occurrences of template parameter variables with invocation argument expressions. Because Phase 172 does not implement binder hygiene, only binder-free expression shapes may execute.

| Expr variant | MVP status | Reason / required handling |
|---|---|---|
| `Literal` | allowed | No bindings or authority. |
| `Variable` | allowed | Substituted if the name is a macro parameter; otherwise preserved as ordinary source reference. |
| `Call` | recursively allowed | Ordinary callable syntax; rows/authority come from downstream callable semantics. |
| `FieldAccess` | recursively allowed | Binder-free projection shape. |
| `IndexAccess` | recursively allowed | Binder-free projection shape. |
| `Unary` | recursively allowed | Binder-free expression composition. |
| `Binary` | recursively allowed | Binder-free expression composition; raw operator/notation resolution remains downstream. |
| `Constructor` | recursively allowed | Binder-free if all field/payload expressions are allowed. |
| `List` | recursively allowed | Binder-free if all items are allowed. |
| `FnApply` | recursively allowed | Binder-free if callee/args are allowed; callable authority remains downstream. |
| `OperatorSection` | allowed as unresolved syntax only if later notation/operator pass can resolve it | Macro output re-enters notation/operator expansion; unresolved sections still fail closed. |
| `MacroInvocation` | rejected or recursively expanded under explicit depth limit | Prevents accidental unbounded recursion and unsupported macro carriers. |
| `FnDef` | rejected for MVP | Introduces binders; needs real def-site/call-site hygiene. |
| `Block` | rejected for MVP | Contains `let` binders and sequencing. |
| `Match` | rejected for MVP | Match arms introduce pattern binders. |
| `IfLet` | rejected for MVP | Pattern binder. |
| `ActBlock` | rejected for MVP | Statement binders/effectful sequencing. |
| `DoBlock` | rejected for MVP | Statement binders and typed do semantics. |
| `Comprehension` | rejected for MVP | Qualifier binders. |
| `WithError` | rejected for MVP | Handler arms introduce patterns/binders and operational semantics. |
| `Fail` | rejected for MVP | Operational bottom should not be macro-introduced in MVP without explicit design. |
| `Panic` | rejected for MVP | Avoid introducing diagnostic/runtime behavior through macro templates in MVP. |
| `CheckObligation` | rejected for MVP | Proof/evidence semantics out of scope. |
| `Policy` | rejected for MVP | Policy-specific surface requires separate authority/evidence review. |

This whitelist is intentionally conservative. If implementation discovers a needed allowed variant, patch this audit and the affected task before adding behavior.

## Fail-closed unsupported forms

Phase 172 must retain or add explicit diagnostics for:

1. macro declarations outside module definition lists;
2. duplicate local macro declarations;
3. macro declarations exported/imported as callable summaries;
4. macro invocation of missing local macro;
5. macro invocation using bracket or brace delimiters;
6. qualified macro-like invocation syntax;
7. malformed parenthesized argument lists;
8. arity mismatch;
9. unsupported template expression variant;
10. recursive macro expansion or expansion-depth overflow;
11. any `MacroInvocation` surviving expansion before Core lowering/export/typecheck acceptance.

## Scope model

Macro declarations are module-local for Phase 172.

- Local macro declarations are visible only to expansion of definitions/workflows in the same module file or same inline module scope.
- Inline modules get their own local macro table, mirroring the Phase 171 local notation-table model.
- Parent modules do not see inline-module macros.
- Inline modules do not see parent macros unless a later phase adds explicit macro summary/import carriers.
- `pub macro` may parse as syntax metadata if TASK-1754 keeps a visibility field, but TASK-1755 must prove that it does not create importable exports or downstream activation.

## Expansion order target

Target Phase 172 order inside `expand_surface_module`:

1. Build local macro registry for each module/inline-module definition list.
2. Expand supported local parenthesized expression macros with a bounded depth limit.
3. Record `SurfaceOrigin::MacroExpansion` sidecars for macro-produced nodes.
4. Elaborate notation/operator sections on the macro-expanded output using the existing local notation table.
5. Preserve macro origin as parent origin for notation/operator-section products generated inside macro output.
6. Reject leftover macro invocations, unresolved operator sections, and unsupported forms before Core lowering.

## Origin and hygiene requirements

- Every macro expansion product needs a stable `ExpansionId` and `SurfaceOrigin::MacroExpansion` origin sidecar.
- Nested notation/operator-section expansion inside macro output must preserve the macro expansion origin as parent.
- Macro templates cannot create source-spellable generated helper identifiers. Phase 172 should not introduce generated helpers beyond existing notation/operator-section helpers unless it adds equivalent `$ash_generated_*` spelling fences.
- Macro parameter substitution preserves the invocation argument expressions' source spans for diagnostics where possible, while the expanded enclosing product records macro call origin.

## Test ownership map

| Task | Test/artifact ownership |
|---|---|
| TASK-1753 | Spec/changelog/index checks; no Rust behavior. |
| TASK-1754 | Parser tests for macro declarations, structured parenthesized args, unsupported delimiters, and qualified rejection. |
| TASK-1755 | Parser/engine tests for local registry, duplicate/missing macro, no import/export activation. |
| TASK-1756 | Parser expansion tests for successful local expression macro expansion, arity mismatch, unsupported template forms, recursion/depth, bracket/brace rejection, macro output re-entering notation expansion. |
| TASK-1757 | Parser metadata tests for macro expansion origins, parent origin through notation/operator expansion, generated-name capture negatives. |
| TASK-1758 | Engine/parser/typechecker cross-boundary tests proving supported local macros pass high-level routes and unsupported/imported leftover macros fail before Core/export/typecheck acceptance. |
| TASK-1759 | Broad gates, independent review, status reconciliation. |

## File ownership map

| File | Expected Phase 172 role |
|---|---|
| `crates/ash-parser/src/surface.rs` | Macro declaration carrier, registry, expansion, whitelist validation, origin metadata, traversal updates. |
| `crates/ash-parser/src/parse_module.rs` | Parse module-level `macro name(params) => expr;`. |
| `crates/ash-parser/src/parse_expr.rs` | Preserve raw invocation data and add structured parenthesized argument parsing for executable subset. |
| `crates/ash-parser/src/lower.rs` | Keep macro declarations/invocations rejected or erased before Core only after successful expansion. |
| `crates/ash-engine/src/module_loader.rs` | Ensure high-level module checks use expanded validation and do not export/import macro declarations. |
| `crates/ash-typeck/src/*` | Keep direct leftover `Expr::MacroInvocation` rejected; update exhaustive consumers for new carriers. |
| `crates/ash-lint/src/rules.rs` | Update traversal/ignore behavior for new carriers explicitly. |
| `crates/ash-repl/src/ast.rs` | Render or reject new surface carriers explicitly. |

## Stop conditions

Stop and ask for human input if:

- the `macro name(args) => expr;` syntax conflicts with existing parser grammar in a way that would require broad surface semantics changes;
- preserving origin metadata requires changing Core provenance/runtime schemas;
- executing a desired template requires binder hygiene, typed macro inference, token-tree parsing, or imported macro summary carriers;
- local-only macro scope conflicts with an existing implemented import/export invariant.
