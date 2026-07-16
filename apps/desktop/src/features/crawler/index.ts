import { Search } from "@desk/ui/icons";

export const crawlerFeature = {
  id: "crawler",
  path: "/features/crawler",
  navItem: {
    id: "crawler",
    path: "/features/crawler",
    label: "Crawler",
    icon: Search,
  },
};

export { CrawlerPage } from "./crawler-page";
export { useCrawlerJob } from "./use-crawler-job";
