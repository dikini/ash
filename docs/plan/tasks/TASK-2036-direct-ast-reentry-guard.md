# TASK-2036: Direct AST Re-entry Guard

**Status:** Planned
**Semantic task classification:** non-semantic-workflow-enforcement
**Phase:** [PLAN-204](../PLAN-204-DIRECT-AST-RETIREMENT-AUDIT-AND-CONTRACT-FREEZE.md)
**Depends on:** TASK-2034 and TASK-2035

## Description

Add a deterministic repository gate using the frozen TASK-2034 manifest. Until Phase 205 closes,
the gate permits only manifest-listed Rust legacy locations and rejects new direct AST evaluation,
public non-Engine CPS execution, differential-oracle reachability, a Lean reference presented as
current Ash execution authority, or client-local execution in run, daemon, test, and REPL code.
It permits preserved Lean material only under TASK-2034's deferred separate-project label. It scans exactly the manifest's
declared source/document/workflow roots and reports the matching manifest ID or unknown location.

## Requirements

- Reject unlisted Rust direct-AST, independent-CPS, differential, and client-local evaluator use.
- Permit Lean only as a manifest-listed deferred separate-project reference; reject current-Ash
  authority or executable-route wording for it.
- Keep the guard deterministic and manifest-scoped; it must not synthesize cases or infer new
  evaluator domains.
- Record gate tests as workflow evidence only, never as target-runtime implementation or parity.

## Handoffs

- **Run-route impact:** `prerequisite`.
- **Consumes:** TASK-2034 manifest and TASK-2035's single-executor contract words.
- **Produces:** a re-entry gate and allowlist ownership boundary for every Phase-205 task.
- **Downstream owner:** TASK-2041 converts the allowlist requirement to a zero-current-use gate.
- **Does not own:** deletion, AST/CPS implementation, or semantic rule realization.
- **Integration/proof responsibility:** TASK-2041 proves final zero use; this task proves only
  that new unlisted use cannot enter during migration.

## TDD and verification steps

1. Add failing fixtures for a new AST evaluator call, a public CPS evaluator export, a test-runner
   call, a REPL call, and stale current documentation; retain a manifest-listed control.
2. Implement deterministic scanning and manifest resolution.
3. Add negative fixtures for duplicate/stale allowlist IDs and a historical document mislabeled as
   current.
4. Install the gate in the documented local validation path and run its self-tests/docs gate.

## Completion checklist

- [ ] New legacy use fails closed with an actionable manifest-ID diagnostic.
- [ ] Existing listed material remains visible as migration debt, not as approved architecture.
- [ ] The gate has no wildcard or generated-case exemption.
- [ ] CHANGELOG and planning indexes record the guard without claiming removal or parity.
