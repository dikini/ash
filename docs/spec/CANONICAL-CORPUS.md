# Canonical Corpus Sidecar (`canonical-corpus/v1`)

**Status:** Active canonical-authority overlay and migration routing (TASK-1987)
**Extends:** [SPEC-071: Reference Corpus Metadata and Maintenance](SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md)
**Programme:** [PLAN-202](../plan/PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)

## Purpose and boundary

`CANONICAL-CORPUS.json` is the versioned, machine-readable sidecar for Phase 202's authority
graph. It is an overlay over stable repository paths, not a physical migration and not a
replacement for SPEC-071. In particular, it does not alter the SPEC-071 `reference/` frontmatter
schema, its required fields, or its controlled enum values.

The sidecar registers A0 governance nodes, the compact A1/A2 canonical core, A3 conformance,
A4 reference derivatives, and A5 audit/evidence/history records. The eight PLAN-202 subjects have
exactly one active A1/A2 owner in [Ash Canonical Core](CANONICAL-CORE.md). Selection follows the
reconciled target sources, never chronology, directory, or current implementation behavior.

## Sidecar contract

The JSON object has `schema: "canonical-corpus/v1"`, `nodes`, independent `trace_nodes`, and
`typed_edges` arrays. A node has these required validator-facing fields:

| Field | Meaning |
| --- | --- |
| `id` | Stable, repository-wide node identifier. |
| `path` | Existing repository-relative regular-file path. |
| `kind` | Controlled node kind: `manifest`, `vocabulary`, `semantic-rule-set`, `handoff-contract`, `conformance-case`, `reference`, `agent-card`, `agent-pack`, `generated`, `audit`, `evidence`, `archive`, `plan`, or `result-schema`. |
| `authority_level` | Controlled authority level `A0` through `A5`. A4 is required for every `reference`, `agent-card`, `agent-pack`, or `generated` node and every path beneath top-level `reference/`. |
| `lifecycle` | Controlled lifecycle: `active`, `draft`, `generated`, `superseded`, or `archived`. |
| `canonical_for` | Subject identifiers owned by an A1/A2 node. The validator rejects duplicate A1/A2 ownership. Empty means no semantic ownership claim. |
| `supersedes`, `depends_on` | Node-ID links. Supersession is acyclic. |
| `trace_nodes` | Stable IDs that resolve in the independent `trace_nodes` root array. |

Each independent trace record has an ID from PLAN-202's stable-node namespaces, a trace kind, a
sidecar document-node ID, and a resolvable Markdown anchor. The initial records use the
`REQ-CORPUS-*` namespace and PLAN-202 anchors for authority, metadata, generated-derivative,
snapshot, and conflict requirements; the four TASK-1984 conflicts retain distinct trace IDs without
acquiring semantic ownership.

`typed_edges` use sidecar node IDs (or existing file paths during migration) rather than the
SPEC-071 reference identifiers. Their controlled vocabulary is PLAN-202's `defines`, `refines`,
`requires`, `lowers_to`, `projects_to`, `implemented_by`, `tested_by`, `proved_by`, `assumes`, and
`supersedes`; legacy path-edge names remain accepted only for migration compatibility. This is the
explicit boundary between the sidecar graph and SPEC-071's independent frontmatter relationship
enums.

Nodes may include `generated_from` only for generated material. It contains `sources` (sidecar
node IDs) and `source_hashes` (SHA-256 digests keyed by source ID). The validator recomputes each
hash, so a generated context pack is stale on any source change. Agent-pack provenance is mandatory
and may source only A0-A4 nodes; A5 plans, audits, archives, and evidence can never feed current
agent guidance. Generated material is A4 and cannot become semantic authority. The context-pack
index therefore starts current semantic work at the canonical-core/default route; A5 records are
retained only for explicitly historical investigation.

## Authority and lifecycle policy

- A0 records corpus governance and schema policy.
- A1 owns target vocabulary, grammar, types/effects, Core/CPS syntax, and operational semantics.
- A2 owns surface-to-Core, runtime-observable, and implementation-conformance handoffs.
- A3 instantiates canonical rules in conformance material.
- A4 remains the top-level `reference/` derivative surface required by SPEC-071.
- A5 records plans, audits, conflicts, and evidence; it never owns a semantic subject.

`reference/` remains a derivative reading surface even when linked by a sidecar edge. Its existing
frontmatter is validated by the SPEC-071 tooling, not converted to `canonical-corpus/v1` metadata.

## Snapshots and archive preservation

This overlay preserves SPEC-071's git-backed snapshot-manifest model: named snapshots identify a
Git commit and extraction profile rather than copying a corpus tree. See
[SPEC-071 §12](SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md#12-snapshot-manifests) and
[PLAN-202 §5.1](../plan/PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md#51-target-layout-and-existing-governance).
No snapshot, archive, redirect, or source path was moved by TASK-1985. TASK-1987 records its
Git-backed archive provenance in `reference/manifests/phase-202-archive.json`, its logical
productive-link routes in `reference/manifests/phase-202-redirects.json`, and deterministic
pre/post retrieval evidence in `docs/plan/audits/TASK-1987-retrieval-quality.json`. These are
metadata artifacts, not a copied archive tree: unique historical content remains in the repository
and at the named Git revision.

## Canonical graph and default paths

The graph retains the TASK-1984 audit and its conflict trace IDs, but makes no A5 artifact a
productive owner. `docs/reference/formalization-boundary.md` and
`docs/reference/parser-to-core-lowering-contract.md` are A5 superseded records with typed links to
their replacements. Their unique rationale remains available without appearing in default paths.

`default_read_paths.human` and `default_read_paths.agent` are the manifest-generated productive
routes: A0 authority, the eight compact-core subjects, required A2 handoffs, and the linked A3
conformance node. Neither path can include an A5 plan, audit, archive, historical document, or
unknown node. The A4 agent context-pack index retains deterministic provenance and is checked on
every validation run. TASK-1987 supplies the Git-backed archive, redirect, and retrieval artifacts
that keep historical material available without leaking it into productive context.

Run:

```bash
python3 tools/docs/validate_canonical_corpus.py --root . \
  --manifest docs/spec/CANONICAL-CORPUS.json --format json \
  --check-reference-frontmatter --require-promotion-completeness \
  --require-migration-completeness
```

An empty `errors` array means the sidecar's structural and promotion-completeness contracts are
valid. It does not resolve unrelated TASK-1984 evidence-path findings or claim that current Rust
realization is complete.

## Semantic traceability evidence

[SEMANTIC-TRACEABILITY.json](SEMANTIC-TRACEABILITY.json) is the TASK-1990 evidence graph for the
eight canonical owners and the staged `λAsh-CPS` rules.  It records canonical rule identity,
implementation and test evidence, and proof/disposition status separately.  It is A5 evidence in
this sidecar—not a semantic owner—and therefore is excluded from productive default read paths.
The committed coverage reports are reproducible with:

```bash
python3 tools/docs/validate_semantic_traceability.py --root . \
  --graph docs/spec/SEMANTIC-TRACEABILITY.json \
  --reports-dir docs/plan/audits/TASK-1990-semantic-traceability --format json
```

The validator rejects dangling graph endpoints, unstable anchors, false proof states, unowned
canonical rules, and public semantic implementations with no canonical owner.
