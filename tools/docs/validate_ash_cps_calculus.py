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
    "resumed_tail_completion", "abortive_clause_completion", "done_terminalization", "handler_result_terminalization", "handler_body_trap", "provider_invocation", "provider_external_outcome", "provider_success_resume", "missing_discharge", "missing_discharge_terminalization", "provider_failure_terminalization", "generic_failure_terminalization", "timeout_terminalization", "timeout_external_terminalization",
    "cancellation_terminalization", "cancellation_external_terminalization", "terminalization",
}
EFFECT_MACHINE_DEFINITIONS = {
    "HandlerFrame": {"clauses", "done_clause", "residual_row", "captured_affine_resume"},
    "ProviderFrame": {
        "operation_identity", "authority", "persistent_across_invocation",
        "success_continuation", "failure_continuation",
    },
    "captured_affine_resume": {"binding", "consumption", "reinstall_handler_position"},
    "operation_clause_matching": {"frame_order", "clause_order", "done_clause", "residual_row"},
}
EFFECT_ENDPOINT_VOCABULARY = {
    "Raise", "Lookup", "Dispatch", "selected", "Handler", "Provider", "clause", "tail",
    "resume_1", "resume_0", "RecordDischarge", "Return", "HandledReturn", "ResumedTailReturn",
    "AbortiveClauseReturn", "HandlerBodyTrap", "TerminalReady", "done", "handler-result", "Trap",
    "InvokeProvider", "ExternalSuccess", "ExternalOutcome", "MissingDischarge", "External",
    "Terminal", "P.success", "P.failure",
}
EFFECT_TRANSITION_ENDPOINT_VOCABULARY = {
    "frame_lookup": {"Raise", "Lookup"},
    "record_discharge": {"RecordDischarge"},
    "raise_dispatch": {"Lookup", "Dispatch", "selected"},
    "handler_entry_selected_frame_removed": {"Dispatch", "selected", "Handler", "clause"},
    "affine_resume_reinstates_handler": {"resume_1", "tail"},
    "affine_resume_reuse_rejected": {"resume_0", "Trap"},
    "handled_computation_completion": {"HandledReturn", "done"},
    "resumed_tail_completion": {"ResumedTailReturn", "done"},
    "abortive_clause_completion": {"AbortiveClauseReturn", "handler-result"},
    "done_terminalization": {"done", "TerminalReady", "Return"},
    "handler_result_terminalization": {"handler-result", "TerminalReady", "Return"},
    "handler_body_trap": {"HandlerBodyTrap", "TerminalReady", "Trap"},
    "provider_invocation": {"Dispatch", "selected", "Provider", "InvokeProvider"},
    "provider_external_outcome": {"InvokeProvider", "ExternalSuccess", "ExternalOutcome"},
    "provider_success_resume": {"ExternalSuccess"},
    "missing_discharge": {"Dispatch", "selected", "MissingDischarge"},
    "missing_discharge_terminalization": {"MissingDischarge", "TerminalReady"},
    "provider_failure_terminalization": {"ExternalOutcome", "TerminalReady"},
    "generic_failure_terminalization": {"TerminalReady", "ExternalOutcome", "Terminal"},
    "timeout_terminalization": {"ExternalOutcome", "TerminalReady"},
    "timeout_external_terminalization": {"TerminalReady", "ExternalOutcome", "Terminal"},
    "cancellation_terminalization": {"ExternalOutcome", "TerminalReady"},
    "cancellation_external_terminalization": {"TerminalReady", "ExternalOutcome", "Terminal"},
    "terminalization": {"TerminalReady", "Return", "Trap", "MissingDischarge", "Terminal"},
}
EFFECT_CONFIGURATION_TRANSITIONS = set(EFFECT_TRANSITION_ENDPOINT_VOCABULARY)
EFFECT_COMPLETION_PHASES = {
    "handled_computation_completion": ("handled-return", "done", "HandledReturn"),
    "resumed_tail_completion": ("resumed-tail-return", "done", "ResumedTailReturn"),
    "abortive_clause_completion": ("abortive-clause-return", "handler-result", "AbortiveClauseReturn"),
    "done_terminalization": ("done", "terminal-ready", "done"),
    "handler_result_terminalization": ("handler-result", "terminal-ready", "handler-result"),
    "missing_discharge_terminalization": ("missing-discharge", "terminal-ready", "MissingDischarge"),
    "terminalization": ("terminal-ready", "terminal-envelope", "TerminalReady"),
    "provider_success_resume": ("provider-external-success", "provider-success-resume", "ExternalSuccess"),
    "provider_failure_terminalization": ("provider-failure", "provider-failure-ready", "ExternalOutcome"),
    "generic_failure_terminalization": ("provider-failure-ready", "terminal-envelope", "TerminalReady"),
    "timeout_terminalization": ("timeout-outcome", "timeout-ready", "ExternalOutcome"),
    "timeout_external_terminalization": ("timeout-ready", "terminal-envelope", "TerminalReady"),
    "cancellation_terminalization": ("cancellation-outcome", "cancellation-ready", "ExternalOutcome"),
    "cancellation_external_terminalization": ("cancellation-ready", "terminal-envelope", "TerminalReady"),
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


def endpoint_constructor_names(endpoint: object) -> set[str]:
    """Return constructor-shaped names written in a mathematical endpoint."""
    if not isinstance(endpoint, str):
        return set()
    return set(re.findall(r"\b([A-Za-z][A-Za-z0-9_.-]*)\s*\(", endpoint)) - {"r"}


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


def validate_effect_determinism_theorem(data: dict[str, Any], errors: list[dict[str, object]]) -> None:
    ladder = data.get("theorem_ladder")
    theorem = next(
        (item for item in ladder if isinstance(item, dict) and item.get("id") == "THM-EFFECT-DET-001"),
        None,
    ) if isinstance(ladder, list) else None
    if theorem is None:
        return
    expected_certificate = {
        "relation": "effect_correspondence.reduction_determinism",
        "chain": ["frame_lookup", "raise_dispatch"],
        "single_next_configuration": True,
    }
    if (
        theorem.get("scope") != "raise-lookup-dispatch-only"
        or theorem.get("machine_certificate") != expected_certificate
        or "effect fragment is deterministic" in str(theorem.get("claim", "")).lower()
    ):
        errors.append(issue(
            "effect_determinism_theorem_scope_mismatch",
            "THM-EFFECT-DET-001 must certify only the declared Raise/Lookup/Dispatch route",
        ))


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
    definitions = correspondence.get("formal_definitions")
    definitions_valid = isinstance(definitions, dict)
    if isinstance(definitions, dict):
        for definition_name, required_fields in EFFECT_MACHINE_DEFINITIONS.items():
            definition = definitions.get(definition_name)
            if (
                not isinstance(definition, dict)
                or not required_fields.issubset(definition)
                or any(
                    field != "persistent_across_invocation" and not nonempty_string(definition.get(field))
                    for field in required_fields
                )
            ):
                definitions_valid = False
        provider = definitions.get("ProviderFrame")
        if not isinstance(provider, dict) or provider.get("persistent_across_invocation") is not True:
            definitions_valid = False
    if not definitions_valid:
        errors.append(issue(
            "incomplete_effect_machine_definitions",
            "effect machine definitions must make handler/provider frames, affine resumption, and matching explicit",
        ))
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
    if not isinstance(transitions, dict) or any(
        not isinstance(transitions.get(relation_name), dict)
        or transitions[relation_name].get("relation_kind") != "configuration-to-configuration"
        or not nonempty_string(transitions[relation_name].get("source_configuration"))
        or not nonempty_string(transitions[relation_name].get("target_configuration"))
        for relation_name in EFFECT_CONFIGURATION_TRANSITIONS
    ):
        errors.append(issue(
            "invalid_effect_configuration_transition",
            "effect machine transitions must expose configuration-to-configuration endpoints",
        ))
    endpoint_vocabulary = string_list(correspondence.get("endpoint_vocabulary"))
    if endpoint_vocabulary is None or not EFFECT_ENDPOINT_VOCABULARY.issubset(set(endpoint_vocabulary)):
        errors.append(issue(
            "incomplete_effect_endpoint_vocabulary",
            "effect endpoint vocabulary must declare every constructor used by canonical configurations",
        ))
    elif isinstance(transitions, dict):
        for relation_name, expected_vocabulary in EFFECT_TRANSITION_ENDPOINT_VOCABULARY.items():
            relation = transitions.get(relation_name)
            referenced_vocabulary = relation.get("endpoint_vocabulary") if isinstance(relation, dict) else None
            if string_list(referenced_vocabulary) is None or not set(referenced_vocabulary).issubset(endpoint_vocabulary):
                errors.append(issue(
                    "unknown_effect_endpoint_vocabulary",
                    "transition endpoint vocabulary must be declared by the effect machine",
                    transition=relation_name,
                ))
            elif not expected_vocabulary.issubset(set(referenced_vocabulary)):
                errors.append(issue(
                    "incomplete_effect_endpoint_vocabulary",
                    "transition endpoint vocabulary must account for its source and target constructors",
                    transition=relation_name,
                ))
    determinism = correspondence.get("reduction_determinism")
    branch_cases = {
        "handler_entry_selected_frame_removed": "handler",
        "provider_invocation": "provider",
        "missing_discharge": "missing-discharge",
    }
    deterministic_chain_valid = (
        isinstance(determinism, dict)
        and determinism.get("strategy") == "chained-raise-lookup-dispatch"
        and determinism.get("chain") == ["frame_lookup", "raise_dispatch"]
        and determinism.get("raise_dispatch_single_next_configuration") is True
        and isinstance(transitions, dict)
        and isinstance(transitions.get("frame_lookup"), dict)
        and isinstance(transitions.get("raise_dispatch"), dict)
        and transitions["raise_dispatch"].get("source_configuration") == transitions["frame_lookup"].get("target_configuration")
        and all(
            isinstance(transitions.get(relation_name), dict)
            and transitions[relation_name].get("source_configuration") == transitions["raise_dispatch"].get("target_configuration")
            and transitions[relation_name].get("selection_case") == selection_case
            and nonempty_string(transitions[relation_name].get("selection_premise"))
            for relation_name, selection_case in branch_cases.items()
        )
    )
    if not deterministic_chain_valid:
        errors.append(issue(
            "nondeterministic_effect_reduction",
            "Raise must step through Lookup then Dispatch before one tagged selected-frame branch",
        ))
    if isinstance(determinism, dict) and determinism.get("single_next_configuration_per_state") is True:
        sources: dict[str, list[str]] = {}
        if isinstance(transitions, dict):
            for relation_name in EFFECT_CONFIGURATION_TRANSITIONS:
                relation = transitions.get(relation_name)
                source = relation.get("source_configuration") if isinstance(relation, dict) else None
                if nonempty_string(source):
                    sources.setdefault(source, []).append(relation_name)
        overlaps = {source: names for source, names in sources.items() if len(names) > 1}
        if overlaps:
            errors.append(issue(
                "overlapping_effect_transition",
                "a global effect determinism claim requires pairwise-disjoint transition sources",
                overlaps=overlaps,
            ))
        disjointness = determinism.get("disjointness")
        if not isinstance(disjointness, dict) or disjointness.get("method") != "explicit-state-phase-or-mutually-exclusive-premises":
            errors.append(issue(
                "nondeterministic_effect_reduction",
                "a global effect determinism claim requires explicit state-phase disjointness evidence",
            ))
    elif not (
        isinstance(determinism, dict)
        and determinism.get("single_next_configuration_per_state") is False
        and determinism.get("scope") == "raise-lookup-dispatch-only"
    ):
        errors.append(issue(
            "nondeterministic_effect_reduction",
            "effect determinism must be either globally disjoint or explicitly limited to Raise/Lookup/Dispatch",
        ))
    provider_invocation = transitions.get("provider_invocation") if isinstance(transitions, dict) else None
    provider_outcome = transitions.get("provider_external_outcome") if isinstance(transitions, dict) else None
    provider_success = transitions.get("provider_success_resume") if isinstance(transitions, dict) else None
    expected_provider_outcomes = {
        "success": {"target": "r(value)", "retains_configuration_state": True},
        "failure": {"target": "ExternalOutcome(ξ)", "retains_configuration_state": True},
    }
    provider_transition_valid = (
        isinstance(provider_invocation, dict)
        and isinstance(provider_outcome, dict)
        and isinstance(provider_success, dict)
        and provider_outcome.get("source_configuration") == provider_invocation.get("target_configuration")
        and provider_outcome.get("invocation_bindings") == {
            "arguments": "a*", "captured_resume": "r", "provider_frame": "P",
        }
        and provider_outcome.get("frame_persistence") == "preserved"
        and provider_outcome.get("outcomes") == expected_provider_outcomes
        and "ExternalSuccess(value, r)" in str(provider_outcome.get("target_configuration", ""))
        and "ExternalOutcome(ξ)" in str(provider_outcome.get("target_configuration", ""))
        and "Fpre · P · Fpost" in str(provider_outcome.get("target_configuration", ""))
        and "ExternalSuccess(value, r)" in str(provider_success.get("source_configuration", ""))
        and "r(value)" in str(provider_success.get("target_configuration", ""))
        and "Fpre · P · Fpost" in str(provider_success.get("target_configuration", ""))
    )
    if not provider_transition_valid:
        errors.append(issue(
            "lost_provider_resume",
            "provider invocation must retain its arguments, captured resume, and persistent provider frame",
        ))
    expected_terminal_chains = {
        "done": ["done_terminalization", "terminalization"],
        "handler-result": ["handler_result_terminalization", "terminalization"],
        "MissingDischarge": ["missing_discharge_terminalization", "terminalization"],
    }
    terminal_chains = correspondence.get("terminal_successor_chains")
    terminal_chain_valid = terminal_chains == expected_terminal_chains and isinstance(transitions, dict)
    if terminal_chain_valid:
        expected_sources = {
            "done_terminalization": "done(",
            "handler_result_terminalization": "handler-result(",
            "missing_discharge_terminalization": "MissingDischarge(",
        }
        for transition_name, source_constructor in expected_sources.items():
            transition = transitions.get(transition_name)
            terminal_chain_valid = (
                isinstance(transition, dict)
                and source_constructor in str(transition.get("source_configuration", ""))
                and "terminalization" in expected_terminal_chains[next(
                    outcome for outcome, chain in expected_terminal_chains.items() if transition_name in chain
                )]
            )
            if not terminal_chain_valid:
                break
    if not terminal_chain_valid:
        errors.append(issue(
            "incomplete_effect_terminal_successor_chain",
            "done, handler-result, and MissingDischarge configurations require declared terminal-envelope successor chains",
        ))
    expected_provider_routes = {
        "success": {
            "outcome_kind": "success",
            "disposition": "cps-resumption",
            "transitions": ["provider_success_resume"],
            "resumption_target": "r(value)",
            "terminalizing": False,
        },
        "generic_failure": {
            "outcome_kind": "generic_failure",
            "disposition": "external-terminal",
            "transitions": ["provider_failure_terminalization", "generic_failure_terminalization"],
            "terminal_configuration": "Terminal(ExternalOutcome(ξ))",
            "terminalizing": True,
        },
        "timeout": {
            "outcome_kind": "timeout",
            "disposition": "external-terminal",
            "transitions": ["timeout_terminalization", "timeout_external_terminalization"],
            "terminal_configuration": "Terminal(ExternalOutcome(timeout))",
            "terminalizing": True,
        },
        "cancellation": {
            "outcome_kind": "cancellation",
            "disposition": "external-terminal",
            "transitions": ["cancellation_terminalization", "cancellation_external_terminalization"],
            "terminal_configuration": "Terminal(ExternalOutcome(cancelled))",
            "terminalizing": True,
        },
    }
    provider_routes = correspondence.get("provider_outcome_routes")
    provider_route_complete = provider_routes == expected_provider_routes and isinstance(transitions, dict)
    provider_route_continuous = provider_route_complete
    if provider_route_complete:
        success = transitions.get("provider_success_resume")
        provider_route_complete = (
            isinstance(success, dict)
            and success.get("outcome_branch") == "success"
            and success.get("target_configuration") == "⟨r(value), η, κ, α, Fpre · P · Fpost, δ, ρ, ξ⟩"
            and isinstance(success.get("state_phase"), dict)
            and success["state_phase"].get("target") == "provider-success-resume"
            and "provider_success_terminalization" not in transitions
        )
        for route_name, route in expected_provider_routes.items():
            if not route["terminalizing"]:
                continue
            first_name, second_name = route["transitions"]
            first, second = transitions.get(first_name), transitions.get(second_name)
            if not isinstance(first, dict) or not isinstance(second, dict):
                provider_route_complete = False
                continue
            if (
                first.get("relation_kind") != "configuration-to-configuration"
                or second.get("relation_kind") != "configuration-to-configuration"
                or first.get("outcome_kind") != route["outcome_kind"]
                or second.get("outcome_kind") != route["outcome_kind"]
                or not isinstance(first.get("state_phase"), dict)
                or not isinstance(second.get("state_phase"), dict)
            ):
                provider_route_complete = False
            if (
                first.get("target_configuration") != second.get("source_configuration")
                or route["terminal_configuration"] not in str(second.get("target_configuration", ""))
            ):
                provider_route_continuous = False
    if not provider_route_complete:
        errors.append(issue(
            "incomplete_provider_outcome_terminal_chain",
            "provider success/resumption and external outcome routes require their declared typed transitions",
        ))
    elif not provider_route_continuous:
        errors.append(issue(
            "discontinuous_provider_outcome_terminal_chain",
            "external provider outcome terminal transitions must have continuous configuration endpoints",
        ))
    expected_external_projection = {
        "terminal_shape": "Terminal(ExternalOutcome(ξ))",
        "owner": "provider-specific-terminalization",
        "owner_transition": "generic_failure_terminalization",
        "owner_transitions": [
            "generic_failure_terminalization",
            "timeout_external_terminalization",
            "cancellation_external_terminalization",
        ],
    }
    external_projection = correspondence.get("external_projection_contract")
    external_projection_valid = external_projection == expected_external_projection and isinstance(transitions, dict)
    if external_projection_valid:
        owner_transitions = set(expected_external_projection["owner_transitions"])
        generic_terminalization = transitions.get("terminalization")
        if not isinstance(generic_terminalization, dict) or "ExternalOutcome" in str(generic_terminalization.get("source_configuration", "")):
            external_projection_valid = False
        projectors = {
            relation_name
            for relation_name, relation in transitions.items()
            if isinstance(relation, dict)
            and "Terminal(ExternalOutcome" in str(relation.get("target_configuration", ""))
        }
        if projectors != owner_transitions:
            if projectors > owner_transitions:
                errors.append(issue(
                    "duplicate_external_terminal_projection",
                    "only the declared external projection owner may target Terminal(ExternalOutcome(...))",
                    transitions=sorted(projectors),
                ))
            else:
                errors.append(issue(
                    "noncanonical_external_terminal_projection",
                    "every declared external projection owner must preserve the ExternalOutcome wrapper",
                    transitions=sorted(projectors),
                ))
        elif any(
            not isinstance(transitions.get(relation_name), dict)
            or "Terminal(ExternalOutcome" not in str(transitions[relation_name].get("target_configuration", ""))
            for relation_name in owner_transitions
        ):
            errors.append(issue(
                "noncanonical_external_terminal_projection",
                "external terminal projection must retain the ExternalOutcome wrapper",
            ))
    if not external_projection_valid:
        errors.append(issue(
            "noncanonical_external_terminal_projection",
            "external projection policy must name one strict provider-specific canonical owner set",
        ))
    if endpoint_vocabulary is not None and isinstance(transitions, dict):
        declared_vocabulary = set(endpoint_vocabulary)
        for relation_name in EFFECT_CONFIGURATION_TRANSITIONS:
            relation = transitions.get(relation_name)
            if not isinstance(relation, dict):
                continue
            written_constructors = (
                endpoint_constructor_names(relation.get("source_configuration"))
                | endpoint_constructor_names(relation.get("target_configuration"))
            )
            undeclared = sorted(written_constructors - declared_vocabulary)
            if undeclared:
                errors.append(issue(
                    "undeclared_effect_endpoint_constructor",
                    "textual configuration endpoints may not introduce constructors outside the declared vocabulary",
                    transition=relation_name,
                    constructors=undeclared,
                ))
    phase_edges: dict[str, set[str]] = {}
    completion_phase_valid = isinstance(transitions, dict)
    if isinstance(transitions, dict):
        for relation_name, (expected_source, expected_target, required_constructor) in EFFECT_COMPLETION_PHASES.items():
            relation = transitions.get(relation_name)
            phase = relation.get("state_phase") if isinstance(relation, dict) else None
            source_configuration = relation.get("source_configuration") if isinstance(relation, dict) else None
            if (
                isinstance(phase, dict)
                and nonempty_string(phase.get("source"))
                and nonempty_string(phase.get("target"))
                and phase.get("source") != phase.get("target")
            ):
                phase_edges.setdefault(phase["source"], set()).add(phase["target"])
            if (
                not isinstance(phase, dict)
                or phase.get("source") != expected_source
                or phase.get("target") != expected_target
                or expected_source == expected_target
                or required_constructor not in str(source_configuration)
            ):
                completion_phase_valid = False
                continue
    if not completion_phase_valid:
        errors.append(issue(
            "invalid_effect_completion_phase",
            "completion and terminal routes must use their declared phase-separated source and target states",
        ))
    def phase_graph_has_cycle(phase: str, visiting: set[str], visited: set[str]) -> bool:
        if phase in visiting:
            return True
        if phase in visited:
            return False
        visiting.add(phase)
        cyclic = any(phase_graph_has_cycle(target, visiting, visited) for target in phase_edges.get(phase, set()))
        visiting.remove(phase)
        visited.add(phase)
        return cyclic
    if any(phase_graph_has_cycle(phase, set(), set()) for phase in phase_edges):
        errors.append(issue(
            "cyclic_effect_transition_graph",
            "completion and terminal state phases must form an acyclic successor graph",
        ))
    handler_body_trap = transitions.get("handler_body_trap") if isinstance(transitions, dict) else None
    handler_trap_phase = handler_body_trap.get("state_phase") if isinstance(handler_body_trap, dict) else None
    if (
        not isinstance(handler_body_trap, dict)
        or handler_body_trap.get("relation_kind") != "configuration-to-configuration"
        or not isinstance(handler_trap_phase, dict)
        or handler_trap_phase.get("source") != "handler-body-trap"
        or handler_trap_phase.get("target") != "terminal-ready"
        or "HandlerBodyTrap" not in str(handler_body_trap.get("source_configuration", ""))
        or "TerminalReady(Trap(" not in str(handler_body_trap.get("target_configuration", ""))
    ):
        errors.append(issue(
            "invalid_handler_body_trap_transition",
            "handler-body traps must be phase-separated from terminal trap projection and cannot self-loop",
        ))
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
    validate_effect_determinism_theorem(data, errors)
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
