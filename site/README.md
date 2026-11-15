# bwsshd website

The landing page, docs and changelog for bwsshd, built with [Astro](https://astro.build) and a small
[three.js](https://threejs.org) kit for the isometric demos.

## Develop

```sh
cd site
npm install
npm run dev
```

The site is served on <http://localhost:4321>.

## Build

```sh
npm run build
```

This type-checks with `astro check`, then writes a fully static site to `site/dist/`. Host it on anything that
serves files.

Set the public URL at build time. It is used for canonical links and stamped into the install one-liners, the docs
and the install scripts:

```sh
SITE_URL=https://bwsshd.example.org npm run build
```

## Layout

| Path | What lives there |
|------|------------------|
| `src/pages/` | The landing page, `docs/[...slug]` and the changelog |
| `src/content/docs/` | Docs pages in Markdown; `section` and `order` in the front matter place them in the sidebar |
| `src/iso/kit.ts` | The scene kit: isometric camera, three-tone blocks, dither pass, labels, hover, orbit, timeline |
| `src/iso/scenes.ts` | The four scenes (hero and the three demos) |
| `src/naming.ts` | A port of `src/target.rs` that drives the naming playground |
| `public/install.sh`, `public/install.ps1` | One-line installers served by the site |
| `public/stills/` | SVG stills shown before the 3D loads, and without WebGL |

The changelog page renders the repository's `CHANGELOG.md` and the version badge reads `Cargo.toml`, so neither
needs editing by hand.

> Changing the naming rules in `src/target.rs`? Update `src/naming.ts` too, or the playground will drift.
