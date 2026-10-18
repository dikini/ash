import { readdir, readFile, stat } from "node:fs/promises";
import { resolve, relative, join } from "node:path";
import { pathToFileURL } from "node:url";
import { parseHTML } from "linkedom";

export async function checkSite(directory) {
  const root = resolve(directory);
  const documents = new Map();
  async function walk(folder) {
    for (const entry of await readdir(folder, { withFileTypes: true })) {
      const path = join(folder, entry.name);
      if (entry.isDirectory()) await walk(path);
      else if (entry.name.endsWith(".html"))
        documents.set(path, parseHTML(await readFile(path, "utf8")).document);
    }
  }
  await walk(root);
  const errors = [];
  for (const [file, document] of documents) {
    const pathname = `/${relative(root, file).split("\\").join("/")}`;
    for (const node of document.querySelectorAll(
      "a[href], link[href], img[src], script[src]",
    )) {
      const href = node.getAttribute("href") ?? node.getAttribute("src");
      if (!href || /^(?:https?:|mailto:|tel:|data:|\/\/)/i.test(href)) continue;
      const url = new URL(href, `https://local.invalid${pathname}`);
      let target = join(root, decodeURIComponent(url.pathname));
      try {
        if ((await stat(target)).isDirectory())
          target = join(target, "index.html");
        await stat(target);
        if (url.hash && documents.has(target)) {
          const id = decodeURIComponent(url.hash.slice(1));
          if (!documents.get(target).getElementById(id))
            errors.push(`${pathname}: missing anchor ${href}`);
        }
      } catch {
        errors.push(`${pathname}: missing local file ${href}`);
      }
    }
  }
  return errors;
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(resolve(process.argv[1])).href
) {
  const errors = await checkSite(process.argv[2] ?? "dist");
  if (errors.length) {
    console.error(errors.join("\n"));
    process.exitCode = 1;
  } else
    console.log("Local page links, anchors, and referenced assets verified.");
}
