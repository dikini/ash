# PLAN-118: DESIGN-040/041 Alpha Implementation Packet

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Start with TASK-920 audit before Rust changes; downstream implementation tasks deliberately fail closed until TASK-920 patches exact focused evidence commands.

**Goal:** Promote DESIGN-040 and DESIGN-041 into an implementation-grade alpha packet covering visible tower algebra, generalized do lowering, traceable execution artifacts, and a one-kernel/two-host-mode runtime regime.

**Architecture:** SPEC-069 owns language semantics: visible `Monad<K>` evidence, full `do:K` bind lowering, tower algebra, TCIR/AMIR/bytecode traceability, and OODA demotion. SPEC-070 owns OS-facing runtime hosting: `RuntimeKernel`, `ash run`, local `ashd`, roots, admission, artifacts, reload, and control surface.

**Tech Stack:** Rust workspace (`ash-core`, `ash-parser`, `ash-typeck`, `ash-engine`, `ash-interp`, `ash-cli`), Markdown specs/plans/tasks, repo-owned serial verification (`scripts/check-rust-tests.sh`), and scoped docs gates.

---

## 1. Status

**Status:** 🚧 In progress / implementation underway after completed TASK-920 audit gate
**Spec:** [SPEC-069](../spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md), [SPEC-070](../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)
**Design:** [DESIGN-040](../design/DESIGN-040-ALPHA-ALGEBRAIC-TOWER.md), [DESIGN-041](../design/DESIGN-041-RUNTIME-REGIME-AND-OS-SURFACE.md)
**Task range:** [TASK-919](tasks/TASK-919-design040041-current-state-and-scope-reconciliation.md) through [TASK-932](tasks/TASK-932-alpha-closeout-review-remediation.md)

TASK-919 is the completed documentation-packet task. TASK-920 completed the hard pre-implementation audit gate and bound exact callsites plus zero-test-safe focused commands. TASK-921 added the public tower manifest, and TASK-922 added selected Monad evidence method-body/shim carriers at the do-target boundary. TASK-923 through TASK-930 remain planned implementation/compatibility tasks. TASK-931/TASK-932 close acceptance and review.

## 2. Current-state reconciliation

| Substrate | Current owner | PLAN-118 implication |
| --- | --- | --- |
| explicit `do:K` syntax and Act/Proc MVP dictionaries | [SPEC-054](../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) | keep parser surface; replace bridge-only lowering with evidence-selected full bind lowering |
| first-class `Workflow<A>` carrier | [SPEC-056](../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md) | preserve workflow construction artifacts when unifying tower algebra |
| partial-constructor holes such as `Result<_, E>` | [SPEC-066](../spec/SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md) | consume existing target-shape elaboration; do not re-spec holes |
| constructor-kinded binders and `Monad<K>` evidence lookup | [SPEC-067](../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md) | extend from return-only/target evidence into full method-body bind lowering |
| process/runtime/failure/workflow semantics | [SPEC-049](../spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md), [SPEC-050](../spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md), [SPEC-051](../spec/SPEC-051-WORKFLOW-SEMANTICS.md) | update authority handoffs without replacing each spec's focused semantics |
| CLI/runtime observable behavior | [SPEC-005](../spec/SPEC-005-CLI.md), [SPEC-021](../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md) | add RuntimeKernel/host-mode semantics without weakening existing `ash run` behavior |

## 3. Task table

| Task | Description | Est. Hours | Status |
| --- | --- | ---: | --- |
| [TASK-919](tasks/TASK-919-design040041-current-state-and-scope-reconciliation.md) | Promote DESIGN-040/041 into SPEC-069/SPEC-070/PLAN-118 and register Phase 122 | 4 | ✅ Complete |
| [TASK-920](tasks/TASK-920-alpha-visible-tower-audit-gate.md) | Audit live do/evidence/tower/runtime/CLI seams and patch downstream exact evidence commands | 8 | ✅ Complete |
| [TASK-921](tasks/TASK-921-public-tower-stdlib-manifest.md) | Public `Act`/`Proc`/`Workflow`/`Result` algebra manifest and no-magic intrinsic mapping | 8 | ✅ Complete |
| [TASK-922](tasks/TASK-922-monad-evidence-method-body-lowering.md) | TypeEnv/evidence carriers for selected `Monad<K>` operation bodies or intrinsic shims | 10 | ✅ Complete |
| [TASK-923](tasks/TASK-923-generalized-do-full-bind-lowering.md) | Full generalized `do:K` `<-` lowering through selected Monad evidence | 12 | ✅ Complete |
| [TASK-924](tasks/TASK-924-act-proc-workflow-opaque-carrier-alignment.md) | Align Act/Proc/Workflow opaque carrier semantics with visible algebra and explicit lifts | 10 | 📝 Planned |
| [TASK-925](tasks/TASK-925-tcir-computation-expression-boundary.md) | Typed computation-expression/TCIR carrier with source/evidence/tower provenance | 12 | 📝 Planned |
| [TASK-926](tasks/TASK-926-amir-bytecode-logical-schema.md) | Minimal AMIR and bytecode logical schema plus verifier/debug traceability contract | 14 | 📝 Planned |
| [TASK-927](tasks/TASK-927-runtime-kernel-host-mode-audit-and-carriers.md) | RuntimeKernel audit and core host-mode/definition/instance identity carriers | 10 | 📝 Planned |
| [TASK-928](tasks/TASK-928-ash-run-runtime-kernel-mode.md) | Route one-shot `ash run` through RuntimeKernel without requiring daemon state | 12 | 📝 Planned |
| [TASK-929](tasks/TASK-929-ashd-local-daemon-control-plane.md) | Local `ashd` daemon control surface, roots, instance table, and reload semantics | 16 | 📝 Planned |
| [TASK-930](tasks/TASK-930-ooda-library-demotion-compatibility.md) | OODA library/template/lint compatibility and primitive-IR demotion plan | 8 | 📝 Planned |
| [TASK-931](tasks/TASK-931-alpha-semantics-correspondence-and-acceptance-matrix.md) | SPEC-069/SPEC-070 acceptance matrix, semantics correspondence, and non-interference evidence | 8 | 📝 Planned |
| [TASK-932](tasks/TASK-932-alpha-closeout-review-remediation.md) | Closeout docs, broad gates, and independent review remediation | 6 | 📝 Planned |

## 4. Track structure

- **Track A — Packet and audit (12h):** TASK-919/TASK-920 establish docs, authority handoffs, live call graph, and downstream focused commands.
- **Track B — Visible algebra and do lowering (40h):** TASK-921 through TASK-924 make tower operations visible and lower arbitrary accepted `do:K` binds through selected evidence.
- **Track C — Traceable execution artifacts (26h):** TASK-925/TASK-926 introduce typed computation-expression, TCIR, AMIR, and bytecode schema contracts.
- **Track D — Runtime regime (38h):** TASK-927 through TASK-929 introduce `RuntimeKernel`, route `ash run`, and add local `ashd` semantics. Final artifact/version equivalence depends on TASK-925/TASK-926; any earlier runtime work must record an explicit interim source/check-summary identity substrate.
- **Track E — Compatibility, acceptance, review (22h):** TASK-930 through TASK-932 demote OODA, prove acceptance/non-interference, and remediate review findings.

## 5. Decision gates

- **D1:** Visible algebra is the normative construction authority. Runtime intrinsics are implementations of named/typeable operations, not extra semantic roots.
- **D2:** `Monad<K>` evidence owns generalized do sequencing. Hidden Act/Proc/Workflow dictionaries may remain only as compiler-prelude evidence shaped like ordinary `Monad<K>` entries during migration.
- **D3:** Full generalized user/library Monad `<-` lowering is alpha scope. It must not be downgraded to beta-only without a new design decision.
- **D4:** `Result<_, E>` is the canonical partial-constructor do-target acceptance case; domain failure remains separate from operational bottom.
- **D5:** No implicit tower lifts. `Act` inside `Proc`, and `Proc` inside `Workflow`, require explicit visible operations.
- **D6:** Runtime execution has one semantic `RuntimeKernel` with two host modes: one-shot `ash run` and local daemon `ashd`.
- **D7:** File presence does not execute code. Definitions are indexed; workflow instances start only by explicit run/start/autostart policy.
- **D8:** Provider/resource existence is not authority. Admission grants capability/resource authority to a workflow instance.
- **D9:** Existing running daemon instances keep their admitted artifact/version across reload; successful reload affects future starts only.
- **D10:** OODA moves to library/template/lint surface by default. Alpha AMIR/bytecode must not depend on OODA as a privileged primitive.
- **D11:** AMIR/bytecode artifacts must be verifier-checkable and source-traceable without reparsing source.
- **D12:** TASK-920 is a hard gate requiring downstream exact file/test/callsite bindings and zero-test-safe focused commands.

## 6. Supersession and authority update map

| Existing artifact | PLAN-118 update |
| --- | --- |
| DESIGN-040 | Promoted by SPEC-069; remains rationale/design context. |
| DESIGN-041 | Promoted by SPEC-070; remains rationale/design context. |
| SPEC-047 | Still owns Act public carrier/effectful computation; SPEC-069 supersedes Act-specific magic sequencing implications. |
| SPEC-048/SPEC-049 | Still own Proc library/runtime details; SPEC-069 owns no-magic algebra handoff and SPEC-070 owns host-runtime boundaries. |
| SPEC-050 | Still owns operational bottom/failure; SPEC-069 clarifies `fail` is not domain failure for `Result`/`Option` do targets. |
| SPEC-051/SPEC-056 | Still own workflow governance/carrier details; SPEC-069 owns shared tower algebra and TCIR/AMIR lowering requirements. |
| SPEC-054 | Still owns do syntax/grammar; SPEC-069 extends target from MVP hidden dictionaries to full evidence-selected `bind` lowering. |
| SPEC-066/SPEC-067 | Remain implemented prerequisites; SPEC-069 consumes hole/HKT/evidence substrate. |
| SPEC-001/SPEC-004/SPEC-025 | Future patches must align IR/semantics with TCIR/AMIR/bytecode correspondence and OODA demotion. |
| SPEC-005/SPEC-021 | SPEC-070 adds RuntimeKernel/host-mode semantics while preserving CLI/observable compatibility. |

## 7. Verification strategy

TASK-920 must record the live call graph and replace fail-closed downstream guards with exact commands. The phase must eventually verify parser surface fidelity, typechecker evidence identity, `do:Result<_, E>` and user `Monad<Option>` binds, explicit lift diagnostics, TCIR/AMIR/bytecode provenance, shared `ash run`/`ashd` RuntimeKernel semantics, admission/resource authority, OODA ordinary-library behavior, scoped markdown links, and broad Rust gates.

## 8. Closeout criteria

The phase is not complete until SPEC-069 and SPEC-070 are updated to Implemented MVP or honest partial status; every status surface matches; A69/A70 acceptance rows map to evidence; old authority claims are patched; `git diff --check`, docs links, fmt, clippy, docs, and repo-owned serial tests have fresh evidence; and independent review findings are patched and re-reviewed.
