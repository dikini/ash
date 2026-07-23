# Addendum A: Ash Dogfooding, Runtime Evolution, and Unified Workspace Interfaces

**Status:** Accepted product-direction addendum

This addendum clarifies the intended relationship between Ash and the Agent Semantic Workspace.
It supplements the PRD; where the two differ, this addendum controls the implementation and
dogfooding direction described here.

## 1. Product and implementation boundary

The Agent Semantic Workspace is a separate product and repository, implemented primarily in Ash.
It is Ash's principal runtime-dogfooding application: a demanding real workload through which Ash
runtime, standard-library, capability, provenance, policy, and orchestration features are designed
and evaluated.

Ash remains the owner of Ash language meaning and of authoritative native semantic services. The
workspace remains the owner of its product behavior and of generic, multi-language orchestration.
In particular:

- Ash owns the language specification, compiler, runtime, standard library, native analysis,
  diagnostics, canonical Ash semantic profile, and Ash feature-coverage declarations.
- The workspace owns persistent project state, evidence and task ledgers, context compilation,
  provider orchestration, cross-language bridges, agent-facing operations, and trace evaluation.
- Rust is appropriate for Ash host services and ecosystem-facing adapters, including compiler
  services, language-server bridges, persistent indexes, storage drivers, and protocol/process
  boundaries. Ash expresses the workspace's workflows, policies, state transitions, and provider
  orchestration wherever the implemented language supports them.

The workspace must consume Ash semantic facts and authority-aware corpus material rather than
duplicate or reinterpret Ash semantics. Ash-specific integration/profile material may be developed
alongside Ash; generic workbench facilities belong to the separate workspace product.

## 2. Dogfooding-driven Ash evolution

Workspace requirements may reveal missing Ash capabilities. They do not, on their own, define Ash
semantics. A capability becomes an Ash feature only through this explicit promotion loop:

1. A concrete workspace command or integration scenario exposes a missing capability.
2. The need, constraints, alternatives, and expected observable behavior are recorded.
3. Ash receives an independently reviewed specification, tests, and an implementation plan.
4. The compiler, runtime, or standard library implements the feature behind an experimental
   capability where appropriate.
5. The workspace adopts it with a declared fallback and captures real usability, safety,
   correctness, and performance evidence.
6. The feature is stabilized, redesigned, or removed based on that evidence.

The workspace should deliberately exercise durable task state, snapshots, supervision, bounded
concurrency, cancellation, streams, retries, resource limits, capability-scoped external actions,
provenance, policy/admission, typed workflows, and verification obligations. Workspace-specific
conveniences must not be added to the Ash standard library unless they first demonstrate a durable,
general-purpose language or platform abstraction.

## 3. One command model, multiple transports

The CLI, MCP, and any later agent-harness transport expose the same typed command model. No
operation may be available only through a harness, and the CLI must support structured results,
snapshot and continuation handles, task/session identifiers, evidence, completeness, warnings, and
streaming/subscription modes where the operation supports them.

```text
CLI / JSON stdin-stdout ─┐
MCP / agent harness ─────┼──> workspace command core ──> local or daemon host
future transport ────────┘        policy + audit
```

Consequently, a command such as task preparation, context construction, concept tracing,
task-ledger resumption, change planning, controlled application, verification, or event watching
has equivalent CLI and harness invocation. Transport choice changes framing and connection
mechanics, not authority, semantics, evidence, or permission checks.

## 4. CLI and daemon roles

The CLI is a primary product surface, not a reduced administrative shell. It enables direct human
use, automation, deterministic reproduction, debugging, and operation without a running daemon.
An invocation may start an ephemeral local workspace host or connect to a daemon.

The daemon is the durable integration and execution host for MCP and other agent harnesses. It
provides persistent workspace identity and state, warm provider processes and indexes, incremental
refresh, background work, event subscriptions, session continuity, cancellation, concurrency
control, resource limits, policy/admission state, and audit provenance. It does not define a
separate set of workspace capabilities.

The initial driver is a CLI-first, read-oriented vertical slice. Daemon-backed persistence,
long-lived provider supervision, streaming, multi-client operation, controlled mutation, and
verification are deliberate later runtime-dogfooding milestones. Each must preserve equivalent
observable command results when run through the CLI against an ephemeral host, subject only to
explicit freshness and latency differences.

## 5. Repository coordination

Ash and the workspace remain separate repositories. Avoid submodules, a premature coordination
repository, or direct coupling to unstable implementation internals. Coordinate through:

- an experimental, versioned capability contract and feature-coverage declarations;
- first-party Ash profile/integration material, versioned with Ash while it remains Ash-specific;
- a compatibility matrix recording required Ash tooling, supported workspace versions, fallbacks,
  and verification evidence; and
- linked cross-repository epics whose actionable tasks remain in the repository that owns the
  deliverable.

During concurrent development, a dedicated integration fixture may use source checkouts of both
repositories. Releases must use declared capability versions rather than path assumptions.

## 6. Immediate architectural consequences

The earliest workspace implementation should validate the common command core before stabilizing
any public protocol. It should start with CLI-compatible read-oriented operations such as workspace
orientation, task preparation, and Ash concept tracing, while retaining a route to a daemon-backed
MCP service. The initial evaluation must compare the direct CLI and daemon/harness forms for
correctness, evidence recall, authority selection, reproducibility, repeated-context reduction,
and explicit handling of unsupported or partial analysis.
