# Phase 171 Hygiene, Origin, and Scope Boundary Audit

## Status

TASK-1744 audit artifact for PLAN-171.

## Scope

This audit maps the live parser, expansion, lowering, engine/module-loader, and typechecker seams that Phase 171 must constrain before implementing macro/notation hygiene behavior. It is intentionally conservative: macro execution is out of scope for Phase 171, imported notation activation remains out of scope unless a later task adds real summary carriers and paired positive/negative tests, and Core/runtime provenance schemas remain unchanged.

## Current-state map

### Parser surface carriers

Relevant live files:

- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/src/parse_expr.rs`
- `crates/ash-parser/src/parse_module.rs`
- `crates/ash-parser/src/lower.rs`

Current carriers in `surface.rs`:

- `RawOperatorToken { spelling, span }` preserves operator spelling and token span.
- `OperatorSection { kind, operator, left, right, span }` preserves binary infix sections before expansion.
- `SurfaceOrigin` already has variants for `Source`, `MacroExpansion`, `NotationExpansion`, `OperatorSection`, and `Desugaring`.
- `ExpandedSurfaceModule { module, diagnostics, origins }` marks the post-expansion boundary.
- `ExpandedSurfaceOrigin { generated_span, origin }` is the current narrow surface-side sidecar for generated nodes.
- `ExpansionDiagnosticKind::{DeferredMacroExpansion, DeferredNotationResolution, DuplicateNotationDeclaration, ConflictingNotationDeclaration, UnresolvedOperatorSection}` names the intended fail-closed categories, but macro invocation syntax is not yet a durable parsed carrier.
- `LocalNotationTable` and `LocalNotationEntry` are module-local; inline modules build separate notation tables.

Current notation parsing in `parse_module.rs` recognizes notation declarations with `prefix`, `infixl`, `infixr`, `infix`, `suffix`, and `mixfix` starts. The parser stores declarations as `Definition::Notation(NotationDecl)`.

### Expansion pass

`expand_surface_module(module)` in `surface.rs` currently:

1. Builds a top-level local notation table from `module.definitions`.
2. Builds a separate local notation table for each inline module's definitions.
3. Elaborates binary operator sections throughout expression-bearing definitions, workflows, contracts, laws, proofs, capability constraints, policy defaults, and do/act/comprehension/list subexpressions.
4. Records one `ExpandedSurfaceOrigin` per expanded operator section or local notation section.
5. Rejects any remaining `Expr::OperatorSection` as `ExpansionError::UnresolvedOperatorSection`.

Important current gap: `ExpandedSurfaceOrigin` records only the generated node span and one `SurfaceOrigin`; it does not provide a stable expansion identity or an origin chain. Multiple generated nodes from the same source span cannot be reliably distinguished, and nested expansion can overwrite rather than link origin context. This is owned by TASK-1745.

Important current gap: generated eta parameters use ordinary names such as `__section_lhs` and `__section_rhs`. Built-in section expansion tracks operand missingness before constructing placeholders, but local notation section expansion still infers parameter shape by comparing expression names with those generated names. This is not a full hygiene boundary and can be confused by source identifiers with the same spelling. This is owned by TASK-1746.

### Lowering boundary

`crates/ash-parser/src/lower.rs` has two relevant surfaces:

- `lower_module_expr(expr)` remains a low-level parser/test helper. It delegates to `lower_expr` and is documented as not carrying module-local notation context.
- `lower_expanded_surface_module(expanded)` validates expression-bearing surfaces in an `ExpandedSurfaceModule` by running `lower_expr` on each visited expression.
- `expand_and_lower_surface_module(module)` expands first and then validates through `lower_expanded_surface_module`.
- `lower_expr` explicitly rejects raw `Expr::OperatorSection` with an unsupported-feature diagnostic.

Current gap: there is no macro invocation carrier for `lower_expr` to reject structurally. Macro-like syntax is not yet represented as a durable node, so TASK-1748 must either add the narrow carrier or explicitly audit and preserve parser rejection while keeping macro expansion out of scope.

### Engine/module-loader boundary

Relevant live file: `crates/ash-engine/src/module_loader.rs`.

High-level engine helpers now include:

- `validate_expanded_surface_module_file(path, source)`
- `expand_surface_module_file(path, source)`
- `collect_module_exports(path, cache, visiting)`

`expand_surface_module_file` parses module type metadata and calls `ash_parser::surface::expand_surface_module`.

`collect_module_exports` now parses the module, expands it, computes effectful names from `expanded.module.definitions`, and collects public function callable exports from the expanded module. It still preserves legacy `pub workflow` and `pub fn` snippet fallback paths for compatibility-projected modules.

Known high-level positive coverage from Phase 170:

- `check_module_file` rejects unresolved operator sections in public callable bodies.
- `check_module_file` accepts built-in and local notation sections after expansion.
- `check_importable_module_file` rejects unresolved sections in public callable exports.
- imported public callable bodies use expanded operator-section bodies.
- public notation declarations remain local-only and do not become active in caller scopes.

Current gap: the module-loader has no macro invocation boundary to validate because the parser has no durable macro-call carrier. TASK-1748 must add or preserve a fail-closed boundary, and TASK-1749 must verify high-level engine/module-loader rejection.

### Typechecker boundary

Searches under `crates/ash-typeck/src` did not find use of `SurfaceOrigin`, `ExpandedSurfaceModule`, or `OperatorSection` as first-class typechecker inputs. The typechecker mostly consumes already-lowered/core-like structures and broad `Expr` paths indirectly through parser/lowering entrypoints.

Current implication: Phase 171 should keep hygiene metadata in parser/surface carriers unless TASK-1744 follow-up work proves a typechecker-owned boundary. Do not widen public typechecker APIs for origin metadata in TASK-1745 or TASK-1746 without a concrete consumer and tests.

### Diagnostics

Current diagnostics identify unresolved operator sections and duplicate/conflicting notation declarations. Origin sidecars exist but are not threaded into diagnostic messages as stable expansion chains.

Current gap: generated-name or macro-boundary diagnostics cannot yet mention a stable expansion identity/chain. TASK-1745 should provide the carrier, TASK-1746/TASK-1748 should consume it for targeted diagnostics where feasible, and TASK-1749 should validate at least one diagnostic path that exposes origin context without changing unrelated public diagnostics.

## Required target-state invariants

1. Expanded surface is the only high-level path into module/file validation for syntax that can contain notation or macro boundary nodes.
2. Surface hygiene metadata is syntax metadata only. It must not grant capability authority, rows, failures, contracts, proof/evidence obligations, or runtime provenance.
3. Every generated surface node has enough identity to distinguish separate expansion products from the same source span.
4. Nested generated nodes preserve an origin chain instead of replacing earlier origin context.
5. Generated identifiers are not ordinary source identifiers. They must not capture same-spelling source bindings, and source code must not reference generated helper names by spelling alone.
6. Local notation scope remains one module-definition scope. Parent notation does not leak into inline modules; inline-module notation does not leak into parents; imported/re-exported notation does not activate in callers without real summary carriers.
7. Macro invocation syntax, if parsed, is durable only as a fail-closed surface node. Macro execution is out of scope for Phase 171.
8. Positive visibility and negative leakage tests are required for every scope-boundary claim.

## Gap ownership

| Gap | Current evidence | Owner task | Required gate |
|---|---|---|---|
| No stable expansion identity or origin chain | `ExpandedSurfaceOrigin` has only `generated_span` plus one `SurfaceOrigin` | TASK-1745 | Parser tests for local notation, operator section, and nested origin preservation |
| Generated section binders are ordinary names | `__section_lhs` / `__section_rhs` are plain `Name` values; local notation expansion infers by name comparison | TASK-1746 | Negative capture/collision tests and diagnostics with origin context |
| Notation scope needs reaffirmed module/import boundary tests in the hygiene packet | Phase 170 tests cover non-propagation but not macro-scope placeholders | TASK-1747 | Positive callable import and negative notation/macro leakage tests |
| No durable macro invocation carrier or explicit macro-boundary rejection path | Only `SurfaceOrigin::MacroExpansion` and deferred diagnostic kind exist; no `Expr::MacroCall` found | TASK-1748 | Parser/engine tests proving macro invocation cannot lower or export |
| Cross-boundary integration must prove all prior tasks agree | Current tests are task-local from Phases 169/170 | TASK-1749 | Engine integration tests across module loading, import/export, origin, generated names, and macro rejection |
| Closeout/status drift risk | Phase 171 has newly created plan/task docs | TASK-1750 | Full parser/typeck/engine/workspace/docs gates plus independent review |

## Positive visibility tests needed downstream

- Local notation and built-in operator sections still expand in the declaring module.
- Direct callable imports remain usable even when provider notation is not imported.
- Public callable export collection uses expanded callable bodies when sections are resolved.
- Origin sidecars remain available after expansion and include enough expansion identity for diagnostics.

## Negative leakage tests needed downstream

- Imported `pub` notation remains inactive in caller scope unless a future task adds real summary carriers.
- Re-exported callables do not activate provider notation transitively.
- Inline-module notation does not leak to parent scope, and parent notation does not leak into inline modules.
- Generated helper identifiers cannot be captured by source bindings with the same spelling.
- Source code cannot refer to generated helper identifiers by spelling alone.
- Macro invocation cannot bypass expanded-surface validation, public callable export collection, or Core lowering.

## Spec patch decision

No normative spec patch is required by this audit before implementation. `SPEC-095c` already states the multi-layer pipeline, `SurfaceOrigin` categories, macro hygiene readiness, notation erasure before Core, and authority-preservation invariants. Phase 171 should implement a conservative subset of those invariants and avoid changing spec prose until implementation finds a concrete mismatch.

## Explicit non-goals retained

- Macro execution is out of scope.
- Typed macros are out of scope.
- Binder-introducing/generalized mixfix notation is out of scope.
- Imported/exported notation activation is out of scope unless a later task adds honest summary carriers and paired positive/negative tests.
- Core/runtime provenance schema changes are out of scope.
- Broad `SPEC-098c` lowering completion is out of scope.
