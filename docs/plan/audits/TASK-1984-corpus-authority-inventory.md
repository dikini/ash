---
id: audit.phase-202.task-1984.corpus-authority-inventory
title: TASK-1984 Corpus Authority and Conflict Inventory
kind: audit
status: current
authority_level: A5
lifecycle: active
canonical_for: [governance.corpus-authority-inventory]
current_target_historical: current
unique_content: [frozen Phase 202 corpus scope, conflict ledger, and Rust realization evidence]
proposed_disposition: retain-as-audit-input
verified_against:
  code:
    - crates/ash-core/src/core_ash_typecheck.rs
    - crates/ash-parser/src/lower.rs
    - crates/ash-typeck/src/lib.rs
    - crates/ash-interp/src/cps/mod.rs
    - crates/ash-engine/src/row_admission.rs
    - crates/ash-provenance/src/trace.rs
  tests:
    - crates/ash-core/tests/task_1814_core_cps_row_preservation.rs
    - crates/ash-parser/tests/task_1734_expanded_surface_lowering_gate.rs
    - crates/ash-typeck/tests/task_1814_row_cross_boundary_non_authority.rs
    - crates/ash-interp/tests/task_1858_1859_handler_provider_semantics.rs
    - crates/ash-engine/tests/task_1829_1830_1831_1832_1833_row_admission.rs
    - crates/ash-provenance/tests/runtime_trace_boundaries.rs
related:
  explains: [plan.202.formal-semantics-verification-programme]
  superseded_by: null
---

# TASK-1984: Corpus Authority and Conflict Inventory

## Scope and method

This audit freezes the Phase 202 inventory at repository revision `c9294828`, with the qualified
dirty paths recorded in [the scope manifest](TASK-1984-corpus-authority-scope.json). It classifies
every Markdown artifact beneath `docs/` and top-level `reference/`, including A5 plans, audits,
notes, and archival material; inclusion is not a claim that each artifact is productive or
normative. The productive entry roots are `docs/README.md`, the SPEC and NOTE indexes, and
`reference/README.md`. The manifest records `.git` and `target` exclusions with reasons.

The frozen `classification_overlay` contains an explicit A5, `unresolved`, noncanonical entry for
each of the 2,302 scoped Markdown paths. It supplies a unique inventory id and a unique
classification-only subject for audit retrieval; neither field promotes a semantic owner. The
manifest also records the two non-Markdown in-scope artifacts reached by documentation links: this
scope manifest and its generated inventory JSON. No other JSON, YAML, TOML, CSV, TSV, TXT, or XML
artifact is linked from the scoped Markdown corpus (the discovery result is therefore two, not an
assumed empty set).

The generated [machine inventory](TASK-1984-corpus-authority-inventory.json) is deliberately
conflicted at this stage. Its nonzero generator result is evidence that corpus authority is not
yet canonical, not a failure to suppress or work around. The final recorded run reports its exact
artifact and conflict counts: **2,304 artifacts and 152 conflicts** (exit status `1`). Every
remaining finding is `invalid_evidence_path` in the pre-existing corpus. Status conflict detection
now evaluates declared metadata rather than incidental prose, so neither this audit nor
`PLAN-INDEX.md` has a false contradictory-status finding. The final result has **zero**
`missing_overlay_classification` and **zero** `unclassified_artifact` findings.

## Expected canonical subjects

PLAN-202 names eight subjects for the eventual compact canonical core. Each is explicitly
`unresolved` in the scope manifest until TASK-1986 has made a rule-level reconciliation decision:

1. authority, terminology, semantic domains, and notation;
2. lexical and surface grammar;
3. surface typing and effects/rows;
4. Core/CPS syntax, well-formedness, and typing;
5. surface-to-Core lowering;
6. Core/CPS operational semantics;
7. observable/runtime projection and nondeterminism boundaries; and
8. implementation-conformance and executable corpus.

No owner is inferred from directory, SPEC display number, document age, or live Rust behavior.

## Known conflict ledger

| ID | Involved paths and competing claims | Evidence, disposition, and state |
|---|---|---|
| `conflict.docs-readme-spec-index` | `docs/spec/README.md` claims older workflow/tower material is active; `docs/spec/SPEC-INDEX.md` routes target work to SPEC-095b through SPEC-100. | Both files are linked structured evidence. **Unresolved:** TASK-1985 must model the claims and TASK-1986 must select rule owners without chronology. |
| `conflict.formalization-boundary` | `docs/reference/formalization-boundary.md` names workflow-first SPEC-004/SPEC-025 theorem subjects; `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md` names the target Core/CPS path. | Both files are linked structured evidence. **Unresolved:** TASK-1986 must decide the replacement/refinement boundary while retaining archival provenance. |
| `conflict.parser-to-core` | `docs/reference/parser-to-core-lowering-contract.md` specifies a workflow-first handoff; `crates/ash-parser/src/lower.rs` realizes expanded surface-to-Core lowering. | Both paths are structured evidence and the generated inventory attaches this ID to both the documentation artifact and the parser realization record. **Unresolved:** TASK-1986 must state total domain, unsupported residuals, and preservation obligations. |
| `conflict.phase-201-handoff` | PLAN-202's earlier text differs from the completion evidence in TASK-1971, TASK-1972, and PLAN-INDEX at `c9294828`. | All four files are linked structured evidence. **Resolved at `c9294828` only:** this qualifies TASK-1988's Phase 201 input and creates no Phase 202 semantic owner. |

The overload of SPEC families `038`, `095`, `096`, `097`, `098`, and `099` is also a stable-id
risk. It is not resolved by display-number chronology; TASK-1985 must assign unique semantic IDs
and validate owners before TASK-1986 promotes any document.

## Rust realization and test evidence

The following live paths are realization evidence only. They neither promote code to semantic
authority nor prove an A1/A2 document correct. They are retained so TASK-1988 can map the eventual
rules to current implementation and behavior-level tests.

| Area | Live realization and symbol | Executed focused test evidence |
|---|---|---|
| Core rows / Core typing | `core_ash_typecheck.rs` — `normalize_core_row` | `cargo test -p ash-core --test task_1814_core_cps_row_preservation` — passed (1) |
| Surface-to-Core lowering | `lower.rs` — `lower_expanded_surface_module` | `cargo test -p ash-parser --test task_1734_expanded_surface_lowering_gate` — passed (4) |
| Surface type/effect boundary | `lib.rs` — `type_check_program` | `cargo test -p ash-typeck --test task_1814_row_cross_boundary_non_authority` — passed (3) |
| Core/CPS evaluation | `cps/mod.rs` — `eval_raise` | `cargo test -p ash-interp --test task_1858_1859_handler_provider_semantics` — passed (5) |
| Runtime admission/projection | `row_admission.rs` — `RowAdmissionEnvironment` | `cargo test -p ash-engine --test task_1829_1830_1831_1832_1833_row_admission` — passed (12) |
| Trace/observable evidence | `trace.rs` — `ApplicationTraceSession` | `cargo test -p ash-provenance --test runtime_trace_boundaries` — passed (2) |

## Inputs to follow-on tasks

- **TASK-1985:** consume the frozen JSON scope, preserve the four ledger IDs, introduce stable
  semantic IDs and ownership validation, and make the generator's explicit missing,
  duplicate, and unclassified findings machine-actionable.
- **TASK-1986:** reconcile the eight unresolved subjects rule-by-rule, including the
  workflow-first formalization and parser handoff conflicts; do not use current code or chronology
  as a shortcut to promotion.
- **TASK-1988:** use the six realization/test records and the `c9294828` Phase 201 handoff as the
  boundary for an evidence-led implementation/deprecation audit.

## Boundary

This is an **audit-only** task. It does **not** promote, move, archive, or delete any documentation,
and it does not declare a canonical semantic owner. Those decisions remain explicitly deferred to
TASK-1985 and TASK-1986.
