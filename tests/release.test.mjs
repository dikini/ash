import test from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";

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
