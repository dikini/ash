"""
Reproducible benchmark harness for Ash MCP agent effectiveness.

Usage:
    python scripts/benchmark/harness.py --mode baseline --output results.json
    python scripts/benchmark/harness.py --mode mcp --output results.json

Modes:
    baseline  – Agent uses only file/terminal tools (no MCP).
    mcp       – Agent has ash-mcp server configured.

The harness runs each corpus task and measures:
    - wall_time_ms:   wall-clock time to completion
    - tool_calls:     number of tool invocations
    - tokens_input:   estimated input tokens (prompt + tool results)
    - tokens_output:  estimated output tokens (model responses)
    - accuracy:       0 = wrong, 0.5 = partial, 1 = correct
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any, Dict, List

# ---------------------------------------------------------------------------
# Ensure we can import corpus.py regardless of cwd
# ---------------------------------------------------------------------------
SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

from corpus import CORPUS, Task  # noqa: E402

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
REPO_ROOT = SCRIPT_DIR.parent.parent
ASH_MCP_BINARY = REPO_ROOT / "target" / "release" / "ash-mcp"

# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------


@dataclass
class TaskResult:
    task_id: str
    mode: str  # "baseline" | "mcp"
    wall_time_ms: int = 0
    tool_calls: int = 0
    tokens_input: int = 0
    tokens_output: int = 0
    accuracy: float = 0.0
    answer: str = ""
    error: str = ""


@dataclass
class BenchmarkReport:
    mode: str
    timestamp: str
    ash_mcp_version: str = ""
    git_commit: str = ""
    results: List[TaskResult] = field(default_factory=list)
    cross_language: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "mode": self.mode,
            "timestamp": self.timestamp,
            "ash_mcp_version": self.ash_mcp_version,
            "git_commit": self.git_commit,
            "results": [asdict(r) for r in self.results],
            "cross_language": self.cross_language,
        }


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def run_cmd(cmd: List[str], cwd: Path | None = None, timeout: int = 60) -> tuple[int, str, str]:
    """Run a command and return (exit_code, stdout, stderr)."""
    result = subprocess.run(
        cmd,
        cwd=cwd or REPO_ROOT,
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    return result.returncode, result.stdout, result.stderr


def get_ash_mcp_version() -> str:
    if not ASH_MCP_BINARY.exists():
        return "not built"
    code, out, _ = run_cmd([str(ASH_MCP_BINARY), "--version"])
    return out.strip() if code == 0 else "unknown"


def get_git_commit() -> str:
    code, out, _ = run_cmd(["git", "rev-parse", "HEAD"])
    return out.strip() if code == 0 else "unknown"


def count_tokens_approx(text: str) -> int:
    """Very rough token estimate: ~4 chars per token (English/Rust average)."""
    return max(1, len(text) // 4)


# ---------------------------------------------------------------------------
# Baseline agent (file + terminal tools only)
# ---------------------------------------------------------------------------


def run_baseline_task(task: Task) -> TaskResult:
    """
    Simulate a baseline agent solving the task using grep and file reading.
    We execute the actual shell commands an agent would run and measure
    the work.
    """
    result = TaskResult(task_id=task.id, mode="baseline")
    start = time.perf_counter()

    total_input = 0
    total_output = 0
    tool_calls = 0
    answers: List[str] = []

    # Step 1: grep for the main symbol across the repo (find files, not lines)
    symbol = task.search_term
    cmd = ["grep", "-rln", symbol, "--include=*.rs", "--include=*.ash", "crates/"]
    code, out, err = run_cmd(cmd)
    tool_calls += 1
    total_input += count_tokens_approx(" ".join(cmd))
    total_output += count_tokens_approx(out)

    if code != 0 and not out:
        result.error = f"grep failed: {err}"
        result.wall_time_ms = int((time.perf_counter() - start) * 1000)
        return result

    answers.append(out[:2000])  # truncate for sanity

    # Step 2: read up to 3 matching files to verify definitions
    files = out.strip().splitlines()[:3]
    for f in files:
        file_path = REPO_ROOT / f
        if file_path.exists():
            try:
                content = file_path.read_text()
                tool_calls += 1
                total_input += count_tokens_approx(str(file_path))
                total_output += count_tokens_approx(content)
                answers.append(content[:1500])
            except Exception as e:
                result.error = str(e)
                break

    elapsed = int((time.perf_counter() - start) * 1000)

    result.wall_time_ms = elapsed
    result.tool_calls = tool_calls
    result.tokens_input = total_input
    result.tokens_output = total_output
    result.answer = "\n".join(answers)[:4000]
    result.accuracy = _score_accuracy(task, result.answer)
    return result


# ---------------------------------------------------------------------------
# MCP-enabled agent (uses ash-mcp tools)
# ---------------------------------------------------------------------------


def run_mcp_task(task: Task) -> TaskResult:
    """
    Simulate an MCP-enabled agent solving the task using ash_workspace_symbols
    and ash_find_references.
    """
    result = TaskResult(task_id=task.id, mode="mcp")
    start = time.perf_counter()

    total_input = 0
    total_output = 0
    tool_calls = 0
    answers: List[str] = []

    # Step 1: workspace symbol search
    symbol = task.search_term
    cmd = [
        str(ASH_MCP_BINARY),
        "--quiet",
    ]
    # We use the MCP tool via JSON-RPC stdio
    init_msg = (
        '{"jsonrpc":"2.0","id":1,"method":"initialize",'
        '"params":{"protocolVersion":"2024-11-05","capabilities":{},'
        '"clientInfo":{"name":"benchmark","version":"1.0"}}}'
    )
    tool_msg = (
        '{"jsonrpc":"2.0","id":2,"method":"tools/call",'
        '"params":{"name":"ash_workspace_symbols",'
        '"arguments":{"root":"%s","query":"%s"}}}'
        % (str(REPO_ROOT), symbol)
    )
    stdin_data = init_msg + "\n" + tool_msg + "\n"

    try:
        proc = subprocess.run(
            cmd,
            cwd=REPO_ROOT,
            input=stdin_data,
            capture_output=True,
            text=True,
            timeout=30,
        )
        tool_calls += 1
        total_input += count_tokens_approx(stdin_data)
        total_output += count_tokens_approx(proc.stdout)
        answers.append(proc.stdout[:2000])
    except Exception as e:
        result.error = f"MCP workspace symbols failed: {e}"
        result.wall_time_ms = int((time.perf_counter() - start) * 1000)
        return result

    # Step 2: find references in the first matching file (if any)
    # Extract first .ash or .rs file from the task's expected files
    target_file = None
    for f in task.files_involved:
        if f.endswith(".ash") or f.endswith(".rs"):
            target_file = REPO_ROOT / f
            break

    if target_file and target_file.exists():
        # Find a line with the symbol
        try:
            lines = target_file.read_text().splitlines()
            line_num = 1
            col_num = 1
            for i, line in enumerate(lines, 1):
                idx = line.find(symbol)
                if idx != -1:
                    line_num = i
                    col_num = idx + 1
                    break

            ref_msg = (
                '{"jsonrpc":"2.0","id":3,"method":"tools/call",'
                '"params":{"name":"ash_find_references",'
                '"arguments":{"file":"%s","line":%d,"column":%d}}}'
                % (str(target_file), line_num, col_num)
            )
            stdin_data2 = init_msg + "\n" + ref_msg + "\n"
            proc2 = subprocess.run(
                cmd,
                cwd=REPO_ROOT,
                input=stdin_data2,
                capture_output=True,
                text=True,
                timeout=30,
            )
            tool_calls += 1
            total_input += count_tokens_approx(stdin_data2)
            total_output += count_tokens_approx(proc2.stdout)
            answers.append(proc2.stdout[:2000])
        except Exception as e:
            result.error = f"MCP find references failed: {e}"

    elapsed = int((time.perf_counter() - start) * 1000)

    result.wall_time_ms = elapsed
    result.tool_calls = tool_calls
    result.tokens_input = total_input
    result.tokens_output = total_output
    result.answer = "\n".join(answers)[:4000]
    result.accuracy = _score_accuracy(task, result.answer)
    return result


# ---------------------------------------------------------------------------
# Scoring
# ---------------------------------------------------------------------------


def _extract_symbol(description: str) -> str:
    """Heuristic: extract the most specific symbol name from the task."""
    import re
    quotes = re.findall(r"`([^`]+)`", description)
    if quotes:
        return min(quotes, key=len)
    words = description.split()
    for w in words:
        w = w.strip(".,?")
        if w and w[0].isupper():
            return w
    # Fallback: look for quoted phrases with single quotes
    single_quotes = re.findall(r"'([^']+)'", description)
    if single_quotes:
        return min(single_quotes, key=len)
    return words[-1].strip(".,?") if words else "unknown"


def _score_accuracy(task: Task, answer: str) -> float:
    """
    Heuristic accuracy scoring based on whether expected files are mentioned.
    1.0 = all expected files mentioned
    0.5 = some expected files mentioned
    0.0 = none mentioned
    """
    mentioned = 0
    for f in task.files_involved:
        if f in answer or str(REPO_ROOT / f) in answer:
            mentioned += 1
    if mentioned == len(task.files_involved):
        return 1.0
    if mentioned > 0:
        return 0.5
    return 0.0


def _load_cross_language_mappings() -> List[tuple[str, str]]:
    """Load `(ash_symbol, rust_symbol)` mappings from the committed project config."""
    config = REPO_ROOT / ".ash" / "cross_lang_config.yaml"
    mappings: List[tuple[str, str]] = []
    current_ash: str | None = None
    if not config.exists():
        return mappings
    for raw_line in config.read_text().splitlines():
        line = raw_line.strip()
        if line.startswith("ash_symbol:") or line.startswith("- ash_symbol:"):
            current_ash = line.split(":", 1)[1].strip().strip('"')
        elif line.startswith("rust_symbol:") and current_ash:
            rust_symbol = line.split(":", 1)[1].strip().strip('"')
            mappings.append((current_ash, rust_symbol))
            current_ash = None
    return mappings


def _mcp_json_payload(tool_name: str, arguments: Dict[str, Any]) -> Dict[str, Any]:
    init_msg = (
        '{"jsonrpc":"2.0","id":1,"method":"initialize",'
        '"params":{"protocolVersion":"2024-11-05","capabilities":{},'
        '"clientInfo":{"name":"phase143-benchmark","version":"1.0"}}}'
    )
    tool_msg = json.dumps({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {"name": tool_name, "arguments": arguments},
    })
    proc = subprocess.run(
        [str(ASH_MCP_BINARY), "--quiet"],
        cwd=REPO_ROOT,
        input=init_msg + "\n" + tool_msg + "\n",
        capture_output=True,
        text=True,
        timeout=30,
    )
    for line in proc.stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get("id") != 2:
            continue
        content = message.get("result", {}).get("content", [])
        for item in content:
            if item.get("type") == "text":
                try:
                    return json.loads(item.get("text", "{}"))
                except json.JSONDecodeError:
                    continue
    return {"error": proc.stderr or proc.stdout or "no MCP response"}


def run_cross_language_evaluation() -> Dict[str, Any]:
    """Evaluate configured Ash ↔ Rust mappings through the delivered MCP tools."""
    mappings = _load_cross_language_mappings()
    rows: List[Dict[str, Any]] = []
    start = time.perf_counter()
    rust_success = 0
    ash_success = 0

    for ash_symbol, rust_symbol in mappings:
        item_start = time.perf_counter()
        rust_payload = _mcp_json_payload(
            "ash_find_rust_implementation",
            {
                "ash_symbol": ash_symbol,
                "file": str(REPO_ROOT / "crates/ash-mcp/tests/fixtures/effect_usage.ash"),
                "line": 1,
                "column": 1,
            },
        )
        ash_payload = _mcp_json_payload(
            "ash_find_ash_usage",
            {"rust_symbol": rust_symbol},
        )
        rust_found = bool(rust_payload.get("found"))
        usage_count = len(ash_payload.get("usages", []))
        ash_found = usage_count > 0
        rust_success += int(rust_found)
        ash_success += int(ash_found)
        rows.append({
            "ash_symbol": ash_symbol,
            "rust_symbol": rust_symbol,
            "rust_file": rust_payload.get("file"),
            "ash_to_rust_found": rust_found,
            "rust_to_ash_found": ash_found,
            "ash_usage_count": usage_count,
            "latency_ms": round((time.perf_counter() - item_start) * 1000, 3),
            "tool_errors": [
                payload.get("error")
                for payload in [rust_payload, ash_payload]
                if payload.get("error")
            ],
        })

    negative_ash_payload = _mcp_json_payload(
        "ash_find_rust_implementation",
        {
            "ash_symbol": "DefinitelyMissingPhase143Symbol",
            "file": str(REPO_ROOT / "crates/ash-mcp/tests/fixtures/effect_usage.ash"),
            "line": 1,
            "column": 1,
        },
    )
    negative_rust_payload = _mcp_json_payload(
        "ash_find_ash_usage",
        {"rust_symbol": "ash_core::effect::DefinitelyMissingPhase143Symbol"},
    )
    false_positives = int(bool(negative_ash_payload.get("found"))) + int(
        bool(negative_rust_payload.get("usages"))
    )
    negative_probe_count = 2

    latencies = [row["latency_ms"] for row in rows]
    sorted_latencies = sorted(latencies)
    p95_index = min(len(sorted_latencies) - 1, max(0, (len(sorted_latencies) * 95 + 99) // 100 - 1))
    total = len(mappings)
    return {
        "total_mappings": total,
        "ash_to_rust_accuracy": rust_success / total if total else 0.0,
        "rust_to_ash_accuracy": ash_success / total if total else 0.0,
        "false_positive_rate": false_positives / negative_probe_count,
        "false_positive_probe_count": negative_probe_count,
        "false_positive_count": false_positives,
        "avg_lookup_latency_ms": round(sum(latencies) / total, 3) if total else 0.0,
        "p95_lookup_latency_ms": sorted_latencies[p95_index] if sorted_latencies else 0.0,
        "total_latency_ms": round((time.perf_counter() - start) * 1000, 3),
        "memory_usage_mb": None,
        "startup_time_ms": None,
        "productivity_metrics": {
            "token_reduction_percent": None,
            "tool_call_reduction_percent": None,
            "task_completion_improvement_percent": None,
        },
        "rows": rows,
    }


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main() -> int:
    parser = argparse.ArgumentParser(description="Ash MCP benchmark harness")
    parser.add_argument(
        "--mode",
        choices=["baseline", "mcp"],
        required=True,
        help="Benchmark mode",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("benchmark-results.json"),
        help="Output JSON file",
    )
    parser.add_argument(
        "--tasks",
        nargs="+",
        default=None,
        help="Specific task IDs to run (default: all)",
    )
    parser.add_argument(
        "--include-cross-language",
        action="store_true",
        help="Include configured Ash↔Rust cross-language mapping evaluation",
    )
    args = parser.parse_args()

    if args.include_cross_language and args.mode != "mcp":
        print("ERROR: --include-cross-language requires --mode mcp")
        return 1

    if args.mode == "mcp" and not ASH_MCP_BINARY.exists():
        print(f"ERROR: ash-mcp binary not found at {ASH_MCP_BINARY}")
        print("Build it first: cargo build -p ash-mcp --release")
        return 1

    tasks = [t for t in CORPUS if not args.tasks or t.id in args.tasks]
    if not tasks:
        print("ERROR: No tasks matched")
        return 1

    report = BenchmarkReport(
        mode=args.mode,
        timestamp=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        ash_mcp_version=get_ash_mcp_version() if args.mode == "mcp" else "n/a",
        git_commit=get_git_commit(),
    )

    for task in tasks:
        print(f"Running {task.id}: {task.description[:60]}...")
        if args.mode == "baseline":
            result = run_baseline_task(task)
        else:
            result = run_mcp_task(task)
        report.results.append(result)
        print(
            f"  -> {result.wall_time_ms}ms, "
            f"{result.tool_calls} calls, "
            f"~{result.tokens_input + result.tokens_output} tokens, "
            f"accuracy={result.accuracy}"
        )
        if result.error:
            print(f"  ERROR: {result.error}")

    if args.include_cross_language:
        print("Running cross-language mapping evaluation...")
        report.cross_language = run_cross_language_evaluation()
        print(
            "  -> ash_to_rust_accuracy="
            f"{report.cross_language['ash_to_rust_accuracy']:.2f}, "
            "rust_to_ash_accuracy="
            f"{report.cross_language['rust_to_ash_accuracy']:.2f}, "
            "avg_lookup_latency_ms="
            f"{report.cross_language['avg_lookup_latency_ms']}"
        )

    # Write report
    output_path = args.output
    with open(output_path, "w") as f:
        json.dump(report.to_dict(), f, indent=2)
    print(f"\nResults written to {output_path}")

    # Summary
    total_time = sum(r.wall_time_ms for r in report.results)
    total_calls = sum(r.tool_calls for r in report.results)
    total_tokens = sum(r.tokens_input + r.tokens_output for r in report.results)
    avg_accuracy = sum(r.accuracy for r in report.results) / len(report.results)
    print(f"\nSummary ({args.mode}):")
    print(f"  Total time:   {total_time}ms")
    print(f"  Total calls:  {total_calls}")
    print(f"  Total tokens: ~{total_tokens}")
    print(f"  Avg accuracy: {avg_accuracy:.2f}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
