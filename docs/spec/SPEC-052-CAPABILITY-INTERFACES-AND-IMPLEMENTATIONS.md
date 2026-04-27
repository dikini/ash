# SPEC-052: Capability Interfaces and Implementations

**Status:** Draft
**Date:** 2026-04-27
**Promotes:** [NOTE-009](../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md) capability-interface, capability-implementation, capability-binding, late-binding, and capability-adapter design direction
**Related:** SPEC-002, SPEC-003, SPEC-009, SPEC-012, SPEC-017, SPEC-047, SPEC-048, SPEC-049, SPEC-051, [SPEC-053](SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md)

## Summary

This specification defines Ash capability interfaces, capability implementations, and capability bindings as a first-class language/runtime contract.

A capability interface is a stateless effectful operation shape. A capability implementation is a definition-time recipe that satisfies an interface using explicit dependencies. A capability binding is the admission-time association of an interface requirement with one implementation recipe and concrete dependency bindings.

This spec deliberately separates:

```text
capability interface = what operations exist
capability implementation = how those operations are realized from explicit dependencies
capability binding = which implementation/dependencies are admitted for one run/workflow/process/effect scope
```

The resource and authority-provenance substrate required by capability implementations is owned by [SPEC-053](SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md).

## 1. Scope and Authority

### 1.1 In scope

This spec defines:

1. capability interfaces as stateless operation surfaces;
2. capability implementations as explicit recipes satisfying interfaces;
3. capability bindings as admitted effect-environment bindings;
4. interface/implementation namespaces and module export/import requirements;
5. operation signature checking and implementation conformance;
6. binding-time semantics for selecting implementations;
7. derived/adapted capability implementations;
8. the relation to existing `pub capability` declarations and Rust `CapabilityProvider`s.

### 1.2 Out of scope

This spec does not define:

1. resource type, resource instance, resource allocation, or split/join semantics; see [SPEC-053](SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md);
2. exact provider implementation APIs in Rust beyond required compatibility hooks;
3. full concrete parser grammar for every sugar form;
4. first-class value-level capability handles;
5. dynamic rebinding after admission;
6. distributed or remote capability discovery;
7. all standard-library capability implementations.

### 1.3 Normative vs informative

Unless marked informative, sections are normative. Syntax blocks are normative where marked "required form" and informative where marked "sketch" or "example".

## 2. Terminology

### 2.1 Capability interface

A capability interface is a stateless declaration of effectful operations.

It defines:

1. operation names;
2. operation modes, initially `observe` and `execute`;
3. parameter names and types;
4. return type, where present;
5. optional policy/contract metadata in future extensions.

A capability interface must not allocate resources, own state, select an implementation, or grant external authority by itself.

Required conceptual form:

```ash
pub capability interface KVStore:
    observe get(key: String) returns Option<String>
  | execute put(key: String, value: String) returns Unit
  | execute delete(key: String) returns Unit;
```

### 2.2 Capability implementation

A capability implementation is a named definition-time recipe satisfying one capability interface.

It defines:

1. target interface;
2. explicit dependencies;
3. one operation body for every required interface operation;
4. effect/resource/authority requirements for those bodies.

Required conceptual form:

```ash
pub capability impl MemoryKV for KVStore
    requires resource kv: WorkflowKV
{
    observe get(key: String) returns Option<String> {
        ...
    }

    execute put(key: String, value: String) returns Unit {
        ...
    }

    execute delete(key: String) returns Unit {
        ...
    }
}
```

A capability implementation is not a mutable object. Its dependencies are explicit recipe parameters resolved at binding/admission time.

### 2.3 Capability binding

A capability binding is an admitted effect-environment binding that associates:

1. a binding name visible to a body;
2. a required capability interface;
3. a selected capability implementation;
4. concrete dependency bindings such as resources, other capabilities, or configuration values;
5. authority provenance metadata.

Required conceptual form:

```ash
workflow example
    owns kv: WorkflowKV
    uses store: KVStore = MemoryKV(kv)
{
    act execute store.put("a", "b");
    let x = act observe store.get("a");
}
```

The binding `store` is an effect-environment binding, not an ordinary pure value binding.

## 3. Relation to Existing `pub capability`

Existing `pub capability` declarations are treated as legacy direct capability declarations. They declare an operation surface and currently bind to module-owned provider metadata or host-backed provider names.

Conformance rule:

1. Existing `pub capability Name: ...;` remains valid.
2. A `pub capability Name: ...;` declaration is equivalent to a capability interface named `Name` plus an implementation/binding path supplied by the existing provider-resolution machinery.
3. New `capability interface` syntax is the preferred explicit form for interfaces that need multiple implementations, mocks, adapters, replay, or internal resources.
4. Existing Rust `CapabilityProvider` implementations remain valid host-backed primitive implementations.
5. A future migration may rewrite standard-library direct capabilities into explicit `capability interface` declarations without changing the public operation shapes.

## 4. Namespaces, Modules, and Visibility

Capability interfaces, implementations, and bindings participate in module ownership.

1. `pub capability interface` exports the interface name and operation metadata.
2. `pub capability impl` exports the implementation recipe if its name is public.
3. Private interfaces or implementations are visible only according to SPEC-009/SPEC-012 visibility rules.
4. Imported capability interfaces may be used in workflow headers, implementation headers, role/capability requirements, and type checking of capability calls.
5. Imported implementations may be selected at binding sites if visible.
6. Capability binding names inside workflow/process/effect headers live in an effect-environment namespace distinct from pure value/function names.

A conforming implementation must not resolve a capability binding by ambient lexical lookup alone. Binding visibility requires explicit admission or explicit local binding construction.

## 5. Interface Operation Signatures

An interface operation signature contains:

```text
CapabilityOperationSignature {
  interface: InterfaceName,
  operation: OperationName,
  mode: Observe | Execute,
  params: List<(Name, Type)>,
  return_type: Type,
}
```

A conforming implementation must reject:

1. duplicate operation names within an interface unless overloading is explicitly added by a future spec;
2. duplicate parameter names in an operation;
3. unknown types in parameter or return positions;
4. unsupported operation modes;
5. operation signatures that require pure functions to invoke capabilities.

## 6. Implementation Conformance

A capability implementation conforms to an interface when:

1. it implements every required operation exactly once;
2. every implemented operation has the same mode as the interface operation;
3. parameter arity and parameter types match the interface operation;
4. return type matches the interface operation;
5. implementation bodies satisfy the effect mode constraints;
6. implementation bodies use only declared dependencies and admitted internal resources/capabilities;
7. implementation bodies do not widen authority beyond dependencies.

The type checker owns static conformance checks. The runtime owns admission-time validation that the selected implementation and dependency bindings are present and authorized.

## 7. Binding-Time Semantics

The capability-resource/implementation link is created at binding/admission time.

Conformance rules:

1. An interface declaration does not allocate resources.
2. A resource type declaration does not select a capability implementation.
3. A capability implementation declaration does not create a concrete binding.
4. A binding/admission site applies an implementation recipe to concrete dependency bindings.
5. The admitted binding records authority provenance.

Conceptual lowering:

```text
uses store: KVStore = MemoryKV(kv)

=>
CapabilityBinding {
  name: store,
  interface: KVStore,
  implementation: MemoryKV,
  dependencies: { kv: ResourceInstanceId(...) },
  provenance: Internal(...),
}
```

## 8. Derived and Adapter Implementations

A capability implementation may depend on another capability interface and produce the same or narrower interface.

Examples:

```text
LoggingHttp(inner: Http, log: Logger) => Http
CachingKV(inner: KVStore, cache: WorkflowKV) => KVStore
SandboxFs(inner: Fs, root: PathPolicy) => Fs
```

Conformance rules:

1. Derived implementations must declare every inner capability dependency.
2. Derived implementations must not expose operations outside the target interface.
3. Derived implementations must not widen authority beyond their dependencies and explicitly allocated internal resources.
4. Derived implementations must preserve provenance links to inner capability bindings and resource instances.

## 9. Effect and Tower Placement

Capability bindings are available only in effectful contexts and below:

```text
Workflow admits/binds capability interfaces and implementations
Proc carries/splits/joins projected capability bindings
Act invokes admitted capability bindings sequentially
Pure cannot invoke capability bindings
```

Pure functions may mention interface types only where the type/elaboration rules allow them, but pure evaluation must not perform capability operations.

## 10. Runtime Invocation Contract

An invocation through a capability binding proceeds conceptually as:

```text
lookup(binding_name, current EffectScopeId / ProcessId / WorkflowId)
  -> CapabilityBinding
check operation exists on binding.interface
check operation mode and admitted authority
execute binding.implementation operation body or host provider action
record provenance/effect/failure evidence
```

A conforming implementation must distinguish:

1. host-backed primitive provider dispatch;
2. Ash-defined implementation body execution;
3. adapter/decorator dispatch through inner capability bindings;
4. internal-resource dispatch backed by [SPEC-053](SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) resource instances.

## 11. Error Conditions

Implementations must report distinct diagnostics or runtime failures for:

1. unknown capability interface;
2. unknown implementation;
3. implementation does not satisfy interface;
4. binding selects implementation for the wrong interface;
5. dependency binding missing;
6. dependency binding type mismatch;
7. binding attempts to manufacture external authority;
8. operation not present on interface;
9. operation body violates declared mode/effect constraints;
10. authority provenance missing.

## 12. Implementation Tasks

- [TASK-720](../plan/tasks/TASK-720-write-spec-052-capability-interface-implementation-contract.md): Write [SPEC-052](SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) capability interface/implementation contract.
- [TASK-722](../plan/tasks/TASK-722-reconcile-capability-resource-spec-ownership.md): Reconcile existing capability specs with [SPEC-052](SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)/[SPEC-053](SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) ownership.
- TASK-724: Add parser and AST substrate for capability interfaces.
- TASK-725: Add parser and AST substrate for capability implementations.
- TASK-727: Export/import capability interface and implementation metadata through modules.
- TASK-729: Add interface operation signature environments.
- TASK-730: Add implementation conformance checking.
- TASK-733: Add module-owned capability binding resolution.
- TASK-736: Add runtime capability binding admission API.
- TASK-741: Execute Ash-defined capability implementation bodies.

## 13. Deferred Questions

1. Exact final concrete grammar for all header clauses.
2. Whether capability binding names share any syntax with ordinary value paths.
3. Whether capability handles ever become first-class values.
4. Whether generic capability interfaces are supported in the first implementation slice.
5. Whether overloading by operation mode/name is ever allowed.
6. How much of existing `pub capability` stdlib surface migrates immediately.
7. Exact serialized representation of capability provenance events.

## Changelog

### 2026-04-27

- Initial draft promoted from [NOTE-009](../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md), defining capability interfaces, capability implementations, capability bindings, module visibility, implementation conformance, binding-time semantics, derived implementations, and runtime invocation boundaries.
