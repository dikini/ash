# Audit: SPEC-001 through SPEC-018 Consistency Review

## Scope

Reviewed the specification set from [docs/spec/SPEC-001-IR.md](../spec/SPEC-001-IR.md) through [docs/spec/SPEC-018-CAPABILITY-MATRIX.md](../spec/SPEC-018-CAPABILITY-MATRIX.md) for cross-spec consistency.

Constraints of this audit:
- Reporting only
- No specification files changed
- Focus on terminology, syntax, cross-references, model alignment, and scope consistency

Review date: 2026-03-19

## Summary

The spec set is directionally coherent, especially around the effect lattice and the input/output capability split, but there are several notable inconsistencies across adjacent specs. The most significant issues are:
- conflicting scope between module/import specs,
- unstable syntax for `receive`,
- incomplete runtime model definitions in SPEC-018,
- stale or incorrect cross-references,
- policy model drift across policy-related specs.

## High-Severity Findings

### 1. Import scope conflict between module and import specs

- [docs/spec/SPEC-009-MODULES.md](../spec/SPEC-009-MODULES.md#L139)
- [docs/spec/SPEC-009-MODULES.md](../spec/SPEC-009-MODULES.md#L215-L220)
- [docs/spec/SPEC-012-IMPORTS.md](../spec/SPEC-012-IMPORTS.md#L67-L122)

`SPEC-009` treats `use` and `pub use` as future work, while `SPEC-012` defines them as active language features.

### 2. `receive` syntax is inconsistent across stream and capability specs

- [docs/spec/SPEC-013-STREAMS.md](../spec/SPEC-013-STREAMS.md#L58-L64)
- [docs/spec/SPEC-013-STREAMS.md](../spec/SPEC-013-STREAMS.md#L68-L99)
- [docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md](../spec/SPEC-017-CAPABILITY-INTEGRATION.md#L392-L399)

`SPEC-013` defines one receive-arm grammar, then uses a different example form immediately after. `SPEC-017` introduces yet another direct receive form.

### 3. SPEC-018 runtime context is underspecified relative to its own algorithms

- [docs/spec/SPEC-018-CAPABILITY-MATRIX.md](../spec/SPEC-018-CAPABILITY-MATRIX.md#L35-L40)
- [docs/spec/SPEC-018-CAPABILITY-MATRIX.md](../spec/SPEC-018-CAPABILITY-MATRIX.md#L104-L113)
- [docs/spec/SPEC-018-CAPABILITY-MATRIX.md](../spec/SPEC-018-CAPABILITY-MATRIX.md#L239-L268)

`RuntimeContext` is defined without `max_effect`, `policy_evaluator`, or `rate_limiter`, but later verification logic depends on all three.

### 4. SPEC-018 error and warning names drift internally

- [docs/spec/SPEC-018-CAPABILITY-MATRIX.md](../spec/SPEC-018-CAPABILITY-MATRIX.md#L47-L64)
- [docs/spec/SPEC-018-CAPABILITY-MATRIX.md](../spec/SPEC-018-CAPABILITY-MATRIX.md#L157-L190)
- [docs/spec/SPEC-018-CAPABILITY-MATRIX.md](../spec/SPEC-018-CAPABILITY-MATRIX.md#L395-L410)

The matrices use names such as `CapabilityUnavailable`, `NotWritable`, and `UnfulfilledObligation`, while the enums/examples use `MissingCapability`, `NotSettable`, and `MissingObligation`.

### 5. Stale forward reference in typed providers spec

- [docs/spec/SPEC-015-TYPED-PROVIDERS.md](../spec/SPEC-015-TYPED-PROVIDERS.md#L327)
- [docs/spec/SPEC-016-OUTPUT.md](../spec/SPEC-016-OUTPUT.md#L1-L14)

`SPEC-015` says schema-first code generation is `SPEC-016`, but `SPEC-016` is actually the output-capabilities spec.

### 6. Policy conflict example in SPEC-007 contradicts its own SMT encoding

- [docs/spec/SPEC-007-POLICY-COMBINATORS.md](../spec/SPEC-007-POLICY-COMBINATORS.md#L284-L298)

The example claims two rate-limit constraints are conflicting, but the provided SMT constraints are satisfiable together.

## Medium-Severity Findings

### 7. Policy model drifts across SPEC-006, SPEC-007, and SPEC-008

- [docs/spec/SPEC-006-POLICY-DEFINITIONS.md](../spec/SPEC-006-POLICY-DEFINITIONS.md#L14-L57)
- [docs/spec/SPEC-007-POLICY-COMBINATORS.md](../spec/SPEC-007-POLICY-COMBINATORS.md#L20-L46)
- [docs/spec/SPEC-008-DYNAMIC-POLICIES.md](../spec/SPEC-008-DYNAMIC-POLICIES.md#L28-L68)

These specs present policies as:
- named struct-like declarations,
- functional combinators,
- runtime-loaded `Policy` values,
without fully reconciling the abstraction levels or syntax between them.

### 8. Provider effect granularity is coarser than the type/effect model

- [docs/spec/SPEC-010-EMBEDDING.md](../spec/SPEC-010-EMBEDDING.md#L84-L113)
- [docs/spec/SPEC-003-TYPE-SYSTEM.md](../spec/SPEC-003-TYPE-SYSTEM.md#L49-L109)

The embedding API assigns one `effect()` per provider, while the type system distinguishes read-only and side-effecting operations more precisely.

### 9. REPL commands and options drift from CLI spec

- [docs/spec/SPEC-011-REPL.md](../spec/SPEC-011-REPL.md#L74-L80)
- [docs/spec/SPEC-011-REPL.md](../spec/SPEC-011-REPL.md#L170-L180)
- [docs/spec/SPEC-005-CLI.md](../spec/SPEC-005-CLI.md#L176-L193)

The REPL spec lists commands and flags that differ from the CLI spec.

### 10. REPL type names drift from the canonical type system

- [docs/spec/SPEC-011-REPL.md](../spec/SPEC-011-REPL.md#L84-L91)
- [docs/spec/SPEC-003-TYPE-SYSTEM.md](../spec/SPEC-003-TYPE-SYSTEM.md#L27-L35)

The REPL examples use `Number`, while the type system defines `Int`.

### 11. Capability declaration rule is contradicted by a recovery example

- [docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md](../spec/SPEC-017-CAPABILITY-INTEGRATION.md#L389-L417)
- [docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md](../spec/SPEC-017-CAPABILITY-INTEGRATION.md#L564-L587)

The spec says all capabilities must be declared, but an example sends `alert:critical` without declaring it.

### 12. Provenance event shape mismatch in SPEC-017

- [docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md](../spec/SPEC-017-CAPABILITY-INTEGRATION.md#L291-L303)
- [docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md](../spec/SPEC-017-CAPABILITY-INTEGRATION.md#L363-L379)

`ProvenanceEvent` is defined without a `result` field, but later execution logic records one.

## Low-Severity Findings

### 13. Example type names are not normalized

- [docs/spec/SPEC-009-MODULES.md](../spec/SPEC-009-MODULES.md#L35)
- [docs/spec/SPEC-009-MODULES.md](../spec/SPEC-009-MODULES.md#L79)
- [docs/spec/SPEC-012-IMPORTS.md](../spec/SPEC-012-IMPORTS.md#L201)
- [docs/spec/SPEC-003-TYPE-SYSTEM.md](../spec/SPEC-003-TYPE-SYSTEM.md#L27-L35)

Examples use `string` and `json`, while the canonical type system uses `String` and does not define `json`.

### 14. Editorial/status formatting is uneven across the spec set

Some specs use an explicit `Status` section and tightly structured numbering, while others are less uniform. This is editorial rather than semantic, but it makes the set feel less synchronized.

## Areas That Appear Consistent

### Effect split between input and output capabilities

- [docs/spec/SPEC-016-OUTPUT.md](../spec/SPEC-016-OUTPUT.md#L210-L220)
- [docs/spec/SPEC-018-CAPABILITY-MATRIX.md](../spec/SPEC-018-CAPABILITY-MATRIX.md#L7-L20)

The distinction between `observe` / `receive` as Epistemic and `set` / `send` as Operational is stable.

### Stream vs behaviour conceptual model

- [docs/spec/SPEC-013-STREAMS.md](../spec/SPEC-013-STREAMS.md#L1-L20)
- [docs/spec/SPEC-014-BEHAVIOURS.md](../spec/SPEC-014-BEHAVIOURS.md#L1-L20)

The discrete-vs-continuous distinction is described consistently.

## Conclusion

The reviewed specs are broadly aligned in overall direction, but they are not fully synchronized at the syntax, naming, and runtime-model levels. The main concentration of inconsistencies is around imports/modules, policies, capability runtime verification, and command-language examples.

No repository files in `docs/spec/` were modified as part of this audit.
