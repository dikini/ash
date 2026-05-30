# PLAN-123: SPEC-073 Ashgrove Release, Trust, Registry, and Runtime-Support Completion

> **For Hermes:** Use subagent-driven-development to implement this plan task-by-task. This phase closes documented SPEC-073 deferred rows from Phase 127. Do not rewrite Phase 127 history, create a hosted package registry, add global/system installs, or claim SPEC-073 beyond the implemented MVP boundary without concrete TASK-986 evidence for every row.

**Goal:** Promote SPEC-073 from Draft with deferred acceptance rows to Implemented MVP by closing the release, trust, registry-ready metadata, cleanup reachability, runtime-support, and end-to-end deployment gaps documented by TASK-974.

**Architecture:** Build on the existing `ashgrove` crate, XDG toolchain layout, local source/tarball install substrate, launcher dispatch, git lock/fetch/vendor flow, and module-loader dependency-root integration from Phase 127. Add missing release metadata, authenticated release/download policy, runtime-support payload contracts, trust/signing preservation and enforcement, cleanup reachability, and final acceptance evidence without changing the core Phase 127 alpha invariants.

**Tech Stack:** Rust 2024; `crates/ashgrove`, `ash-cli`, `ash-engine`; local git fixtures; tarball/release packaging scripts; TOML metadata preservation; XDG temp roots; repo-native broad gates; Markdown specs/plans/tasks/audits.

---

## 1. Status

**Status:** ✅ Complete; TASK-986 promoted SPEC-073 to Implemented MVP with explicit non-goal boundaries
**Spec:** [SPEC-073](../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md)
**Task range:** [TASK-975](tasks/TASK-975-spec073-ashgrove-completion-packet.md) through [TASK-986](tasks/TASK-986-spec073-implemented-mvp-closeout.md)

TASK-975 created this docs packet. TASK-976 completed the hard audit gate and patched exact downstream verification commands before Rust implementation starts. TASK-985 proves the completed release/deployment slices compose end to end. TASK-986 completed final closeout, broad status reconciliation, independent review, and SPEC-073 promotion to Implemented MVP.

## 2. Scope

### In scope

- Source-archive release metadata and reproducibility checks.
- Concrete runtime-support payload metadata in installed toolchains.
- Authenticated tarball URL recording and release-index trust policy.
- Packaged dispatcher lifecycle policy.
- Registry-ready package metadata substrate without a hosted registry service.
- Broader cleanup reachability across lockfiles, fetched cache, vendor metadata, and installed toolchains.
- Manifest/lockfile trust metadata preservation during rewrites.
- Mandatory trust/signing enforcement for release/download/git trust boundaries.
- Remote-authenticated git fetch policy.
- End-to-end release/deployment acceptance evidence.
- SPEC-073 status reconciliation only after evidence passes.

### Out of scope

- Hosted Ash package registry service.
- Global/system install roots.
- OS package-manager integration.
- Editor plugin management.
- Independent stdlib updates outside toolchain updates.
- Arbitrary SemVer dependency solving across registry packages unless a later spec expands scope.
- Rewriting Phase 127 history.

## 3. Task table

| Task | Description | Est. Hours | Status |
|------|-------------|------------|--------|
| [TASK-975](tasks/TASK-975-spec073-ashgrove-completion-packet.md) | Create PLAN-123/TASK-975..986 packet and register Phase 128 | 4 | ✅ Complete |
| [TASK-976](tasks/TASK-976-ashgrove-completion-acceptance-delta-and-audit-gate.md) | Map every SPEC-073 deferred row to exact owners, files, tests, and RED failure modes | 8 | ✅ Complete |
| [TASK-977](tasks/TASK-977-source-archive-release-metadata.md) | Implement source-archive release metadata and reproducibility checks | 10 | ✅ Complete |
| [TASK-978](tasks/TASK-978-runtime-support-payload-metadata.md) | Define and verify concrete runtime-support payload metadata across source/tarball installs | 10 | ✅ Complete |
| [TASK-979](tasks/TASK-979-release-index-authenticated-tarball-url-policy.md) | Add authenticated tarball URL recording and release-index trust policy | 14 | ✅ Complete |
| [TASK-980](tasks/TASK-980-packaged-dispatcher-lifecycle-policy.md) | Finalize packaged dispatcher lifecycle and launcher update/remove policy | 10 | ✅ Complete |
| [TASK-981](tasks/TASK-981-registry-scale-package-metadata-substrate.md) | Add registry-ready package metadata without creating a hosted registry service | 12 | ✅ Complete |
| [TASK-982](tasks/TASK-982-cleanup-lockfile-cache-reachability.md) | Implement broader cleanup reachability across lockfiles, fetched cache, vendor metadata, and installed toolchains | 12 | ✅ Complete |
| [TASK-983](tasks/TASK-983-manifest-rewrite-trust-preservation.md) | Preserve manifest and lockfile trust metadata during read-modify-write operations | 8 | ✅ Complete |
| [TASK-984](tasks/TASK-984-mandatory-trust-signing-and-remote-git-fetch-policy.md) | Implement mandatory trust/signing enforcement and remote-authenticated git fetch policy | 16 | ✅ Complete |
| [TASK-985](tasks/TASK-985-ashgrove-release-deployment-acceptance-integration.md) | Prove release/deployment flows cover the completed SPEC-073 rows end-to-end | 12 | ✅ Complete |
| [TASK-986](tasks/TASK-986-spec073-implemented-mvp-closeout.md) | Promote SPEC-073 only after acceptance matrix, broad gates, and independent review pass | 8 | ✅ Complete |

## 4. Deferred-row ownership

| Deferred gap | Source | Owner |
| --- | --- | --- |
| Source archive release metadata | TASK-974 deferred gap; A73-1 | TASK-977 |
| Runtime-support payload metadata and source/tarball equivalence | TASK-974 deferred gap; A73-3/A73-10 | TASK-978 |
| Authenticated tarball URL recording and release-index trust policy | TASK-974 deferred gap; A73-2/A73-4 | TASK-979 |
| Packaged dispatcher lifecycle policy | TASK-974 deferred gap; A73-5 | TASK-980 |
| Registry-scale package metadata | TASK-974 deferred gap; A73-8/A73-9 adjacency | TASK-981 |
| Broader cleanup reachability | TASK-974 deferred gap; A73-7 | TASK-982 |
| Manifest rewrite trust metadata preservation | TASK-974 deferred gap; A73-11 | TASK-983 |
| Mandatory trust/signing enforcement | TASK-974 deferred gap; A73-11 hardening; requires SPEC-073 wording amendment before closeout | TASK-984 |
| Remote-authenticated git fetch policy | TASK-974 deferred gap; A73-8/A73-9/A73-12 | TASK-984 |
| End-to-end release/deployment acceptance | SPEC-073 A73-1 through A73-12 | TASK-985 |

## 5. Decision gates

- D1: Phase 127 remains the historical partial closeout; Phase 128 owns completion evidence.
- D2: SPEC-073 is Implemented MVP after TASK-986 mapped every deferred row to concrete evidence.
- D3: Registry-ready metadata is in scope; a hosted registry service and SemVer dependency solver remain out of scope unless a new spec explicitly expands scope.
- D4: Runtime-support payload metadata must be concrete and equivalent across source and tarball install paths before A73-3/A73-10 can be promoted.
- D5: Release-index and tarball URL support must be authenticated or fail closed; bare version install/update must not become best-effort network lookup.
- D6: Trust/signing metadata preservation and enforcement must be tested as data-preservation and security behavior, not prose-only policy.
- D7: Cleanup reachability must be conservative, visible in dry-run, and must never delete project-local `ash.toml` or `ash.lock`.
- D8: TASK-976 is a hard audit gate and must replace downstream placeholder verification before implementation starts.
- D9: A73-11's Phase 127 wording covered reserved trust/signing preservation without mandatory enforcement. Phase 128 may only claim mandatory enforcement after TASK-984 amends SPEC-073 and proves source/tarball publish plus git fetch/lock fail closed.

## 6. Verification strategy

Docs-only packet verification:

```bash
git diff --check
python3 -c "from pathlib import Path; files=['docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md','docs/spec/README.md','docs/plan/PLAN-INDEX.md','docs/plan/PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md','CHANGELOG.md']+[f'docs/plan/tasks/TASK-{n}-{slug}.md' for n,slug in [(975,'spec073-ashgrove-completion-packet'),(976,'ashgrove-completion-acceptance-delta-and-audit-gate'),(977,'source-archive-release-metadata'),(978,'runtime-support-payload-metadata'),(979,'release-index-authenticated-tarball-url-policy'),(980,'packaged-dispatcher-lifecycle-policy'),(981,'registry-scale-package-metadata-substrate'),(982,'cleanup-lockfile-cache-reachability'),(983,'manifest-rewrite-trust-preservation'),(984,'mandatory-trust-signing-and-remote-git-fetch-policy'),(985,'ashgrove-release-deployment-acceptance-integration'),(986,'spec073-implemented-mvp-closeout')]]; missing=[p for p in files if not Path(p).exists()]; assert not missing, missing; idx=Path('docs/plan/PLAN-INDEX.md').read_text(); assert '## Phase 128:' in idx; [assertion for assertion in []]; assert all(f'TASK-{n}' in idx for n in range(975,987)); print('phase128 docs packet verified')"
```

Implementation-task verification after TASK-976 must add focused non-zero test commands plus broad gates:

```bash
bash scripts/check-rust-format.sh
bash scripts/check-rust-clippy.sh
bash scripts/check-rust-tests.sh --workspace --all-targets
bash scripts/check-doc-tests.sh
git diff --check
```

TASK-986 closeout must also run a scoped docs/link/status sweep for SPEC-073, PLAN-122, PLAN-123, TASK-964 through TASK-986, and related audit artifacts.

## 7. Promotion rule

SPEC-073 may be promoted beyond Draft only after TASK-986 confirms:

- every A73-1 through A73-12 row has concrete current evidence or an explicitly accepted non-MVP boundary;
- every TASK-974 deferred gap has an owning TASK-977 through TASK-985 evidence record;
- current status surfaces agree across SPEC-073, docs/spec/README.md, PLAN-122, PLAN-123, PLAN-INDEX, task files, audit artifacts, and CHANGELOG.md;
- Phase 127 remains historical partial closeout language, and Phase 128 owns completion/promotion evidence;
- broad gates and independent review pass.

## 8. Changelog

### 2026-05-29

- TASK-975 created this follow-on packet for SPEC-073 completion. The packet keeps SPEC-073 Draft, preserves Phase 127 as historical partial closeout, and assigns deferred release, trust, registry-ready metadata, cleanup reachability, runtime-support, remote git, and closeout evidence to TASK-976 through TASK-986.

### 2026-05-30

- TASK-986 completed final closeout. SPEC-073 is promoted to Implemented MVP after the A73-1 through A73-12 evidence matrix, broad gates, status reconciliation, and independent review. The non-goal boundary remains unchanged: no hosted registry service, no global/system install roots, no OS package-manager integration, no arbitrary SemVer dependency solver, and no signed release-index-as-digest evidence.
- TASK-976 completed the hard audit gate by creating the acceptance-delta artifact, binding TASK-977 through TASK-985 to focused non-zero verification commands, and recording the A73-11 wording amendment required before mandatory trust/signing enforcement can be claimed.
- TASK-977 implemented source-archive release metadata by requiring typed `release-source.toml` origin-commit metadata unless `--allow-unidentified-source` is explicit, recording `source_archive_digest` and `source_origin_commit` in source archive install records, and keeping unidentified archives non-reproducible.
- TASK-978 implemented concrete runtime-support payload metadata by requiring source and tarball toolchain manifests to carry equivalent `[runtime_support]` identity/path/required metadata, validating the payload directory before publish, propagating the selected runtime-support identity through launcher dispatch, and including that identity in runtime artifact construction.
- TASK-979 implemented authenticated tarball URL install/update policy for explicit-digest `file://` tarball URLs, records URL/digest/authentication provenance in `install-record.toml`, rejects URL installs without explicit digest evidence, rejects digest mismatches before publish, and keeps unsupported network URL schemes fail-closed rather than adding best-effort lookup. Release-index signature metadata is not accepted as digest evidence until a later resolver binds signed entries to toolchain id, tarball URL, and digest.
- TASK-981 implemented the registry-ready package metadata substrate. `ashgrove lock` preserves package/version/registry metadata for explicit git-pinned dependencies, vendor provenance records and checks that metadata, ash-engine accepts the registry-style lock carrier, and hosted registry/SemVer dependency resolution remains fail-closed and out of scope.
- TASK-982 implemented cleanup lockfile/cache reachability. Cleanup now derives reachability only from supplied or registered known projects, preserves lock/vendor-provenance referenced fetched git checkouts and repos plus project-pinned toolchains, reports reachable and unreachable git cache entries in dry-run, and preserves project-local `ash.toml`/`ash.lock` files.
- TASK-984 implemented mandatory trust/signing enforcement and remote-authenticated git policy. Required tarball sidecar signature evidence, source-archive attestation evidence, unsigned or unbound release indexes, lock signature mismatches in both ashgrove and ash-engine consumers, untrusted git protocols, and credential-bearing lockfile origins fail closed before publish, fetch, or lock use. HTTPS credentials are redacted before lockfile serialization, and credential-bearing `ssh://` URLs are rejected.
- TASK-985 completed release/deployment acceptance integration. Focused integration tests prove source archive installs compose with runtime-support metadata, selected-toolchain state, git dependency fetch, and cleanup reachability; tarball URL update composes with explicit digest evidence, unsigned release-index fail-closed behavior, required signature sidecar evidence, packaged dispatcher refresh, and remove; and the installed CLI composes locked authenticated dependency resolution with selected-toolchain stdlib/runtime-support dispatch. TASK-986 subsequently completed final promotion.
