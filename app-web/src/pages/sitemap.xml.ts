// FILE: sitemap.xml.ts
// Purpose: Automatically generates production XML sitemap for Open Document Spec (ODS).

import type { APIRoute } from "astro";

const SITE_URL = "https://opendocify.com";

const pages = [
  { url: "/", priority: "1.0", changefreq: "daily" },
  { url: "/docs", priority: "0.9", changefreq: "weekly" },
  { url: "/docs/introduction", priority: "0.8", changefreq: "weekly" },
  { url: "/docs/quickstart", priority: "0.8", changefreq: "weekly" },
  { url: "/docs/adoption", priority: "0.8", changefreq: "weekly" },
  { url: "/docs/tooling", priority: "0.8", changefreq: "weekly" },
  { url: "/docs/profiles", priority: "0.8", changefreq: "weekly" },
  { url: "/docs/advanced", priority: "0.8", changefreq: "weekly" },
  { url: "/docs/troubleshooting-and-diagnostics", priority: "0.7", changefreq: "monthly" },
  { url: "/docs/enterprise-deployment", priority: "0.7", changefreq: "monthly" },
  { url: "/docs/features", priority: "0.7", changefreq: "weekly" },
  { url: "/docs/roi-calculator", priority: "0.7", changefreq: "monthly" },
  { url: "/docs/faq", priority: "0.7", changefreq: "monthly" },
  { url: "/docs/use-cases", priority: "0.7", changefreq: "weekly" },
  { url: "/docs/use-cases/compliance-governance", priority: "0.7", changefreq: "monthly" },
  { url: "/docs/use-cases/cross-repo-contracts", priority: "0.7", changefreq: "monthly" },
  { url: "/docs/use-cases/secrets-isolation", priority: "0.7", changefreq: "monthly" },
  { url: "/spec", priority: "0.9", changefreq: "weekly" },
  { url: "/spec/ods/intro", priority: "0.9", changefreq: "weekly" },
  { url: "/spec/ods/keys", priority: "0.9", changefreq: "weekly" },
  { url: "/spec/ods/core", priority: "0.9", changefreq: "weekly" },
  { url: "/spec/ods/profiles", priority: "0.8", changefreq: "weekly" },
  { url: "/spec/ods/graph", priority: "0.8", changefreq: "weekly" },
  { url: "/spec/ods/context", priority: "0.8", changefreq: "weekly" },
  { url: "/spec/ods/indexes", priority: "0.8", changefreq: "weekly" },
  { url: "/spec/ods/assets", priority: "0.8", changefreq: "weekly" },
  { url: "/spec/ods/validation", priority: "0.8", changefreq: "weekly" },
  { url: "/spec/ods/scope", priority: "0.7", changefreq: "monthly" },
  { url: "/spec/okf/intro", priority: "0.7", changefreq: "monthly" },
  { url: "/spec/okf/keys", priority: "0.7", changefreq: "monthly" },
  { url: "/spec/skills/intro", priority: "0.7", changefreq: "monthly" },
  { url: "/spec/skills/keys", priority: "0.7", changefreq: "monthly" },
  { url: "/features", priority: "0.8", changefreq: "weekly" },
  { url: "/examples", priority: "0.8", changefreq: "weekly" },
  { url: "/download", priority: "0.8", changefreq: "weekly" },
  { url: "/pricing", priority: "0.8", changefreq: "weekly" },
  { url: "/changelog", priority: "0.7", changefreq: "weekly" },
  { url: "/privacy", priority: "0.6", changefreq: "monthly" },
];

export const GET: APIRoute = async () => {
  const lastmod = new Date().toISOString().split("T")[0];

  const sitemapXml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemap.org/schemas/sitemap/0.9">
${pages
  .map(
    (page) => `  <url>
    <loc>${SITE_URL}${page.url}</loc>
    <lastmod>${lastmod}</lastmod>
    <changefreq>${page.changefreq}</changefreq>
    <priority>${page.priority}</priority>
  </url>`
  )
  .join("\n")}
</urlset>`;

  return new Response(sitemapXml, {
    headers: {
      "Content-Type": "application/xml; charset=utf-8",
    },
  });
};
