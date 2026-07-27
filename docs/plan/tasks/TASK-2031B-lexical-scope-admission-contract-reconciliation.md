# TASK-2031B: Lexical-Scope Admission Contract Reconciliation

**Status:** Complete
**Phase:** [PLAN-203](../PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md)
**Type:** Bounded test-contract remediation
**Depends on:** TASK-2003's bounded `PureAnf` lowering and TASK-2004/TASK-2014's Path-B closed
admission boundary
**Blocks:** TASK-2031A and TASK-2031 workspace-gate closeout only

## Description

Two legacy lexical-scope CLI tests require an obsolete atomic-let-only rejection string. The
canonical bounded `PureAnf` documents now admit selected computed lets and retain a generic
checked Core-to-CPS bridge-domain rejection for unsupported forms. Reconcile those assertions to
the canonical fail-closed admission boundary.

## Semantic rule and bounded domain

**Canonical owners:** `SEM-TARGET-CORE-CPS-001`, TASK-2003 § bounded typed `PureAnf`, and
TASK-2004/TASK-2014 Path-B closed admission.

**Declared domain:** **bounded** to the two existing lexical-scope integration assertions. It
changes neither source acceptance nor Type → Core → CPS lowering, admission authority, runtime,
terminal projection, or CLI/daemon parity. It records existing negative evidence only.

**Handoffs:** Consumes the existing bounded checked Core/CPS admission result from TASK-2003 and
the closed public-route boundary from TASK-2004/TASK-2014. Produces corrected CLI negative
evidence for that boundary. General lexical-scope lowering remains owned by TASK-2003/TASK-2004;
PLAN-203 integration owns client route parity, and no proof obligation transfers here.

## Requirements

1. Preserve successful `ash check` behavior for both valid lexical-scope sources.
2. Preserve nonzero `ash run` and `ash trace` behavior; assert the canonical generic checked
   Core-to-CPS bridge-domain rejection for `run` and the existing missing-typed-lowering rejection
   for `trace`.
3. Do not alter production Rust, parser/typechecker/lowering behavior, Engine routing, admission
   authority, or terminal envelopes.
4. Prove the focused lexical-scope target and the workspace Rust gate pass.

## TDD steps

1. **RED:** Add focused assertions that name the canonical generic bridge-domain message in the
   two controls; demonstrate failure while the legacy atomic-let-only expectation remains.
2. **GREEN:** Replace only the obsolete shared run-admission expectation.
3. **QA/review:** Run the focused target, workspace tests, formatter, Clippy, docs gate, and
   independent review.

## Completion checklist

**Completion evidence:** The focused lexical-scope target passed 6/6 with the canonical shared
run-admission message. The recorded workspace Rust gate passed; historical change `de4043d8`
contains documentation and test assertions only, with no production Rust change. This bounded
remediation makes no source, lowering, admission, runtime, terminal, or client-parity claim.

- [x] The focused RED proved the legacy message was stale.
- [x] Both lexical-scope controls assert the current bounded fail-closed admission boundary.
- [x] No production route or semantic domain was broadened.
- [x] Workspace Rust tests, formatter, Clippy, and docs gate passed.
- [x] CHANGELOG, plan index, and task evidence are updated; QA and review are recorded.
