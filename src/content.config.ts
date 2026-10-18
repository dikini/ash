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
      "Developer Diaries",
      "Contribute",
      "Study",
    ]),
    route: z.string().regex(/^[a-z0-9-]+(?:\/[a-z0-9-]+)*$/),
    updated: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
    published: z.string().regex(/^\d{4}-\d{2}-\d{2}$/).optional(),
    author: z.string().trim().min(1).optional(),
    draft: z.boolean().default(false),
  }).superRefine((page, ctx) => {
    if (page.section !== "Developer Diaries") return;
    if (!page.route.startsWith("diaries/")) ctx.addIssue({ code: 'custom', path: ['route'], message: 'Diary entries must use a diaries/ route' });
    if (!page.published) ctx.addIssue({ code: 'custom', path: ['published'], message: 'Diary entries require a publication date' });
    if (!page.author) ctx.addIssue({ code: 'custom', path: ['author'], message: 'Diary entries require an author' });
    if (page.published && page.updated < page.published) ctx.addIssue({ code: 'custom', path: ['updated'], message: 'Update date cannot precede publication' });
  }),
});

export const collections = { pages };
