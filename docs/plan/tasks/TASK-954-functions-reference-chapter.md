# TASK-954: Expand functions reference chapter

## Status: ✅ Complete

## Description

Turn the Phase 124 `reference/language/functions.md` pilot page into a usable daily reference chapter for pure functions. The chapter must explain the concept directly, split detailed sections into sub-pages, include examples for each supported syntax shape, and give the agent card enough operational detail for implementation agents to use Ash functions without treating old working docs as the primary reader surface.

## Specification Reference

- [SPEC-027](../../spec/SPEC-027-PURE-FUNCTIONS.md): Pure Functions
- [SPEC-031](../../spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md): First-Class Functions and Closure Values
- [SPEC-071](../../spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md): Reference Corpus Metadata and Maintenance
- PLAN-INDEX Phase 125

## Dependencies

- ✅ TASK-953: Reference corpus closeout and drift report

## Requirements

### Functional Requirements

1. Keep `reference/language/functions.md` as the canonical functions page and convert it into a chapter index with Summary, Concept, Status, and sub-page TOC.
2. Add `reference/language/functions/` sub-pages for module declarations, body/expression syntax, local/anonymous functions, calls/function values, pattern matching, boundaries, implementation notes, and authority.
3. Include concrete Ash examples for each syntax variation documented.
4. Distinguish pure functions, local closures, builtin functions, and effectful `Act`-returning functions without blurring the Pure < Act < Proc < Workflow tower.
5. Update `reference/agents/cards/functions.md` so an agent can use pure functions from the card after checking the canonical page.
6. Preserve `docs/` as working/historical evidence; make the reference chapter useful without requiring readers to follow spec cross-references for ordinary use.

### Non-goals

- Do not implement new function syntax.
- Do not claim full closure/runtime maturity beyond the cited alpha evidence.
- Do not migrate the whole reference corpus.

## Work Steps

1. Inspect current parser/typechecker/spec evidence for function syntax.
2. Draft the chapter index and sub-pages using Ash reference style.
3. Add examples with honest labels where behavior is reference-level or alpha-scoped.
4. Update the agent card with usable syntax, boundaries, and preflight checks.
5. Run reference validator and markdown diff checks.
6. Request independent documentation review before merge.

## Dispatch

```yaml
agent: codex
reasoning: high
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 -m py_compile tools/reference/check_frontmatter.py
  - python3 tools/reference/check_frontmatter.py
  - python3 tools/reference/check_frontmatter.py --pilot
checklist:
  - [x] Functions chapter index created.
  - [x] Section sub-pages created under `reference/language/functions/`.
  - [x] Agent card updated with usable function-reference details.
  - [x] CHANGELOG.md updated.
  - [x] Reference metadata and links validated.
```

## Completion Notes

Completed as a reference-only documentation slice. The chapter now serves human and agent readers directly while preserving specs and code as authority links rather than forcing daily readers through the working corpus.
