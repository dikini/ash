# TASK-1991: Verus Toolchain, TCB, and CI Isolation Spike

**Status:** Complete
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

- [x] Verus verification is reproducible and isolated: `verification/verus/run-fixtures.sh`
  fetches only the pinned x86-Linux release into an external cache, validates its SHA-256, uses the
  shared Rust `1.96.0-x86_64-unknown-linux-gnu` toolchain, and emits checked JSON for both fixtures.
- [x] TCB and assumptions are machine-readable in `verification/verus/tcb-report.json`; every
  required logical escape category is explicitly present and empty.
- [x] Ordinary Cargo workflows do not require Verus: the runner and
  `.github/workflows/verus-spike.yml` invoke no Cargo command, and the workspace has no Verus
  package configuration.
- [x] The narrow go decision in `verification/verus/README.md` authorizes TASK-1992 toolchain use,
  but not a production proof claim or TASK-1993.

## Evidence

- `verus-0.2026.07.23.64c47f0-x86-linux.zip` from the rolling release, commit
  `64c47f0043972a17bcb40cc893cfe3901068a15f`, SHA-256
  `2f4f437e9f89ebcef23b0bce8a8b18319937a0545942e1375553198df7e86134`.
- The real pinned run reports `pass.rs`: exit `0`, `1 verified`, `0 errors`; and `fail.rs`: exit
  `1`, `0 verified`, `1 errors`. It leaves no `pass`/`fail` compiler output in the checkout.
- `python3 tools/docs/validate_verus_spike.py --root . --manifest verification/verus/verus-spike-manifest.json --format json`
  reports no errors.
