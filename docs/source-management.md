# Sources, build records, and checkpoint updates

`sources.json` is the committed source configuration. It declares repository
URLs and requested refs, reference directories and routes, the study route,
renderer pins, and the compiler checkpoint used by website examples and claims.
Imported ash-lang and ash-doc documents default to `main`. Advancing the website
compiler checkpoint does not change those defaults or ash-doc's own Ash target.

The renderer archives and `vendor/renderers.json` retain evidence of packaged
runtime files. Their revisions must match the configuration. The build checks
installed runtime files against that evidence and records archive hashes.
`package-lock.json` continues to pin npm dependencies.

## Refresh documents locally

No website Git remote is required:

```sh
npm run docs:refresh -- --event milestone --reason "Ash milestone name"
```

Use `manual`, `milestone`, `release`, or `commit` for `--event`. The reason is
recorded as supplied; it does not declare a language release. The command runs
type checks, fetches and renders the configured documentation, builds Astro and
search, checks local links, and runs unit/publication tests. Add `--browser` to
run browser checks after installing Chromium, or select an installed Chrome:

```sh
CHROME_BIN=/usr/bin/google-chrome npm run docs:refresh -- \
  --event release --reason "Documentation refresh for release name" --browser
```

The result is `dist/`, including:

- `build-manifest.json`: exact input identities;
- `refresh-report.json`: completed checks and the SHA-256 of that manifest.

Failed checks stop the command. A prior refresh report is removed at startup;
a new success report is written only after every requested check passes.
Plain `npm run build` also emits a build manifest, without claiming that the
separate test commands ran.

Fresh study renders prepare the diagram renderers required by the book's
diagram jobs from the book's own `rendering/specimen.json` pins. Their bundles
are checked against the book's hashes. The book's checksum-verified npm lockfile
installs Mermaid, Puppeteer, and its pinned Chrome into the ignored tool cache.
Linux builds also need Poppler (`pdftocairo`) and Chrome system libraries; the
GitHub workflow installs these before building. Study renderer pins remain
independent of the website's packaged renderer pins.

## Advance the compiler checkpoint

Choose the milestone tag, release tag, or significant commit explicitly. First
inspect the proposed change:

```sh
npm run sources:pin -- --target checkpoint --ref TAG-OR-FULL-SHA \
  --reason "Adopt milestone name" --dry-run
```

Then apply it without `--dry-run`. The command resolves the ref from the
configured ash-lang repository, stores its full commit SHA, reads the committed
`rust-toolchain.toml`, records the reason, and clears `checkpoint.reviewed`.
Repeating the same resolved commit makes no change. Imported docs retain `main`.

To resolve an existing local tag or commit without fetching, add
`--checkout /path/to/ash-lang`. This reads committed blobs and leaves that
checkout and its uncommitted files untouched.

```sh
npm run docs:refresh -- --event milestone --reason "Check new checkpoint" --browser
```

A pending checkpoint review automatically enables executable-example
verification. The refresh prepares a site-owned compiler checkout in
`.cache/ash-compiler/source`, installs the selected Rust toolchain, and checks
the examples against it. `--compiler-source /path/to/ash-lang` uses an existing
checkout instead; its HEAD and compiler inputs must match the selected pin.
Use `--verify-examples` to request this check even after review.

Review the status page, tour, installation instructions, and affected claims.
Fix examples or expectations only when the new checkpoint warrants those
changes. Set `checkpoint.reviewed` to the actual review date (`YYYY-MM-DD`)
after that review, then run a final refresh with `--verify-examples --browser`.
Review the Git diff and commit the configuration and any editorial changes.
Running examples successfully does not review narrative language claims.

Source references in site Markdown use explicit placeholders such as
`{{checkpoint.revision}}`, `{{checkpoint.short}}`, `{{checkpoint.toolchain}}`,
`{{checkpoint.reviewed}}`, `{{source.ash-lang}}`, `{{repository.ash-lang}}`, and
`{{route.spec}}` (with corresponding keys for the other reference collections).
They expand before Astro's native Markdown renderer runs, including in links
and installation commands. The configuration hash participates in Astro's
Markdown cache identity. Pending review is shown explicitly. Imported upstream
Markdown is not processed as a website template.

## Advance a renderer pin

`sources:pin` also accepts `--target railroad` or `--target sequent-md` with
the same ref, reason, checkout, and dry-run options. After selecting the commit,
build that renderer from its committed source and repackage the HTML runtime
using the [renderer update procedure](../vendor/README.md). The packaging
script requires the checkout to match the configured pin. Update npm's archive
integrity as described there, then build and test. Changing a renderer pin alone
does not silently substitute a new runtime.

## Inspect and reproduce a build

`build-manifest.json` records the website commit and dirty flag, configuration
and npm-lock hashes, Node version, requested and resolved documentation refs,
publication mappings, study compiler target and Quarto identity, renderer
provenance and archive hashes, compiler checkpoint, design-token hash, and the
refresh event/reason. It uses each import's resolved identity without fetching
moving refs a second time. Preparation from a different configuration is
rejected. The ordinary per-page source links and study snapshot remain available.

For a committed website checkout with matching configuration, dependencies,
and renderer archives, select both documentation commits from the manifest:

```sh
ASH_SPEC_REF=FULL-ASH-LANG-SHA ASH_DOC_REF=FULL-ASH-DOC-SHA npm run build
```

Replace the illustrative values with the recorded full commits. The compiler
checkpoint and study compiler target can differ. A manifest with `site.dirty`
set to `true` also requires retaining the local changes; its website commit
alone does not reconstruct those inputs. Exact inputs do not imply byte-identical
timestamps or output on every operating system.

## Later GitHub automation

The local commands are the future workflow entry points: regenerate `main`,
optionally propose an explicit pin change, run verification, then open a draft
PR containing the reviewed diff and build artifacts. No command currently
creates a PR, configures a remote, merges, or deploys.

After a website remote is configured, an upstream milestone/release job can
invoke a workflow through GitHub's
[repository dispatch event](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#repository_dispatch).
That workflow and its credentials must be configured in the destination
repository. Keep docs-only regeneration separate from advancing a reviewed
compiler or renderer pin.
