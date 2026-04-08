# Formalization Boundary and Proof Targets

## Status

TASK-431 boundary refresh.

## Purpose

This note freezes the current formalization boundary for future Lean work and other proof-facing
reference artifacts.

It exists so proof work, canonical corpus design, canonical result-format work, and later differential-conformance work all target
one explicit authority split:

- canonical semantic and observable specifications define language truth,
- source and handoff contracts define authoritative layer boundaries,
- the implementation-conformance contract defines how implementations are compared against that
  truth, and
- planning, closeout, and evidence artifacts remain historical or migration guidance only.

## Authority Hierarchy

Future Lean and reference work should use the following hierarchy.

1. Canonical semantic and observable specifications.
2. The implementation-conformance contract for cross-implementation comparison rules.
3. Authoritative source and handoff contracts for their own layer-specific boundaries.
4. Historical planning, closeout, and runtime-evidence artifacts.

If any lower layer disagrees with a higher layer, the higher layer wins.

## Canonical Semantic and Observable Corpus

Lean should treat the following documents as the canonical semantic and observable corpus for Ash:

- [SPEC-001: Intermediate Representation](../spec/SPEC-001-IR.md)
- [SPEC-003: Type System](../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-004: Operational Semantics](../spec/SPEC-004-SEMANTICS.md)
- [SPEC-020: Algebraic Data Types](../spec/SPEC-020-ADT-TYPES.md)
- [SPEC-021: Runtime Observable Behavior](../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
- [SPEC-022: Workflow Typing](../spec/SPEC-022-WORKFLOW-TYPING.md)
- [SPEC-025: Small-Step Operational Semantics](../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [Semantic Execution Record Contract](semantic-execution-record-contract.md)
- [Canonical IR Semantics Corpus](canonical-ir-semantics-corpus.md)
- [Canonical Semantics Result Format](canonical-semantics-result-format.md)

Within that corpus, the authority roles are split explicitly:

- [SPEC-004: Operational Semantics](../spec/SPEC-004-SEMANTICS.md) is the normative owner of
  big-step / terminal workflow meaning, including `Return(...)` and `Reject(...)` outcomes and the
  terminal semantic dimensions carried by `Ω`, `π`, trace `T`, and terminal effect classification.
- [SPEC-025: Small-Step Operational Semantics](../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
  is the normative owner of the workflow-first small-step judgment, canonical configuration
  vocabulary, helper-owned atomicity boundaries, the frozen state taxonomy, and the terminal
  projection contract back to SPEC-004.
- [Semantic Execution Record Contract](semantic-execution-record-contract.md) is the normative owner
  of the runtime-facing cumulative semantic packaging contract for `Ω`, `π`, `T`, `ε̂`, the
  runtime-facing execution phase taxonomy, and exact terminal projection from a canonical
  execution-record carrier back to `SPEC-004` workflow outcomes and completion-style payload
  projection.
- [Canonical IR Semantics Corpus](canonical-ir-semantics-corpus.md) is the normative owner of the
  file-backed canonical case inventory and case-selection policy future conformance harnesses and
  reference implementations must share.
- [Canonical Semantics Result Format](canonical-semantics-result-format.md) is the normative owner of
  the machine-readable expected-result envelope and exact-versus-allowed-set comparison artifact
  shape for those canonical cases.
- [SPEC-021: Runtime Observable Behavior](../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md) is the
  normative owner of user-visible and tooling-visible projections of runtime behavior.
- [SPEC-003: Type System](../spec/SPEC-003-TYPE-SYSTEM.md) and
  [SPEC-022: Workflow Typing](../spec/SPEC-022-WORKFLOW-TYPING.md) own the typing judgments and
  typed-fragment assumptions needed by later preservation or progress-style theorems.
- [SPEC-001: Intermediate Representation](../spec/SPEC-001-IR.md) and
  [SPEC-020: Algebraic Data Types](../spec/SPEC-020-ADT-TYPES.md) own the canonical workflow,
  expression, value, constructor, and pattern forms referenced by both the big-step and small-step
  semantics.

These documents are the theorem subjects for semantic formalization. Lean should model them
 directly rather than treating current Rust carrier shapes or older planning prose as the meaning of
Ash.

## Implementation-Conformance Authority

[SPEC-026: Implementation Conformance Contract](../spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md)
is authoritative for cross-implementation comparison only.

It is not a replacement semantics document and it is not part of the theorem-defining semantic
corpus above. Instead, it freezes:

- the conformance surfaces that implementations may be judged on,
- the preserved dimensions required at each surface,
- the bounded nondeterminism allowances used during comparison, and
- the comparison rules future corpus/harness work must follow.

Accordingly:

- semantic theorems should be stated first against the canonical semantic corpus;
- conformance obligations should then cite SPEC-026 when comparing Rust, Lean, or alternate
  implementations against that corpus;
- canonical corpus fixtures and expected-result artifacts should then be built against the shared
  corpus/result-format references rather than per-harness local conventions;
- proof artifacts must not silently move semantic authority from SPEC-004 or SPEC-025 into
  SPEC-026.

## Authoritative Source and Handoff Contracts

Lean should also treat the following documents as authoritative for their own source-level or
layer-boundary contracts:

- [SPEC-002: Surface Language](../spec/SPEC-002-SURFACE.md)
- [SPEC-005: CLI Specification](../spec/SPEC-005-CLI.md)
- [SPEC-011: REPL](../spec/SPEC-011-REPL.md)
- [SPEC-016: Output Capabilities](../spec/SPEC-016-OUTPUT.md)
- [Surface-to-Parser Contract](surface-to-parser-contract.md)
- [Parser-to-Core Lowering Contract](parser-to-core-lowering-contract.md)
- [Type-to-Runtime Contract](type-to-runtime-contract.md)
- [Runtime Observable Behavior Contract](runtime-observable-behavior-contract.md)

These contracts remain authoritative for syntax, lowering, CLI/REPL boundaries, and runtime
handoff surfaces. They do not replace the canonical semantic corpus, but they do constrain how Lean
or other proof-facing artifacts model source forms and cross-layer transitions.

## Historical, Planning, and Evidence Artifacts

The following sources remain useful, but they are not canonical semantic authority:

- planning artifacts under `docs/plan/`, including phase plans, task files, audits, and closeout
  notes;
- exploratory and accepted design/evidence artifacts under `docs/ideas/`, including
  [MCE-005: Small-Step Semantics](../ideas/minimal-core/MCE-005-SMALL-STEP.md),
  [MCE-006: Small-Step ↔ IR Execution](../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md), and
  [MCE-007: Full Layer Alignment](../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md);
- the old reference interpreter sketch at
  [Lean Reference Interpreter](../spec/SPEC-021-LEAN-REFERENCE.md).

Lean may consult these artifacts for rationale, migration context, or current implementation-evidence
boundaries, but not as a substitute for the canonical specs.

In particular:

- accepted MCE notes explain why the current small-step and runtime-alignment story looks the way it
  does;
- task files explain intended follow-on work;
- runtime-evidence notes explain what current Rust realizations do or do not yet realize;
- none of those artifacts override the semantics frozen by SPEC-004, SPEC-025, or SPEC-021.

## Lean Modeling Guidance

Future Lean work should follow these rules.

1. Model the canonical semantic and observable corpus directly.
2. Use source and handoff contracts as authoritative assumptions about syntax, lowering, and layer
   boundaries.
3. Treat SPEC-026 as the comparison contract for implementation conformance, not as the source of
   semantic meaning.
4. Treat task files, phase plans, MCE closeouts, and implementation evidence as historical guidance
   or proof-planning context only.
5. Avoid coupling definitions or theorem statements to current Rust module names, enum layouts,
   helper function names, or runtime storage details unless a canonical spec explicitly freezes that
   surface.

This means Lean should not re-infer Ash semantics from current code shape, nor from older big-step-
only proof-target sketches. The current canonical semantic corpus is now explicitly big-step,
small-step, and observable, with implementation conformance defined as a downstream comparison
layer.

## Proof-Facing Theorem Targets Over the Canonical Corpus

The theorem targets below are targets for future proof work. They are not claims that the repo has
already mechanized or discharged them.

### 1. Terminal Projection and Big-Step / Small-Step Correspondence

The first correspondence target is the terminal projection from admitted small-step terminal
configurations to authoritative SPEC-004 outcomes.

Primary theorem target:

- if a complete admitted small-step execution reaches terminal configuration `κt`, then
  `project(κt)` is exactly the authoritative [SPEC-004](../spec/SPEC-004-SEMANTICS.md) workflow
  outcome for the same canonical input.

Proof-facing obligations included in that target:

- `Returned(v, Ω', π', T, ε̂')` projects to `Return(v, eff', T, Ω', π')`;
- `Rejected(err, Ω', π', T, ε̂')` projects to `Reject(err, eff', T, Ω', π')`;
- the terminal effect classification `eff'` is the correct projection of terminal `ε̂'`;
- the projection preserves the owning success-versus-rejection boundary fixed by SPEC-004.

The execution-record contract in
[Semantic Execution Record Contract](semantic-execution-record-contract.md) packages the same
projection target as a runtime-facing cumulative-state record. Future proof and conformance work
should therefore treat:

- `SPEC-025` terminal configurations as the canonical semantic source for the projection theorem,
  and
- the execution-record contract as the canonical runtime-facing packaging target for implementations
  that wish to expose or preserve those same cumulative semantic dimensions directly.

Planned follow-on correspondence targets may strengthen this into execution-level soundness and, for
admitted fragments, converse reconstruction results from big-step derivations to terminal small-step
executions. Those stronger completeness-style results remain staged work rather than already-frozen
requirements for this note.

### 2. Progress-or-Blocked Classification Goals

For the admitted small-step fragment, the primary nonterminal classification target is:

- every admitted running configuration is either
  1. progress-capable now,
  2. blocked/suspended at an explicit helper-owned or runtime-owned wait boundary, or
  3. outside the admitted semantic fragment and therefore invalid/inadmissible.

Equivalently, future proof work should not treat ordinary nonterminal stuck states as part of the
admitted semantics.

When this theorem is specialized to typed fragments, the typing assumptions must come from
[SPEC-003](../spec/SPEC-003-TYPE-SYSTEM.md) and
[SPEC-022](../spec/SPEC-022-WORKFLOW-TYPING.md), not from ad hoc proof-local typing notions.

### 3. Deterministic-Fragment Determinism Targets

Future proofs should isolate deterministic fragments of the corpus and prove single-valued behavior
there.

At minimum, the deterministic-fragment targets are:

- pure expression determinism for the `Γ ⊢e expr ⇓ v` judgment where SPEC-004 already intends a
  deterministic result;
- pure pattern determinism for the `Γ ⊢p pat ⇐ v ⇓ ΔΓ` judgment on admissible patterns;
- deterministic small-step evolution for workflow fragments whose next transition is not delegated to
  a helper contract that admits bounded nondeterminism;
- uniqueness of terminal projection for deterministic complete executions.

These determinism targets must stay honest about excluded fragments. `Par`, helper-owned receive
selection, and other helper-bounded choices are not to be misclassified as globally deterministic
when the canonical corpus intentionally leaves a bounded set of outcomes open.

### 4. Helper-Bounded Nondeterminism Obligations

The current corpus explicitly permits bounded nondeterminism only at designated helper-owned
boundaries. Future proofs should package that as an obligation rather than leaving it informal.

The core obligation is:

- any semantic nondeterminism in the admitted corpus must be owned by a helper or concurrency
  boundary that the canonical specs already admit, and every realized branch/outcome must remain
  within the helper-bounded set authorized by the corpus.

At minimum this applies to:

- `Par` interleavings and helper-backed terminal aggregation;
- `receive` selection, timeout, fallthrough, and blocking outcomes;
- runtime-owned control/completion observation boundaries where the canonical runtime contract keeps
  behavior helper-owned rather than surface-stepped.

This theorem family should connect directly to
[SPEC-026](../spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md): proof work establishes the admitted
semantic set, and conformance work later checks that implementations stay inside that set.

### 5. Preservation of Cumulative Semantic Dimensions

Future correspondence work should make preservation and reconstruction of the cumulative semantic
carriers explicit.

The proof targets are:

- stepwise preservation/incorporation of obligation state `Ω`;
- stepwise preservation/incorporation of provenance state `π`;
- stepwise preservation/incorporation of cumulative trace `T` together with per-step `ΔT` labels;
- stepwise preservation/incorporation of cumulative effect-summary carrier `ε̂` together with
  per-step `δε` labels;
- terminal preservation strong enough that small-step execution reconstructs the same terminal
  semantic dimensions reported by SPEC-004.

For proof planning, these four dimensions should be treated cumulatively and jointly rather than as
optional commentary. Future work may introduce better execution-record packaging for them, but the
semantic preservation target already exists at the corpus level.

That packaging target is now frozen by
[Semantic Execution Record Contract](semantic-execution-record-contract.md). Future runtime work may
adopt it conservatively in stages, but semantic proofs and conformance artifacts should not treat
weaker retained-completion or coarse outcome-state slices as equivalent to the full canonical
execution record unless the weaker status is stated explicitly.

### 6. Secondary Typed and Structural Targets

After the correspondence and classification targets above, later proof work may stage additional
results such as:

- typed preservation across admitted workflow steps;
- admissibility lemmas for specification-only residual forms introduced by SPEC-025;
- helper-interface soundness lemmas for receive selection, parallel aggregation, obligation
  transition, and spawned-child completion projection;
- lowering soundness from source-level contracts into canonical IR when future Lean work reaches the
  parser/lowering boundary.

These remain legitimate future theorem targets, but the primary TASK-431 packaging goal is to make
the current big-step / small-step / observable corpus proof-usable first.

## Implementation Conformance Obligations Are Separate

The theorem targets above are semantic statements about the canonical corpus itself. They are not
identical to implementation-conformance obligations.

Implementation-conformance obligations are owned by
[SPEC-026](../spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md) and instead ask whether a concrete Rust,
Lean, or alternate implementation preserves the required projection for a declared surface.

That distinction must remain explicit:

- corpus theorems ask what is true of the canonical semantics;
- conformance obligations ask whether an implementation stays within the admitted surface defined by
  those semantics;
- observable comparison obligations ask whether surfaced behavior matches SPEC-021 where that
  surface is the declared comparison target.

Proof artifacts and future differential-testing harnesses must therefore declare which layer they are
operating on instead of implicitly mixing theorem proving with implementation certification.

## Out-of-Scope and Future Work Boundary

This note does not claim or require any of the following to be already done:

- completed Lean mechanization of the listed theorem targets;
- full current-Rust conformance certification on every SPEC-026 surface;
- a fairness theorem or concrete scheduler proof;
- expression-level micro-stepping in SPEC-025 v1;
- one concrete runtime carrier or storage layout for `Ω`, `π`, `T`, or `ε̂`;
- full retained-completion parity or full runtime implementation of the already-frozen execution-
  record contract in later Phase 67 tasks;
- JIT-specific proof work.

Those are either downstream tasks, implementation work, or intentionally deferred research.

## Contract Hygiene

- Canonical specs define semantic truth.
- SPEC-026 defines implementation comparison against that truth; it does not replace it.
- Source and handoff contracts define authoritative layer boundaries.
- Historical planning and evidence artifacts remain non-canonical guidance.
- Lean should formalize the canonical corpus directly and cite lower-authority artifacts only for
  migration context.
- Recoverable failure remains canonical only as explicit `Result` handling.
- `catch` is not part of the canonical language contract.
