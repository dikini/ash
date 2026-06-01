---
id: ref.tools.ashgrove.trust_signing
title: Ashgrove Trust and Signing
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: ashgrove
last_verified: 2026-06-01
verified_against:
  git_commit: e06944a
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
  code:
    - crates/ashgrove/src/lib.rs
    - crates/ash-engine/src/lib.rs
  tests:
    - cargo run -p ashgrove -- install --help
    - cargo run -p ashgrove -- update --help
    - cargo run -p ashgrove -- lock --help
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.tools.ashgrove
  explains:
    - ref.tools.ashgrove.install
    - ref.tools.ashgrove.update
    - ref.tools.ashgrove.project_dependencies
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md
refresh_trigger:
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - crates/ashgrove/src/** changes
  - crates/ash-engine/src/** changes
  - reference/tools/ashgrove/trust-and-signing.md changes
---

# Ashgrove Trust and Signing

Ashgrove preserves trust/signing metadata and enforces required evidence at the implemented release, download, and git boundaries. This page records the current boundary without claiming a hosted registry or signed release-index resolver.

## Metadata Preservation

Project manifests and lockfiles may carry reserved trust/signing metadata. Ashgrove read-modify-write operations preserve unknown future-compatible trust fields where the current preservation path applies.

Illustrative metadata shape from SPEC-073, not a complete policy configuration:

```toml
[trust]
signing = "none"
signature = ""
attestation = ""
```

## Enforced Boundaries

Ashgrove fails closed for:

- required tarball sidecar signature evidence missing or mismatched;
- required source-archive attestation evidence missing or mismatched;
- unsigned or unbound release-index metadata;
- lock signature evidence missing or mismatched when required;
- untrusted git protocols before fetch/lock use;
- credential-bearing lockfile origins before serialization or consumption.

Ash Engine lock consumers enforce the same required lock signature policy where locked dependencies are consumed.

## Release Index Boundary

Explicit-digest tarball URL install/update is the current URL boundary. Release-index signature metadata is not accepted as digest evidence until a later resolver binds toolchain id, tarball URL, and digest.

Reference-only forms from live help:

```bash
ashgrove install --from tarball --url URL --digest DIGEST
ashgrove update --to TO --from tarball --url URL --digest DIGEST
```

`--release-index RELEASE_INDEX` appears in live install/update help, but unsupported or unbound release-index flows fail closed under the current MVP boundary. Do not document a release-index file as sufficient evidence until a later task implements and proves that resolver.

## Non-Goals

This page does not claim transparency logs, hosted registry trust roots, global/system install trust policy, OS package-manager signatures, arbitrary SemVer resolver trust, or broad source-ignore glob policy.
