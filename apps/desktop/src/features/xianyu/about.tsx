/**
 * 闲鱼关于页（迁移自原前端 `pages/about/About.tsx`，静态适配版）。
 *
 * 保留：系统标识 + 主要功能列表 + 贡献者 + 相关链接。
 * 移除：更新检查 / 群二维码 / 使用人数（依赖 SaaS 服务端）。
 */

import { PageScaffold } from "@desk/ui";
import {
  Bell,
  Bot,
  MessageSquare,
  Package,
  ShoppingCart,
  Truck,
  Users,
} from "@desk/ui/icons";

const FEATURES = [
  { title: "多账号管理", desc: "同时管理多个账号", icon: Users, color: "text-blue-500" },
  { title: "智能回复", desc: "关键词自动回复", icon: MessageSquare, color: "text-emerald-600" },
  { title: "AI 助手", desc: "智能处理复杂问题", icon: Bot, color: "text-purple-500" },
  { title: "自动发货", desc: "支持卡密发货", icon: Truck, color: "text-orange-600" },
  { title: "消息通知", desc: "多渠道推送", icon: Bell, color: "text-pink-500" },
  { title: "订单管理", desc: "订单状态跟踪", icon: ShoppingCart, color: "text-cyan-600" },
  { title: "卡券管理", desc: "多种卡券类型", icon: Package, color: "text-amber-600" },
];

/**
 * 闲鱼关于页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuAboutPage() {
  return (
    <PageScaffold subtitle="关于闲鱼自动化管理系统">
      <div className="mx-auto max-w-4xl space-y-4">
        {/* 系统标识 */}
        <div className="mb-6 text-center">
          <div className="mx-auto mb-4 flex size-16 items-center justify-center rounded-2xl bg-gradient-to-br from-blue-500 to-blue-600 shadow-md">
            <MessageSquare className="size-8 text-white" aria-hidden />
          </div>
          <h1 className="text-2xl font-bold">闲鱼自动回复管理系统</h1>
          <p className="mt-1 text-[length:var(--text-sm)] text-muted-foreground">
            智能管理您的闲鱼店铺，提升客服效率
          </p>
        </div>

        {/* 主要功能 */}
        <div className="rounded-xl border border-border bg-shell p-4">
          <h2 className="mb-3 font-medium">主要功能</h2>
          <div className="grid grid-cols-2 gap-3 md:grid-cols-3">
            {FEATURES.map((feature) => {
              const Icon = feature.icon;
              return (
                <div
                  key={feature.title}
                  className="flex items-center gap-3 rounded-lg bg-muted/40 p-4"
                >
                  <div
                    className={`flex size-10 shrink-0 items-center justify-center rounded-lg bg-background shadow-sm ${feature.color}`}
                  >
                    <Icon className="size-5" aria-hidden />
                  </div>
                  <div className="text-left">
                    <p className="text-[length:var(--text-sm)] font-medium">{feature.title}</p>
                    <p className="text-[length:var(--text-xs)] text-muted-foreground">
                      {feature.desc}
                    </p>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* 相关链接 */}
        <div className="rounded-xl border border-border bg-shell p-4">
          <h2 className="mb-3 font-medium">相关链接</h2>
          <div className="flex gap-3">
            <a
              href="https://github.com/zhinianboke/xianyu-auto-reply"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-2 rounded-lg bg-foreground px-4 py-2 text-[length:var(--text-sm)] text-background transition-colors hover:opacity-80"
            >
              GitHub
            </a>
          </div>
        </div>
      </div>
    </PageScaffold>
  );
}
