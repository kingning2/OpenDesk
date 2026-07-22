/**
 * Crawler results list page — persisted channel table with filters.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

import type { ReactNode } from "react";
import { useMemo } from "react";
import {
  Button,
  DataTable,
  Input,
  PageScaffold,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  cn,
} from "@desk/ui";
import { useT } from "../../i18n";
import { createCrawlerResultsColumns } from "./crawler-results-columns";
import {
  EMAIL_STATUS_OPTIONS,
  PAGE_SIZE,
  useCrawlerChannels,
  type CrawlerChannelFilters,
  type HasEmailFilter,
} from "./use-crawler-channels";

/**
 * Persisted crawler channel results table.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export function CrawlerResultsPage() {
  const t = useT();
  const {
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
  } = useCrawlerChannels();

  const columns = useMemo(
    () =>
      createCrawlerResultsColumns({
        t,
        savingIds,
        onSaveVerifiedEmail: saveVerifiedEmail,
      }),
    [t, savingIds, saveVerifiedEmail],
  );
  const rangeStart = total === 0 ? 0 : page * PAGE_SIZE + 1;
  const rangeEnd = Math.min(total, (page + 1) * PAGE_SIZE);

  return (
    <PageScaffold fill containerPadding="none" className="flex min-h-0 flex-col">
      <div className="shrink-0 space-y-3 border-b border-border/70 px-4 py-3">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div>
            <h1 className="text-[length:var(--text-base)] font-semibold">
              {t("crawler-results.title")}
            </h1>
            <p className="text-[length:var(--text-sm)] text-muted-foreground">
              {t("crawler-results.subtitle")}
            </p>
          </div>
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={loading}
            onClick={() => void refresh()}
          >
            {t("crawler-results.refresh")}
          </Button>
        </div>

        <div className="grid gap-2 md:grid-cols-2 xl:grid-cols-5">
          <FilterField label={t("crawler-results.filters.search")}>
            <Input
              value={filters.search}
              placeholder={t("crawler-results.filters.searchPlaceholder")}
              onChange={(event) => updateFilters({ search: event.target.value })}
            />
          </FilterField>
          <FilterField label={t("crawler-results.filters.keyword")}>
            <Input
              value={filters.keyword}
              placeholder={t("crawler-results.filters.keywordPlaceholder")}
              onChange={(event) => updateFilters({ keyword: event.target.value })}
            />
          </FilterField>
          <FilterField label={t("crawler-results.filters.country")}>
            <Input
              value={filters.country}
              placeholder={t("crawler-results.filters.countryPlaceholder")}
              onChange={(event) => updateFilters({ country: event.target.value.toUpperCase() })}
            />
          </FilterField>
          <FilterField label={t("crawler-results.filters.hasEmail")}>
            <Select
              value={filters.hasEmail}
              onValueChange={(value) => updateFilters({ hasEmail: value as HasEmailFilter })}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">{t("crawler-results.filters.all")}</SelectItem>
                <SelectItem value="yes">{t("crawler-results.filters.hasEmailYes")}</SelectItem>
                <SelectItem value="no">{t("crawler-results.filters.hasEmailNo")}</SelectItem>
              </SelectContent>
            </Select>
          </FilterField>
          <FilterField label={t("crawler-results.filters.emailStatus")}>
            <Select
              value={filters.emailStatus}
              onValueChange={(value) =>
                updateFilters({
                  emailStatus: value as CrawlerChannelFilters["emailStatus"],
                })
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">{t("crawler-results.filters.all")}</SelectItem>
                {EMAIL_STATUS_OPTIONS.map((status) => (
                  <SelectItem key={status} value={status}>
                    {t(`crawler-results.emailStatus.${status}`)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </FilterField>
        </div>

        <div className="flex flex-wrap items-center justify-between gap-2">
          <p className="text-[length:var(--text-xs)] text-muted-foreground">
            {t("crawler-results.total", { count: total.toLocaleString() })}
          </p>
          <Button type="button" variant="ghost" size="sm" onClick={resetFilters}>
            {t("crawler-results.resetFilters")}
          </Button>
        </div>
      </div>

      {error ? (
        <p className="shrink-0 border-b border-border/50 px-4 py-3 text-[length:var(--text-sm)] text-destructive">
          {error}
        </p>
      ) : null}

      <div className="flex min-h-0 flex-1 flex-col overflow-hidden px-4 py-2">
        <DataTable
          className="min-h-0 flex-1"
          columns={columns}
          data={items}
          loading={loading}
          emptyMessage={t("crawler-results.empty")}
        />
      </div>

      <div className="flex shrink-0 items-center justify-between gap-3 border-t border-border/70 px-4 py-3">
        <p className="text-[length:var(--text-xs)] text-muted-foreground">
          {t("crawler-results.range", {
            start: rangeStart.toLocaleString(),
            end: rangeEnd.toLocaleString(),
            total: total.toLocaleString(),
          })}
        </p>
        <div className="flex items-center gap-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={loading || page <= 0}
            onClick={() => setPage((value) => Math.max(0, value - 1))}
          >
            {t("crawler-results.prevPage")}
          </Button>
          <span className="text-[length:var(--text-xs)] text-muted-foreground">
            {t("crawler-results.page", { current: page + 1, total: totalPages })}
          </span>
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={loading || page + 1 >= totalPages}
            onClick={() => setPage((value) => value + 1)}
          >
            {t("crawler-results.nextPage")}
          </Button>
        </div>
      </div>
    </PageScaffold>
  );
}

function FilterField({
  label,
  children,
  className,
}: {
  label: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <label className={cn("block space-y-1.5", className)}>
      <span className="text-[length:var(--text-xs)] text-muted-foreground">{label}</span>
      {children}
    </label>
  );
}
