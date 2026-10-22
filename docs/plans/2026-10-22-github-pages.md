# GitHub Pages implementation plan

**Goal:** Publish `main` at `https://dikini.github.io/ash/` through GitHub Actions.

**Architecture:** Keep the existing root-based Astro build and upstream preparation.
Prepare a separate Pages artifact under `.cache/github-pages/ash/`, adapting local
URLs to the selected Pages path. Verify that artifact at its mounted path before
uploading it. Deploy only after the existing website and example checks succeed.

**Tech stack:** Astro, Node.js, linkedom, Pagefind, GitHub Actions and Pages.

1. Add fixture tests for Pages navigation, CSS assets, canonical metadata,
   discovery exports, integrity hashes, and preservation of code and external URLs.
2. Implement `scripts/prepare-pages.mjs` and expose `npm run prepare:pages`.
   Copy the build, adapt URL-bearing fields, update discovery hashes, and check
   mounted links. Never alter fetched sources or the original build.
3. Extend the existing check workflow with Pages configuration, artifact upload,
   and deployment after both check jobs pass on `main`; retain PR checks.
4. Document deployment and local artifact verification. Run check, build, tests,
   Pages preparation, and a browser smoke check under `/ash/`.
5. Enable Pages using the Actions build type, commit and push the configuration,
   follow the workflow, and verify the public site.

Fresh-runner validation also exposed missing study diagram dependencies in the
existing build. Provision required renderer checkouts from upstream specimen
pins, verify bundle hashes and the diagram npm lockfile, and install the pinned
browser. Install Poppler and browser system libraries in CI. Verify this path
with a clean temporary checkout rather than relying on cached rendered books.

The selected project URL requires a path prefix. A custom domain would avoid that
prefix but has not been requested. A separate deploy workflow would duplicate
the existing build or require cross-workflow artifact coordination; extending
the check workflow keeps deployment dependent on its verified build.
