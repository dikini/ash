# Effect-Row Provider Binding Design

**Status:** Implemented by TASK-2025 as a bounded V7 semantic-summary/typechecking contract;
runtime/provider authority remains explicitly out of scope.
**Related task:** [TASK-2025](../plan/tasks/TASK-2025-effect-row-provider-binding-identity-and-sanitization.md)
**Scope boundary:** semantic-summary transport and typechecking only. This design grants no
provider, handler, admission, dispatch, or runtime authority.

## Problem

The current effect-row import path preserves a declaration ID while rewriting an exported name
for an alias, and it carries a transitive best-effort dependency closure. That is an important
start, but it conflates provider-owned identity with caller-visible binding and represents an
inaccessible dependency as a serializable name. The latter can disclose private structure and
cannot support a fail-closed cross-module contract.

This design replaces that implicit convention with an explicit provider/binding boundary.

## Model

### Immutable provider identity

Each exported effect row has one immutable **provider identity**. It is derived from the
provider module's canonical module identity plus its declaration identity and is never changed by
an import alias, glob import, facade module, or public re-export. It is the identity used for
dependency closure, equality, conflict detection, and cache identity.

The provider identity is deliberately separate from a **visible binding**. A binding records the
name visible in one consumer or re-exporting module, its exposure mode, and the immutable provider
identity it denotes. Rebinding `Audit` as `PublicAudit` therefore creates a `PublicAudit` binding
to the same provider; it does not create a provider rooted at the facade.

### Sanitizing closure

Every named import, glob import, and `pub use` of an effect row must pass through one sanitizing
closure operation before its selected summary can be registered or re-exported. The operation:

1. starts at the selected provider identity;
2. traverses only public/exportable row dependencies by immutable provider identity;
3. emits the selected visible binding and only the bindings needed to make its public closure
   resolvable; and
4. classifies a non-exportable, missing, or otherwise inaccessible dependency as opaque.

The closure must be deterministic and independent of import order. A facade may rename or expose
bindings, but it may not alter a provider identity, make a private dependency public, or retain
source-private names merely to make a later import work.

### Opaque inaccessible dependencies

An inaccessible dependency is a boundary condition, not a transported private declaration. The
exported/serialized summary and semantic cache must contain no private dependency name, path,
source anchor, signature, row text, or provider identity for it. Consumers receive only a stable
opaque classification sufficient to reject the binding fail closed. Diagnostics may identify the
public binding and its importing/re-exporting boundary, but must not reveal the inaccessible
provider or private dependency.

### Conflicts

Registration rejects, before publication or cache insertion, any conflict where a visible binding
would denote more than one provider identity, where one provider identity is paired with
incompatible sanitized closure content, or where a selected closure cannot be represented without
an inaccessible dependency. There is no last-import-wins or import-order shadowing rule.

### Versioning and cache coverage

Introduce a new semantic-summary schema version for provider bindings. Its validated payload and
semantic cache key must cover:

- the immutable provider identity for every exported/imported effect-row provider;
- visible binding name and exposure mode, separately from that identity;
- the sanitizer/closure schema version and a deterministic digest of the public closure; and
- the opaque-inaccessible classification without serializing private data.

Older summary payloads and cache entries do not implicitly acquire this meaning. A consumer that
needs provider-binding data must reject an older or incomplete payload, and a cache entry with a
different schema/digest must miss or reject before registration. Unknown newer versions remain
unsupported rather than partially interpreted.

## Invariants

1. Provider identity is immutable under aliasing, glob import, and public re-export.
2. A visible binding maps to exactly one provider identity and one compatible sanitized closure.
3. Named, glob, and public-use paths share the same sanitizer; no alternate path bypasses it.
4. Private dependency information never crosses serialization, cache, or public diagnostic
   boundaries.
5. Conflict handling is deterministic and rejects before cache publication.
6. Effect rows remain non-granting requirements. This design does not imply a provider is
   installed, admitted, invoked, or executable.

## Acceptance evidence

The implementation must prove, with focused tests, that aliases/facades preserve provider
identity; named/glob/`pub use` share closure sanitization; inaccessible private dependencies reject
without leakage; conflicts reject in either import order; and stale/pre-schema caches fail closed.
Existing symbolic `ImplType::operation(args)` resolution and handler-local row inspection remain
separate controls, not alternate provider-binding semantics.

## Non-goals

- General effect-row syntax, expansion taxonomy, or arbitrary handler residual-row semantics.
- Provider selection, handler-frame construction, runtime dispatch, host I/O, timeout,
  cancellation, tracing, monitoring, or admission.
- Backward-compatible interpretation of legacy serialized summaries that lack provider-binding
  information.
