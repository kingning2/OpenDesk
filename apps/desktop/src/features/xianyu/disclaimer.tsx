/**
 * 闲鱼免责声明页（迁移自原前端 `pages/disclaimer/Disclaimer.tsx`，静态适配版）。
 *
 * 原页面从系统设置读取免责声明内容；桌面端以默认文案静态展示（无需 IPC）。
 */

import { AlertTriangle } from "@desk/ui/icons";
import { PageScaffold } from "@desk/ui";

const DISCLAIMER_TITLE = "免责声明";
const DISCLAIMER_CONTENT =
  "数据存储说明\n" +
  "1. 本系统在运行过程中，为保障服务正常运行，会存储用户账号密码、登录 Cookie、商品信息、卡券信息等业务数据。\n" +
  "2. 上述数据仅用于系统功能运行、自动化处理和业务管理，不作为其他用途。\n" +
  "3. 请您自行确认服务器环境、账号权限和数据保管措施的安全性。\n\n" +
  "用户须知\n" +
  "1. 用户应确保使用本系统的行为符合相关平台规则和法律法规。\n" +
  "2. 因用户自身违规操作、账号共享、密码泄露、服务器安全问题导致的损失，由用户自行承担。\n" +
  "3. 建议用户定期备份重要数据，因系统故障、第三方平台变更、不可抗力等导致的异常或损失，本系统不承担责任。\n" +
  "4. 本系统依赖第三方平台接口和网络环境，无法保证服务始终连续、稳定、无中断。\n\n" +
  "隐私与风险提示\n" +
  "1. 请勿在未充分评估风险的情况下接入生产环境或敏感账号。\n" +
  "2. 使用本系统即表示您已充分理解并接受相关风险，并愿意自行承担相应责任。";

/**
 * 闲鱼免责声明页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuDisclaimerPage() {
  return (
    <PageScaffold subtitle="免责声明与使用条款">
      <div className="mx-auto max-w-4xl">
        <div className="mb-6 flex items-center gap-3">
          <AlertTriangle className="size-8 text-amber-600" aria-hidden />
          <h1 className="text-2xl font-semibold">{DISCLAIMER_TITLE}</h1>
        </div>
        <div className="rounded-xl border border-border bg-shell p-6">
          <pre className="whitespace-pre-wrap font-sans text-[length:var(--text-sm)] leading-relaxed text-foreground/90">
            {DISCLAIMER_CONTENT}
          </pre>
        </div>
      </div>
    </PageScaffold>
  );
}
