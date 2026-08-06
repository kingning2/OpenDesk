/**
 * AI 建议的动作按钮：把动作工具（`navigate_page` / `open_settings`）的成功结果
 * 渲染成可点击按钮，用户点击后才跳转页面 / 打开设置分区，AI 不会自动执行。
 * Chat 页与帮助页共用。
 *
 * @author coisini
 */

import { Button } from "@desk/ui";
import { useSettingsDialog, type SettingsSectionId } from "@feature/setting";
import { useNavigate } from "react-router";

import { useT } from "../../i18n";
import type { ToolAction, ToolStep } from "./chat-tool-utils";

/**
 * 渲染动作按钮。
 *
 * @author coisini
 *
 * @param props.tools - 一条回复里的工具调用步骤（含 action 的才会渲染）
 */
export function SuggestedActions({ tools }: { tools: ToolStep[] }) {
  const t = useT();
  const navigate = useNavigate();
  const { openSettings } = useSettingsDialog();
  const actions: ToolAction[] = tools.flatMap((tool) => (tool.action ? [tool.action] : []));
  if (actions.length === 0) {
    return null;
  }

  return (
    <div className="mb-2 flex flex-col items-start gap-1.5">
      {actions.map((action, index) =>
        action.kind === "navigate_page" ? (
          <Button
            key={index}
            type="button"
            variant="outline"
            size="sm"
            onClick={() => navigate(action.path ?? "")}
          >
            {t("chat.action.gotoPage", { label: action.label ?? action.path ?? "" })}
          </Button>
        ) : (
          <Button
            key={index}
            type="button"
            variant="outline"
            size="sm"
            onClick={() => openSettings(action.section as SettingsSectionId)}
          >
            {t("chat.action.openSettings", { section: action.section ?? "" })}
          </Button>
        ),
      )}
    </div>
  );
}
