/**
 * 渠道连接状态 — 与 Rust `channel/status` 事件 `state` 字段对齐的 UI 映射表。
 *
 * 后端只推送本表中的 canonical `state`，`detail` 为可选短中文（禁止原始 JSON）。
 * 前端用本 map 渲染，勿再靠 detail 关键词猜状态。
 *
 * @author Xiaoman
 * @created 2026-08-21
 */

/** 与后端约定的连接状态键。 */
export const CHANNEL_CONNECTION_STATES = [
  "disconnected",
  "connecting",
  "connected",
  "error",
  "auth_expired",
  "renewing",
  "queued",
] as const;

/** 渠道连接状态键。 */
export type ChannelConnectionState = (typeof CHANNEL_CONNECTION_STATES)[number];

/**
 * 单条状态的展示配置。
 *
 * @author Xiaoman
 * @created 2026-08-21
 */
export type ChannelConnectionStatusView = {
  /** 徽章文案。 */
  label: string;
  /** Tailwind 徽章样式。 */
  badgeClass: string;
  /** 无 detail 时的默认提示；`null` 表示不展示副文案。 */
  defaultHint: string | null;
};

/**
 * 状态 → UI 映射（唯一展示源）。
 *
 * @author Xiaoman
 * @created 2026-08-21
 */
export const CHANNEL_CONNECTION_STATUS_MAP: Record<
  ChannelConnectionState,
  ChannelConnectionStatusView
> = {
  disconnected: {
    label: "未连接",
    badgeClass: "bg-muted text-muted-foreground",
    defaultHint: null,
  },
  connecting: {
    label: "连接中",
    badgeClass: "bg-amber-500/15 text-amber-600",
    defaultHint: "正在连接闲鱼…",
  },
  connected: {
    label: "已连接",
    badgeClass: "bg-emerald-500/15 text-emerald-600",
    defaultHint: null,
  },
  error: {
    label: "异常",
    badgeClass: "bg-red-500/15 text-red-600",
    defaultHint: "连接异常，请稍后重试或查看运行日志",
  },
  auth_expired: {
    label: "登录过期",
    badgeClass: "bg-orange-500/15 text-orange-700",
    defaultHint: "登录态已过期，请重新扫码后再连接",
  },
  renewing: {
    label: "过滑块中",
    badgeClass: "bg-sky-500/15 text-sky-700",
    defaultHint: "正在过滑块验证，请稍候",
  },
  queued: {
    label: "排队中",
    badgeClass: "bg-violet-500/15 text-violet-700",
    defaultHint: "排队等待过滑块，请稍候",
  },
};

/**
 * 将事件 `state` 规范为已知键；未知值回退 `disconnected`。
 *
 * @author Xiaoman
 * @created 2026-08-21
 *
 * @param state - `channel/status` 的 state 字段
 * @returns canonical 状态键
 */
export function normalizeChannelConnectionState(
  state: string | null | undefined,
): ChannelConnectionState {
  const value = (state ?? "").trim();
  if ((CHANNEL_CONNECTION_STATES as readonly string[]).includes(value)) {
    return value as ChannelConnectionState;
  }
  return "disconnected";
}

/**
 * 合并协议噪声事件：续期/排队/过期过程中的 disconnected 不得冲掉合成态。
 *
 * @author Xiaoman
 * @created 2026-08-21
 *
 * @param previous - 当前 UI 状态
 * @param incoming - 事件带来的 state
 * @returns 应用后的 UI 状态
 */
export function mergeChannelConnectionState(
  previous: ChannelConnectionState | undefined,
  incoming: string,
): ChannelConnectionState {
  const next = normalizeChannelConnectionState(incoming);
  if (
    next === "connected" ||
    next === "auth_expired" ||
    next === "renewing" ||
    next === "queued" ||
    next === "error"
  ) {
    return next;
  }
  if (
    (previous === "renewing" || previous === "queued") &&
    (next === "disconnected" || next === "connecting")
  ) {
    return previous;
  }
  if (previous === "auth_expired" && next === "disconnected") {
    return "auth_expired";
  }
  return next;
}

/**
 * 取展示文案：优先用后端短 detail，否则用 map 默认 hint。
 *
 * @author Xiaoman
 * @created 2026-08-21
 *
 * @param state - canonical 状态
 * @param detail - 事件 detail（已应为短中文）
 * @returns 副文案；无则 null
 */
export function connectionStatusHint(
  state: ChannelConnectionState,
  detail?: string | null,
): string | null {
  const trimmed = detail?.trim() ?? "";
  // 防御：绝不把原始 punish/token JSON 渲到 UI。
  if (
    trimmed &&
    !trimmed.startsWith("{") &&
    !trimmed.includes("FAIL_SYS_") &&
    !trimmed.includes("_____tmd_____") &&
    trimmed.length < 120
  ) {
    return trimmed;
  }
  return CHANNEL_CONNECTION_STATUS_MAP[state].defaultHint;
}
