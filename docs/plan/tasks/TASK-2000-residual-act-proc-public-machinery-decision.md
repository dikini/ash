# TASK-2000: Residual Act/Proc Public Machinery Decision

**Status:** Complete — the public `Act<T>`/`Proc<T>` wrapper decision is deletion, with source,
type, manifest, builtin, diagnostic, and direct-source-`invoke` absence evidence. Residual generic
parser/lowering and internal-runtime machinery are explicitly owned follow-up work, not public
wrapper compatibility.
**Phase:** Follow-up from [TASK-1988](TASK-1988-semantic-implementation-deprecation-audit.md)

## Description

Decide the semantic disposition of reachable public `Act<T>`/`Proc<T>` builtin wrappers and tower
machinery under the function-first canonical vocabulary.

## Requirements

- Inventory every public constructor, builtin, diagnostic, lowering branch, and test consumer.
- Choose retain-private, fold-into-target, or delete; a rename-only result is invalid.
- Before deletion, prove target-function/effect-row behavior parity or absence of reachable use.
- Preserve Phase 201 removed-form guarantees and update canonical trace ownership.

## TDD Steps

1. Add failing reachability/negative fixtures for the chosen disposition.
2. Add behavior parity cases for any target replacement.
3. Implement only after the decision record is accepted.
4. Run affected Rust, docs, traceability, and removal gates.

## Completion Checklist

- [x] Public machinery inventory and decision are recorded.
- [x] Behavior-level parity or absence evidence is passing.
- [x] No public tower mechanism remains semantically unowned.
- [x] Changelog, task/index, and canonical trace edges are updated.

## Evidence required

TASK-1988 records the residual wrapper conflict; this task must cite exact public symbols and
demonstrate that any removal preserves or intentionally removes observable behavior.

## Inventory gate and reachability evidence

`crates/ash-cli/tests/task_2000_tower_machinery_inventory.rs` reads the versioned inventory at
`crates/ash-cli/tests/fixtures/task_2000_tower_machinery_inventory.json`. It discovers every Rust
source or test file under `crates/` mentioning `Act` or `Proc` and fails if that exact set is not
classified. The current inventory covers 50 files, including diagnostic/lowering paths,
runtime-internal carriers, positive test consumers, and Phase 201 negative-regression coverage.
Every listed source anchor must still exist.

The completed public admission and interpreter bridge slices removed `Act<T>`/`Proc<T>` TypeEnv,
manifest, source-entry, dispatch-table, and direct-evaluator exposure. Remaining references are
classified rather than implied to be public: they include target/do diagnostics, parser/lowering
paths, and internal runtime carriers. The focused inventory, formatting, test, and Clippy gates
pass.

## Accepted disposition and first public-admission deletion slice

The inventory established that `Act<T>` and `Proc<T>` were public and executable rather than stale
names alone. The accepted disposition is now **delete the public admission surface**, rather than
retain it privately or rename it. This is an intentional source-language removal: no
function/effect-row behavior-parity claim is made for a removed `Act`/`Proc` program.

The first completed deletion slice removes public TypeEnv/manifest admission for `Act` and `Proc`
and the public bridge-builtin admission for `act::unit`, `proc::unit`, and `proc::yield`.
`crates/ash-engine/tests/task_2000_tower_public_surface_rejection.rs` proves those names are absent
from the public computation manifest and type environment, and rejects source uses while retaining
canonical ambient `do` as the control case. This is source-entry rejection evidence, not a claim
that every historical runtime implementation has been deleted.

The first slice deliberately preserves the distinction between the removed public wrapper spelling
and runtime-internal concepts. In particular, `EffectType::Act` remains an effect classification
and `ActEnv` remains a hidden interpreter context carrier; neither is a public source type named
`Act<T>`, and neither is removed or renamed by this task. Likewise, process/closure machinery that
historically supported `Proc` remains runtime work until a separately scoped deletion or
replacement slice establishes its ownership and safety.

## Completed interpreter bridge deletion slice

The second completed slice deletes all fourteen source-callable interpreter bridges:
`act::{unit, bind, __guard, policy_check}` and
`proc::{unit, from_act, bind, then, await, yield, par, scatter, join, gather}`. They are absent
from the builtin dispatch table, sync and async direct evaluator fast paths, and the
`eval_function_call` fallback; the now-dead `runtime_proc_*` wrapper constructors and
bridge-specific tests were removed. `task_2000_tower_runtime_bridge_rejection.rs` proves metadata
absence and fail-closed sync/async calls while retaining an ordinary canonical builtin control.

This deletion does **not** remove runtime capabilities. `invoke` continues to construct and force
its hidden `ActEnv`-backed provider capture through its unqualified internal path; `EffectType::Act`
continues to classify effects; and low-level process handles, scheduler admission, and internal
`Proc*Capture` values remain available to their runtime owners. Unit coverage now constructs those
internal captures directly, so retaining scheduler behavior is not evidence for a restored
source-callable `proc::*` API.

## Completed stale diagnostic and prelude cleanup slice

The completed typechecker cleanup removes only stale wrapper-specific guidance and synthetic
evidence. Diagnostic traversal now delegates to ordinary expression checking rather than
fabricating `Act<T>` or `Proc<T>` for rejected `act::unit`/`proc::unit` calls. Missing-comprehension
guidance now requests an explicit process target, and cross-constructor sequencing diagnostics use
the generic explicit-lift advice rather than recommending the removed `proc::from_act` bridge. The
compiler prelude no longer synthesizes `Monad` evidence for the removed wrapper carriers. Focused
unit evidence also keeps canonical ambient `do` as a valid control.

This is not a full deletion claim. The unqualified public `invoke` typing path still returns the
legacy-shaped carrier and requires a separately scoped canonical row-bearing or rejection decision.
Purity checking and the hidden runtime `ActEnv` provider capture remain explicitly owned runtime
work, as do process captures, scheduler admission, and remaining parser/lowering references. They
must not be deleted or renamed merely because stale diagnostics and prelude synthesis are gone.
Any future source-level replacement needs its own behavior evidence for return values, declared
runtime traps, row/admission failures, and observable process behavior.

## Accepted direct-source `invoke` rejection slice

The remaining public typechecker special case for unqualified
`invoke(provider: String, action: String, args: List<Value>)` is an unowned legacy path because
it returns the removed `Act<Value>` carrier. Its accepted disposition is **source-level
rejection**, not a compatibility retyping as a target row-bearing operation.

The target replacement remains a future, separately scoped implementation of admitted named
interface or binding operations. Such an implementation must resolve a stable
impl/type-qualified operation identity, result signature, requirement-row item, discharge route,
Core `Raise` lowering, and observable behavior. Dynamic provider/action strings cannot establish
those target facts, so this task makes no row-typing claim.

The focused typechecker regression
`task_2000_direct_source_invoke_is_rejected_without_tower_type_leakage` proves that a correctly
shaped direct source call fails closed with guidance toward an admitted named interface or binding
operation and does not expose `Act` or `Proc`. The runtime's hidden `ActEnv`/provider-capture path
remains internal implementation machinery and is preserved by existing interpreter dispatch
coverage; this source-admission decision neither removes nor promotes that runtime path.

## Completion reconciliation

TASK-2000 is complete as the decision and removal task for **public tower wrappers**. The
50-reference detector inventory is exhaustive for Rust `Act`/`Proc` and qualified `act::`/`proc::`
mentions, and each reference is classified. Public absence is independently covered by the
TypeEnv/manifest/source rejection test, the fourteen-bridge runtime-dispatch rejection test,
diagnostic/prelude cleanup tests, and direct-source `invoke` rejection; canonical ambient `do` and
an ordinary builtin remain controls. No public `Act<T>`/`Proc<T>` constructor, bridge builtin, or
legacy `invoke` carrier typing path remains admitted.

The completion scope does not delete generic parser spelling, historical diagnostic wording, or
runtime-owned implementation details merely because they retain the tokens `Act` or `Proc`.
Those references have explicit non-wrapper owners: target source/lowering sidecars and generic
`do` behavior are owned by TASK-2002; target named interface/binding operation and row realization
remain future target-effect work; and hidden `ActEnv`, provider capture, process captures,
scheduler admission, and `EffectType::Act` remain runtime-internal machinery with their existing
runtime tests. They are not source types, public manifest entries, source-callable bridges, or
compatibility aliases. Any future change to those paths requires its own task and behavior evidence.
