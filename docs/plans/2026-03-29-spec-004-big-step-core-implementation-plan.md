# SPEC-004 Big-Step Core Semantics Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Revise `SPEC-004` into a complete, proof-suitable big-step core semantics for Ash while preserving explicit runtime authority and execution-neutrality.

**Architecture:** First normalize the semantic backbone so every judgment, algebraic operator, and failure category is declared explicitly. Then complete the pure expression/pattern core, extract runtime-bearing helper contracts into explicit relations, and finish with determinism, invariants, and cross-spec proof-boundary cleanup.

**Tech Stack:** Markdown specs and planning docs; canonical semantics in `docs/spec/SPEC-004-SEMANTICS.md` with alignment checks against `docs/spec/SPEC-001-IR.md`, `docs/reference/formalization-boundary.md`, `docs/spec/SPEC-013-STREAMS.md`, `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`, `docs/spec/SPEC-018-CAPABILITY-MATRIX.md`, and `CHANGELOG.md`.

---

## Planning assumptions

- This is a docs-first semantics refinement task, not a Rust or Lean implementation task.
- `SPEC-004` must remain execution-neutral and runtime-authoritative.
- The pure fragment should be strong enough for immediate Lean formalization.
- Provider/runtime helpers may remain abstract, but not semantically vague.
- Cross-spec name alignment matters more than preserving current local shorthand such as `eval(...)` and `bind(...)`.

## Sequencing rules

1. Define or normalize semantic vocabulary before editing any inference rule.
2. Replace implicit `eval(...)` / `bind(...)` use with explicit judgments before adding new meta-properties.
3. Classify deterministic vs nondeterministic fragments only after helper contracts are explicit.
4. Do the cross-spec consistency pass after the main `SPEC-004` rewrite, not before.
5. Update `CHANGELOG.md` alongside the planning/spec artifacts.

## Verification defaults

- Focused `rg` checks for leftover implicit helper names such as `eval(`, `bind(`, and undefined helper relations.
- Manual cross-reference review across the touched spec set.
- `git diff --check`

---

### Task 1: Create the task-backed planning scaffold

**Files:**

- Create: `docs/plans/2026-03-29-spec-004-big-step-core-design.md`
- Create: `docs/plans/2026-03-29-spec-004-big-step-core-implementation-plan.md`
- Create: `docs/plan/tasks/TASK-350-revise-spec-004-to-complete-big-step-core-semantics.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing planning checklist**

Document the current blockers that make `SPEC-004` less than proof-suitable:

- missing explicit expression judgment,
- split pattern semantics,
- implicit propagation and failure ownership,
- helper contracts without judgment-shaped signatures,
- no centralized determinism boundary.

**Step 2: Verify RED**

Run:

```bash
rg -n "eval\(|bind\(|determin|nondetermin|Helper Relations|Pattern Matching Judgment|Expression Evaluation Judgment" docs/spec/SPEC-004-SEMANTICS.md
```

Expected: the current document still depends on mixed shorthand and lacks a centralized proof-shaped structure.

**Step 3: Save the design and implementation plan**

Create the design doc and this implementation plan with the final scope, sequencing, and proof-oriented constraints.

**Step 4: Create the task file**

Write `TASK-350` with description, requirements, red/green criteria, affected files, and completion checklist.

**Step 5: Update changelog bookkeeping**

Add an `[Unreleased]` entry describing the new planning/design work for proof-grade `SPEC-004` completion.

**Step 6: Verify planning artifacts exist**

Run:

```bash
ls docs/plans/2026-03-29-spec-004-big-step-core-design.md docs/plans/2026-03-29-spec-004-big-step-core-implementation-plan.md docs/plan/tasks/TASK-350-revise-spec-004-to-complete-big-step-core-semantics.md
```

Expected: all three files exist.

**Step 7: Commit**

```bash
git add docs/plans/2026-03-29-spec-004-big-step-core-design.md docs/plans/2026-03-29-spec-004-big-step-core-implementation-plan.md docs/plan/tasks/TASK-350-revise-spec-004-to-complete-big-step-core-semantics.md CHANGELOG.md
git commit -m "docs: plan proof-grade SPEC-004 completion"
```

---

### Task 2: Normalize the semantic backbone in `SPEC-004`

**Files:**

- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing structural checklist**

List the backbone gaps to close:

- no explicit effect-order subsection,
- no explicit trace-concatenation subsection,
- no explicit environment-extension subsection,
- no explicit runtime-failure categories subsection,
- only one broad big-step judgment section.

**Step 2: Verify RED**

Run:

```bash
rg -n "Effect Join|Trace and Concatenation|Environment Extension|Runtime Failure Categories|Expression Evaluation Judgment|Pattern Matching Judgment" docs/spec/SPEC-004-SEMANTICS.md
```

Expected: one or more sections are absent.

**Step 3: Add the minimal backbone structure**

Edit `docs/spec/SPEC-004-SEMANTICS.md` to add:

- effect order and join laws,
- trace concatenation,
- environment extension/shadowing,
- runtime failure categories,
- explicit subjudgments for workflow, expression, and pattern semantics.

**Step 4: Verify GREEN**

Run:

```bash
rg -n "Effect Order and Join|Trace and Concatenation|Environment Extension and Shadowing|Runtime Failure Categories|Expression Evaluation Judgment|Pattern Matching Judgment" docs/spec/SPEC-004-SEMANTICS.md
```

Expected: all backbone sections and judgment headers are now present.

**Step 5: Commit**

```bash
git add docs/spec/SPEC-004-SEMANTICS.md CHANGELOG.md
git commit -m "docs: normalize SPEC-004 semantic backbone"
```

---

### Task 3: Complete pure expression semantics

**Files:**

- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Review: `docs/spec/SPEC-001-IR.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing expression checklist**

Identify the expression forms still lacking one uniform semantics:

- literals (including literal container values),
- variables,
- variant construction,
- field access,
- index access,
- unary operators,
- generic binary operators,
- pure call evaluation,
- constructor,
- match,
- equality,
- boolean connectives,
- dynamic expression misuse ownership.

**Step 2: Verify RED**

Run:

```bash
rg -n "eval\(|CONSTRUCTOR|MATCH-HIT|MATCH-MISS|EXPR-" docs/spec/SPEC-004-SEMANTICS.md
```

Expected: expression semantics are still split between helper-style prose and partial formal rules.

**Step 3: Write the minimal complete expression section**

Add one canonical expression-judgment section and move constructor/match semantics into that structure. Keep runtime-owned dynamic failures explicit at the enclosing workflow boundary.

**Step 4: Verify GREEN**

Run:

```bash
rg -n "EXPR-LITERAL|EXPR-VAR|EXPR-VARIANT|EXPR-FIELD|EXPR-INDEX|EXPR-UNARY|EXPR-EQ|EXPR-BINARY|EXPR-AND|EXPR-OR|EXPR-CALL|EXPR-MATCH|Expression Evaluation Judgment" docs/spec/SPEC-004-SEMANTICS.md
```

Expected: the expression core is now defined in one place.

**Step 5: Commit**

```bash
git add docs/spec/SPEC-004-SEMANTICS.md CHANGELOG.md
git commit -m "docs: complete SPEC-004 expression semantics"
```

---

### Task 4: Complete pattern and binding semantics

**Files:**

- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Review: `docs/spec/SPEC-001-IR.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing pattern checklist**

List the pattern semantics gaps:

- no single pattern judgment used everywhere,
- split `bind(...)` semantics,
- unclear duplicate binder policy,
- no clear distinction between branch-local non-match and runtime rejection.

**Step 2: Verify RED**

Run:

```bash
rg -n "bind\(|Pattern Matching Judgment|PAT-|PatternBindFailure|PatternMatchFailure" docs/spec/SPEC-004-SEMANTICS.md
```

Expected: `bind(...)` still carries semantically significant behavior that is not fully judgment-shaped.

**Step 3: Write the minimal complete pattern section**

Add or normalize explicit rules for:

- wildcard,
- variable,
- literal,
- tuple/list,
- record,
- variant,
- optional rest-binding policy if retained.

State the duplicate-binder rule explicitly.

**Step 4: Verify GREEN**

Run:

```bash
rg -n "PAT-WILDCARD|PAT-BIND|PAT-LIT|PAT-LIST|PAT-RECORD|PAT-VARIANT|Pattern Matching Judgment" docs/spec/SPEC-004-SEMANTICS.md
```

Expected: one canonical pattern semantics is present.

**Step 5: Commit**

```bash
git add docs/spec/SPEC-004-SEMANTICS.md CHANGELOG.md
git commit -m "docs: complete SPEC-004 pattern semantics"
```

---

### Task 5: Formalize propagation and failure ownership

**Files:**

- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing propagation checklist**

Identify all still-implicit semantics around:

- left-to-right premise order,
- early rejection propagation,
- effect accumulation through failure,
- trace-prefix preservation,
- obligation/provenance state at failure,
- lookup-failure mapping.

**Step 2: Verify RED**

Run:

```bash
rg -n "Unless a rule states otherwise|propagat|Lookup Failure|Post-Lowering Assumptions|Runtime Conventions" docs/spec/SPEC-004-SEMANTICS.md
```

Expected: conventions are missing, scattered, or not yet strong enough.

**Step 3: Write the minimal conventions section**

Add formal propagation and lookup-failure conventions, including the static-vs-dynamic boundary for malformed states that still reach runtime.

**Step 4: Verify GREEN**

Run:

```bash
rg -n "Propagation Conventions|Lookup Failure Conventions|Post-Lowering Assumptions" docs/spec/SPEC-004-SEMANTICS.md
```

Expected: the document now has one normative home for propagation and failure ownership.

**Step 5: Commit**

```bash
git add docs/spec/SPEC-004-SEMANTICS.md CHANGELOG.md
git commit -m "docs: formalize SPEC-004 propagation and failure ownership"
```

---

### Task 6: Extract and normalize runtime helper contracts

**Files:**

- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Review: `docs/spec/SPEC-013-STREAMS.md`
- Review: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Review: `docs/spec/SPEC-018-CAPABILITY-MATRIX.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing helper-contract checklist**

Check whether each semantically significant helper currently states:

- signature,
- determinism status,
- failure mapping,
- preserved invariants.

Priority helpers:

- `lookup(C, cap)`
- `lookup(P, policy)`
- `select_receive_outcome(...)`
- `perform_action(...)`
- `check_obligation(...)`
- `combine_parallel_outcomes(...)`
- provenance helpers.

**Step 2: Verify RED**

Run:

```bash
rg -n "select_receive_outcome|perform_action|check_obligation|combine_parallel_outcomes|Helper Relations" docs/spec/SPEC-004-SEMANTICS.md
```

Expected: helper behavior is present but not yet uniformly contract-shaped.

**Step 3: Write the minimal helper-contract section**

Add a dedicated section describing each helper relation's domain, range, determinism, failure mapping, and required semantic laws.

**Step 4: Verify GREEN**

Run:

```bash
rg -n "Helper Relations|Receive Selection|Action Performance|Obligation Checking|Parallel Outcome Combination|Provenance" docs/spec/SPEC-004-SEMANTICS.md
```

Expected: helper-backed rules now cite explicit contracts.

**Step 5: Commit**

```bash
git add docs/spec/SPEC-004-SEMANTICS.md CHANGELOG.md
git commit -m "docs: extract SPEC-004 helper contracts"
```

---

### Task 7: Add determinism, invariants, and proof-target cleanup

**Files:**

- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/reference/formalization-boundary.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing theorem-boundary checklist**

Record what remains implicit:

- deterministic fragment,
- nondeterministic fragment,
- intended proof targets,
- conformance notes for helper-backed runtime relations.

**Step 2: Verify RED**

Run:

```bash
rg -n "Determinism|Nondeterminism|Semantic Invariants|Proof Targets|Conformance" docs/spec/SPEC-004-SEMANTICS.md docs/reference/formalization-boundary.md
```

Expected: these theorem and conformance boundaries are absent or incomplete.

**Step 3: Write the minimal meta-property sections**

Add to `SPEC-004`:

- determinism/nondeterminism section,
- semantic invariants section,
- proof-target / conformance notes.

Adjust `formalization-boundary.md` only if the proof-target list needs explicit alignment with the new `SPEC-004` structure.

**Step 4: Verify GREEN**

Run:

```bash
rg -n "Determinism and Nondeterminism|Semantic Invariants|Proof Targets|Conformance" docs/spec/SPEC-004-SEMANTICS.md docs/reference/formalization-boundary.md
```

Expected: theorem scope is now explicit.

**Step 5: Commit**

```bash
git add docs/spec/SPEC-004-SEMANTICS.md docs/reference/formalization-boundary.md CHANGELOG.md
git commit -m "docs: add SPEC-004 proof boundaries and invariants"
```

---

### Task 8: Run the cross-spec consistency pass and close out

**Files:**

- Modify: `docs/spec/SPEC-001-IR.md` (if needed)
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/reference/formalization-boundary.md` (if needed)
- Modify: `CHANGELOG.md`

**Step 1: Write the failing consistency checklist**

Check for drift between `SPEC-004` and:

- `SPEC-001` core expression/pattern names,
- `SPEC-013` receive terms,
- `SPEC-017` / `SPEC-018` capability-verification boundary language,
- formalization-boundary proof targets.

**Step 2: Run RED consistency review**

Run:

```bash
rg -n "Expression Evaluation Judgment|Pattern Matching Judgment|Receive|PolicyDecision|PatternBindFailure|PatternMatchFailure|RuntimeFailure" docs/spec/SPEC-001-IR.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-013-STREAMS.md docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md docs/spec/SPEC-018-CAPABILITY-MATRIX.md docs/reference/formalization-boundary.md
```

Expected: any remaining naming or boundary drift is now visible.

**Step 3: Apply the minimal consistency edits**

Edit only the mismatched names, references, and boundary notes needed to keep the final corpus coherent.

**Step 4: Run GREEN verification**

Run:

```bash
git diff --check
rg -n "Expression Evaluation Judgment|Pattern Matching Judgment|Determinism and Nondeterminism|Semantic Invariants" docs/spec/SPEC-004-SEMANTICS.md
```

Expected: no diff-check issues and the final `SPEC-004` shape is stable.

**Step 5: Commit**

```bash
git add docs/spec/SPEC-001-IR.md docs/spec/SPEC-004-SEMANTICS.md docs/reference/formalization-boundary.md CHANGELOG.md
git commit -m "docs: finalize proof-grade SPEC-004 big-step core semantics"
```

---

## Execution handoff

Plan complete and saved to `docs/plans/2026-03-29-spec-004-big-step-core-implementation-plan.md`.

Two execution options:

1. **Subagent-Driven (this session)** - I dispatch a fresh subagent per task, review between tasks, fast iteration
2. **Parallel Session (separate)** - Open a new session with executing-plans, batch execution with checkpoints
