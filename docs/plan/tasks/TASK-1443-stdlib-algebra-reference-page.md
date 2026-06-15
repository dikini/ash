# TASK-1443: Stdlib Algebra Reference Page

## Status: ✅ Complete

## Description

Create and validate the `reference/stdlib/algebra.md` reference page documenting `std::algebra` interfaces (Semigroup, Monoid, Functor, Applicative, Monad), instances, laws, and proofs. This is Phase 144 Stream C (Reference Pages).

## Requirements

1. Create `reference/stdlib/algebra.md` with SPEC-071 frontmatter documenting:
   - `std::algebra::semigroup` — `Semigroup<A>` with `append`
   - `std::algebra::monoid` — `Monoid<A>` with `empty` and `append`; requires `Semigroup<A>` evidence
   - `std::algebra::functor` — `Functor<F>` with generic `map`
   - `std::algebra::applicative` — `Applicative<F>` with `pure` and `apply`; requires `Functor<F>` evidence
   - `std::algebra::monad` — `Monad<M>` with `unit` and `bind`; requires `Applicative<M>` evidence
   - `std::algebra::comonad` — `Comonad<W>` with `extract` and `extend`
   - `std::algebra::kleisli` — currently deferred, no concrete carrier wrappers

2. Document instances:
   - `std::option` — `Functor<Option>`, `Applicative<Option>`, `Monad<Option>`
   - `std::result` — `Functor<Result<_, E>>`, `Applicative<Result<_, E>>`, `Monad<Result<_, E>>`
   - `std::list` — `Functor<List>`, `Semigroup<List<A>>`, `Monoid<List<A>>`
   - `std::string` — `Semigroup<String>`, `Monoid<String>`
   - No `Comonad` stdlib carrier instances

3. Document source-visible law declarations:
   - Semigroup: `law associativity(a, b, c, eq)`
   - Monoid: `law left_identity(a, eq)`, `law right_identity(a, eq)`
   - Functor: `law identity(value, eq)`, `law composition(value, f, g, eq)`
   - Applicative: `law identity(value, eq)`, `law homomorphism(x, f, eq)`, `law interchange(u, y, eq)`, `law composition(u, v, w, eq)`
   - Monad: `law left_identity(a, f, eq)`, `law right_identity(m, eq)`, `law associativity(m, f, g, eq)`

4. Document proof declarations:
   - `std::option` and `std::result` carry `by test "..."` proof declarations
   - `by_definition` proofs deferred until proof checker validates them

5. Follow the exact pattern of `reference/stdlib/act.md` and `reference/stdlib/proc.md`:
   - SPEC-071 YAML frontmatter with all required fields
   - `id: ref.stdlib.algebra`
   - `kind: reference`
   - `authority: canonical-adjacent`
   - Proper `verified_against` with git_commit, specs, tasks, code, tests, examples
   - Proper `related` with depends_on, explains, supersedes, superseded_by, historical_rationale
   - `refresh_trigger` listing all relevant change events

6. Update `reference/INDEX.md` to link to the new page under "Standard library" section.

7. Update `reference/stdlib/README.md` to link to the new page in the page list.

## Completion Checklist

- [x] `reference/stdlib/algebra.md` created with SPEC-071 frontmatter
- [x] All algebra interfaces documented (Semigroup, Monoid, Functor, Applicative, Monad, Comonad, Kleisli)
- [x] All instances documented (Option, Result, List, String)
- [x] Law declarations documented
- [x] Proof declarations documented
- [x] `reference/INDEX.md` links to `stdlib/algebra.md`
- [x] `reference/stdlib/README.md` links to `algebra.md`
- [x] `python3 tools/reference/check_frontmatter.py --root .` passes for the new page
- [x] All markdown links resolve
- [x] CHANGELOG.md updated
