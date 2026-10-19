# Source configuration and reproducible build records

Centralise source repository URLs, default refs, publication mappings, renderer
pins, and the website compiler checkpoint in `sources.json`. Imported ash-lang
and ash-doc publications continue to follow `main`; per-build full-SHA overrides
remain supported. Source Markdown, study output, and renderer code retain their
existing ownership and authority.

Every successful build emits `dist/build-manifest.json`, recording the website
commit and dirty state, requested and resolved documentation refs, publication
paths, the study compiler target and Quarto identity, renderer commits/archive
and installed-file hashes, compiler checkpoint, configuration and npm-lock
hashes, and design-token hash. Read the resolved identities produced by each
import, rather than fetching or resolving moving refs a second time. Reject
stale preparation metadata or renderer provenance that disagrees with config.

Pinned compiler references in site Markdown and the tour derive from the same
configuration. A source-pin command resolves a tag/ref to an immutable commit,
reads its committed Rust toolchain when updating the compiler checkpoint, and
records the reason. Advancing a compiler checkpoint clears its review date;
example verification and human review of narrative claims remain distinct.
Repeated updates to the same commit are no-ops. Renderer pin changes require
repackaging and verifying the corresponding vendored runtimes.

Provide a local refresh command for milestones, releases, and significant
commits, recording event/reason metadata in the manifest. Pin changes remain
explicit local changes for review and example verification. Keep commands usable
without a GitHub remote; a later PR workflow can wrap these same commands.
Do not create PRs, configure a remote, merge, deploy, or schedule anything.
Keep existing push/PR checks using the same source config.

Implementation:

1. Write failing unit tests for config validation, Markdown substitutions,
   manifest provenance/coherence, and pin advancement using temporary Git repos.
2. Implement the config helpers and migrate importers, provenance links, source
   publication mappings, compiler verification, renderer packaging, and CI inputs.
3. Add manifest generation to the build, with installed-renderer checks and stale
   input rejection. Test the generated artifact against published source links.
4. Add source-pin tooling, explicit pending-review presentation, local refresh
   orchestration, and documented milestone/release/reproduction commands.
5. Build, type-check, run unit and affected browser checks, inspect the manifest,
   validate workflow syntax, and request independent review. Keep actual pins
   unchanged and leave this work uncommitted for the user's separate request.
