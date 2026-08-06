/**
 * 聊天工具调用共享类型与解析：Chat 页（数据查询）与 Help 页（系统导航）共用。
 *
 * 动作工具（`navigate_page` / `open_settings`）的返回结果被解析为 `ToolAction`，
 * UI 据此渲染成按钮，用户点击后才跳转 / 打开设置。
 *
 * @author coisini
 */

/** 动作工具（`navigate_page` / `open_settings`）解析出的可执行动作：UI 渲染成按钮，用户点击才执行。 */
export interface ToolAction {
  kind: "navigate_page" | "open_settings";
  /** `navigate_page` 目标路径（如 `/features/mail`）。 */
  path?: string;
  /** `navigate_page` 目标页显示名（如 `Mail`）。 */
  label?: string;
  /** `open_settings` 目标设置分区 id（如 `llm`）。 */
  section?: string;
}

/** 一次工具调用（LLM 查询业务库 / 动作工具时产生）。 */
export interface ToolStep {
  /** 工具名（如 `list_databases` / `run_query` / `navigate_page`）。 */
  name: string;
  /** JSON 编码的工具参数。 */
  arguments: string;
  /** 调用是否成功。 */
  ok: boolean;
  /** 工具返回（JSON 字符串）；失败时为错误描述。 */
  result?: string;
  /** 动作工具解析出的可执行动作；仅 `navigate_page` / `open_settings` 且成功时有值。 */
  action?: ToolAction;
}

/** 解析后端工具结果 JSON 里的某个字符串字段；结果非法或缺字段时返回 undefined。 */
export function parseResultField(result: string | undefined, key: string): string | undefined {
  if (!result) {
    return undefined;
  }
  try {
    const value: unknown = JSON.parse(result);
    return typeof (value as Record<string, unknown>)?.[key] === "string"
      ? ((value as Record<string, string>)[key] as string)
      : undefined;
  } catch {
    return undefined;
  }
}

/** 从一条工具调用记录解析出可执行动作；非动作工具或失败时返回 `undefined`。 */
export function deriveAction(step: ToolStep): ToolAction | undefined {
  if (!step.ok) {
    return undefined;
  }
  if (step.name === "navigate_page") {
    const path = parseResultField(step.result, "path");
    if (!path) {
      return undefined;
    }
    return {
      kind: "navigate_page",
      path,
      label: parseResultField(step.result, "label"),
    };
  }
  if (step.name === "open_settings") {
    const section = parseResultField(step.result, "section");
    if (!section) {
      return undefined;
    }
    return { kind: "open_settings", section };
  }
  return undefined;
}
