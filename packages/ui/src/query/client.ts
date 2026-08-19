/**
 * 桌面端 TanStack Query 默认客户端。
 *
 * 关闭窗口聚焦 refetch：数据来自 Tauri IPC，不是 HTTP。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */

import { keepPreviousData, QueryClient } from "@tanstack/react-query";

/**
 * 创建桌面默认 QueryClient。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @returns 配置好的 QueryClient
 */
export function createDeskQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: 1,
        staleTime: 30_000,
        refetchOnWindowFocus: false,
        placeholderData: keepPreviousData,
      },
    },
  });
}
