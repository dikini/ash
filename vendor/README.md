# Specification renderer runtimes

These npm archives contain the HTML runtime files of the existing local
`railroad` and `sequent-md` projects, rather than their editor extensions or
standalone SVG exporters. Renderer JavaScript is copied unchanged. Package
metadata exposes only these runtime files and the dependencies they require.

`renderers.json` records upstream repository commits and SHA-256 hashes of every
copied file. The railroad archive includes its MIT license and third-party
notices. Sequent-md currently declares no license file in its source repository.

To update from sibling checkouts:

```sh
# First build railroad in its own checkout if its renderer has changed.
node scripts/vendor-spec-renderers.mjs
npm install
```

`RAILROAD_SOURCE` and `SEQUENT_SOURCE` override the sibling paths. Review source
changes and the generated provenance, then build the site and run its tests.
The update script rejects uncommitted tracked renderer changes; build railroad
from its committed sources before packaging its generated runtime.
When replacing an archive at the same version, remove its installed package
and lockfile entry before `npm install` so npm recalculates archive integrity.

Ordinary `npm ci` uses these archives and needs no sibling renderer checkouts,
Java, sbt, or client-side diagram bundle.
