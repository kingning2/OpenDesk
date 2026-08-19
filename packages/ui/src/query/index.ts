/**
 * TanStack Query 出口 — Feature 从 `@desk/ui` 取 hook，不要直连 `@tanstack/react-query`。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */

export { QueryProvider } from "./provider";
export type { QueryProviderProps } from "./provider";
export { createDeskQueryClient } from "./client";
export { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
export type { QueryClient, QueryKey, UseMutationResult, UseQueryResult } from "@tanstack/react-query";
