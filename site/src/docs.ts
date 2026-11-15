import { getCollection, type CollectionEntry } from "astro:content";

export const SECTIONS = ["Getting started", "Guides", "Reference"] as const;

/** Docs pages in reading order: by section, then by `order`. */
export async function docsInOrder(): Promise<CollectionEntry<"docs">[]> {
  const all = await getCollection("docs");
  const rank = (d: CollectionEntry<"docs">) => SECTIONS.indexOf(d.data.section) * 1000 + d.data.order;
  return all.sort((a, b) => rank(a) - rank(b));
}
