/**
 * 闲鱼消息日志页（迁移自原前端 `pages/autoReplyLogs/AutoReplyLogs.tsx`）。
 *
 * 按原前端核心交互重写：账号/日期/消息类型/规则类型/发送状态筛选 + 回复明细分页表格。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/auto-reply-log`），复用 crates/app AutoReplyLogService。
 */

import { useEffect, useState } from "react";
import {
  Button,
  Input,
  Loading,
  PageScaffold,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  toast,
} from "@desk/ui";
import { Calendar, RefreshCw } from "@desk/ui/icons";
import { accountList, type XianyuAccount } from "@desk/platform/ipc/account";
import {
  autoReplyLogList,
  type AutoReplyLogItem,
} from "@desk/platform/ipc/auto-reply-log";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入
const PAGE_SIZE_OPTIONS = [10, 20, 50, 100];

const REPLY_STRATEGY_LABELS: Record<string, string> = {
  keyword: "关键词回复",
  ai: "AI回复",
  default: "默认回复",
  auto_delivery: "自动发货",
};

const DEFAULT_SCOPE_LABELS: Record<string, string> = {
  item: "商品默认",
  account: "账号默认",
};

const DECISION_REASON_LABELS: Record<string, string> = {
  processing: "处理中",
  self_message: "本人发送消息",
  system_message: "系统消息",
  auto_delivery_trigger: "自动发货触发消息",
  item_not_belong: "商品不属于当前账号",
  duplicate_message: "重复消息",
  skip_reply_filter: "命中过滤规则",
  reply_sent: "已发送回复",
  send_failed: "发送失败",
  no_rule_matched: "未匹配回复规则",
  failed: "处理失败",
  auto_delivery: "自动发货",
  chat_paused: "会话已暂停",
  chat_paused_after_delay: "延迟后会话已暂停",
  empty_reply: "回复内容为空",
  default_reply_once: "默认回复仅回复一次",
};

const SEND_STATUS_LABELS: Record<string, string> = {
  success: "发送成功",
  failed: "发送失败",
  unknown: "待确认",
  timeout: "超时",
};

function strategyLabel(log: AutoReplyLogItem): string {
  if (log.reply_strategy === "default" && log.default_reply_scope) {
    return DEFAULT_SCOPE_LABELS[log.default_reply_scope] ?? `默认回复(${log.default_reply_scope})`;
  }
  return REPLY_STRATEGY_LABELS[log.reply_strategy] ?? (log.reply_strategy || "-");
}

function labelOf(map: Record<string, string>, value?: string | null): string {
  if (!value) return "-";
  return map[value] ?? value;
}

/** 前端筛选项。 */
interface Filters {
  accountId: string;
  startDate: string;
  endDate: string;
  messageType: "auto_reply" | "auto_delivery";
  ruleType: string;
  sendStatus: string;
}

const EMPTY_FILTERS: Filters = {
  accountId: "",
  startDate: "",
  endDate: "",
  messageType: "auto_reply",
  ruleType: "",
  sendStatus: "",
};

/**
 * 闲鱼消息日志页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuMessageLogsPage() {
  const [logs, setLogs] = useState<AutoReplyLogItem[]>([]);
  const [accounts, setAccounts] = useState<XianyuAccount[]>([]);
  const [filters, setFilters] = useState<Filters>(EMPTY_FILTERS);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [total, setTotal] = useState(0);
  const [totalPages, setTotalPages] = useState(0);
  const [loading, setLoading] = useState(true);

  async function load(nextPage = page, nextPageSize = pageSize) {
    setLoading(true);
    try {
      const result = await autoReplyLogList({
        page: nextPage,
        page_size: nextPageSize,
        account_id: filters.accountId || undefined,
        start_date: filters.startDate || undefined,
        end_date: filters.endDate || undefined,
        matched_rule_type:
          filters.messageType === "auto_delivery" ? undefined : filters.ruleType || undefined,
        send_status: filters.sendStatus || undefined,
        message_type: filters.messageType,
      });
      setLogs(result.data);
      setPage(result.page);
      setPageSize(result.page_size);
      setTotal(result.total);
      setTotalPages(result.total_pages);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    let cancelled = false;
    void accountList(OWNER_ID)
      .then((list) => {
        if (!cancelled) setAccounts(list);
      })
      .catch(() => {
        // 账号列表加载失败不阻塞日志查询
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    void autoReplyLogList({ page: 1, page_size: 20, message_type: "auto_reply" })
      .then((result) => {
        if (cancelled) return;
        setLogs(result.data);
        setPage(result.page);
        setPageSize(result.page_size);
        setTotal(result.total);
        setTotalPages(result.total_pages);
      })
      .catch((error) => {
        if (!cancelled) {
          toast.error(error instanceof Error ? error.message : String(error));
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const patchFilters = (patch: Partial<Filters>) =>
    setFilters((current) => ({ ...current, ...patch }));

  return (
    <PageScaffold subtitle="闲鱼消息日志 — 自动回复成功明细，口径与账号管理今日回复一致">
      <div className="space-y-4">
        {/* 筛选区 */}
        <div className="rounded-xl border border-border bg-shell p-4">
          <div className="flex flex-wrap items-end gap-3">
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">筛选账号</span>
              <Select
                value={filters.accountId}
                onValueChange={(value) => patchFilters({ accountId: value })}
              >
                <SelectTrigger className="w-52" aria-label="筛选账号">
                  <SelectValue placeholder="全部账号" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">全部账号</SelectItem>
                  {accounts.map((account) => (
                    <SelectItem key={account.account_id} value={account.account_id}>
                      {account.remark
                        ? `${account.account_id} (${account.remark})`
                        : account.account_id}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">开始日期</span>
              <Input
                type="date"
                value={filters.startDate}
                onChange={(event) => patchFilters({ startDate: event.target.value })}
                className="w-40"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">结束日期</span>
              <Input
                type="date"
                value={filters.endDate}
                onChange={(event) => patchFilters({ endDate: event.target.value })}
                className="w-40"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">消息类型</span>
              <Select
                value={filters.messageType}
                onValueChange={(value) =>
                  patchFilters({ messageType: value as Filters["messageType"] })
                }
              >
                <SelectTrigger className="w-36" aria-label="消息类型">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="auto_reply">自动回复</SelectItem>
                  <SelectItem value="auto_delivery">自动发货</SelectItem>
                </SelectContent>
              </Select>
            </label>
            {filters.messageType !== "auto_delivery" ? (
              <label className="block space-y-1">
                <span className="text-[length:var(--text-xs)] text-muted-foreground">规则类型</span>
                <Select
                  value={filters.ruleType}
                  onValueChange={(value) => patchFilters({ ruleType: value })}
                >
                  <SelectTrigger className="w-36" aria-label="规则类型">
                    <SelectValue placeholder="全部类型" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="">全部类型</SelectItem>
                    <SelectItem value="keyword_item">商品关键词</SelectItem>
                    <SelectItem value="keyword_common">通用关键词</SelectItem>
                    <SelectItem value="ai">AI回复</SelectItem>
                    <SelectItem value="default_item">商品默认回复</SelectItem>
                    <SelectItem value="default_account">账号默认回复</SelectItem>
                  </SelectContent>
                </Select>
              </label>
            ) : null}
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">发送状态</span>
              <Select
                value={filters.sendStatus}
                onValueChange={(value) => patchFilters({ sendStatus: value })}
              >
                <SelectTrigger className="w-32" aria-label="发送状态">
                  <SelectValue placeholder="全部状态" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">全部状态</SelectItem>
                  <SelectItem value="success">发送成功</SelectItem>
                  <SelectItem value="failed">发送失败</SelectItem>
                  <SelectItem value="unknown">待确认</SelectItem>
                  <SelectItem value="timeout">超时</SelectItem>
                </SelectContent>
              </Select>
            </label>
            <Button onClick={() => void load(1)}>
              <Calendar className="size-4" aria-hidden />
              查询
            </Button>
            <Button variant="outline" onClick={() => void load()}>
              <RefreshCw className="size-4" aria-hidden />
              刷新
            </Button>
          </div>
        </div>

        {/* 明细表 */}
        {loading ? (
          <Loading size="lg" text="加载中..." className="py-16" />
        ) : logs.length === 0 ? (
          <div className="py-16 text-center text-muted-foreground">暂无消息日志</div>
        ) : (
          <div className="overflow-hidden rounded-xl border border-border">
            <div className="flex items-center justify-between border-b border-border bg-muted/30 px-4 py-2">
              <span className="text-[length:var(--text-sm)] font-medium">回复明细</span>
              <span className="rounded-full bg-primary/15 px-2 py-0.5 text-[length:var(--text-xs)] text-primary">
                {total} 条记录
              </span>
            </div>
            <div className="max-h-[60vh] overflow-auto">
              <table className="w-full min-w-[1400px] text-[length:var(--text-xs)]">
                <thead className="sticky top-0 bg-muted/80 text-muted-foreground">
                  <tr>
                    <th className="px-3 py-2 text-left font-medium">账号</th>
                    <th className="px-3 py-2 text-left font-medium">发送方</th>
                    <th className="px-3 py-2 text-left font-medium">商品</th>
                    <th className="px-3 py-2 text-left font-medium">策略</th>
                    <th className="px-3 py-2 text-left font-medium">命中关键词</th>
                    <th className="px-3 py-2 text-left font-medium">收到消息</th>
                    <th className="px-3 py-2 text-left font-medium">回复内容</th>
                    <th className="px-3 py-2 text-left font-medium">发送状态</th>
                    <th className="px-3 py-2 text-left font-medium">决策原因</th>
                    <th className="px-3 py-2 text-left font-medium">创建时间</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {logs.map((log) => (
                    <tr key={log.id} className="hover:bg-muted/30">
                      <td className="px-3 py-2 align-top">
                        <div className="font-medium text-primary">{log.account_id}</div>
                        {log.account_name ? (
                          <div className="text-muted-foreground">{log.account_name}</div>
                        ) : null}
                      </td>
                      <td className="px-3 py-2 align-top">
                        <div>{log.sender_user_name ?? "-"}</div>
                        <div className="text-muted-foreground">{log.sender_user_id}</div>
                      </td>
                      <td className="max-w-44 px-3 py-2 align-top">
                        <div className="truncate">{log.item_title ?? "-"}</div>
                        <div className="truncate text-muted-foreground">{log.item_id ?? ""}</div>
                      </td>
                      <td className="whitespace-nowrap px-3 py-2 align-top">
                        {strategyLabel(log)}
                      </td>
                      <td className="max-w-40 px-3 py-2 align-top">
                        <span className="break-all">{log.matched_keyword ?? "-"}</span>
                      </td>
                      <td className="max-w-52 px-3 py-2 align-top whitespace-pre-wrap break-words">
                        {log.source_message ?? "-"}
                      </td>
                      <td className="max-w-56 px-3 py-2 align-top whitespace-pre-wrap break-words">
                        <div>{log.reply_text ?? "-"}</div>
                        {log.reply_image_url ? (
                          <div className="break-all text-primary">{log.reply_image_url}</div>
                        ) : null}
                      </td>
                      <td className="whitespace-nowrap px-3 py-2 align-top">
                        {labelOf(SEND_STATUS_LABELS, log.send_status)}
                      </td>
                      <td className="max-w-40 px-3 py-2 align-top break-all">
                        {labelOf(DECISION_REASON_LABELS, log.decision_reason)}
                      </td>
                      <td className="whitespace-nowrap px-3 py-2 align-top">
                        {log.created_at ?? "-"}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {/* 分页 */}
        {total > 0 ? (
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="flex items-center gap-2 text-[length:var(--text-sm)] text-muted-foreground">
              <span>共 {total} 条</span>
              <Select
                value={String(pageSize)}
                onValueChange={(value) => void load(1, Number(value))}
              >
                <SelectTrigger className="w-20" aria-label="每页条数">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {PAGE_SIZE_OPTIONS.map((size) => (
                    <SelectItem key={size} value={String(size)}>
                      {size}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <span>条/页</span>
            </div>
            <div className="flex items-center gap-2">
              <Button
                size="sm"
                variant="outline"
                disabled={page <= 1}
                onClick={() => void load(Math.max(1, page - 1))}
              >
                上一页
              </Button>
              <span className="text-[length:var(--text-sm)] text-muted-foreground">
                第 {page} / {Math.max(1, totalPages)} 页
              </span>
              <Button
                size="sm"
                variant="outline"
                disabled={page >= totalPages}
                onClick={() => void load(Math.min(totalPages, page + 1))}
              >
                下一页
              </Button>
            </div>
          </div>
        ) : null}
      </div>
    </PageScaffold>
  );
}
