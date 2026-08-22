/**
 * 会话时间格式化 — 兼容毫秒字符串（created_at）与 ISO 时间（下单时间）。
 * 统一输出 `MM-DD HH:mm`；非法输入返回空串。
 */
export function formatChatTime(value?: string | number | null): string {
  if (value == null) {
    return "";
  }
  let ms: number;
  if (typeof value === "number") {
    ms = value;
  } else if (/^\d+$/.test(value)) {
    ms = Number(value);
  } else {
    ms = new Date(value).getTime();
  }
  if (!Number.isFinite(ms) || ms <= 0) {
    return "";
  }
  return new Date(ms).toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}
