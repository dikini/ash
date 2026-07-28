# TASK-2035: Canonical Client and Test Contracts

**Status:** Complete
**Semantic task classification:** semantic-contract-definition
**Phase:** [PLAN-204](../PLAN-204-DIRECT-AST-RETIREMENT-AUDIT-AND-CONTRACT-FREEZE.md)
**Depends on:** TASK-2034, SPEC-011, SPEC-026, SPEC-077, and the target Core/CPS specifications
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Rule coverage:** [Engine-only client contracts](../SEMANTIC-RULE-COVERAGE.md#engine-only-client-contracts)

## Description

Amend the target contracts before runtime work so every supported test and REPL evaluation is a
Surface-Ash program admitted by Engine. Define each selected source-derived synthesized-test wrapper
by an exact source-contract ID and its consumed audit record: a source-backed callable identity,
exact source-representable literal inputs, a source-backed postcondition, and an exact wrapper
`main` that returns the observable Boolean or terminal result. The wrapper is parsed, checked,
lowered to checked Core/CPS, admitted, and run by Engine. Core-only predicates, closure
environments without exact source reconstruction, non-source-representable inputs, and any case
absent from the declared catalogue are deferred.

Amend SPEC-077 for the wrapper route, SPEC-011 for REPL-as-Engine-client evaluation, and SPEC-026
for single-executor conformance evidence. No contract may authorize AST evaluation, a
differential oracle, or a client-local CPS evaluator.

## Requirements

- Define only source-contract IDs with their exact source wrappers, consumed audit records, and
  explicit deferred results; do not specify a general synthesizer, a generated form, a case cross
  product, or a new test language.
- Specify Engine as the sole evaluator for normal REPL submissions and selected exact source
  wrappers.
- State the exact rule IDs, target-spec changes, and implementation/evidence/parity reporting
  obligations that TASK-2038, TASK-2039, and TASK-2042 must activate before semantic Rust work.
- Keep Lean as a deferred formalization handoff, not a conformance executor or runtime-proof
  claim.

## Handoffs

- **Run-route impact:** `prerequisite`.
- **Consumes:** TASK-2034's enumerated audit and existing target grammar/type/Core/CPS rules.
- **Produces:** rule IDs, source-wrapper contract, deferred-case classifications, and observable
  result contract for TASK-2038, TASK-2039, and TASK-2042.
- **Downstream owner:** TASK-2038 implements test wrappers; TASK-2039 implements REPL;
  TASK-2042 implements daemon transport and `ash run` parity; TASK-2041 updates final
  traceability/evidence after activation.
- **Does not own:** wrapper lowering, Engine APIs, REPL implementation, or execution coverage.
- **Integration/proof responsibility:** TASK-2038/2039 own focused execution evidence; TASK-2041
  owns cross-client final conformance evidence.

## TDD and activation steps

1. Promote this task and add a coverage section and traceability links naming the amended target
   rules. A semantic-task record is required before any later semantic Rust change.
2. Add contract examples and negative examples for one supported source wrapper and each enumerated
   deferred category; examples define the contract and are not runtime evidence.
3. Amend SPEC-077, SPEC-011, SPEC-026, the canonical read path, and any linked current contract
   so they state one Engine executor without contradictory direct-runtime wording.
4. Validate links, specification indexes, and semantic-task records.

## Semantic workflow record

**Canonical rules:** `CONF-SYNTH-SOURCE-WRAPPER-001`, `OBS-REPL-ENGINE-CLIENT-001`, and
`CONF-ENGINE-ONLY-CLIENT-001`.

**Implementation:** not_implemented
**Evidence:** none
**Parity:** below_spec

The contract text is complete for its declared catalogue. The three axes report the
runtime realization of those target rules, not the existence of this documentation. A prose
example or a frozen audit row is not test or proof evidence.

**Missing target-spec clauses:** Realize every selected wrapper, REPL route, and daemon route through Engine; then realize the remaining target SPEC-077 and SPEC-011 domains before claiming parity.

**Layers:** type partial; core partial; cps partial; admission-runtime not_implemented;
verification not_implemented.

**Run-route impact:** prerequisite.

**Consumes:** `AUDIT-204-TEST-EXEC-002`, `AUDIT-204-REPL-001`, `AUDIT-204-REPL-002`,
`AUDIT-204-CLIENT-006`, and `AUDIT-204-DEFERRED-001` through `AUDIT-204-DEFERRED-007`; target
grammar/type/Core/CPS rules; and the existing Engine admitted-request seam.

**Produces:** the exact source-wrapper catalogue and fail-closed case results in `SPEC-077`,
the REPL Engine-client rule in `SPEC-011`, and the single-executor comparison rule in `SPEC-026`.

**Does not own:** Source lowering, Engine APIs, test-runner execution, REPL execution, daemon transport, a general source synthesizer, and Lean implementation.

**Integration/proof responsibility:** TASK-2038, TASK-2039, and TASK-2042 must supply focused
tests for their routes. TASK-2041 owns four-client parity. The deferred Lean separate project,
not this task, owns any Lean implementation, conformance comparison, theorem, or refinement
bridge.

**Next obligation:** TASK-2038, TASK-2039, and TASK-2042 must implement their named routes with focused tests; TASK-2041 must establish the same-source-contract four-client terminal comparison.

## Contract examples and enumerated deferred cases

The source-contract IDs below are defined by this task. The audit IDs name retirement records that
the contracts consume; they do not contain the source text or digest. Source digests cover the
shown lines separated by LF and terminated by LF.

| Source-contract ID | Consumed audit record | Exact source | Callable/input/postcondition | Required observation |
|---|---|---|---|---|
| `TASK-2035-SYNTH-WRAPPER-001` | `AUDIT-204-TEST-EXEC-002` | `fn contract_target_zero() -> Int { 0 }`<br>`fn main() -> Bool { contract_target_zero() == 0 }`<br>source digest `sha256:71990ce4a503c89efb95340a6d7c6674a036858b8e337f8b9bc4337839ebe390` | callable `contract_target_zero`; input `[]`; source postcondition `contract_target_zero() == 0` in `main` | Engine terminal projection of `Bool(true)` after parse, check, checked Core/CPS lowering, and admission. |
| `TASK-2035-REPL-ROUTE-001` | `AUDIT-204-REPL-001` | `fn main() -> Int { 42 }`<br>source digest `sha256:ed4088d136e54744d258b170222ad3b2a064feda91b78b0a248f2ccfb9b7684c` | normal evaluable REPL source; entry `main`; input `[]` | Engine terminal projection of `Int(42)`. |
| `TASK-2035-REPL-ROUTE-002` | `AUDIT-204-REPL-002` | `fn main() -> Bool { 1 == 1 }`<br>source digest `sha256:697ab016d7ae6b9ab7088d17713e0e57d91965b911fc02d1ae1e0da54fa77811` | normal evaluable REPL source; entry `main`; input `[]` | Engine terminal projection of `Bool(true)`. |

The first row is the selected synthesized-contract implementation case. It does not widen
`SPEC-077` into a general wrapper generator or authorize a new source form. The two REPL rows are
selected route controls, not the complete `SPEC-011` evaluation domain. Target-spec wrapper forms
outside these task cases retain their target-spec status; an unimplemented route may defer or reject
at its admitted boundary, but it may not use an AST fallback.

### Shared four-client parity case

`TASK-2035-SHARED-ROUTE-001` is the sole shared parity case for TASK-2038, TASK-2039,
TASK-2042, and TASK-2041. It consumes `AUDIT-204-CLIENT-006` and
`TASK-2035-REPL-ROUTE-001`; it does not claim that its source was present in the audit.

| Field | Exact value |
|---|---|
| source identity | `task-2035-shared-int-42-v1` |
| source digest | `sha256:ed4088d136e54744d258b170222ad3b2a064feda91b78b0a248f2ccfb9b7684c` |
| source | `fn main() -> Int { 42 }` followed by LF |
| entry | `main` |
| inputs | `[]` |
| bindings | `{}` |
| run control | `{ deadline: none, cancellation: none, host_profile: none }` |
| expected normalized terminal envelope | `CanonicalTerminalEnvelopeV1::returned(Value::Int(42))` |

TASK-2038 must submit this source through the test client, TASK-2039 through REPL, TASK-2042
through daemon and `ash run`, and TASK-2041 must compare all four normalized envelopes. Direct
clients take source and retain its exact bytes in their local Engine; the daemon validates the
complete submitted descriptor. Each execution independently mints its process-local opaque
request. No route may substitute the source contract or terminal result, or select another
evaluator.

| Audit case ID | Missing obligation | Required fail-closed result |
|---|---|---|
| `test:contract_postcondition_without_executable_target_metadata` | executable postcondition target metadata | `deferred: contract metadata lacks executable postcondition target metadata` |
| `test:contract_postcondition_without_structured_oracle_metadata` | source-backed structured postcondition wrapper | `deferred: contract postcondition metadata is not executable` |
| `test:contract_postcondition_with_unsupported_target_kind_defers` | target-spec clause for a runtime-callable contract wrapper | `deferred: unsupported contract target kind runtime_callable` |
| `test:contract_postcondition_with_missing_setup_defers` | admitted source wrapper for missing setup metadata | `deferred: contract target execution setup is missing` |
| `test:contract_postcondition_explicit_finite_setup_defers` | target-spec clause for an exact setup wrapper | `deferred: explicit finite setup is not executable for pure target slice` |
| `test:contract_postcondition_unsupported_body_defers` | source-backed admitted wrapper for an unsupported contract body | `deferred: contract target body is not executable` |
| `test:contract_postcondition_missing_exact_input_defers` | exact source literal input wrapper | `deferred: contract postcondition oracle lacks exact valid input representatives` |

The table is a normative contract example and negative-case catalogue, not runtime evidence. Each
row remains an explicit enumerated deferred result until the target specification authorizes and the
Engine route realizes it.

## Lean handoff

Lean is deferred to the separate `external:lean-reference-project`. It is not a case in the
exact source-wrapper catalogue; it has no current execution, conformance, proof, or refinement
authority for Ash. A later separate project must define and check a refinement bridge before any
Lean theorem is reported as production-runtime evidence.

## Completion checklist

- [x] The target specification defines the wrapper input, identity, terminal observation, and
      fail-closed deferred disposition.
- [x] REPL normal evaluation is specified as an Engine client; non-evaluating commands retain
      their separately declared inspection behavior.
- [x] Differential comparison is not a current conformance route.
- [x] The task reports implementation/evidence/parity independently and does not claim runtime
      realization from a contract amendment.

## Documentation verification

- `python3 -m unittest tools.docs.tests.test_task_2035_semantic_task_record`
- `python3 -m unittest tools.docs.tests.test_task_2035_contract_documents`
- `python3 tools/docs/validate_semantic_task_records.py --root . --manifest docs/plan/semantic-task-records.json`

These commands validate the documentation contract and record. They do not change the runtime
evidence status of `none`.
