# TASK-1031 Comonad and Kleisli Audit

## Live syntax findings

Current `std::algebra` source uses ordinary file-backed modules under
`std/src/algebra/`, public interfaces, and carrier-local `pub impl` blocks.
Constructor-kinded interface parameters are accepted, and Phase 135 later
superseded the Phase 133/134 monomorphic payload placeholders with generic
interface method payloads:

```ash
pub interface Monad<M : * -> *> {
    unit(A) -> M<A>
    bind(M<A>, A -> M<B>) -> M<B>
}
```

The live Comonad surface now uses the same generic payload shape:

```ash
pub interface Comonad<W : * -> *> {
    extract(W<A>) -> A
    extend(W<A>, W<A> -> B) -> W<B>
}
```

Kleisli helper functions are not published from `std::algebra`. The earlier
Phase 134 concrete Option/Result wrapper sketch was a temporary first-slice
surface and is now superseded: concrete operations belong to carrier modules,
and a lawful carrier-polymorphic Kleisli helper remains deferred until source
can dispatch selected `Monad<M>` methods honestly.

Cokleisli helpers are not source-implemented in this phase. The interface can be
imported, but there is no lawful first-slice Comonad carrier and no current
source dispatch form for invoking arbitrary selected `Comonad<W>` evidence
methods inside a generic helper.

No accepted Coapplicative source syntax is frozen by this task. TASK-1035 must
decide the name precisely or defer it with the source module absent.

## Interface and impl registration seams

`ash_parser` parses constructor-kinded interfaces and partial constructor impl
heads used by existing algebra files:

- `std/src/algebra/functor.ash` and `std/src/algebra/monad.ash` use
  `F : * -> *` / `M : * -> *`.
- `std/src/result.ash` registers `pub impl <E : *> Monad<Result<_, E>>`.
- `crates/ash-typeck/tests/task_1021_algebra_interface_registration.rs`
  verifies interface registration and method arity from real stdlib files.
- `crates/ash-typeck/tests/task_1022_pure_algebra_instances.rs` verifies
  source evidence registration and selected evidence lookup from parsed stdlib
  modules.

TASK-1032 should add only the `Comonad` interface. It must not add `pub impl
Comonad<...>` rows unless a carrier has total extraction and source-denotable
operations.

## Module loading and stdlib import seams

`std/src/lib.ash` already exports `pub mod algebra;`, and
`std/src/algebra/mod.ash` exports child modules with:

```ash
pub mod monad;
pub use monad::{Monad};
```

Final-surface tests must load real stdlib paths through
`ash_engine::module_loader::load_ordinary_file` or `Engine::check_module_file`,
matching `crates/ash-engine/tests/task_1021_std_algebra_namespace_and_interfaces.rs`.

TASK-1032 must add:

```ash
pub mod comonad;
pub use comonad::{Comonad};
```

TASK-1033 must add:

```ash
pub mod kleisli;
```

only if the helper source checks. TASK-1034 and TASK-1035 must leave
`cokleisli` and `coapplicative` out of `mod.ash` when deferred.

## Evidence selection and helper-function seams

Carrier modules can call public module functions such as `option::and_then`,
`result::and_then`, `option::pure`, and `result::pure` when implementing their
own evidence. `std::algebra` itself should not publish carrier-specific wrapper
functions such as `bind_option`, `unit_option`, `id_option`, or
`compose_option`.

Current `do:K` lowering resolves selected `Monad<K>` evidence in Rust
typechecker code. This phase must not alter that lowering. Generic Kleisli
helpers remain deferred until selected evidence-method dispatch is source-visible.

There is no source syntax in the existing algebra modules for calling arbitrary
interface methods from selected evidence. That blocks a generic Cokleisli
`compose` helper over `Comonad<W>` and a generic Kleisli `compose` over
`Monad<M>`.

## Comonad carrier classification

| Carrier | Decision | Evidence |
|---|---|---|
| `Option` | Reject as `Comonad` | `std/src/option.ash` defines `None`; `extract(None)` would be partial. |
| `Result<_, E>` | Reject as `Comonad` | `std/src/result.ash` defines `Err`; `extract(Err)` would be partial. |
| Ordinary `List` | Reject as `Comonad` | `std/src/list.ash` exposes ordinary lists and partial `head`; empty lists and no focus prevent total extraction. |
| `Act` | Reject as `Comonad` | `std/src/act.ash` declares opaque `Act<A>` with runtime-owned environment; extraction would inspect/run an effectful computation. |
| `Proc` | Reject as `Comonad` | `std/src/proc.ash` exposes runtime-managed process computations; extraction would cross scheduler/runtime boundaries. |
| `Workflow` | Reject as `Comonad` | `std/src/workflow.ash` exposes governed computations; extraction would cross admission/governance/runtime boundaries. |
| `Identity` | Candidate follow-up | No current `std::identity` carrier exists in the live stdlib. |
| `NonEmpty`, `Store`, `Env`, focused zipper | Candidate follow-up | No current lawful source carrier and operations exist in this phase. |

TASK-1032 may implement the interface without instances. Negative tests must
assert required carriers still lack `Comonad` evidence.

## Coapplicative decision inputs

SPEC-079 lists multiple possible meanings for `Coapplicative`; the live Ash
stdlib has no focused lawful carrier and no source syntax requiring or
demonstrating such an interface. Implementing a source module now would be a
placeholder API.

TASK-1035 should choose `Decision: defer`, require `No source module`, and name
the blockers: unsettled Ash-facing laws, no lawful current carrier, and no
final-surface example that exercises the interface without inventing a carrier.

## Downstream verification replacements

### TASK-1032

Use these focused gates:

```bash
test -f std/src/algebra/comonad.ash
python3 -c 'from pathlib import Path; text=Path("std/src/algebra/mod.ash").read_text(); assert "pub mod comonad;" in text and "pub use comonad::{Comonad};" in text'
RUSTC_WRAPPER= cargo test -p ash-engine stdlib_comonad -- --nocapture
RUSTC_WRAPPER= cargo test -p ash-typeck comonad -- --nocapture
git diff --check
```

### TASK-1033

Use these focused gates:

```bash
test -f std/src/algebra/kleisli.ash
python3 -c 'from pathlib import Path; text=Path("std/src/algebra/mod.ash").read_text(); assert "pub mod kleisli;" in text'
RUSTC_WRAPPER= cargo test -p ash-engine stdlib_kleisli -- --nocapture
RUSTC_WRAPPER= cargo test -p ash-cli kleisli -- --nocapture
git diff --check
```

### TASK-1034

TASK-1034 is deferred unless a lawful carrier is added by a later task. Use an
artifact assertion instead of a fake source-module check:

```bash
python3 -c 'from pathlib import Path; audit=Path("docs/plan/audits/TASK-1031-comonad-kleisli-audit.md").read_text(); assert "Cokleisli helpers are not source-implemented" in audit and not Path("std/src/algebra/cokleisli.ash").exists()'
git diff --check
```

### TASK-1035

Use the decision artifact gates already present in TASK-1035, with the expected
Phase 134 decision being deferral and no source module.

### TASK-1036

Use these focused gates:

```bash
python3 -c 'from pathlib import Path; text=Path("docs/plan/tasks/TASK-1029-generated-algebra-law-tests.md").read_text(); required=["Comonad", "Kleisli", "Cokleisli"]; missing=[r for r in required if r not in text]; assert not missing, missing'
python3 -c 'from pathlib import Path; text=Path("reference/stdlib/algebra.md").read_text(); assert "Comonad" in text and "Kleisli" in text and "SPEC-079" in text'
git diff --check
```

### TASK-1037

Closeout must run, at minimum:

```bash
cargo fmt --check
RUSTC_WRAPPER= cargo check --workspace
RUSTC_WRAPPER= cargo test -p ash-engine stdlib_comonad -- --list
RUSTC_WRAPPER= cargo test -p ash-engine stdlib_comonad -- --nocapture
RUSTC_WRAPPER= cargo test -p ash-engine stdlib_kleisli -- --list
RUSTC_WRAPPER= cargo test -p ash-engine stdlib_kleisli -- --nocapture
RUSTC_WRAPPER= cargo test -p ash-typeck comonad -- --list
RUSTC_WRAPPER= cargo test -p ash-typeck comonad -- --nocapture
RUSTC_WRAPPER= cargo test -p ash-cli kleisli -- --list
RUSTC_WRAPPER= cargo test -p ash-cli kleisli -- --nocapture
git diff --check
```

Broad `cargo clippy --workspace --all-targets --all-features -- -D warnings`
and `cargo test --workspace` remain PLAN-129 broad gates. If they are not run,
TASK-1037 must not promote the full phase to complete.
