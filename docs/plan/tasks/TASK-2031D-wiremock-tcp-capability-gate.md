# TASK-2031D: Wiremock TCP Capability Gate

**Status:** Complete
**Phase:** [PLAN-203](../PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md)
**Type:** Bounded test-host remediation
**Depends on:** TASK-536 LLM mock integration coverage
**Blocks:** TASK-2031 workspace-gate closeout

## Description

Preserve the existing Ash Engine loopback integration targets while making their host-network
requirement explicit. The managed sandbox rejects loopback TCP binding with `PermissionDenied`,
before any provider behavior is exercised. This task adds a bounded preflight to the Wiremock
LLM targets (`llm_integration_tests`, `llm_provider_integration`), the Wiremock MCP target
(`mcp_provider_tests`), and the HTTP-wrapper target (`task_1937_http_provider_wrappers`, which
also reserves an unused loopback port) so their assertions run on TCP-capable hosts and
unsupported hosts are diagnosed rather than reported as product failures.

## Requirements

1. Prove Wiremock's `127.0.0.1:0` loopback TCP bind capability before starting each test's
   mock server.
2. Keep every existing mock assertion unchanged when the preflight succeeds.
3. Permit an explicit skip only for the observed bind-denied capability failure; setup failures or
   unexpected bind errors must remain test failures with diagnostics.
4. Do not modify provider, HTTP client, LLM behavior, network policy, or test responses.

## TDD steps

1. **RED:** Run the four affected Ash Engine test targets (`llm_integration_tests`,
   `llm_provider_integration`, `mcp_provider_tests`, and
   `task_1937_http_provider_wrappers`) in the managed sandbox; observe the exact loopback bind
   `PermissionDenied` before test setup.
2. Add a test-only capability preflight with a precise `PermissionDenied` classification.
3. **GREEN:** Gate the existing LLM, MCP, and HTTP loopback tests only on that proven capability
   result.
4. Run focused, workspace, formatter, Clippy, docs, and independent-review gates.

## Completion checklist

**Completion evidence:** The test audit confirms that only the exact loopback-bind
`PermissionDenied` capability failure skips; every other bind or setup error remains a test
failure. The affected LLM, MCP, and HTTP loopback controls passed 26 tests (15 + 4 + 4 + 3), and
`de4043d8` changes test fixtures only. No provider, HTTP client, external-network, or production
behavior changed; workspace/fmt/Clippy evidence is retained from the TASK-2031A closeout.

- [x] TCP bind denial is distinguished from product and harness setup errors.
- [x] Every Wiremock assertion still executes unchanged on a capable host.
- [x] No production, provider, semantic, or external-network behavior changes.
- [x] Workspace Rust tests, formatter, Clippy, and docs gate pass; QA/review evidence is recorded.
