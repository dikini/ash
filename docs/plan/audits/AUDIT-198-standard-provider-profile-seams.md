# AUDIT-198: Standard Provider/Profile Seams

**Status:** Complete for TASK-1935 planning/audit ownership
**Phase:** [PLAN-198: Standard Providers And Profiles](../PLAN-198-STANDARD-PROVIDERS-AND-PROFILES.md)
**Task:** [TASK-1935](../tasks/TASK-1935-standard-provider-profile-audit.md)

## Summary

Phase 197 left the required host-boundary substrate in place: standard providers expose explicit
provider authoring metadata, `RuntimeState` retains host sandbox policies, projected provider
bindings enforce sandbox decisions before host execution, and every attempted projected host call can
emit redacted host-boundary evidence. Phase 198 should build on that path rather than adding a
parallel provider mechanism.

The first implementation slice adds `ash_engine::standard_profiles`, a `LoggingProvider`, and a
runtime provider-registration helper. The profiles install providers, sandbox policies, and explicit
capability bindings while keeping profile metadata authority-neutral.

## Provider Surface Inventory

| Surface | Current state | Phase 198 owner |
|---|---|---|
| `std/src/io/fs.ash` | Parser-checkable builtin declarations exist. Runtime `FsProvider` supports read/write/append/copy/rename/remove/dir/metadata operations with explicit metadata. | TASK-1936 |
| `std/src/io/dir.ash` | Parser-checkable builtin declarations exist. Runtime directory operations are exposed by `FsProvider` metadata and execution. | TASK-1936 |
| `std/src/io/path.ash` | Current pure path helpers exist as ordinary Ash. | TASK-1936 |
| `std/src/http.ash` | Parser-checkable builtin declarations exist. Runtime `HttpProvider` supports get/head/post/put/delete with explicit metadata and host allow-list config. | TASK-1937 |
| `std/src/time.ash` | Parser-checkable builtin declarations exist. Runtime `TimeProvider` supports `mock` deterministic time, now/ISO/epoch/sleep metadata. | TASK-1938 |
| `std/src/process.ash` | Runtime `ProcessProvider` already has explicit Phase 197 metadata. Process libraries are not a Phase 198 standard-provider target except as profile compatibility context. | Deferred to Phase 199 process/channel library work |
| `std/src/runtime/args.ash` | Capability declaration only; no concrete Phase 198 provider wrapper target. | Deferred |
| `std/src/runtime/error.ash` | Type-only runtime error surface. | No Phase 198 action |
| Logging | No prior stdlib provider module. Runtime provider added as `LoggingProvider` with debug/info/warn/error metadata. | TASK-1939 |

## Profile And Evidence Seams

| Seam | Current state | Owner |
|---|---|---|
| Provider authoring metadata | `ProviderAuthoringMetadata` and validation exist in `ash-core`; fs/http/time/process/stdio/mcp/llm providers override compatibility shims. | TASK-1936 through TASK-1939 |
| Sandbox policy registration | `RuntimeState::register_host_sandbox_policy` stores policies by identity. | TASK-1940 |
| Projected provider enforcement | `create_capability_context_for_bindings` wraps admitted host bindings with `ProjectedProviderWrapper`; wrapper checks sandbox and records evidence. | TASK-1940 |
| Host-boundary evidence | `HostBoundaryEvidence`, sandbox denials, trace facts, and monitor evidence are retained by `RuntimeState`. | TASK-1936 through TASK-1941 |
| Standard profiles | Added `StandardProviderProfile` for read-only fs, read-write fs, sandboxed HTTP, deterministic test, logging-only, and application-default profiles. | TASK-1940 |
| Contract/evidence helpers | Not implemented yet. Need source-visible helpers and final-surface fixtures. | TASK-1941 |

## Classification

- **Usable now:** runtime fs/http/time provider metadata; deterministic time provider constructor;
  projected provider sandbox/evidence path; standard profile installation API; logging runtime
  provider metadata.
- **Stubbed or parser-only:** stdlib `.ash` builtin declarations remain parser-checkable surfaces,
  but final-source wrapper ergonomics still need TASK-1936 through TASK-1939.
- **Deferred:** process/channel convenience libraries and app templates belong to Phase 199.
- **No authority shortcut:** standard profiles admit explicit `CapabilityBinding` rows and register
  sandbox policies; they do not grant authority by profile name.

## Evidence

- `cargo test -p ash-engine --test task_1940_standard_provider_profiles` covers:
  - read-only filesystem profile admits read rows and rejects write projection;
  - deterministic test profile installs fixed `TimeProvider`;
  - sandboxed HTTP profile exposes explicit method/host profile metadata;
  - application-default profile installs explicit fs/http/time/logging rows;
  - malformed and authority-widening profiles fail closed;
  - projected fs/time success, fs failure, HTTP denial, and logging denial retain redacted evidence;
  - logging-only profile records redacted denied host-boundary evidence;
  - fs/http/time/logging provider metadata validates without compatibility shims.
