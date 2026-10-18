import { defineCollection } from "astro:content";
import { z } from "astro/zod";
import { glob } from "astro/loaders";

const pages = defineCollection({
  loader: glob({ pattern: "**/*.md", base: "./content" }),
  schema: z.object({
    title: z.string(),
    description: z.string(),
    section: z.enum([
      "Discover",
      "Learn",
      "Documentation",
      "Status",
      "Updates",
      "Contribute",
      "Study",
    ]),
    route: z.string().regex(/^[a-z0-9-]+(?:\/[a-z0-9-]+)*$/),
    updated: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
    draft: z.boolean().default(false),
  }),
});

export const collections = { pages };
