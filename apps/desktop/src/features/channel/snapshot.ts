/**
 * 浏览器快照工具 — 解析 Chrome 扩展导出的闲鱼登录快照。
 *
 * 快照结构（ai-goofish-monitor Chrome 扩展导出）：
 * { capturedAt, pageUrl, page, env:{navigator,screen,intl}, storage:{local,session},
 *   meta, headers, cookies:[{name,value,domain,path,expires,httpOnly,secure,sameSite}] }
 */

import type { ChannelCookie } from "@desk/contracts";

/** 解析结果。 */
export interface SnapshotParseResult {
  ok: boolean;
  /** 解析出的 cookies（仅名称+值）。 */
  cookies: ChannelCookie[];
  /** 人类可读说明。 */
  detail: string;
}

/** 从快照提取关键 cookie 的显示名。 */
function keyCookieNames(cookies: ChannelCookie[]): string[] {
  const names = ["unb", "cookie2", "_m_h5_tk", "cna", "sgcookie"];
  return names.filter((name) => cookies.some((cookie) => cookie.name === name));
}

/**
 * 解析凭据：接受快照 JSON / cookies 数组 / 旧 cookie 字符串。
 * 返回解析结果与可展示信息。
 */
export function parseSnapshot(credential: string): SnapshotParseResult {
  const text = credential.trim();
  if (!text) {
    return { ok: false, cookies: [], detail: "凭据为空" };
  }

  // 尝试 JSON 解析。
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch {
    // 不是 JSON —— 可能是旧 cookie 字符串，允许但不推荐。
    if (text.includes("=")) {
      return {
        ok: true,
        cookies: [],
        detail: "检测到旧 Cookie 字符串（兼容模式）。建议改用 Chrome 扩展导出的快照 JSON。",
      };
    }
    return { ok: false, cookies: [], detail: "不是有效的 JSON 或 Cookie 字符串" };
  }

  // 快照 JSON（含 cookies 数组）。
  if (typeof parsed === "object" && parsed !== null && !Array.isArray(parsed)) {
    const obj = parsed as Record<string, unknown>;
    const cookiesArr = obj.cookies;
    if (Array.isArray(cookiesArr)) {
      const cookies = cookiesArr.filter(isChannelCookie);
      if (cookies.length === 0) {
        return { ok: false, cookies: [], detail: "快照中的 cookies 为空" };
      }
      const keys = keyCookieNames(cookies);
      const envKeys = Object.keys(obj.env ?? {}).length;
      return {
        ok: true,
        cookies,
        detail: `快照有效：${cookies.length} 个 cookie（${keys.join("、")}），env 字段 ${envKeys} 项。`,
      };
    }
    return { ok: false, cookies: [], detail: "快照缺少 cookies 字段" };
  }

  // cookies 数组 JSON。
  if (Array.isArray(parsed)) {
    const cookies = parsed.filter(isChannelCookie);
    if (cookies.length === 0) {
      return { ok: false, cookies: [], detail: "cookies 数组为空" };
    }
    const keys = keyCookieNames(cookies);
    return {
      ok: true,
      cookies,
      detail: `cookies 数组有效：${cookies.length} 个（${keys.join("、")}）。`,
    };
  }

  return { ok: false, cookies: [], detail: "无法识别的凭据格式" };
}

function isChannelCookie(value: unknown): value is ChannelCookie {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const cookie = value as Record<string, unknown>;
  return (
    typeof cookie.name === "string" &&
    typeof cookie.value === "string" &&
    typeof cookie.domain === "string"
  );
}
