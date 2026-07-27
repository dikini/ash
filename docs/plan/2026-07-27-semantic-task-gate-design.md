# Semantic Task Conformance Gate Design

## Goal

Make the semantic-rule-first workflow mechanically enforceable for implementation agents: a
semantic change must name its canonical rule and bounded domain, carry the required evidence, and
run its task-owned integration checks before it can be committed.

## Decision

Use one checked-in, machine-readable manifest rather than parsing free-form task prose. The
manifest is the validator input; task files remain the human explanation and link to their manifest
record. The coverage map remains the rule-family dashboard and the traceability graph remains the
canonical rule-to-code/test graph.

The initial scope migrates the nine active TASK-1988 follow-ups (TASK-2001 through TASK-2005,
TASK-2008, TASK-2013, TASK-2014, and TASK-439). It must not claim that their bounded slices are
general semantics.

## Data flow

```text
semantic task record
  -> semantic-task validator
  -> coverage-map/task links + traceability rule IDs
  -> staged semantic-change policy
  -> task-owned verification commands
  -> pre-commit / pre-push result
```

Each record contains its task ID, canonical rule IDs, bounded/general domain, layer statuses,
evidence classes, explicit non-goals, next obligation, and a bounded list of verification commands.
The validator rejects unknown traceability IDs, missing required fields, inconsistent bounded
labelling, inactive task links, and unsafe command declarations. The staged policy requires changes
to semantic Rust paths to include the appropriate task, manifest, coverage-map, traceability, and
changelog evidence.

## Gate tiers

`pre-commit` runs the manifest validator plus task-owned targeted commands for semantic tasks
touched by the staged change. `pre-push` retains the full repository gate and additionally runs
all active semantic-task verification commands. The targeted commands close the current gap where
the library-only Rust test command misses semantic integration tests.

## Production-boundary evidence

TASK-2004 and TASK-2014 records require both a checked-Core/CPS positive route and a genuine
unsupported-source rejection. The stale nested-arithmetic rejection controls will be replaced with
an unsupported source form while preserving the assertion that no direct evaluator runs.

## Non-goals

- Infer semantic ownership from source-file names alone.
- Treat a named fixture as a semantic rule.
- Convert bounded status to a general-rule claim.
- Make private differential evaluation a production fallback.
