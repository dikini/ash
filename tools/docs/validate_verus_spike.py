#!/usr/bin/env python3
"""Fail-closed metadata gate for the isolated TASK-1991 Verus spike.

This deliberately validates evidence *about* a future Verus invocation.  It
does not install, download, or execute Verus (and, in particular, never calls
Cargo).  Keeping that boundary explicit lets ordinary Ash development remain
independent of the experimental verifier toolchain.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
from typing import Any


REPORT_SCHEMA = "verus-spike-validation-report/v1"
MANIFEST_SCHEMA = "verus-spike-manifest/v1"
TCB_SCHEMA = "verus-tcb-report/v1"
FINGERPRINT = re.compile(r"^sha256:[A-Za-z0-9][A-Za-z0-9._-]*$")

TOOLCHAIN_COMPONENTS = {"verus", "wrapper", "rust", "z3", "vstd"}
TCB_TOOLING = {"verus", "wrapper", "rustc", "z3", "vstd"}
TCB_CATEGORIES = {
    "assume", "axiom", "external_body", "external_specification",
    "external_item", "external_trait_impl",
}
FIXTURE_OUTCOMES = {
    "verification/verus/fixtures/pass.rs": "verified",
    "verification/verus/fixtures/fail.rs": "rejected",
}


def issue(kind: str, message: str, **details: object) -> dict[str, object]:
    return {"kind": kind, "message": message, **details}


def nonempty(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip())


def relative_file(root: Path, value: object) -> Path | None:
    """Resolve a repository-relative file without accepting traversal escapes."""
    if not nonempty(value):
        return None
    candidate = Path(value)
    if candidate.is_absolute() or ".." in candidate.parts:
        return None
    resolved_root = root.resolve()
    resolved = (root / candidate).resolve()
    try:
        resolved.relative_to(resolved_root)
    except ValueError:
        return None
    return resolved


def load_json(path: Path) -> tuple[object | None, str | None]:
    try:
        return json.loads(path.read_text(encoding="utf-8")), None
    except (OSError, json.JSONDecodeError) as exc:
        return None, str(exc)


def validate_runner(root: Path, manifest: dict[str, Any], errors: list[dict[str, object]]) -> None:
    runner = manifest.get("runner")
    if not isinstance(runner, dict):
        errors.append(issue("missing_runner", "manifest requires an isolated runner object"))
        return
    path_value = runner.get("path")
    runner_path = relative_file(root, path_value)
    if runner_path is None or not runner_path.is_file():
        errors.append(issue("missing_runner", "isolated runner path is absent or unsafe", path=path_value))
    command = runner.get("command")
    if not isinstance(command, list) or not command or not all(nonempty(value) for value in command):
        errors.append(issue("invalid_runner", "runner command must be a non-empty string array"))
    else:
        command_tokens = [value.lower() for value in command if isinstance(value, str)]
        if any("cargo" in token for token in command_tokens):
            errors.append(issue("runner_not_isolated", "runner command must not invoke Cargo", command=command))
    if runner.get("isolated_from_cargo") is not True:
        errors.append(issue("runner_not_isolated", "runner must explicitly be isolated from Cargo"))
    if isinstance(path_value, str) and ("cargo" in path_value.lower() or "target/" in path_value.lower()):
        errors.append(issue("runner_not_isolated", "runner path must not use Cargo build output", path=path_value))


def validate_fixtures(root: Path, manifest: dict[str, Any], errors: list[dict[str, object]]) -> None:
    fixtures = manifest.get("fixtures")
    if not isinstance(fixtures, list):
        errors.append(issue("invalid_fixtures", "fixtures must be a list containing pass and fail witnesses"))
        return
    seen: dict[str, object] = {}
    malformed = False
    for index, fixture in enumerate(fixtures):
        if not isinstance(fixture, dict):
            malformed = True
            continue
        path, outcome = fixture.get("path"), fixture.get("expected_outcome")
        if not nonempty(path) or not nonempty(outcome):
            malformed = True
            continue
        source = relative_file(root, path)
        if source is None or not source.is_file():
            errors.append(issue("missing_fixture", "fixture path is absent or unsafe", index=index, path=path))
        else:
            try:
                text = source.read_text(encoding="utf-8")
            except OSError as exc:
                errors.append(issue("missing_fixture", "fixture cannot be read", index=index, path=path, detail=str(exc)))
            else:
                # These inert source witnesses are intentionally cheap to inspect;
                # verifier execution belongs to the future pinned runner.
                if path.endswith("pass.rs") and "ensures 1int == 1int" not in text:
                    errors.append(issue("invalid_fixture", "positive fixture is not the required pass witness", path=path))
                if path.endswith("fail.rs") and "ensures 1int == 2int" not in text:
                    errors.append(issue("invalid_fixture", "negative fixture is not the required rejection witness", path=path))
        if isinstance(path, str):
            if path in seen:
                malformed = True
            seen[path] = outcome
    if malformed:
        errors.append(issue("invalid_fixtures", "each fixture must have one path and expected outcome"))
    if any(seen.get(path) != outcome for path, outcome in FIXTURE_OUTCOMES.items()):
        errors.append(issue("fixture_outcome_mismatch", "pass and fail witnesses must be explicitly opposed", fixtures=seen))


def validate_tcb(root: Path, manifest: dict[str, Any], errors: list[dict[str, object]]) -> None:
    report_value = manifest.get("tcb_report")
    report_path = relative_file(root, report_value)
    if report_path is None or not report_path.is_file():
        errors.append(issue("missing_tcb_report", "machine-readable TCB report is absent or unsafe", path=report_value))
        return
    report, read_error = load_json(report_path)
    if read_error is not None or not isinstance(report, dict):
        errors.append(issue("invalid_tcb_report", "TCB report must be readable JSON object", detail=read_error))
        return
    if report.get("schema") != TCB_SCHEMA:
        errors.append(issue("invalid_tcb_report", f"TCB report schema must be {TCB_SCHEMA}"))
    manifest_fp = manifest.get("manifest_fingerprint")
    if not isinstance(manifest_fp, str) or FINGERPRINT.fullmatch(manifest_fp) is None:
        errors.append(issue("invalid_manifest_fingerprint", "manifest fingerprint must be sha256-addressable"))
    if report.get("manifest_fingerprint") != manifest_fp:
        errors.append(issue("manifest_fingerprint_mismatch", "TCB report must name the exact manifest fingerprint"))
    tool_fingerprints = report.get("tool_fingerprints")
    if not isinstance(tool_fingerprints, dict) or set(tool_fingerprints) != TCB_TOOLING or any(
        not isinstance(tool_fingerprints.get(name), str) or FINGERPRINT.fullmatch(tool_fingerprints[name]) is None
        for name in TCB_TOOLING
    ):
        errors.append(issue("incomplete_tool_fingerprints", "TCB report must fingerprint every trusted tooling component"))
    trusted = report.get("trusted_tooling")
    if not isinstance(trusted, list) or set(trusted) != TCB_TOOLING or not all(isinstance(item, str) for item in trusted):
        errors.append(issue("incomplete_trusted_tooling", "TCB report must enumerate exactly the trusted tooling components"))
    required = manifest.get("tcb_required_categories")
    if not isinstance(required, list) or set(required) != TCB_CATEGORIES or len(required) != len(TCB_CATEGORIES):
        errors.append(issue("incomplete_tcb_categories", "manifest must require every logical-assumption category"))
    assumptions = report.get("logical_assumptions")
    if not isinstance(assumptions, dict) or set(assumptions) != TCB_CATEGORIES or any(
        not isinstance(assumptions.get(name), list) or not all(isinstance(item, str) for item in assumptions[name])
        for name in TCB_CATEGORIES
    ):
        errors.append(issue("incomplete_tcb_categories", "TCB report must enumerate every logical-assumption category"))
    for name in ("unsupported_features", "production_adapters"):
        value = report.get(name)
        if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
            errors.append(issue("invalid_tcb_report", f"TCB report {name} must be a string list"))
    if report.get("outcome") != "verified":
        errors.append(issue("invalid_tcb_outcome", "TCB report must record a verified isolated fixture run", outcome=report.get("outcome")))


def validate(root: Path, manifest_path: Path) -> list[dict[str, object]]:
    if not manifest_path.is_file():
        return [issue("missing_pinned_manifest", "pinned Verus spike manifest is missing", path=str(manifest_path))]
    manifest, read_error = load_json(manifest_path)
    if read_error is not None or not isinstance(manifest, dict):
        return [issue("invalid_pinned_manifest", "pinned manifest must be readable JSON object", detail=read_error)]
    errors: list[dict[str, object]] = []
    if manifest.get("schema") != MANIFEST_SCHEMA:
        errors.append(issue("invalid_pinned_manifest", f"manifest schema must be {MANIFEST_SCHEMA}"))
    toolchain = manifest.get("toolchain")
    if not isinstance(toolchain, dict) or set(toolchain) != TOOLCHAIN_COMPONENTS or any(not nonempty(toolchain.get(name)) for name in TOOLCHAIN_COMPONENTS):
        errors.append(issue("incomplete_toolchain_pin", "manifest must pin Verus, wrapper, Rust, Z3, and vstd"))
    validate_runner(root, manifest, errors)
    validate_fixtures(root, manifest, errors)
    validate_tcb(root, manifest, errors)
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--format", choices=("json",), default="json")
    args = parser.parse_args()
    root = args.root.resolve()
    manifest = args.manifest if args.manifest is not None else root / "verification/verus/verus-spike-manifest.json"
    errors = validate(root, manifest)
    print(json.dumps({"schema": REPORT_SCHEMA, "errors": errors}, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
