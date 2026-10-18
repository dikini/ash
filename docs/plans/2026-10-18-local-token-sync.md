# Local bidirectional token synchronization

**Goal:** Synchronize token edits from either repository during local workflows without hosted CI.

**Architecture:** Both repositories carry the same dependency-free Python engine and a committed CSS baseline. Compare both working files with that baseline; propagate a one-sided edit, accept identical edits, and reject divergent edits with a diff. Discover peers beside the primary Git checkout, with explicit environment overrides for worktrees. Missing automatic peers allow standalone builds; explicit invalid paths fail. Verification is read-only. Build manifests record token SHA-256.

**Tech stack:** Python standard library, Node wrapper, npm lifecycle scripts, Quarto preparation.

1. Test both directions, identical edits, conflict preservation, mismatched baselines, read-only checks, missing/invalid peers, standalone mode, and worktree discovery with temporary repository fixtures.
2. Implement the shared engine in scripts/token_sync.py and rendering/token_sync.py. Seed .design-tokens-base.css in each repository. Lock synchronization and atomically replace individual files.
3. Preserve the existing Node command as a wrapper. Wire npm predev/prebuild and read-only precheck. Record the site token hash in dist/design-tokens.json.
4. Invoke synchronization before ash-doc HTML preparation, provide a read-only CLI, and add the token hash to build.json. Keep PDF/EPUB-only preparation independent.
5. Document configuration, build side effects, conflict recovery, standalone mode, and local verification. Test and render representative HTML, review diffs, then integrate both local branches.

No GitHub workflow additions or remote changes are included. Existing hosted CI is outside this change.
