import { defineConfig } from "astro/config";
import mdx from "@astrojs/mdx";

export default defineConfig({
  integrations: [mdx()],
  server: {
    port: Number(process.env.PORT ?? 4173),
  },
  // Legacy flat /spec/* paths → multi-spec one-word modules
  redirects: {
    "/spec/spec": "/spec/ods/core",
    "/spec/SPEC": "/spec/ods/core",
    "/spec/profiles": "/spec/ods/profiles",
    "/spec/graph": "/spec/ods/graph",
    "/spec/context": "/spec/ods/context",
    "/spec/indexes": "/spec/ods/indexes",
    "/spec/resources-and-code": "/spec/ods/assets",
    "/spec/validation": "/spec/ods/validation",
    "/spec/non-goals": "/spec/ods/scope",
    "/specs/spec": "/spec/ods/core",
    "/specs/profiles": "/spec/ods/profiles",
    "/specs/graph": "/spec/ods/graph",
    "/specs/context": "/spec/ods/context",
    "/specs/indexes": "/spec/ods/indexes",
    "/specs/resources-and-code": "/spec/ods/assets",
    "/specs/validation": "/spec/ods/validation",
    "/specs/non-goals": "/spec/ods/scope",
  },
});

