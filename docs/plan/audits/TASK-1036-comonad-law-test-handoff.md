# TASK-1036 Comonad and Kleisli Law Test Handoff

## Status

Complete for Phase 134 handoff. This artifact extends the TASK-1029 generated
algebra law-test owner and deliberately does not implement law-test execution.

## Implemented Surface Inputs

- `std::algebra::comonad::Comonad` is a source-visible interface with
  monomorphic `Int` payload methods: `extract` and `extend`.
- `std::algebra::kleisli` exposes concrete Option/Result helper wrappers:
  `id_option`, `compose_option`, `id_result`, and `compose_result`.
- No `Comonad` instances are registered for `Option`, `Result`, ordinary
  `List`, `Act`, `Proc`, or `Workflow`.

## Deferred Surface Inputs

- Cokleisli helpers remain deferred until a lawful Comonad carrier exists or
  source evidence-method dispatch can express generic helpers honestly.
- Coapplicative is deferred by
  `docs/plan/audits/TASK-1035-coapplicative-decision.md`; no source module is
  present.

## Normative Law Profiles

### Comonad

- Left identity: `extend(wa, extract) == wa`.
- Right identity: `extract(extend(wa, f)) == f(wa)`.
- Associativity:
  `extend(extend(wa, f), g) == extend(wa, fn(wb) { g(extend(wb, f)) })`.
- Required metadata: generator for `W<Int>`, total `extract`, total `extend`,
  generator set for total `W<Int> -> Int` fixtures, and an equivalence relation
  for `W<Int>`.

### Kleisli

- Left identity: `compose(unit, f) == f`.
- Right identity: `compose(f, unit) == f`.
- Associativity:
  `compose(compose(f, g), h) == compose(f, compose(g, h))`.
- Current executable helper candidates: Option and Result concrete wrappers.
- Required metadata: Monad law metadata, generator for `Int`, generator set for
  total `Int -> M<Int>` fixtures, and equivalence for `M<Int>`.

### Cokleisli

- Left identity: `compose(extract, f) == f`.
- Right identity: `compose(f, extract) == f`.
- Associativity:
  `compose(compose(f, g), h) == compose(f, compose(g, h))`.
- Required metadata: Comonad law metadata plus a lawful carrier with total
  extraction. Phase 134 has no executable candidate.

## TASK-1029 Ownership

`docs/plan/tasks/TASK-1029-generated-algebra-law-tests.md` remains the concrete
owner for generated algebra law execution. TASK-1036 extends that owner rather
than creating a parallel runner task.

TASK-1029 must treat Comonad, Kleisli, and Cokleisli law families as generated
law-test profiles, while gating Cokleisli execution until a lawful Comonad
carrier exists.
