# Audit: Final Convergence Closeout

## Scope

This audit closes the spec-to-implementation convergence program by rechecking the original
drift classes against:

- the stabilized spec set in `docs/spec/`
- the frozen handoff/reference contracts in `docs/reference/`
- the completed convergence tasks in `docs/plan/tasks/`
- the current Rust workspace state

Acceptance basis:

- [docs/audit/2026-03-19-spec-001-018-consistency-review.md](2026-03-19-spec-001-018-consistency-review.md)
- [docs/audit/2026-03-19-rust-codebase-review-findings.md](2026-03-19-rust-codebase-review-findings.md)
- [docs/plan/2026-03-19-spec-to-implementation-convergence-plan.md](../plan/2026-03-19-spec-to-implementation-convergence-plan.md)

Review date: 2026-03-20

## Addendum: Phase 34 Follow-up

Phase 34 (`TASK-213` through `TASK-215`) resolved the residual spec-only findings that this audit
originally left open:

- the module/import scope conflict between `SPEC-009` and `SPEC-012`
- the stale typed-provider forward reference and incorrect policy example
- the remaining low-severity example-type hygiene in the affected specs

This audit remains the historical record of the closeout state at the end of `TASK-176`, but the
residual spec-only documentation debt described here is now closed.

## Summary

The Rust/spec convergence implementation program is closed, and the residual spec-only findings
from the older 2026-03-19 consistency audit were later closed by Phase 34.

Closed:

- the original implementation drift classes around workflow forms, parser/lowering contracts,
  runtime verification, `receive`, runtime policy outcomes, REPL/CLI observable behavior, ADT
  contracts, and runtime trace/provenance boundaries
- the residual spec-only findings that remained open at the end of `TASK-176`

Still explicitly open:

- [TASK-212](../plan/tasks/TASK-212-design-control-link-retention-policy.md), which tracks the
  long-term retention/cleanup policy for terminated `ControlLink` supervision state

This audit therefore closes the implementation convergence path and, with the Phase 34 addendum,
also records the later closure of the residual spec-only documentation debt without erasing the
historical end-of-`TASK-176` snapshot.

## Final Acceptance Checklist

The original drift classes were rechecked as the final acceptance checklist:

1. Workflow-form/parser/lowering drift
2. Policy model and lowering drift
3. Stream and `receive` pipeline drift
4. Runtime verification structure drift
5. REPL authority and command-surface drift
6. CLI observable output drift
7. ADT type/value/pattern/helper-surface drift
8. Runtime action/control execution completeness drift
9. Trace/provenance boundary drift
10. Spec cross-reference and terminology drift

## Closure Matrix

| Drift class | Original issue | Governing tasks / contracts | Status |
| --- | --- | --- | --- |
| Workflow forms, parser, lowering | `check`, `decide`, and `receive` drifted across surface syntax, core AST, and lowering | `TASK-156`, `TASK-164`, `TASK-165`, `TASK-167`, [surface-to-parser-contract.md](../reference/surface-to-parser-contract.md), [parser-to-core-lowering-contract.md](../reference/parser-to-core-lowering-contract.md) | Closed |
| Policy model and lowering | Multiple incompatible policy representations and placeholder lowering paths | `TASK-157`, `TASK-166`, `TASK-168`, `TASK-171`, [type-to-runtime-contract.md](../reference/type-to-runtime-contract.md), [runtime-verification-input-contract.md](../reference/runtime-verification-input-contract.md) | Closed for convergence target |
| Streams and `receive` pipeline | `receive` parsed in isolation, lowered incorrectly, and failed end to end | `TASK-158`, `TASK-164`, `TASK-167`, `TASK-168`, `TASK-169`, `TASK-170`, `TASK-209` | Closed |
| Runtime verification structure | Runtime context, obligation enforcement, and verification inputs drifted across type and runtime layers | `TASK-158`, `TASK-169`, `TASK-209`, `TASK-171`, `TASK-206`, [runtime-verification-input-contract.md](../reference/runtime-verification-input-contract.md), [type-to-runtime-contract.md](../reference/type-to-runtime-contract.md) | Closed |
| REPL authority and command surface | Two different REPL implementations and divergent command/history behavior | `TASK-159`, `TASK-172`, `TASK-173`, `TASK-208`, [runtime-observable-behavior-contract.md](../reference/runtime-observable-behavior-contract.md) | Closed |
| CLI observable behavior | `run` / `trace` output categories and presentation drifted from the observable contract | `TASK-182`, `TASK-207`, `TASK-208`, [runtime-observable-behavior-contract.md](../reference/runtime-observable-behavior-contract.md) | Closed |
| ADT type, value, pattern, and stdlib surface | ADT modeling drifted across specs, type checking, runtime values, exhaustiveness, and helper surface | `TASK-160`, `TASK-174`, `TASK-175`, [type-to-runtime-contract.md](../reference/type-to-runtime-contract.md) | Closed for convergence target |
| Runtime action/control execution | Runtime `Act`, spawn/control, and control authority semantics were incomplete or inconsistent | `TASK-205`, `TASK-206`, `TASK-211` | Closed for convergence target |
| Trace and provenance boundaries | Wrapper-level runtime tracing and completion/error framing were not guaranteed | `TASK-207` | Closed |
| Spec cross-reference / terminology drift | Cross-spec naming, reference, and boundary discipline were inconsistent | `TASK-186` through `TASK-198`, `TASK-191` through `TASK-195`, [runtime-reasoner-separation-rules.md](../reference/runtime-reasoner-separation-rules.md), [runtime-to-reasoner-interaction-contract.md](../reference/runtime-to-reasoner-interaction-contract.md) | Closed for current docs corpus |

## Original Spec-Audit Findings Recheck

The 2026-03-19 spec consistency audit contained findings that were broader than the Rust
implementation convergence chain. Their current status is:

### Explicitly Closed

- `receive` syntax drift across stream/capability specs
- `SPEC-018` runtime-context underspecification relative to its own algorithms
- `SPEC-018` internal naming drift for verification errors/warnings
- REPL command/flag drift and REPL type-name drift
- capability declaration example drift in `SPEC-017`
- provenance-event shape mismatch in `SPEC-017`

These are covered by the spec-freezing tasks and the later implementation/observable convergence
work cited in the closure matrix above.

### Explicitly Still Open

None after Phase 34.

### Explicitly Non-Blocking / Out of Current Closeout Scope

- provider effect granularity being coarser than the type/effect model
- example-type normalization such as `string` / `json`
- uneven editorial/status formatting across the spec set

These are lower-severity spec-quality issues. They do not block the implementation convergence
claim and were not the target of the Rust convergence chain.

## Evidence

### Specs and References

The convergence program produced stable reference boundaries that did not exist in the original
audit state:

- [surface-to-parser-contract.md](../reference/surface-to-parser-contract.md)
- [parser-to-core-lowering-contract.md](../reference/parser-to-core-lowering-contract.md)
- [type-to-runtime-contract.md](../reference/type-to-runtime-contract.md)
- [runtime-verification-input-contract.md](../reference/runtime-verification-input-contract.md)
- [runtime-observable-behavior-contract.md](../reference/runtime-observable-behavior-contract.md)
- [runtime-reasoner-separation-rules.md](../reference/runtime-reasoner-separation-rules.md)
- [runtime-to-reasoner-interaction-contract.md](../reference/runtime-to-reasoner-interaction-contract.md)

### Implementation and Test Coverage

The closure claims above are backed by completed implementation tasks and their focused regression
coverage across:

- parser and lowering (`TASK-164` through `TASK-167`)
- type checking and runtime verification (`TASK-168`, `TASK-169`, `TASK-209`)
- runtime execution and policy outcomes (`TASK-170`, `TASK-171`)
- runtime hardening (`TASK-205`, `TASK-206`, `TASK-207`)
- REPL/CLI observable convergence (`TASK-172`, `TASK-173`, `TASK-208`)
- ADT convergence (`TASK-174`, `TASK-175`)

## Full Verification

The repository-level convergence gate was re-run from the current repository root on
2026-03-20:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace
```

Result: pass.

## Explicit Remaining Follow-up

The audit leaves one explicit residual bucket:

- [TASK-212](../plan/tasks/TASK-212-design-control-link-retention-policy.md)

`TASK-212` remains non-blocking because:

- the current runtime behavior is intentional and documented
- the follow-up is explicit rather than implicit
- it does not reopen the original parser/type/runtime/CLI/ADT drift classes

The residual spec-only findings that were open at the end of `TASK-176` were later closed by
Phase 34 and are preserved here only as historical audit context.

## Conclusion

`TASK-176` completes the final audit and closes the main Rust/spec convergence path through:

- `TASK-164` through `TASK-175`
- `TASK-205` through `TASK-209`
- `TASK-211`

The repository now has:

- stabilized specs for the audited drift classes
- explicit reference contracts between layers
- merged Rust implementations for the converged clusters
- a clean repository-wide verification gate

What is now closed:

- the implementation convergence program that began with the original drift reports

What remains open in this neighborhood:

- [TASK-212](../plan/tasks/TASK-212-design-control-link-retention-policy.md)

The repository no longer has residual spec-only findings from the final convergence audit.
