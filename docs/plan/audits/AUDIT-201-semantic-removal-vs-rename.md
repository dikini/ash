# AUDIT-201: Semantic Removal Vs Rename

**Status:** Complete for TASK-1969.
**Owner:** TASK-1969.
**Phase:** [PLAN-201: Deprecated Functionality Removal](../PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md).
**Follow-up plan:** [PLAN-201 Semantic Cleanup Follow-up](../PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md).

## Scope

This audit reviews Phase 201 cleanup for the risk that old workflow, tower, capability, or
compatibility mechanisms were only renamed to target-shaped vocabulary. Passing stale-token gates
is necessary but not sufficient: retained mechanisms must either be deleted, refactored into target
Ash primitives, or justified as implementation details by current target specs and tests.

Target Ash authority for this audit:

- `SPEC-095b` target grammar: current surface code is ordinary modules, functions, target
  callable types, target do forms, and current declarations.
- `SPEC-096b` target effect system: effects are rows/profiles, not public tower wrapper
  distinctions.
- `SPEC-097b` target type system: target callable/effect-row typing replaces public
  Act/Proc/Workflow carrier typing.
- `SPEC-098c` surface-to-Core lowering: target surface families lower through current Core/CPS
  carriers, not compatibility syntax.
- Phase 196 application runtime: runtime reports may keep application/entry identifiers, but must
  not teach a separate workflow language.
- Phase 198/199 libraries/templates and Phase 200 tooling: productive examples, templates, docs,
  formatter, LSP, and diagnostics are target-only.

## Decisions

| Decision | Meaning |
|----------|---------|
| Delete now | The mechanism is stale functionality and no target Ash path depends on it. |
| Refactor to target primitive | The mechanism is useful only after folding it into functions, rows, provider profiles, contracts/evidence, process/channel primitives, or application reports. |
| Keep as implementation detail | The mechanism is target-justified, hidden from user-facing Ash, and covered by a test/gate. |
| Plan required | The audit cannot safely delete it in this phase slice; TASK-1970 owns the concrete cleanup plan. |

## Phase 201 Slice Review

| Slice/task | Changed files | Claimed cleanup | Removed behavior proof | Rename-only risk | Decision |
|------------|---------------|-----------------|------------------------|------------------|----------|
| TASK-1961 Ash artifact cleanup | `std/`, `examples/`, `templates/`, `tests/`, active fixtures | Removed repository Ash code using old workflow/tower/capability forms. | Phase 201 gate scans active Ash artifacts and Rust literals; stdlib corpus passes with zero expected failures. | Low: deleted files and target rewrites are observable behavior removal. | Keep gate; no semantic follow-up beyond stale-form sweeps. |
| TASK-1962 parser/checker removal | parser, CLI, engine rejection tests | Removed parser acceptance for old capability declarations, old act block `ret`, removed callable spellings, and old workflow headers. | Focused parser/engine removed-syntax tests plus Phase 201 gate. | Medium: some retained `WorkflowDef`/contract structures can mask old semantics if they accept only current contracts by construction. | Plan required for remaining parser/lowering carrier audit. |
| TASK-1963 surface/lowering carriers | parser lowerers, engine metadata, lint/REPL/LSP symbol paths | Removed obsolete surface carriers and retargeted current metadata vocabulary. | Gate blocks removed carrier names in selected active paths; focused parser/engine/LSP tests pass. | Medium: retargeted entry-artifact and source-contract carriers may still preserve workflow-form architecture. | Plan required. |
| TASK-1964 type/effect/runtime carriers | `ash-core`, `ash-typeck`, `ash-interp`, `ash-engine` runtime paths | Removed or retargeted tower and workflow runtime/type vocabulary. | Focused type/runtime checks and Phase 201 gate cover selected tokens and paths. | High: callable-entry registry, child-entry registry, entry projection, entry artifacts, application-boundary reports, and ambient effects may preserve old workflow semantics under target names. | Plan required. |
| TASK-1965 tooling removal | formatter, LSP, CLI, template, daemon, synthesized tests | Productive tooling now rejects or omits removed syntax and uses target wording. | Phase 201 gate, Phase 199/200 tests, LSP symbol tests, formatter/docs gates. | Medium: daemon/report JSON and runtime artifact APIs can continue teaching separate entry categories unless unified with functions/effect rows. | Refactor to target primitive where public/tooling-visible. |
| TASK-1966 docs/reference quarantine | `docs/`, `reference/`, book, indexes | Productive docs/examples are target-only; historical prose is labeled or routed away from current docs. | Docs gate and Phase 201 productive-doc scan pass for current roots. | Medium: reference pages still contain historical concepts that can look current if indexes route users there. | Plan required for documentation quarantine hardening. |
| TASK-1967 fail-closed gates | `crates/ash-cli/tests/phase201_deprecated_functionality_removal_gate.rs` | Gate blocks stale code/snippets/labels across active code paths. | Gate passes and has RED/GREEN evidence in TASK-1967. | Medium: token gates prove absence of named stale forms, not semantic equivalence to target primitives. | Keep as implementation detail plus add semantic gates from TASK-1970. |

## Retained Mechanism Inventory

| Mechanism | Current names | Old semantic origin | Target replacement | Decision | Owner |
|-----------|---------------|---------------------|--------------------|----------|-------|
| Function body runtime cache | `RegisteredFunctionBody`, `function_bodies`, `register_function_body` | first-class workflow/callable workflow registry | ordinary function metadata plus effect-row-admitted runtime invocation | Keep as implementation detail after TASK-1975; continue proving effect-row unification in TASK-1970 closeout | TASK-1975, TASK-1970-C |
| Spawned process body registry | `spawned_process_bodies`, `register_spawned_process_body` | child workflow spawn registry | process/channel spawn handles and application runtime metadata | Keep as implementation detail after TASK-1976; continue proving process/effect-row unification in TASK-1970 closeout | TASK-1976, TASK-1970-C |
| Application/entry identities | `Application*`, `application_id`, `entry_name`, `entry_type` | workflow definition/artifact/instance ids | application runtime report identifiers over target function entries | Keep as implementation detail after TASK-1977; public report/admission schema exposes application identity | TASK-1977, TASK-1970-C |
| Application result/report projection | application admission/report APIs | workflow projection wrapper | function/result boundary projection or application report projection | Delete stale entry Proc executor after TASK-1973; current projection evidence is through application boundary reports | TASK-1973, TASK-1970-C |
| TCIR/AMIR entry artifacts | entry-artifact carriers | workflow artifact carriers | target Core/TCIR/AMIR computation artifacts with effect rows | Refactor to target primitive | TASK-1970-B |
| Source contract carrier | `source_contract`, `contract::requires`, `contract::ensures` | workflow contract header/body adapter | contract/evidence helper layer over target functions | Refactor source-contract carrier to target primitive; helper intrinsic identities are retargeted after TASK-1978 | TASK-1978, TASK-1970-B |
| `WorkflowDef` residual carrier | current contract-only workflow-form structures | workflow declaration AST | remove or confine to historical parser tests; target function contracts are current path | Plan required | TASK-1970-A |
| Ambient effect context | `ambient_effect`, `entry_effect` | workflow effect/tower level context | row/profile effect typing | Keep as target profile/application metadata after TASK-1979 wording cleanup; closeout must continue proving row/profile coverage | TASK-1979, TASK-1970-B |
| Provider/capability runtime vocabulary | provider operation metadata, admitted capability bindings | direct capability/provider forms | provider profiles and row admission | Keep as implementation detail where explicit metadata is required | TASK-1970-B |
| Daemon/application reports | application/entry report JSON | workflow execution reports | application runtime reports over checked target entry functions | Refactor to target primitive if schema exposes stale distinctions | TASK-1970-C |
| Historical reference routing | feature matrix, reference index, context-pack index, historical cards, current function reference pages | old docs treated as current | quarantined prose excluded from productive docs | Keep as historical-only after TASK-1974/TASK-1980; productive routing points to target functions, effect rows, provider profiles, process/channel helpers, application runtime, and examples | TASK-1974, TASK-1980, TASK-1970-E |

## Cosmetic Rename Suspects

| Old name | New name | Behavior preserved | Why target justification is weak | Required proof |
|----------|----------|--------------------|---------------------------------|----------------|
| callable workflow registry | callable-entry registry | likely same registry semantics under new identifiers | target Ash should call ordinary functions with effect rows, not a separate workflow registry | prove registry is only an implementation cache for checked function metadata, or delete/refactor it |
| child workflow registry | child-entry registry | likely same child invocation semantics | Phase 195 process/channel primitives should own child concurrency semantics | map to process handles/channels or remove duplicate registry |
| workflow projection | entry projection | wrapper still projects old workflow-like boundary | target projection should be result/application report projection, not workflow Proc projection | prove projection works for ordinary target functions and has no workflow-only assumptions |
| workflow artifact | entry artifact | TCIR/AMIR artifact path may preserve old carrier layer | target IR should model computations/effect rows directly | cite Core/TCIR/AMIR target spec and tests for function/effect-row artifacts |
| workflow contract | source contract | contract carrier may preserve header/body workflow adapter shape | target contracts should be function/contract evidence helpers | prove only current `requires`/`ensures` contract events remain, or refactor |
| workflow effect | ambient/entry effect | effect context may preserve entry-specific tower effect distinction | target effects are rows/profiles | link to row/profile checker tests and remove entry-only special cases |

## Completed Semantic Cleanup Rows

| Task | Cleanup | Proof | Residual risk |
|------|---------|-------|---------------|
| TASK-1975 | Runtime callable-entry registry was retargeted to a function-body cache: `RegisteredFunctionBody`, `function_bodies`, and `register_function_body`. | Focused big-step, small-step, engine, dynamic-contract, and Phase 201 removal-gate tests pass. The gate now blocks the old callable-entry registry identifiers. | The cache still stores Core `Workflow` bodies because Core execution uses that internal representation. TASK-1970-C remains responsible for proving full function/effect-row unification at closeout. |
| TASK-1976 | Spawn execution registry was retargeted to a spawned-process body cache: `spawned_process_bodies`, `register_spawned_process_body`, and `spawned_process_body`. | Focused spawn/control integration tests, runtime boundary visibility tests, ash-engine helper checks, and Phase 201 removal-gate tests pass. The gate now blocks the old child-entry registry identifiers. | Spawn bodies still execute through the internal Core `Workflow` representation. TASK-1970-C remains responsible for proving full process/channel and effect-row unification at closeout. |
| TASK-1973 | Stale entry Proc projection executor was deleted from interpreter and engine public APIs. | Phase 201 removal-gate rows fail on `entry_projection`, `execute_entry_proc_projection`, `unsupported_entry_proc_projection_message`, and `FirstClassEntryProjectionExecutionUnsupported`; application admission/completion tests cover target result/report projection. | Core `WorkflowProcProjection` carriers still exist for lowering artifacts and remain owned by TASK-1972. |
| TASK-1974 | Historical workflow/tower reference routing was quarantined: feature matrix, reference index, context-pack index, getting-started next steps, and Act/Proc/Workflow/generalized-do cards no longer present removed tower pages as current guidance. | Docs orientation self-test, docs gate, markdown-link check, and diff whitespace check pass. Target routing now points to functions, runtime admission, application/runtime reports, checked examples, Result, and algebra pages. | Historical pages remain available for old links and migration context; closeout still needs a broader stale-claim sweep. |
| TASK-1977 | Application-boundary report/admission identity fields were retargeted from `workflow_id` to `application_id`; provenance report notes now use application wording. | Focused core report-carrier tests, contract-evidence schema tests, engine admission tests, completion report tests, compile checks, and Phase 201 removal-gate rows pass. | The underlying identity type remains `WorkflowId` as an internal implementation detail until a broader identity-type migration is planned. |
| TASK-1978 | Compiler-known contract helper intrinsic identities were retargeted from `workflow::requires` / `workflow::ensures` to `contract::requires` / `contract::ensures`, and standalone misuse tests now encode that contract helpers are not ordinary first-class calls. | Focused contract-misuse tests, `cargo check -p ash-typeck -p ash-cli --all-targets`, and Phase 201 removal-gate rows pass. The gate blocks workflow-scoped helper spellings in active typechecker source and tests. | `source_contract` and `WorkflowForm` carrier cleanup remains owned by TASK-1970-B/TASK-1971/TASK-1972; existing workflow algebra tests also expose a separate `Monad<Workflow>` evidence regression outside this slice. |
| TASK-1979 | Ambient effect-context comments and ambient target contract diagnostics were retargeted away from workflow-scoped wording to profile/target-contract vocabulary. | RED/GREEN Phase 201 removal-gate rows cover `Workflow effect context` and `workflow contract statement`; focused ambient-do, closure/effect, compile, and stale-term scan checks pass. | `entry_effect` remains as application/runtime metadata; Phase 201 closeout still needs broad proof that retained effect checks are row/profile-driven rather than tower-specific. |
| TASK-1980 | Current function reference pages and the function agent card were retargeted away from public tower guidance. They now point readers to target effect rows, provider profiles, process/channel helpers, contract/evidence helpers, and application runtime boundaries. | RED/GREEN Phase 201 removal-gate rows cover stale current-reference phrases such as `runtime-managed effect tower`, `explicit tower API`, `higher tower contexts`, and `Act/Proc/Workflow closures`; focused stale-phrase scan passes. | Historical tower pages remain for old links; closeout still needs a broader historical-doc source-snippet sweep. |
| TASK-1981 | Added `reference/status/removed-forms.md` as a prose-only authority page for removed historical forms and current target replacements, then routed the status index, root reference index, agent common-confusions page, and context-pack index to it. | Phase 201 removal gate asserts the page exists; direct scan proves the page has no Ash code fences or source-shaped removed snippets; docs orientation self-test and docs gate pass. | The page is a routing authority, not a full stale-claim sweep. TASK-1968 still owns broad closeout verification across historical docs. |
| TASK-1982 | Deleted compatibility-only typechecker bridge suites that asserted implicit Act/Proc/Workflow do-target behavior without explicit `Monad<K>` evidence, and retargeted pure-closure ambient context assertions to profile wording. | RED/GREEN Phase 201 removal-gate rows cover stale `behavior_still`, `do_workflow_still`, and `workflow-context` labels; deleted suites first failed on missing explicit Monad evidence; remaining closure test, compile check, gate, and stale-label scan pass. | Some filenames and historical task numbers still mention workflow where they cover current contract/parser/runtime internals; closeout owns the broader stale-claim sweep. |

## Target Function Unification Risks

| Surface/runtime path | Separate entry/workflow behavior | Expected replacement | Test gap |
|----------------------|----------------------------------|----------------------|----------|
| `ash run` / engine entry execution | entry artifacts and registry paths can differ from ordinary function invocation | checked `fn main` as ordinary function with effect-row admission and application report projection | need a test proving entry invocation shares ordinary function type/effect checking and no separate carrier admission |
| runtime callable registration | callable-entry registry can act as a second call semantics | ordinary function metadata table/cache, no separate semantic category | need equivalence/absence tests for ordinary function call vs registry-backed runtime call |
| child entry spawning | child-entry registry can preserve workflow child semantics | process/channel spawn primitives and join handles | need tests showing child execution is process primitive based, not workflow-registry based |
| TCIR/AMIR artifact construction | entry artifacts can preserve workflow artifact semantics | effect-row computation artifact over target Core/TCIR/AMIR | need IR-level tests asserting no workflow/tower carrier survives lowering |
| contract intrinsic storage | contract context still descends from workflow intrinsic context | contract/evidence helpers over target function contracts | need tests tying contract intrinsics to target function contracts, not WorkflowForm |

## Documentation Staleness Risks

| Doc/reference path | Stale concept | Target doc replacement | Gate needed |
|--------------------|---------------|------------------------|-------------|
| `reference/language/workflows.md` and stdlib workflow cards | workflow language pages can look current | historical quarantine page or redirect to application runtime/contracts/effects | gate productive indexes from routing to removed current claims |
| `reference/language/tower.md`, `effects-act.md`, `processes-proc.md` | public tower model | target effects/rows/process-channel references | source-shaped snippet and current-claim sweep |
| older specs such as `SPEC-054`, `SPEC-056`, `SPEC-097*`, `SPEC-098*` | migration wording can imply compatibility | explicit historical labels and Phase 201 authority notes | docs stale-claim sweep in TASK-1968 |
| agent cards for stdlib act/proc/workflow | agent guidance may suggest removed APIs | cards should be archived or marked historical | orientation-index read-path audit |

## Test Adequacy Review

| Test/gate | What it proves | What it does not prove | Required stronger evidence |
|-----------|----------------|------------------------|----------------------------|
| `phase201_deprecated_functionality_removal_gate` | removed tokens/snippets/labels are absent from active scanned paths | renamed mechanisms are semantically target-correct | semantic gates for registry/projection/artifact unification |
| parser removed-syntax tests | selected old forms fail closed without embedding full stale source files | every internal carrier was deleted | AST/lowering carrier absence tests |
| stdlib corpus check | active stdlib Ash files parse/check as target Ash | docs/reference pages are target-correct | docs quarantine sweeps and link/index review |
| formatter/LSP/template tests | tooling no longer surfaces selected old forms | runtime/tooling APIs do not expose stale categories | schema/report public-field review |
| cargo check/clippy | changed Rust compiles cleanly | old semantics are gone | targeted negative tests and public API scans |

## TASK-1970 Inputs

TASK-1970 must turn the `Plan required` and `Refactor to target primitive` rows into concrete
workstreams:

- parser/lowering residual `WorkflowDef` and contract-form carriers;
- Core/TCIR/AMIR entry-artifact and contract/evidence integration;
- runtime callable-entry, child-entry, entry-projection, and application-report unification;
- tooling/schema/docs removal of public stale distinctions;
- semantic gates that prove behavior removal, not only token removal.

## Verification

This audit is documentation-only. Verification for this slice:

```bash
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```
