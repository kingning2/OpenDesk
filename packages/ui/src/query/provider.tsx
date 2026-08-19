/**
 * QueryProvider — 桌面端 TanStack Query 根。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import * as React from "react";

import { createDeskQueryClient } from "./client";

/**
 * QueryProvider 属性。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
export type QueryProviderProps = {
  /** 应用树。 */
  children: React.ReactNode;
  /** 可选注入，便于测试。 */
  client?: QueryClient;
};

/**
 * TanStack Query 上下文。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 见 {@link QueryProviderProps}
 * @returns Provider 节点
 */
export function QueryProvider({ children, client }: QueryProviderProps) {
  const [fallback] = React.useState(createDeskQueryClient);
  return <QueryClientProvider client={client ?? fallback}>{children}</QueryClientProvider>;
}
