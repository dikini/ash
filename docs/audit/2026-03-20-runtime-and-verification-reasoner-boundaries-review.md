# Audit: Runtime and Verification Reasoner Boundaries

## Scope

Reviewed the runtime-facing contracts in:

- [docs/spec/SPEC-001-IR.md](/home/dikini/Projects/ash/docs/spec/SPEC-001-IR.md)
- [docs/spec/SPEC-004-SEMANTICS.md](/home/dikini/Projects/ash/docs/spec/SPEC-004-SEMANTICS.md)
- [docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md](/home/dikini/Projects/ash/docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md)
- [docs/spec/SPEC-018-CAPABILITY-MATRIX.md](/home/dikini/Projects/ash/docs/spec/SPEC-018-CAPABILITY-MATRIX.md)

Review protocol:

- [docs/reference/runtime-reasoner-separation-rules.md](/home/dikini/Projects/ash/docs/reference/runtime-reasoner-separation-rules.md)

Constraints of this audit:

- Reporting only
- No normative spec files changed
- Classify each reviewed area as `runtime-only`, `interaction-layer`, or `split`
- Distinguish `Aligned`, `Silent`, and `Tension`

Review date: 2026-03-20

## Summary

The reviewed runtime and verification specs are consistently `runtime-only`. They define authoritative
IR contracts, effect semantics, capability validation, approval routing, and provenance boundaries
without depending on a reasoner being present. No `interaction-layer` or `split` concern was found
inside the reviewed runtime docs.

The main residual gap is intentional silence: these specs do not define projection, injected
reasoner context, or reasoner acceptance semantics. That omission is correct for runtime contracts,
but it means the runtime-to-reasoner interaction story must remain in a separate contract family.

## Findings

### 1. SPEC-001 is cleanly `runtime-only` and aligned with the separation rules

- [docs/spec/SPEC-001-IR.md#L7-L19](/home/dikini/Projects/ash/docs/spec/SPEC-001-IR.md#L7-L19)
- [docs/spec/SPEC-001-IR.md#L21-L32](/home/dikini/Projects/ash/docs/spec/SPEC-001-IR.md#L21-L32)
- [docs/spec/SPEC-001-IR.md#L38-L55](/home/dikini/Projects/ash/docs/spec/SPEC-001-IR.md#L38-L55)
- [docs/spec/SPEC-001-IR.md#L56-L79](/home/dikini/Projects/ash/docs/spec/SPEC-001-IR.md#L56-L79)
- [docs/spec/SPEC-001-IR.md#L171-L183](/home/dikini/Projects/ash/docs/spec/SPEC-001-IR.md#L171-L183)

Classification: `runtime-only`

Status: `Aligned`

Reasoning: The IR contract is explicit about execution neutrality, canonical core forms, and effect
semantics. It defines stable runtime meaning without any need for a reasoner-side model.

### 2. SPEC-004 is `runtime-only`; its silence on reasoner projection is appropriate

- [docs/spec/SPEC-004-SEMANTICS.md#L7-L11](/home/dikini/Projects/ash/docs/spec/SPEC-004-SEMANTICS.md#L7-L11)
- [docs/spec/SPEC-004-SEMANTICS.md#L31-L38](/home/dikini/Projects/ash/docs/spec/SPEC-004-SEMANTICS.md#L31-L38)
- [docs/spec/SPEC-004-SEMANTICS.md#L47-L62](/home/dikini/Projects/ash/docs/spec/SPEC-004-SEMANTICS.md#L47-L62)
- [docs/spec/SPEC-004-SEMANTICS.md#L68-L109](/home/dikini/Projects/ash/docs/spec/SPEC-004-SEMANTICS.md#L68-L109)
- [docs/spec/SPEC-004-SEMANTICS.md#L111-L230](/home/dikini/Projects/ash/docs/spec/SPEC-004-SEMANTICS.md#L111-L230)

Classification: `runtime-only`

Status: `Silent`

Reasoning: The operational semantics fully cover acquisition, deliberation, gating, rejection
boundaries, and effectful commitment. They do not discuss reasoner projection or advisory acceptance,
which is correct for a runtime semantic contract, but leaves the reasoner-facing layer entirely
outside this file.

### 3. SPEC-017 remains `runtime-only`, including monitor/exposed-view handling

- [docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md#L7-L20](/home/dikini/Projects/ash/docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md#L7-L20)
- [docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md#L196-L219](/home/dikini/Projects/ash/docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md#L196-L219)
- [docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md#L250-L259](/home/dikini/Projects/ash/docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md#L250-L259)

Classification: `runtime-only`

Status: `Aligned`

Reasoning: The capability integration spec keeps `observe`, `receive`, `set`, `send`, and monitor
views inside runtime ownership. `exposes` and `MonitorLink` are treated as read-only runtime
monitoring features, not as reasoner projection machinery.

### 4. SPEC-018 is `runtime-only`; approval routing stays within runtime verification

- [docs/spec/SPEC-018-CAPABILITY-MATRIX.md#L7-L18](/home/dikini/Projects/ash/docs/spec/SPEC-018-CAPABILITY-MATRIX.md#L7-L18)
- [docs/spec/SPEC-018-CAPABILITY-MATRIX.md#L32-L50](/home/dikini/Projects/ash/docs/spec/SPEC-018-CAPABILITY-MATRIX.md#L32-L50)
- [docs/spec/SPEC-018-CAPABILITY-MATRIX.md#L79-L103](/home/dikini/Projects/ash/docs/spec/SPEC-018-CAPABILITY-MATRIX.md#L79-L103)
- [docs/spec/SPEC-018-CAPABILITY-MATRIX.md#L105-L218](/home/dikini/Projects/ash/docs/spec/SPEC-018-CAPABILITY-MATRIX.md#L105-L218)

Classification: `runtime-only`

Status: `Aligned`

Reasoning: The verification matrix is framed as runtime compatibility checking over capabilities,
obligations, policy outcomes, and effect ceilings. `RequireApproval` and related runtime outcomes are
handled as verification-time concerns, not reasoner-facing state.

## Cross-Cutting Observation

No reviewed area was `interaction-layer` or `split`. That is the correct result for these four
contracts: they are runtime authority documents, not runtime-to-reasoner interaction documents.

The only substantive omission is a separate interaction contract for projection, injected context,
and reasoner-produced advisory artifacts. That omission is `Silent` by design and should be handled
in a distinct contract family, not by expanding the runtime specs reviewed here.

## Conclusion

No tensions were found in the reviewed runtime and verification contracts. The docs are consistent
with the frozen separation rules, and the runtime-only boundary remains intact.
