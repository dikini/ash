# Wiremock TCP Capability Gate Design

**Goal:** Keep the four affected Ash Engine loopback integration targets authoritative on capable
hosts: the LLM Wiremock targets (`llm_integration_tests` and `llm_provider_integration`), the
MCP Wiremock target (`mcp_provider_tests`), and the HTTP-wrapper target
(`task_1937_http_provider_wrappers`). The managed sandbox refuses to bind a local TCP listener.

**Architecture:** Leave `LlmProvider`, Wiremock responses, and all test assertions unchanged. A
test-only loopback TCP preflight distinguishes `PermissionDenied` from every other bind result.
Only the former reports the host as unsupported; success permits the existing suite to run and
all other errors fail loudly.

## Alternatives considered

1. Change provider networking. Rejected: Wiremock fails before provider construction.
2. Treat all mock-server startup errors as skips. Rejected: it could hide a broken test harness.
3. Skip the suite unconditionally in this sandbox. Rejected: capable hosts would lose integration
   coverage.
4. Precise test-only bind capability preflight. Selected: it preserves product coverage where the
   host can supply the required local resource.

## TDD plan

1. Reproduce the exact Wiremock bind `PermissionDenied` on the isolated integration target.
2. Add a small loopback bind preflight in each affected test target that only classifies that
   error as unsupported.
3. Keep existing test bodies intact behind the preflight and run the full workspace gate.
