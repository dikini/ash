---
id: ref.tools.ashgrove.source_payload
title: Ashgrove Source Payload and Local State
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
    - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
  code:
    - crates/ashgrove/src/lib.rs
  tests:
    - cargo run -p ashgrove -- install --help
    - cargo run -p ashgrove -- update --help
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.tools.ashgrove.install
    - ref.tools.ashgrove.update
  explains:
    - ref.tools.ashgrove.trust_signing
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md
refresh_trigger:
  - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md changes
  - crates/ashgrove/src/** changes
  - reference/tools/ashgrove/source-payload.md changes
---

# Ashgrove Source Payload and Local State

Source install/update builds from a reproducible source payload, not from every file that happens to exist in a developer checkout. SPEC-074 defines the current payload/local-state boundary.

Live help checked:

```bash
cargo run -p ashgrove -- install --help
cargo run -p ashgrove -- update --help
```

## Source Payload Rule

For live source roots, Ashgrove computes the source-root payload digest and the isolated build copy from the same file set. A file excluded from the digest is also excluded from the isolated build copy.

Git source roots use git-compatible ignore semantics for local state. Gitignored files do not affect source-root payload identity, are not copied into the isolated build root, and do not require `--allow-dirty-source`.

Nonignored source files remain part of the payload. Nonignored untracked, modified, deleted, or build-mutated source payload remains fail-closed unless the user explicitly passes `--allow-dirty-source`, which records the non-reproducible boundary.

## Built-In Non-Git Local-State Ignores

For non-git source roots, Ashgrove applies a narrow built-in local-state ignore set for known local state and build outputs. This is not a broad substring filter and is not user-extensible through arbitrary ignore globs.

The current policy explicitly avoids adding a broad `--ignore` or `--exclude` CLI because such flags could hide real source changes unless recorded in toolchain identity.

## Source Archives

Source archives remain separate from live source-root payload policy. Source-archive digest and attestation behavior must stay fail-closed and must not be silently weakened by developer-checkout ignore rules.

Source archive metadata such as `source_archive_digest` remains archive evidence. Source-root payload metadata such as `source_payload_digest_policy` and `source_payload_digest` records the live source-root payload policy. These fields must not be overloaded.

## Diagnostics

Ignored local state changing during build should not abort source-root install/update. Nonignored source payload mutation during build should fail before publish with a source-payload mutation diagnostic.

Git membership failures in git-like source roots fail closed. Ashgrove should not silently fall back to broad filesystem walking when git cannot determine payload membership.

## Reference-Only Command Forms

```bash
ashgrove install --from source --path PATH
ashgrove update --to TO --from source --path PATH
ashgrove install --from source --path PATH --allow-dirty-source
ashgrove install --from source --path PATH --allow-unidentified-source
```

The forms are from live help. Whether an override is appropriate depends on source identity and reproducibility evidence; the reference does not recommend using overrides by default.

## Non-Goals

Source payload policy does not provide a hosted registry, global/system install path, OS package-manager integration, arbitrary SemVer solver, broad source-ignore glob CLI, or relaxed source-archive attestation.
