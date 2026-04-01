# TASK-362 System Supervisor Design

## Goal

Complete the ash-std `system_supervisor` surface so the runtime entry contract has a canonical, documented supervisor workflow signature and exit-code shaping story, while keeping runtime-only spawn/completion mechanics deferred to downstream bootstrap work.

## Scope

This task covers the stdlib-visible contract in `std/src/runtime/supervisor.ash` and the regression coverage/docs that lock that contract in place.

In scope:

- canonical `system_supervisor(args: cap Args) -> Int` workflow surface
- explicit `Result`/`RuntimeError`-based exit-code shaping
- regression tests that pin the stdlib surface and any parser-feasible syntax
- task tracking and changelog updates

Out of scope:

- runtime bootstrap wiring that actually launches the supervisor (`TASK-363c`)
- entry-signature enforcement beyond the existing task boundaries
- new user-facing `await` syntax
- broad new parser/runtime features unrelated to the supervisor surface

## Constraints

1. `Args` remains a capability-typed parameter: `args: cap Args`.
2. The supervisor returns `Int`, representing the process exit code.
3. Completion observation is runtime-internal. The stdlib file should describe/specify this without inventing unsupported user syntax.
4. Exit-code extraction must stay aligned with the canonical `RuntimeError` ADT shape.
5. Changes should be minimal and avoid speculative runtime plumbing.

## Design Decision

Implement TASK-362 as a stdlib contract-completion task, not as bootstrap/runtime execution work.

The supervisor file should move from a bare placeholder to a canonical ash-std definition that:

- imports the canonical runtime/result names it depends on
- preserves the stable supervisor signature
- documents that `main` spawn and terminal-completion observation are runtime-internal semantics
- shapes the terminal `Result<(), RuntimeError>` into an `Int` exit code using the best currently supported Ash surface available to the repo

Because spawn/completion observation is not yet a user-facing surface feature, the supervisor body should avoid inventing new syntax solely for this task. If the exact normative body cannot be represented end-to-end with current surface support, the file should still encode the stable contract and exit-code mapping in the narrowest feasible form, with comments making the runtime boundary explicit.

## Testing Strategy

1. Update stdlib surface tests so they no longer accept the old `ret 0;` placeholder.
2. Add focused regression coverage for the canonical supervisor surface that is actually supported today:
   - imports present
   - signature present
   - no `await`
   - exit-code shaping references canonical `RuntimeError`/`Result` surface
3. If the chosen body is parser-feasible, add a parsing regression for the concrete syntax.
4. Update task/docs/changelog alongside the code change.

## Acceptance Mapping

- “Supervisor workflow in stdlib” → `std/src/runtime/supervisor.ash` contains the canonical contract.
- “Spawns main / observes completion” → documented as runtime-internal supervisor semantics in the stdlib surface and task notes, without adding unsupported syntax.
- “Returns Int exit code” → stable workflow signature and exit-code shaping logic/comments remain explicit.
- “Nested variant destructuring / RuntimeError shape” → regression coverage and/or source text pins the canonical result/error structure.

## Risks and Mitigations

- **Risk:** Current parser/runtime surface cannot express the full normative supervisor algorithm.
  - **Mitigation:** Keep this task tightly scoped to the stdlib contract, document the runtime boundary clearly, and avoid speculative syntax additions.
- **Risk:** Tests could accidentally keep accepting the old placeholder.
  - **Mitigation:** Replace placeholder-oriented assertions with canonical-surface assertions.
- **Risk:** Scope drift into bootstrap/runtime implementation.
  - **Mitigation:** Leave execution wiring to `TASK-363c` and related downstream runtime tasks.
