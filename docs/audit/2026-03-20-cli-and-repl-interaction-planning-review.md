# CLI and REPL Interaction Planning Review

Date: 2026-03-20
Task: TASK-202

## Scope

Reviewed the user-facing CLI and REPL surfaces against:

- [Runtime Observable Behavior Contract](/home/dikini/Projects/ash/docs/reference/runtime-observable-behavior-contract.md)
- [Surface Guidance Boundary](/home/dikini/Projects/ash/docs/reference/surface-guidance-boundary.md)
- [Runtime-to-Reasoner Interaction Contract](/home/dikini/Projects/ash/docs/reference/runtime-to-reasoner-interaction-contract.md)
- [SPEC-005: CLI](/home/dikini/Projects/ash/docs/spec/SPEC-005-CLI.md)
- [SPEC-011: REPL](/home/dikini/Projects/ash/docs/spec/SPEC-011-REPL.md)
- [SPEC-021: Runtime Observable Behavior](/home/dikini/Projects/ash/docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)

Review protocol:

- keep runtime-observable behavior separate from explanatory-only stage guidance
- ask whether each surface still makes sense without any reasoner present
- report only; do not change normative specs

## Summary

The reviewed CLI and REPL surfaces are mostly `runtime-observable` and remain meaningful without a
reasoner present. The command entry points, output formatting, error visibility, and inspection
commands are all user-facing runtime behavior, not runtime-to-reasoner projection.

The remaining pressure is presentation-level: some wording and placeholder output still reflect
implementation drift relative to the frozen observable contract. That drift should be handled as
runtime-observable convergence or surface guidance, not as new syntax or interaction-layer design.

## Findings

### 1. `ash run` is runtime-observable and aligned

- [crates/ash-cli/src/commands/run.rs](/home/dikini/Projects/ash/crates/ash-cli/src/commands/run.rs)

Classification: `runtime-observable`

Status: `Aligned`

Reasoning: `run` parses input, builds the engine, executes the workflow, and prints either the
result or a trace summary. It is a direct user-facing runtime boundary and still makes complete
sense without any reasoner present.

### 2. `ash trace` is runtime-observable and aligned

- [crates/ash-cli/src/commands/trace.rs](/home/dikini/Projects/ash/crates/ash-cli/src/commands/trace.rs)

Classification: `runtime-observable`

Status: `Aligned`

Reasoning: `trace` executes the workflow, records provenance, and exports trace data in a user-
selectable format. This is runtime visibility and audit output, not reasoner projection.

### 3. REPL command handling is runtime-observable and aligned

- [crates/ash-repl/src/lib.rs](/home/dikini/Projects/ash/crates/ash-repl/src/lib.rs)

Classification: `runtime-observable`

Status: `Aligned`

Reasoning: `Repl::run`, `handle_command`, `print_help`, and the multiline prompt behavior define
the interactive user-facing command surface. They are valid runtime behavior even when no reasoner
participates.

### 4. `:ast` and `:type` are runtime-observable inspection surfaces with presentation drift

- [crates/ash-repl/src/lib.rs](/home/dikini/Projects/ash/crates/ash-repl/src/lib.rs)

Classification: `runtime-observable`

Status: `Silent`

Reasoning: `:ast` prints parsed structure and `:type` currently prints a placeholder
`"Type: (inferred from context)"`. Both are user-visible runtime outputs, but `:type` still needs
convergence with the canonical observable contract. This is an observable-behavior task, not a
reasoner-boundary task.

### 5. Help text and banner wording are presentation-level guidance, not semantic authority

- [crates/ash-repl/src/lib.rs](/home/dikini/Projects/ash/crates/ash-repl/src/lib.rs)

Classification: `runtime-observable`

Status: `Silent`

Reasoning: The REPL banner, command help, and command descriptions explain the surface to humans.
That wording is still runtime-visible and useful, but any future advisory/gated/committed stage
explanation should remain explanatory only, as defined by the surface-guidance boundary.

### 6. Trace summary text is runtime-observable presentation, not projection

- [crates/ash-cli/src/commands/run.rs](/home/dikini/Projects/ash/crates/ash-cli/src/commands/run.rs)
- [crates/ash-cli/src/commands/trace.rs](/home/dikini/Projects/ash/crates/ash-cli/src/commands/trace.rs)

Classification: `runtime-observable`

Status: `Aligned`

Reasoning: Messages such as trace counts, output write confirmations, and integrity acknowledgements
are runtime-visible presentation details. They should stay aligned with the observable contract, but
they do not become a reasoner context channel.

## Cross-Cutting Observation

The audit does not introduce any blocking dependency for tooling/surface planning. It confirms that
the CLI and REPL remain runtime-facing surfaces, while stage guidance remains an explanatory layer
that later `SPEC-002` text can describe without changing command semantics.

The only concrete follow-up pressure is to reconcile `:type` and any nearby wording with the frozen
observable-behavior contract so the REPL output matches the canonical user-visible story.

## Conclusion

No blocking contradiction was found in the CLI and REPL surfaces.
The surfaces remain runtime-observable, the explanatory stage-guidance boundary remains separate,
and the remaining work is presentation-level convergence plus wording cleanup for later tooling
planning.
