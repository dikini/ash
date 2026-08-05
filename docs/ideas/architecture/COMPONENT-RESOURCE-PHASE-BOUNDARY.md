---
status: drafting
created: 2026-08-05
last-revised: 2026-08-05
related-plan-tasks: []
tags: [research, architecture, type-system, runtime, components, resources, providers, identity, admission]
---

# Component-resource phase boundary

## Purpose

This is a narrow cross-cutting research note. It records the boundary between the static component model explored by [TYPES-005](../type-system/TYPES-005-component-abstraction-with-interfaces.md) and the dynamic resource-provider model explored by [RESOURCES-001](../runtime/RESOURCES-001-resource-providers-and-runtime-identity.md).

Its purpose is to let both explorations evolve independently without collapsing compile-time component identities into runtime resource or provider identities. It does not select surface syntax, a Core representation, or a runtime implementation.

## Scope

- **In scope:** phase-separated identity taxonomy, static contexts, dynamic admission facts, the component-to-provider realization bridge, and non-equality invariants.
- **Out of scope:** component generativity design, resource lifecycle design, provider syntax, resource allocation syntax, and a dependent type system over runtime instance IDs.
- **Related but separate:** effect-row inference, interface coherence, process runtime semantics, module visibility, resource sharing policy, and first-class resource references.

## Identity taxonomy

Ash needs distinct identities for distinct questions.

| Symbol | Phase | Question answered | Example |
|---|---|---|---|
| `C : Component` | static | Which implementation/component family is this? | `PostgresKv` |
| `R : Resource` | static | Which category of managed runtime state is required? | `Database` |
| `b : ResourceSlot<R>` | static lexical/admission context | Which dependency slot may an implementation use? | `db : Database` |
| `ι : ResourceInstanceId` | dynamic | Which concrete allocated/admitted state instance is bound now? | a tenant-specific DB pool |
| `f : ProviderFrameId` | dynamic | Which admitted realization services an operation in this scope? | `PostgresKv` over `ι` |

A type or kind is static even if a compiler can also manipulate a runtime representation for it. A value is dynamic even if it is constant-folded or known at compile time. No automatic value-to-type promotion follows from compile-time knowledge.

## Static contexts

A resource-aware typing judgment needs more than an ordinary value-variable context:

```text
Σ ; Γ ; Δ ; Φ ⊢ e : A ! ρ
```

where:

- `Σ` contains static declarations: kinds, types, components, resource kinds, interface facts, implementation recipes, and public equations;
- `Γ` contains ordinary value bindings;
- `Δ` contains resource slots, their resource kinds, and their static access/ownership requirements;
- `Φ` contains the provider/handler facts required or available in a checked scope;
- `ρ` is the computation row of outstanding requirements.

For example, the checker can establish that a call requires `PostgresKv::put` and `resource db write`. It does not know a concrete `ResourceInstanceId` merely from that judgment.

## Dynamic admission facts

Admission supplies concrete runtime facts after static checking:

```text
H ; F ⊨ Δ ; Φ
```

where:

- `H` maps resource instance identities to live state, resource-kind metadata, lifecycle state, access policy, and provenance;
- `F` maps admitted provider bindings to provider/handler frames closed over their concrete dependencies.

A provider frame is constructed only by combining a checked recipe with compatible admitted dependencies:

```text
ProviderRecipe(C)
+ { b1 ↦ ι1, ..., bn ↦ ιn }
+ admitted provider/authority facts
= ProviderFrame(f)
```

The runtime may reject this construction. A declared resource type, a row item, a module import, or a static implementation fact never creates a resource instance or installs a provider frame by itself.

## Non-equality invariants

The following distinctions are semantic rules, not documentation conventions:

```text
same resource kind          does not imply same resource instance
same component recipe       does not imply same admitted provider binding
same source resource slot   does not imply same instance across separate runs
fresh component identity    is not runtime resource allocation
runtime allocation          is not a new static component identity
```

For example, two admitted bindings may both realize `PostgresKv` and both require a `Database` resource, while still targeting different tenant database instances. The static operation identity remains `PostgresKv::query`; runtime provenance distinguishes the concrete provider frame and resource instance.

## Resource slots are not ordinary values

A slot such as `db : Database` belongs in `Δ`, not automatically in `Γ`. It denotes a scoped dependency whose authority, lifecycle, and process projection may be constrained independently of ordinary value typing.

A future first-class resource-reference feature could expose a controlled value-level handle. It must retain the phase boundary: its type may state a static resource kind and a lexical abstract brand, but ordinary type equality must not expose or depend on the raw dynamic instance ID.

## Static brands, if needed

Some future operations may need static proof that two uses refer to the same scoped resource without exposing runtime identity. A lexical abstract brand is one possible direction:

```text
α : ResourceBrand
handle : ResourceRef<α, Database>
```

`α` is a static scope witness, comparable to a region/lifetime name. It does not equal the dynamic instance identity `ι`; admission binds the branded lexical slot to a compatible `ι` for one execution. This remains an open design direction, not proposed Ash syntax.

## Responsibilities of the sibling explorations

| Concern | Static component exploration: TYPES-005 | Runtime provider exploration: RESOURCES-001 | This note |
|---|---|---|---|
| Component identity and families | owns | consumes | constrains phase meaning |
| Associated types and opacity | owns | consumes public facts | records static-only status |
| Resource kind declarations | consumes classification | owns runtime use | distinguishes kind from instance |
| Resource instance/lifecycle | out of scope | owns | requires dynamic-only identity |
| Provider recipes | owns static shape | owns admission/execution realization | defines the handoff |
| Provider frames | out of scope | owns | requires dynamic construction |
| Type equality | owns equations | must not widen | forbids runtime-ID equality |

## Research questions

1. Should `Component` and `Resource` be explicit kinds, declaration classes over nominal types, or both?
2. What static information must a provider recipe export without exposing private representations or concrete bindings?
3. Which access/ownership facts belong in `Δ`, and which require dynamic checks only?
4. What is the minimal static brand discipline needed before first-class resource references can be considered?
5. How should resource/provider facts appear in Core and CPS while remaining non-authorizing until admission?

## Related explorations

- [TYPES-005: Component abstraction with interfaces and private types](../type-system/TYPES-005-component-abstraction-with-interfaces.md) — the static component model.
- [RESOURCES-001: Resource providers and runtime identity](../runtime/RESOURCES-001-resource-providers-and-runtime-identity.md) — the dynamic provider/resource model.
- [NOTE-020: Computation row taxonomy](../../notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md) — requirement rows as a common accounting layer with kind-specific discharge.
- [SPEC-096b: Target effect system](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md) — operations and resources as distinct row-item kinds.
- [SPEC-097b: Target type system](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md) — target provider/handler discharge framing.

## References

### Internal references

- [TYPES-005: Component abstraction with interfaces and private types](../type-system/TYPES-005-component-abstraction-with-interfaces.md) — source exploration for static identities and associated public/private type facts.
- [RESOURCES-001: Resource providers and runtime identity](../runtime/RESOURCES-001-resource-providers-and-runtime-identity.md) — source exploration for dynamic resource/provider realization.
- [SPEC-053: Runtime resources and authority provenance](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) — current/historical resource instance and provenance substrate.
- [SPEC-096b: Target effect system](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md) — row kinds and discharge separation.
- [SPEC-100: Core type checking](../../spec/SPEC-100-CORE-TYPE-CHECKING.md) — Core checking as requirement validation rather than runtime authority.

### External references

- Robert Harper, *Practical Foundations for Programming Languages*, 2nd ed., 2016, Chapters 48–49. Relevant background for static module identity, abstraction, and generativity. <https://www.cs.cmu.edu/~rwh/pfpl/2nded.pdf>
- Robin Milner, Mads Tofte, Robert Harper, and David MacQueen, *The Definition of Standard ML (Revised)*, 1997. Relevant background for static module identity and applicative versus generative construction. <https://smlfamily.github.io/sml97-defn.pdf>
- The Rust Reference, “Namespaces” and “Items.” Relevant implementation-facing background for keeping compile-time name/type resolution distinct from runtime values. <https://doc.rust-lang.org/reference/names/namespaces.html>

## Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-08-05 | Record a dedicated bridge contract instead of duplicating phase-boundary prose in the component and resource notes. | The static/dynamic distinction must remain stable while each detailed exploration evolves independently. |
| 2026-08-05 | Do not expose raw runtime instance IDs in ordinary type equality. | Runtime allocation history is not a valid input to compile-time definitional equality. |
| 2026-08-05 | Treat lexical resource slots as a separate context from ordinary values. | Ownership, lifecycle, authority, and process projection need independent checking. |

## Next steps

- [ ] Review the identity taxonomy against existing interface/type-family vocabulary and resource runtime carriers.
- [ ] Use the `Σ ; Γ ; Δ ; Φ` sketch to test one host-backed and one Ash-defined provider example.
- [ ] Decide whether lexical brands are needed before considering a first-class resource reference.

## Changelog

| Date | Change |
|------|--------|
| 2026-08-05 | Created the cross-cutting static/dynamic identity and admission-boundary exploration linking components to resource providers. |
