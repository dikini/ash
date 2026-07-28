---
id: plan.202.formal-semantics-verification-programme
title: Formal Semantics and Verification Programme
kind: plan
status: planned
authority: planning
owner: language-semantics
last_verified: 2026-07-27
---

# PLAN-202: Formal Semantics and Verification Programme

## 1. Purpose

This programme establishes the authority, documentation, semantics, and implementation-verification
foundation required before Ash designs or implements an Ash-native `spec`/`proof` system.

It has four outcomes:

1. humans and agents can identify the authoritative meaning of Ash without reconciling historical
   plans, duplicated specs, implementation notes, and target-state documents ad hoc;
2. a staged Core/CPS calculus suite explains CPS control, effects, and later extensions without
   becoming a second execution pipeline;
3. selected Rust properties can be verified incrementally against that calculus using experimental
   Verus pilots; and
4. the resulting Rust `spec -> types -> contracted skeleton/tests -> code -> proof` workflow
   provides evidence for the later design of the corresponding Ash workflow and LLM-guided proof
   synthesis.

This is a programme plan, not a claim that the corpus is already canonical or that the named
theorems are already proved.

## 2. Entry Conditions and Hard Gates

The documentation and semantic authority work is a prerequisite for proof-system implementation.
The tracks may overlap only after their declared inputs are frozen.

```text
Phase 201 frozen handoff (for overlapping implementation audit)
                 |
                 v
       authority/corpus inventory
                 |
                 v
       canonical manifest + core
          /          |          \
         v           v           v
  CPS calculus   Rust/Verus   traceability
       freeze      pilots        gates
         \           |           /
          \          v          /
           +-- measured evidence --+
                      |
                      v
        Ash spec/proof design programme
```

Hard gates:

- No document is promoted to the canonical corpus until its current/target/historical role and
  conflicting claims have been audited.
- No deprecated Rust mechanism is removed merely because its name is old; a semantic replacement
  and behavior-level evidence are required.
- No Verus proof may define Ash semantics from current Rust carrier shape. Each pilot must name the
  canonical rule or model it refines.
- No successful SMT or Verus run is accepted as portable Ash proof evidence without recorded tool,
  version, options, assumptions, and model/implementation identity.
- No model proof is reported as a production-runtime proof without a checked refinement bridge from
  the production implementation to the proved model.
- No Ash-native proof syntax implementation begins before the CPS calculus, obligation identity,
  evidence taxonomy, and hole/trust policy have approved specifications.
- A justified toolchain or pilot no-go result is a valid programme result. It must preserve the
  failed obligation, evidence, and remediation owner; it conditionally skips downstream pilots
  rather than making programme closeout impossible.

## 3. Programme Invariants

- **One owner per semantic fact.** Multiple documents may explain a rule; exactly one canonical
  node owns it.
- **Authority is explicit and machine-readable.** Directory placement and recency are never used as
  proxies for authority.
- **Canonical sources are human-authored; agent packs are derived.** Generated summaries and context
  packs may improve retrieval but cannot override their sources.
- **History remains available but inert.** Archived documents retain provenance and links while
  productive read paths exclude them by default.
- **Surface meaning factors through Core/CPS.** Surface lowers to Core, Core lowers to CPS, and the
  calculus suite is the mathematical semantics of CPS. Runtime behavior is related through
  refinement and observable projection, never through a parallel evaluator.
- **Unknown is not proved.** Unsupported, timed-out, holed, tested, monitored, and admitted
  obligations remain distinct from verified obligations.
- **LLMs propose; checkers decide.** LLM-generated specs, invariants, tests, code, and proof steps
  become evidence only after a deterministic checker or declared proof provider validates them.

### 3.1 Target-spec parity and evidence

The target Ash specification defines the complete implementation domain of each feature. Reports
MUST state **Implementation** (`implemented`, `partial`, or `not_implemented`), **Evidence**
(`proved`, `tested`, or `none`), and **Parity** (`matches_spec` or `below_spec`) independently.
Tests provide confidence in implementation; a proof provides evidence only for its stated theorem
and refinement scope. A task or handoff may be complete while the associated target feature remains
`partial` and `below_spec`.

New behavior outside a target rule requires a target-spec update before implementation. Existing
behavior below that rule is a parity gap, not a complete feature. The traceability graph records
implementation links, test links, and proof links separately, including any bridge needed for a
production-runtime proof.

## 4. Authoritative Corpus

### 4.1 Authority levels

The canonical manifest introduced by this programme must classify every productive documentation
artifact at one of these levels:

| Level | Role | May define language truth? | Examples |
|---|---|---:|---|
| A0 | authority root, vocabulary, identity and conflict policy | Yes, for governance | canonical manifest, semantic vocabulary |
| A1 | normative language and semantic definitions | Yes, for intended meaning | grammar, types/effects, Core/CPS calculus, operational semantics |
| A2 | normative layer-handoff and observable contracts | Only for the owned boundary | surface-to-parser, lowering, runtime projection |
| A3 | conformance corpus and expected-result artifacts | No; instantiates A1/A2 | canonical cases, result schema, implementation-conformance rules |
| A4 | derived explanation and retrieval artifacts | No | reference pages, tutorials, agent context packs |
| A5 | research, plans, audits, evidence, and archive | No | notes, phase plans, closeouts, historical specs |

Within A1/A2, the canonical manifest assigns a single `canonical_for` owner for each semantic
subject. A conflict is a validation failure; chronology does not resolve it. A proposed change must
update the owner, its dependents, conformance cases, and supersession metadata in one change.

Authority is question-sensitive: A1/A2 own intended language meaning, while live code/tests own
realization evidence about what the current implementation does. When they disagree the result is a
drift finding, never an implicit promotion of code over semantics or semantics over observed fact.

### 4.2 Candidate promotion set

The following is the starting set to audit, not an automatic declaration that every current claim is
canonical:

| Subject | Candidate sources | Required reconciliation |
|---|---|---|
| vocabulary and language overview | `docs/SHARO_CORE_LANGUAGE.md`, target specs, removed-form authority | remove historical tower/workflow authority and freeze target terminology |
| target grammar | `SPEC-095b`, `SPEC-095c`, surface-to-parser contract | reconcile implemented grammar with target claims and macro/source preservation |
| type and effect systems | `SPEC-096b`, `SPEC-097b`, `SPEC-100` | distinguish surface elaboration, Core checking, requirements, and authority discharge |
| Core and CPS syntax | `SPEC-098b`, `SPEC-099`, CPS reference files | unify duplicate term/state vocabularies and freeze identifiers |
| lowering | `SPEC-098c`, historical parser-to-Core rationale, Core-to-CPS implementation/reference | state total domain, residual unsupported forms, and preservation obligations |
| operational semantics | `SPEC-099b`, relevant rules retained from `SPEC-004` and `SPEC-025` | replace workflow-first authority with target function/Core/CPS semantics or archive it explicitly |
| observable/runtime semantics | `SPEC-021`, semantic execution-record contract, runtime observable contract | freeze projection fields and bounded nondeterminism |
| implementation conformance | `SPEC-026`, canonical IR corpus, canonical result format | bind tests and proofs to semantic rule identities |
| contracts and proof obligations | `PLAN-194` results, `NOTE-030` through `NOTE-038`, `SPEC-064`, `SPEC-081`, `SPEC-085` | promote only accepted behavior; retain proof-language proposals as research until separately approved |

The active proof-facing authority statement is the
[Ash Canonical Core](../spec/CANONICAL-CORE.md). The former
[Formalization Boundary and Proof Targets](../reference/formalization-boundary.md) is retained
only as a historical route; the audit records its workflow-first and Lean-specific assumptions as
superseded rather than silently routing around them.

The initial conflict ledger already contains these blocking findings:

- `docs/spec/README.md` labels older workflow/tower specs active while `SPEC-INDEX.md` classifies
  several of them as historical or routes target work to SPEC-095b through SPEC-100;
- `docs/reference/formalization-boundary.md` treats SPEC-004/SPEC-025 workflow-first semantics as
  the canonical theorem subject while Phase 201 removes those source and runtime categories and the
  target orientation path names SPEC-099b;
- `docs/reference/parser-to-core-lowering-contract.md` still describes a workflow-first handoff;
- several SPEC number families are overloaded (`038`, `095`, `096`, `097`, `098`, and `099`), so
  stable semantic ids cannot be inferred from display numbers alone;
- Phase 201 is labelled complete while TASK-1971 remains in progress and TASK-1972 planned in
  `PLAN-INDEX.md`, and the indexed TASK-1972 task file is absent; and
- the current docs gate checks links and index structure but does not prove semantic authority
  consistency or implementation alignment.

These are audit inputs, not conclusions to resolve by bulk relabelling. The owning rules and actual
implementation behavior must decide each replacement.

### 4.3 Target canonical core

The target canonical corpus should be small enough to load as a coherent agent context and complete
enough to define the language:

1. authority, terminology, semantic domains, and notation;
2. lexical and surface grammar;
3. surface typing and effect/row judgments;
4. Core/CPS syntax, well-formedness, and typing;
5. surface-to-Core and Core-to-CPS lowering relations;
6. Core/CPS operational semantics;
7. runtime/observable projection and explicit nondeterminism boundaries;
8. implementation-conformance contract and canonical executable corpus.

All other current documentation must either derive from these nodes, declare an unresolved proposal,
or move to the archive.

### 4.4 Scope manifest

TASK-1984 freezes a versioned scope manifest before claiming corpus completeness. It includes:

- all Markdown under `docs/` and top-level `reference/`, classified even when A5/nonproductive;
- productive roots reached from `docs/README.md`, SPEC/NOTE read paths, top-level reference indexes,
  tutorials, templates, examples, and documentation gates;
- canonical and handoff data artifacts referenced by those documents; and
- public semantic Rust surfaces in `ash-core`, `ash-parser`, `ash-typeck`, `ash-interp`,
  `ash-engine`, and `ash-provenance` when they define or expose grammar, typing, lowering,
  evaluation, authority, trace, or observable behavior.

Generated build output, vendored dependencies, caches, external submodules, archived git snapshots,
and purely internal tooling helpers are excluded unless a productive authority edge points to them.
An exported Rust item is `semantic` only when it realizes or exposes an A1/A2 rule; other public API
items remain API evidence but are not part of semantic proof coverage. Every exclusion is recorded,
not inferred from an unsearched directory.

The manifest records the git revision and dirty-worktree qualification used for the inventory.
TASK-1988 requires either completion of TASK-1971/TASK-1972 or an explicit Phase 201 handoff revision
that freezes which in-flight mechanisms it may audit. TASK-1983 plan creation itself does not depend
on Phase 201 completion.

## 5. Canonical Documentation Architecture

### 5.1 Target layout and existing governance

This programme extends, and must not silently replace, SPEC-071 and DESIGN-035/DESIGN-042. Those
artifacts already establish a two-corpus model, typed authority links, frontmatter, drift findings,
and git-backed snapshot manifests. PLAN-202 adds the missing compact normative core and semantic
traceability graph.

The migration audit should converge on this logical layout. Exact physical moves occur only after
the manifest and redirect map exist.

The first implementation is a manifest overlay over stable paths:

```text
docs/spec/
  CANONICAL-CORPUS.yaml   machine authority/rule manifest
  CANONICAL-CORPUS.md     generated compact human/agent map
  ...existing specs...    promoted by rule, never by directory alone

docs/reference/           A2 handoff contracts pending promotion/reconciliation
docs/design/, notes/      A5 proposals and rationale
docs/plan/, audit/        A5 plans, tasks, audits, and completion evidence
docs/ideas/               A5 exploratory history

reference/                SPEC-071 A4 curated current corpus
  language/, runtime/, stdlib/
  agents/                 derived cards and context packs
  status/                 current limitations, drift, and removed-form routing
  manifests/              git-backed snapshots and extraction/archive manifests
```

This overlay makes authority usable before high-churn moves. After reconciliation, TASK-1986 may
propose a compact physical `docs/canonical/` tree, but only if the measured navigation benefit
outweighs link churn and the manifest/redirect system makes the move reversible. Top-level
`reference/` remains the curated human/agent reading surface defined by SPEC-071; it does not become
semantic authority. Physical organization must not get ahead of semantic reconciliation.

### 5.2 Required metadata

The canonical authority manifest uses a versioned `canonical-corpus/v1` sidecar schema. It extends
SPEC-071 concepts without changing the allowed enums or required frontmatter of top-level
`reference/` pages. Existing reference pages continue to validate under SPEC-071; canonical nodes
and trace nodes validate under their own schema and link to reference derivatives through typed
edges.

Every indexed productive canonical document must provide or inherit:

```yaml
id: spec.ash.core-cps.operational-semantics
schema: canonical-corpus/v1
kind: semantic-rule-set
audience: [human, agent]
authority_level: A1
lifecycle: active
stability: alpha
owner: language-semantics
canonical_for: [semantics.core-cps.step]
supersedes: []
depends_on: []
verified_against:
  git_commit: unknown
  specs: []
  tasks: []
  code: []
  tests: []
  examples: []
related:
  explains: []
  superseded_by: null
refresh_trigger: []
trace_nodes: []
last_verified: YYYY-MM-DD
```

The schema must define controlled values and validate unique `id`, unique `canonical_for` ownership,
acyclic supersession, valid links, and freshness for generated artifacts.

### 5.3 LLM and agent access

Generated agent packs are selected by semantic topic and include:

- the relevant A0 authority statement;
- the owning A1 rule definitions;
- required A2 handoff contracts;
- linked A3 conformance cases;
- unresolved gaps and explicit exclusions.

Each pack records source identities and content hashes. It must fail generation on authority
conflicts and must never summarize A5 material as current guidance. A small retrieval benchmark must
test whether agents select the right grammar, type, lowering, runtime, and proof sources without
broad repository search.

### 5.4 Change protocol

A canonical semantic change is complete only when the same change updates:

1. the owning rule and manifest metadata;
2. dependent rules and handoff contracts;
3. traceability edges;
4. executable conformance cases or an explicit no-case rationale;
5. implementation/proof impact status;
6. generated indexes/context packs; and
7. `CHANGELOG.md`.

### 5.5 Validation expansion

The present documentation gate validates changed Markdown links and orientation-index structure; it
does not validate semantic authority, status consistency, or code/spec alignment. PLAN-202 adds:

- schema parsing, unique stable ids, and resolvable anchors;
- exactly one active owner for each canonical subject/rule;
- acyclic supersession and valid typed authority edges;
- full canonical/reference link validation rather than changed-files-only validation;
- no archived/historical artifact in productive authority or default retrieval paths;
- current-reference freshness and resolvable evidence paths;
- deterministic context-pack generation with manifest/source hashes;
- canonical-rule coverage by implementation/test/proof or an explicit owned gap;
- public semantic implementation coverage by a canonical owner or private-machinery declaration;
- phase/task consistency so a phase cannot claim completion while required indexed follow-ups remain
  active; and
- proof status validation that keeps `verified`, `admitted`, `deferred`, `failed`, and empirical
  evidence distinct.

Baseline failures and staleness discovered by TASK-1984 are inputs to migration tasks, not waived
because the current gate is green.

## 6. Archive and Supersession Programme

Archiving is classification plus routing, not deletion or a copied shadow tree. Git commits/tags
remain the immutable snapshots, following DESIGN-042 and SPEC-071; named manifests describe the
meaning, extraction profile, verification evidence, and known exclusions of those snapshots.

### 6.1 Classification

Each document receives one disposition:

- `promote`: becomes or contributes to an A0-A3 canonical node;
- `derive`: retained as A4 explanation generated or checked against canonical sources;
- `research`: unresolved proposal retained outside productive authority;
- `archive`: superseded/historical but still useful for provenance;
- `merge-then-archive`: unique content is migrated to an owner before archival;
- `delete-generated`: reproducible duplicate with no independent provenance value.

### 6.2 Reversible migration

1. Produce an inventory with current path, stable id, inbound links, authority claims, disposition,
   replacement, and unique-content notes.
2. Create the canonical manifest and replacement nodes before moving sources.
3. Add a machine-readable redirect/supersession map.
4. Record archived artifacts in a named git-backed archive manifest, preserving original path and
   revision; move a file physically only when doing so improves current routing and the redirect is
   already validated.
5. Leave short tombstones at high-value old paths for at least one documented migration cycle.
6. Exclude archive content from productive indexes, generated context packs, examples, and default
   agent read paths.
7. Run link, orientation, stale-authority, and context-pack tests before removing tombstones.

Every archive page must display a historical banner, replacement link when one exists, archive
reason, and last authoritative revision. Archive snippets are not required to parse as current Ash
and must not be copied into productive generated packs.

## 7. Audit, Deprecation, and Removal Work

This workstream extends Phase 201 rather than reopening it or replacing its evidence.

### 7.1 Audit method

For each canonical rule or carrier:

1. identify the current Rust implementation symbols through language-aware search;
2. identify tests, examples, diagnostics, and runtime artifacts that claim the behavior;
3. classify alignment as `implements`, `partially-implements`, `conflicts`, `orphaned`, or
   `unmapped`;
4. classify stale mechanisms as `delete`, `fold-into-target`, `retain-private`, or `needs-decision`;
5. record behavior-level evidence, not only vocabulary scans; and
6. create a task file before any implementation change.

### 7.2 Removal gates

- `delete` requires a failing absence/negative test or equivalent reachability evidence before
  removal, plus proof that no canonical rule depends on the mechanism.
- `fold-into-target` requires parity tests against the target primitive before the old boundary is
  removed.
- `retain-private` requires an implementation note and a gate preventing it from becoming public
  semantic authority.
- `needs-decision` blocks proof claims for the affected rule but does not license speculative code
  deletion.

The audit output must reconcile TASK-1971/TASK-1972 and any remaining Phase 201 worktree changes
before assigning overlapping removals.

## 8. Ash Core/CPS Calculus

### 8.1 Role

The programme should formalize a small calculus—provisionally `λAsh-CPS`—for the distinctive
semantic mechanisms of target Ash. Like the small Rust-inspired calculus in the original Verus
paper, it is a model for the properties Ash needs to state and prove, not an attempt to formalize
every parser, tooling, host, or optimizer detail. (`λRust` itself originates in RustBelt; later
VerusBelt work connects Verus foundations to it. PLAN-202 should preserve that literature
distinction.)

The calculus suite explains the CPS layer of the semantic pivot:

```text
surface Ash --lowering--> Ash-Core --lowering--> CPS --realization--> Rust Engine executor
                                                  |
                                                  +--mathematical semantics--> λAsh-CPS₀ → λAsh-Effect
                                                               --observable projection--> results/traces
```

### 8.2 Kernel `λAsh-CPS₀`

The initial calculus has the following domain; it does not cover the full target runtime:

- structured identifiers and operation/row-item identities;
- atoms and inert values: variables, unit, integers, booleans, strings, tuples/records, ADT
  constructors, and mathematical closures;
- terms: `LetVal`, total `LetPrim`, `LetCont`, `LetContCall`, `Jump`, `Call`, `If`, `Match`,
  `Return`, and structured `Trap`;
- function and continuation types with a fixed answer type;
- closed concrete rows and affine continuations; and
- configurations separating syntax from value environments, continuation stores, and affine-use
  state.

The kernel has deterministic small-step semantics. A big-step relation is derived only for the
terminating fragment; it is not a competing primary semantics.

### 8.3 Effect extension `λAsh-Effect`

Define the next complete conservative extension, then pursue its proof obligations independently:

- `Raise`, `Handle`, administrative discharge records, and deep affine resume;
- an ordered stack containing both handler and provider frames;
- one innermost-first lookup relation in which either frame kind may shadow the other;
- provider execution as an abstract labelled external transition;
- row subtraction/residual-row rules;
- affine source-handler continuation consumption and any separately specified multi-shot policy,
  both with explicit state; and
- missing discharge as a structured terminal outcome, never ordinary stuckness.

Rows remain requirements, never authority. Runtime authority is represented by admitted facts and
ordered frames. Relative to a fixed provider oracle, the admitted effect fragment should be
deterministic. Any remaining nondeterminism must be owned by an explicitly named external/helper
relation with a bounded allowed-outcome set.

### 8.4 Later extensions

Stage these independently after `λAsh-Effect`:

- recursion and mutually recursive encodings;
- lazy/memo thunks and memo-store behavior;
- dynamic contracts, snapshots, and predicate faults;
- trace facts, temporal monitors, provenance, and execution records;
- process/channel/concurrency semantics; and
- open rows, aliases, effect groups, and inference.

### 8.5 Theorem ladder

The calculus work is staged:

1. syntax and state well-formedness;
2. deterministic lookup and primitive evaluation;
3. substitution/environment/row normalization lemmas;
4. kernel progress to a step, `Return`, or explicit `Trap`;
5. kernel type/row preservation and small-step determinism;
6. kernel big-step/small-step correspondence for terminating programs;
7. frame lookup selects the greatest matching frame index;
8. handler/provider shadowing and missing-discharge correctness;
9. affine consumption safety and multi-shot-pure row legality;
10. effect-fragment determinism relative to a fixed provider oracle;
11. cumulative obligation/provenance/trace/effect preservation in later extensions;
12. terminal execution-record projection correctness;
13. Core-to-CPS and then surface-to-Core lowering preservation; and
14. bounded-nondeterminism conformance for helper-owned runtime boundaries.

### 8.6 Explicit exclusions from the kernel/effect calculus

- lexer/parser recovery, formatting, macro hygiene, and LSP behavior;
- floats and underspecified overflow/division semantics until primitive behavior is frozen;
- concrete host-provider internals, FFI, OS behavior, and sandbox implementation;
- scheduler fairness, real-time liveness, distributed execution, and network semantics;
- optimizer, bytecode, JIT, and machine-code correctness;
- unrestricted temporal logic and full monitor liveness proofs;
- the future Ash proposition universe, proof-term checker, and proof language itself;
- full compatibility semantics for forms removed by Phase 201.

Concrete Rust storage choices such as `Rc<RefCell<_>>`, `HashMap`, timestamps, captured-environment
layout, and serialization do not appear in the calculus. Checked view/refinement functions relate
them to mathematical closures and state where later implementation proofs require it.

Excluded behavior may be modeled later by boundary assumptions or separate calculi. It must not be
smuggled into the trusted base as an unexplained Rust helper.

## 9. Traceability Schema

### 9.1 Stable nodes

Semantic traceability uses stable, content-independent identifiers. Initial namespaces are:

| Namespace | Meaning | Example |
|---|---|---|
| `REQ` | user/system requirement or invariant | `REQ-ROW-NONAUTHORITY` |
| `GRAM` | grammar production | `GRAM-FN-DECL` |
| `TYPE` | typing/effect rule | `TYPE-CORE-CALL` |
| `LOWER` | lowering rule | `LOWER-CORE-CPS-CALL` |
| `SEM` | operational rule/theorem | `SEM-CPS-JUMP` |
| `OBS` | observable projection rule | `OBS-TERMINAL-RETURN` |
| `CONF` | conformance case | `CONF-ROW-DEDUP-001` |
| `IMPL` | implementation symbol or adapter | `IMPL-CORE-NORMALIZE-ROW` |
| `TEST` | deterministic/property/fuzz test | `TEST-CORE-ROW-IDEMPOTENT` |
| `PROOF` | theorem, obligation, or proof artifact | `PROOF-ROW-NORMALIZE-IDEMPOTENT` |

### 9.2 Edges

The manifest supports controlled relations:

```text
defines, refines, requires, lowers_to, projects_to,
implemented_by, tested_by, proved_by, assumes, supersedes
```

Nodes point to stable document anchors or symbol identities, not line numbers. Rust symbols include
crate/module/item identity and a source fingerprint; proof artifacts include provider, version,
options, assumptions, model identity, implementation revision, and artifact hash.

### 9.3 Coverage and policy

Validation generates bidirectional matrices:

- every A1 semantic rule has implementation/test/proof status or an explicit planned/excluded reason;
- every public semantic implementation symbol maps to an A1/A2 owner or is declared private
  machinery;
- every proof declares the rule/model and implementation revision it establishes;
- every archived node has a supersession or historical-only reason; and
- no generated context pack contains unresolved authority conflicts.

Coverage status is not binary. Use `specified`, `implemented`, `tested`, `modelled`, `proved`,
`assumed`, `deferred`, `refuted`, and `not-applicable` as distinct facts.

Verus remains an experimental assurance track. A deferred or pilot proof obligation records useful
future work but does not block executable realization; only an artifact with recorded verified
outcome may be reported as proved.

## 10. Verus Pilot 1: Core Row Algebra

### 10.0 Integration strategy

The repository currently has no `verus`, `cargo-verus`, `vstd`, or Verus package configuration.
TASK-1991 must therefore prove toolchain compatibility rather than assuming the workspace Rust
version can be verified directly.

The preferred integration is a small Verus-compatible kernel crate or module that production Ash
calls, with audited adapters to richer serde/runtime carriers. Directly annotating dependency-heavy
`ash-core`/`ash-interp` code is an allowed experiment, but proof-only duplicate models are not
production implementation verification. They are model/differential evidence until a checked
refinement bridge connects them to production Rust.

### 10.1 Target

- `crates/ash-core/src/core_ash_typecheck.rs::normalize_core_row`
- `crates/ash-core/src/core_ash_typecheck.rs::core_row_included_in`
- only closed rows initially; open tails, structural environment equivalence, output ordering, and
  diagnostic payload equivalence enter later slices.

### 10.2 Model and theorem set

Define a Verus `spec` model of row item identity and canonical set-like row semantics. Prove:

- normalization preserves row membership;
- normalization removes exact duplicates;
- normalization is idempotent;
- normalization preserves stable first-occurrence order;
- normalization never increases item count;
- closed-row inclusion is reflexive and transitive;
- inclusion truth and membership are invariant under normalization and item permutation, without
  claiming ordered output or `missing_items` payload equality; and
- ambiguous group references are rejected rather than interpreted as authority or silently erased.

The executable Rust functions must be connected to the spec model by an explicit view/refinement
function; tests alone do not establish that correspondence.

### 10.3 Trusted boundary

The pilot separates logical assumptions from trusted tooling and records:

- Verus verifier/VC generator release, `cargo-verus` or build wrapper, Rust toolchain and codegen/LLVM
  relationship, Z3 version/options/resource limits, `vstd` revision, pinned artifacts, and exact
  verification command;
- every `assume`, axiom, external body/specification/item, external trait implementation, trusted
  library specification, unsupported Rust feature, and production adapter;
- the translation/view between `CoreRow`/`CoreRowItem` and the mathematical model; and
- which public functions remain outside the verified fragment.

### 10.4 Acceptance and stop/go gate

Accept the pilot only if:

- proofs run reproducibly in CI from a pinned toolchain;
- the admitted assumptions and unsupported code are enumerated automatically;
- existing property tests remain and cross-check the executable/spec view;
- proof diagnostics identify source obligations usefully enough for an agent repair loop; and
- maintenance cost is measured after one deliberate representation-preserving refactor.

Stop expansion if the model duplicates production types without a checked correspondence, the proof
depends on broad `external_body` assumptions, or ordinary Ash toolchain use becomes coupled to the
Verus toolchain.

## 11. Verus Pilot 2: Frame-Ordered Operation Dispatch

### 11.1 Target

- `crates/ash-core/src/cps.rs::HandlerChain::find_operation_frame`
- the selection boundary used by `crates/ash-interp/src/cps/mod.rs::eval_raise`
- selection/shadowing only; provider execution, handler-clause evaluation, resume behavior, and
  other evaluator effects remain outside the pilot theorem.

### 11.2 Model and theorem set

Define a Verus spec model:

```text
Frame = Handler(operation, payload) | Provider(operation, payload)
lookup(frames, operation) = greatest matching frame index, if one exists
```

Prove:

- lookup returns `None` exactly when no frame matches;
- any returned index is in bounds and matches the requested structured operation identity;
- no later/inner frame matches;
- the result kind and payload come from exactly the selected frame;
- pushing a matching handler or provider shadows every previous match;
- pushing a nonmatching frame preserves the previous result;
- handler and provider frames obey the same ordering rule; and
- `eval_raise` branches only through this lookup result, with no second handler-first/provider-second
  search.

This pilot must cite the final `λAsh-Effect` lookup/shadowing rules. It cannot derive semantics from
the current vector order or evaluator branches alone.

### 11.3 Acceptance and stop/go gate

Pilot 2 begins only after Pilot 1 establishes a usable toolchain and the effect-calculus
lookup/shadowing rules are frozen. Accept it only if:

- the proof establishes a real Rust-to-canonical-model relation;
- the production lookup is verified directly or delegates to a verified kernel used in production;
- existing inner-provider/outer-handler and inner-handler/outer-provider fixtures pass;
- holes, assumptions, external bodies, and provider versions appear in traceability evidence;
- an LLM can reproduce or repair at least one intentionally removed lemma using checker feedback,
  with no unchecked generated evidence admitted; and
- a mutation from innermost-first to outermost-first search fails verification.

Execution-record terminal projection remains the next cross-layer pilot candidate after PLAN-202.
It depends on later calculus extensions for traces, obligations, provenance, and observable record
projection, whereas frame lookup exercises genuine operational semantics immediately after the row
algebra pilot.

### 11.4 Long-term grammar and semantics proof portfolio

The two pilots are entry points, not the end state. If their gates succeed, follow-on proof work
should cover:

- a machine-readable, macro-free post-expansion grammar with complete token consumption, AST
  well-formedness, bounded span correctness, and parse/print round-trip modulo trivia;
- Core-to-CPS lowering and validator soundness for the admitted calculus;
- production one-step dispatch/evaluation refinement to `λAsh-CPS` rules;
- terminal execution-record projection after trace/provenance/obligation extensions are formalized;
  and
- later macro expansion, recovery parsing, concurrency, and temporal semantics as separately scoped
  theorem families.

Verus should primarily verify executable Rust algorithms and their refinement contracts. Lean may
remain useful for inductive metatheory such as preservation, progress, and relational
correspondence, but both must consume the same canonical rule/trace manifest so they cannot become
competing semantic authorities.

## 12. Parallel Tracks and Later Ash Proof-System Handoff

After the canonical manifest and applicable rule owners are frozen, work may proceed in parallel:

| Track | May start after | Produces |
|---|---|---|
| documentation migration/archive | inventory + manifest schema | compact canonical corpus and inert archive |
| semantic/code audit and cleanup | rule ownership for audited subsystem | deprecation/removal tasks and behavior gates |
| CPS calculus | Core/CPS syntax and vocabulary freeze | calculus, theorem ladder, conformance obligations |
| Verus toolchain/Pilot 1 | canonical metadata/proof schema + row-rule ownership | toolchain/TCB evidence and algebraic proof experience; does not wait for the effect calculus |
| Verus Pilot 2 | `λAsh-Effect` lookup rules + Pilot 1 go decision | operational shadowing proof and LLM pilot |
| Ash `spec`/`proof` design | calculus + evidence/hole policy + pilot reports | separate approved design; no implementation in PLAN-202 |
| LLM proof synthesis research | stable obligation/proof artifact schema | checked proposal/repair loop and benchmark evidence |

The Ash design track must use evidence from the pilots to decide predicate fragments, SMT/Lean/Verus
provider roles, proof scripting, termination, erasure, holes, trust policy, and diagnostics. It must
not assume that Verus syntax or Rust-specific linear ghost mechanisms transfer directly to Ash.

## 13. Task Sequence

| Task | Deliverable | Depends on |
|---|---|---|
| TASK-1983 | Programme plan packet | Current-worktree inspection; no Phase 201 completion dependency |
| TASK-1984 | Corpus authority and conflict inventory | TASK-1983 |
| TASK-1985 | Canonical manifest, metadata, and validation schema | TASK-1984 |
| TASK-1986 | Canonical core reconciliation and promotion | TASK-1985 |
| TASK-1987 | Archive, redirect, and generated-context migration | TASK-1986 |
| TASK-1988 | Semantic implementation audit and deprecation/removal packet | TASK-1986; Phase 201 reconciliation |
| TASK-1989 | Ash Core/CPS calculus freeze | TASK-1986 |
| TASK-1990 | Traceability graph and coverage gates | TASK-1985; TASK-1989 |
| TASK-1991 | Verus toolchain, TCB, and CI isolation spike | TASK-1985 proof-artifact schema |
| TASK-1992 | Verus Pilot 1: Core row algebra | TASK-1991; canonical row-rule owner from TASK-1986 |
| TASK-1993 | Verus Pilot 2: frame-ordered operation dispatch | TASK-1992; effect-calculus freeze |
| TASK-1994 | Programme closeout and Ash proof-design handoff | TASK-1987 through TASK-1992; TASK-1993 when authorized by the Pilot 1 go decision |

Each implementation task must have its own task file before work begins and must update
`CHANGELOG.md`. Rust tasks use TDD, property tests where algebraic invariants apply, Rust-aware code
review, and the full local gate required by `AGENTS.md`.

## 14. Programme Completion Evidence

PLAN-202 is complete only when evidence proves:

- one canonical manifest resolves every productive grammar/type/lowering/Core/CPS/runtime authority
  claim in scope;
- generated agent packs select canonical sources and reject archive/research authority leakage;
- superseded documentation is archived or derived with redirects and no productive inbound routing;
- every deprecation/removal item has semantic ownership and behavior-level evidence;
- the Core/CPS calculus and theorem ladder are frozen with explicit exclusions;
- traceability matrices link canonical rules to implementation/tests/proofs or explicit gaps;
- the Verus toolchain and pilot sequence reaches a documented go/no-go result with enumerated
  assumptions and reproducible commands; Pilot 2 passes when authorized, or is conditionally
  skipped with retained obligations and an owned remediation decision when an earlier no-go stops
  expansion; and
- the later Ash `spec`/`proof` design has an evidence-backed handoff without implementing proof
  syntax prematurely.

## 15. Initial References

Local authority and implementation context:

- [Ash Specification Orientation Index](../spec/SPEC-INDEX.md)
- [SPEC-071: Reference Corpus Metadata and Maintenance](../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md)
- [DESIGN-035: Documentation Corpus Governance](../design/DESIGN-035-DOCUMENTATION-CORPUS-GOVERNANCE.md)
- [DESIGN-042: Reference Corpus and Documentation Governance](../design/DESIGN-042-REFERENCE-CORPUS-AND-DOCUMENTATION-GOVERNANCE.md)
- [Ash Canonical Core](../spec/CANONICAL-CORE.md) (current semantic and proof-facing authority)
- [Historical Formalization Boundary](../reference/formalization-boundary.md) (migration rationale only)
- [Verification and Prover Integration Survey](../reference/verification-and-prover-integration-survey.md)
- [SPEC-099b: Target Operational Semantics](../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)
- [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
- [PLAN-159: CPS IR Interpreter](PLAN-159-CPS-IR-INTERPRETER.md)
- [PLAN-201: Semantic Cleanup Follow-up](PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md)

Verus sources to be frozen into the task-specific literature packet:

- Verus guide: <https://verus-lang.github.io/verus/guide/>
- Verus repository: <https://github.com/verus-lang/verus>
- Lattuada et al., “Verus: Verifying Rust Programs using Linear Ghost Types”:
  <https://arxiv.org/abs/2303.05491>
- VerusBelt / `λRust` foundation: <https://iris-project.org/pdfs/2026-pldi-verusbelt.pdf>
- Verus publications and projects: <https://verus-lang.github.io/verus/publications-and-projects/>
