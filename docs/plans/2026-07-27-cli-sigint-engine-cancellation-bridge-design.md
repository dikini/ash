# CLI SIGINT Delivery Capability Gate Design

**Goal:** Keep the bounded admitted `time::sleep` cancellation evidence trustworthy by proving
that the test host can deliver a programmatic SIGINT to Tokio before it evaluates the existing
CLI-to-Engine cancellation contract.

**Architecture:** Keep production CLI behavior, admission, run control, Engine cancellation
precedence, and terminal projection unchanged. The managed sandbox's standalone Tokio probe never
received a programmatic SIGINT after a pre-exec disposition reset, so an eager CLI listener cannot
repair the failure. A Linux-only ignored probe in the integration-test binary now self-spawns,
waits until its Tokio listener is ready, receives one signal, and caches that host capability. The
existing full Ash cancellation assertions run unchanged when the probe succeeds.

**Tech Stack:** Rust 2024, Tokio signal support, existing ash-cli integration tests, Cargo.

## Alternatives considered

1. Change Engine cancellation handling. Rejected: Engine watch-control and cancellation-priority
   tests already pass; the standalone Tokio probe shows the signal never reaches any Tokio listener.
2. Relax SIGINT tests to accept normal completion. Rejected: this removes the required canonical
   external/execution/cancelled terminal evidence on capable hosts.
3. Add `tokio-graceful-shutdown`. Rejected: its `catch_signals()` is built on `tokio::signal`, so
   it retains the failed boundary while adding a second shutdown lifecycle beside the Engine.
4. Test-only, self-contained Tokio capability probe. Selected: it distinguishes an unsupported
   host from an Ash regression without changing the CLI, Engine, or terminal contract.

## TDD plan

1. Preserve the existing process-level SIGINT tests as RED, including listener-readiness evidence.
2. Reproduce the same failure with a standalone Tokio child after resetting SIGINT to its default
   disposition; this identifies the host boundary rather than a CLI or Engine regression.
3. Add a cached test-only capability probe that runs the original Ash assertions only on a
   signal-capable host.
4. Run focused terminal, CLI, Engine cancellation, workspace, formatter, Clippy, and docs gates.
