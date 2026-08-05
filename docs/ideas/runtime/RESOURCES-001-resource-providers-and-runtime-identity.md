---
status: drafting
created: 2026-08-05
last-revised: 2026-08-05
related-plan-tasks: []
tags: [research, runtime, resources, providers, interfaces, authority, admission, provenance, processes]
---

# RESOURCES-001: Resource providers and runtime identity

## Purpose

This is a research idea, not an implementation proposal. It explores how Ash can realize interface operations through providers whose state dependencies are explicit runtime resources. It treats provider recipes as the runtime-realization side of the broader component model in [TYPES-005](../type-system/TYPES-005-component-abstraction-with-interfaces.md), without redefining static component identity, associated-type equality, or module abstraction.

The central question is how an admitted provider may connect a checked implementation recipe to concrete resource instances while preserving authority boundaries, lifecycle policy, process behavior, and provenance.

## Scope

- **In scope:** resource kinds, instances, bindings, provider recipes, host and Ash-defined provider bodies, admission, provider frames, runtime identity, lifecycle, and shared-state boundaries.
- **Out of scope:** a final surface syntax for provider declarations; a complete runtime resource implementation; a new module system; implicit resource discovery; and making runtime instance identities into type-level equalities.
- **Related but separate:** static components, associated type families, public/private type equations, interface coherence, effect-row syntax, process runtime semantics, and first-class resource references.

All Ash syntax in this document is schematic unless explicitly marked as current syntax.

## The phase boundary

Resource providers connect static descriptions to runtime facts. They do not erase the distinction.

| Entity | Phase | Role |
|---|---|---|
| component identity `C` | static | Names an implementation family and its public interface facts. |
| resource kind `R` | static | Describes a permitted category of managed runtime state. |
| resource slot `b : R` | static lexical/admission context | Names a dependency that an implementation recipe requires. |
| resource instance `ι` | dynamic | Identifies one allocated or host-admitted state instance. |
| provider frame `f` | dynamic | Realizes operations of one component recipe over concrete dependencies. |

A component application does not allocate a resource. Resource allocation does not create a fresh component identity. Repeated admission of one provider recipe over different resource instances creates different dynamic bindings, not different static component identities.

The shared boundary contract is developed in [Component-resource phase boundary](../architecture/COMPONENT-RESOURCE-PHASE-BOUNDARY.md).

## Provider recipes

A provider is a checked implementation recipe with explicit dependencies. The recipe has one semantic shape regardless of whether its method bodies run in Ash or through trusted host primitives.

```text
ProviderRecipe {
  component: C,
  implements: Interface<C>,
  resource_dependencies: { b1: R1 @ access1, ... },
  provider_dependencies: { p1: Interface<P1>, ... },
  configuration_dependencies: { ... },
  operations: { C::operation -> checked body },
}
```

The static checker verifies that every operation implements the declared interface signature and that the body can use only declared dependencies. A recipe does not select a concrete resource instance, create authority, or install a provider frame.

### Example: a stateful key-value provider

The following is proposed pseudo-code. It illustrates dependency shape only.

```ash
interface Kv<K, V> {
    get(K) -> Option<V>;
    put(K, V) -> Unit;
}

resource type KeyValueState {
    -- Representation and runtime policy are deliberately omitted.
}

type MemoryKv;

impl Kv<String, Bytes> for MemoryKv
    requires resource store: KeyValueState
{
    get(key) -> {resource store read} Option<Bytes> {
        ...
    }

    put(key, value) -> {resource store write} Unit {
        ...
    }
}
```

`MemoryKv` is static. `KeyValueState` is a static resource kind. `store` is a checked dependency slot. A concrete resource instance is selected only when a host or application admission boundary realizes this recipe.

## Admission and provider frames

Admission combines a static recipe with dynamic dependencies:

```text
ProviderRecipe(MemoryKv)
+ store ↦ ι_42 : KeyValueState
+ admitted authority/provenance facts
= ProviderFrame f_9
```

A provider frame contains the implementation route, concrete resource instance identities, admitted upstream providers, operation identities, lifecycle/access guards, and provenance links. Operation dispatch finds an admitted matching frame; a computation row remains only a requirement and never installs a frame by itself.

The runtime must reject admission when a required resource is absent, has the wrong kind, has an incompatible lifecycle/access policy, or would widen the recipe's declared authority.

## Two implementation routes

### Ash-defined providers

An Ash-defined provider lowers checked operation bodies to the ordinary Core/CPS execution path, closed over an admitted dependency environment. It can adapt or compose existing providers, validate inputs, add logging, cache against a declared cache resource, or enforce a narrower policy.

It must not obtain arbitrary resource state by ambient lookup. An Ash-defined provider can only perform resource access through operations or primitives justified by its declared resource slots and residual row requirements.

### Host-backed providers

A host-backed provider implements the same checked recipe contract, but selected methods delegate to typed runtime primitives. The primitive descriptor must align with the Ash method signature and declare required resource slots, access modes, lifecycle constraints, failure behavior, and provenance effects.

Host code receives a restricted resource-access facade rather than an unrestricted engine registry or ordinary mutable value. This keeps host authority explicit and makes resource use auditable.

## Resource access and operation effects

Operation requirements and resource requirements remain distinct:

```text
operation item: the computation may invoke C::op through an admitted provider/handler
resource item:  the realization may access slot b with a stated discipline
```

A provider may require both. For example, an adapter may invoke `InnerKv::get` and write an explicitly declared cache resource. Having operation authority does not choose a resource instance; having a resource binding does not authorize arbitrary operations.

## Process and shared-state policy

Resource types are useful for shared state only when the runtime or host supplies a real sharing mechanism. A declaration cannot make separate process heaps shared or make writes atomic.

A resource crossing a process boundary must declare or inherit an honest policy:

- `ReadOnlyShare` for immutable or read-only access;
- `BranchLocalClone` for isolated copies;
- `LinearMove` for exclusive transfer;
- `Mergeable` only with a specified reconciliation operation;
- `NonShareable` when projection is forbidden;
- `CommunicationOnly` when access must occur through an actor, broker, database, or another endpoint protocol.

`CommunicationOnly` is the conservative default for mutable shared state. The provider should expose atomic operations such as compare-and-swap or transactions rather than handing writable state to multiple processes.

## Research questions

1. Which declarations should have distinct static kinds: `Component`, `Resource`, and perhaps `ProviderRecipe`?
2. Should a provider recipe be expressed as a specialized interface implementation header, a separate declaration class, or a library-level component pattern?
3. How should admission bind lexical resource slots to runtime instances without making instance IDs ordinary values?
4. What subset of resource access primitives is sufficient for Ash-defined stateful providers?
5. How can provider frames preserve innermost-to-outermost handler/provider dispatch while also carrying resource provenance?
6. Which resource policies can be enforced by the first runtime slice, and which must initially reject at admission?
7. When should a resource binding gain a first-class, branded reference type rather than remain a scoped environment entry?

## Related explorations

- [TYPES-005: Component abstraction with interfaces and private types](../type-system/TYPES-005-component-abstraction-with-interfaces.md) — static component identity, opacity, associated members, and generic/fresh component application.
- [Component-resource phase boundary](../architecture/COMPONENT-RESOURCE-PHASE-BOUNDARY.md) — the shared static/dynamic identity contract.
- [NOTE-009: Capability interfaces, implementations, and internal authority](../../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md) — historical resource and authority exploration that remains useful for runtime identity, ownership, and provenance.
- [SPEC-053: Runtime resources and authority provenance](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) — historical/current-state resource substrate and target-reconciliation boundary.
- [SPEC-096b: Target effect system](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md) — target operation/resource row taxonomy and kind-specific discharge.

## References

### Internal references

- [TYPES-005: Component abstraction with interfaces and private types](../type-system/TYPES-005-component-abstraction-with-interfaces.md) — static component abstraction that this note consumes.
- [Component-resource phase boundary](../architecture/COMPONENT-RESOURCE-PHASE-BOUNDARY.md) — shared identity and admission invariants.
- [SPEC-053: Runtime resources and authority provenance](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) — resource identity, ownership, lifecycle, split/join, and provenance substrate.
- [SPEC-096b: Target effect system](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md) — operation and resource effects as separate row-item kinds.
- [SPEC-097b: Target type system](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md) — provider/handler-frame discharge model.

### External references

- Martin Kleppmann, *Designing Data-Intensive Applications*, 2017. Relevant background for explicit consistency, coordination, and failure assumptions in shared-state backends. <https://dataintensive.net/>
- Erlang/OTP Documentation, “Processes.” Relevant operational background for process-local state and interaction through explicit message-passing endpoints. <https://www.erlang.org/doc/system/ref_man_processes.html>
- The Rust Programming Language, “Shared-State Concurrency.” A practical contrast between ordinary runtime-managed shared state and an explicitly constrained access discipline. <https://doc.rust-lang.org/book/ch16-03-shared-state.html>

## Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-08-05 | Keep resource providers as a sibling runtime exploration rather than a section of TYPES-005. | Providers require runtime admission, lifecycle, authority, and provenance questions that would obscure the broader static component design. |
| 2026-08-05 | Treat provider recipes as the shared abstraction and host/Ash bodies as two realization routes. | Callers need one operation contract, dependency discipline, and provenance model regardless of where a method executes. |
| 2026-08-05 | Keep resource instance identities dynamic and opaque to ordinary type equality. | Resource allocation history must not affect compile-time definitional equality. |

## Next steps

- [ ] Compare a provider-recipe declaration with an interface-implementation header plus dependency clauses.
- [ ] Identify the smallest typed primitive/access-facade surface required by one Ash-defined provider and one host-backed provider.
- [ ] Model one `CommunicationOnly` shared-resource provider and one `Mergeable` process-local resource as contrasting cases.
- [ ] Revisit this note after the component kind and static dependency-signature questions in TYPES-005 have narrowed.

## Changelog

| Date | Change |
|------|--------|
| 2026-08-05 | Created the exploratory runtime companion for resource providers, admission, provider frames, and runtime identity. |
