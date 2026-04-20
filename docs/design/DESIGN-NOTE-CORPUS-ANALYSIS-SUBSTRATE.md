# DESIGN-NOTE: Shared Document / Corpus Analysis Substrate

## Status

Draft design note for practical reuse between the Ash wiki, the spec processor, and related linting tools.

## Goal

Factor out a shared document/corpus analysis substrate beneath multiple products without merging those products at the interface or policy layer.

The immediate consumers are:

- the Ash wiki knowledge substrate (`DESIGN-029`, `SPEC-045`)
- the spec processor / Phase 90 repository audit workflow (`DESIGN-SPEC-PROCESSOR`, `PLAN-090`)
- the Ash lint ecosystem (`SPEC-041`) where code-lint outputs may need compatible evidence/reporting surfaces even though source-code linting remains a distinct product

## Core Decision

Keep the products separate:

1. The spec processor remains a repository-audit / CI-facing product.
2. The Ash wiki remains a corpus-semantic, navigation, audit, onboarding, and query/service product.
3. `ash-lint` remains the Ash source-code lint product.

But factor out a shared substrate underneath for document/corpus analysis.

## Why a Shared Substrate Is Worth It

Without a shared substrate, the project will duplicate:

- repository file discovery
- markdown/frontmatter extraction
- document identity normalization
- link/task/spec/plan reference extraction
- relationship graph construction
- evidence/finding formatting

The spec processor already needs these for broken links, example conformance, PLAN-INDEX coherence, and changelog checks. The wiki needs the same foundations for registries, computed views, supersession maps, drift dashboards, onboarding bundles, and query services.

## Shared Substrate Scope

The shared substrate should own reusable analysis primitives, not product-specific policy.

### A. Corpus Discovery

Responsibilities:

- scan managed repository paths
- classify artifacts by path/convention (`spec`, `design`, `plan`, `task`, `reference`, `idea`, etc.)
- extract stable repository-relative identity
- normalize task/spec/plan references

### B. Document Extraction

Responsibilities:

- parse frontmatter when present
- extract headings and anchors
- extract markdown links
- extract code fences/examples
- extract document references such as `TASK-123`, `SPEC-045`, `PLAN-090`

### C. Normalized Artifact Model

Responsibilities:

- provide one normalized record shape regardless of carrier
- merge frontmatter-backed and registry-fallback metadata
- expose common fields such as `id`, `type`, `authority`, `status`, `health`, `path`, `related`, and supersession fields

### D. Relationship / Lineage Graph

Responsibilities:

- build doc-to-doc edges
- connect spec -> plan -> task references
- record supersession edges
- record evidence links into code/tests where known

### E. Findings / Evidence Base Model

Responsibilities:

- shared finding identity
- severity / category / evidence representation
- rendering-friendly normalized output suitable for markdown or JSON

The spec processor may still emit `SpecFinding`-flavored outputs, and `ash-lint` may still emit source-lint diagnostics, but both should be mappable onto a shared evidence/finding base where practical.

## What Stays Product-Specific

### Spec Processor

Owns:

- CI/blocking semantics
- Tier 0 / Tier 1 / Tier 2 escalation logic
- repository-audit workflows such as example conformance and changelog coverage
- meta-validation of its own audit rules

### Ash Wiki

Owns:

- authority / status / health semantics over the broader corpus
- supersession and residual-authority modeling
- computed views and service exports
- onboarding bundles
- browser/agent query surfaces

### `ash-lint`

Owns:

- Ash source/module lint rules
- parser/AST-based code diagnostics
- LSP-facing source lint integration

`ash-lint` is not a document-audit product, but it is adjacent enough that compatible finding/evidence conventions are desirable.

## Practical Architectural Shape

A good practical decomposition is:

1. `corpus-core`
   - discovery, extraction, normalization, graph building
2. `corpus-rules`
   - reusable document/corpus checks
3. product adapters
   - spec processor adapter
   - wiki registry/view adapter
   - optional finding/evidence bridge for `ash-lint`

This note does not require those exact crate/module names. It only fixes the architectural split.

## Near-Term Recommendation

During the current Ash wiki rollout:

1. keep building the wiki semantics first (`authority`, `status`, `health`, supersession)
2. do not merge the spec processor and wiki at the product level
3. when registry and lint work begins, factor the lowest-level reusable corpus analysis pieces so both products can consume them

## Relationship to Other Documents

- [DESIGN-029: Ash Wiki Knowledge Substrate](DESIGN-029-ASH-WIKI-KNOWLEDGE-SUBSTRATE.md)
- [SPEC-045: Ash Wiki Knowledge Substrate](../spec/SPEC-045-ASH-WIKI.md)
- [DESIGN-SPEC-PROCESSOR](DESIGN-SPEC-PROCESSOR.md)
- [PLAN-090: Spec Processor](../plan/PLAN-090-SPEC-PROCESSOR.md)
- [SPEC-041: Ash Lint Library Extraction](../spec/SPEC-041-ASH-LINT-LIBRARY.md)

## Bottom Line

The spec processor, the wiki, and `ash-lint` should remain distinct products.

They should, however, converge on a shared document/corpus analysis substrate wherever they need the same discovery, extraction, normalization, graph, and evidence machinery.