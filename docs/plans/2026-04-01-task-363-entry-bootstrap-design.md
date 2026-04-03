# TASK-364 / TASK-363b / TASK-363c Entry Bootstrap Design

## Goal

Implement the validated entry-workflow path for Ash programs: parse an entry workflow with its declared return type, verify the canonical `main` signature, preload the runtime stdlib required by that contract, and bootstrap execution so the runtime can derive the correct process exit result from the entry outcome.

## Scope

This design covers the three requested tasks plus the minimum prerequisite support needed to make them real:

- `TASK-364`: canonical entry signature verification
- `TASK-363b`: runtime-facing entry workflow verification
- `TASK-363c`: bootstrap orchestration and exit-code shaping
- the minimal stdlib-loading support from `TASK-363a` required to implement `TASK-363b` and `TASK-363c` honestly
- parser/engine metadata changes needed because the current workflow surface does not preserve declared workflow return types

In scope:

- adding optional declared workflow return-type support to parsed workflow definitions
- exposing enough engine metadata to inspect an entry workflow definition after parse
- loading runtime stdlib source into the engine before entry verification/execution
- verifying the `main` contract against the exact `SPEC-022` rule
- bootstrapping entry execution and mapping outcomes to the required exit result classes
- focused regression tests, task bookkeeping, and changelog updates

Out of scope:

- full CLI rewiring beyond what the bootstrap API needs for downstream tasks
- speculative new runtime syntax such as surface `await`
- broad parser or module-system redesign beyond what entry loading requires
- downstream CLI polish tasks (`TASK-365`, `TASK-366`, `TASK-367`, `TASK-368a`)

## Constraints

1. The canonical entry contract remains exactly the `SPEC-022` rule: workflow name `main`, declared return type `Result<(), RuntimeError>`, zero or more parameters, and every parameter must be a usage-site capability type `cap X`.
2. Success/failure classification must remain aligned with `SPEC-021`:
   - `0` for successful `main`
   - `N` for `RuntimeError { exit_code: N, ... }`
   - pre-entry/bootstrap/verification failures remain bootstrap failures
3. The implementation must use the existing `Engine` abstraction, not invent a new runtime type.
4. The runtime supervisor remains a stdlib-visible contract, but bootstrap should not require unsupported user-visible spawn/await syntax.
5. Changes should stay localized and prefer explicit helper functions over broad refactors.

## Design Decision

Implement this work as an engine-centered entry bootstrap path with explicit signature verification and minimal stdlib preloading.

### 1. Preserve declared workflow return types in the parser surface

The current `ash_parser::surface::WorkflowDef` stores parameters and body but not a declared workflow return type. That makes `TASK-364` impossible to implement faithfully. The parser will therefore be extended so workflow definitions may carry an optional return type, parsed from the existing `-> T` syntax.

This is intentionally narrow:

- `WorkflowDef` gains `return_type: Option<Type>`
- `parse_workflow::workflow_def()` parses the optional workflow return type
- lowering remains unchanged except for plumbing through the richer surface metadata

This gives the engine something real to validate rather than forcing string inspection.

### 2. Add engine helpers for entry-aware parsing and stdlib loading

The engine already caches parsed surface workflow definitions, which is enough to build this feature without redesigning the full execution pipeline. The implementation should add small, explicit helpers that:

- parse an entry source/file while preserving the surface workflow definition
- load the runtime stdlib sources needed by entry contracts into the engine context
- expose the parsed `WorkflowDef` for verification

Because downstream tasks depend on `TASK-363a`, the minimum real stdlib-loading slice must land here instead of being faked. That support should stay narrow and focused on runtime entry modules.

### 3. Implement `TASK-364` as the canonical signature checker

`TASK-364` should be a pure verifier over parsed surface metadata:

- workflow identifier must be `main`
- declared return type must be exactly `Result<(), RuntimeError>`
- all parameters must be `Type::Capability(_)`

The verifier should report stable failure classes for:

- missing `main`
- wrong declared return type
- non-capability parameter

This logic belongs in the engine-facing runtime entry path rather than in the general type checker, because the rule applies only to the designated entry workflow.

### 4. Implement `TASK-363b` as runtime entry verification

Runtime entry verification should compose:

- stdlib-preloaded engine creation/loading
- entry module parse/check
- `TASK-364` signature validation

This layer is the runtime-facing boundary that downstream bootstrap can call. It should not duplicate the signature logic; it should wrap and surface it.

### 5. Implement `TASK-363c` as bootstrap orchestration, not new syntax

Bootstrap should be implemented as a Rust helper that:

1. creates or configures an `Engine`
2. loads the required stdlib modules
3. parses and checks the entry file
4. verifies the `main` signature
5. executes the entry workflow through the existing engine/interpreter path
6. converts the result into the required exit classification

Given the current runtime surface, the simplest correct implementation is to derive the observable exit result from the executed entry workflow outcome directly, while keeping the stdlib supervisor contract as the normative downstream shape for the full runtime story. This avoids inventing unsupported spawn/completion syntax while still satisfying the observable contract required by `SPEC-021`.

## Testing Strategy

1. Parser tests for workflow return-type parsing:
   - `workflow main() -> Result<(), RuntimeError> { ... }` preserves the declared return type
2. Entry-signature verification tests:
   - valid zero-parameter `main`
   - valid capability-parameter `main`
   - wrong return type rejected
   - non-capability parameter rejected
   - missing `main` rejected
3. Bootstrap tests:
   - success path yields `0`
   - `RuntimeError` path yields the carried exit code
   - missing `main` fails in pre-entry verification
4. Focused engine/runtime regression coverage only; avoid widening the task to unrelated CLI semantics.

## Acceptance Mapping

- “Verify entry workflow type signature” → parser preserves declared return types and the runtime verifier enforces the exact `SPEC-022` shape.
- “Runtime entry workflow verification” → the engine exposes an entry-verification path that validates `main` after stdlib loading and type checking.
- “Bootstrap and supervisor execution” → the runtime exposes a bootstrap helper that loads stdlib, validates entry, executes the entry workflow, and derives the correct exit result classification.
- “Use Engine, not fictional APIs” → all work is built on `ash_engine::Engine` and helper methods inside the engine crate.

## Risks and Mitigations

- **Risk:** workflow return-type parsing may touch more parser plumbing than expected.
  - **Mitigation:** keep the AST change minimal and cover it with focused parser tests first.
- **Risk:** exact stdlib-loading support may tempt a broader module/import redesign.
  - **Mitigation:** load only the minimum runtime stdlib sources needed for entry verification/bootstrap.
- **Risk:** bootstrap could overreach into downstream CLI behavior.
  - **Mitigation:** expose a focused runtime helper and keep CLI wiring for later tasks.
- **Risk:** exit-code derivation may accidentally depend on unsupported supervisor mechanics.
  - **Mitigation:** derive the observable result from the executed entry workflow outcome directly, while preserving the stdlib supervisor contract for future runtime integration.
