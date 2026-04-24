# Track C Tasks: Capability Provider Surface

## TASK-666: HTTP Capability Provider

**Spec:** SPEC-017 (capability integration)
**Track:** C
**Depends on:** D3 resolution (constraint model)
**Est. Hours:** 6-8

Implement an HTTP capability provider:

1. `HttpProvider` implementing `CapabilityProvider` trait
2. Operations: `get`, `post`, `put`, `delete`, `head`
3. Constraint model: `HttpConstraints { allowed_methods, allowed_hosts, max_body_size, timeout }`
4. Request/response value mapping (Ash Record → HTTP request, HTTP response → Ash Record)
5. Header and body content-type handling
6. Error mapping to `CapabilityError`

Uses `reqwest` (already in the workspace via `async-openai`).

**Files:**
- Create: `crates/ash-engine/src/providers/http.rs`
- Modify: `crates/ash-engine/src/providers/mod.rs` (exports)

**Verification:** Provider passes `CapabilityProvider` tests. HTTP get with constraint enforcement works.

---

## TASK-667: Time Capability Provider

**Track:** C
**Depends on:** D3 resolution
**Est. Hours:** 3-4

Implement a time capability provider:

1. `TimeProvider` implementing `CapabilityProvider`
2. Operations: `now` → Time value, `sleep` (async), `format` (Time → String), `parse_time` (String → Time)
3. Constraint model: `TimeConstraints { allowed_operations, timezone }`
4. Uses `chrono` (add to workspace deps if not present)

**Files:**
- Create: `crates/ash-engine/src/providers/time.rs`
- Modify: `crates/ash-engine/src/providers/mod.rs`

**Verification:** `now()` returns current time. `format(now(), "%Y-%m-%d")` returns formatted string.

---

## TASK-668: Process Provider Hardening

**Track:** C
**Depends on:** D3 resolution
**Est. Hours:** 4-6

Upgrade the existing `process::run` builtin with proper capability constraints:

1. `ProcessProvider` implementing `CapabilityProvider` (move from builtin to provider)
2. Constraint model: `ProcessConstraints { allowed_commands, capture_output, timeout, max_output_bytes }`
3. Argument sanitization and shell-injection prevention
4. Timeout enforcement (tokio::time::timeout)
5. Output truncation for large outputs
6. Error mapping with exit-code preservation

**Files:**
- Create: `crates/ash-engine/src/providers/process.rs`
- Modify: `crates/ash-engine/src/providers/mod.rs`
- Modify: `crates/ash-interp/src/builtins/process.rs` (delegate to provider)

**Verification:** `process::run("echo hello")` works. Disallowed commands are rejected. Timeout kills hung processes.
