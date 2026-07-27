# Lexical-Scope Admission Contract Design

**Goal:** Reconcile two stale lexical-scope CLI assertions with the bounded checked Core-to-CPS
admission domain, without broadening source lowering or execution authority.

**Architecture:** The implementation is test-contract-only. The existing bounded `PureAnf` route
already owns the accepted lexical forms; the legacy fixtures must assert the current fail-closed
bridge-domain diagnostic for their unsupported shapes. No parser, Core, CPS, Engine, or CLI code
changes.

**Tech Stack:** Rust integration tests, Cargo test/Clippy, Common Changelog documentation.

## Alternatives considered

1. Restore the atomic-let-only diagnostic in production. Rejected: it contradicts the documented
   bounded `PureAnf` admission slice and would regress current checked Core-to-CPS behavior.
2. Assert only a nonzero exit. Rejected: that loses evidence that rejection occurs at the intended
   checked admission boundary.
3. Update the fixtures to assert the canonical generic bridge-domain rejection. Selected: it
   preserves fail-closed behavior and pins the actual bounded ownership boundary.

## TDD plan

1. Add a focused test assertion for the generic bridge-domain message in the two existing
   lexical-scope controls; run it and observe RED against the stale atomic-let expectation.
2. Replace the obsolete shared expectation with the canonical bridge-domain expectation only.
3. Run the focused integration target, workspace tests, formatter, Clippy, and docs gate.
4. Obtain independent QA and code review; update the task, plan index, and changelog only after
   the evidence is recorded.
