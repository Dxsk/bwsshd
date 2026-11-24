import { defineCollection } from "astro:content";
import { glob } from "astro/loaders";
import { z } from "astro/zod";

const docs = defineCollection({
  loader: glob({ pattern: "**/*.md", base: "./src/content/docs" }),
  schema: z.object({
    title: z.string(),
    description: z.string(),
    section: z.enum(["Getting started", "Guides", "Reference"]),
    order: z.number(),
  }),
});

// The repo changelog, maintained by release-please, rendered as-is.
const changelog = defineCollection({
  loader: glob({ pattern: "CHANGELOG.md", base: ".." }),
});

export const collections = { docs, changelog };
