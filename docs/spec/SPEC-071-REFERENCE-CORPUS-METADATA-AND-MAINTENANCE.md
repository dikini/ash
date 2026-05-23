# SPEC-071: Reference Corpus Metadata and Maintenance

**Status:** Draft
**Date:** 2026-05-23
**Promotes:** [DESIGN-042](../design/DESIGN-042-REFERENCE-CORPUS-AND-DOCUMENTATION-GOVERNANCE.md)
**Related:** [DESIGN-035](../design/DESIGN-035-DOCUMENTATION-CORPUS-GOVERNANCE.md), [SPEC-045](SPEC-045-ASH-WIKI.md)
**Plan:** [PLAN-120](../plan/PLAN-120-REFERENCE-CORPUS-ROLLOUT.md)
**Implementation Tasks:** [TASK-946](../plan/tasks/TASK-946-reference-corpus-design-packet.md) through [TASK-953](../plan/tasks/TASK-953-reference-corpus-closeout-and-drift-report.md)

## 1. Summary

SPEC-071 defines the metadata, authority, crosslinking, lifecycle, tone, and maintenance rules for Ash's curated `reference/` corpus. The reference corpus is a separate top-level documentation surface for humans and AI agents. It is derived from current specs, live implementation, examples, tests, tasks, and known limitations while preserving `docs/` as the working and historical corpus.

## 2. Normative terms

- **Working corpus:** the existing `docs/` tree, including ideas, design notes, specs, plans, tasks, audits, and historical evidence.
- **Reference corpus:** the top-level `reference/` tree that presents current curated Ash behavior and APIs.
- **Reference page:** a human-readable page under `reference/` with required metadata and typed authority links.
- **Agent derivative:** a card, context pack, retrieval index, or common-confusion guide derived from reference pages for AI-agent use.
- **Authority link:** a typed link from a reference artifact to a spec, code path, task, test, example, historical rationale, limitation, or derivative.
- **Drift finding:** a recorded mismatch between a reference claim and its authority/evidence sources.

## 3. Corpus authority rules

1. The working corpus remains preserved in `docs/`.
2. The reference corpus MUST live outside `docs/` at top-level `reference/` unless a later design supersedes that name.
3. Reference pages MUST NOT silently rewrite historical rationale from `docs/`.
4. Reference pages MUST NOT replace current specs as normative contracts.
5. Reference pages MUST state status, stability, authority, and verification metadata.
6. If live implementation and current spec disagree, the reference page MUST record a drift finding or link to one.
7. Agent derivatives MUST link back to their source reference pages and MUST NOT fork semantic claims.

## 4. Required frontmatter

Every Markdown artifact under `reference/`, including index and README pages, MUST have YAML frontmatter with these fields unless it is generated with an equivalent sidecar metadata file:

| Field | Required | Description |
| --- | --- | --- |
| `id` | yes | Stable identifier such as `ref.language.act`. |
| `title` | yes | Human title. |
| `kind` | yes | Artifact kind. |
| `audience` | yes | List containing `human`, `agent`, or both. |
| `authority` | yes | Authority class. |
| `status` | yes | Lifecycle status. |
| `stability` | yes | API/semantic stability class. |
| `owner` | yes | Owning subsystem or document group. |
| `last_verified` | yes | Date of last verification, or `unknown` for draft skeletons. |
| `verified_against` | yes | Structured sources used for verification. |
| `related` | yes | Dependency and supersession links. |
| `refresh_trigger` | yes | Events that require review or regeneration. |

Allowed `kind` values:

- `reference`
- `index`
- `status`
- `guide`
- `agent-card`
- `agent-pack`
- `generated`
- `methodology`
- `style-guide`

Allowed `authority` values:

- `canonical`
- `canonical-adjacent`
- `derivative`
- `generated`
- `historical-summary`
- `draft`

`canonical` is reserved for future explicitly approved authority manifests or generated indexes. Ordinary explanatory reference pages MUST NOT use `canonical`; they normally use `canonical-adjacent`, `derivative`, `generated`, `historical-summary`, or `draft`.

Allowed `status` values:

- `current`
- `partial`
- `draft`
- `stale`
- `superseded`
- `generated`
- `unknown`

Allowed `stability` values:

- `alpha`
- `beta`
- `stable`
- `experimental`
- `historical`
- `unknown`

## 5. `verified_against` schema

`verified_against` MUST be a mapping with these keys:

```yaml
verified_against:
  git_commit: <commit-or-unknown>
  specs: []
  tasks: []
  code: []
  tests: []
  examples: []
```

Rules:

1. Paths MUST be repo-relative.
2. `git_commit` MAY be `unknown` only for draft skeletons.
3. Each non-empty path list MUST resolve under the repository root.
4. Test entries MAY be command strings when no single path owns the evidence.
5. Generated pages MAY store equivalent metadata in sidecar files if the generator emits the same schema.

## 6. `related` schema

`related` MUST be a mapping with these keys:

```yaml
related:
  depends_on: []
  explains: []
  supersedes: []
  superseded_by: null
  historical_rationale: []
```

Rules:

1. Reference IDs SHOULD be used for links inside `reference/`.
2. Repo paths SHOULD be used for working-corpus and code links.
3. `superseded_by` MUST be either null or one target.
4. Historical rationale links MUST NOT be treated as current authority unless separately listed under `verified_against`.

## 7. Required page sections

Every `kind: reference` page SHOULD contain these sections in order:

1. `Summary`
2. `Status`
3. `Concept`
4. `Syntax / surface` when applicable
5. `Semantics` when applicable
6. `API / stdlib surface` when applicable
7. `Examples`
8. `Implementation notes`
9. `Known limitations`
10. `Common confusions`
11. `Authority and traceability`
12. `Agent notes`

A section MAY be marked `Not applicable` but SHOULD NOT be omitted during the pilot.

## 8. Tone and methodology rules

Reference pages MUST:

1. describe current Ash behavior, not future aspiration;
2. mark alpha/partial/deferred behavior explicitly;
3. keep historical rationale in linked sections rather than mainline explanation;
4. use small examples and tables where useful;
5. avoid marketing language;
6. avoid identity drift: Ash is a programming language; governance and AI/human collaboration are design pressures and domains;
7. distinguish normative examples from illustrative or aspirational examples;
8. use conservative claims such as `Implemented MVP` only when backed by status/evidence links.

## 9. Agent derivative rules

Agent cards and packs MUST:

1. carry `kind: agent-card` or `kind: agent-pack` metadata;
2. link to one or more canonical reference pages;
3. include retrieval tags;
4. include common-confusion warnings when the concept has known stale traps;
5. include `must_check_before_editing` links for implementation-sensitive topics;
6. avoid new semantic claims not present in the linked reference pages.

## 10. Maintenance rules

Every Ash implementation task closeout SHOULD classify documentation impact as one of:

- `no-doc-change`
- `spec-change`
- `reference-change`
- `stdlib-doc-change`
- `example-change`
- `agent-pack-change`
- `drift-repair`

If public semantics or public stdlib/tool behavior changes, the task MUST either update the relevant reference page or record a stale marker/drift finding.

Every phase closeout SHOULD refresh:

1. `reference/status/feature-matrix.md` if the phase changes public feature status;
2. `reference/status/known-limitations.md` if limitations move;
3. any agent cards whose `refresh_trigger` matches the changed subsystem;
4. drift report output from the reference checker once the checker exists.

## 11. Static validation requirements

The initial validator MVP SHOULD check:

1. required frontmatter exists and parses;
2. allowed enum values are used;
3. repo-relative paths in `verified_against` resolve;
4. internal `ref.*` IDs resolve;
5. markdown links in changed reference files resolve;
6. CLI reference subcommands match live CLI declarations;
7. stdlib reference pages match public `std/src/*.ash` declarations at least by module file existence;
8. examples cited by reference pages are classified.

PLAN-120 may initially run these checks in report mode before promoting any to hard gates.

## 12. Snapshot manifests

Reference releases SHOULD be represented by named snapshot manifests rather than copied directory trees. A manifest records:

```yaml
id: ash-alpha-phase-123
git_commit: <commit>
phase_range: "1-123"
reference_root: reference/
generated_at: YYYY-MM-DD
verification_commands: []
known_exclusions: []
```

Materialized bundles for humans or agents SHOULD be generated from manifests, not hand-maintained as separate truth surfaces.

## 13. Non-goals

- This spec does not require moving existing `docs/` files.
- This spec does not define a dynamic wiki or knowledge service.
- This spec does not stabilize all Ash language APIs.
- This spec does not replace current language/runtime/tool specs.
- This spec does not require all reference pages to exist before the pilot closes.

## 14. Acceptance criteria

SPEC-071 reaches Implemented MVP when:

| ID | Criterion |
| --- | --- |
| R71-1 | `reference/` skeleton exists with authority, methodology, style, and status indexes. |
| R71-2 | Required metadata schema is validated for pilot pages. |
| R71-3 | Pure/Act/Proc/Workflow pilot pages exist and link to specs/code/examples. |
| R71-4 | Agent concept cards for the pilot slice link back to reference pages and include common-confusion warnings. |
| R71-5 | Static validator checks frontmatter, path links, internal reference IDs, and pilot code/spec paths. |
| R71-6 | Pilot example/status classification distinguishes normative-pass, illustrative-pass, expected-fail, aspirational, historical, and reference-only. |
| R71-7 | PLAN-120 closeout records drift findings and next-slice recommendations without claiming full corpus migration. |
