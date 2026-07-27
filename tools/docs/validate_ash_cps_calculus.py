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
EFFECT_RULE_INDEX = {
    "innermost_lookup": "SEM-EFFECT-LOOKUP-001",
    "record_discharge": "SEM-EFFECT-DISCHARGE-001",
    "missing_discharge": "SEM-EFFECT-MISSDISCHARGE-001",
    "deep_affine_resume": "SEM-EFFECT-RESUME-001",
    "handler_body_trap": "SEM-EFFECT-HANDLERTRAP-001",
    "provider_invocation": "SEM-EFFECT-PROVIDER-001",
    "timeout": "SEM-EFFECT-TIMEOUT-001",
    "cancellation": "SEM-EFFECT-CANCEL-001",
}
EFFECT_MAPPING_FIELDS = {"cps_artifact", "target_operational", "engine_view", "terminal_projection"}
EFFECT_CONFORMANCE_CASES = {
    "normal_return", "missing_admission", "malformed_unchecked_cps", "handler_body_trap", "timeout", "cancellation",
}
EFFECT_FORMAL_JUDGMENTS = {"configuration_well_formedness", "effect_typing"}
EFFECT_FORMAL_TRANSITIONS = {
    "frame_lookup", "record_discharge", "raise_dispatch", "handler_entry_selected_frame_removed",
    "affine_resume_reinstates_handler", "affine_resume_reuse_rejected", "handled_computation_completion",
    "resumed_tail_completion", "abortive_clause_completion", "handler_body_trap", "provider_external_outcome", "missing_discharge", "timeout_terminalization",
    "cancellation_terminalization", "terminalization",
}
EFFECT_MAPPING_SINGLE_AUTHORITIES = {
    "SEM-EFFECT-LOOKUP-001": "SPEC-099b §5",
    "SEM-EFFECT-RAISE-001": "SPEC-099b §5",
    "SEM-EFFECT-HANDLE-001": "SPEC-099b §5",
    "SEM-EFFECT-DISCHARGE-001": "SPEC-099b §5",
    "SEM-EFFECT-MISSDISCHARGE-001": "SPEC-099b §5",
    "SEM-EFFECT-RESUME-001": "SPEC-099b §5",
    "SEM-EFFECT-HANDLERTRAP-001": "SPEC-099b §6",
    "SEM-EFFECT-ADMISSION-001": "PLAN-203 Admission and client parity",
    "SEM-EFFECT-TIMEOUT-001": "PLAN-203 Admission and client parity",
    "SEM-EFFECT-CANCEL-001": "PLAN-203 Admission and client parity",
}
EFFECT_MAPPING_MULTI_AUTHORITIES = {
    "SEM-EFFECT-PROVIDER-001": {
        "provider_boundary": "SPEC-099b §5",
        "run_control_outcomes": "PLAN-203 Admission and client parity",
    },
    "SEM-EFFECT-TERMINAL-001": {
        "kernel_return": "λAsh-CPS₀ kernel Return",
        "structured_trap": "SPEC-099b §6",
        "external_projection": "OBS-TARGET-PROJECTION-001",
        "run_control": "PLAN-203 Admission and client parity",
    },
}


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
        if not nonempty_string(stage) or stage not in STAGES or not nonempty_string(kind):
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
                 and term["form"] in (KERNEL_FORMS | EFFECT_FORMS) and isinstance(projection, dict) and projection.get("kind") in {"return", "trap", "external"})
        if valid:
            if projection["kind"] == "return":
                valid = nonempty_string(projection.get("value"))
            else:
                valid = nonempty_string(projection.get("reason"))
        if not valid:
            errors.append(issue("invalid_canonical_example", "example needs a kernel term, known rules, and terminal projection", index=index, example_id=example_id))
        elif isinstance(example_id, str):
            seen.add(example_id)


def validate_effect_correspondence(data: dict[str, Any], rule_ids: set[str], errors: list[dict[str, object]]) -> None:
    """Validate TASK-2031's rule-indexed, non-authorizing effect handoff when present."""
    correspondence = data.get("effect_correspondence")
    if correspondence is None:
        return
    if not isinstance(correspondence, dict):
        errors.append(issue("invalid_effect_correspondence", "effect_correspondence must be an object"))
        return
    if (
        correspondence.get("status") != "complete"
        or correspondence.get("calculus") != "lambda-Ash-Effect"
        or correspondence.get("conservative_extension_of") != "lambda-Ash-CPS0"
    ):
        errors.append(issue("invalid_effect_correspondence", "effect correspondence must identify the complete conservative extension"))

    configuration = correspondence.get("configuration")
    required_components = {
        "term", "value_environment", "continuation_store", "affine_continuation_consumption",
        "ordered_handler_provider_frames", "discharge_record", "residual_closed_rows", "external_outcome",
    }
    components = configuration.get("components") if isinstance(configuration, dict) else None
    if not string_list(components) or not required_components.issubset(set(components)):
        errors.append(issue("invalid_effect_correspondence", "effect configuration must name its complete mathematical state"))
    syntax = correspondence.get("syntax")
    required_forms = {"Raise", "Handle", "RecordDischarge", "HandlerFrame", "ProviderFrame", "AffineResume", "ExternalOutcome"}
    forms = syntax.get("forms") if isinstance(syntax, dict) else None
    if not string_list(forms) or not required_forms.issubset(set(forms)):
        errors.append(issue("invalid_effect_correspondence", "effect syntax must name every required form"))
    formal_relations = correspondence.get("formal_relations")
    relations_valid = isinstance(formal_relations, dict)
    if isinstance(formal_relations, dict):
        for group, required in (
            ("judgments", EFFECT_FORMAL_JUDGMENTS),
            ("transitions", EFFECT_FORMAL_TRANSITIONS),
        ):
            relations = formal_relations.get(group)
            if not isinstance(relations, dict) or not required.issubset(relations):
                relations_valid = False
                continue
            for relation_name in required:
                relation = relations.get(relation_name)
                if (
                    not isinstance(relation, dict)
                    or not nonempty_string(relation.get("notation"))
                    or not string_list(relation.get("rule_ids"))
                    or not set(relation["rule_ids"]).issubset(rule_ids)
                ):
                    relations_valid = False
    if not relations_valid:
        errors.append(issue("incomplete_effect_formal_relations", "effect judgments and transitions require notation and stable rule identifiers"))
    transitions = formal_relations.get("transitions") if isinstance(formal_relations, dict) else None
    expected_completion_routes = {
        "handled_computation_completion": "done_once",
        "resumed_tail_completion": "done_once",
        "abortive_clause_completion": "handler_result_directly",
    }
    if not isinstance(transitions, dict) or any(
        not isinstance(transitions.get(relation), dict)
        or transitions[relation].get("completion_route") != completion_route
        or (completion_route == "done_once" and "done" not in str(transitions[relation].get("notation", "")))
        or (completion_route == "handler_result_directly" and "done" in str(transitions[relation].get("notation", "")))
        for relation, completion_route in expected_completion_routes.items()
    ) or any(
        isinstance(relation, dict)
        and "handler-body" in str(relation.get("notation", ""))
        and "done" in str(relation.get("notation", ""))
        for relation in transitions.values()
    ):
        errors.append(issue("invalid_effect_handler_completion_routes", "handled and resumed completion use done once; abortive clauses return directly"))
    coverage = correspondence.get("effect_extension_coverage")
    if coverage != {
        "status": "complete",
        "separate_from_admitted_kernel_fragment": True,
        "kernel_fragment_excludes_effect_forms": True,
    }:
        errors.append(issue("invalid_effect_extension_coverage", "complete effect coverage must stay separate from the frozen kernel fragment"))
    authority = correspondence.get("authority_boundary")
    if not isinstance(authority, dict) or authority.get("rows_are_requirements_only") is not True or authority.get("frame_installation") != "separately_authorized_admission_only" or authority.get("no_second_execution_route") is not True:
        errors.append(issue("invalid_effect_authority_boundary", "rows must not install frames or authorize another execution route"))

    index = correspondence.get("rule_index")
    if not isinstance(index, dict) or any(index.get(name) != rule_id for name, rule_id in EFFECT_RULE_INDEX.items()):
        errors.append(issue("mis_mapped_effect_correspondence_rule", "effect rule index must preserve its stable rule identities"))
    effect_rule_ids = {
        rule_id for rule_id in rule_ids if rule_id.startswith("SEM-EFFECT-")
    }
    mapping = correspondence.get("mapping")
    mapping_valid = isinstance(mapping, dict)
    if isinstance(mapping, dict):
        for rule_id in effect_rule_ids:
            row = mapping.get(rule_id)
            if not isinstance(row, dict) or any(not nonempty_string(row.get(field)) for field in EFFECT_MAPPING_FIELDS):
                mapping_valid = False
                break
    if not mapping_valid:
        errors.append(issue("incomplete_effect_correspondence_mapping", "every effect rule requires CPS, operational, Engine-view, and terminal mapping fields"))
    if not isinstance(mapping, dict) or any(
        not isinstance(mapping.get(rule_id), dict)
        or mapping[rule_id].get("target_authority") != authority
        or authority not in str(mapping[rule_id].get("target_operational", ""))
        for rule_id, authority in EFFECT_MAPPING_SINGLE_AUTHORITIES.items()
    ) or any(
        not isinstance(mapping.get(rule_id), dict)
        or "target_authority" in mapping[rule_id]
        or mapping[rule_id].get("target_authorities") != authorities
        for rule_id, authorities in EFFECT_MAPPING_MULTI_AUTHORITIES.items()
    ):
        errors.append(issue("incorrect_effect_mapping_authority", "effect mapping authority must match the declared operational or PLAN-203 source"))

    obligations = correspondence.get("conformance_obligations")
    cases: dict[str, dict[str, object]] = {}
    if isinstance(obligations, list):
        for case in obligations:
            if not isinstance(case, dict):
                continue
            case_name = case.get("case")
            if nonempty_string(case_name):
                cases[case_name] = case
    if not EFFECT_CONFORMANCE_CASES.issubset(set(cases)) or any(
        not isinstance(case, dict)
        or not string_list(case.get("rule_ids"))
        or not set(case["rule_ids"]).issubset(rule_ids)
        or not nonempty_string(case.get("terminal_outcome"))
        or case.get("classification") != "planned-obligation"
        for name, case in cases.items() if name in EFFECT_CONFORMANCE_CASES
    ):
        errors.append(issue("invalid_effect_conformance_obligations", "effect conformance cases must remain rule-indexed planned obligations"))
    examples = data.get("examples")
    normal_return = next(
        (example for example in examples if isinstance(example, dict) and example.get("id") == "EX-CPS-EFFECT-NORMAL-RETURN-001"),
        None,
    ) if isinstance(examples, list) else None
    normal_term = normal_return.get("term") if isinstance(normal_return, dict) else None
    if (
        not isinstance(normal_return, dict)
        or not isinstance(normal_term, dict)
        or normal_term.get("form") != "Handle"
        or not normal_term.get("done_clause")
        or not normal_term.get("done_clause_once")
        or "SEM-EFFECT-HANDLE-001" not in normal_return.get("rule_ids", [])
    ):
        errors.append(issue("invalid_effect_normal_return_witness", "normal effect return must witness Handle done-clause completion exactly once"))
    verus = correspondence.get("verus_candidates")
    if (
        not isinstance(verus, list)
        or not verus
        or any(
            not isinstance(candidate, dict)
            or candidate.get("status") != "deferred"
            or candidate.get("scope") != "TASK-2031 λAsh-Effect correspondence"
            or candidate.get("disposition") != "deferred-unproved"
            for candidate in verus
        )
    ):
        errors.append(issue("invalid_effect_verus_candidates", "selected Verus candidates must remain deferred, not proved"))
    lookup_candidates = [
        candidate for candidate in verus
        if isinstance(candidate, dict) and candidate.get("rule_id") == "SEM-EFFECT-LOOKUP-001"
    ] if isinstance(verus, list) else []
    if len(lookup_candidates) != 1 or (
        lookup_candidates[0].get("candidate_kind") != "correspondence-bridge"
        or lookup_candidates[0].get("distinct_from_proof") != "PROOF-CPS-FRAME-LOOKUP-001"
    ):
        errors.append(issue("invalid_effect_lookup_bridge_candidate", "TASK-2031 lookup candidate must remain distinct from the limited existing proof"))


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
    validate_effect_correspondence(data, rule_ids, errors)
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
