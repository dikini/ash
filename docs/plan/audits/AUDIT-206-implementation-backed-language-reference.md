---
id: audit.206.implementation-backed-language-reference
title: Implementation-Backed Language Reference Census
kind: audit
status: complete
authority: evidence-inventory
owner: language-reference
last_verified: 2026-07-29
---

# AUDIT-206: Implementation-Backed Language Reference Census

## Method and status key

This is a planning census, not a language manual. It records the current source route at the
audited revision. “Accepted” means the active `parse_module::module_file` or expression parser
has a route; a `Definition` enum variant alone does not establish that. `P` means parser/static
acceptance, `L` means lowering, and `R` means admission/runtime. `bounded` means an exact test
fixture or closed Engine bridge, not general execution.

The audit used targeted source/tests because rust-analyzer workspace activation failed. The
canonical implementation source remains code and executable tests; specs, JSON indexes, and old
reference pages are conflict/navigation evidence only.

## Implementation census

| Feature ID / reference topic | Accepted spellings | P / static / L / R status | Code and test evidence | Stale/conflicting documents | Proposed page or exclusion | Gaps / owner |
|---|---|---|---|---|---|---|
| LANG-001 source structure | `.ash` module file; `mod` declarations; comments and ordinary tokens | accepted / module static route partial / summary/lowering partial / no general file execution claim | `parse_module.rs::module_file`; `surface.rs::ModuleFile`; `crates/ash-parser/src/parse_module/tests.rs`; `crates/ash-engine/tests/module_file_check_tests.rs` | `docs/spec/README.md` now routes current source claims to the implementation-backed manual | `lexical-and-modules/source-files-names-and-literals.md` | Documented by TASK-2046 with route-specific source/module/import examples. |
| LANG-002 imports and visibility | module imports and visibility-bearing declarations | accepted / checked summary visibility partial / module-summary lowering partial / runtime import behavior bounded | `ash-engine/src/module_loader/**`; `crates/ash-engine/tests/module_import_resolution_tests.rs`; `task_785_modulefile_summary_exports.rs` | old capability/workflow module material in `docs/reference/surface-to-parser-contract.md` | `lexical-and-modules/modules-imports-and-visibility.md` | Documented by TASK-2046; import resolution is not presented as callable execution. |
| LANG-003 notation and macros | `notation`; expression `macro` declarations/invocations | accepted / syntax/summary checking partial / macro expansion boundaries partial / Engine execution fixture-bounded | parser `module_file`, `surface.rs`; `task_1730_notation_declaration_parser_ast.rs`, `task_1754_macro_declaration_parse.rs`, `task_1758_macro_lowering_boundaries.rs`; Engine macro-boundary tests | top-level `reference/` is old curated corpus, not current grammar authority | `lexical-and-modules/notation-and-expression-macros.md` | Documented by TASK-2046 with hygiene, summary, and execution boundaries. |
| LANG-004 active declarations | `role`, `resource`, `type fn`, `prop`, `data kind`, `type`, `newtype`, `effect`, `sealed`, `interface`, `impl`, `builtin fn`, `handler`, `fn`, `law`, `proof` | parser branches active / static status varies by family / lowering varies / no uniform runtime | exact branches in `parse_module.rs::module_file`; `surface.rs::Definition`; family tests below | `Definition::Capability` and `Definition::Policy` remain AST variants but are not active module parser branches | split across TASK-2047–2051 pages; declaration inventory in `forms/declarations-and-functions.md` | Documented across TASK-2047–TASK-2051; each family retains its own route status. |
| LANG-005 functions and bindings | `fn`, `builtin fn`, function expressions, `let`, blocks, calls | accepted / checked partial / direct Core lowering for selected forms / `fn main` execution bounded | `parse_module/fn_defs.rs`; `parse_expr.rs`; `lower.rs`; `task_959_pure_closure_arrow.rs`; `task_1865_surface_fn_main_entry.rs` | `reference/language/functions/local-and-anonymous.md` claims named local-function behaviour requiring fresh proof; its listed source evidence no longer exists | `forms/declarations-and-functions.md`; `forms/values-bindings-blocks-and-calls.md` | Documented by TASK-2047 with the local/anonymous distinction and example classifications. |
| LANG-006 values, expressions and control | literals, constructors, records, tuples, lists, calls, lambdas, blocks, `if`, `match`, `if let` | accepted / checking partial / selected lowering / execution depends on admitted subset | `parse_expr.rs`; `surface.rs::Expr`; `lower.rs`; parser expression tests; `task_1007_if_let_parser_entrypoints.rs` | old tutorials/examples can contain removed workflow vocabulary | `forms/values-bindings-blocks-and-calls.md`; `forms/control-flow-and-patterns.md` | Documented by TASK-2047 with nonlowerable forms explicitly labelled. |
| LANG-007 patterns | binder and structured patterns, match/if-let sites | accepted / exhaustiveness/type diagnostics partial / lowering varies / bounded runtime evidence | `surface.rs`; `crates/ash-typeck/tests/task_916_pattern_canonicalization_diagnostics.rs`; `ash-cli/tests/task_1008_matching_diagnostics_surface.rs` | older CPS pages describe IR patterns, not necessarily source semantics | `forms/control-flow-and-patterns.md` | Documented by TASK-2047 with the static-versus-runtime split. |
| LANG-008 ordinary types/data/newtypes/callables | `type`, `newtype`, data-kind declarations; callable type/closure spelling | accepted / checked partial / summary/Core routes partial / no general runtime proof | parser type modules; `task_782_modulefile_type_surface.rs`; `task_960_reserved_callable_arrows.rs`; `task_959_pure_closure_arrow.rs` | legacy tower callable pages are historical; old `Fn(...)` source spelling is removed | `types/data-newtypes-and-callables.md` | Documented by TASK-2048; removed arrows stay excluded and constructor execution remains separately bounded. |
| LANG-009 generics, kinds, interfaces and impls | generic binders, kind annotations, `interface`, `impl` | accepted / type evidence partial / summary lowering partial / runtime only selected library/Engine cases | `surface.rs`; `task_910_hkt_diagnostics_surface.rs`; Engine module-loader and stdlib constraint tests | specs describe more target type system than current admitted runtime | `types/generics-kinds-interfaces-and-impls.md` | Documented by TASK-2048 with only the selected checked evidence. |
| LANG-010 type-level computation | `sealed`, `type fn`, `prop`, `data kind`, associated-family/proposition type syntax | accepted for listed declarations / checked partial / summaries/normalization partial / not a general source execution route | `task_813_sealed_domain_diagnostics.rs`; `task_846_public_type_fn_visibility.rs`; `task_881_proposition_parse_diagnostics.rs`; typeck diagnostics tests | `dtype` appears in planning vocabulary but has no active parser spelling | `types/type-level-domains-functions-families-and-propositions.md`; exclude `dtype` | Documented by TASK-2049 with exact syntax and static-semantic limits. |
| LANG-011 rows and effect declarations | `effect` aliases/groups and callable row annotations | accepted / checked resolution partial / Core metadata partial / rows never grant R authority | `task_2001_effect_alias_group_surface.rs`; `task_1809_computation_row_parser.rs`; `task_1814_row_cross_boundary_non_authority.rs`; `task_1822_row_authority_neutrality.rs` | old capability-provider/tower material overstates rows as authority | `effects/rows-aliases-groups-and-operations.md` | Documented by TASK-2050 with non-authority and alias/group limits. |
| LANG-012 declared operations, resources and roles | operation identities through interfaces/impls; `resource`; `role` | resource/role parser branches accepted / static and summary routes partial / admission is separate / no direct source grant | `parse_module.rs`; `surface.rs`; `task_1810`-family evidence; row admission tests | top-level `capability` and `policy` declarations are AST-only/removed source forms | `effects/resources-roles-and-authority-boundaries.md`; exclude direct capability/policy declarations | Documented by TASK-2050 with operation identity and resource/role authority boundaries. |
| LANG-013 handlers and failure | `handler`; `on computation { ... }`; `handle expression with name`; scoped error form | accepted / handler facts checked partial / typed Core/CPS lower partial / exact closed cases admitted, otherwise closed | `parse_expr.rs`; `surface.rs::Expr::{On,HandleWith}`; typeck handler tests; `task_2013_handler_core_lowering.rs`; `task_2014_handler_production_admission.rs` | legacy `Act`/`Proc` descriptions and CPS reference pages cannot establish source runtime coverage | `effects/handlers-failure-and-do.md` | Documented by TASK-2051 with fixture-bounded deep-affine/continuation evidence. |
| LANG-014 `do` and comprehensions | `do { ... }`, target-annotated do targets, `[result | qualifiers]` | accepted / typed-do evidence partial / ambient do lowers; generic do and comprehensions reject ordinary lowering / runtime bounded | `parse_expr.rs::parse_do_block_expr`; `lower.rs` generic-do/comprehension rejections; `task_1024_do_and_comprehension_stdlib_evidence.rs`; Engine `task_1024_stdlib_do_evidence.rs` | historic `do:Act`/`do:Proc`/`do:Workflow` examples are removed | `effects/handlers-failure-and-do.md`; `effects/comprehensions.md` | Documented by TASK-2051 with ambient versus generic/static boundaries. |
| LANG-015 laws and proofs | `law`, `proof` | parser accepted / evidence machinery partial / no ordinary program runtime meaning | `parse_module.rs`; `task_1361_law_keyword_module_scope.rs`; `task_1363_proof_keyword_module_scope.rs` | target proof/design documents may overstate proof status | `forms/declarations-and-functions.md` | Documented by TASK-2047 as authoring-only, not runner/runtime coverage. |
| LANG-016 entry, admission and clients | `fn main`; CLI run/test, REPL, daemon descriptors | accepted / checking/lowering partial / Engine-only admitted subset / four-client parity narrow and terminal-oriented | `ash-engine/src/lib.rs::{execute_admitted_program,execute}`; `task_1865_surface_fn_main_entry.rs`; CLI terminal/daemon tests; task records 2032/2038/2039/2042 | legacy workflow entry documentation is stale routing | `execution/entry-lowering-and-admission.md`; `execution/clients-terminals-and-diagnostics.md` | Documented by TASK-2052; generic APIs remain fail-closed and no evaluator fallback is claimed. |
| LANG-017 standard library | public modules/declarations imported from `std/src/**` | parser/import evidence exists / static evidence partial / selected Engine e2e only / not blanket runnable proof | `crates/ash-cli/tests/stdlib_corpus_check.rs`; parser `stdlib_parsing.rs`; Engine stdlib e2e tests | historical stdlib Act/Proc/Workflow pages are retained only for old links | `library/index.md`; `library/modules-and-imports.md` | Documented by TASK-2053: 59-file corpus, loader/static evidence, registry limits, and one selected runtime witness. |
| LANG-018 diagnostics and limitations | parser/type/Engine/CLI diagnostics | accepted diagnostic surfaces / checked partial / runtime classification bounded | `ash-typeck/src/diagnostic.rs`; `ash-cli/tests/check_parse_diagnostics.rs`; terminal tests | legacy reference frontmatter has fourteen dangling evidence paths; disposition recorded below | `execution/clients-terminals-and-diagnostics.md`; `library/diagnostics-and-errors.md` | Documented by TASK-2053; TASK-2054 completed the stale-reference disposition without rewriting the legacy corpus. |
| LANG-019 function and handler contracts | `requires:` and `ensures:` clauses on current `fn` and `handler` declarations | accepted / contract static/lowering evidence partial / Engine contract behaviour selected/bounded | `crates/ash-parser/src/parse_module/fn_defs.rs:47-48,219-260`; `crates/ash-parser/tests/fn_parser_tests/contracts_and_types.rs`; `crates/ash-engine/tests/function_contracts_integration.rs` | workflow-header contracts are removed and must not be used to explain current functions | `forms/declarations-and-functions.md` | Documented by TASK-2047 with grammar, static/lowering, and execution limits. |
| LANG-020 capability type spelling | `capability Name` in a source type position | accepted / lowered to an operational capability type / selected Engine entry validation/binding / not a top-level declaration or authority grant | `crates/ash-parser/src/parse_module.rs:2745-2753`; `crates/ash-typeck/src/surface_type_lowering.rs:209-212`; `crates/ash-engine/src/entry.rs:359-375`; `crates/ash-parser/tests/stdlib_parsing/runtime.rs:100-115` | historical top-level capability definitions and direct authority syntax are distinct and excluded | `types/data-newtypes-and-callables.md`, cross-link `effects/resources-roles-and-authority-boundaries.md` | Documented by TASK-2048 with TASK-2050's non-authority cross-link. |
| LANG-021 full computation-row grammar | operation paths; `resource`, `role`, `policy`, `channel`, `process`, `fail`, `evidence`, `group`; whole-row variables and tails; resource/channel modes | accepted / row normalization/handler residual checking partial / Core metadata partial / rows never grant runtime authority | `crates/ash-parser/src/parse_module.rs:900-1116`; `crates/ash-parser/src/surface.rs:535-625`; `crates/ash-parser/tests/task_1809_computation_row_parser.rs`; `crates/ash-typeck/tests/task_2013_handler_row_typing.rs:375-464` | older references reduce rows to aliases/operations or imply authority | `effects/rows-aliases-groups-and-operations.md` | Documented by TASK-2050 with every listed family/mode and its static/runtime gaps. |
| LANG-022 source failure and scoped failure handling | `fail payload`; `with_error { body } handle { pattern => expression; ... }` | accepted / type checking partial / parser lowers to Core failure carriers / general admitted runtime not established | `crates/ash-parser/src/parse_expr.rs:589-625,1496-1504`; `crates/ash-parser/tests/task_708_fail_with_error.rs`; `crates/ash-typeck/tests/task_708_operational_bottom.rs` | old tower operational-bottom explanations are not current source authority | `effects/handlers-failure-and-do.md` | Documented by TASK-2051 with syntax, static/lowering evidence, and runtime limitation. |
| LANG-023 obligation checking expression | `check obligation_name` | accepted / static and runtime semantics require fresh route audit / lowering/runtime not assumed | `crates/ash-parser/src/parse_expr.rs:1402-1478`; `crates/ash-engine/tests/function_contracts_integration.rs` | workflow-obligation material is historical routing only | `forms/control-flow-and-patterns.md` | Documented by TASK-2047 with the precise rejected static/closed runtime classification. |
| LANG-024 operator sections | parser-recognized parenthesized operator sections | accepted / elaboration required before lowering / unresolved sections rejected; selected macro/notation boundary evidence | `crates/ash-parser/src/parse_expr.rs:1402-1478`; `crates/ash-parser/tests/task_1724_operator_section_boundary.rs`; `task_1733_operator_section_elaboration.rs`; `task_1725_expanded_surface_boundary.rs` | notation/macro pages cannot imply arbitrary execution | `lexical-and-modules/notation-and-expression-macros.md` | Documented by TASK-2046 with syntax, elaboration diagnostics, and status. |

## Planned and target-only conclusion

No target-only/planned source-language feature is known at the audited revision. Every current
manual topic is either a live route with its own implemented/partial/closed status or an explicit
exclusion. A target specification may describe a desired future state, but it is not a manual
feature until it has a target-only register entry, explicit `planned` label, and target authority.
In particular, a `partial`, `below_spec`, bounded, or closed live route is not reclassified as a
future feature.

## Exclusion register

These are never current-language reference examples. They may occur in this planning audit only as
plain names and exclusions.

| Excluded item | Evidence and disposition |
|---|---|
| workflow declarations, headers, proxies, yield/receive workflow carriers | TASK-1971 records parser/AST/lowering removal; Phase 201 gate rejects re-entry. |
| public Act/Proc/Workflow tower carrier syntax, `do:Act`, `do:Proc`, `do:Workflow`, old callable arrows | Removed source forms; reserved-arrow parser test and Phase 201 gate are the boundary. |
| top-level `capability` and `policy` declarations | `Definition` still has variants, but `module_file` has no active branches; treat as AST/internal historical carrier unless reintroduced by parser evidence. |
| source `raise` | No active source parser production. `Raise` belongs to Core/CPS semantics and is not a source-language chapter. |
| `dtype` | No active source parser implementation. Document `type`, `newtype`, and `data kind` separately. |
| `par`, direct capability/provider grant declarations, old observe/act-with forms, workflow-specific contract helpers | Phase 201 removed-form policy; exclude. |
| Core/CPS `Raise`, frames, private Engine helper APIs | Internal-only unless a separate accepted source route is evidenced. |

## Documentation conflicts and placement reconciliation

1. `docs/reference/surface-to-parser-contract.md` retains workflow/policy-era surface claims and
   remains a cross-cutting planning contract, not current grammar authority.
2. `reference/language/functions/local-and-anonymous.md` retains a named-local-function claim
   whose listed `ash-interp` evidence paths no longer exist. The current manual supplies fresh
   implementation/test evidence where available and does not adopt the historical claim otherwise.
3. `docs/spec/README.md` now explicitly routes current source-language claims to
   `docs/reference/language/` and its live-code evidence order. `SPEC-INDEX.md` remains routing
   for current-state compatibility and target rules.
4. `reference/` validation still reports fourteen dangling evidence paths. TASK-2054 records this
   legacy-corpus health disposition only: it is metadata drift, not proof that every page is
   semantically false, and no legacy page was rewritten as part of this manual.
5. The narrow SPEC-071 §3.1 exception remains the placement authority for
   `docs/reference/language/`; neither `docs/reference/` nor top-level `reference/` decides a
   current source claim.

## Handoff

TASK-2044 supplied this census to TASK-2045 through TASK-2054. The completed manual links every
owned topic from `docs/reference/language/index.md`; its task pages record reviewed evidence,
examples, test commands, and implementation/evidence/parity axes. A missing lowering or admission
path remains a documented limitation, never a reason to widen the manual's claim.
