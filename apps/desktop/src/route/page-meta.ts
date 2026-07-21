/**
 * 路由 → 页面元信息（标题 / 描述使用 i18n key）。
 *
 * @author coisini
 * @created 2026-07-20
 */

/**
 * 页面元信息。
 *
 * @author coisini
 * @created 2026-07-20
 */
export interface PageMeta {
  /** 标题 i18n key。 */
  titleKey: string;
  /** 描述 i18n key。 */
  descriptionKey?: string;
}

const pageMetaByPath: Record<string, PageMeta> = {
  "/": {
    titleKey: "meta.home",
    descriptionKey: "meta.homeDescription",
  },
  "/features/agent": {
    titleKey: "meta.agent",
    descriptionKey: "meta.agentDescription",
  },
  "/features/crawler": {
    titleKey: "meta.crawler",
    descriptionKey: "meta.crawlerDescription",
  },
  "/features/chat": {
    titleKey: "meta.chat",
    descriptionKey: "meta.chatDescription",
  },
  "/features/mail": {
    titleKey: "meta.mail",
    descriptionKey: "meta.mailDescription",
  },
  "/features/knowledge": {
    titleKey: "meta.knowledge",
    descriptionKey: "meta.knowledgeDescription",
  },
};

/**
 * 按路径取页面元信息。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @param pathname - 路由路径
 * @returns 元信息；未知路径回退到应用名
 */
export function getPageMeta(pathname: string): PageMeta {
  return pageMetaByPath[pathname] ?? { titleKey: "app.name" };
}
