import { readFileSync, readdirSync } from "node:fs";
import { resolve, join } from "node:path";
import { spawnSync } from "node:child_process";
import assert from "node:assert/strict";
import { readSourceConfig } from './source-config.mjs';

const root = resolve(import.meta.dirname, "..");
const { checkpoint } = readSourceConfig().config;
const source = resolve(process.env.ASH_SOURCE ?? join(root, "../ash-lang"));
function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: source,
    encoding: "utf8",
    ...options,
  });
  if (result.error) throw result.error;
  return result;
}
const revision = run("git", ["rev-parse", "HEAD"]);
assert.equal(
  revision.status,
  0,
  `ASH_SOURCE must name an ash-lang checkout: ${revision.stderr}`,
);
assert.equal(
  revision.stdout.trim(),
  checkpoint.revision,
  "Compiler source must match the website checkpoint",
);
const clean = run("git", [
  "diff",
  "--quiet",
  "HEAD",
  "--",
  "Cargo.toml",
  "Cargo.lock",
  "rust-toolchain.toml",
  "src",
  "native-buffer",
]);
assert.equal(
  clean.status,
  0,
  "Compiler inputs have uncommitted changes; use a clean checkout at the pinned revision",
);
const build = run(
  "cargo",
  [
    `+${checkpoint.toolchain}`,
    "build",
    "--locked",
    "--bin",
    "ash",
    "--target-dir",
    join(source, "target"),
  ],
  { env: { ...process.env, RUSTC_WRAPPER: "" } },
);
assert.equal(build.status, 0, `Compiler build failed:\n${build.stderr}`);
const expectations = JSON.parse(
  readFileSync(join(root, "examples/expectations.json"), "utf8"),
);
assert.deepEqual(
  readdirSync(join(root, "examples"))
    .filter((f) => f.endsWith(".ash"))
    .sort(),
  expectations.map((e) => e.file).sort(),
  "Every canonical example needs an expectation",
);
const binary = join(
  source,
  "target/debug",
  process.platform === "win32" ? "ash.exe" : "ash",
);
for (const expected of expectations) {
  const result = run(binary, [
    "--message-format",
    "json",
    "--locale",
    "en",
    join(root, "examples", expected.file),
  ]);
  assert.equal(
    result.status,
    expected.status,
    `${expected.file}: ${result.stderr}`,
  );
  assert.equal(
    result.stdout,
    expected.stdout,
    `${expected.file}: value/type output changed`,
  );
  if (expected.reason)
    assert.equal(
      JSON.parse(result.stderr).reason,
      expected.reason,
      `${expected.file}: rejection reason changed`,
    );
  else
    assert.equal(result.stderr, "", `${expected.file}: unexpected diagnostics`);
  console.log(
    `Verified ${expected.file}: ${expected.reason ?? expected.stdout.trim()}`,
  );
}
console.log(`Examples verified against Ash ${checkpoint.revision}`);
