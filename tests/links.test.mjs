import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { checkSite } from "../scripts/check-links.mjs";

test("link verification detects missing local files and fragment targets", async () => {
  const root = await mkdtemp(join(tmpdir(), "ash-links-"));
  try {
    await writeFile(
      join(root, "index.html"),
      '<a href="/missing/">Missing</a><a href="#absent">Absent</a><a href="https://example.com/">External</a>',
    );
    const errors = await checkSite(root);
    assert.equal(errors.length, 2);
    assert.match(errors.join("\n"), /missing/);
    assert.match(errors.join("\n"), /absent/);
  } finally {
    await rm(root, { recursive: true });
  }
});

test("link verification accepts relative assets and existing anchors", async () => {
  const root = await mkdtemp(join(tmpdir(), "ash-links-"));
  try {
    await writeFile(
      join(root, "index.html"),
      '<h1 id="hello">Hi</h1><a href="#hello">Hello</a><img src="./icon.svg" alt="Icon">',
    );
    await writeFile(join(root, "icon.svg"), "<svg/>");
    assert.deepEqual(await checkSite(root), []);
  } finally {
    await rm(root, { recursive: true });
  }
});
