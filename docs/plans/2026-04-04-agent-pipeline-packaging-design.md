# Agent Pipeline Packaging Fix Design

**Date:** 2026-04-04

## Problem

The packaged `tools/agent-pipeline` deployment has three correctness gaps:

1. The systemd unit sandboxes writes to `tools/agent-pipeline/.agents`, while the CLI defaults to repo-root `.agents` and the `impl` stage expects to edit the Ash repo.
2. `ash-pipeline queue --from-spec` creates queued state before validating the source file, leaving a poisoned task bundle on failure.
3. The install and Vila integration helpers are hardcoded to `~/Projects/ash`, which breaks clones in any other location.

## Constraints

- Preserve the existing file-based runtime model.
- Keep packaged runtime state isolated under `tools/agent-pipeline/.agents`.
- Allow the packaged supervisor to edit the checked-out repo during `impl`.
- Avoid changing the default CLI discovery model for normal interactive use unless packaged mode explicitly overrides it.
- Keep the helper scripts portable across clone locations.

## Chosen Approach

### 1. Make packaged mode explicit via environment configuration

The packaged deployment should no longer rely on implicit workspace/state discovery.

- The systemd unit will set:
  - `AGENT_PIPELINE_BASE_DIR=<repo>/tools/agent-pipeline/.agents`
  - `AGENT_PIPELINE_WORKSPACE_ROOT=<repo>`
- This preserves the current CLI defaults for ad hoc use, while making the packaged mode deterministic.

### 2. Align the systemd sandbox with actual runtime behavior

The service must permit writes to:

- `tools/agent-pipeline/.agents` for runtime state
- the repository working tree for `impl`-stage edits

The unit should therefore stop advertising a sandbox that contradicts the pipeline’s purpose. The sandbox can remain restrictive in other ways, but it must not block the repo edits the staged prompts require.

### 3. Validate `--from-spec` before mutating state

`queue()` should check that the source file exists and is readable before calling `create_task()`. If validation fails, the command must exit without leaving any lifecycle directory entries behind.

### 4. Make helper scripts repo-relative

`install.sh`, `agent-pipeline.service`, and `vila-integration.sh` should compute paths from their own on-disk location instead of assuming `$HOME/Projects/ash`.

That keeps installation portable while still letting the packaged unit point at the correct repository and source tree.

## Alternatives Considered

### Repo-root `.agents` for packaged mode

Rejected because it weakens state isolation and is a larger behavior shift for packaged users.

### Keep strict tool-local sandbox and forbid repo edits

Rejected because it conflicts with the current `impl` stage contract and would make the packaged supervisor unable to perform its intended job.

## Testing Strategy

1. Add a CLI regression test proving `queue --from-spec` leaves no task bundle when the file is missing.
2. Add helper/path tests that verify installation/runtime path generation is clone-location agnostic.
3. Run the existing `tools/agent-pipeline` lint and test suite after the changes.

## Expected Outcome

After the fix:

- packaged mode writes state only to `tools/agent-pipeline/.agents`
- packaged mode can still edit the repo during `impl`
- `queue --from-spec` is atomic with respect to invalid input
- install/integration scripts work from arbitrary clone locations
