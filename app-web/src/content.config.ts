import { glob } from "astro/loaders";
import { defineCollection, z } from "astro:content";

const docs = defineCollection({
  loader: glob({ pattern: "**/*.{md,mdx}", base: "../docs/guide" }),
  schema: z.object({
    title: z.string().optional(),
    description: z.string().optional(),
    status: z.string().optional(),
    order: z.number().optional(),
    // Accept map, empty `ods:`, or nested profile/status objects from authoring tools.
    ods: z.any().optional(),
  }).passthrough(),
});

const specs = defineCollection({
  loader: glob({ pattern: "**/*.{md,mdx}", base: "../specs" }),
  schema: z.object({
    title: z.string().optional(),
    description: z.string().optional(),
    status: z.string().optional(),
    order: z.number().optional(),
    ods: z.any().optional(),
  }).passthrough(),
});

export const collections = { docs, specs };

