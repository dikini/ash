# PLAN-138: Stdlib Algebra Laws and Pure-Carrier Proofs

> **For Hermes:** This is a planning-only handoff. Use `ash-phase-implementation`, `subagent-driven-development`, `test-driven-development`, and `verification-before-completion` before executing any implementation task.

**Goal:** Add source-visible laws to the existing `std/src/algebra/{semigroup,monoid,functor,applicative,monad}.ash` definitions, then evaluate and add honest manual proofs for `Option` and `Result<_, E>` where current proof semantics can support them.

**Architecture:** Treat this as a small law/proof phase after Phase 136. The first slice edits stdlib `.ash` law declarations only; the proof slice is gated by a live audit of `ProofBody::{ByDefinition, ByTest, Expr}` behavior and totality checking. `Option` and `Result` proofs must not overclaim: if the checker cannot validate structural case proofs yet, the implementation must use explicit `by test "..."` delegation and record manual proof support as a follow-up.

**Status:** 📝 Planned
**Spec:** Planned follow-up to [SPEC-078](../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md), [SPEC-079](../spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md), and [DESIGN-NOTE-INTERFACE-LAWS](../design/DESIGN-NOTE-INTERFACE-LAWS.md)
**Depends on:** [PLAN-128](PLAN-128-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md), [PLAN-136](PLAN-136-INTERFACE-LAW-SYNTAX.md)
**Task range:** TASK-1388 through TASK-1394 (task files are created at implementation start)

---

## Live context verified before planning

The current stdlib already has these surfaces:

- `std/src/algebra/semigroup.ash`
  - `Semigroup<A>` with `append(A, A) -> A`
  - existing law: `associativity`
- `std/src/algebra/monoid.ash`
  - `Monoid<A> where A: Semigroup` with `empty` and `append`
  - existing laws: `left_identity`, `right_identity`
- `std/src/algebra/functor.ash`
  - `Functor<F : * -> *>` with `map(F<A>, A -> B) -> F<B>`
  - no laws yet
- `std/src/algebra/applicative.ash`
  - `Applicative<F : * -> *> where F: Functor` with `pure`, `apply`
  - no laws yet
- `std/src/algebra/monad.ash`
  - `Monad<M : * -> *> where M: Applicative` with `unit`, `bind`
  - no laws yet
- `std/src/option.ash`
  - `impl Functor<Option>`, `impl Applicative<Option>`, `impl Monad<Option>`
- `std/src/result.ash`
  - `impl <E : *> Functor<Result<_, E>>`, `Applicative<Result<_, E>>`, `Monad<Result<_, E>>`

Phase 136 parser/typechecker support exists for:

- interface laws stored on `InterfaceDef.laws`
- impl/module proofs stored as `ProofDef`
- proof bodies:
  - `by_definition`
  - `by test "test_name"`
  - expression body (`ProofBody::Expr`)
- proof totality checks with fuel, match exhaustiveness, and circular proof detection

Important syntax constraints:

- Interface method signatures are positional: `map(F<A>, A -> B) -> F<B>`, not named parameter signatures.
- Laws currently use named value parameters and explicit equivalence evidence, e.g. `law associativity(a: A, b: A, c: A, eq: Eq<A>): ...`.
- Current source examples use closures like `fn(x) => x * 2`; law bodies that need lambdas should prefer that spelling unless the audit finds the law parser rejects it.

---

## Proposed normative law profiles

### Semigroup

Keep the existing law, but normalize it during the audit if the law/proof pipeline now requires a changed signature.

```ash
use algebra::eq::{Eq}

pub interface Semigroup<A> {
    append(A, A) -> A
    law associativity(a: A, b: A, c: A, eq: Eq<A>):
        eq.equiv(append(append(a, b), c), append(a, append(b, c)))
}
```

### Monoid

Keep the existing laws. `Monoid<A>` requires `Semigroup` evidence; it should not duplicate associativity unless the typechecker cannot connect required evidence to law reporting.

```ash
use algebra::semigroup::{Semigroup}
use algebra::eq::{Eq}

pub interface Monoid<A> where A: Semigroup {
    empty() -> A
    append(A, A) -> A
    law left_identity(a: A, eq: Eq<A>): eq.equiv(append(empty(), a), a)
    law right_identity(a: A, eq: Eq<A>): eq.equiv(append(a, empty()), a)
}
```

### Functor

Add identity and composition laws.

```ash
use algebra::eq::{Eq}

pub interface Functor<F : * -> *> {
    map(F<A>, A -> B) -> F<B>

    law identity(value: F<A>, eq: Eq<F<A>>):
        eq.equiv(map(value, fn(x) => x), value)

    law composition(value: F<A>, f: A -> B, g: B -> C, eq: Eq<F<C>>):
        eq.equiv(map(map(value, f), g), map(value, fn(x) => g(f(x))))
}
```

Audit point: if the law parser/typechecker cannot infer `fn(x) => x` or `fn(x) => g(f(x))` in a proposition, add private module-scope helpers instead of weakening the law:

```ash
fn law_id<A>(x: A) -> A { x }
fn law_compose<A, B, C>(f: A -> B, g: B -> C) -> A -> C { fn(x) => g(f(x)) }
```

Only keep helpers if they parse/check through the real stdlib path.

### Applicative

Add the four standard applicative laws. Prefer curried helper functions if inline nested lambdas are brittle.

```ash
use algebra::functor::{Functor}
use algebra::eq::{Eq}

pub interface Applicative<F : * -> *> where F: Functor {
    pure(A) -> F<A>
    apply(F<A -> B>, F<A>) -> F<B>

    law identity(value: F<A>, eq: Eq<F<A>>):
        eq.equiv(apply(pure(fn(x) => x), value), value)

    law homomorphism(x: A, f: A -> B, eq: Eq<F<B>>):
        eq.equiv(apply(pure(f), pure(x)), pure(f(x)))

    law interchange(u: F<A -> B>, y: A, eq: Eq<F<B>>):
        eq.equiv(apply(u, pure(y)), apply(pure(fn(f) => f(y)), u))

    law composition(u: F<B -> C>, v: F<A -> B>, w: F<A>, eq: Eq<F<C>>):
        eq.equiv(
            apply(apply(apply(pure(fn(f) => fn(g) => fn(x) => f(g(x))), u), v), w),
            apply(u, apply(v, w))
        )
}
```

Audit point: this is the riskiest law syntactically because Ash has no implicit currying. If nested function-returning closures do not parse/check, the implementation task must introduce small private helpers in `applicative.ash` or defer applicative composition with a documented syntax/substrate blocker. Do not silently drop the composition law.

### Monad

Add the three standard monad laws. Keep `unit` as the canonical operation name; `return` remains do-block syntax.

```ash
use algebra::applicative::{Applicative}
use algebra::eq::{Eq}

pub interface Monad<M : * -> *> where M: Applicative {
    unit(A) -> M<A>
    bind(M<A>, A -> M<B>) -> M<B>

    law left_identity(a: A, f: A -> M<B>, eq: Eq<M<B>>):
        eq.equiv(bind(unit(a), f), f(a))

    law right_identity(m: M<A>, eq: Eq<M<A>>):
        eq.equiv(bind(m, fn(x) => unit(x)), m)

    law associativity(m: M<A>, f: A -> M<B>, g: B -> M<C>, eq: Eq<M<C>>):
        eq.equiv(bind(bind(m, f), g), bind(m, fn(x) => bind(f(x), g)))
}
```

Audit point: `bind(m, unit)` may be attractive, but `fn(x) => unit(x)` is clearer for current first-class function handling and avoids overloading a method name as a value.

---

## Manual proof policy for `Option` and `Result`

### Why proofs are gated

`Option` and `Result` laws are structurally true by case analysis, but they are not all single-step definitional equalities unless the proof checker can reason through pattern matches and reduce both sides under constructors.

Examples:

- `Option` functor identity requires showing:
  - `map(Some { value: x }, id) = Some { value: x }`
  - `map(None, id) = None`
- `Result<_, E>` monad left identity is direct for `Ok`, but right identity and associativity require preserving `Err` exactly.

A `by_definition` proof is acceptable only if the implementation verifies both sides after expanding the actual `option::*` / `result::*` helper body. If `by_definition` currently means only “accepted proof marker,” then using it for these laws would overclaim.

### Proof staging

Use this policy in task execution:

1. **Manual proof attempt:** Add `ProofBody::Expr` or `by_definition` proofs for `Option` and `Result` only after a focused audit proves the checker validates the law body against the referenced law.
2. **Honest fallback:** If validation is not strong enough, add `proof ... { by test "..." }` entries tied to generated/synthetic law tests and mark manual proof checking as deferred.
3. **No false proof success:** Do not add `by_definition` to `Option` or `Result` simply because the parser accepts it.

### Candidate proof names

For `std/src/option.ash`:

```ash
pub impl Functor<Option> {
    map(value, f) = option::map(value, f)

    proof identity(value: Option<A>) {
        by test "option_functor_identity"
    }

    proof composition(value: Option<A>, f: A -> B, g: B -> C) {
        by test "option_functor_composition"
    }
}

pub impl Applicative<Option> {
    pure(value) = option::pure(value)
    apply(functions, value) = option::apply(functions, value)

    proof identity(value: Option<A>) {
        by test "option_applicative_identity"
    }
    proof homomorphism(x: A, f: A -> B) {
        by test "option_applicative_homomorphism"
    }
    proof interchange(functions: Option<A -> B>, y: A) {
        by test "option_applicative_interchange"
    }
    proof composition(u: Option<B -> C>, v: Option<A -> B>, w: Option<A>) {
        by test "option_applicative_composition"
    }
}

pub impl Monad<Option> {
    unit(value) = option::pure(value)
    bind(value, f) = option::and_then(value, f)

    proof left_identity(a: A, f: A -> Option<B>) {
        by test "option_monad_left_identity"
    }
    proof right_identity(m: Option<A>) {
        by test "option_monad_right_identity"
    }
    proof associativity(m: Option<A>, f: A -> Option<B>, g: B -> Option<C>) {
        by test "option_monad_associativity"
    }
}
```

For `std/src/result.ash`, use analogous names with fixed error constructor target:

```ash
pub impl <E : *> Monad<Result<_, E>> {
    unit(value) = result::pure(value)
    bind(value, f) = result::and_then(value, f)

    proof left_identity(a: A, f: A -> Result<B, E>) {
        by test "result_monad_left_identity"
    }
    proof right_identity(m: Result<A, E>) {
        by test "result_monad_right_identity"
    }
    proof associativity(m: Result<A, E>, f: A -> Result<B, E>, g: B -> Result<C, E>) {
        by test "result_monad_associativity"
    }
}
```

These `by test` bodies are the safe first spelling. A task may upgrade selected proofs to real expression/definition proofs only after it demonstrates proof validation, not just parse acceptance.

---

## Implementation plan

### TASK-1388: Audit law/proof stdlib readiness

**Type:** Audit/Planning

**Objective:** Freeze exact accepted syntax and proof semantics for stdlib law declarations before editing algebra files.

**Files:**

- Inspect: `std/src/algebra/*.ash`
- Inspect: `std/src/option.ash`
- Inspect: `std/src/result.ash`
- Inspect: `crates/ash-parser/src/parse_module.rs`
- Inspect: `crates/ash-parser/src/surface.rs`
- Inspect: `crates/ash-typeck/src/type_env/*.rs`
- Create: `docs/plan/audits/TASK-1388-stdlib-law-proof-readiness.md`

**Steps:**

1. Parse/check the current stdlib law-bearing files:
   ```bash
   RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-parser task_1360 task_1362 task_1363 -- --nocapture
   ```
2. Add temporary throwaway snippets outside the repo or in a scratch test to verify whether these law-body forms parse:
   - `fn(x) => x`
   - `fn(x) => g(f(x))`
   - `fn(f) => f(y)`
   - `fn(f) => fn(g) => fn(x) => f(g(x))`
3. Verify whether `by_definition` is semantically checked or only accepted as proof syntax.
4. Verify whether `ProofBody::Expr` can express a total case proof that typechecks as a proposition.
5. Record the result in the audit artifact with exact commands, failing snippets if any, and the chosen proof policy.
6. Patch downstream task files if the audit changes any planned code shape.

**Acceptance:** The audit tells implementers exactly which law syntax and proof-body form to use, and it forbids false `by_definition` proofs if the checker cannot validate them.

### TASK-1389: Add/normalize Semigroup and Monoid law declarations

**Type:** Stdlib/Law declarations

**Objective:** Keep Semigroup and Monoid laws in source, normalize imports/formatting, and add regression coverage that the existing laws survive stdlib parsing/import.

**Files:**

- Modify: `std/src/algebra/semigroup.ash`
- Modify: `std/src/algebra/monoid.ash`
- Test: parser/typechecker stdlib law fixture chosen by TASK-1388

**Steps:**

1. Write a failing regression if Semigroup/Monoid laws are not currently asserted in tests.
2. Preserve `associativity`, `left_identity`, and `right_identity` with explicit `Eq` evidence.
3. Do not duplicate Semigroup associativity inside `Monoid` unless the audit proves required-law reporting cannot traverse `where A: Semigroup`.
4. Run focused parser/typechecker tests from TASK-1388.

### TASK-1390: Add Functor law declarations

**Type:** Stdlib/Law declarations

**Objective:** Add `identity` and `composition` laws to `std/src/algebra/functor.ash`.

**Files:**

- Modify: `std/src/algebra/functor.ash`
- Test: focused parser/typechecker law test chosen by TASK-1388

**Steps:**

1. Add `use algebra::eq::{Eq}`.
2. Add `identity` and `composition` law declarations.
3. If inline lambdas do not pass, add private helpers and document why in comments/audit.
4. Run focused tests and ensure non-zero coverage.

### TASK-1391: Add Applicative law declarations

**Type:** Stdlib/Law declarations

**Objective:** Add `identity`, `homomorphism`, `interchange`, and `composition` laws to `std/src/algebra/applicative.ash`.

**Files:**

- Modify: `std/src/algebra/applicative.ash`
- Test: focused parser/typechecker law test chosen by TASK-1388

**Steps:**

1. Add `use algebra::eq::{Eq}`.
2. Add the four laws.
3. Treat composition as mandatory unless the audit records a concrete syntax/substrate blocker.
4. If nested curried lambdas fail, add private helper functions or create a follow-up blocker task; do not silently omit the law.
5. Run focused tests and ensure the test target executed at least one law-bearing fixture.

### TASK-1392: Add Monad law declarations

**Type:** Stdlib/Law declarations

**Objective:** Add `left_identity`, `right_identity`, and `associativity` laws to `std/src/algebra/monad.ash`.

**Files:**

- Modify: `std/src/algebra/monad.ash`
- Test: focused parser/typechecker law test chosen by TASK-1388

**Steps:**

1. Add `use algebra::eq::{Eq}`.
2. Add the three laws using `unit` and `bind`.
3. Prefer `fn(x) => unit(x)` over bare `unit` as a function value unless the audit proves bare method references are accepted.
4. Run focused tests and ensure non-zero coverage.

### TASK-1393: Add honest Option and Result proof declarations

**Type:** Stdlib/Proof declarations

**Objective:** Add proof declarations to `std/src/option.ash` and `std/src/result.ash` without overstating proof-checking strength.

**Files:**

- Modify: `std/src/option.ash`
- Modify: `std/src/result.ash`
- Test: parser/typechecker proof tests chosen by TASK-1388
- Optional generated-test integration: task file or audit artifact if `by test` names need registration

**Steps:**

1. Add parser RED tests for proof declarations inside `impl Functor<Option>`, `Applicative<Option>`, `Monad<Option>`, and the corresponding `Result<_, E>` impls.
2. Add `by test "..."` proof bodies for all laws as the safe baseline.
3. Only upgrade individual proofs to `by_definition` or `ProofBody::Expr` if the audit proves the checker validates that proof form against the law proposition.
4. Ensure `Result` proof names preserve the fixed error type `E` and do not confuse `Err` domain values with operational `fail`.
5. Run focused tests with a non-zero proof declaration count.

### TASK-1394: Reference, generated-test handoff, and closeout

**Type:** Docs/Closeout

**Objective:** Reconcile reference docs, generated-test handoff docs, changelog, and broad verification for the new law/proof surface.

**Files:**

- Modify: `reference/stdlib/algebra.md`
- Modify: `docs/plan/audits/TASK-1026-algebra-law-test-handoff.md` or add a successor audit if this phase supersedes parts of it
- Modify: `CHANGELOG.md`
- Modify: `docs/spec/SPEC-078-...` or create `docs/spec/SPEC-081-...` depending on selected authority model
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/PLAN-138-STDLIB-ALGEBRA-LAWS-AND-PURE-CARRIER-PROOFS.md`

**Steps:**

1. Update reference docs to say laws are source-visible in `std/src/algebra`, not only handoff prose.
2. Separate law declaration status from proof status:
   - law declarations present in stdlib source
   - `Option`/`Result` proofs present as `by test`, `by_definition`, or expression proof depending on actual validation
   - generated law execution still owned by SPEC-077/TASK-1029 or successor tasks unless implemented here
3. Add a changelog entry under `[Unreleased]`.
4. Run docs and code gates:
   ```bash
   bash scripts/check-rust-format.sh
   git diff --check
   RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo check --workspace
   RUSTC_WRAPPER= CARGO_NET_OFFLINE=true bash scripts/check-rust-clippy.sh
   RUSTC_WRAPPER= CARGO_NET_OFFLINE=true bash scripts/check-doc-tests.sh
   RUSTC_WRAPPER= CARGO_NET_OFFLINE=true bash scripts/check-rust-tests.sh --workspace --all-targets
   RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo doc --workspace --no-deps 2>&1 | tee /tmp/phase-138-main-doc.log
   ! grep -i '^warning:' /tmp/phase-138-main-doc.log
   ```
5. Run independent review focused on:
   - law names matching interface methods
   - no fake proof success
   - `Result` domain failure distinct from operational bottom
   - Applicative composition not silently omitted
   - generated-test handoff consistency

---

## Key risks and decisions

1. **Applicative composition may expose missing curried-function ergonomics.** The task must either use a verified helper or record a blocker; the law must not disappear silently.
2. **`by_definition` may be syntactic rather than semantic.** Manual proofs for `Option` and `Result` must be tied to real validation or downgraded to `by test`.
3. **Law declarations need explicit equivalence evidence.** Keep `Eq<F<A>>`, `Eq<M<A>>`, etc.; do not reintroduce overloaded `==`.
4. **Result is not operational failure.** `Err` preservation in laws must remain domain-level and must not interact with `fail`.
5. **Generated tests and proofs are different surfaces.** A `proof ... by test` declaration is an explicit delegation to the runner, not a completed manual proof.

## Recommended execution approach

Create a dedicated worktree, then execute as a docs/spec + stdlib source phase:

```bash
git worktree add .worktrees/phase-138-stdlib-algebra-laws -b phase-138-stdlib-algebra-laws
```

Start with TASK-1388. Do not edit `std/src/algebra/applicative.ash` or add `Option`/`Result` proofs until the audit has frozen the exact syntax/proof checker behavior.
