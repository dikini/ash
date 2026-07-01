# Phase 176 Deferred Cleanup Readiness Audit

Date: 2026-07-01

## Scope

Readiness audit for the deferred Phase 176 cleanup candidates:

- `Value::List` runtime-variant removal / canonical `Cons`/`Nil` list representation.
- Module-level function visibility inside closures.
- QuickCheck recursive combinator status and implementation shape.
- Stale phase-status reconciliation after the above decisions.

## Summary disposition

| Candidate | Readiness | Decision | Follow-up gate |
|---|---:|---|---|
| `Value::List` runtime variant | Ready for ordered migration | Remove the enum variant; use semantic helpers for pattern/authority sites; keep only temporary constructor-position compatibility until tests/fixtures are migrated. | No pattern-position `Value::List`; workspace check; list/JSON/runtime tests. |
| Module-level function visibility in closures | Substrate present but lookup gap remains | Implement explicit module/global callable visibility for closure calls instead of heuristic capture. | Positive imported/module-level function call inside closure plus negative private/effect leakage tests. |
| QuickCheck recursive combinators | Self-reference shape blocked; depth-threaded helper shape viable | Re-scope recursive combinators to bounded/depth-threaded helpers, not self-referential `Strategy` values. | Final-surface QuickCheck fixtures and no accidental general recursive-value feature. |
| Phase status drift | Ready after code decisions | Reconcile PLAN/task/status surfaces after the implementation tasks settle. | Plan/task/index/changelog consistency check. |

## `Value::List` classification snapshot

Current migration state after TASK-1797 completion:

- `Value::List` enum variant removed from `ash_core::Value`.
- Canonical helpers exist on `Value`:
  - `list_nil()`
  - `list_cons(head, tail)`
  - `list_from_vec(values)`
  - `list_to_vec()`
  - `is_list()`
- `cargo check --workspace --all-targets` passes.
- Pattern-position `Value::List(...)` references have been migrated out of live Rust source.
- Constructor-position compatibility references have been migrated to `Value::list_from_vec(...)` / `Value::list_nil()`.
- The temporary compatibility constructor has been removed.
- Repository assertion result: no `Value::List` references remain in Rust source under `crates/`.

## Verification evidence collected

Commands run during this audit/migration slice:

```text
cargo check -p ash-core --all-targets
cargo check -p ash-interp --all-targets
cargo check -p ash-engine -p ash-cli --all-targets
cargo check --workspace --all-targets
cargo test -p ash-core -p ash-interp -p ash-engine -p ash-cli --all-targets
cargo fmt --check
git diff --check
cargo test -p ash-engine providers::llm --lib
cargo test -p ash-cli value_convert --lib
python3 - <<'PY'
from pathlib import Path
bad=[]
for p in Path("crates").rglob("*.rs"):
    if "Value::List" in p.read_text(errors="ignore"):
        bad.append(str(p))
assert not bad, bad
PY
```

Observed results:

- `cargo check --workspace --all-targets`: passed.
- `cargo test -p ash-core -p ash-interp -p ash-engine -p ash-cli --all-targets`: passed; output was truncated by the tool but exit code was 0.
- `cargo fmt --check`: passed after formatting.
- `git diff --check`: passed.
- `cargo test -p ash-engine providers::llm --lib`: 119 passed, 0 failed.
- `cargo test -p ash-cli value_convert --lib`: 9 passed, 0 failed.
- Repository assertion for absence of `Value::List` in Rust source: passed.

## Notes for TASK-1798

The closure visibility audit found that `Value::Closure` captures a lexical `EnvFrame`, `Expr::Call` resolves closures through `ctx.get(func)`, and module-level exported callables are not automatically visible inside closure call contexts unless they were captured or explicitly threaded. The implementation should add an explicit callable/module visibility path and test that private/module-absent names do not leak.

## Notes for TASK-1800

Do not implement QuickCheck recursive combinators with self-referential `Strategy` values. Use a bounded helper such as `recursive_at(base, expand, config, depth)` and have `recursive_with` thread/decrease depth based on configured or size-derived limits.
