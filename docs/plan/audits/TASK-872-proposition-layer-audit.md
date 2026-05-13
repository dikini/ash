# TASK-872 proposition-layer audit gate

Status: complete for pre-implementation binding. This audit intentionally changes no Rust behavior.

Scope: SPEC-064 / PLAN-112 / DESIGN-034 §16.8, live worktree `/home/dikini/Projects/ash/.worktrees/phase-116-constraint-proposition`.

Hard gate statement: no TASK-873+ Rust implementation starts until this binding table is complete and TASK-873 through TASK-882 have had their intentional failing verification guards replaced by exact non-zero focused test commands. This audit completes that precondition for planning only; downstream tasks still own their Rust implementation and tests.

## Live owner map

| Layer | Owner | Current live responsibility | Proposition-layer ownership decision |
| --- | --- | --- | --- |
| Parser surface | `crates/ash-parser/src/surface.rs`, `parse_module.rs`, `parse_type_def.rs`, `parse_expr.rs`, `lower.rs` | Raw syntax for module definitions, type expressions, `type fn`, `fn`/`builtin fn`, interface/impl where bounds, workflow/runtime constraints and fn contracts. | TASK-874 owns raw proposition syntax only. Parser must preserve spans and names but must not resolve proposition semantics. |
| Core IR/summary | `crates/ash-core/src/type_ir.rs`, `semantic_summary.rs`, `ast.rs`, `workflow_contract.rs` | Shared canonical type IR, normal forms, type-function/family carriers, semantic summaries through V4, legacy AST interface/where carriers, runtime workflow contracts. | TASK-873 owns stable proposition carriers and V5 summary schema. Core owns boundary facts, not solving. |
| Type checker | `crates/ash-typeck/src/type_env.rs`, `normalizer.rs`, `error.rs`, `diagnostic.rs` | TypeEnv registrations, impl/family/type-function tables, canonical lowering, normalizer/definitional equality, diagnostics. | TASK-875..TASK-878/TASK-880/TASK-881 own proposition environment, solver, registration, checking integration, and diagnostics. |
| Engine/module transport | `crates/ash-engine/src/lib.rs`, `module_loader.rs` | Loads/imports modules, stores imported summaries and visible type-function heads, transports type/family summaries. | TASK-879/TASK-880 own only summary transport/storage callsites needed for public propositions; no engine solving. |
| Workflow/runtime constraints | `ash-core::workflow_contract`, parser lowerer, runtime/capability AST constraints | Runtime/value-level postconditions, arithmetic predicates, capability/provider constraints. | Non-overlap: no TASK-873+ may route runtime workflow/capability predicates into type-level proposition solving. |

## Current carriers and gaps

| Audit ID | File(s) | Live carrier/callsite | Gap for SPEC-064 | Downstream owner |
| --- | --- | --- | --- | --- |
| H-AUD-CORE-01 | `crates/ash-core/src/type_ir.rs` lines 133-154, 458-497 | `CanonicalTypeExpr` has Primitive/Var/NominalApp/Projection/ComputationHeadApp; `NormalTypeExpr` has typed `DomainConstructorApp`, neutral computation apps, and projections. | Equality/disequality proposition operands need a stable term carrier that can represent sealed-domain constructor apps without pretending `CanonicalTypeExpr` can. | TASK-873 |
| H-AUD-CORE-02 | `crates/ash-core/src/type_ir.rs` lines 245-270, 375-399 | Type-function and associated-family result expressions already have `DomainConstructorApp`. | Proposition carriers should reuse/bridge the domain-constructor identity model, not encode constructor heads as strings or nominal ADTs. | TASK-873, TASK-876 |
| H-AUD-CORE-03 | `crates/ash-core/src/semantic_summary.rs` lines 549-568, 818+ | Summary versions stop at `SPEC063_ASSOCIATED_FAMILY_V4`; validation rejects type-function/family facts before V3/V4 and future versions. | Add V5 proposition facts and fail-closed validation for proposition payloads in older summaries. | TASK-873, TASK-879 |
| H-AUD-CORE-04 | `crates/ash-core/src/ast.rs` lines 251-260, 823-828, 830+ | AST has `TypeParam.bounds`, `InterfaceBound`, and `WhereBound { param, bound }`; interface/impl definitions keep SPEC-035 carriers. | These are interface-bound inputs only; they are not generalized proposition facts yet. | TASK-873, TASK-875, TASK-877 |
| H-AUD-CORE-05 | `crates/ash-core/src/workflow_contract.rs`, `crates/ash-core/src/ast.rs` lines 593-626 | Workflow/capability constraints use value-level predicates and runtime contracts. | Must remain separate from type-level propositions and summary facts. | TASK-882 non-interference |
| H-AUD-PARSE-01 | `crates/ash-parser/src/surface.rs` lines 128-146, 447-480 | `TypeFnDef`, `FnDef`, and `BuiltinFnDef` have no proposition-clause field. | Parse `where` proposition tails on `type fn`, `fn`, and `builtin fn`; preserve spans. | TASK-874 |
| H-AUD-PARSE-02 | `crates/ash-parser/src/surface.rs` lines 650-657, 663-752 | Existing `WhereBound` is only `T: Interface` for impl/interface behavior. | Keep legacy behavior; generalized proposition tails require new raw carriers, not mutation of old where-bound meaning unless explicitly bridged. | TASK-874, TASK-877 |
| H-AUD-PARSE-03 | `crates/ash-parser/src/parse_module.rs` lines 194-210, 781-784, 1087-1214 | Module dispatch handles interface/impl/builtin/fn; capability definitions have unrelated `where` constraints. | Add `prop` declaration dispatch and proposition-tail parsers without confusing capability constraint grammar. | TASK-874, TASK-878 |
| H-AUD-PARSE-04 | `crates/ash-parser/src/parse_type_def.rs` lines 80-95, 565-588 | Type parser carries associated/family projections but no proposition expression grammar. | Proposition operands must parse type expressions and operators `==`, `!=`, `:` and named predicate calls with spans. | TASK-874 |
| H-AUD-PARSE-05 | `crates/ash-parser/src/lower.rs` lines 262-350, 565-592, 1597+ | Lowerer maps fn contracts and capability constraints to runtime/core value contracts; lowers interface/impl AST. | Raw proposition pass-through must not enter `workflow_contract` or runtime capability constraints. | TASK-874, TASK-882 |
| H-AUD-TYPECK-01 | `crates/ash-typeck/src/type_env.rs` lines 630-655, 2035-2081 | `WhereBound`, `ImplScheme.where_bounds`, `TypeEnv.impls`, and `type_var_interface_bounds` carry interface-bound evidence. | Proposition environment must ingest these as typed assumptions/evidence separately from ordinary impl search. | TASK-875, TASK-877 |
| H-AUD-TYPECK-02 | `crates/ash-typeck/src/type_env.rs` lines 792-805, 1035-1041 | Associated projections resolve from type-var bounds through `resolve_associated_interface_from_type_var_bounds`. | Rigid where-bound projection evidence must remain a projection-selection boundary, not an interface-bound solver shortcut. | TASK-877, TASK-880 |
| H-AUD-TYPECK-03 | `crates/ash-typeck/src/type_env.rs` lines 11124-11227, 11430-11437 | `register_impl` validates impls, overlap, where-bounds, and inserts bounds into a cloned env for associated bindings. | Concrete impl evidence can satisfy interface-bound propositions only at exact known sites; missing evidence defers/errors without broad search. | TASK-877 |
| H-AUD-TYPECK-04 | `crates/ash-typeck/src/normalizer.rs` lines 427-446, 545-565 | `DefinitionalEqualityResult` is `Equal`, closed `NotEqual`, or `BlockedByNeutrality` with no-inversion note. | Equality propositions must accept only `Equal`, refute only closed `NotEqual`, and defer blockers. | TASK-876 |
| H-AUD-TYPECK-05 | `crates/ash-typeck/src/normalizer.rs` lines 587-615 | `require_concrete_normal_form` distinguishes concrete data forms from neutral/projection/unavailable reductions. | Disequality solver needs a structural normal-form helper for sealed-domain constructor-head disjointness and open/neutral deferral. | TASK-876 |
| H-AUD-TYPECK-06 | `crates/ash-typeck/src/error.rs` lines 171-392, `diagnostic.rs` lines 55-121 | Existing TypeEnv errors/codes cover summary versions, malformed computation/family summaries, private dependency failures, associated-family errors. | Add proposition-specific error variants and stable codes without reusing associated-family success/failure claims. | TASK-881 |
| H-AUD-TYPECK-07 | `crates/ash-typeck/src/type_env.rs` import/export summary callsites | TypeEnv imports/exports type-function and associated-family summaries and validates versions. | Public proposition requirements/evidence need V5 export/import and private-dependency checks. | TASK-879 |
| H-AUD-ENGINE-01 | `crates/ash-engine/src/lib.rs` lines 79-85, 727-822 | Engine stores imported `ModuleSemanticSummary` and source-visible type-function heads by workflow id. | Add no-solving transport/storage for V5 proposition summaries only if TypeEnv needs them at check time. | TASK-879, TASK-880 |
| H-AUD-ENGINE-02 | `crates/ash-engine/src/module_loader.rs` lines 78-94, 216-233, 315-354 | Loader collects imported summaries, type-function heads, associated-family summaries, callables, named/glob/pub-use metadata. | Transport proposition payloads in `ModuleSemanticSummary` along named/glob/pub-use paths without creating engine semantics. | TASK-879 |
| H-AUD-NONINT-01 | `crates/ash-parser/src/lower.rs`, `crates/ash-core/src/workflow_contract.rs`, runtime/capability tests | Runtime constraints already have separate Stage 1 arithmetic and capability/provider semantics. | SPEC-H propositions must never decide runtime workflow predicates, capability policy, provider access, or operational checks. | TASK-882 |

## Live call graph for proposition-relevant seams

1. Source parsing:
   - Module files dispatch in `parse_module.rs`: `Definition::TypeFn`, `Interface`, `Impl`, `BuiltinFn`, `Function`.
   - Type expressions parse through `parse_type_def.rs` and are converted into `surface::Type` in `parse_module.rs`.
   - Existing workflow/capability/fn-contract parsing is separate and must stay separate.
2. Parser lower/core handoff:
   - `lower.rs` lowers runtime fn contracts to `ash_core::workflow_contract` and capability constraints to `ash_core::Constraint`.
   - `lower_interface_def`/impl lowering preserve core interface/where carriers.
   - New proposition clauses must be raw parser data until TypeEnv canonical lowering.
3. TypeEnv registration/checking:
   - Interface and impl metadata feed `TypeEnv.interfaces`, `impls`, and `type_var_interface_bounds`.
   - Source/imported type computation and associated-family facts register into TypeEnv before normalizer use.
   - Proposition environment should be introduced in TypeEnv, keyed by source/generator site IDs, and not by parser structs.
4. Normalization/equality:
   - `Normalizer::normalize` produces `NormalTypeExpr` with neutral/projection blockers.
   - `Normalizer::definitional_equality` returns `DefinitionalEqualityResult` and explicitly refuses inversion/output solving.
   - Proposition equality/disequality must wrap this evidence conservatively.
5. Summary/engine transport:
   - TypeEnv produces public summaries; engine loader stores/transports `ModuleSemanticSummary` through imports and re-exports.
   - V5 proposition payloads belong in core summaries; engine only transports, TypeEnv validates/uses.

## Proposition solving and forcing matrix

| Site ID | Future site | Proposition action | Must force? | Must not do | Owner |
| --- | --- | --- | --- | --- | --- |
| H-FORCE-01 | Core stable proposition and outcome carriers | Define canonical equality/disequality/interface/named-predicate operands, evidence/refutation/deferred carriers, source anchors, V5 payloads. | No solving. | No strings/debug-only facts; no `CanonicalTypeExpr` lie for domain constructors. | TASK-873 |
| H-FORCE-02 | Parser proposition tails and `prop` declarations | Parse raw clauses/spans on `type fn`, `fn`, `builtin fn`; parse explicit named predicates. | Parser errors only for malformed/unsupported surfaces. | No semantic resolution; no TypeEnv identities. | TASK-874 |
| H-FORCE-03 | TypeEnv proposition environment | Canonically lower raw propositions, record assumptions/obligations/provenance. | Classification/registration only. | No equality/disequality discharge. | TASK-875 |
| H-FORCE-04 | Normalized equality propositions | Use `Normalizer::definitional_equality` and normal-form structural disequality. | Yes at solver callsites. | No inversion, no substitution/meta solving, no associated-family output solving. | TASK-876 |
| H-FORCE-05 | Interface-bound propositions | Consult exact in-scope where-bound facts and known concrete impl evidence. | Yes for required interface-bound obligations. | No broad impl search; no family selection side effects. | TASK-877 |
| H-FORCE-06 | Named predicates | Register predicate identities and lower uses; builtin predicates may have explicit TypeEnv-known behavior. | Defer arbitrary supported-but-unimplemented predicates; error unknown names. | No unrestricted SMT/proof search. | TASK-878 |
| H-FORCE-07 | Public summaries/imports | Export/import public proposition requirements and optional evidence in V5. | Validate before use; reject private leaks. | No engine solving; no V4 proposition payloads. | TASK-879 |
| H-FORCE-08 | Checking entrypoints | Generate/discharge proposition obligations at audited public signature/type-function/impl/fn/builtin sites. | Required sites fail on refuted/deferred obligations. | No parser-owned semantics; no inference-meta solving. | TASK-880 |
| H-FORCE-09 | User diagnostics | Emit structured proposition diagnostics/codes/spans/fixes. | Yes at all user-facing failure/defer sites. | Do not claim unsupported proof search succeeded. | TASK-881 |
| H-FORCE-10 | Acceptance and non-interference | Map SPEC-064 H1-H12 to focused tests and existing regressions. | Yes, non-zero evidence only. | No zero-test or placeholder passes. | TASK-882 |

## Summary seams and version policy

| Audit ID | Current state | Required binding |
| --- | --- | --- |
| H-SUM-01 | `SummaryVersion` currently has V1 ordinary, V2 sealed domains, V3 type computation, V4 associated families. | TASK-873 adds `SPEC064_PROPOSITION_V5`; TASK-879 validates V5 import/export. |
| H-SUM-02 | Validation rejects type-function/family payloads before their versions and future unsupported versions. | TASK-873/TASK-879 add analogous proposition-payload rejection for V1-V4 and unsupported future versions. |
| H-SUM-03 | Engine loader carries summaries as opaque core structs and keeps source-visible selection maps separate. | TASK-879 extends transport only; TypeEnv remains semantic owner. |
| H-SUM-04 | Private dependency export failure exists for type computations/families. | TASK-879/TASK-881 add proposition private-leak failures for private type functions, domains, families, predicates. |

## Diagnostics and non-interference risks

| Risk ID | Risk | Required test owner |
| --- | --- | --- |
| H-RISK-01 | Proposition equality accidentally uses legacy unification and solves variables from outputs. | TASK-876, TASK-880 |
| H-RISK-02 | Interface-bound solver broadens impl search or changes associated-family selection. | TASK-877 plus existing `task_864_rigid_where_bound_projection` regression. |
| H-RISK-03 | Parser `where` proposition syntax conflicts with capability constraints or legacy impl where-bounds. | TASK-874, TASK-878, TASK-882 |
| H-RISK-04 | V5 proposition summaries leak private helper facts or are accepted by V4 validation. | TASK-873, TASK-879, TASK-881 |
| H-RISK-05 | Engine starts interpreting proposition facts instead of transporting summaries. | TASK-879, TASK-880, TASK-882 |
| H-RISK-06 | Runtime workflow/capability constraints get merged with type-level propositions. | TASK-882 |

## Downstream binding table

| Task | Source files | Test targets | Callsite/audit-row IDs | Task-file action |
| --- | --- | --- | --- | --- |
| TASK-873 | `crates/ash-core/src/type_ir.rs`; `crates/ash-core/src/semantic_summary.rs`; `crates/ash-core/src/lib.rs` for public proposition exports. | `crates/ash-core/tests/task_873_proposition_carriers.rs` | H-AUD-CORE-01..05, H-SUM-01..02, H-FORCE-01 | Bound exact files/tests; guard replaced. |
| TASK-874 | `crates/ash-parser/src/surface.rs`; `parse_module.rs`; `parse_type_def.rs`; `parse_expr.rs`; `lower.rs`. | `crates/ash-parser/tests/task_874_proposition_surface.rs` | H-AUD-PARSE-01..05, H-FORCE-02, H-RISK-03 | Bound exact files/tests; guard replaced. |
| TASK-875 | `crates/ash-typeck/src/type_env.rs`; `error.rs`; `diagnostic.rs`. | `crates/ash-typeck/tests/task_875_proposition_environment.rs` | H-AUD-TYPECK-01..03, H-FORCE-03 | Bound exact files/tests; guard replaced. |
| TASK-876 | `crates/ash-typeck/src/type_env.rs`; `normalizer.rs`; `error.rs`; `diagnostic.rs`. | `crates/ash-typeck/tests/task_876_proposition_solver.rs` | H-AUD-TYPECK-04..05, H-FORCE-04, H-RISK-01 | Bound exact files/tests; guard replaced. |
| TASK-877 | `crates/ash-typeck/src/type_env.rs`; `error.rs`; `diagnostic.rs`. | `crates/ash-typeck/tests/task_877_interface_bound_propositions.rs`; regression `crates/ash-typeck/tests/task_864_rigid_where_bound_projection.rs` | H-AUD-TYPECK-01..03, H-FORCE-05, H-RISK-02 | Bound exact files/tests; guard replaced. |
| TASK-878 | `crates/ash-parser/src/surface.rs`; `parse_module.rs`; `parse_type_def.rs`; `crates/ash-typeck/src/type_env.rs`; `error.rs`; `diagnostic.rs`. | `crates/ash-parser/tests/task_878_named_predicate_surface.rs`; `crates/ash-typeck/tests/task_878_named_predicate_registration.rs` | H-AUD-PARSE-01, H-AUD-PARSE-03..04, H-AUD-TYPECK-06, H-FORCE-06 | Bound exact files/tests; guard replaced. |
| TASK-879 | `crates/ash-core/src/semantic_summary.rs`; `crates/ash-typeck/src/type_env.rs`; `crates/ash-engine/src/lib.rs`; `crates/ash-engine/src/module_loader.rs`. | `crates/ash-core/tests/task_879_proposition_summary_schema.rs`; `crates/ash-typeck/tests/task_879_proposition_summary_import.rs`; `crates/ash-engine/tests/task_879_proposition_summary_transport.rs` | H-AUD-CORE-03, H-AUD-TYPECK-07, H-AUD-ENGINE-01..02, H-SUM-01..04, H-FORCE-07 | Bound exact files/tests; guard replaced. |
| TASK-880 | `crates/ash-typeck/src/type_env.rs`; `crates/ash-typeck/src/error.rs`; `crates/ash-typeck/src/diagnostic.rs`; inspect `crates/ash-typeck/src/normalizer.rs` without adding reduction behavior; `crates/ash-engine/src/lib.rs` for imported-summary handoff only. | `crates/ash-typeck/tests/task_880_proposition_checking_points.rs`; `crates/ash-engine/tests/task_880_proposition_public_integration.rs` | H-FORCE-08, H-RISK-01, H-RISK-02, H-RISK-03, H-RISK-04, H-RISK-05, H-AUD-TYPECK-01, H-AUD-TYPECK-04, H-AUD-TYPECK-07, H-AUD-ENGINE-01 | Bound exact files/tests; guard replaced. |
| TASK-881 | `crates/ash-typeck/src/error.rs`; `diagnostic.rs`; parser diagnostics in `crates/ash-parser/src/parse_module.rs`/`parse_type_def.rs` only for unsupported-surface parse errors. | `crates/ash-typeck/tests/task_881_proposition_diagnostics.rs`; `crates/ash-parser/tests/task_881_proposition_parse_diagnostics.rs` | H-AUD-TYPECK-06, H-FORCE-09, H-RISK-04 | Bound exact files/tests; guard replaced. |
| TASK-882 | `docs/plan/audits/TASK-882-proposition-acceptance-matrix.md`; focused aggregator tests only. | `crates/ash-core/tests/task_882_spec_h_summary_non_interference.rs`; `crates/ash-parser/tests/task_882_spec_h_surface_non_interference.rs`; `crates/ash-typeck/tests/task_882_spec_h_acceptance_matrix.rs`; `crates/ash-engine/tests/task_882_spec_h_transport_non_interference.rs` | H-FORCE-10, H-RISK-01..06, H-AUD-NONINT-01 | Bound exact files/tests; guard replaced. |

## Existing regression suites TASK-882 must cite or run

- SPEC-035/SPEC-063 associated behavior: `cargo test -p ash-typeck --test task_864_rigid_where_bound_projection`, `task_866_associated_family_normalizer`, `task_867_associated_family_import`, `task_870_associated_family_public_lowering`.
- SPEC-060 normalizer/equality: `cargo test -p ash-typeck --test task_824_definitional_equality`, `task_825_non_inverting_unification_boundary`, `task_827_normalizer_diagnostics`.
- SPEC-057/058/059/061/062 summary/type pipeline: `task_787_semantic_summary_typeenv`, `task_798_canonical_lowering_typeenv_registry_red`, `task_812_domain_registration_validation`, `task_840_type_function_acceptance`, `task_854_type_computation_summary_acceptance`.
- Engine transport non-interference: `cargo test -p ash-engine --test task_867_associated_family_summary_transport`, `task_870_associated_family_public_lowering`, `task_854_type_computation_summary_acceptance`.

## Pre-implementation verification command pattern

Each downstream task now has fail-closed focused commands of this form:

```sh
test -f <exact future test file>
cargo test -p <crate> --test <test_target> -- --list | grep -q task_<task>_
cargo test -p <crate> --test <test_target>
```

The `test -f` and `-- --list | grep -q` guards intentionally fail before the task creates real tests and prevent zero-test/self-satisfying verification.
