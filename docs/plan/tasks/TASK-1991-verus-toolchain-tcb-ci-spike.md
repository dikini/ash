# TASK-1991: Verus Toolchain, TCB, and CI Isolation Spike

**Status:** Planned
**Phase:** [PLAN-202](../PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)
**Depends on:** TASK-1985 proof-artifact schema

## Description

Establish a pinned, isolated Verus verification path without coupling normal Ash builds to the
Verus Rust toolchain.

## Requirements

- Document compatible Verus/Rust/Z3 versions and reproducible commands.
- Define verified crate/module layout and production/spec view boundary.
- Enumerate the Verus verifier/VC generator, cargo/build wrapper, rustc/codegen, Z3, `vstd`, pinned
  artifacts, `assume`, axioms, external bodies/specifications/items/trait impls, unsupported
  features, and production adapters, separating logical assumptions from trusted tooling.
- Integrate artifact fingerprints and outcomes with semantic traceability.
- Provide stop/go evidence before either proof pilot expands.

## TDD Steps

1. Add a minimal intentionally failing and passing Verus fixture.
2. Pin and isolate the toolchain/CI job.
3. Add TCB/assumption report generation and negative fixtures.
4. Run ordinary Ash gates to prove toolchain non-interference.

## Completion Checklist

- [ ] Verus verification is reproducible and isolated.
- [ ] TCB and assumptions are machine-readable.
- [ ] Ordinary Cargo workflows do not require Verus.
- [ ] A documented go/no-go decision authorizes TASK-1992.
