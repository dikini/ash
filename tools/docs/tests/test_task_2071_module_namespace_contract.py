"""Contract tests for the TASK-2071 Phase-207 task split."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]


class Task2071ModuleNamespaceContractTests(unittest.TestCase):
    """Keep the docs-only contract and planned implementation boundaries exact."""

    def read(self, relative: str) -> str:
        """Read one repository UTF-8 document."""
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_spec_defines_expansion_and_two_distinct_collection_views(self) -> None:
        spec = self.read(
            "docs/spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md"
        )
        for required in (
            "CanonicalExpandedModuleGraph",
            "CanonicalCollectedModuleSnapshot",
            "CanonicalProvisionalNameView",
            "M-SYNTAX-PREPASS",
            "(ModuleKey, declaration kind, canonical parent, origin key)",
            "(namespace bucket, visible local key)",
            "`DataKind` | Promoted-kind bucket",
            "`PropositionPredicate` | Proposition bucket",
            "signature, callable shape, body, checked type, equation, final export, or runtime-authority fact.",
        ):
            self.assertIn(required, spec)

        self.assertIn("filesystem lookup, path/source-text fallback", spec)
        self.assertIn("providers before consumers", spec)

    def test_task_split_has_one_closed_contract_one_active_and_one_planned_implementation(self) -> None:
        contract = self.read(
            "docs/plan/tasks/TASK-2071-module-namespace-and-provisional-view-contract.md"
        )
        expanded = self.read(
            "docs/plan/tasks/TASK-2074-canonical-expanded-module-graph.md"
        )
        collection = self.read(
            "docs/plan/tasks/TASK-2075-two-tier-complete-module-collection.md"
        )

        self.assertIn("**Status:** Complete", contract)
        self.assertIn("MOD-REAL-001–004 syntax-prepass, expansion", contract)
        self.assertIn("**Implementation:** not_implemented", contract)
        self.assertIn("**Evidence:** none", contract)
        self.assertIn("**Status:** In progress", expanded)
        self.assertIn("**Status:** Planned", collection)
        self.assertNotIn("**Status:** In progress", collection)

    def test_plans_cover_required_evidence_without_claiming_it(self) -> None:
        expanded_plan = self.read(
            "docs/plans/2026-08-04-task-2074-canonical-expanded-module-graph-implementation-plan.md"
        )
        collection_plan = self.read(
            "docs/plans/2026-08-04-task-2075-two-tier-complete-module-collection-implementation-plan.md"
        )
        for required in (
            "shallow `ModuleBody`",
            "syntax dependency",
            "file/inline",
            "no-FS",
            "atomic",
            "property",
        ):
            self.assertIn(required, expanded_plan)
        for required in (
            "Build a table covering",
            "plus `ModuleDecl`",
            "visibility carriers",
            "namespace",
            "member/constructor",
            "drift",
            "TASK-2068",
            "TASK-2070",
            "authority fence",
        ):
            self.assertIn(required, collection_plan)

    def test_manifest_closes_task_2071_and_activates_only_task_2074(self) -> None:
        manifest = json.loads(
            self.read("docs/plan/semantic-task-records.json")
        )
        tasks = set(manifest["active_tasks"])
        self.assertIn("TASK-2071", tasks)
        self.assertIn("TASK-2074", tasks)
        self.assertNotIn("TASK-2075", tasks)

        trace = json.loads(self.read("docs/spec/SEMANTIC-TRACEABILITY.json"))
        contract_node = "REQ-TASK-2071-MODULE-EXPANSION-AND-NAMESPACE-CONTRACT"
        self.assertTrue(any(node.get("id") == contract_node for node in trace["nodes"]))
        contract_rules = {
            edge["from"]
            for edge in trace["edges"]
            if edge.get("kind") == "assumes" and edge.get("to") == contract_node
        }
        self.assertEqual(
            contract_rules,
            {f"SEM-MODULE-REALIZATION-{number:03d}" for number in range(1, 5)},
        )

        record = next(
            item for item in manifest["records"] if item["task"] == "TASK-2071"
        )
        self.assertEqual(
            record["canonical_rule_ids"],
            [f"SEM-MODULE-REALIZATION-{number:03d}" for number in range(1, 5)],
        )
        self.assertEqual(record["implementation"], "not_implemented")
        self.assertEqual(record["evidence"]["status"], "none")
        self.assertEqual(record["parity"], "below_spec")

    def test_downstream_consumers_use_only_their_declared_view(self) -> None:
        task_2072 = self.read(
            "docs/plan/tasks/TASK-2072-parsed-import-resolution-and-atomic-binding.md"
        )
        task_2073 = self.read(
            "docs/plan/tasks/TASK-2073-checked-module-finalization-and-export-closure.md"
        )
        self.assertIn("CanonicalProvisionalNameView", task_2072)
        self.assertIn("must never inspect", task_2072)
        self.assertIn("CanonicalCollectedModuleSnapshot", task_2073)
        self.assertIn("must not recover signatures or bodies", task_2073)


if __name__ == "__main__":
    unittest.main()
