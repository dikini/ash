# TASK-2002: Generic `do` and Lowering-Sidecar Strategy

**Status:** In progress — the engine source-entry boundary now accepts ambient `do` with
source/evidence/Core-sequencing evidence, retains an entry-body source-anchor sidecar, expands
surface macros before source-entry lowering, retains successful macro-expansion origin sidecars,
retains successful notation-expansion origin sidecars,
and rejects every named `do:<target>` deterministically;
it also retains every local callable's fully lowered contract artifact and rejects an invalid local
contract before publishing an entry. Predicate environments now retain exact parameter-name spans,
the synthetic `result` binder uses the enclosing `FnDef` signature span, and file/direct provenance
is explicit; broader lowering-sidecar and conformance work remains open.
**Phase:** Follow-up from [TASK-1988](TASK-1988-semantic-implementation-deprecation-audit.md)

**Status:** In progress

**Semantic task record:** [TASK-2002 workflow record](../semantic-task-records.json)

**Semantic coverage map:** [TASK-2002 semantic workflow record](../SEMANTIC-RULE-COVERAGE.md#task-2002-semantic-workflow-record)

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Carry every required target sidecar or an explicit unsupported outcome through lowering.

## Semantic workflow record

Ambient `do` and retained sidecars do not realize every target lowering clause. The tests listed
in the record provide confidence only for the realized behavior.

## Description

Decide and realize the target relationship between retained ambient `do`, rejected generic `do`,
and required macro/notation, handler, evidence, trace, and diagnostic lowering sidecars.

## Requirements

- State the exact supported `do` domain and explicit rejection diagnostics.
- Preserve source origin and record all required target sidecars through lowering.
- Keep macro/notation expansion distinct from Core semantics and authority grant.
- Compare supported lowering with canonical `LOWER-SURFACE-CORE-001` obligations.

## TDD Steps

1. Add red fixtures for generic `do`, ambient `do`, and sidecar preservation.
2. Define expected Core term plus origin/evidence/trace/diagnostic sidecars.
3. Implement the selected lowering strategy.
4. Run parser→typechecker→Core boundary and docs/traceability gates.

## Completion Checklist

- [x] Generic and ambient `do` behavior is unambiguous and tested at the current source-entry boundary.
- [x] Every local `fn` retains its fully lowered contract artifact, while an invalid helper contract
  rejects atomically before an `Entry` exists. Each retained arithmetic `requires` expression and
  `ensures` clause has its exact source offsets; file-backed lowering also retains the module path,
  while direct/in-memory lowering deliberately retains no file path.
- [x] Predicate environments retain exact parameter declaration-name spans and a stable enclosing
  `FnDef` signature span for synthetic `result`; file-backed lowering retains the canonical path
  while direct/in-memory lowering intentionally retains `file: None`.
- [ ] Required sidecars survive lowering or have explicit unsupported-boundary outcomes.
- [ ] No macro/notation shortcut becomes semantic authority.
- [ ] Conformance fixtures and changelog are updated.

## Evidence required

TASK-1988 found ambient lowering retained while generic `do` rejects and sidecar coverage absent.
Completion requires end-to-end behavior evidence rather than a syntactic rewrite.

## Current partial implementation evidence

The focused end-to-end fixture
[`task_2002_do_lowering_sidecars.rs`](../../../crates/ash-engine/tests/task_2002_do_lowering_sidecars.rs)
records the supported ambient path.  A file-backed `fn main` with `do { ... }` preserves its parsed
file origin, retains `evidence audit_log` as a callable `where row` requirement summary, typechecks
without converting that requirement into authority, and lowers the block to an ordinary Core `Let`
spine.  The fixture also requires `do:K`, `do:Act`, `do:Proc`, and `do:Workflow` to fail before
typechecking or generalized-do lowering with exactly:

```text
generic do target annotations are removed; use ambient `do { ... }` with row requirements
```

This records source-entry rejection, existing ambient requirement metadata, and one entry-body
source anchor. The fixture proves that the lowered `Entry` carries an
`EntryLoweringSidecars::entry_body_origin` with the source file and `main` span. This is an
enclosing callable anchor, not a claim that every legacy Core `Expr` node has a target-Core
annotation. It does **not** establish that every lowered Core term carries a source-origin
sidecar, nor that facts, evidence, contracts, traces, and diagnostics are retained in a unified
lowering artifact. Macro and notation expansion, handler/provider sidecars, trace monitor plans,
and complete
`LOWER-SURFACE-CORE-001` conformance remain required work.  In particular, the lexical source-entry
guard is a rejection boundary, not a new semantic authority or a replacement for expanded-AST
lowering.

Successful source entry now crosses `expand_surface_module` before program extraction. The same
fixture proves that a local `answer!()` macro inside ambient `do` is expanded to
`CoreExpr::Literal(Int(42))`, never accepted as a residual macro carrier. This preserves the
separation between surface notation and Core semantics; expansion alone grants no authority.

The source-entry handoff additionally carries the successful
`ExpandedSurfaceOrigin` records emitted by that expansion into
`EntryLoweringSidecars::expansion_origins`. The `answer!()` fixture observes its retained macro
call-site origin there. These records are source diagnostic/audit metadata only: they do not add
per-node annotations to legacy `Expr`, emit a runtime trace event, install a trace monitor, or
produce an `ExpansionDiagnostic` for an otherwise successful entry. Failed expansion continues to
reject before an `Entry` exists. Trace contracts, monitor plans, unified diagnostics, broader
notation provenance coverage, and handler/provider sidecars remain open.

The companion local-notation fixture puts a declared `infixl <+> = combine` partial operator
section inside ambient `do`. It verifies that entry lowering contains no residual `<+>` carrier and
that `EntryLoweringSidecars::expansion_origins` retains the successful
`SurfaceOrigin::NotationExpansion` target `combine`. This is existing diagnostic/audit-only
sidecar behavior: it neither changes Core execution nor grants authority, emits a runtime trace,
installs a monitor, or establishes per-node notation provenance. Broader notation provenance and
the unified lowering-sidecar obligations remain open.

## All-local callable contract sidecar evidence

`EntryLoweringSidecars::callable_contracts` deterministically retains a complete
`LoweredFnContract` for every local `fn`, including a contract-less helper with an explicit empty
artifact. The focused helper-plus-`main` control retains requires/ensures discharge counts and the
public discharge-status projection. A zero-parameter inline-row callable records its enclosed
result type for the `result` binder, matching the typechecker rather than retaining the raw
function type. A parser-accepted helper `ensures: value >= 0` fails contract lowering because a
postcondition must be over `result`; source entry returns that failure before publishing a partial
`Entry` or sidecar map.

This contract-signature carrier is immutable diagnostic/evidence metadata only. It does not add a
callable row (the focused inline-row control retains only its explicitly declared row), enforce a
contract at execution, install a runtime check or monitor, select a provider, construct a frame,
admit an entry, or grant any runtime authority. It is not a claim of per-Core-term contract
annotation or complete `LOWER-SURFACE-CORE-001` conformance.

The source-accurate discharge control now compares every local callable sidecar's arithmetic
`requires` expression and `ensures` clause against its parsed source offsets. A file-backed module
retains that module path in each `CoreSourceSpan`; the public direct/in-memory lowering API retains
the same exact offsets with `file: None`. This is clause provenance only: it neither re-parses
source nor changes predicate meaning, discharge classification, authority, admission, execution,
monitoring, or diagnostics.

## Completed Predicate-Environment Provenance Slice

Each lowered `requires` and `ensures` predicate environment now carries source provenance for its
admitted binders. Parameter binders use the exact declaration-name span, not the enclosing
parameter/type annotation span. The synthetic `result` binder is anchored to the enclosing
`FnDef` signature span, because no source identifier exists for it. The focused direct and
file-backed controls prove the same offsets in both predicate environments; the latter carries the
canonical file path and the former deliberately carries `file: None`.

This completes binder provenance for the retained all-local contract sidecars. It does not make
predicate environments executable authority, alter contract evaluation/discharge, add per-Core
term provenance, or complete unified source/evidence/trace/diagnostic sidecars.

For ordinary file-backed modules, the loader consumes the leading `use`/`pub use` prelude for
import resolution but replaces its non-newline bytes with parse-neutral whitespace before local
callable parsing. The retained source has the original byte positions and line endings, so the
contract sidecars remain anchored to the original module rather than a shortened reparse buffer.
The direct/in-memory entry boundary applies the same-byte-length mask after validating each
accepted leading runtime import, so direct entry helpers and `main` retain their original clause offsets
with `file: None`; unregistered imports still reject before an `Entry` or sidecar exists. The
import-bearing regressions cover this coordinate-preserving transport. It remains provenance only
and does not alter import authority, contract execution, or admission.

`Engine::parse_entry_file` now follows the same file-backed runtime-entry rule. After validating
and same-byte-length masking its accepted leading runtime-import prelude, it passes the canonical
module identity into lowering; its entry anchor and every local contract discharge therefore retain
the canonical file path and original clause offsets. `parse_entry_source` intentionally remains the
direct/in-memory API and retains `file: None`. The focused runtime-entry file regression proves the
two local callable sidecars, file anchor, prelude masking, and source-coordinate distinction without
changing runtime-import authority, contract execution, or admission.
