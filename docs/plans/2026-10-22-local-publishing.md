# Local publishing implementation plan

**Goal:** Publish only when the maintainer chooses to build and verify locally.

**Architecture:** `main` owns source and a manually triggered verification
workflow. `gh-pages` owns the prepared static files and `.nojekyll`. GitHub
Pages uses its built-in branch deployment, which uploads those files without
rebuilding the site or running the test suite.

1. Add a publishing command that runs the existing local refresh with browser
   verification, prepares the `/ash/` artifact, and pushes it to `gh-pages`.
2. Accept `--prepared` only for a verification receipt matching the current,
   clean source commit and the prepared manifest. Preserve normal publishing
   history and reject concurrent updates with a normal non-forced push.
3. Keep the local refresh receipt valid through Pages URL preparation, recording
   its original manifest hash and the additional mounted-link check.
4. Make the expensive GitHub check workflow manual-only and remove deployment
   from it. Document the local publishing commands and branch ownership.
5. Test the publisher against a temporary local bare remote, commit the source
   changes, build/test locally, push the initial publishing branch, and select
   `gh-pages` as the Pages source. Verify the lightweight deployment.
