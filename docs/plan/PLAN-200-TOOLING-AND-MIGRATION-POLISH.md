# PLAN-200: Tooling And Migration Polish

**Status:** In Progress (6/9 tasks complete)
**Depends on:** Phase 199 Productive App Libraries And Templates.
**Specs/notes:** `PLAN-199`, `PLAN-198`, `PLAN-196`, `PLAN-195`, `SPEC-096b`,
`SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`, and `NOTE-035`.

## Goal

Make current Ash the obvious path in tooling, examples, and docs by eliminating or explicitly
demoting legacy and deprecated forms from every productive surface.

## Architecture

Phase 200 is a migration-first polish layer over the current parser, checker, formatter, LSP,
examples, and documentation. It starts with an inventory of old syntax and deprecated forms, then
uses that inventory to drive diagnostics, formatter behavior, LSP behavior, example refreshes, docs
updates, and final removal/demotion gates. The phase treats legacy/deprecated form elimination as
the central product goal, not a cleanup item after tooling work.

## Scope

- Inventory all legacy and deprecated syntax/forms in productive code, examples, docs, diagnostics,
  formatter tests, LSP tests, templates, and migration fixtures.
- Improve diagnostics for old syntax so users get precise migration guidance instead of generic
  parser/type errors.
- Polish formatter coverage for current target syntax and prevent formatting from preserving or
  normalizing deprecated forms in productive paths.
- Polish LSP diagnostics, hover/symbols, semantic tokens, and navigation for current syntax while
  surfacing old forms as migration diagnostics.
- Refresh examples so productive examples are current syntax and legacy examples are explicitly
  historical, compatibility-only, or removed.
- Refresh docs/tutorials/reference paths so they teach current syntax first and quarantine old
  syntax in migration notes only.
- Remove, demote, or fail-closed old syntax from productive gates.

## Non-Goals

- No new language syntax.
- No new provider/runtime authority model.
- No semantic rewrite of current target Ash.
- No broad editor feature expansion beyond migration-relevant LSP polish.
- No package registry or marketplace workflow.

## Design Locks

1. Legacy and deprecated forms must not appear in productive examples, templates, tutorials, or
   default docs except as explicitly labeled migration material.
2. Diagnostics are the migration entrypoint: every retained old-form parser/checker path must either
   emit targeted migration guidance or be quarantined as a compatibility test.
3. Formatter and LSP behavior must follow parser/typechecker truth and must not duplicate syntax
   policy with ad hoc string rules except in tests that guard known migration patterns.
4. Productive examples and docs must have executable or artifact gates so old syntax cannot drift
   back into the happy path.
5. Compatibility support is allowed only when labeled, tested, and excluded from current-syntax
   teaching surfaces.

## Task Overview

| Task | Description | Estimate | Depends on | Status |
|------|-------------|----------|------------|--------|
| [TASK-1951](tasks/TASK-1951-tooling-migration-polish-plan-packet.md) | Create the Phase 200 plan and task packet | 2h | Phase 199 | ✅ Complete |
| [TASK-1952](tasks/TASK-1952-legacy-deprecated-form-audit.md) | Audit diagnostics, formatter, LSP, examples, docs, and old-form productive paths | 12h | TASK-1951 | ✅ Complete |
| [TASK-1953](tasks/TASK-1953-migration-diagnostics.md) | Improve stale/deprecated syntax diagnostics and migration hints | 16h | TASK-1952 | ✅ Complete |
| [TASK-1954](tasks/TASK-1954-formatter-current-syntax-polish.md) | Polish formatter coverage for current target syntax and old-form quarantine | 14h | TASK-1952 | ✅ Complete |
| [TASK-1955](tasks/TASK-1955-lsp-current-syntax-migration-polish.md) | Polish LSP diagnostics, hover, symbols, semantic tokens, and navigation for current syntax | 16h | TASK-1952 | ✅ Complete |
| [TASK-1956](tasks/TASK-1956-examples-current-syntax-refresh.md) | Refresh examples corpus and classify or remove legacy examples | 12h | TASK-1952 | ✅ Complete |
| [TASK-1957](tasks/TASK-1957-docs-current-syntax-refresh.md) | Refresh docs/tutorials/reference paths around current syntax and migration notes | 12h | TASK-1956 | Planned |
| [TASK-1958](tasks/TASK-1958-old-syntax-removal-demotion.md) | Remove or demote old syntax from productive paths with fail-closed gates | 18h | TASK-1953, TASK-1954, TASK-1955, TASK-1956, TASK-1957 | Planned |
| [TASK-1959](tasks/TASK-1959-tooling-migration-polish-closeout.md) | Close out Phase 200 with full gates, stale-claim sweep, docs, and review remediation | 8h | TASK-1958 | Planned |

Estimated implementation effort after the plan packet: 108 hours.

## Required Test Families

### Legacy/Demoted Form Inventory Tests

Every productive old-form occurrence must be classified as removed, migrated, compatibility-only,
historical-reference-only, or retained with a targeted migration diagnostic. Inventory tests must
fail when a productive path gains an unclassified deprecated form.

### Migration Diagnostic Tests

Diagnostics must cover stale forms such as old workflow entry syntax, legacy `act ... with` and
`observe ... with` spelling, removed tower carrier spellings, old capability/provider language,
legacy callable arrows where no longer accepted, and stale docs examples. Expected output should
include a stable diagnostic code, source span, concise explanation, and migration hint when a
current spelling exists.

### Formatter Tests

Formatter tests must cover current syntax for functions, do blocks, rows, contracts/evidence,
providers/profiles, process/channel helpers, testing helpers, templates, records, matches, and
imports. Formatter gates must avoid canonicalizing deprecated forms into productive output unless
the test is explicitly compatibility-only.

### LSP Tests

LSP tests must verify current-syntax diagnostics, hover/symbols, semantic tokens, definition and
reference navigation, and migration hints for deprecated forms. The LSP must not report current
target syntax as legacy merely because compatibility syntax still exists in parser tests.

### Examples And Docs Tests

Examples and docs gates must prove productive examples, templates, tutorials, and getting-started
paths use current syntax. Historical docs may retain old syntax only with explicit labels and
links to migration guidance.

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

Before closeout, search live docs, examples, templates, formatter fixtures, LSP fixtures, parser
fixtures, and stdlib comments for stale or undemoted old-form claims:

```text
legacy workflow
old syntax
deprecated syntax
observe .* with
act .* with
Proc<|Act<|Workflow<
capability.*direct provider
ambient authority
formatter.*legacy
LSP.*legacy
example.*compatibility
template.*workflow
```

Historical/reference-only and compatibility-only files may keep old syntax only when explicitly
labeled and excluded from productive tutorial, template, example, formatter, and LSP happy paths.

## Acceptance Criteria

- [x] Phase 200 plan and task files exist and are indexed.
- [x] Legacy and deprecated forms are inventoried across diagnostics, formatter, LSP, examples,
      docs, templates, parser fixtures, and stdlib comments.
- [ ] Productive old-form occurrences are removed, migrated, or explicitly demoted.
- [ ] Diagnostics provide targeted migration hints for retained/deprecated forms.
- [ ] Formatter gates prefer current target syntax and quarantine old-form behavior.
- [ ] LSP gates prefer current target syntax and surface migration diagnostics where applicable.
- [ ] Productive examples and templates use current syntax and have executable gates.
- [ ] Productive docs/tutorials teach current syntax and quarantine old syntax in migration notes.
- [ ] Closeout gates, changelog, docs gates, stale-claim sweep, and review remediation complete the
      phase.
