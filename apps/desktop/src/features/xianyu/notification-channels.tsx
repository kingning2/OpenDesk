/**
 * 闲鱼通知渠道管理页（迁移自原前端 `pages/notifications/NotificationChannels.tsx`）。
 *
 * 按原前端核心交互重写：渠道类型网格新建 + 已配置渠道列表（启用/测试/编辑/删除）。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/notification`），复用 crates/app NotificationService。
 *
 * 说明：渠道「测试」为配置校验（实际投递由 sidecar 执行）。
 */

import { useEffect, useState } from "react";
import {
  Button,
  ConfirmModal,
  Input,
  Loading,
  PageScaffold,
  Textarea,
  toast,
} from "@desk/ui";
import { Bell, Pencil, Plus, Send, Trash2 } from "@desk/ui/icons";
import {
  CHANNEL_TYPES,
  channelKindLabel,
  notificationChannelCreate,
  notificationChannelDelete,
  notificationChannelList,
  notificationChannelSetEnabled,
  notificationChannelTest,
  notificationChannelUpdate,
  type ChannelKind,
  type NotificationChannel,
} from "@desk/platform/ipc/notification";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入

/** 配置弹窗表单值。 */
interface ChannelFormState {
  name: string;
  config: string;
  enabled: boolean;
}

/**
 * 闲鱼通知渠道管理页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuNotificationChannelsPage() {
  const [channels, setChannels] = useState<NotificationChannel[]>([]);
  const [loading, setLoading] = useState(true);
  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<NotificationChannel | null>(null);
  const [formKind, setFormKind] = useState<ChannelKind>("dingtalk");
  const [form, setForm] = useState<ChannelFormState>({ name: "", config: "", enabled: true });
  const [saving, setSaving] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<NotificationChannel | null>(null);

  async function load() {
    try {
      const list = await notificationChannelList(OWNER_ID);
      setChannels(list);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  useEffect(() => {
    let cancelled = false;
    void notificationChannelList(OWNER_ID)
      .then((list) => {
        if (cancelled) return;
        setChannels(list);
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

  const countByKind = (kind: ChannelKind) =>
    channels.filter((channel) => channel.kind === kind).length;

  function openCreate(kind: ChannelKind) {
    const info = CHANNEL_TYPES.find((item) => item.type === kind);
    const count = countByKind(kind);
    const baseName = info?.label ?? kind;
    setEditing(null);
    setFormKind(kind);
    setForm({
      name: count > 0 ? `${baseName} ${count + 1}` : baseName,
      config: info?.defaultConfig ? JSON.stringify(info.defaultConfig, null, 2) : "",
      enabled: true,
    });
    setFormOpen(true);
  }

  function openEdit(channel: NotificationChannel) {
    const info = CHANNEL_TYPES.find((item) => item.type === channel.kind);
    setEditing(channel);
    setFormKind(channel.kind);
    setForm({
      name: channel.name,
      config: channel.config.trim() || (info?.defaultConfig ? JSON.stringify(info.defaultConfig, null, 2) : ""),
      enabled: channel.enabled,
    });
    setFormOpen(true);
  }

  async function handleSubmit() {
    if (!form.name.trim()) {
      toast.error("请输入渠道名称");
      return;
    }
    let parsed: Record<string, unknown>;
    try {
      parsed = form.config.trim() ? (JSON.parse(form.config) as Record<string, unknown>) : {};
    } catch {
      toast.error("配置 JSON 格式错误");
      return;
    }
    const config = JSON.stringify(parsed);
    setSaving(true);
    try {
      if (editing) {
        await notificationChannelUpdate(OWNER_ID, { ...editing, name: form.name.trim(), config, enabled: form.enabled });
        toast.success("渠道已更新");
      } else {
        await notificationChannelCreate(OWNER_ID, {
          name: form.name.trim(),
          kind: formKind,
          config,
          enabled: form.enabled,
        });
        toast.success("渠道已添加");
      }
      setFormOpen(false);
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  async function handleToggle(channel: NotificationChannel) {
    try {
      await notificationChannelSetEnabled(OWNER_ID, channel.id, !channel.enabled);
      toast.success(channel.enabled ? "渠道已禁用" : "渠道已启用");
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function handleTest(channel: NotificationChannel) {
    try {
      const message = await notificationChannelTest(OWNER_ID, channel.id);
      toast.success(message);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function handleDelete() {
    if (!deleteTarget) return;
    try {
      await notificationChannelDelete(OWNER_ID, deleteTarget.id);
      toast.success("渠道已删除");
      setDeleteTarget(null);
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <PageScaffold subtitle="闲鱼消息通知渠道 — 钉钉 / 飞书 / 邮件等通知方式">
      <div className="space-y-5">
        {/* 渠道类型网格（点击新建） */}
        <div className="rounded-xl border border-border bg-shell p-4">
          <h2 className="mb-1 font-medium">选择通知方式</h2>
          <p className="mb-4 text-[length:var(--text-sm)] text-muted-foreground">
            点击下方卡片新建通知渠道，同一类型可创建多个
          </p>
          <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
            {CHANNEL_TYPES.map((info) => {
              const count = countByKind(info.type);
              return (
                <button
                  key={info.type}
                  type="button"
                  onClick={() => openCreate(info.type)}
                  className="relative rounded-xl border border-border p-4 text-center transition-colors hover:border-primary/50"
                >
                  {count > 0 ? (
                    <span className="absolute right-2 top-2 rounded-full bg-primary/15 px-1.5 text-[length:var(--text-xs)] text-primary">
                      {count}
                    </span>
                  ) : null}
                  <Bell className="mx-auto mb-2 size-5" aria-hidden />
                  <h3 className="text-[length:var(--text-sm)] font-medium">{info.label}</h3>
                  <p className="mt-0.5 text-[length:var(--text-xs)] text-muted-foreground">{info.desc}</p>
                  <span className="mt-3 inline-flex items-center gap-1 rounded border border-primary/30 px-2 py-0.5 text-[length:var(--text-xs)] text-primary">
                    <Plus className="size-3" aria-hidden />
                    {count > 0 ? "新建" : "配置"}
                  </span>
                </button>
              );
            })}
          </div>
        </div>

        {/* 已配置渠道列表 */}
        {loading ? (
          <Loading size="lg" text="加载中..." className="py-16" />
        ) : channels.length === 0 ? (
          <div className="py-12 text-center text-muted-foreground">暂无已配置渠道</div>
        ) : (
          <div className="overflow-hidden rounded-xl border border-border">
            <table className="w-full text-[length:var(--text-sm)]">
              <thead className="bg-muted/50 text-muted-foreground">
                <tr>
                  <th className="px-4 py-2.5 text-left font-medium">渠道名称</th>
                  <th className="px-4 py-2.5 text-left font-medium">类型</th>
                  <th className="px-4 py-2.5 text-left font-medium">状态</th>
                  <th className="px-4 py-2.5 text-right font-medium">操作</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {channels.map((channel) => (
                  <tr key={channel.id} className="hover:bg-muted/30">
                    <td className="px-4 py-2.5 font-medium">{channel.name}</td>
                    <td className="px-4 py-2.5 text-muted-foreground">
                      {channelKindLabel(channel.kind)}
                    </td>
                    <td className="px-4 py-2.5">
                      <span
                        className={
                          channel.enabled
                            ? "rounded-full bg-emerald-500/15 px-2 py-0.5 text-[length:var(--text-xs)] text-emerald-600"
                            : "rounded-full bg-muted px-2 py-0.5 text-[length:var(--text-xs)] text-muted-foreground"
                        }
                      >
                        {channel.enabled ? "启用" : "停用"}
                      </span>
                    </td>
                    <td className="px-4 py-2.5 text-right">
                      <div className="flex items-center justify-end gap-1">
                        <Button size="sm" variant="outline" onClick={() => void handleToggle(channel)}>
                          {channel.enabled ? "停用" : "启用"}
                        </Button>
                        <Button size="sm" variant="ghost" onClick={() => void handleTest(channel)}>
                          <Send className="size-3.5" aria-hidden />
                        </Button>
                        <Button size="sm" variant="ghost" onClick={() => openEdit(channel)}>
                          <Pencil className="size-3.5" aria-hidden />
                        </Button>
                        <Button
                          size="sm"
                          variant="ghost"
                          className="text-destructive"
                          onClick={() => setDeleteTarget(channel)}
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

      {/* 配置弹窗（新建 / 编辑） */}
      <ConfirmModal
        isOpen={formOpen}
        title={`${editing ? "编辑" : "配置"}${channelKindLabel(formKind)}`}
        message={
          <div className="space-y-3 text-left">
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">渠道名称</span>
              <Input
                value={form.name}
                onChange={(event) => setForm((current) => ({ ...current, name: event.target.value }))}
                placeholder={`如：我的${channelKindLabel(formKind)}`}
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">配置 (JSON)</span>
              <Textarea
                value={form.config}
                onChange={(event) => setForm((current) => ({ ...current, config: event.target.value }))}
                placeholder={
                  CHANNEL_TYPES.find((info) => info.type === formKind)?.placeholder
                }
                rows={6}
                className="font-mono text-[length:var(--text-xs)]"
              />
            </label>
            <label className="flex items-center justify-between">
              <span className="text-[length:var(--text-sm)]">启用此渠道</span>
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
        onConfirm={() => void handleSubmit()}
        onCancel={() => {
          setFormOpen(false);
          setEditing(null);
        }}
      />

      {/* 删除确认 */}
      <ConfirmModal
        isOpen={deleteTarget !== null}
        type="danger"
        title="删除通知渠道"
        message={`确认删除渠道「${deleteTarget?.name ?? ""}」？删除后无法恢复。`}
        confirmText="删除"
        onConfirm={() => void handleDelete()}
        onCancel={() => setDeleteTarget(null)}
      />
    </PageScaffold>
  );
}
