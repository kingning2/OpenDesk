import type { MetadataRoute } from 'next';

export const dynamic = 'force-static';

const SITE = 'https://kingning2.github.io/OpenDesk';

export default function robots(): MetadataRoute.Robots {
  return {
    rules: [{ userAgent: '*', allow: '/' }],
    sitemap: `${SITE}/sitemap.xml`,
  };
}
