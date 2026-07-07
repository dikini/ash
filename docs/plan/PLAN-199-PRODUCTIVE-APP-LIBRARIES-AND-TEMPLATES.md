# PLAN-199: Productive App Libraries And Templates

**Status:** ✅ Complete (9/9 tasks complete)
**Depends on:** Phase 198 Standard Providers And Profiles.
**Specs/notes:** `PLAN-198`, `PLAN-197`, `PLAN-196`, `PLAN-195`, `PLAN-194`, `SPEC-096b`,
`SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`, `SPEC-081`, `SPEC-082`,
`SPEC-083`, `SPEC-084`, and `NOTE-035`.

## Goal

Turn the standard provider/profile foundation into productive Ash application libraries, testing
helpers, process/channel helpers, app templates, and tutorial-quality examples.

## Architecture

Phase 199 is a usability layer over current target Ash. It starts with a current-syntax audit and
remediation pass over existing stdlib modules, examples, and template-like files, because templates
must teach the current language rather than preserve historical syntax. After that audit, the phase
adds higher-level libraries and templates that compose Phase 198 provider/profile boundaries without
adding new authority paths.

## Scope

- Review stdlib modules, examples, and template-like assets for stale or historical syntax.
- Revise productive libraries to current target syntax where required.
- Add testing helper libraries over the existing Ash test, QuickCheck, law/evidence, coverage, and
  flake orchestration substrate.
- Add process/channel convenience libraries over Phase 195 semantics.
- Define app template metadata, validation, instantiation behavior, and conformance gates.
- Add canonical templates for CLI tools, file pipelines, HTTP fetch/process apps, supervised
  workers, and provider-profile test apps.
- Add tutorial-quality examples and docs that use current target syntax.

## Non-Goals

- No new language syntax.
- No new host provider family beyond Phase 198 providers.
- No template that depends on legacy `workflow` syntax as a target primitive.
- No bypass of provider/profile admission, sandboxing, sendability, contracts, or evidence.
- No package registry or marketplace workflow in this phase.

## Design Locks

1. The first implementation task after the packet must audit and revise libraries/templates to
   current syntax before new templates are accepted.
2. Templates are generated examples of current Ash application structure, not special runtime
   privileges.
3. Testing libraries must reuse existing law/evidence/QuickCheck/test-runner substrates rather than
   creating a parallel test mechanism.
4. Process/channel helpers must preserve sendability, ownership, cancellation, failure propagation,
   and trace evidence.
5. Template gates must run parse/check/run or artifact assertions so a template cannot silently rot.

## Task Overview

| Task | Description | Estimate | Depends on | Status |
|------|-------------|----------|------------|--------|
| [TASK-1942](tasks/TASK-1942-productive-app-libraries-templates-plan-packet.md) | Create the Phase 199 plan and task packet | 2h | Phase 198 | ✅ Complete |
| [TASK-1943](tasks/TASK-1943-current-syntax-library-template-audit-remediation.md) | Review and revise libraries, examples, and template-like files to current syntax | 12h | TASK-1942 | ✅ Complete |
| [TASK-1944](tasks/TASK-1944-testing-helper-libraries.md) | Add testing helper libraries over QuickCheck, law/evidence, coverage, and flake orchestration | 14h | TASK-1943 | ✅ Complete |
| [TASK-1945](tasks/TASK-1945-process-channel-convenience-library.md) | Add process/channel convenience helpers over Phase 195 semantics | 14h | TASK-1943 | ✅ Complete |
| [TASK-1946](tasks/TASK-1946-app-template-manifest-and-validation.md) | Define app template manifest/schema and validation model | 10h | TASK-1943 | ✅ Complete |
| [TASK-1947](tasks/TASK-1947-template-instantiation-cli.md) | Add CLI/template instantiation path with fail-closed diagnostics | 12h | TASK-1946 | ✅ Complete |
| [TASK-1948](tasks/TASK-1948-canonical-app-template-corpus.md) | Add canonical current-syntax app templates | 16h | TASK-1947 | ✅ Complete |
| [TASK-1949](tasks/TASK-1949-tutorial-examples-and-template-docs.md) | Add tutorial examples and template docs tied to executable gates | 10h | TASK-1948 | ✅ Complete |
| [TASK-1950](tasks/TASK-1950-productive-app-libraries-templates-closeout.md) | Close out Phase 199 with cross-template gates, docs, changelog, and review remediation | 8h | TASK-1949 | ✅ Complete |

Estimated implementation effort after the plan packet: 96 hours.

## Required Test Families

### Current-Syntax Audit Tests

Every productive stdlib module, example, and template-like file selected for Phase 199 must be
classified as current executable syntax, current reference syntax, historical/reference-only, or
removed from the productive path. Files promoted to productive templates must parse/check through
the real CLI or engine path.

### Testing Library Tests

Testing helpers must cover ordinary assertions, property helpers, generated evidence, law/evidence
integration, counterexample artifacts, coverage, mutation, flake quarantine, and deterministic
provider profiles where applicable.

### Process/Channel Library Tests

Process/channel helpers must preserve sendability validation, ownership transfer, cancellation,
failure propagation, channel close/empty/full diagnostics, and trace evidence.

### Template Tests

Templates must instantiate into parseable/checkable projects or fixtures. Canonical templates must
include at least CLI tool, file pipeline, HTTP fetch/process, supervised worker, and provider-profile
test app shapes.

## Verification Gates

Each implementation task must run focused tests for touched crates plus:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
```

Phase closeout must run:

```bash
cargo fmt --check
cargo test --all
cargo clippy --all-targets --all-features
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```

## Stale-Claim Sweep

Before closeout, search live docs, examples, templates, and stdlib comments for stale claims:

```text
template.*workflow
example.*legacy workflow
library.*old syntax
Proc<|Act<|Workflow<
capability.*direct provider
test helper.*parallel mechanism
channel.*untyped
process.*no sendability
template.*unchecked
```

Historical/reference-only files may keep old syntax only when explicitly excluded from productive
template and tutorial paths.

## Acceptance Criteria

- [x] Phase 199 plan and task files exist and are indexed.
- [x] Productive libraries, examples, and template-like files are audited and revised to current
      syntax where required.
- [x] Testing helper libraries compose with existing test/evidence/QuickCheck/law substrates.
- [x] Process/channel convenience helpers preserve Phase 195 invariants.
- [x] App template manifest/schema and validation are implemented.
- [x] CLI/template instantiation fails closed and emits structured diagnostics.
- [x] Canonical templates instantiate into current-syntax apps with provider/profile boundaries.
- [x] Tutorial docs and examples are tied to executable gates.
- [x] Cross-template gates, changelog, docs gates, and review remediation complete the phase.
