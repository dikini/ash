# TASK-1440: Law Profile Data Structures and Pure Carrier Generators

## Status: ✅ Complete

## Description

Implement the data structures and generators needed to turn `std::algebra` law declarations into executable property tests. This task creates the foundation for generated algebra law tests without yet wiring them into the runner.

## Owner

Phase 144 — Stream A (Law Tests)

## Specification References

- `docs/spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md`
- `docs/spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md`
- `docs/plan/audits/TASK-1026-algebra-law-test-handoff.md`
- `docs/plan/tasks/TASK-1029-generated-algebra-law-tests.md`

## Requirements

1. **Law profile structs** in `crates/ash-engine/src/test/` or `crates/ash-cli/src/test_runner/`:
   - `LawProfile { interface, law_name, arity, proposition, carrier_filter }`
   - `CarrierInstance { type_key, generator, equivalence, is_tower }`
   - `LawTestCase { law_profile, carrier_instance, seed, cases }`

2. **Pure carrier generators** for:
   - `String`: arbitrary UTF-8 strings, empty string, Unicode edge cases
   - `List<A>`: empty, singleton, small multi-element lists (reuse existing `proptest` list generators)
   - `Option<A>`: `None`, `Some` with inner generator
   - `Result<A, E>`: `Ok`, `Err` with independent generators

3. **Equivalence relations** per carrier:
   - `String`: equality (`==`)
   - `List`: structural equality
   - `Option`/`Result`: variant equality with inner equivalence

4. **Interface law registry**:
   - Semigroup: associativity (`a <> (b <> c) == (a <> b) <> c`)
   - Monoid: left identity (`empty <> a == a`), right identity (`a <> empty == a`)
   - Functor: identity (`fmap id == id`), composition (`fmap (f . g) == fmap f . fmap g`)
   - Applicative: identity, homomorphism, interchange, composition
   - Monad: left identity, right identity, associativity

5. **Tower carrier gating**:
   - `Act`, `Proc`, `Workflow` instances exist in registry but are marked `is_tower: true`
   - No generators or equivalence for tower carriers in this task
   - Tower law execution deferred to future phase with bounded equivalence metadata

## Acceptance Criteria

- [x] `LawProfile` and `CarrierInstance` structs compile with `cargo check`
- [x] Pure carrier generators produce values for property tests
- [x] Equivalence relations correctly compare values
- [x] Interface law registry contains all Semigroup/Monoid/Functor/Applicative/Monad laws
- [x] Tower carriers are registered but gated (no panic, no silent skip — explicit deferred status)
- [x] Unit tests for generators and equivalence relations pass
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo fmt --check` passes

## Verification

```bash
cargo test -p ash-engine law_profile -- --nocapture
cargo test -p ash-cli law_profile -- --nocapture
cargo clippy -p ash-engine --all-targets --all-features -- -D warnings
cargo clippy -p ash-cli --all-targets --all-features -- -D warnings
cargo fmt --check
```

## Out of Scope

- Runner integration (TASK-1441)
- Comonad/Kleisli/Cokleisli laws (deferred; TASK-1036 handoff preserved)
- Tower carrier law execution (deferred)
- Proof body verification

## Notes

- Use `proptest` for generators if already a dependency; otherwise use `quickcheck` or a minimal seeded generator
- Keep generators deterministic — seed must be reproducible
- Do not add new dependencies without checking workspace Cargo.toml
