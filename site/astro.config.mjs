import { readdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "astro/config";

// The public URL is only known at deploy time; it feeds canonical links and the
// install one-liners. Override with SITE_URL when building for a real domain.
const PLACEHOLDER = "https://bwsshd.example";
const SITE = process.env.SITE_URL ?? PLACEHOLDER;

// The docs and the install scripts are static text that spell the placeholder
// URL; swap in the real one once the build is written. Match on the host alone:
// syntax highlighting splits URLs across several spans.
function stampSiteUrl() {
  return {
    name: "stamp-site-url",
    hooks: {
      "astro:build:done": async ({ dir }) => {
        if (SITE === PLACEHOLDER) return;
        const from = new URL(PLACEHOLDER).host;
        const to = new URL(SITE).host;
        const root = fileURLToPath(dir);
        for (const entry of await readdir(root, { recursive: true, withFileTypes: true })) {
          if (!entry.isFile() || !/\.(html|sh|ps1)$/.test(entry.name)) continue;
          const file = join(entry.parentPath, entry.name);
          const text = await readFile(file, "utf8");
          if (text.includes(from)) await writeFile(file, text.replaceAll(from, to));
        }
      },
    },
  };
}

export default defineConfig({
  site: SITE,
  trailingSlash: "ignore",
  build: { format: "directory" },
  integrations: [stampSiteUrl()],
  markdown: {
    shikiConfig: { theme: "github-dark-dimmed" },
  },
  vite: {
    server: { fs: { allow: [".."] } },
    // three.js is ~550 kB (~140 kB gzipped) and only fetched lazily when a
    // scene nears the viewport, so the default 500 kB warning is noise here.
    build: { chunkSizeWarningLimit: 650 },
  },
});
