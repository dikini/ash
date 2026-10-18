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
  "updates/first-version",
  "contribute",
  "study",
  "search",
];

test("all reader entry paths have a statically generated page", () => {
  for (const page of pages) {
    assert.ok(
      existsSync(`dist/${page ? `${page}/` : ""}index.html`),
      `Missing route /${page}`,
    );
  }
});

test("study points to a pinned source edition without claiming a published download", () => {
  assert.ok(
    existsSync("dist/study/index.html"),
    "Study landing page must exist",
  );
  const html = readFileSync("dist/study/index.html", "utf8");
  assert.match(html, /github\.com\/dikini\/ash-doc\/tree\/[a-f0-9]{40}/);
  assert.match(html, /unpublished/i);
  assert.doesNotMatch(html, /href="[^"]+\.(?:pdf|epub)"/);
});
