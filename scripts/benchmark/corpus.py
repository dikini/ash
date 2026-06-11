"""Benchmark corpus: real codebase exploration tasks for Ash."""
from dataclasses import dataclass
from typing import List


@dataclass(frozen=True)
class Task:
    """A single benchmark task."""

    id: str
    description: str
    search_term: str  # Symbol to search for
    expected_answer: str  # Human-verified ground truth summary
    files_involved: List[str]  # Relative paths from repo root


# Corpus tasks require cross-file understanding of the Ash codebase.
# All paths are relative to the repository root.
CORPUS: List[Task] = [
    Task(
        id="T1",
        description=(
            "Find where the Effect lattice is defined and list all files "
            "where Effect variants (Epistemic, Deliberative, Evaluative, Operational) "
            "are referenced."
        ),
        search_term="Effect",
        expected_answer=(
            "Defined in crates/ash-core/src/effect.rs. "
            "Referenced in crates/ash-core/src/effect.rs, "
            "crates/ash-typeck/src/solver.rs, "
            "crates/ash-core/src/proptest_helpers.rs, "
            "and test files."
        ),
        files_involved=[
            "crates/ash-core/src/effect.rs",
            "crates/ash-typeck/src/solver.rs",
            "crates/ash-core/src/proptest_helpers.rs",
        ],
    ),
    Task(
        id="T2",
        description=(
            "Find the definition of the capability checker and all workflows "
            "or functions that use capability constraints."
        ),
        search_term="CapabilityChecker",
        expected_answer=(
            "Defined in crates/ash-typeck/src/capability_check.rs. "
            "Used in crates/ash-typeck/src/lib.rs and test files."
        ),
        files_involved=[
            "crates/ash-typeck/src/capability_check.rs",
            "crates/ash-typeck/src/lib.rs",
        ],
    ),
    Task(
        id="T3",
        description=(
            "Locate all 'observe' statement parsers and the runtime execution "
            "function that handles observe."
        ),
        search_term="parse_observe",
        expected_answer=(
            "Parser in crates/ash-parser/src/parse_observe.rs. "
            "Runtime in crates/ash-interp/src/execute_observe.rs."
        ),
        files_involved=[
            "crates/ash-parser/src/parse_observe.rs",
            "crates/ash-interp/src/execute_observe.rs",
        ],
    ),
    Task(
        id="T4",
        description=(
            "Find where the 'workflow' keyword is parsed and where workflow "
            "definitions are type-checked."
        ),
        search_term="workflow",
        expected_answer=(
            "Parsed in crates/ash-parser/src/parse_workflow.rs. "
            "Type-checked in crates/ash-typeck/src/lib.rs."
        ),
        files_involved=[
            "crates/ash-parser/src/parse_workflow.rs",
            "crates/ash-typeck/src/lib.rs",
        ],
    ),
    Task(
        id="T5",
        description=(
            "Find all implementations of the Stream trait and where "
            "BidirectionalStreamProvider is defined."
        ),
        search_term="BidirectionalStreamProvider",
        expected_answer=(
            "Defined in crates/ash-interp/src/stream.rs. "
            "Also referenced in crates/ash-core/src/stream.rs."
        ),
        files_involved=[
            "crates/ash-interp/src/stream.rs",
            "crates/ash-core/src/stream.rs",
        ],
    ),
    Task(
        id="T6",
        description=(
            "Locate the SMT policy checker and find all policy test files "
            "that exercise it."
        ),
        search_term="SmtContext",
        expected_answer=(
            "Defined in crates/ash-typeck/src/smt.rs. "
            "Re-exported in crates/ash-typeck/src/lib.rs. "
            "Used in crates/ash-typeck/src/requirements.rs."
        ),
        files_involved=[
            "crates/ash-typeck/src/smt.rs",
            "crates/ash-typeck/src/lib.rs",
            "crates/ash-typeck/src/requirements.rs",
        ],
    ),
    Task(
        id="T7",
        description=(
            "Find where 'fn helper' is defined in the agent query fixtures "
            "and all places it is called."
        ),
        search_term="helper",
        expected_answer=(
            "Defined in crates/ash-mcp/tests/agent_queries/fixtures/lib.ash. "
            "Called in crates/ash-mcp/tests/agent_queries/fixtures/main.ash."
        ),
        files_involved=[
            "crates/ash-mcp/tests/agent_queries/fixtures/lib.ash",
            "crates/ash-mcp/tests/agent_queries/fixtures/main.ash",
        ],
    ),
    Task(
        id="T9",
        description=(
            "Find all workflow algebra primitives (unit, bind, then) in the "
            "standard library and list the files where they are defined."
        ),
        search_term="unit",
        expected_answer=(
            "Defined in std/src/workflow.ash (unit, bind, then). "
            "Also related definitions in std/src/act.ash and std/src/proc.ash."
        ),
        files_involved=[
            "std/src/workflow.ash",
            "std/src/act.ash",
            "std/src/proc.ash",
        ],
    ),
    Task(
        id="T10",
        description=(
            "Find all capability implementations in the examples directory "
            "and list their file paths."
        ),
        search_term="capability",
        expected_answer=(
            "Found in examples/06-capability-implementations/*.ash "
            "and examples/code_review.ash."
        ),
        files_involved=[
            "examples/06-capability-implementations/01-mock-internal-kv.ash",
            "examples/06-capability-implementations/02-caching-kv-adapter.ash",
            "examples/code_review.ash",
        ],
    ),
]


def get_task(task_id: str) -> Task:
    for task in CORPUS:
        if task.id == task_id:
            return task
    raise KeyError(f"Unknown task: {task_id}")
