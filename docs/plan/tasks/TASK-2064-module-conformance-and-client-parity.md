# TASK-2064: Module Conformance and Client Parity

**Status:** Complete for the frozen callable-module completion domain
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §11; PLAN-203
**Owned rules:** MOD-REAL-001 through MOD-REAL-006 integration
**Run-route impact:** active
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2064](../SEMANTIC-RULE-COVERAGE.md#task-2064-module-conformance-and-client-parity)

## Semantic accounting

**Implementation:** implemented
**Evidence:** tested
**Parity:** matches_spec
**Completion scope:** Completion requires the callable-module source-to-terminal route and the
checked metadata dependencies it needs. Role semantics; policy instances, enforcement, persistence,
inheritance, or authority; and runtime behavior for roles/policies are out of scope. Role/policy
declarations are compatibility-only fixtures: their non-authorizing fences remain valuable, but
they are excluded from completion criteria and must not grow into runtime or admission semantics.
Conformance must also verify that every public declaration used by an importing module survives
the checked interface/import route with identity, origin, visibility, and metadata preserved;
non-callable declarations need import propagation, not standalone execution.
**Missing target-spec clauses:** None within the frozen source-to-finalization-to-Core/CPS-to-
Engine-to-CLI/daemon conformance domain. Raw synthesized-pattern compatibility APIs, dynamic
module loading, and role/policy authority remain explicitly outside the domain.
**Layers:** type/Core/CPS/admission-runtime `implemented`; verification `implemented`.
**Evidence identifiers:** positive `TEST-MOD-REAL-CONFORMANCE-POSITIVE`,
`TEST-MOD-REAL-CONFORMANCE-MAIN-PARITY`, and
`TEST-MOD-REAL-CONFORMANCE-PARAMETERIZED-CALL-PARITY`, and
`TEST-MOD-REAL-CONFORMANCE-MULTIPLE-IMPORTED-CALLABLES-PARITY`, and
`TEST-MOD-REAL-CONFORMANCE-CRATE-VISIBLE-CALLABLE-PARITY`, and
`TEST-MOD-REAL-CONFORMANCE-SUPER-VISIBLE-CALLABLE-PARITY`, and
`TEST-MOD-REAL-CONFORMANCE-RESTRICTED-VISIBLE-CALLABLE-PARITY`, and
`TEST-MOD-REAL-CONFORMANCE-LET-LOWERING` and
`TEST-MOD-REAL-CONFORMANCE-IF-LET-PARITY`; negative
`TEST-MOD-REAL-CONFORMANCE-SHORT-CIRCUIT-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-SHORT-CIRCUIT-LET-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-SHORT-CIRCUIT-IF-LET-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-SHORT-CIRCUIT-ARGUMENT-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-SHORT-CIRCUIT-MATCH-ARGUMENT-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-SHORT-CIRCUIT-NESTED-ARGUMENT-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-SHORT-CIRCUIT-RECORD-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-NESTED-SHORT-CIRCUIT-BOOLEAN-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-ORDINARY-ROOT-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-MODULO-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-RECORD-FIELD-CALL-PARITY`; negative
`TEST-MOD-REAL-CONFORMANCE-NESTED-RECORD-FIELD-CALL-PARITY`; negative
`TEST-MOD-REAL-CONFORMANCE-RECORD-FIELD-EXPRESSION-CALL-PARITY`; negative
`TEST-MOD-REAL-CONFORMANCE-DECLARATION-ORDER-PARITY`; negative
`TEST-MOD-REAL-CONFORMANCE-NEGATIVE`, `TEST-MOD-REAL-CONFORMANCE-BUILTIN-ADMISSION-FENCE`,
`TEST-MOD-REAL-CONFORMANCE-STRUCTURAL-NEGATIVE`, and
`TEST-MOD-REAL-CONFORMANCE-VISIBILITY-NEGATIVE`, and
`TEST-MOD-REAL-CONFORMANCE-AMBIGUITY-NEGATIVE`, and
`TEST-MOD-REAL-CONFORMANCE-INVALID-REEXPORT-NEGATIVE`, and
`TEST-MOD-REAL-CONFORMANCE-NO-IMPLICIT-FLATTENING`, and
`TEST-MOD-REAL-CONFORMANCE-IMPORT-CYCLE-NEGATIVE` and
`TEST-MOD-REAL-006-IMPORTED-CALLABLE-LINKING` and
`TEST-MOD-REAL-CONFORMANCE-LOCAL-CALL-CYCLE-NEGATIVE`;
mutation `TEST-MOD-REAL-CONFORMANCE-MUTATION`; property
`TEST-MOD-REAL-CONFORMANCE-METADATA-PROPERTY`; parity `TEST-MOD-REAL-CLI-DAEMON-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-TRANSITIVE-IMPORT` and
`TEST-MOD-REAL-CONFORMANCE-REEXPORT-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-TRANSITIVE-REEXPORT-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-NESTED-STRUCTURAL-CALLABLE-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-STRUCTURAL-MODULE-ALIAS-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-NESTED-STRUCTURAL-MODULE-ALIAS-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-CANONICAL-SOURCE-REEXPORT-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-IMPLEMENTATION-METADATA-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-IMPORTED-INTERFACE-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-IMPORTED-TYPE-SIGNATURE-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-IMPORTED-CONSTRUCTOR-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-IMPORTED-EFFECT-ROW-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-IMPORTED-TYPE-FUNCTION-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-IMPORTED-PROMOTED-KIND-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-IMPORTED-PROPOSITION-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-IMPORTED-EVIDENCE-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-IMPORTED-MACRO-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-IMPORTED-NEWTYPE-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-IMPORTED-RESOURCE-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-TYPE-REEXPORT-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-NOTATION-IMPORT-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-MAIN-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-PARAMETERIZED-CALL-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-MULTIPLE-IMPORTED-CALLABLES-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-CRATE-VISIBLE-CALLABLE-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-SUPER-VISIBLE-CALLABLE-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-RESTRICTED-VISIBLE-CALLABLE-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-SAME-MODULE-ALIAS-PARITY` and
`TEST-MOD-REAL-CONFORMANCE-SHORT-CIRCUIT-RECORD-PARITY`.
Production-client route evidence is also recorded by
`TEST-MOD-REAL-PRODUCTION-CLI-CANONICAL-ROUTE`,
`TEST-MOD-REAL-PRODUCTION-CLI-CANONICAL-INLINE-ROUTE`,
`TEST-MOD-REAL-PRODUCTION-DAEMON-CANONICAL-ROUTE`,
`TEST-MOD-REAL-PRODUCTION-DAEMON-CANONICAL-INLINE-ROUTE`, and
`TEST-MOD-REAL-PRODUCTION-DAEMON-CANONICAL-ROOT-INDEX`,
`TEST-MOD-REAL-PRODUCTION-CLI-ORDINARY-ROOT`, and
`TEST-MOD-REAL-PRODUCTION-DAEMON-ORDINARY-ROOT`, plus
`TEST-MOD-REAL-PRODUCTION-CLI-MODULO`, and
`TEST-MOD-REAL-PRODUCTION-DAEMON-MODULO`, plus
`TEST-MOD-REAL-PRODUCTION-CLI-RECORD-FIELD-CALL`, and
`TEST-MOD-REAL-PRODUCTION-DAEMON-RECORD-FIELD-CALL`, plus
`TEST-MOD-REAL-PRODUCTION-CLI-NESTED-RECORD-FIELD-CALL`, and
`TEST-MOD-REAL-PRODUCTION-DAEMON-NESTED-RECORD-FIELD-CALL`, plus
`TEST-MOD-REAL-PRODUCTION-CLI-RECORD-FIELD-EXPRESSION-CALL`, and
`TEST-MOD-REAL-PRODUCTION-DAEMON-RECORD-FIELD-EXPRESSION-CALL`, plus
`TEST-MOD-REAL-PRODUCTION-CLI-PARSEABLE-LOWERING-FAIL-CLOSED`, and
`TEST-MOD-REAL-PRODUCTION-DAEMON-PARSEABLE-LOWERING-FAIL-CLOSED`, plus
`TEST-MOD-REAL-PRODUCTION-CLI-PARSEABLE-INVALID-IMPORT-FAIL-CLOSED`, and
`TEST-MOD-REAL-PRODUCTION-DAEMON-PARSEABLE-INVALID-IMPORT-FAIL-CLOSED`, plus
`TEST-MOD-REAL-PRODUCTION-CLI-ROLE-POLICY-STUB-FENCE` and
`TEST-MOD-REAL-PRODUCTION-DAEMON-ROLE-POLICY-STUB-FENCE`,
`TEST-MOD-REAL-PRODUCTION-CLI-METADATA-ONLY-ROLE-POLICY-CHILD`, and
`TEST-MOD-REAL-PRODUCTION-DAEMON-METADATA-ONLY-ROLE-POLICY-CHILD`.
The same production routes now retain a checked handler-only child module through a neutral
non-selected carrier while preserving its handler body as non-authorizing local callable data,
with witnesses `TEST-MOD-REAL-PRODUCTION-CLI-HANDLER-ONLY-CHILD` and
`TEST-MOD-REAL-PRODUCTION-DAEMON-HANDLER-ONLY-CHILD`.
The focused corpus now passes 59/59: a real parser → collection → finalization → Core/CPS →
Engine-linked file/inline pair with equal terminals, including an ordinary checked `fn main`
entry, parameterized and multiple imported callables from one multi-function child, crate-, super-,
and restricted-visibility imported callables, and a multi-function child module
whose explicit selected-entry closure transports exactly one artifact per canonical module;
linked-artifact origin parity, incomplete-
closure rejection, provenance mutation rejection, inline metadata property parity, a public
callable-import identity linked and executed through the checked route, transitive imported-call
execution through both clients, and file/inline local-literal-`let`, short-circuit-`let`, boolean-`if let`, short-circuit-`if let`, short-circuit callable-argument, nested short-circuit callable-argument, record-field short-circuit, nested short-circuit boolean, and short-circuit `match`-argument lowering through both clients, public interface plus non-callable implementation metadata carried
through both clients, an imported public interface carried through the same route, a same-module
aliased private callable closure carried through the same route, imported public type metadata in
a child callable signature, recursive selected-entry alias
rejection at the Engine cycle fence, plus real
parser visibility and import-cycle rejection before
lowering/finalization. Imported public constructor identity now also traverses the same
public-interface and Engine route in both source forms, as does imported public effect-row
metadata in a child callable signature. Imported public nominal-newtype metadata also crosses the
same route in both source forms, as does imported public resource-type metadata. Imported public
type re-export identity is also preserved from its original provider through both source forms
and both clients. A callable re-exported through two public importing modules now preserves its
original provider identity and reaches both clients in file and inline forms.
An ordinary callable reached through a nested public structural child also reaches both clients
in file and inline forms while the intermediate structural facade remains metadata-only.
Imported public
notation imports also preserve their defining provider identity as syntax-phase, non-callable
Core/CPS metadata through both source forms and both clients. Imported public
type-function and promoted-kind metadata also
cross the same route in both source forms. Public law/evidence metadata is also carried as an
explicit non-authorizing import through both source forms and both clients. Existing shared-client evidence also passes TASK-2032's
7/7 adapter suite. Public role and policy imports are additionally exercised through file and
inline canonical routes and both clients only as non-authorizing metadata stubs. Production `run` and daemon execution now select the
canonical source → finalization → Core/CPS → Engine-linked route for ordinary roots,
file-backed children, and inline children. Daemon indexing treats a canonical root as
one definition and excludes its child files; its runtime artifact summary is
explicitly metadata-only, while execution remains Engine-linked. The remaining
gap is breadth of source-driven rule coverage, not a second execution route.
Production CLI and daemon witnesses also place minimal role/policy declarations
beside a real callable and verify that these scheduled-for-removal metadata
stubs do not enter callable linking, admission, or runtime authority. Separate
file-backed CLI and daemon witnesses verify that a child module containing only
those metadata stubs remains in the canonical structural closure without
requiring a synthetic callable entry.
**Next obligation:** None within Phase 207. Role/policy fixtures remain regression fences only and
must not grow into authorization semantics.
The older `Next obligation` sentence immediately below is retained only as historical handoff
evidence; its broader-corpus wording is follow-on work and is not a Phase 207 blocker.

## Description

Prove the complete module route with positive, negative, mutation, and client-parity evidence. This task does not invent semantics; it validates the interfaces and execution artifacts produced by TASK-2057 through TASK-2063.

## Dependencies

- ✅ TASK-2067 — canonical graph/state, real module-unit transport, and structural diagnostics.
- ✅ TASK-2073 — complete checked final interfaces, export closure, and normalized Type file/inline
  interface parity, after TASK-2070/2071/2072's separately owned prerequisites.
- ✅ TASK-2069 — complete definition-body lowering and Engine scanner/cache transport fencing.
- ✅ TASK-2063 — Engine-sealed admitted linked module execution after TASK-2069.

## Requirements

1. Establish a compact conformance corpus with paired file and inline module programs.
2. Cover structural graph construction, source parity, public/private interfaces, imports, every visibility form, re-exports, identity preservation, and lowering parity.
3. Cover missing child, duplicate child, structural cycle, import cycle, ambiguity, inaccessible declaration, malformed/stale interface, and forbidden fallback diagnostics.
4. Add property/mutation tests for order independence, alias identity preservation, no implicit flattening, and text-lookalike resistance.
5. Compare one identical admitted multi-module program through CLI and daemon and assert normalized terminal equality.

## TDD Steps and evidence

1. Add rule-indexed fixtures before extending the implementation, one positive and one negative case per `MOD-REAL-*` rule. The bounded positive/negative/mutation/parity slice is now present in `crates/ash-cli/tests/task_2064_module_conformance_and_client_parity.rs`.
2. Add a fixture generator that can materialize equivalent inline/file trees.
3. Run the generator against parser, graph, interface, Core/CPS, and Engine snapshots; shrink any mismatch to a minimal declaration tree.
4. Add mutation controls that inject a source scan, direct evaluator fallback, interface identity rewrite, or visibility bypass and show at least one test fails.
5. Record the exact theorem scope of any proof attempt separately; tests remain `tested`, not `proved`.

## Completion checklist

- [x] Every in-scope callable-route `MOD-REAL-*` rule has positive, negative, and mutation evidence.
- [x] Real ordinary-function file/inline module pairs agree at interface, Core/CPS, admission, and terminal layers.
- [x] CLI and daemon execute the same real linked artifact with equal normalized terminals.
- [x] Focused TASK-2064 and shared-client tests, fmt, and targeted clippy pass.

## Handoffs

- **Consumes:** TASK-2067 structural evidence, TASK-2073 checked finalization/export-closure
  evidence, TASK-2069 full lowering/transport evidence, and TASK-2063's Engine-sealed route
  evidence. TASK-2070/2071/2072 are included through TASK-2073's complete handoff.
- **Produces:** rule-indexed implementation/evidence/parity reports, CLI/daemon terminal comparison, and closeout inputs.
- **Downstream owner:** TASK-2065 owns closeout, documentation, traceability, and review remediation.
- **Non-goals:** new language behavior, new direct evaluator, or broad package/workspace functionality.

## Files and verification

**Files:** focused parser/core/typeck/engine integration tests; CLI and daemon parity tests; `docs/plan/SEMANTIC-RULE-COVERAGE.md`; traceability records required by active semantic-task policy.

```text
cargo test -p ash-parser module
cargo test -p ash-core module
cargo test -p ash-typeck module
cargo test -p ash-engine module
cargo test -p ash-cli module
cargo fmt --check
```

Current focused verification:

```text
  cargo test -p ash-cli --test task_2064_module_conformance_and_client_parity  # 59/59
cargo test -p ash-cli --test task_2032_shared_engine_client_parity           # 7/7
cargo test -p ash-cli --lib commands::run::tests::task_2064_production_run_uses_canonical_module_route
cargo test -p ash-cli --lib commands::daemon::tests::task_2064_daemon
cargo test -p ash-cli --lib commands::run::tests::task_2064_production_run_keeps_role_policy_stubs_out_of_callable_route
cargo test -p ash-cli --lib commands::daemon::tests::task_2064_daemon_keeps_role_policy_stubs_out_of_callable_route
cargo test -p ash-cli --lib commands::run::tests::task_2064_production_run_allows_metadata_only_role_policy_child_module
cargo test -p ash-cli --lib commands::daemon::tests::task_2064_daemon_allows_metadata_only_role_policy_child_module
cargo test -p ash-cli --lib handler_only_child_module
```
