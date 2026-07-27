#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

MANIFEST_PATH="docs/plan/semantic-task-records.json"
VALIDATOR_PATH="tools/docs/validate_semantic_task_records.py"

usage() {
  cat <<'USAGE'
Usage:
  scripts/check-semantic-task-gate.sh [--staged]
  scripts/check-semantic-task-gate.sh --all

--staged (the default) validates and runs only semantic task records selected
by the staged index. --all validates and runs every active task record.
USAGE
}

mode="staged"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --staged)
      mode="staged"
      shift
      ;;
    --all)
      mode="all"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "semantic-task-gate: unknown argument '$1'" >&2
      usage >&2
      exit 2
      ;;
  esac
done

tmp="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT

snapshot="$tmp/index"
mkdir -p "$snapshot"

# Both validation and the focused command run against the same staged snapshot.
# This closes the usual pre-commit gap where a later unstaged edit changes what
# the gate validates or tests.
git checkout-index --all --prefix="$snapshot/"
git diff --cached --name-only -z >"$tmp/staged-paths"

snapshot_manifest="$snapshot/$MANIFEST_PATH"
snapshot_validator="$snapshot/$VALIDATOR_PATH"

if [[ "$mode" == "all" || -f "$snapshot_manifest" ]]; then
  if [[ ! -f "$snapshot_manifest" ]]; then
    echo "semantic-task-gate: missing staged manifest $MANIFEST_PATH" >&2
    exit 1
  fi
  if [[ ! -f "$snapshot_validator" ]]; then
    echo "semantic-task-gate: missing staged validator $VALIDATOR_PATH" >&2
    exit 2
  fi
  python3 "$snapshot_validator" --root "$snapshot" --manifest "$MANIFEST_PATH"
fi

python3 - "$mode" "$snapshot" "$tmp/staged-paths" "$MANIFEST_PATH" "$ROOT" <<'PY'
"""Select declared semantic task commands and execute them without a shell."""
from __future__ import annotations

import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys


mode, snapshot_text, paths_text, manifest_text, checkout_root_text = sys.argv[1:]
snapshot = Path(snapshot_text)
checkout_root = Path(checkout_root_text)
command_environment = os.environ.copy()
command_environment["CARGO_TARGET_DIR"] = str(checkout_root / "target")
staged_paths = [
    path.decode("utf-8", "surrogateescape")
    for path in Path(paths_text).read_bytes().split(b"\0")
    if path
]

task_path = re.compile(r"^docs/plan/tasks/(TASK-[0-9]+)-[^/]+\.md$")
semantic_source = re.compile(
    r"^crates/ash-(?:core|parser|typeck|engine|interp|cli)/(?:src|tests)/.+\.rs$"
)
NON_SEMANTIC_WORKFLOW_CLASSIFICATION = (
    "**Semantic task classification:** non-semantic-workflow-enforcement"
)
TASK_STATUS = re.compile(r"^\*\*Status:\*\*\s*(.+?)\s*$")


def fail(message: str) -> None:
    print(f"semantic-task-gate: {message}", file=sys.stderr)
    raise SystemExit(1)


def is_non_semantic_workflow_task(path: str) -> bool:
    """Accept only the exact staged marker for workflow-enforcement tasks."""
    try:
        text = (snapshot / path).read_text(encoding="utf-8")
    except OSError:
        return False
    return NON_SEMANTIC_WORKFLOW_CLASSIFICATION in text.splitlines()


def first_task_status(path: str) -> str | None:
    """Read the authoritative first task-status metadata line from the snapshot."""
    try:
        text = (snapshot / path).read_text(encoding="utf-8")
    except OSError:
        return None
    for line in text.splitlines():
        if match := TASK_STATUS.fullmatch(line):
            return match.group(1)
    return None


def is_planned_task(path: str) -> bool:
    """Keep an unactivated planned task out of a docs-only active evidence gate."""
    return first_task_status(path) == "Planned"


task_documents = [
    (path, match.group(1))
    for path in staged_paths
    if (match := task_path.fullmatch(path)) is not None
]

semantic_paths = [path for path in staged_paths if semantic_source.fullmatch(path)]
if mode == "staged" and not semantic_paths and not task_documents:
    print("semantic-task-gate: no staged semantic task selected")
    raise SystemExit(0)

manifest_path = snapshot / manifest_text
if not manifest_path.is_file():
    fail(f"missing staged manifest {manifest_text}")
try:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
except (OSError, json.JSONDecodeError) as error:
    fail(f"could not read staged manifest {manifest_text}: {error}")

records = manifest.get("records")
if not isinstance(records, list):
    fail("validated manifest did not contain records")
records_by_task = {
    record.get("task"): record
    for record in records
    if isinstance(record, dict) and isinstance(record.get("task"), str)
}

# A record is semantic authority even if a staged task document carries the
# workflow-enforcement marker. Only an otherwise unregistered workflow task or
# explicitly planned docs-only task may stay outside the active manifest; both
# markers are read solely from the staged snapshot above. Any staged semantic
# Rust change makes every other staged task document an active selection.
selected_tasks: list[str] = []
for path, task in task_documents:
    if task not in records_by_task and (
        is_non_semantic_workflow_task(path)
        or (not semantic_paths and is_planned_task(path))
    ):
        continue
    if task not in selected_tasks:
        selected_tasks.append(task)

if mode == "all":
    selected_records = records
else:
    if semantic_paths and not selected_tasks:
        fail(
            "staged semantic Rust changes require a matching staged "
            "docs/plan/tasks/TASK-*.md record"
        )
    required_paths = {
        manifest_text,
        "docs/plan/SEMANTIC-RULE-COVERAGE.md",
        "docs/spec/SEMANTIC-TRACEABILITY.json",
        "CHANGELOG.md",
    }
    if semantic_paths:
        missing_paths = sorted(required_paths - set(staged_paths))
        if missing_paths:
            fail(
                "staged semantic Rust changes require matching staged evidence: "
                + ", ".join(missing_paths)
            )
    selected_records = []
    for task in selected_tasks:
        record = records_by_task.get(task)
        if record is None:
            fail(f"staged task {task} has no active semantic-task record")
        declared_task_file = record.get("task_file")
        if not isinstance(declared_task_file, str) or declared_task_file not in staged_paths:
            fail(f"staged task {task} must stage its declared task_file")
        selected_records.append(record)

for record in selected_records:
    if not isinstance(record, dict):
        fail("validated manifest contained a non-object record")
    task = record.get("task")
    commands = record.get("verification")
    if not isinstance(task, str) or not isinstance(commands, list):
        fail("validated manifest contained an incomplete record")
    for command in commands:
        if not isinstance(command, str):
            fail(f"task {task} declared a non-string verification command")
        # The validator has already checked this controlled grammar.  Parse
        # again and pass the argument vector directly to subprocess so no
        # command text is ever evaluated by a shell.
        try:
            arguments = shlex.split(command, posix=True)
        except ValueError as error:
            fail(f"task {task} declared an unparsable verification command: {error}")
        print(f"semantic-task-gate: running {task}: {command}")
        completed = subprocess.run(
            arguments,
            cwd=snapshot,
            env=command_environment,
            check=False,
        )
        if completed.returncode != 0:
            raise SystemExit(completed.returncode)
PY

echo "semantic-task-gate: OK"
