# TASK-781 Type Pipeline Audit and Semantic-Summary Gate

Status: Complete
Date: 2026-05-01
Scope: docs/substrate audit only. No Rust behavior changes were made for TASK-781.

## 1. Gate statement

TASK-781 freezes the implementation gate for SPEC-057 / PLAN-105 before parser, core, engine, or typechecker behavior changes begin.

The current implementation is intentionally fragmented:

- ordinary `type` declarations are parsed by the standalone parser-private `ash_parser::parse_type_def::parse_type_def` path;
- the live `ash_parser::surface::Definition` / `surface::ModuleFile` path does not carry ordinary type declarations as normal module items;
- engine module loading and module checking discover ordinary type metadata by source-snippet scanning;
- `ash-typeck::TypeEnv` can already predeclare/register/expose ordinary types, but it consumes ad-hoc `ash_core::ast::TypeDef` values rather than core-owned module semantic summaries;
- Phase 108 workflow summary transport is live and must not be replaced or erased by ordinary-type summary work.

SPEC-057 implementation tasks must therefore add the unified path:

```text
source file
  -> ash-parser ModuleFile ordinary type item
  -> ash-core canonical ordinary type declarations / ModuleSemanticSummary
  -> ash-engine summary transport/import/export
  -> ash-typeck TypeEnv two-pass registration and representation exposure
```

Source-snippet scanning must be removed from normal semantic paths or fenced behind explicit compatibility/test-only entry points after the normal path is proven.

## 2. Exact files and functions inspected

Parser:

- `crates/ash-parser/src/parse_type_def.rs`
  - `TypeDef`, `TypeBody`, `VariantDef`, `VariantPayload`, `Visibility`, `TypeExpr`
  - `parse_type_def`
- `crates/ash-parser/src/parse_module.rs`
  - `module_file`
  - item-dispatch checks around `workflow`, `mod`, `role`, `resource type`, `capability interface`, `capability impl`, `capability`, `proxy`, `interface`, `impl`, `builtin fn`, `fn`
  - unknown-item recovery via `skip_unknown_definition`
- `crates/ash-parser/src/lib.rs`
  - `parse_surface_file`
  - `parse_surface_file_with_path`
- `crates/ash-parser/src/surface.rs`
  - `ModuleFile`
  - `Definition`

Core:

- `crates/ash-core/src/ast.rs`
  - `ModuleItem`
  - `TypeDef`, `TypeBody`, `VariantDef`, `VariantPayload`, `TypeExpr`, `Visibility`
- `crates/ash-core/src/module_graph.rs`
  - `ModuleGraph`, `ModuleNode`, `ModuleId`, crate/module anchoring helpers
- `crates/ash-core/src/workflow_carrier.rs`
  - `PublicWorkflowSummary`

Engine:

- `crates/ash-engine/src/module_loader.rs`
  - `LoadedOrdinaryFile`
  - `InlineCallable`
  - `ModuleExports`
  - `check_importable_module_file`
  - `load_ordinary_file`
  - `collect_public_type_defs_from_source`
  - `collect_type_identity_defs_from_source`
  - `collect_module_exports`
  - `insert_type_export`
  - `insert_type_export_with_name`
  - `insert_constructor_export_with_name`
  - `parse_type_def_snippet`
  - `extract_semicolon_snippets`
  - `stamp_workflow_summary_import_origin`
  - workflow summary builders including `parse_workflow_callable`, `parse_workflow_signature_callable`, `parse_supported_pub_fn_callable`, `public_workflow_summary_from_workflow`, and `public_workflow_summary`
- `crates/ash-engine/src/lib.rs`
  - `Workflow` imported metadata fields
  - `Engine::runtime_stdlib_type_defs`
  - `Engine::parse_file`
  - `Engine::check`
  - `Engine::check_module_file`
  - `register_imported_type_defs`
  - `bind_imported_callable_types`
  - `build_imported_closures`

Typechecker:

- `crates/ash-typeck/src/type_env.rs`
  - `TypeEnv` fields `ast_types`, `type_info`, `constructors`, `transparent_aliases`, `public_workflow_summaries`
  - `declare_type_name`
  - `is_placeholder`
  - `register_type_identity`
  - `expose_type_representation`
  - `register_type`
  - `has_type`
  - `has_full_type`
  - `bind_public_workflow_summary`
  - `lookup_public_workflow_summary`
- `crates/ash-typeck/src/check_expr.rs`
  - imported Workflow summary consumers through `lookup_public_workflow_summary`

## 3. Current parser and ModuleFile drift

Current live behavior:

- `parse_type_def.rs::parse_type_def` accepts ordinary type declarations including:
  - `type Name = ...;`
  - `pub type Name = ...;`
  - `builtin type Name;`
  - `pub builtin type Name;`
- `parse_type_def::TypeDef` is parser-private and separate from `surface::Definition`.
- `surface::Definition` currently has no ordinary type variant. It contains capability/resource/policy/role/proxy/interface/impl/function/builtin-function variants, but no `Definition::Type` or equivalent module item.
- `surface::ModuleFile` contains `definitions: Vec<Definition>`, `module_decls`, optional `workflow`, `span`, comments, and path, so there is currently no ordinary type carrier in the authoritative module-file parse result.
- `parse_module.rs::module_file` dispatches many visible definitions but does not dispatch ordinary `type`, `pub type`, `builtin type`, or `pub builtin type` into a module-file item.
- When `module_file` sees an unrecognized item, it calls `skip_unknown_definition`, then may consume a following semicolon. Therefore an ordinary type declaration that the standalone `parse_type_def` path accepts can be skipped by module-file unknown-item recovery rather than represented or diagnosed as a type item.
- `ash_parser::parse_surface_file` and `ash_parser::parse_surface_file_with_path` call `parse_module::module_file`, so they inherit this drift. They can return a `ModuleFile` whose `definitions` omit ordinary type declarations present in the source.

SPEC-057 consequence:

- TASK-782 must route ordinary type declarations into the normal `ModuleFile` result and prevent unknown-item recovery from silently skipping type declarations accepted by the standalone type parser.
- TASK-784 must lower the new surface type item into core-owned metadata with source anchors.

## 4. Parser-private type carrier limitations

`parse_type_def::TypeDef` currently carries:

- name;
- type params;
- body;
- visibility;
- builtin marker.

It does not carry:

- source span for the declaration;
- per-field/per-variant spans;
- source origin or filesystem/module identity;
- a canonical module/declaration anchor;
- exported-name/re-export origin metadata;
- summary visibility/opacity metadata beyond a simple visibility enum.

It is also structurally separate from `surface::Definition`, so downstream code cannot rely on a parsed `ModuleFile` to preserve ordinary type declarations. Engine code compensates by extracting source snippets and reparsing them with `parse_type_def`.

SPEC-057 consequence:

- TASK-782 should add a surface ordinary type item or equivalent normal module item.
- TASK-784 should preserve source origin/span metadata required by SPEC-057 summaries instead of depending on parser-private snippet output.

## 5. Current core carriers

`ash-core` already has ordinary type metadata in `crates/ash-core/src/ast.rs`:

- `TypeDef { name, params, body, visibility, builtin }`
- `TypeBody::{Struct, Enum, Alias}`
- `VariantDef`, `VariantPayload`, `TypeExpr`, and `Visibility`
- `ModuleItem::Type` exists in the core AST layer.

Limitations against SPEC-057:

- There is no audited core-owned `ModuleSemanticSummary` carrier for ordinary type exports/imports.
- Current `TypeDef` does not encode canonical module-anchored identity, exported alias/re-export identity, source origin, diagnostic spans, representation exposure status, or explicit opaque-placeholder state.
- `ModuleGraph` tracks module/crate graph nodes and identities, but ordinary type identities are not currently minted from it for import/export summaries.

SPEC-057 consequence:

- TASK-783 must add or designate core-owned summary and canonical identity carriers.
- TASK-784/TASK-785 must connect parser/core/module identity to the engine transport path.

## 6. Live ordinary type call graph

### 6.1 Standalone parser path

```text
ash_parser::parse_type_def::parse_type_def
  -> parses parser-private parse_type_def::TypeDef
  -> consumed by ash-engine module_loader::parse_type_def_snippet
  -> converted to ash_core::ast::TypeDef by module_loader conversion helpers
```

This path is not currently connected to `surface::Definition` / `surface::ModuleFile`.

### 6.2 ModuleFile parser path

```text
ash_parser::parse_surface_file(source)
  -> ash_parser::parse_surface_file_with_path(source, None)
    -> ash_parser::parse_module::module_file.parse_next(input)
      -> dispatches recognized module definitions
      -> does not dispatch ordinary type declarations
      -> unknown-item recovery can skip ordinary type declarations
      -> returns surface::ModuleFile without ordinary type items

ash_parser::parse_surface_file_with_path(source, Some(path))
  -> same module_file path
  -> attaches comments and path after parse
```

### 6.3 Engine module check path

```text
Engine::check_module_file(path)
  -> read source
  -> module_loader::collect_public_type_defs_from_source(source)
    -> extract_semicolon_snippets(source, is_public_type_definition_start)
    -> parse_type_def_snippet(snippet)
      -> parse_simple_type_alias_snippet(snippet) compatibility fast path OR
      -> ash_parser::parse_type_def::parse_type_def(snippet)
      -> convert parser-private TypeDef to ash_core::ast::TypeDef
  -> TypeEnv::with_builtin_types()
  -> for each type: TypeEnv::declare_type_name
  -> for each type: TypeEnv::register_type
    -> TypeEnv::register_type_identity
    -> TypeEnv::expose_type_representation
  -> count_pub_fn_snippets(source)
```

This is a normal user-facing module check path and currently depends on snippet scanning for public ordinary type metadata.

### 6.4 Importable module check path

```text
check_importable_module_file(path)
  -> read source
  -> source_contains_workflow_keyword(source)
  -> collect_module_exports(path, cache, visiting)
  -> require at least one export for workflow-containing module
```

The ordinary type work inside this path is delegated to `collect_module_exports` and is snippet-based.

### 6.5 Export collection path

```text
collect_module_exports(path, cache, visiting)
  -> read source
  -> ash_parser::parse_surface_file(source).ok()
      -> only used to collect effectful names from surface definitions
      -> ordinary type declarations are absent from this ModuleFile result
  -> extract_semicolon_snippets(source, is_type_definition_start)
      -> parse_type_def_snippet
      -> insert_type_export
          -> exports public types as-is
          -> exports private/non-public type identities as opaque empty-struct builtin placeholders
          -> exposes enum constructors only for public type definitions
  -> extract public capability names as builtin empty-struct public type identities
  -> extract pub builtin fn snippets
  -> extract workflow/pub fn snippets and build InlineCallable values
  -> extract pub mod children recursively
  -> extract pub use snippets and merge target ModuleExports
  -> cache ModuleExports
```

This is a normal module import/export path and currently uses source snippets, not ModuleFile/core summaries, as the authoritative ordinary type metadata source.

### 6.6 Ordinary file import path

```text
load_ordinary_file(path)
  -> read source
  -> parse leading use/pub use prelude with parse_ordinary_import
  -> for each import:
      -> resolve module path
      -> collect_module_exports(module_path)
      -> named import:
          -> type_defs[name] imports a type identity
          -> constructor_defs[name] imports parent type definition for constructor visibility
          -> callables[name] imports callable and also imports all exports.type_defs so callable signatures can mention private/opaque module-local types
      -> glob import:
          -> import all exports.type_defs
          -> import all exports.callables
  -> returns LoadedOrdinaryFile { workflow_source, imported_type_defs, imported_callables }
```

`LoadedOrdinaryFile.imported_type_defs` is the ad-hoc bridge from engine snippet-derived type metadata into typechecking.

### 6.7 Engine parse/check path

```text
Engine::parse_file(path)
  -> module_loader::load_ordinary_file(path)
  -> Engine::parse_workflow_source_with_imports(workflow_source, imported_type_defs, imported_callables)
      -> build_imported_closures(imported_callables)
      -> stores imported type defs/signatures/workflow summaries on Workflow side metadata

Engine::check(workflow)
  -> if surface Program cached:
      -> TypeEnv::with_builtin_types
      -> imported_type_defs = workflow imported type defs + runtime_stdlib_type_defs()
      -> register_imported_type_defs(type_env, imported_type_defs)
      -> bind_imported_callable_types(type_env, workflow)
      -> type_check_program_in_env
  -> else if entry workflow contract path:
      -> TypeEnv::with_builtin_types
      -> bind_imported_callable_types(type_env, workflow)
      -> type_check_workflow_in_env
  -> else:
      -> imported_type_defs = workflow imported type defs + runtime_stdlib_type_defs()
      -> register_imported_type_defs(type_env, imported_type_defs)
      -> bind_imported_callable_types(type_env, workflow)
      -> type_check_workflow_def_in_env

Engine::runtime_stdlib_type_defs()
  -> runtime stdlib registry source strings
  -> module_loader::collect_type_identity_defs_from_source(source)
      -> snippet scanning for type / pub type / builtin type / pub builtin type
```

### 6.8 Imported type registration and TypeEnv registration path

```text
register_imported_type_defs(type_env, imported_type_defs)
  -> first pass: if !TypeEnv::has_type(name), TypeEnv::declare_type_name(name)
  -> second pass: if !TypeEnv::has_full_type(name), TypeEnv::register_type_identity(type_def)
  -> if imported_type.visibility == Public:
       TypeEnv::expose_type_representation(name)

TypeEnv::declare_type_name(name)
  -> inserts placeholder TypeDef { body: Struct([]), visibility: Public, builtin: false }

TypeEnv::register_type_identity(def)
  -> rejects duplicate non-placeholder ast_types entry
  -> allows placeholder upgrade based on empty struct/no params shape
  -> convert_type_def(def, self)
  -> stores ast_types and type_info

TypeEnv::expose_type_representation(name)
  -> for enum: registers variant constructors
  -> for alias encoded as struct field `__alias_target`: marks transparent alias
  -> for struct: no constructor registration

TypeEnv::register_type(def)
  -> register_type_identity(def)
  -> expose_type_representation(def.name)
```

The TypeEnv side already has a two-pass shape, but it is string-keyed and placeholder-state is inferred from an empty-struct shape rather than represented explicitly.

## 7. Private type export/import compatibility behavior

Current engine compatibility behavior is important and must be handled deliberately by TASK-786/TASK-787:

- `module_loader::insert_type_export` always exports a type identity entry in `ModuleExports.type_defs`.
- If the source `CoreTypeDef.visibility` is `Public`, the type definition is exported as-is.
- If the source type is non-public, the export is converted to an opaque placeholder:

```text
CoreTypeDef {
  name: original name,
  params: original params,
  body: CoreTypeBody::Struct(vec![]),
  visibility: original visibility,
  builtin: true,
}
```

- Constructors are exported only for public enum type definitions, via `ModuleExports.constructor_defs`.
- `load_ordinary_file` imports all `exports.type_defs` when importing a callable so imported callable signatures may mention private/opaque module-local type identities.
- `register_imported_type_defs` registers those imported placeholders with `TypeEnv::register_type_identity`; it only calls `expose_type_representation` when the imported type visibility is `Public`.
- `TypeEnv::declare_type_name` also uses an empty struct placeholder, but with `builtin: false` and public visibility. `TypeEnv::is_placeholder` identifies placeholders only by empty struct/no params, so it can confuse a real empty struct or an opaque placeholder with an upgradeable placeholder.

Decision for follow-on tasks:

- TASK-786 should preserve the current compatibility intent that imported callable signatures can name existing private/opaque/builtin identities without exposing constructors or private representations, but should tag this state explicitly in summaries rather than relying on empty `Struct([])` plus `builtin = true` as the semantic signal.
- TASK-787 should reject or explicitly handle ambiguous placeholder upgrades. It should avoid treating summary-provided opaque identities as generic upgradeable placeholders, and it should distinguish real empty structs from predeclaration placeholders and opaque imported identities.
- General representation hiding must not be expanded. SPEC-057 permits opaque exported identities only for existing explicit builtin/opaque compatibility exceptions.

## 8. Snippet scanner replacement/fencing decisions

Normal paths to replace with ModuleFile/core summaries:

- `Engine::check_module_file` -> `collect_public_type_defs_from_source`
- `check_importable_module_file` -> `collect_module_exports`
- `collect_module_exports` -> `extract_semicolon_snippets(... is_type_definition_start)`
- `load_ordinary_file` import metadata obtained through `collect_module_exports`
- `Engine::runtime_stdlib_type_defs` -> `collect_type_identity_defs_from_source`
- any normal `Engine::parse_file` / `parse_workflow_source_with_imports` path that receives imported type defs from snippet-derived `LoadedOrdinaryFile`

Potential compatibility/test-only paths to fence or remove:

- `collect_public_type_defs_from_source`
- `collect_type_identity_defs_from_source`
- `parse_type_def_snippet`
- `extract_semicolon_snippets` uses for ordinary type metadata
- tests that explicitly assert snippet-scanner behavior, such as structural stdlib tests that call `collect_public_type_defs_from_source`

Do not fence unrelated snippet collectors in TASK-789 until their behavior is replaced or separately scoped:

- pub fn counting and callable extraction currently use snippet collectors too;
- pub use and pub mod extraction also use snippet helpers;
- TASK-781 only gates ordinary type metadata scanning, not a full module-loader parser rewrite for all callable/import snippets.

## 9. Phase 108 workflow summary non-interference call graph

The following live path must be preserved while ordinary-type summaries are added:

```text
collect_module_exports(path)
  -> builds ModuleExports.callables: HashMap<String, InlineCallable>
  -> InlineCallable.workflow_summary: Option<PublicWorkflowSummary>
  -> workflow/pub fn summary builders:
       parse_workflow_callable
       parse_workflow_signature_callable
       parse_supported_pub_fn_callable
       public_workflow_summary_from_workflow
       public_workflow_summary
  -> stamp_workflow_summary_import_origin(summary, module_path, exported_name)
  -> insert_callable_export(exports, exported_name, callable)

load_ordinary_file(path)
  -> imports ModuleExports.callables through named/glob/pub-use paths
  -> returns LoadedOrdinaryFile.imported_callables

Engine::parse_file / parse_workflow_source_with_imports
  -> build_imported_closures(imported_callables)
      -> copies InlineCallable.workflow_summary into workflow_summaries map
  -> Workflow.imported_workflow_summaries stores HashMap<String, PublicWorkflowSummary>

Engine::check(workflow)
  -> bind_imported_callable_types(type_env, workflow)
      -> binds imported function/builtin signatures
      -> for Workflow.imported_workflow_summaries:
           TypeEnv::bind_public_workflow_summary(name, summary)

check_expr imported Workflow consumers
  -> TypeEnv::lookup_public_workflow_summary(name)
  -> recover imported WorkflowForm summaries for variable/call sources in do:Workflow and [...]: Workflow composition
```

`ash_core::workflow_carrier::PublicWorkflowSummary` remains the Phase 108 / SPEC-056 owner of workflow contract/projection summary facts. SPEC-057 ordinary-type `ModuleSemanticSummary` work may provide type identities needed by workflow signatures, but it must not collapse `PublicWorkflowSummary` into ordinary type summaries, clear `InlineCallable.workflow_summary`, drop `Workflow.imported_workflow_summaries`, or bypass `TypeEnv::bind_public_workflow_summary` / `lookup_public_workflow_summary`.

Ordering constraint for TASK-787:

- ordinary imported type identities must be registered before imported callable signatures and before imported `PublicWorkflowSummary` consumers are checked, because workflow-returning callable signatures and imported workflow summary users may mention imported ordinary type identities.

## 10. SPEC-057 requirement-to-task traceability matrix

| SPEC-057 requirement area | Current audited state | Follow-on task(s) |
|---|---|---|
| §6.1 one authoritative ordinary type declaration path | Not satisfied. ModuleFile omits ordinary types; engine snippet scanning is authoritative in normal paths. | TASK-782, TASK-784, TASK-785, TASK-789 |
| §6.2 source-snippet scanning not normal semantic path | Not satisfied. `check_module_file`, `collect_module_exports`, `runtime_stdlib_type_defs`, and import loading use snippet scanning. | TASK-785, TASK-789 |
| §6.3 canonical module/declaration identity | Not satisfied. Current keys are mostly strings; `ModuleGraph` not used to mint type IDs. | TASK-783, TASK-784, TASK-786, TASK-787 |
| §6.4 re-exports preserve original identity | Partially approximated by cloning `CoreTypeDef`; no canonical identity exists. | TASK-783, TASK-786 |
| §6.5 public signatures may reference imported public type identities | Partially supported through imported `CoreTypeDef` registration before callable signatures. Lacks summaries/canonical IDs. | TASK-785, TASK-786, TASK-787 |
| §6.6 private representation must not leak | Compatibility path exports opaque empty-struct/builtin placeholders for private identities. This preserves intent but is untagged and ambiguous. | TASK-786, TASK-787, TASK-790 |
| §6.7 constructors exposed only when representation visible | Current `insert_type_export` exports enum constructors only for public type definitions; TypeEnv exposes representation only for public imported types. Needs summary rules. | TASK-786, TASK-787, TASK-790 |
| §6.8 ash-core owns summary carriers | Not satisfied. Engine-private `ModuleExports` owns transport structure. | TASK-783, TASK-785 |
| §6.9 ash-engine transports but does not own type semantics | Not satisfied. Engine currently decides opaque placeholder shape and snippet extraction semantics. | TASK-785, TASK-786, TASK-789 |
| §6.10 ash-typeck consumes summaries, not snippets | Not satisfied at boundary. TypeEnv consumes `CoreTypeDef`, but engine obtains them from snippets. | TASK-787, TASK-789 |
| §6.11 import order independent | TypeEnv has predeclare/register passes, but imported identities are string-keyed and not summary/canonical-ID based. | TASK-787, TASK-790 |
| §6.12 preserve ADT semantics | Current ADT registration/construction path exists. Follow-on tasks need non-regression tests. | TASK-787, TASK-790 |
| §7 Parser and ModuleFile contract | Not satisfied. `surface::Definition` lacks ordinary type item; unknown-item recovery can skip type declarations. | TASK-782 |
| §8 Lowering and core ownership | Core has `TypeDef`/`ModuleItem::Type`, but no source anchors/canonical summary lowering path. | TASK-783, TASK-784 |
| §9 ModuleSemanticSummary contract | Not satisfied. No core-owned ordinary-type summary carrier found. | TASK-783, TASK-785 |
| §9 workflow-summary preservation | Live Phase 108 path audited; must be preserved. | TASK-785, TASK-786, TASK-787, TASK-790 |
| §10 Visibility and opacity | Current behavior is ad-hoc in engine/typeck with opaque empty-struct placeholders. | TASK-786, TASK-787, TASK-790 |
| §11 Import/export/re-export behavior | Current behavior lives in `ModuleExports`, named/glob/pub-use merge logic, and callable/type import side effects; lacks canonical IDs. | TASK-785, TASK-786 |
| §12 TypeEnv two-pass integration | Partial. `declare_type_name` then `register_type_identity`/`expose_type_representation` exists, but placeholders are shape-based and identities are string-only. | TASK-787 |
| §13 Compatibility and migration | Snippet fallback candidates identified for fencing/removal. | TASK-789 |
| §14 Diagnostics | Missing diagnostics for ModuleFile omission, fallback use, placeholder conflicts, private leaks, re-export identity mismatch. | TASK-790 |
| §15 Crate ownership | Current ownership is mixed: parser-private type carrier, engine-private exports, typeck registration. | TASK-782 through TASK-789 |
| §16 Non-interference | Workflow summary transport audited; ADT/interface/capability/resource/do/comprehension regressions must remain covered. | TASK-790, TASK-791 |
| §17 Acceptance tests | No new tests in TASK-781; acceptance areas mapped to implementation tasks. | TASK-782 through TASK-791 |

## 11. Blockers and implementation notes for follow-on tasks

No blocker prevents Phase 109 implementation from proceeding after this audit. The main risks to handle explicitly are:

1. ModuleFile drift must be fixed first. Do not build semantic summaries from snippets and call that SPEC-057-compliant.
2. Parser-private `parse_type_def::TypeDef` lacks source anchors. Reusing its grammar is fine, but its output must feed a real surface item with spans/origin.
3. Engine-private `ModuleExports` should not become the semantic summary authority. It can temporarily transport core-owned summaries after TASK-783/TASK-785, but ordinary type semantics belong in core/typeck.
4. Existing private/opaque compatibility placeholders must be tagged or separated from real empty structs and TypeEnv predeclaration placeholders.
5. Phase 108 `PublicWorkflowSummary` transport is a separate live summary path and must remain intact.

## 12. Verification

- Audit document created: `docs/plan/audits/TASK-781-type-pipeline-audit.md`
- Rust behavior changes: none
- Cargo gates: not relevant for this docs/audit-only task
- Cheap docs verification required: `git diff --check`
