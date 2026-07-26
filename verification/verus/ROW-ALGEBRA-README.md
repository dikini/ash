# TASK-1992 Core row-algebra pilot

This is a standalone, pinned Verus proof artifact. It does not build through Cargo and it does not
prove Ash production code. It verifies the closed finite-sequence model in `row_algebra.rs` using
the same Rust `1.96.0-x86_64-unknown-linux-gnu` toolchain as Ash and the checksum-pinned Verus
release accepted by TASK-1991.

## Reproduction

```bash
verification/verus/run-row-algebra.sh \
  --manifest verification/verus/row-algebra-manifest.json
```

The runner downloads only the pinned release into `VERUS_ROW_ALGEBRA_CACHE_DIR` (or an external
user cache), verifies the archive and source fingerprints, rejects logical escape hatches, runs
`--output-json --no-cheating --rlimit 120`, and emits a checked JSON outcome. It writes no
verifier output into the checkout.

## Proven claim and boundary

The model uses `Seq<int>` as an injective encoding of an already-expanded exact `CoreRowItem`, with
no tail. It proves membership preservation, duplicate elimination, idempotence, non-increasing
length, stable first-occurrence order, closed inclusion reflexivity/transitivity, normalization
invariance, membership-permutation invariance, group-reference rejection, and row
non-authority. The exact source/run evidence is recorded in `row-algebra-report.json`.

This is not a direct proof of `ash_core::core_ash_typecheck::normalize_core_row` or
`core_row_included_in`: there is no verified Rust-to-model injective encoding or adapter. Open
tails, structural type equivalence, and diagnostic-payload ordering are also out of scope. The
trace graph therefore records a proved model artifact while retaining the explicit direct-CoreRow
bridge gap for TASK-439/follow-up conformance work.

## Representation-preserving maintenance evidence

The executable pilot refactor added generated closed rows across operation, role, policy, and
contract namespaces plus a first-occurrence mutation sentinel in
`crates/ash-core/tests/task_1642_core_row_normalization.rs`. The refactor preserves the production
representation/API; it strengthens only test construction and makes incorrect path-only or
last-occurrence deduplication observable. The model counterpart is the proved
`inclusion_membership_permutation_invariant` theorem. The focused suite reports 18 tests, and the
Verus run reports 15 verified proof items. No production implementation file changed for this
pilot.
