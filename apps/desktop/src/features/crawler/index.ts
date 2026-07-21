import { Search } from "@desk/ui/icons";

export const crawlerFeature = {
  id: "crawler",
  path: "/features/crawler",
  navItem: {
    id: "crawler",
    path: "/features/crawler",
    labelKey: "nav.crawler",
    icon: Search,
  },
};

export { useCrawlerJob } from "./use-crawler-job";
