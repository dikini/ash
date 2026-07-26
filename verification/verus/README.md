# TASK-1991 Verus toolchain spike

This directory is an isolated verifier experiment, not a Cargo package. It establishes that the pinned standalone Verus release shares Ash's one Rust toolchain, accepts one honest proof, and rejects one false postcondition. It does not claim a proof of Ash production behavior.

## Reproducible invocation

```bash
verification/verus/run-fixtures.sh --manifest verification/verus/verus-spike-manifest.json
```

The runner first compares the release's recorded `version.json` toolchain with Ash's required `1.96.0-x86_64-unknown-linux-gnu`. It fails closed before archive download or verifier execution when they differ, verifies the archive SHA-256 before extraction to `${VERUS_SPIKE_CACHE_DIR:-$XDG_CACHE_HOME/ash-verus-spike}`, never runs Cargo, and keeps compiler outputs in a temporary directory. Its machine-readable result goes to stdout; `--output /tmp/verus-fixture-run.json` retains it deliberately.

The pinned rolling release is Verus `0.2026.07.23.64c47f0`, commit `64c47f0043972a17bcb40cc893cfe3901068a15f`, x86-Linux archive SHA-256 `2f4f437e9f89ebcef23b0bce8a8b18319937a0545942e1375553198df7e86134`. Its `version.json` and Ash both require `1.96.0-x86_64-unknown-linux-gnu`; the bundled Z3 is `4.12.5`.

## Boundary and TCB

[The TCB report](tcb-report.json) records all trusted tools and hashes, every logical escape category, unsupported features, adapters, and exact fragment. All six escape categories are explicitly empty. Fixtures run with `--no-cheating`, so `assume`, `admit`, `external_body`, and `assume_specification` cannot silently pass this spike.

No Ash production crate, adapter, or Core-row refinement is in this fragment; those belong to TASK-1992. Normal `cargo check`, `cargo test`, and `cargo clippy` neither discover nor require it.

## Stop/go decision

**GO (narrow):** the shared Rust 1.96.0 toolchain, checksum-checked runner, accepted positive fixture, and rejected negative fixture establish a usable isolated toolchain for TASK-1992.

TASK-1992's separate [Core row-algebra pilot](ROW-ALGEBRA-README.md) reuses this accepted release
without changing this two-fixture spike's manifest, runner, or outcome semantics. Its model-only
proof claim and direct-production correspondence boundary are recorded independently.

**NOT a proof go:** this does not authorize treating the model fixture as a production proof or starting TASK-1993. Stop expansion if a pilot needs broad `external_body` assumptions, lacks a checked production-to-model view, or makes ordinary Cargo depend on this toolchain.
