/**
 * Crawler results list feature — persisted channel table.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

import { List } from "@desk/ui/icons";

export const crawlerResultsFeature = {
  id: "crawler-results",
  path: "/features/crawler-results",
  navItem: {
    id: "crawler-results",
    path: "/features/crawler-results",
    labelKey: "nav.crawlerResults",
    icon: List,
  },
};

export { useCrawlerChannels } from "./use-crawler-channels";
