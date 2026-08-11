---
id: audit.208.language-scope-dispositions
title: Ash Language Scope Disposition Inventory
kind: audit
status: complete
authority: evidence-inventory
owner: language-semantics
last_verified: 2026-08-09
---

# AUDIT-208: Ash Language Scope Disposition Inventory

## 1. Purpose and authority

This docs-only audit records the evidence and repository consequences behind
[SPEC-104](../../spec/SPEC-104-LANGUAGE-SCOPE-FREEZE.md). It does not activate PLAN-208, define
language meaning, or claim implementation completion. SPEC-104 is the binding disposition
authority.

The inventory combines:

- the guided surface-language review completed between 2026-08-08 and 2026-08-09;
- the target documents named in SPEC-104 frontmatter;
- the implementation-backed census in
  [AUDIT-206](AUDIT-206-implementation-backed-language-reference.md);
- the execution seam findings in
  [AUDIT-204](AUDIT-204-direct-ast-retirement.md) and
  [AUDIT-207](AUDIT-207-module-realization-seams.md);
- PLAN-194, PLAN-195, PLAN-196, and PLAN-203.

This was a scope and conflict audit, not a new line-by-line code census. “Tested” below means the
repository has relevant executable evidence recorded by the cited audits; it does not mean full
SPEC-104 coverage.

## 2. Status key

The phase disposition and delivery report are independent:

- **Implementation:** implemented, partial, or not_implemented.
- **Evidence:** proved, tested, or none.
- **Parity:** matches_spec or below_spec.

The P1 umbrella is currently partial and below_spec because the complete frozen domain does not
yet travel through the production Surface-to-Engine route. An individual family may instead be
not_implemented with no exact evidence. For a removed family, partial means code, syntax, tests, or
documents still preserve part of a feature that the frozen target rejects.

## 3. Primary disposition inventory

This table is complete at the feature-family granularity used by SPEC-104. Exact clauses remain
owned by the mapped Canonical Core rules and their reconciled target specifications.

| Feature family | Evidence before freeze | Disposition | Implementation / evidence / parity | Reason | Required follow-up owner |
|---|---|---|---|---|---|
| Ordinary function application entry | fn main parsing and bounded Engine/client tests in AUDIT-206 LANG-005/LANG-016 | P1 | partial / tested / below_spec | Smallest general executable entry; replaces workflow identity | PLAN-203 entry and parity slices |
| Strict application manifest | Runtime descriptors exist, but no frozen exact manifest schema was found | P1 | not_implemented / none / below_spec | Makes app identity, entry, provider recipes, schemas, and binding slots explicit without language magic | New PLAN-203 vertical slice |
| Static file/inline modules | SPEC-103 and PLAN-207; current graph/link seams in AUDIT-207 | P1 | partial / tested / below_spec | Necessary for real programs; file placement must not change meaning | PLAN-207 then PLAN-203 integration |
| Imports, visibility, grouped imports, re-exports | Parser/checker evidence in AUDIT-206 LANG-002; SPEC-103 target | P1 | partial / tested / below_spec | Supports explicit dependency and facade modules without a prelude or globs | Module realization |
| Pure reachable module initialization | Target fragments exist; no test of the exact frozen initialization rule was found | P1 | not_implemented / none / below_spec | Deterministic immutable setup without hidden effects | Module/type/Core slice |
| UTF-8 source, ASCII identifiers, and line-comment subset | Parser evidence exists, but the exact BOM/newline/rejection domain was not audited | P1 | partial / tested / below_spec | Keeps lexical behavior portable and small | Lexer/parser slice |
| Public annotation and local best-effort inference | Current type-checking evidence is fragmented across SPEC-097a and AUDIT-206 | P1 | partial / tested / below_spec | Preserves explicit boundaries while limiting inference complexity | Type-checker slices |
| Strict function/control core | Function, call, block, if/match, closure, return, and local lowering evidence is fixture-bounded in AUDIT-206 | P1 | partial / tested / below_spec | Provides the ordinary executable substrate without loops, currying, or exceptions | Grammar/type/Core/runtime slices |
| Frozen pattern subset and exhaustiveness | SPEC-076 and parser/type tests cover a broader and partly conflicting set | P1 | partial / tested / below_spec | Keeps refutability explicit and lowering finite | Pattern checker/lowering slice |
| Nominal ADTs, tuples, structural records, aliases, opaque and empty types | Parser/type fragments in AUDIT-206 LANG-006/LANG-008 | P1 | partial / tested / below_spec | Small useful data core with unambiguous sum/product syntax | Grammar/type/Core/runtime slices |
| Minimal interfaces and associated types | Parser/checker summaries in AUDIT-206 LANG-009 | P1 | partial / tested / below_spec | Required abstraction without inheritance/default/overlap machinery | Type/evidence slices |
| Coherent dictionaries and using override | Related interface evidence exists; the exact source/Core override rule was not found | P1 | not_implemented / none / below_spec | Allows rare explicit policy selection while preserving default coherence | Dedicated type/Core dictionary slice |
| Rank-1 generics | Existing parser/type evidence | P1 | partial / tested / below_spec | Useful generic functions with bounded inference | Type/Core slices |
| Unary HKT and minimal Monad | SPEC-067/078 show existing ambition and partial evidence | P2 | partial / tested / below_spec | Useful for Option/Result abstraction, but not needed for P1 execution | Future P2 plan |
| Higher constructor kinds | Existing HKT documents do not require this tier for P1/P2 | P3+ | partial / tested / below_spec | High complexity and little ordinary-code value now | Fresh future proposal |
| Direct nominal effect rows | SPEC-096b/097b and handler/provider task evidence | P1 | partial / tested / below_spec | Core orchestration mechanism; rows are requirements, never grants | Minimal effect vertical slices |
| Deep affine lexical handlers | TASK-2013/TASK-2014 evidence summarized in AUDIT-206 LANG-013 | P1 | partial / tested / below_spec | Smallest useful algebraic handling model | Handler integration owner |
| Parameterized exact effects and manifest providers | Provider metadata and admission fragments exist | P1 | partial / tested / below_spec | Supports explicit alternative policies such as real/dummy Fs without first-class provider magic | Provider/admission slice |
| Provider dependency, cancellation, and shutdown substrate | Existing provider/daemon lifecycle evidence does not establish the exact frozen rule | P1 | partial / tested / below_spec | Provider services are required Engine infrastructure, distinct from a language service abstraction | Provider runtime slice |
| Explicit module-visible extern boundary | Existing evidence covers incompatible builtin/host routes, not the frozen replacement | P1 | not_implemented / none / below_spec | Removes magical names and keeps effectful host access behind providers | Stdlib/provider replacement |
| Numeric literals and Int/Float semantics | Parser and primitive runtime tests exist but were not audited against the complete frozen arithmetic domain | P1 | partial / tested / below_spec | Deterministic arithmetic and tier-comparable performance need one exact primitive contract | Primitive/Core/runtime slices |
| String, Char, Bytes, UTF-8, and literal semantics | Parser/stdlib fragments exist; exact scalar/byte traversal and decode behavior is incomplete | P1 | partial / tested / below_spec | Required text/binary substrate without indexing complexity | Primitive/stdlib slices |
| Canonical boundary JSON | Existing JSON/runtime descriptors do not implement the schema-directed encoding frozen by SPEC-104 | P1 | not_implemented / none / below_spec | Gives CLI and daemon one exact typed boundary including ADTs, bytes, maps, and special floats | Boundary codec/admission slice |
| Immutable List and ordered Map | Stdlib evidence in AUDIT-206 LANG-017 and SPEC-089 | P1 | partial / tested / below_spec | Minimum useful collection set with deterministic behavior | Stdlib plus Engine coverage |
| Set | Related algebra/library substrate exists | P2 | partial / tested / below_spec | Useful but not required for executable P1 | Future P2 library |
| General Iterator/lazy collection adapters | Existing interface/HKT work is not a complete executable iterator route | Remove | partial / tested / below_spec | Concrete operations and recursion are simpler; a later fresh proposal has no compatibility duty | Parser/type/stdlib deletion sweep |
| Primitive parse/format companions | No complete canonical behavior found | P1 | not_implemented / none / below_spec | Boundary automation needs deterministic text conversion without generic formatting magic | Primitive-library slice |
| Explicit assertion | Adjacent contract/trap artifacts do not test the exact frozen evaluation rule | P1 | not_implemented / none / below_spec | Supplies the minimum invariant trap | Grammar/type/Core/runtime slice |
| Bottom-typed todo | Adjacent lint/failure artifacts do not test the exact frozen policy and runtime rule | P1 | not_implemented / none / below_spec | Supports incomplete programs without fabricating typed values | Grammar/type/admission/runtime slice |
| Ordinary manifest-selected tests | Existing ash test machinery is broad and contract-oriented | P1 | partial / tested / below_spec | Reuses the real application route and avoids a second evaluator | Test runner simplification |
| Shared-nothing lightweight processes | PLAN-195 and current process/runtime fragments | P1 | partial / tested / below_spec | Central orchestration need; explicit mailboxes and ownership bound semantics | Narrowed PLAN-195 slices |
| Process quotas, mailbox budgets, memory accounting, tombstones, and deadlines | PLAN-195 contains fragments; the complete frozen resource model has no single production proof | P1 | partial / tested / below_spec | Bounds exhaustion while keeping short-lived process cleanup predictable | Process runtime/admission slices |
| Dedicated channels | Broad channel row/process plans exist | Remove | partial / tested / below_spec | One typed process mailbox is enough; any later communication abstraction starts fresh | PLAN-195/code/test deletion |
| Links, supervision, typed exits, service lifecycle | PLAN-195/196 contain designs beyond the frozen minimum | P2 | partial / tested / below_spec | Valuable after scheduler/runtime experience, not a P1 prerequisite | Narrow future runtime plan |
| Cross-app/node distribution and serialization | No complete production route | P3+ | not_implemented / none / below_spec | Must follow local shared-nothing semantics and real distributed requirements | Fresh distributed proposal |
| Optional replay-ready trace recording | Trace/monitor artifacts exist but mix observation and enforcement | P1 | partial / tested / below_spec | Essential debugging evidence; program-invisible and fail-closed when enabled | Runtime trace slice |
| Offline temporal analysis | Contract/monitor plans provide concepts, not the frozen separation | P2 | partial / tested / below_spec | Useful without changing recorded execution | Future analysis tooling |
| Runtime temporal contracts | SPEC-099b and PLAN-194/195 retain monitor machinery | P3+ | partial / tested / below_spec | Complex and unnecessary for P1/P2 runtime correctness | Fresh future contract proposal |
| Lazy/memo/force computation modes | SPEC-101 and implementation/tests claim an MVP | Remove | partial / tested / below_spec | Strictness plus ordinary thunk functions is sufficient; explicit memo belongs in libraries/providers | Core/type/runtime/code/test deletion |
| Macros and notation | SPEC-095c and related tasks contain implemented/target fragments | P3+ | partial / tested / below_spec | Useful later but expands parser, resolver, tooling, and semantic surface now | Remove from active P1 prepass; future fresh proposal |
| Roles, policies, capability/resource/channel row families | Target specs contradict their own removal amendments; historical implementation remains | Remove | partial / tested / below_spec | Duplicates nominal effects/providers and preserves the legacy governance tower | Grammar/type/Core/runtime/code/test deletion |
| Existing requires/ensures/laws/proofs/evidence discharge | PLAN-194 and many completed tasks preserve substantial machinery | Remove | partial / tested / below_spec | No P1 dependency and no legacy compatibility; later evidence syntax starts fresh | Retire PLAN-194 as active authority and delete legacy routes |
| Future metadata/property/proof ladder | Ideas and historical machinery exist, but the frozen staged model is not implemented | P3+ | not_implemented / none / below_spec | Preserves the long-term correctness goal without constraining P1 | Fresh evidence proposal |
| Workflow, Act, Proc, typed do, comprehensions | Extensive historical specs and tests | Remove | partial / tested / below_spec | Ordinary functions, direct effects, and explicit control are the coherent core; future sugar starts fresh | Broad source/Core/runtime/stdlib deletion |

## 4. Conflict map

### 4.1 Canonical Core

Retain the one checked Core/CPS/Engine route, rows-as-requirements rule, structured terminals, and
ordinary fn main. Revise the vocabulary and grammar clauses that still name role, policy,
resource, channel, general evidence discharge, do, macros, or notation as active target features.

### 4.2 SPEC-095b target grammar

Retain static modules, ordinary functions, strict expressions, the frozen type/pattern subset, and
historical-form rejection. Reconcile these areas:

- remove role, policy, workflow, Act/Proc/Workflow, dedicated resource/channel, legacy migration,
  and broad contract declarations from the active target grammar;
- reduce handler grammar to the lexical deep-affine P1 form;
- remove do/comprehensions from P1;
- route macros and notation to P3+ rather than requiring them in the P1 parser/interface path;
- apply the exact ADT, alias, opaque type, record, function-body, import, literal, pattern, assert,
  todo, and source-encoding forms frozen by SPEC-104.

### 4.3 SPEC-096b target effects

Retain requirements-not-grants, operation effects, exact providers, and lexical handling. Remove
the universal governance taxonomy:

- no role, policy, channel, process, general contract, or evidence row items;
- no row profiles preserving the former tower under new names;
- no resource row item unless a future amendment proves that provider-owned host state is
  insufficient;
- narrow aliases/groups and row polymorphism to the exact P1/P2 choices in SPEC-104;
- align provision with lexical handlers plus the unique manifest recipe.

### 4.4 SPEC-097b target types

Retain row identity/normalization, ordinary algebraic types, rank-1 functions, and the minimal
interface substrate. Reconcile:

- delete role/policy/channel/resource/evidence/general-contract variants and discharge rules;
- delete strict/lazy/memo mode conversions and force accounting;
- replace broad HKT/type-computation ambitions with P1 rank 1, P2 unary HKT/minimal Monad, and P3+
  higher constructors/proofs;
- apply the exact data syntax, structural record behavior, associated type rules, interface
  coherence rules, and inference boundary from SPEC-104.

### 4.5 SPEC-098b, SPEC-098c, SPEC-099b, and SPEC-100

Retain strict checked Core/CPS, minimal handler/provider lookup, structured traps, and the
single-executor contract. Remove or defer:

- lazy store, thunk/force/memo modes, and their accounting;
- active runtime monitor sets and MonitorFrame;
- role/policy/channel/resource/evidence/general-contract Core families;
- direct-evaluator or compatibility fallback paths;
- any syntax/lowering requirement for macro, notation, do, comprehension, law, or proof in P1.

The production state is strict computation plus environment, minimal effect frames, process/runtime
state, provider state, and optional observational trace.

### 4.6 SPEC-101

Retire SPEC-101 from the target read path. Its implementation and tests are deletion targets.
Historical rationale may remain clearly labeled, but no P1/P2 task may depend on lazy, memo, or
force compatibility.

### 4.7 SPEC-103

Retain file/inline equivalence, canonical module identity, explicit imports/visibility, checked
interfaces, deterministic linking, and one execution route. Remove Policy and Role namespace
entries. Do not make macros/notation or other P3+ declarations prerequisites for P1 module
interfaces, syntax prepasses, or export closure.

### 4.8 PLAN-194

Treat completed work as historical implementation evidence, not frozen language authority. P1
keeps only assert and ordinary Result/effects. Future metadata, generated property tests, Ash
proofs, and foreign proof assistants require fresh P2/P3+ tasks and explicit compiler trust policy.

### 4.9 PLAN-195

Narrow P1 to isolated processes, explicit typed FIFO mailboxes, spawn/send/receive/stop/join,
limits, cancellation, deadlines, provider messaging, and observational trace. Remove dedicated
channels and runtime temporal monitors. Defer links, supervision, alternative mailbox policies,
typed exits, and service lifecycle to P2.

### 4.10 PLAN-196

Reframe around ordinary-function application entry and explicit manifest admission. Remove
workflow compatibility, roles, policies, capabilities, supervisor profiles, and P1 external
actors. Dedicated services move to P2; distributed actors move to P3+.

### 4.11 PLAN-203

Retain PLAN-203 as the primary execution programme, but select features through SPEC-104 before
opening a vertical slice. “Long-running program” in daemon scope does not authorize P2 service
lifecycle. The lambda-Ash calculi explain retained CPS behavior; they are not extra execution
stages.

## 5. Prioritized cleanup programme

This audit recommends five short packages, each independently reviewable:

1. **Authority repair:** put SPEC-104 first in orientation paths; amend Canonical Core precedence;
   label SPEC-101 and conflicting target clauses as superseded for scope.
2. **Executable kernel:** manifest plus ordinary main, primitive/data/function core, module closure,
   and exact terminal route through CLI and daemon.
3. **Minimal effects:** exact nominal rows, lexical deep-affine handlers, provider recipes,
   admission, and provider lifecycle.
4. **Process kernel:** explicit mailbox/configuration, bounded resources, cancellation, join
   retention, and optional replay-ready trace.
5. **Deletion sweep:** remove workflow/tower, role/policy/capability/resource/channel families,
   lazy/memo/force, broad contracts/evidence, macros/notation from P1 paths, and their obsolete
   code/tests.

Each package should be split by a canonical semantic rule and delivered vertically. A deletion
slice should delete production code, fixtures, redundant tests, compatibility diagnostics, and
stale documentation together where that keeps the patch reviewable.

## 6. Process recommendations

- Keep one active semantic slice and one small cleanup slice at a time.
- Require a one-screen task contract: rule, exact accepted/rejected behavior, owned layers,
  handoffs, non-goals, tests, and deletion targets.
- Update the semantic-rule coverage row before implementation; derive status from the row and
  verification artifacts instead of requesting narrative reports.
- Make task state machine-readable in existing indexes. Do not create a new report document for
  routine progress.
- Time-box discovery. If a slice cannot be stated with a small acceptance table, split it before
  implementation.
- Use the smallest relevant skill set: project TDD for implementation, Rust guidance only for
  Rust changes, systematic debugging only for observed failures, and verification at handoff.
  Avoid invoking overlapping workflow skills by default.
- Use subagents only for genuinely independent test, implementation, audit, or QA work. Give each
  a bounded file/rule scope and require evidence paths rather than prose status.
- Use narrow crate checks during iteration and the full gate at integration. Do not repeatedly run
  the full workspace gate while the slice is knowingly incomplete.
- Track performance per executor tier with stable representative programs. A faster fallback
  evaluator is not progress toward executable-spec parity.

## 7. Exit criteria for the freeze programme

The freeze has served its purpose when:

- every P1 feature is linked to canonical rules and has a complete vertical realization status;
- CLI and daemon agree on normalized terminal outcomes for the same admitted program and envelope;
- every removed form rejects before admission and no compatibility evaluator remains;
- obsolete code and tests are deleted rather than maintained;
- P2 and P3+ items have no dependencies in the P1 build or execution route;
- routine progress is visible from coverage, task, changelog, and verification artifacts without a
  separately requested summary.
