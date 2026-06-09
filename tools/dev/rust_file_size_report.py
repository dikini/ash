#!/usr/bin/env python3
"""Report Rust source file sizes by Cargo workspace package.

The report is intentionally dependency-free and derives package ownership from
`cargo metadata` so later refactor tasks can compare measurements reliably.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

LINE_LIMIT = 500
BYTE_LIMIT = 10 * 1024
IGNORED_DIRS = {".git", "target", ".worktrees"}
BASELINE_LIMITS = {
    "largest_file_lines": 20_935,
    "largest_file_bytes": 826_435,
}


@dataclass(frozen=True)
class FileStat:
    path: Path
    relative_path: str
    lines: int
    bytes: int


def cargo_metadata(cwd: Path) -> dict:
    completed = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=cwd,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return json.loads(completed.stdout)


def is_ignored(path: Path, root: Path) -> bool:
    try:
        relative = path.resolve().relative_to(root)
    except ValueError:
        return False
    return any(part in IGNORED_DIRS for part in relative.parts)


def is_test_path(path: Path, workspace: Path) -> bool:
    try:
        relative = path.resolve().relative_to(workspace)
    except ValueError:
        return False
    parts = relative.parts
    return "tests" in parts or any(
        part.endswith("_tests.rs") or part == "test.rs" for part in parts
    )


def iter_rust_files(
    package_root: Path, workspace: Path, tests_only: bool = False
) -> Iterable[Path]:
    for path in sorted(package_root.rglob("*.rs")):
        if is_ignored(path, workspace):
            continue
        if tests_only and not is_test_path(path, workspace):
            continue
        if path.is_file():
            yield path


def file_stat(path: Path, workspace: Path) -> FileStat:
    data = path.read_bytes()
    line_count = data.count(b"\n") + (1 if data and not data.endswith(b"\n") else 0)
    return FileStat(
        path=path,
        relative_path=path.resolve().relative_to(workspace).as_posix(),
        lines=line_count,
        bytes=len(data),
    )


def package_report(package: dict, workspace: Path, tests_only: bool = False) -> dict:
    manifest = Path(package["manifest_path"]).resolve()
    package_root = manifest.parent
    files = [file_stat(path, workspace) for path in iter_rust_files(package_root, workspace, tests_only)]
    files.sort(key=lambda stat: stat.relative_path)
    largest_by_lines = max(files, key=lambda stat: (stat.lines, stat.bytes, stat.relative_path), default=None)
    largest_by_bytes = max(files, key=lambda stat: (stat.bytes, stat.lines, stat.relative_path), default=None)
    return {
        "package": package["name"],
        "manifest_path": manifest.relative_to(workspace).as_posix(),
        "total_rs_files": len(files),
        "files_above_500_lines": sum(1 for stat in files if stat.lines > LINE_LIMIT),
        "files_above_10kb": sum(1 for stat in files if stat.bytes > BYTE_LIMIT),
        "largest_file_by_lines": stat_to_json(largest_by_lines),
        "largest_file_by_bytes": stat_to_json(largest_by_bytes),
        "files": [stat_to_json(stat) for stat in files],
    }


def stat_to_json(stat: FileStat | None) -> dict | None:
    if stat is None:
        return None
    return {"path": stat.relative_path, "lines": stat.lines, "bytes": stat.bytes}


def build_report(cwd: Path, tests_only: bool = False) -> dict:
    metadata = cargo_metadata(cwd)
    workspace = Path(metadata["workspace_root"]).resolve()
    workspace_members = set(metadata["workspace_members"])
    packages = [pkg for pkg in metadata["packages"] if pkg["id"] in workspace_members]
    packages.sort(key=lambda pkg: pkg["name"])
    crate_reports = [package_report(pkg, workspace, tests_only) for pkg in packages]
    all_files = [file for crate in crate_reports for file in crate["files"]]
    top_by_lines = sorted(all_files, key=lambda item: (-item["lines"], -item["bytes"], item["path"]))[:20]
    return {
        "workspace_root": workspace.as_posix(),
        "line_threshold": LINE_LIMIT,
        "byte_threshold": BYTE_LIMIT,
        "ignored_directories": sorted(IGNORED_DIRS),
        "tests_only": tests_only,
        "summary": {
            "workspace_crates": len(crate_reports),
            "rust_files": len(all_files),
            "files_above_500_lines": sum(1 for file in all_files if file["lines"] > LINE_LIMIT),
            "files_above_10kb": sum(1 for file in all_files if file["bytes"] > BYTE_LIMIT),
        },
        "crates": crate_reports,
        "top_files_by_lines": top_by_lines,
    }


def format_bytes(size: int | None) -> str:
    if size is None:
        return "-"
    if size >= 1024 * 1024:
        return f"{size / (1024 * 1024):.1f}MB"
    if size >= 1024:
        return f"{size / 1024:.1f}KB"
    return f"{size}B"


def format_int(value: int | None) -> str:
    if value is None:
        return "-"
    return f"{value:,}"


def format_file_cell(file: dict | None, metric: str) -> str:
    if file is None:
        return "-"
    value = format_int(file[metric]) if metric == "lines" else format_bytes(file[metric])
    return f"`{file['path']}` ({value})"


def markdown(report: dict) -> str:
    summary = report["summary"]
    lines = [
        "# Rust File Size Report",
        "",
        f"- Workspace crates scanned: {summary['workspace_crates']}",
        f"- Rust files scanned: {summary['rust_files']}",
        f"- Rust files larger than {report['line_threshold']} lines: {summary['files_above_500_lines']}",
        f"- Rust files larger than {format_bytes(report['byte_threshold'])}: {summary['files_above_10kb']}",
        f"- Ignored directories: {', '.join(f'`{name}/`' for name in report['ignored_directories'])}",
        f"- Tests-only filter: {'enabled' if report['tests_only'] else 'disabled'}",
        "",
        "## Per-crate summary",
        "",
        "| Crate | .rs files | >500 lines | >10KB | Largest by lines | Largest by bytes |",
        "|---|---:|---:|---:|---|---|",
    ]
    for crate in report["crates"]:
        lines.append(
            "| {package} | {total} | {over_lines} | {over_bytes} | {largest_lines} | {largest_bytes} |".format(
                package=f"`{crate['package']}`",
                total=crate["total_rs_files"],
                over_lines=crate["files_above_500_lines"],
                over_bytes=crate["files_above_10kb"],
                largest_lines=format_file_cell(crate["largest_file_by_lines"], "lines"),
                largest_bytes=format_file_cell(crate["largest_file_by_bytes"], "bytes"),
            )
        )
    lines.extend(
        [
            "",
            "## Top 20 files by line count",
            "",
            "| Rank | File | Lines | Size |",
            "|---:|---|---:|---:|",
        ]
    )
    for index, file in enumerate(report["top_files_by_lines"], start=1):
        lines.append(
            f"| {index} | `{file['path']}` | {format_int(file['lines'])} | {format_bytes(file['bytes'])} |"
        )
    lines.append("")
    return "\n".join(lines)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    output = parser.add_mutually_exclusive_group()
    output.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    output.add_argument("--markdown", action="store_true", help="emit Markdown")
    parser.add_argument(
        "--tests-only",
        action="store_true",
        help="only include Rust files under test directories or named *_tests.rs/test.rs",
    )
    parser.add_argument(
        "--fail-on-regression",
        action="store_true",
        help="exit non-zero if the largest Rust file exceeds the Phase 137 baseline",
    )
    return parser.parse_args(argv)


def regression_errors(report: dict) -> list[str]:
    all_files = [file for crate in report["crates"] for file in crate["files"]]
    largest_by_lines = max(
        all_files, key=lambda file: (file["lines"], file["bytes"], file["path"]), default=None
    )
    largest_by_bytes = max(
        all_files, key=lambda file: (file["bytes"], file["lines"], file["path"]), default=None
    )
    checks = [
        (
            "largest_file_lines",
            largest_by_lines["lines"] if largest_by_lines else 0,
            BASELINE_LIMITS["largest_file_lines"],
        ),
        (
            "largest_file_bytes",
            largest_by_bytes["bytes"] if largest_by_bytes else 0,
            BASELINE_LIMITS["largest_file_bytes"],
        ),
    ]
    return [
        f"{name}: {actual} exceeds baseline {limit}"
        for name, actual, limit in checks
        if actual > limit
    ]


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    report = build_report(Path.cwd(), tests_only=args.tests_only)
    if args.fail_on_regression and not args.tests_only:
        errors = regression_errors(report)
        if errors:
            for error in errors:
                print(f"rust-file-size regression: {error}", file=sys.stderr)
            return 1
        if not args.json and not args.markdown:
            print("rust-file-size: no Phase 137 baseline regressions")
            return 0
    if args.json:
        json.dump(report, sys.stdout, indent=2, sort_keys=True)
        sys.stdout.write("\n")
    else:
        sys.stdout.write(markdown(report))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
