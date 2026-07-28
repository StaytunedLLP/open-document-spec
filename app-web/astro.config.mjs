import { defineConfig } from "astro/config";
import mdx from "@astrojs/mdx";

export default defineConfig({
  integrations: [mdx()],
  server: {
    port: Number(process.env.PORT ?? 4173),
  },
});

