# TASK-2018: Entry-Lowering Hygiene Sidecar Transport

**Status:** Complete — successful expanded source entries now transport the
parser-emitted identifier-hygiene vector unchanged into the entry-level
audit/diagnostic sidecar.  This remains non-authoritative metadata: it does
not alter Core, checking, evaluation, admission, provider dispatch, traces,
or monitors.  The legacy fallback's explicit empty vector remains a defensive
unreachable-path invariant, not a successful fallback behavior claim.
**Phase:** Implementation follow-up from
[TASK-2002](TASK-2002-generic-do-and-lowering-sidecar-strategy.md)

## Description

Carry the already validated identifier-hygiene metadata of a successful
`ExpandedSurfaceModule` across the source-entry extraction boundary into the
diagnostic/audit-only `EntryLoweringSidecars` product.  This fixes the present
lossy handoff where `module_loader::parse_program_with_functions` extracts a
`Program` and its expansion origins but discards
`ExpandedSurfaceModule::hygiene` before the engine creates the entry sidecars.

The intended product is an entry-level record of the exact expanded-surface
hygiene vector, not a reimplementation of hygiene, a Core annotation scheme,
or a runtime semantics feature.  The legacy fallback function parser has no
expanded-surface boundary; its `Vec::new()` hygiene sidecar is therefore a
defensive internal invariant only.  It is not currently an engine-success
route that a fixture can exercise: `parse_program_with_functions` tries the
surface parser/expander first, and the fallback uses the same function parser
order.  A source rejected by expansion after surface parsing returns that
expansion error before fallback, while a source the surface parser cannot
parse is rejected by the legacy function lowerer too.

## Authoritative References

- [SPEC-095c §6.4](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md):
  each generated binding/reference has explicit hygiene metadata, generated
  identifiers remain distinguishable in diagnostics, and incomplete metadata
  rejects before expanded output is accepted.
- [SPEC-098c §2 and §10](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md):
  lowering starts from expanded surface after hygiene validation, preserves
  source-origin products, and treats generated helper binders as surface
  hygiene metadata rather than authority, row, contract, failure, or proof
  evidence.
- [TASK-2002](TASK-2002-generic-do-and-lowering-sidecar-strategy.md): current
  entry body and expansion-origin sidecars are audit/diagnostic metadata only;
  full unified lowering sidecars remain open.

## Scope

### In scope

- Extend the engine-owned `ParsedProgram` transport product to retain the exact
  `Vec<ash_parser::surface::IdentifierHygieneMetadata>` emitted by a successful
  `expand_surface_module` call.
- Extend `EntryLoweringSidecars` with a clearly named hygiene field and carry
  the unchanged vector through `Engine::parse` / source-entry construction.
- Prove with an ordinary successful source entry containing a generated
  identifier that the entry sidecar has the same ordered metadata—including
  spelling, span, hygiene context, and expansion ID—as the expanded-surface
  product for that source.
- Prove that a successful source entry with ordinary source/call-site
  identifiers preserves their exact metadata too; the sidecar is not filtered
  to generated identifiers.
- Keep the legacy function-parser fallback literal explicit about its empty
  hygiene vector.  This is a defensive internal invariant, not a claimed
  end-to-end behavior: no currently known engine-success source reaches that
  route.
- Preserve the existing successful expansion-origin sidecar behavior and the
  fail-before-entry behavior for rejected expansion/hygiene validation.

### Explicit exclusions

- No change to macro/notation expansion, hygiene collection/validation,
  resolution, type checking, Core lowering, evaluator behavior, or entry
  execution.
- No authority, row, admission, provider, role, contract, evidence, failure,
  trace-event, trace-contract, monitor-plan, provenance, timeout, or
  cancellation effect may arise from this field.
- No diagnostics API redesign, persistence/indexing mechanism, import/export
  hygiene transport, or cross-module hygiene identity.
- No per-`ash_core::Expr` origin/hygiene attachment, Core IR schema change,
  Core/CPS production claim, or runtime trace/monitor event.

## Requirements and Invariants

1. **Exact transport, not reconstruction.** For a successful expanded source
   entry, `EntryLoweringSidecars::identifier_hygiene` equals the expanded
   module's hygiene vector in length, order, and every field.  The engine must
   transport it; it must not recompute, normalize, stringify, deduplicate, or
   infer hygiene from Core or source text.
2. **Boundary provenance.** Only a successful `ExpandedSurfaceModule` can
   supply nonempty entry hygiene.  Expansion/validation failure remains an
   entry-creation failure and exposes no partial `Entry` sidecar.
3. **Fallback honesty.** The legacy function-parser fallback returns an empty
   hygiene vector by construction.  This is retained as a defensive invariant
   for any future reachable fallback, not evidence of a current successful
   engine route.  The current parser order and shared function-parser grammar
   leave it unreachable for engine-success inputs: expansion failures return
   before fallback, and surface-parser failures are rejected by the legacy
   lowerer as well.  Empty must never be presented as successful hygiene
   validation.
4. **Audit/diagnostic only.** Reading or retaining the sidecar cannot alter
   `Entry::core`, evaluator output/error, `Engine::check`, callable rows,
   declared-operation resolution, admission, provider dispatch, trace facts,
   monitor installation, or terminal observables.
5. **No per-Core-origin claim.** The existing enclosing entry-body anchor
   remains the narrow Core-facing origin fact.  Hygiene remains entry-level
   sidecar metadata and does not decorate legacy Core terms.
6. **Compatibility.** Existing expansion-origin sidecar tests and source
   macro/notation lowering behavior remain unchanged.  New field defaults must
   be explicit at every direct `EntryLoweringSidecars` construction so test
   fixtures do not silently acquire unrelated semantics.

## TDD Steps

1. **Freeze the handoff.** Inspect
   `crates/ash-parser/src/surface.rs` (`ExpandedSurfaceModule::hygiene`),
   `crates/ash-engine/src/module_loader.rs` (`ParsedProgram` and
   `parse_program_with_functions`), and `crates/ash-engine/src/lib.rs`
   (`EntryLoweringSidecars`, `entry_lowering_sidecars`, and `Engine::parse`).
   Record all direct test constructions of `EntryLoweringSidecars` that need an
   explicit empty field.
2. **RED: exact successful-entry transport.** Add
   `crates/ash-engine/tests/task_2018_entry_lowering_sidecar_hygiene.rs` with a
   successful macro/binder fixture.  Independently call the parser expansion
   boundary and `Engine::parse`; assert exact vector equality and explicitly
   inspect generated/call-site contexts and expansion IDs.  Add an ordinary
   source-identifier control so the test cannot pass by retaining only
   generated items.
3. **GREEN: preserve parser-carried metadata.** Add the hygiene vector to
   `ParsedProgram`; move it from `ExpandedSurfaceModule` into the successful
   `program_from_module_file` return and into `entry_lowering_sidecars`.  Do
   not call hygiene collection from engine code and do not alter the parser
   collector.
4. **RED/GREEN: fallback invariant and rejection boundary.** Make the fallback
   `ParsedProgram` literal explicit about `Vec::new()`.  Do not invent a
   fallback-only success fixture: the same function parser order makes that
   route currently unreachable.  Instead, record and test the meaningful
   boundary: a failed expansion returns an error before fallback and cannot
   create an entry with an empty/partial sidecar.
5. **RED/GREEN: semantic non-interference.** In the focused task test, compare
   a sidecar-bearing source entry's checked row/core/evaluation result against
   its existing baseline and assert no trace or monitor product appears.  Keep
   this as a no-new-output control rather than broad trace semantics coverage.
6. **Regression and documentation.** Run the focused task test, TASK-2002
   sidecar tests, relevant parser hygiene tests, engine tests, formatting, and
   affected all-target/all-feature Clippy with warnings denied.  Once code and
   tests are green, update `CHANGELOG.md`, this task status,
   `PLAN-INDEX.md`, and semantic traceability links only for the implemented
   entry-level audit/diagnostic transport.  Run the docs/traceability gates and
   `git diff --check`.

## Expected Completion Evidence

- A successful ordinary source entry provides an `Entry` whose
  `lowering_sidecars.identifier_hygiene` is exactly equal to the parser's
  expanded-surface hygiene product, including generated and non-generated
  records and their IDs/spans/contexts.
- The unreachable legacy fallback literal supplies an explicit empty vector as
  a defensive invariant; a rejected expanded source supplies no entry at all.
  The evidence must not claim an end-to-end fallback fixture.
- Focused non-interference controls prove the added metadata does not change
  Core/evaluator/check/admission/provider/trace/monitor behavior.
- Existing TASK-2002 expansion-origin evidence remains green, while task,
  changelog, plan index, and traceability language make no per-Core-origin or
  runtime-authority claim.

## Completion Checklist

- [x] Successful expanded source entry transports exact identifier-hygiene
  metadata to `EntryLoweringSidecars`.
- [x] The engine does not reconstruct, filter, normalize, or infer that
  metadata.
- [x] The fallback parser literal explicitly produces an empty hygiene
  sidecar as a defensive invariant; documentation does not claim a currently
  reachable successful fallback route.
- [x] Failed expansion/hygiene validation cannot create a partial entry.
- [x] Non-interference evidence covers evaluation, checking, admission,
  provider, trace, and monitor authority boundaries.
- [x] No per-Core origin/hygiene metadata or Core/CPS production claim is
  introduced.
- [x] Focused/parser/engine regressions, formatting, Clippy,
  docs/traceability, and diff gates pass.
- [x] Completion documentation and changelog describe only the tested
  audit/diagnostic transport boundary.

## Completion Evidence

`crates/ash-engine/tests/task_2018_entry_lowering_sidecar_hygiene.rs` is
3/3.  Its notation-section entry independently expands the source and then
parses it through `Engine`, requiring byte-for-byte-equivalent structured
hygiene records (including ordering, spelling, spans, contexts, and expansion
IDs).  Its ordinary-identifier control proves that call-/definition-site
metadata is retained too, rather than filtering to generated records.  The
same successful-entry control keeps the existing empty row and no declared
operation facts, then checks and evaluates `Int(42)` with no new authority
product.  A duplicate-notation expansion error is returned as `EngineError::Parse`
before any `Entry` can be created.

The legacy `ParsedProgram` fallback literal explicitly sets
`identifier_hygiene: Vec::new()`.  It is a defensive internal invariant only:
the surface expansion is attempted first, expansion rejection returns before
fallback, and a source rejected by the surface parser is also rejected by the
legacy function lowerer.  No end-to-end successful-fallback fixture is
claimed.

Regression evidence: TASK-2002 sidecars 4/4, parser hygiene coverage, the
focused engine task 3/3, affected engine library tests, all-target/all-feature
engine Clippy with warnings denied, formatting, `git diff --check`, the
semantic-traceability validator, and the documentation gate passed.  QA and
review confirmed that the field has no evaluator, checker, admission,
provider, trace, monitor, Core-origin, or Core/CPS-production consumer.
