# Phase 170 Expanded-Surface Boundary Audit

## Status

TASK-1737 audit artifact for Phase 170. This document inventories the current parsed-surface-to-Core boundaries after Phase 169 and identifies the concrete routing work owned by TASK-1738.

## Scope

Audited surfaces:

- `crates/ash-parser/src/lower.rs` public lowering APIs.
- `crates/ash-engine/src/lib.rs` workflow parsing, program processing, module-file checking, and imported callable closure construction.
- `crates/ash-engine/src/module_loader.rs` module export/type metadata loading.
- `crates/ash-typeck/src/**` parsed-surface consumers that reject surface-only nodes before type semantics.
- `crates/ash-lsp-core/src/**` parsed module consumers that are analysis/tooling-only.

Surface-only nodes considered in this audit:

- `Expr::OperatorSection`.
- `Definition::Notation` / `NotationDecl`.
- Future parsed-surface-only forms that should be erased or rejected before Core.

## Classification vocabulary

| Classification | Meaning |
|---|---|
| High-level boundary | Production parser/engine/module-loading path that should validate expansion before accepting a module/file/program. |
| Low-level/test helper | Public helper intentionally usable by tests or lower layers, but not a production module boundary. It must remain fail-closed for unresolved surface-only nodes. |
| Parser-local helper | Lowering helper for one surface construct; safe if callers are already post-expansion or the helper rejects surface-only nodes. |
| Analysis/tooling consumer | LSP/type/name/purity consumer over parsed surface syntax. It should reject unresolved surface-only expression nodes but does not produce Core. |
| Deferred compatibility path | Legacy/snippet path retained for old surfaces; do not broaden it, but record bypass risk honestly. |

## `ash-parser::lower` public API inventory

| API | Location | Classification | Current surface-only behavior | TASK-1738 action |
|---|---:|---|---|---|
| `lower_fn_contract` | `crates/ash-parser/src/lower.rs:229` | Parser-local helper | Lowers contract expressions through `lower_expr`; unresolved operator sections fail closed. | No routing change. Keep fail-closed. |
| `effectful_names_from_definitions` | `crates/ash-parser/src/lower.rs:441` | Parser-local metadata helper | Reads definitions for effectful names; notation declarations have no effect authority. | No routing change. |
| `lower_workflow` / `lower_workflow_with_context` | `crates/ash-parser/src/lower.rs:467`, `:472` | Low-level/test helper; production callers are high-level engine paths | Workflow body expressions eventually use `lower_expr`; unresolved sections fail closed. Does not expand notation on its own. | Do not remove. Route production engine callers through expansion-aware module/program paths where possible. |
| `lower_workflow_def` | `crates/ash-parser/src/lower.rs:517` | Low-level/test helper | Same as `lower_workflow`; also lowers contracts. | No direct change unless TASK-1738 finds a high-level caller using it as module boundary. |
| `lower_module_type_metadata` | `crates/ash-parser/src/lower.rs:1354` | High-level module metadata helper | Reads parsed module definitions and ignores `Definition::Notation`; no expression lowering. Safe for type metadata, but not a proof that module expression surfaces crossed expansion. | Leave as type-metadata-only. TASK-1738 should add separate expansion validation at engine module-file boundary rather than forcing this helper to lower expressions. |
| `lower_surface_type_def` / `lower_surface_type` | `crates/ash-parser/src/lower.rs:1288`, `:1449` | Parser-local type helper | Type-only; no surface-only expression bypass. | No change. |
| `lower_interface_def` | `crates/ash-parser/src/lower.rs:1678` | Parser-local semantic helper | Lowers interface signatures/constraints; not expression-bearing except type surfaces. | No change. |
| `lower_impl_def` | `crates/ash-parser/src/lower.rs:1724` | Parser-local semantic helper | Lowers impl signatures/associated items; method bodies are not lowered here. | No change. |
| `lower_expr_with_context` / `lower_expr` | `crates/ash-parser/src/lower.rs:1851`, `:1859` | Low-level/test helper | Explicitly rejects `Expr::OperatorSection` with `UnsupportedFeature`; generated/expanded sections lower normally. | Keep as final fail-closed guard. Production callers should not rely on this as the only high-level expansion boundary. |
| `lower_module_expr` | `crates/ash-parser/src/lower.rs:2187` | Low-level/test helper | Adds a module-scope `FnDef` guard, then delegates to `lower_expr`. Existing comment says engine is not wired. | TASK-1738 should either wire module-scope expression call sites or rewrite the stale comment once audited. |
| `lower_expanded_surface_module` | `crates/ash-parser/src/lower.rs:2199` | High-level validation gate | Visits expression-bearing module surfaces and lower-validates post-expansion expressions. It assumes an `ExpandedSurfaceModule`. | Use at high-level engine/module-file boundaries that currently only parse/check metadata. |
| `expand_and_lower_surface_module` | `crates/ash-parser/src/lower.rs:2210` | High-level validation gate | Expands a parsed `ModuleFile`, then validates expression lowering. | Primary TASK-1738 target for `Engine::check_module_file` and module export loading where full `ModuleFile` parse succeeds. |
| `lower_builtin_fn_def` | `crates/ash-parser/src/lower.rs:2245` | Parser-local helper | Type/signature only. | No change. |
| `lower_pattern` | `crates/ash-parser/src/lower.rs:2378` | Parser-local helper | Pattern-only. | No change. |

## Engine/module-loader path inventory

| Path | Location | Classification | Current behavior | Bypass risk | TASK-1738 target |
|---|---:|---|---|---|---|
| `Engine::check_module_file` | `crates/ash-engine/src/lib.rs:1483` | High-level boundary | Parses full module for type metadata and validates public API summaries, but does not call `expand_and_lower_surface_module`. Counts `pub fn` snippets separately. | `pub fn` bodies containing unresolved operator sections can be accepted by module-file checking. This is demonstrated by `crates/ash-engine/tests/task_1737_expanded_surface_boundary_audit.rs`. | Add an expansion-validation step after successful full `ModuleFile` parse/type-metadata collection. Flip the TASK-1737 audit test to expect rejection and add a positive local-notation/section case. |
| `module_loader::collect_module_type_metadata_from_module_file` | `crates/ash-engine/src/module_loader.rs:2215` | High-level metadata boundary | Parses full `ModuleFile`, rejects unsupported inline type/domain summaries, then lowers type metadata. | Type metadata itself is safe, but callers may mistake this for full module validation. | Keep metadata-only; call expansion validation from broader callers (`check_module_file`, export collection) rather than inside this helper if type-only consumers must remain cheap. |
| `module_loader::collect_module_exports` | `crates/ash-engine/src/module_loader.rs:2600` | High-level import/export boundary | Parses module for effectful names/type metadata, then extracts public `builtin fn`, workflow, and `pub fn` snippets. `pub fn` parse/lower failures can be skipped during export collection. | Imported callable bodies can bypass module-level expansion validation; unsupported public functions may be silently skipped rather than failing the module import. | Add expansion validation after full module parse succeeds, before collecting callable exports. Preserve legacy skip behavior only for explicitly documented snippet compatibility, not full ModuleFile-valid modules. |
| `module_loader::load_ordinary_source` / `load_ordinary_file` | `crates/ash-engine/src/module_loader.rs:331`, `:348` | High-level workflow file boundary | Parses imports and later parses workflow/program forms directly. | Single-workflow/program paths call `lower_workflow`/`lower_expr_with_context` directly; unresolved sections fail closed at lower time, but declared notation sections are not expanded in the ordinary program path. | TASK-1738 should decide whether ordinary workflow files need a synthetic `ModuleFile` expansion pass now or remain deferred because this path is not a module-file surface. Document the decision. |
| `Engine::parse_workflow_source_with_imports` | `crates/ash-engine/src/lib.rs:1136` | High-level workflow parse/lower boundary | Parses workflow/program and calls `lower_workflow`/`process_program_definitions`. | Unresolved sections fail closed through `lower_workflow`; declared notation cannot appear unless the ordinary workflow parser accepts module declarations. | Likely deferred for this phase unless TASK-1738 adds a module-file wrapper for ordinary workflow sources. |
| `Engine::process_program_definitions` | `crates/ash-engine/src/lib.rs:1068` | High-level program lowering helper | Lowers entry/helper workflows and fn bodies directly with `lower_workflow` / `lower_expr_with_context`. | Function bodies with local notation declarations are not in this `Program` shape; unresolved sections fail closed. | No standalone change unless ordinary workflow source is routed through module expansion. |
| `build_imported_closures` | `crates/ash-engine/src/lib.rs:2606` | High-level imported callable lowering helper | Lowers imported `InlineCallable` bodies directly with `lower_expr_with_context`. | If `InlineCallable` body came from an unexpanded public `fn`, unresolved sections fail closed only at import-use construction and local notation is not expanded. | Prefer validating module exports before `InlineCallable` creation; do not expand individual bodies without module-local notation table context. |
| `module_loader::parse_program_with_functions` | `crates/ash-engine/src/module_loader.rs:265` | Deferred compatibility path | Parses leading `fn` definitions and workflows outside `ModuleFile`. | No notation declaration surface; direct lowerers still fail closed for sections. | Defer unless ordinary workflow sources become ModuleFile-backed. |

## Typechecker and tooling consumers

| Consumer | Location | Classification | Current behavior | TASK-1738 action |
|---|---:|---|---|---|
| `ash-typeck` expression/name/purity/capability paths | `crates/ash-typeck/src/check_expr/mod.rs`, `names.rs`, `purity.rs`, `capability_check.rs`, `lib.rs` | Analysis/tooling consumer | `Expr::OperatorSection` paths produce unsupported/diagnostic errors rather than silently assigning semantics. | No routing change. Keep fail-closed diagnostics as a second line of defense. |
| `ash-lsp-core` completion/hover/symbol/goto paths | `crates/ash-lsp-core/src/*.rs` | Analysis/tooling consumer | Consumes parsed `ModuleFile`; supports `Definition::Notation` for symbols/hover/completion. Does not lower. | No expansion routing required. |

## Concrete TASK-1738 targets

1. `Engine::check_module_file` should call `ash_parser::lower::expand_and_lower_surface_module` on the parsed module or an equivalent shared helper after authoritative `ModuleFile` parsing succeeds.
2. `module_loader::collect_module_exports` should validate the full parsed module through the same gate before public callable export collection, so importing a module cannot silently skip or defer unresolved surface-only public bodies.
3. The Phase 169 comment on `lower_module_expr` should be updated if TASK-1738 routes the production module/file boundaries instead of direct module-expression lowering.
4. Add/flip tests:
   - negative: `check_module_file` rejects a `pub fn` body containing unresolved `(<*>)`.
   - positive: `check_module_file` accepts a `pub fn` using a resolved built-in section such as `(+)` after expansion.
   - positive: `check_module_file` accepts a local notation section after expansion if the body is otherwise lowerable.
   - import negative: importing a module whose public callable contains unresolved `(<*>)` fails before closure construction or silently skipping the callable.

## Current audit proof

`crates/ash-engine/tests/task_1737_expanded_surface_boundary_audit.rs` records the current bypass: `Engine::check_module_file` accepts a full `ModuleFile` with a public function body containing unresolved `(<*>)`. TASK-1738 owns flipping this expectation after routing the high-level boundary.

## Non-targets for TASK-1738

- Do not remove `lower_expr`, `lower_workflow`, or `lower_workflow_def`; they are valid low-level/test helpers and remain fail-closed.
- Do not implement generalized `SPEC-098c` lowering for every surface form.
- Do not implement imported notation propagation; TASK-1739/TASK-1740 own that design and behavior.
- Do not attach Core origin sidecars; TASK-1741 owns origin metadata.
