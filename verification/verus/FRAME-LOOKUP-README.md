# TASK-1993 frame-ordered operation-dispatch pilot

This standalone checksum-pinned Verus artifact proves only a finite
`Seq<ModelFrame>` selection model. It does not build through Cargo or prove production Ash Rust.

## Reproduction

```bash
verification/verus/run-frame-lookup.sh \
  --manifest verification/verus/frame-lookup-manifest.json
```

The runner checks archive and source fingerprints, rejects logical escape hatches, runs both
artifacts in a temporary directory outside the checkout with
`--output-json --no-cheating --rlimit 120`, and emits checked JSON. Verifier output is not written
into the repository.

## Proven claim and boundary

The repaired model proves absence iff no match, selected-index bounds/source match, greatest
matching (innermost) selection, payload provenance, matching-frame shadowing, nonmatching-frame
preservation, and handler/provider-kind-agnostic order. The checked run reports eight proof items.

This is not a direct theorem about `HandlerChain::find_operation_frame` or its use by `eval_raise`:
there is no verified adapter from `HandlerFrame`, `EffectOp`, `Name`, or `HandlerClause` to the
integer model atoms. Provider execution, handler-body evaluation, resume behavior, and all Rust
representation/error correspondence remain outside the proof. Focused Rust tests are executable
evidence for the reverse scan, not a refinement proof.

## Checked repair benchmark provenance

`benchmarks/frame_lookup_broken.rs` deliberately claims that a nonmatching innermost frame shadows
an earlier match. The pinned Verus run rejects it (exit 1, one postcondition error).
`frame_lookup.rs::append_nonmatching_frame_preserves` is the checker-validated repair, verified as
part of the eight-item artifact (exit 0).

No individual author, LLM provider/model/prompt, or authoring tool is recorded for either file.
This is therefore a deliberate broken-to-repaired checker benchmark, **not** evidence that an LLM
generated or repaired a lemma. The only attributed tool is Verus `0.2026.07.23.64c47f0`.
