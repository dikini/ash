# TASK-1670: Verify thunk capture authority

**Status:** Planned
**Phase:** [PLAN-163](../PLAN-163-CORE-LAZY-MEMO-MODES.md)
**Owner:** Phase 163

## Description

Add lowering/runtime tests proving thunks use the handler/provider chain captured at construction time.

## Specification Reference

- [SPEC-101 §9](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#9-capture-and-authority)
- [SPEC-101 §11](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#11-core-to-cps-lowering)

## Dependencies

- [TASK-1669](TASK-1669-core-mode-lowering.md)

## Existing Code Touchpoints

- `crates/ash-core/src/core_ash_lower.rs`: assert lowered thunks use empty/default capture
  placeholders.
- `crates/ash-interp/src/cps/mod.rs`: verify `eval_value` fills `captured_env` and
  `captured_chain`, and `ForceThunk` restores them while preserving the force-site continuation.
- `crates/ash-core/src/cps.rs`: use `HandlerChain`, `HandlerFrame::Shallow`,
  `HandlerFrame::Provider`, and `Value::ThunkClosure` in structural assertions.

## Requirements

1. Constructing a thunk captures the active handler/provider chain.
2. Forcing outside the construction handler still dispatches thunk-body raises through the captured chain.
3. Construction does not grant authority; force still requires the latent row statically.
4. Tests cover both lazy and memo thunks.
5. Capture happens in `ash-interp` runtime value construction (`eval_value` with current
   `HandlerChain`), not in `ash-core` lowering, which only emits empty/default capture placeholders.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1670_core_thunk_capture_authority.rs`.
2. Add at least one `.core` fixture `force_captured_handler.core`.
3. Run focused tests and confirm force-time dispatch is wrong or unsupported.
4. Implement capture/restore behavior in lowering/runtime.
5. Re-run `task_1670`, `task_1664`, and `task_1669`.

## Completion Checklist

- [ ] Lazy thunk force uses construction-time chain.
- [ ] Memo thunk force uses construction-time chain.
- [ ] Force-site continuation still receives the result.
- [ ] Lowering tests confirm emitted thunk placeholders are filled by runtime construction.
