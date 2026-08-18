/**
 * Platform IPC 封装 — 平台能力清单（前端动态路由数据源）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/**
 * 平台描述（IPC 响应 — 与 Rust `commands/platform.rs` DTO 对齐）。
 *
 * @author agent
 * @created 2026-08-13
 */
export interface PlatformDescriptorDto {
  /** 平台标识（小写，如 `xianyu` / `xiaohongshu`）。 */
  kind: string;
  /** 展示名称（如「闲鱼」）。 */
  name: string;
  /** 能力清单（小写 snake_case，如 `chat` / `coupon`）。 */
  capabilities: string[];
}

/**
 * 查询全部平台及其能力清单。
 *
 * 前端据此动态渲染导航路由：例如闲鱼有 `coupon`（优惠券）板块、
 * 小红书没有 → 切换到小红书时不渲染该路由。
 *
 * @author agent
 * @created 2026-08-13
 *
 * @returns 平台描述列表
 */
export function platformDescriptors(): Promise<PlatformDescriptorDto[]> {
  return call<PlatformDescriptorDto[]>("platform_descriptors");
}

/**
 * 取指定平台的能力清单（空串表示"全平台通用"）。
 *
 * @author agent
 * @created 2026-08-13
 *
 * @param kind - 平台标识
 * @param descriptors - 平台描述列表（可省略，将实时查询）
 * @returns 能力字符串集合
 */
export async function capabilitiesFor(
  kind: string,
  descriptors?: PlatformDescriptorDto[],
): Promise<Set<string>> {
  const list = descriptors ?? (await platformDescriptors());
  const descriptor = list.find((item) => item.kind === kind);
  return new Set(descriptor?.capabilities ?? []);
}
