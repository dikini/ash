# PLAN-130: Interface Evidence Constraints

**Status:** Planned
**Spec:** [SPEC-080](../spec/SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md)
**Depends on:** [SPEC-034](../spec/SPEC-034-WHERE-BOUNDED-GENERIC-INTERFACE-IMPLEMENTATIONS.md), [SPEC-064](../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md), [SPEC-067](../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md), [SPEC-078](../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
**Task range:** TASK-1038 through TASK-1048

## Goal

Add interface-level evidence constraints so an interface can state required evidence directly. The first required user-facing case is `Monad<M>` requiring `Applicative<M>` through `interface Monad<M : * -> *> where M: Applicative { ... }`. The same phase also plans accepted standard algebra constraints `Applicative<F>` requiring `Functor<F>` and `Monoid<A>` requiring `Semigroup<A>`.

## Architecture

This phase is parser/typechecker-first. The parser preserves interface `where` constraints as raw surface evidence requirements. The type checker validates the constraint graph, verifies required evidence when impls are registered or evidence is looked up, and exposes verified required evidence in generic contexts. The feature does not synthesize impls, default methods, or object-hierarchy relations.

## Task breakdown

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-1038](tasks/TASK-1038-interface-evidence-constraints-packet.md) | Create the SPEC-080/PLAN-130 packet, task files, index rows, and changelog entry | Docs/Planning | 6 | ✅ Complete |
| [TASK-1039](tasks/TASK-1039-interface-evidence-constraints-audit-gate.md) | Audit live parser/typechecker/stdlib evidence seams and freeze exact implementation commands | Audit/Planning | 8 | ✅ Complete |
| [TASK-1040](tasks/TASK-1040-interface-constraint-parser-surface.md) | Parse and preserve interface-level `where` evidence constraints with positive/negative tests | Parser | 10 | ✅ Complete |
| [TASK-1041](tasks/TASK-1041-interface-constraint-core-lowering-and-summaries.md) | Carry interface constraints through lowering/core summaries or prove no summary change is needed | Core/Engine | 10 | ✅ Complete |
| [TASK-1042](tasks/TASK-1042-typeenv-interface-constraint-registration.md) | Store interface constraints in TypeEnv and enforce required evidence for concrete impl registration | Typeck | 14 | ✅ Complete |
| [TASK-1043](tasks/TASK-1043-generic-entailment-and-evidence-lookup.md) | Make constrained evidence entail required evidence in generic contexts without reverse derivation | Typeck | 14 | 📝 Planned |
| [TASK-1044](tasks/TASK-1044-stdlib-monad-applicative-constraint.md) | Migrate stdlib `Monad` to `where M: Applicative` and reconcile examples/reference wording | Stdlib/Docs | 10 | 📝 Planned |
| [TASK-1045](tasks/TASK-1045-stdlib-applicative-functor-constraint.md) | Migrate stdlib `Applicative` to `where F: Functor` and reconcile examples/reference wording | Stdlib/Docs | 8 | 📝 Planned |
| [TASK-1046](tasks/TASK-1046-stdlib-monoid-semigroup-constraint.md) | Migrate stdlib `Monoid` to `where A: Semigroup` and reconcile examples/reference wording | Stdlib/Docs | 8 | 📝 Planned |
| [TASK-1048](tasks/TASK-1048-interface-evidence-constraints-closeout.md) | Run diagnostics, broad verification, independent review, and status reconciliation | Closeout | 8 | 📝 Planned |

Total estimate: 90h.

## Execution order

1. TASK-1038 creates the planning packet only. It does not implement parser or typechecker behavior.
2. TASK-1039 is a hard audit gate. It must inspect live parser dispatch, surface/core carriers, lowering, module summaries, TypeEnv interface/impl/evidence lookup, stdlib algebra modules, and existing tests before code changes.
3. TASK-1040 may begin only after TASK-1039 freezes exact parser entry points and non-zero parser test commands.
4. TASK-1041 may begin after TASK-1040. It decides whether constraints must cross core/module-summary boundaries for imports and records exact carriers.
5. TASK-1042 depends on TASK-1040 and TASK-1041. It owns concrete evidence verification, missing-required-evidence rejection, and cycle detection.
6. TASK-1043 depends on TASK-1042. It owns generic-context entailment, evidence lookup integration, and negative reverse-entailment tests.
7. TASK-1044 depends on TASK-1043. It migrates `std::algebra::Monad` only after the parser and typechecker can enforce the constraint through final stdlib import paths.
8. TASK-1045 depends on TASK-1043. It migrates `std::algebra::Applicative` only after `Applicative<F> where F: Functor` can be enforced through final stdlib import paths.
9. TASK-1046 depends on TASK-1043. It migrates `std::algebra::Monoid` only after `Monoid<A> where A: Semigroup` can be enforced through final stdlib import paths.
10. TASK-1048 closes the phase only after all focused and broad gates pass, stale wording is reconciled, and independent review approves.

## Decision gates

- D1: Use interface-level evidence constraints, not blanket generic impls, to express accepted algebra requirements such as `Monad` requiring `Applicative`, `Applicative` requiring `Functor`, and `Monoid` requiring `Semigroup`.
- D2: Avoid object-hierarchy terms in user-facing docs and diagnostics. Use “requires”, “entails”, “evidence constraint”, or “required evidence”.
- D3: The type checker verifies required evidence. It must not synthesize implementations or method bodies.
- D4: Entailment is directional: `M: Monad` entails `M: Applicative`; `M: Applicative` does not entail `M: Monad`.
- D5: The parser must accept `interface Monad<M : * -> *> where M: Applicative { ... }` and reject generalized proposition/object-style extension forms at the interface declaration site.
- D6: Final stdlib tests must go through real `std::algebra` import paths. Fixture-only local interfaces are insufficient.
- D7: No separate `Functor`/`Monoid` evidence constraint is planned. The monoid-in-endofunctors reading belongs to `Monad`; scalar `Monoid<A>` remains governed by `Semigroup<A>`.
- D8: Existing impl `where` constraints from SPEC-034 must keep their current semantics and not be confused with interface constraints.

## Sub-agent delegation model

Use a fresh sub-agent per implementation task. Each task must include three review passes:

1. implementation: create RED tests or audit artifact, implement the minimal slice, and run focused gates;
2. spec review: verify against SPEC-080 and this plan, especially no automatic derivation and no object-hierarchy wording;
3. quality review: inspect for fixture-only coverage, overlap/coherence regressions, stale docs, missing changelog/status updates, and zero-test cargo filters.

TASK-1040 and TASK-1041 may run in sequence or with careful handoff after TASK-1039. TASK-1042 through TASK-1044 are sequential because evidence verification depends on stable carriers.

## Verification strategy

TASK-1039 must replace placeholder commands in downstream task files with exact non-zero focused commands. Minimum expected phase gates:

```bash
cargo fmt --check
RUSTC_WRAPPER= cargo test -p ash-parser --test task_1040_interface_constraint_surface
RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1042_interface_constraint_registration
RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1043_interface_constraint_entailment
RUSTC_WRAPPER= cargo test -p ash-engine --test task_1044_stdlib_monad_constraint
RUSTC_WRAPPER= cargo test -p ash-engine --test task_1045_stdlib_applicative_constraint
RUSTC_WRAPPER= cargo test -p ash-engine --test task_1046_stdlib_monoid_constraint
RUSTC_WRAPPER= cargo check --workspace
RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTC_WRAPPER= cargo test --workspace
git diff --check
```

Filtered cargo commands must be paired with `-- --list`, a test-count assertion, or another non-zero guard proving the target exists and ran.

## Completion checklist

- [x] SPEC-080, PLAN-130, PLAN-INDEX, spec README, task files, and CHANGELOG are coherent for the planning packet.
- [x] Audit gate freezes exact parser/typechecker/stdlib seams and replaces downstream verification placeholders.
- [x] Surface syntax parses for `interface ... where M: Applicative { ... }`.
- [x] Unsupported interface constraint forms are rejected with focused tests.
- [x] Interface constraints are carried through core lowering and imported module summaries.
- [x] TypeEnv stores and validates interface evidence constraints.
- [x] Concrete `impl Monad<K>` is rejected unless `Applicative<K>` evidence is available.
- [ ] Generic `M: Monad` contexts may use `M: Applicative` evidence.
- [ ] Reverse entailment and automatic derivation are rejected/proven absent.
- [ ] `std::algebra::Monad` declares the `Applicative` constraint through final stdlib source.
- [ ] `std::algebra::Applicative` declares the `Functor` constraint through final stdlib source.
- [ ] `std::algebra::Monoid` declares the `Semigroup` constraint through final stdlib source.
- [ ] Broad verification and independent review pass before status promotion.
