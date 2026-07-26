#!/usr/bin/env python3
"""Fail-closed validator for the machine-readable λAsh-CPS calculus freeze.

The calculus is intentionally a mathematical contract.  This tool checks the
small, stable schema used by the Phase 202 documentation gate and keeps Rust
representation choices outside its trusted base.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
from typing import Any


REPORT_SCHEMA = "ash-cps-calculus-validation-report/v1"
ARTIFACT_SCHEMA = "ash-cps-calculus/v1"
STAGES = {"kernel", "effect", "later"}
THEOREM_STATUSES = {"frozen", "target", "admitted", "deferred"}
KERNEL_FORMS = {"LetVal", "LetPrim", "LetCont", "LetContCall", "Jump", "Call", "If", "Match", "Return", "Trap"}
EFFECT_FORMS = {"Raise", "Handle", "RecordDischarge"}
LATER_FORMS = {"LetRec", "Thunk", "Trace", "Monitor", "Process"}
RULE_ID = re.compile(r"^SEM-(?:CPS|EFFECT|LATER)-[A-Z0-9]+-\d{3}$")
THEOREM_ID = re.compile(r"^THM-(?:CPS|EFFECT|LATER)-[A-Z0-9-]+-\d{3}$")
EXAMPLE_ID = re.compile(r"^EX-CPS-[A-Z0-9-]+-\d{3}$")


def issue(kind: str, message: str, **details: object) -> dict[str, object]:
    return {"kind": kind, "message": message, **details}


def nonempty_string(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip())


def string_list(value: object) -> list[str] | None:
    if not isinstance(value, list) or not all(nonempty_string(item) for item in value):
        return None
    return value


def validate_rules(data: dict[str, Any], errors: list[dict[str, object]]) -> set[str]:
    rules = data.get("rules")
    if not isinstance(rules, list) or not rules:
        errors.append(issue("invalid_rules", "rules must be a non-empty list"))
        return set()
    ids: set[str] = set()
    for index, rule in enumerate(rules):
        if not isinstance(rule, dict):
            errors.append(issue("invalid_rule", "rule must be an object", index=index))
            continue
        rule_id, stage, kind = rule.get("id"), rule.get("stage"), rule.get("kind")
        if not nonempty_string(rule_id) or not RULE_ID.fullmatch(rule_id):
            errors.append(issue("invalid_rule_id", "rule id must be a stable SEM-* id", index=index, rule_id=rule_id))
        elif rule_id in ids:
            errors.append(issue("duplicate_rule_id", "rule ids must be unique", rule_id=rule_id))
        else:
            ids.add(rule_id)
        if stage not in STAGES or not nonempty_string(kind):
            errors.append(issue("invalid_rule_stage", "rule requires a known stage and non-empty kind", rule_id=rule_id, stage=stage))
            continue
        prefix = rule_id.split("-", 2)[1] if isinstance(rule_id, str) and rule_id.count("-") >= 2 else ""
        expected_stage = {"CPS": "kernel", "EFFECT": "effect", "LATER": "later"}.get(prefix)
        forbidden_kernel_feature = isinstance(rule_id, str) and re.search(r"(?:RAISE|HANDLE|DISCHARGE|REC|THUNK|TRACE|MONITOR|PROCESS)", rule_id) is not None
        if expected_stage != stage or (stage == "kernel" and forbidden_kernel_feature):
            errors.append(issue("invalid_rule_stage", "rule family must be declared at its frozen stage", rule_id=rule_id, stage=stage, expected=expected_stage))
    return ids


def validate_return_decision(data: dict[str, Any], errors: list[dict[str, object]]) -> None:
    decision = data.get("return_decision")
    required = {"status", "selected", "plan_202_claim", "spec_098b_claim", "resolution"}
    if not isinstance(decision, dict) or any(not nonempty_string(decision.get(key)) for key in required):
        errors.append(issue("unresolved_return_decision", "Return requires a complete explicit resolution"))
        return
    if decision.get("status") != "resolved":
        errors.append(issue("unresolved_return_decision", "Return decision status must be resolved", status=decision.get("status")))


def validate_ladder(data: dict[str, Any], errors: list[dict[str, object]]) -> None:
    fragment = data.get("admitted_fragment")
    ladder = data.get("theorem_ladder")
    bad = not isinstance(fragment, dict) or not isinstance(ladder, list) or not ladder
    includes = fragment.get("includes") if isinstance(fragment, dict) else None
    excludes = fragment.get("excludes") if isinstance(fragment, dict) else None
    if string_list(includes) is None or string_list(excludes) is None:
        bad = True
    else:
        included, excluded = set(includes), set(excludes)
        # The admitted calculus may not smuggle in effect/later constructs.
        if not KERNEL_FORMS.issubset(included) or included & (EFFECT_FORMS | LATER_FORMS) or not ({"Raise", "Handle"} & excluded):
            bad = True
    ids: set[str] = set()
    if isinstance(ladder, list):
        for item in ladder:
            if not isinstance(item, dict) or not nonempty_string(item.get("id")) or not THEOREM_ID.fullmatch(item["id"]) or not nonempty_string(item.get("claim")) or item.get("status") not in THEOREM_STATUSES or item.get("stage") not in STAGES:
                bad = True
                continue
            if item["id"] in ids:
                bad = True
            ids.add(item["id"])
    if bad:
        errors.append(issue("invalid_theorem_ladder", "the admitted fragment and theorem ladder must be explicit and staged"))


def validate_examples(data: dict[str, Any], rule_ids: set[str], errors: list[dict[str, object]]) -> None:
    examples = data.get("examples")
    if not isinstance(examples, list) or not examples:
        errors.append(issue("invalid_canonical_example", "examples must be a non-empty list"))
        return
    seen: set[str] = set()
    for index, example in enumerate(examples):
        valid = isinstance(example, dict)
        if not valid:
            errors.append(issue("invalid_canonical_example", "example must be an object", index=index))
            continue
        example_id = example.get("id")
        rule_refs = string_list(example.get("rule_ids"))
        term, projection = example.get("term"), example.get("expected_terminal_projection")
        valid = (nonempty_string(example_id) and EXAMPLE_ID.fullmatch(example_id) is not None and example_id not in seen and rule_refs is not None and bool(rule_refs)
                 and all(ref in rule_ids for ref in rule_refs) and isinstance(term, dict) and nonempty_string(term.get("form"))
                 and term["form"] in KERNEL_FORMS and isinstance(projection, dict) and projection.get("kind") in {"return", "trap"})
        if valid:
            if projection["kind"] == "return":
                valid = nonempty_string(projection.get("value"))
            else:
                valid = nonempty_string(projection.get("reason"))
        if not valid:
            errors.append(issue("invalid_canonical_example", "example needs a kernel term, known rules, and terminal projection", index=index, example_id=example_id))
        elif isinstance(example_id, str):
            seen.add(example_id)


def validate_trusted_base(data: dict[str, Any], errors: list[dict[str, object]]) -> None:
    base = data.get("trusted_base")
    if not isinstance(base, dict) or string_list(base.get("axioms")) is None or string_list(base.get("exclusions")) is None:
        errors.append(issue("invalid_trusted_base", "trusted_base requires non-empty axioms and exclusions"))
        return
    suspicious = re.compile(r"\b(?:rust|rc\s*<|refcell|arc\s*<|mutex|serde|json|scheduler|host[- ]provider|layout)\b", re.I)
    for axiom in base["axioms"]:
        if suspicious.search(axiom):
            errors.append(issue("rust_helper_axiom", "implementation storage/helper behavior cannot be a calculus axiom", axiom=axiom))
    exclusions = " ".join(base["exclusions"]).lower()
    if "rust" not in exclusions or "storage" not in exclusions:
        errors.append(issue("invalid_trusted_base", "trusted base must explicitly exclude Rust storage", exclusions=base["exclusions"]))


def validate(data: object) -> list[dict[str, object]]:
    if not isinstance(data, dict) or data.get("schema") != ARTIFACT_SCHEMA:
        return [issue("invalid_artifact", f"artifact schema must be {ARTIFACT_SCHEMA}")]
    errors: list[dict[str, object]] = []
    for field in ("id", "name"):
        if not nonempty_string(data.get(field)):
            errors.append(issue("invalid_artifact", f"{field} must be non-empty"))
    syntax = data.get("syntax")
    if not isinstance(syntax, dict) or any(string_list(syntax.get(field)) is None for field in ("atoms", "values", "kernel_terms", "effect_terms", "later_terms")):
        errors.append(issue("invalid_syntax", "syntax must name every term stratum"))
    state = data.get("state")
    if not isinstance(state, dict) or string_list(state.get("mathematical_components")) is None or state.get("excludes_rust_storage") is not True:
        errors.append(issue("invalid_state", "state must be mathematical and explicitly exclude Rust storage"))
    if string_list(data.get("judgments")) is None:
        errors.append(issue("invalid_judgments", "judgments must be a non-empty list of names"))
    rule_ids = validate_rules(data, errors)
    validate_return_decision(data, errors)
    validate_ladder(data, errors)
    validate_examples(data, rule_ids, errors)
    validate_trusted_base(data, errors)
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifact", required=True, type=Path)
    parser.add_argument("--format", choices=("json",), default="json")
    args = parser.parse_args()
    try:
        payload: object = json.loads(args.artifact.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        errors = [issue("invalid_artifact", "artifact is not readable JSON", detail=str(exc))]
    else:
        errors = validate(payload)
    print(json.dumps({"schema": REPORT_SCHEMA, "errors": errors}, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
