import type { MetadataRoute } from 'next';

export const dynamic = 'force-static';

const SITE = 'https://kingning2.github.io/OpenDesk';

export default function sitemap(): MetadataRoute.Sitemap {
  return [
    {
      url: `${SITE}/`,
      lastModified: new Date('2026-08-18'),
      changeFrequency: 'weekly',
      priority: 1,
    },
  ];
}
