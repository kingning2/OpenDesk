/**
 * 闲鱼使用教程页（迁移自原前端 `pages/tutorial/Tutorial.tsx`）。
 *
 * 纯静态内容页：左侧目录（可展开/折叠）+ 右侧详细说明（滚动联动高亮）。
 * 无 API 依赖，数据为本地常量。
 */

import { useEffect, useRef, useState } from "react";
import {
  Bell,
  BookOpen,
  ChevronDown,
  ChevronRight,
  Circle,
  Filter,
  Image,
  Info,
  LayoutDashboard,
  MessageCircle,
  MessageSquare,
  Package,
  Shield,
  ShoppingCart,
  Users,
} from "@desk/ui/icons";
import { cn } from "@desk/ui";

/** 教程章节（与原前端结构一致）。 */
interface TutorialSection {
  id: string;
  icon?: React.ElementType;
  title: string;
  description: string;
  important?: boolean;
  children?: TutorialSection[];
}

const tutorialData: TutorialSection[] = [
  {
    id: "dashboard",
    icon: LayoutDashboard,
    title: "仪表盘",
    description: "系统首页，展示账号统计、订单统计、关键词统计等核心数据概览。",
  },
  {
    id: "accounts",
    icon: Users,
    title: "账号管理",
    description: "管理闲鱼账号，支持扫码登录、密码登录、手动输入Cookie等多种方式添加账号。",
    children: [
      { id: "accounts-qrcode", title: "扫码登录", description: "使用闲鱼APP扫描二维码登录，触发人脸验证后无法处理，不推荐使用。" },
      { id: "accounts-password", title: "账号密码", description: "使用闲鱼账号和密码登录，可能需要进行人脸验证。" },
      { id: "accounts-manual", title: "手动输入", description: "手动输入Cookie信息添加账号，适合高级用户。" },
      { id: "accounts-enable", title: "启用/禁用", description: "切换账号的启用状态，禁用后不再自动回复。" },
      { id: "accounts-ai", title: "AI回复", description: "开启/关闭该账号的AI智能回复功能。" },
      { id: "accounts-redelivery", title: "定时补发货", description: "开启后系统会定时检查未发货订单并自动发货。" },
      { id: "accounts-rate", title: "定时补评价", description: "开启后系统会定时检查未评价订单并自动评价。" },
      { id: "accounts-polish", title: "商品擦亮", description: "开启后系统会定时擦亮商品，提高曝光率。" },
      {
        id: "accounts-auto-confirm",
        title: "自动确认发货",
        description: "开启后买家下单时自动调用闲鱼API确认发货，并发送虚拟商品/卡券内容。想要自动发货必须开启此功能。",
        important: true,
      },
      { id: "accounts-ai-settings", title: "AI设置", description: "配置该账号的AI回复参数，包括AI模型、系统提示词等。" },
      { id: "accounts-default-reply", title: "默认回复", description: "设置该账号的默认回复内容，当没有匹配到关键词且未开启AI时发送。" },
      { id: "accounts-proxy", title: "代理设置", description: "配置该账号使用的网络代理，支持HTTP/SOCKS5代理。" },
      { id: "accounts-msg-wait", title: "消息等待", description: "设置消息等待时间，在该时间内收到的多条消息会合并处理，避免频繁回复。" },
      { id: "accounts-face", title: "人脸验证", description: "当账号需要人脸验证时，通过此功能完成验证流程。" },
      { id: "accounts-confirm-msg", title: "确认收货消息", description: "设置买家确认收货后自动发送的消息内容，如好评引导语。" },
      { id: "accounts-auto-rate", title: "自动评价", description: "配置自动评价的内容。收到评价请求消息时会自动评价买家。" },
    ],
  },
  {
    id: "items",
    icon: Package,
    title: "商品管理",
    description: "管理账号下的商品信息，配置发货规则、默认回复、AI提示词等。",
    children: [
      { id: "items-fetch", title: "获取商品", description: "从闲鱼获取账号下的所有商品信息并保存到本地。" },
      { id: "items-batch-reply", title: "新增默认回复", description: "批量为选中的商品设置默认回复内容。" },
      { id: "items-batch-ai", title: "新增AI提示词", description: "批量为选中的商品设置AI回复的提示词。" },
      { id: "items-delivery", title: "发货配置", description: "配置商品的自动发货规则，支持固定文字、批量数据、API接口、图片等多种卡券类型。" },
      { id: "items-reply", title: "默认回复", description: "设置商品的默认回复内容，买家咨询时自动发送。" },
      { id: "items-ai-prompt", title: "AI提示词", description: "设置商品的AI回复提示词，让AI更了解商品特点。" },
      { id: "items-spec-switch", title: "多规格开关", description: "开启后支持按规格匹配不同的发货内容。", important: true },
      { id: "items-multi-switch", title: "多数量发货开关", description: "开启后支持按购买数量发送多份卡券。" },
    ],
  },
  {
    id: "orders",
    icon: ShoppingCart,
    title: "订单管理",
    description: "查看和管理所有订单，支持手动发货、查看订单详情等操作。",
    children: [
      { id: "orders-manual-delivery", title: "手动发货", description: "对待发货订单执行手动发货操作。" },
      { id: "orders-detail", title: "查看详情", description: "查看订单的详细信息，包括收货地址、发货内容等。" },
    ],
  },
  {
    id: "keywords",
    icon: MessageSquare,
    title: "自动回复",
    description: "配置关键词自动回复规则，当买家消息包含关键词时自动发送预设回复。",
    children: [
      { id: "keywords-add", title: "添加关键词", description: "添加新的关键词回复规则。" },
      { id: "keywords-batch", title: "批量添加", description: "批量导入多个关键词规则。" },
      { id: "keywords-edit", title: "编辑", description: "修改关键词的触发词和回复内容。" },
      { id: "keywords-delete", title: "删除", description: "删除关键词规则。" },
    ],
  },
  {
    id: "message-filters",
    icon: Filter,
    title: "消息过滤",
    description: "配置消息过滤规则，符合规则的消息将被忽略不处理。",
    children: [
      { id: "filters-add", title: "添加过滤词", description: "添加新的过滤规则。" },
      { id: "filters-edit", title: "编辑", description: "修改过滤规则。" },
      { id: "filters-delete", title: "删除", description: "删除过滤规则。" },
    ],
  },
  {
    id: "risk-logs",
    icon: Shield,
    title: "风控日志",
    description: "查看账号风控拦截记录，包括滑块验证、处理状态等。",
  },
  {
    id: "notification-channels",
    icon: Bell,
    title: "通知渠道",
    description: "配置消息通知渠道，支持企业微信、钉钉、飞书、Bark等多种推送方式。",
    children: [
      { id: "channels-add", title: "添加渠道", description: "添加新的通知渠道配置。" },
      { id: "channels-test", title: "测试", description: "发送测试消息验证渠道配置是否正确。" },
      { id: "channels-edit", title: "编辑", description: "修改渠道配置。" },
      { id: "channels-delete", title: "删除", description: "删除通知渠道。" },
      { id: "channels-toggle", title: "启用/禁用", description: "切换渠道的启用状态。" },
    ],
  },
  {
    id: "message-notifications",
    icon: MessageCircle,
    title: "消息通知",
    description: "配置哪些账号需要推送通知，如新订单、新消息等。",
    children: [
      { id: "notifications-add", title: "添加规则", description: "添加新的通知规则。" },
      { id: "notifications-edit", title: "编辑", description: "修改通知规则。" },
      { id: "notifications-delete", title: "删除", description: "删除通知规则。" },
      { id: "notifications-toggle", title: "启用/禁用", description: "切换规则的启用状态。" },
    ],
  },
  {
    id: "product-publish",
    icon: Image,
    title: "商品发布",
    description: "素材库管理发布素材，单品/批量发布商品，维护随机地址池。",
    children: [
      { id: "publish-materials", title: "素材库", description: "管理商品素材（标题/价格/成色/图片），供单品和批量发布引用。" },
      { id: "publish-single", title: "单品发布", description: "填写商品信息，选择账号后发布单个商品。" },
      { id: "publish-batch", title: "批量发布", description: "多账号多素材并发发布，提升发布效率。" },
      { id: "publish-addresses", title: "地址库", description: "维护全局随机地址池与个人地址库，发布时自动分配。" },
      { id: "publish-logs", title: "发布日志", description: "查看所有商品发布记录及结果。" },
    ],
  },
  {
    id: "blacklist",
    icon: Shield,
    title: "黑名单管理",
    description: "管理禁止发货的买家黑名单，支持商品级、账户级、用户级三级匹配。",
  },
  {
    id: "other",
    icon: Info,
    title: "其他功能",
    description: "消息日志、个人设置等辅助功能。",
    children: [
      { id: "message-logs", title: "消息日志", description: "查看账号自动回复成功明细。" },
      { id: "personal-settings", title: "个人设置", description: "管理重发货触发关键字、联系方式、对接卡密秘钥等偏好。" },
    ],
  },
];

/** 目录项组件（onClick 为事件处理器，不触发 refs 规则误判）。 */
function TutorialNavItem({
  section,
  level,
  activeSection,
  expandedSections,
  onToggleExpand,
  onScrollTo,
}: {
  section: TutorialSection;
  level: number;
  activeSection: string;
  expandedSections: Set<string>;
  onToggleExpand: (id: string) => void;
  onScrollTo: (id: string) => void;
}): React.ReactNode {
  const Icon = section.icon ?? Circle;
  const isActive = activeSection === section.id;
  const hasChildren = (section.children?.length ?? 0) > 0;
  const isExpanded = expandedSections.has(section.id);

  return (
    <div key={section.id}>
      <div className="flex items-center">
        {hasChildren ? (
          <button
            type="button"
            onClick={() => onToggleExpand(section.id)}
            className="rounded p-1 hover:bg-muted"
            aria-label={isExpanded ? "收起" : "展开"}
          >
            {isExpanded ? (
              <ChevronDown className="size-3 text-muted-foreground" aria-hidden />
            ) : (
              <ChevronRight className="size-3 text-muted-foreground" aria-hidden />
            )}
          </button>
        ) : null}
        <button
          type="button"
          onClick={() => onScrollTo(section.id)}
          className={cn(
            "flex flex-1 items-center gap-2 rounded-md px-2 py-1.5 text-left text-[length:var(--text-sm)] transition-colors",
            !hasChildren && "ml-5",
            level > 0 && "text-[length:var(--text-xs)]",
            isActive
              ? "bg-primary/15 text-primary"
              : "text-muted-foreground hover:bg-muted/40 hover:text-foreground",
          )}
        >
          <Icon className={cn("shrink-0", level === 0 ? "size-4" : "size-3")} aria-hidden />
          <span className="truncate">{section.title}</span>
        </button>
      </div>
      {hasChildren && isExpanded ? (
        <div className={cn("ml-4", level > 0 && "ml-3")}>
          {section.children?.map((child) => (
            <TutorialNavItem
              key={child.id}
              section={child}
              level={level + 1}
              activeSection={activeSection}
              expandedSections={expandedSections}
              onToggleExpand={onToggleExpand}
              onScrollTo={onScrollTo}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}

/** 内容章节组件（DOM id 供滚动定位）。 */
function TutorialContentSection({ section, level }: { section: TutorialSection; level: number }): React.ReactNode {
  const Icon = section.icon ?? Circle;
  const HeadingTag = level === 0 ? "h2" : level === 1 ? "h3" : "h4";
  const headingClass =
    level === 0
      ? "text-xl font-bold text-primary"
      : level === 1
        ? "text-lg font-semibold text-emerald-600"
        : "text-base font-medium text-foreground";

  return (
    <div
      id={`tutorial-${section.id}`}
      className={cn("mb-6", level > 0 && "ml-4")}
    >
      <HeadingTag
        className={cn(
          "mb-2 flex items-center gap-2",
          headingClass,
          section.important && "text-red-600",
        )}
      >
        <Icon className={cn(level === 0 ? "size-5" : "size-4")} aria-hidden />
        {section.title}
        {section.important ? (
          <span className="rounded bg-red-500/15 px-1.5 py-0.5 text-[length:var(--text-xs)] text-red-600">
            重要
          </span>
        ) : null}
      </HeadingTag>
      <p
        className={cn(
          "mb-3 text-[length:var(--text-sm)]",
          section.important ? "font-medium text-red-600" : "text-muted-foreground",
        )}
      >
        {section.description}
      </p>
      {section.children?.map((child) => (
        <TutorialContentSection key={child.id} section={child} level={level + 1} />
      ))}
    </div>
  );
}

/**
 * 闲鱼使用教程页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuTutorialPage() {
  const [activeSection, setActiveSection] = useState("dashboard");
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set(["dashboard"]));
  const contentRef = useRef<HTMLDivElement>(null);

  function scrollToSection(sectionId: string) {
    setActiveSection(sectionId);
    const container = contentRef.current;
    const element = container?.querySelector(`#tutorial-${sectionId}`);
    if (element && container) {
      const containerTop = container.getBoundingClientRect().top;
      const elementTop = element.getBoundingClientRect().top;
      const offset = elementTop - containerTop + container.scrollTop - 20;
      container.scrollTo({ top: offset, behavior: "smooth" });
    }
  }

  function toggleExpand(sectionId: string) {
    setExpandedSections((prev) => {
      const next = new Set(prev);
      if (next.has(sectionId)) {
        next.delete(sectionId);
      } else {
        next.add(sectionId);
      }
      return next;
    });
  }

  // 滚动监听：更新当前章节高亮。
  useEffect(() => {
    const container = contentRef.current;
    if (!container) return;
    const handleScroll = () => {
      const containerTop = container.getBoundingClientRect().top;
      let currentSection = "dashboard";
      for (const section of tutorialData) {
        const element = container.querySelector(`#tutorial-${section.id}`);
        if (element) {
          const elementTop = element.getBoundingClientRect().top - containerTop;
          if (elementTop <= 100) {
            currentSection = section.id;
          }
        }
      }
      setActiveSection(currentSection);
    };
    container.addEventListener("scroll", handleScroll);
    return () => container.removeEventListener("scroll", handleScroll);
  }, []);

  return (
    <div className="flex h-full flex-col">
      {/* 页头 */}
      <div className="mb-4 flex items-center justify-between">
        <div>
          <h1 className="flex items-center gap-2 font-semibold">
            <BookOpen className="size-5" aria-hidden />
            使用教程
          </h1>
          <p className="text-[length:var(--text-sm)] text-muted-foreground">
            详细了解系统各项功能的使用方法
          </p>
        </div>
      </div>

      {/* 内容区：左目录 + 右内容 */}
      <div className="flex min-h-0 flex-1 gap-4">
        {/* 左侧目录 */}
        <div className="hidden w-72 shrink-0 lg:block">
          <div className="flex h-full flex-col overflow-hidden rounded-xl border border-border bg-shell">
            <div className="border-b border-border px-4 py-2">
              <h2 className="font-medium">目录</h2>
            </div>
            <div className="flex-1 overflow-y-auto p-2">
              {tutorialData.map((section) => (
                <TutorialNavItem
                  key={section.id}
                  section={section}
                  level={0}
                  activeSection={activeSection}
                  expandedSections={expandedSections}
                  onToggleExpand={toggleExpand}
                  onScrollTo={scrollToSection}
                />
              ))}
            </div>
          </div>
        </div>

        {/* 右侧内容 */}
        <div className="min-w-0 flex-1">
          <div className="flex h-full flex-col overflow-hidden rounded-xl border border-border bg-shell">
            <div
              ref={contentRef}
              className="flex-1 overflow-y-auto p-6"
            >
              {tutorialData.map((section) => (
                <TutorialContentSection key={section.id} section={section} level={0} />
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
