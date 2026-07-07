# PLAN-198: Standard Providers And Profiles

**Status:** In Progress (3/8 tasks complete)
**Depends on:** Phase 183 Operation And Authority Model, Phase 184 Handler / Provider Semantics,
Phase 194 Contract And Evidence System, Phase 195 Process And Concurrency Model, Phase 196
Application / Workflow Runtime, and Phase 197 Host / FFI / Builtins.
**Specs/notes:** `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`,
`PLAN-183`, `PLAN-184`, `PLAN-194`, `PLAN-195`, `PLAN-196`, `PLAN-197`, `NOTE-016`,
`NOTE-020`, `NOTE-021`, `NOTE-024`, `NOTE-025`, and `NOTE-035`.

## Goal

Turn the Phase 197 host boundary substrate into usable standard Ash provider libraries and
admission profiles for filesystem, HTTP, clock/time, logging, and contract/evidence helper use.

## Architecture

Phase 198 is the provider/profile foundation for productive Ash. Standard providers are ordinary
operation surfaces backed by trusted runtime adapters, builtin hook metadata, sandbox policy,
provider authoring metadata, and redacted provenance. Standard profiles are named convenience
bundles over explicit row/admission/resource/provider expectations; selecting a profile must never
grant authority by name or bypass handler/provider admission.

## Scope

- Audit the current stdlib provider modules, runtime provider implementations, examples, and tests.
- Implement or repair final-surface wrappers for filesystem, HTTP, clock/time, and logging.
- Add deterministic test-clock support for repeatable tests and evidence.
- Add common row/admission profiles for read-only filesystem, read-write filesystem, sandboxed HTTP,
  deterministic test, logging-only, and application-default cases.
- Add small contract/evidence helper modules for common checks and evidence projection.
- Add final-surface fixtures that exercise the standard provider/profile path through target
  function/application entrypoints.

## Non-Goals

- No ambient provider access from ordinary expressions.
- No broad app templates, scaffolding CLI, or tutorial app corpus; those are Phase 199.
- No user-defined native FFI or revived `extern` surface.
- No hidden authority from profile names, stdlib imports, or application entrypoint selection.
- No legacy `workflow` syntax revival as the target path for provider examples.

## Design Locks

1. Provider wrappers must use Phase 197 metadata and fail closed when metadata, sandbox policy, row
   discharge, or provenance policy is missing.
2. Profiles are selection/configuration artifacts only; authority remains discharged by explicit
   rows, providers, roles, resources, policies, and evidence.
3. Every host boundary attempt must produce redacted evidence, including denied attempts.
4. Deterministic testing must use explicit test-clock/profile inputs, not wall-clock leakage.
5. Final-surface examples must route through target `fn main` and application/runtime entrypoints,
   not legacy `workflow` as a primitive.

## Task Overview

| Task | Description | Estimate | Depends on | Status |
|------|-------------|----------|------------|--------|
| [TASK-1934](tasks/TASK-1934-standard-providers-profiles-plan-packet.md) | Create the Phase 198 plan and task packet | 2h | Phase 197 | ✅ Complete |
| [TASK-1935](tasks/TASK-1935-standard-provider-profile-audit.md) | Audit stdlib provider modules, runtime providers, examples, and profile seams | 6h | TASK-1934 | ✅ Complete |
| [TASK-1936](tasks/TASK-1936-filesystem-provider-wrappers-and-profiles.md) | Implement filesystem stdlib wrappers and read/write row profiles | 14h | TASK-1935 | ✅ Complete |
| [TASK-1937](tasks/TASK-1937-http-provider-wrappers-and-profiles.md) | Implement HTTP stdlib wrappers and sandboxed network profiles | 14h | TASK-1935 | ✅ Complete |
| [TASK-1938](tasks/TASK-1938-clock-time-provider-and-test-clock.md) | Implement clock/time wrappers and deterministic test-clock profile support | 12h | TASK-1935 | Planned |
| [TASK-1939](tasks/TASK-1939-logging-provider-redaction-and-provenance.md) | Implement logging wrappers with redaction and provenance evidence | 12h | TASK-1935 | Planned |
| [TASK-1940](tasks/TASK-1940-common-row-admission-profiles.md) | Add common row/admission profile definitions and validation fixtures | 12h | TASK-1936, TASK-1937, TASK-1938, TASK-1939 | In Progress |
| [TASK-1941](tasks/TASK-1941-contract-evidence-helper-library-and-closeout.md) | Add contract/evidence helpers, final-surface fixtures, docs, gates, and closeout | 10h | TASK-1940 | Planned |

Estimated implementation effort after the plan packet: 80 hours.

## Required Test Families

### Provider Wrapper Tests

Filesystem, HTTP, clock/time, and logging wrappers must parse, check, lower, and execute through the
real stdlib import path. Tests must cover success, denied access, missing provider metadata,
malformed arguments, sandbox rejection, and provenance emission.

### Profile Tests

Profiles must fail closed when missing, malformed, stale, authority-widening, incompatible with
provider/resource bindings, or attempting to bypass row discharge. Positive tests must prove common
profiles select only explicit row/admission expectations.

### Evidence Tests

Host boundary evidence must be redacted, authority-neutral, and present for success, failure, and
denial. Contract/evidence helpers must not acquire provider authority while inspecting reports,
trace facts, or monitor evidence.

### Final-Surface Fixtures

Fixtures must use current target function/application entrypoints. Legacy `workflow` examples may
remain only as compatibility or historical references.

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

Before closeout, search live normative docs, stdlib comments, examples, and code comments for stale
claims:

```text
provider.*ambient authority
profile.*grants authority
filesystem.*bypasses sandbox
http.*bypasses sandbox
clock.*wall clock.*test
logging.*no redaction
evidence helper.*provider authority
workflow.*standard provider target
builtin.*direct stdlib host access
```

Historical docs may retain old provider or workflow wording only when clearly marked as
compatibility or historical context.

## Acceptance Criteria

- [ ] Phase 198 plan and task files exist and are indexed.
- [ ] Stdlib provider/profile seams are audited against Phase 197 metadata.
- [x] Filesystem wrappers execute only through explicit provider/admission/sandbox boundaries.
- [x] HTTP wrappers enforce host/method/body/header sandbox policy before host effects.
- [ ] Clock/time wrappers support deterministic test-clock behavior.
- [ ] Logging wrappers emit redacted provenance and report evidence.
- [ ] Common profiles validate explicit row/admission expectations without granting authority.
- [ ] Contract/evidence helpers inspect evidence without acquiring host/provider authority.
- [ ] Final-surface fixtures cover provider/profile success, denial, diagnostics, and evidence.
