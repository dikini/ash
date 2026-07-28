#!/usr/bin/env python3
"""Frozen documentation contract for TASK-2035's exact Engine-client routes."""
from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
TASK = REPOSITORY_ROOT / "docs/plan/tasks/TASK-2035-canonical-client-test-contracts.md"
AUDIT = REPOSITORY_ROOT / "docs/plan/audits/AUDIT-204-direct-ast-retirement.json"
SPEC_077 = (
    REPOSITORY_ROOT
    / "docs/spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md"
)
SPEC_026 = REPOSITORY_ROOT / "docs/spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md"

EXACT_SOURCES = (
    (
        "TASK-2035-SYNTH-WRAPPER-001",
        "fn contract_target_zero() -> Int { 0 }\n"
        "fn main() -> Bool { contract_target_zero() == 0 }\n",
        "71990ce4a503c89efb95340a6d7c6674a036858b8e337f8b9bc4337839ebe390",
    ),
    (
        "TASK-2035-REPL-ROUTE-001",
        "fn main() -> Int { 42 }\n",
        "ed4088d136e54744d258b170222ad3b2a064feda91b78b0a248f2ccfb9b7684c",
    ),
    (
        "TASK-2035-REPL-ROUTE-002",
        "fn main() -> Bool { 1 == 1 }\n",
        "697ab016d7ae6b9ab7088d17713e0e57d91965b911fc02d1ae1e0da54fa77811",
    ),
    (
        "TASK-2035-SHARED-ROUTE-001",
        "fn main() -> Int { 42 }\n",
        "ed4088d136e54744d258b170222ad3b2a064feda91b78b0a248f2ccfb9b7684c",
    ),
)

DEFERRED_CASES = (
    (
        "AUDIT-204-DEFERRED-001",
        "test:contract_postcondition_without_executable_target_metadata",
        "deferred: contract metadata lacks executable postcondition target metadata",
    ),
    (
        "AUDIT-204-DEFERRED-002",
        "test:contract_postcondition_without_structured_oracle_metadata",
        "deferred: contract postcondition metadata is not executable",
    ),
    (
        "AUDIT-204-DEFERRED-003",
        "test:contract_postcondition_with_unsupported_target_kind_defers",
        "deferred: unsupported contract target kind runtime_callable",
    ),
    (
        "AUDIT-204-DEFERRED-004",
        "test:contract_postcondition_with_missing_setup_defers",
        "deferred: contract target execution setup is missing",
    ),
    (
        "AUDIT-204-DEFERRED-005",
        "test:contract_postcondition_explicit_finite_setup_defers",
        "deferred: explicit finite setup is not executable for pure target slice",
    ),
    (
        "AUDIT-204-DEFERRED-006",
        "test:contract_postcondition_unsupported_body_defers",
        "deferred: contract target body is not executable",
    ),
    (
        "AUDIT-204-DEFERRED-007",
        "test:contract_postcondition_missing_exact_input_defers",
        "deferred: contract postcondition oracle lacks exact valid input representatives",
    ),
)

SHARED_ROUTE_FIELDS = (
    "TASK-2035-SHARED-ROUTE-001",
    "task-2035-shared-int-42-v1",
    "fn main() -> Int { 42 }",
    "entry `main`",
    "inputs `[]`",
    "bindings `{}`",
    "run control `{ deadline: none, cancellation: none, host_profile: none }`",
    "CanonicalTerminalEnvelopeV1::returned(Value::Int(42))",
)

SYNTH_WRAPPER_CATALOGUE_START = (
    "The exact source-wrapper catalogue contains the following one row:"
)
SYNTH_WRAPPER_CATALOGUE_END = "The source-contract ID and source text are exact."
SYNTH_WRAPPER_CATALOGUE_ROW = (
    "`TASK-2035-SYNTH-WRAPPER-001`",
    "`AUDIT-204-TEST-EXEC-002`",
    "`fn contract_target_zero() -> Int { 0 }`<br>"
    "`fn main() -> Bool { contract_target_zero() == 0 }`",
    "source digest "
    "`sha256:71990ce4a503c89efb95340a6d7c6674a036858b8e337f8b9bc4337839ebe390`; "
    "callable `contract_target_zero`; literal input `[]`; postcondition in `main`; "
    "expected Engine terminal projection of `Bool(true)`.",
)

LEAN_BANNER = (
    "**Status:** Deferred to a separate project",
    "external:lean-reference-project",
    "no current Ash execution, conformance, proof, or runtime-refinement authority",
)
LEAN_DOCUMENTS = (
    REPOSITORY_ROOT / "lean_reference/README.md",
    REPOSITORY_ROOT / "lean_reference/docs/DifferentialTesting.md",
    REPOSITORY_ROOT / "docs/plan/LEAN_REFERENCE_SUMMARY.md",
    REPOSITORY_ROOT / "docs/plan/LEAN_IMPLEMENTATION_EFFORT.md",
    REPOSITORY_ROOT / "docs/design/LEAN_REFERENCE_INTERPRETER.md",
)


def normalize_markdown_layout(text: str) -> str:
    """Compare contract fields without treating table separators or line wraps as semantics."""
    return " ".join(text.replace("|", " ").split())


def synthesized_wrapper_catalogue_rows(spec_text: str) -> list[tuple[str, ...]]:
    """Return data rows from SPEC-077's source-wrapper table."""
    section = spec_text.split(SYNTH_WRAPPER_CATALOGUE_START, maxsplit=1)[1].split(
        SYNTH_WRAPPER_CATALOGUE_END, maxsplit=1
    )[0]
    table_rows = [
        line.strip()
        for line in section.splitlines()
        if line.startswith("|")
    ]
    data_rows = table_rows[2:]
    return [tuple(cell.strip() for cell in row.split("|")[1:-1]) for row in data_rows]


class Task2035ContractDocumentsTests(unittest.TestCase):
    """Keep source identities, deferrals, and Lean authority boundaries exact."""

    def test_exact_source_contract_ids_have_lf_sha256_digests(self) -> None:
        """Every selected source contract hashes its LF-terminated source."""
        task_text = TASK.read_text(encoding="utf-8")

        for source_id, source, digest in EXACT_SOURCES:
            with self.subTest(source_id=source_id):
                self.assertEqual(
                    hashlib.sha256(source.encode("utf-8")).hexdigest(), digest
                )
                self.assertIn(source_id, task_text)
                for line in source.splitlines():
                    self.assertIn(line, task_text)
                self.assertIn(f"sha256:{digest}", task_text)

    def test_deferred_audit_cases_are_exact_and_remain_in_contract_docs(self) -> None:
        """The seven source-wrapper gaps retain their named fail-closed results."""
        audit_payload = json.loads(AUDIT.read_text(encoding="utf-8"))
        audit_cases = [
            (entry["id"], entry["case_id"], entry["fail_closed_result"])
            for entry in audit_payload["entries"]
            if entry["disposition"] == "deferred"
        ]
        self.assertEqual(audit_cases, list(DEFERRED_CASES))

        for document in (TASK, SPEC_077):
            text = document.read_text(encoding="utf-8")
            for _, case_id, result in DEFERRED_CASES:
                with self.subTest(document=document.name, case_id=case_id):
                    self.assertIn(case_id, text)
                    self.assertIn(result, text)

    def test_shared_four_client_route_has_one_exact_request_and_observation(self) -> None:
        """The selected parity route fixes source, envelope, and run control for four clients."""
        for document in (TASK, SPEC_026):
            text = normalize_markdown_layout(document.read_text(encoding="utf-8"))
            for field in SHARED_ROUTE_FIELDS:
                with self.subTest(document=document.name, field=field):
                    self.assertIn(field, text)

    def test_spec_077_preserves_the_complete_target_domain(self) -> None:
        """The task catalogue may not narrow wrappers already authorized by SPEC-077."""
        spec_text = SPEC_077.read_text(encoding="utf-8")
        self.assertIn(
            "This selected implementation catalogue does not reject a wrapper already "
            "authorized by Requirements 1 through 7 merely because it is not selected here.",
            spec_text,
        )

    def test_spec_077_has_one_exact_synthesized_wrapper_row(self) -> None:
        """The wrapper table admits only the declared source and digest."""
        spec_text = SPEC_077.read_text(encoding="utf-8")
        self.assertEqual(
            synthesized_wrapper_catalogue_rows(spec_text),
            [SYNTH_WRAPPER_CATALOGUE_ROW],
        )

    def test_lean_documents_have_the_deferred_no_current_authority_banner(self) -> None:
        """Historical Lean material has no current Ash execution or proof authority."""
        for document in LEAN_DOCUMENTS:
            text = document.read_text(encoding="utf-8")
            for banner_line in LEAN_BANNER:
                with self.subTest(document=document.as_posix(), banner_line=banner_line):
                    self.assertIn(banner_line, text)


if __name__ == "__main__":
    unittest.main()
