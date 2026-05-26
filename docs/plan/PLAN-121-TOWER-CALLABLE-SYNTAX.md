# PLAN-121: Tower Callable Type and Closure Syntax

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. This phase is a syntax/typing migration packet; do not implement Act/Proc/Workflow callable runtime semantics unless a later task explicitly expands scope.

**Goal:** Replace `Fn(args...) -> ret` and pure `|args| => body` as the preferred callable syntax with tower-aligned callable arrows and closure arrows.

**Architecture:** SPEC-072 defines a two-axis callable model: the arrow classifies application stratum, while the return type classifies the produced value. The first implementation slice accepts preferred pure callable type syntax and pure closure syntax, keeps legacy `Fn(...) -> ...` compatibility, and reserves Act/Proc/Workflow callable and closure arrows with fail-closed diagnostics.

**Tech Stack:** Rust parser/typechecker/engine documentation updates, Markdown specs/plans/tasks/reference pages, existing Ash test harnesses, existing reference validator.

---

## 1. Status

**Status:** 🚧 In Progress — packet, audit, pure callable type, and pure closure parser slices complete
**Spec:** [SPEC-072](../spec/SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md)
**Task range:** [TASK-955](tasks/TASK-955-tower-callable-syntax-packet.md) through [TASK-963](tasks/TASK-963-stdlib-and-reference-callable-syntax-migration.md)

TASK-955 creates the spec/plan/task packet. TASK-956 through TASK-963 are implementation, migration, and closeout tasks.

## 2. Scope

### In scope

- Preferred pure callable type syntax: `(A, B) -> C`.
- Legacy compatibility for `Fn(A, B) -> C` during migration.
- Explicit distinction between n-ary callable domains and tuple argument types.
- Preferred pure closure syntax: `|args| -> body`.
- Reservation diagnostics for Act callable syntax `-*>` in type and closure positions.
- Reservation diagnostics for Proc callable syntax `=>` in type and closure positions.
- Reservation diagnostics for Workflow callable syntax `=*>` in type and closure positions.
- Rendering/reference/documentation updates that stop teaching stale `Fn(...) -> ...` and pure `|x| => ...` as preferred syntax.
- Standard-library `.ash` migration from legacy callable syntax to preferred `(A, B) -> C` and pure `|args| -> body` where parser/typechecker support has landed.
- Top-level `reference/` corpus migration to the new callable type and pure closure syntax, with legacy examples retained only when explicitly labeled as compatibility guidance.

### Out of scope

- Implementing Act callable values or Act closure application.
- Implementing Proc callable values or Proc closure application.
- Implementing Workflow callable values or Workflow closure application.
- Partial application/currying.
- Closure serialization or cross-process closure transport.
- Replacing `do:Act`, `do:Proc`, or `do:Workflow`.
- Replacing match-arm `=>` syntax unless a later parser task proves ambiguity.

## 3. Task table

| Task | Description | Est. Hours | Status |
| --- | --- | ---: | --- |
| [TASK-955](tasks/TASK-955-tower-callable-syntax-packet.md) | Create SPEC-072/PLAN-121/task packet and register Phase 126 | 4 | ✅ Complete |
| [TASK-956](tasks/TASK-956-callable-syntax-audit-gate.md) | Audit live parser/typechecker/renderer/module-summary closure seams before implementation | 6 | ✅ Complete |
| [TASK-957](tasks/TASK-957-pure-callable-type-parser.md) | Parse preferred pure callable type syntax and legacy compatibility forms | 10 | ✅ Complete |
| [TASK-958](tasks/TASK-958-callable-type-typeck-rendering.md) | Typecheck and render pure callable types with preferred syntax across imports/diagnostics | 10 | ✅ Complete |
| [TASK-959](tasks/TASK-959-pure-closure-arrow-syntax.md) | Implement pure closure `|args| -> body` and migrate old pure fat-arrow handling | 12 | ✅ Complete |
| [TASK-960](tasks/TASK-960-reserved-tower-callable-arrows.md) | Reserve `-*>`, `=>`, and `=*>` in callable type and closure contexts with diagnostics | 8 | 📝 Planned |
| [TASK-961](tasks/TASK-961-callable-syntax-reference-docs.md) | Update reference chapter, agent card, and amended legacy specs for the new syntax | 8 | 📝 Planned |
| [TASK-963](tasks/TASK-963-stdlib-and-reference-callable-syntax-migration.md) | Migrate `std/` and current `reference/` examples to preferred callable syntax | 8 | 📝 Planned |
| [TASK-962](tasks/TASK-962-tower-callable-syntax-closeout.md) | Close out SPEC-072 with acceptance matrix, broad gates, and independent review remediation | 8 | 📝 Planned |

## 4. Decision gates

- **D1:** The callable arrow classifies application stratum; the return type does not.
- **D2:** `(A, B) -> C` means a two-argument pure callable, not a unary tuple-argument callable.
- **D3:** A pure smart constructor may return `Act<A>`, `Proc<A>`, or `Workflow<A>` without becoming an Act/Proc/Workflow callable.
- **D4:** `-*>`, `=>`, and `=*>` are reserved now even if their callable semantics are not implemented now.
- **D5:** `|args| -> body` is the pure closure shorthand. `|args| => body` must stop meaning pure closure after the migration window.
- **D6:** Legacy `Fn(args...) -> ret` remains a compatibility spelling during this phase but reference/rendering should prefer the new syntax.
- **D6a:** `std/` and the top-level `reference/` corpus must be migrated before closeout so executable library surfaces and daily-use examples do not keep legacy syntax as the default.
- **D7:** Implementation must fail closed rather than silently lowering higher-stratum callables to pure callables returning computation values.
- **D8:** Parser tasks remain syntax/span/source-fidelity tasks; semantic carriers shared across crates belong in `ash-core` if needed.

## 5. Tracks

- **Track A — Packet and audit:** TASK-955/TASK-956 freeze the decision and live-code seams.
- **Track B — Pure callable types:** TASK-957/TASK-958 implement parsing, typechecking, rendering, and import/export preservation for preferred pure callable type syntax.
- **Track C — Closures and reserved arrows:** TASK-959/TASK-960 implement pure closure arrow syntax and reserve higher-stratum arrows with diagnostics.
- **Track D — Docs, corpus migration, and closeout:** TASK-961/TASK-963/TASK-962 reconcile references, migrate stdlib and current reference examples, update changelog, record acceptance evidence, and remediate independent review findings.

## 6. Implementation feasibility gates

TASK-956 produced [the callable syntax audit gate](audits/TASK-956-callable-syntax-audit-gate.md) before Rust implementation starts. It explicitly covers both parser paths (`parse_module.rs` and `parse_type_def.rs`), the tuple-vs-callable-domain hazard, `Type::Fn`/`Type::Fun` rendering and application checking, module signature transport, stdlib syntax exposure, top-level `reference/` example exposure, and the live closure parser/lowering seam for old `|args| => body` handling.

The audit must patch TASK-957 through TASK-960 and TASK-963 with exact focused non-zero verification commands. Downstream implementation and migration tasks must not proceed from the placeholder `false` verification guards.

Two known hazards are phase gates:

1. `(A, B) -> C` must not be implemented by parsing `(A, B)` as a tuple and wrapping it as a unary `Fn` argument.
2. Current partial-application behavior, if still present through `instantiate_fn_call` or a successor helper, must be reconciled with SPEC-072's exact-arity rule before closeout.

## 7. Verification strategy

Docs-only packet verification:

```bash
git diff --check
python3 - <<'PY'
from pathlib import Path
required = [
    'docs/spec/SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md',
    'docs/plan/PLAN-121-TOWER-CALLABLE-SYNTAX.md',
    'docs/plan/tasks/TASK-955-tower-callable-syntax-packet.md',
    'docs/plan/tasks/TASK-956-callable-syntax-audit-gate.md',
    'docs/plan/tasks/TASK-957-pure-callable-type-parser.md',
    'docs/plan/tasks/TASK-958-callable-type-typeck-rendering.md',
    'docs/plan/tasks/TASK-959-pure-closure-arrow-syntax.md',
    'docs/plan/tasks/TASK-960-reserved-tower-callable-arrows.md',
    'docs/plan/tasks/TASK-961-callable-syntax-reference-docs.md',
    'docs/plan/tasks/TASK-962-tower-callable-syntax-closeout.md',
    'docs/plan/tasks/TASK-963-stdlib-and-reference-callable-syntax-migration.md',
]
for rel in required:
    assert Path(rel).exists(), rel
PY
```

Implementation-task verification adds focused parser/typechecker/engine tests named in TASK-956, stdlib/reference syntax migration scans from TASK-963, plus:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
python3 tools/reference/check_frontmatter.py
```

Closeout may scope markdown link checks to touched docs if unrelated historical link drift is present, but it must report any unrelated failures explicitly.

## 8. Closeout expectations

PLAN-121 is complete only when:

1. SPEC-072 C72-1 through C72-8 are mapped to concrete evidence.
2. PLAN-INDEX, PLAN-121, task files, spec index, legacy amended specs, `std/`, `reference/`, and CHANGELOG agree.
3. Focused tests prove `(A, B) -> C` is n-ary and not a tuple argument.
4. Focused tests prove old pure closure `|args| => body` is no longer silently accepted as pure syntax.
5. Reserved higher-stratum arrows fail closed with targeted diagnostics.
6. Current `std/` and `reference/` examples use preferred syntax except explicitly labeled compatibility examples.
7. Independent review checks parser ambiguity, stale docs, stdlib/reference migration coverage, and callable-stratum/return-type conflation.
