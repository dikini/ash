# PLAN-201: Deprecated Functionality Removal

**Status:** Base packet complete (11/11 tasks complete); semantic-cleanup follow-up remains in progress.
**Depends on:** Phase 200 Tooling And Migration Polish.
**Specs/notes:** `PLAN-200`, `PLAN-199`, `PLAN-196`, `PLAN-195`, `SPEC-095b`,
`SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`, and
`NOTE-035`.

## Goal

Remove deprecated Ash functionality completely from the repository's code, fixtures, examples,
templates, tooling behavior, and executable/checkable/lowerable/formattable paths. After this
phase, Ash source in the project repository must use target Ash only.

## Architecture

Phase 200 demoted deprecated forms from productive surfaces. Phase 201 turns that policy into hard
removal. The phase starts with a dependency audit that classifies every remaining deprecated
syntax, AST, lowering, type/effect, runtime, tooling, fixture, and documentation occurrence as
one of three outcomes: remove now, rename to target vocabulary, or keep only as historical prose
with explicit labels.

Removal proceeds from the user boundary inward: parser/checker acceptance first, then AST/lowering
and type/effect/runtime carriers, then formatter/LSP/template behavior, then docs/reference
quarantine and fail-closed removal gates. No implementation task may remove an internal carrier
until the audit proves it is not still needed for current target Ash behavior or has a target
vocabulary replacement.

Phase 201 also includes a semantic-removal audit because vocabulary retargeting alone can hide
stale functionality under target-shaped names. Any slice that only renames an old carrier, registry,
adapter, parser path, runtime path, fixture, or diagnostic must prove that the retained mechanism is
part of target Ash. If the mechanism only supports old workflow/tower/capability compatibility, it
must be deleted or assigned to a concrete deletion plan before closeout.

## Scope

- Audit all remaining deprecated functionality and produce `AUDIT-201`.
- Remove parser/checker acceptance of deprecated syntax as valid Ash.
- Remove or rename legacy surface AST, lowering, type/effect, and runtime carriers that are no
  longer part of target Ash.
- Remove deprecated behavior from formatter, LSP, template validation, examples, and test fixtures.
- Remove deprecated Ash snippets from repository code, including `.ash` files, templates, examples,
  fixtures, snapshots, and Rust source string literals.
- Quarantine historical documentation and update current/target spec indexes to describe removed
  functionality honestly.
- Add fail-closed gates that prevent deprecated functionality from re-entering executable or
  productive paths.

## Non-Goals

- No new language syntax.
- No new provider, runtime, authority, process, or contract model.
- No semantic expansion of target Ash.
- No removal of historical/reference prose that is explicitly labeled and excluded from productive
  docs, examples, templates, formatter, LSP, and runtime gates.
- No requirement to preserve migration diagnostics for removed syntax when preserving those tests
  would require stale Ash snippets in repository code.

## Design Locks

1. Deprecated functionality must not parse/check/lower/run/format/template as valid Ash after this
   phase.
2. Deprecated Ash forms must not be present as repository code: not in `.ash` sources, examples,
   templates, fixtures, snapshots, or Rust test/source string literals.
3. Internal carrier removal must be audit-driven. Do not delete a carrier until current target
   tests prove it is unused by target Ash or replaced by row/profile/provider/evidence carriers.
4. Productive docs, examples, templates, and stdlib snippets must remain target-only.
5. Historical/reference material may mention removed forms only as prose, with explicit labels and
   gates that prevent those mentions from becoming Ash code snippets.

## Task Overview

| Task | Description | Estimate | Depends on | Status |
|------|-------------|----------|------------|--------|
| [TASK-1960](tasks/TASK-1960-deprecated-functionality-removal-plan-packet.md) | Create the Phase 201 plan and task packet | 2h | Phase 200 | ✅ Complete |
| [TASK-1961](tasks/TASK-1961-deprecated-functionality-dependency-audit.md) | Audit remaining deprecated functionality and classify removal owners | 14h | TASK-1960 | Complete |
| [TASK-1962](tasks/TASK-1962-parser-checker-deprecated-acceptance-removal.md) | Remove parser/checker acceptance of deprecated Ash forms | 18h | TASK-1961 | Complete |
| [TASK-1963](tasks/TASK-1963-surface-ast-lowering-legacy-carrier-removal.md) | Remove unreachable legacy surface AST and lowering carriers | 18h | TASK-1962 | Complete |
| [TASK-1964](tasks/TASK-1964-type-effect-runtime-deprecated-carrier-removal.md) | Remove deprecated type/effect/runtime vocabulary and carriers | 20h | TASK-1963 | Complete |
| [TASK-1965](tasks/TASK-1965-tooling-deprecated-behavior-removal.md) | Remove deprecated formatter, LSP, template, and CLI behavior | 16h | TASK-1962 | Complete |
| [TASK-1966](tasks/TASK-1966-docs-reference-historical-quarantine.md) | Quarantine historical docs and reconcile current/target spec references | 12h | TASK-1961, TASK-1965 | Complete |
| [TASK-1967](tasks/TASK-1967-deprecated-functionality-removal-gates.md) | Add fail-closed gates for deprecated functionality removal | 14h | TASK-1962, TASK-1963, TASK-1964, TASK-1965, TASK-1966 | Complete |
| [TASK-1968](tasks/TASK-1968-deprecated-functionality-removal-closeout.md) | Close out Phase 201 with full gates, stale-claim sweep, docs, and review remediation | 8h | TASK-1967 | Complete |
| [TASK-1969](tasks/TASK-1969-semantic-removal-vs-rename-audit.md) | Audit Phase 201 for rename-only cleanup and stale mechanisms preserved under target names | 16h | TASK-1961, TASK-1962, TASK-1963, TASK-1964, TASK-1965, TASK-1966, TASK-1967 | ✅ Complete |
| [TASK-1970](tasks/TASK-1970-semantic-cleanup-plan-from-audit.md) | Elaborate the deletion/refactor plan from the semantic-removal audit and target specs | 10h | TASK-1969 | ✅ Complete |

Estimated implementation effort after the plan packet: 146 hours.

## Required Audit

TASK-1961 must produce `docs/plan/audits/AUDIT-201-deprecated-functionality-removal.md` with at
least these tables:

- Deprecated syntax accepted by parser/checker.
- Deprecated AST, lowering, Core/CPS, type/effect, runtime, and report carriers.
- Deprecated diagnostics, formatter, LSP, template, and CLI behavior.
- Deprecated examples, fixtures, stdlib snippets, source string literals, docs, and references.
- Each occurrence's owner task, outcome, replacement target, required tests, and removal risk.

The audit must distinguish:

- **Remove:** deprecated Ash code, executable/checkable/lowerable behavior, fixtures, and tooling
  behavior deleted in this phase.
- **Rename:** internal or diagnostic vocabulary moved to target Ash terms without retaining old Ash
  source forms.
- **Historical prose only:** labeled reference prose excluded from productive paths and containing
  no deprecated Ash code block or snippet.

TASK-1969 must extend or accompany `AUDIT-201` with a semantic-removal audit that identifies
cleanup slices where deprecated mechanisms may have been preserved under target vocabulary. It must
include at least these tables:

- Phase 201 cleanup slice, changed files, and whether the slice deleted behavior or only renamed
  identifiers.
- Retained runtime/parser/typechecker/tooling mechanisms whose target-architecture justification
  is not yet proven.
- Code paths where old workflow/tower/capability/function-entry distinctions survived under names
  such as entry, application, computation, callable, registry, bridge, adapter, shim, or fallback.
- Tests that currently prove renamed APIs still work but do not prove stale functionality is gone.
- Target-spec replacement for each retained mechanism, or deletion/refactor owner when no target
  replacement exists.

The semantic-removal audit must classify each row as:

- **Delete now:** code is stale functionality, no target Ash path depends on it.
- **Refactor to target primitive:** code is useful, but only after it is integrated into the target
  mechanism such as ordinary functions with effect rows, row admission, provider profiles, contract
  evidence, process/channel primitives, or application runtime reports.
- **Keep as implementation detail:** code is target-justified and has no user-visible stale
  semantic distinction. The audit must cite the target spec and the test that proves this.
- **Plan required:** the audit cannot safely decide without a detailed deletion/refactor plan; the
  row must be owned by TASK-1970.

## Required Test Families

### Audit Inventory Tests

The inventory must fail when deprecated Ash forms appear in parser, typechecker, engine, runtime,
formatter, LSP, template, example, fixture, snapshot, Rust source string literal, or productive-doc
paths.

### Parser And Checker Removal Tests

Deprecated forms such as legacy `workflow` entry syntax, old `observe ... with`, old
`act ... with`, public `Act`/`Proc`/`Workflow` tower carriers, legacy capability declarations,
direct provider/capability authority forms, and obsolete callable arrows must reject as invalid
current Ash. Tests must not preserve deprecated Ash snippets in repository source; use grammar
unit assertions, generated token streams, or denylist gates that prove the forms are absent.

### Internal Carrier Removal Tests

Surface-to-Core, Core-to-CPS, runtime, reports, traces, and engine summaries must prove that
current target Ash no longer depends on deprecated carriers. Remaining internal names must either
be renamed to target vocabulary or removed.

### Tooling Removal Tests

Formatter, LSP, template CLI, `ash check`, and example/doc gates must reject deprecated inputs
instead of preserving, normalizing, instantiating, or navigating them as valid current Ash. The
repository must not retain deprecated Ash source snippets to exercise those paths.

### Documentation Quarantine Tests

Productive docs, tutorials, templates, and examples must contain no deprecated functionality.
Historical/reference docs may mention removed forms only as labeled prose, not as Ash code blocks,
snippets, examples, or fixtures.

## Verification Gates

Each implementation task must run focused tests for touched crates plus:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
```

Phase closeout must run:

```bash
cargo fmt --check
cargo test --all
cargo clippy --all-targets --all-features
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```

## Stale-Claim Sweep

Before closeout, search live code, docs, examples, templates, formatter fixtures, LSP fixtures,
parser fixtures, stdlib comments, and runtime reports for stale claims or executable remnants:

```text
legacy workflow
deprecated syntax
compatibility-only
old syntax
observe .* with
act .* with
Proc<|Act<|Workflow<
Expr::Act|Expr::Proc|Expr::Workflow
Type::Act|Type::Proc|Type::Workflow
capability.*direct provider
ambient authority
legacy callable
workflow.*accepted
```

Historical/reference-only prose may mention removed forms only when explicitly owned by AUDIT-201
and excluded from productive execution, formatting, LSP, template, and tutorial paths. Deprecated
Ash snippets must not remain in repository code, fixtures, templates, examples, snapshots, or Rust
source string literals.

## Acceptance Criteria

- [x] Phase 201 plan and task files exist and are indexed.
- [x] AUDIT-201 classifies every remaining deprecated functionality occurrence and owner.
- [x] Deprecated syntax no longer parses/checks/lowers/runs as valid Ash.
- [x] Deprecated Ash forms are absent from repository code, fixtures, examples, templates,
      snapshots, and Rust source string literals.
- [x] Legacy AST/lowering/type/effect/runtime carriers are removed or renamed to target
      vocabulary.
- [x] Formatter, LSP, CLI, template, example, and docs paths reject deprecated functionality.
- [x] Productive docs and examples remain target-only; historical docs use labeled prose only.
- [x] Fail-closed gates prevent deprecated functionality from re-entering productive or executable
      paths.
- [x] Closeout gates, changelog, docs gates, stale-claim sweep, and review remediation complete the
      phase.
- [x] Semantic-removal audit proves Phase 201 cleanup removed stale mechanisms rather than only
      renaming them.
- [x] TASK-1970 deletion/refactor plan exists for every retained mechanism that is not proven to
      be target-justified.
