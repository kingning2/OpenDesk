/**
 * 闲鱼消息通知管理页（迁移自原前端 `pages/notifications/MessageNotifications.tsx`）。
 *
 * 按原前端核心交互重写：账号×渠道绑定规则列表 + 添加通知（选账号/渠道/启用）+ 启用切换 + 删除。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/notification`），复用 crates/app NotificationService。
 */

import { useEffect, useState } from "react";
import {
  Button,
  ConfirmModal,
  Loading,
  PageScaffold,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  toast,
} from "@desk/ui";
import { Plus, Trash2 } from "@desk/ui/icons";
import { accountList, type XianyuAccount } from "@desk/platform/ipc/account";
import {
  notificationChannelList,
  notificationDelete,
  notificationList,
  notificationSet,
  type MessageNotification,
  type NotificationChannel,
} from "@desk/platform/ipc/notification";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入

/** 添加通知表单值。 */
interface AddFormState {
  accountId: string;
  channelId: string;
  enabled: boolean;
}

/**
 * 闲鱼消息通知管理页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuMessageNotificationsPage() {
  const [notifications, setNotifications] = useState<MessageNotification[]>([]);
  const [channels, setChannels] = useState<NotificationChannel[]>([]);
  const [accounts, setAccounts] = useState<XianyuAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [formOpen, setFormOpen] = useState(false);
  const [form, setForm] = useState<AddFormState>({ accountId: "", channelId: "", enabled: true });
  const [saving, setSaving] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<MessageNotification | null>(null);

  async function loadNotifications() {
    try {
      const list = await notificationList(OWNER_ID);
      setNotifications(list);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  useEffect(() => {
    let cancelled = false;
    void Promise.all([
      notificationList(OWNER_ID),
      notificationChannelList(OWNER_ID),
      accountList(OWNER_ID),
    ])
      .then(([list, channelList, accountListData]) => {
        if (cancelled) return;
        setNotifications(list);
        setChannels(channelList);
        setAccounts(accountListData);
      })
      .catch((error) => {
        if (!cancelled) {
          toast.error(error instanceof Error ? error.message : String(error));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  async function handleToggle(notification: MessageNotification) {
    try {
      await notificationSet(
        OWNER_ID,
        notification.account_id,
        notification.channel_id,
        !notification.enabled,
      );
      toast.success(notification.enabled ? "通知已禁用" : "通知已启用");
      await loadNotifications();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function handleAdd() {
    if (!form.accountId) {
      toast.error("请选择账号");
      return;
    }
    if (!form.channelId) {
      toast.error("请选择通知渠道");
      return;
    }
    setSaving(true);
    try {
      await notificationSet(OWNER_ID, form.accountId, Number(form.channelId), form.enabled);
      toast.success("通知已添加");
      setFormOpen(false);
      await loadNotifications();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete() {
    if (!deleteTarget) return;
    try {
      await notificationDelete(OWNER_ID, deleteTarget.id);
      toast.success("通知已删除");
      setDeleteTarget(null);
      await loadNotifications();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  const accountLabel = (accountId: string) => {
    const account = accounts.find((item) => item.account_id === accountId);
    return account?.remark ? `${accountId} (${account.remark})` : accountId;
  };

  const channelLabel = (notification: MessageNotification) =>
    notification.channel_name ?? `渠道 ${notification.channel_id}`;

  return (
    <PageScaffold subtitle="闲鱼消息通知 — 账号 × 渠道绑定规则">
      <div className="space-y-4">
        {/* 工具栏 */}
        <div className="flex items-center justify-end gap-3">
          <Button onClick={() => setFormOpen(true)}>
            <Plus className="size-4" aria-hidden />
            添加通知
          </Button>
        </div>

        {/* 规则列表 */}
        {loading ? (
          <Loading size="lg" text="加载中..." className="py-16" />
        ) : notifications.length === 0 ? (
          <div className="py-16 text-center text-muted-foreground">暂无消息通知配置</div>
        ) : (
          <div className="overflow-hidden rounded-xl border border-border">
            <table className="w-full text-[length:var(--text-sm)]">
              <thead className="bg-muted/50 text-muted-foreground">
                <tr>
                  <th className="px-4 py-2.5 text-left font-medium">账号</th>
                  <th className="px-4 py-2.5 text-left font-medium">通知渠道</th>
                  <th className="px-4 py-2.5 text-left font-medium">状态</th>
                  <th className="px-4 py-2.5 text-right font-medium">操作</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {notifications.map((notification) => (
                  <tr key={notification.id} className="hover:bg-muted/30">
                    <td className="px-4 py-2.5 font-medium">{accountLabel(notification.account_id)}</td>
                    <td className="px-4 py-2.5 text-muted-foreground">
                      {channelLabel(notification)}
                    </td>
                    <td className="px-4 py-2.5">
                      <span
                        className={
                          notification.enabled
                            ? "rounded-full bg-emerald-500/15 px-2 py-0.5 text-[length:var(--text-xs)] text-emerald-600"
                            : "rounded-full bg-muted px-2 py-0.5 text-[length:var(--text-xs)] text-muted-foreground"
                        }
                      >
                        {notification.enabled ? "启用" : "禁用"}
                      </span>
                    </td>
                    <td className="px-4 py-2.5 text-right">
                      <div className="flex items-center justify-end gap-1">
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => void handleToggle(notification)}
                        >
                          {notification.enabled ? "禁用" : "启用"}
                        </Button>
                        <Button
                          size="sm"
                          variant="ghost"
                          className="text-destructive"
                          onClick={() => setDeleteTarget(notification)}
                        >
                          <Trash2 className="size-3.5" aria-hidden />
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* 添加通知弹窗 */}
      <ConfirmModal
        isOpen={formOpen}
        title="添加消息通知"
        message={
          <div className="space-y-3 text-left">
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">选择账号</span>
              <Select
                value={form.accountId}
                onValueChange={(value) => setForm((current) => ({ ...current, accountId: value }))}
              >
                <SelectTrigger aria-label="选择账号">
                  <SelectValue placeholder="请选择账号" />
                </SelectTrigger>
                <SelectContent>
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
              <span className="text-[length:var(--text-sm)] text-muted-foreground">选择通知渠道</span>
              <Select
                value={form.channelId}
                onValueChange={(value) => setForm((current) => ({ ...current, channelId: value }))}
              >
                <SelectTrigger aria-label="选择通知渠道">
                  <SelectValue placeholder="请选择通知渠道" />
                </SelectTrigger>
                <SelectContent>
                  {channels.map((channel) => (
                    <SelectItem key={channel.id} value={String(channel.id)}>
                      {channel.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </label>
            <label className="flex items-center justify-between">
              <span className="text-[length:var(--text-sm)]">启用此通知</span>
              <input
                type="checkbox"
                checked={form.enabled}
                onChange={(event) =>
                  setForm((current) => ({ ...current, enabled: event.target.checked }))
                }
                className="size-4 accent-primary"
              />
            </label>
          </div>
        }
        confirmText={saving ? "保存中…" : "保存"}
        loading={saving}
        onConfirm={() => void handleAdd()}
        onCancel={() => setFormOpen(false)}
      />

      {/* 删除确认 */}
      <ConfirmModal
        isOpen={deleteTarget !== null}
        type="danger"
        title="删除消息通知"
        message={`确认删除账号「${deleteTarget?.account_id ?? ""}」的这条通知规则？`}
        confirmText="删除"
        onConfirm={() => void handleDelete()}
        onCancel={() => setDeleteTarget(null)}
      />
    </PageScaffold>
  );
}
