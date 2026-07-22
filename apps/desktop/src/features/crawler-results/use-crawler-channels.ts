/**
 * Persisted crawler channel list hook — search, filters, pagination.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

import { useCallback, useEffect, useMemo, useState } from "react";
import {
  crawlerChannelList,
  crawlerChannelUpdate,
  type CrawlerChannelListRow,
} from "@desk/platform/ipc/crawler";

export const PAGE_SIZE = 50;

export const EMAIL_STATUS_OPTIONS = [
  "found_api",
  "pending_enrich",
  "enriching",
  "found_playwright",
  "not_found",
  "enrich_failed",
  "verified_manual",
] as const;

export type EmailStatusFilter = (typeof EMAIL_STATUS_OPTIONS)[number] | "all";
export type HasEmailFilter = "all" | "yes" | "no";

export interface CrawlerChannelFilters {
  search: string;
  keyword: string;
  country: string;
  hasEmail: HasEmailFilter;
  emailStatus: EmailStatusFilter;
}

const defaultFilters = (): CrawlerChannelFilters => ({
  search: "",
  keyword: "",
  country: "",
  hasEmail: "all",
  emailStatus: "all",
});

/**
 * Load and filter persisted crawler channels from SQLite.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export function useCrawlerChannels() {
  const [items, setItems] = useState<CrawlerChannelListRow[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(0);
  const [filters, setFilters] = useState<CrawlerChannelFilters>(defaultFilters);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [savingIds, setSavingIds] = useState<Set<number>>(() => new Set());

  const totalPages = useMemo(
    () => Math.max(1, Math.ceil(total / PAGE_SIZE)),
    [total],
  );

  const refresh = useCallback(async (nextPage = page, nextFilters = filters) => {
    setLoading(true);
    setError(null);
    try {
      const result = await crawlerChannelList({
        search: nextFilters.search.trim() || undefined,
        keyword: nextFilters.keyword.trim() || undefined,
        country: nextFilters.country.trim() || undefined,
        has_email:
          nextFilters.hasEmail === "all"
            ? undefined
            : nextFilters.hasEmail === "yes",
        email_status:
          nextFilters.emailStatus === "all" ? undefined : nextFilters.emailStatus,
        limit: PAGE_SIZE,
        offset: nextPage * PAGE_SIZE,
      });
      setItems(result.items);
      setTotal(result.total);
    } catch (cause) {
      setError(String(cause));
    } finally {
      setLoading(false);
    }
  }, [filters, page]);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void refresh(page, filters);
    }, 0);
    return () => {
      window.clearTimeout(timer);
    };
  }, [page, filters, refresh]);

  function updateFilters(patch: Partial<CrawlerChannelFilters>) {
    setPage(0);
    setFilters((prev) => ({ ...prev, ...patch }));
  }

  function resetFilters() {
    setPage(0);
    setFilters(defaultFilters());
  }

  const saveVerifiedEmail = useCallback(
    async (row: CrawlerChannelListRow, verifiedEmail: string) => {
      setSavingIds((prev) => new Set(prev).add(row.id));
      setError(null);
      try {
        const response = await crawlerChannelUpdate({
          id: row.id,
          verified_email: verifiedEmail.trim(),
        });
        setItems((prev) =>
          prev.map((item) =>
            item.id === row.id
              ? {
                  ...item,
                  verified_email: response.verified_email,
                  email_status: response.email_status ?? item.email_status,
                }
              : item,
          ),
        );
      } catch (cause) {
        setError(String(cause));
      } finally {
        setSavingIds((prev) => {
          const next = new Set(prev);
          next.delete(row.id);
          return next;
        });
      }
    },
    [],
  );

  return {
    items,
    total,
    page,
    setPage,
    totalPages,
    filters,
    updateFilters,
    resetFilters,
    loading,
    error,
    refresh,
    savingIds,
    saveVerifiedEmail,
  };
}
