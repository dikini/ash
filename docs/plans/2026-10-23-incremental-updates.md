# Incremental Updates Implementation Plan

**Goal:** Reduce routine editorial update work while preserving complete static artifacts and explicit upstream refreshes.

**Architecture:** A timed build coordinator caches verified publication stages and search output by content fingerprints. Astro caches individual dynamic routes using complete render inputs. An update command selects editorial validation from changed inputs and falls back to full checks for shared code, configuration, or unknown changes. Complete builds continue resolving upstream refs.

**Tech Stack:** Node ESM, Astro 7.4 incremental static builds, Pagefind, node:test.

### Task 1: Cache and impact primitives
- Create `scripts/update-build.mjs` and `tests/update-build.test.mjs`.
- Test content hashing, removed files, cache corruption, route dependencies, and conservative validation selection before implementation.
- Store successful stage receipts outside dist; never reuse an unchecked output.

### Task 2: Timed build and update commands
- Route `npm run build` through the coordinator with full upstream preparation.
- Add `npm run update` for publication reuse and targeted checks.
- Restore cached search output only for identical HTML and search tool inputs.
- Preserve build manifests; do not issue full-refresh verification receipts from targeted updates.

### Task 3: Incremental Astro routes
- Enable incremental static builds in `astro.config.mjs`.
- Hash editorial body/data/config and reference render props including group navigation.
- Leave small static listing routes rebuilding to capture collection additions, deletions and metadata changes.

### Task 4: Documentation and verification
- Document update/full refresh semantics, cache fallback, timings and validation scope.
- Run check, tests, full build and warm update; exercise body edits, metadata edits and route removals through regression tests.
- Run `git diff --check`; report measured timings and remaining global work.
