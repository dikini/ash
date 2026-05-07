# TASK-831 Type Function Seam Audit

Status: complete
Scope: SPEC-061 / PLAN-109 audit only; no Rust implementation changes.

## Inputs inspected

- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/src/parse_module.rs`
- `crates/ash-parser/src/parse_type_def.rs` as the ordinary `type` helper consumed by `parse_module.rs`
- `crates/ash-parser/src/lower.rs`
- `crates/ash-core/src/type_ir.rs`
- `crates/ash-core/src/semantic_summary.rs`
- `crates/ash-typeck/src/type_env.rs`
- `crates/ash-typeck/src/normalizer.rs`
- `crates/ash-typeck/src/error.rs`
- `crates/ash-engine/src/module_loader.rs`
- `crates/ash-engine/src/lib.rs`

## 1. Parser dispatch functions and AST carriers to change

### File-level dispatch

Exact dispatch seams in `crates/ash-parser/src/parse_module.rs`:

- `pub fn module_file(input: &mut ParseInput) -> ModalResult<crate::surface::ModuleFile>`
  - Current order checks `starts_with_type_definition(input)` before `starts_with_sealed_domain(input)`.
  - TASK-832 must add a `starts_with_type_fn_definition(input)` branch before `starts_with_type_definition(input)` so `type fn` is not consumed by ordinary type parsing.
  - Add equivalent recovery coverage to `starts_with_recoverable_definition(input)`.
- `fn parse_definitions(input: &mut ParseInput) -> ModalResult<Vec<Definition>>`
  - Inline-module definition dispatch currently accepts ordinary `type` via `starts_with_type_definition(input)` and intentionally rejects sealed domains by falling through to `starts_with_unsupported_inline_definition(input)`.
  - SPEC-061 says `type fn` is a top-level module definition; if inline modules remain unsupported for sealed/type-function substrate, add `type fn` to the unsupported-inline guard path rather than silently skipping or parsing it as ordinary `type`.
- `fn starts_with_type_definition(input: &ParseInput) -> bool`
  - Current implementation returns true for any bare `type` or visible `type` after `parse_visibility`; it must explicitly exclude `type fn` and visible `pub type fn` / `pub(crate) type fn` unless a prior `starts_with_type_fn_definition` branch has already claimed them.
- `fn starts_with_visible_keyword(input: &ParseInput, word: &str) -> bool`
  - Reuse for `pub fn` style prefixes is not enough for `pub type fn`; add a dedicated lookahead because the discriminant is the two-keyword sequence `type fn` after visibility.
- `fn starts_with_unsupported_inline_definition(input: &ParseInput) -> bool`
  - Currently lists `"pub"`, `"workflow"`, `"policy"`, `"datatype"`, `"memory"`, `"mod"`, `"interface"`, `"impl"`, `"sealed"`.
  - Ensure inline `type fn` trips this path with a clear diagnostic/rejection rather than unknown-item recovery.
- `fn parse_type_definition(input: &mut ParseInput) -> ModalResult<Definition>`
  - Delegates to `crate::parse_type_def::parse_type_def`; do not extend this to parse `type fn`. Add a sibling parser such as `parse_type_fn_definition` in `parse_module.rs` or a new file, and dispatch before this function.
- Existing type-expression parser seams to reuse for signatures/RHS source syntax:
  - `fn parse_surface_type(input: &mut ParseInput) -> ModalResult<Type>`
  - `fn parse_surface_type_atom(input: &mut ParseInput) -> ModalResult<Type>`
  - `fn parse_required_type_arguments(input: &mut ParseInput) -> ModalResult<Vec<Type>>`
  - These already parse `Name`, `Name<...>`, and `base::Assoc` into the surface `Type` carrier. Type-function RHSs can initially reuse the same spelling but must not reuse semantic lowering that treats all `Name<...>` as nominal constructors.

### Ordinary type helper seam

`crates/ash-parser/src/parse_type_def.rs` owns legacy ordinary `type` parsing:

- `pub fn parse_type_def(input: &mut ParseInput) -> ModalResult<TypeDef>`
- `fn parse_type_expr(input: &mut ParseInput) -> ModalResult<TypeExpr>`
- `fn parse_named_type(input: &mut ParseInput) -> ModalResult<TypeExpr>`
- `fn parse_constructor_type(input: &mut ParseInput) -> ModalResult<TypeExpr>`

Do not add `type fn` here except possibly a targeted rejection if `parse_module.rs` lookahead cannot fully protect it. This parser's `TypeExpr::Constructor { name, args }` is ordinary-type-oriented and should not become the type-function equation carrier.

### Surface AST carriers

Exact carriers in `crates/ash-parser/src/surface.rs` to extend:

- `pub enum Definition`
  - Add a new `TypeFn(TypeFnDef)` variant near `Type(TypeDef)` / `SealedDomain(SealedDomainDef)`.
- Add a new `pub struct TypeFnDef` with at least:
  - `visibility: Visibility` retained for `pub type fn` / `pub(crate) type fn` rejection diagnostics;
  - `name: Name`;
  - `params: Vec<TypeFnParam>` or reuse `Param` if parameter spans are not needed beyond the declaration span;
  - `return_type: Type`;
  - `decreases: Option<TypeFnDecreases>` preserving the name and span;
  - `equations: Vec<TypeFnEquation>`;
  - `span: Span`.
- Add `TypeFnParam` if TASK-832 needs per-param spans. Existing `Param { name, ty }` has no span and is likely insufficient for invalid `decreases`, duplicate params, and signature diagnostics.
- Add `TypeFnEquation` with:
  - source `case` head `Name` plus span;
  - `patterns: Vec<TypePattern>`;
  - `result: Type` for the raw RHS spelling, or a dedicated raw RHS wrapper over `Type` if spans for nested RHS heads are required;
  - row `span`.
- Add raw type-level pattern carrier, distinct from runtime `Pattern`:
  - `TypePattern::Constructor { name: Name, args: Vec<TypePattern>, span: Span }`
  - `TypePattern::Var { name: Name, span: Span }`
  - `TypePattern::Wildcard { span: Span }`

Do not reuse runtime `Pattern`: SPEC-061 patterns resolve only sealed-domain marker constructors, variables, and wildcards; runtime data constructors, literals, tuples, records, lists, and guards are out of scope.

## 2. Core/type_ir and semantic_summary carriers to extend

### `crates/ash-core/src/type_ir.rs`

Live carriers:

- `pub struct TypeComputationHeadId { pub module: ModuleIdentity, pub name: String }`
- `pub enum CanonicalTypeExpr`
  - `Primitive(String)`
  - `Var(String)`
  - `NominalApp { origin: TypeDeclId, visible_name, args, kind }`
  - `Projection { interface, member, args, kind, rigidity }`
  - `ComputationHeadApp { head: TypeComputationHeadId, args, kind }`
- `pub enum NormalTypeExpr`
  - already has `DomainConstructorApp { constructor: DomainConstructorId, domain: SealedDomainId, args, kind }`
  - already has `NeutralComputationApp { head: TypeComputationHeadId, args, kind, reason }`

Required SPEC-061 extension:

- Add core-owned source/checked equation carriers rather than encoding source RHS marker constructors as `CanonicalTypeExpr::NominalApp`.
- Prefer adding a dedicated carrier in `type_ir.rs`:
  - `TypeFunctionDef`
  - `TypeFunctionParam`
  - `TypeFunctionEquation`
  - `TypeFunctionPattern`
  - `TypeFunctionResultExpr`
  - source-anchor metadata for header, decreases, case head, pattern variables, RHS heads, and whole rows.
- `TypeFunctionResultExpr` must include a dedicated `DomainConstructorApp { constructor: DomainConstructorId, domain: SealedDomainId, args, kind }` plus `ComputationHeadApp { head: TypeComputationHeadId, args, kind }`.
- If TASK-833 instead extends `CanonicalTypeExpr`, the exact missing variant is `DomainConstructorApp { constructor: DomainConstructorId, domain: SealedDomainId, args: Vec<CanonicalTypeExpr>, kind: Kind }`; all existing consumers of `CanonicalTypeExpr` must then be updated deliberately. Silent use of `NominalApp` is forbidden.

### `crates/ash-core/src/semantic_summary.rs`

Live carriers:

- `pub struct SealedDomainId`
- `pub struct DomainConstructorId`
- `pub struct DomainFieldSummary { kind, domain_constraint, structural_status, ... }`
- `pub struct DomainConstructorSummary`
- `pub struct SealedDomainSummary`
- `pub struct ModuleSemanticSummary`
  - `exported_types`
  - `exported_constructors`
  - `interface_identities`
  - `associated_member_identities`
  - `reserved_identity_slots: ReservedSemanticIdentitySlots`
  - `exported_sealed_domains`
- `pub struct ReservedSemanticIdentitySlots { future_type_functions: Vec<String>, ... }` is a placeholder only.
- `SummaryVersion` currently supports V1 ordinary types and V2 sealed domains.

Required SPEC-061 extension/decision:

- Module-local SPEC-E does not export/import type-function equations. Do not add public equation summary transport in this phase.
- Add local semantic carriers outside `ModuleSemanticSummary`, or add non-exported internal checked carriers in `type_ir.rs`; do not populate `reserved_identity_slots.future_type_functions` as semantic behavior.
- If any public ordinary summary can mention a `TypeComputationHeadId` before SPEC-F, add a summary validation/leakage diagnostic and keep `ModuleSemanticSummary` free of `exported_type_functions` / equation tables until SPEC-F.

## 3. TypeEnv integration seams

Exact seams in `crates/ash-typeck/src/type_env.rs`:

- `pub struct TypeEnv`
  - Currently owns ordinary type maps, canonical identity maps, interface/member registries, sealed-domain registries, and equality forcing points.
  - Add module-local type-function registries here, not in `ash-engine`:
    - visible local name -> provisional/published `TypeComputationHeadId`;
    - head id -> checked `TypeFunctionDef` / equation table;
    - source-order state: provisional self only, earlier validated heads visible, later heads rejected.
- `register_module_semantic_summary_inner` / `pub fn register_module_semantic_summary`
  - Current import path registers ordinary types, interface/member identities, and sealed domains from dependency summaries.
  - SPEC-E must not import type-function equations. Add validation that dependency summaries do not carry public type-function metadata before SPEC-F if such fields appear later.
- `fn validate_summary_visibility_and_duplicates(summary: &ModuleSemanticSummary)`
  - Extend with public computation-head leakage checks if new summary fields or canonical public surface carriers can contain `ComputationHeadApp`.
- Sealed-domain helpers:
  - `fn declare_sealed_domain_identity`
  - `fn validate_and_register_sealed_domain`
  - `pub fn lookup_sealed_domain(&self, name: &str)`
  - `pub fn lookup_sealed_domain_by_id(&self, id: &SealedDomainId)`
  - `pub fn sealed_domain_names(&self)`
  - Type-function checking must consume these for parameter-domain validation, pattern constructor resolution, nested-field structural metadata, and coverage matrix construction.
- Canonical/source type-expression lowering:
  - `pub fn lower_core_type_expr_to_canonical(&self, expr: &TypeExpr)`
  - `pub fn lower_surface_type_to_canonical(&self, ty: &SurfaceType)`
  - These currently lower `Name<...>` only to `CanonicalTypeExpr::NominalApp` after `resolve_type` and have no sealed-domain constructor app branch. Do not use them unchanged for type-function equation RHSs.
  - Add a type-function-specific lowering/resolution seam that knows pattern variables, expected sealed-domain constraints, provisional self head, prior local type-function heads, marker constructors, and nominal heads.
- Equality forcing points:
  - `pub fn unify_types(&self, left: &Type, right: &Type)`
  - `pub fn types_equivalent_for_equality(&self, left: &Type, right: &Type)`
  - `fn definitionally_equal_types_when_canonicalizable(&self, left: &Type, right: &Type)`
  - `fn type_to_canonical_expr_for_equality(&self, ty: &Type)`
  - These call `Normalizer::new(self).definitional_equality(...)` for canonicalizable shapes. TASK-838 must ensure module-local source equations are visible to `Normalizer::new(self)` or an equivalent source-backed registry at these forcing points.

## 4. Normalizer integration seams

Exact seams in `crates/ash-typeck/src/normalizer.rs`:

- Fixture-only carrier stack:
  - `pub enum FixturePattern`
  - `pub struct FixtureDomainConstructorPattern`
  - `pub enum FixtureResultExpr`
  - `pub struct FixtureEquation`
  - `pub struct FixtureEquationRegistry`
  - `pub enum FixtureEquationRegistryError`
- Normalizer API and reduction seams:
  - `pub struct Normalizer<'env>`
  - `pub fn new(env: &'env TypeEnv) -> Self`
  - `pub fn with_registry(env: &'env TypeEnv, registry: FixtureEquationRegistry) -> Self`
  - canonical normalization functions that handle `CanonicalTypeExpr::ComputationHeadApp`
  - fixture equation selection/matching/substitution functions for domain-constructor patterns and results
  - `pub fn definitional_equality(...)`

Required SPEC-061 integration:

- Keep fixture APIs for tests/internal setup, but add source-backed equation registration owned by `TypeEnv` and consumed by `Normalizer::new(self)` or an explicit production registry builder.
- Convert checked `TypeFunctionPattern::DomainConstructor` to the same matching shape as `FixturePattern::DomainConstructor`, preserving ordered row semantics and abstract-scrutinee neutrality.
- Convert checked `TypeFunctionResultExpr::DomainConstructorApp` to `NormalTypeExpr::DomainConstructorApp` during reduction.
- Preserve existing SPEC-060 behavior: known-scrutinee reduction, partial-prefix reduction, open neutral applications with `NormalFormBlockReason::AbstractScrutinee`/neutral blockers, and no inversion.

## 5. Engine public/import boundary seams

Exact seams in `crates/ash-engine/src/module_loader.rs`:

- `pub struct LoadedOrdinaryFile`
  - transports `imported_type_defs`, `imported_semantic_summaries`, and `imported_callables`; no type-function field should be added in SPEC-E.
- `pub fn load_ordinary_file(path: &Path) -> Result<LoadedOrdinaryFile, EngineError>`
  - collects imports and dependency summaries; must not import type-function equations before SPEC-F.
- `fn collect_module_exports(...)` and module export structures around `ModuleExports { type_defs, callables, semantic_summary, child_exports, ... }`
  - This is where public ordinary exports are collected. Add leakage checks before export summary construction/transport: public `type` aliases, exposed public type representations, public `fn`/`builtin fn` signatures, public interface method surfaces, associated-type bindings if public, and workflow/callable summaries must not contain local `ComputationHeadApp` before SPEC-F.
- `collect_module_type_metadata_from_module_file` / `ash_parser::lower::lower_module_type_metadata` call path
  - Currently lowers ordinary types and sealed domains into `ModuleSemanticSummary`. Keep type functions out of exported summaries for SPEC-E.
- Import selections in `load_ordinary_file`
  - glob and named imports pull `exports.semantic_summary`; ensure any future summary type-function fields are rejected/ignored with a SPEC-F handoff diagnostic, not normalized across modules.

Exact seams in `crates/ash-engine/src/lib.rs`:

- `Engine` fields:
  - `imported_semantic_summaries: Mutex<HashMap<u64, Vec<ModuleSemanticSummary>>>`
- Workflow storage/retrieval:
  - `store_imported_semantic_summaries`
  - `get_imported_semantic_summaries`
  - `parse_file`
  - `parse_workflow_source_with_imports`
- Typechecking setup must continue passing imported summaries to `TypeEnv::register_module_semantic_summary`; no source equation table should cross this boundary in SPEC-E.

Public ordinary export leakage before SPEC-F:

- Even though `pub type fn` itself is rejected, leakage can happen when a public ordinary surface references a private local type function in a type expression, e.g. a public alias or public callable signature. The engine/export-summary path must reject such public surfaces before a `ModuleSemanticSummary` is emitted or imported.
- Boundary diagnostic family should map to SPEC-061 `TypeFunctionPublicLeakageDeferred` / `TypeFunctionCrossModuleUnsupported`.

## 6. Source type-expression resolution seams and ambiguity checks

Source type-function declarations need a dedicated resolver instead of existing ordinary-type lowering.

Resolution inputs:

- Raw parser `surface::TypeFnDef` signatures, equations, raw `TypePattern`s, and RHS `surface::Type` expressions.
- Pattern-variable environment for the current equation row.
- Expected type/domain constraint per parameter position or constructor field.
- `TypeEnv` sealed-domain registries and current local type-function registry.
- Ordinary nominal type registry and interface/member projection registry.

Required resolution order for equation RHSs:

1. Pattern variables in the current row.
2. Expected-domain sealed marker constructors using `lookup_sealed_domain` / domain constructor metadata.
3. Current provisional self head.
4. Earlier validated same-module type-function heads.
5. Ordinary nominal type names and associated projections.

Required ambiguity checks:

- Pattern position: a bare name in a sealed-domain slot resolves to that domain's marker constructor if present; if the same name also resolves to another usable type-level head in that position, reject with `TypeFunctionAmbiguousMarkerConstructor`.
- Pattern position: lower-case marker constructors must still resolve as marker constructors under the expected sealed domain; lower-case names bind variables only if they do not resolve to a marker constructor for that domain.
- Pattern position: unconstrained `Type` slots may bind variables/wildcards only; marker-constructor matching is unavailable there, and uppercase/non-variable bare names are rejected in SPEC-E.
- RHS position: if a visible nominal constructor and visible type-function head share a name where both can apply, reject with `TypeFunctionAmbiguousHead`.
- RHS position: if a marker constructor and nominal/type-function head share a name under the expected domain, reject with `TypeFunctionAmbiguousMarkerConstructor`.
- Source order: self references are provisional only inside the current definition; earlier validated same-module type functions are allowed; later same-module heads are `TypeFunctionForwardReferenceUnsupported`; cross-module heads/equations are `TypeFunctionCrossModuleUnsupported` until SPEC-F.

## 7. Diagnostics seam

`crates/ash-typeck/src/error.rs` currently defines `TypeEnvError` and existing associated-type ambiguity diagnostics. TASK-840 should add SPEC-061 diagnostic variants or a type-function diagnostic enum reachable from `TypeEnvError` / `TypeError` for:

- public/export deferral;
- case-head and arity mismatches;
- unknown/wrong-domain marker constructors;
- repeated pattern variables;
- coverage/overlap/default/unreachable diagnostics;
- missing/invalid decreases and non-decreasing recursion;
- result kind/domain mismatch;
- no sealed scrutinee;
- public leakage/cross-module unsupported;
- ambiguous nominal/type-function/marker names;
- forward-reference unsupported.

## 8. Implementation target matrix

- TASK-832: parser dispatch/carriers in `surface.rs` and `parse_module.rs`; keep `parse_type_def.rs` ordinary-type-only.
- TASK-833: core carriers in `type_ir.rs`; optionally summary placeholder validation in `semantic_summary.rs`, but no public equation transport.
- TASK-834/TASK-835: `TypeEnv` local registry, source-order publication, signature/RHS/pattern resolution, kind/domain checks, ambiguity checks, and public leakage checks.
- TASK-836: pattern matrix over checked `TypeFunctionPattern` plus `SealedDomainSummary` constructor/field metadata.
- TASK-837: structural recursion walker over checked `TypeFunctionResultExpr` / canonical children.
- TASK-838: source-backed normalizer equation table integration; fixture APIs remain test-only.
- TASK-839: engine import/export non-interference and public ordinary leakage gate.

## Verification evidence for this audit

- Read and mapped live parser dispatch and AST carriers in `surface.rs` and `parse_module.rs`.
- Read and mapped live core canonical/normal carriers in `type_ir.rs` and semantic-summary carriers in `semantic_summary.rs`.
- Read and mapped live TypeEnv sealed-domain, canonical lowering, import summary, and equality/normalizer forcing seams in `type_env.rs`.
- Read and mapped fixture-only normalizer carriers and APIs in `normalizer.rs`.
- Read and mapped engine summary/import/export storage and module loading seams in `module_loader.rs` and `lib.rs`.
- No Rust files were modified for TASK-831.
