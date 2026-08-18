/**
 * 闲鱼个人设置页（迁移自原前端 `pages/personalSettings/PersonalSettings.tsx`）。
 *
 * 按原前端核心交互重写：重发货触发关键字 + 联系方式 + 对接卡密秘钥（纯业务配置项）。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/setting`），复用 crates/app UserSettingService。
 *
 * 说明：原页面的余额 / 充值 / 提现 / 结算 / 修改密码 / 分销对接码等依赖 SaaS 服务端
 * （页面本身注明需在 agent.zhinianboke.com 操作），桌面单用户场景不迁移。
 */

import { useEffect, useState } from "react";
import {
  Button,
  Input,
  Loading,
  PageScaffold,
  toast,
} from "@desk/ui";
import { Eye, EyeOff, Key, Package, RefreshCw, Save, User } from "@desk/ui/icons";
import { userSettingsGetAll, userSettingSet, type PersonalSettings } from "@desk/platform/ipc/setting";

/** 表单状态（与聚合视图字段对齐）。 */
interface FormState {
  redeliveryKeyword: string;
  contactWechat: string;
  contactQq: string;
  cardSecretKey: string;
}

const EMPTY_FORM: FormState = {
  redeliveryKeyword: "",
  contactWechat: "",
  contactQq: "",
  cardSecretKey: "",
};

function toForm(settings: PersonalSettings): FormState {
  return {
    redeliveryKeyword: settings.redelivery_keyword,
    contactWechat: settings.contact_wechat,
    contactQq: settings.contact_qq,
    cardSecretKey: settings.card_secret_key,
  };
}

/**
 * 闲鱼个人设置页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuPersonalSettingsPage() {
  const [loading, setLoading] = useState(true);
  const [form, setForm] = useState<FormState>(EMPTY_FORM);
  const [savingKeyword, setSavingKeyword] = useState(false);
  const [savingContact, setSavingContact] = useState(false);
  const [savingCardKey, setSavingCardKey] = useState(false);
  const [showCardKey, setShowCardKey] = useState(false);

  async function load() {
    try {
      const settings = await userSettingsGetAll();
      setForm(toForm(settings));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    let cancelled = false;
    void userSettingsGetAll()
      .then((settings) => {
        if (!cancelled) setForm(toForm(settings));
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

  const patchForm = (patch: Partial<FormState>) =>
    setForm((current) => ({ ...current, ...patch }));

  async function handleSaveKeyword() {
    setSavingKeyword(true);
    try {
      await userSettingSet("personal.redelivery_keyword", form.redeliveryKeyword);
      toast.success("重发货触发关键字保存成功");
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSavingKeyword(false);
    }
  }

  async function handleSaveContact() {
    setSavingContact(true);
    try {
      await userSettingSet("personal.contact_wechat", form.contactWechat);
      await userSettingSet("personal.contact_qq", form.contactQq);
      toast.success("联系方式保存成功");
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSavingContact(false);
    }
  }

  async function handleSaveCardKey() {
    setSavingCardKey(true);
    try {
      await userSettingSet("personal.card_secret_key", form.cardSecretKey);
      toast.success("对接卡密秘钥已保存");
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSavingCardKey(false);
    }
  }

  return (
    <PageScaffold subtitle="闲鱼个人设置 — 账户业务偏好配置">
      <div className="space-y-4">
        {/* 页头 */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="font-semibold">个人设置</h1>
            <p className="text-[length:var(--text-sm)] text-muted-foreground">
              管理个人账户信息和偏好设置
            </p>
          </div>
          <Button variant="outline" size="sm" onClick={() => void load()}>
            <RefreshCw className="size-4" aria-hidden />
            刷新
          </Button>
        </div>

        {loading ? (
          <Loading size="lg" text="加载中..." className="py-16" />
        ) : (
          <>
            {/* 账户信息（桌面单用户：固定展示） */}
            <div className="rounded-xl border border-border bg-shell p-4">
              <h2 className="mb-3 flex items-center gap-2 font-medium">
                <User className="size-4" aria-hidden />
                账户信息
              </h2>
              <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
                <div className="space-y-1">
                  <span className="text-[length:var(--text-xs)] text-muted-foreground">当前用户</span>
                  <Input value="本地桌面用户" disabled />
                </div>
                <div className="space-y-1">
                  <span className="text-[length:var(--text-xs)] text-muted-foreground">授权方式</span>
                  <Input value="License 激活（见授权面板）" disabled />
                </div>
              </div>
            </div>

            {/* 重发货触发关键字 */}
            <div className="rounded-xl border border-border bg-shell p-4">
              <h2 className="mb-3 flex items-center gap-2 font-medium">
                <Package className="size-4" aria-hidden />
                重发货触发关键字
              </h2>
              <p className="mb-3 text-[length:var(--text-xs)] leading-relaxed text-muted-foreground">
                设置后，在闲鱼聊天中自己发送「关键字+订单号」即可触发自动重新发货。例如关键字为「重新触发」，发送「4502144774044041438重新触发」将提取订单号并自动发货。
                <br />
                <span className="text-amber-600">
                  注意：关键字不包含前后空格；如果订单不在数据库中，系统会自动根据订单号获取订单信息后再发货。
                </span>
              </p>
              <div className="space-y-1">
                <span className="text-[length:var(--text-xs)] text-muted-foreground">触发关键字</span>
                <Input
                  value={form.redeliveryKeyword}
                  onChange={(event) => patchForm({ redeliveryKeyword: event.target.value })}
                  placeholder="例如：重新触发"
                  className="max-w-md"
                />
                <p className="text-[length:var(--text-xs)] text-muted-foreground">
                  保存时会自动去除前后空格；留空则关闭此功能
                </p>
              </div>
              <Button className="mt-3" onClick={() => void handleSaveKeyword()} disabled={savingKeyword}>
                <Save className="size-4" aria-hidden />
                {savingKeyword ? "保存中…" : "保存"}
              </Button>
            </div>

            {/* 联系方式 */}
            <div className="rounded-xl border border-border bg-shell p-4">
              <h2 className="mb-3 flex items-center gap-2 font-medium">
                <User className="size-4" aria-hidden />
                联系方式
              </h2>
              <p className="mb-3 text-[length:var(--text-xs)] text-muted-foreground">
                设置您的微信和QQ，方便分销商联系您
              </p>
              <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
                <div className="space-y-1">
                  <span className="text-[length:var(--text-xs)] text-muted-foreground">微信</span>
                  <Input
                    value={form.contactWechat}
                    onChange={(event) => patchForm({ contactWechat: event.target.value })}
                    placeholder="请输入微信号"
                  />
                </div>
                <div className="space-y-1">
                  <span className="text-[length:var(--text-xs)] text-muted-foreground">QQ</span>
                  <Input
                    value={form.contactQq}
                    onChange={(event) => patchForm({ contactQq: event.target.value })}
                    placeholder="请输入QQ号"
                  />
                </div>
              </div>
              <Button className="mt-3" onClick={() => void handleSaveContact()} disabled={savingContact}>
                <Save className="size-4" aria-hidden />
                {savingContact ? "保存中…" : "保存联系方式"}
              </Button>
            </div>

            {/* 对接卡密秘钥 */}
            <div className="rounded-xl border border-border bg-shell p-4">
              <h2 className="mb-3 flex items-center gap-2 font-medium">
                <Key className="size-4" aria-hidden />
                对接卡密秘钥
              </h2>
              <p className="mb-3 text-[length:var(--text-xs)] leading-relaxed text-muted-foreground">
                用于「分销卡券」页面对接上游卡券系统的鉴权秘钥，请妥善保管。修改后点击「保存」生效。
              </p>
              <div className="relative max-w-md">
                <Input
                  type={showCardKey ? "text" : "password"}
                  value={form.cardSecretKey}
                  onChange={(event) => patchForm({ cardSecretKey: event.target.value })}
                  placeholder="请输入对接卡密秘钥"
                  className="pr-10"
                />
                <button
                  type="button"
                  onClick={() => setShowCardKey((value) => !value)}
                  className="absolute right-2 top-1/2 -translate-y-1/2 p-1.5 text-muted-foreground transition-colors hover:text-foreground"
                  title={showCardKey ? "隐藏" : "显示"}
                >
                  {showCardKey ? <EyeOff className="size-4" aria-hidden /> : <Eye className="size-4" aria-hidden />}
                </button>
              </div>
              <Button className="mt-3" onClick={() => void handleSaveCardKey()} disabled={savingCardKey}>
                <Save className="size-4" aria-hidden />
                {savingCardKey ? "保存中…" : "保存"}
              </Button>
            </div>
          </>
        )}
      </div>
    </PageScaffold>
  );
}
