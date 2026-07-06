# PLAN-197: Host / FFI / Builtins

**Status:** ✅ Complete (10/10 tasks complete)
**Depends on:** Phase 183 Operation And Authority Model, Phase 184 Handler / Provider Semantics,
Phase 194 Contract And Evidence System, Phase 195 Process And Concurrency Model, and Phase 196
Application / Workflow Runtime.
**Specs/notes:** `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`,
`PLAN-183`, `PLAN-184`, `PLAN-194`, `PLAN-195`, `PLAN-196`, `NOTE-016`, `NOTE-020`,
`NOTE-021`, `NOTE-025`, and `NOTE-035`.

## Goal

Expose host functionality carefully through audited builtins, provider authoring APIs, trusted
runtime adapters, sandboxing, and provenance without creating a backdoor around Ash authority
semantics.

## Architecture

Host access is an authority boundary, not a convenience layer. Phase 197 treats builtin host hooks,
provider implementations, runtime adapters, and any future `extern` surface as different views over
the same explicit authority substrate: declared operation rows, admitted provider/resource bindings,
application boundary metadata, sandbox policy, and redacted provenance evidence. The phase comes
after authority, handler/provider, contract/evidence, process, and application runtime work so host
integration cannot bypass row admission, policy checks, sendability, cancellation, or report/trace
obligations.

## Scope

- Audit current builtin dispatch, host provider, runtime artifact, daemon, sandbox, and provenance
  seams.
- Define builtin host hooks as trusted runtime hooks with explicit capability, row, sandbox, and
  provenance metadata.
- Design and implement a provider authoring API that fails closed unless providers declare
  operation surfaces, effect levels, constraints, resource use, and provenance behavior.
- Register trusted runtime adapters separately from untrusted user code and application entrypoint
  metadata.
- Enforce sandbox policy for filesystem, process, network, environment, clock, LLM, MCP, and other
  host-facing providers.
- Attach redacted provenance and report evidence to every host boundary crossing.
- Decide whether an `extern` surface is still needed after provider/builtin adapters are explicit;
  if needed, plan it as a later authority-checked surface, not as part of this phase's MVP.

## Non-Goals

- No ambient host calls from ordinary expressions.
- No `extern` keyword or FFI ABI surface until the decision gate proves provider/builtin adapters
  are insufficient.
- No builtin that grants authority merely because it is in the stdlib or dispatch table.
- No bypass of handler/provider admission, row discharge, application admission profiles, process
  sendability, sandbox constraints, or contract/evidence checks.
- No raw plugin ABI, dynamic library loading, or unconstrained native callbacks in this phase.

## Design Locks

1. Builtins are not trusted by name. A builtin host hook must have explicit hook metadata,
   capability/operation identity, effect classification, sandbox policy, and provenance behavior.
2. Provider authoring APIs must make authority and constraints visible at registration time before
   any provider operation can execute.
3. Runtime adapters are admitted host components, not language semantics. They must be identified,
   versioned, sandboxed, and reported.
4. Sandboxing applies before execution, and provenance applies after every attempted boundary
   crossing, including denied attempts.
5. `extern` remains a future option only. If retained, it must lower to the same trusted adapter and
   provider admission substrate rather than introducing a parallel host-call path.
6. Host boundary reports and traces are evidence artifacts; they must not mutate authority or leak
   secrets.

## Task Overview

| Task | Description | Estimate | Depends on | Status |
|------|-------------|----------|------------|--------|
| [TASK-1924](tasks/TASK-1924-host-ffi-builtins-plan-packet.md) | Create the Phase 197 plan and task packet | 2h | Phase 196 | ✅ Complete |
| [TASK-1925](tasks/TASK-1925-host-boundary-seam-audit.md) | Audit builtin, provider, runtime adapter, sandbox, and provenance seams | 6h | TASK-1924 | ✅ Complete |
| [TASK-1926](tasks/TASK-1926-builtin-host-hook-metadata.md) | Add builtin host hook metadata and fail-closed diagnostics | 12h | TASK-1925 | ✅ Complete |
| [TASK-1927](tasks/TASK-1927-provider-authoring-api.md) | Define provider authoring API for operation surfaces, constraints, resources, and effects | 16h | TASK-1926 | ✅ Complete |
| [TASK-1928](tasks/TASK-1928-trusted-runtime-adapter-registry.md) | Add trusted runtime adapter registry with identity, versioning, and admission boundaries | 14h | TASK-1927 | ✅ Complete |
| [TASK-1929](tasks/TASK-1929-host-sandbox-policy-enforcement.md) | Enforce sandbox policies for host-facing providers and adapters | 16h | TASK-1928 | ✅ Complete |
| [TASK-1930](tasks/TASK-1930-host-provenance-and-redaction.md) | Attach provenance, trace, report, and redaction evidence to host boundary crossings | 12h | TASK-1929 | ✅ Complete |
| [TASK-1931](tasks/TASK-1931-extern-decision-gate.md) | Decide whether `extern` is still needed and document the authority-checked path | 6h | TASK-1930 | ✅ Complete |
| [TASK-1932](tasks/TASK-1932-host-boundary-cross-boundary-fixtures.md) | Add cross-boundary fixtures for builtins, providers, adapters, sandboxing, and provenance | 12h | TASK-1931 | ✅ Complete |
| [TASK-1933](tasks/TASK-1933-host-ffi-builtins-closeout.md) | Close out Phase 197 with docs, changelog, gates, and review remediation | 8h | TASK-1932 | ✅ Complete |

Estimated implementation effort after the plan packet: 102 hours.

## Required Test Families

### Builtin Hook Tests

Prove builtin host hooks cannot execute without explicit metadata, operation identity, effect
classification, sandbox policy, and provenance configuration. Cover existing builtin dispatch table
entries, stdlib `pub builtin fn` declarations, and unimplemented builtin diagnostics.

### Provider Authoring Tests

Cover provider registration, operation surface declaration, row discharge, effect level
classification, constraints, resource ownership, and malformed provider definitions. Providers must
fail closed when metadata is missing, stale, overbroad, or authority-widening.

### Runtime Adapter Tests

Verify trusted runtime adapters have stable identity, version, admission source, sandbox policy,
provenance policy, and report identity. Runtime adapters must not become target language semantics
or application-entrypoint authority.

### Sandbox Tests

Cover filesystem paths, process commands, network hosts, environment variables, clocks/time,
LLM/MCP calls, and denied attempts. Sandbox checks must happen before host effects and must retain
redacted denial evidence.

### Provenance And Report Tests

Assert every host boundary crossing emits redacted provenance, runtime trace facts, monitor/report
evidence, and diagnostics without leaking secrets or mutating admission state.

### Extern Decision Tests

If `extern` is rejected for the MVP, tests should prove provider/builtin adapters cover current
host surfaces. If retained for later work, tests should assert `extern` remains unimplemented or
fails closed until it lowers through the trusted adapter substrate.

## Verification Gates

Each implementation task must run focused tests for touched crates plus:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
```

Phase closeout must run:

```bash
cargo fmt --check
cargo test --all
cargo clippy --all-targets --all-features
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```

## Stale-Claim Sweep

Before closeout, search live normative docs and code comments for stale claims:

```text
builtin.*trusted by name
builtin.*bypasses provider
host hook.*ambient authority
provider authoring.*implicit authority
runtime adapter.*language primitive
extern.*direct host call
extern.*bypasses admission
sandbox.*after execution
provenance.*optional host
host.*no redaction
FFI.*unconstrained
```

Historical docs may discuss older builtin/host behavior when clearly marked as current-state
compatibility or legacy context. Target guidance must route host access through explicit operation
rows, provider/admission boundaries, sandbox policy, and provenance evidence.

## Acceptance Criteria

- [x] Phase 197 plan and task files exist and are indexed.
- [x] Host boundary seam audit identifies builtin, provider, adapter, sandbox, and provenance owners.
- [x] Builtin host hooks require explicit metadata and fail closed when metadata is missing.
- [x] Provider authoring API exposes operation surfaces, constraints, resources, effects, and
      provenance without granting ambient authority.
- [x] Trusted runtime adapters are registered and reported as host components, not language
      primitives.
- [x] Sandbox policy is enforced before host effects and records denied attempts.
- [x] Host provenance and redaction evidence is emitted for success, failure, and denial.
- [x] `extern` is either explicitly deferred or specified as a later authority-checked adapter
      surface.
- [x] Cross-boundary fixtures cover builtins, providers, adapters, sandboxing, reports, and
      provenance.
