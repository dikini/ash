# TASK-1026 Algebra Law Test Handoff

## Status

Complete for Phase 133 handoff. This artifact deliberately does not implement law-test execution.

## RED Evidence

Phase 133 now has source-visible algebra interfaces and carrier evidence, but there is no owned generated test packet that tells the SPEC-077 runner framework how to derive Semigroup, Monoid, Functor, Applicative, and Monad law tests. Without this packet, law checking remains an unowned historical deferral.

## Normative Law Profiles

### Semigroup

- Associativity: `append(append(a, b), c) == append(a, append(b, c))`.
- Required metadata: finite generator for `A`, equivalence relation for `A`, shrink/display hooks for diagnostics.

### Monoid

- Semigroup associativity inherited.
- Left identity: `append(empty(), a) == a`.
- Right identity: `append(a, empty()) == a`.
- Required metadata: Semigroup metadata plus an executable `empty` witness.

### Functor

- Identity: `map(value, id) == value`.
- Composition: `map(map(value, f), g) == map(value, compose(g, f))`.
- Required metadata: generator for `F<Int>`, generator set for total `Int -> Int` fixtures, equivalence for `F<Int>`.

### Applicative

- Identity: `apply(pure(id), v) == v`.
- Homomorphism: `apply(pure(f), pure(x)) == pure(f(x))`.
- Interchange: `apply(u, pure(y)) == apply(pure(fn(f) { f(y) }), u)`.
- Composition: `apply(apply(apply(pure(compose), u), v), w) == apply(u, apply(v, w))`.
- Required metadata: Functor metadata, generator for `F<Int -> Int>`, pure/apply evidence, effect/evaluation mode.

### Monad

- Left identity: `bind(unit(a), f) == f(a)`.
- Right identity: `bind(m, unit) == m`.
- Associativity: `bind(bind(m, f), g) == bind(m, fn(x) { bind(f(x), g) })`.
- Required metadata: Applicative/Functor metadata, generator for `M<Int>`, generator set for total `Int -> M<Int>` fixtures, equality/equivalence for `M<Int>`.

## Pure Instances Requirements

Pure instances (`Option`, `Result<_, E>`, `List`, `String`) need these pure instances metadata entries:

- Small finite value generators.
- Stable textual counterexample renderers.
- A declared equality relation. Structural equality is sufficient for `Option`, `Result`, `List`, and `String` once exposed to the runner.
- Error-type generator for `Result<_, E>` fixtures.

## Tower Carrier Requirements

Tower carriers (`Act`, `Proc`, `Workflow`) require a separate equivalence story from pure structural equality:

- `Act`: compare observable runtime result plus normalized failure/provenance boundary, not hidden `ActEnv` internals.
- `Proc`: compare process summaries/traces under bounded deterministic schedules.
- `Workflow`: compare admitted workflow artifact and deterministic small-world execution result under the SPEC-077 runner.
- All tower generators must be bounded, side-effect-free or sandboxed, and must not expose hidden runtime state.

## SPEC-077 Generated-Test Integration

The follow-up task must integrate law profiles into the generated-test framework as data-driven law suites:

1. Register algebra law profiles as generated-test families.
2. Associate each public impl/evidence record with optional law metadata.
3. Generate concrete small examples for pure carriers first.
4. Add tower law tests only when bounded equivalence metadata exists.
5. Report law failures as generated-test diagnostics with the interface, instance key, law name, seed, and minimized counterexample.

## Follow-up Task Owner

Concrete owner: `docs/plan/tasks/TASK-1029-generated-algebra-law-tests.md`.

Acceptance rows are defined there; TASK-1028 must verify the file remains present before Phase 133 closeout. These acceptance rows make ownership testable rather than prose-only.
