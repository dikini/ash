# NOTE-007: Context/RoleContext Send+Sync refactor, clone-cost semantics, and optimisation opportunities

Date: 2026-04-23
Status: Active implementation note for TASK-689D follow-on runtime work

## Why this note exists

To unblock wider async `Act` interpreter integration, `ash-interp` must make runtime context futures `Send`-compatible under the current workflow executor boundary.

The immediate refactor replaces `RefCell<HashSet<Name>>` obligation stores with thread-safe storage while preserving today's clone semantics.

That preservation choice is intentionally conservative but has a cost: cloning `Context` and `RoleContext` now deep-copies obligation/discharge sets instead of sharing them.

This note exists in `docs/notes/` so the performance tradeoff and future optimisation opportunities remain discoverable outside the task file.

## What changed conceptually

Current preserved semantics:
- `Context::clone()` copies local obligation state by value.
- `RoleContext::clone()` copies discharged-obligation state by value.
- `Context::extend()` still creates a fresh local obligation set for the child frame.

This avoids semantic drift during the Send/Sync refactor, but it means clone cost scales with the number of tracked obligations/discharges.

## Why the deep copy was chosen now

The refactor goal is executor compatibility first, not performance redesign.

Using shared `Arc<Mutex<HashSet<_>>>` state with derived `Clone` would silently change semantics:
- clones would alias the same obligation/discharge set,
- discharge/reset in one clone would mutate the other.

That would be a behavior change, not just a storage change.

So the initial Send/Sync-safe path keeps behavior stable first.

## Where the cost can show up

Potential observable slowdown depends on runtime shape and volume:
- repeated `Context::clone()` on hot async/effectful paths,
- frequent `RoleContext::clone()` when many obligations are tracked,
- large obligation sets during high-volume `Act`/workflow orchestration,
- recursive or highly nested evaluation where context cloning compounds.

This is not copy-on-write today.
It is eager deep copy on clone, with lock-based mutation afterward.

## Future optimisation opportunities

### 1. Explicit copy-on-write obligation state

Replace deep-copy clone semantics with a representation that is logically by-value but physically shared until mutation.

Possible designs:
- `Arc<HashSet<Name>>` + `Arc::make_mut` where mutation sites own `&mut self`
- `Arc<BTreeSet<Name>>` or persistent immutable set structures
- custom small persistent set wrapper if obligation counts remain low/medium

Constraint:
- current APIs like `discharge(&self)` and `add_obligation(&self)` rely on interior mutability,
  so a true COW design likely requires API reshaping or an internal transactional layer.

### 2. Split immutable and mutable context state

Separate runtime context into:
- immutable/shared bindings + hidden runtime handles,
- mutable linear obligation state.

Then only the mutable obligation component needs COW or locking, while ordinary context cloning can stay cheap.

This likely gives the cleanest long-term performance story if async interpreter integration expands.

### 3. Local-frame obligation sharing with explicit fork points

Keep `extend()` creating a fresh local obligation set, but avoid deep-copying obligation state for ordinary structural clones that do not need isolation.

This would require making clone/fork semantics explicit in the API, for example:
- `clone_shared()`
- `clone_isolated()`
- or removing blanket `Clone` in favor of named operations.

### 4. Small-set optimized storage

If obligation counts are usually tiny, use a small-set representation before falling back to heap-backed sets.
Examples:
- `smallvec`-backed sorted small set
- `arrayvec`-style bounded local set for common fast path

This reduces clone cost without changing semantics.

### 5. Executor-lane split instead of universal Send/Sync pressure

If Send/Sync refactoring becomes too invasive, an alternative is a non-`Send` local execution lane for async Act forcing.

That would reduce pressure to make every runtime context component thread-safe, but it increases executor complexity and branching.

## Recommendation ladder

Preferred order for future optimisation work:
1. Measure clone frequency / obligation-set sizes on realistic workloads.
2. If needed, split immutable context from mutable obligation state.
3. Then evaluate COW or persistent-set representation for the mutable obligation portion.
4. Only introduce executor-lane splits if the Send/Sync-safe context becomes too costly or too semantically awkward.

## Current takeaway

The Send/Sync refactor should be treated as a semantic-preserving compatibility step.

Performance is not assumed to be free:
- clone cost is real,
- not copy-on-write,
- and should be revisited once the async interpreter path is integrated more broadly.
