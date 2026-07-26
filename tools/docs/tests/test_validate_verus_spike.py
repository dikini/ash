#!/usr/bin/env python3
"""RED contracts for TASK-1991's isolated Verus spike gate.

These tests do not install or invoke Verus.  They define the fail-closed
metadata boundary which a future pinned runner must satisfy: both an accepted
and rejected source fixture are recorded, the runner is isolated from Cargo,
and the generated TCB record identifies every trusted component and its
manifest fingerprint.
"""
from __future__ import annotations

import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
TOOL = REPOSITORY_ROOT / "tools/docs/validate_verus_spike.py"
FIXTURES = Path(__file__).with_name("fixtures") / "verus_spike"
REPORT_SCHEMA = "verus-spike-validation-report/v1"


class VerusSpikeContractTests(unittest.TestCase):
    """Specify metadata validation without coupling ordinary Cargo to Verus."""

    def run_spike(self, root: Path, manifest: Path | None = None) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        self.assertTrue(TOOL.exists(), f"missing TASK-1991 Verus spike validator: {TOOL}")
        command = ["python3", str(TOOL), "--root", str(root), "--format", "json"]
        if manifest is not None:
            command.extend(["--manifest", str(manifest)])
        result = subprocess.run(command, check=False, capture_output=True, text=True)
        try:
            report = json.loads(result.stdout)
        except json.JSONDecodeError as error:
            self.fail(f"Verus spike validator must write JSON to stdout: {error}; stderr: {result.stderr}")
        return result, report

    def run_fixture(self, name: str) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        root = FIXTURES / name
        return self.run_spike(root, root / "verification/verus/verus-spike-manifest.json")

    def run_mutation(self, mutate: object) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory) / "fixture"
            shutil.copytree(FIXTURES / "valid", root)
            manifest = root / "verification/verus/verus-spike-manifest.json"
            payload = json.loads(manifest.read_text(encoding="utf-8"))
            assert callable(mutate)
            mutate(payload)
            manifest.write_text(json.dumps(payload), encoding="utf-8")
            return self.run_spike(root, manifest)

    def assert_rejected(self, mutate: object, kind: str) -> None:
        result, report = self.run_mutation(mutate)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report.get("schema"), REPORT_SCHEMA)
        errors = report.get("errors")
        self.assertIsInstance(errors, list)
        self.assertTrue(any(error.get("kind") == kind for error in errors if isinstance(error, dict)), errors)

    def test_minimal_pass_and_fail_fixtures_are_present_and_explicitly_opposed(self) -> None:
        """The repository carries real inert Verus source witnesses for both outcomes."""
        pass_fixture = REPOSITORY_ROOT / "verification/verus/fixtures/pass.rs"
        fail_fixture = REPOSITORY_ROOT / "verification/verus/fixtures/fail.rs"
        self.assertIn("ensures 1int == 1int", pass_fixture.read_text(encoding="utf-8"))
        self.assertIn("ensures 1int == 2int", fail_fixture.read_text(encoding="utf-8"))

    def test_complete_pinned_spike_contract_is_accepted_without_running_cargo(self) -> None:
        """A future runner accepts an isolated manifest, both outcomes, and a complete TCB report."""
        result, report = self.run_fixture("valid")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report, {"schema": REPORT_SCHEMA, "errors": []})

    def test_missing_pinned_manifest_is_a_json_failure_not_a_tool_crash(self) -> None:
        """Absent configuration must block proof expansion with a stable diagnostic."""
        result, report = self.run_spike(FIXTURES / "missing_manifest")
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report.get("schema"), REPORT_SCHEMA)
        self.assertTrue(any(error.get("kind") == "missing_pinned_manifest" for error in report["errors"]))

    def test_missing_runner_and_tcb_report_are_each_rejected(self) -> None:
        """Metadata alone cannot claim reproducible verification evidence."""
        self.assert_rejected(
            lambda payload: payload["runner"].__setitem__("path", "verification/verus/missing-runner.sh"),
            "missing_runner",
        )
        self.assert_rejected(
            lambda payload: payload.__setitem__("tcb_report", "verification/verus/missing-tcb-report.json"),
            "missing_tcb_report",
        )

    def test_manifest_and_tcb_fingerprints_must_agree(self) -> None:
        """A TCB result is bound to the exact pinned manifest it reports on."""
        def mutate(payload: dict[str, object]) -> None:
            payload["manifest_fingerprint"] = "sha256:inconsistent-manifest"

        self.assert_rejected(mutate, "manifest_fingerprint_mismatch")

    def test_negative_fixture_must_be_recorded_as_rejected(self) -> None:
        """A runner cannot silently skip a negative proof or invert its result."""
        def mutate(payload: dict[str, object]) -> None:
            fixtures = payload["fixtures"]
            assert isinstance(fixtures, list)
            fixtures[1]["expected_outcome"] = "verified"

        self.assert_rejected(mutate, "fixture_outcome_mismatch")

    def test_tcb_report_enumerates_logical_assumptions_and_trusted_tooling(self) -> None:
        """Every assumption category and tooling component stays machine-addressable."""
        def mutate(payload: dict[str, object]) -> None:
            payload["tcb_required_categories"] = ["assume"]

        self.assert_rejected(mutate, "incomplete_tcb_categories")


if __name__ == "__main__":
    unittest.main()
