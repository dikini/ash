# TASK-415: Closed-World Interfaces MVP Spec Cut

## Status: ✅ Complete

## Description

Promote `TYPES-002 V2` through a narrowed MVP cut rather than trying to implement ad-hoc
polymorphism directly from the broad exploration.

This task should define the smallest serious interface-constrained generic feature Ash could adopt
without collapsing capabilities, effects, and authority into one mechanism.

The MVP direction is closed-world interfaces with explicit impl sites, strong coherence, and a
clear separation between interface abstraction and capability authority.

This task is docs/spec work only. It should not implement parser, typechecker, or runtime support
for interfaces yet.

## Specification Reference

- [TYPES-002: Ad-Hoc Polymorphism](../../ideas/type-system/TYPES-002-ad-hoc-polymorphism.md)
- [TYPES-002 V2: Ad-Hoc Polymorphism](../../ideas/type-system/TYPES-002-ad-hoc-polymorphism-v2.md)
- [TYPES-002 V2 MVP Cut](../../ideas/type-system/TYPES-002-v2-mvp-cut.md)
- [TYPES-003: Capability and Effect Vocabulary](../../ideas/type-system/TYPES-003-capabilities-effects-vocabulary.md)
- [TYPES-004: Effect Typing Foundations](../../ideas/type-system/TYPES-004-effect-typing-foundations.md)
- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ `TYPES-002 V2` exists as the polished exploration
- ✅ MVP cut document exists to narrow the design space
- ✅ `TYPES-003` clarifies capability vs interface vocabulary pressure

## Requirements

### Functional Requirements

1. Define one MVP documentation/spec surface for closed-world interfaces.
2. Specify the MVP feature boundary explicitly:
   - interface declarations
   - impl declarations
   - constrained generic parameters
   - one canonical method call form
   - one canonical bound form
3. Record an MVP coherence rule that rejects overlapping/conflicting impls.
4. Record that capabilities remain separate from interfaces.
5. Record an MVP effect stance that keeps interface methods effect-conservative and avoids associated effects in the first pass.
6. Explicitly defer:
   - open-world typeclasses
   - associated types
   - associated effects
   - dynamic dispatch / trait objects / existential packaging
   - capability/interface unification
7. Update `docs/ideas/README.md` and `docs/ideas/IMPLEMENTABILITY-REPORT.md` so the repository has one narrowed follow-on target rather than only the broad v1/v2 exploration pair.
8. Update `PLAN-INDEX.md`.
9. Update `CHANGELOG.md`.

### Non-Functional Requirements

1. Keep the task documentation/spec-first.
2. Do not commit the language to every future interface feature now.
3. Optimize for clarity and coherence over surface-language cleverness.
4. Preserve the user-provided framing that v1 remains the non-normative reasoning trace and v2/MVP cut are the serious discussion surfaces.

## Deliverables

1. MVP-cut document for `TYPES-002 V2`.
2. Planning record for follow-on spec work.
3. Corpus updates that clearly distinguish reasoning-trace material from narrowed candidate design.

## TDD Evidence

### Red

Before this task:

- `TYPES-002 V2` is the best exploration, but still too broad for honest implementation planning;
- no narrowed MVP cut records what the first real interface feature should exclude;
- later work could too easily reopen open-world typeclasses, associated effects, and capability/interface unification all at once.

### Green

This task is complete when:

- the repository has one explicit MVP cut for closed-world interfaces;
- later planning can treat that cut as the basis for real spec work rather than treating the whole design space as in-scope.

## Files

- Modify: `docs/ideas/type-system/TYPES-002-ad-hoc-polymorphism.md`
- Modify: `docs/ideas/type-system/TYPES-002-ad-hoc-polymorphism-v2.md`
- Modify: `docs/ideas/type-system/TYPES-002-v2-mvp-cut.md`
- Modify: `docs/ideas/README.md`
- Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## Completion Checklist

- [x] MVP cut document present
- [x] v1/v2/MVP relationship made explicit in docs
- [x] closed-world interface boundary documented
- [x] major deferred features explicitly listed
- [x] ideas/reporting corpus updated
- [x] `PLAN-INDEX.md` updated
- [x] `CHANGELOG.md` updated

## Notes

This task should produce a spec-ready boundary, not a full implementation design. Any later parser or
typechecker work should depend on this MVP cut rather than on the full unconstrained exploration.

Completed scope:

- `TYPES-002 V2` remains the broader polished exploration, with `v1` preserved as reasoning trace;
- `TYPES-002 V2 MVP Cut` is now the narrowed follow-on target for interface planning/spec work;
- the MVP surface now fixes one canonical bound form (`T: Interface`) and one canonical method-call
  form (`Interface::method(value)`), records strict non-overlapping impl coherence, keeps
  capabilities separate, keeps methods effect-conservative, and explicitly defers open-world
  typeclasses, associated items/effects, dynamic dispatch, and capability/interface unification.
