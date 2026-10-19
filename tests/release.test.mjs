import test from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { createHash } from 'node:crypto';

const pages = [
  "",
  "discover",
  "learn",
  "learn/tour",
  "docs",
  "docs/install",
  "status",
  "updates",
  "diaries",
  "updates/first-version",
  "contribute",
  "study",
  "search",
];

test("developer diaries are discoverable and unpublished drafts stay private", () => {
  assert.ok(existsSync("dist/diaries/index.html"), "Missing diary landing page");
  const html = readFileSync("dist/diaries/index.html", "utf8");
  assert.match(readFileSync("dist/updates/index.html", "utf8"), /href="\/diaries\/"/);
  assert.doesNotMatch(html, /Draft: first developer diary/);
  assert.ok(!existsSync("dist/diaries/first-entry/index.html"));
});

test("all reader entry paths have a statically generated page", () => {
  for (const page of pages) {
    assert.ok(
      existsSync(`dist/${page ? `${page}/` : ""}index.html`),
      `Missing route /${page}`,
    );
  }
});

test("study serves the latest source snapshot with explicit targets and no release claim", () => {
  assert.ok(
    existsSync("dist/study/index.html"),
    "Study landing page must exist",
  );
  const html = readFileSync("dist/study/index.html", "utf8");
  assert.match(html, /github\.com\/dikini\/ash-doc\/tree\/[a-f0-9]{40}/);
  assert.match(html, /not a published book edition/i);
  assert.match(html, /href="\/study\/latest\/"/);
  const snapshot = JSON.parse(readFileSync('dist/study/latest/snapshot.json', 'utf8'));
  assert.match(snapshot.sourceRevision, /^[a-f0-9]{40}$/);
  assert.match(snapshot.ashRevision, /^[a-f0-9]{40}$/);
  assert.ok(snapshot.lessonCount > 0);
  assert.ok(existsSync('dist/study/latest/learn-what-is-an-ash-program.html'));
  assert.doesNotMatch(html, /href="[^"]+\.(?:pdf|epub)"/);
});

test('build manifest records the exact published source and pinned runtime identities', () => {
  const manifest = JSON.parse(readFileSync('dist/build-manifest.json', 'utf8'));
  const config = JSON.parse(readFileSync('sources.json', 'utf8'));
  const reference = JSON.parse(readFileSync('.cache/ash-spec/publication.json', 'utf8'));
  const study = JSON.parse(readFileSync('dist/study/latest/snapshot.json', 'utf8'));
  assert.equal(manifest.version, 1);
  assert.match(manifest.site.revision, /^[a-f0-9]{40}$/);
  assert.equal(typeof manifest.site.dirty, 'boolean');
  assert.equal(manifest.sources[config.reference.source].revision, reference.revision);
  assert.equal(manifest.sources[config.study.source].revision, study.sourceRevision);
  assert.equal(manifest.sources[config.study.source].ashRevision, study.ashRevision);
  assert.equal(manifest.checkpoint.revision, config.checkpoint.revision);
  assert.equal(manifest.inputs.sourceConfigSha256, createHash('sha256').update(readFileSync('sources.json')).digest('hex'));
  for (const [name, source] of Object.entries(config.renderers)) {
    assert.equal(manifest.renderers[name].revision, config.repositories[source].ref);
    assert.match(manifest.renderers[name].archiveSha256, /^[a-f0-9]{64}$/);
  }
  for (const path of ['docs/install', 'status', 'discover', 'contribute', 'docs']) {
    const html = readFileSync(`dist/${path}/index.html`, 'utf8');
    assert.doesNotMatch(html, /\{\{(?:checkpoint|source|repository)\./);
    assert.ok(html.includes(config.checkpoint.revision), `${path}: missing compiler checkpoint`);
  }
});
