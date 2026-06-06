# TASK-1020: Stdlib Algebra Audit Gate

## Status

Complete for the audit gate only. No `std::algebra` modules, interfaces, instances, do-target rewiring, law-test runner support, or reference migration behavior was implemented here.

## Scope

This audit freezes the live seams, deferral-retirement decisions, syntax handoff, and focused non-zero verification commands for TASK-1021 through TASK-1028. It deliberately stops before TASK-1021 implementation.

## RED/GREEN Evidence

- RED: `python3` audit-artifact guard from TASK-1020 failed before this artifact existed with `AssertionError: docs/plan/audits/TASK-1020-stdlib-algebra-audit-gate.md`.
- GREEN: the final TASK-1020 verification in `docs/plan/tasks/TASK-1020-stdlib-algebra-audit-gate.md` records exit 0 for the audit-artifact guard, `RUSTC_WRAPPER= cargo test -p ash-typeck do_target`, `cargo fmt --check`, and `git diff --check`.

## Live Seams

### Standard library layout

- `std/src/lib.ash` currently exports root-level `option`, `result`, runtime/test/LLM/IO helpers, `act` helper functions, and `pub mod proc`, `pub mod workflow`, and `pub mod ooda`.
- `std/src/lib.ash` does not yet contain `pub mod algebra;`.
- `std/src/algebra/` does not yet exist. TASK-1021 owns creating `mod.ash`, `semigroup.ash`, `monoid.ash`, `functor.ash`, `applicative.ash`, and `monad.ash`.
- `std/src/act.ash` already exposes `unit`, `bind`, `then`, and `guard` over opaque `Act<A>`, with hidden runtime `ActEnv` not source-denotable.
- `std/src/proc.ash` already exposes public builtin `unit`, `from_act`, `bind`, `then`, and process helpers over `Proc<A>`.
- `std/src/workflow.ash` already exposes public builtin `unit`, `bind`, `then`, `from_proc`, and `from_act` over `Workflow<A>`.

### Parser and lowering

- `crates/ash-parser/src/parse_module.rs` parses `pub mod algebra;` and file-backed child modules through the existing module declaration path.
- `parse_interface_definition` accepts `[pub] interface Name<...> { ... }`.
- Interface methods are positional signatures only: `method(Type, Type) -> Type`. Named method parameters such as `append(a: A, b: A) -> A` are not accepted.
- Interface method signatures are not semicolon-terminated by the live parser. Use newlines or commas between methods, not trailing `;`.
- Interface method-level generic binders such as `map<A, B>(...)` are not parsed.
- `parse_optional_interface_type_params` accepts constructor-kinded interface parameters such as `M : * -> *`.
- `crates/ash-parser/src/lower.rs` still rejects non-`*` interface and impl params in `lower_interface_def` and `lower_impl_def` with stale TASK-907 wording. TASK-1021 must either avoid that legacy lowering path for final-surface imports or patch the lowerer consistently with the current `ash-typeck` support.

### Typechecker and evidence

- `crates/ash-typeck/src/type_env.rs::register_interface` registers constructor-kinded interface params and stores `type_param_kinds`.
- `TypeEnv::lower_interface_evidence_args` can lower constructor evidence arguments for non-`*` interface parameters, including constructor heads and partial constructor applications.
- `TypeEnv::register_impl` still rejects constructor-kinded impl type parameters, but it accepts constructor evidence in impl heads such as `impl Monad<Option>`.
- Interface method bodies in impls are checked against positional method parameter names only: `method(x, y) = expr`.
- Existing selected evidence tests in `crates/ash-typeck/tests/task_909_monad_do_target_resolution.rs` use local fixture syntax:

```ash
interface Monad<M : * -> *> {
    return(Int) -> M<Int>
    bind(M<Int>, Int -> M<Int>) -> M<Int>
}
impl Monad<Option> {
    return(value) = Some { value: value }
    bind(value, f) = value
}
```

That fixture proves the constructor-kinded evidence seam, but it is not final-surface stdlib evidence for TASK-1021+.

### Do-target and TCIR

- `crates/ash-typeck/src/do_target.rs::resolve_do_target` currently preserves an anonymous hidden bridge for `Act`: `HiddenActReturn` and `HiddenActBind`.
- `Proc` and `Workflow` currently resolve to ordinary public operations: `proc::unit`/`proc::bind` and `workflow::unit`/`workflow::bind`.
- Non-tower unary targets route through `resolve_interface_evidence("Monad", ...)`.
- The selected evidence path currently asks for `return` and `bind`, not `unit` and `bind`.
- Result partial-constructor evidence exists as a special case for `Result<_, E>` and has intrinsic shims for `return -> Ok` and `bind -> result::and_then`.
- `crates/ash-typeck/src/check_expr.rs` records selected `Monad` evidence in TCIR and fails closed when selected evidence is missing a required method body or intrinsic shim.

### Module loader

- `crates/ash-engine/src/module_loader.rs` resolves imports from the importing directory, `ASH_DEP_ROOTS`, `ASH_DEPENDENCY_ROOTS`, `ASH_LIBRARY_PATH`, the built-in stdlib root, and locked project roots.
- File-backed `pub mod name;` resolves through `name.ash` or `name/mod.ash`.
- The built-in stdlib root is `../../std/src` unless overridden by `ASH_STDLIB_ROOT`.
- TASK-1021 final-surface tests must exercise the real stdlib path, not only parser-local fixtures.

## Exact Syntax Decisions

### Namespace and imports

- Canonical namespace: `std::algebra` source files under `std/src/algebra/`.
- Canonical source import path in user/stdlib examples: `use algebra::monad::{Monad};` and sibling module imports such as `use algebra::functor::{Functor};`.
- TASK-1021 adds `pub mod algebra;` to `std/src/lib.ash`. Root-level re-exports remain out of scope for TASK-1021.

### Interface surface

Use the current accepted positional interface syntax for the first committed source files. Do not copy SPEC-078 logical pseudocode verbatim when it contains named parameters, semicolons, or method-level generics.

Accepted first-slice shape examples:

```ash
pub interface Semigroup<A> {
    append(A, A) -> A
}

pub interface Monoid<A> {
    empty() -> A
    append(A, A) -> A
}

pub interface Monad<M : * -> *> {
    unit(Int) -> M<Int>
    bind(M<Int>, Int -> M<Int>) -> M<Int>
}
```

The `Monad` example above is intentionally a monomorphic live-syntax shape, not the final polymorphic semantic target. Its `unit` method still takes a raw payload (`Int`) and returns the carrier (`M<Int>`), matching the existing public `act::unit`, `proc::unit`, and `workflow::unit` operation shape. TASK-1021 must record any minimal typechecker/parser work needed to make the final source-visible interface honest without adding new syntax.

### Canonical Monad method naming

- Canonical public method name: `unit`.
- Rationale: `std/src/act.ash`, `std/src/proc.ash`, and `std/src/workflow.ash` already expose public `unit` operations; `return` is block syntax in `do:K` bodies and should not become the ordinary public method name unless unavoidable.
- Required downstream patch: TASK-1024 must switch selected Monad evidence lookup from `return` to `unit` and update Result/tower shims accordingly. Until that task, existing `do_target` tests may still describe the old bridge and fixture method name.

## Deferral Retirement

| Prior deferral | Current audit decision | Owner |
|---|---|---|
| Public stdlib `Monad<M>` deferred | Retire. Current parser/typechecker substrate is enough to start final-surface work, but TASK-1021 must fix or route around stale parser-lowering rejection for constructor-kinded interface params. | TASK-1021 |
| Hidden tower dictionaries | Retire or quarantine. `Act` still has hidden bridge leakage; `Proc` and `Workflow` already point to public operation names but need named evidence tied to `std::algebra`. | TASK-1023/TASK-1024 |
| Pure `Option`/`Result`/`List` dictionaries deferred | Retire where source syntax can express bodies. If a body cannot be expressed honestly, fallback must be named, stdlib-tied, and paired with a replacement follow-up. | TASK-1022 |
| Law proof/test derivation | Keep deferred only as a concrete generated-test follow-up packet with owner and acceptance rows. | TASK-1026 |
| Fully self-hosted tower runtime representation | Keep deferred explicitly. `ActEnv`, process identity, scheduler state, workflow admission/kernel internals remain runtime-owned and non-denotable. | TASK-1023/TASK-1028 |

## Deferral Retirement Follow-Ups

- TASK-1021: prove stdlib algebra modules parse/check/import through the real stdlib path; patch stale constructor-kinded lowering only as needed for importable interfaces.
- TASK-1022: add pure carrier evidence or named fallback rows tied to importable stdlib symbols.
- TASK-1023: eliminate hidden bridge leakage for `Act` or quarantine it as named compiler-prelude evidence tied to `act::unit`/`act::bind`.
- TASK-1024: make `do:K` and comprehensions select `unit`/`bind` stdlib/prelude evidence for pure and tower carriers.
- TASK-1026: create the generated law-test follow-up owner; do not leave law profiles as prose-only deferral.
- TASK-1027: run the stale deferral sweep across normative/reference docs and mark old language historical or superseded.
- TASK-1028: close only after TASK-1021 through TASK-1027 evidence, hidden bridge leakage coverage, and stale deferral sweep all pass.

## Focused Commands

All filtered cargo commands below use `-- --list` plus a non-zero count guard before running `-- --nocapture`. This avoids zero-test green traps.

### TASK-1021

```bash
python3 - <<'PY'
from pathlib import Path
for rel in ['std/src/algebra/mod.ash','std/src/algebra/semigroup.ash','std/src/algebra/monoid.ash','std/src/algebra/functor.ash','std/src/algebra/applicative.ash','std/src/algebra/monad.ash']:
    assert Path(rel).is_file(), rel
lib=Path('std/src/lib.ash').read_text()
assert any(line.strip() == 'pub mod algebra;' for line in lib.splitlines()), 'non-comment pub mod algebra;'
mod=Path('std/src/algebra/mod.ash').read_text()
for module in ['semigroup','monoid','functor','applicative','monad']:
    assert f'pub mod {module};' in mod, module
for rel, iface in [
    ('std/src/algebra/semigroup.ash','Semigroup'),
    ('std/src/algebra/monoid.ash','Monoid'),
    ('std/src/algebra/functor.ash','Functor'),
    ('std/src/algebra/applicative.ash','Applicative'),
    ('std/src/algebra/monad.ash','Monad'),
]:
    assert f'interface {iface}' in Path(rel).read_text(), rel
print('std::algebra files and lib export exist')
PY
bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-engine algebra_interface -- --list | tee /tmp/task1021-ash-engine-algebra-interface.list; matches=$(grep -E "(^|::)algebra_interface[^[:space:]]*: test$" /tmp/task1021-ash-engine-algebra-interface.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-engine algebra_interface tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-engine algebra_interface -- --nocapture'
bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-typeck algebra_interface -- --list | tee /tmp/task1021-ash-typeck-algebra-interface.list; matches=$(grep -E "(^|::)algebra_interface[^[:space:]]*: test$" /tmp/task1021-ash-typeck-algebra-interface.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-typeck algebra_interface tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-typeck algebra_interface -- --nocapture'
cargo fmt --check
git diff --check
```

### TASK-1022

```bash
bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-typeck pure_algebra_instances -- --list | tee /tmp/task1022-ash-typeck-pure-algebra-instances.list; matches=$(grep -E "(^|::)pure_algebra_instances[^[:space:]]*: test$" /tmp/task1022-ash-typeck-pure-algebra-instances.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-typeck pure_algebra_instances tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-typeck pure_algebra_instances -- --nocapture'
bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-engine pure_algebra_instances -- --list | tee /tmp/task1022-ash-engine-pure-algebra-instances.list; matches=$(grep -E "(^|::)pure_algebra_instances[^[:space:]]*: test$" /tmp/task1022-ash-engine-pure-algebra-instances.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-engine pure_algebra_instances tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-engine pure_algebra_instances -- --nocapture'
cargo fmt --check
git diff --check
```

### TASK-1023

```bash
bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-typeck task1023_tower_algebra_instances -- --list | tee /tmp/task1023-tower-algebra-instances.list; matches=$(grep -E "(^|::)task1023_tower_algebra_instances[^[:space:]]*: test$" /tmp/task1023-tower-algebra-instances.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-typeck task1023_tower_algebra_instances tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-typeck task1023_tower_algebra_instances -- --nocapture'
bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-typeck task1023_hidden_bridge_leakage -- --list | tee /tmp/task1023-hidden-bridge-leakage.list; matches=$(grep -E "(^|::)task1023_hidden_bridge_leakage[^[:space:]]*: test$" /tmp/task1023-hidden-bridge-leakage.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-typeck task1023_hidden_bridge_leakage tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-typeck task1023_hidden_bridge_leakage -- --nocapture'
bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-interp task1023_act_tower_runtime -- --list | tee /tmp/task1023-ash-interp-act.list; matches=$(grep -E "(^|::)task1023_act_tower_runtime[^[:space:]]*: test$" /tmp/task1023-ash-interp-act.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-interp task1023_act_tower_runtime tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-interp task1023_act_tower_runtime -- --nocapture'
bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-interp task1023_proc_tower_runtime -- --list | tee /tmp/task1023-ash-interp-proc.list; matches=$(grep -E "(^|::)task1023_proc_tower_runtime[^[:space:]]*: test$" /tmp/task1023-ash-interp-proc.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-interp task1023_proc_tower_runtime tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-interp task1023_proc_tower_runtime -- --nocapture'
bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-interp task1023_workflow_tower_runtime -- --list | tee /tmp/task1023-ash-interp-workflow.list; matches=$(grep -E "(^|::)task1023_workflow_tower_runtime[^[:space:]]*: test$" /tmp/task1023-ash-interp-workflow.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-interp task1023_workflow_tower_runtime tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-interp task1023_workflow_tower_runtime -- --nocapture'
cargo fmt --check
git diff --check
```

### TASK-1024

```bash
bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-typeck stdlib_do_evidence -- --list | tee /tmp/task1024-stdlib-do-evidence.list; matches=$(grep -E "(^|::)stdlib_do_evidence[^[:space:]]*: test$" /tmp/task1024-stdlib-do-evidence.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-typeck stdlib_do_evidence tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-typeck stdlib_do_evidence -- --nocapture'
bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-typeck stdlib_comprehension_evidence -- --list | tee /tmp/task1024-stdlib-comprehension-evidence.list; matches=$(grep -E "(^|::)stdlib_comprehension_evidence[^[:space:]]*: test$" /tmp/task1024-stdlib-comprehension-evidence.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-typeck stdlib_comprehension_evidence tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-typeck stdlib_comprehension_evidence -- --nocapture'
bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-engine stdlib_do_evidence -- --list | tee /tmp/task1024-engine-stdlib-do-evidence.list; matches=$(grep -E "(^|::)stdlib_do_evidence[^[:space:]]*: test$" /tmp/task1024-engine-stdlib-do-evidence.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-engine stdlib_do_evidence tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-engine stdlib_do_evidence -- --nocapture'
cargo fmt --check
git diff --check
```

### TASK-1025

```bash
bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-engine algebra_combinators -- --list | tee /tmp/task1025-algebra-combinators.list; matches=$(grep -E "(^|::)algebra_combinators[^[:space:]]*: test$" /tmp/task1025-algebra-combinators.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-engine algebra_combinators tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-engine algebra_combinators -- --nocapture'
bash -lc 'set -euo pipefail; RUSTC_WRAPPER= cargo test -p ash-cli algebra_examples -- --list | tee /tmp/task1025-algebra-examples.list; matches=$(grep -E "(^|::)algebra_examples[^[:space:]]*: test$" /tmp/task1025-algebra-examples.list | wc -l); test "$matches" -gt 0; printf "non-zero ash-cli algebra_examples tests: %s\n" "$matches"; RUSTC_WRAPPER= cargo test -p ash-cli algebra_examples -- --nocapture'
cargo fmt --check
git diff --check
```

### TASK-1026

Artifact assertion:

```bash
python3 - <<'PY'
from pathlib import Path
import re
p=Path('docs/plan/audits/TASK-1026-algebra-law-test-handoff.md')
assert p.is_file(), p
text=p.read_text()
for s in ['Semigroup','Monoid','Functor','Applicative','Monad','generated test','SPEC-077','follow-up task','acceptance rows','owner','pure instances','tower carriers']:
    assert s in text, s
assert Path('docs/plan/tasks/TASK-1029-generated-algebra-law-tests.md').is_file() or (
    'Generated Algebra Law Tests' in Path('docs/plan/PLAN-INDEX.md').read_text()
    and re.search(r'owner|acceptance rows', Path('docs/plan/PLAN-INDEX.md').read_text(), re.I)
)
print('law-test handoff artifact and concrete follow-up owner exist')
PY
cargo fmt --check
git diff --check
```

### TASK-1027

Artifact assertion for the stale deferral sweep:

```bash
python3 - <<'PY'
from pathlib import Path
page=Path('reference/stdlib/algebra.md')
assert page.is_file(), page
page_text=page.read_text(errors='ignore')
for s in ['std::algebra','Semigroup','Monoid','Functor','Applicative','Monad','instances','examples','do:','comprehension']:
    assert s in page_text, s
paths=[Path('reference'),Path('docs/spec'),Path('docs/plan')]
terms=['stdlib Monad deferred','future Monad evidence only','hidden Act dictionary','Monad dictionaries deferred','pure List/Option/Result dictionaries remain deferred','Option/Result/List dictionaries deferred','stdlib Monad unavailable','bridge dictionaries','hidden dictionaries','Generalized runtime lowering through arbitrary user-defined Monad']
for root in paths:
    for p in root.rglob('*.md'):
        text=p.read_text(errors='ignore')
        for t in terms:
            if t in text and 'historical' not in text.lower() and 'superseded' not in text.lower():
                raise SystemExit(f'stale unqualified wording: {p}: {t}')
print('scoped stale wording check passed')
PY
cargo fmt --check
git diff --check
```

### TASK-1028

```bash
python3 - <<'PY'
from pathlib import Path
p=Path('docs/plan/audits/TASK-1028-stdlib-algebra-closeout-evidence.md')
assert p.is_file(), p
text=p.read_text()
for task in ['TASK-1021','TASK-1022','TASK-1023','TASK-1024','TASK-1025','TASK-1026','TASK-1027']:
    assert task in text, task
for s in ['command','package','filter','test_count','test_count > 0','artifact assertion','hidden bridge leakage','stale deferral sweep']:
    assert s in text, s
print('closeout focused evidence artifact records guarded focused gates')
PY
cargo fmt --check
RUSTC_WRAPPER= cargo check --workspace
RUSTC_WRAPPER= cargo test -p ash-typeck --all-targets
RUSTC_WRAPPER= cargo test -p ash-engine --all-targets
RUSTC_WRAPPER= cargo test -p ash-cli --all-targets
RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTC_WRAPPER= cargo test --workspace
RUSTC_WRAPPER= cargo doc --workspace --no-deps
git diff --check
```

## Non-Zero Guard Rule

Downstream tasks must not claim focused verification from a filtered `cargo test` command until the matching `-- --list` command has produced a non-zero count. Artifact-only gates must assert exact files and required terms. A command that runs zero filtered tests is a blocker, not green evidence.

## Hidden Bridge Leakage Gate

TASK-1023 and TASK-1024 must prove that `Act`, `Proc`, and `Workflow` sequencing authority is public `std::algebra`/compiler-prelude evidence tied to public operations. The current `Act` hidden bridge is live leakage and must not remain anonymous independent authority.

## Stale Deferral Sweep

TASK-1027 owns the stale deferral sweep across current normative/reference docs. Historical wording can remain only when clearly marked historical or superseded. Current docs must not continue to teach that stdlib Monad, pure carrier evidence, or generalized user Monad evidence are merely future-only when SPEC-078 has retired that deferral for this phase.

## Deferred Items

- No TASK-1021+ behavior was implemented by this audit.
- Generic algebra method syntax remains a live implementation risk. TASK-1021 must prove the selected final source surface through real stdlib import/check tests and record any minimal non-syntax typechecker/parser routing needed.
- `Act` hidden bridge leakage remains present until TASK-1023/TASK-1024.
